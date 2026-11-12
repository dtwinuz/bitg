// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Lug'atlarni (`src/data/*.tsv`) qurish vaqtida tayyor hash-jadvalga aylantiradi: ishga tushganda tahlil
//! qilish va xotira ajratish yo'q, jadval binarning faqat o'qiladigan qismida turadi (IME yuklangan barcha
//! dasturlar uni bitta nusxada bo'lishadi). Format `src/lexicon.rs` da.

use std::{env, fs, path::Path};

/// `src/lexicon.rs` dagi bilan bir xil bo'lishi shart.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn main() {
    let out = env::var("OUT_DIR").expect("OUT_DIR");
    for name in ["uz", "en", "sh_split"] {
        let src = format!("src/data/{name}.tsv");
        println!("cargo:rerun-if-changed={src}");
        let text = fs::read_to_string(&src).expect("lug'at fayli");
        // Takror so'zda oxirgisi qoladi (avvalgi HashMap::insert xulqi).
        let mut rows: Vec<(&str, f32)> = Vec::new();
        let mut seen = std::collections::HashMap::new();
        for line in text.lines().filter(|l| !l.starts_with('#')) {
            let Some((w, s)) = line.split_once('\t') else {
                continue;
            };
            let Ok(v) = s.trim().parse::<f32>() else {
                continue;
            };
            match seen.get(w) {
                Some(&i) => rows[i] = (w, v),
                None => {
                    seen.insert(w, rows.len());
                    rows.push((w, v));
                }
            }
        }
        let mut keys = Vec::new();
        let mut offs = Vec::with_capacity((rows.len() + 1) * 4);
        let mut weights = Vec::with_capacity(rows.len() * 4);
        for (w, v) in &rows {
            offs.extend_from_slice(&(keys.len() as u32).to_le_bytes());
            keys.extend_from_slice(w.as_bytes());
            weights.extend_from_slice(&v.to_le_bytes());
        }
        offs.extend_from_slice(&(keys.len() as u32).to_le_bytes());
        // Ochiq manzilli jadval, to'ldirilish <= 50%: slot = so'z raqami + 1, 0 = bo'sh.
        let size = (rows.len() * 2).next_power_of_two().max(8);
        let mut slots = vec![0u32; size];
        for (i, (w, _)) in rows.iter().enumerate() {
            let mut s = fnv1a(w.as_bytes()) as usize & (size - 1);
            while slots[s] != 0 {
                s = (s + 1) & (size - 1);
            }
            slots[s] = i as u32 + 1;
        }
        let tab: Vec<u8> = slots.iter().flat_map(|s| s.to_le_bytes()).collect();
        let dir = Path::new(&out);
        fs::write(dir.join(format!("{name}.keys")), &keys).unwrap();
        fs::write(dir.join(format!("{name}.off")), &offs).unwrap();
        fs::write(dir.join(format!("{name}.w")), &weights).unwrap();
        fs::write(dir.join(format!("{name}.tab")), &tab).unwrap();
    }
}
