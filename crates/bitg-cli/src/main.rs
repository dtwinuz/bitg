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
  --dry-run, -n   (file) faylga yozmasdan o'zgarishlar ro'yxati va soni
  --in-place      (file) kirish faylining o'ziga yozish
";

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = Options::default();
    let mut ascii = false;
    args.retain(|a| match a.as_str() {
        "--no-guard" => {
            opts.english_guard = false;
            false
        }
        "--no-cyrillic" => {
            opts.cyrillic = false;
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
            false
        }
        _ => true,
    });
    let cmd = args.first().map(String::as_str).unwrap_or("convert");
    let rest = |n: usize| -> String {
        if args.len() > n {
            args[n..].join(" ")
        } else {
            let mut s = String::new();
            let _ = io::stdin().read_to_string(&mut s);
            s
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
            out(&bitg_core::to_old(&rest(1), style))
        }
        "cyr" => out(&bitg_core::to_cyrillic(&rest(1))),
        "explain" => {
            let (converted, report) = bitg_core::convert_words(&rest(1), &opts);
            for w in report {
                let r = match w.reason {
                    Reason::Converted => "o'girildi",
                    Reason::Unchanged => "-",
                    Reason::Protected => "himoya (havola/kod)",
                    Reason::Exception => "istisno",
                    Reason::English => "inglizcha",
                };
                let (tok, ev) = (&w.token, w.evidence);
                match &w.alt {
                    Some(a) => println!("{tok:<24} {ev:>6.2}  {r}  (ikki ma'noli: {a})"),
                    None => println!("{tok:<24} {ev:>6.2}  {r}"),
                }
            }
            println!("\n{converted}");
            ExitCode::SUCCESS
        }
        "eval" => match args.get(1) {
            Some(path) => eval(path, &opts),
            None => {
                eprintln!("bitg eval FAYL.tsv");
                ExitCode::FAILURE
            }
        },
        "file" => file::run(&args[1..], &opts, ascii),
        "diff" => file::diff(&args[1..], &opts, ascii),
        "convert" => out(&bitg_core::convert(&rest(1), &opts)),
        _ => out(&bitg_core::convert(&rest(0), &opts)),
    }
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
