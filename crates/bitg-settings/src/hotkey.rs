// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Belgilangan matnni o'girish tugmasi (standart: Ctrl+Shift+Y). Tray ilovasi tugmani `RegisterHotKey` bilan
//! ushlaydi; bosilganda: clipboard saqlanadi → Ctrl+C → matn o'giriladi → clipboard'ga yoziladi → Ctrl+V →
//! qisqa kutishdan keyin asl clipboard qaytariladi. Hech narsa belgilanmagan bo'lsa (Ctrl+C clipboard'ni
//! o'zgartirmadi) — hech narsa qilinmaydi.
//!
//! Bu yerdagi sof qismlar (birikmani o'qish, format tanlash, o'girish) platformaga bog'liq emas — testlanadi.
//! Windows qismi (`win`) — alohida oqim: o'z xabar navbati, sozlamalar o'zgarsa tugmani qayta ro'yxatdan o'tkazadi.

use bitg_tsf::config::Config;

/// `RegisterHotKey` modifikatorlari.
pub const MOD_ALT: u32 = 0x1;
pub const MOD_CONTROL: u32 = 0x2;
pub const MOD_SHIFT: u32 = 0x4;
pub const MOD_WIN: u32 = 0x8;

/// Tugmalar birikmasi: modifikatorlar + virtual tugma kodi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Combo {
    pub mods: u32,
    pub vk: u16,
}

/// Nomli tugmalar (harf, raqam va F1–F24 dan tashqari).
const NAMED: &[(&str, u16, &str)] = &[
    ("space", 0x20, "Space"),
    ("insert", 0x2D, "Insert"),
    ("pause", 0x13, "Pause"),
    ("scrolllock", 0x91, "ScrollLock"),
];

fn key_of(name: &str) -> Option<u16> {
    let b = name.as_bytes();
    if b.len() == 1 && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit()) {
        return Some(u16::from(b[0].to_ascii_uppercase()));
    }
    if let Some(n) = name.strip_prefix('f')
        && let Ok(n) = n.parse::<u16>()
        && (1..=24).contains(&n)
    {
        return Some(0x70 + n - 1); // VK_F1
    }
    NAMED.iter().find(|k| k.0 == name).map(|k| k.1)
}

fn key_label(vk: u16) -> String {
    match vk {
        0x30..=0x39 | 0x41..=0x5A => char::from(vk as u8).to_string(),
        0x70..=0x87 => format!("F{}", vk - 0x70 + 1),
        _ => NAMED
            .iter()
            .find(|k| k.1 == vk)
            .map_or_else(|| format!("0x{vk:02X}"), |k| k.2.to_string()),
    }
}

fn is_fkey(vk: u16) -> bool {
    (0x70..=0x87).contains(&vk)
}

/// `ctrl+shift+y` ko'rinishidagi matn. Talablar: bitta asosiy tugma; Ctrl, Alt yoki Win dan kamida biri
/// (faqat Shift + harf oddiy yozishni buzadi) — F tugmalar bundan mustasno.
pub fn parse(s: &str) -> Option<Combo> {
    let s = bitg_tsf::config::keys_value(s);
    let mut mods = 0;
    let mut vk = None;
    for part in s.split('+') {
        let m = match part {
            "ctrl" | "control" => MOD_CONTROL,
            "shift" => MOD_SHIFT,
            "alt" => MOD_ALT,
            "win" => MOD_WIN,
            _ => {
                if vk.replace(key_of(part)?).is_some() {
                    return None; // ikkita asosiy tugma
                }
                continue;
            }
        };
        if mods & m != 0 {
            return None;
        }
        mods |= m;
    }
    let vk = vk?;
    if mods & (MOD_CONTROL | MOD_ALT | MOD_WIN) == 0 && !is_fkey(vk) {
        return None;
    }
    Some(Combo { mods, vk })
}

impl Combo {
    /// Oynada ko'rsatish uchun: `Ctrl+Shift+Y`.
    pub fn label(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        for (m, n) in [
            (MOD_CONTROL, "Ctrl"),
            (MOD_ALT, "Alt"),
            (MOD_SHIFT, "Shift"),
            (MOD_WIN, "Win"),
        ] {
            if self.mods & m != 0 {
                parts.push(n.into());
            }
        }
        parts.push(key_label(self.vk));
        parts.join("+")
    }
}

/// O'girish yo'nalishi: `New` — eski lotin/kirill → yangi alifbo; `Old` — yangi → eski (rasmiy).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    New,
    Old,
}

/// Belgilangan matnni o'giradi (foydalanuvchi sozlamalari: istisnolar, inglizcha so'zlar, tutuq...).
/// Bo'sh yoki o'zgarmagan matn — `None`: clipboard qaytariladi, hech narsa joylanmaydi.
pub fn transform(text: &str, dir: Direction, c: &Config) -> Option<String> {
    if text.trim().is_empty() {
        return None;
    }
    let out = match dir {
        Direction::New => bitg_core::convert(text, &c.opts),
        Direction::Old => bitg_core::to_old(text, bitg_core::OldStyle::Official),
    };
    (out != text).then_some(out)
}

/// `GetClipboardData` HGLOBAL bermaydigan formatlar (GDI dastagi, metafayl, oyna chizadigan) — nusxalanmaydi.
/// `CF_TEXT`/`CF_OEMTEXT` — `CF_UNICODETEXT` bo'lsa Windows ularni o'zi yasaydi (nusxasi ziddiyat beradi).
pub fn keep_format(fmt: u32, has_unicode: bool) -> bool {
    const CF_TEXT: u32 = 1;
    const CF_BITMAP: u32 = 2;
    const CF_METAFILEPICT: u32 = 3;
    const CF_OEMTEXT: u32 = 7;
    const CF_PALETTE: u32 = 9;
    const CF_ENHMETAFILE: u32 = 14;
    match fmt {
        0 | CF_BITMAP | CF_METAFILEPICT | CF_PALETTE | CF_ENHMETAFILE => false,
        CF_TEXT | CF_OEMTEXT => !has_unicode,
        0x80..=0x8F => false,   // CF_OWNERDISPLAY, CF_DSP*
        0x300..=0x3FF => false, // CF_GDIOBJFIRST..LAST
        _ => true,
    }
}

/// Joylashdan keyin asl clipboard qaytariladimi: oraliqda boshqa kimdir (foydalanuvchi) yozmagan bo'lsa.
pub fn should_restore(after_our_write: u32, now: u32) -> bool {
    after_our_write == now
}

/// VS Code (va Monaco) bo'sh belgida Ctrl+C bilan butun qatorni nusxalaydi va buni o'z formatida belgilaydi.
/// Bunday "belgi"ni o'girib joylasak, qator ikki marta chiqadi — to'xtaymiz.
pub fn copied_without_selection(editor_data: &str) -> bool {
    editor_data
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .contains("\"isFromEmptySelection\":true")
}

/// Oyna uchun holat: `{"on": .., "label": "Ctrl+Shift+Y", "ok": ..}` (`ok` — ro'yxatdan o'tdi).
pub fn status_json(c: &Config, registered: Option<bool>) -> serde_json::Value {
    let combo = parse(&c.hotkey_keys);
    serde_json::json!({
        "on": c.enabled && c.hotkey,
        "label": combo.map(|k| k.label()),
        "valid": combo.is_some(),
        "ok": registered.unwrap_or(false),
    })
}

#[cfg(windows)]
pub use win::{registered, start};

#[cfg(windows)]
mod win {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU8, Ordering};
    use std::time::{Duration, Instant};

    use bitg_tsf::config::{self, Config};
    use windows::Win32::Foundation::{CloseHandle, GlobalFree, HANDLE, HGLOBAL, HWND};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
        GetClipboardSequenceNumber, IsClipboardFormatAvailable, OpenClipboard,
        RegisterClipboardFormatW, SetClipboardData,
    };
    use windows::Win32::System::Memory::{
        GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, HOT_KEY_MODIFIERS, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS,
        KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC, MOD_NOREPEAT,
        MapVirtualKeyW, RegisterHotKey, SendInput, UnregisterHotKey, VIRTUAL_KEY, VK_CONTROL,
        VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId, HWND_MESSAGE,
        MSG, SetTimer, WINDOW_EX_STYLE, WINDOW_STYLE, WM_HOTKEY, WM_TIMER,
    };
    use windows::core::{PWSTR, w};

    use super::{Combo, Direction};

    const CF_UNICODETEXT: u32 = 13;
    const ID_NEW: i32 = 1;
    const ID_OLD: i32 = 2;
    /// Ctrl+C dan keyin clipboard o'zgarishini kutish.
    const COPY_TIMEOUT: Duration = Duration::from_millis(300);
    /// Ctrl+V dan keyin dastur clipboard'ni o'qib ulgurishi uchun.
    const PASTE_DELAY: Duration = Duration::from_millis(400);
    /// Bitta formatning eng katta hajmi (undan kattasi saqlanmaydi).
    const MAX_FORMAT: usize = 64 << 20;

    /// 0 — hali urinilmagan/o'chiq, 1 — ro'yxatdan o'tdi, 2 — band (boshqa dastur ushlagan) yoki noto'g'ri.
    static REGISTERED: AtomicU8 = AtomicU8::new(0);
    static LOG: Mutex<()> = Mutex::new(());

    /// Asosiy tugma holati oyna uchun: `None` — o'chiq.
    pub fn registered() -> Option<bool> {
        match REGISTERED.load(Ordering::Relaxed) {
            1 => Some(true),
            2 => Some(false),
            _ => None,
        }
    }

    /// `%LOCALAPPDATA%\bitg\hotkey.log` — faqat hodisalar (matn yozilmaydi).
    fn log(msg: &str) {
        use std::io::Write;
        let _g = LOG.lock();
        let Some(dir) =
            std::env::var_os("LOCALAPPDATA").map(|d| std::path::PathBuf::from(d).join("bitg"))
        else {
            return;
        };
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("hotkey.log");
        if std::fs::metadata(&path).is_ok_and(|m| m.len() > 256 << 10) {
            let _ = std::fs::remove_file(&path);
        }
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(f, "{ts} {msg}");
        }
    }

    #[derive(Default, PartialEq)]
    struct Wanted {
        new: Option<Combo>,
        old: Option<Combo>,
    }

    fn wanted(c: &Config) -> (Wanted, bool) {
        if !(c.enabled && c.hotkey) {
            return (Wanted::default(), true);
        }
        let new = super::parse(&c.hotkey_keys);
        let old = if c.hotkey_old_keys.is_empty() {
            None
        } else {
            super::parse(&c.hotkey_old_keys)
        };
        if new.is_none() {
            log(&format!("hotkey: invalid hotkey_keys={}", c.hotkey_keys));
        }
        if old.is_none() && !c.hotkey_old_keys.is_empty() {
            log(&format!(
                "hotkey: invalid hotkey_old_keys={}",
                c.hotkey_old_keys
            ));
        }
        (Wanted { new, old }, new.is_some())
    }

    fn register(id: i32, combo: Option<Combo>) -> bool {
        unsafe {
            let _ = UnregisterHotKey(None, id);
        }
        let Some(k) = combo else { return true };
        let mods = HOT_KEY_MODIFIERS(k.mods) | MOD_NOREPEAT;
        match unsafe { RegisterHotKey(None, id, mods, u32::from(k.vk)) } {
            Ok(()) => {
                log(&format!("hotkey: registered {}", k.label()));
                true
            }
            Err(e) => {
                log(&format!("hotkey: register {} failed: {e}", k.label()));
                false
            }
        }
    }

    /// Alohida oqim: tugmalarni ro'yxatdan o'tkazadi, sozlamalar fayli o'zgarsa (sekundiga bir tekshiruv)
    /// qayta o'tkazadi va har o'zgarishda `on_change` ni chaqiradi (oynadagi holatni yangilash uchun).
    pub fn start(on_change: impl Fn() + Send + 'static) {
        std::thread::spawn(move || {
            // Clipboard egasi bo'lish uchun oyna kerak (NULL egasi bilan SetClipboardData ishlamaydi).
            let hwnd = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("STATIC"),
                    w!("bitg-hotkey"),
                    WINDOW_STYLE(0),
                    0,
                    0,
                    0,
                    0,
                    Some(HWND_MESSAGE),
                    None,
                    None,
                    None,
                )
            };
            let hwnd = match hwnd {
                Ok(h) => h,
                Err(e) => {
                    log(&format!("hotkey: message window failed: {e}"));
                    return;
                }
            };
            let mut current = Wanted::default();
            let mut stamp = None;
            unsafe { SetTimer(None, 0, 1000, None) };
            let mut apply = |current: &mut Wanted| {
                let s = config::stamp();
                if stamp.as_ref() == Some(&s) {
                    return;
                }
                stamp = Some(s);
                let (w, valid) = wanted(&config::load());
                if w != *current {
                    let ok_new = register(ID_NEW, w.new);
                    register(ID_OLD, w.old);
                    let state = match (w.new.is_some(), ok_new && valid) {
                        (false, true) => 0,
                        (true, true) => 1,
                        _ => 2,
                    };
                    REGISTERED.store(state, Ordering::Relaxed);
                    *current = w;
                    on_change();
                }
            };
            apply(&mut current);
            let mut msg = MSG::default();
            while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
                match msg.message {
                    WM_HOTKEY => {
                        let (dir, combo) = if msg.wParam.0 as i32 == ID_OLD {
                            (Direction::Old, current.old)
                        } else {
                            (Direction::New, current.new)
                        };
                        if let Some(combo) = combo {
                            convert_selection(hwnd, dir, combo);
                        }
                    }
                    WM_TIMER => apply(&mut current),
                    _ => {}
                }
            }
        });
    }

    /// Fokusdagi oyna dasturining fayl nomi (`telegram.exe`).
    fn foreground_exe() -> Option<String> {
        unsafe {
            let w = GetForegroundWindow();
            let mut pid = 0u32;
            GetWindowThreadProcessId(w, Some(&mut pid));
            if pid == 0 {
                return None;
            }
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let r = QueryFullProcessImageNameW(
                h,
                PROCESS_NAME_WIN32,
                PWSTR(buf.as_mut_ptr()),
                &mut len,
            );
            let _ = CloseHandle(h);
            r.ok()?;
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            path.rsplit(['\\', '/']).next().map(str::to_lowercase)
        }
    }

    fn key(vk: VIRTUAL_KEY, up: bool) -> INPUT {
        let ext = matches!(vk, VK_RCONTROL | VK_RMENU | VK_LWIN | VK_RWIN);
        let mut flags = KEYBD_EVENT_FLAGS(0);
        if up {
            flags |= KEYEVENTF_KEYUP;
        }
        if ext {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: unsafe { MapVirtualKeyW(u32::from(vk.0), MAPVK_VK_TO_VSC) } as u16,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn send(inputs: &[INPUT]) {
        unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
    }

    fn down(vk: VIRTUAL_KEY) -> bool {
        unsafe { GetAsyncKeyState(i32::from(vk.0)) as u16 & 0x8000 != 0 }
    }

    /// Foydalanuvchi hali ushlab turgan tugmalar: asosiy tugma qo'yilishini kutamiz (aks holda u Ctrl bilan
    /// takrorlanib, masalan Ctrl+Y bo'lib ketadi), modifikatorlarni esa sun'iy qo'yib yuboramiz — Ctrl+C
    /// Ctrl+Shift+C bo'lib qolmasin.
    fn release_held(combo: Combo) {
        let t = Instant::now();
        while down(VIRTUAL_KEY(combo.vk)) && t.elapsed() < Duration::from_millis(600) {
            std::thread::sleep(Duration::from_millis(10));
        }
        let mods = [
            VK_LSHIFT,
            VK_RSHIFT,
            VK_LCONTROL,
            VK_RCONTROL,
            VK_LMENU,
            VK_RMENU,
            VK_LWIN,
            VK_RWIN,
        ];
        let held: Vec<VIRTUAL_KEY> = mods.into_iter().filter(|&vk| down(vk)).collect();
        if held.is_empty() {
            return;
        }
        let mut inputs = Vec::new();
        if held
            .iter()
            .any(|&vk| matches!(vk, VK_LMENU | VK_RMENU | VK_LWIN | VK_RWIN))
        {
            // Alt/Win yolg'iz qo'yilsa menyu yoki Start ochiladi: oraga tayinlanmagan tugma (0xE8).
            inputs.push(key(VIRTUAL_KEY(0xE8), false));
            inputs.push(key(VIRTUAL_KEY(0xE8), true));
        }
        inputs.extend(held.iter().map(|&vk| key(vk, true)));
        send(&inputs);
        std::thread::sleep(Duration::from_millis(20));
    }

    fn ctrl(letter: u8) {
        let l = VIRTUAL_KEY(u16::from(letter));
        send(&[
            key(VK_CONTROL, false),
            key(l, false),
            key(l, true),
            key(VK_CONTROL, true),
        ]);
    }

    /// Clipboard'ni ochish: boshqa dastur hali yozayotgan bo'lsa — biroz kutib qayta urinadi.
    struct Open;

    impl Open {
        fn new(hwnd: HWND) -> Option<Open> {
            for _ in 0..20 {
                if unsafe { OpenClipboard(Some(hwnd)) }.is_ok() {
                    return Some(Open);
                }
                std::thread::sleep(Duration::from_millis(15));
            }
            log("hotkey: OpenClipboard failed");
            None
        }
    }

    impl Drop for Open {
        fn drop(&mut self) {
            let _ = unsafe { CloseClipboard() };
        }
    }

    fn global_bytes(h: HANDLE) -> Option<Vec<u8>> {
        let g = HGLOBAL(h.0);
        unsafe {
            let size = GlobalSize(g);
            if size == 0 || size > MAX_FORMAT {
                return None;
            }
            let p = GlobalLock(g) as *const u8;
            if p.is_null() {
                return None;
            }
            let v = std::slice::from_raw_parts(p, size).to_vec();
            let _ = GlobalUnlock(g);
            Some(v)
        }
    }

    fn put(fmt: u32, bytes: &[u8]) -> bool {
        unsafe {
            let Ok(g) = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1)) else {
                return false;
            };
            let p = GlobalLock(g) as *mut u8;
            if p.is_null() {
                let _ = GlobalFree(Some(g));
                return false;
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
            let _ = GlobalUnlock(g);
            if SetClipboardData(fmt, Some(HANDLE(g.0))).is_err() {
                let _ = GlobalFree(Some(g));
                return false;
            }
            true
        }
    }

    /// Asl clipboard: (format, baytlar) ro'yxati.
    fn save(hwnd: HWND) -> Option<Vec<(u32, Vec<u8>)>> {
        let _o = Open::new(hwnd)?;
        let has_unicode = unsafe { IsClipboardFormatAvailable(CF_UNICODETEXT) }.is_ok();
        let mut out = Vec::new();
        let mut fmt = 0;
        loop {
            fmt = unsafe { EnumClipboardFormats(fmt) };
            if fmt == 0 {
                break;
            }
            if !super::keep_format(fmt, has_unicode) {
                continue;
            }
            if let Ok(h) = unsafe { GetClipboardData(fmt) }
                && let Some(b) = global_bytes(h)
            {
                out.push((fmt, b));
            }
        }
        Some(out)
    }

    fn restore(hwnd: HWND, saved: &[(u32, Vec<u8>)]) {
        let Some(_o) = Open::new(hwnd) else { return };
        let _ = unsafe { EmptyClipboard() };
        for (fmt, b) in saved {
            put(*fmt, b);
        }
    }

    fn registered_format(name: &windows::core::PCWSTR) -> u32 {
        unsafe { RegisterClipboardFormatW(*name) }
    }

    /// Ctrl+C natijasi: matn (yoki `None` — matn yo'q yoki VS Code bo'sh belgidan qator nusxaladi).
    fn read_text(hwnd: HWND) -> Option<String> {
        let _o = Open::new(hwnd)?;
        for name in [w!("vscode-editor-data"), w!("VSCode Editor Data")] {
            let f = registered_format(&name);
            if let Ok(h) = unsafe { GetClipboardData(f) }
                && let Some(b) = global_bytes(h)
                && super::copied_without_selection(&String::from_utf8_lossy(&b))
            {
                log("hotkey: editor copied a whole line (no selection)");
                return None;
            }
        }
        let h = unsafe { GetClipboardData(CF_UNICODETEXT) }.ok()?;
        let b = global_bytes(h)?;
        let wide: Vec<u16> = b
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&c| u16::from_le_bytes(c))
            .take_while(|&c| c != 0)
            .collect();
        Some(String::from_utf16_lossy(&wide))
    }

    /// O'girilgan matnni yozadi; Win+V tarixiga va clipboard kuzatuvchilarga tushmasin deb belgilanadi.
    fn write_text(hwnd: HWND, text: &str) -> bool {
        let Some(_o) = Open::new(hwnd) else {
            return false;
        };
        if unsafe { EmptyClipboard() }.is_err() {
            return false;
        }
        let mut b: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
        b.extend([0, 0]);
        if !put(CF_UNICODETEXT, &b) {
            return false;
        }
        put(
            registered_format(&w!("ExcludeClipboardContentFromMonitorProcessing")),
            &[0],
        );
        put(
            registered_format(&w!("CanIncludeInClipboardHistory")),
            &0u32.to_le_bytes(),
        );
        true
    }

    fn convert_selection(hwnd: HWND, dir: Direction, combo: Combo) {
        let c = config::load();
        if !(c.enabled && c.hotkey) {
            return;
        }
        // Terminallar: Ctrl+C u yerda jarayonni to'xtatadi.
        if let Some(exe) = foreground_exe()
            && c.app_disabled(&exe)
        {
            log(&format!("hotkey: skipped, disabled app {exe}"));
            return;
        }
        release_held(combo);
        let Some(saved) = save(hwnd) else { return };
        let before = unsafe { GetClipboardSequenceNumber() };
        ctrl(b'C');
        let t = Instant::now();
        while unsafe { GetClipboardSequenceNumber() } == before {
            if t.elapsed() > COPY_TIMEOUT {
                log("hotkey: nothing selected");
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        // Dastur bir nechta formatni ketma-ket yozishi mumkin.
        std::thread::sleep(Duration::from_millis(30));
        let out = read_text(hwnd).and_then(|t| super::transform(&t, dir, &c));
        let Some(out) = out else {
            log("hotkey: nothing to convert");
            restore(hwnd, &saved);
            return;
        };
        if !write_text(hwnd, &out) {
            log("hotkey: clipboard write failed");
            restore(hwnd, &saved);
            return;
        }
        let ours = unsafe { GetClipboardSequenceNumber() };
        ctrl(b'V');
        std::thread::sleep(PASTE_DELAY);
        if super::should_restore(ours, unsafe { GetClipboardSequenceNumber() }) {
            restore(hwnd, &saved);
        } else {
            log("hotkey: clipboard changed meanwhile, not restored");
        }
        log(&format!(
            "hotkey: converted {} chars ({:?})",
            out.chars().count(),
            dir
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn parses_combos() {
        let k = parse("ctrl+shift+y").unwrap();
        assert_eq!(
            k,
            Combo {
                mods: MOD_CONTROL | MOD_SHIFT,
                vk: 0x59
            }
        );
        assert_eq!(k.label(), "Ctrl+Shift+Y");
        assert_eq!(parse(" Shift + Ctrl + Y "), Some(k));
        assert_eq!(parse("control+alt+1").unwrap().label(), "Ctrl+Alt+1");
        assert_eq!(parse("ctrl+shift+f8").unwrap().vk, 0x77);
        assert_eq!(parse("f9").unwrap().label(), "F9");
        assert_eq!(parse("win+shift+space").unwrap().label(), "Shift+Win+Space");
    }

    #[test]
    fn rejects_bad_combos() {
        for s in [
            "",
            "ctrl",
            "ctrl+shift",
            "shift+y",
            "y",
            "ctrl+y+u",
            "ctrl+ctrl+y",
            "ctrl+f25",
            "ctrl+f0",
            "ctrl+enter",
            "ctrl++y",
            "ctrl+.",
            "ctrl+fx",
        ] {
            assert_eq!(parse(s), None, "{s}");
        }
        assert_eq!(parse("ctrl+f").unwrap().label(), "Ctrl+F");
    }

    #[test]
    fn transform_uses_user_options() {
        let c = bitg_tsf::config::parse("", "");
        assert_eq!(
            transform("shahar va choy", Direction::New, &c).as_deref(),
            Some("şahar va çoy")
        );
        assert_eq!(transform("check", Direction::New, &c), None);
        assert_eq!(transform("  \r\n", Direction::New, &c), None);
        let ex = bitg_tsf::config::parse("", "choy\n");
        assert_eq!(
            transform("choy shahar", Direction::New, &ex).as_deref(),
            Some("choy şahar")
        );
        let ru = bitg_tsf::config::parse("cyrillic=0", "");
        assert_eq!(transform("шаҳар", Direction::New, &ru), None);
        assert_eq!(
            transform("Özbekiston", Direction::Old, &c).as_deref(),
            Some("Oʻzbekiston")
        );
        // Qator oxirlari saqlanadi.
        assert_eq!(
            transform("shahar\r\nchoy", Direction::New, &c).as_deref(),
            Some("şahar\r\nçoy")
        );
    }

    #[test]
    fn clipboard_decisions() {
        assert!(keep_format(13, true));
        assert!(!keep_format(1, true));
        assert!(keep_format(1, false));
        assert!(!keep_format(7, true));
        for fmt in [0, 2, 3, 9, 14, 0x82, 0x300] {
            assert!(!keep_format(fmt, false), "{fmt}");
        }
        assert!(keep_format(8, true)); // CF_DIB
        assert!(keep_format(0xC123, true)); // ro'yxatdan o'tgan format (HTML Format va h.k.)
        assert!(should_restore(7, 7));
        assert!(!should_restore(7, 8));
        assert!(copied_without_selection(
            r#"{"version":1,"isFromEmptySelection": true,"mode":"rust"}"#
        ));
        assert!(!copied_without_selection(
            r#"{"isFromEmptySelection":false}"#
        ));
    }

    #[test]
    fn status_reports_label() {
        let c = bitg_tsf::config::parse("hotkey_keys=ctrl+shift+b", "");
        let v = status_json(&c, Some(true));
        assert_eq!(v["label"], "Ctrl+Shift+B");
        assert_eq!(
            (&v["on"], &v["ok"], &v["valid"]),
            (&json!(true), &json!(true), &json!(true))
        );
        let off = status_json(
            &bitg_tsf::config::parse("enabled=0\nhotkey_keys=x", ""),
            None,
        );
        assert_eq!(
            (&off["on"], &off["valid"], &off["ok"], &off["label"]),
            (&json!(false), &json!(false), &json!(false), &Value::Null)
        );
        let no_hotkey = status_json(&bitg_tsf::config::parse("hotkey=0", ""), Some(false));
        assert_eq!(no_hotkey["on"], false);
    }

    #[test]
    fn labels_for_all_key_kinds() {
        assert_eq!(parse("alt+f24").unwrap().label(), "Alt+F24");
        assert_eq!(parse("ctrl+pause").unwrap().label(), "Ctrl+Pause");
        assert_eq!(parse("ctrl+scrolllock").unwrap().vk, 0x91);
        assert_eq!(Combo { mods: 0, vk: 0x0D }.label(), "0x0D");
    }
}
