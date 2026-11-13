// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Windows `VERSIONINFO` resursi (Explorer → Xususiyatlar → Tafsilotlar). `build.rs` larga `#[path]` bilan
//! ulanadi: `bitg-settings`, `bitg-cli`, `bitg-tsf`. Versiya `CARGO_PKG_VERSION` dan (ish maydoni versiyasi) —
//! qo'lda yozilmaydi. SignPath Foundation imzolash shartlaridan biri: har bir PE faylda ProductName va
//! ProductVersion bo'lishi. `.rc` matni `OUT_DIR` ga yoziladi va embed-resource bilan bog'lanadi.

use std::path::PathBuf;

/// Bitta binar haqida.
pub struct Info<'a> {
    /// `FileDescription`: Explorer'da fayl nomi o'rnida ko'rinadi.
    pub description: &'a str,
    /// `OriginalFilename`, masalan `bitg_tsf.dll`.
    pub file_name: &'a str,
    /// DLL (`VFT_DLL`) yoki dastur (`VFT_APP`).
    pub dll: bool,
    /// `.rc` oxiriga qo'shiladigan qatorlar (masalan ikonkalar). Yo'llar mutlaq bo'lishi kerak.
    pub extra: &'a str,
}

/// `.rc` satri ichida `"` va `\` qochiriladi.
fn quote(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\"\"")
}

/// `0.4.0`, `0.4.0-rc1` → `(0, 4, 0)`. Raqam bo'lmagan qism — 0.
pub fn numeric(version: &str) -> (u16, u16, u16) {
    let core = version.split(['-', '+']).next().unwrap_or("");
    let mut it = core.split('.').map(|p| p.parse::<u16>().unwrap_or(0));
    (
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
    )
}

/// `.rc` matni (UTF-8, `#pragma code_page(65001)`: © belgisi uchun).
pub fn rc_text(info: &Info, version: &str) -> String {
    let (a, b, c) = numeric(version);
    let internal = info
        .file_name
        .rsplit_once('.')
        .map_or(info.file_name, |(n, _)| n);
    let strings = [
        ("CompanyName", "dtwinuz"),
        ("FileDescription", info.description),
        ("FileVersion", version),
        ("InternalName", internal),
        (
            "LegalCopyright",
            "\u{a9} 2026 dtwinuz and bitg contributors, MIT OR Apache-2.0",
        ),
        ("OriginalFilename", info.file_name),
        ("ProductName", "bitg"),
        ("ProductVersion", version),
    ];
    let mut values = String::new();
    for (k, v) in strings {
        values.push_str(&format!("      VALUE \"{k}\", \"{}\"\n", quote(v)));
    }
    format!(
        "#pragma code_page(65001)\n\
         1 VERSIONINFO\n\
         FILEVERSION {a},{b},{c},0\n\
         PRODUCTVERSION {a},{b},{c},0\n\
         FILEFLAGSMASK 0x3fL\n\
         FILEFLAGS 0x0L\n\
         FILEOS 0x40004L\n\
         FILETYPE {ty}\n\
         FILESUBTYPE 0x0L\n\
         BEGIN\n  BLOCK \"StringFileInfo\"\n  BEGIN\n    BLOCK \"040904b0\"\n    BEGIN\n\
         {values}    END\n  END\n  BLOCK \"VarFileInfo\"\n  BEGIN\n    VALUE \"Translation\", 0x409, 1200\n  END\nEND\n\
         {extra}",
        ty = if info.dll { "0x2L" } else { "0x1L" },
        extra = info.extra,
    )
}

/// Windows nishonida `.rc` ni `OUT_DIR` ga yozadi va yo'lini qaytaradi; boshqa OS'da — `None`.
pub fn write(info: &Info) -> Option<PathBuf> {
    println!("cargo:rerun-if-changed=../../tools/winres/versioninfo.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return None;
    }
    let version = std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR")).join("bitg.rc");
    std::fs::write(&out, rc_text(info, &version)).expect("bitg.rc");
    Some(out)
}

/// Manifest papkasidagi fayl uchun mutlaq yo'l (`.rc` ichida; `/` rc.exe va windres'da ishlaydi).
#[allow(dead_code)]
pub fn manifest_path(rel: &str) -> String {
    let dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    quote(&format!("{}/{rel}", dir.replace('\\', "/")))
}
