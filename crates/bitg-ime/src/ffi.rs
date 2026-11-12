//! C ABI (`linux/include/bitg_ime.h`). Barcha satrlar UTF-8. Qaytgan ko'rsatkichlar dvigatelga tegishli va
//! shu dvigatelning keyingi chaqiruvigacha yaroqli — frontend ularni bo'shatmaydi.
//! Har chaqiruv `catch_unwind` ichida: xato bo'lsa tugma odatdagidek dasturga o'tadi, jarayon yiqilmaydi.

use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;

use crate::{Engine, Key, Out, config};

pub const BITG_KEY_CHAR: u32 = 0;
pub const BITG_KEY_SPACE: u32 = 1;
pub const BITG_KEY_ENTER: u32 = 2;
pub const BITG_KEY_TAB: u32 = 3;
pub const BITG_KEY_BACKSPACE: u32 = 4;
pub const BITG_KEY_NAVIGATE: u32 = 5;
pub const BITG_KEY_OTHER: u32 = 6;

#[repr(C)]
pub struct BitgImeOut {
    /// 1 — tugma dasturga yuborilmasin.
    pub consumed: u32,
    /// Kursor oldidan o'chiriladigan belgilar (Unicode scalar), `commit`dan oldin.
    pub delete_before: u32,
    /// Yoziladigan matn yoki NULL.
    pub commit: *const c_char,
    /// 1 — preedit `preedit` bilan almashtirilsin (bo'sh satr — yashirilsin).
    pub preedit_changed: u32,
    pub preedit: *const c_char,
}

pub struct BitgIme {
    engine: Engine,
    commit: CString,
    preedit: CString,
    pending: CString,
    out: BitgImeOut,
}

const PASS: BitgImeOut = BitgImeOut {
    consumed: 0,
    delete_before: 0,
    commit: ptr::null(),
    preedit_changed: 0,
    preedit: ptr::null(),
};

/// O'zgarishsiz javob (statik): tugma dasturga o'tadi.
fn pass() -> *const BitgImeOut {
    &PASS
}

fn cstring(s: String) -> CString {
    // NUL belgisi klaviaturadan kelmaydi; kelsa ham — olib tashlanadi
    CString::new(s.replace('\0', "")).unwrap_or_default()
}

impl BitgIme {
    fn store(&mut self, o: Out) -> *const BitgImeOut {
        self.commit = cstring(o.commit);
        let changed = o.preedit.is_some();
        self.preedit = cstring(o.preedit.unwrap_or_default());
        self.out = BitgImeOut {
            consumed: o.consumed.into(),
            delete_before: u32::try_from(o.delete_before).unwrap_or(0),
            commit: if self.commit.is_empty() {
                ptr::null()
            } else {
                self.commit.as_ptr()
            },
            preedit_changed: changed.into(),
            preedit: self.preedit.as_ptr(),
        };
        &self.out
    }
}

fn guard<T>(fallback: T, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(fallback)
}

/// Yangi dvigatel: sozlamalar `$XDG_CONFIG_HOME/bitg/` dan, lug'atlar shu yerda yuklanadi.
#[unsafe(no_mangle)]
pub extern "C" fn bitg_ime_new() -> *mut BitgIme {
    guard(ptr::null_mut(), || {
        bitg_core::lang::warm_up();
        Box::into_raw(Box::new(BitgIme {
            engine: Engine::new(config::config_dir()),
            commit: CString::default(),
            preedit: CString::default(),
            pending: CString::default(),
            out: PASS,
        }))
    })
}

/// # Safety
/// `ime` — `bitg_ime_new` qaytargan ko'rsatkich yoki NULL; bir marta chaqiriladi.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_ime_free(ime: *mut BitgIme) {
    if !ime.is_null() {
        drop(unsafe { Box::from_raw(ime) });
    }
}

/// Bitta tugma bosildi. `kind` — `BITG_KEY_*`; `codepoint` faqat `BITG_KEY_CHAR`/`BITG_KEY_SPACE` uchun.
/// `before` — kursor oldidagi matn (dastur surrounding text bersa) yoki NULL.
/// Faqat modifikator (Shift, Ctrl, ...) bosilishi va tugma qo'yib yuborilishi bu yerga yuborilmaydi.
///
/// # Safety
/// `ime` — tirik dvigatel; `before` — NUL bilan tugagan UTF-8 satr yoki NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_ime_key(
    ime: *mut BitgIme,
    kind: u32,
    codepoint: u32,
    before: *const c_char,
) -> *const BitgImeOut {
    let Some(ime) = (unsafe { ime.as_mut() }) else {
        return pass();
    };
    let before = if before.is_null() {
        None
    } else {
        // SAFETY: chaqiruvchi NUL bilan tugagan satr beradi; UTF-8 bo'lmasa — matn yo'q deb hisoblaymiz
        unsafe { CStr::from_ptr(before) }.to_str().ok()
    };
    let key = to_key(kind, codepoint);
    guard(pass(), move || {
        let out = ime.engine.key(key, before);
        ime.store(out)
    })
}

fn to_key(kind: u32, codepoint: u32) -> Key {
    match kind {
        BITG_KEY_CHAR | BITG_KEY_SPACE => match char::from_u32(codepoint) {
            Some(c) if kind == BITG_KEY_SPACE => Key::Space(c),
            Some(c) if !c.is_control() => Key::Char(c),
            _ => Key::Other,
        },
        BITG_KEY_ENTER => Key::Enter,
        BITG_KEY_TAB => Key::Tab,
        BITG_KEY_BACKSPACE => Key::Backspace,
        BITG_KEY_NAVIGATE => Key::Navigate,
        _ => Key::Other,
    }
}

/// 1 — shu tugma uchun `bitg_ime_key` ga kursor oldidagi matn kerak; 0 — `before = NULL` berilsa ham natija
/// bir xil. Matnni so'rash qimmat bo'lgan frontend (macOS IMK) har tugmada so'ramasligi uchun.
///
/// # Safety
/// `ime` — tirik dvigatel yoki NULL (NULL — 0).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_ime_wants_before(
    ime: *mut BitgIme,
    kind: u32,
    codepoint: u32,
) -> u32 {
    let Some(ime) = (unsafe { ime.as_mut() }) else {
        return 0;
    };
    // xato bo'lsa — matnni so'raymiz: ortiqcha so'rov xavfsiz, yetishmagani emas
    guard(1, || {
        u32::from(ime.engine.wants_before(to_key(kind, codepoint)))
    })
}

/// # Safety
/// `ime` — tirik dvigatel yoki NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_ime_focus_in(ime: *mut BitgIme) {
    if let Some(ime) = unsafe { ime.as_mut() } {
        guard((), || ime.engine.focus_in());
    }
}

/// Fokus ketdi yoki kursor siljidi. Preedit'dagi o'girilmagan so'zni qaytaradi (bo'lmasa NULL):
/// frontend uni yozishi kerak, agar dastur preedit'ni o'zi yozmasa.
///
/// # Safety
/// `ime` — tirik dvigatel yoki NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_ime_reset(ime: *mut BitgIme) -> *const c_char {
    let Some(ime) = (unsafe { ime.as_mut() }) else {
        return ptr::null();
    };
    let pending = guard(String::new(), || ime.engine.reset());
    if pending.is_empty() {
        return ptr::null();
    }
    ime.pending = cstring(pending);
    ime.pending.as_ptr()
}

/// Parol, PIN yoki terminal maydoni (`on` != 0): dvigatel hech narsaga tegmaydi.
///
/// # Safety
/// `ime` — tirik dvigatel yoki NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_ime_set_suspended(ime: *mut BitgIme, on: u32) -> *const BitgImeOut {
    let Some(ime) = (unsafe { ime.as_mut() }) else {
        return pass();
    };
    guard(pass(), move || {
        let out = ime.engine.set_suspended(on != 0);
        ime.store(out)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(p: *const c_char) -> String {
        if p.is_null() {
            String::new()
        } else {
            unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
        }
    }

    #[test]
    fn abi_wants_before() {
        let ime = bitg_ime_new();
        assert!(!ime.is_null());
        unsafe {
            bitg_ime_focus_in(ime);
            assert_eq!(bitg_ime_wants_before(ime, BITG_KEY_CHAR, 'c' as u32), 1);
            let b = CString::new("").unwrap();
            bitg_ime_key(ime, BITG_KEY_CHAR, 'c' as u32, b.as_ptr());
            // so'z o'rtasi: matn kerak emas; so'z oxiri: kerak
            assert_eq!(bitg_ime_wants_before(ime, BITG_KEY_CHAR, 'h' as u32), 0);
            assert_eq!(bitg_ime_wants_before(ime, BITG_KEY_SPACE, ' ' as u32), 1);
            assert_eq!(bitg_ime_wants_before(ime, BITG_KEY_NAVIGATE, 0), 0);
            assert_eq!(
                bitg_ime_wants_before(ptr::null_mut(), BITG_KEY_CHAR, 'c' as u32),
                0
            );
            bitg_ime_free(ime);
        }
    }

    #[test]
    fn abi_round_trip() {
        let ime = bitg_ime_new();
        assert!(!ime.is_null());
        unsafe {
            bitg_ime_focus_in(ime);
            let mut before = String::new();
            for c in "choy".chars() {
                let b = CString::new(before.clone()).unwrap();
                let o = &*bitg_ime_key(ime, BITG_KEY_CHAR, c as u32, b.as_ptr());
                assert_eq!(o.consumed, 0);
                before.push(c);
            }
            let b = CString::new(before).unwrap();
            let o = &*bitg_ime_key(ime, BITG_KEY_SPACE, ' ' as u32, b.as_ptr());
            assert_eq!(
                (o.consumed, o.delete_before, s(o.commit)),
                (1, 4, "çoy ".to_string())
            );
            // preedit rejimi: matn berilmaydi
            for c in "shahar".chars() {
                let o = &*bitg_ime_key(ime, BITG_KEY_CHAR, c as u32, ptr::null());
                assert_eq!((o.consumed, o.preedit_changed), (1, 1));
            }
            assert_eq!(s(bitg_ime_reset(ime)), "shahar");
            assert!(bitg_ime_reset(ime).is_null());
            let o = &*bitg_ime_key(ptr::null_mut(), BITG_KEY_CHAR, 'a' as u32, ptr::null());
            assert_eq!(o.consumed, 0);
            bitg_ime_free(ime);
        }
    }
}
