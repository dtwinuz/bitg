// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `regsvr32` uchun: COM klassi (HKCR\CLSID) va TSF profili + kategoriyalar.

use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, HMODULE, MAX_PATH};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Registry::{
    HKEY, HKEY_CLASSES_ROOT, HKEY_LOCAL_MACHINE, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;
use windows::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, CLSID_TF_InputProcessorProfiles, GUID_TFCAT_TIP_KEYBOARD,
    GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT, GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
    GUID_TFCAT_TIPCAP_UIELEMENTENABLED, ITfCategoryMgr, ITfInputProcessorProfileMgr,
};
use windows_core::{GUID, HSTRING, PCWSTR, Result};

use crate::exports::{CLSID_BITG, DLL_INSTANCE, PROFILE_BITG};

/// O'zbek (lotin, O'zbekiston): uz-Latn-UZ.
const LANGID_UZ_LATN: u16 = 0x0443;
const NAME: &str = "bitg";
/// "00000409" — US klaviatura tartibi.
const US_LAYOUT: isize = 0x0409_0409;

const CATEGORIES: [GUID; 4] = [
    GUID_TFCAT_TIP_KEYBOARD,
    GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT, // Store / UWP / WinUI dasturlari
    GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
    GUID_TFCAT_TIPCAP_UIELEMENTENABLED,
];

fn dll_path() -> String {
    let mut buf = [0u16; MAX_PATH as usize];
    let h = HMODULE(DLL_INSTANCE.load(std::sync::atomic::Ordering::Relaxed) as *mut _);
    let n = unsafe { GetModuleFileNameW(Some(h), &mut buf) } as usize;
    String::from_utf16_lossy(&buf[..n])
}

fn clsid_key() -> String {
    format!("CLSID\\{{{:?}}}", CLSID_BITG)
}

fn set_value(sub: &str, name: Option<&str>, value: &str) -> Result<()> {
    let mut key = HKEY::default();
    unsafe {
        RegCreateKeyExW(
            HKEY_CLASSES_ROOT,
            &HSTRING::from(sub),
            None,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut key,
            None,
        )
        .ok()?;
        let data: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 2);
        let name_h = name.map(HSTRING::from);
        let pname = name_h
            .as_ref()
            .map(|h| PCWSTR(h.as_ptr()))
            .unwrap_or(PCWSTR::null());
        let r = RegSetValueExW(key, pname, None, REG_SZ, Some(bytes));
        let _ = RegCloseKey(key);
        r.ok()
    }
}

pub fn register() -> Result<()> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    let path = dll_path();
    let key = clsid_key();
    set_value(&key, None, NAME)?;
    set_value(&format!("{key}\\InprocServer32"), None, &path)?;
    set_value(
        &format!("{key}\\InprocServer32"),
        Some("ThreadingModel"),
        "Apartment",
    )?;

    unsafe {
        let profiles: ITfInputProcessorProfileMgr =
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;
        let desc: Vec<u16> = NAME.encode_utf16().collect();
        let icon: Vec<u16> = path.encode_utf16().collect();
        profiles.RegisterProfile(
            &CLSID_BITG,
            LANGID_UZ_LATN,
            &PROFILE_BITG,
            &desc,
            &icon,
            0,
            // Harflar odatdagi US (QWERTY) tartibidan keladi; o'zbek tilining standart kirill tartibi emas.
            HKL(US_LAYOUT as *mut _),
            0,
            true,
            0,
        )?;
        let cats: ITfCategoryMgr =
            CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
        for c in CATEGORIES {
            cats.RegisterCategory(&CLSID_BITG, &c, &CLSID_BITG)?;
        }
    }
    crate::log::write("registered");
    Ok(())
}

pub fn unregister() -> Result<()> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if let Ok(profiles) = CoCreateInstance::<_, ITfInputProcessorProfileMgr>(
            &CLSID_TF_InputProcessorProfiles,
            None,
            CLSCTX_INPROC_SERVER,
        ) {
            let _ = profiles.UnregisterProfile(&CLSID_BITG, LANGID_UZ_LATN, &PROFILE_BITG, 0);
        }
        if let Ok(cats) =
            CoCreateInstance::<_, ITfCategoryMgr>(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)
        {
            for c in CATEGORIES {
                let _ = cats.UnregisterCategory(&CLSID_BITG, &c, &CLSID_BITG);
            }
        }
        let r = RegDeleteTreeW(HKEY_CLASSES_ROOT, &HSTRING::from(clsid_key()));
        if r.is_err() && r != ERROR_FILE_NOT_FOUND {
            r.ok()?;
        }
        // TSF profil va toifalarni o'chirgach bo'sh kalitlarni qoldiradi — bu kalit faqat bitg'niki.
        let tip = format!("SOFTWARE\\Microsoft\\CTF\\TIP\\{{{CLSID_BITG:?}}}");
        let _ = RegDeleteTreeW(HKEY_LOCAL_MACHINE, &HSTRING::from(tip));
    }
    crate::log::write("unregistered");
    Ok(())
}
