// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! C ABI (`bitg_core::ffi`) tashqi chaqiruvchi kabi: satr beriladi, natija o'qiladi va `bitg_free` bilan bo'shatiladi.

use std::ffi::{CStr, CString, c_char};

use bitg_core::ffi::*;

fn take(p: *mut c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(p) }.to_str().unwrap().to_owned();
    unsafe { bitg_free(p) };
    Some(s)
}

fn c(s: &str) -> CString {
    CString::new(s).unwrap()
}

#[test]
fn convert_with_flags() {
    let t = c("O'zbekiston shaharlari");
    assert_eq!(
        take(unsafe { bitg_convert(t.as_ptr(), 0) }).unwrap(),
        "Özbekiston şaharlari"
    );
    let t = c("ma'no");
    assert_eq!(
        take(unsafe { bitg_convert(t.as_ptr(), BITG_ASCII_TUTUQ) }).unwrap(),
        "ma'no"
    );
    let t = c("шаҳар");
    assert_eq!(
        take(unsafe { bitg_convert(t.as_ptr(), BITG_NO_CYRILLIC) }).unwrap(),
        "шаҳар"
    );
    let t = c("ozbek");
    assert_eq!(
        take(unsafe { bitg_convert(t.as_ptr(), BITG_NO_RESTORE) }).unwrap(),
        "ozbek"
    );
    let t = c("check");
    assert_eq!(
        take(unsafe { bitg_convert(t.as_ptr(), 0) }).unwrap(),
        "check"
    );
    assert_ne!(
        take(unsafe { bitg_convert(t.as_ptr(), BITG_NO_ENGLISH_GUARD) }).unwrap(),
        "check"
    );
}

#[test]
fn old_and_cyrillic() {
    let t = c("Özbekiston");
    assert_eq!(
        take(unsafe { bitg_to_old(t.as_ptr(), 1) }).unwrap(),
        "O'zbekiston"
    );
    assert_eq!(
        take(unsafe { bitg_to_old(t.as_ptr(), 0) }).unwrap(),
        "O\u{02BB}zbekiston"
    );
    let t = c("şahar");
    assert_eq!(
        take(unsafe { bitg_to_cyrillic(t.as_ptr()) }).unwrap(),
        "шаҳар"
    );
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
    let bad = [0xffu8, 0xfe, 0];
    assert!(unsafe { bitg_convert(bad.as_ptr().cast(), 0) }.is_null());
    assert!(unsafe { bitg_to_old(bad.as_ptr().cast(), 0) }.is_null());
    assert!(unsafe { bitg_to_cyrillic(bad.as_ptr().cast()) }.is_null());
}

#[test]
fn session_create_feed_free() {
    bitg_warm_up();
    let s = bitg_session_new();
    assert!(!s.is_null());
    let w = c("shahar");
    let mut delete = usize::MAX;
    assert_eq!(
        take(unsafe { bitg_session_complete_word(s, w.as_ptr(), 0, &mut delete) }).unwrap(),
        "şahar"
    );
    assert_eq!(delete, 6);

    let en = c("check");
    let mut untouched = 7;
    assert!(unsafe { bitg_session_complete_word(s, en.as_ptr(), 0, &mut untouched) }.is_null());
    assert_eq!(untouched, 7);
    assert!(
        take(unsafe { bitg_session_complete_word(s, w.as_ptr(), 0, std::ptr::null_mut()) })
            .is_some()
    );

    unsafe {
        bitg_session_reset(s);
        assert!(bitg_session_complete_word(s, std::ptr::null(), 0, &mut delete).is_null());
        assert!(
            bitg_session_complete_word(std::ptr::null_mut(), w.as_ptr(), 0, &mut delete).is_null()
        );
        bitg_session_free(s);
    }
}
