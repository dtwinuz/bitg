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
mod hotkey;
#[cfg_attr(not(windows), allow(dead_code))]
mod logic;
#[cfg(windows)]
mod system;

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
    use serde_json::{Value, json};
    use tao::dpi::LogicalSize;
    use tao::event::{Event, StartCause, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
    use tao::platform::windows::IconExtWindows;
    use tao::window::{Theme, Window, WindowBuilder};
    use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
    use wry::{WebContext, WebView, WebViewBuilder};

    use crate::{hotkey, logic, system};

    const PAGE: &str = include_str!("ui.html");
    const ICON_ON: u16 = 1;
    const ICON_OFF: u16 = 2;
    /// Oynadan ochiladigan havolalar (boshqasi ochilmaydi).
    const LINKS: &[&str] = &[
        "https://bitg.uz",
        "https://github.com/dtwinuz/bitg",
        FEEDBACK,
    ];
    /// Fikr sahifasi: versiya qo'shib ochiladi, GitHub shaklida o'zi yoziladi.
    const FEEDBACK: &str = "https://bitg.uz/fikr/";

    fn feedback_url() -> String {
        format!("{FEEDBACK}?v={}&os=windows", env!("CARGO_PKG_VERSION"))
    }

    fn open_url(u: &str) {
        let _ = std::process::Command::new("explorer").arg(u).spawn();
    }

    enum Ev {
        /// Oynaga JSON javob.
        Reply(String),
        Show,
        Menu(MenuEvent),
        Tray(TrayIconEvent),
        /// Sozlamalar oynadan saqlandi — tray holatini yangilash.
        Saved,
        Quit,
    }

    struct Tray {
        icon: TrayIcon,
        enabled: CheckMenuItem,
        open: MenuItem,
        feedback: MenuItem,
        quit: MenuItem,
    }

    fn tray_icon(on: bool) -> Option<tray_icon::Icon> {
        tray_icon::Icon::from_resource(if on { ICON_ON } else { ICON_OFF }, None).ok()
    }

    fn make_tray(on: bool) -> Result<Tray, Box<dyn std::error::Error>> {
        let enabled = CheckMenuItem::new("Yoqilgan", true, on, None);
        let open = MenuItem::new("bitg'ni ochish", true, None);
        let feedback = MenuItem::new("Xato haqida xabar berish", true, None);
        let quit = MenuItem::new("Chiqish", true, None);
        let menu = Menu::new();
        menu.append_items(&[
            &enabled,
            &PredefinedMenuItem::separator(),
            &open,
            &feedback,
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
        let data = std::env::var_os("LOCALAPPDATA")
            .map(|d| std::path::PathBuf::from(d).join("bitg").join("webview"));
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
                Event::UserEvent(Ev::Saved) => {
                    if let Some(t) = &tray {
                        sync_tray(t);
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
                        open_url(&feedback_url());
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
            "version": env!("CARGO_PKG_VERSION"),
        })
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
            "stats" => {
                let log = std::env::var_os("LOCALAPPDATA")
                    .map(|d| std::path::PathBuf::from(d).join("bitg").join("tsf.log"))
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
                if let Some(u) = msg.get("url").and_then(Value::as_str)
                    && LINKS.contains(&u)
                {
                    open_url(&if u == FEEDBACK {
                        feedback_url()
                    } else {
                        u.to_owned()
                    });
                }
            }
            "quit" => {
                let _ = p.send_event(Ev::Quit);
            }
            _ => {}
        }
    }
}
