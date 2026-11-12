// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Toza tezlik o'lchovi: lug'at oldindan yuklanadi, `convert` dan boshqa narsa vaqtga kirmaydi.
//! `cargo run --release -p bitg-core --example bench -- eval/out/uz.tsv`

use std::time::Instant;

use bitg_core::{Options, convert};

fn main() {
    let path = std::env::args().nth(1).expect("tsv fayl yo'li");
    let text = std::fs::read_to_string(&path).expect("faylni o'qib bo'lmadi");
    let lines: Vec<&str> = text
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split('\t').nth(1))
        .collect();
    let words: usize = lines.iter().map(|l| l.split_whitespace().count()).sum();
    let opts = Options::default();

    let t0 = Instant::now();
    bitg_core::lang::warm_up();
    let load = t0.elapsed();

    for _ in 0..2 {
        for l in &lines {
            std::hint::black_box(convert(l, &opts));
        }
    }
    let reps = 5;
    let t0 = Instant::now();
    for _ in 0..reps {
        for l in &lines {
            std::hint::black_box(convert(l, &opts));
        }
    }
    let per_word = t0.elapsed().as_secs_f64() * 1e6 / (reps * words) as f64;
    println!("lug'at yuklash: {:.1} ms", load.as_secs_f64() * 1e3);
    println!("{per_word:.2} µs/so'z ({words} so'z, {} gap)", lines.len());
}
