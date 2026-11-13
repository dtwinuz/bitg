// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! HTTPS so'rovlar Windows'ning o'z WinHTTP'si bilan: tizim sertifikatlari va proxy sozlamasi ishlatiladi,
//! qo'shimcha TLS kutubxonasi yo'q. Faqat `https://`; HTTPS → HTTP yo'naltirish taqiqlangan (WinHTTP standarti).

use windows::Win32::Networking::WinHttp::{
    INTERNET_DEFAULT_HTTPS_PORT, WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_FLAG_SECURE,
    WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2, WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_3,
    WINHTTP_OPTION_SECURE_PROTOCOLS, WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_STATUS_CODE,
    WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest, WinHttpQueryHeaders,
    WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest, WinHttpSetOption,
    WinHttpSetTimeouts,
};
use windows::core::{HSTRING, PCWSTR};

/// WinHTTP dastagi: tushib qolganda yopiladi.
struct H(*mut core::ffi::c_void);

impl Drop for H {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: dasta WinHttpOpen/Connect/OpenRequest dan, bir marta yopiladi.
            let _ = unsafe { WinHttpCloseHandle(self.0) };
        }
    }
}

fn split(url: &str) -> Result<(&str, &str), String> {
    let rest = url.strip_prefix("https://").ok_or("faqat https manzil")?;
    let (host, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    if host.is_empty() || host.contains([':', '@']) {
        return Err("manzil noto'g'ri".into());
    }
    Ok((host, if path.is_empty() { "/" } else { path }))
}

/// So'rov: (holat kodi, javob tanasi). `limit` dan katta javob — xato.
pub fn request(
    method: &str,
    url: &str,
    body: Option<(&str, &[u8])>,
    limit: usize,
    timeout_ms: i32,
) -> Result<(u32, Vec<u8>), String> {
    let (host, path) = split(url)?;
    let agent = HSTRING::from(format!("bitg/{} (Windows)", env!("CARGO_PKG_VERSION")));
    // SAFETY: WinHTTP C API; har dasta `H` ichida yopiladi, buferlar chaqiruv davomida yashaydi.
    unsafe {
        let session = H(WinHttpOpen(
            &agent,
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            0,
        ));
        if session.0.is_null() {
            return Err("tarmoq ochilmadi".into());
        }
        let protocols = WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2 | WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_3;
        let bytes = protocols.to_ne_bytes();
        if WinHttpSetOption(
            Some(session.0),
            WINHTTP_OPTION_SECURE_PROTOCOLS,
            Some(&bytes),
        )
        .is_err()
        {
            // Eski Windows 10 TLS 1.3 ni bilmaydi.
            let bytes = WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2.to_ne_bytes();
            let _ = WinHttpSetOption(
                Some(session.0),
                WINHTTP_OPTION_SECURE_PROTOCOLS,
                Some(&bytes),
            );
        }
        let _ = WinHttpSetTimeouts(session.0, timeout_ms, timeout_ms, timeout_ms, timeout_ms);
        let conn = H(WinHttpConnect(
            session.0,
            &HSTRING::from(host),
            INTERNET_DEFAULT_HTTPS_PORT,
            0,
        ));
        if conn.0.is_null() {
            return Err("serverga ulanib bo'lmadi".into());
        }
        let req = H(WinHttpOpenRequest(
            conn.0,
            &HSTRING::from(method),
            &HSTRING::from(path),
            PCWSTR::null(),
            PCWSTR::null(),
            std::ptr::null(),
            WINHTTP_FLAG_SECURE,
        ));
        if req.0.is_null() {
            return Err("so'rov ochilmadi".into());
        }
        let headers: Vec<u16>;
        let (hdr, data, len) = match body {
            Some((ctype, data)) => {
                headers = format!("Content-Type: {ctype}\r\n")
                    .encode_utf16()
                    .collect();
                (
                    Some(headers.as_slice()),
                    Some(data.as_ptr() as *const core::ffi::c_void),
                    data.len() as u32,
                )
            }
            None => (None, None, 0),
        };
        WinHttpSendRequest(req.0, hdr, data, len, len, 0)
            .map_err(|_| "internetga ulanib bo'lmadi".to_string())?;
        WinHttpReceiveResponse(req.0, std::ptr::null_mut())
            .map_err(|_| "server javob bermadi".to_string())?;
        let mut status = 0u32;
        let mut size = 4u32;
        let _ = WinHttpQueryHeaders(
            req.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            PCWSTR::null(),
            Some(&mut status as *mut u32 as *mut core::ffi::c_void),
            &mut size,
            std::ptr::null_mut(),
        );
        let mut out = Vec::new();
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let mut read = 0u32;
            WinHttpReadData(
                req.0,
                buf.as_mut_ptr() as *mut core::ffi::c_void,
                buf.len() as u32,
                &mut read,
            )
            .map_err(|_| "javob uzildi".to_string())?;
            if read == 0 {
                break;
            }
            out.extend_from_slice(&buf[..read as usize]);
            if out.len() > limit {
                return Err("javob juda katta".into());
            }
        }
        Ok((status, out))
    }
}

#[cfg(test)]
mod tests {
    use super::split;

    #[test]
    fn only_https_hosts() {
        assert_eq!(split("https://bitg.uz/a?b").unwrap(), ("bitg.uz", "/a?b"));
        assert_eq!(split("https://bitg.uz").unwrap(), ("bitg.uz", "/"));
        assert!(split("http://bitg.uz/").is_err());
        assert!(split("https://u@x/").is_err());
        assert!(split("https://x:1/").is_err());
        assert!(split("https:///").is_err());
    }

    /// Haqiqiy tarmoq: `cargo test -p bitg-settings -- --ignored live_`.
    #[test]
    #[ignore]
    fn live_get_and_redirect() {
        let (code, body) =
            super::request("GET", "https://bitg.uz/version.txt", None, 4096, 15_000).unwrap();
        assert_eq!(code, 200, "{}", String::from_utf8_lossy(&body));
        // GitHub reliz fayli boshqa domenga yo'naltiradi — WinHTTP o'zi kuzatadi.
        let url = "https://github.com/dtwinuz/bitg/releases/download/v0.3.1/bitg-0.3.1-x64.msi";
        let (code, msi) = super::request("GET", url, None, 80 << 20, 60_000).unwrap();
        assert_eq!(code, 200);
        assert!(msi.len() > 1 << 20);
        // Chegara: javob `limit` dan katta bo'lsa — xato.
        assert!(super::request("GET", "https://bitg.uz/", None, 10, 15_000).is_err());
    }
}
