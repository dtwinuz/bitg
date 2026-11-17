// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! demo/fixtures/layouts.json: oynaning "loaded" xabaridagi tayyor tartiblar (logic.rs keymap_json bilan bir xil) va
//! standart o'chiq dasturlar. Klaviatura tartibi o'zgarsa:
//! `cargo run -q --manifest-path demo/fixtures/gen/Cargo.toml > demo/fixtures/layouts.json`

use bitg_tsf::keymap::{self, KEYS};

fn esc(s: &str) -> String {
    let mut o = String::new();
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c => o.push(c),
        }
    }
    o
}

fn main() {
    let mut out = String::from("{");
    let mut first = true;
    for name in keymap::PRESETS.iter().copied().chain(["custom"]) {
        let m = if name == "custom" {
            keymap::preset(keymap::DEFAULT_LAYOUT).unwrap()
        } else {
            keymap::preset(name).unwrap()
        };
        if !first {
            out.push(',');
        }
        first = false;
        out.push_str(&format!("\"{}\":{{", name));
        let mut f2 = true;
        for &(label, _) in KEYS {
            let row = m.row(label);
            if row.iter().any(Option::is_some) {
                if !f2 {
                    out.push(',');
                }
                f2 = false;
                let cells: Vec<String> = row
                    .iter()
                    .map(|c| format!("\"{}\"", esc(&c.map(String::from).unwrap_or_default())))
                    .collect();
                out.push_str(&format!("\"{}\":[{}]", esc(label), cells.join(",")));
            }
        }
        out.push('}');
    }
    out.push('}');
    let apps: Vec<String> = bitg_tsf::config::DEFAULT_DISABLED_APPS
        .iter()
        .map(|a| format!("\"{a}\""))
        .collect();
    println!("{{\"layouts\":{out},\"default_apps\":[{}]}}", apps.join(","));
}
