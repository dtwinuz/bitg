//! Lotin (2026 yoki eski: o' g' sh ch ') → o'zbek kirill.
//!
//! Avval matn oldinga yo'nalishdagi tahlildan o'tadi (`convert_token`): eski yozuv yangi harflarga
//! keltiriladi, s'h / Ishoq ajratiladi, havola, email, kod, brend va inglizcha so'zlar aniqlanadi —
//! ular lotinda qoladi. Keyin yangi lotin harfma-harf kirillga o'tadi:
//!
//! - ö→ў, ğ→ғ, ş→ш, ç→ч, q→қ, x→х, h→ҳ, j→ж; ng → нг; tutuq ʼ → ъ; alohida s+h (Ishoq) → сҳ.
//! - ya/yo/yu → я/ё/ю, ye → е; yö (yöl) → йў.
//! - e: so'z boshida, unli yoki tutuqdan keyin → э (Erkin, aeroport), boshqa joyda → е.
//! - Undoshdan keyin so'z ichida ye → ъе (obyekt → объект, subyekt → субъект).
//! - ts → ц faqat ruscha o'zlashmalarning ishonchli qo'shimchalarida: -tsiya, -tsion, -tsey, -tsent,
//!   -tsept (politsiya, litsey, protsent, retsept). O'zbekcha t+s (aytsa, ketsin) o'zgarmaydi.
//!
//! Lotinda tiklab bo'lmaydigan farqlar: ь (aprel → апрел), undoshdan keyingi ц (sirk → сирк), щ, ы,
//! ruscha "йо" (mayor → маёр). Bunday so'zlar kirill → lotin → kirill aylanishida farq qiladi.

use crate::chars::{is_apostrophe, normalize};
use crate::{Options, cyrillic, lang, reverse, token};

/// Lotin matnni o'zbek kirilliga o'giradi. Kirill harfli, inglizcha va himoyalangan tokenlar o'zgarmaydi.
pub fn to_cyrillic(text: &str) -> String {
    let opts = Options {
        cyrillic: false,
        ..Options::default()
    };
    let text = normalize(text);
    let text = text.as_ref();
    let mut toks = crate::tokens(text);
    for t in &mut toks {
        let core = token::split_edges(&text[t.start..t.end]).core;
        // Lug'at eski yozuvda: yangi harfli so'z (özbek) ham o'zbekcha deb tanilsin. Eski yozuvdagi
        // so'zga to_old qo'llanmaydi — u "ch" ni alohida c+h deb c'h qiladi.
        t.evidence = if core.chars().any(crate::chars::is_new_letter) {
            lang::evidence(&reverse::to_old(core, reverse::OldStyle::Ascii))
        } else {
            lang::evidence(core)
        };
    }
    let mut out = String::with_capacity(text.len() * 2);
    let mut last = 0;
    let mut sent_start = 0;
    let mut quote_open = false;
    for i in 0..toks.len() {
        if i > 0 && toks[i - 1].sentence_end {
            sent_start = i;
        }
        let mut sent_end = i;
        while sent_end + 1 < toks.len() && !toks[sent_end].sentence_end {
            sent_end += 1;
        }
        let t = &toks[i];
        let ws = &text[last..t.start];
        out.push_str(ws);
        if ws.contains('\n') {
            quote_open = false;
        }
        let ctx = crate::context(&toks, sent_start..sent_end + 1, i - sent_start);
        let tok = &text[t.start..t.end];
        out.push_str(&token_to_cyr(tok, &opts, t.evidence, ctx, &mut quote_open));
        last = t.end;
    }
    out.push_str(&text[last..]);
    out
}

fn token_to_cyr(tok: &str, opts: &Options, own: f32, ctx: f32, quote_open: &mut bool) -> String {
    if cyrillic::has_cyrillic(tok) {
        return tok.to_string();
    }
    let p0 = token::split_edges(tok);
    // Raqam + qo'shimcha: 20ga, 23:30da, 43chi, 1-2-dekabr, 16a-uy. Oldinga yo'nalishda bular parolga
    // o'xshash deb himoyalanadi; kirillda qo'shimcha ham kirillda yoziladi.
    if let Some(split) = number_prefix(p0.core) {
        let (num, rest) = p0.core.split_at(split);
        let rest = token_to_cyr(rest, opts, lang::evidence(rest), ctx, &mut false);
        return format!("{}{num}{rest}{}", p0.lead, p0.trail);
    }
    let latin = if tok.chars().any(crate::chars::is_new_letter) {
        // Yangi yozuv: o'girish kerak emas, faqat himoyani tekshiramiz. ÖzA (O'zA) camelCase emas.
        if token::is_protected(&reverse::to_old(tok, reverse::OldStyle::Ascii)) {
            return tok.to_string();
        }
        tok.to_string()
    } else {
        let (l, reason) =
            crate::convert_token(tok, opts, own, ctx, crate::Hints::default(), quote_open);
        if matches!(
            reason,
            crate::Reason::Protected | crate::Reason::English | crate::Reason::Exception
        ) {
            return tok.to_string();
        }
        l
    };
    let mut p = token::split_edges(&latin);
    if !p.core.chars().any(char::is_alphabetic) {
        return tok.to_string();
    }
    // dekabrʼ: oxiridagi U+02BC — tirnoq emas, tutuq harfi (kirillda ъ).
    let tutuq_end = p.trail.starts_with(crate::chars::DEFAULT_TUTUQ);
    if tutuq_end {
        p.trail = &p.trail[crate::chars::DEFAULT_TUTUQ.len_utf8()..];
    }
    // Belgisiz so'z (hello, world): o'zbekcha belgisi yo'q — inglizcha dalilga qaraymiz.
    if !tutuq_end && !has_uz_marker(p.core) && english_plain(p.core, own, ctx) {
        return tok.to_string();
    }
    let mut s = String::with_capacity(latin.len() * 2);
    s.push_str(p.lead);
    for (i, part) in p.core.split('-').enumerate() {
        if i > 0 {
            s.push('-');
        }
        if keep_latin(part) {
            s.push_str(part);
        } else {
            s.push_str(&word_to_cyr(part));
        }
    }
    if tutuq_end {
        let upper = p.core.chars().last().is_some_and(char::is_uppercase);
        s.push(if upper { 'Ъ' } else { 'ъ' });
    }
    s.push_str(p.trail);
    s
}

/// Boshidagi raqamli qism (raqam, `.` `:` `,` `-`) va undan keyin faqat harf/tutuq/chiziqcha bo'lsa —
/// harf boshlanadigan bayt o'rni.
fn number_prefix(core: &str) -> Option<usize> {
    let split = core.find(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | ':' | ',' | '-')))?;
    let (num, rest) = core.split_at(split);
    let ok = num.starts_with(|c: char| c.is_ascii_digit())
        && rest.starts_with(char::is_alphabetic)
        && rest
            .chars()
            .all(|c| c.is_alphabetic() || c == '-' || is_apostrophe(c));
    ok.then_some(split)
}

/// Yangi alifbo harfi yoki tutuq — so'z aniq o'zbekcha.
fn has_uz_marker(core: &str) -> bool {
    core.chars()
        .any(|c| crate::chars::is_new_letter(c) || is_apostrophe(c))
}

/// Oldinga yo'nalishdagi sh/ch qoidasi bilan bir xil: kuchli inglizcha va o'zbek lug'atida yo'q,
/// yoki o'zi + kontekst inglizcha tomonda.
fn english_plain(core: &str, own: f32, ctx: f32) -> bool {
    if own <= crate::STRONG_ENGLISH && !lang::known(core).0 {
        return true;
    }
    own + crate::CONTEXT_WEIGHT * ctx < 0.0
}

/// O'zbek lotinida yo'q harf (c, w, ü, é ...) yoki unlisiz qisqartma (TSH) — lotinda qoladi.
fn keep_latin(part: &str) -> bool {
    part.chars().any(|c| matches!(c, 'c' | 'C' | 'w' | 'W'))
        || crate::is_acronym(part)
        || crate::has_foreign_letters(part)
}

fn is_lat_vowel(c: char) -> bool {
    matches!(
        c.to_lowercase().next().unwrap_or(c),
        'a' | 'e' | 'i' | 'o' | 'u' | 'ö'
    )
}

/// Ruscha o'zlashmadagi "ts" → ц: t+s dan keyin shu qismlardan biri kelsa.
const TS_TAILS: &[&str] = &["iya", "ion", "ey", "ent", "ept"];

fn ts_is_tse(lower: &[char], i: usize) -> bool {
    lower.get(i) == Some(&'t')
        && lower.get(i + 1) == Some(&'s')
        && TS_TAILS.iter().any(|tail| {
            let tail: Vec<char> = tail.chars().collect();
            lower.get(i + 2..i + 2 + tail.len()) == Some(&tail[..])
        })
}

fn cased(upper: bool, c: char) -> char {
    if upper {
        c.to_uppercase().next().unwrap_or(c)
    } else {
        c
    }
}

/// Bitta yangi lotin so'z (tinish belgilarisiz, chiziqchasiz) → kirill.
fn word_to_cyr(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let lower: Vec<char> = chars
        .iter()
        .map(|c| c.to_lowercase().next().unwrap_or(*c))
        .collect();
    let mut out = String::with_capacity(word.len() * 2);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let l = lower[i];
        let up = c != l;
        let prev = if i > 0 { Some(chars[i - 1]) } else { None };
        let word_start = !prev.is_some_and(|p| p.is_alphabetic() || is_apostrophe(p));
        let next = lower.get(i + 1).copied();

        if l == 'y' {
            let y = match next {
                Some('a') => Some('я'),
                Some('o') => Some('ё'),
                Some('u') => Some('ю'),
                Some('e') => Some('е'),
                _ => None,
            };
            if let Some(y) = y {
                // obyekt, subyekt: undoshdan keyingi ye — ajratish belgisi bilan.
                if y == 'е'
                    && prev
                        .is_some_and(|p| p.is_alphabetic() && !is_apostrophe(p) && !is_lat_vowel(p))
                {
                    out.push(cased(up && chars[i + 1].is_uppercase(), 'ъ'));
                }
                out.push(cased(up, y));
                i += 2;
                continue;
            }
        }
        if ts_is_tse(&lower, i) {
            out.push(cased(up, 'ц'));
            i += 2;
            continue;
        }
        let m = match l {
            'a' => 'а',
            'b' => 'б',
            'd' => 'д',
            'e' => {
                if word_start || prev.is_some_and(|p| is_lat_vowel(p) || is_apostrophe(p)) {
                    'э'
                } else {
                    'е'
                }
            }
            'f' => 'ф',
            'g' => 'г',
            'h' => 'ҳ',
            'i' => 'и',
            'j' => 'ж',
            'k' => 'к',
            'l' => 'л',
            'm' => 'м',
            'n' => 'н',
            'o' => 'о',
            'p' => 'п',
            'q' => 'қ',
            'r' => 'р',
            's' => 'с',
            't' => 'т',
            'u' => 'у',
            'v' => 'в',
            'x' => 'х',
            'y' => 'й',
            'z' => 'з',
            'ö' => 'ў',
            'ğ' => 'ғ',
            'ş' => 'ш',
            'ç' => 'ч',
            _ if is_apostrophe(c) => {
                // Katta harfli so'zda (MA'NO) ham ъ: kichik/kattasini qo'shnisiga qarab.
                let upper_word = prev.is_some_and(char::is_uppercase)
                    && chars.get(i + 1).is_none_or(|n| !n.is_lowercase());
                out.push(if upper_word { 'Ъ' } else { 'ъ' });
                i += 1;
                continue;
            }
            _ => {
                out.push(c);
                i += 1;
                continue;
            }
        };
        out.push(cased(up, m));
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters() {
        assert_eq!(word_to_cyr("Özbekiston"), "Ўзбекистон");
        assert_eq!(word_to_cyr("şahar"), "шаҳар");
        assert_eq!(word_to_cyr("Ishoq"), "Исҳоқ");
        assert_eq!(word_to_cyr("maʼno"), "маъно");
        assert_eq!(word_to_cyr("MAʼNO"), "МАЪНО");
        assert_eq!(word_to_cyr("yöl"), "йўл");
        assert_eq!(word_to_cyr("obyekt"), "объект");
    }

    #[test]
    fn ts_is_conservative() {
        assert_eq!(word_to_cyr("politsiya"), "полиция");
        assert_eq!(word_to_cyr("litsey"), "лицей");
        assert_eq!(word_to_cyr("protsent"), "процент");
        assert_eq!(word_to_cyr("aytsa"), "айтса");
        assert_eq!(word_to_cyr("ketsin"), "кетсин");
    }
}
