// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Sozlamalar: Windows bilan bir xil format (`settings.ini` key=value, `exceptions.txt` — har qatorda so'z).
//! Linux'da joyi: `$XDG_CONFIG_HOME/bitg/` (bo'lmasa `~/.config/bitg/`); macOS'da `~/Library/Application Support/bitg/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use bitg_core::Options;
use bitg_core::chars::{DEFAULT_TUTUQ, fold_key};

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enabled: bool,
    pub auto_convert: bool,
    pub english_guard: bool,
    pub cyrillic: bool,
    pub restore_apostrophes: bool,
    pub tutuq: char,
    pub undo_backspace: bool,
    pub undo_remember: bool,
    /// Fayldagi istisnolar (`fold_key` ko'rinishida).
    pub exceptions: Vec<String>,
    /// Fayldagi `so'z=yozuv` qoidalari (`bitg_core::user`).
    pub rules: Vec<bitg_core::user::Rule>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_convert: true,
            english_guard: true,
            cyrillic: true,
            restore_apostrophes: true,
            tutuq: DEFAULT_TUTUQ,
            undo_backspace: true,
            undo_remember: false,
            exceptions: Vec::new(),
            rules: Vec::new(),
        }
    }
}

impl Settings {
    /// Yozish paytida o'girish yoqilganmi.
    pub fn converting(&self) -> bool {
        self.enabled && self.auto_convert
    }

    pub fn options(&self, extra: &[String]) -> Options {
        let mut exceptions = self.exceptions.clone();
        exceptions.extend(extra.iter().cloned());
        Options {
            tutuq: self.tutuq,
            english_guard: self.english_guard,
            cyrillic: self.cyrillic,
            exceptions,
            restore_apostrophes: self.restore_apostrophes,
            rules: self.rules.clone(),
        }
    }

    /// `settings.ini` matnini o'qiydi. Noma'lum kalit va noto'g'ri qiymat e'tiborsiz (standart qoladi).
    pub fn parse(ini: &str, exceptions: &str) -> Self {
        let mut s = Self::default();
        for line in ini.trim_start_matches('\u{feff}').lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(['#', ';', '[']) {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let (k, v) = (k.trim().to_ascii_lowercase(), v.trim());
            let slot = match k.as_str() {
                "enabled" => &mut s.enabled,
                "auto_convert" => &mut s.auto_convert,
                "english_guard" => &mut s.english_guard,
                "cyrillic" => &mut s.cyrillic,
                "restore_apostrophes" => &mut s.restore_apostrophes,
                "undo_backspace" => &mut s.undo_backspace,
                "undo_remember" => &mut s.undo_remember,
                "tutuq" => {
                    let mut it = v.chars();
                    if let (Some(c), None) = (it.next(), it.next()) {
                        s.tutuq = c;
                    }
                    continue;
                }
                _ => continue,
            };
            if let Some(b) = flag(v) {
                *slot = b;
            }
        }
        s.exceptions = parse_exceptions(exceptions);
        s.rules = bitg_core::user::parse(exceptions).1;
        s
    }
}

fn flag(v: &str) -> Option<bool> {
    match v.to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "ha" => Some(true),
        "0" | "false" | "no" | "off" | "yoq" | "yo'q" | "yo‘q" => Some(false),
        _ => None,
    }
}

/// Istisno so'zlar (`so'z=yozuv` qoidalarisiz), `fold_key` ko'rinishida, saralangan.
pub fn parse_exceptions(text: &str) -> Vec<String> {
    let mut out = bitg_core::user::parse(text).0;
    out.sort();
    out.dedup();
    out
}

/// Sozlamalar papkasi: macOS'da `~/Library/Application Support/bitg`.
#[cfg(target_os = "macos")]
pub fn config_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").filter(|v| !v.is_empty())?;
    Some(PathBuf::from(home).join("Library/Application Support/bitg"))
}

/// Sozlamalar papkasi: `$XDG_CONFIG_HOME/bitg` yoki `$HOME/.config/bitg`.
#[cfg(not(target_os = "macos"))]
pub fn config_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("bitg"))
}

/// Fayllar o'zgarganini (hajm + vaqt) ko'pi bilan sekundiga bir marta tekshirib, qayta o'qiydi.
#[derive(Debug)]
pub struct Watched {
    dir: Option<PathBuf>,
    stamp: Option<[(u64, SystemTime); 2]>,
    checked: Option<Instant>,
    pub settings: Settings,
}

impl Watched {
    pub fn new(dir: Option<PathBuf>) -> Self {
        let mut w = Self {
            dir,
            stamp: None,
            checked: None,
            settings: Settings::default(),
        };
        w.reload();
        w
    }

    fn files(&self) -> Option<(PathBuf, PathBuf)> {
        let d = self.dir.as_ref()?;
        Some((d.join("settings.ini"), d.join("exceptions.txt")))
    }

    fn reload(&mut self) {
        self.checked = Some(Instant::now());
        let Some((ini, exc)) = self.files() else {
            return;
        };
        let stamp = [meta(&ini), meta(&exc)];
        if self.stamp == Some(stamp) {
            return;
        }
        self.stamp = Some(stamp);
        let read = |p: &Path| fs::read_to_string(p).unwrap_or_default();
        self.settings = Settings::parse(&read(&ini), &read(&exc));
    }

    /// O'zgargan bo'lsa qayta o'qiydi; `true` — sozlamalar yangilandi.
    pub fn refresh(&mut self) -> bool {
        if self
            .checked
            .is_some_and(|t| t.elapsed() < Duration::from_secs(1))
        {
            return false;
        }
        let before = self.settings.clone();
        self.reload();
        self.settings != before
    }

    /// Backspace bilan qaytarilgan so'zni `exceptions.txt` ga qo'shadi (`undo_remember=1`).
    pub fn remember(&mut self, word: &str) -> std::io::Result<()> {
        let Some((_, exc)) = self.files() else {
            return Ok(());
        };
        if let Some(d) = exc.parent() {
            fs::create_dir_all(d)?;
        }
        let mut text = fs::read_to_string(&exc).unwrap_or_default();
        let key = fold_key(word);
        if parse_exceptions(&text).contains(&key) {
            return Ok(());
        }
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(word);
        text.push('\n');
        write_private(&exc, &text)?;
        self.stamp = None;
        self.reload();
        Ok(())
    }
}

/// `exceptions.txt` foydalanuvchi yozgan so'zlarni saqlaydi: boshqa foydalanuvchilar o'qiy olmasin (Unix'da 0600)
/// va yozish yarmida uzilsa eski ro'yxat buzilmasin (vaqtinchalik fayl + rename).
fn write_private(path: &Path, text: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".{}.tmp", std::process::id()));
    let tmp = std::path::PathBuf::from(name);
    let _ = fs::remove_file(&tmp);
    let mut o = fs::OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut o, 0o600);
    let written = o.open(&tmp).and_then(|mut f| {
        f.write_all(text.as_bytes())?;
        f.sync_all()
    });
    written
        .and_then(|()| fs::rename(&tmp, path))
        .inspect_err(|_| {
            let _ = fs::remove_file(&tmp);
        })
}

fn meta(p: &Path) -> (u64, SystemTime) {
    fs::metadata(p)
        .map(|m| (m.len(), m.modified().unwrap_or(SystemTime::UNIX_EPOCH)))
        .unwrap_or((0, SystemTime::UNIX_EPOCH))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_windows_format() {
        let s = Settings::parse(
            "\u{feff}[bitg]\n# izoh\nenabled = yo'q\nenglish_guard=0\ntutuq='\nundo_remember=ha\nnoma'lum=1\ncyrillic=xato\n",
            "# istisno\nShop\nma’no\nshop\n",
        );
        assert!(!s.enabled);
        assert!(!s.english_guard);
        assert_eq!(s.tutuq, '\'');
        assert!(s.undo_remember);
        assert!(s.cyrillic, "noto'g'ri qiymat standartni o'zgartirmaydi");
        assert_eq!(s.exceptions, vec!["ma'no".to_string(), "shop".to_string()]);
    }

    #[test]
    fn ignores_malformed_lines() {
        let s = Settings::parse(
            "\n\nenabled\nauto_convert = no\ntutuq=ab\ntutuq=\nrestore_apostrophes=off\nundo_backspace=false\n",
            "\n#x\n  Choy  \n\n",
        );
        assert_eq!(
            s,
            Settings {
                auto_convert: false,
                restore_apostrophes: false,
                undo_backspace: false,
                exceptions: vec!["choy".to_string()],
                ..Settings::default()
            }
        );
        assert!(!s.converting());
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bitg-ime-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn refresh_reads_only_changed_files() {
        let dir = temp("refresh");
        fs::create_dir_all(&dir).unwrap();
        let mut w = Watched::new(Some(dir.clone()));
        assert!(!w.refresh(), "bir sekund ichida tekshirilmaydi");
        w.checked = None;
        assert!(!w.refresh());
        fs::write(dir.join("settings.ini"), "cyrillic=0\n").unwrap();
        assert!(!w.refresh());
        w.checked = None;
        assert!(w.refresh());
        assert!(!w.settings.cyrillic);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn remember_without_dir_or_trailing_newline() {
        let mut w = Watched::new(None);
        w.remember("choy").unwrap();
        assert_eq!(w.settings, Settings::default());

        let dir = temp("append");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("exceptions.txt"), "shop").unwrap();
        let mut w = Watched::new(Some(dir.clone()));
        w.remember("chat").unwrap();
        w.remember("bog'").unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("exceptions.txt")).unwrap(),
            "shop\nchat\nbog'\n"
        );
        assert_eq!(w.settings.exceptions, ["bog'", "chat", "shop"]);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn remembers_exception_once() {
        let dir = std::env::temp_dir().join(format!("bitg-ime-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let mut w = Watched::new(Some(dir.clone()));
        w.remember("Chat").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let m = fs::metadata(dir.join("exceptions.txt")).unwrap();
            assert_eq!(m.permissions().mode() & 0o777, 0o600);
        }
        w.remember("chat").unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("exceptions.txt")).unwrap(),
            "Chat\n"
        );
        assert_eq!(w.settings.exceptions, vec!["chat".to_string()]);
        fs::remove_dir_all(&dir).unwrap();
    }
}
