#!/usr/bin/env python3
"""bitg.uz statistikasi: Caddy kirish jurnalidan umumiy sonlar, Telegram xulosa, uptime kuzatuvi.

Faqat standart kutubxona. Serverda `bitg-stats` konteynerida ishlaydi (deploy/docker-compose.yml).

Kirish:   LOG_DIR/access*.log[.gz]  — Caddy JSON jurnali (IP allaqachon /24 ga qisqartirilgan)
Chiqish:  DATA_DIR/state.json       — kunlik umumiy sonlar (IP va brauzer yo'q), abadiy saqlanadi
          DATA_DIR/dashboard.html   — bitta faylli hisobot (Telegram'ga ham yuboriladi)
          PUBLIC_DIR/stats.json     — {"downloads": N, "updated": "..."} — bosh sahifa hisoblagichi

Muhit:    BOT_TOKEN, ADMIN_CHAT_ID  — bo'lmasa Telegram o'chiq (hisob baribir yuritiladi)
          SITE (https://bitg.uz), GITHUB_REPO (dtwinuz/bitg), DIGEST_HOUR (9), INTERVAL (300)

Ishlatish:
  stats.py               — doimiy tsikl (konteyner)
  stats.py --once        — bir marta hisoblash
  stats.py --digest-now  — kechagi xulosani hozir yuborish (sinov)
  stats.py --print       — kechagi xulosa matnini chiqarish, hech narsa yubormaslik
  stats.py --selftest    — ichki sinov (CI)
"""
import argparse
import datetime as dt
import gzip
import html
import json
import os
import re
import socket
import ssl
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid
from collections import Counter, defaultdict
from pathlib import Path

TZ = dt.timezone(dt.timedelta(hours=5), "Asia/Tashkent")
UA = "bitg-stats/1 (+https://bitg.uz/maxfiylik/)"

ENV = {
    "LOG_DIR": os.environ.get("LOG_DIR", "/logs"),
    "DATA_DIR": os.environ.get("DATA_DIR", "/data"),
    "PUBLIC_DIR": os.environ.get("PUBLIC_DIR", "/public"),
    "SITE": os.environ.get("SITE", "https://bitg.uz").rstrip("/"),
    "GITHUB_REPO": os.environ.get("GITHUB_REPO", "dtwinuz/bitg"),
    "DIGEST_HOUR": int(os.environ.get("DIGEST_HOUR", "9")),
    "INTERVAL": int(os.environ.get("INTERVAL", "300")),
    "BOT_TOKEN": os.environ.get("BOT_TOKEN", "").strip(),
    "ADMIN_CHAT_ID": os.environ.get("ADMIN_CHAT_ID", "").strip(),
}

KINDS = ("win", "mac", "linux", "fcitx")
KIND_NAME = {"win": "Windows", "mac": "Mac", "linux": "Linux", "fcitx": "Fcitx5"}
OS_NAME = {"win": "Windows", "mac": "Mac", "linux": "Linux", "android": "Android", "ios": "iPhone/iPad",
           "other": "Boshqa"}
UZ_MONTHS = ["yanvar", "fevral", "mart", "aprel", "may", "iyun", "iyul", "avgust", "sentabr", "oktabr",
             "noyabr", "dekabr"]

# Qidiruv botlari, havola ko'rinishi (Telegram, WhatsApp), monitoring va skriptlar — tashrif emas.
BOT_RE = re.compile(
    r"bot\b|bot/|crawl|spider|slurp|preview|facebookexternalhit|embedly|whatsapp|vkshare|skypeuripreview|"
    r"headless|lighthouse|pagespeed|pingdom|uptime|monitor|python-|python/|go-http|okhttp|java/|libwww|"
    r"httpclient|axios|node-fetch|undici|scrapy|ahrefs|semrush|mj12|dataforseo|bitg-stats|^$", re.I)
CLI_RE = re.compile(r"^(curl|wget|powershell|winget|homebrew|aria2)", re.I)
APT_RE = re.compile(r"\bAPT-HTTP|\bapt\b|Debian APT", re.I)


def log(msg):
    print(f"{dt.datetime.now(TZ):%Y-%m-%d %H:%M:%S} {msg}", flush=True)


# ---------------------------------------------------------------- jurnal

def header(req, name):
    v = (req.get("headers") or {}).get(name)
    if isinstance(v, list):
        return v[0] if v else ""
    return v or ""


def ua_os(ua):
    if "Android" in ua:
        return "android"
    if re.search(r"iPhone|iPad|iPod", ua):
        return "ios"
    if "Windows NT" in ua:
        return "win"
    if re.search(r"Macintosh|Mac OS X", ua):
        return "mac"
    if re.search(r"Linux|X11|CrOS", ua):
        return "linux"
    return "other"


def dl_kind(name):
    if name.endswith(".msi"):
        return "win"
    if name.endswith(".pkg"):
        return "mac"
    if name.startswith("fcitx5-bitg") and name.endswith(".deb"):
        return "fcitx"
    if name.endswith(".deb"):
        return "linux"
    return None


def site_host(h):
    h = (h or "").lower().split(":")[0]
    return h in ("bitg.uz", "www.bitg.uz", "bitg.dtwin.uz", "localhost", "127.0.0.1")


def new_day():
    return {"visitors": 0, "pageviews": 0, "pages": Counter(), "os": Counter(), "refs": Counter(),
            "dl": Counter(), "dl_files": Counter(), "dl_cli": 0, "intents": Counter(),
            "apt_pkg": Counter(), "apt_clients": 0, "notfound": Counter()}


def aggregate(lines):
    """Jurnal qatorlari -> {sana: kunlik umumiy son}. IP va brauzer faqat xotirada, takrorni olib tashlash uchun."""
    days = defaultdict(new_day)
    visitors = defaultdict(set)
    seen_dl = defaultdict(set)
    apt_clients = defaultdict(set)
    for raw in lines:
        try:
            e = json.loads(raw)
        except (ValueError, TypeError):
            continue
        req = e.get("request") if isinstance(e, dict) else None
        if not isinstance(req, dict) or "ts" not in e or req.get("method") != "GET":
            continue
        status = int(e.get("status") or 0)
        parts = urllib.parse.urlsplit(req.get("uri", ""))
        path, query = parts.path, parts.query
        ua = header(req, "User-Agent")
        ip = req.get("client_ip") or req.get("remote_ip") or ""
        who = (ip, ua)
        day = dt.datetime.fromtimestamp(float(e["ts"]), TZ).date().isoformat()
        d = days[day]

        # to'liq fayl: 200 yoki boshidan boshlangan qismli (206 + Range: bytes=0-)
        full = status == 200 or (status == 206 and header(req, "Range").replace(" ", "").startswith("bytes=0-"))

        if path.startswith("/download/") or path.startswith("/apt/pool/"):
            name = path.rsplit("/", 1)[-1]
            kind = dl_kind(name)
            if not kind or not full:
                continue
            key = (ip, ua, name)
            if key in seen_dl[day]:
                continue
            seen_dl[day].add(key)
            if path.startswith("/apt/"):
                d["apt_pkg"][kind] += 1
                continue
            cli = bool(CLI_RE.search(ua))
            if BOT_RE.search(ua) and not cli:
                continue
            d["dl"][kind] += 1
            d["dl_files"][name] += 1
            d["dl_cli"] += cli
            continue

        if path.startswith("/apt/dists/"):
            if path.endswith("/InRelease") and status in (200, 304) and APT_RE.search(ua):
                apt_clients[day].add(who)
            continue

        is_page = path.endswith("/") or path.endswith(".html")
        if not is_page or BOT_RE.search(ua) or CLI_RE.search(ua):
            continue
        if status == 404:
            d["notfound"][path] += 1
            continue
        if status != 200:
            continue
        d["pageviews"] += 1
        d["pages"][path] += 1
        if path == "/yuklab-olish/rahmat/":
            os_q = urllib.parse.parse_qs(query).get("os", [""])[0]
            if os_q in KINDS:
                d["intents"][os_q] += 1
        if who not in visitors[day]:
            visitors[day].add(who)
            d["visitors"] += 1
            d["os"][ua_os(ua)] += 1
            ref = header(req, "Referer")
            host = urllib.parse.urlsplit(ref).hostname if ref else ""
            if host and not site_host(host):
                d["refs"][re.sub(r"^(www\.|m\.|l\.)", "", host)] += 1
    for day, s in apt_clients.items():
        days[day]["apt_clients"] = len(s)
    return {k: {f: (dict(v) if isinstance(v, Counter) else v) for f, v in d.items()} for k, d in days.items()}


def read_logs(log_dir):
    for f in sorted(Path(log_dir).glob("access*.log*")):
        opener = gzip.open if f.suffix == ".gz" else open
        try:
            with opener(f, "rt", encoding="utf-8", errors="replace") as fh:
                yield from fh
        except OSError as e:
            log(f"jurnal o'qilmadi: {f.name}: {e}")


def merge_days(state, fresh):
    """Yangi hisobni holatga qo'shadi. Jurnaldagi eng eski kun qisman o'chgan bo'lishi mumkin —
    holatda allaqachon bo'lsa, u qayta yozilmaydi."""
    days = state.setdefault("days", {})
    if not fresh:
        return
    oldest = min(fresh)
    for day, agg in fresh.items():
        if day == oldest and day in days and len(fresh) > 1:
            continue
        days[day] = agg


# ---------------------------------------------------------------- tarmoq

def http(url, data=None, headers=None, timeout=20):
    req = urllib.request.Request(url, data=data, headers={"User-Agent": UA, **(headers or {})})
    return urllib.request.urlopen(req, timeout=timeout)


def github_assets(repo):
    out = {}
    with http(f"https://api.github.com/repos/{repo}/releases?per_page=100",
              headers={"Accept": "application/vnd.github+json"}) as r:
        for rel in json.load(r):
            for a in rel.get("assets", []):
                if dl_kind(a["name"]):
                    out[a["name"]] = out.get(a["name"], 0) + int(a.get("download_count", 0))
    return out


def refresh_github(state, now):
    gh = state.setdefault("github", {})
    if now.timestamp() - gh.get("fetched_ts", 0) < 3600:
        return
    try:
        gh["assets"] = github_assets(ENV["GITHUB_REPO"])
        gh["fetched_ts"] = now.timestamp()
    except (urllib.error.URLError, OSError, ValueError) as e:
        log(f"github: {e}")


def gh_by_kind(state):
    c = Counter()
    for name, n in (state.get("github", {}).get("assets") or {}).items():
        c[dl_kind(name)] += n
    return c


# ---------------------------------------------------------------- hisob-kitob

def window(state, end, n):
    days = state.get("days", {})
    return [days.get((end - dt.timedelta(days=i)).isoformat()) or {} for i in range(n)]


def total(items, field, kind=None):
    s = 0
    for d in items:
        v = d.get(field, 0)
        if isinstance(v, dict):
            s += v.get(kind, 0) if kind else sum(v.values())
        else:
            s += v or 0
    return s


def sum_counter(items, field, top=None):
    c = Counter()
    for d in items:
        c.update(d.get(field) or {})
    return c.most_common(top)


def all_time_downloads(state):
    return total(state.get("days", {}).values(), "dl"), sum(gh_by_kind(state).values())


def pct(a, b):
    return f"{100 * a / b:.0f}%" if b else "—"


def trend(cur, prev):
    if not prev:
        return ""
    ch = (cur - prev) * 100 / prev
    return f" ({'▲' if ch >= 0 else '▼'}{abs(ch):.0f}%)"


def fmt(n):
    return f"{n:,}".replace(",", " ")


def write_atomic(path, text):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(path.name + ".tmp")
    tmp.write_text(text, encoding="utf-8")
    os.replace(tmp, path)


def write_public(state, now):
    site, gh = all_time_downloads(state)
    write_atomic(Path(ENV["PUBLIC_DIR"]) / "stats.json",
                 json.dumps({"downloads": site + gh, "updated": now.isoformat(timespec="minutes")}) + "\n")


# ---------------------------------------------------------------- xulosa (Telegram)

def digest(state, day):
    d = state.get("days", {}).get(day.isoformat()) or {}
    w7 = window(state, day, 7)
    p7 = window(state, day - dt.timedelta(days=7), 7)
    v, v7, vp = d.get("visitors", 0), total(w7, "visitors"), total(p7, "visitors")
    dl, dl7, dlp = total([d], "dl"), total(w7, "dl"), total(p7, "dl")
    by = " · ".join(f"{KIND_NAME[k]} {d['dl'][k]}" for k in KINDS if (d.get("dl") or {}).get(k))
    site, gh = all_time_downloads(state)
    e = html.escape
    rows = [
        f"📊 <b>bitg.uz — {day.day}-{UZ_MONTHS[day.month - 1]}</b>",
        "",
        f"👥 Tashrif: <b>{fmt(v)}</b>  ·  7 kun: {fmt(v7)}{trend(v7, vp)}",
        f"⬇️ Yuklab olish: <b>{fmt(dl)}</b>" + (f" — {by}" if by else "") + f"  ·  7 kun: {fmt(dl7)}{trend(dl7, dlp)}",
        f"🎯 Tashrifdan yuklashga: {pct(dl, v)}  ·  7 kun: {pct(dl7, v7)}",
    ]
    if d.get("apt_clients") or total([d], "apt_pkg"):
        rows.append(f"🐧 apt: {d.get('apt_clients', 0)} qurilma yangilanish tekshirdi, "
                    f"{total([d], 'apt_pkg')} paket o'rnatildi yoki yangilandi")
    os_mix = sum_counter(w7, "os")
    if os_mix:
        vt = sum(n for _, n in os_mix)
        rows.append("💻 Kim kiryapti (7 kun): " + ", ".join(f"{OS_NAME.get(k, k)} {pct(n, vt)}" for k, n in os_mix[:5]))
    refs = sum_counter([d], "refs", 5)
    if refs:
        rows.append("🔗 Qayerdan: " + ", ".join(f"{e(h)} {n}" for h, n in refs))
    pages = sum_counter([d], "pages", 4)
    if pages:
        rows.append("📄 Sahifalar: " + ", ".join(f"{e(p)} {n}" for p, n in pages))
    nf = sum_counter([d], "notfound", 3)
    if nf:
        rows.append("⚠️ 404: " + ", ".join(f"{e(p)} ({n})" for p, n in nf))
    rows += ["", f"🗂 Jami yuklab olish: <b>{fmt(site + gh)}</b> (sayt {fmt(site)} + GitHub {fmt(gh)})"]
    bad = [k for k, s in state.get("uptime", {}).items() if not s.get("ok", True)]
    if bad:
        rows.append("🔴 Hozir ishlamayapti: " + ", ".join(e(k) for k in bad))
    return "\n".join(rows)


def tg(method, fields=None, files=None):
    """ADMIN_CHAT_ID vergul bilan bir nechta bo'lishi mumkin; hech bo'lmasa bittasiga yetsa True."""
    chats = [c.strip() for c in ENV["ADMIN_CHAT_ID"].split(",") if c.strip()]
    if not ENV["BOT_TOKEN"] or not chats:
        return False
    return any([tg_one(method, c, fields, files) for c in chats])


def tg_one(method, chat, fields, files):
    url = f"https://api.telegram.org/bot{ENV['BOT_TOKEN']}/{method}"
    fields = {"chat_id": chat, **(fields or {})}
    if files:
        b = uuid.uuid4().hex
        parts = [f'--{b}\r\nContent-Disposition: form-data; name="{k}"\r\n\r\n{v}\r\n'.encode()
                 for k, v in fields.items()]
        for k, (fname, data, ctype) in files.items():
            parts.append(f'--{b}\r\nContent-Disposition: form-data; name="{k}"; filename="{fname}"\r\n'
                         f'Content-Type: {ctype}\r\n\r\n'.encode() + data + b"\r\n")
        parts.append(f"--{b}--\r\n".encode())
        body, ctype = b"".join(parts), f"multipart/form-data; boundary={b}"
    else:
        body, ctype = json.dumps(fields).encode(), "application/json"
    try:
        with http(url, data=body, headers={"Content-Type": ctype}, timeout=30) as r:
            return bool(json.load(r).get("ok"))
    except (urllib.error.URLError, OSError, ValueError) as e:
        # URL ichida token bor: xato matnida URL chiqarilmaydi
        log(f"telegram {method}: {type(e).__name__} {getattr(e, 'code', '')}")
        return False


def send_digest(state, day, with_file):
    ok = tg("sendMessage", {"text": digest(state, day), "parse_mode": "HTML", "disable_web_page_preview": True})
    p = Path(ENV["DATA_DIR"]) / "dashboard.html"
    if ok and with_file and p.exists():
        tg("sendDocument", {"caption": "bitg.uz hisobot — brauzerda oching"},
           {"document": (f"bitg-stats-{day.isoformat()}.html", p.read_bytes(), "text/html")})
    return ok


# ---------------------------------------------------------------- uptime

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *a, **k):
        return None


def check_site():
    """[(nom, ishlayaptimi, izoh)]: sayt, www yo'naltirishi, TLS sertifikat muddati."""
    out = []
    site = ENV["SITE"]
    host = urllib.parse.urlsplit(site).hostname
    try:
        with http(site + "/version.txt", timeout=15) as r:
            out.append((host, r.status == 200, f"HTTP {r.status}"))
    except urllib.error.HTTPError as e:
        out.append((host, False, f"HTTP {e.code}"))
    except (urllib.error.URLError, OSError) as e:
        out.append((host, False, type(e).__name__))

    www = "www." + host
    try:
        r = urllib.request.build_opener(NoRedirect).open(
            urllib.request.Request(f"https://{www}/", headers={"User-Agent": UA}), timeout=15)
        out.append((www, False, f"HTTP {r.status}, yo'naltirmayapti"))
    except urllib.error.HTTPError as e:
        loc = e.headers.get("Location", "")
        out.append((www, e.code in (301, 308) and loc.startswith(site), f"HTTP {e.code} -> {loc}"))
    except (urllib.error.URLError, OSError) as e:
        out.append((www, False, f"{type(e).__name__} (DNS yozuvi?)"))

    try:
        ctx = ssl.create_default_context()
        ctx.minimum_version = ssl.TLSVersion.TLSv1_2
        with socket.create_connection((host, 443), timeout=15) as s, ctx.wrap_socket(s, server_hostname=host) as t:
            end = ssl.cert_time_to_seconds(t.getpeercert()["notAfter"])
        left = int((end - time.time()) // 86400)
        out.append((f"{host} sertifikat", left >= 14, f"{left} kun qoldi"))
    except (OSError, ssl.SSLError, KeyError, ValueError) as e:
        out.append((f"{host} sertifikat", False, type(e).__name__))
    return out


def uptime(state):
    up = state.setdefault("uptime", {})
    for name, ok, note in check_site():
        s = up.setdefault(name, {"ok": True, "fails": 0})
        s["note"] = note
        if ok:
            if not s["ok"]:
                tg("sendMessage", {"text": f"✅ {name} yana ishlayapti ({note})"})
                log(f"uptime: {name} tiklandi")
            s.update(ok=True, fails=0)
            continue
        s["fails"] += 1
        # ikki marta ketma-ket xato — tasodifiy uzilishga signal bermaslik uchun
        if s["ok"] and s["fails"] >= 2:
            s["ok"] = False
            tg("sendMessage", {"text": f"🔴 {name} ishlamayapti: {note}"})
            log(f"uptime: {name} xato: {note}")


# ---------------------------------------------------------------- dashboard

CSS = """
:root{--bg:#f6f6f3;--surface:#fff;--text:#1d2027;--muted:#5d626b;--faint:#8a9099;--line:#e2e4e7;--bar:#2f62c0;--bar2:#b9c9ee;color-scheme:light}
@media (prefers-color-scheme:dark){:root{--bg:#0f1117;--surface:#171a22;--text:#e8eaee;--muted:#a3a9b4;--faint:#7c838f;--line:#2a2f3a;--bar:#7ea2f0;--bar2:#34497a;color-scheme:dark}}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--text);font:15px/1.5 Inter,"Segoe UI",system-ui,-apple-system,sans-serif}
main{max-width:1080px;margin:0 auto;padding:28px 16px 60px}h1{font-size:24px;margin:0;letter-spacing:-.02em}
h2{font-size:16px;margin:34px 0 12px}.sub{color:var(--muted);margin:4px 0 0;font-size:14px}
.tiles{display:grid;grid-template-columns:repeat(auto-fit,minmax(170px,1fr));gap:12px;margin:22px 0 0}
.tile{background:var(--surface);border:1px solid var(--line);border-radius:12px;padding:14px 16px}
.tile b{display:block;font-size:28px;letter-spacing:-.02em;font-variant-numeric:tabular-nums}.tile span{color:var(--muted);font-size:13px}
.card{background:var(--surface);border:1px solid var(--line);border-radius:12px;padding:16px;overflow-x:auto}
svg{display:block;width:100%;height:auto}svg text{fill:var(--faint);font-size:11px}svg .grid{stroke:var(--line)}
svg path.b{fill:var(--bar)}svg rect.hit{fill:transparent}svg g:hover path.b{fill:var(--text)}
.grid2{display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:0 12px}
table{width:100%;border-collapse:collapse;font-size:14px;font-variant-numeric:tabular-nums}
th,td{text-align:left;padding:7px 8px;border-bottom:1px solid var(--line)}th{color:var(--muted);font-weight:600;font-size:12.5px}
td.n,th.n{text-align:right}tr:last-child td{border-bottom:0}.mbar{height:6px;border-radius:3px;background:var(--bar2);margin-top:4px}
details summary{cursor:pointer;color:var(--muted);margin-top:10px}.note{color:var(--faint);font-size:13px;margin-top:28px}
"""


def bar_chart(series, label):
    """series: [(sana, son)] eskidan yangiga. Bitta qator, bitta o'q; ustunga olib borilsa — aniq son."""
    w, h, pad_l, pad_b, pad_t = 960, 200, 36, 22, 10
    n = max(len(series), 1)
    top = max([v for _, v in series] + [1])
    step = 10 ** max(len(str(top)) - 1, 0)
    top = -(-top // step) * step
    bw = (w - pad_l) / n
    gap = 2 if bw > 4 else 0
    ph = h - pad_t - pad_b
    out = [f'<svg viewBox="0 0 {w} {h}" role="img" aria-label="{html.escape(label)}">']
    for i in range(3):
        y = pad_t + ph * i / 2
        out.append(f'<line class="grid" x1="{pad_l}" x2="{w}" y1="{y:.1f}" y2="{y:.1f}"/>'
                   f'<text x="{pad_l - 6}" y="{y + 4:.1f}" text-anchor="end">{fmt(round(top * (1 - i / 2)))}</text>')
    for i, (day, v) in enumerate(series):
        bh = ph * v / top
        x = pad_l + i * bw + gap / 2
        bwi = max(bw - gap, 1)
        r = min(4, bwi / 2, bh / 2) if bh else 0
        # ustun pastki chiziqqa tayanadi; yuqori burchaklar yumaloq (path), sichqoncha uchun butun balandlik
        out.append(f'<g><title>{day}: {fmt(v)}</title><rect class="hit" x="{x:.1f}" y="{pad_t}" width="{bwi:.1f}" height="{ph}"/>')
        if bh:
            y0, y1 = h - pad_b, h - pad_b - bh
            out.append(f'<path class="b" d="M{x:.1f},{y0:.1f}V{y1 + r:.1f}Q{x:.1f},{y1:.1f} {x + r:.1f},{y1:.1f}'
                       f'H{x + bwi - r:.1f}Q{x + bwi:.1f},{y1:.1f} {x + bwi:.1f},{y1 + r:.1f}V{y0:.1f}Z"/>')
        out.append("</g>")
        if i % max(n // 6, 1) == 0:
            out.append(f'<text x="{x + bwi / 2:.1f}" y="{h - 6}" text-anchor="middle">{day[5:]}</text>')
    out.append("</svg>")
    return "".join(out)


def table(rows, head, bar=True):
    if not rows:
        return '<p class="sub">Hali ma\'lumot yo\'q</p>'
    top = max(r[1] for r in rows) or 1
    body = "".join(
        f"<tr><td>{html.escape(str(k))}" + (f'<div class="mbar" style="width:{100 * v / top:.0f}%"></div>' if bar else "")
        + f'</td><td class="n">{fmt(v) if bar else ""}</td></tr>' for k, v in rows)
    return f'<table><tr><th>{head[0]}</th><th class="n">{head[1]}</th></tr>{body}</table>'


def dashboard(state, now):
    today = now.date()
    w7, w30 = window(state, today, 7), window(state, today, 30)
    site, gh = all_time_downloads(state)
    ghk = gh_by_kind(state)
    alld = state.get("days", {})
    days90 = [(today - dt.timedelta(days=i)).isoformat() for i in range(89, -1, -1)]
    vis = [(d, (alld.get(d) or {}).get("visitors", 0)) for d in days90]
    dls = [(d, total([alld.get(d) or {}], "dl")) for d in days90]
    v7, d7 = total(w7, "visitors"), total(w7, "dl")
    kinds = [(KIND_NAME[k], total(w30, "dl", k)) for k in KINDS]
    kinds_all = [(KIND_NAME[k], total(alld.values(), "dl", k) + ghk.get(k, 0)) for k in KINDS]
    intents = [(KIND_NAME[k], total(w30, "intents", k)) for k in KINDS]
    daily_rows = "".join(f"<tr><td>{d}</td><td class='n'>{fmt(v)}</td><td class='n'>{fmt(x)}</td></tr>"
                         for (d, v), (_, x) in list(zip(vis, dls))[::-1][:30])
    up_rows = [(f"{'✅' if s.get('ok', True) else '🔴'} {k} — {s.get('note', '')}", 0)
               for k, s in state.get("uptime", {}).items()]
    apt7 = max((d.get("apt_clients", 0) for d in w7), default=0)
    return f"""<!doctype html><html lang="uz"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1"><title>bitg.uz statistika</title>
<style>{CSS}</style></head><body><main>
<h1>bitg.uz statistika</h1>
<p class="sub">{now:%Y-%m-%d %H:%M} (Toshkent) · manba: sayt jurnali (IP qisqartirilgan) + GitHub Releases</p>
<div class="tiles">
<div class="tile"><b>{fmt(v7)}</b><span>tashrif, 7 kun</span></div>
<div class="tile"><b>{fmt(d7)}</b><span>yuklab olish, 7 kun</span></div>
<div class="tile"><b>{pct(d7, v7)}</b><span>tashrifdan yuklashga, 7 kun</span></div>
<div class="tile"><b>{fmt(site + gh)}</b><span>jami yuklab olish (sayt {fmt(site)} + GitHub {fmt(gh)})</span></div>
<div class="tile"><b>{fmt(apt7)}</b><span>apt orqali yangilanadigan Linux qurilma (7 kunda eng ko'p)</span></div>
</div>
<h2>Tashrif buyuruvchilar, kunlik (90 kun)</h2><div class="card">{bar_chart(vis, "Kunlik tashrif")}</div>
<h2>Yuklab olishlar, kunlik (90 kun)</h2><div class="card">{bar_chart(dls, "Kunlik yuklab olish")}</div>
<details><summary>Jadval ko'rinishi (30 kun)</summary><div class="card"><table><tr><th>Sana</th><th class="n">Tashrif</th><th class="n">Yuklab olish</th></tr>{daily_rows}</table></div></details>
<div class="grid2">
<div><h2>Tizim bo'yicha yuklab olish, 30 kun</h2><div class="card">{table(kinds, ("Tizim", "Soni"))}</div></div>
<div><h2>Tizim bo'yicha, butun vaqt</h2><div class="card">{table(kinds_all, ("Tizim", "Soni"))}</div></div>
<div><h2>Tashrif buyuruvchilar tizimi, 7 kun</h2><div class="card">{table([(OS_NAME.get(k, k), n) for k, n in sum_counter(w7, "os")], ("Tizim", "Tashrif"))}</div></div>
<div><h2>"Yuklab olish" bosildi, 30 kun</h2><div class="card">{table(intents, ("Tizim", "Bosish"))}</div></div>
<div><h2>Qayerdan kelishdi, 30 kun</h2><div class="card">{table(sum_counter(w30, "refs", 12), ("Sayt", "Tashrif"))}</div></div>
<div><h2>Eng ko'p ochilgan sahifalar, 30 kun</h2><div class="card">{table(sum_counter(w30, "pages", 15), ("Sahifa", "Ko'rish"))}</div></div>
<div><h2>Topilmagan sahifalar (404), 30 kun</h2><div class="card">{table(sum_counter(w30, "notfound", 12), ("Manzil", "Soni"))}</div></div>
<div><h2>Kuzatuv</h2><div class="card">{table(up_rows, ("Holat", ""), bar=False)}</div></div>
</div>
<p class="note">Tashrif — kun ichida bir xil qisqartirilgan IP va brauzer bir marta sanaladi. Yuklab olish — bir kunda bir xil
fayl bir marta; botlar va davom ettirilgan qismli so'rovlar sanalmaydi. iPad Safari o'zini Mac deb ataydi, server tomonida
u Mac'ga qo'shiladi. Jurnal 30 kun saqlanadi, kunlik umumiy sonlar state.json'da qoladi.</p>
</main></body></html>
"""


# ---------------------------------------------------------------- tsikl

def load_state():
    try:
        return json.loads((Path(ENV["DATA_DIR"]) / "state.json").read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}


def save_state(state):
    write_atomic(Path(ENV["DATA_DIR"]) / "state.json", json.dumps(state, ensure_ascii=False, sort_keys=True))


def run_once(state, now):
    merge_days(state, aggregate(read_logs(ENV["LOG_DIR"])))
    refresh_github(state, now)
    write_public(state, now)
    write_atomic(Path(ENV["DATA_DIR"]) / "dashboard.html", dashboard(state, now))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--once", action="store_true")
    ap.add_argument("--digest-now", action="store_true")
    ap.add_argument("--print", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        return selftest()

    state = load_state()
    if a.once or a.print or a.digest_now:
        now = dt.datetime.now(TZ)
        run_once(state, now)
        yesterday = now.date() - dt.timedelta(days=1)
        if a.print:
            print(digest(state, yesterday))
        if a.digest_now:
            print("yuborildi" if send_digest(state, yesterday, True) else "yuborilmadi (BOT_TOKEN/ADMIN_CHAT_ID?)")
        save_state(state)
        return 0

    if not (ENV["BOT_TOKEN"] and ENV["ADMIN_CHAT_ID"]):
        log("BOT_TOKEN yoki ADMIN_CHAT_ID yo'q: Telegram xabarlari o'chiq, hisob yuritiladi")
    log(f"ishga tushdi: har {ENV['INTERVAL']} s, xulosa har kuni {ENV['DIGEST_HOUR']}:00 (Toshkent)")
    while True:
        now = dt.datetime.now(TZ)
        try:
            run_once(state, now)
            uptime(state)
            today = now.date().isoformat()
            if now.hour >= ENV["DIGEST_HOUR"] and state.get("digest_sent") != today:
                # dushanba — haftalik hisobot fayli ham
                if send_digest(state, now.date() - dt.timedelta(days=1), now.weekday() == 0) or not ENV["BOT_TOKEN"]:
                    state["digest_sent"] = today
            save_state(state)
        except Exception as e:  # tsikl to'xtamasin: xato logga, keyingi aylanishda qayta urinadi
            log(f"xato: {type(e).__name__}: {e}")
        time.sleep(ENV["INTERVAL"])


# ---------------------------------------------------------------- sinov

def selftest():
    base = dt.datetime(2026, 11, 12, 10, 0, tzinfo=TZ).timestamp()

    def line(uri, ua, ip="203.0.113.0", status=200, ref="", rng="", t=0, method="GET"):
        h = {"User-Agent": [ua]}
        if ref:
            h["Referer"] = [ref]
        if rng:
            h["Range"] = [rng]
        return json.dumps({"ts": base + t, "status": status, "size": 10,
                           "request": {"client_ip": ip, "method": method, "uri": uri, "headers": h}})

    win = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/130"
    mac = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) Safari/605"
    andr = "Mozilla/5.0 (Linux; Android 14) Mobile Chrome/130"
    apt = "Debian APT-HTTP/1.3 (2.7.14)"
    lines = [
        line("/", win, ref="https://www.google.com/search?q=bitg"),
        line("/yuklab-olish/", win),
        line("/yuklab-olish/rahmat/?os=win", win),
        line("/download/bitg-0.2.0-x64.msi", win),
        line("/download/bitg-0.2.0-x64.msi", win, t=5),                       # takror — sanalmaydi
        line("/download/bitg-0.2.0-x64.msi.sha256", win),                     # sha — sanalmaydi
        line("/download/bitg-0.2.0-macos.pkg", mac, ip="198.51.100.0", status=206, rng="bytes=0-"),
        line("/download/bitg-0.2.0-macos.pkg", mac, ip="198.51.100.0", status=206, rng="bytes=500-"),
        line("/", mac, ip="198.51.100.0", ref="https://t.me/bitguz"),
        line("/", andr, ip="192.0.2.0"),
        line("/", "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)", ip="66.249.66.0"),
        line("/", "TelegramBot (like TwitterBot)", ip="149.154.161.0"),
        line("/", win, method="HEAD"),
        line("/eski/", win, status=404),
        line("/download/bitg_0.2.0_amd64.deb", "Wget/1.21.4", ip="203.0.114.0"),
        line("/apt/dists/stable/InRelease", apt, ip="203.0.115.0"),
        line("/apt/dists/stable/InRelease", apt, ip="203.0.116.0", status=304),
        line("/apt/pool/main/b/bitg/bitg_0.2.0_amd64.deb", apt, ip="203.0.115.0"),
        line("/", win, t=86400),                                              # ertasi kun
        "not json", "{}", "[]",
    ]
    days = aggregate(lines)
    d = days["2026-11-12"]
    assert d["visitors"] == 3, d                     # win, mac, android — botlar emas
    assert d["pageviews"] == 5, d
    assert d["dl"] == {"win": 1, "mac": 1, "linux": 1}, d["dl"]
    assert d["dl_cli"] == 1, d
    assert d["intents"] == {"win": 1}, d
    assert d["os"] == {"win": 1, "mac": 1, "android": 1}, d["os"]
    assert d["refs"] == {"google.com": 1, "t.me": 1}, d["refs"]
    assert d["notfound"] == {"/eski/": 1}, d
    assert d["apt_clients"] == 2 and d["apt_pkg"] == {"linux": 1}, d
    assert days["2026-11-13"]["visitors"] == 1

    state = {}
    merge_days(state, days)
    merge_days(state, {"2026-11-12": {"visitors": 1}, "2026-11-13": {"visitors": 9}})  # eski kun qisman: tegilmaydi
    assert state["days"]["2026-11-12"]["visitors"] == 3 and state["days"]["2026-11-13"]["visitors"] == 9
    state["github"] = {"assets": {"bitg-0.2.0-x64.msi": 5, "bitg_0.2.0_amd64.deb": 2}}
    assert all_time_downloads(state) == (3, 7)
    text = digest(state, dt.date(2026, 11, 12))
    assert "Tashrif: <b>3</b>" in text and "Windows 1" in text and "Jami yuklab olish: <b>10</b>" in text, text
    page = dashboard(state, dt.datetime(2026, 11, 13, 9, 0, tzinfo=TZ))
    assert page.count("<svg") == 2 and "bitg.uz statistika" in page
    assert dl_kind("fcitx5-bitg_0.2.0_amd64.deb") == "fcitx" and dl_kind("x.sha256") is None
    print("selftest: OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
