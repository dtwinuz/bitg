// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Butun matn o'girishlari (sayt konvertori, CLI, tray): ixtiyoriy UTF-8 matnda panic bo'lmasligi kerak.
#![no_main]

use bitg_core::{OldStyle, Options, convert_words, to_cyrillic, to_old};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&flags, rest)) = data.split_first() else {
        return;
    };
    let Ok(text) = std::str::from_utf8(rest) else {
        return;
    };
    let opts = Options {
        english_guard: flags & 1 != 0,
        cyrillic: flags & 2 != 0,
        restore_apostrophes: flags & 4 != 0,
        ..Options::default()
    };
    let (out, words) = convert_words(text, &opts);
    assert!(words.iter().all(|w| !w.token.is_empty()));
    // Each char grows at most 2x (Cyrillic shch, ' -> U+02BC, a -> Cyrillic a); 4x is a safe bound.
    let bound = |n: usize| 4 * n + 4;
    assert!(
        out.len() <= bound(text.len()),
        "convert {} -> {}",
        text.len(),
        out.len()
    );
    for style in [OldStyle::Official, OldStyle::Ascii] {
        let old = to_old(&out, style);
        assert!(
            old.len() <= bound(out.len()),
            "to_old {} -> {}",
            out.len(),
            old.len()
        );
    }
    let _ = to_old(text, OldStyle::Ascii);
    let cyr = to_cyrillic(text);
    assert!(
        cyr.len() <= bound(text.len()),
        "to_cyrillic {} -> {}",
        text.len(),
        cyr.len()
    );
});
