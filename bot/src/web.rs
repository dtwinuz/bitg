// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ilovalardan fikr qabul qilish: `POST /api/feedback` (bitg sozlamalar oynasidagi "Xabar berish") va
//! `GET /api/health`. Faqat Caddy ulanadi (bitg.uz/api/* -> bitg-bot:HTTP_PORT), port tashqariga ochilmaydi.
//! Fikr ADMIN_IDS chatlariga matn (va rasm bo'lsa sendPhoto) sifatida yetkaziladi. Xabar matni va token
//! log'ga yozilmaydi: faqat chipta raqami va HTTP holati.

use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

/// So'rov tanasi chegarasi (Caddy'da ham shunday: request_body max_size).
pub const MAX_BODY: usize = 3 * 1024 * 1024;
const MAX_HEAD: usize = 16 * 1024;
/// Rasm (dekodlangandan keyin) chegarasi.
const MAX_IMAGE: usize = 2_500_000;
const MAX_DIAG: usize = 4096;
/// Bitta o'qish yoki yozish uchun kutish.
const TIMEOUT: Duration = Duration::from_secs(10);
/// Butun so'rov (sarlavha + tana) shu vaqt ichida kelishi kerak: sekin mijoz oqimni ushlab turmasin.
const DEADLINE: Duration = Duration::from_secs(15);
/// Bir vaqtda ishlanadigan ulanishlar; ortig'i darhol 503 oladi.
const MAX_CONNS: usize = 16;
/// Tanasi BIG dan katta (rasmli) so'rovlar bir vaqtda ko'pi bilan BIG_MAX ta: konteyner xotirasi 64 MB.
const BIG: usize = 256 * 1024;
const BIG_MAX: usize = 2;
/// Telegram'ga yuborilishini kutayotgan fikrlar (har birida 2,5 MB gacha rasm bo'lishi mumkin).
const QUEUE: usize = 4;
/// Telegram xabar chegarasi: 4096 UTF-16 birlik (emoji 2 birlik oladi).
pub const TG_MAX: usize = 4096;

/// (ko'pi bilan, oyna soniyada): bitta IP — soatiga 5, bitta o'rnatish — kuniga 10, hammasi — kuniga 300.
const BY_IP: (usize, u64) = (5, 3600);
const BY_ID: (usize, u64) = (10, 86_400);
const ALL: (usize, u64) = (300, 86_400);
const MCP_IP: (usize, u64) = (120, 60);
const MCP_ALL: (usize, u64) = (3000, 60);

#[derive(Debug, PartialEq, Eq)]
pub struct Head {
    pub method: String,
    pub path: String,
    /// Kalitlar kichik harfda.
    pub headers: Vec<(String, String)>,
}

impl Head {
    pub fn header(&self, k: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == k)
            .map(|(_, v)| v.as_str())
    }
}

/// So'rov qatori va sarlavhalar (bo'sh qatorgacha bo'lgan matn).
pub fn parse_head(raw: &str) -> Option<Head> {
    let mut lines = raw.split("\r\n").filter(|l| !l.is_empty());
    let mut first = lines.next()?.split(' ');
    let (method, target, proto) = (first.next()?, first.next()?, first.next()?);
    if !proto.starts_with("HTTP/1.") || first.next().is_some() {
        return None;
    }
    let path = target.split('?').next().unwrap_or(target).to_string();
    let mut headers = Vec::new();
    for l in lines {
        let (k, v) = l.split_once(':')?;
        headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
    }
    Some(Head {
        method: method.to_string(),
        path,
        headers,
    })
}

/// Har o'qishdan oldin qolgan vaqtni qo'yadi: TIMEOUT — bitta o'qish uchun, `until` — butun so'rov uchun.
struct Timed<'a> {
    s: &'a TcpStream,
    until: Instant,
    late: &'a Cell<bool>,
}

impl Read for Timed<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let left = self.until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            self.late.set(true);
            return Err(io::ErrorKind::TimedOut.into());
        }
        self.s.set_read_timeout(Some(left.min(TIMEOUT)))?;
        let mut s = self.s;
        let r = s.read(buf);
        if let Err(e) = &r
            && matches!(
                e.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
            )
        {
            self.late.set(true);
        }
        r
    }
}

/// Bir vaqtdagi ishlar hisoblagichi (semafor). `Slot` tushib ketganda joy bo'shaydi.
struct Slots {
    used: AtomicUsize,
    max: usize,
}

struct Slot(Arc<Slots>);

impl Slots {
    fn new(max: usize) -> Arc<Self> {
        Arc::new(Slots {
            used: AtomicUsize::new(0),
            max,
        })
    }

    fn take(self: &Arc<Self>) -> Option<Slot> {
        if self.used.fetch_add(1, Ordering::AcqRel) >= self.max {
            self.used.fetch_sub(1, Ordering::AcqRel);
            return None;
        }
        Some(Slot(Arc::clone(self)))
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.used.fetch_sub(1, Ordering::AcqRel);
    }
}

type Request = (Head, Vec<u8>, Option<Slot>);

/// So'rovni `until` gacha o'qiydi. Xato — HTTP holat kodi (400, 408, 411, 413, 503).
/// Katta tana `big` dan joy oladi; u javob yozilguncha ushlab turiladi.
fn read_request(stream: &TcpStream, until: Instant, big: &Arc<Slots>) -> Result<Request, u16> {
    let late = Cell::new(false);
    let fail = |_: io::Error| if late.get() { 408u16 } else { 400 };
    let mut r = BufReader::new(Timed {
        s: stream,
        until,
        late: &late,
    });
    let mut raw = Vec::new();
    loop {
        let before = raw.len();
        let n = (&mut r)
            .take((MAX_HEAD + 1 - raw.len()) as u64)
            .read_until(b'\n', &mut raw)
            .map_err(fail)?;
        if n == 0 || raw.len() > MAX_HEAD {
            return Err(400);
        }
        if raw[before..] == *b"\r\n" || raw[before..] == *b"\n" {
            break;
        }
    }
    let text = String::from_utf8(raw)
        .map_err(|_| 400u16)?
        .replace("\r\n", "\n");
    let head = parse_head(&text.replace('\n', "\r\n")).ok_or(400u16)?;
    if head.method != "POST" {
        return Ok((head, Vec::new(), None));
    }
    let len: usize = head
        .header("content-length")
        .ok_or(411u16)?
        .parse()
        .map_err(|_| 400u16)?;
    if len > MAX_BODY {
        return Err(413);
    }
    let slot = if len > BIG {
        Some(big.take().ok_or(503u16)?)
    } else {
        None
    };
    let mut body = vec![0; len];
    r.read_exact(&mut body).map_err(fail)?;
    Ok((head, body, slot))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bug,
    Idea,
    Conversion,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Kind::Bug => "🐞 Xato",
            Kind::Idea => "💡 Taklif",
            Kind::Conversion => "🔤 Noto'g'ri o'girish",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Feedback {
    pub kind: Kind,
    pub text: String,
    pub contact: String,
    pub typed: String,
    pub got: String,
    pub expected: String,
    /// (mime, baytlar)
    pub image: Option<(String, Vec<u8>)>,
    pub diag: Option<String>,
    pub app: String,
    pub version: String,
    pub id: String,
}

/// Ixtiyoriy matn maydoni: yo'q yoki null — bo'sh; boshqa tur yoki uzun — xato (maydon nomi).
fn opt_str(v: &Value, k: &'static str, max: usize) -> Result<String, &'static str> {
    match v.get(k) {
        None | Some(Value::Null) => Ok(String::new()),
        Some(Value::String(s)) if s.chars().count() <= max => Ok(s.trim().to_string()),
        _ => Err(k),
    }
}

/// Ilova yuborgan JSON ni tekshiradi. Xato — noto'g'ri maydon nomi (javobda qaytadi).
pub fn validate(v: &Value) -> Result<Feedback, &'static str> {
    if v.get("v").and_then(Value::as_u64) != Some(1) {
        return Err("v");
    }
    let kind = match v.get("kind").and_then(Value::as_str) {
        Some("bug") => Kind::Bug,
        Some("idea") => Kind::Idea,
        Some("conversion") => Kind::Conversion,
        _ => return Err("kind"),
    };
    let text = opt_str(v, "text", 4000)?;
    let contact = opt_str(v, "contact", 200)?;
    let typed = opt_str(v, "typed", 500)?;
    let got = opt_str(v, "got", 500)?;
    let expected = opt_str(v, "expected", 500)?;
    let app = opt_str(v, "app", 16)?;
    if !matches!(app.as_str(), "windows" | "linux" | "macos") {
        return Err("app");
    }
    let version = opt_str(v, "version", 32)?;
    if version.is_empty() {
        return Err("version");
    }
    let id = opt_str(v, "id", 64)?;
    if id.len() < 8
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("id");
    }
    if text.is_empty() && !(kind == Kind::Conversion && !typed.is_empty() && !got.is_empty()) {
        return Err("text");
    }
    let image = match v.get("image") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(data_url(s).ok_or("image")?),
        _ => return Err("image"),
    };
    let diag = match v.get("diag") {
        None | Some(Value::Null) => None,
        Some(d @ Value::Object(_)) => {
            let s = d.to_string();
            if s.len() > MAX_DIAG {
                return Err("diag");
            }
            Some(s)
        }
        _ => return Err("diag"),
    };
    Ok(Feedback {
        kind,
        text,
        contact,
        typed,
        got,
        expected,
        image,
        diag,
        app,
        version,
        id,
    })
}

/// `data:image/jpeg;base64,...` yoki png -> (mime, baytlar). Bo'sh, buzuq yoki katta — None.
fn data_url(s: &str) -> Option<(String, Vec<u8>)> {
    let (mime, b64) = ["image/jpeg", "image/png"]
        .iter()
        .find_map(|m| Some((*m, s.strip_prefix(&format!("data:{m};base64,"))?)))?;
    if b64.len() / 4 * 3 > MAX_IMAGE + 3 {
        return None;
    }
    let bytes = base64_decode(b64)?;
    (!bytes.is_empty() && bytes.len() <= MAX_IMAGE).then(|| (mime.to_string(), bytes))
}

/// Standart base64 (padding bilan yoki usiz). Boshqa belgi — None.
pub fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.trim_end_matches('=');
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    (bits < 6).then_some(out)
}

/// Xotiradagi chegaralar (bot qayta ishga tushsa nolga qaytadi — bu yetarli). IP va o'rnatish raqami
/// faqat o'z oynasi davomida turadi: `sweep` oynasi tugagan kalitlarni o'chiradi (maxfiylik sahifasi).
#[derive(Default)]
pub struct Limits {
    ip: HashMap<String, VecDeque<u64>>,
    id: HashMap<String, VecDeque<u64>>,
    all: VecDeque<u64>,
    mcp_ip: HashMap<String, VecDeque<u64>>,
    mcp_all: VecDeque<u64>,
    swept: Option<u64>,
}

fn prune(q: &mut VecDeque<u64>, window: u64, now: u64) {
    while q.front().is_some_and(|&t| now.saturating_sub(t) >= window) {
        q.pop_front();
    }
}

fn sweep_map(m: &mut HashMap<String, VecDeque<u64>>, window: u64, now: u64) {
    m.retain(|_, q| {
        prune(q, window, now);
        !q.is_empty()
    });
}

impl Limits {
    /// Oynasi tugagan kalitlarni o'chiradi; soniyasiga ko'pi bilan bir marta ishlaydi. Har so'rovda va
    /// `serve` dagi har soniyalik oqimda chaqiriladi — so'rov kelmasa ham IP xotirada qolmaydi.
    pub fn sweep(&mut self, now: u64) {
        if self.swept == Some(now) {
            return;
        }
        self.swept = Some(now);
        sweep_map(&mut self.ip, BY_IP.1, now);
        sweep_map(&mut self.id, BY_ID.1, now);
        sweep_map(&mut self.mcp_ip, MCP_IP.1, now);
        prune(&mut self.all, ALL.1, now);
        prune(&mut self.mcp_all, MCP_ALL.1, now);
    }
}

/// To'lgan bo'lsa — eng eski yozuv oynadan chiqquncha qolgan soniya.
fn full(q: &VecDeque<u64>, (max, window): (usize, u64), now: u64) -> Option<u64> {
    (q.len() >= max).then(|| (q[0] + window).saturating_sub(now).max(1))
}

impl Limits {
    /// Ruxsat bo'lsa hisobga yozadi; bo'lmasa — necha soniyadan keyin qayta urinish.
    pub fn check(&mut self, ip: &str, id: &str, now: u64) -> Result<(), u64> {
        self.sweep(now);
        let qi = self.ip.entry(ip.to_string()).or_default();
        prune(qi, BY_IP.1, now);
        let qd = self.id.entry(id.to_string()).or_default();
        prune(qd, BY_ID.1, now);
        prune(&mut self.all, ALL.1, now);
        let wait = [
            full(qi, BY_IP, now),
            full(qd, BY_ID, now),
            full(&self.all, ALL, now),
        ]
        .into_iter()
        .flatten()
        .max();
        if let Some(w) = wait {
            return Err(w);
        }
        qi.push_back(now);
        qd.push_back(now);
        self.all.push_back(now);
        Ok(())
    }
}

impl Limits {
    /// MCP: bitta IP — daqiqasiga 120, hammasi — daqiqasiga 3000 (AI xizmatlari umumiy IP'lardan keladi).
    pub fn check_mcp(&mut self, ip: &str, now: u64) -> Result<(), u64> {
        self.sweep(now);
        let qi = self.mcp_ip.entry(ip.to_string()).or_default();
        prune(qi, MCP_IP.1, now);
        prune(&mut self.mcp_all, MCP_ALL.1, now);
        if let Some(w) = [full(qi, MCP_IP, now), full(&self.mcp_all, MCP_ALL, now)]
            .into_iter()
            .flatten()
            .max()
        {
            return Err(w);
        }
        qi.push_back(now);
        self.mcp_all.push_back(now);
        Ok(())
    }
}

/// 6 belgili chipta (Crockford base32): foydalanuvchi va admin bir xil raqamni ko'radi.
pub fn ticket(seed: u64) -> String {
    const A: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    let mut x = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (0..6)
        .map(|i| A[((x >> (i * 5)) & 31) as usize] as char)
        .collect()
}

pub fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// `s` ning ko'pi bilan `max` UTF-16 birlikka sig'adigan boshi (belgi o'rtasidan kesilmaydi).
pub fn cut_utf16(s: &str, max: usize) -> &str {
    let mut n = 0;
    for (i, c) in s.char_indices() {
        n += c.len_utf16();
        if n > max {
            return &s[..i];
        }
    }
    s
}

/// Diagnostikaga matn uzun bo'lsa ham qoldiriladigan joy (UTF-16 birlik).
const DIAG_MIN: usize = 600;

/// Adminlarga boradigan matn (Telegram 4096 UTF-16 chegarasiga sig'adi). Sarlavha, maydonlar va aloqa
/// hech qachon kesilmaydi (validate ularni cheklaydi); joy yetmasa avval erkin matn, so'ng diagnostika kesiladi.
pub fn summary(f: &Feedback, ticket: &str) -> String {
    let head = format!(
        "{} · #{ticket}\n{} {} · o'rnatish {}\n",
        f.kind.label(),
        f.app,
        f.version,
        &f.id[..f.id.len().min(8)]
    );
    let mut fields = String::new();
    for (k, v) in [
        ("Yozildi", &f.typed),
        ("Chiqdi", &f.got),
        ("Kutilgan", &f.expected),
    ] {
        if !v.is_empty() {
            fields.push_str(&format!("\n{k}: {v}"));
        }
    }
    if !f.contact.is_empty() {
        fields.push_str(&format!("\n\nAloqa: {}", f.contact));
    }
    let diag = f
        .diag
        .as_ref()
        .map(|d| format!("\n\nDiagnostika: {d}"))
        .unwrap_or_default();
    let room = TG_MAX.saturating_sub(utf16_len(&head) + utf16_len(&fields));
    // "\n" + matn + "\n"
    let text_need = if f.text.is_empty() {
        0
    } else {
        utf16_len(&f.text) + 2
    };
    let diag_room = room
        .saturating_sub(text_need)
        .max(room.saturating_sub(3).min(DIAG_MIN));
    let diag = cut_utf16(&diag, diag_room);
    let text_room = room - utf16_len(diag);
    let mut s = head;
    if !f.text.is_empty() {
        s.push('\n');
        if text_need <= text_room {
            s.push_str(&f.text);
        } else {
            // "\n" + "…" + "\n" uchun 3 birlik
            s.push_str(cut_utf16(&f.text, text_room.saturating_sub(3)));
            s.push('…');
        }
        s.push('\n');
    }
    s.push_str(&fields);
    s.push_str(diag);
    s
}

/// multipart/form-data: oddiy maydonlar + bitta fayl.
pub fn multipart(
    boundary: &str,
    fields: &[(&str, &str)],
    file: (&str, &str, &str, &[u8]),
) -> Vec<u8> {
    let mut out = Vec::with_capacity(file.3.len() + 512);
    for (k, v) in fields {
        out.extend_from_slice(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{k}\"\r\n\r\n{v}\r\n")
                .as_bytes(),
        );
    }
    let (name, filename, mime, data) = file;
    out.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\nContent-Type: {mime}\r\n\r\n"
        )
        .as_bytes(),
    );
    out.extend_from_slice(data);
    out.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    out
}

fn lock(l: &Mutex<Limits>) -> MutexGuard<'_, Limits> {
    // panic bo'lgan oqim chegaralarni buzmaydi: hisoblagichlar baribir izchil
    l.lock().unwrap_or_else(|e| e.into_inner())
}

/// Marshrut: (holat, javob). `forward` fikrni yuborish navbatiga qo'yadi (testda — soxta); xato — 503
/// (navbat to'la) yoki 502. `now` va `seed` tashqaridan — test aniq bo'lsin.
#[allow(clippy::too_many_arguments)]
pub fn route(
    head: &Head,
    body: &[u8],
    peer: &str,
    admins_on: bool,
    limits: &Mutex<Limits>,
    now: u64,
    seed: u64,
    forward: &mut dyn FnMut(Feedback, &str) -> Result<(), u16>,
) -> (u16, Value) {
    let err = |code: u16, e: &str| (code, json!({"ok": false, "error": e}));
    // Caddy haqiqiy mijoz IP'sini X-Forwarded-For boshida beradi.
    let ip = head
        .header("x-forwarded-for")
        .and_then(|x| x.split(',').next())
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .unwrap_or(peer);
    match (head.method.as_str(), head.path.as_str()) {
        ("GET", "/api/health") => return (200, json!({"ok": true})),
        ("POST", "/mcp") => return mcp_route(head, body, ip, limits, now),
        (_, "/mcp") => return err(405, "method"),
        (_, "/api/feedback") if head.method != "POST" => return err(405, "method"),
        ("POST", "/api/feedback") => {}
        _ => return err(404, "not_found"),
    }
    if !head
        .header("content-type")
        .is_some_and(|t| t.to_ascii_lowercase().starts_with("application/json"))
    {
        return err(415, "type");
    }
    if !admins_on {
        return err(503, "off");
    }
    let Ok(v) = serde_json::from_slice::<Value>(body) else {
        return err(400, "json");
    };
    let f = match validate(&v) {
        Ok(f) => f,
        Err(field) => return err(400, field),
    };
    let limited = lock(limits).check(ip, &f.id, now);
    if let Err(retry) = limited {
        return (429, json!({"ok": false, "error": "rate", "retry": retry}));
    }
    let t = ticket(seed);
    match forward(f, &t) {
        Ok(()) => (200, json!({"ok": true, "ticket": t})),
        Err(503) => err(503, "busy"),
        Err(_) => err(502, "send"),
    }
}

/// `POST /mcp`: JSON-RPC (`mcp::handle`). Bildirishnoma — 202, tana yo'q (`Value::Null`).
fn mcp_route(head: &Head, body: &[u8], ip: &str, limits: &Mutex<Limits>, now: u64) -> (u16, Value) {
    let rpc_err = |code: i64, m: &str| json!({"jsonrpc": "2.0", "id": null, "error": {"code": code, "message": m}});
    if !head
        .header("content-type")
        .is_some_and(|t| t.to_ascii_lowercase().starts_with("application/json"))
    {
        return (
            415,
            rpc_err(-32700, "Content-Type must be application/json"),
        );
    }
    let limited = lock(limits).check_mcp(ip, now);
    if let Err(retry) = limited {
        return (
            429,
            rpc_err(-32000, &format!("Rate limited, retry in {retry} s")),
        );
    }
    let Ok(v) = serde_json::from_slice::<Value>(body) else {
        return (400, rpc_err(-32700, "Parse error"));
    };
    crate::mcp::handle_http(&v).unwrap_or((202, Value::Null))
}

fn reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        411 => "Length Required",
        413 => "Payload Too Large",
        415 => "Unsupported Media Type",
        429 => "Too Many Requests",
        502 => "Bad Gateway",
        _ => "Service Unavailable",
    }
}

fn respond(s: &mut TcpStream, code: u16, body: &Value) -> io::Result<()> {
    let b = if body.is_null() {
        String::new()
    } else {
        body.to_string()
    };
    write!(
        s,
        "HTTP/1.1 {code} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{b}",
        reason(code),
        b.len()
    )?;
    s.flush()
}

pub struct Ctx {
    pub agent: ureq::Agent,
    /// `https://api.telegram.org/bot<token>` — hech qachon log'ga chiqmaydi.
    pub base: String,
    pub admins: Vec<i64>,
}

fn status_of(e: &ureq::Error) -> String {
    match e {
        ureq::Error::Status(c, _) => format!("HTTP {c}"),
        ureq::Error::Transport(t) => format!("tarmoq ({:?})", t.kind()),
    }
}

/// Har adminga matn, so'ng rasm. Kamida bittasiga yetsa — true.
fn forward(ctx: &Ctx, f: &Feedback, t: &str) -> bool {
    let text = summary(f, t);
    let mut ok = false;
    for &chat in &ctx.admins {
        let body =
            json!({"chat_id": chat, "text": text, "link_preview_options": {"is_disabled": true}});
        if let Err(e) = ctx
            .agent
            .post(&format!("{}/sendMessage", ctx.base))
            .send_json(body)
        {
            eprintln!("fikr #{t}: sendMessage {chat}: {}", status_of(&e));
            continue;
        }
        ok = true;
        if let Some((mime, data)) = &f.image {
            let boundary = format!("bitg{t}{}", data.len());
            let ext = if mime == "image/png" { "png" } else { "jpg" };
            let chat_s = chat.to_string();
            let caption = format!("#{t}");
            let mp = multipart(
                &boundary,
                &[("chat_id", &chat_s), ("caption", &caption)],
                ("photo", &format!("{t}.{ext}"), mime, data),
            );
            let r = ctx
                .agent
                .post(&format!("{}/sendPhoto", ctx.base))
                .set(
                    "Content-Type",
                    &format!("multipart/form-data; boundary={boundary}"),
                )
                .send_bytes(&mp);
            if let Err(e) = r {
                eprintln!("fikr #{t}: sendPhoto {chat}: {}", status_of(&e));
            }
        }
    }
    if ok {
        println!("fikr #{t}: yetkazildi");
    }
    ok
}

static SEQ: AtomicU64 = AtomicU64::new(0);

/// Fikr navbati: chipta javobi darhol qaytadi, Telegram'ga yuborishni alohida oqim bajaradi.
type Outbox = SyncSender<(Feedback, String)>;

fn enqueue(tx: &Outbox, f: Feedback, t: &str) -> Result<(), u16> {
    match tx.try_send((f, t.to_string())) {
        Ok(()) => Ok(()),
        Err(TrySendError::Full(_)) => Err(503),
        Err(TrySendError::Disconnected(_)) => Err(502),
    }
}

fn error_name(code: u16) -> &'static str {
    match code {
        413 => "size",
        408 => "timeout",
        503 => "busy",
        _ => "request",
    }
}

struct Shared {
    admins_on: bool,
    limits: Mutex<Limits>,
    big: Arc<Slots>,
    outbox: Outbox,
}

fn handle(mut s: TcpStream, sh: &Shared, until: Instant) -> io::Result<()> {
    s.set_write_timeout(Some(TIMEOUT))?;
    let peer = s
        .peer_addr()
        .map(|a| a.ip().to_string())
        .unwrap_or_default();
    let (code, body) = match read_request(&s, until, &sh.big) {
        Err(code) => (code, json!({"ok": false, "error": error_name(code)})),
        // _big: katta tana uchun joy javob yozilguncha band
        Ok((head, body, _big)) => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            let seed = now.as_nanos() as u64
                ^ (SEQ.fetch_add(1, Ordering::Relaxed) << 40)
                ^ u64::from(std::process::id());
            route(
                &head,
                &body,
                &peer,
                sh.admins_on,
                &sh.limits,
                now.as_secs(),
                seed,
                &mut |f, t| enqueue(&sh.outbox, f, t),
            )
        }
    };
    respond(&mut s, code, &body)
}

/// Joy yo'q: so'rovni o'qimasdan 503 va ulanishni yopish (qabul qiluvchi oqim band bo'lmasin).
fn busy(mut s: TcpStream) {
    let _ = s.set_write_timeout(Some(Duration::from_secs(1)));
    let _ = respond(&mut s, 503, &json!({"ok": false, "error": "busy"}));
    let _ = s.shutdown(Shutdown::Write);
}

/// Har ulanish o'z oqimida (bir vaqtda ko'pi bilan MAX_CONNS ta, so'rov DEADLINE ichida o'qiladi).
pub fn serve(listener: TcpListener, ctx: Ctx) {
    run(listener, ctx, MAX_CONNS, DEADLINE);
}

fn run(listener: TcpListener, ctx: Ctx, max_conns: usize, deadline: Duration) {
    let (outbox, inbox) = mpsc::sync_channel::<(Feedback, String)>(QUEUE);
    let sh = Arc::new(Shared {
        admins_on: !ctx.admins.is_empty(),
        limits: Mutex::new(Limits::default()),
        big: Slots::new(BIG_MAX),
        outbox,
    });
    thread::spawn(move || {
        for (f, t) in inbox {
            if !forward(&ctx, &f, &t) {
                eprintln!("fikr #{t}: hech bir adminga yetmadi");
            }
        }
    });
    let sweeper = Arc::clone(&sh);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs());
            lock(&sweeper.limits).sweep(now);
        }
    });
    let conns = Slots::new(max_conns);
    for conn in listener.incoming().flatten() {
        let Some(slot) = conns.take() else {
            busy(conn);
            continue;
        };
        let until = Instant::now() + deadline;
        let sh = Arc::clone(&sh);
        let spawned = thread::Builder::new().spawn(move || {
            let _slot = slot;
            if let Err(e) = handle(conn, &sh, until) {
                eprintln!("fikr API: ulanish xatosi ({:?})", e.kind());
            }
        });
        if let Err(e) = spawned {
            eprintln!("fikr API: oqim ochilmadi ({:?})", e.kind());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_json() -> Value {
        json!({"v": 1, "kind": "bug", "text": "Word'da ikki marta", "app": "windows",
               "version": "0.4.0", "id": "abcd1234efgh"})
    }

    fn head(m: &str, p: &str, ct: Option<&str>) -> Head {
        let mut headers = vec![];
        if let Some(c) = ct {
            headers.push(("content-type".into(), c.into()));
        }
        Head {
            method: m.into(),
            path: p.into(),
            headers,
        }
    }

    #[test]
    fn parses_head() {
        let h = parse_head(
            "POST /api/feedback?x=1 HTTP/1.1\r\nHost: a\r\nContent-Type: application/json\r\nX-Forwarded-For: 1.2.3.4, 10.0.0.1\r\n",
        )
        .unwrap();
        assert_eq!(h.path, "/api/feedback");
        assert_eq!(h.header("content-type"), Some("application/json"));
        assert!(parse_head("GET /\r\n").is_none());
        assert!(parse_head("GET / FTP/1\r\n").is_none());
        assert!(parse_head("GET / HTTP/1.1\r\nbad header\r\n").is_none());
        assert!(parse_head("").is_none());
    }

    #[test]
    fn validates_fields() {
        let f = validate(&ok_json()).unwrap();
        assert_eq!((f.kind, f.app.as_str()), (Kind::Bug, "windows"));
        let bad = |k: &str, v: Value| {
            let mut j = ok_json();
            j[k] = v;
            validate(&j).unwrap_err()
        };
        assert_eq!(bad("v", json!(2)), "v");
        assert_eq!(bad("kind", json!("x")), "kind");
        assert_eq!(bad("text", json!("a".repeat(4001))), "text");
        assert_eq!(bad("text", json!(5)), "text");
        assert_eq!(bad("text", json!("  ")), "text");
        assert_eq!(bad("app", json!("android")), "app");
        assert_eq!(bad("version", json!("")), "version");
        assert_eq!(bad("id", json!("short")), "id");
        assert_eq!(bad("id", json!("has space 1234")), "id");
        assert_eq!(bad("image", json!("data:image/gif;base64,AAAA")), "image");
        assert_eq!(bad("image", json!(1)), "image");
        assert_eq!(bad("diag", json!("x")), "diag");
        assert_eq!(bad("diag", json!({"a": "b".repeat(5000)})), "diag");
        // noto'g'ri o'girish: matnsiz, lekin yozildi + chiqdi bilan
        let mut c = ok_json();
        c["kind"] = json!("conversion");
        c["text"] = Value::Null;
        assert_eq!(validate(&c).unwrap_err(), "text");
        c["typed"] = json!("shahar");
        c["got"] = json!("shahar");
        c["image"] = json!("data:image/png;base64,iVBORw0KGgo=");
        c["diag"] = json!({"os": "10.0.26100"});
        let f = validate(&c).unwrap();
        assert_eq!(f.image.unwrap().0, "image/png");
        assert!(f.diag.unwrap().contains("26100"));
    }

    #[test]
    fn base64_round() {
        assert_eq!(base64_decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(base64_decode("aGVsbG8").unwrap(), b"hello");
        assert_eq!(base64_decode("+/8=").unwrap(), [0xfb, 0xff]);
        assert!(base64_decode("a").is_none());
        assert!(base64_decode("a$==").is_none());
        assert!(data_url("data:image/jpeg;base64,").is_none());
    }

    #[test]
    fn limits_per_ip_id_and_global() {
        let mut l = Limits::default();
        for i in 0..5 {
            assert!(l.check("1.1.1.1", &format!("id{i}xxxxx"), 1000).is_ok());
        }
        assert_eq!(l.check("1.1.1.1", "other123", 1000), Err(3600));
        assert!(l.check("1.1.1.1", "other123", 4600).is_ok());
        let mut l = Limits::default();
        for i in 0..10 {
            assert!(l.check(&format!("ip{i}"), "sameid12", 0).is_ok());
        }
        assert_eq!(l.check("ip99", "sameid12", 100), Err(86_300));
        let mut l = Limits::default();
        for i in 0..300 {
            assert!(l.check(&format!("i{i}"), &format!("d{i}"), 0).is_ok());
        }
        assert!(l.check("new", "new", 0).is_err());
    }

    type Fwd<'a> = dyn FnMut(Feedback, &str) -> Result<(), u16> + 'a;

    #[test]
    fn routes() {
        let mut l = Mutex::new(Limits::default());
        let mut sent = vec![];
        let mut fwd = |f: Feedback, t: &str| {
            sent.push((f.kind, t.to_string()));
            Ok(())
        };
        let body = ok_json().to_string();
        let js = Some("application/json; charset=utf-8");
        let r = |h: &Head, b: &str, on: bool, l: &mut Mutex<Limits>, fw: &mut Fwd| {
            route(h, b.as_bytes(), "9.9.9.9", on, l, 0, 7, fw)
        };
        assert_eq!(
            r(
                &head("GET", "/api/health", None),
                "",
                true,
                &mut l,
                &mut fwd
            )
            .0,
            200
        );
        assert_eq!(
            r(&head("GET", "/x", None), "", true, &mut l, &mut fwd).0,
            404
        );
        assert_eq!(
            r(
                &head("GET", "/api/feedback", None),
                "",
                true,
                &mut l,
                &mut fwd
            )
            .0,
            405
        );
        assert_eq!(
            r(
                &head("POST", "/api/feedback", Some("text/plain")),
                &body,
                true,
                &mut l,
                &mut fwd
            )
            .0,
            415
        );
        assert_eq!(
            r(
                &head("POST", "/api/feedback", js),
                &body,
                false,
                &mut l,
                &mut fwd
            )
            .0,
            503
        );
        assert_eq!(
            r(
                &head("POST", "/api/feedback", js),
                "{",
                true,
                &mut l,
                &mut fwd
            )
            .1["error"],
            "json"
        );
        let (c, v) = r(
            &head("POST", "/api/feedback", js),
            &body,
            true,
            &mut l,
            &mut fwd,
        );
        assert_eq!((c, v["ticket"].as_str().unwrap().len()), (200, 6));
        let mut h = head("POST", "/api/feedback", js);
        h.headers
            .push(("x-forwarded-for".into(), "5.5.5.5, 10.0.0.2".into()));
        for _ in 0..5 {
            assert_eq!(r(&h, &body, true, &mut l, &mut fwd).0, 200);
        }
        let (c, v) = r(&h, &body, true, &mut l, &mut fwd);
        assert_eq!((c, v["error"].as_str()), (429, Some("rate")));
        assert_eq!(sent.len(), 6);
        let mut fail = |_: Feedback, _: &str| Err(502);
        let (c, _) = r(
            &head("POST", "/api/feedback", js),
            &body,
            true,
            &mut Mutex::default(),
            &mut fail,
        );
        assert_eq!(c, 502);
        // navbat to'la: darhol 503, chipta yo'q
        let mut full = |_: Feedback, _: &str| Err(503);
        let (c, v) = r(
            &head("POST", "/api/feedback", js),
            &body,
            true,
            &mut Mutex::default(),
            &mut full,
        );
        assert_eq!((c, v["error"].as_str()), (503, Some("busy")));
    }

    #[test]
    fn enqueue_reports_full_and_closed_queue() {
        let (tx, rx) = mpsc::sync_channel(1);
        let f = || validate(&ok_json()).unwrap();
        assert_eq!(enqueue(&tx, f(), "T1"), Ok(()));
        assert_eq!(enqueue(&tx, f(), "T2"), Err(503));
        assert_eq!(rx.recv().unwrap().1, "T1");
        drop(rx);
        assert_eq!(enqueue(&tx, f(), "T3"), Err(502));
    }

    #[test]
    fn mcp_routes() {
        let l = Mutex::new(Limits::default());
        let mut fwd = |_: Feedback, _: &str| Ok(());
        let js = Some("application/json");
        let call = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"convert_text","arguments":{"text":"shahar"}}}"#;
        let (c, v) = route(
            &head("POST", "/mcp", js),
            call.as_bytes(),
            "1.1.1.1",
            false,
            &l,
            0,
            1,
            &mut fwd,
        );
        assert_eq!(c, 200);
        assert_eq!(
            v["result"]["structuredContent"]["converted_by"],
            "bitg (https://bitg.uz)"
        );
        let note = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let (c, v) = route(
            &head("POST", "/mcp", js),
            note.as_bytes(),
            "1.1.1.1",
            false,
            &l,
            0,
            1,
            &mut fwd,
        );
        assert_eq!((c, v.is_null()), (202, true));
        assert_eq!(
            route(
                &head("GET", "/mcp", None),
                b"",
                "1.1.1.1",
                false,
                &l,
                0,
                1,
                &mut fwd
            )
            .0,
            405
        );
        assert_eq!(
            route(
                &head("POST", "/mcp", Some("text/plain")),
                b"{}",
                "1.1.1.1",
                false,
                &l,
                0,
                1,
                &mut fwd
            )
            .0,
            415
        );
        assert_eq!(
            route(
                &head("POST", "/mcp", js),
                b"{",
                "1.1.1.1",
                false,
                &l,
                0,
                1,
                &mut fwd
            )
            .0,
            400
        );
        let l = Mutex::new(Limits::default());
        for _ in 0..MCP_IP.0 {
            assert_eq!(
                route(
                    &head("POST", "/mcp", js),
                    call.as_bytes(),
                    "2.2.2.2",
                    false,
                    &l,
                    5,
                    1,
                    &mut fwd
                )
                .0,
                200
            );
        }
        assert_eq!(
            route(
                &head("POST", "/mcp", js),
                call.as_bytes(),
                "2.2.2.2",
                false,
                &l,
                5,
                1,
                &mut fwd
            )
            .0,
            429
        );
        assert_eq!(
            route(
                &head("POST", "/mcp", js),
                call.as_bytes(),
                "2.2.2.2",
                false,
                &l,
                70,
                1,
                &mut fwd
            )
            .0,
            200
        );
    }

    #[test]
    fn summary_and_multipart() {
        let mut j = ok_json();
        j["kind"] = json!("conversion");
        j["typed"] = json!("shahar");
        j["got"] = json!("shahar");
        j["expected"] = json!("şahar");
        j["contact"] = json!("@ali");
        j["diag"] = json!({"os": "11"});
        let s = summary(&validate(&j).unwrap(), "AB12CD");
        for part in [
            "Noto'g'ri o'girish · #AB12CD",
            "windows 0.4.0",
            "Kutilgan: şahar",
            "Aloqa: @ali",
            "Diagnostika:",
        ] {
            assert!(s.contains(part), "{s}");
        }
        let m = multipart(
            "B",
            &[("chat_id", "1")],
            ("photo", "a.jpg", "image/jpeg", b"xyz"),
        );
        let m = String::from_utf8(m).unwrap();
        assert!(
            m.starts_with("--B\r\nContent-Disposition: form-data; name=\"chat_id\"\r\n\r\n1\r\n")
        );
        assert!(
            m.contains("filename=\"a.jpg\"\r\nContent-Type: image/jpeg\r\n\r\nxyz\r\n--B--\r\n")
        );
        assert_eq!(ticket(1), ticket(1));
        assert_ne!(ticket(1), ticket(2));
        for c in [200, 400, 404, 405, 408, 411, 413, 415, 429, 502, 503] {
            assert!(!reason(c).is_empty());
        }
    }

    #[test]
    fn long_text_keeps_contact_and_diag() {
        let mut j = ok_json();
        // 4000 belgi, lekin 8000 UTF-16 birlik
        j["text"] = json!("😀".repeat(4000));
        j["contact"] = json!("@ali");
        j["diag"] = json!({"os": "11", "pad": "d".repeat(3000)});
        j["typed"] = json!("ş".repeat(500));
        let s = summary(&validate(&j).unwrap(), "AB12CD");
        assert!(utf16_len(&s) <= TG_MAX, "{}", utf16_len(&s));
        assert!(s.contains("Aloqa: @ali"));
        assert!(s.contains("Diagnostika: {"));
        assert!(s.contains("😀…\n"));
        assert!(s.starts_with("🐞 Xato · #AB12CD"));
        // qisqa matn kesilmaydi, diagnostika to'liq
        let mut j = ok_json();
        j["diag"] = json!({"os": "11"});
        let s = summary(&validate(&j).unwrap(), "AB12CD");
        assert!(s.contains("\nWord'da ikki marta\n") && s.ends_with("{\"os\":\"11\"}"));
        assert_eq!(cut_utf16("a😀b", 2), "a");
        assert_eq!(cut_utf16("a😀b", 3), "a😀");
        assert_eq!(cut_utf16("ab", 9), "ab");
    }

    #[test]
    fn sweep_forgets_ips_after_window() {
        let mut l = Limits::default();
        l.check("1.1.1.1", "id000001", 0).unwrap();
        l.check_mcp("2.2.2.2", 0).unwrap();
        l.sweep(59);
        assert!(l.mcp_ip.contains_key("2.2.2.2"));
        l.sweep(60);
        assert!(l.mcp_ip.is_empty() && l.mcp_all.is_empty());
        assert!(l.ip.contains_key("1.1.1.1"));
        // boshqa IP so'rovi ham eskisini tozalaydi
        l.check_mcp("3.3.3.3", 3599).unwrap();
        assert!(l.ip.contains_key("1.1.1.1"));
        l.check_mcp("3.3.3.3", 3600).unwrap();
        assert!(!l.ip.contains_key("1.1.1.1"));
        assert_eq!(l.mcp_ip.len(), 1);
        assert!(l.id.contains_key("id000001"));
        l.sweep(86_400);
        assert!(l.id.is_empty() && l.all.is_empty());
    }

    #[test]
    fn slots_cap_and_release() {
        let s = Slots::new(2);
        let a = s.take().unwrap();
        let _b = s.take().unwrap();
        assert!(s.take().is_none());
        drop(a);
        assert!(s.take().is_some());
    }

    #[test]
    fn slow_body_hits_deadline() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        let client = thread::spawn(move || {
            let mut c = TcpStream::connect(addr).unwrap();
            c.write_all(
                b"POST /api/feedback HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 1000\r\n\r\n",
            )
            .unwrap();
            // har bayt o'qish timeout'idan tez keladi, lekin butun tana hech qachon kelmaydi
            for _ in 0..30 {
                if c.write_all(b"x").is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
        let (s, _) = l.accept().unwrap();
        let start = Instant::now();
        let r = read_request(&s, start + Duration::from_millis(500), &Slots::new(1));
        assert_eq!(r.err(), Some(408));
        assert!(start.elapsed() < Duration::from_secs(2));
        drop(s);
        client.join().unwrap();
    }

    #[test]
    fn big_body_needs_a_slot() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        let client = thread::spawn(move || {
            let mut c = TcpStream::connect(addr).unwrap();
            let head = format!(
                "POST /api/feedback HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
                BIG + 1
            );
            let _ = c.write_all(head.as_bytes());
        });
        let (s, _) = l.accept().unwrap();
        let big = Slots::new(1);
        let _held = big.take().unwrap();
        let r = read_request(&s, Instant::now() + Duration::from_secs(5), &big);
        assert_eq!(r.err(), Some(503));
        client.join().unwrap();
    }

    fn get(addr: std::net::SocketAddr, path: &str) -> String {
        let mut c = TcpStream::connect(addr).unwrap();
        c.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let _ = write!(c, "GET {path} HTTP/1.1\r\nHost: t\r\n\r\n");
        let mut out = String::new();
        let _ = c.read_to_string(&mut out);
        out
    }

    #[test]
    fn caps_concurrent_connections() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        let ctx = Ctx {
            agent: ureq::AgentBuilder::new().build(),
            base: String::new(),
            admins: vec![],
        };
        thread::spawn(move || run(l, ctx, 2, Duration::from_secs(5)));
        assert!(get(addr, "/api/health").starts_with("HTTP/1.1 200"));
        // oldingi so'rov oqimi joyini bo'shatsin
        thread::sleep(Duration::from_millis(200));
        // ikki sekin mijoz ikkala joyni egallaydi
        let idle: Vec<TcpStream> = (0..2).map(|_| TcpStream::connect(addr).unwrap()).collect();
        let start = Instant::now();
        let out = get(addr, "/api/health");
        assert!(out.starts_with("HTTP/1.1 503"), "{out}");
        assert!(out.contains("\"busy\""), "{out}");
        assert!(start.elapsed() < Duration::from_secs(2));
        drop(idle);
        // joy bo'shagach yana ishlaydi
        let ok = (0..50).any(|_| {
            thread::sleep(Duration::from_millis(50));
            get(addr, "/api/health").starts_with("HTTP/1.1 200")
        });
        assert!(ok);
    }
}
