// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! O'zbek kirill → 2026 lotin. Kirillda sh/ch va tutuq bo'yicha noaniqlik yo'q (ш ≠ сҳ), shuning uchun
//! bu yo'nalish eng aniq: lug'atsiz, faqat kontekst qoidalari (е, ц, ъ).

use crate::chars::is_cyrillic;

fn is_cyr_vowel(c: char) -> bool {
    matches!(
        c.to_lowercase().next().unwrap_or(c),
        'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ў' | 'э' | 'ю' | 'я' | 'ы'
    )
}

fn single(c: char) -> Option<&'static str> {
    Some(match c {
        'а' => "a",
        'б' => "b",
        'в' => "v",
        'г' => "g",
        'д' => "d",
        'ж' => "j",
        'з' => "z",
        'и' => "i",
        'й' => "y",
        'к' => "k",
        'л' => "l",
        'м' => "m",
        'н' => "n",
        'о' => "o",
        'п' => "p",
        'р' => "r",
        'с' => "s",
        'т' => "t",
        'у' => "u",
        'ф' => "f",
        'х' => "x",
        'ч' => "ç",
        'ш' => "ş",
        'щ' => "şç",
        'ы' => "i",
        'э' => "e",
        'ё' => "yo",
        'ю' => "yu",
        'я' => "ya",
        'ў' => "ö",
        'қ' => "q",
        'ғ' => "ğ",
        'ҳ' => "h",
        'ь' => "",
        _ => return None,
    })
}

/// Matndagi kirill harflarni yangi lotinga o'giradi; boshqa belgilar o'zgarmaydi.
pub fn to_new_latin(text: &str, tutuq: char) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + text.len() / 4);
    for (i, &c) in chars.iter().enumerate() {
        if !is_cyrillic(c) {
            out.push(c);
            continue;
        }
        let lower = c.to_lowercase().next().unwrap_or(c);
        let upper = c != lower;
        let prev = if i > 0 { Some(chars[i - 1]) } else { None };
        let word_start = !prev.is_some_and(|p| p.is_alphabetic());

        let mapped: String = match lower {
            'е' => {
                let after = prev.map(|p| p.to_lowercase().next().unwrap_or(p));
                if word_start || after.is_some_and(|p| is_cyr_vowel(p) || p == 'ъ' || p == 'ь') {
                    "ye".into()
                } else {
                    "e".into()
                }
            }
            'ц' => {
                if prev.is_some_and(is_cyr_vowel) {
                    "ts".into()
                } else {
                    "s".into()
                }
            }
            'ъ' => tutuq.to_string(),
            other => match single(other) {
                Some(s) => s.into(),
                None => {
                    out.push(c); // noma'lum kirill belgi (ukraincha va h.k.) — o'zgarmaydi
                    continue;
                }
            },
        };

        if !upper {
            out.push_str(&mapped);
            continue;
        }
        // Katta harf: butun so'z katta bo'lsa — YE, aks holda Ye.
        if mapped.chars().count() > 1 && !word_is_upper(&chars, i) {
            let mut it = mapped.chars();
            if let Some(f) = it.next() {
                out.extend(f.to_uppercase());
                out.push_str(it.as_str());
            }
        } else {
            out.push_str(&mapped.to_uppercase());
        }
    }
    out
}

/// i-belgi turgan so'zda (kamida 2 harf) hamma harflar kattami.
fn word_is_upper(chars: &[char], i: usize) -> bool {
    let mut s = i;
    while s > 0 && chars[s - 1].is_alphabetic() {
        s -= 1;
    }
    let mut e = i;
    while e + 1 < chars.len() && chars[e + 1].is_alphabetic() {
        e += 1;
    }
    let letters: Vec<char> = chars[s..=e]
        .iter()
        .copied()
        .filter(|c| c.is_alphabetic())
        .collect();
    letters.len() > 1 && letters.iter().all(|c| !c.is_lowercase())
}

/// Matnda kirill harf bormi.
pub fn has_cyrillic(text: &str) -> bool {
    text.chars().any(is_cyrillic)
}

#[cfg(test)]
mod tests {
    use super::*;
    const T: char = '\u{02BC}';

    #[test]
    fn basic() {
        assert_eq!(to_new_latin("Ўзбекистон", T), "Özbekiston");
        assert_eq!(to_new_latin("шаҳар", T), "şahar");
        assert_eq!(to_new_latin("чой", T), "çoy");
        assert_eq!(to_new_latin("тоғ", T), "toğ");
        assert_eq!(to_new_latin("қишлоқ", T), "qişloq");
        assert_eq!(to_new_latin("Исҳоқ", T), "Ishoq");
    }

    #[test]
    fn context_rules() {
        assert_eq!(to_new_latin("ер", T), "yer");
        assert_eq!(to_new_latin("Ер", T), "Yer");
        assert_eq!(to_new_latin("поезд", T), "poyezd");
        assert_eq!(to_new_latin("келди", T), "keldi");
        assert_eq!(to_new_latin("цирк", T), "sirk");
        assert_eq!(to_new_latin("лицей", T), "litsey");
        assert_eq!(to_new_latin("маъно", T), "ma\u{02BC}no");
        assert_eq!(to_new_latin("ЯНГИ", T), "YANGI");
        assert_eq!(to_new_latin("Янги", T), "Yangi");
        assert_eq!(to_new_latin("ШАҲАР", T), "ŞAHAR");
    }

    #[test]
    fn upper_multi_letter_inside_word() {
        assert_eq!(to_new_latin("ПОЕЗД", T), "POYEZD");
        assert_eq!(to_new_latin("(ЯНГИ)", T), "(YANGI)");
        assert_eq!(to_new_latin("Я", T), "Ya");
    }

    #[test]
    fn mixed_text_keeps_latin() {
        assert_eq!(to_new_latin("Салом, World! 2026", T), "Salom, World! 2026");
    }
}
