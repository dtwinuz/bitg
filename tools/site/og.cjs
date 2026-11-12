// OG rasmlar (1200×630) va logo PNG'lari: yig'ilgan saytdagi har sahifa h1'idan.
// Natija web/og/*.png, web/logo-512.png, web/apple-touch-icon.png — repoga commit qilinadi.
//
//   python -I tools/site/build.py --out target/site-preview
//   NODE_PATH="$(npm root -g)/@playwright/cli/node_modules" [CHROME=.../chrome.exe] node tools/site/og.cjs target/site-preview
'use strict';
const fs = require('fs');
const path = require('path');
const { chromium } = require('playwright-core');

const ROOT = path.resolve(__dirname, '..', '..');
const WEB = path.join(ROOT, 'web');
const SITE = path.resolve(process.argv[2] || path.join(ROOT, 'target', 'site-preview'));
const FONT = 'file:///' + path.join(WEB, 'fonts').replace(/\\/g, '/');

const MARK = fs.readFileSync(path.join(WEB, 'icon.svg'), 'utf8');
const TAG = { uz: 'Yangi o‘zbek alifbosi uchun', ru: 'Для нового узбекского алфавита', en: 'For the new Uzbek alphabet' };

function pages(dir, base = '') {
  const out = [];
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) out.push(...pages(p, base + '/' + e.name));
    else if (e.name === 'index.html' && base) out.push({ slug: base + '/', file: p });
  }
  return out;
}
const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;');
// teglar natija o'zgarmay qolguncha olinadi; &amp; eng oxirida — "&amp;quot;" ikki marta ochilmasin
const plain = (s) => {
  for (let p = ''; p !== s; ) { p = s; s = s.replace(/<[^>]*>/g, ''); }
  return s.replace(/&#x27;|&#39;/g, "'").replace(/&quot;/g, '"').replace(/&amp;/g, '&').trim();
};

function card(h1, lang) {
  const size = h1.length > 52 ? 64 : h1.length > 34 ? 74 : 86;
  return `<!doctype html><html><head><meta charset="utf-8"><style>
@font-face { font-family: Inter; font-weight: 100 900; src: url("${FONT}/inter-latin.woff2"); unicode-range: U+0000-00FF, U+02BB-02BC, U+2000-206F; }
@font-face { font-family: Inter; font-weight: 100 900; src: url("${FONT}/inter-latin-ext.woff2"); unicode-range: U+0100-02AF; }
@font-face { font-family: Inter; font-weight: 100 900; src: url("${FONT}/inter-cyrillic.woff2"); unicode-range: U+0400-045F, U+0490-0491, U+04B0-04B1; }
* { margin: 0; box-sizing: border-box; }
body { width: 1200px; height: 630px; background: #f6f6f3; color: #1d2027; font-family: Inter, sans-serif; padding: 64px 72px; display: grid; grid-template-rows: auto 1fr auto; position: relative; overflow: hidden; }
.brand { display: flex; align-items: center; gap: 16px; font-size: 40px; font-weight: 700; letter-spacing: -.03em; }
.brand svg { width: 56px; height: 56px; }
h1 { align-self: center; font-size: ${size}px; line-height: 1.06; letter-spacing: -.035em; font-weight: 740; max-width: 980px; }
.foot { display: flex; justify-content: space-between; align-items: center; font-size: 26px; color: #5d626b; }
.keys { display: flex; gap: 12px; }
.keys b { width: 64px; height: 64px; display: grid; place-items: center; border-radius: 14px; background: #fff; border: 1.5px solid #c9cdd3; border-bottom-width: 5px; color: #2553a8; font-size: 34px; font-weight: 700; }
.line { position: absolute; left: 0; right: 0; bottom: 0; height: 10px; background: #2f62c0; }
</style></head><body>
<div class="brand">${MARK}<span>bitg</span></div>
<h1>${esc(h1)}</h1>
<div class="foot"><span>${esc(TAG[lang] || TAG.uz)} · bitg.uz</span><div class="keys"><b>ö</b><b>ğ</b><b>ş</b><b>ç</b></div></div>
<div class="line"></div>
</body></html>`;
}

(async () => {
  // CHROME=<chrome.exe> — playwright-core kutgan versiya o'rnatilmagan bo'lsa
  const browser = await chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : {});
  const page = await browser.newPage({ viewport: { width: 1200, height: 630 } });
  const outDir = path.join(WEB, 'og');
  fs.mkdirSync(outDir, { recursive: true });
  const list = pages(SITE).filter((p) => p.slug !== '/404/');
  for (const p of list) {
    const html = fs.readFileSync(p.file, 'utf8');
    const h1 = plain((html.match(/<h1[^>]*>([\s\S]*?)<\/h1>/) || [])[1] || 'bitg');
    const lang = (html.match(/<html lang="([^"]+)"/) || [])[1] || 'uz';
    const tmp = path.join(SITE, '__og.html');
    fs.writeFileSync(tmp, card(h1, lang));
    await page.goto('file:///' + tmp.replace(/\\/g, '/'));
    await page.evaluate(() => document.fonts.ready);
    const name = p.slug.replace(/^\/|\/$/g, '').replace(/\//g, '-');
    await page.screenshot({ path: path.join(outDir, name + '.png') });
    fs.unlinkSync(tmp);
    console.log('og', p.slug, '->', 'web/og/' + name + '.png');
  }
  // logo: icon.svg to'liq o'lchamda (Organization.logo, apple-touch-icon — iOS burchakni o'zi yumaloqlaydi)
  const square = MARK.replace(/ width="\d+" height="\d+"/, '').replace(/ rx="[\d.]+"/, '');
  for (const [file, size] of [['logo-512.png', 512], ['apple-touch-icon.png', 180]]) {
    await page.setViewportSize({ width: size, height: size });
    await page.setContent(`<html><body style="margin:0;width:${size}px;height:${size}px;background:#1b1e26">
      <div style="width:${size}px;height:${size}px">${square.replace('<svg', '<svg width="100%" height="100%"')}</div></body></html>`);
    await page.screenshot({ path: path.join(WEB, file) });
    console.log('logo', file);
  }
  await browser.close();
})().catch((e) => { console.error(e); process.exit(1); });
