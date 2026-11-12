// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Diagnostika: bitg profilini shu jarayon uchun yoqadi va TSF qaysi profilni faol deb bilishini chiqaradi.
//! `cargo run -p bitg-tsf --example activate` — so'ng `%LOCALAPPDATA%\bitg\tsf.log` ga qarang.

#[cfg(windows)]
fn main() -> windows_core::Result<()> {
    use bitg_tsf::{CLSID_BITG, PROFILE_BITG};
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::HKL;
    use windows::Win32::UI::TextServices::{
        CLSID_TF_InputProcessorProfiles, CLSID_TF_ThreadMgr, GUID_TFCAT_TIP_KEYBOARD,
        ITfInputProcessorProfileMgr, ITfThreadMgr, TF_INPUTPROCESSORPROFILE, TF_IPPMF_FORPROCESS,
        TF_PROFILETYPE_INPUTPROCESSOR,
    };

    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let tm: ITfThreadMgr = CoCreateInstance(&CLSID_TF_ThreadMgr, None, CLSCTX_INPROC_SERVER)?;
        let tid = tm.Activate()?;
        println!("thread mgr tid={tid}");
        let pm: ITfInputProcessorProfileMgr =
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;
        let r = pm.ActivateProfile(
            TF_PROFILETYPE_INPUTPROCESSOR,
            0x0443,
            &CLSID_BITG,
            &PROFILE_BITG,
            HKL::default(),
            TF_IPPMF_FORPROCESS,
        );
        println!("ActivateProfile: {r:?}");
        // TSF faollashtirishni xabarlar orqali yetkazadi — biroz xabar siklini aylantiramiz.
        {
            use windows::Win32::UI::WindowsAndMessaging::{
                DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
            };
            let t0 = std::time::Instant::now();
            let mut msg = MSG::default();
            while t0.elapsed().as_millis() < 1500 {
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        }
        let direct: windows_core::Result<windows_core::IUnknown> =
            CoCreateInstance(&CLSID_BITG, None, CLSCTX_INPROC_SERVER);
        println!("CoCreateInstance(bitg): {:?}", direct.map(|_| "ok"));
        let mut p = TF_INPUTPROCESSORPROFILE::default();
        match pm.GetActiveProfile(&GUID_TFCAT_TIP_KEYBOARD, &mut p) {
            Ok(()) => println!(
                "active: lang={:#x} clsid={:?} profile={:?} hkl={:?}",
                p.langid, p.clsid, p.guidProfile, p.hkl
            ),
            Err(e) => println!("GetActiveProfile: {e:?}"),
        }
        // TSF o'zbek (0x443) uchun nimalarni ko'radi
        let en = pm.EnumProfiles(0x0443)?;
        let mut buf = [TF_INPUTPROCESSORPROFILE::default(); 8];
        let mut n = 0u32;
        en.Next(&mut buf, &mut n)?;
        for p in &buf[..n as usize] {
            println!(
                "profile: type={} lang={:#x} clsid={:?} guid={:?} hkl={:?} subst={:?} caps={:#x} flags={:#x}",
                p.dwProfileType,
                p.langid,
                p.clsid,
                p.guidProfile,
                p.hkl,
                p.hklSubstitute,
                p.dwCaps,
                p.dwFlags
            );
        }
        tm.Deactivate()?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn main() {}
