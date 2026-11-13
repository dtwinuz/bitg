// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ilovalardan fikr qabul qilish: `POST /api/feedback` (bitg sozlamalar oynasidagi "Xabar berish") va
//! `GET /api/health`. Faqat Caddy ulanadi (bitg.uz/api/* -> bitg-bot:HTTP_PORT), port tashqariga ochilmaydi.
//! Fikr ADMIN_IDS chatlariga matn (va rasm bo'lsa sendPhoto) sifatida yetkaziladi. Xabar matni va token
//! log'ga yozilmaydi: faqat chipta raqami va HTTP holati.

use std::collections::{HashMap, VecDeque};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

/// So'rov tanasi chegarasi (Caddy'da ham shunday: request_body max_size).
pub const MAX_BODY: usize = 3 * 1024 * 1024;
const MAX_HEAD: usize = 16 * 1024;
/// Rasm (dekodlangandan keyin) chegarasi.
const MAX_IMAGE: usize = 2_500_000;
const MAX_DIAG: usize = 4096;
const TIMEOUT: Duration = Duration::from_secs(10);

/// (ko'pi bilan, oyna soniyada): bitta IP — soatiga 5, bitta o'rnatish — kuniga 10, hammasi — kuniga 300.
const BY_IP: (usize, u64) = (5, 3600);
const BY_ID: (usize, u64) = (10, 86_400);
const ALL: (usize, u64) = (300, 86_400);

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

/// So'rovni o'qiydi. Xato — HTTP holat kodi (400, 411, 413).
fn read_request(stream: &TcpStream) -> Result<(Head, Vec<u8>), u16> {
    let mut r = BufReader::new(stream);
    let mut raw = Vec::new();
    loop {
        let before = raw.len();
        let n = (&mut r)
            .take((MAX_HEAD + 1 - raw.len()) as u64)
            .read_until(b'\n', &mut raw)
            .map_err(|_| 400u16)?;
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
        return Ok((head, Vec::new()));
    }
    let len: usize = head
        .header("content-length")
        .ok_or(411u16)?
        .parse()
        .map_err(|_| 400u16)?;
    if len > MAX_BODY {
        return Err(413);
    }
    let mut body = vec![0; len];
    r.read_exact(&mut body).map_err(|_| 400u16)?;
    Ok((head, body))
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

/// Xotiradagi chegaralar (bot qayta ishga tushsa nolga qaytadi — bu yetarli).
#[derive(Default)]
pub struct Limits {
    ip: HashMap<String, VecDeque<u64>>,
    id: HashMap<String, VecDeque<u64>>,
    all: VecDeque<u64>,
}

fn prune(q: &mut VecDeque<u64>, window: u64, now: u64) {
    while q.front().is_some_and(|&t| now.saturating_sub(t) >= window) {
        q.pop_front();
    }
}

/// To'lgan bo'lsa — eng eski yozuv oynadan chiqquncha qolgan soniya.
fn full(q: &VecDeque<u64>, (max, window): (usize, u64), now: u64) -> Option<u64> {
    (q.len() >= max).then(|| (q[0] + window).saturating_sub(now).max(1))
}

impl Limits {
    /// Ruxsat bo'lsa hisobga yozadi; bo'lmasa — necha soniyadan keyin qayta urinish.
    pub fn check(&mut self, ip: &str, id: &str, now: u64) -> Result<(), u64> {
        if self.ip.len() + self.id.len() > 20_000 {
            self.ip
                .retain(|_, q| q.back().is_some_and(|&t| now - t < BY_IP.1));
            self.id
                .retain(|_, q| q.back().is_some_and(|&t| now - t < BY_ID.1));
        }
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

/// Adminlarga boradigan matn (Telegram 4096 chegarasiga sig'adi).
pub fn summary(f: &Feedback, ticket: &str) -> String {
    let mut s = format!(
        "{} · #{ticket}\n{} {} · o'rnatish {}\n",
        f.kind.label(),
        f.app,
        f.version,
        &f.id[..f.id.len().min(8)]
    );
    if !f.text.is_empty() {
        s.push('\n');
        s.push_str(&f.text);
        s.push('\n');
    }
    for (k, v) in [
        ("Yozildi", &f.typed),
        ("Chiqdi", &f.got),
        ("Kutilgan", &f.expected),
    ] {
        if !v.is_empty() {
            s.push_str(&format!("\n{k}: {v}"));
        }
    }
    if !f.contact.is_empty() {
        s.push_str(&format!("\n\nAloqa: {}", f.contact));
    }
    if let Some(d) = &f.diag {
        s.push_str(&format!("\n\nDiagnostika: {d}"));
    }
    s.chars().take(4000).collect()
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

/// Marshrut: (holat, javob). `forward` fikrni adminlarga yetkazadi (testda — soxta); `now` va `seed`
/// tashqaridan — test aniq bo'lsin.
#[allow(clippy::too_many_arguments)]
pub fn route(
    head: &Head,
    body: &[u8],
    peer: &str,
    admins_on: bool,
    limits: &mut Limits,
    now: u64,
    seed: u64,
    forward: &mut dyn FnMut(&Feedback, &str) -> bool,
) -> (u16, Value) {
    let err = |code: u16, e: &str| (code, json!({"ok": false, "error": e}));
    match (head.method.as_str(), head.path.as_str()) {
        ("GET", "/api/health") => return (200, json!({"ok": true})),
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
    // Caddy haqiqiy mijoz IP'sini X-Forwarded-For boshida beradi.
    let ip = head
        .header("x-forwarded-for")
        .and_then(|x| x.split(',').next())
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .unwrap_or(peer);
    if let Err(retry) = limits.check(ip, &f.id, now) {
        return (429, json!({"ok": false, "error": "rate", "retry": retry}));
    }
    let t = ticket(seed);
    if !forward(&f, &t) {
        return err(502, "send");
    }
    (200, json!({"ok": true, "ticket": t}))
}

fn reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        411 => "Length Required",
        413 => "Payload Too Large",
        415 => "Unsupported Media Type",
        429 => "Too Many Requests",
        502 => "Bad Gateway",
        _ => "Service Unavailable",
    }
}

fn respond(s: &mut TcpStream, code: u16, body: &Value) -> io::Result<()> {
    let b = body.to_string();
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

fn handle(mut s: TcpStream, ctx: &Ctx, limits: &mut Limits) -> io::Result<()> {
    s.set_read_timeout(Some(TIMEOUT))?;
    s.set_write_timeout(Some(TIMEOUT))?;
    let peer = s
        .peer_addr()
        .map(|a| a.ip().to_string())
        .unwrap_or_default();
    let (code, body) = match read_request(&s) {
        Err(code) => (
            code,
            json!({"ok": false, "error": if code == 413 { "size" } else { "request" }}),
        ),
        Ok((head, body)) => {
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
                !ctx.admins.is_empty(),
                limits,
                now.as_secs(),
                seed,
                &mut |f, t| forward(ctx, f, t),
            )
        }
    };
    respond(&mut s, code, &body)
}

/// Ulanishlar ketma-ket (trafik kichik, chegaralar va 10 s timeout bor).
pub fn serve(listener: TcpListener, ctx: Ctx) {
    let mut limits = Limits::default();
    for conn in listener.incoming().flatten() {
        if let Err(e) = handle(conn, &ctx, &mut limits) {
            eprintln!("fikr API: ulanish xatosi ({:?})", e.kind());
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

    #[test]
    fn routes() {
        let mut l = Limits::default();
        let mut sent = vec![];
        let mut fwd = |f: &Feedback, t: &str| {
            sent.push((f.kind, t.to_string()));
            true
        };
        let body = ok_json().to_string();
        let js = Some("application/json; charset=utf-8");
        let r = |h: &Head,
                 b: &str,
                 on: bool,
                 l: &mut Limits,
                 fw: &mut dyn FnMut(&Feedback, &str) -> bool| {
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
        let mut fail = |_: &Feedback, _: &str| false;
        let (c, _) = r(
            &head("POST", "/api/feedback", js),
            &body,
            true,
            &mut Limits::default(),
            &mut fail,
        );
        assert_eq!(c, 502);
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
        for c in [200, 400, 404, 405, 411, 413, 415, 429, 502, 503] {
            assert!(!reason(c).is_empty());
        }
    }
}
