// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! bitg ilovasi: tray belgisi + sozlamalar oynasi. Sozlamalar darhol `%APPDATA%\bitg` ga yoziladi; IME ularni
//! bir soniya ichida, dasturlarni qayta ochmasdan qabul qiladi. Namuna IME bilan bir xil yadroni ishlatadi.
//!
//! `bitg-settings.exe` — oynani ochadi; `--tray` — faqat tray (Windows bilan ishga tushganda).
//! Ikkinchi nusxa ochilmaydi: birinchisining oynasini ko'rsatadi.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg_attr(not(windows), allow(dead_code))]
mod feedback;
#[cfg_attr(not(windows), allow(dead_code))]
mod hotkey;
#[cfg_attr(not(windows), allow(dead_code))]
mod logic;
#[cfg(windows)]
mod net;
#[cfg(windows)]
mod shot;
#[cfg(windows)]
mod system;
#[cfg_attr(not(windows), allow(dead_code))]
mod updates;

#[cfg(not(windows))]
fn main() {
    eprintln!("bitg-settings hozircha faqat Windows uchun.");
}

#[cfg(windows)]
fn main() {
    // --tray: foydalanuvchining o'z Run yozuvi; --autostart: o'rnatuvchining (barcha foydalanuvchilar) yozuvi.
    let from_machine_run = std::env::args().any(|a| a == "--autostart");
    if from_machine_run && !system::autostart_allowed() {
        return; // shu foydalanuvchi Windows bilan ishga tushishni o'chirgan
    }
    let tray_only = from_machine_run || std::env::args().any(|a| a == "--tray");
    // Ikkinchi nusxa: oddiy ochilishda birinchisining oynasini ko'rsatadi, avtostartda jim chiqadi.
    if !system::first_instance(!tray_only) {
        return;
    }
    system::first_run_setup();
    if let Err(e) = app::run(tray_only) {
        eprintln!("bitg-settings: {e}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
mod app {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    use serde_json::{Value, json};
    use tao::dpi::LogicalSize;
    use tao::event::{Event, StartCause, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
    use tao::platform::windows::IconExtWindows;
    use tao::window::{Theme, Window, WindowBuilder};
    use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
    use wry::{WebContext, WebView, WebViewBuilder};

    use crate::{feedback, hotkey, logic, net, shot, system, updates};

    const PAGE: &str = include_str!("ui.html");
    const ICON_ON: u16 = 1;
    const ICON_OFF: u16 = 2;
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    /// Oynadan ochiladigan havolalar (boshqasi ochilmaydi).
    const LINKS: &[&str] = &[
        "https://bitg.uz",
        "https://github.com/dtwinuz/bitg",
        "https://github.com/dtwinuz/bitg/releases",
        FEEDBACK,
    ];
    /// Fikr sahifasi: versiya qo'shib ochiladi, GitHub shaklida o'zi yoziladi.
    const FEEDBACK: &str = "https://bitg.uz/fikr/";

    /// Topilgan yangi versiya (manifest tekshirilgan).
    static UPDATE: Mutex<Option<updates::Update>> = Mutex::new(None);
    /// Yuklab olish yoki xabar yuborish ketmoqda — ikkinchisi boshlanmaydi.
    static INSTALLING: AtomicBool = AtomicBool::new(false);
    static SENDING: AtomicBool = AtomicBool::new(false);
    /// Ilova shu foydalanuvchida avval ochilganmi (yangi o'rnatishda «Yangiliklar» chiqmaydi).
    static RAN_BEFORE: AtomicBool = AtomicBool::new(false);

    fn feedback_url() -> String {
        format!("{FEEDBACK}?v={VERSION}&os=windows")
    }

    fn open_url(u: &str) {
        let _ = std::process::Command::new("explorer").arg(u).spawn();
    }

    enum Ev {
        /// Oynaga JSON javob.
        Reply(String),
        Show,
        Hide,
        Menu(MenuEvent),
        Tray(TrayIconEvent),
        /// Sozlamalar oynadan saqlandi — tray holatini yangilash.
        Saved,
        /// Yangi versiya topildi — tray menyusida ko'rsatish.
        UpdateFound(String),
        Quit,
    }

    struct Tray {
        icon: TrayIcon,
        enabled: CheckMenuItem,
        open: MenuItem,
        feedback: MenuItem,
        update: MenuItem,
        quit: MenuItem,
    }

    fn tray_icon(on: bool) -> Option<tray_icon::Icon> {
        tray_icon::Icon::from_resource(if on { ICON_ON } else { ICON_OFF }, None).ok()
    }

    fn make_tray(on: bool) -> Result<Tray, Box<dyn std::error::Error>> {
        let enabled = CheckMenuItem::new("Yoqilgan", true, on, None);
        let open = MenuItem::new("bitg'ni ochish", true, None);
        let feedback = MenuItem::new("Xato yoki taklif yozish", true, None);
        let update = MenuItem::new("Yangilanishni tekshirish", true, None);
        let quit = MenuItem::new("Chiqish", true, None);
        let menu = Menu::new();
        menu.append_items(&[
            &enabled,
            &PredefinedMenuItem::separator(),
            &open,
            &feedback,
            &update,
            &PredefinedMenuItem::separator(),
            &quit,
        ])?;
        let mut b = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip(tooltip(on));
        if let Some(i) = tray_icon(on) {
            b = b.with_icon(i);
        }
        Ok(Tray {
            icon: b.build()?,
            enabled,
            open,
            feedback,
            update,
            quit,
        })
    }

    fn tooltip(on: bool) -> &'static str {
        if on {
            "bitg — yoqilgan"
        } else {
            "bitg — o'chiq"
        }
    }

    fn sync_tray(t: &Tray) {
        let on = bitg_tsf::config::load().enabled;
        t.enabled.set_checked(on);
        let _ = t.icon.set_icon(tray_icon(on));
        let _ = t.icon.set_tooltip(Some(tooltip(on)));
    }

    fn show(w: &Window) {
        w.set_visible(true);
        w.set_minimized(false);
        w.set_focus();
    }

    pub fn run(tray_only: bool) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoopBuilder::<Ev>::with_user_event().build();
        let proxy = event_loop.create_proxy();
        // WebView2 papkasi birinchi ochilishda yaratiladi: bor bo'lsa — bu yangilanish, yangi o'rnatish emas.
        let data = bitg_tsf::config::local_dir().map(|d| d.join("webview"));
        RAN_BEFORE.store(data.as_ref().is_some_and(|d| d.exists()), Ordering::Relaxed);

        {
            let p = proxy.clone();
            MenuEvent::set_event_handler(Some(move |e| {
                let _ = p.send_event(Ev::Menu(e));
            }));
            let p = proxy.clone();
            TrayIconEvent::set_event_handler(Some(move |e| {
                let _ = p.send_event(Ev::Tray(e));
            }));
            let p = proxy.clone();
            system::listen_show(move || {
                let _ = p.send_event(Ev::Show);
            });
            // Belgilangan matnni o'girish tugmasi: holati o'zgarsa oynaga xabar beradi.
            let p = proxy.clone();
            hotkey::start(move || reply(&p, status()));
            // Yangilanish: ishga tushgandan 30 s keyin, so'ng kuniga bir marta (sozlamada o'chirilsa — yo'q).
            let p = proxy.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(30));
                let mut last: Option<Instant> = None;
                loop {
                    let due = last.is_none_or(|t| t.elapsed() >= Duration::from_secs(24 * 3600));
                    if due && bitg_tsf::config::load().update_check {
                        check_update(&p, false);
                        last = Some(Instant::now());
                    }
                    std::thread::sleep(Duration::from_secs(3600));
                }
            });
        }

        let window = WindowBuilder::new()
            .with_title("bitg")
            .with_inner_size(LogicalSize::new(1120.0, 740.0))
            .with_min_inner_size(LogicalSize::new(440.0, 560.0))
            .with_visible(false)
            .with_window_icon(tao::window::Icon::from_resource(ICON_ON, None).ok())
            .build(&event_loop)?;
        let dark = window.theme() == Theme::Dark;

        // WebView2 ma'lumotlari: Program Files'ga yozib bo'lmaydi — foydalanuvchi papkasiga.
        let mut ctx = WebContext::new(data);
        let ipc_proxy = proxy.clone();
        let webview = WebViewBuilder::new_with_web_context(&mut ctx)
            .with_html(PAGE)
            .with_background_color(if dark {
                (20, 25, 34, 255)
            } else {
                (236, 238, 234, 255)
            })
            .with_devtools(cfg!(debug_assertions))
            .with_ipc_handler(move |req| handle(req.body(), &ipc_proxy))
            .build(&window)?;
        if !tray_only {
            show(&window);
        }

        let mut tray: Option<Tray> = None;
        event_loop.run(move |event, _, flow| {
            *flow = ControlFlow::Wait;
            match event {
                Event::NewEvents(StartCause::Init) => {
                    match make_tray(bitg_tsf::config::load().enabled) {
                        Ok(t) => tray = Some(t),
                        Err(e) => eprintln!("tray: {e}"),
                    }
                }
                Event::UserEvent(Ev::Reply(json)) => send(&webview, &json),
                Event::UserEvent(Ev::Show) => show(&window),
                Event::UserEvent(Ev::Hide) => window.set_visible(false),
                Event::UserEvent(Ev::Saved) => {
                    if let Some(t) = &tray {
                        sync_tray(t);
                    }
                }
                Event::UserEvent(Ev::UpdateFound(v)) => {
                    if let Some(t) = &tray {
                        t.update.set_text(format!("Yangilash: v{v}"));
                        let _ = t.icon.set_tooltip(Some(format!("bitg — v{v} chiqdi")));
                    }
                }
                Event::UserEvent(Ev::Quit) => *flow = ControlFlow::Exit,
                Event::UserEvent(Ev::Tray(TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                })) => show(&window),
                Event::UserEvent(Ev::Menu(e)) => {
                    let Some(t) = &tray else { return };
                    if e.id == t.enabled.id() {
                        let on = t.enabled.is_checked();
                        let _ = logic::set_enabled(on);
                        sync_tray(t);
                        send(&webview, &logic::load().to_string());
                    } else if e.id == t.open.id() {
                        show(&window);
                    } else if e.id == t.feedback.id() {
                        show(&window);
                        send(&webview, r#"{"kind":"goto","page":"fb"}"#);
                    } else if e.id == t.update.id() {
                        show(&window);
                        send(&webview, r#"{"kind":"goto","page":"news"}"#);
                        let p = proxy.clone();
                        std::thread::spawn(move || check_update(&p, true));
                    } else if e.id == t.quit.id() {
                        *flow = ControlFlow::Exit;
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    // Yopish = trayga yashirish; to'liq chiqish — tray menyusidan.
                    if tray.is_some() {
                        window.set_visible(false);
                        send(&webview, r#"{"kind":"hidden"}"#);
                    } else {
                        *flow = ControlFlow::Exit;
                    }
                }
                _ => {}
            }
        });
    }

    fn send(webview: &WebView, json: &str) {
        // JSON — JavaScript ifodasi sifatida ham to'g'ri.
        let _ = webview.evaluate_script(&format!("window.bitg && bitg.recv({json})"));
    }

    fn reply(p: &EventLoopProxy<Ev>, v: Value) {
        let _ = p.send_event(Ev::Reply(v.to_string()));
    }

    fn status() -> Value {
        json!({
            "kind": "status",
            "ime": system::ime_registered(),
            "keyboard": system::keyboard_added(),
            "autostart": system::autostart(),
            "hotkey": hotkey::status_json(&bitg_tsf::config::load(), hotkey::registered()),
            "version": VERSION,
        })
    }

    fn update_state(state: &str) -> Value {
        json!({"kind": "update", "state": state, "current": VERSION})
    }

    /// Manifestni o'qiydi. `manual` — foydalanuvchi o'zi so'radi: «eng yangisi» va xato ham ko'rsatiladi.
    fn check_update(p: &EventLoopProxy<Ev>, manual: bool) {
        if manual {
            reply(p, update_state("checking"));
        }
        let r = net::request("GET", updates::MANIFEST_URL, None, 256 * 1024, 15_000).and_then(
            |(code, body)| match code {
                200 => updates::parse_manifest(&String::from_utf8_lossy(&body), VERSION),
                _ => Err(format!("server {code}")),
            },
        );
        match r {
            Ok(Some(u)) => {
                let mut v = update_state("available");
                v["update"] = updates::update_json(&u);
                let _ = p.send_event(Ev::UpdateFound(u.version.clone()));
                *UPDATE.lock().unwrap_or_else(|e| e.into_inner()) = Some(u);
                reply(p, v);
            }
            Ok(None) if manual => reply(p, update_state("latest")),
            Err(e) if manual => {
                let mut v = update_state("error");
                v["error"] = json!(format!("Tekshirib bo'lmadi: {e}."));
                reply(p, v);
            }
            _ => {}
        }
    }

    /// MSI ni yuklab, SHA-256 ni tekshiradi va o'rnatuvchini ishga tushiradi. O'rnatuvchi bitg'ni o'zi yopadi;
    /// tugagach yangi versiya ochiladi. Bekor qilinsa (UAC) — eski versiya qayta ochiladi.
    fn install_update(p: &EventLoopProxy<Ev>) {
        let Some(u) = UPDATE.lock().unwrap_or_else(|e| e.into_inner()).clone() else {
            return;
        };
        if INSTALLING.swap(true, Ordering::SeqCst) {
            return;
        }
        let fail = |e: String| {
            INSTALLING.store(false, Ordering::SeqCst);
            let mut v = update_state("error");
            v["error"] = json!(e);
            reply(p, v);
        };
        reply(p, update_state("downloading"));
        let data = match net::request("GET", &u.url, None, updates::MAX_SIZE as usize, 60_000) {
            Ok((200, d)) => d,
            Ok((code, _)) => return fail(format!("Yuklab bo'lmadi (server {code}).")),
            Err(e) => return fail(format!("Yuklab bo'lmadi: {e}.")),
        };
        if data.len() as u64 != u.size || updates::sha256_hex(&data) != u.sha256 {
            return fail("Fayl tekshiruvdan o'tmadi (SHA-256). O'rnatilmadi.".into());
        }
        let Some(dir) = bitg_tsf::config::local_dir().map(|d| d.join("updates")) else {
            return fail("Vaqtinchalik papka topilmadi.".into());
        };
        let _ = std::fs::remove_dir_all(&dir);
        let msi = dir.join(format!("bitg-{}.msi", u.version));
        if let Err(e) = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&msi, &data)) {
            return fail(format!("Faylni saqlab bo'lmadi: {e}."));
        }
        let exe = std::env::current_exe().unwrap_or_default();
        reply(p, update_state("installing"));
        // msiexec tugagach (yoki bekor qilinsa) ilova qayta ochiladi; MSI eski sozlamalarini (AUTOSTART,
        // ADDKEYBOARD) reestrdan o'zi oladi.
        let cmd = format!(
            "msiexec /i \"{}\" /passive /norestart & start \"\" \"{}\"",
            msi.display(),
            exe.display()
        );
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        match std::process::Command::new("cmd")
            .arg("/c")
            .raw_arg(&cmd)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
        {
            Ok(_) => {
                let _ = p.send_event(Ev::Quit);
            }
            Err(e) => fail(format!("O'rnatuvchi ochilmadi: {e}.")),
        }
    }

    fn news() -> Value {
        let c = bitg_tsf::config::load();
        let mut v = updates::news_json(updates::NEWS);
        let fresh = c.last_seen.is_empty() && !RAN_BEFORE.load(Ordering::Relaxed);
        if fresh {
            // Yangi o'rnatish: «nima o'zgardi» kerak emas, shu versiya ko'rilgan hisoblanadi.
            let _ = logic::change(|c| c.last_seen = VERSION.into());
        }
        v["current"] = json!(VERSION);
        v["show"] = json!(c.whats_new && !fresh && c.last_seen != VERSION);
        v
    }

    fn send_feedback(msg: Value, p: &EventLoopProxy<Ev>) {
        if SENDING.swap(true, Ordering::SeqCst) {
            return;
        }
        let with_diag = msg.get("diag").and_then(Value::as_bool).unwrap_or(false);
        let diag = with_diag.then(diag_json);
        let r = match feedback::payload(&msg, &shot::install_id(), diag) {
            Ok(body) => feedback::result(net::request(
                "POST",
                feedback::URL,
                Some(("application/json", body.to_string().as_bytes())),
                64 * 1024,
                30_000,
            )),
            Err(e) => json!({"kind": "feedback_result", "ok": false, "error": e}),
        };
        SENDING.store(false, Ordering::SeqCst);
        reply(p, r);
    }

    fn diag_json() -> Value {
        let c = bitg_tsf::config::load();
        let mut d = feedback::diag(
            &c,
            &shot::os_build(),
            c.opts.rules.len(),
            c.opts.exceptions.len(),
        );
        d["ime"] = json!(system::ime_registered());
        d["keyboard"] = json!(system::keyboard_added());
        d
    }

    fn handle(body: &str, p: &EventLoopProxy<Ev>) {
        let Ok(msg) = serde_json::from_str::<Value>(body) else {
            return;
        };
        let Some(cmd) = msg.get("cmd").and_then(Value::as_str) else {
            return;
        };
        match cmd {
            "load" => {
                reply(p, logic::load());
                reply(p, status());
            }
            "save" => {
                reply(p, logic::save(&msg));
                let _ = p.send_event(Ev::Saved);
            }
            "preview" => reply(p, logic::preview(&msg)),
            "convert" => reply(p, logic::convert_text(&msg)),
            "status" => reply(p, status()),
            "news" => {
                reply(p, news());
                if let Some(u) = UPDATE.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
                    let mut v = update_state("available");
                    v["update"] = updates::update_json(u);
                    reply(p, v);
                }
            }
            "news_seen" => {
                let _ = logic::change(|c| c.last_seen = VERSION.into());
            }
            "check_update" => {
                let p = p.clone();
                std::thread::spawn(move || check_update(&p, true));
            }
            "install_update" => {
                let p = p.clone();
                std::thread::spawn(move || install_update(&p));
            }
            "feedback" => {
                let p = p.clone();
                std::thread::spawn(move || send_feedback(msg, &p));
            }
            "feedback_diag" => reply(p, json!({"kind": "feedback_diag", "diag": diag_json()})),
            "screenshot" => {
                // Oyna yashiriladi — rasmda bitg'ning o'zi bo'lmasin; parcha tanlangach qaytadi.
                let _ = p.send_event(Ev::Hide);
                let p = p.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(250));
                    let v = match shot::snip(Duration::from_secs(120)) {
                        Some(bmp) => json!({"kind": "screenshot",
                            "image": format!("data:image/bmp;base64,{}", feedback::base64(&bmp))}),
                        None => json!({"kind": "screenshot", "error": "Rasm olinmadi."}),
                    };
                    let _ = p.send_event(Ev::Show);
                    reply(&p, v);
                });
            }
            "stats" => {
                let log = bitg_tsf::config::local_dir()
                    .map(|d| d.join("tsf.log"))
                    .and_then(|f| std::fs::read_to_string(f).ok())
                    .unwrap_or_default();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let tz = msg.get("tz").and_then(Value::as_i64).unwrap_or(0);
                reply(p, logic::stats(&log, now, tz));
            }
            "autostart" => {
                let on = msg.get("on").and_then(Value::as_bool).unwrap_or(false);
                if let Err(e) = system::set_autostart(on) {
                    reply(
                        p,
                        json!({"kind": "save_failed", "error": format!("Windows bilan ishga tushirish: {e}")}),
                    );
                }
                reply(p, status());
            }
            "open_folder" => {
                if let Some(dir) = bitg_tsf::config::dir() {
                    let _ = std::fs::create_dir_all(&dir);
                    let _ = std::process::Command::new("explorer").arg(dir).spawn();
                }
            }
            "add_keyboard" => {
                if !system::add_keyboard() {
                    // Siyosat taqiqlagan yoki API yo'q — foydalanuvchi o'zi qo'shadi.
                    let _ = std::process::Command::new("explorer")
                        .arg("ms-settings:regionlanguage")
                        .spawn();
                }
                reply(p, status());
            }
            "open_language_settings" => {
                let _ = std::process::Command::new("explorer")
                    .arg("ms-settings:regionlanguage")
                    .spawn();
            }
            "open_url" => {
                if let Some(u) = msg.get("url").and_then(Value::as_str) {
                    let page = UPDATE
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .as_ref()
                        .map(|u| u.page.clone())
                        .filter(|s| !s.is_empty());
                    if LINKS.contains(&u) || page.as_deref() == Some(u) {
                        open_url(&if u == FEEDBACK {
                            feedback_url()
                        } else {
                            u.to_owned()
                        });
                    }
                }
            }
            "quit" => {
                let _ = p.send_event(Ev::Quit);
            }
            _ => {}
        }
    }
}
