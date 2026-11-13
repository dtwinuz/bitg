// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! So'z o'zbekchami yoki inglizchami — dalillar yig'indisi (musbat = o'zbekcha, manfiy = inglizcha).
//!
//! Manbalar: o'zbek korpusi chastotasi (Leipzig Corpora, CC BY), inglizcha so'zlar (SCOWL, ruxsat beruvchi
//! litsenziya), o'zbek qo'shimchalari bo'yicha o'zak qidirish va harf belgilari. Lug'atlar
//! `tools/build_lexicon.py` bilan quriladi va binarga kiritiladi.

use std::sync::OnceLock;

use crate::chars::fold_key;
use crate::lexicon::{EN, Lexicon, SPLIT, UZ};

// Lug'atlar qurish vaqtida tayyorlangan jadval (`build.rs`): yuklash va xotira ajratish yo'q.
fn uz() -> &'static Lexicon {
    &UZ
}

fn en() -> &'static Lexicon {
    &EN
}

fn split() -> &'static Lexicon {
    &SPLIT
}

/// Kirillda s|h alohida bo'lgan so'z (Исҳоқ → Ishoq): yozuvdagi "sh" ş ga aylanmasligi kerak.
/// Qo'shimchali shakl ham (nushalari, Ishoqqa): o'zak kamida 5 harf.
pub fn is_sh_split(word: &str) -> bool {
    const SUFFIXES: &[&str] = &[
        "laridan", "larning", "lari", "ning", "dagi", "dan", "lar", "ga", "qa", "da", "ni", "si",
        "i",
    ];
    let key = fold_key(word);
    split().contains_key(key.as_str())
        || SUFFIXES.iter().any(|s| {
            key.strip_suffix(s)
                .is_some_and(|stem| stem.chars().count() >= 5 && split().contains_key(stem))
        })
}

static NAMES_TXT: &str = include_str!("data/names.txt");

fn names() -> &'static std::collections::HashSet<&'static str> {
    static S: OnceLock<std::collections::HashSet<&'static str>> = OnceLock::new();
    S.get_or_init(|| {
        NAMES_TXT
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    })
}

/// Brend yoki mahsulot nomi (`data/names.txt`), o'zbek qo'shimchasi yoki apostrofli qo'shimcha bilan ham:
/// Photoshop, Photoshopda, Chevrolet'ni.
pub fn is_brand(word: &str) -> bool {
    let lower = word.to_lowercase();
    let base = lower
        .split(|c: char| crate::chars::is_apostrophe(c))
        .next()
        .unwrap_or(&lower);
    let set = names();
    if set.contains(base) {
        return true;
    }
    const SUFFIXES: &[&str] = &[
        "larning", "larni", "larga", "larda", "lardan", "dagi", "ning", "dan", "lar", "dir", "da",
        "ga", "ka", "ni", "ni", "ning", "imiz", "ingiz", "im", "ing", "i",
    ];
    SUFFIXES.iter().any(|s| {
        base.strip_suffix(s)
            .is_some_and(|stem| stem.chars().count() >= 4 && set.contains(stem))
    })
}

static FOREIGN_NAMES_TXT: &str = include_str!("data/foreign_names.txt");

fn foreign_names() -> &'static std::collections::HashSet<&'static str> {
    static S: OnceLock<std::collections::HashSet<&'static str>> = OnceLock::new();
    S.get_or_init(|| {
        FOREIGN_NAMES_TXT
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    })
}

/// Bosh harfli so'z ism sifatida: xorijiy imloda (Michelle, Chicago), ikki xil o'qiladigan (Richard, Chaplin —
/// o'zbek korpusida ham, inglizchada ham bor) yoki bog'lovchi ("va", "and").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NameKind {
    #[default]
    Other,
    /// Faqat xorijiy imloda uchraydi — o'girilmaydi.
    Foreign,
    /// O'zbek imlosida ham shunday yoziladi (Richard → Riçard), lekin yonida xorijiy nom bo'lsa o'shaning bir qismi:
    /// "Charlie Chaplin", "Richard va Michelle".
    Maybe,
    Link,
}

/// Inglizcha imlo belgisi (o'zbek lotinida bo'lmaydi): yolg'iz c (Chicago), w, th, ph, ck, ee, oo,
/// qo'sh undosh + e bilan tugash (Michelle, Babette).
fn foreign_orthography(key: &str) -> bool {
    let b = key.as_bytes();
    let lone_c = b
        .iter()
        .enumerate()
        .any(|(i, &c)| c == b'c' && b.get(i + 1) != Some(&b'h'));
    lone_c
        || key.contains('w')
        || ["th", "ph", "ck", "ee", "oo"]
            .iter()
            .any(|p| key.contains(p))
        || ["lle", "tte", "nne", "sse", "ffe"]
            .iter()
            .any(|p| key.len() >= 5 && key.ends_with(p))
}

/// Nomga qo'shilgan o'zbek qo'shimchasi: Chicagoda, Michelleni, Sanchezning.
const NAME_SUFFIXES: &[&str] = &[
    "larning", "larni", "larga", "larda", "lardan", "dagi", "ning", "dan", "lar", "da", "ga", "ni",
    "ka", "dek", "ham",
];

/// Ism turi. Faqat bosh harfli, sh/ch li, o'zbek lug'atida yo'q so'z xorijiy bo'la oladi: o'zbek ismi va joy nomi
/// (Shohruh, Chorsu, Pushkin) korpusda bor va o'giriladi.
pub fn name_kind(word: &str) -> NameKind {
    let key = fold_key(word);
    if matches!(key.as_str(), "va" | "and" | "&") {
        return NameKind::Link;
    }
    if !word.starts_with(|c: char| c.is_uppercase()) {
        return NameKind::Other;
    }
    // Apostrofli qo'shimcha (Washington'ga) — asosi bo'yicha. Boshqa apostrof — o'zbekcha tutuq (Mash'al).
    let base = match key.split_once('\'') {
        None => key.as_str(),
        Some((b, suf)) if NAME_SUFFIXES.contains(&suf) => b,
        Some(_) => return NameKind::Other,
    };
    if !(base.contains("ch") || base.contains("sh")) {
        return NameKind::Other;
    }
    let foreign_stem = |s: &str| {
        uz_weight(s).is_none()
            && (foreign_names().contains(s) || en().contains_key(s) || foreign_orthography(s))
    };
    if foreign_stem(base)
        || NAME_SUFFIXES.iter().any(|suf| {
            base.strip_suffix(suf).is_some_and(|stem| {
                stem.len() >= 4 && uz_weight(base).is_none() && foreign_stem(stem)
            })
        })
    {
        return NameKind::Foreign;
    }
    if en().contains_key(base) {
        return NameKind::Maybe;
    }
    NameKind::Other
}

/// Lug'atlarni oldindan yuklash (IME ishga tushganda, birinchi so'z kutib qolmasin).
pub fn warm_up() {
    let _ = uz().len() + en().len() + split().len() + names().len() + foreign_names().len();
}

// Eski lotin yozuvidagi qo'shimchalar — uzunidan qisqasiga. Lug'atda yo'q so'zning o'zagini topish uchun.
const SUFFIXES: &[&str] = &[
    "lardagi",
    "laridan",
    "larning",
    "laridagi",
    "imizning",
    "ingizning",
    "ayotgan",
    "yotgan",
    "yapti",
    "moqda",
    "lardan",
    "larida",
    "larini",
    "larga",
    "larda",
    "lari",
    "ning",
    "ngiz",
    "imiz",
    "ingiz",
    "dagi",
    "gani",
    "ganlar",
    "gan",
    "kan",
    "qan",
    "lik",
    "liq",
    "chi",
    "chilik",
    "siz",
    "dan",
    "tan",
    "lar",
    "lab",
    "lash",
    "ni",
    "ga",
    "ka",
    "qa",
    "da",
    "ta",
    "di",
    "ib",
    "ish",
    "moq",
    "mas",
    "may",
    "sa",
    "sin",
    "dir",
    "man",
    "san",
    "miz",
    "dek",
    "day",
    "cha",
    "roq",
    "gina",
    "li",
    "mi",
    "ku",
    "im",
    "ing",
    "si",
    "i",
];

fn uz_weight(key: &str) -> Option<f32> {
    if let Some(w) = uz().get(key) {
        return Some(w);
    }
    // O'zak + qo'shimcha: kitob|larimizdan. Ikki qatlamgacha.
    let mut best: Option<f32> = None;
    for s1 in SUFFIXES {
        if let Some(stem) = key.strip_suffix(s1) {
            if stem.chars().count() < 3 {
                continue;
            }
            if let Some(w) = uz().get(stem) {
                best = Some(best.map_or(w * 0.8, |b: f32| b.max(w * 0.8)));
                continue;
            }
            for s2 in SUFFIXES {
                if let Some(stem2) = stem.strip_suffix(s2)
                    && stem2.chars().count() >= 3
                    && let Some(w) = uz().get(stem2)
                {
                    best = Some(best.map_or(w * 0.6, |b: f32| b.max(w * 0.6)));
                }
            }
        }
    }
    best
}

/// Harflarning o'zi aytadigan dalil (lug'atdan tashqari so'zlar uchun).
fn letter_cues(key: &str) -> f32 {
    let c: Vec<char> = key.chars().collect();
    let mut s = 0.0f32;
    for i in 0..c.len() {
        let a = c[i];
        let b = c.get(i + 1).copied();
        match a {
            'w' => s -= 3.0,
            'c' if b != Some('h') => s -= 3.0, // o'zbek lotinida c faqat ch ichida
            'q' if b != Some('u') => s += 2.5,
            'x' => s += 0.5,
            _ => {}
        }
        match (a, b) {
            ('t', Some('h')) | ('p', Some('h')) | ('c', Some('k')) => s -= 2.0,
            ('e', Some('e')) | ('o', Some('o')) => s -= 1.5,
            ('e', Some('a')) | ('o', Some('u')) | ('a', Some('w')) => s -= 1.0,
            ('o' | 'g', Some('\'')) => s += 3.0,
            _ => {}
        }
    }
    if key.ends_with("tion") || key.ends_with("ness") || key.ends_with("ment") {
        s -= 2.0;
    }
    s
}

/// Inglizcha qisqartma yoki egalik: don't, we're, you'll, they've, Obama's, CNN's, I'm, she'd
/// (`he'd` emas: `he` o'zbekcha lug'atda ham bor).
/// O'zbek so'zi tutuq + s / re / ll / ve bilan tugamaydi. 'm, 'd — faqat o'zagi inglizcha bo'lsa (ta'm o'zbekcha).
fn is_english_contraction(key: &str) -> bool {
    let always = ["n't", "'re", "'ve", "'ll", "'s"].iter().any(|s| {
        key.strip_suffix(s)
            .is_some_and(|stem| stem.chars().count() >= 1)
    });
    // "i" o'zbekchada ham bor (kirill «и»), lekin I'm / I'd — faqat inglizcha.
    always
        || key == "i'm"
        || key == "i'd"
        || ["'m", "'d"].iter().any(|s| {
            key.strip_suffix(s)
                .is_some_and(|stem| en().contains_key(stem) && uz_weight(stem).is_none())
        })
}

/// So'z bo'yicha dalil: >0 o'zbekcha, <0 inglizcha, 0 — bilmaymiz.
pub fn evidence(word: &str) -> f32 {
    let key = fold_key(word);
    if key.is_empty() {
        return 0.0;
    }
    if is_english_contraction(&key) {
        return -6.0;
    }
    let en_w = en().get(key.as_str());
    let uz_w = uz_weight(&key).map(|w| w.min(UZ_EVIDENCE_CAP));
    // Apostrofli inglizcha so'z (o'clock, ma'am, rock'n'roll) — o'/tutuq bo'lsa ham o'zbekcha emas.
    if en_w.is_some() && uz_w.is_none() && key.contains('\'') {
        return -6.0;
    }
    match (uz_w, en_w) {
        (Some(u), Some(e)) => u - e,
        (Some(u), None) => u + letter_cues(&key).min(0.0) * 0.5,
        (None, Some(e)) => -e,
        (None, None) => letter_cues(&key),
    }
}

/// Til dalilida o'zbek og'irligining shifti (lug'atda 6.0 gacha — chastota nisbatlari uchun).
const UZ_EVIDENCE_CAP: f32 = 4.5;

/// Ikkala shakli ham haqiqiy va tez-tez uchraydigan so'zlar: apostrofsiz yozilgan bo'lsa — taxmin qilinmaydi.
/// (qol "to'xta" / qo'l, oz / o'z, yoq / yo'q, otdi / o'tdi, tort / to'rt, som / so'm ...)
const NEVER_RESTORE: &[&str] = &[
    "qol", "qoling", "oz", "yoq", "on", "ot", "otdi", "otadi", "otib", "otgan", "otish", "otar",
    "otishi", "otganini", "otuvchi", "tort", "som", "bor", "tola", "toy", "top", "toq", "qoya",
    "otin", "oqi", "sol", "oshda", "bolaman", "bolamiz", "oldirdi",
];

/// Shu og'irlikdan yuqori so'z o'zi to'g'ri — apostrof qo'shilmaydi. Pastrog'i ko'pincha kirill korpusdagi
/// xato yozuv (кора, оша, огир, гарб) — uni nomzod bilan solishtiramiz.
const RESTORE_KNOWN: f32 = 3.3;
/// Nomzod asl so'zdan shuncha kuchli bo'lsin (log10 shkala: ~30 baravar ko'p uchraydi).
const RESTORE_MARGIN: f32 = 1.5;
/// Apostrofsiz shakl ham lug'atda bor, nomzod o'zak orqali taxmin: ~60 baravar ko'p uchrashi kerak
/// (1.5 va 1.65 da "oshganlar → o'shganlar" kabi xatolar chiqdi — o'zak "o'sh" fe'l qo'shimchasini olmaydi).
const RESTORE_MARGIN_BOTH: f32 = 1.8;
/// Ikkala shakl ham korpusda o'zi uchraydi (ogir/og'ir, garb/g'arb): ~16 baravar ko'p bo'lsa kifoya.
/// Ikkalasi ham haqiqiy so'z bo'lgan juftlar (toy/to'y, top/to'p) — `NEVER_RESTORE` da.
const RESTORE_MARGIN_DIRECT: f32 = 1.2;
/// Inglizcha so'z (song, bog, bug) faqat o'zbekcha gap ichida va nomzod keng tarqalgan bo'lsa tiklanadi.
const RESTORE_EN_MIN: f32 = 3.0;
/// Asl shakl korpusda o'zi uchramaydi — faqat o'zak orqali taxmin (organ|ib, soz|lar), nomzod esa o'zi uchraydi
/// (o'rganib, so'zlar): ~6 baravar ko'p uchrashi kifoya.
const RESTORE_MARGIN_STEM: f32 = 0.8;
/// O'zak orqali taxmin qilingan asl shakl shundan kuchli bo'lsa — tegilmaydi.
const RESTORE_KNOWN_STEM: f32 = 3.2;
/// Nomzodning eng kichik og'irligi (siyrak so'zlarga taxmin qilmaymiz).
const RESTORE_MIN: f32 = 2.0;
/// o/g o'rinlari ko'pi bilan (2^6 = 64 nomzod).
const RESTORE_MAX_SLOTS: usize = 6;

/// Tushib qolgan apostrofni tiklash: "ozbek" → "o'zbek", "Togri" → "To'g'ri", "ogil" → "o'g'il".
/// Asl so'z o'zi ma'lum bo'lsa (ozod, bor) — None. Inglizcha so'z (song) faqat `uz_context` da (gap aniq
/// o'zbekcha: "Bundan song keldim"). Harf registri saqlanadi.
///
/// `loose` — matn muallifi o'/g' apostrofini umuman qo'ymaydi (shu matnda boshqa so'z ham tiklandi): ikkala
/// shakli ham haqiqiy so'z bo'lsa (oz / o'z, yoq / yo'q) ham chastota nisbati hal qiladi.
pub fn restore_apostrophes(word: &str, uz_context: bool, loose: bool) -> Option<String> {
    restore_impl(word, uz_context, loose, false).map(|(r, _)| r)
}

/// Ikki ma'noli so'zning apostrofli muqobili: "ochmaydi" → "o'chmaydi", "otdim" → "o'tdim", "oz" → "o'z".
/// Konvertor bunday so'zni o'zi hal qilmaydi (yoki chastota bilan hal qiladi) — UI "tekshiring" deb ko'rsatadi.
pub fn alternative(word: &str) -> Option<String> {
    if word.contains('-') {
        return None;
    }
    restore_impl(word, false, true, true).and_then(|(r, ambiguous)| ambiguous.then_some(r))
}

/// Ikki ma'noli o'zak: ikkala o'zak ham keng tarqalgan so'z (och / o'ch, ot / o't) va chastotasi yaqin.
/// kor / ko'r (2.5), soz / so'z (2.44) emas: apostrofsiz o'zak — kirill korpusidagi imlo xatosi.
const AMBIG_ROOT_MIN: f32 = 2.9;
const AMBIG_ROOT_GAP: f32 = 1.2;
/// Apostrofsiz shakli korpusda yo'q ikki ma'noli so'z nomzodi shundan kam uchrasa (log10 shkala, ~100 marta)
/// tiklanmaydi. `eval/uz_noapos.tsv` bo'yicha: 3.0 → 2434, 3.2 → 2430, 3.4 → 2428, 3.6 → 2424, 4.0 → 2402.
const AMBIG_UNSEEN_MIN: f32 = 3.0;
/// Fe'l shaxs-son va zamon qo'shimchalari (`SUFFIXES` da yo'qlari): ot|dim, och|maydi, och|asan.
const VERB_ENDINGS: &[&str] = &[
    "dim", "ding", "dik", "dingiz", "dilar", "adi", "aman", "asan", "amiz", "asiz", "ar", "ay",
    "ma", "mang", "gin", "ganman", "ganmiz", "moqchi", "masdan",
];
/// Shaxs qo'shimchasi — faqat so'z oxirida: och|di|m, lekin o't|m|ish emas (o'tmish — ot).
const FINAL_ENDINGS: &[&str] = &["m", "ng", "k"];

/// `s` faqat qo'shimchalardan iborat (ko'pi bilan 3 qatlam): "maydi" = may|di, "kazdi" emas.
fn is_suffix_chain(s: &str, depth: u8) -> bool {
    s.is_empty()
        || FINAL_ENDINGS.contains(&s)
        || depth < 3
            && SUFFIXES.iter().chain(VERB_ENDINGS).any(|x| {
                s.strip_prefix(x)
                    .is_some_and(|rest| is_suffix_chain(rest, depth + 1))
            })
}

/// Bo'sh bo'lmagan o'zbek qo'shimchalari zanjiri (`dan`, `larda`, `imizning`): foydalanuvchi qoidasi o'zagi uchun.
pub(crate) fn is_suffix(s: &str) -> bool {
    !s.is_empty() && is_suffix_chain(s, 0)
}

/// `key` (apostrofsiz) va `cand` (apostrofli) bir xil qo'shimcha bilan ikki haqiqiy o'zakdan yasalgan:
/// och|maydi / o'ch|maydi. o't|kazdi esa yo'q — "kazdi" qo'shimcha emas.
fn ambiguous_roots(key: &str, cand: &str) -> bool {
    key.char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(key.len()))
        .filter(|&i| i >= 2)
        .any(|i| {
            let (r0, suf) = key.split_at(i);
            let Some(r1) = cand.strip_suffix(suf) else {
                return false;
            };
            r1 != r0
                && is_suffix_chain(suf, 0)
                && matches!(
                    (uz().get(r0), uz().get(r1)),
                    (Some(a), Some(b)) if a >= AMBIG_ROOT_MIN && b >= AMBIG_ROOT_MIN && (a - b).abs() <= AMBIG_ROOT_GAP
                )
        })
}

/// `probe` — `alternative` uchun: ikki ma'nolilik to'xtatmaydi, faqat belgilanadi.
/// Qaytadi: (tiklangan so'z, ikki ma'noli-mi).
fn restore_impl(word: &str, uz_context: bool, loose: bool, probe: bool) -> Option<(String, bool)> {
    // Chiziqchali so'z qism-qism: qollab-quvvatlash → qo'llab-quvvatlash, togridan-togri → to'g'ridan-to'g'ri.
    // Bir bo'lagida o'/g' yozilgan bo'lsa (ozmi-ko'pmi) — muallif apostrof qo'yadi, qolganiga tegilmaydi.
    if word.contains('-') {
        let k = fold_key(word);
        if k.contains("o'") || k.contains("g'") {
            return None;
        }
        let mut changed = false;
        let parts: Vec<String> = word
            .split('-')
            .map(|p| match restore_apostrophes(p, uz_context, loose) {
                Some(r) => {
                    changed = true;
                    r
                }
                None => p.to_string(),
            })
            .collect();
        return changed.then(|| (parts.join("-"), false));
    }
    let key = fold_key(word);
    let kc: Vec<char> = key.chars().collect();
    let wc: Vec<char> = word.chars().collect();
    let min_len = if loose { 2 } else { 3 };
    if kc.len() != wc.len() || kc.len() < min_len || !loose && NEVER_RESTORE.contains(&key.as_str())
    {
        return None;
    }
    let english = en().contains_key(key.as_str());
    if english && !uz_context {
        return None;
    }
    let w0 = uz_weight(&key).unwrap_or(0.0);
    let direct0 = uz().contains_key(key.as_str());
    if !loose
        && w0
            >= if direct0 {
                RESTORE_KNOWN
            } else {
                RESTORE_KNOWN_STEM
            }
    {
        return None;
    }
    let slots: Vec<usize> = (0..kc.len())
        .filter(|&i| matches!(kc[i], 'o' | 'g') && kc.get(i + 1) != Some(&'\''))
        .collect();
    if slots.is_empty() || slots.len() > RESTORE_MAX_SLOTS {
        return None;
    }
    let mut best: Option<(f32, u32, u32, bool)> = None; // (og'irlik, -qo'shimchalar soni, mask, o'zi uchraydimi)
    let mut cand = String::with_capacity(key.len() + slots.len());
    for mask in 1u32..(1 << slots.len()) {
        cand.clear();
        let mut si = 0;
        for (i, &c) in kc.iter().enumerate() {
            cand.push(c);
            if si < slots.len() && slots[si] == i {
                if mask & (1 << si) != 0 {
                    cand.push('\'');
                }
                si += 1;
            }
        }
        if let Some(w) = uz_weight(&cand) {
            let fewer = u32::MAX - mask.count_ones();
            let direct = uz().contains_key(cand.as_str());
            // Korpusda o'zi uchragan nomzod o'zak orqali taxmin qilinganidan ustun: ogirlab → o'g'irlab (o'zi
            // uchraydi), og'ir|lab emas.
            let solid = direct && w >= RESTORE_MIN;
            if best.is_none_or(|(bw, bf, _, bd)| {
                let bsolid = bd && bw >= RESTORE_MIN;
                (solid, w, fewer) > (bsolid, bw, bf)
            }) {
                best = Some((w, fewer, mask, direct));
            }
        }
    }
    let (w, _, mask, cand_direct) = best?;
    // Ikkala shakl ham haqiqiy so'z (qol/qo'l, otdi/o'tdi, yoq/yo'q) — juda katta farq bo'lsagina.
    let margin = if w0 <= 0.0 {
        RESTORE_MARGIN
    } else if !direct0 && cand_direct {
        RESTORE_MARGIN_STEM
    } else if direct0 && cand_direct {
        RESTORE_MARGIN_DIRECT
    } else {
        RESTORE_MARGIN_BOTH
    };
    let min = if english { RESTORE_EN_MIN } else { RESTORE_MIN };
    if w < min || w < w0 + margin {
        return None;
    }
    let cand_key = with_apostrophes(&kc, &slots, mask);
    let ambiguous = NEVER_RESTORE.contains(&key.as_str()) || ambiguous_roots(&key, &cand_key);
    // Ikki haqiqiy o'zakdan yasalgan, apostrofsiz shakli korpusda o'zi uchramaydi, nomzod ham siyrak (ochmaydi /
    // o'chmaydi 2.15, otdim / o'tdim 2.75): yo'qlik hech narsa demaydi — muallif yozganicha qoladi. Nomzod keng
    // tarqalgan bo'lsa (oqish / o'qish, boyicha / bo'yicha) yo'qlikning o'zi dalil; ikkalasi ham uchrasa (otgan /
    // o'tgan) yuqoridagi nisbat hal qiladi.
    if !probe
        && ambiguous
        && !NEVER_RESTORE.contains(&key.as_str())
        && !uz().get(&key).is_some_and(|w| w > 0.0)
        && w < AMBIG_UNSEEN_MIN
    {
        return None;
    }
    Some((with_apostrophes(&wc, &slots, mask), ambiguous))
}

fn with_apostrophes(chars: &[char], slots: &[usize], mask: u32) -> String {
    let mut out = String::with_capacity(chars.len() + slots.len());
    let mut si = 0;
    for (i, &c) in chars.iter().enumerate() {
        out.push(c);
        if si < slots.len() && slots[si] == i {
            if mask & (1 << si) != 0 {
                out.push('\'');
            }
            si += 1;
        }
    }
    out
}

/// Lug'atda bormi (test va CLI uchun).
pub fn known(word: &str) -> (bool, bool) {
    let key = fold_key(word);
    (uz_weight(&key).is_some(), en().contains_key(key.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_endings_and_contractions() {
        assert_eq!(letter_cues("zention"), -2.0);
        assert_eq!(letter_cues("zenness"), -2.0);
        assert_eq!(letter_cues("zenment"), -2.0);
        assert_eq!(evidence("I'd"), -6.0);
        assert!(is_english_contraction("she'd"));
        assert!(!is_english_contraction("ta'm"));
    }

    #[test]
    fn short_stems_are_not_looked_up() {
        assert_eq!(uz_weight("xqni"), None);
        assert_eq!(uz_weight("xqnini"), None);
    }

    #[test]
    fn suffix_chain_depth_is_limited() {
        assert!(is_suffix_chain("dandandan", 0));
        assert!(!is_suffix_chain("dandandandan", 0));
    }

    #[test]
    fn hyphen_and_case_folding_block_restore() {
        assert_eq!(alternative("ot-ot"), None);
        assert_eq!(restore_apostrophes("İgor", true, true), None);
    }

    #[test]
    fn names_with_short_stem_or_w() {
        assert_eq!(name_kind("Shoga"), NameKind::Other);
        assert_eq!(name_kind("Shwartz"), NameKind::Foreign);
    }

    #[test]
    fn restore_via_stems_and_seen_ambiguous_words() {
        assert_eq!(
            restore_apostrophes("ogirni", false, false).as_deref(),
            Some("og'irni")
        );
        assert_eq!(
            restore_apostrophes("boyi", false, false).as_deref(),
            Some("bo'yi")
        );
    }
}
