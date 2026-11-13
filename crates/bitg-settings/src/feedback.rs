// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! «Xabar berish»: oynadagi shakl → `https://bitg.uz/api/feedback` ga yuboriladigan JSON. Platformaga bog'liq
//! emas — testlanadi. Yozilgan matn jurnaldan olinmaydi: faqat foydalanuvchi shaklga o'zi yozgani ketadi.

use bitg_tsf::config::Config;
use serde_json::{Value, json};

pub const URL: &str = "https://bitg.uz/api/feedback";
/// Rasm (base64 data URL) chegarasi: server 2.5 MB rasm qabul qiladi.
pub const MAX_IMAGE: usize = 3_400_000;

fn field(msg: &Value, k: &str, max: usize) -> String {
    msg.get(k)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .chars()
        .take(max)
        .collect()
}

/// Diagnostika: versiya, Windows yig'ilishi va kalitlar holati. Matn, istisnolar, lug'at — yo'q.
pub fn diag(c: &Config, os_build: &str, rules: usize, words: usize) -> Value {
    json!({
        "os": os_build,
        "enabled": c.enabled,
        "auto_convert": c.auto_convert,
        "english_guard": c.opts.english_guard,
        "cyrillic": c.opts.cyrillic,
        "restore_apostrophes": c.opts.restore_apostrophes,
        "fallback": c.fallback,
        "keymap": c.keymap,
        "layout": c.layout,
        "tutuq": format!("U+{:04X}", c.opts.tutuq as u32),
        "dictionary": {"rules": rules, "words": words},
    })
}

/// Oyna xabaridan server so'rovi. Xato — oynada ko'rsatiladigan matn.
pub fn payload(msg: &Value, install_id: &str, diag: Option<Value>) -> Result<Value, String> {
    let kind = match msg.get("type").and_then(Value::as_str) {
        Some(k @ ("bug" | "idea" | "conversion")) => k,
        _ => return Err("Xabar turini tanlang.".into()),
    };
    let text = field(msg, "text", 4000);
    let typed = field(msg, "typed", 500);
    let got = field(msg, "got", 500);
    let expected = field(msg, "expected", 500);
    if kind == "conversion" {
        if typed.is_empty() || got.is_empty() {
            return Err("Nima yozganingizni va bitg nima chiqarganini kiriting.".into());
        }
    } else if text.chars().count() < 3 {
        return Err("Bir-ikki gap yozing.".into());
    }
    let image = msg.get("image").and_then(Value::as_str).unwrap_or("");
    if !image.is_empty() {
        let ok_type = ["data:image/jpeg;base64,", "data:image/png;base64,"]
            .iter()
            .any(|p| image.starts_with(p));
        if !ok_type {
            return Err("Rasm JPEG yoki PNG bo'lsin.".into());
        }
        if image.len() > MAX_IMAGE {
            return Err("Rasm juda katta (2.5 MB gacha).".into());
        }
    }
    let mut p = json!({
        "v": 1,
        "kind": kind,
        "text": text,
        "app": "windows",
        "version": env!("CARGO_PKG_VERSION"),
        "id": install_id,
    });
    for (k, v) in [
        ("contact", field(msg, "contact", 200)),
        ("typed", typed),
        ("got", got),
        ("expected", expected),
        ("image", image.to_string()),
    ] {
        if !v.is_empty() {
            p[k] = Value::String(v);
        }
    }
    if let Some(d) = diag {
        p["diag"] = d;
    }
    Ok(p)
}

/// Server javobi → oynaga: `{"kind":"feedback_result","ok":..,"ticket"|"error":..}`.
pub fn result(status: Result<(u32, Vec<u8>), String>) -> Value {
    let (code, body) = match status {
        Ok(r) => r,
        Err(e) => {
            return json!({"kind": "feedback_result", "ok": false,
                "error": format!("Yuborilmadi: {e}. Internetni tekshirib, qayta urinib ko'ring.")});
        }
    };
    let v: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    if code == 200 && v["ok"] == true {
        return json!({"kind": "feedback_result", "ok": true, "ticket": v["ticket"]});
    }
    let error = match (code, v["error"].as_str()) {
        (429, _) => {
            let mins = v["retry"].as_u64().unwrap_or(600).div_ceil(60);
            format!("Juda ko'p xabar. {mins} daqiqadan keyin urinib ko'ring.")
        }
        (413, _) => "Rasm juda katta.".to_string(),
        (503, _) => "Qabul vaqtincha o'chiq. Keyinroq urinib ko'ring.".to_string(),
        (_, Some(f)) => format!("Server qabul qilmadi ({f})."),
        _ => format!("Server xatosi ({code}). Keyinroq urinib ko'ring."),
    };
    json!({"kind": "feedback_result", "ok": false, "error": error})
}

/// Bufer rasmi (`CF_DIB`: BITMAPINFOHEADER + ranglar + piksellar) → `.bmp` fayl. Oyna uni JPEG ga siqadi.
pub fn dib_to_bmp(dib: &[u8]) -> Option<Vec<u8>> {
    let u32_at = |i: usize| {
        dib.get(i..i + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let head = u32_at(0)? as usize;
    let bits = dib.get(14..16).map(|b| u16::from_le_bytes([b[0], b[1]]))?;
    let compression = u32_at(16)?;
    let used = u32_at(32)? as usize;
    if !(40..=124).contains(&head) || dib.len() <= head {
        return None;
    }
    let colors = if used > 0 {
        used
    } else if bits <= 8 {
        1usize << bits
    } else {
        0
    };
    // BI_BITFIELDS: 40 baytli sarlavhadan keyin uchta niqob.
    let masks = if compression == 3 && head == 40 {
        12
    } else {
        0
    };
    let offset = 14 + head + masks + colors * 4;
    let total = 14 + dib.len();
    if offset >= total {
        return None;
    }
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&(offset as u32).to_le_bytes());
    out.extend_from_slice(dib);
    Some(out)
}

/// Standart base64 (ekran rasmini oynaga data URL qilib berish uchun).
pub fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (u32::from(c[0]) << 16)
            | (u32::from(*c.get(1).unwrap_or(&0)) << 8)
            | u32::from(*c.get(2).unwrap_or(&0));
        for (i, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            out.push(if i <= c.len() {
                T[(n >> shift) as usize & 63] as char
            } else {
                '='
            });
        }
    }
    out
}

/// O'rnatish identifikatori: 16 hex belgi (spamdan himoya uchun; shaxsga bog'lanmaydi).
pub fn new_install_id(seed: u128) -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u128(seed);
    format!("{:016x}", h.finish())
}

pub fn valid_install_id(s: &str) -> bool {
    s.len() == 16 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_validates_and_drops_empty_fields() {
        let ok = payload(
            &json!({"type": "bug", "text": "  Telegramda ishlamadi ", "contact": "", "image": ""}),
            "0123456789abcdef",
            None,
        )
        .unwrap();
        assert_eq!(ok["text"], "Telegramda ishlamadi");
        assert_eq!(ok["kind"], "bug");
        assert!(
            ok.get("contact").is_none() && ok.get("image").is_none() && ok.get("diag").is_none()
        );

        let conv = payload(
            &json!({"type": "conversion", "typed": "bog'cha", "got": "boğça", "image": "data:image/png;base64,AA=="}),
            "x",
            Some(json!({"os": "19045"})),
        )
        .unwrap();
        assert_eq!(conv["got"], "boğça");
        assert_eq!(conv["diag"]["os"], "19045");
        assert!(
            conv["image"]
                .as_str()
                .unwrap()
                .starts_with("data:image/png")
        );

        for bad in [
            json!({"type": "spam", "text": "salom"}),
            json!({"type": "idea", "text": "a"}),
            json!({"type": "conversion", "typed": "x"}),
            json!({"type": "bug", "text": "salom", "image": "data:text/html;base64,AA"}),
            json!({"type": "bug", "text": "salom", "image": format!("data:image/png;base64,{}", "A".repeat(MAX_IMAGE))}),
        ] {
            assert!(payload(&bad, "x", None).is_err(), "{bad}");
        }
        let long = payload(
            &json!({"type": "idea", "text": "y".repeat(5000)}),
            "x",
            None,
        )
        .unwrap();
        assert_eq!(long["text"].as_str().unwrap().len(), 4000);
    }

    #[test]
    fn diag_has_no_text() {
        let d = diag(&Config::default(), "26100", 2, 5);
        assert_eq!(d["dictionary"]["rules"], 2);
        assert_eq!(d["tutuq"], "U+02BC");
        assert!(!d.to_string().contains("exceptions"));
    }

    #[test]
    fn server_answers_become_messages() {
        let r = |code, body: &str| result(Ok((code, body.as_bytes().to_vec())));
        assert_eq!(
            r(200, r#"{"ok":true,"ticket":"AB12CD"}"#)["ticket"],
            "AB12CD"
        );
        assert!(
            r(429, r#"{"ok":false,"error":"rate","retry":90}"#)["error"]
                .as_str()
                .unwrap()
                .contains("2 daqiqa")
        );
        assert!(r(429, "")["error"].as_str().unwrap().contains("10 daqiqa"));
        assert_eq!(r(413, "")["error"], "Rasm juda katta.");
        assert!(r(503, "")["error"].as_str().unwrap().contains("o'chiq"));
        assert!(
            r(400, r#"{"ok":false,"error":"text"}"#)["error"]
                .as_str()
                .unwrap()
                .contains("(text)")
        );
        assert!(r(500, "x")["error"].as_str().unwrap().contains("500"));
        assert_eq!(result(Err("uzildi".into()))["ok"], false);
    }

    #[test]
    fn dib_becomes_bmp() {
        let mut dib = vec![0u8; 40];
        dib[0] = 40;
        dib[14] = 32;
        dib.extend_from_slice(&[1, 2, 3, 4]);
        let bmp = dib_to_bmp(&dib).unwrap();
        assert_eq!(&bmp[..2], b"BM");
        assert_eq!(u32::from_le_bytes(bmp[2..6].try_into().unwrap()), 58);
        assert_eq!(u32::from_le_bytes(bmp[10..14].try_into().unwrap()), 54);
        // Bitfields niqoblari va 8-bitli palitra.
        let mut bf = dib.clone();
        bf[16] = 3;
        bf.extend_from_slice(&[0; 12]);
        assert_eq!(
            u32::from_le_bytes(dib_to_bmp(&bf).unwrap()[10..14].try_into().unwrap()),
            66
        );
        let mut pal = dib.clone();
        pal[14] = 1;
        pal.extend_from_slice(&[0; 8]);
        assert_eq!(
            u32::from_le_bytes(dib_to_bmp(&pal).unwrap()[10..14].try_into().unwrap()),
            62
        );
        let mut used = pal.clone();
        used[32] = 200;
        assert!(dib_to_bmp(&used).is_none());
        assert!(dib_to_bmp(&[0; 10]).is_none());
        assert!(dib_to_bmp(&dib[..40]).is_none());
        let mut bad = dib.clone();
        bad[0] = 12;
        assert!(dib_to_bmp(&bad).is_none());
    }

    #[test]
    fn base64_matches_rfc4648() {
        for (i, o) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(i.as_bytes()), o);
        }
        assert_eq!(base64(&[255, 254]), "//4=");
    }

    #[test]
    fn install_ids() {
        let a = new_install_id(1);
        assert!(valid_install_id(&a));
        assert!(!valid_install_id("xyz"));
        assert!(!valid_install_id("0123456789abcdeg"));
    }
}
