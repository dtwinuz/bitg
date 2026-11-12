//! Yozish paytidagi sessiya (IME): har so'z uchun tahrir faqat shu so'zning o'zini almashtiradi.
//! `delete` so'zdan uzun bo'lsa, IME foydalanuvchining oldingi matnini o'chirib yuborgan bo'lardi.
#![no_main]

use bitg_core::{Options, Session};
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
    let mut s = Session::new();
    for (i, word) in text.split(char::is_whitespace).enumerate() {
        if flags & 8 != 0 && i % 7 == 6 {
            s.reset();
        }
        if word.is_empty() {
            continue;
        }
        if let Some(e) = s.complete_word(word, &opts) {
            assert!(
                e.delete <= word.chars().count(),
                "delete {} > {:?}",
                e.delete,
                word
            );
            assert_ne!(e.insert, word);
        }
    }
});
