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
//! enabled_apps=
//! disabled_defaults_rev=2
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
//! `disabled_defaults_rev` — fayl yozilganda standart ro'yxatning qaysi tahriri ma'lum edi. Keyin qo'shilgan
//! standart dasturlar (`DEFAULT_DISABLED`) eski faylga o'qishda qo'shiladi, `enabled_apps` dagilaridan tashqari:
//! u — foydalanuvchi ro'yxatdan ataylab olib tashlagan standart dasturlar (yozishda hisoblanadi).
//!
//! `hotkey` — tray ilovasining global tugmasi: belgilangan matnni yangi alifboga o'giradi (`hotkey_keys`).
//! `hotkey_old_keys` — teskari yo'nalish (yangi → eski), bo'sh = o'chiq. IME bu kalitlarni ishlatmaydi.
//!
//! `update_check` — tray ilovasi kuniga bir marta `bitg.uz/update/stable.json` ni o'qiydi. `whats_new` —
//! yangilangandan keyin «Yangiliklar» bir marta ochiladi; `last_seen` — ko'rilgan oxirgi versiya.
//!
//! `exceptions.txt`: `so'z` — o'girilmaydi, `so'z=yozuv` — shaxsiy lug'at (`bitg_core::user`).

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
    /// Tray ilovasi kuniga bir marta yangi versiyani tekshiradi (IME ishlatmaydi).
    pub update_check: bool,
    /// Yangilangandan keyin «Yangiliklar» sahifasi bir marta ochiladi.
    pub whats_new: bool,
    /// Foydalanuvchi «Yangiliklar» ni ko'rgan oxirgi versiya (bo'sh — hali hech qaysi).
    pub last_seen: String,
}

/// Belgilangan matnni o'girish tugmasining standart birikmasi.
pub const DEFAULT_HOTKEY: &str = "ctrl+shift+y";

/// Standart holatda tegilmaydigan dasturlar va ular qaysi tahrirda qo'shilgani: terminallar (buyruq va parol
/// yoziladi; 0), Windows kirish/UAC oynalari va parol menejerlari (maydon turi har doim ham TSF orqali kelmaydi,
/// #41; 1), masofaviy ish stoli va virtual mashina oynalari (tugmalar boshqa kompyuterga ketadi, u yerda parol
/// ham yoziladi; 2). Yangi dastur qo'shilsa — `DISABLED_DEFAULTS_REV` oshiriladi va dastur shu raqam bilan.
pub const DEFAULT_DISABLED: &[(&str, u32)] = &[
    ("logonui.exe", 1),
    ("consent.exe", 1),
    ("credentialuibroker.exe", 1),
    ("keepass.exe", 1),
    ("keepassxc.exe", 1),
    ("1password.exe", 1),
    ("bitwarden.exe", 1),
    ("enpass.exe", 1),
    ("mstsc.exe", 2),
    ("msrdc.exe", 2),
    ("vmconnect.exe", 2),
    ("vmware.exe", 2),
    ("vmplayer.exe", 2),
    ("virtualboxvm.exe", 2),
    ("anydesk.exe", 2),
    ("teamviewer.exe", 2),
    ("termius.exe", 0),
    ("windowsterminal.exe", 0),
    ("openconsole.exe", 0),
    ("conhost.exe", 0),
    ("putty.exe", 0),
    ("kitty.exe", 0),
    ("mintty.exe", 0),
    ("alacritty.exe", 0),
    ("wezterm-gui.exe", 0),
    ("mobaxterm.exe", 0),
];

/// `DEFAULT_DISABLED` ning joriy tahriri (eng katta raqam).
pub const DISABLED_DEFAULTS_REV: u32 = 2;

/// Standart ro'yxat nomlari (`DEFAULT_DISABLED` tartibida).
pub const DEFAULT_DISABLED_APPS: &[&str] = &{
    let mut out = [""; DEFAULT_DISABLED.len()];
    let mut i = 0;
    while i < out.len() {
        out[i] = DEFAULT_DISABLED[i].0;
        i += 1;
    }
    out
};

/// Standart ro'yxatdagi, lekin `disabled` da yo'q dasturlar: foydalanuvchi ularni ataylab yoqqan.
pub fn removed_defaults(disabled: &[String]) -> Vec<String> {
    DEFAULT_DISABLED_APPS
        .iter()
        .filter(|d| !disabled.iter().any(|a| a == *d))
        .map(|d| d.to_string())
        .collect()
}

fn app_list(v: &str) -> Vec<String> {
    v.split([',', ';'])
        .map(|a| a.trim().to_lowercase())
        .filter(|a| !a.is_empty())
        .collect()
}

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
            update_check: true,
            whats_new: true,
            last_seen: String::new(),
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
    // Fayldagi ro'yxat, ataylab yoqilganlar va standart ro'yxat tahriri (kalit yo'q — 0, ya'ni #49 dan oldin).
    let mut listed: Option<Vec<String>> = None;
    let mut enabled_apps = Vec::new();
    let mut rev = 0u32;
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
            "update_check" => c.update_check = flag(v).unwrap_or(c.update_check),
            "whats_new" => c.whats_new = flag(v).unwrap_or(c.whats_new),
            "last_seen" => {
                let v = v.trim();
                if v.len() <= 32
                    && v.chars()
                        .all(|c| c.is_ascii_alphanumeric() || ".-+".contains(c))
                {
                    c.last_seen = v.to_string();
                }
            }
            "disabled_apps" => listed = Some(app_list(v)),
            "enabled_apps" => enabled_apps = app_list(v),
            "disabled_defaults_rev" => rev = v.trim().parse().unwrap_or(0),
            "tutuq" => {
                let mut it = v.trim().chars();
                if let (Some(ch), None) = (it.next(), it.next()) {
                    c.opts.tutuq = ch;
                }
            }
            _ => {}
        }
    }
    c.disabled_apps = match listed {
        // Ro'yxat yozilgan: fayldan keyin qo'shilgan standartlar ham (ataylab yoqilganlaridan tashqari).
        Some(mut apps) => {
            for &(d, added) in DEFAULT_DISABLED {
                if added > rev
                    && !enabled_apps.iter().any(|a| a == d)
                    && !apps.iter().any(|a| a == d)
                {
                    apps.push(d.to_string());
                }
            }
            apps
        }
        None => DEFAULT_DISABLED_APPS
            .iter()
            .filter(|d| !enabled_apps.iter().any(|a| a == *d))
            .map(|d| d.to_string())
            .collect(),
    };
    // Istisnolar va `so'z=yozuv` qoidalari bitta faylda (`bitg_core::user`).
    let (never, rules) = bitg_core::user::parse(exceptions);
    c.opts.exceptions = never;
    c.opts.rules = rules;
    c
}

/// `settings.ini` matnida kalit aniq yozilganmi: qiymati (`Some`) yoki yo'q/noto'g'ri (`None`). Takror kalitda
/// oxirgisi (`parse` dagidek). Tray ilovasi shu bilan foydalanuvchi tanlovini o'rnatuvchi standartidan ajratadi.
pub fn explicit_flag(settings: &str, key: &str) -> Option<bool> {
    let mut found = None;
    for line in settings.lines() {
        let line = line.trim().trim_start_matches('\u{FEFF}');
        if let Some((k, v)) = line.split_once('=')
            && k.trim().eq_ignore_ascii_case(key)
            && let Some(b) = flag(v)
        {
            found = Some(b);
        }
    }
    found
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
         undo_backspace={}\nundo_remember={}\nfallback={}\ndisabled_apps={}\nenabled_apps={}\n\
         disabled_defaults_rev={}\nauto_convert={}\nkeymap={}\nlayout={}\n\
         hotkey={}\nhotkey_keys={}\nhotkey_old_keys={}\nupdate_check={}\nwhats_new={}\nlast_seen={}\n",
        bit(c.enabled),
        bit(c.opts.english_guard),
        bit(c.opts.cyrillic),
        bit(c.opts.restore_apostrophes),
        c.opts.tutuq,
        bit(c.undo_backspace),
        bit(c.undo_remember),
        bit(c.fallback),
        c.disabled_apps.join(", "),
        removed_defaults(&c.disabled_apps).join(", "),
        DISABLED_DEFAULTS_REV,
        bit(c.auto_convert),
        bit(c.keymap),
        c.layout,
        bit(c.hotkey),
        keys_value(&c.hotkey_keys),
        keys_value(&c.hotkey_old_keys),
        bit(c.update_check),
        bit(c.whats_new),
        c.last_seen,
    )
}

#[cfg(test)]
thread_local! {
    /// Testlar uchun `%APPDATA%` o'rnini bosuvchi papka: `Some(None)` — papka yo'q holati.
    static TEST_BASE: std::cell::RefCell<Option<Option<std::path::PathBuf>>> = const { std::cell::RefCell::new(None) };
}

/// Windows bilim papkasi (`FOLDERID_*`). Jarayon muhitidagi `%APPDATA%` ishlatilmaydi: uni bitg yuklangan
/// ilovani ishga tushirgan jarayon o'zgartira oladi.
#[cfg(windows)]
fn known_folder(id: &windows::core::GUID) -> Option<std::path::PathBuf> {
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{KF_FLAG_DONT_VERIFY, SHGetKnownFolderPath};
    // SAFETY: qaytgan satr CoTaskMemAlloc bilan ajratilgan; nusxa olingach bo'shatiladi.
    unsafe {
        let p = SHGetKnownFolderPath(id, KF_FLAG_DONT_VERIFY, None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const core::ffi::c_void));
        s.filter(|s| !s.is_empty()).map(std::path::PathBuf::from)
    }
}

/// Foydalanuvchining Roaming AppData papkasi (`%APPDATA%`).
pub fn roaming_app_data() -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        known_folder(&windows::Win32::UI::Shell::FOLDERID_RoamingAppData)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("APPDATA").map(std::path::PathBuf::from)
    }
}

/// Foydalanuvchining Local AppData papkasi (`%LOCALAPPDATA%`).
pub fn local_app_data() -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        known_folder(&windows::Win32::UI::Shell::FOLDERID_LocalAppData)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from)
    }
}

/// `C:\Windows\System32` (Windows o'zi beradi, muhit o'zgaruvchisi emas). Tizim dasturlari shu yerdan ochiladi.
pub fn system_dir() -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        known_folder(&windows::Win32::UI::Shell::FOLDERID_System)
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// `%APPDATA%\bitg`: sozlamalar, istisnolar, klaviatura xaritasi.
pub fn dir() -> Option<std::path::PathBuf> {
    #[cfg(test)]
    if let Some(base) = TEST_BASE.with(|b| b.borrow().clone()) {
        return base.map(|b| b.join("bitg"));
    }
    roaming_app_data().map(|b| b.join("bitg"))
}

/// `%LOCALAPPDATA%\bitg`: loglar va WebView ma'lumotlari.
pub fn local_dir() -> Option<std::path::PathBuf> {
    local_app_data().map(|b| b.join("bitg"))
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
    fn explicit_flag_only_when_written() {
        assert_eq!(explicit_flag("", "update_check"), None);
        assert_eq!(explicit_flag("enabled=1\n", "update_check"), None);
        assert_eq!(explicit_flag("update_check=maybe", "update_check"), None);
        assert_eq!(explicit_flag("# update_check=0", "update_check"), None);
        assert_eq!(
            explicit_flag("\u{FEFF}Update_Check = 0\n", "update_check"),
            Some(false)
        );
        assert_eq!(
            explicit_flag("update_check=0\nupdate_check=1", "update_check"),
            Some(true)
        );
    }

    #[test]
    fn defaults_when_empty() {
        let c = parse("", "");
        assert_eq!(
            (
                c.enabled,
                c.undo_backspace,
                c.opts.english_guard,
                c.opts.cyrillic
            ),
            (true, true, true, true)
        );
        assert_eq!(c.opts.tutuq, '\u{02BC}');
    }

    #[test]
    fn skips_blank_and_semicolon_lines_and_bad_flags() {
        let c = parse(
            "\n; izoh\nenabled=maybe\nfallback=no\n   \nundo_remember=ha\n",
            "\n\nChoy\n",
        );
        assert_eq!(
            (c.enabled, c.fallback, c.undo_remember),
            (true, false, true)
        );
        assert_eq!(c.opts.exceptions, vec!["choy".to_string()]);
    }

    #[test]
    fn reads_values_and_ignores_garbage() {
        let c = parse(
            "\u{FEFF}[bitg]\n# izoh\nenabled=0\nenglish_guard = off\ntutuq='\nundo_backspace=yo'q\nfoo=bar\nbroken\n",
            "Shoh\n# izoh\n  O‘zbekcha \nshoh\n",
        );
        assert_eq!(
            (c.enabled, c.opts.english_guard, c.undo_backspace),
            (false, false, false)
        );
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
        assert!(c.app_disabled("Termius.exe"));
        assert!(!c.app_disabled("Telegram.exe"));
        let c = parse(
            "disabled_apps=
fallback=0",
            "",
        );
        assert!(!c.app_disabled("Termius.exe"));
        assert!(!c.fallback);
        let c = parse("disabled_apps = Foo.exe; bar.exe", "");
        assert!(c.app_disabled("foo.EXE"));
        assert!(c.app_disabled("bar.exe"));
    }

    #[test]
    fn new_defaults_reach_old_files() {
        // #49 dan oldingi fayl (tahrir yo'q): terminallar ro'yxatidan keyin qo'shilganlar qo'shiladi.
        let old = parse("disabled_apps=termius.exe, putty.exe\n", "");
        for a in [
            "termius.exe",
            "putty.exe",
            "keepass.exe",
            "mstsc.exe",
            "anydesk.exe",
        ] {
            assert!(old.app_disabled(a), "{a}");
        }
        // Foydalanuvchi o'chirgan eski standart qaytmaydi.
        assert!(!old.app_disabled("windowsterminal.exe"));
        // 1-tahrirdagi fayl: faqat 2-tahrir qo'shiladi, ataylab yoqilgani qaytmaydi.
        let one = parse(
            "disabled_apps=termius.exe\nenabled_apps=keepass.exe, mstsc.exe\ndisabled_defaults_rev=1\n",
            "",
        );
        assert!(!one.app_disabled("keepass.exe"), "rev 1 app not re-added");
        assert!(!one.app_disabled("mstsc.exe"), "re-enabled by the user");
        assert!(one.app_disabled("vmconnect.exe") && one.app_disabled("termius.exe"));
        // Joriy tahrir: fayl aynan shunday.
        let cur = parse(
            &format!("disabled_apps=a.exe\ndisabled_defaults_rev={DISABLED_DEFAULTS_REV}\n"),
            "",
        );
        assert_eq!(cur.disabled_apps, ["a.exe"]);
        // Ro'yxat yozilmagan: standart, ataylab yoqilganlaridan tashqari.
        let none = parse("enabled_apps=conhost.exe\n", "");
        assert!(!none.app_disabled("conhost.exe") && none.app_disabled("mstsc.exe"));
        // Yaroqsiz tahrir — 0 (hammasi qo'shiladi).
        assert!(parse("disabled_apps=\ndisabled_defaults_rev=x\n", "").app_disabled("logonui.exe"));
    }

    #[test]
    fn removals_survive_save() {
        let mut c = parse("", "");
        c.disabled_apps
            .retain(|a| a != "mstsc.exe" && a != "conhost.exe");
        c.disabled_apps.push("foo.exe".into());
        let ini = to_ini(&c);
        assert!(ini.contains(&format!("disabled_defaults_rev={DISABLED_DEFAULTS_REV}\n")));
        assert!(
            ini.contains("enabled_apps=mstsc.exe, conhost.exe\n"),
            "{ini}"
        );
        let back = parse(&ini, "");
        assert_eq!(back.disabled_apps, c.disabled_apps);
        // Kelajakdagi tahrir yangi dastur qo'shsa ham, yoqilganlari qaytmaydi (eski raqamli fayl kabi).
        let older = ini.replace(
            &format!("disabled_defaults_rev={DISABLED_DEFAULTS_REV}"),
            "disabled_defaults_rev=0",
        );
        let again = parse(&older, "");
        assert!(!again.app_disabled("mstsc.exe") && !again.app_disabled("conhost.exe"));
        assert!(again.app_disabled("foo.exe") && again.app_disabled("keepass.exe"));
    }

    #[test]
    fn default_names_match_table() {
        assert_eq!(DEFAULT_DISABLED_APPS.len(), DEFAULT_DISABLED.len());
        for (n, (d, rev)) in DEFAULT_DISABLED_APPS.iter().zip(DEFAULT_DISABLED) {
            assert_eq!(n, d);
            assert!(*rev <= DISABLED_DEFAULTS_REV);
            assert_eq!(*d, d.to_lowercase());
        }
        assert!(
            DEFAULT_DISABLED
                .iter()
                .any(|&(_, r)| r == DISABLED_DEFAULTS_REV)
        );
    }

    #[test]
    fn to_ini_round_trips() {
        let mut c = parse(
            &format!(
                "enabled=0\nfallback=0\ntutuq='\ndisabled_apps=a.exe;b.exe\ndisabled_defaults_rev={DISABLED_DEFAULTS_REV}"
            ),
            "",
        );
        c.opts.restore_apostrophes = false;
        let back = parse(&to_ini(&c), "");
        assert_eq!(
            (back.enabled, back.fallback, back.opts.restore_apostrophes),
            (false, false, false)
        );
        assert_eq!((back.opts.english_guard, back.undo_backspace), (true, true));
        assert_eq!(back.opts.tutuq, '\'');
        assert_eq!(back.disabled_apps, vec!["a.exe", "b.exe"]);
        let empty = parse(
            &to_ini(&parse(
                &format!("disabled_apps=\ndisabled_defaults_rev={DISABLED_DEFAULTS_REV}"),
                "",
            )),
            "",
        );
        assert!(empty.disabled_apps.is_empty());
    }

    #[test]
    fn update_and_news_keys() {
        let d = parse("", "");
        assert!(d.update_check && d.whats_new && d.last_seen.is_empty());
        let c = parse("update_check=0\nwhats_new=off\nlast_seen=0.4.0-rc1", "");
        assert!(!c.update_check && !c.whats_new);
        assert_eq!(c.last_seen, "0.4.0-rc1");
        let back = parse(&to_ini(&c), "");
        assert_eq!(
            (back.update_check, back.whats_new, back.last_seen.as_str()),
            (false, false, "0.4.0-rc1")
        );
        // Yaroqsiz qiymat — e'tiborsiz (fayl qo'lda buzilgan bo'lsa ham qator buzilmaydi).
        for bad in [
            "last_seen=1 2",
            "last_seen=<x>",
            &format!("last_seen={}", "9".repeat(40)),
        ] {
            assert_eq!(parse(bad, "").last_seen, "", "{bad}");
        }
        let rules = parse("", "shop\nbog'cha=bog'ça\n");
        assert_eq!(rules.opts.exceptions, ["shop"]);
        assert_eq!(rules.opts.rules[0].to, "bog'ça");
    }

    #[test]
    fn keymap_settings() {
        let c = parse("", "");
        assert_eq!(
            (c.auto_convert, c.keymap, c.layout.as_str()),
            (true, false, "tilmoch")
        );
        let c = parse(
            "auto_convert=0
keymap=1
layout=Turkish",
            "",
        );
        assert_eq!(
            (c.auto_convert, c.keymap, c.layout.as_str()),
            (false, true, "turkish")
        );
        let back = parse(&to_ini(&c), "");
        assert_eq!(
            (back.auto_convert, back.keymap, back.layout.as_str()),
            (false, true, "turkish")
        );
        assert_eq!(parse("layout=dvorak", "").layout, "tilmoch");
        let mut c = parse("layout=custom", "");
        assert_eq!(c.resolve_keymap("[ x X - -").get(0x1A, 0), Some('x'));
        c.layout = "altgr".into();
        assert_eq!(c.resolve_keymap("").get(0x18, 2), Some('ö'));
        c.layout = "nope".into();
        assert_eq!(c.resolve_keymap(""), keymap::preset("tilmoch").unwrap());
    }

    #[test]
    fn hotkey_settings() {
        let c = parse("", "");
        assert_eq!(
            (c.hotkey, c.hotkey_keys.as_str(), c.hotkey_old_keys.as_str()),
            (true, DEFAULT_HOTKEY, "")
        );
        let c = parse(
            "hotkey=0\nhotkey_keys = Ctrl + Shift + B\nhotkey_old_keys=ctrl+shift+o",
            "",
        );
        assert_eq!((c.hotkey, c.hotkey_keys.as_str()), (false, "ctrl+shift+b"));
        assert_eq!(c.hotkey_old_keys, "ctrl+shift+o");
        let back = parse(&to_ini(&c), "");
        assert_eq!(
            (back.hotkey, back.hotkey_keys.as_str()),
            (false, "ctrl+shift+b")
        );
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

    /// Faqat shu oqimdagi `dir()` ga ta'sir qiladi; haqiqiy `%APPDATA%\bitg` ga yozilmaydi.
    fn set_appdata(v: Option<&std::ffi::OsStr>) {
        TEST_BASE.with(|b| *b.borrow_mut() = Some(v.map(std::path::PathBuf::from)));
    }

    #[test]
    fn known_folders_are_absolute() {
        for d in [roaming_app_data(), local_app_data()].into_iter().flatten() {
            assert!(d.is_absolute(), "{}", d.display());
        }
        assert_eq!(local_dir(), local_app_data().map(|b| b.join("bitg")));
        #[cfg(windows)]
        {
            // muhitdagi qiymat emas, Windows'ning o'zi beradi
            assert!(roaming_app_data().is_some_and(|d| d.exists()));
            assert!(local_app_data().is_some_and(|d| d.exists()));
        }
    }

    #[test]
    fn files_in_appdata_dir() {
        let base = std::env::temp_dir().join(format!(
            "bitg-config-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        set_appdata(Some(base.as_os_str()));
        let d = base.join("bitg");
        let ex = d.join(EXCEPTIONS_FILE);

        assert_eq!(dir(), Some(d.clone()));
        assert_eq!(read_raw(), (String::new(), String::new()));
        assert_eq!(read_keymap(), "");
        assert_eq!(stamp(), [None, None, None]);

        add_exception("   ").unwrap();
        add_exception("a\nb").unwrap();
        add_exception("a\rb").unwrap();
        assert!(!d.exists());

        add_exception(" Shoh ").unwrap();
        assert_eq!(std::fs::read_to_string(&ex).unwrap(), "Shoh\n");
        add_exception("shoh").unwrap();
        assert_eq!(std::fs::read_to_string(&ex).unwrap(), "Shoh\n");

        std::fs::write(&ex, "Shoh\nchoy").unwrap();
        add_exception("Non").unwrap();
        add_exception("olma").unwrap();
        assert_eq!(
            std::fs::read_to_string(&ex).unwrap(),
            "Shoh\nchoy\nNon\nolma\n"
        );

        std::fs::write(d.join(SETTINGS_FILE), "enabled=0\n").unwrap();
        std::fs::write(d.join(KEYMAP_FILE), "[ x X - -\n").unwrap();
        let (s, e) = read_raw();
        assert_eq!(s, "enabled=0\n");
        assert_eq!(e, "Shoh\nchoy\nNon\nolma\n");
        let c = load();
        assert!(!c.enabled);
        assert_eq!(c.opts.exceptions, vec!["shoh", "choy", "non", "olma"]);
        assert_eq!(read_keymap(), "[ x X - -\n");
        let st = stamp();
        assert_eq!(
            st.map(|s| s.map(|(len, _)| len)),
            [Some(10), Some(19), Some(10)]
        );

        set_appdata(None);
        assert_eq!(dir(), None);
        assert_eq!(read_raw(), (String::new(), String::new()));
        assert_eq!(read_keymap(), "");
        assert_eq!(stamp(), [None, None, None]);
        add_exception("yangi").unwrap();
        assert!(load().opts.exceptions.is_empty());
        assert_eq!(
            std::fs::read_to_string(&ex).unwrap(),
            "Shoh\nchoy\nNon\nolma\n"
        );

        TEST_BASE.with(|b| *b.borrow_mut() = None);
        std::fs::remove_dir_all(&base).unwrap();
    }
}
