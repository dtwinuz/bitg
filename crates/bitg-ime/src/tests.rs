// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dastur modeli bilan: dvigatel javobini matn maydoniga qo'llab, natijaviy matnni tekshiramiz.

use super::*;

/// Matn maydoni modeli. `surrounding` — dastur kursor oldidagi matnni beradimi; `broken` — beraman deydi,
/// lekin bo'sh qaytaradi (ba'zi Electron/X11 dasturlar).
#[derive(Default)]
struct App {
    text: String,
    preedit: String,
    surrounding: bool,
    broken: bool,
}

impl App {
    fn direct() -> Self {
        Self {
            surrounding: true,
            ..Self::default()
        }
    }

    fn before(&self) -> Option<String> {
        if self.broken {
            Some(String::new())
        } else {
            self.surrounding.then(|| self.text.clone())
        }
    }

    fn apply(&mut self, key: Key, o: Out) {
        for _ in 0..o.delete_before {
            self.text.pop();
        }
        self.text.push_str(&o.commit);
        if let Some(p) = o.preedit {
            self.preedit = p;
        }
        if !o.consumed {
            match key {
                Key::Char(c) | Key::Space(c) => self.text.push(c),
                Key::Enter => self.text.push('\n'),
                Key::Tab => self.text.push('\t'),
                Key::Backspace => {
                    self.text.pop();
                }
                Key::Navigate | Key::Other => {}
            }
        }
    }

    /// `\u{8}` — Backspace, `\n` — Enter, `\t` — Tab, `\u{1b}` — strelka (Navigate).
    fn typ(&mut self, e: &mut Engine, s: &str) -> &mut Self {
        for c in s.chars() {
            let key = match c {
                ' ' => Key::Space(' '),
                '\n' => Key::Enter,
                '\t' => Key::Tab,
                '\u{8}' => Key::Backspace,
                '\u{1b}' => Key::Navigate,
                c => Key::Char(c),
            };
            let b = self.before();
            let o = e.key(key, b.as_deref());
            self.apply(key, o);
        }
        self
    }

    fn shown(&self) -> String {
        format!("{}{}", self.text, self.preedit)
    }
}

fn engine() -> Engine {
    let mut e = Engine::with_settings(Settings::default());
    e.focus_in();
    e
}

#[test]
fn direct_mode_converts_on_space() {
    let mut e = engine();
    let mut app = App::direct();
    app.typ(&mut e, "keyin choy ichamiz ");
    assert_eq!(app.text, "keyin çoy içamiz ");
    assert!(app.preedit.is_empty());
}

#[test]
fn direct_mode_letters_reach_app_immediately() {
    let mut e = engine();
    let mut app = App::direct();
    app.typ(&mut e, "chap");
    // qidiruv maydonlari (avtoto'ldirish) harflarni darhol ko'radi
    assert_eq!(app.text, "chap");
}

#[test]
fn english_and_links_untouched() {
    let mut e = engine();
    let mut app = App::direct();
    app.typ(
        &mut e,
        "Please check the schedule, keyin choy ichamiz https://example.com/shop ",
    );
    assert_eq!(
        app.text,
        "Please check the schedule, keyin çoy içamiz https://example.com/shop "
    );
}

#[test]
fn punctuation_stays_with_word() {
    let mut e = engine();
    let mut app = App::direct();
    app.typ(&mut e, "Choy, o‘zbek! ");
    assert_eq!(app.text, "Çoy, özbek! ");
}

#[test]
fn cyrillic_goes_to_new_latin() {
    let mut e = engine();
    let mut app = App::direct();
    app.typ(&mut e, "чой ичамиз ");
    assert_eq!(app.text, "çoy içamiz ");
}

#[test]
fn enter_converts_and_newline_is_kept() {
    let mut e = engine();
    let mut app = App::direct();
    app.typ(&mut e, "salom shahar\nchoy ");
    assert_eq!(app.text, "salom şahar\nçoy ");
}

#[test]
fn backspace_right_after_restores_and_learns() {
    let mut e = engine();
    let mut app = App::direct();
    app.typ(&mut e, "choy ");
    assert_eq!(app.text, "çoy ");
    app.typ(&mut e, "\u{8}");
    assert_eq!(app.text, "choy", "asl so'z qaytadi, probel o'chadi");
    // davom etgan harflar shu so'zga qo'shiladi — o'girilmaydi
    app.typ(&mut e, "lar ");
    assert_eq!(app.text, "choylar ");
    // qaytarilgan so'z shu seansda boshqa o'girilmaydi, boshqalari o'giriladi
    app.typ(&mut e, "choy shahar ");
    assert_eq!(app.text, "choylar choy şahar ");
}

#[test]
fn undo_only_immediately_after_replacement() {
    let mut e = engine();
    let mut app = App::direct();
    app.typ(&mut e, "choy x\u{8}\u{8}\u{8}");
    // x yozildi va o'chirildi, keyin probel, keyin "y" — oddiy Backspace'lar
    assert_eq!(app.text, "ço");
}

#[test]
fn undo_disabled_by_setting() {
    let mut e = Engine::with_settings(Settings {
        undo_backspace: false,
        ..Settings::default()
    });
    e.focus_in();
    let mut app = App::direct();
    app.typ(&mut e, "choy \u{8}");
    assert_eq!(app.text, "çoy");
    // probeldan keyin davom etgan so'z oldingisiga yopishgan — tegilmaydi
    app.typ(&mut e, "lar ");
    assert_eq!(app.text, "çoylar ");
}

#[test]
fn no_edit_when_app_text_does_not_match() {
    let mut e = engine();
    let mut app = App::direct();
    app.typ(&mut e, "cho");
    // dastur o'zi matnni o'zgartirdi (avtotuzatish)
    app.text = "chop".into();
    app.typ(&mut e, "y ");
    assert_eq!(app.text, "chopy ");
}

#[test]
fn word_glued_to_previous_text_is_not_converted() {
    let mut e = engine();
    let mut app = App::direct();
    app.text = "Toshkent".into();
    e.reset(); // sichqoncha bilan matn oxiriga
    app.typ(&mut e, "cha ");
    assert_eq!(app.text, "Toshkentcha ");
}

#[test]
fn preedit_mode_holds_word_and_commits_converted() {
    let mut e = engine();
    let mut app = App::default();
    app.typ(&mut e, "shahar");
    assert_eq!((app.text.as_str(), app.preedit.as_str()), ("", "shahar"));
    app.typ(&mut e, " choy\n");
    assert_eq!(app.text, "şahar çoy\n");
    assert!(app.preedit.is_empty());
}

#[test]
fn preedit_backspace_edits_word() {
    let mut e = engine();
    let mut app = App::default();
    app.typ(&mut e, "chox\u{8}y ");
    assert_eq!(app.text, "çoy ");
    app.typ(&mut e, "a\u{8}\u{8}");
    assert_eq!(
        app.shown(),
        "çoy",
        "bo'sh preedit'dan keyin Backspace dasturga o'tadi"
    );
}

#[test]
fn preedit_reset_returns_raw_word_and_next_word_untrusted() {
    let mut e = engine();
    let mut app = App::default();
    app.typ(&mut e, "cho");
    assert_eq!(e.reset(), "cho");
    app.preedit.clear();
    app.typ(&mut e, "oy choy ");
    assert_eq!(
        app.text, "oy çoy ",
        "sichqonchadan keyingi birinchi so'z noma'lum joyda — tegilmaydi"
    );
}

#[test]
fn navigation_flushes_preedit_unconverted() {
    let mut e = engine();
    let mut app = App::default();
    app.typ(&mut e, "choy\u{1b}");
    assert_eq!(app.shown(), "choy");
}

#[test]
fn broken_surrounding_switches_to_preedit() {
    let mut e = engine();
    let mut app = App {
        surrounding: true,
        broken: true,
        ..App::default()
    };
    app.typ(&mut e, "choy choy ");
    // birinchi so'z dasturga allaqachon yozilgan — tekshirib bo'lmaydi; keyingisi preedit orqali
    assert_eq!(app.text, "choy çoy ");
    e.focus_in();
    app.typ(&mut e, "choy ");
    assert_eq!(
        app.text, "choy çoy choy ",
        "yangi fokusda yana sinab ko'riladi"
    );
}

#[test]
fn long_word_is_left_alone() {
    let mut e = engine();
    let mut app = App::direct();
    let long = "sh".repeat(25);
    app.typ(&mut e, &format!("{long} choy "));
    assert_eq!(app.text, format!("{long} çoy "));
    let mut e = engine();
    let mut app = App::default();
    app.typ(&mut e, &format!("{long} choy "));
    assert_eq!(app.text, format!("{long} çoy "));
}

#[test]
fn suspended_and_disabled_pass_everything() {
    let mut e = engine();
    let mut app = App::default();
    app.typ(&mut e, "cho");
    let o = e.set_suspended(true);
    app.apply(Key::Other, o);
    app.typ(&mut e, "y choy ");
    assert_eq!(app.shown(), "choy choy ");

    let mut e = Engine::with_settings(Settings {
        enabled: false,
        ..Settings::default()
    });
    let mut app = App::direct();
    app.typ(&mut e, "choy ");
    assert_eq!(app.text, "choy ");
}

#[test]
fn exceptions_from_settings() {
    let s = Settings {
        exceptions: config::parse_exceptions("Choy\n"),
        ..Settings::default()
    };
    let mut e = Engine::with_settings(s);
    e.focus_in();
    let mut app = App::direct();
    app.typ(&mut e, "choy shahar ");
    assert_eq!(app.text, "choy şahar ");
}

#[test]
fn double_space_and_leading_space_pass_through() {
    let mut e = engine();
    let mut app = App::direct();
    app.typ(&mut e, "  choy  ");
    assert_eq!(app.text, "  çoy  ");
}

/// `wants_before` aniq: matn faqat u `true` degan tugmada berilsa ham, dvigatel javobi va natijaviy matn
/// har qadamda "matn doim beriladi" holati bilan bir xil. Tasodifiy ketma-ketliklar, uch xil dastur.
#[test]
fn wants_before_is_exact() {
    const KEYS: &[char] = &[
        'c', 'h', 'o', 'y', 's', 'a', 'r', 'g', 'i', 'l', '\'', 'ʻ', 'O', 'S', 'ч', 'о', 'й', 'x',
        'w', '1', '.', ',', ' ', ' ', ' ', '\u{a0}', '\n', '\t', '\u{8}', '\u{8}', '\u{1b}',
        '\u{7}',
    ];
    let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut asked = 0usize;
    let mut total = 0usize;
    for round in 0..600 {
        let mode = round % 3;
        let mk = || match mode {
            0 => App::direct(),
            1 => App {
                broken: true,
                surrounding: true,
                ..App::default()
            },
            _ => App::default(),
        };
        let (mut a, mut b) = (mk(), mk());
        let (mut ea, mut eb) = (engine(), engine());
        let len = 20 + (next() % 60) as usize;
        for step in 0..len {
            let c = KEYS[(next() % KEYS.len() as u64) as usize];
            let key = match c {
                ' ' | '\u{a0}' => Key::Space(c),
                '\n' => Key::Enter,
                '\t' => Key::Tab,
                '\u{8}' => Key::Backspace,
                '\u{1b}' => Key::Navigate,
                '\u{7}' => Key::Other,
                c => Key::Char(c),
            };
            let oa = ea.key(key, a.before().as_deref());
            let want = eb.wants_before(key);
            total += 1;
            asked += usize::from(want);
            let bb = if want { b.before() } else { None };
            let ob = eb.key(key, bb.as_deref());
            assert_eq!(
                oa,
                ob,
                "round {round} step {step} key {key:?}: {:?}",
                a.shown()
            );
            a.apply(key, oa);
            b.apply(key, ob);
            assert_eq!(a.shown(), b.shown(), "round {round} step {step}");
        }
    }
    // tejash haqiqiy: matn tugmalarning yarmidan kamida so'raladi
    assert!(asked * 2 < total, "so'raldi {asked}/{total}");
}

#[test]
fn wants_before_typical_word() {
    let mut e = engine();
    let mut app = App::direct();
    let mut asked = String::new();
    for c in "choy ".chars() {
        let key = if c == ' ' {
            Key::Space(c)
        } else {
            Key::Char(c)
        };
        let want = e.wants_before(key);
        asked.push(if want { '1' } else { '0' });
        let b = if want { app.before() } else { None };
        let o = e.key(key, b.as_deref());
        app.apply(key, o);
    }
    // so'z boshi va probel — 2 ta so'rov, 5 emas
    assert_eq!(asked, "10001");
    assert_eq!(app.text, "çoy ");
    // darhol Backspace — asl so'zni qaytarish uchun matn kerak
    assert!(e.wants_before(Key::Backspace));
    assert!(!e.wants_before(Key::Navigate));
}
