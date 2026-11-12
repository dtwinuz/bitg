#!/usr/bin/env python3
"""bitg Telegram akkauntlarining profili kod sifatida: kanal nomi, tavsifi, rasmi, qadalgan posti; bot nomi va rasmi.

  python3 tools/telegram/setup.py                     # nima o'zgarishini ko'rsatadi, hech narsa yubormaydi
  python3 tools/telegram/setup.py --apply             # nom, tavsif, rasm (o'zgarganlarini)
  python3 tools/telegram/setup.py --apply --welcome   # + qadalgan "xush kelibsiz" posti (yo'q yoki matn o'zgargan bo'lsa)

Tokenlar: DEVBOT_TOKEN (yoki ~/.bitgdevbot-token) — kanallar va @bitgdevbot; BITGUZBOT_TOKEN (ixtiyoriy) — @bitguzbot
nomi va rasmi. Serverdagi tokenni ekranga chiqarmasdan olish:
  BITGUZBOT_TOKEN="$(ssh dtwin "sed -n 's/^BOT_TOKEN=//p' /opt/bitg-bot/.env")" python3 tools/telegram/setup.py --apply
@bitguzbot buyruqlari va tavsifi bot kodida (bot/src/main.rs, setup()) — bu yerda emas.
Rasm: web/logo-512.png, nuqtalar rangi akkauntga qarab (Pillow kerak, faqat rasm qo'yilganda).
"""
import argparse
import html
import io
import json
import os
import re
import sys
import urllib.error
import urllib.request
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LOGO = ROOT / "web" / "logo-512.png"
CONTACT = "@Xakimov_Nosirbek"

BLUE, GREEN, AMBER = (143, 176, 234), (124, 207, 165), (232, 179, 90)

CHANNELS = {
    # Kanal ovozi yangi alifboda (egasining qadalgan posti shunday). Qadalgan post kanalning o'z posti bo'lsa,
    # yangisi qo'yilmaydi — o'sha tahrirlanadi.
    "@bitguz": {
        "title": "bitg · yangi özbek alifbosi",
        "accent": BLUE,
        "description": (
            "bitg yozayotganingizda sözni yangi özbek alifbosiga (ö ğ ş ç) özi ögiradi — istalgan dasturda, "
            "internetsiz. Windows, Mac, Linux, bepul.\n"
            "⬇️ bitg.uz\n"
            "🤖 @bitguzbot · 🛠 @bitgtech\n"
            f"✉️ {CONTACT}"
        ),
        "parse_mode": None,
        "welcome": (
            "Eskiça yozing — yangiça çiqadi. ⌨️\n\n"
            "2027-yil 22-aprelda yangi özbek alifbosi kuçga kiradi: O‘ → Ö, G‘ → Ğ, Sh → Ş, Ch → Ç.\n\n"
            "Bu kanalda:\n"
            "• qonun va ötiş jadvali — faqat rasmiy faktlar\n"
            "• sözlar, ismlar, joy nomlari yangi alifboda\n"
            "• yangi harflarni kompyuter va telefonda yoziş\n"
            "• bitgʼning yangi versiyalari\n\n"
            "bitg — bepul dastur: odatdagidek yozasiz, söz tugaşi bilan yangi alifboga özi ögiradi. "
            "Inglizça söz, havola va kodga tegmaydi. Word, Telegram, brauzer — istalgan dasturda, internetsiz.\n\n"
            "💻 Windows · Mac · Linux: bitg.uz\n"
            "🤖 Telegramʼda ögiriş: @bitguzbot\n"
            "🐞 Xato yoki taklif: @bitguzbot → /fikr\n"
            "🛠 Texnik yangiliklar: @bitgtech\n\n"
            "#yangialifbo #bitg"
        ),
    },
    "@bitgtech": {
        "title": "bitg tech",
        "accent": GREEN,
        "parse_mode": "HTML",
        "description": (
            "bitg texnik kanali: har versiyaning to‘liq o‘zgarishlar ro‘yxati, reliz fayllari, SHA-256 va "
            "attestation. Ochiq kod: github.com/dtwinuz/bitg\n"
            "📣 @bitguz · 🐞 @bitguzbot → /fikr\n"
            f"✉️ {CONTACT}"
        ),
        "welcome": (
            "🛠 <b>bitg tech — texnik kanal</b>\n\n"
            "Har reliz chiqqanda bu yerga avtomatik post tushadi: CHANGELOG’dagi to‘liq o‘zgarishlar ro‘yxati "
            "va GitHub Release havolasi.\n\n"
            "📦 <b>Relizlar:</b> <a href=\"https://github.com/dtwinuz/bitg/releases\">github.com/dtwinuz/bitg/releases</a>\n"
            "🔐 <b>Faylni tekshirish:</b> SHA-256 relizda, kelib chiqishi:\n"
            "<code>gh attestation verify &lt;fayl&gt; -R dtwinuz/bitg</code>\n"
            "🍺 <b>Mac:</b> <code>brew install --cask dtwinuz/tap/bitg</code>\n"
            "🐧 <b>Linux:</b> imzolangan apt ombori — <a href=\"https://bitg.uz/yuklab-olish/#apt\">bitg.uz/apt</a>\n"
            "🧩 <b>Kod:</b> Rust, MIT / Apache-2.0 — <a href=\"https://github.com/dtwinuz/bitg\">github.com/dtwinuz/bitg</a>\n"
            "🐞 <b>Xato:</b> <a href=\"https://github.com/dtwinuz/bitg/issues/new/choose\">GitHub Issues</a> "
            "yoki @bitguzbot → /fikr\n\n"
            "Foydalanuvchilar uchun qisqa yangiliklar: @bitguz"
        ),
    },
}

BOTS = {
    "devbot": {
        "env": "DEVBOT_TOKEN",
        "name": "bitg dev",
        "accent": AMBER,
        "short_description": "bitg ishchi boti: statistika, uptime va reliz e'lonlari. Foydalanuvchilar uchun: @bitguzbot",
        "description": (
            "bitg jamoasining ichki boti: bitg.uz statistikasi, uptime ogohlantirishlari va @bitguz / @bitgtech "
            "ga reliz e'lonlari.\n\nMatnni o‘girish va fikr yozish uchun: @bitguzbot"
        ),
    },
    "bitguzbot": {
        "env": "BITGUZBOT_TOKEN",
        "name": "bitg — yangi alifbo",
        "accent": BLUE,
    },
}

LIMITS = {"title": 128, "description": 255, "name": 64, "short_description": 120, "bot_description": 512}
BOT_TEXT = {"short_description": ("getMyShortDescription", "setMyShortDescription"),
            "description": ("getMyDescription", "setMyDescription")}


class Api:
    def __init__(self, token):
        self.base = f"https://api.telegram.org/bot{token}"

    def __call__(self, method, fields=None, files=None):
        fields = fields or {}
        if files:
            b = uuid.uuid4().hex
            parts = [f'--{b}\r\nContent-Disposition: form-data; name="{k}"\r\n\r\n{v}\r\n'.encode()
                     for k, v in fields.items()]
            for k, (fname, data) in files.items():
                parts.append(f'--{b}\r\nContent-Disposition: form-data; name="{k}"; filename="{fname}"\r\n'
                             f'Content-Type: image/png\r\n\r\n'.encode() + data + b"\r\n")
            parts.append(f"--{b}--\r\n".encode())
            body, ctype = b"".join(parts), f"multipart/form-data; boundary={b}"
        else:
            body, ctype = json.dumps(fields).encode(), "application/json"
        req = urllib.request.Request(f"{self.base}/{method}", data=body, headers={"Content-Type": ctype})
        try:
            with urllib.request.urlopen(req, timeout=60) as r:
                return json.load(r)
        except urllib.error.HTTPError as e:
            try:
                return json.load(e)
            except ValueError:
                return {"ok": False, "description": f"HTTP {e.code}"}
        except (urllib.error.URLError, OSError) as e:
            # URL ichida token bor: xato matnida URL chiqarilmaydi
            return {"ok": False, "description": f"tarmoq xatosi ({type(e).__name__})"}


def avatar(accent) -> bytes:
    """logo-512.png dagi ko'k nuqtalarni boshqa rangga bo'yaydi (chekka yumshoqligi saqlanadi)."""
    from PIL import Image
    im = Image.open(LOGO).convert("RGB")
    bg = im.getpixel((8, 8))
    px = im.load()
    for y in range(im.height):
        for x in range(im.width):
            r, g, b = px[x, y]
            if b - r > 25:  # faqat ko'k tusli piksellar (nuqtalar va ularning chekkasi)
                t = max(0.0, min(1.0, (b - bg[2]) / (BLUE[2] - bg[2])))
                px[x, y] = tuple(round(bg[i] + t * (accent[i] - bg[i])) for i in range(3))
    out = io.BytesIO()
    im.save(out, "PNG", optimize=True)
    return out.getvalue()


def plain(s):
    return html.unescape(re.sub(r"<[^>]+>", "", s))


def check_limits():
    errs = []
    for chat, c in CHANNELS.items():
        for k in ("title", "description"):
            if len(c[k]) > LIMITS[k]:
                errs.append(f"{chat} {k}: {len(c[k])} > {LIMITS[k]}")
        if len(plain(c["welcome"])) > 4096:
            errs.append(f"{chat} welcome uzun")
    for n, b in BOTS.items():
        for k, lim in (("name", "name"), ("short_description", "short_description"), ("description", "bot_description")):
            if len(b.get(k, "")) > LIMITS[lim]:
                errs.append(f"{n} {k}: {len(b[k])} > {LIMITS[lim]}")
    if errs:
        raise SystemExit("chegaradan oshdi:\n  " + "\n  ".join(errs))


def ok(res, what):
    if res.get("ok") or "not modified" in str(res.get("description", "")):
        print(f"  ✓ {what}")
        return True
    print(f"  ✗ {what}: {res.get('description')}")
    return False


def setup_channel(api, chat, c, a):
    cur = api("getChat", {"chat_id": chat})
    if not cur.get("ok"):
        print(f"{chat}: {cur.get('description')}")
        return False
    cur = cur["result"]
    print(chat)
    good = True
    if cur.get("title") != c["title"]:
        print(f"  nom: {cur.get('title')!r} -> {c['title']!r}")
        if a.apply:
            good &= ok(api("setChatTitle", {"chat_id": chat, "title": c["title"]}), "nom")
    if (cur.get("description") or "") != c["description"]:
        print(f"  tavsif yangilanadi ({len(c['description'])} belgi)")
        if a.apply:
            good &= ok(api("setChatDescription", {"chat_id": chat, "description": c["description"]}), "tavsif")
    if not cur.get("photo") or a.photo:
        print("  rasm qo'yiladi")
        if a.apply:
            good &= ok(api("setChatPhoto", {"chat_id": chat}, {"photo": ("bitg.png", avatar(c["accent"]))}), "rasm")
    pinned = cur.get("pinned_message") or {}
    own = (pinned.get("sender_chat") or {}).get("id") == cur["id"]
    text = c["welcome"]
    shown = plain(text) if c["parse_mode"] else text
    if shown.strip() != (pinned.get("text") or "").strip():
        act = "tahrirlanadi" if own else "yuboriladi va qadaladi"
        print(f"  qadalgan post: {'yo' + chr(39) + 'q' if not pinned else 'matn farq qiladi'} -> {act}"
              + ("" if a.welcome else " (--welcome)"))
        if a.apply and a.welcome:
            body = {"chat_id": chat, "text": text, "link_preview_options": {"is_disabled": True}}
            if c["parse_mode"]:
                body["parse_mode"] = c["parse_mode"]
            if own:
                good &= ok(api("editMessageText", {**body, "message_id": pinned["message_id"]}), "post tahrirlandi")
            else:
                m = api("sendMessage", body)
                if ok(m, "xush kelibsiz posti"):
                    good &= ok(api("pinChatMessage", {"chat_id": chat, "message_id": m["result"]["message_id"],
                                                      "disable_notification": True}), "qadaldi")
                else:
                    good = False
    return good


def setup_bot(api, b, a):
    me = api("getMe")
    if not me.get("ok"):
        print(f"bot: token ishlamadi ({me.get('description')})")
        return False
    print(f"@{me['result']['username']}")
    good = True
    cur = api("getMyName").get("result", {}).get("name")
    if cur != b["name"]:
        print(f"  nom: {cur!r} -> {b['name']!r}")
        if a.apply:
            good &= ok(api("setMyName", {"name": b["name"]}), "nom")
    for k, (getter, setter) in BOT_TEXT.items():
        if k in b and api(getter).get("result", {}).get(k) != b[k]:
            print(f"  {k} yangilanadi")
            if a.apply:
                good &= ok(api(setter, {k: b[k]}), k)
    photos = api("getUserProfilePhotos", {"user_id": me["result"]["id"], "limit": 1}).get("result", {})
    if not photos.get("total_count") or a.photo:
        print("  rasm qo'yiladi")
        if a.apply:
            res = api("setMyProfilePhoto", {"photo": json.dumps({"type": "static", "photo": "attach://p"})},
                      {"p": ("bitg.png", avatar(b["accent"]))})
            if not ok(res, "rasm"):
                print("    -> BotFather: /setuserpic (rasm: --export-avatars DIR)")
    return good


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--welcome", action="store_true", help="qadalgan postni yuborish (ommaviy kanalga yoziladi)")
    ap.add_argument("--photo", action="store_true", help="rasm bor bo'lsa ham qayta qo'yish")
    ap.add_argument("--check", action="store_true", help="faqat matn chegaralarini tekshirish (CI)")
    ap.add_argument("--export-avatars", metavar="DIR", help="rasmlarni PNG qilib saqlash")
    a = ap.parse_args()
    check_limits()
    if a.check:
        print("telegram setup: chegaralar OK")
        return 0
    if a.export_avatars:
        d = Path(a.export_avatars)
        d.mkdir(parents=True, exist_ok=True)
        for name, acc in [("bitguz", BLUE), ("bitgtech", GREEN), ("bitgdevbot", AMBER)]:
            (d / f"{name}.png").write_bytes(avatar(acc))
        print(f"rasmlar: {d}")
        return 0
    dev = os.environ.get("DEVBOT_TOKEN", "").strip()
    f = Path.home() / ".bitgdevbot-token"
    if not dev and f.exists():
        dev = f.read_text().strip()
    if not dev:
        raise SystemExit("DEVBOT_TOKEN yo'q (yoki ~/.bitgdevbot-token)")
    good = True
    devapi = Api(dev)
    for chat, c in CHANNELS.items():
        good &= setup_channel(devapi, chat, c, a)
    for key, b in BOTS.items():
        tok = dev if key == "devbot" else os.environ.get(b["env"], "").strip()
        if not tok:
            print(f"{key}: {b['env']} berilmagan, o'tkazildi")
            continue
        good &= setup_bot(Api(tok), b, a)
    if not a.apply:
        print("\n(sinov: hech narsa yuborilmadi; qo'llash uchun --apply)")
    return 0 if good else 1


if __name__ == "__main__":
    sys.exit(main())
