// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 2026 alifbosi → eski lotin. O'tish davrida (2027–2031) yangi harflarni qabul qilmaydigan saytlar,
//! bazalar va hujjatlar uchun.

use crate::chars::is_apostrophe;

/// Eski lotindagi o' / g' / tutuq qanday belgi bilan yozilsin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OldStyle {
    /// Rasmiy: oʻ gʻ (U+02BB) va tutuq ʼ (U+02BC).
    #[default]
    Official,
    /// Oddiy klaviatura apostrofi: o' g' '. Eski saytlar va bazalar uchun eng mos.
    Ascii,
}

pub fn to_old(text: &str, style: OldStyle) -> String {
    let (og, tutuq) = match style {
        OldStyle::Official => ('\u{02BB}', '\u{02BC}'),
        OldStyle::Ascii => ('\'', '\''),
    };
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + 8);
    for (i, &c) in chars.iter().enumerate() {
        let next = chars.get(i + 1).copied();
        let upper_word = || {
            next.is_some_and(|n| n.is_uppercase())
                || (next.is_none_or(|n| !n.is_alphabetic()) && i > 0 && chars[i - 1].is_uppercase())
        };
        match c {
            'ö' => {
                out.push('o');
                out.push(og);
            }
            'Ö' => {
                out.push('O');
                out.push(og);
            }
            'ğ' => {
                out.push('g');
                out.push(og);
            }
            'Ğ' => {
                out.push('G');
                out.push(og);
            }
            'ş' => out.push_str("sh"),
            'ç' => out.push_str("ch"),
            'Ş' => out.push_str(if upper_word() { "SH" } else { "Sh" }),
            'Ç' => out.push_str(if upper_word() { "CH" } else { "Ch" }),
            // Yangi yozuvdagi alohida s + h (Ishoq) eski yozuvda tutuq bilan ajratiladi: Isʼhoq.
            's' | 'S' | 'c' | 'C' if matches!(next, Some('h' | 'H')) => {
                out.push(c);
                out.push(tutuq);
            }
            _ if is_apostrophe(c) => out.push(tutuq),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_and_ascii() {
        assert_eq!(to_old("Özbekiston", OldStyle::Ascii), "O'zbekiston");
        assert_eq!(
            to_old("Özbekiston", OldStyle::Official),
            "O\u{02BB}zbekiston"
        );
        assert_eq!(to_old("şahar va çoy", OldStyle::Ascii), "shahar va choy");
        assert_eq!(to_old("ŞAHAR", OldStyle::Ascii), "SHAHAR");
        assert_eq!(to_old("Şahnoza", OldStyle::Ascii), "Shahnoza");
        assert_eq!(to_old("ÇOY", OldStyle::Ascii), "CHOY");
        assert_eq!(to_old("Çoy", OldStyle::Ascii), "Choy");
        assert_eq!(to_old("Ishoq", OldStyle::Ascii), "Is'hoq");
        assert_eq!(to_old("ma\u{02BC}no", OldStyle::Ascii), "ma'no");
    }

    #[test]
    fn roundtrip() {
        use crate::latin::to_new;
        for w in [
            "O'zbekiston",
            "shahar",
            "choy",
            "tog'",
            "ma'no",
            "Is'hoq",
            "SHAHAR",
            "Shahnoza",
        ] {
            let new = to_new(w, '\'');
            assert_eq!(to_old(&new, OldStyle::Ascii), w, "{w} -> {new}");
        }
    }
}
