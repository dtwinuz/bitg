// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! TSF xizmati. Tugmani faqat ikki holatda ushlab qoladi (qolgani doim dasturga boradi):
//! - klaviatura tartibi yoqilgan va tugmaga belgi qo'yilgan — belgi tahrir sessiyasida yoziladi
//!   (IMM32 dasturlarida qisqa kompozitsiya bilan); AltGr qatlamlari — "preserved key" (o'ng Alt);
//! - o'girishdan keyin darhol Backspace — asl so'z qaytariladi va istisnolarga qo'shiladi.
//!
//! 1) Asosiy yo'l: fokusdagi matnning o'zgarishini kuzatadi (ITfTextEditSink). So'zdan keyin probel/Enter/Tab
//!    yozilgani ko'rinsa (kursor oldidagi matn = oldingi matn + bo'shliq), so'zni asinxron tahrirda almashtiradi.
//! 2) Zaxira yo'l (dastur matnni TSF'ga bermasa — Telegram Desktop): bosilgan harflardan so'z yig'iladi
//!    (`typed`), probeldan keyin Backspace + Unicode kiritish (SendInput) bilan almashtiriladi.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use std::time::{Duration, Instant};

use bitg_core::Session;
use bitg_core::chars::fold_key;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, GetKeyboardLayout, GetKeyboardState, INPUT, INPUT_0,
    INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, MAPVK_VK_TO_VSC, MAPVK_VSC_TO_VK, MapVirtualKeyExW, SendInput, ToUnicodeEx,
    VIRTUAL_KEY, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME,
    VK_LBUTTON, VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_NEXT, VK_PACKET,
    VK_PRIOR, VK_RBUTTON, VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
    VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::TextServices::{
    INSERT_TEXT_AT_SELECTION_FLAGS, ITfCompositionSink, ITfContext, ITfContextComposition,
    ITfDocumentMgr, ITfEditRecord, ITfEditSession, ITfEditSession_Impl, ITfInsertAtSelection,
    ITfKeyEventSink, ITfKeyEventSink_Impl, ITfKeystrokeMgr, ITfRange, ITfSource, ITfTextEditSink,
    ITfTextEditSink_Impl, ITfTextInputProcessor, ITfTextInputProcessor_Impl,
    ITfTextInputProcessorEx, ITfTextInputProcessorEx_Impl, ITfThreadMgr, ITfThreadMgrEventSink,
    ITfThreadMgrEventSink_Impl, TF_AE_END, TF_ANCHOR_END, TF_DEFAULT_SELECTION, TF_E_SYNCHRONOUS,
    TF_ES_ASYNC, TF_ES_ASYNCDONTCARE, TF_ES_READWRITE, TF_ES_SYNC, TF_IAS_QUERYONLY, TF_MOD_RALT,
    TF_MOD_SHIFT, TF_PRESERVEDKEY, TF_SELECTION, TF_SELECTIONSTYLE, TF_SS_TRANSITORY, TS_S_ASYNC,
};
use windows::Win32::UI::WindowsAndMessaging::GetMessageExtraInfo;
use windows_core::{BOOL, GUID, IUnknown, IUnknownImpl, Interface, Ref, Result, implement};

use crate::config::{self, Config};
use crate::keymap::{self, Keymap, REMAP_TAG};
use crate::log;
use crate::plan;
use crate::typed::Typed;

/// Kursor oldidan o'qiladigan eng ko'p UTF-16 birlik.
const LOOKBACK: i32 = 64;
const NO_COOKIE: u32 = u32::MAX;
/// Qayta yuborilgan foydalanuvchi tugmasi (`dwExtraInfo`): ikkinchi marta ushlanmaydi, lekin kuzatiladi.
const INJECT_TAG: usize = 0xB17B;
/// Zaxira yo'l tahriri (Backspace + yangi matn): so'z kuzatuviga umuman kirmaydi. Sanagich o'rniga belgi —
/// dastur tugmalarimizdan birini TSF'dan o'tkazmasa ham, foydalanuvchining Backspace'i yutilib ketmaydi.
const RETYPE_TAG: usize = 0xB17D;
/// Tugma ishlovchisi bundan uzoq ishlasa — log (dastur "qotishi"ni topish uchun).
const SLOW_KEY: Duration = Duration::from_millis(25);

fn own_key() -> bool {
    matches!(
        unsafe { GetMessageExtraInfo() }.0 as usize,
        INJECT_TAG | RETYPE_TAG
    )
}

/// IMM32 kontekstida ushlangan tugma qanday qayta yuboriladi.
enum Eat {
    /// Unicode belgi (VK_PACKET, REMAP_TAG bilan).
    Char(char),
    /// Tugmaning o'zi (bo'shliq, Enter, Backspace, strelkalar).
    Key,
}

/// Oxirgi o'girish: darhol Backspace bosilsa — qaytariladi.
struct Undo {
    original: String,
    converted: String,
    /// Zaxira yo'lda (SendInput bilan) o'girilgan.
    blind: bool,
}

struct State {
    thread_mgr: Option<ITfThreadMgr>,
    tid: u32,
    tm_cookie: u32,
    /// Matn o'zgarishi kuzatilayotgan kontekst.
    watched: Option<(ITfContext, u32)>,
    /// Oxirgi ko'rilgan "kursor oldidagi matn".
    last: Vec<u16>,
    /// Bu kontekstda matn o'qilmadi (Qt/IMM32 kabi dasturlar) — log bir marta.
    blind_logged: bool,
    session: Session,
    cfg: Config,
    /// Sozlama fayllari holati va oxirgi tekshiruv (oyna o'zgartirsa — dasturni qayta ochmasdan qabul qilinadi).
    stamp: config::Stamp,
    checked: Option<Instant>,
    /// Shu dasturda ishlaydimi (enabled va disabled_apps bo'yicha).
    active: bool,
    /// Kontekst matnni bermayapti — zaxira yo'l ishlaydi.
    blind: bool,
    key_sink: bool,
    typed: Typed,
    /// Faol klaviatura tartibi (yoqilgan va bo'sh emas).
    keymap: Option<Keymap>,
    /// Ro'yxatga olingan AltGr tugmalari va ular tartibga mos kelishi.
    preserved: Vec<(GUID, TF_PRESERVEDKEY)>,
    keys_dirty: bool,
    undo: Option<Undo>,
    /// Tartib belgilarimiz (`send_char`), yuborilgan, lekin hali qaytib kelmagan — UTF-16 birligi bo'yicha
    /// (surrogatning ikkinchi yarmi `'\0'`). Qt `dwExtraInfo`ni bermagani uchun tartib bo'yicha taniladi.
    remap: VecDeque<(char, Instant)>,
    /// Zaxira tahririmizdan hali kelmagan Backspace va Unicode tugmalar. Qt (Telegram) `dwExtraInfo`ni
    /// TSF'ga yetkazmaydi — belgi o'qilmasa, shu sanagich o'z tugmamizni foydalanuvchinikidan ajratadi.
    retype_back: u32,
    retype_packet: u32,
    retype_until: Option<Instant>,
    /// Zaxira tahririmiz navbatda turganda bosilgan (oldindan yozilgan) tugmalar tahrirdan keyin qayta
    /// yuborilgan — shu tartibda qaytishi kutiladi. Qt `dwExtraInfo`ni bermagani uchun tartib bo'yicha taniladi.
    resent: VecDeque<(Resent, Instant)>,
    /// `resent` boshidagi shuncha tugma oxirgi zaxira tahriridan oldin navbatga tushgan.
    resent_ahead: usize,
    /// `OnTestKeyDown` ushlagan, `OnKeyDown` qayta yuboradigan tugma (VK, lParam, nima yuboriladi).
    hold: Option<(u16, isize, Resent)>,
}

/// Qayta yuborilgan tugma: belgi (Unicode, Shift holati bosilgan paytdagidek) yoki tugmaning o'zi.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Resent {
    Char(char),
    Key(u16),
}

/// Zaxira tahriri tugmalari shu vaqt ichida kelishi kerak; kelmasa sanagich tashlanadi.
const RETYPE_WINDOW: Duration = Duration::from_millis(1500);
/// Tartib belgimiz shu vaqt ichida qaytishi kerak.
const REMAP_WINDOW: Duration = Duration::from_secs(1);

impl State {
    /// Zaxira tahririmiz (yoki undan keyin qayta yuborilgan tugmalar) hali navbatda — yangi tugma ulardan
    /// oldin dasturga yetsa, Backspace'larimiz foydalanuvchi harflarini o'chiradi.
    fn retype_pending(&mut self) -> bool {
        let now = Instant::now();
        self.resent.retain(|&(_, until)| now < until);
        self.retyping() || !self.resent.is_empty()
    }

    /// Zaxira tahririmiz tugmalari hali kelmagan.
    fn retyping(&self) -> bool {
        self.retype_until.is_some_and(|t| Instant::now() < t)
            && (self.retype_back > 0 || self.retype_packet > 0)
    }

    /// Bu tugma navbatdagi qayta yuborilgan tugmamizmi.
    fn take_resent(&mut self, vk: u16) -> Option<Resent> {
        let now = Instant::now();
        self.resent.retain(|&(_, until)| now < until);
        let &(front, _) = self.resent.front()?;
        let hit = match front {
            Resent::Char(_) => vk == VK_PACKET.0,
            Resent::Key(k) => k == vk,
        };
        if hit {
            self.resent.pop_front();
            self.resent_ahead = self.resent_ahead.saturating_sub(1);
            Some(front)
        } else {
            None
        }
    }

    /// `send_char` yuboradigan belgi: har UTF-16 birligi navbatga (ikkinchi yarmi `'\0'`).
    fn expect_remap(&mut self, c: char) {
        let until = Instant::now() + REMAP_WINDOW;
        self.remap.push_back((c, until));
        if c.len_utf16() == 2 {
            self.remap.push_back(('\0', until));
        }
    }

    /// Yo'qolgan belgi navbatni abadiy band qilmasin.
    fn purge_remap(&mut self) {
        let now = Instant::now();
        self.remap.retain(|&(_, until)| now < until);
    }

    fn expect_retype(&mut self, backs: usize, packets: usize) {
        self.retype_back += backs as u32;
        self.retype_packet += packets as u32;
        self.retype_until = Some(Instant::now() + RETYPE_WINDOW);
        // Hozir navbatdagi qayta yuborilganlar bu tahrirdan OLDIN keladi.
        self.resent_ahead = self.resent.len();
    }

    /// Navbatda bu tugma qayta yuborilgan tugmamiz bo'lishi kerak (tahrirdan oldin yuborilgan).
    fn resent_first(&self) -> bool {
        self.resent_ahead.min(self.resent.len()) > 0
    }

    /// Bu tugma bizning zaxira tahririmizmi (belgi yoki sanagich bo'yicha).
    fn take_retype(&mut self, vk: u16) -> bool {
        if !self.retype_until.is_some_and(|t| Instant::now() < t) {
            self.retype_back = 0;
            self.retype_packet = 0;
        }
        let extra = unsafe { GetMessageExtraInfo() }.0 as usize;
        if (extra as u64) >> 32 == REMAP_TAG {
            return false; // klaviatura tartibi belgisi — so'zning davomi
        }
        let tagged = extra == RETYPE_TAG;
        let counter = if vk == VK_BACK.0 {
            &mut self.retype_back
        } else if vk == VK_PACKET.0 {
            &mut self.retype_packet
        } else {
            return tagged;
        };
        if *counter > 0 {
            *counter -= 1;
            return true;
        }
        tagged
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            thread_mgr: None,
            tid: 0,
            tm_cookie: NO_COOKIE,
            watched: None,
            last: Vec::new(),
            blind_logged: false,
            session: Session::new(),
            cfg: Config::default(),
            stamp: Default::default(),
            checked: None,
            active: false,
            blind: true,
            key_sink: false,
            typed: Typed::default(),
            keymap: None,
            preserved: Vec::new(),
            keys_dirty: true,
            undo: None,
            remap: VecDeque::new(),
            retype_back: 0,
            retype_packet: 0,
            retype_until: None,
            resent: VecDeque::new(),
            resent_ahead: 0,
            hold: None,
        }
    }
}

#[implement(
    ITfTextInputProcessorEx,
    ITfTextInputProcessor,
    ITfThreadMgrEventSink,
    ITfTextEditSink,
    ITfKeyEventSink
)]
pub struct TextService {
    state: Rc<RefCell<State>>,
}

impl TextService {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(State::default())),
        }
    }
}

/// Sozlama fayllari ko'pi bilan sekundiga bir marta tekshiriladi.
const CONFIG_CHECK: Duration = Duration::from_secs(1);

impl State {
    fn apply(&mut self, cfg: Config, stamp: config::Stamp) {
        let active = cfg.enabled && !cfg.app_disabled(log::process());
        if active != self.active {
            self.keys_dirty = true;
        }
        self.active = active;
        let km = if cfg.keymap {
            Some(cfg.resolve_keymap(&config::read_keymap())).filter(|m| !m.is_empty())
        } else {
            None
        };
        if km != self.keymap {
            self.keymap = km;
            self.keys_dirty = true;
        }
        self.cfg = cfg;
        self.stamp = stamp;
        self.checked = Some(Instant::now());
    }

    /// Fayllar o'zgargan bo'lsa qayta o'qiydi. Qaytaradi: shu dasturda ishlash kerakmi.
    fn refresh(&mut self) -> bool {
        if self.checked.is_none_or(|t| t.elapsed() >= CONFIG_CHECK) {
            self.checked = Some(Instant::now());
            let now = config::stamp();
            if now != self.stamp {
                self.apply(config::load(), now);
                log::write(if self.active {
                    "settings reloaded: active"
                } else {
                    "settings reloaded: off for this app"
                });
            }
        }
        self.active
    }

    /// So'zni o'girish kerakmi: shu dasturda yoqilgan va avto o'girish o'chirilmagan.
    fn converting(&mut self) -> bool {
        self.refresh() && self.cfg.auto_convert
    }
}

fn guard<T: Default>(what: &str, f: impl FnOnce() -> Result<T>) -> Result<T> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => {
            log::write(&format!("{what} failed hr={:#x}", e.code().0));
            Ok(T::default())
        }
        Err(_) => {
            log::write(&format!("panic in {what}"));
            Ok(T::default())
        }
    }
}

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32) -> Result<()> {
        self.ActivateEx(ptim, tid, 0)
    }

    fn Deactivate(&self) -> Result<()> {
        guard("deactivate", || {
            self.unwatch();
            // Holat avval olinadi: TSF unadvise paytida bizni (OnSetFocus) chaqirishi mumkin — borrow ushlanmasin.
            let (tm, cookie, preserved, key_sink, tid) = {
                let mut st = self.state.borrow_mut();
                let r = (
                    st.thread_mgr.take(),
                    std::mem::replace(&mut st.tm_cookie, NO_COOKIE),
                    std::mem::take(&mut st.preserved),
                    std::mem::replace(&mut st.key_sink, false),
                    st.tid,
                );
                st.keys_dirty = true;
                st.undo = None;
                r
            };
            if let Some(tm) = tm {
                if cookie != NO_COOKIE
                    && let Ok(src) = tm.cast::<ITfSource>()
                {
                    let _ = unsafe { src.UnadviseSink(cookie) };
                }
                if let Ok(ks) = tm.cast::<ITfKeystrokeMgr>() {
                    for (g, k) in &preserved {
                        let _ = unsafe { ks.UnpreserveKey(g, k) };
                    }
                    if key_sink {
                        let _ = unsafe { ks.UnadviseKeyEventSink(tid) };
                    }
                }
            }
            log::write("deactivate");
            Ok(())
        })
    }
}

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    /// Xato yoki panic TSF'ga chiqmaydi: `ActivateEx` muvaffaqiyatsiz bo'lsa, Windows bitg'ni tashlab oldingi
    /// klaviaturaga qaytaradi (#13). Yarim ulangan holatni `Deactivate` tozalaydi.
    fn ActivateEx(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32, flags: u32) -> Result<()> {
        log::write(&format!("activate start flags={flags:#x}"));
        guard("activate", || self.activate(ptim, tid))
    }
}

impl TextService_Impl {
    fn activate(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32) -> Result<()> {
        let tm = ptim.ok()?.clone();
        let stamp = config::stamp();
        let cfg = config::load();
        let active = {
            let mut st = self.state.borrow_mut();
            st.thread_mgr = Some(tm.clone());
            st.tid = tid;
            st.apply(cfg, stamp);
            st.active
        };
        // Kuzatuvchilar har doim ulanadi: sozlama oynada yoqilsa, dasturni qayta ochish shart emas.
        // O'chiq holatda ular faqat kuzatadi, hech narsa yozmaydi (har chaqiruvda `refresh`).
        let me: IUnknown = self.to_object().to_interface();
        let src: ITfSource = tm.cast()?;
        let cookie = unsafe { src.AdviseSink(&ITfThreadMgrEventSink::IID, &me)? };
        self.state.borrow_mut().tm_cookie = cookie;
        let sink: ITfKeyEventSink = self.to_object().to_interface();
        let ks: ITfKeystrokeMgr = tm.cast()?;
        match unsafe { ks.AdviseKeyEventSink(tid, &sink, true) } {
            Ok(()) => self.state.borrow_mut().key_sink = true,
            Err(e) => log::write(&format!("advise key sink failed hr={:#x}", e.code().0)),
        }
        self.sync_preserved();
        // Hozir fokusdagi hujjat.
        if let Ok(dim) = unsafe { tm.GetFocus() } {
            self.watch(Some(&dim));
        }
        std::thread::spawn(bitg_core::lang::warm_up);
        log::write(if active {
            "activate"
        } else {
            "activate: off for this app (settings)"
        });
        Ok(())
    }
}

impl ITfThreadMgrEventSink_Impl for TextService_Impl {
    fn OnInitDocumentMgr(&self, _pdim: Ref<'_, ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    fn OnUninitDocumentMgr(&self, _pdim: Ref<'_, ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    fn OnSetFocus(
        &self,
        focus: Ref<'_, ITfDocumentMgr>,
        _prev: Ref<'_, ITfDocumentMgr>,
    ) -> Result<()> {
        guard("focus", || {
            self.watch(focus.ok().ok());
            Ok(())
        })
    }

    fn OnPushContext(&self, pic: Ref<'_, ITfContext>) -> Result<()> {
        guard("push context", || {
            if let Ok(ctx) = pic.ok() {
                let dim = unsafe { ctx.GetDocumentMgr()? };
                self.watch(Some(&dim));
            }
            Ok(())
        })
    }

    fn OnPopContext(&self, _pic: Ref<'_, ITfContext>) -> Result<()> {
        Ok(())
    }
}

impl ITfTextEditSink_Impl for TextService_Impl {
    fn OnEndEdit(
        &self,
        pic: Ref<'_, ITfContext>,
        ec: u32,
        _rec: Ref<'_, ITfEditRecord>,
    ) -> Result<()> {
        guard("end edit", || {
            let Ok(ctx) = pic.ok() else { return Ok(()) };
            self.on_text_changed(ctx, ec)
        })
    }
}

impl TextService_Impl {
    fn unwatch(&self) {
        let old = self.state.borrow_mut().watched.take();
        if let Some((ctx, cookie)) = old
            && let Ok(src) = ctx.cast::<ITfSource>()
        {
            let _ = unsafe { src.UnadviseSink(cookie) };
        }
    }

    /// Fokusdagi hujjatning yuqori kontekstida matn o'zgarishini kuzatishni boshlaydi.
    fn watch(&self, dim: Option<&ITfDocumentMgr>) {
        self.unwatch();
        {
            let mut st = self.state.borrow_mut();
            st.last.clear();
            st.session.reset();
            st.blind_logged = false;
            st.blind = true;
            st.typed.boundary();
            st.undo = None;
        }
        let Some(dim) = dim else { return };
        let Ok(ctx) = (unsafe { dim.GetTop() }) else {
            return;
        };
        let me: IUnknown = self.to_object().to_interface();
        let Ok(src) = ctx.cast::<ITfSource>() else {
            return;
        };
        match unsafe { src.AdviseSink(&ITfTextEditSink::IID, &me) } {
            Ok(cookie) => self.state.borrow_mut().watched = Some((ctx, cookie)),
            Err(e) => log::write(&format!("advise edit sink failed hr={:#x}", e.code().0)),
        }
    }

    fn on_text_changed(&self, ctx: &ITfContext, ec: u32) -> Result<()> {
        let Some((_, before, reached_start)) = read_before_caret(ctx, ec)? else {
            return Ok(());
        };
        let prev = std::mem::replace(&mut self.state.borrow_mut().last, before.clone());
        {
            // Asosiy yo'lga faqat kontekst bosilayotgan harflarni haqiqatan ko'rsatsa o'tamiz. Telegram fokusda
            // bir marta eski matnni (qoralama, oldingi xabar) beradi, keyin yangilamaydi: shunga ishonilsa,
            // birinchi xabar butunlay o'girilmay qolardi.
            let mut st = self.state.borrow_mut();
            let live = !prev.is_empty() && !before.is_empty() && prev != before;
            if st.blind && (live || st.typed.mirrored_in(&before)) {
                st.blind = false;
                log::write("context mirrors typing: TSF path");
            }
        }
        if before.is_empty() {
            let mut st = self.state.borrow_mut();
            if !st.blind_logged {
                st.blind_logged = true;
                log::write("no text before caret in this context (app does not expose text)");
            }
            return Ok(());
        }
        // Faqat bitta bo'shliq yozilganda: yangi = eski + bo'shliq (bufer 64 da kesilganini ham hisobga olib).
        let Some(&last) = before.last() else {
            return Ok(());
        };
        if !matches!(last, 0x20 | 0x09 | 0x0A | 0x0D | 0xA0) {
            return Ok(());
        }
        let head = &before[..before.len() - 1];
        // Qat'iy: avvalgi matn aynan shu (yoki 64 dan uzun bo'lib kesilgan) — joylashtirish (paste) emas.
        let typed_space = prev.ends_with(head);
        if !typed_space || prev.is_empty() {
            return Ok(());
        }
        let Some(t) = plan::word_before_caret(&before, reached_start) else {
            return Ok(());
        };
        if t.trailing != 1 || !self.state.borrow_mut().converting() {
            return Ok(());
        }
        let edit = {
            let mut st = self.state.borrow_mut();
            let st = &mut *st;
            st.session.complete_word(&t.word, &st.cfg.opts)
        };
        if last == 0x0A || last == 0x0D {
            self.state.borrow_mut().session.reset();
        }
        let Some(edit) = edit else {
            log::write("word unchanged");
            return Ok(());
        };
        let es: ITfEditSession = Replace {
            ctx: ctx.clone(),
            word: t.word,
            insert: edit.insert,
            state: self.state.clone(),
        }
        .into();
        let tid = self.state.borrow().tid;
        if let Err(e) = unsafe { ctx.RequestEditSession(tid, &es, TF_ES_ASYNC | TF_ES_READWRITE) } {
            log::write(&format!("request edit failed hr={:#x}", e.code().0));
        }
        Ok(())
    }
}

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, _foreground: BOOL) -> Result<()> {
        if let Ok(mut st) = self.state.try_borrow_mut() {
            st.typed.boundary();
            st.undo = None;
        }
        Ok(())
    }

    /// TRUE faqat ushlanadigan tugmada (tartib belgisi yoki o'girishni bekor qiluvchi Backspace) —
    /// keyin TSF `OnKeyDown`ni chaqiradi. Qolgan hamma tugma kuzatiladi va dasturga boradi.
    fn OnTestKeyDown(&self, pic: Ref<'_, ITfContext>, w: WPARAM, l: LPARAM) -> Result<BOOL> {
        let t0 = Instant::now();
        let eat = guard("key", || {
            if pic.is_null() {
                // Matn maydoni emas (Explorer ro'yxati, o'yin): Backspace bu yerda boshqa ish qiladi — tegmaymiz.
                self.state
                    .borrow_mut()
                    .typed
                    .lost("key outside a text field");
                return Ok(false);
            }
            self.sync_preserved();
            let (vk, l) = (w.0 as u16, l.0);
            // Zaxira tahririmiz (Backspace + yangi so'z) — dasturga o'tadi, so'z kuzatuviga kirmaydi.
            // Navbat tartibi: tahrirdan oldin qayta yuborilganlar, keyin tahrir tugmalari, keyin qolganlar.
            let resent_first = self.state.borrow().resent_first();
            if !resent_first && self.state.borrow_mut().take_retype(vk) {
                return Ok(false);
            }
            // Tahrirdan keyin qayta yuborilgan tugmamiz — endi to'g'ri tartibda, so'zga qo'shiladi.
            let resent = self.state.borrow_mut().take_resent(vk);
            if resent.is_none() && resent_first && self.state.borrow_mut().take_retype(vk) {
                return Ok(false);
            }
            if let Some(r) = resent {
                // Oldingi qayta yuborilgan tugma yangi o'girishni boshlagan bo'lsa, qolganlari endi uning
                // tahriridan oldin turibdi — yana navbat oxiriga.
                let mut st = self.state.borrow_mut();
                if st.retyping() {
                    st.hold = Some((vk, l, r));
                    return Ok(true);
                }
                drop(st);
                self.observe_key(vk, l, Some(r));
                return Ok(false);
            }
            // Tahririmiz hali navbatda: bu tugma undan oldin yetsa, Backspace'larimiz uni o'chiradi —
            // ushlanadi va tahrirdan keyin (navbat oxiriga) qayta yuboriladi.
            if let Some(r) = self.hold_ahead(vk, l) {
                let mut st = self.state.borrow_mut();
                if st.resent.is_empty() && st.hold.is_none() {
                    log::write("fallback: keys typed ahead of retype, reordered");
                }
                st.hold = Some((vk, l, r));
                return Ok(true);
            }
            // O'zimiz yuborgan tugma — dasturga o'tadi.
            if !own_key()
                && let Ok(ctx) = pic.ok()
                && (self.transitory_eat(ctx, vk, l).is_some()
                    || self.mapped(vk, l).is_some()
                    || self.undo_ready(vk))
            {
                return Ok(true);
            }
            self.observe_key(vk, l, None);
            Ok(false)
        })?;
        slow("test key", t0);
        Ok(eat.into())
    }

    fn OnTestKeyUp(&self, _pic: Ref<'_, ITfContext>, _w: WPARAM, _l: LPARAM) -> Result<BOOL> {
        Ok(false.into())
    }

    fn OnKeyDown(&self, pic: Ref<'_, ITfContext>, w: WPARAM, l: LPARAM) -> Result<BOOL> {
        let t0 = Instant::now();
        let eaten = guard("key down", || {
            let (vk, l) = (w.0 as u16, l.0);
            let hold = self.state.borrow_mut().hold.take();
            if let Some((hvk, hl, r)) = hold.filter(|h| h.0 == vk) {
                resend(hvk, hl, r);
                self.state
                    .borrow_mut()
                    .resent
                    .push_back((r, Instant::now() + RETYPE_WINDOW));
                return Ok(true);
            }
            let Ok(ctx) = pic.ok() else { return Ok(false) };
            if own_key() {
                return Ok(false);
            }
            match self.transitory_eat(ctx, vk, l) {
                Some(Eat::Char(c)) => {
                    let mut st = self.state.borrow_mut();
                    st.undo = None;
                    st.expect_remap(c);
                    drop(st);
                    send_char(c); // kelganda observe_key so'zga qo'shadi va navbatdan chiqaradi
                    return Ok(true);
                }
                Some(Eat::Key) => {
                    resend_key(vk, l);
                    return Ok(true);
                }
                None => {}
            }
            if let Some(c) = self.mapped(vk, l) {
                self.type_char(ctx, c);
                return Ok(true);
            }
            if self.undo_ready(vk) {
                self.restore(ctx);
                return Ok(true);
            }
            Ok(false)
        })?;
        slow("key down", t0);
        Ok(eaten.into())
    }

    fn OnKeyUp(&self, _pic: Ref<'_, ITfContext>, _w: WPARAM, _l: LPARAM) -> Result<BOOL> {
        Ok(false.into())
    }

    /// O'ng Alt + tugma (tartibning AltGr qatlami).
    fn OnPreservedKey(&self, pic: Ref<'_, ITfContext>, guid: *const GUID) -> Result<BOOL> {
        let eaten = guard("preserved key", || {
            let (Ok(ctx), Some(g)) = (pic.ok(), unsafe { guid.as_ref() }) else {
                return Ok(false);
            };
            let Some((scan, shift)) = parse_preserved(g) else {
                return Ok(false);
            };
            self.sync_preserved();
            let c = {
                let mut st = self.state.borrow_mut();
                if !st.refresh() {
                    return Ok(false);
                }
                let caps = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0;
                st.keymap
                    .as_ref()
                    .and_then(|m| m.lookup(scan, shift, caps, true))
            };
            let Some(c) = c else { return Ok(false) };
            self.type_char(ctx, c);
            suppress_menu();
            Ok(true)
        })?;
        Ok(eaten.into())
    }
}

impl TextService_Impl {
    /// IMM32 (transitory) kontekstda belgimiz dasturga keyinroq yetadi, oddiy tugma esa darhol — tartib
    /// buzilmasin deb, navbatda belgimiz turganda keyingi tugmalar ham ushlanib, ketma-ket qayta yuboriladi.
    fn transitory_eat(&self, ctx: &ITfContext, vk: u16, lparam: isize) -> Option<Eat> {
        let transitory =
            unsafe { ctx.GetStatus() }.is_ok_and(|s| s.dwStaticFlags & TF_SS_TRANSITORY != 0);
        if !transitory {
            return None;
        }
        if let Some(c) = self.mapped(vk, lparam) {
            return Some(Eat::Char(c));
        }
        let mut st = self.state.borrow_mut();
        st.purge_remap();
        let k = VIRTUAL_KEY(vk);
        if st.remap.is_empty()
            || k == VK_PACKET
            || vk == 0xE8
            || held(VK_CONTROL)
            || held(VK_MENU)
            || held(VK_LWIN)
            || held(VK_RWIN)
            || matches!(
                k,
                VK_SHIFT
                    | VK_LSHIFT
                    | VK_RSHIFT
                    | VK_CONTROL
                    | VK_MENU
                    | VK_CAPITAL
                    | VK_LWIN
                    | VK_RWIN
            )
        {
            return None;
        }
        // Belgi Shift holati bilan hozir aniqlanadi (qayta yuborilganda Shift qo'yib yuborilgan bo'lishi mumkin).
        // Bo'shliq va boshqaruv tugmalari tugma sifatida qayta yuboriladi — zaxira yo'l ularni so'z chegarasi deb biladi.
        Some(match key_char(vk, lparam) {
            Some(c) if !c.is_whitespace() => Eat::Char(c),
            _ => Eat::Key,
        })
    }

    /// Tugma tartibda belgiga ega bo'lsa — o'sha belgi. Ctrl/Alt/Win bilan — hech qachon (tezkor tugmalar).
    /// Skan-kod VK'dan olinadi: WPF kabi dasturlar TSF'ga lParam'ni to'liq bermaydi.
    fn mapped(&self, vk: u16, _lparam: isize) -> Option<char> {
        // Faqat harf, raqam va OEM tugmalari (numpad, strelka, F1... — VK'lari boshqa).
        if !matches!(vk, 0x30..=0x39 | 0x41..=0x5A | 0xBA..=0xC0 | 0xDB..=0xDF | 0xE2) {
            return None;
        }
        if held(VK_CONTROL) || held(VK_MENU) || held(VK_LWIN) || held(VK_RWIN) {
            return None;
        }
        let hkl = unsafe { GetKeyboardLayout(0) };
        let scan =
            (unsafe { MapVirtualKeyExW(vk as u32, MAPVK_VK_TO_VSC, Some(hkl)) } & 0xFF) as u8;
        let mut st = self.state.borrow_mut();
        if !st.refresh() {
            return None;
        }
        let km = st.keymap.as_ref()?;
        let caps = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0;
        km.lookup(scan, held(VK_SHIFT), caps, false)
    }

    /// Belgini yozadi va so'z kuzatuviga qo'shadi. Tahrir ishlamasa — Unicode tugma sifatida yuboriladi.
    fn type_char(&self, ctx: &ITfContext, c: char) {
        let mut buf = [0u16; 2];
        let done = Rc::new(Cell::new(false));
        let es: ITfEditSession = Insert {
            ctx: ctx.clone(),
            text: c.encode_utf16(&mut buf).to_vec(),
            done: done.clone(),
        }
        .into();
        let tid = {
            let mut st = self.state.borrow_mut();
            st.undo = None;
            st.tid
        };
        match run_session(ctx, tid, &es, &done) {
            Outcome::Done | Outcome::Pending => self.state.borrow_mut().typed.push(c),
            Outcome::Failed => {
                log::write("keymap: edit session failed, sending as unicode key");
                self.state.borrow_mut().expect_remap(c);
                send_char(c); // observe_key uni navbat bo'yicha so'zga qo'shadi
            }
        }
    }

    /// Backspace oxirgi o'girishni bekor qiladimi (asosiy yo'l; zaxira yo'l observe_key'da).
    fn undo_ready(&self, vk: u16) -> bool {
        if VIRTUAL_KEY(vk) != VK_BACK
            || held(VK_CONTROL)
            || held(VK_MENU)
            || held(VK_SHIFT)
            || held(VK_LWIN)
            || held(VK_RWIN)
        {
            return false;
        }
        let st = self.state.borrow();
        st.cfg.undo_backspace && st.undo.as_ref().is_some_and(|u| !u.blind)
    }

    fn restore(&self, ctx: &ITfContext) {
        let Some(u) = self.state.borrow_mut().undo.take() else {
            return;
        };
        // Natijani sessiyaning o'zi hal qiladi (asinxron bajarilsa ham): mos — istisno, mos emas — oddiy Backspace.
        let done = Rc::new(Cell::new(false));
        let es: ITfEditSession = Restore {
            ctx: ctx.clone(),
            converted: u.converted,
            original: u.original,
            state: self.state.clone(),
            done: done.clone(),
        }
        .into();
        let tid = self.state.borrow().tid;
        if run_session(ctx, tid, &es, &done) == Outcome::Failed {
            plain_backspace(&self.state);
        }
    }

    fn remember_exception(&self, original: &str) {
        remember_exception(&self.state, original);
    }

    /// AltGr qatlamidagi tugmalarni TSF'da ro'yxatga oladi (tartib o'zgarganda qayta).
    fn sync_preserved(&self) {
        let (tm, tid, old, want) = {
            let mut st = self.state.borrow_mut();
            st.refresh();
            if !st.keys_dirty {
                return;
            }
            let Some(tm) = st.thread_mgr.clone() else {
                return;
            };
            st.keys_dirty = false;
            let mut want = Vec::new();
            if st.active
                && let Some(km) = &st.keymap
            {
                let hkl = unsafe { GetKeyboardLayout(0) };
                for &(_, scan) in keymap::KEYS {
                    if km.get(scan, 2).is_none() && km.get(scan, 3).is_none() {
                        continue;
                    }
                    let vk = unsafe { MapVirtualKeyExW(scan as u32, MAPVK_VSC_TO_VK, Some(hkl)) };
                    if vk == 0 {
                        continue;
                    }
                    for shift in [false, true] {
                        // Belgi bermaydigan holat ro'yxatga olinmaydi (tugma dasturga o'tsin).
                        if km.lookup(scan, shift, false, true).is_none()
                            && km.lookup(scan, shift, true, true).is_none()
                        {
                            continue;
                        }
                        let mods = TF_MOD_RALT | if shift { TF_MOD_SHIFT } else { 0 };
                        want.push((
                            preserved_guid(scan, shift),
                            TF_PRESERVEDKEY {
                                uVKey: vk,
                                uModifiers: mods,
                            },
                        ));
                    }
                }
            }
            (tm, st.tid, std::mem::take(&mut st.preserved), want)
        };
        let Ok(ks) = tm.cast::<ITfKeystrokeMgr>() else {
            return;
        };
        for (g, k) in &old {
            let _ = unsafe { ks.UnpreserveKey(g, k) };
        }
        let desc: Vec<u16> = "bitg".encode_utf16().collect();
        let mut ok = Vec::new();
        for (g, k) in want {
            match unsafe { ks.PreserveKey(tid, &g, &k, &desc) } {
                Ok(()) => ok.push((g, k)),
                // Boshqa IME yoki dastur shu tugmani olgan — u qatlam ishlamaydi.
                Err(e) => log::write(&format!("preserve key failed hr={:#x}", e.code().0)),
            }
        }
        log::write(&format!("altgr keys: {}", ok.len()));
        self.state.borrow_mut().preserved = ok;
    }
}

fn slow(what: &str, t0: Instant) {
    let ms = t0.elapsed();
    if ms >= SLOW_KEY {
        log::write(&format!("slow {what}: {} ms", ms.as_millis()));
    }
}

fn held(vk: VIRTUAL_KEY) -> bool {
    let s = unsafe { GetKeyState(vk.0 as i32) };
    s < 0
}

/// Tugma qaysi belgini beradi (joriy tartib, Shift/CapsLock bilan). Klaviatura holatini o'zgartirmaydi.
fn key_char(vk: u16, lparam: isize) -> Option<char> {
    let mut state = [0u8; 256];
    unsafe { GetKeyboardState(&mut state).ok()? };
    let scan = ((lparam >> 16) & 0xFF) as u32;
    let mut buf = [0u16; 4];
    // 0x4: dead-key holatini buzmaslik (Windows 10 1607+)
    let n = unsafe {
        ToUnicodeEx(
            vk as u32,
            scan,
            &state,
            &mut buf,
            0x4,
            Some(GetKeyboardLayout(0)),
        )
    };
    if n != 1 {
        return None;
    }
    char::from_u32(buf[0] as u32).filter(|c| !c.is_control())
}

fn key_input(vk: VIRTUAL_KEY, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    tagged_input(vk, scan, flags, INJECT_TAG)
}

fn retype_input(vk: VIRTUAL_KEY, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    tagged_input(vk, scan, flags, RETYPE_TAG)
}

fn tagged_input(vk: VIRTUAL_KEY, scan: u16, flags: KEYBD_EVENT_FLAGS, tag: usize) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: tag,
            },
        },
    }
}

impl TextService_Impl {
    /// Tahririmiz navbatda turganda kelgan tugma ushlanadimi va qanday qayta yuboriladi. Modifikatorlar va
    /// Ctrl/Alt/Win bilan bosilganlar o'tadi (holati buzilmasin; tezkor tugma matnga yozmaydi).
    fn hold_ahead(&self, vk: u16, lparam: isize) -> Option<Resent> {
        let k = VIRTUAL_KEY(vk);
        if vk == 0xE8
            || matches!(
                k,
                VK_PACKET
                    | VK_SHIFT
                    | VK_LSHIFT
                    | VK_RSHIFT
                    | VK_CONTROL
                    | VK_LCONTROL
                    | VK_RCONTROL
                    | VK_MENU
                    | VK_LMENU
                    | VK_RMENU
                    | VK_LWIN
                    | VK_RWIN
                    | VK_CAPITAL
            )
            || held(VK_CONTROL)
            || held(VK_MENU)
            || held(VK_LWIN)
            || held(VK_RWIN)
            || !self.state.borrow_mut().retype_pending()
        {
            return None;
        }
        // Belgi hozir (Shift va tartib shu paytdagi holatda) aniqlanadi; bo'shliq va boshqaruv — tugma.
        let c = self
            .mapped(vk, lparam)
            .or_else(|| key_char(vk, lparam))
            .filter(|c| !c.is_whitespace() && (*c as u32) < 0x10000);
        Some(match c {
            Some(c) => Resent::Char(c),
            None => Resent::Key(vk),
        })
    }

    fn observe_key(&self, vk: u16, lparam: isize, resent: Option<Resent>) {
        // Menyuni to'xtatuvchi belgisiz tugma (suppress_menu) — so'z kuzatuviga ta'sir qilmaydi.
        if vk == 0xE8 {
            return;
        }
        let mut st = self.state.borrow_mut();
        let k = VIRTUAL_KEY(vk);
        // Oldindan yozilgan harf, tahrirdan keyin qayta yuborilgan: so'zning davomi.
        if let Some(Resent::Char(c)) = resent {
            st.undo = None;
            st.typed.push(c);
            return;
        }
        // Tartib belgisi tahrir sessiyasisiz yuborilgan (send_char): so'zning davomi.
        if k == VK_PACKET {
            let extra = unsafe { GetMessageExtraInfo() }.0 as u64;
            st.purge_remap();
            let tagged = extra >> 32 == REMAP_TAG;
            if tagged || !st.remap.is_empty() {
                let front = st.remap.pop_front().map(|(c, _)| c);
                // Belgi bo'lsa — undan, bo'lmasa (Qt) navbatdan. Surrogatning ikkinchi yarmida 0 — o'tkaziladi.
                let c = if tagged {
                    char::from_u32(extra as u32)
                } else {
                    front
                };
                if let Some(c) = c.filter(|&c| c != '\0') {
                    st.typed.push(c);
                }
                return;
            }
        }
        // Zaxira yo'l tahririmiz (o'chirish + qayta yozish) — so'z kuzatuviga kirmaydi.
        if unsafe { GetMessageExtraInfo() }.0 as usize == RETYPE_TAG {
            return;
        }
        // O'girishni bekor qilish faqat darhol keyingi Backspace bilan.
        if !matches!(k, VK_BACK | VK_SHIFT | VK_LSHIFT | VK_RSHIFT) {
            st.undo = None;
        }
        // Tugmalar orasida sichqoncha bosilgan bo'lsa — kursor boshqa joyda bo'lishi mumkin.
        let clicked = unsafe {
            (GetAsyncKeyState(VK_LBUTTON.0 as i32) & 1) != 0
                || (GetAsyncKeyState(VK_RBUTTON.0 as i32) & 1) != 0
        };
        if clicked {
            st.undo = None;
            // So'z boshlanmagan bo'lsa (maydonga bosib, yozishni boshlash) — yangi so'z boshi.
            if st.typed.is_empty() {
                st.typed.boundary();
            } else {
                st.typed.lost("mouse click inside the word");
            }
        }
        if held(VK_CONTROL) || held(VK_MENU) || held(VK_LWIN) || held(VK_RWIN) {
            st.typed.lost("Ctrl, Alt or Win held"); // Ctrl+V, Ctrl+Z, Alt+Tab ...
            return;
        }
        match k {
            VK_SPACE => {
                let had_letters = !st.typed.is_empty();
                let word = st.typed.finish();
                if !st.blind {
                    return; // matn o'qiladigan kontekst — asosiy yo'l ishlaydi
                }
                let Some(word) = word else {
                    if had_letters {
                        log::write(&format!("fallback: word not tracked: {}", st.typed.why()));
                    }
                    return;
                };
                if !st.converting() || !st.cfg.fallback {
                    return;
                }
                let st = &mut *st;
                let Some(edit) = st.session.complete_word(&word, &st.cfg.opts) else {
                    // Uzunlik va toifa — matnning o'zi logga yozilmaydi.
                    log::write(&format!(
                        "fallback: word unchanged (len {}{})",
                        word.chars().count(),
                        if is_known_exception(st, &word) {
                            ", exception"
                        } else {
                            ""
                        }
                    ));
                    return;
                };
                if drop_and_retype(word.chars().count(), &edit.insert, true) {
                    st.expect_retype(
                        word.chars().count() + 1,
                        edit.insert.encode_utf16().count() + 1,
                    );
                    st.undo = Some(Undo {
                        original: word,
                        converted: edit.insert,
                        blind: true,
                    });
                }
            }
            VK_RETURN | VK_TAB => {
                st.typed.boundary();
                st.session.reset();
            }
            VK_BACK => {
                let undo = st.undo.take().filter(|u| u.blind && st.cfg.undo_backspace);
                let Some(u) = undo else {
                    st.typed.backspace();
                    return;
                };
                // Bu Backspace probelni o'chiradi; so'ng o'girilgan so'z o'rniga asli yoziladi.
                let back = u.converted.chars().count().saturating_sub(1);
                if drop_and_retype(back, &u.original, false) {
                    st.expect_retype(back + 1, u.original.encode_utf16().count());
                    drop(st);
                    self.remember_exception(&u.original);
                }
            }
            VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_HOME | VK_END | VK_PRIOR | VK_NEXT
            | VK_DELETE | VK_ESCAPE => {
                st.typed.lost("arrows, Home/End or Delete");
                st.session.reset();
            }
            VK_PACKET => {
                st.typed.lost("unicode key from another program");
                st.session.reset();
            }
            _ => {
                // Shift, CapsLock, F1... belgi bermaydi — e'tiborsiz.
                if let Some(c) = key_char(vk, lparam) {
                    st.typed.push(c);
                }
            }
        }
    }
}

fn is_known_exception(st: &State, word: &str) -> bool {
    let core = bitg_core::token::split_edges(word).core;
    st.cfg.opts.exceptions.contains(&fold_key(core))
}

/// Bekor qilingan so'z shu dasturda boshqa o'girilmaydi. Doimiy istisno (`exceptions.txt`) faqat
/// `undo_remember=1` bo'lsa: bo'shliqni o'chirmoqchi bo'lgan bitta Backspace so'zni abadiy o'chirib qo'ymasin.
fn remember_exception(state: &Rc<RefCell<State>>, original: &str) {
    let Ok(mut st) = state.try_borrow_mut() else {
        return;
    };
    let core = bitg_core::token::split_edges(original).core.to_string();
    let k = fold_key(&core);
    if !st.cfg.opts.exceptions.contains(&k) {
        st.cfg.opts.exceptions.push(k.clone());
    }
    st.session.reset();
    st.typed.lost("undo");
    if !st.cfg.undo_remember {
        log::write("undo: restored (kept for this app only)");
        return;
    }
    match config::add_exception(&core) {
        Ok(()) => log::write("undo: restored, added to exceptions"),
        Err(e) => log::write(&format!("undo: restored, exceptions write failed: {e}")),
    }
    // O'z yozuvimiz fayl holatini o'zgartirdi — qayta o'qish shart emas.
    st.stamp = config::stamp();
}

/// Ushlangan Backspace bekor qilishga yaramadi (matn o'zgargan) — odatdagidek ishlasin.
fn plain_backspace(state: &Rc<RefCell<State>>) {
    log::write("undo: text changed, plain backspace");
    if let Ok(mut st) = state.try_borrow_mut() {
        st.expect_retype(1, 0);
    }
    let inputs = [
        retype_input(VK_BACK, 0, KEYBD_EVENT_FLAGS(0)),
        retype_input(VK_BACK, 0, KEYEVENTF_KEYUP),
    ];
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
}

/// Bosilgan tugma (probel yoki Backspace) navbatda bizdan oldin turibdi: u ishlagach `back` ta
/// Backspace + 1 va yangi matn (+ probel) yuboriladi.
fn drop_and_retype(back: usize, insert: &str, space: bool) -> bool {
    let mut inputs = Vec::new();
    for _ in 0..=back {
        inputs.push(retype_input(VK_BACK, 0, KEYBD_EVENT_FLAGS(0)));
        inputs.push(retype_input(VK_BACK, 0, KEYEVENTF_KEYUP));
    }
    let units: Vec<u16> = insert.encode_utf16().chain(space.then_some(0x20)).collect();
    for &u in &units {
        inputs.push(retype_input(VIRTUAL_KEY(0), u, KEYEVENTF_UNICODE));
        inputs.push(retype_input(
            VIRTUAL_KEY(0),
            u,
            KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
        ));
    }
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) } as usize;
    if sent == inputs.len() {
        log::write(if space {
            "fallback: converted"
        } else {
            "fallback: undo"
        });
        true
    } else {
        log::write(&format!("fallback: SendInput sent {sent}/{}", inputs.len()));
        false
    }
}

/// Kursor (bo'sh tanlov) va undan oldingi matn. Tanlov bo'sh bo'lmasa — None.
fn read_before_caret(ctx: &ITfContext, ec: u32) -> Result<Option<(ITfRange, Vec<u16>, bool)>> {
    let mut sel = [TF_SELECTION::default()];
    let mut fetched = 0u32;
    unsafe { ctx.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut fetched)? };
    let [s] = sel;
    let range = std::mem::ManuallyDrop::into_inner(s.range);
    if fetched != 1 {
        return Ok(None);
    }
    let Some(caret) = range else { return Ok(None) };
    if !unsafe { caret.IsEmpty(ec)? }.as_bool() {
        return Ok(None);
    }
    let before = unsafe { caret.Clone()? };
    let mut moved = 0i32;
    unsafe { before.ShiftStart(ec, -LOOKBACK, &mut moved, std::ptr::null())? };
    let want = (-moved) as usize;
    let mut buf = vec![0u16; want];
    let mut got = 0u32;
    if want > 0 {
        unsafe { before.GetText(ec, 0, &mut buf, &mut got)? };
    }
    buf.truncate(got as usize);
    Ok(Some((caret, buf, moved > -LOOKBACK)))
}

/// Asinxron tahrir: matn hali ham "so'z + bitta bo'shliq" bo'lsa — so'zni almashtiradi.
#[implement(ITfEditSession)]
struct Replace {
    ctx: ITfContext,
    word: String,
    insert: String,
    state: Rc<RefCell<State>>,
}

impl ITfEditSession_Impl for Replace_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        guard("edit session", || {
            let r = self.run(ec)?;
            log::write(r);
            Ok(())
        })
    }
}

impl Replace_Impl {
    fn run(&self, ec: u32) -> Result<&'static str> {
        let Some((caret, before, reached)) = read_before_caret(&self.ctx, ec)? else {
            return Ok("skip: selection changed");
        };
        let Some(t) = plan::word_before_caret(&before, reached) else {
            return Ok("skip: word gone");
        };
        if t.word != self.word || t.trailing != 1 {
            return Ok("skip: text changed meanwhile");
        }
        let target = unsafe { caret.Clone()? };
        let back = (t.trailing + t.word_len) as i32;
        let mut m = 0i32;
        unsafe {
            target.ShiftStart(ec, -back, &mut m, std::ptr::null())?;
            if m != -back {
                return Ok("skip: range shift mismatch");
            }
        }
        // Bo'shliq ham almashtiriladi (so'z + bo'shliq → yangi so'z + o'sha bo'shliq). Chromium/Electron
        // (ChatGPT, Chrome) faqat so'z almashtirilganda yangi yozilgan probelni yo'qotadi.
        // Yangi qator (Enter) qayta yozilmaydi: chat dasturida u xabarni yuboradi.
        let trail = before.last().copied().unwrap_or(0x20);
        let keep_trail = matches!(trail, 0x20 | 0xA0 | 0x09);
        let mut text: Vec<u16> = self.insert.encode_utf16().collect();
        if keep_trail {
            text.push(trail);
        } else {
            unsafe { target.ShiftEnd(ec, -(t.trailing as i32), &mut m, std::ptr::null())? };
        }
        unsafe { target.SetText(ec, 0, &text)? };

        // Kursor: so'z va bo'shliqdan keyin.
        let after = unsafe { target.Clone()? };
        unsafe {
            after.Collapse(ec, TF_ANCHOR_END)?;
            if !keep_trail {
                after.ShiftEnd(ec, t.trailing as i32, &mut m, std::ptr::null())?;
                after.Collapse(ec, TF_ANCHOR_END)?;
            }
        }
        let sel = TF_SELECTION {
            range: std::mem::ManuallyDrop::new(Some(after)),
            style: TF_SELECTIONSTYLE {
                ase: TF_AE_END,
                fInterimChar: false.into(),
            },
        };
        let r = unsafe { self.ctx.SetSelection(ec, std::slice::from_ref(&sel)) };
        drop(std::mem::ManuallyDrop::into_inner(sel.range));
        r?;
        if keep_trail && let Ok(mut st) = self.state.try_borrow_mut() {
            st.undo = Some(Undo {
                original: self.word.clone(),
                converted: self.insert.clone(),
                blind: false,
            });
        }
        Ok("converted")
    }
}

/// Kursorga belgi yozadi. IMM32 orqali ishlaydigan (transitory) kontekstda — qisqa kompozitsiya bilan,
/// aks holda dastur yozuvni ko'rmaydi.
#[implement(ITfEditSession)]
struct Insert {
    ctx: ITfContext,
    text: Vec<u16>,
    done: Rc<Cell<bool>>,
}

impl ITfEditSession_Impl for Insert_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        guard("insert", || self.run(ec))
    }
}

impl Insert_Impl {
    /// `done` belgi yozilgan zahoti qo'yiladi: keyingi xato (kursor, kompozitsiyani yopish) zaxira yo'lni
    /// ishga tushirmasin — aks holda belgi ikki marta yoziladi.
    fn run(&self, ec: u32) -> Result<()> {
        let transitory =
            unsafe { self.ctx.GetStatus() }.is_ok_and(|s| s.dwStaticFlags & TF_SS_TRANSITORY != 0);
        let ins: ITfInsertAtSelection = self.ctx.cast()?;
        if !transitory {
            let range = unsafe {
                ins.InsertTextAtSelection(ec, INSERT_TEXT_AT_SELECTION_FLAGS(0), &self.text)?
            };
            self.done.set(true);
            return set_caret_after(&self.ctx, ec, &range);
        }
        let range = unsafe { ins.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &[])? };
        let cc: ITfContextComposition = self.ctx.cast()?;
        let comp = unsafe { cc.StartComposition(ec, &range, None::<&ITfCompositionSink>)? };
        let r = unsafe { range.SetText(ec, 0, &self.text) };
        if r.is_ok() {
            self.done.set(true);
            let _ = set_caret_after(&self.ctx, ec, &range);
        }
        // Kompozitsiya har holda yopiladi — aks holda dastur "yozilmoqda" holatida qoladi.
        let end = unsafe { comp.EndComposition(ec) };
        r.and(end)
    }
}

/// Backspace: "o'girilgan so'z + bo'shliq" → asl so'z (bo'shliq Backspace kabi o'chadi).
/// Natija shu yerda hal bo'ladi (asinxron bajarilsa ham): mos — istisno, mos emas — oddiy Backspace.
#[implement(ITfEditSession)]
struct Restore {
    ctx: ITfContext,
    converted: String,
    original: String,
    state: Rc<RefCell<State>>,
    /// Sessiya bajarildi (natijasi qanday bo'lmasin).
    done: Rc<Cell<bool>>,
}

impl ITfEditSession_Impl for Restore_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        self.done.set(true);
        let restored = guard("restore", || self.run(ec))?;
        if restored {
            remember_exception(&self.state, &self.original);
        } else {
            plain_backspace(&self.state);
        }
        Ok(())
    }
}

impl Restore_Impl {
    fn run(&self, ec: u32) -> Result<bool> {
        let Some((caret, before, reached)) = read_before_caret(&self.ctx, ec)? else {
            return Ok(false);
        };
        let Some(t) = plan::word_before_caret(&before, reached) else {
            return Ok(false);
        };
        if t.word != self.converted || t.trailing != 1 {
            return Ok(false);
        }
        let target = unsafe { caret.Clone()? };
        let back = (t.trailing + t.word_len) as i32;
        let mut m = 0i32;
        unsafe { target.ShiftStart(ec, -back, &mut m, std::ptr::null())? };
        if m != -back {
            return Ok(false);
        }
        let text: Vec<u16> = self.original.encode_utf16().collect();
        unsafe { target.SetText(ec, 0, &text)? };
        let _ = set_caret_after(&self.ctx, ec, &target);
        Ok(true)
    }
}

fn set_caret_after(ctx: &ITfContext, ec: u32, range: &ITfRange) -> Result<()> {
    let after = unsafe { range.Clone()? };
    unsafe { after.Collapse(ec, TF_ANCHOR_END)? };
    let sel = TF_SELECTION {
        range: std::mem::ManuallyDrop::new(Some(after)),
        style: TF_SELECTIONSTYLE {
            ase: TF_AE_END,
            fInterimChar: false.into(),
        },
    };
    let r = unsafe { ctx.SetSelection(ec, std::slice::from_ref(&sel)) };
    drop(std::mem::ManuallyDrop::into_inner(sel.range));
    r
}

#[derive(PartialEq, Eq, Debug)]
enum Outcome {
    Done,
    /// Keyinroq bajariladi — zaxira yo'l ishlatilmaydi (ikki marta yozilmasin).
    Pending,
    Failed,
}

/// Sinxron tahrir (tugma ishlovchisi ichida ruxsat). Rad etilsa — asinxron.
fn run_session(ctx: &ITfContext, tid: u32, es: &ITfEditSession, done: &Rc<Cell<bool>>) -> Outcome {
    let sync = unsafe { ctx.RequestEditSession(tid, es, TF_ES_SYNC | TF_ES_READWRITE) };
    if done.get() {
        return Outcome::Done;
    }
    if !matches!(sync, Ok(hr) if hr == TF_E_SYNCHRONOUS) {
        return Outcome::Failed;
    }
    match unsafe { ctx.RequestEditSession(tid, es, TF_ES_ASYNCDONTCARE | TF_ES_READWRITE) } {
        _ if done.get() => Outcome::Done,
        Ok(hr) if hr == TS_S_ASYNC => Outcome::Pending,
        _ => Outcome::Failed,
    }
}

/// AltGr qatlami tugmasi GUID'i: skan-kod va Shift ichida.
fn preserved_guid(scan: u8, shift: bool) -> GUID {
    GUID::from_values(
        0xB17C_0000 | (scan as u32) << 8 | shift as u32,
        0x6B17,
        0x4D3A,
        [0x9E, 0x21, 0x5C, 0x0F, 0x7A, 0x33, 0x10, 0x62],
    )
}

fn parse_preserved(g: &GUID) -> Option<(u8, bool)> {
    let base = preserved_guid(0, false);
    (g.data1 >> 16 == 0xB17C
        && g.data2 == base.data2
        && g.data3 == base.data3
        && g.data4 == base.data4)
        .then_some((((g.data1 >> 8) & 0xFF) as u8, g.data1 & 1 == 1))
}

/// O'zimiz yuboradigan belgi (tahrir sessiyasi ishlamaganda): IME uni so'zning davomi deb biladi.
fn send_char(c: char) {
    let mut buf = [0u16; 2];
    let units = c.encode_utf16(&mut buf);
    let mut inputs = Vec::new();
    for (i, &u) in units.iter().enumerate() {
        let extra = (REMAP_TAG << 32) | if i == 0 { c as u64 } else { 0 };
        for f in [KEYEVENTF_UNICODE, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP] {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: u,
                        dwFlags: f,
                        time: 0,
                        dwExtraInfo: extra as usize,
                    },
                },
            });
        }
    }
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
}

/// Ushlangan tugmani navbat oxiriga qayta yuboradi (INJECT_TAG bilan — ikkinchi marta ushlanmaydi).
fn resend_key(vk: u16, lparam: isize) {
    let scan = ((lparam >> 16) & 0xFF) as u16;
    let ext = if (lparam >> 24) & 1 != 0 {
        KEYEVENTF_EXTENDEDKEY
    } else {
        KEYBD_EVENT_FLAGS(0)
    };
    let inputs = [
        key_input(VIRTUAL_KEY(vk), scan, ext),
        key_input(VIRTUAL_KEY(vk), scan, ext | KEYEVENTF_KEYUP),
    ];
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
}

/// Oldindan yozilib ushlangan tugmani navbat oxiriga (zaxira tahriridan keyin) qayta yuboradi.
fn resend(vk: u16, lparam: isize, r: Resent) {
    match r {
        Resent::Char(c) => {
            let u = c as u32 as u16;
            let inputs = [
                key_input(VIRTUAL_KEY(0), u, KEYEVENTF_UNICODE),
                key_input(VIRTUAL_KEY(0), u, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP),
            ];
            unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
        }
        Resent::Key(_) => resend_key(vk, lparam),
    }
}

/// O'ng Alt bosilgan tugma ushlangach Alt qo'yib yuborilsa, dastur menyusi ochiladi. Belgisiz tugma
/// yuborib buni to'xtatamiz (PowerToys usuli).
fn suppress_menu() {
    let inputs = [
        key_input(VIRTUAL_KEY(0xE8), 0, KEYBD_EVENT_FLAGS(0)),
        key_input(VIRTUAL_KEY(0xE8), 0, KEYEVENTF_KEYUP),
    ];
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
}

#[allow(dead_code)]
fn _iids() -> [GUID; 2] {
    [ITfTextInputProcessorEx::IID, ITfTextInputProcessor::IID]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserved_guid_round_trip() {
        for &(_, scan) in keymap::KEYS {
            for shift in [false, true] {
                assert_eq!(
                    parse_preserved(&preserved_guid(scan, shift)),
                    Some((scan, shift))
                );
            }
        }
        assert_eq!(parse_preserved(&GUID::zeroed()), None);
    }

    #[test]
    fn typed_ahead_keys_wait_for_retype_and_return_in_order() {
        let mut st = State::default();
        assert!(!st.retype_pending());
        st.expect_retype(7, 6);
        assert!(st.retype_pending());
        st.retype_back = 0;
        st.retype_packet = 0;
        assert!(!st.retype_pending());

        let until = Instant::now() + RETYPE_WINDOW;
        st.resent.push_back((Resent::Char('o'), until));
        st.resent.push_back((Resent::Key(VK_SPACE.0), until));
        assert!(st.retype_pending());
        assert_eq!(st.take_resent(VK_SPACE.0), None);
        assert_eq!(st.take_resent(VK_PACKET.0), Some(Resent::Char('o')));
        assert_eq!(st.take_resent(VK_SPACE.0), Some(Resent::Key(VK_SPACE.0)));
        assert!(!st.retype_pending());

        // Qayta yuborilganlar navbatda turganda yangi tahrir: ular tahrirdan oldin keladi.
        st.resent.push_back((Resent::Key(VK_BACK.0), until));
        st.expect_retype(5, 4);
        assert!(st.resent_first() && st.retyping());
        assert_eq!(st.take_resent(VK_BACK.0), Some(Resent::Key(VK_BACK.0)));
        assert!(!st.resent_first());
        st.retype_back = 0;
        st.retype_packet = 0;

        // Tartib belgilari: surrogat juft ikki birlik, muddati o'tgani tashlanadi.
        st.expect_remap('ö');
        st.expect_remap('😀');
        assert_eq!(
            st.remap.iter().map(|&(c, _)| c).collect::<String>(),
            "ö😀\0"
        );
        st.remap.push_back(('x', Instant::now()));
        st.purge_remap();
        assert_eq!(st.remap.len(), 3);
        st.remap.clear();

        // Qaytmagan tugma navbatni abadiy band qilmaydi.
        st.resent.push_back((Resent::Char('x'), Instant::now()));
        assert!(!st.retype_pending());
        assert_eq!(st.take_resent(VK_PACKET.0), None);
    }
}
