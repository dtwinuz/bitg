// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Oyna bilan Rust o'rtasidagi JSON xabarlar. Platformaga bog'liq emas — testlanadi.
//!
//! Oyna yuboradi: `{"cmd":"load"}`, `{"cmd":"save","settings":{..},"exceptions":[..]}`,
//! `{"cmd":"preview","id":N,"text":"..","settings":{..},"exceptions":[..]}`, `{"cmd":"open_folder"}`.

use bitg_core::Reason;
use bitg_tsf::config::{self, Config, DEFAULT_DISABLED_APPS};
use bitg_tsf::keymap::{self, KEYS, Keymap, LEVELS};
use serde_json::{Value, json};

pub fn settings_json(c: &Config) -> Value {
    json!({
        "enabled": c.enabled,
        "english_guard": c.opts.english_guard,
        "cyrillic": c.opts.cyrillic,
        "restore_apostrophes": c.opts.restore_apostrophes,
        "tutuq": c.opts.tutuq.to_string(),
        "fallback": c.fallback,
        "undo_backspace": c.undo_backspace,
        "undo_remember": c.undo_remember,
        "disabled_apps": c.disabled_apps,
        "auto_convert": c.auto_convert,
        "keymap": c.keymap,
        "layout": c.layout,
        "hotkey": c.hotkey,
        "hotkey_keys": c.hotkey_keys,
        "hotkey_old_keys": c.hotkey_old_keys,
    })
}

/// Oyna uchun: `{"[": ["ö","Ö","[","{"], ...}` — faqat o'zgargan tugmalar, bo'sh qatlam "".
pub fn keymap_json(m: &Keymap) -> Value {
    let mut o = serde_json::Map::new();
    for &(label, _) in KEYS {
        let row = m.row(label);
        if row.iter().any(Option::is_some) {
            let cells: Vec<String> = row
                .iter()
                .map(|c| c.map(String::from).unwrap_or_default())
                .collect();
            o.insert(label.to_string(), json!(cells));
        }
    }
    Value::Object(o)
}

/// Oynadan kelgan maxsus tartib. Noto'g'ri belgi — bo'sh qatlam.
pub fn keymap_from(v: &Value) -> Keymap {
    let mut m = Keymap::default();
    let Some(o) = v.as_object() else { return m };
    for (label, cells) in o {
        let (Some(scan), Some(cells)) = (keymap::scan_of(label), cells.as_array()) else {
            continue;
        };
        for (lvl, c) in cells.iter().take(LEVELS).enumerate() {
            let c = c
                .as_str()
                .and_then(|t| keymap::parse_cell(t.trim()))
                .flatten();
            m.set(scan, lvl, c);
        }
    }
    m
}

/// Maxsus tartib: fayl bo'lsa — undan, aks holda standart tartib nusxasi (tahrir shundan boshlanadi).
fn custom_keymap(raw: &str) -> Keymap {
    let m = Keymap::parse(raw);
    if m.is_empty() {
        keymap::preset(keymap::DEFAULT_LAYOUT).unwrap_or_default()
    } else {
        m
    }
}

/// Oynadagi qiymatlardan `Config` (yo'q yoki noto'g'ri maydon — standart qiymat).
pub fn config_from(settings: &Value, exceptions: &[String]) -> Config {
    let mut c = Config::default();
    let b = |k: &str, d: bool| settings.get(k).and_then(Value::as_bool).unwrap_or(d);
    c.enabled = b("enabled", c.enabled);
    c.opts.english_guard = b("english_guard", c.opts.english_guard);
    c.opts.cyrillic = b("cyrillic", c.opts.cyrillic);
    c.opts.restore_apostrophes = b("restore_apostrophes", c.opts.restore_apostrophes);
    c.fallback = b("fallback", c.fallback);
    c.undo_backspace = b("undo_backspace", c.undo_backspace);
    c.undo_remember = b("undo_remember", c.undo_remember);
    c.auto_convert = b("auto_convert", c.auto_convert);
    c.keymap = b("keymap", c.keymap);
    c.hotkey = b("hotkey", c.hotkey);
    // Tugmalar: faqat to'g'ri birikma qabul qilinadi (oynada ro'yxatdan tanlanadi, ini'da qo'lda ham).
    if let Some(k) = settings.get("hotkey_keys").and_then(Value::as_str)
        && crate::hotkey::parse(k).is_some()
    {
        c.hotkey_keys = config::keys_value(k);
    }
    if let Some(k) = settings.get("hotkey_old_keys").and_then(Value::as_str) {
        let k = config::keys_value(k);
        if k.is_empty() || crate::hotkey::parse(&k).is_some() {
            c.hotkey_old_keys = k;
        }
    }
    if let Some(l) = settings.get("layout").and_then(Value::as_str)
        && (l == keymap::CUSTOM || keymap::PRESETS.contains(&l))
    {
        c.layout = l.to_string();
    }
    if let Some(t) = settings.get("tutuq").and_then(Value::as_str) {
        let mut it = t.chars();
        if let (Some(ch), None) = (it.next(), it.next()) {
            c.opts.tutuq = ch;
        }
    }
    if let Some(apps) = settings.get("disabled_apps").and_then(Value::as_array) {
        c.disabled_apps.clear();
        for a in apps.iter().filter_map(Value::as_str) {
            let a = a.trim().to_lowercase();
            if !a.is_empty() && !a.contains([',', ';', '\n']) && !c.disabled_apps.contains(&a) {
                c.disabled_apps.push(a);
            }
        }
    }
    c.opts.exceptions = config::parse("", &exceptions.join("\n")).opts.exceptions;
    c
}

/// Istisnolar fayli: izohsiz, bo'sh bo'lmagan qatorlar, takrorsiz (yozilishi saqlanadi).
pub fn exception_words(raw: &str) -> Vec<String> {
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for w in raw.lines() {
        let w = w.trim().trim_start_matches('\u{FEFF}');
        if w.is_empty() || w.starts_with('#') || w.chars().any(char::is_whitespace) {
            continue;
        }
        let k = bitg_core::chars::fold_key(w);
        if !seen.contains(&k) {
            seen.push(k);
            out.push(w.to_string());
        }
    }
    out
}

fn strings(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub fn load() -> Value {
    let (s, e) = config::read_raw();
    let c = config::parse(&s, &e);
    let mut layouts = serde_json::Map::new();
    for name in keymap::PRESETS {
        if let Some(m) = keymap::preset(name) {
            layouts.insert(name.to_string(), keymap_json(&m));
        }
    }
    let custom = custom_keymap(&config::read_keymap());
    layouts.insert(keymap::CUSTOM.into(), keymap_json(&custom));
    json!({
        "kind": "loaded",
        "settings": settings_json(&c),
        "layouts": layouts,
        "exceptions": exception_words(&e),
        "default_apps": DEFAULT_DISABLED_APPS,
        "folder": config::dir().map(|d| d.display().to_string()),
    })
}

/// Ikkala faylni yozadi. Avval vaqtinchalik faylga, so'ng almashtirish — IME yarim yozilgan faylni o'qimaydi.
pub fn save(msg: &Value) -> Value {
    let words = exception_words(&strings(msg.get("exceptions")).join("\n"));
    let c = config_from(msg.get("settings").unwrap_or(&Value::Null), &words);
    let result = (|| -> std::io::Result<()> {
        let dir = config::dir().ok_or_else(|| std::io::Error::other("APPDATA topilmadi"))?;
        std::fs::create_dir_all(&dir)?;
        let mut ex = String::from("# O'girilmaydigan so'zlar, har qatorda bittadan.\n");
        for w in &words {
            ex.push_str(w);
            ex.push('\n');
        }
        write_atomic(&dir.join(config::EXCEPTIONS_FILE), &ex)?;
        if let Some(custom) = msg.get("custom") {
            let text = keymap_from(custom).to_text();
            if config::read_keymap() != text {
                write_atomic(&dir.join(config::KEYMAP_FILE), &text)?;
            }
        }
        write_atomic(&dir.join(config::SETTINGS_FILE), &config::to_ini(&c))
    })();
    match result {
        Ok(()) => json!({"kind": "saved"}),
        Err(e) => json!({"kind": "save_failed", "error": e.to_string()}),
    }
}

/// Tray menyusidan: faqat `enabled` ni o'zgartiradi, qolgan sozlamalar va istisnolar saqlanadi.
pub fn set_enabled(on: bool) -> std::io::Result<()> {
    let (s, e) = config::read_raw();
    let mut c = config::parse(&s, &e);
    c.enabled = on;
    let dir = config::dir().ok_or_else(|| std::io::Error::other("APPDATA topilmadi"))?;
    std::fs::create_dir_all(&dir)?;
    write_atomic(&dir.join(config::SETTINGS_FILE), &config::to_ini(&c))
}

fn write_atomic(path: &std::path::Path, text: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)
}

fn reason_name(r: Reason) -> &'static str {
    match r {
        Reason::Converted => "converted",
        Reason::Unchanged => "unchanged",
        Reason::Protected => "protected",
        Reason::Exception => "exception",
        Reason::English => "english",
    }
}

/// Matn bo'laklari (bo'shliq bo'yicha): `[{"s": kirish, "d": chiqish, "r": sabab}]`. Bo'shliqlar ham bo'lak
/// (`r` = null). `convert` bo'shliqlarni aynan saqlaydi, shuning uchun kirish va chiqish bo'laklari mos keladi.
pub fn preview(msg: &Value) -> Value {
    let text = msg.get("text").and_then(Value::as_str).unwrap_or("");
    let words = exception_words(&strings(msg.get("exceptions")).join("\n"));
    let c = config_from(msg.get("settings").unwrap_or(&Value::Null), &words);
    let src = bitg_core::chars::normalize(text);
    let (out, report) = bitg_core::convert_explained(&src, &c.opts);
    let a = chunks(&src);
    let b = chunks(&out);
    let mut segs = Vec::with_capacity(a.len());
    if a.len() != b.len() {
        // Kutilmagan holat: bo'laklab ko'rsatib bo'lmaydi, butun matn bitta bo'lak.
        segs.push(json!({"s": src, "d": out, "r": null}));
    } else {
        let mut rep = report.iter().peekable();
        for (s, d) in a.iter().zip(&b) {
            if s.trim().is_empty() {
                segs.push(json!({"s": s, "d": d, "r": null}));
                continue;
            }
            // Bo'lak ichidagi tokenlar (tire bilan bo'lingan bo'lishi mumkin): eng muhim sababni olamiz.
            let mut pos = 0;
            let mut best: Option<Reason> = None;
            while let Some((tok, r, _)) = rep.peek() {
                let Some(i) = s[pos..].find(tok.as_str()) else {
                    break;
                };
                pos += i + tok.len();
                best = Some(match (best, *r) {
                    (Some(Reason::Converted), _) | (_, Reason::Converted) => Reason::Converted,
                    (Some(Reason::Unchanged) | None, r) => r,
                    (Some(b), _) => b,
                });
                rep.next();
            }
            segs.push(json!({"s": s, "d": d, "r": best.map(reason_name)}));
        }
    }
    json!({
        "kind": "preview",
        "id": msg.get("id").cloned().unwrap_or(Value::Null),
        "out": out,
        "segments": segs,
    })
}

/// "Matn" sahifasi: butun matnni o'girish. `dir`: "new" (eski lotin yoki kirill → yangi) yoki "old".
pub fn convert_text(msg: &Value) -> Value {
    let text = msg.get("text").and_then(Value::as_str).unwrap_or("");
    let dir = msg.get("dir").and_then(Value::as_str).unwrap_or("new");
    let out = if dir == "old" {
        let style = match msg.get("style").and_then(Value::as_str) {
            Some("ascii") => bitg_core::OldStyle::Ascii,
            _ => bitg_core::OldStyle::Official,
        };
        bitg_core::to_old(text, style)
    } else {
        let words = exception_words(&strings(msg.get("exceptions")).join("\n"));
        let mut c = config_from(msg.get("settings").unwrap_or(&Value::Null), &words);
        c.opts.cyrillic = true;
        bitg_core::convert(text, &c.opts)
    };
    let changed = chunks(text)
        .iter()
        .zip(chunks(&out).iter())
        .filter(|(a, b)| a != b)
        .count();
    json!({
        "kind": "converted",
        "id": msg.get("id").cloned().unwrap_or(Value::Null),
        "out": out,
        "changed": changed,
    })
}

/// IME log'idan statistika (`ts pid=N dastur.exe xabar`). Matn log'da yo'q — faqat sanaladi.
/// `now` va `tz` (soniya, mahalliy vaqt farqi) — kunlarni mahalliy vaqtda ajratish uchun.
pub fn stats(log: &str, now: u64, tz: i64) -> Value {
    const DAY: i64 = 86_400;
    let local_day = |ts: i64| (ts + tz).div_euclid(DAY);
    let today = local_day(now as i64);
    let mut days = [0u64; 14];
    let mut apps: Vec<(String, u64, i64)> = Vec::new(); // dastur, o'girilgan so'zlar, oxirgi faollik
    let mut first: Option<i64> = None;
    let mut total = 0u64;
    for line in log.lines() {
        let mut it = line.splitn(3, ' ');
        let (Some(ts), Some(pid), Some(rest)) = (it.next(), it.next(), it.next()) else {
            continue;
        };
        let Ok(ts) = ts.parse::<i64>() else { continue };
        if !pid.starts_with("pid=") {
            continue;
        }
        let lower = rest.to_ascii_lowercase();
        let (exe, msg) = match lower.find(".exe ") {
            Some(i) => (&rest[..i + 4], &rest[i + 5..]),
            None => match rest.split_once(' ') {
                Some((a, b)) => (a, b),
                None => continue,
            },
        };
        first.get_or_insert(ts);
        let converted = msg == "converted" || msg == "fallback: converted";
        let k = match apps.iter().position(|a| a.0.eq_ignore_ascii_case(exe)) {
            Some(k) => k,
            None => {
                apps.push((exe.to_string(), 0, ts));
                apps.len() - 1
            }
        };
        apps[k].2 = apps[k].2.max(ts);
        if converted {
            apps[k].1 += 1;
            total += 1;
            let back = today - local_day(ts);
            if (0..14).contains(&back) {
                days[13 - back as usize] += 1;
            }
        }
    }
    apps.sort_by_key(|a| std::cmp::Reverse(a.2));
    json!({
        "kind": "stats",
        "days": days,
        "today": days[13],
        "week": days[7..].iter().sum::<u64>(),
        "total": total,
        "since": first,
        "apps": apps.iter().map(|(e, n, last)| json!({"exe": e, "n": n, "last": last})).collect::<Vec<_>>(),
    })
}

/// Bo'shliq va bo'shliq bo'lmagan qismlar navbatma-navbat.
fn chunks(s: &str) -> Vec<&str> {
    let mut v = Vec::new();
    let mut start = 0;
    let mut ws: Option<bool> = None;
    for (i, c) in s.char_indices() {
        let w = c.is_whitespace();
        if ws.is_some_and(|p| p != w) {
            v.push(&s[start..i]);
            start = i;
        }
        ws = Some(w);
    }
    if start < s.len() {
        v.push(&s[start..]);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_marks_converted_and_english() {
        let v = preview(&json!({"id": 3, "text": "Please check, keyin  choy", "settings": {}}));
        assert_eq!(v["id"], 3);
        assert_eq!(v["out"], "Please check, keyin  çoy");
        let segs = v["segments"].as_array().unwrap();
        let r = |s: &str| {
            segs.iter()
                .find(|x| x["s"] == s)
                .map(|x| x["r"].clone())
                .unwrap()
        };
        assert_eq!(r("choy"), "converted");
        assert_eq!(r("check,"), "english");
        assert_eq!(r("  "), Value::Null);
        let joined: String = segs.iter().map(|x| x["d"].as_str().unwrap()).collect();
        assert_eq!(joined, v["out"]);
    }

    #[test]
    fn preview_uses_unsaved_settings() {
        let off =
            preview(&json!({"text": "ozbek choy", "settings": {"restore_apostrophes": false}}));
        assert_eq!(off["out"], "ozbek çoy");
        let ex = preview(&json!({"text": "choy", "settings": {}, "exceptions": ["Choy"]}));
        assert_eq!(ex["out"], "choy");
        assert_eq!(ex["segments"][0]["r"], "exception");
    }

    #[test]
    fn config_from_ignores_bad_values() {
        let c = config_from(
            &json!({"enabled": "ha", "tutuq": "ab", "disabled_apps": ["Foo.EXE", " ", "a,b", 5]}),
            &[],
        );
        assert!(c.enabled);
        assert_eq!(c.opts.tutuq, '\u{02BC}');
        assert_eq!(c.disabled_apps, vec!["foo.exe"]);
    }

    #[test]
    fn hotkey_settings_round_trip() {
        let c = config_from(&json!({}), &[]);
        assert_eq!(
            (c.hotkey, c.hotkey_keys.as_str(), c.hotkey_old_keys.as_str()),
            (true, "ctrl+shift+y", "")
        );
        let c = config_from(
            &json!({"hotkey": false, "hotkey_keys": "Ctrl+Shift+B", "hotkey_old_keys": "ctrl+shift+o"}),
            &[],
        );
        assert_eq!(
            (c.hotkey, c.hotkey_keys.as_str(), c.hotkey_old_keys.as_str()),
            (false, "ctrl+shift+b", "ctrl+shift+o")
        );
        let mut v = settings_json(&c);
        v["hotkey_old_keys"] = json!(" ");
        assert_eq!(config_from(&v, &[]).hotkey_old_keys, "");
        let v = settings_json(&c);
        assert_eq!(v["hotkey"], false);
        let back = config_from(&v, &[]);
        assert_eq!(back.hotkey_keys, "ctrl+shift+b");
        assert_eq!(back.hotkey_old_keys, "ctrl+shift+o");
        let bad = config_from(
            &json!({"hotkey_keys": "shift+y", "hotkey_old_keys": "q"}),
            &[],
        );
        assert_eq!(bad.hotkey_keys, "ctrl+shift+y");
        assert!(bad.hotkey_old_keys.is_empty());
    }

    #[test]
    fn keymap_json_round_trip() {
        let m = keymap::preset("tilmoch").unwrap();
        let v = keymap_json(&m);
        assert_eq!(v["["], json!(["ö", "Ö", "[", "{"]));
        assert_eq!(v["3"], json!(["", "", "", "№"]));
        assert!(v.get("a").is_none());
        assert_eq!(keymap_from(&v), m);
        let bad = keymap_from(&json!({"[": ["xy", "U+00D6", "\u{7}"], "nope": ["a"]}));
        assert_eq!(bad.get(0x1A, 0), None);
        assert_eq!(bad.get(0x1A, 1), Some('Ö'));
        assert_eq!(bad.get(0x1A, 2), None);
        assert_eq!(custom_keymap(""), m);
        let c = config_from(
            &json!({"layout": "custom", "keymap": true, "auto_convert": false}),
            &[],
        );
        assert_eq!(
            (c.keymap, c.auto_convert, c.layout.as_str()),
            (true, false, "custom")
        );
        assert_eq!(config_from(&json!({"layout": "x"}), &[]).layout, "tilmoch");
        assert!(keymap_from(&json!(null)).is_empty());
        assert!(keymap_from(&json!(["["])).is_empty());
        let own = custom_keymap("[ x X - -\n");
        assert_eq!(own.get(0x1A, 0), Some('x'));
        assert_eq!(own.get(0x1B, 0), None);
    }

    #[test]
    fn config_from_drops_duplicate_apps() {
        let c = config_from(
            &json!({"disabled_apps": ["a.exe", "A.EXE", "b.exe"], "tutuq": "'"}),
            &["Choy".to_string()],
        );
        assert_eq!(c.disabled_apps, vec!["a.exe", "b.exe"]);
        assert_eq!(c.opts.tutuq, '\'');
        assert_eq!(c.opts.exceptions, vec!["choy"]);
    }

    #[test]
    fn preview_of_empty_text() {
        let v = preview(&json!({"text": ""}));
        assert_eq!(v["out"], "");
        assert_eq!(v["id"], Value::Null);
        assert_eq!(v["segments"], json!([]));
    }

    #[test]
    fn stats_count_by_local_day_and_app() {
        let day = 86_400u64;
        let now = 100 * day + 3600; // 100-kun, 01:00 UTC
        let log = format!(
            "{a} pid=1 Telegram.exe activate\n{a} pid=1 Telegram.exe fallback: converted\n\
             {b} pid=2 chrome.exe converted\n{b} pid=2 chrome.exe word unchanged\n\
             {c} pid=3 Code Helper.exe converted\nbuzuq qator\n{d} pid=2 chrome.exe converted\n",
            a = now - 60,
            b = now - day,
            c = now - 2 * day,
            d = now - 30 * day,
        );
        let v = stats(&log, now, 0);
        assert_eq!(v["today"], 1);
        assert_eq!(v["week"], 3);
        assert_eq!(v["total"], 4);
        assert_eq!(v["days"][12], 1);
        assert_eq!(v["apps"][0]["exe"], "Telegram.exe");
        let code = v["apps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["exe"] == "Code Helper.exe");
        assert_eq!(code.unwrap()["n"], 1);
        // UTC+5: 01:00 UTC = 06:00 mahalliy, "kecha" o'sha-o'sha.
        assert_eq!(stats(&log, now, 5 * 3600)["today"], 1);
    }

    #[test]
    fn stats_skip_malformed_lines() {
        let log = "abc pid=1 Telegram.exe converted\n\
                   5 proc=1 Telegram.exe converted\n\
                   6 pid=1 yolgiz\n\
                   7 pid=1 app converted\n";
        let v = stats(log, 7, 0);
        assert_eq!(v["total"], 1);
        assert_eq!(v["since"], 7);
        assert_eq!(v["apps"], json!([{"exe": "app", "n": 1, "last": 7}]));
        let empty = stats("", 7, 0);
        assert_eq!(empty["total"], 0);
        assert_eq!(empty["since"], Value::Null);
    }

    #[test]
    fn convert_text_both_directions() {
        let v = convert_text(
            &json!({"text": "Toshkent шаҳар", "dir": "new", "settings": {"cyrillic": false}}),
        );
        assert_eq!(v["out"], "Toşkent şahar");
        assert_eq!(v["changed"], 2);
        let o = convert_text(&json!({"text": "Özbekiston", "dir": "old", "style": "ascii"}));
        assert_eq!(o["out"], "O'zbekiston");
    }

    #[test]
    fn exception_words_cleans_list() {
        assert_eq!(
            exception_words("# izoh\nShoh\n\nshoh\n  O‘zbekcha \nikki soz\n"),
            vec!["Shoh", "O‘zbekcha"]
        );
    }
}
