// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Klaviatura tartibi: tugma (US joylashuvidagi nomi bilan) → 4 qatlam belgisi (oddiy, Shift, AltGr,
//! AltGr+Shift). Bo'sh qatlam — tugma odatdagidek (inglizcha) ishlaydi. Platformaga bog'liq emas.
//!
//! `%APPDATA%\bitg\keymap.txt` (maxsus tartib):
//!
//! ```text
//! # tugma  oddiy  Shift  AltGr  AltGr+Shift
//! [  ö  Ö  [  {
//! -  -  -  –  —
//! ```
//!
//! `-` — qatlam o'zgarmaydi, `U+XXXX` — belgi kodi (probel, `-`, `#` va ko'rinmas belgilar uchun).

/// Tugmalar: nomi (US tartibidagi belgi) va skan-kod (set 1). Tartib — klaviaturadagi qatorlar.
pub const KEYS: &[(&str, u8)] = &[
    ("`", 0x29),
    ("1", 0x02),
    ("2", 0x03),
    ("3", 0x04),
    ("4", 0x05),
    ("5", 0x06),
    ("6", 0x07),
    ("7", 0x08),
    ("8", 0x09),
    ("9", 0x0A),
    ("0", 0x0B),
    ("-", 0x0C),
    ("=", 0x0D),
    ("q", 0x10),
    ("w", 0x11),
    ("e", 0x12),
    ("r", 0x13),
    ("t", 0x14),
    ("y", 0x15),
    ("u", 0x16),
    ("i", 0x17),
    ("o", 0x18),
    ("p", 0x19),
    ("[", 0x1A),
    ("]", 0x1B),
    ("\\", 0x2B),
    ("a", 0x1E),
    ("s", 0x1F),
    ("d", 0x20),
    ("f", 0x21),
    ("g", 0x22),
    ("h", 0x23),
    ("j", 0x24),
    ("k", 0x25),
    ("l", 0x26),
    (";", 0x27),
    ("'", 0x28),
    ("z", 0x2C),
    ("x", 0x2D),
    ("c", 0x2E),
    ("v", 0x2F),
    ("b", 0x30),
    ("n", 0x31),
    ("m", 0x32),
    (",", 0x33),
    (".", 0x34),
    ("/", 0x35),
];

pub const LEVELS: usize = 4;

/// Tahrir sessiyasi ishlamaganda IME belgini Unicode tugma sifatida yuboradi: `dwExtraInfo = REMAP_TAG << 32 | belgi`
/// (surrogat juftning ikkinchi yarmida belgi 0). IME ularni so'zning davomi deb hisoblaydi.
pub const REMAP_TAG: u64 = 0xB17C;
const SLOTS: usize = 0x60;

/// Tayyor tartiblar.
pub const PRESETS: &[&str] = &[
    "tilmoch",
    "kiritish",
    "tezyechim",
    "turkish",
    "altgr",
    "kirill",
];
pub const DEFAULT_LAYOUT: &str = "tilmoch";
pub const CUSTOM: &str = "custom";

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Keymap {
    slots: [[Option<char>; LEVELS]; SLOTS],
}

impl Default for Keymap {
    fn default() -> Self {
        Self {
            slots: [[None; LEVELS]; SLOTS],
        }
    }
}

pub fn scan_of(label: &str) -> Option<u8> {
    KEYS.iter().find(|(l, _)| *l == label).map(|&(_, s)| s)
}

/// Boshqaruv belgilari (Enter, Tab, ...) tugmaga qo'yilmaydi.
pub fn usable(c: char) -> bool {
    !c.is_control()
}

/// "-" — bo'sh qatlam (`Some(None)`); "U+00AB" — kod; bitta belgi — o'zi. Boshqasi — `None` (xato).
pub fn parse_cell(t: &str) -> Option<Option<char>> {
    if t == "-" || t.is_empty() {
        return Some(None);
    }
    let c = if let Some(hex) = t.strip_prefix("U+").or_else(|| t.strip_prefix("u+"))
        && !hex.is_empty()
    {
        u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)?
    } else {
        let mut it = t.chars();
        match (it.next(), it.next()) {
            (Some(c), None) => c,
            _ => return None,
        }
    };
    usable(c).then_some(Some(c))
}

/// Ko'rinmas belgilar (nol kenglikdagi, yo'nalish).
fn invisible(c: char) -> bool {
    matches!(c, '\u{200B}'..='\u{200F}' | '\u{2028}'..='\u{202E}' | '\u{2060}'..='\u{206F}' | '\u{FEFF}')
}

fn cell_text(c: Option<char>) -> String {
    match c {
        None => "-".into(),
        Some(c) if c.is_whitespace() || c == '-' || c == '#' || invisible(c) => {
            format!("U+{:04X}", c as u32)
        }
        Some(c) => c.to_string(),
    }
}

impl Keymap {
    pub fn get(&self, scan: u8, level: usize) -> Option<char> {
        self.slots.get(scan as usize)?.get(level).copied().flatten()
    }

    pub fn set(&mut self, scan: u8, level: usize, c: Option<char>) {
        if let Some(slot) = self.slots.get_mut(scan as usize)
            && level < LEVELS
        {
            slot[level] = c.filter(|&c| usable(c));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|s| s.iter().all(Option::is_none))
    }

    /// Qatlamlar: tugma nomi bo'yicha (oyna uchun).
    pub fn row(&self, label: &str) -> [Option<char>; LEVELS] {
        scan_of(label).map_or([None; LEVELS], |s| self.slots[s as usize])
    }

    /// Bosilgan tugma qaysi belgini berishi kerak. `None` — tugma o'zgarmaydi.
    /// CapsLock faqat harflarda (kichik/katta jufti bor qatlamda) Shift'ni almashtiradi.
    pub fn lookup(&self, scan: u8, shift: bool, caps: bool, altgr: bool) -> Option<char> {
        let slot = self.slots.get(scan as usize)?;
        let base = if altgr { 2 } else { 0 };
        let (lo, up) = (slot[base], slot[base + 1]);
        let letter = matches!((lo, up), (Some(l), Some(u)) if l.is_lowercase() && u.is_uppercase());
        if shift ^ (caps && letter) { up } else { lo }
    }

    /// Faylni o'qish. Noma'lum tugma yoki buzuq belgi bo'lgan qator o'tkazib yuboriladi.
    pub fn parse(text: &str) -> Self {
        let mut m = Self::default();
        for line in text.lines() {
            let line = line.trim().trim_start_matches('\u{FEFF}');
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut tok = line.split_whitespace();
            let Some(scan) = tok.next().and_then(scan_of) else {
                continue;
            };
            let Some(cells) = tok.take(LEVELS).map(parse_cell).collect::<Option<Vec<_>>>() else {
                continue;
            };
            for (lvl, c) in cells.into_iter().enumerate() {
                m.set(scan, lvl, c);
            }
        }
        m
    }

    pub fn to_text(&self) -> String {
        let mut out = String::from(
            "# bitg klaviatura tartibi. Sozlamalar oynasi yozadi; qo'lda tahrirlash ham mumkin.\n\
             # tugma  oddiy  Shift  AltGr  AltGr+Shift   (\"-\" = o'zgarmaydi, U+XXXX = belgi kodi)\n",
        );
        for &(label, scan) in KEYS {
            let s = self.slots[scan as usize];
            if s.iter().all(Option::is_none) {
                continue;
            }
            let cells: Vec<String> = s.iter().map(|&c| cell_text(c)).collect();
            out.push_str(&format!("{label}  {}\n", cells.join("  ")));
        }
        out
    }

    fn with(rows: &[(&str, [&str; LEVELS])]) -> Self {
        let mut m = Self::default();
        for (label, cells) in rows {
            let scan = scan_of(label).expect("preset key");
            for (lvl, t) in cells.iter().enumerate() {
                m.set(scan, lvl, parse_cell(t).expect("preset cell"));
            }
        }
        m
    }
}

/// Tayyor tartib nomi bo'yicha. Noma'lum nom — `None`.
pub fn preset(name: &str) -> Option<Keymap> {
    Some(match name {
        // tilmoch.ai/terilmalar: ö ğ ş ç kvadrat qavs, nuqta-vergul va apostrof joyida, tutuq ` tugmasida.
        // Tutuq — qonundagi ʼ (U+02BC), o'girish natijasi bilan bir xil (’ U+2019 tirnoq, tutuq emas).
        // AltGr+Shift'dagi { } " ` | \ — dasturchi belgilari yo'qolmasin deb qo'shilgan.
        "tilmoch" => Keymap::with(&[
            ("`", ["ʼ", "\"", "‘", "`"]),
            ("3", ["-", "-", "-", "№"]),
            ("9", ["-", "-", "‘", "-"]),
            ("0", ["-", "-", "’", "-"]),
            ("-", ["-", "-", "–", "—"]),
            ("[", ["ö", "Ö", "[", "{"]),
            ("]", ["ğ", "Ğ", "]", "}"]),
            ("\\", [";", ":", "\\", "|"]),
            (";", ["ş", "Ş", ";", ":"]),
            ("'", ["ç", "Ç", "'", "\""]),
            (",", [",", "“", "<", "«"]),
            (".", [".", "”", ">", "»"]),
        ]),
        // kiritish.uz "Özbekça - yangi" (new-uzb.klc, v1.0, MIT): Tilmoch'ga yaqin; AltGr+K/L — “ ” va ‘ ’.
        // AltGr+; va AltGr+Shift+' manbada bo'sh — dasturchi belgilari uchun qo'shilgan.
        "kiritish" => Keymap::with(&[
            ("`", ["ʼ", "\"", "`", "~"]),
            ("-", ["-", "-", "—", "–"]),
            ("[", ["ö", "Ö", "[", "{"]),
            ("]", ["ğ", "Ğ", "]", "}"]),
            ("\\", [";", ":", "\\", "|"]),
            (";", ["ş", "Ş", ";", ":"]),
            ("'", ["ç", "Ç", "'", "\""]),
            ("k", ["-", "-", "“", "‘"]),
            ("l", ["-", "-", "”", "’"]),
        ]),
        // tezyechim.uz "Özbek lotin 2026": [ ç, ] ö, - ğ, = ş, tutuq — o'ng Alt + '.
        // Ko'chgan [ ] - = belgilari AltGr'da (manba aniq qatlamni aytmaydi — US tartibidagidek qo'yilgan).
        "tezyechim" => Keymap::with(&[
            ("[", ["ç", "Ç", "[", "{"]),
            ("]", ["ö", "Ö", "]", "}"]),
            ("-", ["ğ", "Ğ", "U+002D", "_"]),
            ("=", ["ş", "Ş", "=", "+"]),
            ("'", ["-", "-", "ʼ", "\""]),
        ]),
        // Turkcha Q harflari US tugmalarida; joyidan ko'chgan belgilar AltGr'da.
        "turkish" => Keymap::with(&[
            ("i", ["ı", "I", "-", "-"]),
            ("[", ["ğ", "Ğ", "[", "{"]),
            ("]", ["ü", "Ü", "]", "}"]),
            (";", ["ş", "Ş", ";", ":"]),
            ("'", ["i", "İ", "'", "\""]),
            (",", ["ö", "Ö", "<", "-"]),
            (".", ["ç", "Ç", ">", "-"]),
            ("/", [".", ":", "/", "?"]),
            ("\\", [",", ";", "\\", "|"]),
        ]),
        // Hech bir tugma joyidan ko'chmaydi: yangi harflar faqat AltGr (o'ng Alt) bilan.
        "altgr" => Keymap::with(&[
            ("o", ["-", "-", "ö", "Ö"]),
            ("g", ["-", "-", "ğ", "Ğ"]),
            ("s", ["-", "-", "ş", "Ş"]),
            ("c", ["-", "-", "ç", "Ç"]),
            ("'", ["-", "-", "ʼ", "\""]),
            ("-", ["-", "-", "–", "—"]),
            (",", ["-", "-", "«", "“"]),
            (".", ["-", "-", "»", "”"]),
        ]),
        // Windows "O'zbek (kirill)" tartibi: harf tugmalarida kirill, AltGr'da US belgilari qoladi.
        // Yozilgan kirill so'z oxirida yangi lotinga o'giriladi (avto o'chiq bo'lsa — kirill qoladi).
        CYRILLIC => cyrillic(),
        _ => return None,
    })
}

pub const CYRILLIC: &str = "kirill";

fn cyrillic() -> Keymap {
    const LETTERS: &[(&str, char)] = &[
        ("`", 'ё'),
        ("-", 'ғ'),
        ("=", 'ҳ'),
        ("q", 'й'),
        ("w", 'ц'),
        ("e", 'у'),
        ("r", 'к'),
        ("t", 'е'),
        ("y", 'н'),
        ("u", 'г'),
        ("i", 'ш'),
        ("o", 'ў'),
        ("p", 'з'),
        ("[", 'х'),
        ("]", 'ъ'),
        ("a", 'ф'),
        ("s", 'қ'),
        ("d", 'в'),
        ("f", 'а'),
        ("g", 'п'),
        ("h", 'р'),
        ("j", 'о'),
        ("k", 'л'),
        ("l", 'д'),
        (";", 'ж'),
        ("'", 'э'),
        ("z", 'я'),
        ("x", 'ч'),
        ("c", 'с'),
        ("v", 'м'),
        ("b", 'и'),
        ("n", 'т'),
        ("m", 'ь'),
        (",", 'б'),
        (".", 'ю'),
    ];
    // Raqam qatori Shift bilan — ruscha tartibdagidek.
    const SHIFTED: &[(&str, char)] = &[
        ("2", '"'),
        ("3", '№'),
        ("4", ';'),
        ("6", ':'),
        ("7", '?'),
        ("/", ','),
    ];
    const US: &[(&str, char, char)] = &[
        ("`", '`', '~'),
        ("-", '-', '_'),
        ("=", '=', '+'),
        ("[", '[', '{'),
        ("]", ']', '}'),
        (";", ';', ':'),
        ("'", '\'', '"'),
        (",", ',', '<'),
        (".", '.', '>'),
        ("/", '/', '?'),
    ];
    let mut m = Keymap::default();
    for &(label, lo) in LETTERS {
        let scan = scan_of(label).expect("preset key");
        m.set(scan, 0, Some(lo));
        m.set(scan, 1, lo.to_uppercase().next());
    }
    for &(label, c) in SHIFTED {
        m.set(scan_of(label).expect("preset key"), 1, Some(c));
    }
    m.set(scan_of("/").expect("preset key"), 0, Some('.'));
    for &(label, lo, up) in US {
        let scan = scan_of(label).expect("preset key");
        m.set(scan, 2, Some(lo));
        m.set(scan, 3, Some(up));
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    const LBR: u8 = 0x1A;
    const QUOTE: u8 = 0x28;

    #[test]
    fn tilmoch_letters_and_levels() {
        let m = preset("tilmoch").unwrap();
        assert_eq!(m.lookup(LBR, false, false, false), Some('ö'));
        assert_eq!(m.lookup(LBR, true, false, false), Some('Ö'));
        assert_eq!(m.lookup(LBR, false, false, true), Some('['));
        assert_eq!(m.lookup(LBR, true, false, true), Some('{'));
        assert_eq!(m.lookup(QUOTE, false, false, false), Some('ç'));
        assert_eq!(
            m.lookup(scan_of("`").unwrap(), false, false, false),
            Some('ʼ')
        );
        // A–Z o'zgarmaydi.
        assert_eq!(m.lookup(scan_of("a").unwrap(), false, false, false), None);
        // 3 faqat AltGr+Shift bilan.
        assert_eq!(m.lookup(scan_of("3").unwrap(), true, false, false), None);
        assert_eq!(
            m.lookup(scan_of("3").unwrap(), true, false, true),
            Some('№')
        );
    }

    #[test]
    fn caps_lock_only_for_letters() {
        let m = preset("tilmoch").unwrap();
        assert_eq!(m.lookup(LBR, false, true, false), Some('Ö'));
        assert_eq!(m.lookup(LBR, true, true, false), Some('ö'));
        let comma = scan_of(",").unwrap();
        assert_eq!(m.lookup(comma, false, true, false), Some(','));
        let t = preset("turkish").unwrap();
        assert_eq!(t.lookup(QUOTE, false, true, false), Some('İ'));
        assert_eq!(
            t.lookup(scan_of("i").unwrap(), false, true, false),
            Some('I')
        );
    }

    #[test]
    fn altgr_preset_keeps_base_keys() {
        let m = preset("altgr").unwrap();
        let o = scan_of("o").unwrap();
        assert_eq!(m.lookup(o, false, false, false), None);
        assert_eq!(m.lookup(o, false, false, true), Some('ö'));
        assert_eq!(m.lookup(o, false, true, true), Some('Ö'));
        assert_eq!(m.lookup(LBR, false, false, false), None);
    }

    #[test]
    fn text_round_trip_and_escapes() {
        for name in PRESETS {
            let m = preset(name).unwrap();
            assert_eq!(Keymap::parse(&m.to_text()), m, "{name}");
        }
        let text = preset("tilmoch").unwrap().to_text();
        assert!(text.contains("[  ö  Ö  [  {\n"));
        assert!(text.contains("\n-  -  -  –  —\n"));
        let mut m = Keymap::default();
        m.set(LBR, 0, Some(' '));
        m.set(LBR, 1, Some('#'));
        assert!(m.to_text().contains("[  U+0020  U+0023  -  -"));
        assert_eq!(Keymap::parse(&m.to_text()), m);
    }

    #[test]
    fn parse_skips_bad_lines() {
        let m = Keymap::parse(
            "\u{FEFF}# izoh\n[ ö Ö\nfoo x y\n] ab - - -\n; U+015F U+015E\n' \u{7} - - -\n",
        );
        assert_eq!(m.get(LBR, 0), Some('ö'));
        assert_eq!(m.get(LBR, 2), None);
        assert_eq!(m.get(0x1B, 0), None);
        assert_eq!(m.get(0x27, 1), Some('Ş'));
        assert_eq!(m.get(QUOTE, 0), None);
    }

    #[test]
    fn cyrillic_preset_follows_windows_uzbek_layout() {
        let m = preset(CYRILLIC).unwrap();
        let k = |l: &str| scan_of(l).unwrap();
        assert_eq!(m.lookup(k("o"), false, false, false), Some('ў'));
        assert_eq!(m.lookup(k("s"), false, false, false), Some('қ'));
        assert_eq!(m.lookup(k("-"), true, false, false), Some('Ғ'));
        assert_eq!(m.lookup(k("="), false, true, false), Some('Ҳ'));
        assert_eq!(m.lookup(k("3"), true, false, false), Some('№'));
        assert_eq!(m.lookup(k("3"), false, false, false), None);
        assert_eq!(m.lookup(k("/"), true, false, false), Some(','));
        assert_eq!(m.lookup(LBR, false, false, true), Some('['));
        assert_eq!(m.lookup(LBR, false, false, false), Some('х'));
    }

    #[test]
    fn competitor_layouts_match_their_published_tables() {
        let k = |l: &str| scan_of(l).unwrap();
        let ki = preset("kiritish").unwrap();
        assert_eq!(ki.lookup(LBR, false, false, false), Some('ö'));
        assert_eq!(ki.lookup(k(";"), true, false, false), Some('Ş'));
        assert_eq!(ki.lookup(k("`"), false, false, false), Some('ʼ'));
        assert_eq!(ki.lookup(k("l"), false, false, true), Some('”'));
        let tz = preset("tezyechim").unwrap();
        assert_eq!(tz.lookup(LBR, false, false, false), Some('ç'));
        assert_eq!(tz.lookup(k("="), true, false, false), Some('Ş'));
        assert_eq!(tz.lookup(k("-"), false, false, true), Some('-'));
        assert_eq!(tz.lookup(QUOTE, false, false, true), Some('ʼ'));
        // Tutuq hamma tartibda o'girish natijasi bilan bir xil: ʼ (U+02BC).
        let ti = preset("tilmoch").unwrap();
        assert_eq!(ti.lookup(k("`"), false, false, false), Some('\u{2BC}'));
    }

    #[test]
    fn presets_exist() {
        for name in PRESETS {
            assert!(!preset(name).unwrap().is_empty());
        }
        assert!(preset("nope").is_none());
    }
}
