// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `bitg file` va `bitg diff`: fayllarni (.docx, .xlsx, matn) o'girish va o'zgarishlarni oldindan ko'rish.

use std::collections::BTreeMap;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use bitg_core::{OldStyle, Options};

use crate::align;
use crate::ooxml::{self, Kind};

/// Yo'nalish: yangi alifboga yoki eski lotinga.
fn converter(old: bool, ascii: bool, opts: &Options) -> impl Fn(&str) -> String + '_ {
    let style = if ascii {
        OldStyle::Ascii
    } else {
        OldStyle::Official
    };
    move |s: &str| {
        if old {
            bitg_core::to_old(s, style)
        } else {
            bitg_core::convert(s, opts)
        }
    }
}

enum Format {
    Office(Kind),
    Text,
}

fn format_of(path: &Path) -> Result<Format, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "docx" | "docm" | "dotx" | "dotm" => Ok(Format::Office(Kind::Word)),
        "xlsx" | "xlsm" | "xltx" | "xltm" => Ok(Format::Office(Kind::Excel)),
        "doc" | "xls" | "pdf" | "rtf" | "odt" | "ods" | "pptx" | "ppt" => Err(format!(
            ".{ext} qo'llab-quvvatlanmaydi — .docx, .xlsx yoki matn (.txt, .md) sifatida saqlang"
        )),
        _ => Ok(Format::Text),
    }
}

/// `hujjat.docx` → `hujjat.yangi.docx` (`--old` bilan `hujjat.eski.docx`).
fn default_output(input: &Path, old: bool) -> PathBuf {
    let tag = if old { "eski" } else { "yangi" };
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name = match input.extension() {
        Some(ext) => format!("{stem}.{tag}.{}", ext.to_string_lossy()),
        None => format!("{stem}.{tag}"),
    };
    input.with_file_name(name)
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}

/// Bitta fayl natijasi: yangi baytlar va o'zgarishlar (joy, eski, yangi).
struct Outcome {
    bytes: Vec<u8>,
    changes: Vec<(String, String, String)>,
    text: bool,
}

fn process(path: &Path, f: &dyn Fn(&str) -> String) -> Result<Outcome, String> {
    let format = format_of(path)?;
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    match format {
        Format::Office(kind) => {
            let mut located = Vec::new();
            let bytes = ooxml::convert_package(&data, kind, f, &mut located)?;
            Ok(Outcome {
                bytes,
                changes: located
                    .into_iter()
                    .map(|l| (l.part, l.old, l.new))
                    .collect(),
                text: false,
            })
        }
        Format::Text => {
            let text = String::from_utf8(data)
                .map_err(|_| "matn UTF-8 emas (faylni UTF-8 sifatida saqlang)".to_string())?;
            let (bom, body) = match text.strip_prefix('\u{FEFF}') {
                Some(rest) => ("\u{FEFF}", rest),
                None => ("", text.as_str()),
            };
            let new = f(body);
            let changes = line_changes(body, &new)
                .into_iter()
                .map(|(n, old, new)| (n.to_string(), old, new))
                .collect();
            Ok(Outcome {
                bytes: format!("{bom}{new}").into_bytes(),
                changes,
                text: true,
            })
        }
    }
}

/// Qatorma-qator o'zgargan so'zlar: `(qator raqami, eski, yangi)`.
fn line_changes(old: &str, new: &str) -> Vec<(usize, String, String)> {
    let (a, b): (Vec<&str>, Vec<&str>) = (old.split('\n').collect(), new.split('\n').collect());
    if a.len() != b.len() {
        return align::changed_words(old, new)
            .into_iter()
            .map(|(o, n)| (0, o, n))
            .collect();
    }
    a.iter()
        .zip(&b)
        .enumerate()
        .filter(|(_, (x, y))| x != y)
        .flat_map(|(n, (x, y))| {
            align::changed_words(x, y)
                .into_iter()
                .map(move |(o, w)| (n + 1, o, w))
        })
        .collect()
}

/// O'zgarishlar ro'yxati: matnda qator raqami bilan, hujjatda takrorlanish soni bilan.
fn print_changes(changes: &[(String, String, String)], per_line: bool, indent: &str) {
    if per_line {
        for (line, old, new) in changes {
            println!("{indent}{line}: {old} → {new}");
        }
    } else {
        let mut counts: BTreeMap<(&str, &str), usize> = BTreeMap::new();
        let mut parts: BTreeMap<&str, usize> = BTreeMap::new();
        for (part, old, new) in changes {
            *counts.entry((old, new)).or_default() += 1;
            *parts.entry(part).or_default() += 1;
        }
        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        for ((old, new), n) in sorted {
            println!("{indent}{n:>4}× {old} → {new}");
        }
        let summary: Vec<String> = parts.iter().map(|(p, n)| format!("{p}: {n}")).collect();
        if !summary.is_empty() {
            println!("{indent}({})", summary.join(", "));
        }
    }
    let distinct: std::collections::BTreeSet<_> = changes.iter().map(|c| (&c.1, &c.2)).collect();
    println!(
        "{indent}{} ta o'zgarish, {} xil so'z",
        changes.len(),
        distinct.len()
    );
}

/// `bitg file KIRISH... [-o CHIQISH] [--old] [--dry-run] [--in-place]`.
pub fn run(args: &[String], opts: &Options, ascii: bool) -> ExitCode {
    let mut inputs: Vec<PathBuf> = Vec::new();
    let mut output: Option<PathBuf> = None;
    let (mut old, mut dry, mut in_place) = (false, false, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--old" => old = true,
            "--dry-run" | "-n" => dry = true,
            "--in-place" => in_place = true,
            "-o" | "--output" => match it.next() {
                Some(p) => output = Some(PathBuf::from(p)),
                None => return fail("-o dan keyin chiqish fayli kerak"),
            },
            _ if a.starts_with("--output=") => {
                output = Some(PathBuf::from(&a["--output=".len()..]))
            }
            _ if a.starts_with('-') && a.len() > 1 => {
                return fail(&format!("noma'lum sozlama: {a}"));
            }
            _ => inputs.push(PathBuf::from(a)),
        }
    }
    if inputs.is_empty() {
        return fail("bitg file FAYL [-o CHIQISH] [--old] [--dry-run] [--in-place]");
    }
    if output.is_some() && (inputs.len() > 1 || in_place) {
        return fail("-o faqat bitta fayl bilan va --in-place'siz ishlaydi");
    }
    let f = converter(old, ascii, opts);
    let mut ok = true;
    for input in &inputs {
        let label = input.display();
        let outcome = match process(input, &f) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("{label}: {e}");
                ok = false;
                continue;
            }
        };
        if dry {
            if inputs.len() > 1 {
                println!("{label}");
                print_changes(&outcome.changes, outcome.text, "  ");
            } else {
                print_changes(&outcome.changes, outcome.text, "");
            }
            continue;
        }
        let target = if in_place {
            input.clone()
        } else {
            output.clone().unwrap_or_else(|| default_output(input, old))
        };
        if !in_place && same_file(input, &target) {
            eprintln!("{label}: kirish fayli ustiga yozilmaydi (--in-place bilan ruxsat bering)");
            ok = false;
            continue;
        }
        if let Err(e) = write_atomic(&target, &outcome.bytes) {
            eprintln!("{}: {e}", target.display());
            ok = false;
            continue;
        }
        println!(
            "{label} → {} ({} ta o'zgarish)",
            target.display(),
            outcome.changes.len()
        );
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Avval vaqtinchalik faylga yozib, keyin almashtiradi — yarim yozilgan hujjat qolmaydi.
fn write_atomic(target: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut tmp = target.as_os_str().to_owned();
    tmp.push(".bitg-tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, target).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

/// `bitg diff [--old] [MATN...]`: nima o'zgarishini ko'rsatadi, hech narsa yozmaydi (MATN bo'lmasa — stdin).
pub fn diff(args: &[String], opts: &Options, ascii: bool) -> ExitCode {
    let old = args.iter().any(|a| a == "--old");
    let words: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| *a != "--old")
        .collect();
    let text = if words.is_empty() {
        let mut s = String::new();
        let _ = io::stdin().read_to_string(&mut s);
        s
    } else {
        words.join(" ")
    };
    let new = converter(old, ascii, opts)(&text);
    let changes: Vec<_> = line_changes(&text, &new)
        .into_iter()
        .map(|(n, o, w)| (n.to_string(), o, w))
        .collect();
    print_changes(&changes, true, "");
    ExitCode::SUCCESS
}

fn fail(msg: &str) -> ExitCode {
    eprintln!("{msg}");
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_names() {
        assert_eq!(
            default_output(Path::new("dir/hujjat.docx"), false),
            Path::new("dir/hujjat.yangi.docx")
        );
        assert_eq!(
            default_output(Path::new("a.txt"), true),
            Path::new("a.eski.txt")
        );
        assert_eq!(
            default_output(Path::new("README"), false),
            Path::new("README.yangi")
        );
    }

    #[test]
    fn lines_numbered() {
        let old = "salom\nO'zbekiston, bog'\n\nchoy";
        let new = bitg_core::convert(old, &Options::default());
        let ch = line_changes(old, &new);
        assert_eq!(
            ch,
            [
                (2, "O'zbekiston".into(), "Özbekiston".into()),
                (2, "bog'".into(), "boğ".into()),
                (4, "choy".into(), "çoy".into()),
            ]
        );
    }
}
