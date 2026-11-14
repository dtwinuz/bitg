// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `bitg` buyruqlari: convert, old, cyr, explain, eval, sozlamalar va stdin.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn bitg(args: &[&str], stdin: Option<&str>) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bitg"))
        .args(args)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(s) = stdin {
        child.stdin.take().unwrap().write_all(s.as_bytes()).unwrap();
    }
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

fn tsv(name: &str, body: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("bitg-cli-{name}-{}.tsv", std::process::id()));
    std::fs::write(&path, body).unwrap();
    path
}

#[test]
fn help_and_version() {
    for flag in ["-h", "--help", "help"] {
        let (code, out, _) = bitg(&[flag], None);
        assert_eq!(code, 0);
        assert!(
            out.starts_with("bitg — o'zbek matnini 2026-yilgi alifboga"),
            "{out}"
        );
        assert!(out.contains("bitg eval FAYL.tsv"), "{out}");
    }
    let (code, out, _) = bitg(&["-V"], None);
    assert_eq!(code, 0);
    assert_eq!(out, format!("bitg {}\n", env!("CARGO_PKG_VERSION")));
    assert_eq!(bitg(&["--version"], None).1, out);
}

#[test]
fn convert_from_args_and_stdin() {
    assert_eq!(bitg(&["choy"], None), (0, "çoy\n".into(), String::new()));
    assert_eq!(bitg(&["convert", "bog'da", "choy"], None).1, "boğda çoy\n");
    assert_eq!(bitg(&[], Some("shahar\n")).1, "şahar\n");
    assert_eq!(bitg(&["convert"], Some("choy")).1, "çoy\n");
    assert_eq!(bitg(&[], None).1, "\n");
}

#[test]
fn options_are_removed_from_text() {
    assert_eq!(bitg(&["--tutuq='", "ma'no"], None).1, "ma'no\n");
    assert_eq!(bitg(&["--tutuq=", "ma'no"], None).1, "maʼno\n");
    assert_eq!(
        bitg(&["--no-guard", "--no-cyrillic", "shop", "чой"], None).1,
        "şop чой\n"
    );
    assert_eq!(bitg(&["shop", "чой"], None).1, "shop çoy\n");
}

#[test]
fn old_and_cyrillic() {
    assert_eq!(
        bitg(&["old", "Özbekiston", "çoy"], None).1,
        "Oʻzbekiston choy\n"
    );
    assert_eq!(
        bitg(&["old", "--ascii", "Özbekiston çoy"], None).1,
        "O'zbekiston choy\n"
    );
    assert_eq!(bitg(&["old"], Some("şahar\n")).1, "shahar\n");
    assert_eq!(bitg(&["cyr", "choy shahar"], None).1, "чой шаҳар\n");
}

#[test]
fn explain_lists_reasons() {
    let (code, out, _) = bitg(&["explain", "salom", "ochmaydi"], None);
    assert_eq!(code, 0);
    assert_eq!(
        out,
        "salom                      3.40  -\n\
         ochmaydi                   1.28  o'girildi  (ikki ma'noli: öçmaydi)\n\
         \n\
         salom oçmaydi\n"
    );
    let (_, out, _) = bitg(&["explain"], Some("O'zbekiston https://x.uz Shop"));
    assert_eq!(
        out,
        "O'zbekiston                4.50  o'girildi\n\
         https://x.uz               0.50  himoya (havola/kod)\n\
         Shop                      -4.00  inglizcha\n\
         \n\
         Özbekiston https://x.uz Shop\n"
    );
}

#[test]
fn eval_reports_groups_and_failures() {
    let path = tsv(
        "fail",
        "# izoh\n\nyolg'iz\nuz\tchoy\tçoy\nuz\tsalom\t\nuz\tshahar\tshahar\nen\tshop\n",
    );
    let (code, out, _) = bitg(&["eval", path.to_str().unwrap()], None);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(code, 1);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[..7],
        [
            "6:uz  shahar",
            "    kutilgan: shahar",
            "    chiqdi:   şahar",
            "",
            "en           1/1     100.00%",
            "uz           2/3      66.67%",
            lines[6],
        ]
    );
    assert!(lines[6].starts_with("tezlik: "), "{out}");
    assert!(lines[6].ends_with(" µs/so'z (4 so'z)"), "{out}");
    assert_eq!(lines.len(), 7);
}

#[test]
fn eval_all_pass() {
    let path = tsv("pass", "uz\tchoy\tçoy\n");
    let (code, out, _) = bitg(&["eval", path.to_str().unwrap()], None);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(code, 0);
    assert!(
        out.starts_with("\nuz           1/1     100.00%\ntezlik: "),
        "{out}"
    );
}

#[test]
fn eval_errors() {
    assert_eq!(
        bitg(&["eval"], None),
        (1, String::new(), "bitg eval FAYL.tsv\n".into())
    );
    let missing = std::env::temp_dir().join(format!("bitg-cli-none-{}.tsv", std::process::id()));
    let missing = missing.to_str().unwrap();
    let (code, out, err) = bitg(&["eval", missing], None);
    assert_eq!(code, 1);
    assert!(out.is_empty());
    assert!(err.starts_with(&format!("{missing}: ")), "{err}");
}

#[test]
fn exceptions_file_keeps_words() {
    let ex = tsv("exceptions", "# izoh\nshahar\n");
    let p = ex.to_str().unwrap();
    assert_eq!(bitg(&["--exceptions", p, "shahar"], None).1, "shahar\n");
    assert_eq!(
        bitg(&[&format!("--exceptions={p}"), "shahar choy"], None).1,
        "shahar çoy\n"
    );
    let (_, out, _) = bitg(&["explain", "--exceptions", p, "shahar"], None);
    assert!(out.contains("istisno"), "{out}");
    assert_eq!(bitg(&["shahar"], None).1, "şahar\n");
    let (code, _, err) = bitg(&["--exceptions"], None);
    assert_eq!(code, 1);
    assert!(err.contains("--exceptions"), "{err}");
    let (code, _, _) = bitg(&["--exceptions", "/yoq/fayl.txt", "shahar"], None);
    assert_eq!(code, 1);
    // old va cyr bu sozlamani ishlatmaydi — jimgina e'tiborsiz qoldirilmaydi.
    let (code, out, err) = bitg(&["old", "--exceptions", p, "şahar"], None);
    assert_eq!((code, out.as_str()), (1, ""));
    assert!(err.contains("--exceptions"), "{err}");
    std::fs::remove_file(ex).unwrap();
}

#[test]
fn reverse_commands_reject_convert_options() {
    for (args, flag) in [
        (&["old", "--no-guard", "şahar"][..], "--no-guard"),
        (&["cyr", "--no-guard", "shahar"][..], "--no-guard"),
        (&["cyr", "--no-cyrillic", "shahar"][..], "--no-cyrillic"),
        (&["old", "--tutuq='", "maʼno"][..], "--tutuq"),
        (&["diff", "--old", "--no-guard", "şahar"][..], "--no-guard"),
    ] {
        let (code, out, err) = bitg(args, None);
        assert_eq!(code, 1, "{args:?}");
        assert!(out.is_empty(), "{out}");
        assert!(
            err.contains(flag) && err.contains("faqat yangi alifboga"),
            "{err}"
        );
    }
    // diff (yangi alifboga) sozlamani qabul qiladi
    assert_eq!(bitg(&["diff", "--no-guard", "shop"], None).0, 0);
}

fn bitg_bytes(args: &[&str], stdin: &[u8]) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bitg"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin).unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

#[test]
fn stdin_encodings() {
    // UTF-16 LE (PowerShell 5 quvuri) va BE, BOM bilan
    let le: Vec<u8> = [0xFF, 0xFE]
        .into_iter()
        .chain("shahar\n".encode_utf16().flat_map(u16::to_le_bytes))
        .collect();
    assert_eq!(bitg_bytes(&[], &le), (0, "şahar\n".into(), String::new()));
    let be: Vec<u8> = [0xFE, 0xFF]
        .into_iter()
        .chain("choy".encode_utf16().flat_map(u16::to_be_bytes))
        .collect();
    assert_eq!(bitg_bytes(&["cyr"], &be).1, "чой\n");
    assert_eq!(
        bitg_bytes(&["old"], "\u{FEFF}şahar".as_bytes()).1,
        "shahar\n"
    );
    // UTF-8 emas (cp1251): xato, bo'sh natija emas
    for cmd in [&[][..], &["old"], &["cyr"], &["explain"], &["diff"]] {
        let (code, out, err) = bitg_bytes(cmd, b"\xF7\xEE\xE9 shahar");
        assert_eq!(code, 1, "{cmd:?}");
        assert!(out.is_empty(), "{cmd:?}: {out}");
        assert!(err.contains("UTF-8"), "{cmd:?}: {err}");
    }
}
