// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Foydalanuvchi lug'ati: `exceptions.txt` qatorlari (Windows, Linux, macOS bir xil fayl).
//!
//! - `shop` — so'z hech qachon o'girilmaydi (istisno);
//! - `shamollash=şamollaş` — so'z aynan shunday yoziladi, qo'shimchali shakli ham (`shamollashdan` →
//!   `şamollaşdan`). O'ng tomon — tayyor natija: `bog'cha=bog'ça` faqat ch ni o'giradi, g' qoladi.
//!
//! Chap tomon eski yozuvda, katta-kichik harf va apostrof turi farq qilmaydi (`chars::fold_key`).
//! Natijaning registri yozilgan so'zdan olinadi: `Shamollash` → `Şamollaş`, `SHAMOLLASH` → `ŞAMOLLAŞ`.

use crate::chars::fold_key;

/// `so'z=yozuv` qoidasi. `key` — chap tomonning `fold_key` ko'rinishi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub key: String,
    pub to: String,
}

/// Qoida chap tomoni kamida shuncha harf bo'lsa qo'shimchali shaklga ham qo'llanadi (`osh` → `oshxona` emas).
const STEM_MIN: usize = 4;

/// Fayl matni → (istisnolar, qoidalar). Izoh (`#`), bo'sh qator va BOM o'tkaziladi, takror — birinchisi.
pub fn parse(text: &str) -> (Vec<String>, Vec<Rule>) {
    let mut never: Vec<String> = Vec::new();
    let mut rules: Vec<Rule> = Vec::new();
    for line in text.lines() {
        let line = line.trim().trim_start_matches('\u{FEFF}');
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line.split_once('=') {
            Some((from, to)) => {
                let (from, to) = (from.trim(), to.trim());
                if from.is_empty() || to.is_empty() || from.contains(char::is_whitespace) {
                    continue;
                }
                let key = fold_key(from);
                if !rules.iter().any(|r| r.key == key) {
                    rules.push(Rule {
                        key,
                        to: to.to_string(),
                    });
                }
            }
            None => {
                let key = fold_key(line);
                if !never.contains(&key) {
                    never.push(key);
                }
            }
        }
    }
    (never, rules)
}

/// `core` uchun qoida natijasi (tinish belgilarisiz so'z). Aniq mos kelishi yoki o'zak + o'zbek qo'shimchasi;
/// qo'shimcha odatdagidek o'giriladi.
pub(crate) fn apply(rules: &[Rule], core: &str, tutuq: char) -> Option<String> {
    if rules.is_empty() {
        return None;
    }
    let key = fold_key(core);
    if let Some(r) = rules.iter().find(|r| r.key == key) {
        return Some(with_case(core, &r.to));
    }
    // Registr o'zgarishi belgilar sonini o'zgartirsa (İ) — faqat aniq moslik.
    if key.chars().count() != core.chars().count() {
        return None;
    }
    let r = rules
        .iter()
        .filter(|r| r.key.chars().count() >= STEM_MIN && key.starts_with(r.key.as_str()))
        .filter(|r| crate::lang::is_suffix(&key[r.key.len()..]))
        .max_by_key(|r| r.key.len())?;
    let n = r.key.chars().count();
    let split = core.char_indices().nth(n).map_or(core.len(), |(i, _)| i);
    let (stem, rest) = core.split_at(split);
    Some(format!(
        "{}{}",
        with_case(stem, &r.to),
        crate::latin::to_new(rest, tutuq)
    ))
}

/// Yozilgan so'zning registri: hammasi katta (kamida 2 harf) yoki bosh harf katta.
fn with_case(typed: &str, to: &str) -> String {
    let letters: Vec<char> = typed.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.len() >= 2 && letters.iter().all(|c| c.is_uppercase()) {
        return to.to_uppercase();
    }
    if letters.first().is_some_and(|c| c.is_uppercase()) {
        let mut it = to.chars();
        return match it.next() {
            Some(f) => f.to_uppercase().chain(it).collect(),
            None => String::new(),
        };
    }
    to.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(text: &str) -> Vec<Rule> {
        parse(text).1
    }

    #[test]
    fn parses_words_and_rules() {
        let (never, rules) = parse(
            "\u{FEFF}# izoh\nshop\n\nShamollash = şamollaş\nshop\nbad rule=x\n=x\nx=\nshamollash=boshqa\n",
        );
        assert_eq!(never, ["shop"]);
        assert_eq!(
            rules,
            [Rule {
                key: "shamollash".into(),
                to: "şamollaş".into()
            }]
        );
    }

    #[test]
    fn exact_stem_and_case() {
        let r = rules("shamollash=şamollaş\nbog'cha=bog'ça\nosh=osh");
        let t = crate::chars::DEFAULT_TUTUQ;
        assert_eq!(apply(&r, "shamollash", t).as_deref(), Some("şamollaş"));
        assert_eq!(apply(&r, "Shamollash", t).as_deref(), Some("Şamollaş"));
        assert_eq!(apply(&r, "SHAMOLLASH", t).as_deref(), Some("ŞAMOLLAŞ"));
        assert_eq!(
            apply(&r, "shamollashdan", t).as_deref(),
            Some("şamollaşdan")
        );
        assert_eq!(apply(&r, "bog‘cha", t).as_deref(), Some("bog'ça"));
        assert_eq!(apply(&r, "bog'chalarda", t).as_deref(), Some("bog'çalarda"));
        // Qisqa o'zak qo'shimcha olmaydi, qo'shimcha bo'lmagan davom ham.
        assert_eq!(apply(&r, "oshxona", t), None);
        assert_eq!(apply(&r, "shamollashxyz", t), None);
        assert_eq!(apply(&[], "shamollash", t), None);
        assert_eq!(apply(&r, "İshamollash", t), None);
    }

    #[test]
    fn case_helper_edges() {
        assert_eq!(with_case("A", "ö"), "Ö");
        assert_eq!(with_case("", "x"), "x");
        assert_eq!(with_case("Ab", ""), "");
    }
}
