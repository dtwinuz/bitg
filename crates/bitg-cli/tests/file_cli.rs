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

fn run(args: &[&std::ffi::OsStr]) -> (i32, String, String) {
    let out = bitg().args(args).output().unwrap();
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

#[test]
fn old_direction_and_output_options() {
    let dir = temp_dir("old");
    let a = dir.join("a.txt");
    std::fs::write(&a, "Özbekiston çoy\n").unwrap();
    let (code, out, _) = run(&[
        "--ascii".as_ref(),
        "file".as_ref(),
        "--old".as_ref(),
        a.as_ref(),
    ]);
    assert_eq!(code, 0);
    let eski = dir.join("a.eski.txt");
    assert_eq!(
        out,
        format!("{} → {} (2 ta o'zgarish)\n", a.display(), eski.display())
    );
    assert_eq!(
        std::fs::read_to_string(&eski).unwrap(),
        "O'zbekiston choy\n"
    );

    let o = dir.join("o.txt");
    let (code, _, _) = run(&[
        "file".as_ref(),
        "--old".as_ref(),
        a.as_ref(),
        "-o".as_ref(),
        o.as_ref(),
    ]);
    assert_eq!(code, 0);
    assert_eq!(std::fs::read_to_string(&o).unwrap(), "Oʻzbekiston choy\n");

    let b = dir.join("b.txt");
    std::fs::write(&b, "choy\n").unwrap();
    let p = dir.join("p.txt");
    let flag = format!("--output={}", p.display());
    let (code, _, _) = run(&["file".as_ref(), flag.as_ref(), b.as_ref()]);
    assert_eq!(code, 0);
    assert_eq!(std::fs::read_to_string(&p).unwrap(), "çoy\n");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn argument_errors() {
    let dir = temp_dir("args");
    let a = dir.join("a.txt");
    let b = dir.join("b.txt");
    std::fs::write(&a, "choy").unwrap();
    std::fs::write(&b, "choy").unwrap();
    let z = dir.join("z.txt");
    let only_one = "-o faqat bitta fayl bilan va --in-place'siz ishlaydi\n";
    let cases: [(Vec<&std::ffi::OsStr>, &str); 5] = [
        (
            vec!["file".as_ref(), "-x".as_ref(), a.as_ref()],
            "noma'lum sozlama: -x\n",
        ),
        (
            vec!["file".as_ref(), "--old".as_ref()],
            "bitg file FAYL [-o CHIQISH] [--old] [--dry-run] [--in-place]\n",
        ),
        (
            vec!["file".as_ref(), a.as_ref(), "-o".as_ref()],
            "-o dan keyin chiqish fayli kerak\n",
        ),
        (
            vec![
                "file".as_ref(),
                "-o".as_ref(),
                z.as_ref(),
                a.as_ref(),
                b.as_ref(),
            ],
            only_one,
        ),
        (
            vec![
                "file".as_ref(),
                "-o".as_ref(),
                z.as_ref(),
                "--in-place".as_ref(),
                a.as_ref(),
            ],
            only_one,
        ),
    ];
    for (args, err) in cases {
        assert_eq!(run(&args), (1, String::new(), err.to_string()), "{args:?}");
    }
    assert!(!z.exists());
    assert_eq!(std::fs::read_to_string(&a).unwrap(), "choy");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn unreadable_inputs_fail_but_others_are_written() {
    let dir = temp_dir("bad");
    let bad = dir.join("bad.txt");
    std::fs::write(&bad, b"\xff").unwrap();
    let good = dir.join("good.txt");
    std::fs::write(&good, "choy").unwrap();
    let (code, out, err) = run(&["file".as_ref(), "-".as_ref(), bad.as_ref(), good.as_ref()]);
    assert_eq!(code, 1);
    let lines: Vec<&str> = err.lines().collect();
    assert_eq!(lines.len(), 2, "{err}");
    assert!(lines[0].starts_with("-: "), "{err}");
    assert_eq!(
        lines[1],
        format!(
            "{}: matn UTF-8 emas (faylni UTF-8 sifatida saqlang)",
            bad.display()
        )
    );
    let new = dir.join("good.yangi.txt");
    assert_eq!(
        out,
        format!("{} → {} (1 ta o'zgarish)\n", good.display(), new.display())
    );
    assert_eq!(std::fs::read_to_string(&new).unwrap(), "çoy");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn write_error_is_reported() {
    let dir = temp_dir("nodir");
    let a = dir.join("a.txt");
    std::fs::write(&a, "choy").unwrap();
    let target = dir.join("yoq").join("x.txt");
    let (code, out, err) = run(&["file".as_ref(), a.as_ref(), "-o".as_ref(), target.as_ref()]);
    assert_eq!(code, 1);
    assert!(out.is_empty());
    assert!(err.starts_with(&format!("{}: ", target.display())), "{err}");
    assert!(!dir.join("yoq").exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn dry_run_several_files_is_indented() {
    let dir = temp_dir("many");
    let a = dir.join("a.txt");
    let b = dir.join("b.txt");
    std::fs::write(&a, "salom\n").unwrap();
    std::fs::write(&b, "choy\n").unwrap();
    let (code, out, _) = run(&["file".as_ref(), "-n".as_ref(), a.as_ref(), b.as_ref()]);
    assert_eq!(code, 0);
    assert_eq!(
        out,
        format!(
            "{}\n  0 ta o'zgarish, 0 xil so'z\n{}\n  1: choy → çoy\n  1 ta o'zgarish, 1 xil so'z\n",
            a.display(),
            b.display()
        )
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn dry_run_docx_without_changes() {
    let dir = temp_dir("same-docx");
    let input = dir.join("h.docx");
    docx(
        &input,
        "<w:document><w:body><w:p><w:r><w:t>salom</w:t></w:r></w:p></w:body></w:document>",
    );
    let (code, out, _) = run(&["file".as_ref(), "--dry-run".as_ref(), input.as_ref()]);
    assert_eq!((code, out.as_str()), (0, "0 ta o'zgarish, 0 xil so'z\n"));

    docx(&input, DOC);
    let (_, out, _) = run(&["file".as_ref(), "--dry-run".as_ref(), input.as_ref()]);
    assert_eq!(
        out,
        "   2× choy → çoy\n   1× O'zbekiston → Özbekiston\n(word/document.xml: 3)\n3 ta o'zgarish, 2 xil so'z\n"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn diff_old_from_args() {
    let (code, out, _) = run(&[
        "diff".as_ref(),
        "--old".as_ref(),
        "Özbekiston".as_ref(),
        "çoy".as_ref(),
    ]);
    assert_eq!(code, 0);
    assert_eq!(
        out,
        "1: Özbekiston → Oʻzbekiston\n1: çoy → choy\n2 ta o'zgarish, 2 xil so'z\n"
    );
    let (_, out, _) = run(&[
        "--ascii".as_ref(),
        "diff".as_ref(),
        "--old".as_ref(),
        "Özbekiston".as_ref(),
    ]);
    assert_eq!(
        out,
        "1: Özbekiston → O'zbekiston\n1 ta o'zgarish, 1 xil so'z\n"
    );
    let (_, out, _) = run(&["diff".as_ref(), "salom".as_ref()]);
    assert_eq!(out, "0 ta o'zgarish, 0 xil so'z\n");
}
