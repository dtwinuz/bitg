// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Sozlamalar: `%APPDATA%\bitg\settings.ini` (kalit=qiymat) va `%APPDATA%\bitg\exceptions.txt` (har qatorda so'z).
//! IME faqat o'qiydi; yozuvchi — tray/sozlamalar ilovasi. Fayl yo'q yoki buzuq bo'lsa — standart qiymatlar.
//!
//! ```ini
//! enabled=1
//! english_guard=1
//! cyrillic=1
//! tutuq=ʼ
//! undo_backspace=1
//! undo_remember=0
//! fallback=1
//! disabled_apps=termius.exe, windowsterminal.exe
//! auto_convert=1
//! keymap=0
//! layout=tilmoch
//! hotkey=1
//! hotkey_keys=ctrl+shift+y
//! hotkey_old_keys=
//! ```
//!
//! `auto_convert` — probeldan keyin so'zni o'girish. `keymap` — yangi harflarni tugmalardan yozish
//! (`layout`: tayyor tartib nomi yoki `custom` — `keymap.txt`).
//!
//! `fallback` — matnni TSF'ga bermaydigan dasturlar (Telegram Desktop): bosilgan harflar kuzatiladi va so'z
//! Backspace + qayta yozish bilan almashtiriladi. `disabled_apps` — bitg umuman ishlamaydigan dasturlar
//! (standart: terminallar, chunki u yerda parol va buyruqlar yoziladi). Bo'sh qiymat = hamma joyda.
//!
//! `hotkey` — tray ilovasining global tugmasi: belgilangan matnni yangi alifboga o'giradi (`hotkey_keys`).
//! `hotkey_old_keys` — teskari yo'nalish (yangi → eski), bo'sh = o'chiq. IME bu kalitlarni ishlatmaydi.

use bitg_core::Options;
use bitg_core::chars::fold_key;

use crate::keymap::{self, Keymap};

#[derive(Debug, Clone)]
pub struct Config {
    pub enabled: bool,
    pub undo_backspace: bool,
    /// Backspace bilan qaytarilgan so'z `exceptions.txt`ga ham yozilsinmi (standart: yo'q — faqat shu dasturda).
    pub undo_remember: bool,
    pub fallback: bool,
    /// Kichik harflarda: "termius.exe".
    pub disabled_apps: Vec<String>,
    pub opts: Options,
    /// Probeldan keyin so'zni yangi alifboga o'girish.
    pub auto_convert: bool,
    /// Yangi harflarni tugmalardan yozish (klaviatura tartibi).
    pub keymap: bool,
    /// `keymap::PRESETS` dan biri yoki `keymap::CUSTOM`.
    pub layout: String,
    /// Belgilangan matnni o'girish tugmasi (tray ilovasi ushlaydi).
    pub hotkey: bool,
    /// Tugmalar birikmasi, masalan `ctrl+shift+y` (kichik harflarda; tekshiruv tray ilovasida).
    pub hotkey_keys: String,
    /// Teskari yo'nalish (yangi → eski) tugmasi; bo'sh — o'chiq.
    pub hotkey_old_keys: String,
}

/// Belgilangan matnni o'girish tugmasining standart birikmasi.
pub const DEFAULT_HOTKEY: &str = "ctrl+shift+y";

/// Terminallar: buyruq va parol yoziladi — standart holatda tegilmaydi.
pub const DEFAULT_DISABLED_APPS: &[&str] = &[
    "termius.exe",
    "windowsterminal.exe",
    "openconsole.exe",
    "conhost.exe",
    "putty.exe",
    "kitty.exe",
    "mintty.exe",
    "alacritty.exe",
    "wezterm-gui.exe",
    "mobaxterm.exe",
];

impl Config {
    /// Tanlangan tartib. `custom` — `keymap.txt` matnidan; noma'lum nom — standart.
    pub fn resolve_keymap(&self, custom_text: &str) -> Keymap {
        if self.layout == keymap::CUSTOM {
            return Keymap::parse(custom_text);
        }
        keymap::preset(&self.layout)
            .unwrap_or_else(|| keymap::preset(keymap::DEFAULT_LAYOUT).unwrap_or_default())
    }

    pub fn app_disabled(&self, exe: &str) -> bool {
        let exe = exe.to_lowercase();
        self.disabled_apps.contains(&exe)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            undo_backspace: true,
            undo_remember: false,
            fallback: true,
            disabled_apps: DEFAULT_DISABLED_APPS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            opts: Options::default(),
            auto_convert: true,
            keymap: false,
            layout: keymap::DEFAULT_LAYOUT.into(),
            hotkey: true,
            hotkey_keys: DEFAULT_HOTKEY.into(),
            hotkey_old_keys: String::new(),
        }
    }
}

fn flag(v: &str) -> Option<bool> {
    match v.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "ha" => Some(true),
        "0" | "false" | "no" | "off" | "yoq" | "yo'q" => Some(false),
        _ => None,
    }
}

/// `settings.ini` va `exceptions.txt` matnidan. Noma'lum kalitlar e'tiborsiz.
pub fn parse(settings: &str, exceptions: &str) -> Config {
    let mut c = Config::default();
    for line in settings.lines() {
        let line = line.trim().trim_start_matches('\u{FEFF}');
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with(';')
            || line.starts_with('[')
        {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let k = k.trim().to_ascii_lowercase();
        match k.as_str() {
            "enabled" => c.enabled = flag(v).unwrap_or(c.enabled),
            "english_guard" => c.opts.english_guard = flag(v).unwrap_or(c.opts.english_guard),
            "cyrillic" => c.opts.cyrillic = flag(v).unwrap_or(c.opts.cyrillic),
            "restore_apostrophes" => {
                c.opts.restore_apostrophes = flag(v).unwrap_or(c.opts.restore_apostrophes)
            }
            "undo_backspace" => c.undo_backspace = flag(v).unwrap_or(c.undo_backspace),
            "undo_remember" => c.undo_remember = flag(v).unwrap_or(c.undo_remember),
            "fallback" => c.fallback = flag(v).unwrap_or(c.fallback),
            "auto_convert" => c.auto_convert = flag(v).unwrap_or(c.auto_convert),
            "keymap" => c.keymap = flag(v).unwrap_or(c.keymap),
            "layout" => {
                let v = v.trim().to_ascii_lowercase();
                if v == keymap::CUSTOM || keymap::PRESETS.contains(&v.as_str()) {
                    c.layout = v;
                }
            }
            "hotkey" => c.hotkey = flag(v).unwrap_or(c.hotkey),
            "hotkey_keys" => {
                let v = keys_value(v);
                if !v.is_empty() {
                    c.hotkey_keys = v;
                }
            }
            "hotkey_old_keys" => c.hotkey_old_keys = keys_value(v),
            "disabled_apps" => {
                c.disabled_apps = v
                    .split([',', ';'])
                    .map(|a| a.trim().to_lowercase())
                    .filter(|a| !a.is_empty())
                    .collect();
            }
            "tutuq" => {
                let mut it = v.trim().chars();
                if let (Some(ch), None) = (it.next(), it.next()) {
                    c.opts.tutuq = ch;
                }
            }
            _ => {}
        }
    }
    for w in exceptions.lines() {
        let w = w.trim().trim_start_matches('\u{FEFF}');
        if w.is_empty() || w.starts_with('#') {
            continue;
        }
        let k = fold_key(w);
        if !c.opts.exceptions.contains(&k) {
            c.opts.exceptions.push(k);
        }
    }
    c
}

/// Tugmalar birikmasi: probelsiz, kichik harflarda (`Ctrl + Shift + Y` → `ctrl+shift+y`).
pub fn keys_value(v: &str) -> String {
    v.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn bit(b: bool) -> &'static str {
    if b { "1" } else { "0" }
}

/// `parse` ning teskarisi: sozlamalar oynasi shu matnni yozadi (istisnolar alohida faylda).
pub fn to_ini(c: &Config) -> String {
    format!(
        "# bitg sozlamalari. Sozlamalar oynasi yozadi; qo'lda tahrirlash ham mumkin.\n\
         enabled={}\nenglish_guard={}\ncyrillic={}\nrestore_apostrophes={}\ntutuq={}\n\
         undo_backspace={}\nundo_remember={}\nfallback={}\ndisabled_apps={}\nauto_convert={}\nkeymap={}\nlayout={}\n\
         hotkey={}\nhotkey_keys={}\nhotkey_old_keys={}\n",
        bit(c.enabled),
        bit(c.opts.english_guard),
        bit(c.opts.cyrillic),
        bit(c.opts.restore_apostrophes),
        c.opts.tutuq,
        bit(c.undo_backspace),
        bit(c.undo_remember),
        bit(c.fallback),
        c.disabled_apps.join(", "),
        bit(c.auto_convert),
        bit(c.keymap),
        c.layout,
        bit(c.hotkey),
        keys_value(&c.hotkey_keys),
        keys_value(&c.hotkey_old_keys),
    )
}

/// `%APPDATA%\bitg`.
pub fn dir() -> Option<std::path::PathBuf> {
    std::env::var_os("APPDATA").map(|b| std::path::PathBuf::from(b).join("bitg"))
}

pub const SETTINGS_FILE: &str = "settings.ini";
pub const EXCEPTIONS_FILE: &str = "exceptions.txt";
pub const KEYMAP_FILE: &str = "keymap.txt";

/// `keymap.txt` matni (yo'q bo'lsa — bo'sh).
pub fn read_keymap() -> String {
    dir()
        .and_then(|d| std::fs::read_to_string(d.join(KEYMAP_FILE)).ok())
        .unwrap_or_default()
}

/// So'zni `exceptions.txt` oxiriga qo'shadi (Backspace bilan o'girish bekor qilinganda).
/// Bor bo'lsa — qo'shilmaydi.
pub fn add_exception(word: &str) -> std::io::Result<()> {
    use std::io::Write;
    let word = word.trim();
    if word.is_empty() || word.contains(['\n', '\r']) {
        return Ok(());
    }
    let Some(dir) = dir() else { return Ok(()) };
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(EXCEPTIONS_FILE);
    let old = std::fs::read_to_string(&path).unwrap_or_default();
    let key = fold_key(word);
    if old.lines().any(|l| fold_key(l.trim()) == key) {
        return Ok(());
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let sep = if old.is_empty() || old.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    writeln!(f, "{sep}{word}")
}

/// Ikkala fayl matni (yo'q bo'lsa — bo'sh).
pub fn read_raw() -> (String, String) {
    let Some(dir) = dir() else {
        return Default::default();
    };
    let s = std::fs::read_to_string(dir.join(SETTINGS_FILE)).unwrap_or_default();
    let e = std::fs::read_to_string(dir.join(EXCEPTIONS_FILE)).unwrap_or_default();
    (s, e)
}

/// Diskdan o'qish (Windows: %APPDATA%\bitg). Xato — standart.
pub fn load() -> Config {
    let (s, e) = read_raw();
    parse(&s, &e)
}

/// Fayllar o'zgarganini bilish uchun: (o'lcham, o'zgartirilgan vaqt) har fayl uchun.
pub type Stamp = [Option<(u64, std::time::SystemTime)>; 3];

pub fn stamp() -> Stamp {
    let one = |name: &str| {
        let m = std::fs::metadata(dir()?.join(name)).ok()?;
        Some((m.len(), m.modified().ok()?))
    };
    [one(SETTINGS_FILE), one(EXCEPTIONS_FILE), one(KEYMAP_FILE)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_empty() {
        let c = parse("", "");
        assert!(c.enabled && c.undo_backspace && c.opts.english_guard && c.opts.cyrillic);
        assert_eq!(c.opts.tutuq, '\u{02BC}');
    }

    #[test]
    fn reads_values_and_ignores_garbage() {
        let c = parse(
            "\u{FEFF}[bitg]\n# izoh\nenabled=0\nenglish_guard = off\ntutuq='\nundo_backspace=yo'q\nfoo=bar\nbroken\n",
            "Shoh\n# izoh\n  O‘zbekcha \nshoh\n",
        );
        assert!(!c.enabled && !c.opts.english_guard && !c.undo_backspace);
        assert!(c.opts.cyrillic);
        assert_eq!(c.opts.tutuq, '\'');
        assert_eq!(
            c.opts.exceptions,
            vec!["shoh".to_string(), "o'zbekcha".to_string()]
        );
    }

    #[test]
    fn disabled_apps_default_and_override() {
        let c = parse("", "");
        assert!(c.app_disabled("Termius.exe") && !c.app_disabled("Telegram.exe"));
        let c = parse(
            "disabled_apps=
fallback=0",
            "",
        );
        assert!(!c.app_disabled("Termius.exe") && !c.fallback);
        let c = parse("disabled_apps = Foo.exe; bar.exe", "");
        assert!(c.app_disabled("foo.EXE") && c.app_disabled("bar.exe"));
    }

    #[test]
    fn to_ini_round_trips() {
        let mut c = parse(
            "enabled=0\nfallback=0\ntutuq='\ndisabled_apps=a.exe;b.exe",
            "",
        );
        c.opts.restore_apostrophes = false;
        let back = parse(&to_ini(&c), "");
        assert!(!back.enabled && !back.fallback && !back.opts.restore_apostrophes);
        assert!(back.opts.english_guard && back.undo_backspace);
        assert_eq!(back.opts.tutuq, '\'');
        assert_eq!(back.disabled_apps, vec!["a.exe", "b.exe"]);
        let empty = parse(&to_ini(&parse("disabled_apps=", "")), "");
        assert!(empty.disabled_apps.is_empty());
    }

    #[test]
    fn keymap_settings() {
        let c = parse("", "");
        assert!(c.auto_convert && !c.keymap && c.layout == "tilmoch");
        let c = parse(
            "auto_convert=0
keymap=1
layout=Turkish",
            "",
        );
        assert!(!c.auto_convert && c.keymap && c.layout == "turkish");
        let back = parse(&to_ini(&c), "");
        assert!(!back.auto_convert && back.keymap && back.layout == "turkish");
        assert_eq!(parse("layout=dvorak", "").layout, "tilmoch");
        let mut c = parse("layout=custom", "");
        assert_eq!(c.resolve_keymap("[ x X - -").get(0x1A, 0), Some('x'));
        c.layout = "altgr".into();
        assert_eq!(c.resolve_keymap("").get(0x18, 2), Some('ö'));
    }

    #[test]
    fn hotkey_settings() {
        let c = parse("", "");
        assert!(c.hotkey && c.hotkey_keys == DEFAULT_HOTKEY && c.hotkey_old_keys.is_empty());
        let c = parse(
            "hotkey=0\nhotkey_keys = Ctrl + Shift + B\nhotkey_old_keys=ctrl+shift+o",
            "",
        );
        assert!(!c.hotkey && c.hotkey_keys == "ctrl+shift+b");
        assert_eq!(c.hotkey_old_keys, "ctrl+shift+o");
        let back = parse(&to_ini(&c), "");
        assert!(!back.hotkey && back.hotkey_keys == "ctrl+shift+b");
        assert_eq!(back.hotkey_old_keys, "ctrl+shift+o");
        assert_eq!(parse("hotkey_keys=", "").hotkey_keys, DEFAULT_HOTKEY);
        assert!(
            parse(&to_ini(&parse("", "")), "")
                .hotkey_old_keys
                .is_empty()
        );
    }

    #[test]
    fn bad_tutuq_keeps_default() {
        assert_eq!(parse("tutuq=ab", "").opts.tutuq, '\u{02BC}');
    }
}
