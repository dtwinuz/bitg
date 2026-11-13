// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! «Xabar berish» uchun Windows yordamchilari: ekran parchasi (Windows'ning o'z «Snipping» vositasi —
//! foydalanuvchi faqat kerakli joyni tanlaydi), Windows yig'ilish raqami, o'rnatish identifikatori.

use std::time::{Duration, Instant};

use windows::Win32::Foundation::HGLOBAL;
use windows::Win32::System::DataExchange::{
    CloseClipboard, GetClipboardData, GetClipboardSequenceNumber, IsClipboardFormatAvailable,
    OpenClipboard,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW};
use windows::core::HSTRING;

const CF_DIB: u32 = 8;
/// Bufer rasmi chegarasi (4K ekran 32 bit ≈ 33 MB).
const MAX_DIB: usize = 64 * 1024 * 1024;

/// Buferdagi rasm `.bmp` ko'rinishida.
fn clipboard_bmp() -> Option<Vec<u8>> {
    // SAFETY: bufer ochiq paytda dasta qulflanib nusxa olinadi, keyin qulf va bufer yopiladi.
    unsafe {
        IsClipboardFormatAvailable(CF_DIB).ok()?;
        OpenClipboard(None).ok()?;
        let dib = (|| {
            let h = GetClipboardData(CF_DIB).ok()?;
            let g = HGLOBAL(h.0);
            let size = GlobalSize(g);
            if size == 0 || size > MAX_DIB {
                return None;
            }
            let p = GlobalLock(g) as *const u8;
            if p.is_null() {
                return None;
            }
            let v = std::slice::from_raw_parts(p, size).to_vec();
            let _ = GlobalUnlock(g);
            Some(v)
        })();
        let _ = CloseClipboard();
        crate::feedback::dib_to_bmp(&dib?)
    }
}

/// Ekran parchasi vositasini ochadi va bufer yangi rasm olguncha kutadi (ko'pi `wait`).
pub fn snip(wait: Duration) -> Option<Vec<u8>> {
    // SAFETY: argumentsiz o'qish.
    let before = unsafe { GetClipboardSequenceNumber() };
    std::process::Command::new("explorer")
        .arg("ms-screenclip:")
        .spawn()
        .ok()?;
    let start = Instant::now();
    while start.elapsed() < wait {
        std::thread::sleep(Duration::from_millis(250));
        // SAFETY: argumentsiz o'qish.
        if unsafe { GetClipboardSequenceNumber() } != before {
            // Vosita bir nechta format yozadi — hammasi tushishini kutamiz.
            std::thread::sleep(Duration::from_millis(300));
            return clipboard_bmp();
        }
    }
    None
}

/// Windows yig'ilish raqami (`26100`), diagnostika uchun.
pub fn os_build() -> String {
    let mut buf = [0u16; 16];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: bufer hajmi baytlarda beriladi; qaytgan satr nol bilan tugaydi.
    let ok = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            &HSTRING::from(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion"),
            &HSTRING::from("CurrentBuild"),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut size),
        )
    }
    .is_ok();
    if !ok {
        return String::new();
    }
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

/// `%LOCALAPPDATA%\bitg\install-id`: birinchi xabarda yaratiladi (serverda spam chegarasi uchun).
pub fn install_id() -> String {
    let Some(dir) = bitg_tsf::config::local_dir() else {
        return "0000000000000000".into();
    };
    let file = dir.join("install-id");
    if let Ok(s) = std::fs::read_to_string(&file)
        && crate::feedback::valid_install_id(s.trim())
    {
        return s.trim().to_string();
    }
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
        ^ u128::from(std::process::id());
    let id = crate::feedback::new_install_id(seed);
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(&file, &id);
    id
}
