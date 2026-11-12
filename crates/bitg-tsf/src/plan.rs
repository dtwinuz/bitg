// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Kursor oldidagi matndan qaysi so'zni almashtirishni hisoblash (platformaga bog'liq emas — test qilinadi).
//! Uzunliklar UTF-16 birliklarida: TSF `ITfRange::ShiftStart` shunday sanaydi.

/// Almashtirish rejasi: kursordan orqaga `trailing` ta bo'shliq (asinxron holatda probel allaqachon yozilgan),
/// undan oldin `word_len` ta birlikli so'z `word`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub trailing: usize,
    pub word_len: usize,
    pub word: String,
}

/// `before` — kursor oldidagi matn (UTF-16). So'z yo'q yoki juda uzun (bufer boshiga yetgan) bo'lsa — None.
pub fn word_before_caret(before: &[u16], reached_start: bool) -> Option<Target> {
    let is_space = |u: u16| matches!(u, 0x20 | 0x09 | 0x0A | 0x0D | 0xA0 | 0x2009 | 0x202F);
    let mut end = before.len();
    while end > 0 && is_space(before[end - 1]) {
        end -= 1;
    }
    let trailing = before.len() - end;
    if trailing > 2 {
        return None; // bir nechta bo'sh qator/probel — so'z hozirgina tugamagan
    }
    let mut start = end;
    while start > 0 && !is_space(before[start - 1]) {
        start -= 1;
    }
    if start == end {
        return None;
    }
    // Bufer boshiga yetdik, lekin matn undan oldin ham davom etishi mumkin — so'z chala bo'lishi mumkin.
    if start == 0 && !reached_start {
        return None;
    }
    let word = String::from_utf16(&before[start..end]).ok()?;
    Some(Target {
        trailing,
        word_len: end - start,
        word,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn sync_case_no_space_yet() {
        let t = word_before_caret(&u("salom choy"), true).unwrap();
        assert_eq!((t.trailing, t.word_len, t.word.as_str()), (0, 4, "choy"));
    }

    #[test]
    fn async_case_space_already_typed() {
        let t = word_before_caret(&u("salom choy,"), true).unwrap();
        assert_eq!(t.word, "choy,");
        let t = word_before_caret(&u("salom choy "), true).unwrap();
        assert_eq!((t.trailing, t.word.as_str()), (1, "choy"));
    }

    #[test]
    fn nothing_to_do() {
        assert_eq!(word_before_caret(&u(""), true), None);
        assert_eq!(word_before_caret(&u("   "), true), None);
        assert_eq!(word_before_caret(&u("choy   "), true), None);
        // 64 belgilik bufer to'lgan, so'z boshi ko'rinmaydi
        assert_eq!(word_before_caret(&u("averyveryverylongword"), false), None);
    }

    #[test]
    fn utf16_lengths() {
        let t = word_before_caret(&u("a 😀choy"), true).unwrap();
        assert_eq!(t.word_len, 6); // surrogat juftlik = 2 birlik
    }
}
