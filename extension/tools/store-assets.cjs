// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Chrome Web Store rasmlari (extension/store-assets/): haqiqiy yig'ilgan kengaytma Chromium'ga yuklanadi,
// popup va sahifa holatlari ekrandan olinadi, so'ng bitg.uz tokenlari bilan ramkaga qo'yiladi.
// Promo plitkalar — logo (web/icon.svg) va matn. Hammasi 24-bit RGB (alfa yo'q) — Pillow bilan tekislanadi.
//
//   node extension/tools/build.mjs
//   NODE_PATH="$(npm root -g)/@playwright/cli/node_modules" CHROME=<chromium>/chrome.exe node extension/tools/store-assets.cjs
'use strict';
const fs = require('fs');
const os = require('os');
const path = require('path');
const http = require('http');
const { execFileSync } = require('child_process');
const { chromium } = require('playwright-core');

const EXT = path.resolve(__dirname, '..');
const ROOT = path.resolve(EXT, '..');
const DIST = path.join(EXT, 'dist', 'chrome');
const OUT = path.join(EXT, 'store-assets');
const FONTS = 'file:///' + path.join(ROOT, 'web', 'fonts').replace(/\\/g, '/');
const ICON = fs.readFileSync(path.join(ROOT, 'web', 'icon.svg'), 'utf8').replace(/ width="\d+" height="\d+"/, '');

const COMPOSER = () => `<!doctype html><html lang="uz"><head><meta charset="utf-8"><title>Xabar</title><style>
@font-face{font-family:Inter;font-weight:100 900;src:url("/fonts/inter-latin.woff2")}
@font-face{font-family:Inter;font-weight:100 900;src:url("/fonts/inter-latin-ext.woff2");unicode-range:U+0100-02AF}
body{margin:0;font:15px/1.5 Inter,system-ui,sans-serif;background:#fff;color:#1d2027;overflow:hidden}
.w{padding:28px 30px}h2{margin:0 0 4px;font-size:19px}p{margin:0 0 16px;color:#5d626b;font-size:13.5px}
label{display:block;font-size:12.5px;font-weight:600;color:#5d626b;margin-bottom:6px}
input,textarea{width:100%;box-sizing:border-box;font:inherit;border:1px solid #c9cdd3;border-radius:8px;padding:10px 12px;color:inherit}
input{margin-bottom:14px}textarea:focus{outline:none;border-color:#2f62c0;box-shadow:0 0 0 3px #e6edfb}textarea{height:170px;resize:none;line-height:1.6;font-size:16px}
textarea::selection{background:#b9cdf3}
.b{margin-top:14px;display:inline-block;background:#2f62c0;color:#fff;border-radius:8px;padding:9px 18px;font-weight:600;font-size:14px}
</style></head><body><div class="w"><h2>Yangi xabar</h2><p>Hamkasblarga e’lon</p>
<label>Mavzu</label><input value="Ertangi uchrashuv">
<label>Matn</label>
<textarea id="ta">Assalomu alaykum! Ertaga soat o‘nda choyxonada ko‘rishamiz. Shahar markazidagi yangi bog‘ haqida gaplashamiz, chiroyli rasmlar ham olib kelaman.</textarea>
<span class="b">Yuborish</span></div></body></html>`;

const NEWS = () => `<!doctype html><html lang="uz"><head><meta charset="utf-8"><title>Shahar kundaligi</title><style>
@font-face{font-family:Inter;font-weight:100 900;src:url("/fonts/inter-latin.woff2")}
@font-face{font-family:Inter;font-weight:100 900;src:url("/fonts/inter-latin-ext.woff2");unicode-range:U+0100-02AF}
body{margin:0;font:15px/1.6 Georgia,"Times New Roman",serif;background:#fff;color:#1d2027;overflow:hidden}
header{font-family:Inter,system-ui,sans-serif;display:flex;align-items:center;justify-content:space-between;padding:14px 26px;border-bottom:1px solid #e2e4e7}
.logo{font-weight:750;font-size:18px;letter-spacing:-.01em}.logo b{color:#b23b2e}
nav{font-family:Inter,system-ui,sans-serif;font-size:13px;color:#5d626b;display:flex;gap:16px;padding:9px 26px;border-bottom:1px solid #e2e4e7}
input{font:13px Inter,system-ui,sans-serif;border:1px solid #c9cdd3;border-radius:6px;padding:6px 10px;width:150px}
article{padding:20px 26px}.k{font:600 11.5px Inter,system-ui,sans-serif;color:#b23b2e;text-transform:uppercase;letter-spacing:.06em}
h1{font-size:27px;line-height:1.25;margin:6px 0 10px}.m{font:12.5px Inter,system-ui,sans-serif;color:#8a9099;margin-bottom:14px}
p{margin:0 0 12px;font-size:15.5px}
</style></head><body>
<header><span class="logo">Shahar<b>.</b>kundaligi</span><input placeholder="Qidirish..." value="choyxona"></header>
<nav><span>Bosh sahifa</span><span>Jamiyat</span><span>Shahar</span><span>Sport</span><span>Ob-havo</span></nav>
<article><div class="k">Shahar</div>
<h1>Toshkentda yangi choyxonalar va bog‘lar ochildi</h1>
<div class="m">Bugun, 09:30 · O‘qish uchun 3 daqiqa</div>
<p>Poytaxtning Chilonzor va Shayxontohur tumanlarida shu hafta to‘rtta yangi choyxona ochildi. Ularning har birida bolalar uchun o‘yin maydonchasi va kichik kutubxona bor.</p>
<p>Shahar hokimligi ma’lumotiga ko‘ra, kuzgacha yana o‘nta bog‘ obodonlashtiriladi. Aholi takliflarini rasmiy sahifa orqali yuborishi mumkin.</p>
<p>Choyxona egalaridan biri: «Ertalab nonushta, kechqurun esa o‘zbek musiqasi kechalari bo‘ladi», — dedi.</p>
</article></body></html>`;

const CSS = `
@font-face{font-family:Inter;font-weight:100 900;src:url("${FONTS}/inter-latin.woff2")}
@font-face{font-family:Inter;font-weight:100 900;src:url("${FONTS}/inter-latin-ext.woff2");unicode-range:U+0100-02AF,U+1E00-1E9F}
*{box-sizing:border-box}html,body{margin:0;height:100%}
body{font-family:Inter,system-ui,sans-serif;background:#f6f6f3;color:#1d2027;-webkit-font-smoothing:antialiased;overflow:hidden}
.mark{display:flex;align-items:center;gap:10px;font-weight:650;font-size:20px;letter-spacing:-.01em}
.mark svg{width:34px;height:34px}
h1{margin:0;font-weight:650;letter-spacing:-.025em;line-height:1.08}
.dot{color:#2f62c0}
.sub{color:#5d626b}
.shot{background:#fff;border:1px solid #e2e4e7;border-radius:14px;overflow:hidden;
  box-shadow:0 1px 0 rgb(20 24 32 / .04),0 24px 60px -24px rgb(20 24 32 / .28)}
.shot img{display:block;width:100%}
.tag{display:inline-flex;align-items:center;gap:7px;font-size:13px;font-weight:600;color:#5d626b;margin-bottom:10px}
.tag i{width:8px;height:8px;border-radius:50%;background:#c9cdd3}
.tag.new i{background:#2f62c0}.tag.new{color:#2553a8}
`;

const b64 = (p) => 'data:image/png;base64,' + fs.readFileSync(p).toString('base64');
function step(m) { console.log('  ' + m); }

(async () => {
  if (!fs.existsSync(path.join(DIST, 'core', 'bitg_core.wasm'))) throw new Error('avval: node extension/tools/build.mjs');
  fs.mkdirSync(OUT, { recursive: true });
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bitg-assets-'));
  // sinov nusxasi: 127.0.0.1 uchun host ruxsati — activeTab (bosish) o'rnida, xuddi smoke.cjs kabi
  const extDir = path.join(tmp, 'ext');
  fs.cpSync(DIST, extDir, { recursive: true });
  const mf = JSON.parse(fs.readFileSync(path.join(extDir, 'manifest.json'), 'utf8'));
  mf.host_permissions = ['http://127.0.0.1/*'];
  fs.writeFileSync(path.join(extDir, 'manifest.json'), JSON.stringify(mf, null, 2));

  const pages = { '/composer': COMPOSER(), '/news': NEWS() };
  const server = http.createServer((q, r) => {
    const font = /^\/fonts\/(inter-[a-z-]+\.woff2)$/.exec(q.url);
    if (font) { r.writeHead(200, { 'content-type': 'font/woff2' }); r.end(fs.readFileSync(path.join(ROOT, 'web', 'fonts', font[1]))); return; }
    r.writeHead(200, { 'content-type': 'text/html; charset=utf-8' }); r.end(pages[q.url] || '');
  });
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  const base = `http://127.0.0.1:${server.address().port}`;

  const ctx = await chromium.launchPersistentContext(path.join(tmp, 'profile'), {
    executablePath: process.env.CHROME || undefined, headless: false, deviceScaleFactor: 2,
    args: ['--headless=new', `--disable-extensions-except=${extDir}`, `--load-extension=${extDir}`, '--allow-file-access-from-files'],
    permissions: ['clipboard-read', 'clipboard-write'], colorScheme: 'light'
  });
  try {
    let [sw] = ctx.serviceWorkers();
    if (!sw) sw = await ctx.waitForEvent('serviceworker');
    const id = new URL(sw.url()).host;
    const raw = (n) => path.join(tmp, n);

    // --- 1) popup (haqiqiy popup sahifasi) ---
    const popup = await ctx.newPage();
    await popup.setViewportSize({ width: 380, height: 640 });
    await popup.goto(`chrome-extension://${id}/popup/popup.html`);
    await popup.fill('#in', 'O‘zbekiston shaharlari va choyxonalar. Ertaga Shahrisabzga boramiz, push xabarni https://example.uz/sh ga yuboraman.');
    await popup.waitForFunction(() => document.querySelectorAll('#out mark').length > 0);
    // tab sifatida ochilgan popup o'zini "brauzer sahifasi" deb ko'radi — bu bo'lim kadrga kirmaydi
    await popup.evaluate(() => { document.getElementById('site').hidden = true; document.activeElement.blur(); });
    const outText = await popup.textContent('#out');
    await popup.locator('main').screenshot({ path: raw('popup.png') });
    step('popup: ' + outText);

    // --- 2) belgilangan matn (kontekst menyu bilan bir xil yo'l: convertSelection) ---
    const page = await ctx.newPage();
    await page.setViewportSize({ width: 560, height: 452 });
    await page.goto(base + '/composer');
    await page.bringToFront();
    await page.evaluate(() => document.fonts.ready);
    await page.focus('#ta');
    await page.$eval('#ta', (e) => e.setSelectionRange(0, e.value.length));
    await page.screenshot({ path: raw('sel-before.png') });
    const tabId = await sw.evaluate(async (u) => (await chrome.tabs.query({ url: u + '/*' }))[0].id, base);
    await sw.evaluate((t) => convertSelection(t, 0), tabId);
    await page.waitForTimeout(450); // xabarcha chiqib ulgursin
    await page.screenshot({ path: raw('sel-after.png') });
    step('textarea: ' + (await page.$eval('#ta', (e) => e.value)));

    // --- 3) butun sahifa ---
    await page.setViewportSize({ width: 560, height: 452 });
    await page.goto(base + '/news');
    await page.evaluate(() => document.fonts.ready);
    await page.screenshot({ path: raw('page-before.png') });
    await sw.evaluate((t) => togglePage(t), tabId);
    await page.waitForTimeout(450);
    await page.screenshot({ path: raw('page-after.png') });
    step('sahifa: ' + (await page.textContent('h1')));

    // --- ramkalar ---
    const frame = await ctx.newPage();
    async function render(name, w, h, body, css = '') {
      await frame.setViewportSize({ width: w, height: h });
      const f = path.join(tmp, name + '.html');
      fs.writeFileSync(f, `<!doctype html><html><head><meta charset="utf-8"><style>${CSS}${css}</style></head><body>${body}</body></html>`);
      await frame.goto('file:///' + f.replace(/\\/g, '/'));
      await frame.evaluate(() => document.fonts.ready);
      await frame.screenshot({ path: path.join(OUT, name + '.png'), scale: 'css' });
    }
    const mark = `<div class="mark">${ICON}<span>bitg</span></div>`;

    await render('shot-1-popup', 1280, 800, `
      <div style="display:grid;grid-template-columns:1fr 400px;gap:72px;align-items:center;height:100%;padding:0 96px">
        <div>${mark}
          <h1 style="font-size:58px;margin:34px 0 22px">Eski matn <span class="dot">→</span> yangi alifbo, bir bosishda</h1>
          <p class="sub" style="font-size:21px;line-height:1.5;margin:0;max-width:520px">O‘, g‘, sh, ch va kirill matn Ö Ğ Ş Ç ga o‘tadi. Inglizcha so‘z va havolalarga tegilmaydi. Internet kerak emas.</p>
        </div>
        <div class="shot" style="border-radius:16px"><img src="${b64(raw('popup.png'))}"></div>
      </div>`);

    const pair = (title, sub, a, b, la, lb) => `
      <div style="padding:52px 72px 0">${mark}
        <h1 style="font-size:46px;margin:22px 0 8px">${title}</h1>
        <p class="sub" style="font-size:19px;margin:0 0 30px">${sub}</p>
        <div style="display:grid;grid-template-columns:1fr 1fr;gap:40px">
          <div><div class="tag"><i></i>${la}</div><div class="shot"><img src="${b64(a)}"></div></div>
          <div><div class="tag new"><i></i>${lb}</div><div class="shot"><img src="${b64(b)}"></div></div>
        </div></div>`;
    await render('shot-2-selection', 1280, 800, pair('Belgilang <span class="dot">→</span> o‘giring',
      'O‘ng tugma yoki Alt+Shift+Y: matn maydonning o‘zida almashadi, Ctrl+Z qaytaradi.',
      raw('sel-before.png'), raw('sel-after.png'), 'Belgilangan matn', 'bitg: yangi alifboga o‘girish'));
    await render('shot-3-page', 1280, 800, pair('Butun sahifa <span class="dot">—</span> yangi alifboda',
      '«Sahifani yangi alifboda ko‘rish»: matn o‘giriladi, maydon va formalarga tegilmaydi. Yana bir bosish — asl holi.',
      raw('page-before.png'), raw('page-after.png'), 'Asl sahifa', 'Sahifani yangi alifboda ko‘rish'));

    const letters = (size) => `<div style="display:flex;gap:${size * 0.18}px;font-weight:650;font-size:${size}px;letter-spacing:-.01em">
      ${['Ö', 'Ğ', 'Ş', 'Ç'].map((c) => `<span style="width:${size * 1.25}px;height:${size * 1.35}px;display:grid;place-items:center;background:#fff;border:1px solid #e2e4e7;border-radius:${size * 0.22}px;box-shadow:0 1px 0 #d6d9de,0 ${size * 0.12}px 0 #d6d9de">${c}</span>`).join('')}</div>`;
    await render('promo-small', 440, 280, `
      <div style="height:100%;padding:30px 32px;display:flex;flex-direction:column;justify-content:space-between">
        <div class="mark" style="font-size:24px">${ICON}<span>bitg</span></div>
        ${letters(30)}
        <div style="font-size:17px;font-weight:600;letter-spacing:-.01em">Yangi o‘zbek alifbosi — <span class="dot">brauzerda</span></div>
      </div>`, '.mark svg{width:40px;height:40px}');
    await render('promo-marquee', 1400, 560, `
      <div style="height:100%;display:grid;grid-template-columns:1fr auto;align-items:center;gap:80px;padding:0 110px">
        <div>
          <div class="mark" style="font-size:34px;gap:14px">${ICON}<span>bitg</span></div>
          <h1 style="font-size:62px;margin:34px 0 18px">Yangi o‘zbek alifbosi — <span class="dot">brauzerda</span></h1>
          <p class="sub" style="font-size:22px;margin:0">Belgilangan matn, butun sahifa va yozayotganda. Oflayn.</p>
        </div>
        ${letters(62)}
      </div>`, '.mark svg{width:56px;height:56px}');
  } finally {
    await ctx.close();
    server.close();
    fs.rmSync(tmp, { recursive: true, force: true });
  }

  // 24-bit RGB, alfa yo'q (do'kon talabi) + o'lcham tekshiruvi
  const want = { 'shot-1-popup': [1280, 800], 'shot-2-selection': [1280, 800], 'shot-3-page': [1280, 800], 'promo-small': [440, 280], 'promo-marquee': [1400, 560] };
  const py = `
import sys, json
from PIL import Image
want = json.loads(sys.argv[2])
for name, (w, h) in want.items():
    p = f"{sys.argv[1]}/{name}.png"
    im = Image.open(p)
    bg = Image.new("RGB", im.size, (246, 246, 243))
    if im.mode in ("RGBA", "LA", "P"):
        im = im.convert("RGBA"); bg.paste(im, mask=im.split()[3]); im = bg
    else:
        im = im.convert("RGB")
    assert im.size == (w, h), (name, im.size)
    im.save(p, optimize=True)
    with open(p, "rb") as f:
        head = f.read(26)
    print(f"{name}.png {im.size[0]}x{im.size[1]} mode={Image.open(p).mode} IHDR color_type={head[25]} bit_depth={head[24]}")
    assert head[25] == 2 and head[24] == 8
`;
  console.log(execFileSync('python', ['-I', '-c', py, OUT, JSON.stringify(want)], { encoding: 'utf8' }).trim());
})().catch((e) => { console.error(e); process.exit(1); });
