// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `bitg file` va `bitg diff` buyruqlari: chiqish nomi, kirishni saqlash, --dry-run.

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

fn bitg() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bitg"))
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bitg-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn docx(path: &Path, body: &str) {
    let mut w = ZipWriter::new(Cursor::new(Vec::new()));
    w.start_file("[Content_Types].xml", SimpleFileOptions::default())
        .unwrap();
    w.write_all(b"<Types/>").unwrap();
    w.start_file("word/document.xml", SimpleFileOptions::default())
        .unwrap();
    w.write_all(body.as_bytes()).unwrap();
    std::fs::write(path, w.finish().unwrap().into_inner()).unwrap();
}

fn entry(path: &Path, name: &str) -> String {
    let mut a = ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut s = String::new();
    a.by_name(name).unwrap().read_to_string(&mut s).unwrap();
    s
}

const DOC: &str = r#"<w:document><w:body><w:p><w:r><w:rPr><w:b/></w:rPr><w:t>O</w:t></w:r><w:r><w:t>'zbekiston</w:t></w:r></w:p><w:p><w:r><w:t>choy, choy</w:t></w:r></w:p></w:body></w:document>"#;

#[test]
fn docx_default_output_next_to_input() {
    let dir = temp_dir("docx");
    let input = dir.join("hujjat.docx");
    docx(&input, DOC);
    let before = std::fs::read(&input).unwrap();
    let out = bitg().arg("file").arg(&input).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(std::fs::read(&input).unwrap(), before);
    let doc = entry(&dir.join("hujjat.yangi.docx"), "word/document.xml");
    assert!(doc.contains("<w:b/></w:rPr><w:t>Ö</w:t></w:r><w:r><w:t>zbekiston</w:t>"));
    assert!(doc.contains("<w:t>çoy, çoy</w:t>"));
    assert_eq!(
        entry(&dir.join("hujjat.yangi.docx"), "[Content_Types].xml"),
        "<Types/>"
    );
}

#[test]
fn refuses_to_overwrite_input() {
    let dir = temp_dir("same");
    let input = dir.join("a.txt");
    std::fs::write(&input, "choy").unwrap();
    let out = bitg()
        .args(["file", "-o"])
        .arg(&input)
        .arg(&input)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(std::fs::read_to_string(&input).unwrap(), "choy");

    let out = bitg()
        .args(["file", "--in-place"])
        .arg(&input)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(std::fs::read_to_string(&input).unwrap(), "çoy");
}

#[test]
fn dry_run_writes_nothing() {
    let dir = temp_dir("dry");
    let input = dir.join("hujjat.docx");
    docx(&input, DOC);
    let out = bitg()
        .args(["file", "--dry-run"])
        .arg(&input)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("2× choy → çoy"), "{stdout}");
    assert!(stdout.contains("1× O'zbekiston → Özbekiston"), "{stdout}");
    assert!(stdout.contains("3 ta o'zgarish, 2 xil so'z"), "{stdout}");
    assert!(!dir.join("hujjat.yangi.docx").exists());
}

#[test]
fn text_file_dry_run_has_line_numbers() {
    let dir = temp_dir("txt");
    let input = dir.join("matn.md");
    std::fs::write(&input, "\u{FEFF}# Sarlavha\r\nO'zbekiston\r\nchoy\r\n").unwrap();
    let out = bitg().args(["file", "-n"]).arg(&input).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("2: O'zbekiston → Özbekiston"), "{stdout}");
    assert!(stdout.contains("3: choy → çoy"), "{stdout}");

    assert!(bitg().arg("file").arg(&input).status().unwrap().success());
    assert_eq!(
        std::fs::read_to_string(dir.join("matn.yangi.md")).unwrap(),
        "\u{FEFF}# Sarlavha\r\nÖzbekiston\r\nçoy\r\n"
    );
}

#[test]
fn diff_from_stdin() {
    let mut child = bitg()
        .arg("diff")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("salom\nbog'da choy".as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("2: bog'da → boğda"), "{stdout}");
    assert!(stdout.contains("2: choy → çoy"), "{stdout}");
    assert!(stdout.contains("2 ta o'zgarish"), "{stdout}");
}

#[test]
fn unsupported_binary_format() {
    let dir = temp_dir("doc");
    let input = dir.join("eski.doc");
    std::fs::write(&input, b"\xD0\xCF\x11\xE0").unwrap();
    let out = bitg().arg("file").arg(&input).output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains(".docx"));
}
