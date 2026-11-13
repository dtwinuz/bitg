#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

"""bitg tarqatish kanallari so'nggi relizga yetib kelganmi: har kanal o'z ochiq manbasidan o'qiladi.

  python tools/release/channels.py                     # jadval stdout'ga
  python tools/release/channels.py --report r.md --json r.json --github-output "$GITHUB_OUTPUT"
  python tools/release/channels.py --selftest

Har kanal holati:
  ok       — so'nggi reliz (yoki undan yangi) kanalda bor
  waiting  — hali yo'q, lekin relizdan beri kanal muddati (soatda) o'tmagan
  lag      — muddat o'tdi, kanal ortda: tekshirish kerak
  pending  — kanal hali ochilmagan (birinchi paket ko'rib chiqilmoqda yoki akkaunt yo'q): xato emas
  n/a      — bu reliz kanalga tegishli emas (masalan, pacman ombori 0.4.1 dan)
  error    — manba javob bermadi yoki formati kutilmagan

lag va error `problems` soniga kiradi; channels.yml shu bo'yicha bitta issue ochadi, yangilaydi yoki yopadi.
Token ixtiyoriy (GH_TOKEN / GITHUB_TOKEN): GitHub API limiti uchun.
"""
import argparse
import gzip
import io
import json
import os
import re
import sys
import tarfile
import urllib.error
import urllib.request
from datetime import datetime, timezone

REPO = "dtwinuz/bitg"
UA = "bitg-channels/1 (+https://github.com/dtwinuz/bitg)"
RAW = "https://raw.githubusercontent.com"


def fetch(url, accept=None, token=False):
    h = {"User-Agent": UA}
    if accept:
        h["Accept"] = accept
    tok = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token and tok:
        h["Authorization"] = f"Bearer {tok}"
    with urllib.request.urlopen(urllib.request.Request(url, headers=h), timeout=30) as r:
        return r.read()


class Missing(Exception):
    """Kanalda bitg umuman yo'q (404 yoki bo'sh natija)."""


def get(url, **kw):
    try:
        return fetch(url, **kw)
    except urllib.error.HTTPError as e:
        if e.code == 404:
            raise Missing(url) from e
        raise


def gh_api(path):
    return json.loads(get(f"https://api.github.com/{path}", accept="application/vnd.github+json", token=True))


def vkey(v):
    """'0.4.1-1' (pacman/AUR pkgrel) -> (0, 4, 1): taqqoslash faqat yuqori oqim versiyasi bo'yicha."""
    base = re.split(r"[-+~]", v.strip(), maxsplit=1)[0]
    try:
        return tuple(int(x) for x in base.split("."))
    except ValueError:
        return ()


def newest(vs, what):
    if not vs:
        raise Missing(what)
    return max(vs, key=vkey)


# --- o'quvchilar: javob matni -> versiya (tarmoqdan alohida, selftest shularni sinaydi) ---

def parse_cask(text):
    m = re.search(r'^\s*version "([^"]+)"', text, re.M)
    if not m:
        raise ValueError("cask'da version yo'q")
    return m.group(1)


def parse_scoop(data):
    return json.loads(data)["version"]


def parse_choco(xml):
    return newest(re.findall(r"<d:Version>([^<]+)</d:Version>", xml), "chocolatey")


def parse_apt(text, pkg="bitg"):
    vs = []
    for block in text.split("\n\n"):
        f = dict(re.findall(r"^([A-Za-z-]+): (.*)$", block, re.M))
        if f.get("Package") == pkg and "Version" in f:
            vs.append(f["Version"])
    return newest(vs, "apt")


def parse_pacman_db(data, pkg="bitg"):
    vs = []
    with tarfile.open(fileobj=io.BytesIO(gzip.decompress(data))) as t:
        for m in t.getmembers():
            if m.isfile() and m.name.endswith("/desc"):
                sec = dict(re.findall(r"%([A-Z0-9]+)%\n([^\n]*)", t.extractfile(m).read().decode("utf-8")))
                if sec.get("NAME") == pkg:
                    vs.append(sec.get("VERSION", ""))
    return newest(vs, "pacman")


def parse_aur(data):
    r = json.loads(data)
    if not r.get("resultcount"):
        raise Missing("aur")
    return r["results"][0]["Version"]


def parse_winget(entries):
    return newest([e["name"] for e in entries if e.get("type") == "dir" and re.fullmatch(r"\d+(\.\d+)+", e["name"])],
                  "winget")


def parse_crate(data):
    return json.loads(data)["crate"]["max_stable_version"]


def parse_stable(data):
    return json.loads(data)["version"]


# (kalit, nom, muddat soatda, topilmasa pending, shu versiyadan boshlab, o'quvchi, havola)
CHANNELS = [
    ("site", "bitg.uz yangilanish (update/stable.json)", 48, False, None,
     lambda: parse_stable(get("https://bitg.uz/update/stable.json")), "https://bitg.uz/update/stable.json"),
    ("apt", "apt (bitg.uz/apt)", 48, False, None,
     lambda: parse_apt(get("https://bitg.uz/apt/dists/stable/main/binary-amd64/Packages").decode("utf-8")),
     "https://bitg.uz/apt/"),
    ("pacman", "pacman (bitg.uz/arch)", 48, False, "0.4.1",
     lambda: parse_pacman_db(get("https://bitg.uz/arch/x86_64/bitg.db")), "https://bitg.uz/arch/"),
    ("homebrew", "Homebrew (dtwinuz/tap)", 12, False, None,
     lambda: parse_cask(get(f"{RAW}/dtwinuz/homebrew-tap/main/Casks/bitg.rb").decode("utf-8")),
     "https://github.com/dtwinuz/homebrew-tap"),
    ("scoop", "Scoop (dtwinuz/scoop-bucket)", 12, False, None,
     lambda: parse_scoop(get(f"{RAW}/dtwinuz/scoop-bucket/main/bucket/bitg.json")),
     "https://github.com/dtwinuz/scoop-bucket"),
    ("crates", "crates.io (bitg-core)", 12, False, None,
     lambda: parse_crate(get("https://crates.io/api/v1/crates/bitg-core")), "https://crates.io/crates/bitg-core"),
    # har versiya qo'lda moderatsiyadan o'tadi; FindPackagesById faqat tasdiqlanganlarini qaytaradi
    ("chocolatey", "Chocolatey (moderatsiya)", 240, True, None,
     lambda: parse_choco(get("https://community.chocolatey.org/api/v2/FindPackagesById()?id='bitg'").decode("utf-8")),
     "https://community.chocolatey.org/packages/bitg"),
    ("winget", "winget (microsoft/winget-pkgs)", 240, True, None,
     lambda: parse_winget(gh_api("repos/microsoft/winget-pkgs/contents/manifests/d/dtwinuz/bitg")),
     "https://github.com/microsoft/winget-pkgs/tree/master/manifests/d/dtwinuz/bitg"),
    # AUR'da ro'yxatdan o'tish yopiq: paket yo'qligi hozircha kutilgan
    ("aur", "AUR (bitg)", 48, True, None,
     lambda: parse_aur(get("https://aur.archlinux.org/rpc/v5/info?arg[]=bitg")), "https://aur.archlinux.org/packages/bitg"),
]


def judge(latest, age_h, found, tolerance, onboarding, since):
    """Kanaldagi versiya (None — kanalda bitg yo'q) -> holat."""
    if since and vkey(latest) < vkey(since):
        return "ok" if found is not None and vkey(found) >= vkey(latest) else "n/a"
    if found is not None and vkey(found) >= vkey(latest):
        return "ok"
    if found is None and onboarding:
        return "pending"
    return "waiting" if age_h <= tolerance else "lag"


def run(channels, latest, published):
    age_h = (datetime.now(timezone.utc) - published).total_seconds() / 3600
    rows = []
    for key, name, tol, onboarding, since, read, link in channels:
        row = {"key": key, "name": name, "found": None, "tolerance_h": tol, "detail": "", "link": link}
        try:
            row["found"] = read()
            row["status"] = judge(latest, age_h, row["found"], tol, onboarding, since)
        except Missing:
            row["status"] = judge(latest, age_h, None, tol, onboarding, since)
        except Exception as e:  # noqa: BLE001 — bitta kanal xatosi qolganlarini to'xtatmaydi, hisobotga yoziladi
            row["status"], row["detail"] = "error", f"{type(e).__name__}: {e}"[:200]
        rows.append(row)
    return age_h, rows


ICON = {"ok": "✅", "waiting": "⏳", "lag": "🔴", "pending": "🕓", "n/a": "➖", "error": "⚠️"}


def report(latest, age_h, rows):
    bad = [r for r in rows if r["status"] in ("lag", "error")]
    head = (f"So'nggi reliz: **v{latest}** ({age_h:.0f} soat oldin) — "
            + ("hamma kanal joyida." if not bad else f"**{len(bad)} kanal muammoli.**"))
    lines = [head, "", "| Kanal | Holat | Kanaldagi versiya | Muddat | Izoh |", "|---|---|---|---|---|"]
    for r in rows:
        lines.append(f"| [{r['name']}]({r['link']}) | {ICON[r['status']]} {r['status']} | {r['found'] or '—'} "
                     f"| {r['tolerance_h']} soat | {r['detail'].replace('|', '/')} |")
    lines += ["", "ok — joyida; waiting — muddat ichida; lag — muddat o'tdi; pending — kanal hali ochilmagan; "
                  "n/a — bu reliz uchun emas; error — manba javob bermadi.",
              "", "Manba: `tools/release/channels.py` (`channels` workflow, har kuni)."]
    return "\n".join(lines) + "\n", len(bad)


def selftest():
    def missing(fn, *args):
        try:
            fn(*args)
        except Missing:
            return True
        return False

    assert parse_cask('cask "bitg" do\n  version "0.4.0"\n  sha256 "x"\n') == "0.4.0"
    assert parse_scoop(b'{"version": "0.4.1", "url": "x"}') == "0.4.1"
    xml = ("<feed><entry><m:properties><d:Version>0.4.0</d:Version></m:properties></entry>"
           "<entry><m:properties><d:Version>0.10.0</d:Version></m:properties></entry></feed>")
    assert parse_choco(xml) == "0.10.0" and missing(parse_choco, "<feed></feed>")
    pk = ("Package: fcitx5-bitg\nVersion: 0.9.0\n\nPackage: bitg\nVersion: 0.4.0\nArchitecture: amd64\n\n"
          "Package: bitg\nVersion: 0.4.1\n")
    assert parse_apt(pk) == "0.4.1" and missing(parse_apt, "Package: fcitx5-bitg\nVersion: 1.0.0\n")
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w") as t:
        for name, ver in (("bitg", "0.4.1-1"), ("fcitx5-bitg", "0.5.0-1")):
            d = f"%FILENAME%\n{name}-{ver}-x86_64.pkg.tar.zst\n\n%NAME%\n{name}\n\n%VERSION%\n{ver}\n\n".encode()
            ti = tarfile.TarInfo(f"{name}-{ver}/desc")
            ti.size = len(d)
            t.addfile(ti, io.BytesIO(d))
    assert parse_pacman_db(gzip.compress(buf.getvalue())) == "0.4.1-1"
    assert parse_aur(b'{"resultcount":1,"results":[{"Version":"0.4.0-2"}]}') == "0.4.0-2"
    assert missing(parse_aur, b'{"resultcount":0,"results":[]}')
    assert parse_winget([{"name": "0.3.1", "type": "dir"}, {"name": "0.4.0", "type": "dir"},
                         {"name": "README.md", "type": "file"}]) == "0.4.0"
    assert parse_crate(b'{"crate":{"max_stable_version":"0.4.0"}}') == "0.4.0"
    assert vkey("0.4.1-1") == (0, 4, 1) and vkey("0.10.0") > vkey("0.9.9") and vkey("x") == ()

    # judge(latest, yosh soatda, kanalda, muddat, topilmasa pending, since)
    assert judge("0.4.1", 1, "0.4.1-1", 48, False, None) == "ok"
    assert judge("0.4.1", 1, "0.5.0", 48, False, None) == "ok"
    assert judge("0.4.1", 1, "0.4.0", 12, False, None) == "waiting"
    assert judge("0.4.1", 13, "0.4.0", 12, False, None) == "lag"
    assert judge("0.4.1", 13, None, 12, False, None) == "lag"
    assert judge("0.4.1", 500, None, 240, True, None) == "pending"
    assert judge("0.4.1", 500, "0.4.0", 240, True, None) == "lag"  # ochilgan kanal ortda qolsa — lag
    assert judge("0.4.0", 100, None, 48, False, "0.4.1") == "n/a"
    assert judge("0.4.1", 100, None, 48, False, "0.4.1") == "lag"

    def boom():
        raise OSError("tarmoq")

    def gone():
        raise Missing("x")

    fake = [("a", "A", 12, False, None, lambda: "0.4.1", "l"), ("b", "B", 12, False, None, boom, "l"),
            ("c", "C", 12, True, None, gone, "l"), ("d", "D", 12, False, None, gone, "l")]
    _, rows = run(fake, "0.4.1", datetime.now(timezone.utc))
    assert [r["status"] for r in rows] == ["ok", "error", "pending", "waiting"], rows
    text, bad = report("0.4.1", 0, rows)
    assert bad == 1 and "| [B](l) | ⚠️ error |" in text and "OSError: tarmoq" in text, text
    print("channels selftest: ok")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--report", help="markdown hisobot fayli")
    ap.add_argument("--json", help="natija JSON fayli")
    ap.add_argument("--github-output", help="$GITHUB_OUTPUT: problems=<n>, latest=<v>")
    a = ap.parse_args()
    if a.selftest:
        return selftest()

    rel = gh_api(f"repos/{REPO}/releases/latest")
    tag = rel["tag_name"]
    if not re.fullmatch(r"v\d+\.\d+\.\d+", tag):
        sys.exit(f"channels: kutilmagan teg {tag}")
    latest = tag[1:]
    published = datetime.fromisoformat(rel["published_at"].replace("Z", "+00:00"))
    age_h, rows = run(CHANNELS, latest, published)
    text, bad = report(latest, age_h, rows)
    sys.stdout.buffer.write(text.encode("utf-8"))
    if a.report:
        with open(a.report, "w", encoding="utf-8") as f:
            f.write(text)
    if a.json:
        with open(a.json, "w", encoding="utf-8") as f:
            json.dump({"latest": latest, "published": rel["published_at"], "channels": rows}, f,
                      ensure_ascii=False, indent=1)
    if a.github_output:
        with open(a.github_output, "a", encoding="utf-8") as f:
            f.write(f"problems={bad}\nlatest={latest}\n")


if __name__ == "__main__":
    main()
