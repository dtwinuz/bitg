//! Brauzer ABI (sayt sinov maydoni, `web/`): JS kirishni `bitg_alloc` xotirasiga UTF-8 qilib yozadi,
//! `bitg_run` natijani JSON qilib ichki buferga qo'yadi va uzunligini qaytaradi, `bitg_out` — bufer manzili.
//! wasm-bindgen'siz: yadro bog'liqliksiz qoladi.

use std::cell::RefCell;

use crate::{OldStyle, Reason, ffi};

thread_local! {
    static OUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// `len` baytlik bufer ajratadi (JS kirish satrini shu yerga yozadi).
#[unsafe(no_mangle)]
pub extern "C" fn bitg_alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len.max(1));
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// `bitg_alloc` bilan olingan buferni qaytaradi.
///
/// # Safety
/// `p` va `len` — aynan `bitg_alloc(len)` qaytargan juftlik.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_dealloc(p: *mut u8, len: usize) {
    // SAFETY: shartnoma bo'yicha p shu sig'imli Vec'dan olingan.
    drop(unsafe { Vec::from_raw_parts(p, 0, len.max(1)) });
}

/// Natija buferining manzili (oxirgi `bitg_run` dan keyin).
#[unsafe(no_mangle)]
pub extern "C" fn bitg_out() -> *const u8 {
    OUT.with(|o| o.borrow().as_ptr())
}

/// `mode` 0 — yangi alifboga: `{"out":"…","toks":[["so'z","c"],["ochmaydi","c","öçmaydi"],…]}`
/// (c o'girildi, u o'zgarmadi, p himoyalangan, x istisno, e inglizcha; 3-element — ikki ma'noli so'zning
/// natijaga kirmagan o'qilishi, "tekshiring");
/// 1 — eski lotinga (rasmiy oʻ), 2 — eski lotinga (o'), 3 — kirillga: `{"out":"…"}`.
/// `flags` — `ffi::BITG_*`. Qaytadi: JSON uzunligi, UTF-8 bo'lmasa 0.
///
/// # Safety
/// `p` — kamida `len` bayt o'qiladigan bufer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitg_run(p: *const u8, len: usize, mode: u32, flags: u32) -> usize {
    // SAFETY: chaqiruvchi len bayt beradi.
    let bytes = unsafe { std::slice::from_raw_parts(p, len) };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return 0;
    };
    let mut json = String::with_capacity(text.len() * 2 + 32);
    json.push_str("{\"out\":");
    match mode {
        0 => {
            let (out, toks) = crate::convert_words(text, &ffi::options(flags));
            push_str(&mut json, &out);
            json.push_str(",\"toks\":[");
            for (i, w) in toks.iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                json.push('[');
                push_str(&mut json, &w.token);
                json.push_str(match w.reason {
                    Reason::Converted => ",\"c\"",
                    Reason::Unchanged => ",\"u\"",
                    Reason::Protected => ",\"p\"",
                    Reason::Exception => ",\"x\"",
                    Reason::English => ",\"e\"",
                });
                if let Some(alt) = &w.alt {
                    json.push(',');
                    push_str(&mut json, alt);
                }
                json.push(']');
            }
            json.push(']');
        }
        3 => push_str(&mut json, &crate::to_cyrillic(text)),
        _ => {
            let style = if mode == 2 {
                OldStyle::Ascii
            } else {
                OldStyle::Official
            };
            push_str(&mut json, &crate::to_old(text, style));
        }
    }
    json.push('}');
    let n = json.len();
    OUT.with(|o| *o.borrow_mut() = json.into_bytes());
    n
}

fn push_str(json: &mut String, s: &str) {
    json.push('"');
    for c in s.chars() {
        match c {
            '"' => json.push_str("\\\""),
            '\\' => json.push_str("\\\\"),
            '\n' => json.push_str("\\n"),
            '\r' => json.push_str("\\r"),
            '\t' => json.push_str("\\t"),
            c if (c as u32) < 0x20 => json.push_str(&format!("\\u{:04x}", c as u32)),
            c => json.push(c),
        }
    }
    json.push('"');
}
