// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 2026 alifbosi → eski lotin. O'tish davrida (2027–2031) yangi harflarni qabul qilmaydigan saytlar,
//! bazalar va hujjatlar uchun.
//!
//! Matn so'zma-so'z o'tadi: havola, email, @user, #teg, kod va `backtick` oralig'i tegilmaydi; yangi harfi
//! yo'q inglizcha so'z, brend va xorijiy nom (Check, Michelle) ham. s|h ajratuvchisi (Is'hoq) faqat kirillda
//! ham alohida yoziladigan so'zga qo'yiladi; apostrof faqat harflar orasida tutuqqa aylanadi, backtick hech
//! qachon.

use crate::chars::{DEFAULT_TUTUQ, is_apostrophe, is_new_letter};
use crate::{lang, token};

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
    let mut out = String::with_capacity(text.len() + 8);
    let mut rest = text;
    while let Some((before, code, after)) = next_code_span(rest) {
        words_to_old(before, style, &mut out);
        out.push_str(code);
        rest = after;
    }
    words_to_old(rest, style, &mut out);
    out
}

/// Birinchi `kod` oralig'i: (oldingi matn, backtick'lari bilan oraliq, keyingi matn). Ochuvchi backtick
/// harfdan keyin emas, yopuvchisidan keyin harf yo'q — so'z ichidagi backtick (ma`no) oraliq ochmaydi.
fn next_code_span(s: &str) -> Option<(&str, &str, &str)> {
    let mut from = 0;
    while let Some(off) = s[from..].find('`') {
        let open = from + off;
        from = open + 1;
        if s[..open]
            .chars()
            .next_back()
            .is_some_and(char::is_alphabetic)
        {
            continue;
        }
        let mut k = open + 1;
        while let Some(off) = s[k..].find('`') {
            let close = k + off;
            k = close + 1;
            let letter_after = s[k..].chars().next().is_some_and(char::is_alphabetic);
            if close > open + 1 && !letter_after {
                return Some((&s[..open], &s[open..k], &s[k..]));
            }
        }
        return None;
    }
    None
}

fn words_to_old(seg: &str, style: OldStyle, out: &mut String) {
    let mut start = None;
    for (i, c) in seg.char_indices() {
        if c.is_whitespace() {
            if let Some(s) = start.take() {
                word_to_old(&seg[s..i], style, out);
            }
            out.push(c);
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        word_to_old(&seg[s..], style, out);
    }
}

/// Havola, email, @user, #teg yoki kod belgisi bor token — yangi harf bo'lsa ham tegilmaydi.
fn is_link_or_code(tok: &str) -> bool {
    let lower = tok.to_lowercase();
    lower.contains("://")
        || lower.contains("www.")
        || tok.contains(['@', '#'])
        || ["::", "->", "=>", "()", "</", "/>"]
            .iter()
            .any(|s| tok.contains(s))
}

fn word_to_old(tok: &str, style: OldStyle, out: &mut String) {
    let has_new = tok.chars().any(is_new_letter);
    if is_link_or_code(tok) || !has_new && token::is_protected(tok) {
        out.push_str(tok);
        return;
    }
    let p = token::split_edges(tok);
    let core = p.core;
    // Yangi harfsiz so'z: o'zgarishi mumkin bo'lgan narsa faqat s|h ajratuvchisi va apostrof. Inglizcha so'z
    // (Check, don't), brend va xorijiy nom (Michelle Chicago) o'z yozuvida qoladi.
    if !has_new
        && (lang::is_brand(core)
            || lang::name_kind(core) == lang::NameKind::Foreign
            || lang::evidence(core) < 0.0)
    {
        out.push_str(tok);
        return;
    }
    let plain = letters_to_old(core, style, Mode::Word { split: false });
    let split = lang::is_sh_split(&plain) || crate::is_dis_h(&plain);
    out.push_str(p.lead);
    out.push_str(&letters_to_old(core, style, Mode::Word { split }));
    // dekabrʼ: so'z oxiridagi U+02BC — tirnoq emas, tutuq harfi.
    let trail = match p.trail.strip_prefix(DEFAULT_TUTUQ) {
        Some(rest) if core.chars().next_back().is_some_and(char::is_alphabetic) => {
            out.push(tutuq_of(style));
            rest
        }
        _ => p.trail,
    };
    out.push_str(trail);
}

fn tutuq_of(style: OldStyle) -> char {
    match style {
        OldStyle::Official => '\u{02BC}',
        OldStyle::Ascii => '\'',
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Mode {
    /// Harfma-harf: har s+h ga ajratuvchi, har apostrof tutuq. Faqat lug'atga kalit uchun (`to_cyr`).
    Raw,
    /// Bitta so'z o'zagi: ajratuvchi faqat `split` bo'lsa, apostrof faqat harflar orasida, backtick emas.
    Word { split: bool },
}

pub(crate) fn letters_to_old(text: &str, style: OldStyle, mode: Mode) -> String {
    let (og, tutuq) = match style {
        OldStyle::Official => ('\u{02BB}', '\u{02BC}'),
        OldStyle::Ascii => ('\'', '\''),
    };
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + 8);
    for (i, &c) in chars.iter().enumerate() {
        let next = chars.get(i + 1).copied();
        let prev = i.checked_sub(1).map(|p| chars[p]);
        let upper_word = || {
            next.is_some_and(|n| n.is_uppercase())
                || (next.is_none_or(|n| !n.is_alphabetic()) && prev.is_some_and(char::is_uppercase))
        };
        let separate = match mode {
            Mode::Raw => true,
            Mode::Word { split } => split,
        };
        let tutuq_here = match mode {
            Mode::Raw => true,
            Mode::Word { .. } => {
                c != '`'
                    && prev.is_some_and(char::is_alphabetic)
                    && next.is_some_and(char::is_alphabetic)
            }
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
            's' | 'S' | 'c' | 'C' if separate && matches!(next, Some('h' | 'H')) => {
                out.push(c);
                out.push(tutuq);
            }
            _ if tutuq_here && is_apostrophe(c) => out.push(tutuq),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(s: &str) -> String {
        to_old(s, OldStyle::Ascii)
    }

    #[test]
    fn official_and_ascii() {
        assert_eq!(a("Özbekiston"), "O'zbekiston");
        assert_eq!(
            to_old("Özbekiston", OldStyle::Official),
            "O\u{02BB}zbekiston"
        );
        assert_eq!(a("şahar va çoy"), "shahar va choy");
        assert_eq!(a("ŞAHAR"), "SHAHAR");
        assert_eq!(a("Şahnoza"), "Shahnoza");
        assert_eq!(a("ÇOY"), "CHOY");
        assert_eq!(a("Çoy"), "Choy");
        assert_eq!(a("Ishoq"), "Is'hoq");
        assert_eq!(a("ma\u{02BC}no"), "ma'no");
        assert_eq!(a("dekabr\u{02BC},"), "dekabr',");
        assert_eq!(a("Ş.Şaripov"), "Sh.Sharipov");
        assert_eq!(
            to_old("ma'no", OldStyle::Official),
            "ma\u{02BC}no",
            "harflar orasidagi apostrof tutuq"
        );
    }

    #[test]
    fn protected_and_foreign_tokens_are_kept() {
        for s in [
            "https://shop.uz/choy?ch=sh",
            "shahzod@mail.uz",
            "@shahzod",
            "#shahar",
            "`check()`",
            "`git push` va",
            "Michelle Chicago Washington",
            "Check the shop",
            "don't",
            "'Yaxshi'",
            "rock 'n' roll",
            "Photoshop",
            "https://şahar.uz",
            "şahar()",
        ] {
            assert_eq!(a(s), s, "{s}");
            assert_eq!(to_old(s, OldStyle::Official), s, "{s}");
        }
        assert_eq!(a("`şahar` va şahar"), "`şahar` va shahar");
        assert_eq!(
            a("ma`no `kod"),
            "ma`no `kod",
            "backtick hech qachon tutuq emas"
        );
        assert_eq!(a("``"), "``");
        assert_eq!(a("`a`b şahar"), "`a`b shahar");
    }

    #[test]
    fn raw_mode_is_letter_by_letter() {
        assert_eq!(
            letters_to_old("shop' SH ch CH", OldStyle::Ascii, Mode::Raw),
            "s'hop' S'H c'h C'H"
        );
        assert_eq!(
            a("ISHOQ Ğalaba www.şahar.uz"),
            "IS'HOQ G'alaba www.şahar.uz"
        );
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
