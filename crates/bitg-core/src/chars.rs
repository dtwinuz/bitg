// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Belgilar: tutuq belgisining barcha ko'rinishlari, yangi harflar va kichik normalizatsiya.

/// Tutuq / o' / g' uchun ishlatiladigan barcha belgilar. Klaviatura, Word avtotuzatishi va saytlar
/// har xilini qo'yadi — hammasi bitta belgi deb qaraladi.
pub const APOSTROPHES: &[char] = &[
    '\'',       // U+0027 oddiy apostrof
    '`',        // U+0060 grave
    '\u{00B4}', // ´ acute
    '\u{02B9}', // ʹ modifier prime
    '\u{02BB}', // ʻ turned comma (eski rasmiy o' / g')
    '\u{02BC}', // ʼ modifier apostrophe (tutuq)
    '\u{02BD}', // ʽ reversed comma
    '\u{2018}', // ‘
    '\u{2019}', // ’
    '\u{201B}', // ‛
    '\u{2032}', // ′ prime
    '\u{FF07}', // ＇ fullwidth
];

/// Yangi alifbodagi tutuq belgisi uchun standart: U+02BC (MODIFIER LETTER APOSTROPHE) — harf sifatida
/// qaraladi, so'z ichida qidiruv va tanlash buzilmaydi. Rasmiy matn tasdiqlansa sozlamadan o'zgartiriladi.
pub const DEFAULT_TUTUQ: char = '\u{02BC}';

#[inline]
pub fn is_apostrophe(c: char) -> bool {
    APOSTROPHES.contains(&c)
}

/// Yangi alifboning o'ziga xos harflari (ikkala registr).
#[inline]
pub fn is_new_letter(c: char) -> bool {
    matches!(c, 'ö' | 'Ö' | 'ğ' | 'Ğ' | 'ş' | 'Ş' | 'ç' | 'Ç')
}

/// Kirill harfimi (asosiy blok + o'zbekcha ў қ ғ ҳ).
#[inline]
pub fn is_cyrillic(c: char) -> bool {
    matches!(c, '\u{0400}'..='\u{04FF}')
}

/// Kombinatsiyalanuvchi belgilarni tayyor harfga yig'adi (o + ̈ → ö, s + ̧ → ş) va rumincha ș ni
/// turkiy ş ga keltiradi. To'liq NFC emas — faqat bizga kerakli juftliklar; boshqa matn o'zgarmaydi.
pub fn normalize(text: &str) -> std::borrow::Cow<'_, str> {
    let needs = text.chars().any(|c| {
        matches!(
            c,
            '\u{0308}' | '\u{0306}' | '\u{0327}' | '\u{0219}' | '\u{0218}'
        )
    });
    if !needs {
        return std::borrow::Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        let combined = match (c, it.peek().copied()) {
            ('o', Some('\u{0308}')) => Some('ö'),
            ('O', Some('\u{0308}')) => Some('Ö'),
            ('g', Some('\u{0306}')) => Some('ğ'),
            ('G', Some('\u{0306}')) => Some('Ğ'),
            ('s', Some('\u{0327}')) => Some('ş'),
            ('S', Some('\u{0327}')) => Some('Ş'),
            ('c', Some('\u{0327}')) => Some('ç'),
            ('C', Some('\u{0327}')) => Some('Ç'),
            _ => None,
        };
        if let Some(ch) = combined {
            it.next();
            out.push(ch);
            continue;
        }
        out.push(match c {
            '\u{0219}' => 'ş', // ș (vergulli) — ko'pincha ş o'rniga xato yoziladi
            '\u{0218}' => 'Ş',
            other => other,
        });
    }
    std::borrow::Cow::Owned(out)
}

/// Kalit uchun: kichik harf va barcha tutuq variantlari `'` ga.
pub fn fold_key(word: &str) -> String {
    let mut s = String::with_capacity(word.len());
    for c in word.chars() {
        if is_apostrophe(c) {
            s.push('\'');
        } else {
            for l in c.to_lowercase() {
                s.push(l);
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combining_marks_are_composed() {
        assert_eq!(normalize("ko\u{0308}z"), "köz");
        assert_eq!(normalize("tog\u{0306}"), "toğ");
        assert_eq!(normalize("s\u{0327}ahar"), "şahar");
        assert_eq!(normalize("\u{0219}i"), "şi");
        assert!(matches!(normalize("choy"), std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn fold_key_unifies_apostrophes() {
        assert_eq!(fold_key("O\u{2018}zbek"), "o'zbek");
        assert_eq!(fold_key("Ma\u{02BC}no"), "ma'no");
    }
}
