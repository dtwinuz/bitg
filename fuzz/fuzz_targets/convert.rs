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
    let _ = to_old(&out, OldStyle::Official);
    let _ = to_old(text, OldStyle::Ascii);
    let _ = to_cyrillic(text);
});
