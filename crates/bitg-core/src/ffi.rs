// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! C ABI: TSF IME (C++/Rust), tray ilova (C#/.NET), Linux (Fcitx5/IBus) va boshqa tillar uchun.
//! Barcha satrlar UTF-8. Qaytarilgan satrni faqat `bitg_free` bo'shatadi.

use std::ffi::{CStr, CString, c_char};

use crate::{OldStyle, Options, Session};

/// `flags` bitlari.
pub const BITG_NO_ENGLISH_GUARD: u32 = 1;
pub const BITG_NO_CYRILLIC: u32 = 2;
pub const BITG_ASCII_TUTUQ: u32 = 4;
/// Tushib qolgan apostrofni tiklamaslik ("ozbek" o'zgarmay qoladi).
pub const BITG_NO_RESTORE: u32 = 8;

#[cfg(target_arch = "wasm32")]
pub(crate) fn options(flags: u32) -> Options {
    opts(flags)
}

fn opts(flags: u32) -> Options {
    Options {
        english_guard: flags & BITG_NO_ENGLISH_GUARD == 0,
        cyrillic: flags & BITG_NO_CYRILLIC == 0,
        tutuq: if flags & BITG_ASCII_TUTUQ != 0 {
            '\''
        } else {
            crate::chars::DEFAULT_TUTUQ
        },
        exceptions: Vec::new(),
        restore_apostrophes: flags & BITG_NO_RESTORE == 0,
    }
}

unsafe fn input<'a>(s: *const c_char) -> Option<&'a str> {
    if s.is_null() {
        return None;
    }
    // SAFETY: chaqiruvchi NUL bilan tugagan UTF-8 satr beradi.
    unsafe { CStr::from_ptr(s) }.to_str().ok()
}

/// Panik C chegarasidan o'tmasin: `extern "C"` ichidagi panik butun dasturni (IME bo'lsa foydalanuvchi
/// yozayotgan ilovani) to'xtatadi. Panik bo'lsa `fallback` qaytadi.
fn guarded<T>(fallback: T, f: impl FnOnce() -> T) -> T {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(fallback)
}

fn output(s: String) -> *mut c_char {
    CString::new(s)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

/// Matnni yangi alifboga o'giradi. Xato bo'lsa NULL.
///
/// # Safety
/// `text` — NUL bilan tugagan UTF-8 satr yoki NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_convert(text: *const c_char, flags: u32) -> *mut c_char {
    guarded(std::ptr::null_mut(), || match unsafe { input(text) } {
        Some(t) => output(crate::convert(t, &opts(flags))),
        None => std::ptr::null_mut(),
    })
}

/// Yangi alifbodan eski lotinga. `ascii` != 0 — o' g' ' (aks holda rasmiy oʻ gʻ ʼ).
///
/// # Safety
/// `text` — NUL bilan tugagan UTF-8 satr yoki NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_to_old(text: *const c_char, ascii: u32) -> *mut c_char {
    let style = if ascii != 0 {
        OldStyle::Ascii
    } else {
        OldStyle::Official
    };
    guarded(std::ptr::null_mut(), || match unsafe { input(text) } {
        Some(t) => output(crate::to_old(t, style)),
        None => std::ptr::null_mut(),
    })
}

/// Lotin (yangi yoki eski yozuv) → o'zbek kirill. Xato bo'lsa NULL.
///
/// # Safety
/// `text` — NUL bilan tugagan UTF-8 satr yoki NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_to_cyrillic(text: *const c_char) -> *mut c_char {
    guarded(std::ptr::null_mut(), || match unsafe { input(text) } {
        Some(t) => output(crate::to_cyrillic(t)),
        None => std::ptr::null_mut(),
    })
}

/// `bitg_*` qaytargan satrni bo'shatadi.
///
/// # Safety
/// `s` — shu kutubxona qaytargan satr yoki NULL; bir marta chaqiriladi.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_free(s: *mut c_char) {
    if !s.is_null() {
        guarded((), || drop(unsafe { CString::from_raw(s) }));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bitg_session_new() -> *mut Session {
    guarded(std::ptr::null_mut(), || {
        Box::into_raw(Box::new(Session::new()))
    })
}

/// # Safety
/// `s` — `bitg_session_new` qaytargan ko'rsatkich yoki NULL; bir marta chaqiriladi.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_session_free(s: *mut Session) {
    if !s.is_null() {
        guarded((), || drop(unsafe { Box::from_raw(s) }));
    }
}

/// # Safety
/// `s` — tirik sessiya.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_session_reset(s: *mut Session) {
    if let Some(s) = unsafe { s.as_mut() } {
        guarded((), || s.reset());
    }
}

/// So'z tugadi. O'zgarish kerak bo'lsa: `*delete` ta belgi (Unicode scalar) o'chirilib, qaytgan satr yoziladi.
/// O'zgarish yo'q bo'lsa NULL.
///
/// # Safety
/// `s` — tirik sessiya, `word` — UTF-8 satr, `delete` — yozish mumkin bo'lgan ko'rsatkich.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_session_complete_word(
    s: *mut Session,
    word: *const c_char,
    flags: u32,
    delete: *mut usize,
) -> *mut c_char {
    let (Some(s), Some(w)) = (unsafe { s.as_mut() }, unsafe { input(word) }) else {
        return std::ptr::null_mut();
    };
    guarded(std::ptr::null_mut(), || {
        match s.complete_word(w, &opts(flags)) {
            Some(e) => {
                if let Some(d) = unsafe { delete.as_mut() } {
                    *d = e.delete;
                }
                output(e.insert)
            }
            None => std::ptr::null_mut(),
        }
    })
}

/// Lug'atlarni oldindan yuklaydi.
#[unsafe(no_mangle)]
pub extern "C" fn bitg_warm_up() {
    guarded((), crate::lang::warm_up);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// C chaqiruvchi kabi: satr beradi, natijani o'qib `bitg_free` bilan bo'shatadi.
    fn call(f: impl FnOnce(*const c_char) -> *mut c_char, text: &str) -> Option<String> {
        let c = CString::new(text).unwrap();
        let p = f(c.as_ptr());
        if p.is_null() {
            return None;
        }
        // SAFETY: p — shu kutubxona qaytargan satr, bir marta bo'shatiladi.
        let s = unsafe { CStr::from_ptr(p) }.to_str().unwrap().to_owned();
        unsafe { bitg_free(p) };
        Some(s)
    }

    #[test]
    fn convert_and_back() {
        bitg_warm_up();
        let new = call(
            |p| unsafe { bitg_convert(p, 0) },
            "O'zbekiston shaharlari: Toshkent",
        )
        .unwrap();
        assert_eq!(new, "Özbekiston şaharlari: Toşkent");
        assert_eq!(
            call(|p| unsafe { bitg_to_old(p, 1) }, &new).unwrap(),
            "O'zbekiston shaharlari: Toshkent"
        );
        assert_eq!(
            call(|p| unsafe { bitg_to_cyrillic(p) }, "Toşkent").unwrap(),
            "Тошкент"
        );
    }

    #[test]
    fn panic_does_not_cross_the_boundary() {
        assert_eq!(guarded(7, || -> i32 { panic!("sinov") }), 7);
        assert_eq!(guarded(7, || 1), 1);
    }

    #[test]
    fn flags_reach_options() {
        let o = opts(BITG_NO_ENGLISH_GUARD | BITG_NO_CYRILLIC | BITG_ASCII_TUTUQ | BITG_NO_RESTORE);
        assert!(!o.english_guard);
        assert!(!o.cyrillic);
        assert!(!o.restore_apostrophes);
        assert_eq!(o.tutuq, '\'');
        let d = opts(0);
        assert!(d.english_guard);
        assert!(d.cyrillic);
        assert!(d.restore_apostrophes);
        assert_eq!(d.tutuq, crate::chars::DEFAULT_TUTUQ);
    }

    #[test]
    fn null_and_invalid_input_return_null() {
        unsafe {
            assert!(bitg_convert(std::ptr::null(), 0).is_null());
            assert!(bitg_to_old(std::ptr::null(), 0).is_null());
            assert!(bitg_to_cyrillic(std::ptr::null()).is_null());
            bitg_free(std::ptr::null_mut());
            bitg_session_free(std::ptr::null_mut());
            bitg_session_reset(std::ptr::null_mut());
        }
        // UTF-8 emas
        let bad = [0xffu8, 0xfe, 0];
        assert!(unsafe { bitg_convert(bad.as_ptr().cast(), 0) }.is_null());
    }

    #[test]
    fn session_round_trip() {
        let s = bitg_session_new();
        let mut delete = usize::MAX;
        let w = CString::new("shahar").unwrap();
        let p = unsafe { bitg_session_complete_word(s, w.as_ptr(), 0, &mut delete) };
        assert!(!p.is_null());
        assert_eq!(unsafe { CStr::from_ptr(p) }.to_str().unwrap(), "şahar");
        assert_eq!(delete, "shahar".chars().count());
        unsafe { bitg_free(p) };

        // ingliz so'zi o'zgarmaydi -> NULL, `delete` ga tegilmaydi
        let mut untouched = 7;
        let en = CString::new("check").unwrap();
        assert!(unsafe { bitg_session_complete_word(s, en.as_ptr(), 0, &mut untouched) }.is_null());
        assert_eq!(untouched, 7);
        // `delete` NULL bo'lishi mumkin
        let p = unsafe { bitg_session_complete_word(s, w.as_ptr(), 0, std::ptr::null_mut()) };
        unsafe { bitg_free(p) };
        unsafe {
            bitg_session_reset(s);
            assert!(
                bitg_session_complete_word(std::ptr::null_mut(), w.as_ptr(), 0, &mut delete)
                    .is_null()
            );
            bitg_session_free(s);
        }
    }

    #[test]
    fn official_old_style_and_null_word() {
        assert_eq!(
            call(|p| unsafe { bitg_to_old(p, 0) }, "Özbekiston").unwrap(),
            "O\u{02BB}zbekiston"
        );
        let s = bitg_session_new();
        let mut delete = 3;
        unsafe {
            assert!(bitg_session_complete_word(s, std::ptr::null(), 0, &mut delete).is_null());
        }
        assert_eq!(delete, 3);
        assert_eq!(
            call(
                |p| unsafe { bitg_session_complete_word(s, p, 0, &mut delete) },
                "check"
            ),
            None
        );
        unsafe { bitg_session_free(s) };
    }
}
