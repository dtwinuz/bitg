// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # bitg-tsf — Windows matn kiritish xizmati (TSF Text Input Processor)
//!
//! Har bir dastur ichida (DLL sifatida) ishlaydi. Probel bosilganda kursor oldidagi so'zni TSF orqali o'qiydi
//! va o'sha joyda almashtiradi — hook yo'q, SendInput yo'q, clipboard yo'q. Parol maydonlarida Windows
//! TSF'ni o'zi o'chiradi. Phase 0 prototipi: har bir dasturda o'qish/yozish ishladimi — log'ga yoziladi
//! (`%LOCALAPPDATA%\bitg\tsf.log`).
//!
//! O'rnatish (administrator): `regsvr32 bitg_tsf.dll`, so'ng Sozlamalar → Til → O'zbek (lotin) → klaviatura
//! qo'shish → "bitg". O'chirish: `regsvr32 /u bitg_tsf.dll`.

// LNK4104: COM eksportlari PRIVATE emas — regsvr32 uchun zararsiz.
#![allow(linker_messages)]

pub mod config;
pub mod keymap;
pub mod plan;
pub mod typed;

#[cfg(windows)]
mod log;
#[cfg(windows)]
mod register;
#[cfg(windows)]
mod service;

#[cfg(windows)]
pub use exports::*;

#[cfg(windows)]
mod exports {
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicIsize, Ordering};

    use windows::Win32::Foundation::{
        CLASS_E_CLASSNOTAVAILABLE, E_POINTER, E_UNEXPECTED, HINSTANCE, S_FALSE, S_OK,
    };
    use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl};
    use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
    use windows_core::{BOOL, GUID, HRESULT, IUnknown, Interface, Ref, implement};

    use crate::service::TextService;

    /// bitg IME COM klassi.
    pub const CLSID_BITG: GUID = GUID::from_u128(0xe37d4f70_f82a_435e_af8b_82e264391062);
    /// TSF profil identifikatori.
    pub const PROFILE_BITG: GUID = GUID::from_u128(0xbed48fc8_8690_4712_bbe0_1fd3284ce7b7);

    pub(crate) static DLL_INSTANCE: AtomicIsize = AtomicIsize::new(0);

    #[unsafe(no_mangle)]
    extern "system" fn DllMain(hinst: HINSTANCE, reason: u32, _: *mut c_void) -> BOOL {
        if reason == DLL_PROCESS_ATTACH {
            DLL_INSTANCE.store(hinst.0 as isize, Ordering::Relaxed);
        }
        true.into()
    }

    /// COM chegarasi: panik dasturni to'xtatmasin (`extern "system"` ichidagi panik jarayonni yopadi),
    /// o'rniga E_UNEXPECTED qaytadi va logga yoziladi.
    fn com_guard(what: &str, f: impl FnOnce() -> HRESULT) -> HRESULT {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|_| {
            crate::log::write(&format!("panic in {what}"));
            E_UNEXPECTED
        })
    }

    #[implement(IClassFactory)]
    struct Factory;

    impl IClassFactory_Impl for Factory_Impl {
        fn CreateInstance(
            &self,
            outer: Ref<'_, IUnknown>,
            iid: *const GUID,
            object: *mut *mut c_void,
        ) -> windows_core::Result<()> {
            if object.is_null() || iid.is_null() {
                return Err(E_POINTER.into());
            }
            unsafe { *object = std::ptr::null_mut() };
            if outer.is_some() {
                return Err(windows::Win32::Foundation::CLASS_E_NOAGGREGATION.into());
            }
            com_guard("create instance", || {
                crate::log::write("create instance");
                let unknown: IUnknown = TextService::new().into();
                unsafe { unknown.query(iid, object) }
            })
            .ok()
        }

        fn LockServer(&self, _lock: BOOL) -> windows_core::Result<()> {
            Ok(())
        }
    }

    /// # Safety
    /// COM chaqiradi: ko'rsatkichlar haqiqiy.
    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn DllGetClassObject(
        clsid: *const GUID,
        iid: *const GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        if clsid.is_null() || iid.is_null() || out.is_null() {
            return E_POINTER;
        }
        unsafe { *out = std::ptr::null_mut() };
        if unsafe { *clsid } != CLSID_BITG {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        com_guard("get class object", || {
            crate::log::write("get class object");
            let factory: IClassFactory = Factory.into();
            unsafe { factory.query(iid, out) }
        })
    }

    /// DLL doim yuklangan qoladi (oddiy va xavfsiz: TSF ob'ektlari har dasturda qisqa yashaydi).
    #[unsafe(no_mangle)]
    pub extern "system" fn DllCanUnloadNow() -> HRESULT {
        S_FALSE
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn DllRegisterServer() -> HRESULT {
        com_guard("register", || match crate::register::register() {
            Ok(()) => S_OK,
            Err(e) => e.code(),
        })
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn DllUnregisterServer() -> HRESULT {
        com_guard("unregister", || match crate::register::unregister() {
            Ok(()) => S_OK,
            Err(e) => e.code(),
        })
    }

    #[allow(dead_code)]
    fn _assert_interface() {
        let _ = IClassFactory::IID;
    }
}
