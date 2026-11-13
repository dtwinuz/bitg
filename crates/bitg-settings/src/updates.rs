// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! «Yangiliklar» (NEWS.md) va yangilanish manifesti (`https://bitg.uz/update/stable.json`). Platformaga bog'liq
//! emas — testlanadi. Tarmoq va o'rnatish `net.rs` / `main.rs` da.

use serde_json::{Value, json};

/// Ilova ichidagi «Yangiliklar»: qurish vaqtida kiritiladi, tarmoqsiz ishlaydi.
pub const NEWS: &str = include_str!("../../../NEWS.md");
/// Manifest manzili (faqat HTTPS, bitg.uz).
pub const MANIFEST_URL: &str = "https://bitg.uz/update/stable.json";
/// O'rnatuvchi faqat shu joydan yuklanadi: manifest buzilgan bo'lsa ham boshqa saytdan fayl olinmaydi.
pub const DOWNLOAD_PREFIX: &str = "https://github.com/dtwinuz/bitg/releases/download/";
/// MSI hajmi chegarasi (hozir ~5 MB).
pub const MAX_SIZE: u64 = 80 * 1024 * 1024;

/// `**Lead.** matn` → (lead, matn); belgilash (`**`, `` ` ``, havolalar) olib tashlanadi.
fn item(line: &str) -> Value {
    let plain = |s: &str| {
        let mut out = String::with_capacity(s.len());
        let mut rest = s;
        // [matn](url) → matn
        while let Some(i) = rest.find('[') {
            let (head, tail) = rest.split_at(i);
            out.push_str(head);
            match (tail.find("]("), tail.find(')')) {
                (Some(m), Some(e)) if m < e => {
                    out.push_str(&tail[1..m]);
                    rest = &tail[e + 1..];
                }
                _ => {
                    out.push('[');
                    rest = &tail[1..];
                }
            }
        }
        out.push_str(rest);
        out.replace("**", "").replace('`', "").trim().to_string()
    };
    if let Some(body) = line.strip_prefix("**")
        && let Some(end) = body.find("**")
    {
        return json!({"lead": plain(&body[..end]), "text": plain(&body[end + 2..])});
    }
    json!({"lead": "", "text": plain(line)})
}

/// NEWS.md → `{"kind":"news","versions":[{"version","date","groups":[{"kind","title","items":[..]}]}]}`.
/// Bo'lim turi: `new` (Yangi), `better` (Yaxshilandi), `fixed` (Tuzatildi), qolgani `other`.
pub fn news_json(md: &str) -> Value {
    let mut versions: Vec<Value> = Vec::new();
    for line in md.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            let (v, d) = h.split_once('—').unwrap_or((h, ""));
            versions.push(json!({"version": v.trim(), "date": d.trim(), "groups": []}));
        } else if let Some(h) = line.strip_prefix("### ") {
            let Some(groups) = versions.last_mut().and_then(|v| v["groups"].as_array_mut()) else {
                continue;
            };
            let title = h.trim();
            let kind = match title.to_lowercase().as_str() {
                "yangi" => "new",
                "yaxshilandi" => "better",
                "tuzatildi" => "fixed",
                _ => "other",
            };
            groups.push(json!({"kind": kind, "title": title, "items": []}));
        } else if let Some(b) = line.strip_prefix("- ")
            && let Some(items) = versions
                .last_mut()
                .and_then(|v| v["groups"].as_array_mut())
                .and_then(|g| g.last_mut())
                .and_then(|g| g["items"].as_array_mut())
        {
            items.push(item(b));
        }
    }
    json!({"kind": "news", "versions": versions})
}

/// `x.y.z` raqamlari; `-rc1` kabi qo'shimcha relizdan oldin (0.4.0-rc1 < 0.4.0).
fn parts(v: &str) -> Option<([u64; 3], bool)> {
    let v = v.trim().trim_start_matches('v');
    let (core, pre) = match v.split_once('-') {
        Some((c, _)) => (c, true),
        None => (v, false),
    };
    let mut n = [0u64; 3];
    let mut it = core.split('.');
    for slot in &mut n {
        *slot = it.next()?.parse().ok()?;
    }
    if it.next().is_some() {
        return None;
    }
    Some((n, pre))
}

/// `a` versiyasi `b` dan yangiroqmi. Noto'g'ri versiya — yo'q.
pub fn newer(a: &str, b: &str) -> bool {
    match (parts(a), parts(b)) {
        (Some((x, xp)), Some((y, yp))) => x > y || (x == y && yp && !xp),
        _ => false,
    }
}

/// Tekshirilgan manifest.
#[derive(Clone, Debug, PartialEq)]
pub struct Update {
    pub version: String,
    pub date: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub notes: Vec<String>,
    pub page: String,
}

/// Manifestni tekshiradi. `Ok(None)` — joriy versiya eng yangisi.
pub fn parse_manifest(text: &str, current: &str) -> Result<Option<Update>, String> {
    let v: Value = serde_json::from_str(text).map_err(|_| "manifest JSON emas".to_string())?;
    let s = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let version = s("version");
    if parts(&version).is_none() {
        return Err("versiya noto'g'ri".into());
    }
    if !newer(&version, current) {
        return Ok(None);
    }
    let w = v.get("windows").ok_or("windows yo'q")?;
    let ws = |k: &str| w.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let url = ws("url");
    let file = url.strip_prefix(DOWNLOAD_PREFIX).unwrap_or("");
    if file.is_empty()
        || !file.ends_with(".msi")
        || file.contains(['?', '#', '\\', ' ', '"'])
        || file.contains("..")
    {
        return Err("yuklash manzili ruxsat etilmagan".into());
    }
    let sha256 = ws("sha256").to_ascii_lowercase();
    if sha256.len() != 64 || !sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("sha256 noto'g'ri".into());
    }
    let size = w.get("size").and_then(Value::as_u64).unwrap_or(0);
    if size == 0 || size > MAX_SIZE {
        return Err("hajm noto'g'ri".into());
    }
    let notes = v
        .get("notes")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .take(6)
                .map(|n| n.chars().take(300).collect())
                .collect()
        })
        .unwrap_or_default();
    let page = s("page");
    Ok(Some(Update {
        version,
        date: s("date").chars().take(10).collect(),
        url,
        sha256,
        size,
        notes,
        page: if page.starts_with("https://github.com/dtwinuz/bitg/") {
            page
        } else {
            String::new()
        },
    }))
}

pub fn update_json(u: &Update) -> Value {
    json!({
        "version": u.version, "date": u.date, "notes": u.notes, "size": u.size, "page": u.page,
    })
}

/// Yuklangan faylning SHA-256 (kichik hex).
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"{"version":"0.4.0","date":"2026-11-14","notes":["**Lug'at.** yangi"],
        "page":"https://github.com/dtwinuz/bitg/releases/tag/v0.4.0",
        "windows":{"url":"https://github.com/dtwinuz/bitg/releases/download/v0.4.0/bitg-0.4.0-x64.msi",
        "sha256":"ABababababababababababababababababababababababababababababababab","size":5000000}}"#;

    #[test]
    fn versions_compare() {
        assert!(newer("0.4.0", "0.3.1"));
        assert!(newer("v1.0.0", "0.9.9"));
        assert!(newer("0.4.0", "0.4.0-rc1"));
        assert!(!newer("0.4.0-rc1", "0.4.0"));
        assert!(!newer("0.4.0", "0.4.0"));
        assert!(!newer("0.3.9", "0.4.0"));
        assert!(!newer("x", "0.1.0"));
        assert!(!newer("1.2", "0.1.0"));
        assert!(!newer("1.2.3.4", "0.1.0"));
    }

    #[test]
    fn manifest_is_validated() {
        let u = parse_manifest(GOOD, "0.3.1").unwrap().unwrap();
        assert_eq!(u.version, "0.4.0");
        assert_eq!(u.sha256.len(), 64);
        assert_eq!(u.sha256, u.sha256.to_ascii_lowercase());
        assert_eq!(update_json(&u)["notes"][0], "**Lug'at.** yangi");
        assert_eq!(parse_manifest(GOOD, "0.4.0"), Ok(None));
        assert!(parse_manifest("nope", "0.1.0").is_err());
        assert!(parse_manifest(r#"{"version":"z"}"#, "0.1.0").is_err());
        assert!(parse_manifest(r#"{"version":"9.0.0"}"#, "0.1.0").is_err());
        for (from, to) in [
            (
                "https://github.com/dtwinuz/bitg/releases/download/",
                "https://evil.example/",
            ),
            ("bitg-0.4.0-x64.msi", "a.exe"),
            ("bitg-0.4.0-x64.msi", "../x.msi"),
            ("ABabab", "zz"),
            ("5000000", "0"),
            ("5000000", "999999999999"),
        ] {
            assert!(
                parse_manifest(&GOOD.replace(from, to), "0.1.0").is_err(),
                "{to}"
            );
        }
        let odd_page = GOOD.replace(
            "https://github.com/dtwinuz/bitg/releases/tag",
            "https://x.uz",
        );
        assert_eq!(
            parse_manifest(&odd_page, "0.1.0").unwrap().unwrap().page,
            ""
        );
    }

    #[test]
    fn news_is_parsed() {
        let v = news_json(
            "# t\n- ignored\n### ignored\n## 1.0.0 — 2026-01-02\n### Yangi\n- **Bir.** ikki `kod` [havola](https://x) [qavs\n- oddiy\n### Boshqa\n- x\n## 0.9.0\n",
        );
        let vs = v["versions"].as_array().unwrap();
        assert_eq!(vs.len(), 2);
        assert_eq!(vs[0]["date"], "2026-01-02");
        let g = &vs[0]["groups"][0];
        assert_eq!(g["kind"], "new");
        assert_eq!(g["items"][0]["lead"], "Bir.");
        assert_eq!(g["items"][0]["text"], "ikki kod havola [qavs");
        assert_eq!(g["items"][1]["lead"], "");
        assert_eq!(vs[0]["groups"][1]["kind"], "other");
        assert_eq!(vs[1]["date"], "");
        // Haqiqiy fayl: eng yangi versiya ilova versiyasidan eski emas.
        let real = news_json(NEWS);
        let top = real["versions"][0]["version"].as_str().unwrap();
        assert!(
            !newer(env!("CARGO_PKG_VERSION"), top),
            "NEWS.md da {top} eski"
        );
    }

    #[test]
    fn sha256_of_abc() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
