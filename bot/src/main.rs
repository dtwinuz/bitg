// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! @bitguzbot: matnni bitg yadrosi bilan yangi alifboga (yoki /eski bilan eski lotinga) o'giradi,
//! /yuklash bilan kompyuter dasturiga yo'l ko'rsatadi, /fikr bilan xato va takliflarni adminlarga yetkazadi.
//! Long polling (getUpdates) — serverda kiruvchi port kerak emas.
//! Muhit: BOT_TOKEN (majburiy), ADMIN_IDS (vergul bilan; fikrlar shu chatlarga keladi, ular botga /start
//! yozgan bo'lishi kerak), NEWS_CHANNEL (standart bitguz), SITE_URL (standart https://bitg.uz), API_URL (faqat sinov).
//! Token URL ichida bo'ladi, shuning uchun xato matnida URL hech qachon chiqarilmaydi.

use std::collections::{HashMap, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};
use std::{env, process, thread};

use bitg_core::{OldStyle, Options, convert, to_old};
use serde_json::{Value, json};

/// Telegram xabar chegarasi 4096; UTF-16 hisobidagi farq uchun zaxira.
const CHUNK: usize = 4000;
/// /fikr dan keyin keyingi xabar shuncha vaqt ichida kelsa, u fikr sifatida yuboriladi.
const FEEDBACK_WAIT: Duration = Duration::from_secs(15 * 60);
/// Bitta foydalanuvchidan ikki fikr orasidagi eng kam vaqt (spamdan himoya).
const FEEDBACK_GAP: Duration = Duration::from_secs(20);
/// Adminga yuborilgan xabar -> foydalanuvchi: shuncha oxirgi yozuv eslab qolinadi.
const THREADS_MAX: usize = 2000;
const ISSUES: &str = "https://github.com/dtwinuz/bitg/issues/new/choose";

struct Bot {
    agent: ureq::Agent,
    base: String,
    site: String,
    user: String,
    news: String,
    opts: Options,
    admins: Vec<i64>,
    waiting: HashMap<i64, Instant>,
    last_feedback: HashMap<i64, Instant>,
    /// (admin chat, admindagi xabar id) -> foydalanuvchi chat id
    threads: VecDeque<((i64, i64), i64)>,
    seen_msg: u64,
    seen_inline: u64,
    seen_feedback: u64,
}

impl Bot {
    fn call(&self, method: &str, body: Value) -> Result<Value, String> {
        let url = format!("{}/{method}", self.base);
        let resp = match self.agent.post(&url).send_json(body) {
            Ok(r) => r,
            Err(ureq::Error::Status(code, r)) => {
                let d = r
                    .into_json::<Value>()
                    .ok()
                    .and_then(|v| v["description"].as_str().map(str::to_owned))
                    .unwrap_or_default();
                return Err(format!("{method}: HTTP {code} {d}"));
            }
            Err(e) => return Err(format!("{method}: tarmoq xatosi ({:?})", e.kind())),
        };
        let v: Value = resp
            .into_json()
            .map_err(|_| format!("{method}: javob JSON emas"))?;
        if v["ok"].as_bool() == Some(true) {
            Ok(v["result"].clone())
        } else {
            Err(format!("{method}: {}", v["description"]))
        }
    }

    fn log_err(r: Result<Value, String>) {
        if let Err(e) = r {
            eprintln!("xato: {e}");
        }
    }

    fn to_new(&self, s: &str) -> String {
        convert(s, &self.opts)
    }

    fn site(&self, path: &str) -> String {
        format!("{}{path}", self.site.trim_end_matches('/'))
    }

    fn help(&self) -> String {
        format!(
            "Salom! Men bitg botiman.\n\n\
             ✍️ Matn yuboring — yangi alifboda (ö ğ ş ç) qaytaraman.\n\
             ↩️ /eski matn — yangi alifbodan eski lotinga.\n\
             💬 Istalgan chatda: @{u} matn — natijani tanlang.\n\
             💻 /yuklash — kompyuter uchun bitg (Windows, Mac, Linux).\n\
             🐞 /fikr — xato yoki taklif yozish.\n\n\
             Kirill matnni ham yuborsangiz bo‘ladi. Inglizcha so‘z, havola va kodga tegmayman.\n\n\
             Yangiliklar: @{n}",
            u = self.user,
            n = self.news
        )
    }

    fn keyboard(&self) -> Value {
        json!({ "inline_keyboard": [
            [{ "text": "💻 Kompyuter uchun bitg", "url": self.site("/yuklab-olish/") }],
            [{ "text": "🔔 Yangiliklar", "url": format!("https://t.me/{}", self.news) },
             { "text": "🐞 Fikr yozish", "url": format!("https://t.me/{}?start=fikr", self.user) }],
            [{ "text": "💬 Boshqa chatda ishlatish", "switch_inline_query": "" }]
        ]})
    }

    fn downloads(&self, chat: i64, reply_to: Option<i64>) {
        let text = format!(
            "💻 bitg kompyuterda yozayotganingizda so‘zni o‘zi o‘giradi — istalgan dasturda, internetsiz.\n\n\
             🪟 Windows 10/11: {w}\n\
             🍏 Mac (sinov): brew install --cask dtwinuz/tap/bitg yoki {m}\n\
             🐧 Linux (Ubuntu, Debian): apt ombori — {l}\n\
             📱 Telefonda bitg o‘rnatilmaydi, Tilmoch klaviaturasi bor: {p}",
            w = self.site("/yuklab-olish/"),
            m = self.site("/yuklab-olish/#mac"),
            l = self.site("/yuklab-olish/#apt"),
            p = self.site("/yangi-harflarni-yozish/#telefon"),
        );
        let markup = json!({ "inline_keyboard": [
            [{ "text": "Windows", "url": self.site("/yuklab-olish/") },
             { "text": "Mac", "url": self.site("/yuklab-olish/#mac") },
             { "text": "Linux", "url": self.site("/yuklab-olish/#linux") }],
            [{ "text": "📱 Telefonda", "url": self.site("/yangi-harflarni-yozish/#telefon") }]
        ]});
        let mut body = json!({ "chat_id": chat, "text": text, "reply_markup": markup,
                               "link_preview_options": { "is_disabled": true } });
        if let Some(id) = reply_to {
            body["reply_parameters"] =
                json!({ "message_id": id, "allow_sending_without_reply": true });
        }
        Self::log_err(self.call("sendMessage", body));
    }

    // ------------------------------------------------------------------ fikr (feedback)

    fn ask_feedback(&mut self, chat: i64) {
        if self.admins.is_empty() {
            self.send(
                chat,
                None,
                &format!("Xato yoki taklifni GitHub’da yozing: {ISSUES}"),
                false,
            );
            return;
        }
        self.waiting.insert(chat, Instant::now());
        self.send(
            chat,
            None,
            "🐞 Xato yoki taklifingizni bitta xabarda yozing: qaysi dasturda, nima yozdingiz, nima chiqdi. \
             Skrinshot ham yuborsa bo‘ladi.\n\nBekor qilish: /bekor",
            false,
        );
    }

    fn remember(&mut self, admin: i64, msg: i64, user: i64) {
        if self.threads.len() >= THREADS_MAX {
            self.threads.pop_front();
        }
        self.threads.push_back(((admin, msg), user));
    }

    /// Admin qaysi foydalanuvchiga javob beryapti: eslab qolingan xabar yoki sarlavhadagi "id 123".
    fn thread_user(&self, admin: i64, replied: &Value) -> Option<i64> {
        let id = replied["message_id"].as_i64()?;
        if let Some((_, u)) = self.threads.iter().rev().find(|(k, _)| *k == (admin, id)) {
            return Some(*u);
        }
        let text = replied["text"].as_str()?;
        if !text.starts_with("💬 Fikr") {
            return None;
        }
        const MARK: &str = " · id ";
        let tail = &text[text.find(MARK)? + MARK.len()..];
        let end = tail
            .find(|c: char| !(c.is_ascii_digit() || c == '-'))
            .unwrap_or(tail.len());
        tail[..end].parse().ok()
    }

    fn deliver(&mut self, m: &Value) {
        let Some(chat) = m["chat"]["id"].as_i64() else {
            return;
        };
        let now = Instant::now();
        if self
            .last_feedback
            .get(&chat)
            .is_some_and(|t| now.duration_since(*t) < FEEDBACK_GAP)
        {
            self.send(chat, None, "Biroz kuting va qayta yuboring.", false);
            return;
        }
        let from = &m["from"];
        let name = [from["first_name"].as_str(), from["last_name"].as_str()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        let uname = from["username"]
            .as_str()
            .map(|u| format!(" (@{u})"))
            .unwrap_or_default();
        let header = format!(
            "💬 Fikr · {name}{uname} · id {chat}\nJavob: shu xabarga yoki pastdagi nusxaga reply qiling."
        );
        let mut delivered = false;
        for admin in self.admins.clone() {
            let head = match self.call("sendMessage", json!({ "chat_id": admin, "text": header })) {
                Ok(h) => h["message_id"].as_i64(),
                Err(e) => {
                    eprintln!("xato: fikr sarlavhasi admin {admin} ga: {e}");
                    continue;
                }
            };
            if let Some(h) = head {
                self.remember(admin, h, chat);
            }
            let mut copy =
                json!({ "chat_id": admin, "from_chat_id": chat, "message_id": m["message_id"] });
            if let Some(h) = head {
                copy["reply_parameters"] =
                    json!({ "message_id": h, "allow_sending_without_reply": true });
            }
            match self.call("copyMessage", copy) {
                Ok(c) => {
                    if let Some(id) = c["message_id"].as_i64() {
                        self.remember(admin, id, chat);
                    }
                    delivered = true;
                }
                Err(e) => eprintln!("xato: fikr nusxasi admin {admin} ga: {e}"),
            }
        }
        if delivered {
            self.seen_feedback += 1;
            self.last_feedback.insert(chat, now);
            self.send(
                chat,
                None,
                "Rahmat! Xabaringiz bitg jamoasiga yetdi. Javob shu chatga keladi.",
                false,
            );
        } else {
            self.send(
                chat,
                None,
                &format!("Hozir yetkazib bo‘lmadi. GitHub’da yozing: {ISSUES}"),
                false,
            );
        }
    }

    fn answer(&self, admin: i64, user: i64, m: &Value) {
        let me = m["message_id"].as_i64();
        let res = self
            .call(
                "sendMessage",
                json!({ "chat_id": user, "text": "✉️ bitg jamoasidan javob:" }),
            )
            .and_then(|_| {
                self.call(
                    "copyMessage",
                    json!({ "chat_id": user, "from_chat_id": admin, "message_id": me }),
                )
            });
        let note = match res {
            Ok(_) => "✅ Yuborildi".to_owned(),
            Err(e) => format!("❌ Yuborilmadi: {e}"),
        };
        let mut body = json!({ "chat_id": admin, "text": note });
        if let Some(id) = me {
            body["reply_parameters"] =
                json!({ "message_id": id, "allow_sending_without_reply": true });
        }
        Self::log_err(self.call("sendMessage", body));
    }

    fn send(&self, chat: i64, reply_to: Option<i64>, text: &str, markup: bool) {
        let parts = split(text);
        let n = parts.len();
        for (i, part) in parts.into_iter().enumerate() {
            let mut body = json!({ "chat_id": chat, "text": part, "link_preview_options": { "is_disabled": true } });
            if let (0, Some(id)) = (i, reply_to) {
                body["reply_parameters"] =
                    json!({ "message_id": id, "allow_sending_without_reply": true });
            }
            if markup && i + 1 == n {
                body["reply_markup"] = self.keyboard();
            }
            Self::log_err(self.call("sendMessage", body));
        }
    }

    fn on_message(&mut self, m: &Value) {
        let Some(chat) = m["chat"]["id"].as_i64() else {
            return;
        };
        let private = m["chat"]["type"].as_str() == Some("private");
        let text = m["text"].as_str();
        let command = text.is_some_and(|t| t.starts_with('/'));

        // admin fikrga reply qildi -> foydalanuvchiga yuboriladi
        if private
            && !command
            && self.admins.contains(&chat)
            && m["reply_to_message"].is_object()
            && let Some(user) = self.thread_user(chat, &m["reply_to_message"])
        {
            self.answer(chat, user, m);
            return;
        }
        // /fikr dan keyingi xabar (matn, rasm yoki fayl)
        if private
            && !command
            && let Some(t) = self.waiting.remove(&chat)
            && t.elapsed() < FEEDBACK_WAIT
        {
            self.deliver(m);
            return;
        }

        let Some(text) = text else { return };
        self.seen_msg += 1;
        // Guruhda javob qaysi xabarga ekanini ko'rsatamiz; shaxsiy chatda shart emas.
        let reply_to = if private {
            None
        } else {
            m["message_id"].as_i64()
        };

        if let Some(cmdline) = text.strip_prefix('/') {
            let end = cmdline.find(char::is_whitespace).unwrap_or(cmdline.len());
            let (head, rest) = cmdline.split_at(end);
            let mut it = head.splitn(2, '@');
            let cmd = it.next().unwrap_or("").to_ascii_lowercase();
            if let Some(to) = it.next()
                && !to.eq_ignore_ascii_case(&self.user)
            {
                return; // boshqa botga yozilgan buyruq
            }
            let rest = rest.trim();
            let source = if rest.is_empty() {
                m["reply_to_message"]["text"]
                    .as_str()
                    .or_else(|| m["reply_to_message"]["caption"].as_str())
            } else {
                Some(rest)
            };
            match cmd.as_str() {
                "start" if rest == "fikr" && private => self.ask_feedback(chat),
                "start" if rest == "yuklash" => self.downloads(chat, reply_to),
                "fikr" if !private => self.send(
                    chat,
                    reply_to,
                    &format!("Fikrni shaxsiy chatda yozing: https://t.me/{}?start=fikr", self.user),
                    false,
                ),
                "fikr" if rest.is_empty() => self.ask_feedback(chat),
                "fikr" => {
                    if self.admins.is_empty() {
                        self.ask_feedback(chat);
                    } else {
                        self.deliver(m);
                    }
                }
                "bekor" => {
                    let had = self.waiting.remove(&chat).is_some();
                    self.send(chat, reply_to, if had { "Bekor qilindi." } else { "Bekor qiladigan narsa yo‘q." }, false);
                }
                "yuklash" | "download" => self.downloads(chat, reply_to),
                "start" | "yordam" | "help" => self.send(chat, reply_to, &self.help(), true),
                "eski" | "yangi" => match source {
                    Some(src) if cmd == "eski" && !has_new(src) => self.send(
                        chat,
                        reply_to,
                        "Matnda yangi alifbo harflari (ö ğ ş ç) yo‘q — u allaqachon eski lotinda.",
                        false,
                    ),
                    Some(src) => {
                        let out = if cmd == "eski" {
                            to_old(src, OldStyle::Official)
                        } else {
                            self.to_new(src)
                        };
                        self.reply_converted(chat, reply_to, src, &out, private);
                    }
                    None => self.send(
                        chat,
                        reply_to,
                        &format!("/{cmd} dan keyin matn yozing yoki biror xabarga javob qilib /{cmd} yuboring."),
                        false,
                    ),
                },
                _ if private => self.send(chat, reply_to, &self.help(), true),
                _ => {}
            }
            return;
        }

        if private {
            let out = self.to_new(text);
            self.reply_converted(chat, None, text, &out, true);
        }
    }

    fn reply_converted(
        &self,
        chat: i64,
        reply_to: Option<i64>,
        src: &str,
        out: &str,
        markup: bool,
    ) {
        if out == src {
            let hint = if has_new(src) {
                "O‘giriladigan so‘z topilmadi — matn allaqachon yangi alifboda. Eski lotinga: /eski"
            } else {
                "O‘giriladigan so‘z topilmadi: matnda o‘, g‘, sh, ch yoki kirill harflari yo‘q."
            };
            self.send(chat, reply_to, hint, false);
        } else {
            self.send(chat, reply_to, out, markup);
        }
    }

    fn on_inline(&mut self, q: &Value) {
        let Some(id) = q["id"].as_str() else { return };
        self.seen_inline += 1;
        let text = q["query"].as_str().unwrap_or("").trim();
        let mut results = Vec::new();
        if !text.is_empty() {
            let new = self.to_new(text);
            let title = if new == text {
                "Yangi alifbo (o‘zgarish yo‘q)"
            } else {
                "Yangi alifbo: ö ğ ş ç"
            };
            results.push(article("n", title, &new));
            // Eski lotinga faqat matnda yangi harflar bo'lsa: eski matnga to_old "sʼh" kabi ajratgich qo'yadi.
            if has_new(text) {
                results.push(article(
                    "o",
                    "Eski lotin: o‘ g‘ sh ch",
                    &to_old(text, OldStyle::Official),
                ));
            }
        }
        let body = json!({
            "inline_query_id": id,
            "results": results,
            "cache_time": 300,
            "button": { "text": "Kompyuterda ham: bitg", "start_parameter": "inline" }
        });
        Self::log_err(self.call("answerInlineQuery", body));
    }

    fn setup(&self) {
        let cmds = json!({ "commands": [
            { "command": "start", "description": "Boshlash" },
            { "command": "eski", "description": "Yangi alifbodan eski lotinga" },
            { "command": "yangi", "description": "Eski lotin yoki kirilldan yangi alifboga" },
            { "command": "yuklash", "description": "Kompyuter uchun bitg (Windows, Mac, Linux)" },
            { "command": "fikr", "description": "Xato yoki taklif yozish" },
            { "command": "yordam", "description": "Qanday ishlatish" }
        ]});
        Self::log_err(self.call("setMyCommands", cmds));
        Self::log_err(self.call("setMyShortDescription", json!({
            "short_description": "Matnni yangi o‘zbek alifbosiga (ö ğ ş ç) o‘giraman. Kompyuter uchun: bitg.uz"
        })));
        Self::log_err(self.call("setMyDescription", json!({
            "description": format!("Matn yuboring — yangi o‘zbek alifbosida (ö ğ ş ç) qaytaraman. Kirill ham bo‘ladi. /eski — eski lotinga. Istalgan chatda: @{} matn.\n\nWindows, Mac va Linux uchun bepul dastur: bitg.uz\nYangiliklar: @{}\nXato yoki taklif: /fikr", self.user, self.news)
        })));
    }
}

fn parse_ids(s: &str) -> Vec<i64> {
    s.split(',').filter_map(|x| x.trim().parse().ok()).collect()
}

fn has_new(s: &str) -> bool {
    s.chars().any(|c| "öğşçÖĞŞÇ".contains(c))
}

/// Xabarni CHUNK belgidan oshmaydigan qismlarga bo'ladi; iloji bo'lsa qator yoki bo'sh joyda.
fn split(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut rest: &str = text;
    while rest.chars().count() > CHUNK {
        let hard = rest
            .char_indices()
            .nth(CHUNK)
            .map_or(rest.len(), |(i, _)| i);
        let window = &rest[..hard];
        let cut = window
            .rfind('\n')
            .or_else(|| window.rfind(' '))
            .filter(|&i| i > hard / 2)
            .map_or(hard, |i| i + 1);
        parts.push(rest[..cut].to_owned());
        rest = &rest[cut..];
    }
    if !rest.is_empty() || parts.is_empty() {
        parts.push(rest.to_owned());
    }
    parts
}

fn article(id: &str, title: &str, text: &str) -> Value {
    let desc: String = text.chars().take(120).collect();
    let body: String = text.chars().take(CHUNK).collect();
    json!({
        "type": "article",
        "id": id,
        "title": title,
        "description": desc,
        "input_message_content": { "message_text": body, "link_preview_options": { "is_disabled": true } }
    })
}

fn main() {
    let token = env::var("BOT_TOKEN").unwrap_or_default();
    if token.trim().is_empty() {
        eprintln!("BOT_TOKEN berilmagan");
        process::exit(2);
    }
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(70))
        .timeout_write(Duration::from_secs(20))
        .build();
    let mut bot = Bot {
        agent,
        // API_URL faqat sinov uchun (soxta Telegram server).
        base: format!(
            "{}/bot{}",
            env::var("API_URL").unwrap_or_else(|_| "https://api.telegram.org".into()),
            token.trim()
        ),
        site: env::var("SITE_URL").unwrap_or_else(|_| "https://bitg.uz".into()),
        user: "bitguzbot".into(),
        news: env::var("NEWS_CHANNEL")
            .ok()
            .map(|s| s.trim().trim_start_matches('@').to_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "bitguz".into()),
        opts: Options::default(),
        admins: parse_ids(&env::var("ADMIN_IDS").unwrap_or_default()),
        waiting: HashMap::new(),
        last_feedback: HashMap::new(),
        threads: VecDeque::new(),
        seen_msg: 0,
        seen_inline: 0,
        seen_feedback: 0,
    };

    match bot.call("getMe", json!({})) {
        Ok(me) => {
            if let Some(u) = me["username"].as_str() {
                bot.user = u.to_owned();
            }
            println!(
                "ishga tushdi: @{} ({}), fikr adminlari: {}",
                bot.user,
                bot.site,
                bot.admins.len()
            );
        }
        Err(e) => {
            eprintln!("token tekshiruvi o'tmadi: {e}");
            process::exit(1);
        }
    }
    bot.setup();

    let mut offset: i64 = 0;
    let mut logged = 0u64;
    loop {
        let poll = json!({ "offset": offset, "timeout": 50, "allowed_updates": ["message", "inline_query"] });
        match bot.call("getUpdates", poll) {
            Ok(Value::Array(updates)) => {
                for u in updates {
                    if let Some(id) = u["update_id"].as_i64() {
                        offset = offset.max(id + 1);
                    }
                    // Bitta xabardagi xato butun botni to'xtatmasin (offset allaqachon oldinga surilgan).
                    let r = catch_unwind(AssertUnwindSafe(|| {
                        if u["message"].is_object() {
                            bot.on_message(&u["message"]);
                        } else if u["inline_query"].is_object() {
                            bot.on_inline(&u["inline_query"]);
                        }
                    }));
                    if r.is_err() {
                        eprintln!("xato: update {} ishlovida panic", u["update_id"]);
                    }
                }
                let total = bot.seen_msg + bot.seen_inline;
                if total >= logged + 50 {
                    logged = total;
                    println!(
                        "statistika: {} xabar, {} inline, {} fikr",
                        bot.seen_msg, bot.seen_inline, bot.seen_feedback
                    );
                }
                let now = Instant::now();
                bot.waiting
                    .retain(|_, t| now.duration_since(*t) < FEEDBACK_WAIT);
                bot.last_feedback
                    .retain(|_, t| now.duration_since(*t) < FEEDBACK_GAP);
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("xato: {e}");
                thread::sleep(Duration::from_secs(5));
            }
        }
    }
}
