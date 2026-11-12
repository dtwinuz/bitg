// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Windows bilan bog'liq ishlar: bitta nusxa, Windows bilan ishga tushish, klaviatura o'rnatilganini tekshirish.
//!
//! O'rnatuvchi (MSI) barcha foydalanuvchilar uchun `HKLM\...\Run` yozadi (`--autostart`); har foydalanuvchi uni
//! o'zi o'chiradi: `HKCU\Software\bitg\Autostart = 0` (administrator huquqi kerak emas). MSI'siz (qo'lda
//! o'rnatish, ishlab chiqish) — eski yo'l: `HKCU\...\Run`.

use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, REG_DWORD, REG_SZ, RRF_RT_ANY,
    RRF_RT_REG_DWORD, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, EVENT_MODIFY_STATE, INFINITE, OpenEventW, SetEvent,
    WaitForSingleObject,
};
use windows::core::{BOOL, HSTRING, PCWSTR, s, w};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE: &str = "bitg";
/// Foydalanuvchi tanlovlari.
const USER_KEY: &str = r"Software\bitg";
/// O'rnatuvchi yozadigan qiymatlar (`ADDKEYBOARD` va h.k., administrator boshqaradi).
const MACHINE_KEY: &str = r"SOFTWARE\bitg";
const CLSID: &str = "{E37D4F70-F82A-435E-AF8B-82E264391062}";
const PROFILE: &str = "{BED48FC8-8690-4712-BBE0-1FD3284CE7B7}";
const MUTEX: &str = r"Local\bitg-app";
const SHOW_EVENT: &str = r"Local\bitg-app-show";

fn exists(root: HKEY, key: &str, value: Option<&str>) -> bool {
    let key = HSTRING::from(key);
    let value = value.map(HSTRING::from);
    let vp = value
        .as_ref()
        .map_or(PCWSTR::null(), |v| PCWSTR(v.as_ptr()));
    unsafe { RegGetValueW(root, &key, vp, RRF_RT_ANY, None, None, None) }.is_ok()
}

/// IME DLL ro'yxatdan o'tganmi (regsvr32).
pub fn ime_registered() -> bool {
    exists(
        HKEY_CLASSES_ROOT,
        &format!(r"CLSID\{CLSID}\InprocServer32"),
        None,
    )
}

/// bitg klaviaturasi foydalanuvchining tillar ro'yxatida bormi (O'zbek lotin → bitg).
pub fn keyboard_added() -> bool {
    exists(
        HKEY_CURRENT_USER,
        r"Control Panel\International\User Profile\uz-Latn",
        Some(&format!("0443:{CLSID}{PROFILE}")),
    )
}

fn dword(root: HKEY, key: &str, value: &str) -> Option<u32> {
    let mut v = 0u32;
    let mut size = 4u32;
    unsafe {
        RegGetValueW(
            root,
            &HSTRING::from(key),
            &HSTRING::from(value),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut v as *mut u32).cast()),
            Some(&mut size),
        )
    }
    .ok()
    .ok()?;
    Some(v)
}

/// O'rnatuvchi qiymati: MSI satr ("1") yozadi, GPO/administrator DWORD qo'yishi mumkin — ikkalasi ham.
fn machine_flag(value: &str) -> bool {
    if let Some(v) = dword(HKEY_LOCAL_MACHINE, MACHINE_KEY, value) {
        return v == 1;
    }
    let mut buf = [0u16; 8];
    let mut size = (buf.len() * 2) as u32;
    let ok = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            &HSTRING::from(MACHINE_KEY),
            &HSTRING::from(value),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut size),
        )
    }
    .is_ok();
    ok && buf[0] == u16::from(b'1') && buf[1] == 0
}

fn set_dword(root: HKEY, key: &str, value: &str, v: u32) -> windows::core::Result<()> {
    unsafe {
        RegSetKeyValueW(
            root,
            &HSTRING::from(key),
            &HSTRING::from(value),
            REG_DWORD.0,
            Some((&v as *const u32).cast()),
            4,
        )
    }
    .ok()
}

/// O'rnatuvchi barcha foydalanuvchilar uchun avtostart yozganmi.
fn machine_autostart() -> bool {
    exists(HKEY_LOCAL_MACHINE, RUN_KEY, Some(RUN_VALUE))
}

fn remove_user_run() -> windows::core::Result<()> {
    let r = unsafe {
        RegDeleteKeyValueW(
            HKEY_CURRENT_USER,
            &HSTRING::from(RUN_KEY),
            &HSTRING::from(RUN_VALUE),
        )
    };
    if r.is_ok() || !exists(HKEY_CURRENT_USER, RUN_KEY, Some(RUN_VALUE)) {
        Ok(())
    } else {
        r.ok()
    }
}

/// `--autostart` (HKLM Run) bilan ishga tushganda: shu foydalanuvchi o'chirib qo'ymaganmi.
pub fn autostart_allowed() -> bool {
    dword(HKEY_CURRENT_USER, USER_KEY, "Autostart") != Some(0)
}

pub fn autostart() -> bool {
    if machine_autostart() {
        autostart_allowed()
    } else {
        exists(HKEY_CURRENT_USER, RUN_KEY, Some(RUN_VALUE))
    }
}

pub fn set_autostart(on: bool) -> windows::core::Result<()> {
    if machine_autostart() {
        remove_user_run()?; // qo'lda o'rnatishdan qolgan ikkinchi yozuv
        return set_dword(HKEY_CURRENT_USER, USER_KEY, "Autostart", on as u32);
    }
    let key = HSTRING::from(RUN_KEY);
    let name = HSTRING::from(RUN_VALUE);
    if on {
        let exe = std::env::current_exe().map_err(|_| windows::core::Error::from_thread())?;
        let cmd = format!("\"{}\" --tray", exe.display());
        let wide: Vec<u16> = cmd.encode_utf16().chain(Some(0)).collect();
        unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                &key,
                &name,
                REG_SZ.0,
                Some(wide.as_ptr().cast()),
                (wide.len() * 2) as u32,
            )
        }
        .ok()
    } else {
        let r = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, &key, &name) };
        if r.is_ok() || !autostart() {
            Ok(())
        } else {
            r.ok()
        }
    }
}

/// bitg klaviaturasini joriy foydalanuvchining tillar ro'yxatiga qo'shadi (rasmiy `InstallLayoutOrTip`,
/// kerak bo'lsa O'zbek (lotin) tilini ham qo'shadi).
pub fn add_keyboard() -> bool {
    type InstallLayoutOrTip = unsafe extern "system" fn(PCWSTR, u32) -> BOOL;
    let Ok(lib) = (unsafe { LoadLibraryW(w!("input.dll")) }) else {
        return false;
    };
    let Some(p) = (unsafe { GetProcAddress(lib, s!("InstallLayoutOrTip")) }) else {
        return false;
    };
    // SAFETY: input.dll eksporti hujjatlangan imzoga ega: BOOL InstallLayoutOrTip(LPCWSTR, DWORD).
    let f: InstallLayoutOrTip = unsafe { std::mem::transmute(p) };
    let spec = HSTRING::from(format!("0443:{CLSID}{PROFILE}"));
    unsafe { f(PCWSTR(spec.as_ptr()), 0) }.as_bool() || keyboard_added()
}

/// Har foydalanuvchida bir marta: o'rnatuvchi ruxsat bergan bo'lsa (`ADDKEYBOARD=1`) klaviaturani qo'shadi.
/// Keyin foydalanuvchi uni o'chirsa — qaytadan qo'shilmaydi.
pub fn first_run_setup() {
    if machine_autostart() {
        let _ = remove_user_run();
    }
    if dword(HKEY_CURRENT_USER, USER_KEY, "KeyboardOffered").is_some() {
        return;
    }
    let _ = set_dword(HKEY_CURRENT_USER, USER_KEY, "KeyboardOffered", 1);
    if machine_flag("AddKeyboard") && ime_registered() && !keyboard_added() {
        add_keyboard();
    }
}

/// Birinchi nusxami? Yo'q bo'lsa — birinchisiga "oynani ko'rsat" signalini yuboradi.
/// Mutex jarayon tugaguncha ochiq turadi (HANDLE ataylab yopilmaydi).
pub fn first_instance(show_existing: bool) -> bool {
    let created = unsafe { CreateMutexW(None, true, &HSTRING::from(MUTEX)) };
    let already = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    if created.is_ok() && !already {
        return true;
    }
    if !show_existing {
        return false;
    }
    if let Ok(ev) = unsafe { OpenEventW(EVENT_MODIFY_STATE, false, &HSTRING::from(SHOW_EVENT)) } {
        let _ = unsafe { SetEvent(ev) };
    }
    false
}

/// Boshqa nusxa "ko'rsat" desa — `on_show` chaqiriladi (alohida oqimda kutadi).
pub fn listen_show(on_show: impl Fn() + Send + 'static) {
    let Ok(ev) = (unsafe { CreateEventW(None, false, false, &HSTRING::from(SHOW_EVENT)) }) else {
        return;
    };
    let ev = ev.0 as usize; // HANDLE Send emas — raqam sifatida o'tkazamiz
    std::thread::spawn(move || {
        let h = HANDLE(ev as *mut _);
        loop {
            if unsafe { WaitForSingleObject(h, INFINITE) }.0 != 0 {
                return;
            }
            on_show();
        }
    });
}
