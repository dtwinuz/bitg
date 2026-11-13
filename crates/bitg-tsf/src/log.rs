// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Diagnostika log'i: `%LOCALAPPDATA%\bitg\tsf.log`. Matnning o'zi yozilmaydi — faqat dastur nomi,
//! natija va uzunliklar (maxfiylik: klaviatura dasturi hech qachon yozilgan so'zni saqlamaydi).

use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BYTES: u64 = 512 * 1024;

fn path() -> Option<&'static PathBuf> {
    static P: OnceLock<Option<PathBuf>> = OnceLock::new();
    P.get_or_init(|| {
        let dir = crate::config::local_dir()?;
        std::fs::create_dir_all(&dir).ok()?;
        Some(dir.join("tsf.log"))
    })
    .as_ref()
}

/// Joriy dastur nomi (chrome.exe, Telegram.exe ...).
pub fn process() -> &'static str {
    static P: OnceLock<String> = OnceLock::new();
    P.get_or_init(|| {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "?".into())
    })
}

pub fn write(msg: &str) {
    let Some(p) = path() else { return };
    if std::fs::metadata(p)
        .map(|m| m.len() > MAX_BYTES)
        .unwrap_or(false)
    {
        let _ = std::fs::remove_file(p);
    }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(p)
    {
        let _ = writeln!(f, "{ts} pid={} {} {msg}", std::process::id(), process());
    }
}
