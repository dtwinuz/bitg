// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # bitg-core
//!
//! O'zbek matnini 2026-yilgi lotin alifbosiga (O'RQ-1184: ö ğ ş ç, tutuq ʼ) o'girish yadrosi.
//! Tarmoqqa chiqmaydi, fayl yozmaydi, holat saqlamaydi — IME, tray ilova, CLI, mobil klaviatura va web
//! (WASM) bir xil natija beradi.
//!
//! ```
//! let out = bitg_core::convert("O'zbekiston shaharlari: Toshkent", &bitg_core::Options::default());
//! assert_eq!(out, "Özbekiston şaharlari: Toşkent");
//! ```

pub mod chars;
pub mod cyrillic;
pub mod ffi;
pub mod lang;
pub mod latin;
mod lexicon;
pub mod reverse;
mod to_cyr;
pub mod token;
pub mod user;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use reverse::OldStyle;

use chars::{is_apostrophe, normalize};
use lang::NameKind;

/// O'girish sozlamalari.
#[derive(Clone, Debug)]
pub struct Options {
    /// Chiqishdagi tutuq belgisi (standart U+02BC).
    pub tutuq: char,
    /// Inglizcha so'z va gaplarni himoya qilish (check, "How are you shah").
    pub english_guard: bool,
    /// Kirill harfli so'zlarni ham yangi lotinga o'girish.
    pub cyrillic: bool,
    /// Hech qachon o'girilmaydigan so'zlar (kichik harfda, tutuq `'` ko'rinishida — `chars::fold_key`).
    pub exceptions: Vec<String>,
    /// Tushib qolgan apostrofni lug'atdan tiklash: "ozbek" → "özbek", "togri" → "töğri".
    pub restore_apostrophes: bool,
    /// Foydalanuvchi lug'ati: so'z (va qo'shimchali shakli) aynan shunday yoziladi (`user::parse`).
    pub rules: Vec<user::Rule>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            tutuq: chars::DEFAULT_TUTUQ,
            english_guard: true,
            cyrillic: true,
            exceptions: Vec::new(),
            restore_apostrophes: true,
            rules: Vec::new(),
        }
    }
}

impl Options {
    fn is_exception(&self, core: &str) -> bool {
        !self.exceptions.is_empty() && {
            let k = chars::fold_key(core);
            self.exceptions.contains(&k)
        }
    }
}

/// So'z uchun qaror: o'girilsinmi va nega (CLI va IME log'i uchun).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Converted,
    Unchanged,
    Protected,
    Exception,
    English,
    /// Foydalanuvchi lug'atidagi qoida bo'yicha yozildi.
    Rule,
}

/// Bitta tokenning (bo'shliqsiz bo'lak) tahlili.
#[derive(Clone, Debug)]
struct Tok {
    start: usize,
    end: usize,
    evidence: f32,
    sentence_end: bool,
    /// Vergul, nuqtali vergul, ikki nuqta bilan tugaydi — gap bo'lagi chegarasi ("..., keyin choy").
    clause_end: bool,
    /// Ism sifatida (`resolve_names` dan keyin `Maybe` yonida xorijiy nom bo'lsa `Foreign` ga aylanadi).
    name: NameKind,
    /// Havola, kod, @user yoki brend (`fetch()`, `McDonald's`): gap tilini aytmaydi, kontekstda ovoz bermaydi.
    silent: bool,
}

/// So'z bo'yicha natija (`convert_words`): token, sabab, til dalili va ikki ma'noli so'zning muqobili.
#[derive(Clone, Debug, PartialEq)]
pub struct Word {
    pub token: String,
    pub reason: Reason,
    pub evidence: f32,
    /// Ikki ma'noli so'z (ochmaydi / o'chmaydi, oz / o'z): natijaga kirmagan ikkinchi o'qilishi, yangi alifboda
    /// ("öçmaydi"). UI uni "tekshiring" deb ko'rsatadi.
    pub alt: Option<String>,
}

/// Matn bo'yicha oldindan aniqlangan belgilar (bitta tokenga).
#[derive(Clone, Copy, Debug, Default)]
struct Hints {
    /// Muallif o'/g' apostrofini umuman qo'ymaydi — `writes_without_apostrophes`.
    loose: bool,
    /// Xorijiy imlodagi nom yoki uning bir qismi (Michelle, "Charlie Chaplin").
    foreign_name: bool,
}

const SENTENCE_END: &[char] = &['.', '!', '?', '…'];
const CLAUSE_END: &[char] = &[',', ';', ':'];

fn tokens(text: &str) -> Vec<Tok> {
    let mut v = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in text.char_indices() {
        // Tire va ko'p nuqta ham so'z chegarasi: "doesn't—and", "hitter…he's".
        if c.is_whitespace() || matches!(c, '—' | '–' | '…') {
            if let Some(s) = start.take() {
                v.push(Tok {
                    start: s,
                    end: i,
                    evidence: 0.0,
                    sentence_end: false,
                    clause_end: false,
                    name: NameKind::Other,
                    silent: false,
                });
            }
            if (c == '\n' || c == '…')
                && let Some(last) = v.last_mut()
            {
                last.sentence_end = true;
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        v.push(Tok {
            start: s,
            end: text.len(),
            evidence: 0.0,
            sentence_end: false,
            clause_end: false,
            name: NameKind::Other,
            silent: false,
        });
    }
    for t in &mut v {
        let tok = &text[t.start..t.end];
        let trimmed = tok.trim_end_matches(|c: char| {
            matches!(c, '"' | '»' | '”' | ')' | ']') || is_apostrophe(c)
        });
        if trimmed.ends_with(SENTENCE_END) {
            t.sentence_end = true;
        }
        if trimmed.ends_with(CLAUSE_END) {
            t.clause_end = true;
        }
    }
    v
}

/// Kontekst: gapdagi boshqa so'zlarning dalili (har biri ±3), yaqinroq so'z kuchliroq (1/masofa),
/// gap bo'lagi chegarasidan (vergul) o'tgan so'z 3 baravar kuchsiz: "Please check, keyin choy ichamiz".
fn context(toks: &[Tok], range: std::ops::Range<usize>, skip: usize) -> f32 {
    let slice = &toks[range];
    let mut sum = 0.0f32;
    let mut weights = 0.0f32;
    for (j, t) in slice.iter().enumerate() {
        // Xorijiy nom gap tilini aytmaydi: "Chicagoda yashaydi" — o'zbekcha gap.
        // Kod, havola va brend ham: "fetch() chaqiruvi", "Shohjahon McDonald's" — o'zbekcha gap.
        if j == skip || t.evidence == 0.0 || t.name == NameKind::Foreign || t.silent {
            continue;
        }
        let dist = j.abs_diff(skip) as f32;
        let (lo, hi) = if j < skip { (j, skip) } else { (skip, j) };
        let crosses = slice[lo..hi].iter().any(|x| x.clause_end);
        let w = (1.0 / dist) * if crosses { 0.3 } else { 1.0 };
        sum += w * t.evidence.clamp(-3.0, 3.0);
        weights += w;
    }
    if weights == 0.0 { 0.0 } else { sum / weights }
}

/// Kontekstda ovoz bermaydigan token: havola, email, kod (`push()`, `snake_case`), @user, #teg yoki brend.
fn is_silent(tok: &str, core: &str) -> bool {
    token::is_protected(tok) || lang::is_brand(core)
}

/// Kontekst qanchalik kuchli ta'sir qiladi. Ikki tilda ham bor so'z (chat, shop) shunga qarab hal bo'ladi.
const CONTEXT_WEIGHT: f32 = 1.5;

/// sh/ch bor, lekin o'/g'/tutuq yo'q so'z inglizcha bo'lishi mumkin — dalilga qaraymiz.
fn english_blocks(core: &str, own: f32, ctx: f32) -> bool {
    if !latin::digraph_only(core) {
        // o'/g' yoki tutuq — o'zbekcha belgi; faqat aniq inglizcha so'z (o'clock) yoki qisqartma to'xtatadi.
        return own <= -5.0;
    }
    // Keng tarqalgan inglizcha so'z, o'zbek lug'atida yo'q (check, shop, chat with) — o'zbekcha gap ichida ham.
    if own <= STRONG_ENGLISH && !lang::known(core).0 {
        return true;
    }
    own + CONTEXT_WEIGHT * ctx < 0.0
}

/// SCOWL 10/20 darajasidagi (eng ko'p uchraydigan) inglizcha so'zning dalili.
const STRONG_ENGLISH: f32 = -2.0;
/// Kontekst shundan inglizcharoq bo'lsa apostrof tiklanmaydi ("Gogol ozone ...").
const RESTORE_CTX_MIN: f32 = -1.0;
/// Inglizcha lug'atdagi so'zni (song → so'ng) tiklash uchun gap shunchalik aniq o'zbekcha bo'lsin.
const RESTORE_EN_CTX: f32 = 1.5;

/// Bitta tokenni o'giradi (kontekst bilan). `quote_open` — oldingi so'zlarda ochilgan tirnoq ('Yaxshi tong').
fn convert_token(
    tok: &str,
    opts: &Options,
    own: f32,
    ctx: f32,
    hints: Hints,
    quote_open: &mut bool,
) -> (String, Reason) {
    let loose = hints.loose;
    if opts.cyrillic && cyrillic::has_cyrillic(tok) {
        let out = cyrillic::to_new_latin(tok, opts.tutuq);
        let reason = if out == tok {
            Reason::Unchanged
        } else {
            Reason::Converted
        };
        return (out, reason);
    }
    if token::is_protected(tok) {
        return (tok.to_string(), Reason::Protected);
    }
    let parts = token::split_edges(tok);
    let lead = parts.lead;
    let mut core = parts.core;
    let mut trail = parts.trail;

    // Tirnoq: boshida apostrof — ochiladi; ochiq tirnoqli so'z apostrof bilan tugasa — bu yopuvchi tirnoq.
    let opens = lead.chars().any(is_apostrophe);
    if (opens || *quote_open) && core.chars().last().is_some_and(is_apostrophe) {
        let cut = core
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(core.len());
        let full_trail_start = parts.lead.len() + cut;
        trail = &tok[full_trail_start..];
        core = &core[..cut];
        *quote_open = false;
    } else if opens {
        *quote_open = true;
    }
    if trail.chars().any(is_apostrophe) {
        *quote_open = false;
    }

    // Foydalanuvchi lug'ati hammasidan ustun: so'zni o'zi qanday yozishni aytgan.
    if let Some(to) = user::apply(&opts.rules, core, opts.tutuq) {
        return (format!("{lead}{to}{trail}"), Reason::Rule);
    }
    // Chiziqchali so'z qism-qism: C-uchli → C-uçli, MR-shartli → MR-şartli, CH-III o'zgarmaydi.
    if core.contains('-')
        && !core.starts_with('-')
        && !core.ends_with('-')
        && core
            .split('-')
            .any(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_uppercase()))
    {
        return convert_compound(lead, core, trail, opts, ctx, loose);
    }
    // Unlisiz katta harfli qisqartma (TSH, PTCH, CH) va o'zbekchada yo'q harfli so'z (Pöschl, Müller).
    if is_acronym(core) || has_foreign_letters(core) {
        return (tok.to_string(), Reason::Unchanged);
    }
    // Brend nomi (Photoshop, Chevroletda) — savdo belgisi asl yozuvida qoladi.
    if opts.english_guard && lang::is_brand(core) {
        return (tok.to_string(), Reason::Protected);
    }
    // Xorijiy imlodagi nom (Michelle, Chicago, "Charlie Chaplin") — "Miçelle" hech bir imloda to'g'ri emas.
    if opts.english_guard && hints.foreign_name && !opts.is_exception(core) {
        return (tok.to_string(), Reason::English);
    }

    // Tushib qolgan apostrof (ozbek → o'zbek). Inglizcha gap ichida emas; istisno so'zlarga tegilmaydi.
    let restored;
    if opts.restore_apostrophes
        && ctx > RESTORE_CTX_MIN
        && !opts.is_exception(core)
        && let Some(r) = lang::restore_apostrophes(core, ctx >= RESTORE_EN_CTX, loose)
    {
        restored = r;
        core = &restored;
    }

    // Harfning o'zi: "O' harfi", "G' yoki O'" — inglizcha gapda bo'lmasa o'giriladi (tirnoq yuqorida ajratilgan).
    if lead.is_empty() && ctx > RESTORE_CTX_MIN && is_lone_letter(core) && !opts.is_exception(core)
    {
        let new_core = latin::to_new(core, opts.tutuq);
        if new_core != core {
            return (format!("{lead}{new_core}{trail}"), Reason::Converted);
        }
    }
    // Boshqa 2 belgili apostrofli bo'lak (rock 'n' roll, d') — so'z emas, tirnoq yoki qisqartma.
    if core.is_empty()
        || core.chars().count() <= 2 && core.chars().any(is_apostrophe)
        || !latin::has_convertible(core)
    {
        return (tok.to_string(), Reason::Unchanged);
    }
    if opts.is_exception(core) {
        return (tok.to_string(), Reason::Exception);
    }
    if opts.english_guard
        && (english_blocks(core, own, ctx) || english_with_suffix(core) || german_sch(core))
    {
        return (tok.to_string(), Reason::English);
    }
    let new_core = if lang::is_sh_split(core) || is_dis_h(core) {
        latin::to_new_split(core, opts.tutuq)
    } else {
        latin::to_new(core, opts.tutuq)
    };
    if new_core == core {
        return (tok.to_string(), Reason::Unchanged);
    }
    let mut out = String::with_capacity(tok.len());
    out.push_str(lead);
    out.push_str(&new_core);
    out.push_str(trail);
    (out, Reason::Converted)
}

/// Chiziqcha bilan ajralgan bo'laklar alohida qaror oladi; natija — eng kuchli sabab.
fn convert_compound(
    lead: &str,
    core: &str,
    trail: &str,
    opts: &Options,
    ctx: f32,
    loose: bool,
) -> (String, Reason) {
    let mut out = String::with_capacity(core.len() + lead.len() + trail.len() + 4);
    out.push_str(lead);
    let mut reason = Reason::Unchanged;
    for (i, part) in core.split('-').enumerate() {
        if i > 0 {
            out.push('-');
        }
        let own = if opts.english_guard {
            lang::evidence(part)
        } else {
            0.0
        };
        // Qisqartmali birikma (C-uchli) atrofidagi lotin harflar gapni "inglizcha" qilib ko'rsatadi — e'tiborsiz.
        let hints = Hints {
            loose,
            ..Hints::default()
        };
        let (conv, r) = convert_token(part, opts, own, ctx.max(0.0), hints, &mut false);
        out.push_str(&conv);
        reason = match (reason, r) {
            (_, Reason::Converted) | (Reason::Converted, _) => Reason::Converted,
            (Reason::Unchanged, r) => r,
            (r, _) => r,
        };
    }
    out.push_str(trail);
    (out, reason)
}

/// Lotincha qisqartma: unlisiz, SH/CH bilan tugaydi (TSH, PTCH) yoki faqat "CH"/"SH".
/// O'zbekcha qisqartmalar (MCHJ, KXSHT, SHHT, AQSH) o'giriladi.
fn is_acronym(core: &str) -> bool {
    core.chars().count() >= 2
        && core.chars().all(|c| c.is_ascii_uppercase())
        && !core
            .chars()
            .any(|c| matches!(c, 'A' | 'E' | 'I' | 'O' | 'U'))
        && (core.len() == 2 || core.ends_with("SH") || core.ends_with("CH"))
}

/// O'zbek alifbosida yo'q harf (Müller, Señor) yoki ö bilan nemischa "sch" (Pöschl, Tröltsch).
/// ö ning o'zi yangi alifbo harfi: "köcha" (ö tugmadan yozilgan) o'giriladi.
fn has_foreign_letters(core: &str) -> bool {
    // ë yo'q: eski o'zbek lotin matnlarida "yo" o'rnida keladi (ëzuvchi, ësh).
    const FOREIGN: &str = "äüßéèêáàâïîñøåæœúùûíìóòôãõýÄÜÉÈÊÁÀÂÏÎÑØÅÆŒÚÙÛÍÌÓÒÔÃÕÝ";
    if core.chars().any(|c| FOREIGN.contains(c)) {
        return true;
    }
    core.contains(['ö', 'Ö']) && core.to_lowercase().contains("sch")
}

/// Nemischa "sch" (Fischer, Schmidt). O'zbekchada s+ch faqat qo'shimchada (asoschi, bokschi) — ular lug'atda.
fn german_sch(core: &str) -> bool {
    let l = core.to_lowercase();
    l.contains("sch") && (l.starts_with("sch") || !lang::known(core).0)
}

/// dis + gormon (dishormonal, dishidroz, disharmoniya): s va h alohida.
fn is_dis_h(core: &str) -> bool {
    let l = core.to_lowercase();
    ["dishorm", "dishidr", "disharm"]
        .iter()
        .any(|p| l.starts_with(p))
}

/// O'zbek qo'shimchasi qo'shilgan inglizcha nom: Chromeda, Shellga, dashboardlar.
const UZ_SUFFIXES: &[&str] = &[
    "larning", "larni", "larga", "larda", "lardan", "dagi", "ning", "dan", "lar", "dir", "da",
    "ga", "ka", "ni",
];

fn english_with_suffix(core: &str) -> bool {
    // Lug'atdagi kichik harfli so'z (matchda) — o'zbekcha; bosh harfli nom (Shellga) — tekshiriladi.
    if lang::known(core).0 && !core.starts_with(|c: char| c.is_uppercase()) {
        return false;
    }
    let lower = core.to_lowercase();
    UZ_SUFFIXES.iter().any(|suf| {
        lower.strip_suffix(suf).is_some_and(|stem| {
            stem.chars().count() >= 4
                && latin::has_convertible(stem)
                && lang::evidence(stem) <= -2.0
        })
    })
}

/// `o'`, `G'` va h.k.: yolg'iz o/g harfi + apostrof.
fn is_lone_letter(core: &str) -> bool {
    let mut c = core.chars();
    matches!(
        (c.next(), c.next(), c.next()),
        (Some('o' | 'O' | 'g' | 'G'), Some(a), None) if is_apostrophe(a)
    )
}

/// So'zda o'/g' + apostrof bor (o'zbek, tog'): muallif apostrof qo'yadi.
fn has_og_apostrophe(core: &str) -> bool {
    let mut prev = ' ';
    for c in core.chars() {
        if is_apostrophe(c) && matches!(prev, 'o' | 'O' | 'g' | 'G') {
            return true;
        }
        prev = c;
    }
    false
}

/// Muallif o'/g' apostrofini umuman qo'ymaydi: matnda o'/g' + apostrof yo'q, matn o'zbekcha, va kamida bitta
/// so'z aniq tiklanadi (ozbek → o'zbek). Shunda ikki ma'noli so'zlar ham (oz / o'z, yoq / yo'q) tiklanadi.
/// Qaytadi: (`loose`, matnda o'/g' apostrofi umuman yo'q).
fn writes_without_apostrophes(text: &str, toks: &[Tok], opts: &Options) -> (bool, bool) {
    if !opts.restore_apostrophes {
        return (false, false);
    }
    let cores = || {
        toks.iter()
            .map(|t| token::split_edges(&text[t.start..t.end]).core)
    };
    if cores().any(has_og_apostrophe) {
        return (false, false);
    }
    let loose = toks.iter().map(|t| t.evidence).sum::<f32>() > 0.0
        && cores().any(|c| {
            !token::is_protected(c)
                && !opts.is_exception(c)
                && lang::restore_apostrophes(c, false, false).is_some()
        });
    (loose, true)
}

/// Yonma-yon bosh harfli nomlar (bog'lovchi "va" / "and" orqali ham) bitta zanjir: zanjirda xorijiy imlodagi nom
/// bo'lsa, ikki xil o'qiladigan qo'shnisi ham xorijiy ("Charlie Chaplin", "Richard va Michelle").
/// Vergul, nuqta zanjirni uzadi; o'zbekcha nom (Shohruh, `Other`) zanjirga kirmaydi.
fn resolve_names(toks: &mut [Tok]) {
    let is_name = |t: &Tok| matches!(t.name, NameKind::Foreign | NameKind::Maybe);
    let mut i = 0;
    while i < toks.len() {
        if !is_name(&toks[i]) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j + 1 < toks.len() && !toks[j].clause_end && !toks[j].sentence_end {
            let next = &toks[j + 1];
            let linked = next.name == NameKind::Link
                && !next.clause_end
                && !next.sentence_end
                && toks.get(j + 2).is_some_and(is_name);
            if !(is_name(next) || linked) {
                break;
            }
            j += if linked { 2 } else { 1 };
        }
        if toks[i..=j].iter().any(|t| t.name == NameKind::Foreign) {
            for t in &mut toks[i..=j] {
                if t.name == NameKind::Maybe {
                    t.name = NameKind::Foreign;
                }
            }
        }
        i = j + 1;
    }
}

/// Ikki ma'noli so'zning natijaga kirmagan o'qilishi (yangi alifboda). `conv` — konvertor bergan natija.
fn alternative_of(tok: &str, conv: &str, opts: &Options) -> Option<String> {
    let parts = token::split_edges(tok);
    let core = parts.core;
    if opts.is_exception(core) || !core.contains(['o', 'O', 'g', 'G']) {
        return None;
    }
    let alt = lang::alternative(core)?;
    let render = |c: &str| {
        format!(
            "{}{}{}",
            parts.lead,
            latin::to_new(c, opts.tutuq),
            parts.trail
        )
    };
    let restored = render(&alt);
    Some(if conv == restored {
        render(core)
    } else {
        restored
    })
}

/// Matnni yangi alifboga o'giradi. Bo'shliqlar va qator tuzilishi aynan saqlanadi.
pub fn convert(text: &str, opts: &Options) -> String {
    convert_impl(text, opts, false).0
}

/// O'girish va har bir token bo'yicha sabab (`bitg explain` uchun).
pub fn convert_explained(text: &str, opts: &Options) -> (String, Vec<(String, Reason, f32)>) {
    let (out, words) = convert_words(text, opts);
    (
        out,
        words
            .into_iter()
            .map(|w| (w.token, w.reason, w.evidence))
            .collect(),
    )
}

/// O'girish va har bir token bo'yicha to'liq ma'lumot, ikki ma'noli so'zlarning muqobili bilan (sayt, CLI).
pub fn convert_words(text: &str, opts: &Options) -> (String, Vec<Word>) {
    convert_impl(text, opts, true)
}

fn convert_impl(text: &str, opts: &Options, want_alts: bool) -> (String, Vec<Word>) {
    let text = normalize(text);
    let text = text.as_ref();
    let mut toks = tokens(text);
    for t in &mut toks {
        let tok = &text[t.start..t.end];
        let core = token::split_edges(tok).core;
        if opts.english_guard {
            t.evidence = lang::evidence(core);
            t.name = lang::name_kind(core);
            t.silent = is_silent(tok, core);
        }
    }
    if opts.english_guard {
        resolve_names(&mut toks);
    }
    let (loose, no_og) = writes_without_apostrophes(text, &toks, opts);
    let mut out = String::with_capacity(text.len() + 16);
    let mut report = Vec::with_capacity(toks.len());
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
        out.push_str(&text[last..t.start]);
        let ws = &text[last..t.start];
        if ws.contains('\n') {
            quote_open = false; // tirnoq qatordan qatorga o'tmaydi
        }
        let ctx = context(&toks, sent_start..sent_end + 1, i - sent_start);
        let tok = &text[t.start..t.end];
        let hints = Hints {
            loose,
            foreign_name: t.name == NameKind::Foreign,
        };
        let (conv, reason) = convert_token(tok, opts, t.evidence, ctx, hints, &mut quote_open);
        // Matnda o'/g' apostrofi yo'q: ikki ma'noli so'zning ikkinchi o'qilishi ko'rsatiladi.
        let alt = if want_alts
            && no_og
            && ctx > RESTORE_CTX_MIN
            && matches!(reason, Reason::Converted | Reason::Unchanged)
            && !cyrillic::has_cyrillic(tok)
        {
            alternative_of(tok, &conv, opts)
        } else {
            None
        };
        report.push(Word {
            token: tok.to_string(),
            reason,
            evidence: t.evidence,
            alt,
        });
        out.push_str(&conv);
        last = t.end;
    }
    out.push_str(&text[last..]);
    (out, report)
}

/// Yangi alifbodan eski lotinga.
pub fn to_old(text: &str, style: OldStyle) -> String {
    reverse::to_old(text, style)
}

/// Lotin (yangi yoki eski yozuv) → o'zbek kirill. Inglizcha so'z, havola, email va kod lotinda qoladi.
///
/// ```
/// assert_eq!(bitg_core::to_cyrillic("O'zbekiston shaharlari"), "Ўзбекистон шаҳарлари");
/// assert_eq!(bitg_core::to_cyrillic("Özbekiston şaharlari"), "Ўзбекистон шаҳарлари");
/// ```
pub fn to_cyrillic(text: &str) -> String {
    to_cyr::to_cyrillic(text)
}

/// Yozish paytida (IME, tray): so'zlar birma-bir keladi, kontekst — faqat shu gapdagi oldingi so'zlar.
#[derive(Debug, Default, Clone)]
pub struct Session {
    recent: Vec<Tok>,
    quote_open: bool,
    /// Muallif o'/g' apostrofini qo'ymayapti (oldingi so'z tiklandi) — `writes_without_apostrophes`.
    loose: bool,
}

/// Yozish paytidagi tahrir: so'z oxiridan `delete` ta belgini o'chirib, `insert` yozilsin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub delete: usize,
    pub insert: String,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    /// Kursor boshqa joyga o'tdi, Enter, boshqa maydon — kontekst unutiladi.
    pub fn reset(&mut self) {
        self.recent.clear();
        self.quote_open = false;
        self.loose = false;
    }

    /// So'z tugadi (probel, tinish belgisi, Enter). O'zgarish kerak bo'lsa — tahrir.
    pub fn complete_word(&mut self, word: &str, opts: &Options) -> Option<Edit> {
        let word_n = normalize(word);
        let word_n = word_n.as_ref();
        let core = token::split_edges(word_n).core;
        let own = if opts.english_guard {
            lang::evidence(core)
        } else {
            0.0
        };
        let trimmed = word_n
            .trim_end_matches(|c: char| matches!(c, '"' | '»' | '”' | ')') || is_apostrophe(c));
        let mut me = Tok {
            start: 0,
            end: 0,
            evidence: own,
            sentence_end: trimmed.ends_with(SENTENCE_END),
            clause_end: trimmed.ends_with(CLAUSE_END),
            name: if opts.english_guard {
                lang::name_kind(core)
            } else {
                NameKind::Other
            },
            silent: opts.english_guard && is_silent(word_n, core),
        };
        // Yozish paytida faqat chap qo'shni ma'lum: "Charlie Chaplin" (Charlie oldin yozilgan) ushlanadi,
        // "Richard va Michelle" dagi Richard esa Michelle yozilishidan oldin o'giriladi.
        if me.name == NameKind::Maybe {
            let n = self.recent.len();
            let open = |t: &Tok| !t.clause_end && !t.sentence_end;
            let prev = self.recent.last();
            let chained = prev.is_some_and(|p| p.name == NameKind::Foreign && open(p))
                || n >= 2
                    && self.recent[n - 1].name == NameKind::Link
                    && open(&self.recent[n - 1])
                    && self.recent[n - 2].name == NameKind::Foreign
                    && open(&self.recent[n - 2]);
            if chained {
                me.name = NameKind::Foreign;
            }
        }
        let mut window = self.recent.clone();
        window.push(me.clone());
        let mut ctx = context(&window, 0..window.len(), window.len() - 1);
        // Yozish paytida o'ng kontekst hali yo'q: faqat o'zbek lug'atidagi kichik harfli so'zni (check shapaloq)
        // oldingi inglizcha so'z yolg'iz to'xtatmasin. Butun matnda (convert) ikki tomon hal qiladi (mosh pit).
        let (uz, en) = lang::known(core);
        if uz && !en && own > 0.0 && core.starts_with(|c: char| c.is_lowercase()) {
            ctx = ctx.max(0.0);
        }
        let hints = Hints {
            loose: self.loose,
            foreign_name: me.name == NameKind::Foreign,
        };
        let (conv, _) = convert_token(word_n, opts, own, ctx, hints, &mut self.quote_open);
        if has_og_apostrophe(core) {
            self.loose = false;
        } else if !self.loose && opts.restore_apostrophes && conv != word {
            self.loose = lang::restore_apostrophes(core, false, false).is_some();
        }
        if me.sentence_end {
            self.recent.clear();
        } else {
            self.recent.push(me);
            if self.recent.len() > 8 {
                self.recent.remove(0);
            }
        }
        if conv == word {
            return None;
        }
        Some(Edit {
            delete: word.chars().count(),
            insert: conv,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts_with(exceptions: &[&str]) -> Options {
        Options {
            exceptions: exceptions.iter().map(|s| s.to_string()).collect(),
            ..Options::default()
        }
    }

    fn tok(name: NameKind, clause_end: bool, sentence_end: bool) -> Tok {
        Tok {
            start: 0,
            end: 0,
            evidence: 0.0,
            sentence_end,
            clause_end,
            name,
            silent: false,
        }
    }

    #[test]
    fn ellipsis_ends_sentence() {
        let t = tokens("keldi…ketdi");
        assert_eq!(t.len(), 2);
        assert!(t[0].sentence_end);
        assert!(!t[1].sentence_end);
    }

    #[test]
    fn known_uzbek_word_is_decided_by_context() {
        assert!(!english_blocks("shahar", -3.0, 2.0));
        assert!(english_blocks("shahar", -3.0, 0.0));
    }

    #[test]
    fn user_rules_win_over_everything_but_links() {
        let (_, rules) = user::parse("shamollash=şamollaş\nbog'cha=bog'ça\nshop=şop");
        let o = Options {
            rules,
            ..Options::default()
        };
        assert_eq!(
            convert("Shamollashdan, bog'chaga va shop!", &o),
            "Şamollaşdan, bog'çaga va şop!"
        );
        assert_eq!(convert("https://shop.uz", &o), "https://shop.uz");
        let (_, words) = convert_words("shamollash", &o);
        assert_eq!(words[0].reason, Reason::Rule);
        let mut s = Session::new();
        assert_eq!(
            s.complete_word("Bog'cha", &o).map(|e| e.insert).as_deref(),
            Some("Bog'ça")
        );
    }

    #[test]
    fn code_links_and_brands_do_not_vote_for_english() {
        let o = Options::default();
        assert_eq!(convert("fetch() chaqiruvi", &o), "fetch() çaqiruvi");
        assert_eq!(convert("Shohjahon McDonald's", &o), "Şohjahon McDonald's");
        assert_eq!(convert("https://x.uz shahar", &o), "https://x.uz şahar");
        assert_eq!(convert("push() the shop", &o), "push() the shop");
        let mut s = Session::new();
        assert_eq!(s.complete_word("fetch()", &o), None);
        assert_eq!(
            s.complete_word("chaqiruvi", &o)
                .map(|e| e.insert)
                .as_deref(),
            Some("çaqiruvi")
        );
    }

    #[test]
    fn unknown_cyrillic_letters_are_unchanged() {
        let (out, words) = convert_words("є", &Options::default());
        assert_eq!(out, "є");
        assert_eq!(words[0].reason, Reason::Unchanged);
        let (out, words) = convert_words("шаҳар", &Options::default());
        assert_eq!(out, "şahar");
        assert_eq!(words[0].reason, Reason::Converted);
        assert_eq!(words[0].alt, None);
    }

    #[test]
    fn exception_beats_foreign_name_and_lone_letter() {
        let (_, words) = convert_words("Michelle keldi", &Options::default());
        assert_eq!(words[0].reason, Reason::English);
        let (out, words) = convert_words("Michelle keldi", &opts_with(&["michelle"]));
        assert_eq!(out, "Michelle keldi");
        assert_eq!(words[0].reason, Reason::Exception);
        assert_eq!(convert("O' harfi", &Options::default()), "Ö harfi");
        let (out, words) = convert_words("O' harfi", &opts_with(&["o'"]));
        assert_eq!(out, "O' harfi");
        assert_eq!(words[0].reason, Reason::Unchanged);
    }

    #[test]
    fn english_stem_with_uzbek_suffix_is_kept() {
        let (out, words) = convert_words("flashlar", &Options::default());
        assert_eq!(out, "flashlar");
        assert_eq!(words[0].reason, Reason::English);
        assert!(!english_with_suffix("Bookda"));
    }

    #[test]
    fn compound_without_english_guard() {
        let o = Options {
            english_guard: false,
            ..Options::default()
        };
        assert_eq!(convert("C-uchli", &o), "C-uçli");
    }

    #[test]
    fn german_sch_inside_unknown_word() {
        assert!(german_sch("Fischer"));
        assert!(german_sch("Schmidt"));
        assert!(!german_sch("bokschi"));
    }

    #[test]
    fn lone_letter_needs_apostrophe() {
        for w in ["o'", "O'", "g'", "G'"] {
            assert!(is_lone_letter(w), "{w}");
        }
        for w in ["ot", "Ot", "gu", "Gu", "o", "o''"] {
            assert!(!is_lone_letter(w), "{w}");
        }
    }

    #[test]
    fn closed_link_breaks_name_chain() {
        use NameKind::*;
        for (link, chained) in [
            (tok(Link, false, false), true),
            (tok(Link, true, false), false),
            (tok(Link, false, true), false),
        ] {
            let mut t = vec![tok(Foreign, false, false), link, tok(Maybe, false, false)];
            resolve_names(&mut t);
            assert_eq!(t[2].name == Foreign, chained);
        }
    }

    #[test]
    fn exception_has_no_alternative() {
        let o = Options::default();
        assert_eq!(
            alternative_of("ochmaydi", "oçmaydi", &o).as_deref(),
            Some("öçmaydi")
        );
        assert_eq!(
            alternative_of("ochmaydi", "oçmaydi", &opts_with(&["ochmaydi"])),
            None
        );
    }

    fn typed(words: &[&str], opts: &Options) -> Vec<String> {
        let mut s = Session::new();
        words
            .iter()
            .map(|w| {
                s.complete_word(w, opts)
                    .map(|e| e.insert)
                    .unwrap_or_else(|| w.to_string())
            })
            .collect()
    }

    #[test]
    fn session_without_guard_or_restore() {
        let no_guard = Options {
            english_guard: false,
            ..Options::default()
        };
        assert_eq!(typed(&["check"], &no_guard), ["çeck"]);
        let no_restore = Options {
            restore_apostrophes: false,
            ..Options::default()
        };
        assert_eq!(
            typed(&["shahar", "ozbek", "oz"], &no_restore),
            ["şahar", "ozbek", "oz"]
        );
    }

    #[test]
    fn session_name_chain_needs_open_neighbours() {
        let o = Options::default();
        assert_eq!(typed(&["Michelle", "Richard"], &o)[1], "Richard");
        assert_eq!(typed(&["Michelle,", "Richard"], &o)[1], "Riçard");
        assert_eq!(typed(&["Bugun", "kelgan", "Richard"], &o)[2], "Riçard");
        assert_eq!(typed(&["Michelle", "va,", "Richard"], &o)[2], "Riçard");
        assert_eq!(typed(&["Bugun", "va", "Richard"], &o)[2], "Riçard");
    }

    #[test]
    fn session_keeps_last_eight_words() {
        let o = Options::default();
        let mut s = Session::new();
        for w in [
            "bir", "ikki", "uch", "tort", "besh", "olti", "yetti", "sakkiz", "toqqiz",
        ] {
            s.complete_word(w, &o);
        }
        assert_eq!(s.recent.len(), 8);
        s.complete_word("tamom.", &o);
        assert!(s.recent.is_empty());
    }
}
