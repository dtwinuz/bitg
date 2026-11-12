// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Eski lotin → 2026 alifbosi: sof qoidalar (lug'at va til tekshiruvisiz).
//!
//! O'RQ-1184 (2026-10-21): Oʻ → Ö, Gʻ → Ğ, Sh → Ş, Ch → Ç; tutuq belgisi qoladi. Ng o'zgarmaydi.
//! s va h alohida o'qilsa eski yozuvda tutuq bilan ajratilgan (Is'hoq) — yangi yozuvda ş alohida harf
//! bo'lgani uchun ajratuvchi kerak emas: Is'hoq → Ishoq.

use crate::chars::is_apostrophe;

/// Bitta so'zni (chetidagi tinish belgilarisiz) o'giradi. `tutuq` — chiqishdagi tutuq belgisi.
pub fn to_new(word: &str, tutuq: char) -> String {
    convert_word(word, tutuq, false)
}

/// `to_new`, lekin tutuqsiz "sh"/"ch" ham alohida harflar deb qoladi (Ishoq, Mushaf — kirillda сҳ).
pub fn to_new_split(word: &str, tutuq: char) -> String {
    convert_word(word, tutuq, true)
}

fn convert_word(word: &str, tutuq: char, split: bool) -> String {
    let chars: Vec<char> = word.chars().collect();
    let mut out = String::with_capacity(word.len());
    let n = chars.len();
    let mut i = 0;
    while i < n {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        let next2 = chars.get(i + 2).copied();
        match c {
            'o' | 'O' | 'g' | 'G' if next.is_some_and(is_apostrophe) => {
                out.push(match c {
                    'o' => 'ö',
                    'O' => 'Ö',
                    'g' => 'ğ',
                    _ => 'Ğ',
                });
                i += 2;
            }
            's' | 'S' | 'c' | 'C' if !split && matches!(next, Some('h' | 'H')) => {
                out.push(match c {
                    's' => 'ş',
                    'S' => 'Ş',
                    'c' => 'ç',
                    _ => 'Ç',
                });
                i += 2;
            }
            // s'h, c'h: tutuq faqat "bu sh emas" deb ajratgan — yangi yozuvda kerak emas.
            's' | 'S' | 'c' | 'C'
                if next.is_some_and(is_apostrophe) && matches!(next2, Some('h' | 'H')) =>
            {
                out.push(c);
                out.push(next2.unwrap_or('h'));
                i += 3;
            }
            _ if is_apostrophe(c) => {
                out.push(tutuq);
                i += 1;
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// So'zda o'girishga arziydigan narsa bormi (sh, ch, o', g' yoki tutuq).
pub fn has_convertible(word: &str) -> bool {
    let chars: Vec<char> = word.chars().collect();
    chars.windows(2).any(|w| {
        let a = w[0].to_ascii_lowercase();
        let b = w[1];
        ((a == 's' || a == 'c') && (b == 'h' || b == 'H'))
            || ((a == 'o' || a == 'g') && is_apostrophe(b))
    }) || chars.iter().any(|&c| is_apostrophe(c))
}

/// So'zdagi o'zgarish sh/ch (inglizcha bilan chalkashishi mumkin) bilangina bo'lyaptimi.
/// o'/g' va tutuq aniq o'zbekcha belgi — ular inglizchada deyarli uchramaydi.
pub fn digraph_only(word: &str) -> bool {
    let chars: Vec<char> = word.chars().collect();
    let has_digraph = chars.windows(2).any(|w| {
        let a = w[0].to_ascii_lowercase();
        (a == 's' || a == 'c') && (w[1] == 'h' || w[1] == 'H')
    });
    has_digraph && !chars.iter().any(|&c| is_apostrophe(c))
}

#[cfg(test)]
mod tests {
    use super::*;
    const T: char = '\u{02BC}';

    #[test]
    fn letters() {
        assert_eq!(to_new("o'zbek", T), "özbek");
        assert_eq!(to_new("O\u{2018}zbekiston", T), "Özbekiston");
        assert_eq!(to_new("tog'", T), "toğ");
        assert_eq!(to_new("shahar", T), "şahar");
        assert_eq!(to_new("choy", T), "çoy");
        assert_eq!(to_new("Shahnoza", T), "Şahnoza");
        assert_eq!(to_new("SHAHAR", T), "ŞAHAR");
        assert_eq!(to_new("O'ZBEKISTON", T), "ÖZBEKISTON");
        assert_eq!(to_new("ming", T), "ming");
    }

    #[test]
    fn tutuq_and_separator() {
        assert_eq!(to_new("ma'no", T), "ma\u{02BC}no");
        assert_eq!(to_new("san`at", T), "san\u{02BC}at");
        assert_eq!(to_new("Is'hoq", T), "Ishoq");
        assert_eq!(to_new("mus'haf", T), "mushaf");
        assert_eq!(to_new("ma'no", '\u{2019}'), "ma\u{2019}no");
    }

    #[test]
    fn already_new_is_stable() {
        for w in ["özbek", "şahar", "çoy", "toğ", "ma\u{02BC}no"] {
            assert_eq!(to_new(w, T), w);
        }
    }

    #[test]
    fn detection() {
        assert!(has_convertible("choy"));
        assert!(has_convertible("o'g'il"));
        assert!(!has_convertible("salom"));
        assert!(digraph_only("check"));
        assert!(!digraph_only("o'shanda"));
    }
}
