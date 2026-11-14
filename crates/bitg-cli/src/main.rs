// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! bitg — o'zbek matnini 2026 alifbosiga o'girish (CLI).

use std::io::{self, Read, Write};
use std::process::ExitCode;
use std::time::Instant;

use bitg_core::{OldStyle, Options, Reason};

mod align;
mod file;
mod ooxml;

const HELP: &str = "\
bitg — o'zbek matnini 2026-yilgi alifboga (ö ğ ş ç ʼ) o'giradi

ISHLATISH:
  bitg [convert] [MATN...]      yangi alifboga (MATN bo'lmasa — stdin)
  bitg old [--ascii] [MATN...]  yangi alifbodan eski lotinga
  bitg cyr [MATN...]            lotindan (yangi yoki eski) o'zbek kirilliga
  bitg explain MATN...          har bir so'z bo'yicha qaror va sabab
  bitg eval FAYL.tsv            sifatni o'lchash (til<TAB>kirish<TAB>kutilgan)
  bitg file FAYL... [-o CHIQISH] [--old] [--dry-run] [--in-place]
                                .docx, .xlsx, .txt, .md ni formatini saqlab o'girish;
                                standart chiqish: FAYL.yangi.docx (--old bilan FAYL.eski.docx)
  bitg diff [--old] [MATN...]   nima o'zgarishini ko'rsatish (qator: eski → yangi), yozmaydi

SOZLAMALAR:
  --no-guard      inglizcha so'zlarni himoya qilmaslik
  --no-cyrillic   kirill matnga tegmaslik
  --tutuq=C       chiqishdagi tutuq belgisi (standart: ʼ U+02BC)
  --exceptions FAYL
                  o'girilmaydigan so'zlar (exceptions.txt formati: har qatorda so'z,
                  # izoh, so'z=yozuv — lug'at qatori)
  --dry-run, -n   (file) faylga yozmasdan o'zgarishlar ro'yxati va soni
  --in-place      (file) kirish faylining o'ziga yozish
";

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = Options::default();
    let mut ascii = false;
    // Faqat yangi alifboga o'girishga tegishli sozlamalar: `old` va `cyr` ularni qabul qilmaydi.
    let mut convert_only: Vec<&'static str> = Vec::new();
    match take_exceptions(&mut args) {
        Ok(Some(path)) => match std::fs::read_to_string(&path) {
            Ok(text) => {
                let (exceptions, rules) = bitg_core::user::parse(&text);
                opts.exceptions = exceptions;
                opts.rules = rules;
                convert_only.push("--exceptions");
            }
            Err(e) => {
                eprintln!("bitg: {path}: {e}");
                return ExitCode::FAILURE;
            }
        },
        Ok(None) => {}
        Err(()) => {
            eprintln!("bitg: --exceptions FAYL kerak");
            return ExitCode::FAILURE;
        }
    }
    args.retain(|a| match a.as_str() {
        "--no-guard" => {
            opts.english_guard = false;
            convert_only.push("--no-guard");
            false
        }
        "--no-cyrillic" => {
            opts.cyrillic = false;
            convert_only.push("--no-cyrillic");
            false
        }
        "--ascii" => {
            ascii = true;
            false
        }
        _ if a.starts_with("--tutuq=") => {
            if let Some(c) = a["--tutuq=".len()..].chars().next() {
                opts.tutuq = c;
            }
            convert_only.push("--tutuq");
            false
        }
        _ => true,
    });
    let cmd = args.first().map(String::as_str).unwrap_or("convert");
    let reverse = matches!(cmd, "old" | "cyr")
        || matches!(cmd, "file" | "diff") && args.iter().any(|a| a == "--old");
    if reverse && !convert_only.is_empty() {
        eprintln!(
            "bitg {cmd}: {} faqat yangi alifboga o'girishda ishlaydi",
            convert_only.join(", ")
        );
        return ExitCode::FAILURE;
    }
    let rest = |n: usize| -> Option<String> {
        if args.len() > n {
            Some(args[n..].join(" "))
        } else {
            read_stdin()
        }
    };
    match cmd {
        "-h" | "--help" | "help" => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        "-V" | "--version" => {
            println!("bitg {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        "old" => {
            let style = if ascii {
                OldStyle::Ascii
            } else {
                OldStyle::Official
            };
            with(rest(1), |t| out(&bitg_core::to_old(&t, style)))
        }
        "cyr" => with(rest(1), |t| out(&bitg_core::to_cyrillic(&t))),
        "explain" => with(rest(1), |text| {
            let (converted, report) = bitg_core::convert_words(&text, &opts);
            for w in report {
                let r = match w.reason {
                    Reason::Converted => "o'girildi",
                    Reason::Unchanged => "-",
                    Reason::Protected => "himoya (havola/kod)",
                    Reason::Exception => "istisno",
                    Reason::English => "inglizcha",
                    Reason::Rule => "lug'at",
                };
                let (tok, ev) = (&w.token, w.evidence);
                match &w.alt {
                    Some(a) => println!("{tok:<24} {ev:>6.2}  {r}  (ikki ma'noli: {a})"),
                    None => println!("{tok:<24} {ev:>6.2}  {r}"),
                }
            }
            println!("\n{converted}");
            ExitCode::SUCCESS
        }),
        "eval" => match args.get(1) {
            Some(path) => eval(path, &opts),
            None => {
                eprintln!("bitg eval FAYL.tsv");
                ExitCode::FAILURE
            }
        },
        "file" => file::run(&args[1..], &opts, ascii),
        "diff" => file::diff(&args[1..], &opts, ascii),
        "convert" => with(rest(1), |t| out(&bitg_core::convert(&t, &opts))),
        _ => with(rest(0), |t| out(&bitg_core::convert(&t, &opts))),
    }
}

/// Matn o'qilmadi (`read_stdin` xatoni yozgan) — chiqish kodi 1.
fn with(text: Option<String>, f: impl FnOnce(String) -> ExitCode) -> ExitCode {
    text.map_or(ExitCode::FAILURE, f)
}

/// stdin'ni butunlay o'qiydi: UTF-8 (BOM bilan ham) yoki BOM'li UTF-16 LE/BE. Boshqa kodlashda xato
/// stderr'ga yoziladi va `None` qaytadi — matn jimgina tashlab yuborilmaydi.
pub(crate) fn read_stdin() -> Option<String> {
    let mut bytes = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut bytes) {
        eprintln!("bitg: stdin: {e}");
        return None;
    }
    decode_text(bytes)
        .map_err(|e| eprintln!("bitg: stdin: {e}"))
        .ok()
}

const NOT_UTF8: &str = "matn UTF-8 emas (UTF-8 yoki BOM'li UTF-16 kerak)";

/// Baytlarni matnga: UTF-8 (BOM olib tashlanadi) yoki BOM'li UTF-16 LE/BE.
fn decode_text(bytes: Vec<u8>) -> Result<String, &'static str> {
    let utf16 = |body: &[u8], be: bool| -> Result<String, &'static str> {
        if !body.len().is_multiple_of(2) {
            return Err("UTF-16 matn toq sonli baytda tugaydi");
        }
        let (pairs, _) = body.as_chunks::<2>();
        let units = pairs.iter().map(|&p| {
            if be {
                u16::from_be_bytes(p)
            } else {
                u16::from_le_bytes(p)
            }
        });
        char::decode_utf16(units)
            .collect::<Result<String, _>>()
            .map_err(|_| "UTF-16 matnda juftsiz surrogat")
    };
    match bytes.as_slice() {
        [0xFF, 0xFE, body @ ..] => utf16(body, false),
        [0xFE, 0xFF, body @ ..] => utf16(body, true),
        [0xEF, 0xBB, 0xBF, body @ ..] => String::from_utf8(body.to_vec()).map_err(|_| NOT_UTF8),
        _ => String::from_utf8(bytes).map_err(|_| NOT_UTF8),
    }
}

/// `--exceptions FAYL` yoki `--exceptions=FAYL` ni ro'yxatdan oladi; oxirgisi hisoblanadi.
fn take_exceptions(args: &mut Vec<String>) -> Result<Option<String>, ()> {
    let mut path = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--exceptions" {
            if i + 1 >= args.len() {
                return Err(());
            }
            path = Some(args.remove(i + 1));
            args.remove(i);
        } else if let Some(p) = args[i].strip_prefix("--exceptions=") {
            if p.is_empty() {
                return Err(());
            }
            path = Some(p.to_string());
            args.remove(i);
        } else {
            i += 1;
        }
    }
    Ok(path)
}

fn out(s: &str) -> ExitCode {
    let mut o = io::stdout().lock();
    let _ = o.write_all(s.as_bytes());
    if !s.ends_with('\n') {
        let _ = o.write_all(b"\n");
    }
    ExitCode::SUCCESS
}

/// Sifat: har bir til guruhi bo'yicha aniq moslik, xato satrlar va tezlik.
fn eval(path: &str, opts: &Options) -> ExitCode {
    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut groups: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
    let mut fails = Vec::new();
    let mut words = 0usize;
    let start = Instant::now();
    for (n, line) in data.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 2 {
            continue;
        }
        let (tag, input) = (cols[0], cols[1]);
        let expected = cols
            .get(2)
            .copied()
            .filter(|s| !s.is_empty())
            .unwrap_or(input);
        words += input.split_whitespace().count();
        let got = bitg_core::convert(input, opts);
        let g = groups.entry(tag.to_string()).or_default();
        g.1 += 1;
        if got == expected {
            g.0 += 1;
        } else {
            fails.push(format!(
                "{}:{tag}  {input}\n    kutilgan: {expected}\n    chiqdi:   {got}",
                n + 1
            ));
        }
    }
    let elapsed = start.elapsed();
    for f in &fails {
        println!("{f}");
    }
    println!();
    let mut ok_all = true;
    for (tag, (ok, total)) in &groups {
        let pct = 100.0 * *ok as f64 / (*total).max(1) as f64;
        println!("{tag:<8} {ok:>5}/{total:<5} {pct:6.2}%");
        ok_all &= ok == total;
    }
    let us = elapsed.as_secs_f64() * 1e6 / words.max(1) as f64;
    println!("tezlik: {us:.2} µs/so'z ({words} so'z)");
    if ok_all {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

#[cfg(test)]
mod tests {
    use super::decode_text;

    fn utf16(bom: [u8; 2], s: &str, be: bool) -> Vec<u8> {
        bom.into_iter()
            .chain(s.encode_utf16().flat_map(
                |u| {
                    if be { u.to_be_bytes() } else { u.to_le_bytes() }
                },
            ))
            .collect()
    }

    #[test]
    fn decodes_utf8_and_utf16_with_bom() {
        assert_eq!(decode_text(b"choy".to_vec()).unwrap(), "choy");
        assert_eq!(decode_text(b"\xEF\xBB\xBFchoy".to_vec()).unwrap(), "choy");
        assert_eq!(
            decode_text(utf16([0xFF, 0xFE], "çoy", false)).unwrap(),
            "çoy"
        );
        assert_eq!(
            decode_text(utf16([0xFE, 0xFF], "çoy", true)).unwrap(),
            "çoy"
        );
    }

    #[test]
    fn rejects_invalid_input() {
        assert!(decode_text(vec![0x63, 0xFF, 0x68]).is_err());
        assert!(decode_text(vec![0xEF, 0xBB, 0xBF, 0xC3]).is_err());
        assert!(decode_text(vec![0xFF, 0xFE, 0x63]).is_err());
        // lone high surrogate
        assert!(decode_text(vec![0xFF, 0xFE, 0x00, 0xD8]).is_err());
    }
}
