// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Asl va o'girilgan matnni belgi darajasida moslash: qaysi bo'lak nimaga almashgani.
//!
//! O'girish bo'shliqlarni aynan saqlaydi, shuning uchun matn so'zlarga bo'linib juftlanadi va faqat
//! o'zgargan so'z ichida eng uzun umumiy qism (LCS) qidiriladi — "o'" → "ö", "sh" → "ş" alohida bo'lak.

/// Asl matndagi `start..end` belgilar (char indeks) o'rniga `insert` yoziladi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hunk {
    pub start: usize,
    pub end: usize,
    pub insert: String,
}

/// So'z ichidagi LCS jadvali shu o'lchamdan oshsa, butun so'z bitta bo'lak hisoblanadi.
const LCS_LIMIT: usize = 1 << 16;

/// Bo'shliq va bo'shliqsiz qismlarga bo'lish: `(boshlanish char indeksi, qism)`.
fn segments(s: &str) -> Vec<(usize, &str)> {
    let mut v = Vec::new();
    let mut start_byte = 0;
    let mut start_char = 0;
    let mut prev: Option<bool> = None;
    for (n, (i, c)) in s.char_indices().enumerate() {
        let ws = c.is_whitespace();
        if prev.is_some_and(|p| p != ws) {
            v.push((start_char, &s[start_byte..i]));
            start_byte = i;
            start_char = n;
        }
        prev = Some(ws);
    }
    if start_byte < s.len() {
        v.push((start_char, &s[start_byte..]));
    }
    v
}

/// So'zma-so'z juftlar; tuzilish mos kelmasa — butun matn bitta juft.
fn pairs<'a>(old: &'a str, new: &'a str) -> Vec<(usize, &'a str, &'a str)> {
    let (a, b) = (segments(old), segments(new));
    let same_shape = a.len() == b.len()
        && a.iter().zip(&b).all(|((_, x), (_, y))| {
            let ws = x.starts_with(char::is_whitespace);
            ws == y.starts_with(char::is_whitespace) && (!ws || x == y)
        });
    if same_shape {
        a.into_iter()
            .zip(b)
            .filter(|((_, x), (_, y))| x != y)
            .map(|((at, x), (_, y))| (at, x, y))
            .collect()
    } else {
        vec![(0, old, new)]
    }
}

/// Bitta so'z ichidagi o'zgargan bo'laklar (indekslar so'z boshidan).
fn word_hunks(a: &[char], b: &[char]) -> Vec<Hunk> {
    let pre = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let suf = a[pre..]
        .iter()
        .rev()
        .zip(b[pre..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (ma, mb) = (&a[pre..a.len() - suf], &b[pre..b.len() - suf]);
    if ma.is_empty() && mb.is_empty() {
        return Vec::new();
    }
    if (ma.len() + 1) * (mb.len() + 1) > LCS_LIMIT || ma.is_empty() || mb.is_empty() {
        return vec![Hunk {
            start: pre,
            end: pre + ma.len(),
            insert: mb.iter().collect(),
        }];
    }
    let w = mb.len() + 1;
    let mut dp = vec![0u32; (ma.len() + 1) * w];
    for i in (0..ma.len()).rev() {
        for j in (0..mb.len()).rev() {
            dp[i * w + j] = if ma[i] == mb[j] {
                dp[(i + 1) * w + j + 1] + 1
            } else {
                dp[(i + 1) * w + j].max(dp[i * w + j + 1])
            };
        }
    }
    let mut out: Vec<Hunk> = Vec::new();
    let (mut i, mut j) = (0, 0);
    let mut open: Option<Hunk> = None;
    while i < ma.len() || j < mb.len() {
        if i < ma.len() && j < mb.len() && ma[i] == mb[j] {
            out.extend(open.take());
            i += 1;
            j += 1;
            continue;
        }
        let h = open.get_or_insert_with(|| Hunk {
            start: pre + i,
            end: pre + i,
            insert: String::new(),
        });
        if j == mb.len() || (i < ma.len() && dp[(i + 1) * w + j] >= dp[i * w + j + 1]) {
            h.end += 1;
            i += 1;
        } else {
            h.insert.push(mb[j]);
            j += 1;
        }
    }
    out.extend(open);
    out
}

/// `old` → `new` o'zgarishlari, asl matn bo'yicha tartiblangan va ustma-ust tushmaydigan bo'laklar.
pub fn hunks(old: &str, new: &str) -> Vec<Hunk> {
    let mut out = Vec::new();
    for (at, x, y) in pairs(old, new) {
        let (a, b): (Vec<char>, Vec<char>) = (x.chars().collect(), y.chars().collect());
        out.extend(word_hunks(&a, &b).into_iter().map(|h| Hunk {
            start: h.start + at,
            end: h.end + at,
            insert: h.insert,
        }));
    }
    out
}

/// O'zgargan so'zlar: `(eski, yangi)`, chetdagi bir xil tinish belgilari olib tashlangan.
pub fn changed_words(old: &str, new: &str) -> Vec<(String, String)> {
    pairs(old, new)
        .into_iter()
        .map(|(_, x, y)| {
            let (a, b): (Vec<char>, Vec<char>) = (x.chars().collect(), y.chars().collect());
            let edge = |p: (&char, &char)| p.0 == p.1 && !p.0.is_alphanumeric();
            let pre = a.iter().zip(&b).take_while(|&p| edge(p)).count();
            let suf = a[pre..]
                .iter()
                .rev()
                .zip(b[pre..].iter().rev())
                .take_while(|&p| edge(p))
                .count();
            (
                a[pre..a.len() - suf].iter().collect(),
                b[pre..b.len() - suf].iter().collect(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(old: &str, hs: &[Hunk]) -> String {
        let a: Vec<char> = old.chars().collect();
        let mut out = String::new();
        let mut i = 0;
        for h in hs {
            out.extend(&a[i..h.start]);
            out.push_str(&h.insert);
            i = h.end;
        }
        out.extend(&a[i..]);
        out
    }

    #[test]
    fn letter_level_hunks() {
        let hs = hunks("O'zbekiston shoshilinch", "Özbekiston şoşilinç");
        assert_eq!(
            hs.iter()
                .map(|h| (h.start, h.end, h.insert.as_str()))
                .collect::<Vec<_>>(),
            [(0, 2, "Ö"), (12, 14, "ş"), (15, 17, "ş"), (21, 23, "ç")]
        );
        assert_eq!(apply("O'zbekiston shoshilinch", &hs), "Özbekiston şoşilinç");
    }

    #[test]
    fn shape_mismatch_still_applies() {
        let (old, new) = ("a b", "ab c");
        assert_eq!(apply(old, &hunks(old, new)), new);
    }

    fn h(start: usize, end: usize, insert: &str) -> Hunk {
        Hunk {
            start,
            end,
            insert: insert.to_string(),
        }
    }

    #[test]
    fn empty_and_equal() {
        assert!(hunks("", "").is_empty());
        assert!(changed_words("", "").is_empty());
        assert!(word_hunks(&['a', 'b'], &['a', 'b']).is_empty());
    }

    #[test]
    fn deletion_and_different_segment_count() {
        assert_eq!(hunks("a b", "ab"), [h(1, 2, "")]);
        assert_eq!(hunks("abc", "b"), [h(0, 1, ""), h(2, 3, "")]);
    }

    #[test]
    fn whitespace_moved_is_one_pair() {
        assert_eq!(
            changed_words("a ", " a"),
            [("a ".to_string(), " a".to_string())]
        );
        assert_eq!(apply("a ", &hunks("a ", " a")), " a");
    }

    #[test]
    fn long_word_is_one_hunk() {
        let (a, b) = ("x".repeat(300), "y".repeat(300));
        assert_eq!(hunks(&a, &b), [h(0, 300, &b)]);
    }

    #[test]
    fn words_trim_punctuation() {
        assert_eq!(
            changed_words("(o'zbek), bog'", "(özbek), boğ"),
            [
                ("o'zbek".to_string(), "özbek".to_string()),
                ("bog'".to_string(), "boğ".to_string())
            ]
        );
    }
}
