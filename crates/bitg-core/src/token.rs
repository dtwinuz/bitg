// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! So'zni chetidagi tinish belgilaridan ajratish va tegilmaydigan tokenlarni (havola, email, kod) aniqlash.

use crate::chars::is_apostrophe;

/// Gap ichida so'z chetida turadigan oddiy tinish belgilari. Apostroflar alohida ko'riladi.
/// `¸` (sedil) — ba'zi saytlarda vergul o'rnida keladi ("bo'lsa¸ keyin").
const EDGE: &[char] = &[
    '(', ')', '[', ']', '{', '}', '"', '«', '»', '“', '”', '„', ',', '.', ';', ':', '!', '?', '…',
    '—', '–', '*', '_', '~', '¸',
];

/// Tokenning qismlari: boshidagi va oxiridagi tinish belgilari va o'zak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parts<'a> {
    pub lead: &'a str,
    pub core: &'a str,
    pub trail: &'a str,
}

/// Tokenni ajratadi. Boshidagi apostrof — har doim tirnoq (o'zbek so'zi tutuq bilan boshlanmaydi).
/// Oxiridagi apostrof o/g dan keyin bo'lsa — harf (tog'), aks holda tirnoq.
pub fn split_edges(token: &str) -> Parts<'_> {
    let mut start = 0;
    for (i, c) in token.char_indices() {
        if EDGE.contains(&c) || is_apostrophe(c) {
            start = i + c.len_utf8();
        } else {
            break;
        }
    }
    let rest = &token[start..];
    let mut end = rest.len();
    let chars: Vec<(usize, char)> = rest.char_indices().collect();
    let mut k = chars.len();
    while k > 0 {
        let (i, c) = chars[k - 1];
        let keep_apostrophe =
            is_apostrophe(c) && k >= 2 && matches!(chars[k - 2].1, 'o' | 'O' | 'g' | 'G');
        if EDGE.contains(&c) || (is_apostrophe(c) && !keep_apostrophe) {
            end = i;
            k -= 1;
        } else {
            break;
        }
    }
    Parts {
        lead: &token[..start],
        core: &rest[..end],
        trail: &rest[end..],
    }
}

const TLDS: &[&str] = &[
    "uz", "com", "org", "net", "ru", "me", "io", "info", "biz", "edu", "gov", "tv", "app", "dev",
    "ai", "co", "kz", "tj", "kg", "tm", "tr", "uk", "us", "de", "be", "ly", "xyz", "site",
    "online", "pro", "su",
];

/// Bu token umuman o'zgartirilmasin: havola, email, @user, #teg, domen, fayl yo'li, kod, parolga o'xshash.
pub fn is_protected(token: &str) -> bool {
    let p = split_edges(token);
    let core = p.core;
    if core.is_empty() {
        return false;
    }
    let lower = core.to_ascii_lowercase();
    if lower.contains("://") || lower.starts_with("www.") {
        return true;
    }
    if core.starts_with('@')
        || core.starts_with('#')
        || token.starts_with('@')
        || token.starts_with('#')
    {
        return true;
    }
    if core.contains('@') {
        return true; // email yoki @ bilan kod
    }
    if ["::", "->", "=>", "()", "</", "/>"]
        .iter()
        .any(|s| token.contains(s))
    {
        return true;
    }
    if is_initials(core) {
        return false; // A.Lukashenko, Sh.Sharipov, M.Jo'rayev
    }
    let chars: Vec<char> = core.chars().collect();
    // O'zA, O'zR, O'zLiDeP: o'/g' bor — kod emas, o'zbekcha qisqartma.
    let uz_marked = chars
        .windows(2)
        .any(|w| matches!(w[0], 'o' | 'O' | 'g' | 'G') && is_apostrophe(w[1]));
    for (i, &c) in chars.iter().enumerate() {
        if "_=(){}[]<>$\\*&|^~#/%+".contains(c) {
            return true;
        }
        // Harflar orasidagi nuqta yoki ikki nuqta: shahar.uz, obj.show, a:b
        if (c == '.' || c == ':')
            && i > 0
            && i + 1 < chars.len()
            && chars[i - 1].is_alphanumeric()
            && chars[i + 1].is_alphanumeric()
        {
            return true;
        }
        // camelCase / iPhone: kichik harfdan keyin katta harf.
        if !uz_marked && c.is_uppercase() && i > 0 && chars[i - 1].is_lowercase() {
            return true;
        }
    }
    // Domen oxirgi nuqtasiz bo'lsa ham: "kun.uz" yuqorida ushlangan; "gmail.com." ham.
    if let Some((_, tld)) = lower.rsplit_once('.')
        && TLDS.contains(&tld)
        && lower.len() > tld.len() + 1
    {
        return true;
    }
    // Raqam va harf aralash (shahar123, a1b2) — parol, kod, model nomi. "5-sinf", "2026-yil" — emas.
    let has_digit = chars.iter().any(|c| c.is_ascii_digit());
    let has_letter = chars.iter().any(|c| c.is_alphabetic());
    if has_digit && has_letter && !is_number_with_suffix(core) {
        return true;
    }
    // O'Brien, O'Neil, D'Artagnan: apostrofdan keyin katta harf (so'z hammasi katta bo'lmasa).
    let all_upper = chars
        .iter()
        .filter(|c| c.is_alphabetic())
        .all(|c| c.is_uppercase());
    !all_upper
        && chars
            .windows(2)
            .any(|w| is_apostrophe(w[0]) && w[1].is_uppercase())
}

/// Ism bosh harflari + familiya: "A.Lukashenko", "Sh.Sharipov", "O'.Turmatov", "A.B.Karimov".
fn is_initials(core: &str) -> bool {
    let mut parts: Vec<&str> = core.split('.').collect();
    let Some(last) = parts.pop() else {
        return false;
    };
    if parts.is_empty() {
        return false;
    }
    let initial_ok = |p: &str| {
        let letters = p.chars().filter(|c| c.is_alphabetic()).count();
        (1..=2).contains(&letters)
            && p.chars().next().is_some_and(|c| c.is_uppercase())
            && p.chars().all(|c| c.is_alphabetic() || is_apostrophe(c))
    };
    let surname_ok = last.chars().count() >= 2
        && last.chars().next().is_some_and(|c| c.is_uppercase())
        && last
            .chars()
            .all(|c| c.is_alphabetic() || is_apostrophe(c) || c == '-')
        && !TLDS.contains(&last.to_lowercase().as_str());
    parts.iter().all(|p| initial_ok(p)) && surname_ok
}

/// "5-sinf", "2026-yilgi", "3-chi": raqam, chiziqcha, so'ng faqat harflar.
fn is_number_with_suffix(core: &str) -> bool {
    match core.split_once('-') {
        Some((num, rest)) => {
            !num.is_empty()
                && num.chars().all(|c| c.is_ascii_digit())
                && !rest.is_empty()
                && rest.chars().all(|c| c.is_alphabetic() || is_apostrophe(c))
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges() {
        assert_eq!(split_edges("(shahar),").core, "shahar");
        assert_eq!(split_edges("tog'").core, "tog'");
        assert_eq!(split_edges("tog',").core, "tog'");
        assert_eq!(split_edges("'Yaxshi").core, "Yaxshi");
        assert_eq!(split_edges("'Yaxshi").lead, "'");
        assert_eq!(split_edges("choy'").core, "choy");
        assert_eq!(split_edges("«Toshkent»").core, "Toshkent");
        assert_eq!(split_edges("...").core, "");
    }

    #[test]
    fn protected() {
        for t in [
            "https://kun.uz/choy",
            "www.shop.com",
            "shahar.uz",
            "kun.uz/news",
            "user@gmail.com",
            "@shahzod",
            "#shahar",
            "push()",
            "std::cout",
            "my_shop",
            "showChart",
            "iPhone",
            "shahar123",
            "C:\\chess",
            "O'Brien",
            "a/b",
        ] {
            assert!(is_protected(t), "{t}");
        }
        for t in [
            "shahar",
            "O'zbekiston",
            "O'ZBEKISTON",
            "5-sinf",
            "2026-yil",
            "ma'no",
            "choy,",
            "(shahar)",
        ] {
            assert!(!is_protected(t), "{t}");
        }
    }

    #[test]
    fn protected_domain_after_apostrophe_and_number_suffix() {
        assert!(is_protected("bog'.uz"));
        assert!(!is_protected("bog'.xx"));
        assert!(is_protected("5-sinf2"));
        assert!(!is_protected("3-o'rin"));
    }
}
