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

/// Maxfiy maydon turlari (Windows `InputScope` qiymatlari): parol, PIN va email aynan yozilganicha qoladi (#26).
pub const SENSITIVE_INPUT_SCOPES: &[i32] = &[
    4,  // IS_EMAIL_USERNAME
    5,  // IS_EMAIL_SMTPEMAILADDRESS
    6,  // IS_LOGINNAME
    31, // IS_PASSWORD
    60, // IS_EMAILNAME_OR_ADDRESS
    63, // IS_NUMERIC_PASSWORD
    64, // IS_NUMERIC_PIN
    65, // IS_ALPHANUMERIC_PIN
    66, // IS_ALPHANUMERIC_PIN_SET
];

/// Maydon turi: o'qildi (maxfiy / oddiy) yoki o'qib bo'lmadi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Sensitive,
    Plain,
    Unknown,
}

/// Zaxira yo'l (#41) so'zni qayta yozadimi. `transitory` - IMM32/CUAS konteksti (Qt, Win32 EDIT); `imc` - fokusdagi
/// oynada IME konteksti bor (Qt va Win32 parol yoki matn bo'lmagan maydonda uni uzadi).
pub fn blind_may_convert(scope: Scope, transitory: bool, imc: bool) -> bool {
    match scope {
        Scope::Sensitive => false,
        Scope::Plain => !transitory || imc,
        Scope::Unknown => transitory && imc,
    }
}

/// `IS_PRIVATE`: maydon o'giriladi, lekin undagi so'zlar istisno sifatida faylga yozilmaydi.
pub const PRIVATE_INPUT_SCOPE: i32 = 61;

/// Maydonning input scope ro'yxatida maxfiy tur bormi.
pub fn sensitive_input_scope(scopes: &[i32]) -> bool {
    scopes.iter().any(|s| SENSITIVE_INPUT_SCOPES.contains(s))
}

/// Scope ro'yxatidan maydon turi: bo'sh ro'yxat hech narsa demaydi.
pub fn classify(scopes: &[i32]) -> Scope {
    if scopes.is_empty() {
        Scope::Unknown
    } else if sensitive_input_scope(scopes) {
        Scope::Sensitive
    } else {
        Scope::Plain
    }
}

/// Maydon `IS_PRIVATE` deb belgilanganmi.
pub fn private_input_scope(scopes: &[i32]) -> bool {
    scopes.contains(&PRIVATE_INPUT_SCOPE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_scopes() {
        assert!(sensitive_input_scope(&[31]));
        assert!(sensitive_input_scope(&[0, 64]));
        assert!(sensitive_input_scope(&[5]));
        assert!(!sensitive_input_scope(&[]));
        assert!(!sensitive_input_scope(&[0, 1, 50, 57, 61]));
        assert!(sensitive_input_scope(&[6]));
        assert!(sensitive_input_scope(&[0, 60]));
    }

    #[test]
    fn classify_and_private() {
        assert_eq!(classify(&[]), Scope::Unknown);
        assert_eq!(classify(&[0]), Scope::Plain);
        assert_eq!(classify(&[61]), Scope::Plain);
        assert_eq!(classify(&[1, 31]), Scope::Sensitive);
        assert_eq!(classify(&[6]), Scope::Sensitive);
        assert!(private_input_scope(&[0, 61]));
        assert!(!private_input_scope(&[0, 31]));
        assert!(!private_input_scope(&[]));
    }

    #[test]
    fn blind_decision() {
        use Scope::*;
        assert!(blind_may_convert(Unknown, true, true));
        assert!(!blind_may_convert(Unknown, true, false));
        assert!(!blind_may_convert(Unknown, false, true));
        assert!(!blind_may_convert(Sensitive, true, true));
        assert!(blind_may_convert(Plain, false, false));
        assert!(!blind_may_convert(Plain, true, false));
    }

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
    fn whole_buffer_is_the_word_when_start_reached() {
        let t = word_before_caret(&u("choy"), true).unwrap();
        assert_eq!((t.trailing, t.word_len, t.word.as_str()), (0, 4, "choy"));
        let t = word_before_caret(&u("choy\u{A0}"), true).unwrap();
        assert_eq!((t.trailing, t.word.as_str()), (1, "choy"));
        assert_eq!(word_before_caret(&[0x20, 0xD800], true), None);
    }

    #[test]
    fn utf16_lengths() {
        let t = word_before_caret(&u("a 😀choy"), true).unwrap();
        assert_eq!(t.word_len, 6); // surrogat juftlik = 2 birlik
    }
}
