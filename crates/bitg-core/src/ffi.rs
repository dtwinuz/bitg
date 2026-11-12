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
    match unsafe { input(text) } {
        Some(t) => output(crate::convert(t, &opts(flags))),
        None => std::ptr::null_mut(),
    }
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
    match unsafe { input(text) } {
        Some(t) => output(crate::to_old(t, style)),
        None => std::ptr::null_mut(),
    }
}

/// Lotin (yangi yoki eski yozuv) → o'zbek kirill. Xato bo'lsa NULL.
///
/// # Safety
/// `text` — NUL bilan tugagan UTF-8 satr yoki NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_to_cyrillic(text: *const c_char) -> *mut c_char {
    match unsafe { input(text) } {
        Some(t) => output(crate::to_cyrillic(t)),
        None => std::ptr::null_mut(),
    }
}

/// `bitg_*` qaytargan satrni bo'shatadi.
///
/// # Safety
/// `s` — shu kutubxona qaytargan satr yoki NULL; bir marta chaqiriladi.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_free(s: *mut c_char) {
    if !s.is_null() {
        drop(unsafe { CString::from_raw(s) });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bitg_session_new() -> *mut Session {
    Box::into_raw(Box::new(Session::new()))
}

/// # Safety
/// `s` — `bitg_session_new` qaytargan ko'rsatkich yoki NULL; bir marta chaqiriladi.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_session_free(s: *mut Session) {
    if !s.is_null() {
        drop(unsafe { Box::from_raw(s) });
    }
}

/// # Safety
/// `s` — tirik sessiya.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_session_reset(s: *mut Session) {
    if let Some(s) = unsafe { s.as_mut() } {
        s.reset();
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
    match s.complete_word(w, &opts(flags)) {
        Some(e) => {
            if let Some(d) = unsafe { delete.as_mut() } {
                *d = e.delete;
            }
            output(e.insert)
        }
        None => std::ptr::null_mut(),
    }
}

/// Lug'atlarni oldindan yuklaydi.
#[unsafe(no_mangle)]
pub extern "C" fn bitg_warm_up() {
    crate::lang::warm_up();
}
