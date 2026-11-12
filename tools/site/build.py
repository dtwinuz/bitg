#!/usr/bin/env python3
"""bitg.uz statik sayt yig'uvchisi.

web/            qo'lda yozilgan bosh sahifa va umumiy fayllar (styles.css, app.js, ...)
web/pages/      ichki sahifalar: boshida JSON meta (---json ... ---), keyin HTML tana
natija          --out papkasi: har sahifa <slug>/index.html, sitemap.xml, 404.html

Ishlatish:
  python tools/site/build.py --out target/site-preview [--v <sha8>] [--msi dist/bitg-0.1.0-x64.msi]
Bog'liqlik yo'q (faqat standart kutubxona).
"""
import argparse
import datetime as dt
import hashlib
import html
import json
import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WEB = ROOT / "web"
PAGES = WEB / "pages"

SITE = "https://bitg.uz"
# Qonun: https://lex.uz/uz/docs/8548754 (4-modda: rasmiy e'lon qilingandan 6 oy o'tib kuchga kiradi)
CONFIG = {
    "lawNo": "O‘RQ-1184",
    "lawNoRu": "ЗРУ-1184",
    "signedDate": "2026-10-21",
    "lawDate": "2027-04-22",
}
COPY_EXT = {".html", ".css", ".js", ".svg", ".png", ".ico", ".txt", ".xml", ".woff2", ".webmanifest"}
SKIP_TOP = {"pages", "download"}
LANG_HTML = {"uz": "uz", "ru": "ru", "en": "en"}
HOME = {"uz": "/", "ru": "/ru/", "en": "/en/"}
OG_LOCALE = {"uz": "uz_UZ", "ru": "ru_RU", "en": "en_US"}

UZ_MONTHS = ["yanvar", "fevral", "mart", "aprel", "may", "iyun", "iyul", "avgust",
             "sentabr", "oktabr", "noyabr", "dekabr"]
RU_MONTHS = ["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа",
             "сентября", "октября", "ноября", "декабря"]
EN_MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August",
             "September", "October", "November", "December"]


def fail(msg):
    print(f"build: {msg}", file=sys.stderr)
    sys.exit(1)


def human_date(iso, lang):
    d = dt.date.fromisoformat(iso)
    if lang == "ru":
        return f"{d.day} {RU_MONTHS[d.month - 1]} {d.year} г."
    if lang == "en":
        return f"{EN_MONTHS[d.month - 1]} {d.day}, {d.year}"
    return f"{d.year}-yil {d.day}-{UZ_MONTHS[d.month - 1]}"


UI = {
    "uz": {
        "skip": "Asosiy matnga o‘tish", "home": "bosh sahifa", "nav": "Bo‘limlar",
        "abc": "Sahifa alifbosi", "new": "Yangi", "old": "Eski", "theme": "Mavzuni almashtirish",
        "gh": "GitHub'dagi manba kodi", "dl": "Yuklab olish", "crumbs": "Yo‘l",
        "updated": "Yangilangan", "sources": "Manbalar", "related": "Bog‘liq sahifalar", "langs": "Til",
        "nav_items": [("/konvertor/", "Konvertor"), ("/yangi-alifbo/", "Yangi alifbo"),
                      ("/klaviatura/", "Klaviatura"), ("/savollar/", "Savollar")],
        "foot_cols": [
            ("Dastur", [("/yuklab-olish/", "Yuklab olish"), ("/klaviatura/windows/", "Windows klaviaturasi"),
                        ("/qiyoslash/", "Vositalarni taqqoslash"), ("https://github.com/dtwinuz/bitg", "Manba kodi")]),
            ("Alifbo", [("/yangi-alifbo/", "Yangi o‘zbek alifbosi"), ("/yangi-alifbo-qonuni/", "Alifbo qonuni"),
                        ("/yangi-harflarni-yozish/", "Ş, Ç, Ğ, Ö yozish"), ("/konvertor/", "Onlayn konvertor")]),
            ("Loyiha", [("/haqida/", "bitg haqida"), ("/maxfiylik/", "Maxfiylik"), ("/savollar/", "Savol-javob"),
                        ("https://t.me/bitguz", "Telegram kanal")]),
            ("Til", [("/ru/", "Русский"), ("/en/", "English")]),
        ],
        "foot_lic": "MIT yoki Apache-2.0 · ochiq kod",
    },
    "ru": {
        "skip": "Перейти к основному тексту", "home": "главная", "nav": "Разделы",
        "theme": "Сменить тему", "gh": "Исходный код на GitHub", "dl": "Скачать", "crumbs": "Навигация",
        "updated": "Обновлено", "sources": "Источники", "related": "Связанные страницы", "langs": "Язык",
        "nav_items": [("/ru/novyy-uzbekskiy-alfavit/", "Новый алфавит"), ("/konvertor/", "Конвертер")],
        "foot_cols": [
            ("bitg", [("/ru/", "О программе"), ("/yuklab-olish/", "Скачать для Windows"),
                      ("https://github.com/dtwinuz/bitg", "Исходный код")]),
            ("Алфавит", [("/ru/novyy-uzbekskiy-alfavit/", "Новый узбекский алфавит"),
                         ("/konvertor/", "Онлайн-конвертер")]),
            ("Язык", [("/", "Oʻzbekcha"), ("/en/", "English"), ("https://t.me/bitguz", "Telegram")]),
        ],
        "foot_lic": "MIT или Apache-2.0 · открытый код",
    },
    "en": {
        "skip": "Skip to main content", "home": "home", "nav": "Sections",
        "theme": "Toggle theme", "gh": "Source code on GitHub", "dl": "Download", "crumbs": "Breadcrumb",
        "updated": "Updated", "sources": "Sources", "related": "Related pages", "langs": "Language",
        "nav_items": [("/konvertor/", "Converter")],
        "foot_cols": [
            ("bitg", [("/en/", "About bitg"), ("/yuklab-olish/", "Download for Windows"),
                      ("https://github.com/dtwinuz/bitg", "Source code")]),
            ("Language", [("/", "Oʻzbekcha"), ("/ru/", "Русский"), ("https://t.me/bitguz", "Telegram")]),
        ],
        "foot_lic": "MIT or Apache-2.0 · open source",
    },
}

BRAND_SVG = ('<svg class="bg-brand__mark" viewBox="0 0 24 24" aria-hidden="true"><path class="bg-m1" '
             'd="M6.95 4.5L6.95 19.5" pathLength="1"/><path class="bg-m2" d="M7.65 14.8a4.7 4.7 0 1 1 9.4 0a4.7 '
             '4.7 0 1 1 -9.4 0z" pathLength="1"/><circle class="bg-m3" cx="11.15" cy="5.8" r="1.6"/><circle '
             'class="bg-m4" cx="15.15" cy="5.8" r="1.6"/></svg>')
ICON_THEME = ('<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4.2"/><path d="M12 2.5v2.2M12 '
              '19.3v2.2M2.5 12h2.2M19.3 12h2.2M5.3 5.3l1.6 1.6M17.1 17.1l1.6 1.6M5.3 18.7l1.6-1.6M17.1 6.9l1.6-1.6"/></svg>')
ICON_GH = ('<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8.5 19c-4 1.2-4-2-5.6-2.4M14.5 21v-3.4c0-1 '
           '.1-1.4-.5-2 2.8-.3 5.5-1.4 5.5-6a4.7 4.7 0 0 0-1.3-3.2 4.3 4.3 0 0 0-.1-3.2s-1.1-.3-3.5 1.3a12 12 0 0 '
           '0-6.2 0C6 2.9 5 3.2 5 3.2a4.3 4.3 0 0 0-.1 3.2A4.7 4.7 0 0 0 3.5 9.6c0 4.6 2.7 5.7 5.5 6-.6.6-.6 1.2-.5 '
           '2V21"/></svg>')
ICON_MENU = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 7h16M4 12h16M4 17h16"/></svg>'


def read_page(path):
    raw = path.read_text(encoding="utf-8").replace("\r\n", "\n")
    m = re.match(r"---json\n(.*?)\n---\n(.*)\Z", raw, re.S)
    if not m:
        fail(f"{path}: boshida ---json ... --- meta yo'q")
    try:
        meta = json.loads(m.group(1))
    except json.JSONDecodeError as e:
        fail(f"{path}: meta JSON xato: {e}")
    meta["body"] = m.group(2)
    meta["src"] = path.relative_to(ROOT).as_posix()
    for k in ("slug", "lang", "title", "description", "h1", "updated"):
        if not meta.get(k):
            fail(f"{path}: meta.{k} yo'q")
    if meta["lang"] not in UI:
        fail(f"{path}: lang {meta['lang']}")
    if not meta["slug"].startswith("/") or not meta["slug"].endswith("/"):
        fail(f"{path}: slug /.../ ko'rinishida bo'lsin: {meta['slug']}")
    return meta


def subst(text, ctx):
    def rep(m):
        k = m.group(1)
        if k not in ctx:
            fail(f"noma'lum o'zgaruvchi {{{{{k}}}}}")
        return str(ctx[k])
    return re.sub(r"\{\{(\w+)\}\}", rep, text)


def esc(s):
    return html.escape(s, quote=True)


def strip_tags(s):
    return re.sub(r"\s+", " ", html.unescape(re.sub(r"<[^>]+>", "", s))).strip()


def faq_items(body):
    """<h2 data-faq>Savol</h2> dan keyingi birinchi <p> — javob."""
    return [(strip_tags(m.group(1)), strip_tags(m.group(2)))
            for m in re.finditer(r"<h[23][^>]*\bdata-faq\b[^>]*>(.*?)</h[23]>\s*<p[^>]*>(.*?)</p>", body, re.S)]


def og_path(slug):
    name = "index" if slug == "/" else slug.strip("/").replace("/", "-")
    return f"/og/{name}.png"


def software_app(ctx):
    return {
        "@type": "SoftwareApplication", "@id": SITE + "/#app", "name": "bitg",
        "description": "Oʻzbek matnini yozish paytida 2026-yilgi yangi lotin alifbosiga (Ö, Ğ, Ş, Ç) avtomatik "
                       "oʻgiradi. Inglizcha soʻz, havola va kodga tegmaydi, Backspace bilan qaytariladi, internetsiz ishlaydi.",
        "applicationCategory": "UtilitiesApplication", "operatingSystem": "Windows 10, Windows 11, Ubuntu 22.04+, Debian 12+, macOS 11+",
        "softwareVersion": ctx["version"], "fileSize": ctx["msiSizeMb"] + " MB",
        "downloadUrl": SITE + ctx["msiUrl"], "installUrl": SITE + "/yuklab-olish/",
        "isAccessibleForFree": True, "license": "https://github.com/dtwinuz/bitg#litsenziya",
        "offers": {"@type": "Offer", "price": "0", "priceCurrency": "UZS"},
        "publisher": {"@id": SITE + "/#org"},
    }


def sitewide():
    return [
        {"@type": "Organization", "@id": SITE + "/#org", "name": "bitg", "url": SITE + "/",
         "logo": SITE + "/logo-512.png",
         "description": "bitg — yozayotganingizda oʻzbek matnini 2026-yilgi yangi alifboga (ö ğ ş ç) avtomatik "
                        "oʻgiradigan bepul, ochiq kodli dastur.",
         "sameAs": ["https://github.com/dtwinuz/bitg", "https://t.me/bitguz", "https://t.me/bitguzbot"]},
        {"@type": "WebSite", "@id": SITE + "/#website", "url": SITE + "/", "name": "bitg",
         "inLanguage": ["uz", "ru", "en"], "publisher": {"@id": SITE + "/#org"}},
    ]


def breadcrumb(meta, by_slug, ctx):
    lang = meta["lang"]
    home = HOME[lang]
    out = [(home, "bitg")]
    if meta["slug"] == home:
        return out
    parts = meta["slug"].strip("/").split("/")
    if lang in ("ru", "en"):
        parts = parts[1:]
    acc = home
    for p in parts:
        acc = acc + p + "/"
        page = by_slug.get(acc)
        if page:
            out.append((acc, subst(page.get("crumb") or strip_tags(page["h1"]), ctx)))
    return out


def jsonld(meta, by_slug, ctx):
    slug, lang = meta["slug"], meta["lang"]
    graph = sitewide()
    schema = meta.get("schema", [])
    if "SoftwareApplication" in schema:
        graph.append(software_app(ctx))
    if "Article" in schema:
        graph.append({
            "@type": "Article", "headline": strip_tags(subst(meta["h1"], ctx)),
            "description": subst(meta["description"], ctx), "inLanguage": lang,
            "datePublished": meta.get("published", meta["updated"]), "dateModified": meta["updated"],
            "author": {"@id": SITE + "/#org"}, "publisher": {"@id": SITE + "/#org"},
            "mainEntityOfPage": SITE + slug, "image": SITE + og_path(slug)})
    if "FAQPage" in schema:
        qa = faq_items(subst(meta["body"], ctx))
        if not qa:
            fail(f"{meta['src']}: FAQPage, lekin data-faq sarlavha yo'q")
        graph.append({"@type": "FAQPage", "mainEntity": [
            {"@type": "Question", "name": q, "acceptedAnswer": {"@type": "Answer", "text": a}} for q, a in qa]})
    crumbs = breadcrumb(meta, by_slug, ctx)
    if len(crumbs) > 1:
        graph.append({"@type": "BreadcrumbList", "itemListElement": [
            {"@type": "ListItem", "position": i + 1, "name": n, "item": SITE + s} for i, (s, n) in enumerate(crumbs)]})
    text = json.dumps({"@context": "https://schema.org", "@graph": graph}, ensure_ascii=False, indent=1)
    return text.replace("</", "<\\/")


def render(meta, ctx, by_slug, alt_groups):
    lang, slug = meta["lang"], meta["slug"]
    ui = UI[lang]
    url = SITE + slug
    body = subst(meta["body"], ctx)
    title = subst(meta["title"], ctx)
    desc = subst(meta["description"], ctx)
    h1 = subst(meta["h1"], ctx)
    og_title = subst(meta.get("og_title", title), ctx)
    og_desc = subst(meta.get("og_description", desc), ctx)
    if len(strip_tags(title)) > 65:
        print(f"ogohlantirish: {slug} title {len(title)} belgi", file=sys.stderr)
    if not 110 <= len(desc) <= 170:
        print(f"ogohlantirish: {slug} description {len(desc)} belgi", file=sys.stderr)
    if body.count("<h1") or h1.count("<h1"):
        fail(f"{meta['src']}: tanada <h1> bo'lmasin (shablon qo'yadi)")

    grp = meta.get("alt")
    langs = alt_groups.get(grp, {}) if grp else {}
    hreflang = ""
    if len(langs) > 1:
        rows = [f'<link rel="alternate" hreflang="{l}" href="{SITE}{s}">' for l, s in sorted(langs.items())]
        rows.append(f'<link rel="alternate" hreflang="x-default" href="{SITE}{langs.get("uz") or slug}">')
        hreflang = "\n".join(rows)

    crumbs = breadcrumb(meta, by_slug, ctx)
    crumbs_html = ""
    if len(crumbs) > 1:
        items = [f'<li aria-current="page">{esc(n)}</li>' if i == len(crumbs) - 1 else f'<li><a href="{s}">{esc(n)}</a></li>'
                 for i, (s, n) in enumerate(crumbs)]
        crumbs_html = f'<nav class="dc-crumbs" aria-label="{ui["crumbs"]}"><ol>{"".join(items)}</ol></nav>'

    lead_html = f'<p class="dc-lead">{subst(meta["lead"], ctx)}</p>' if meta.get("lead") else ""

    src_html = ""
    if meta.get("sources"):
        li = "".join(f'<li><a href="{esc(s["url"])}" rel="noopener" data-no-conv>{esc(s["title"])}</a>'
                     + (f' — {esc(s["note"])}' if s.get("note") else "") + "</li>" for s in meta["sources"])
        src_html = (f'<section class="dc-sources" aria-labelledby="dc-src-h"><h2 id="dc-src-h">{ui["sources"]}</h2>'
                    f'<ol>{li}</ol></section>')

    rel = [s for s in meta.get("links", []) if s in by_slug and s != slug and not by_slug[s].get("manual")]
    missing = [s for s in meta.get("links", []) if s not in by_slug]
    if missing:
        print(f"eslatma: {slug} bog'liq havolalar hali yo'q: {', '.join(missing)}", file=sys.stderr)
    rel_html = ""
    if rel:
        cards = "".join(
            f'<a class="dc-rel__card" href="{s}"><b>{esc(strip_tags(subst(by_slug[s]["h1"], ctx)))}</b>'
            f'<span>{esc(subst(by_slug[s].get("card", by_slug[s]["description"]), ctx))}</span></a>' for s in rel)
        rel_html = (f'<section class="dc-rel" aria-labelledby="dc-rel-h"><h2 id="dc-rel-h">{ui["related"]}</h2>'
                    f'<div class="dc-rel__grid">{cards}</div></section>')

    upd = (f'<p class="dc-updated">{ui["updated"]}: <time datetime="{meta["updated"]}">'
           f'{human_date(meta["updated"], lang)}</time></p>') if not meta.get("no_updated") else ""

    abc_html = ""
    if lang == "uz":
        abc_html = (f'<div class="bg-seg" role="group" aria-label="{ui["abc"]}" data-no-conv>'
                    f'<button type="button" class="bg-seg__b" data-abc="new" aria-pressed="false">{ui["new"]}</button>'
                    f'<button type="button" class="bg-seg__b" data-abc="old" aria-pressed="true">{ui["old"]}</button></div>')

    lang_html = ""
    if len(langs) > 1:
        names = {"uz": "UZ", "ru": "RU", "en": "EN"}
        order = sorted(langs.items(), key=lambda kv: ["uz", "ru", "en"].index(kv[0]))
        lang_html = (f'<div class="dc-langs" role="group" aria-label="{ui["langs"]}" data-no-conv>' + "".join(
            f'<a href="{s}" hreflang="{l}" lang="{l}"' + (' aria-current="true"' if s == slug else "") + f'>{names[l]}</a>'
            for l, s in order) + "</div>")

    nav = "\n      ".join(
        f'<a href="{h}"' + (' aria-current="page"' if h == slug else "") + f'>{esc(n)}</a>' for h, n in ui["nav_items"])

    foot_cols = "".join(
        f'<div class="dc-foot__col"><h2>{esc(t)}</h2><ul>' + "".join(
            f'<li><a href="{h}"' + (' rel="noopener"' if h.startswith("http") else "") + f'>{esc(n)}</a></li>'
            for h, n in items) + "</ul></div>" for t, items in ui["foot_cols"])

    wasm_attr = ' data-wasm="1"' if meta.get("wasm") else ""
    # dl.js: tizimni aniqlash va rahmat sahifasi (yuklab olish sahifalari)
    dl_js = '<script src="/dl.js" defer></script>\n' if meta.get("dl") else ""
    robots = '<meta name="robots" content="noindex">\n' if meta.get("noindex") else ""
    article_cls = "dc-article" + (" dc-article--wide" if meta.get("wide") else "")
    dl_btn = "" if slug == "/yuklab-olish/" else \
        f'<a class="bg-btn bg-btn--pri bg-btn--sm bg-hide-sm" href="/yuklab-olish/">{ui["dl"]}</a>'
    home = HOME[lang]
    og_type = "website" if slug in HOME.values() else "article"

    return f"""<!doctype html>
<html lang="{LANG_HTML[lang]}" data-v=""{wasm_attr}>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{esc(title)}</title>
<meta name="description" content="{esc(desc)}">
{robots}<meta name="theme-color" content="#f6f6f3" media="(prefers-color-scheme: light)">
<meta name="theme-color" content="#0f1117" media="(prefers-color-scheme: dark)">
<link rel="canonical" href="{url}">
{hreflang}
<meta property="og:type" content="{og_type}">
<meta property="og:site_name" content="bitg">
<meta property="og:title" content="{esc(og_title)}">
<meta property="og:description" content="{esc(og_desc)}">
<meta property="og:url" content="{url}">
<meta property="og:image" content="{SITE}{og_path(slug)}">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">
<meta property="og:locale" content="{OG_LOCALE[lang]}">
<meta name="twitter:card" content="summary_large_image">
<link rel="icon" href="/icon.svg" type="image/svg+xml">
<link rel="apple-touch-icon" href="/apple-touch-icon.png">
<link rel="preload" href="/fonts/inter-latin.woff2" as="font" type="font/woff2" crossorigin>
<link rel="stylesheet" href="/styles.css">
<link rel="stylesheet" href="/doc.css">
<script src="/theme.js"></script>
<script type="application/ld+json">
{jsonld(meta, by_slug, ctx)}
</script>
</head>
<body class="dc">
<a class="bg-skip" href="#dc-main">{ui["skip"]}</a>

<header class="bg-head" id="bg-head">
  <div class="bg-wrap bg-head__in">
    <a class="bg-brand" href="{home}" aria-label="bitg — {ui["home"]}">
      {BRAND_SVG}
      <span>bitg</span>
    </a>
    <nav class="bg-nav dc-nav" id="dc-nav" aria-label="{ui["nav"]}">
      {nav}
    </nav>
    <div class="bg-head__end">
      {lang_html}
      {abc_html}
      <button type="button" class="bg-icon" id="bg-theme" aria-label="{ui["theme"]}">{ICON_THEME}</button>
      <a class="bg-icon bg-hide-sm" href="https://github.com/dtwinuz/bitg" aria-label="{ui["gh"]}" rel="noopener">{ICON_GH}</a>
      {dl_btn}
      <button type="button" class="bg-icon dc-menu" id="dc-menu" aria-controls="dc-nav" aria-expanded="false" aria-label="{ui["nav"]}">{ICON_MENU}</button>
    </div>
  </div>
</header>

<main id="dc-main">
  <div class="bg-wrap">
    <article class="{article_cls}">
      {crumbs_html}
      <h1>{h1}</h1>
      {lead_html}
{body}
      {upd}
      {src_html}
    </article>
    {rel_html}
  </div>
</main>

<footer class="bg-foot dc-foot">
  <div class="bg-wrap">
    <div class="dc-foot__cols">{foot_cols}</div>
    <div class="bg-foot__in">
      <a class="bg-brand" href="{home}" aria-label="bitg">{BRAND_SVG}<span>bitg</span></a>
      <span class="bg-foot__txt">{ui["foot_lic"]} · <a href="https://github.com/dtwinuz/bitg" data-no-conv>github.com/dtwinuz/bitg</a></span>
      <span class="bg-foot__txt">Telegram: <a href="https://t.me/bitguz" rel="noopener" data-no-conv>@bitguz</a> · bot: <a href="https://t.me/bitguzbot" rel="noopener" data-no-conv>@bitguzbot</a></span>
    </div>
  </div>
</footer>
<script src="/page.js" defer></script>
{dl_js}</body>
</html>
"""


def version_assets(text, v):
    if not v:
        return text
    text = re.sub(r'(src|href)="(/?[a-z0-9.\-]+\.(?:js|css))"', lambda m: f'{m.group(1)}="{m.group(2)}?v={v}"', text)
    return text.replace('data-v=""', f'data-v="{v}"')


def msi_info(msi):
    if not msi.exists():
        fail(f"MSI topilmadi: {msi} (--msi)")
    data = msi.read_bytes()
    mb = len(data) / (1024 * 1024)
    return {"msiName": msi.name, "msiUrl": f"/download/{msi.name}", "sha256": hashlib.sha256(data).hexdigest(),
            "msiSizeMb": f"{mb:.1f}", "msiSizeUz": f"{mb:.1f}".replace(".", ",")}


def pkg_info(path, key):
    """Linux/macOS paketi: {key}Name/Url/Sha/SizeUz (masalan debName, fcitxSha, macSizeUz)."""
    if not path.exists():
        fail(f"paket topilmadi: {path} (GitHub Release'dan dist/ ga yuklang)")
    data = path.read_bytes()
    kb = len(data) / 1024
    size = f"{kb / 1024:.1f} MB".replace(".", ",") if kb >= 1024 else f"{kb:.0f} KB"
    return {key + "Name": path.name, key + "Url": f"/download/{path.name}",
            key + "Sha": hashlib.sha256(data).hexdigest(), key + "SizeUz": size}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--v", default="", help="kesh versiyasi (git sha qisqa)")
    ap.add_argument("--msi", type=Path, default=None)
    ap.add_argument("--today", default=None, help="kunlar hisobi uchun sana (standart: bugun)")
    a = ap.parse_args()

    version = re.search(r'^version = "(.+?)"', (ROOT / "Cargo.toml").read_text(encoding="utf-8"), re.M).group(1)
    msi = a.msi or ROOT / "dist" / f"bitg-{version}-x64.msi"
    today = dt.date.fromisoformat(a.today) if a.today else dt.date.today()
    ctx = dict(CONFIG)
    ctx.update(msi_info(msi))
    ctx.update(pkg_info(msi.parent / f"bitg_{version}_amd64.deb", "deb"))
    ctx.update(pkg_info(msi.parent / f"fcitx5-bitg_{version}_amd64.deb", "fcitx"))
    ctx.update(pkg_info(msi.parent / f"bitg-{version}-macos.pkg", "mac"))
    fpr = (ROOT / "deploy" / "apt" / "fingerprint.txt").read_text(encoding="utf-8").strip().upper()
    if not re.fullmatch(r"[0-9A-F]{40}", fpr):
        fail("deploy/apt/fingerprint.txt: 40 belgili barmoq izi emas")
    ctx.update({
        "aptFingerprint": " ".join(fpr[i:i + 4] for i in range(0, 40, 4)),
        "version": version,
        "daysLeft": max((dt.date.fromisoformat(CONFIG["lawDate"]) - today).days, 0),
        "buildDate": today.isoformat(),
    })
    for lg in ("Uz", "Ru", "En"):
        ctx["lawDate" + lg] = human_date(CONFIG["lawDate"], lg.lower())
        ctx["signedDate" + lg] = human_date(CONFIG["signedDate"], lg.lower())

    pages = [read_page(p) for p in sorted(PAGES.rglob("*.html"))]
    by_slug = {}
    for p in pages:
        if p["slug"] in by_slug:
            fail(f"slug takror: {p['slug']} ({p['src']}, {by_slug[p['slug']]['src']})")
        by_slug[p["slug"]] = p
    # bosh sahifa qo'lda yozilgan (web/index.html): breadcrumb, hreflang va sitemap uchun ro'yxatda
    by_slug.setdefault("/", {"slug": "/", "lang": "uz", "h1": "bitg", "alt": "home", "updated": ctx["buildDate"],
                             "description": "", "manual": True})
    alt_groups = {}
    for p in by_slug.values():
        if p.get("alt"):
            alt_groups.setdefault(p["alt"], {})[p["lang"]] = p["slug"]
    ctx["homeHreflang"] = "\n".join(
        [f'<link rel="alternate" hreflang="{l}" href="{SITE}{s}">' for l, s in sorted(alt_groups["home"].items())]
        + [f'<link rel="alternate" hreflang="x-default" href="{SITE}/">'])
    ctx["homeJsonLd"] = json.dumps({"@context": "https://schema.org", "@graph": sitewide() + [software_app(ctx)]},
                                   ensure_ascii=False, indent=1).replace("</", "<\\/")

    out = a.out
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)
    for f in WEB.rglob("*"):
        rel = f.relative_to(WEB)
        if f.is_dir() or rel.parts[0] in SKIP_TOP or any(x.startswith(".") for x in rel.parts) \
                or f.suffix.lower() not in COPY_EXT:
            continue
        dst = out / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(f, dst)

    for p in pages:
        text = version_assets(render(p, ctx, by_slug, alt_groups), a.v)
        if p["slug"] == "/404/":
            dst = out / "404.html"
        else:
            dst = out / p["slug"].strip("/") / "index.html"
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_text(text, encoding="utf-8", newline="\n")

    idx = out / "index.html"
    idx.write_text(version_assets(subst(idx.read_text(encoding="utf-8"), ctx), a.v), encoding="utf-8", newline="\n")

    listed = sorted(s for s, p in by_slug.items() if s != "/404/" and not p.get("noindex"))
    (out / "sitemap.xml").write_text(
        '<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
        + "\n".join(f"  <url><loc>{SITE}{s}</loc><lastmod>{by_slug[s]['updated']}</lastmod></url>" for s in listed)
        + "\n</urlset>\n", encoding="utf-8", newline="\n")
    (out / "urls.txt").write_text("\n".join(SITE + s for s in listed) + "\n", encoding="utf-8", newline="\n")
    print(f"build: {len(pages)} sahifa + bosh sahifa, sitemap {len(listed)} URL -> {out}")


if __name__ == "__main__":
    main()
