//! bitg yozish dvigateli — platformadan mustaqil (IBus, Fcitx5 shu yerdan foydalanadi; C ABI — `ffi`).
//!
//! Ikki rejim, so'z boshida tanlanadi:
//! - **to'g'ridan-to'g'ri** — dastur kursor oldidagi matnni beradi (surrounding text: GTK, Qt, Firefox, Chromium,
//!   LibreOffice). Harflar odatdagidek yoziladi; probelda kursor oldida haqiqatan shu so'z turganini tekshirib,
//!   so'z o'chiriladi va o'girilgani yoziladi. Backspace darhol bosilsa — asl so'z qaytadi.
//! - **preedit** — dastur matnni bermaydi (XIM, ba'zi eski dasturlar): so'z tagiga chiziq bilan ushlab turiladi,
//!   chegarada o'girilgan holda yoziladi. Bu yerda Backspace bilan qaytarish yo'q (matnga kira olmaymiz).
//!
//! Hech qachon matn log qilinmaydi va saqlanmaydi (faqat `undo_remember=1` bo'lsa qaytarilgan so'z istisnoga).

pub mod config;
pub mod ffi;

use std::mem::take;
use std::path::PathBuf;

use bitg_core::Session;
use bitg_core::chars::fold_key;
use bitg_core::token::split_edges;
use config::{Settings, Watched};

/// Bitta so'z uchun eng ko'p belgi: bundan uzun so'z o'girilmaydi (Windows bilan bir xil).
pub const MAX_WORD: usize = 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// Bosilgan tugmaning belgisi (tartib allaqachon qo'llangan). Bo'shliq belgisi `Space` kabi.
    Char(char),
    /// Probel yoki NBSP: so'z tugadi.
    Space(char),
    Enter,
    Tab,
    Backspace,
    /// Kursor siljishi (strelkalar, Home, End, PgUp, PgDn, Delete, Esc): gap konteksti unutiladi.
    Navigate,
    /// Ctrl/Alt/Super bilan tugma yoki belgisiz tugma: kursor qayerdaligini endi bilmaymiz.
    Other,
}

/// Dvigatel javobi. Qo'llash tartibi: `delete_before` → `commit` → `preedit`; `consumed` bo'lmasa tugma
/// dasturga o'tadi (commit'dan keyin).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Out {
    pub consumed: bool,
    /// Kursor oldidan o'chiriladigan belgilar soni (Unicode scalar).
    pub delete_before: usize,
    pub commit: String,
    /// `Some` — preedit yangilansin (bo'sh satr = yashirilsin); `None` — o'zgarmaydi.
    pub preedit: Option<String>,
}

impl Out {
    fn pass() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
struct Undo {
    original: String,
    /// Yozilgan matn: o'girilgan so'z + chegara belgisi.
    written: String,
}

#[derive(Debug)]
pub struct Engine {
    cfg: Watched,
    session: Session,
    word: String,
    /// Preedit rejimi uchun: so'z boshi chegaradan keyin ko'rilgan.
    trusted: bool,
    in_preedit: bool,
    /// Dastur surrounding text "beraman" desa ham bo'sh qaytaradi — shu fokusda preedit'ga o'tamiz.
    surround_broken: bool,
    undo: Option<Undo>,
    /// Shu seansda Backspace bilan qaytarilgan so'zlar (`fold_key`).
    learned: Vec<String>,
    /// Parol, PIN, terminal maydoni: hech narsaga tegilmaydi.
    suspended: bool,
}

impl Engine {
    /// `dir` — sozlamalar papkasi (`config::config_dir()`); `None` — standart sozlamalar.
    pub fn new(dir: Option<PathBuf>) -> Self {
        Self::with(Watched::new(dir))
    }

    /// Fayl o'qimasdan, berilgan sozlamalar bilan (test va o'rnatish skriptlari uchun).
    pub fn with_settings(s: Settings) -> Self {
        let mut w = Watched::new(None);
        w.settings = s;
        Self::with(w)
    }

    fn with(cfg: Watched) -> Self {
        Self {
            cfg,
            session: Session::new(),
            word: String::new(),
            trusted: true,
            in_preedit: false,
            surround_broken: false,
            undo: None,
            learned: Vec::new(),
            suspended: false,
        }
    }

    pub fn settings(&self) -> &Settings {
        &self.cfg.settings
    }

    /// Yangi maydonga fokus: keyingi harf — ishonchli so'z boshi.
    pub fn focus_in(&mut self) {
        self.clear(true);
        self.surround_broken = false;
    }

    /// Fokus ketdi yoki kursor sichqoncha bilan siljidi. Preedit'dagi xom so'zni qaytaradi — uni frontend
    /// o'zi yozishi kerak (IBus buni `PREEDIT_COMMIT` rejimida o'zi qiladi).
    pub fn reset(&mut self) -> String {
        let pending = if self.in_preedit {
            take(&mut self.word)
        } else {
            String::new()
        };
        self.clear(false);
        pending
    }

    /// Parol/terminal maydoni (`true`) yoki oddiy maydon. Preedit'dagi so'z o'girilmay yoziladi.
    pub fn set_suspended(&mut self, on: bool) -> Out {
        let out = self.flush();
        self.suspended = on;
        out
    }

    fn clear(&mut self, trusted: bool) {
        self.word.clear();
        self.in_preedit = false;
        self.trusted = trusted;
        self.undo = None;
        self.session.reset();
    }

    /// Preedit'dagi so'zni o'girmasdan yozadi (kursor siljiydi, rejim o'chadi va h.k.).
    fn flush(&mut self) -> Out {
        let word = take(&mut self.word);
        if take(&mut self.in_preedit) && !word.is_empty() {
            return Out {
                commit: word,
                preedit: Some(String::new()),
                ..Out::default()
            };
        }
        Out::pass()
    }

    /// Sozlamalar fayli o'zgargan bo'lsa qayta o'qiydi (sekundiga ko'pi bilan bir marta).
    fn sync(&mut self) {
        if self.cfg.refresh() {
            self.session.reset();
        }
    }

    /// Shu tugma uchun `key()` ga kursor oldidagi matn kerakmi. Matnni so'rash qimmat bo'lgan joyda (macOS:
    /// har so'rov dasturga sinxron IPC) frontend `false` bo'lsa `before = None` beradi — natija baribir bir xil
    /// (`tests::wants_before_is_exact` buni tasodifiy ketma-ketliklarda tekshiradi).
    pub fn wants_before(&mut self, key: Key) -> bool {
        self.sync();
        if self.suspended || !self.cfg.settings.converting() || self.in_preedit {
            return false;
        }
        match key {
            // so'z boshi: matn bor-yo'qligi rejimni tanlaydi (to'g'ridan-to'g'ri yoki preedit)
            Key::Char(c) if !c.is_whitespace() => self.word.is_empty(),
            // so'z oxiri: kursor oldida haqiqatan shu so'z turganini tekshirish uchun
            Key::Char(_) | Key::Space(_) | Key::Enter | Key::Tab => !self.word.is_empty(),
            // faqat "asl so'zni qaytarish" matnni o'qiydi
            Key::Backspace => self.word.is_empty() && self.undo.is_some(),
            Key::Navigate | Key::Other => false,
        }
    }

    /// Bitta tugma. `before` — kursor oldidagi matn (dastur bersa), aks holda `None`.
    pub fn key(&mut self, key: Key, before: Option<&str>) -> Out {
        self.sync();
        if self.suspended || !self.cfg.settings.converting() {
            self.undo = None;
            return self.flush();
        }
        match key {
            Key::Char(c) if c.is_whitespace() => self.finish(Some(c), before),
            Key::Char(c) => self.char(c, before),
            Key::Space(c) => self.finish(Some(c), before),
            Key::Enter | Key::Tab => {
                let out = self.finish(None, before);
                self.session.reset();
                out
            }
            Key::Backspace => self.backspace(before),
            Key::Navigate => {
                let out = self.flush();
                self.session.reset();
                self.trusted = false;
                self.undo = None;
                out
            }
            Key::Other => {
                let out = self.flush();
                self.trusted = false;
                self.undo = None;
                out
            }
        }
    }

    fn char(&mut self, c: char, before: Option<&str>) -> Out {
        self.undo = None;
        if self.word.is_empty() {
            self.in_preedit = before.is_none() || self.surround_broken;
        }
        if self.word.chars().count() >= MAX_WORD {
            // juda uzun — bu so'z o'girilmaydi; keyingi harflar oddiy yoziladi
            let out = self.flush();
            self.trusted = false;
            return out;
        }
        self.word.push(c);
        if self.in_preedit {
            Out {
                consumed: true,
                preedit: Some(self.word.clone()),
                ..Out::default()
            }
        } else {
            Out::pass()
        }
    }

    /// So'z tugadi: probel (`ws`) yoki Enter/Tab (`None` — tugma dasturga o'tadi).
    fn finish(&mut self, ws: Option<char>, before: Option<&str>) -> Out {
        let trusted = take(&mut self.trusted);
        self.trusted = true;
        self.undo = None;
        let word = take(&mut self.word);
        let preedit = take(&mut self.in_preedit);
        if word.is_empty() {
            return Out::pass();
        }
        let ok = if preedit {
            trusted
        } else {
            match before {
                Some(b) if ends_with_word(b, &word) => true,
                Some(b) => {
                    // "beraman" degan, lekin bo'sh matn qaytargan dastur: keyingi so'zlar preedit'da
                    if b.is_empty() {
                        self.surround_broken = true;
                    }
                    false
                }
                None => false,
            }
        };
        let conv = if ok {
            let opts = self.cfg.settings.options(&self.learned);
            self.session.complete_word(&word, &opts).map(|e| e.insert)
        } else {
            self.session.reset();
            None
        };
        let tail: String = ws.into_iter().collect();
        if let (Some(c), Some(_)) = (&conv, ws)
            && self.cfg.settings.undo_backspace
        {
            self.undo = Some(Undo {
                original: word.clone(),
                written: format!("{c}{tail}"),
            });
        }
        let consumed = ws.is_some();
        match (preedit, conv) {
            (true, conv) => Out {
                consumed,
                commit: format!("{}{tail}", conv.unwrap_or(word)),
                preedit: Some(String::new()),
                ..Out::default()
            },
            (false, Some(c)) => Out {
                consumed,
                delete_before: word.chars().count(),
                commit: format!("{c}{tail}"),
                preedit: None,
            },
            (false, None) => Out::pass(),
        }
    }

    fn backspace(&mut self, before: Option<&str>) -> Out {
        if !self.word.is_empty() {
            self.undo = None;
            self.word.pop();
            if !self.in_preedit {
                return Out::pass();
            }
            if self.word.is_empty() {
                self.in_preedit = false;
            }
            return Out {
                consumed: true,
                preedit: Some(self.word.clone()),
                ..Out::default()
            };
        }
        if let (Some(u), Some(b)) = (self.undo.take(), before)
            && b.ends_with(&u.written)
        {
            self.learn(&u.original);
            self.session.reset();
            // kursor asl so'z oxirida: davomi shu so'zga qo'shiladi, uni o'girmaymiz
            self.trusted = false;
            return Out {
                consumed: true,
                delete_before: u.written.chars().count(),
                commit: u.original,
                preedit: None,
            };
        }
        // so'zdan oldingi matn o'chirilmoqda: keyingi so'z boshi noma'lum
        self.trusted = false;
        Out::pass()
    }

    fn learn(&mut self, original: &str) {
        let core = split_edges(original).core;
        let key = fold_key(core);
        if key.is_empty() || self.learned.contains(&key) {
            return;
        }
        self.learned.push(key);
        if self.cfg.settings.undo_remember {
            let _ = self.cfg.remember(core);
        }
    }
}

/// `before` shu so'z bilan tugaydi va undan oldin bo'shliq yoki matn boshi turibdi.
fn ends_with_word(before: &str, word: &str) -> bool {
    before
        .strip_suffix(word)
        .is_some_and(|rest| rest.chars().next_back().is_none_or(char::is_whitespace))
}

#[cfg(test)]
mod tests;
