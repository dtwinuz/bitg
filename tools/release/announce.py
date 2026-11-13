#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Yangi versiyani Telegram kanallarga e'lon qiladi (@bitgdevbot orqali).

  python3 tools/release/announce.py 0.3.0 --dry-run     # xabarlarni chiqaradi, hech narsa yubormaydi
  TG_TOKEN=... python3 tools/release/announce.py 0.3.0  # @bitguz: qisqa post, @bitgtech: to'liq ro'yxat

@bitgtech matni CHANGELOG.md dagi "## [<versiya>]" bo'limidan, @bitguz qisqa posti NEWS.md dagi "## <versiya>"
bandlaridan (foydalanuvchi tili; NEWS.md da versiya bo'lmasa CHANGELOG'dan). Bot ikkala kanalda admin bo'lishi kerak.
Muhit: TG_TOKEN, NEWS_CHANNEL (standart @bitguz), TECH_CHANNEL (standart @bitgtech),
BITG_CLI (bitg binari: @bitguz posti yangi alifboga o'giriladi; bo'lmasa eski lotinda).
Token URL ichida bo'ladi: xato matnida URL chiqarilmaydi.
"""
import argparse
import html
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPO = "https://github.com/dtwinuz/bitg"
SITE = "https://bitg.uz"
LIMIT = 4096          # Telegram xabar chegarasi (ko'rinadigan belgilar)
SHORT_ITEMS = 4


def news_items(news: str, ver: str):
    """NEWS.md "## <versiya> — sana" bo'limidagi bandlar; yo'q bo'lsa []."""
    m = re.search(rf"^## {re.escape(ver)}\b[^\n]*\n(.*?)(?=^## |\Z)", news, re.S | re.M)
    return [l[2:].strip() for l in m.group(1).splitlines() if l.startswith("- ")] if m else []


def section(changelog: str, ver: str):
    """(kirish matni, [(sarlavha, [band, ...])]) — davomi 2 bo'shliq bilan yozilgan qatorlar bandga qo'shiladi."""
    m = re.search(rf"^## \[{re.escape(ver)}\][^\n]*\n(.*?)(?=^## |^\[[^\]]+\]: |\Z)", changelog, re.S | re.M)
    if not m:
        raise SystemExit(f"CHANGELOG.md da [{ver}] bo'limi yo'q")
    groups, intro = [], []
    for line in m.group(1).splitlines():
        if line.startswith("### "):
            groups.append((line[4:].strip(), []))
        elif line.startswith("- "):
            if not groups:
                groups.append(("", []))
            groups[-1][1].append(line[2:].strip())
        elif line.startswith("  ") and groups and groups[-1][1]:
            groups[-1][1][-1] += " " + line.strip()
        elif line.strip() and not groups:
            intro.append(line.strip())
    return " ".join(intro), [(t, items) for t, items in groups if items]


def md(s: str) -> str:
    """CHANGELOG markdown'ining kichik qismi -> Telegram HTML."""
    out, pos = [], 0
    for m in re.finditer(r"\*\*(.+?)\*\*|`([^`]+)`|\[([^\]]+)\]\(([^)\s]+)\)", s):
        out.append(html.escape(s[pos:m.start()], quote=False))
        if m.group(1) is not None:
            out.append(f"<b>{md(m.group(1))}</b>")
        elif m.group(2) is not None:
            out.append(f"<code>{html.escape(m.group(2), quote=False)}</code>")
        else:
            href = m.group(4)
            if not href.startswith(("https://", "http://")):
                # repo ichidagi nisbiy havola (NEWS.md) Telegram'da ishlamaydi -> GitHub'dagi fayl
                href = f"{REPO}/blob/main/{href.lstrip('./')}"
            out.append(f'<a href="{html.escape(href)}">{html.escape(m.group(3), quote=False)}</a>')
        pos = m.end()
    out.append(html.escape(s[pos:], quote=False))
    return "".join(out)


def plain(s: str) -> str:
    s = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", s)
    return s.replace("**", "").replace("`", "")


def headline(item: str) -> str:
    """Qisqa post uchun: qalin qism bo'lsa o'sha, bo'lmasa birinchi gap (140 belgigacha)."""
    b = re.match(r"\*\*(.+?)\*\*", item)
    text = plain(b.group(1) if b else item)
    if not b:
        text = re.split(r"(?<=[.!?])\s", text, maxsplit=1)[0]
    text = text.rstrip(":. ")
    return text if len(text) <= 140 else text[:137].rstrip() + "…"


def short_post(ver, groups, tech, news=()):
    items = list(news) or [i for t, its in groups if not t.lower().startswith("sifat") for i in its]
    lines = [f"🆕 <b>bitg {html.escape(ver)}</b> chiqdi", ""]
    lines += [f"• {html.escape(headline(i), quote=False)}" for i in items[:SHORT_ITEMS]]
    if len(items) > SHORT_ITEMS:
        lines.append(f"• va yana {len(items) - SHORT_ITEMS} ta o‘zgarish")
    lines += ["", f"⬇️ Yuklab olish: {SITE}/yuklab-olish/",
              f"📋 To‘liq ro‘yxat: {html.escape(tech)}"]
    return "\n".join(lines)


def full_post(ver, groups, intro=""):
    url = f"{REPO}/releases/tag/v{ver}"
    head = f'📦 <b>bitg {html.escape(ver)}</b> — <a href="{url}">GitHub Release</a>\n'
    if intro:
        head += md(intro) + "\n"
    tail = f"\n\nFayllar, SHA-256 va attestation: {url}"
    body = []
    for title, items in groups:
        if title:
            body.append(f"\n<b>{html.escape(title, quote=False)}</b>")
        body += [f"• {md(i)}" for i in items]
    text = head + "\n".join(body) + tail
    if visible_len(text) <= LIMIT:
        return text
    # sig'masa: oxirgi bandlardan qisqartirib, to'liq matnga havola qoldiriladi
    note = f"\n…\n\nTo‘liq ro‘yxat: {url}"
    while body and visible_len(head + "\n".join(body) + note) > LIMIT:
        body.pop()
    return head + "\n".join(body) + note


def to_new(text: str, cli: str) -> str:
    """@bitguz kanali yangi alifboda yuritiladi: qisqa post bitg'ning o'zi bilan o'giriladi
    (HTML teg, havola va @username'ga tegmaydi). CLI ishlamasa matn o'zgarishsiz qoladi."""
    try:
        r = subprocess.run([cli], input=text.encode(), capture_output=True, timeout=30, check=True)
    except (OSError, subprocess.SubprocessError) as e:
        print(f"bitg CLI ishlamadi ({type(e).__name__}): qisqa post eski lotinda", file=sys.stderr)
        return text
    out = r.stdout.decode("utf-8").rstrip("\n")
    return out if out.count("<") == text.count("<") else text


def visible_len(s: str) -> int:
    return len(html.unescape(re.sub(r"<[^>]+>", "", s)))


def send(token, chat, text):
    req = urllib.request.Request(
        f"https://api.telegram.org/bot{token}/sendMessage",
        data=json.dumps({"chat_id": chat, "text": text, "parse_mode": "HTML",
                         "link_preview_options": {"is_disabled": True}}).encode(),
        headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            res = json.load(r)
    except urllib.error.HTTPError as e:
        try:
            desc = json.load(e).get("description", "")
        except ValueError:
            desc = ""
        raise SystemExit(f"{chat}: HTTP {e.code} {desc}") from None
    except (urllib.error.URLError, OSError) as e:
        raise SystemExit(f"{chat}: tarmoq xatosi ({type(e).__name__})") from None
    if not res.get("ok"):
        raise SystemExit(f"{chat}: {res.get('description')}")
    return res["result"]["message_id"]


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("version", help="masalan 0.3.0 yoki v0.3.0")
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args()
    ver = a.version.lstrip("v")
    news = os.environ.get("NEWS_CHANNEL") or "@bitguz"
    tech = os.environ.get("TECH_CHANNEL") or "@bitgtech"
    intro, groups = section((ROOT / "CHANGELOG.md").read_text(encoding="utf-8"), ver)
    news_md = ROOT / "NEWS.md"
    items = news_items(news_md.read_text(encoding="utf-8"), ver) if news_md.exists() else []
    short = short_post(ver, groups, tech, items)
    if os.environ.get("BITG_CLI"):
        short = to_new(short, os.environ["BITG_CLI"])
    posts = [(news, short), (tech, full_post(ver, groups, intro))]
    for chat, text in posts:
        if visible_len(text) > LIMIT:
            raise SystemExit(f"{chat}: xabar {visible_len(text)} belgi, chegara {LIMIT}")
    if a.dry_run:
        for chat, text in posts:
            print(f"===== {chat} ({visible_len(text)} belgi)\n{text}\n")
        return 0
    token = os.environ.get("TG_TOKEN", "").strip()
    if not token:
        print("TG_TOKEN yo'q: e'lon yuborilmadi")
        return 0
    for chat, text in posts:
        print(f"{chat}: yuborildi (message_id {send(token, chat, text)})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
