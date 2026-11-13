// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Brauzerda tutun sinovi: yig'ilgan kengaytma (dist/chrome) Chromium'ga --load-extension bilan yuklanadi.
// Popup o'giruvchisi, fon yadrosi, sahifani o'girish/qaytarish, belgilangan matn (input, contenteditable,
// oddiy matn → bufer) va yozayotganda o'girish (⌫ bilan qaytarish) haqiqiy kontent skriptlar bilan tekshiriladi.
// Sinov nusxasiga faqat 127.0.0.1 uchun host_permissions qo'shiladi — activeTab/ruxsat oynasi o'rnida.
//
//   node extension/tools/build.mjs
//   NODE_PATH="$(npm root -g)/@playwright/cli/node_modules" CHROME=<chromium>/chrome.exe node extension/test/smoke.cjs
//
// CHROME — to'liq Chromium (headless shell kengaytma yuklamaydi). SHOTS=<papka> — popup skrinshotlari.
'use strict';
const fs = require('fs');
const os = require('os');
const path = require('path');
const http = require('http');
const assert = require('assert/strict');
const { chromium } = require('playwright-core');

const EXT = path.resolve(__dirname, '..');
const DIST = path.join(EXT, 'dist', 'chrome');
const SAMPLE = 'o‘zbek tili, shahar, choy';
const EXPECT = 'özbek tili, şahar, çoy';

const PAGE = `<!doctype html><html lang="uz"><head><meta charset="utf-8"><title>Shahar sinovi</title></head><body>
<h1 id="h">O‘zbekiston shaharlari</h1>
<p id="p">Toshkent — choyxona va <b>shahar</b>.</p>
<code id="code">shahar</code><pre id="pre">chmod +x</pre>
<textarea id="ta">shahar</textarea>
<input id="inp" value="choy">
<input id="pw" type="password">
<div id="ce" contenteditable="true">chiroyli shahar</div>
<p id="sel">qishloq choy</p>
<div id="ce2" contenteditable="true"></div>
</body></html>`;

function step(name) { console.log('  ✓ ' + name); }

(async () => {
  if (!fs.existsSync(path.join(DIST, 'core', 'bitg_core.wasm'))) throw new Error('avval: node extension/tools/build.mjs');
  // sinov nusxasi: 127.0.0.1 uchun host ruxsati (do'kon arxiviga tushmaydi)
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bitg-ext-'));
  const extDir = path.join(tmp, 'ext');
  fs.cpSync(DIST, extDir, { recursive: true });
  const mf = JSON.parse(fs.readFileSync(path.join(extDir, 'manifest.json'), 'utf8'));
  mf.host_permissions = ['http://127.0.0.1/*'];
  fs.writeFileSync(path.join(extDir, 'manifest.json'), JSON.stringify(mf, null, 2));

  const server = http.createServer((req, res) => { res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' }); res.end(PAGE); });
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  const url = `http://127.0.0.1:${server.address().port}/`;

  const ctx = await chromium.launchPersistentContext(path.join(tmp, 'profile'), {
    executablePath: process.env.CHROME || undefined,
    headless: false,
    args: ['--headless=new', `--disable-extensions-except=${extDir}`, `--load-extension=${extDir}`, '--lang=uz'],
    permissions: ['clipboard-read', 'clipboard-write']
  });
  const external = [];
  ctx.on('request', (r) => {
    const u = r.url();
    if (!u.startsWith('chrome-extension://') && !u.startsWith(url) && !u.startsWith('data:')) external.push(u);
  });
  try {
    let [sw] = ctx.serviceWorkers();
    if (!sw) sw = await ctx.waitForEvent('serviceworker');
    const id = new URL(sw.url()).host;
    console.log('kengaytma ' + id);

    // 1) popup o'giruvchisi
    const popup = await ctx.newPage();
    await popup.setViewportSize({ width: 380, height: 600 });
    await popup.goto(`chrome-extension://${id}/popup/popup.html`);
    await popup.fill('#in', SAMPLE);
    await popup.waitForFunction((e) => document.getElementById('out').textContent === e, EXPECT, { timeout: 10000 });
    const marks = await popup.$$eval('#out mark', (m) => m.map((x) => x.textContent));
    assert.deepEqual(marks, ['özbek', 'şahar,', 'çoy']);
    assert.equal(await popup.textContent('#stat'), '3 ta so‘z o‘girildi');
    step(`popup: "${SAMPLE}" → "${EXPECT}" (belgilangan: ${marks.join(' ')})`);
    const name = await popup.evaluate(() => chrome.i18n.getMessage('extName'));
    step('manifest nomi (_locales): ' + name);
    await popup.fill('#in', 'Ассалому алайкум! Эртага чойхонада кўришамиз');
    await popup.waitForFunction(() => document.getElementById('out').textContent === 'Assalomu alaykum! Ertaga çoyxonada körişamiz');
    step('popup: kirill → yangi lotin');
    await popup.click('[data-lang="ru"]');
    assert.equal(await popup.textContent('#copy span'), 'Копировать');
    await popup.click('[data-lang="uz"]');
    step('popup: interfeys tili almashadi (ru → uz)');
    await popup.click('#copy');
    assert.equal(await popup.evaluate(() => navigator.clipboard.readText()), 'Assalomu alaykum! Ertaga çoyxonada körişamiz');
    step('popup: nusxa olish tugmasi buferga yozadi');
    if (process.env.SHOTS) {
      fs.mkdirSync(process.env.SHOTS, { recursive: true });
      await popup.fill('#in', 'Shu hafta chorshanba kuni push qilaman, linkni https://meet.example.uz/abc ga tashlayman. O‘zbekiston go‘zal!');
      await popup.waitForTimeout(200);
      for (const scheme of ['light', 'dark']) {
        await popup.emulateMedia({ colorScheme: scheme });
        await popup.screenshot({ path: path.join(process.env.SHOTS, `popup-${scheme}.png`), fullPage: true });
      }
      await popup.emulateMedia({ colorScheme: 'light' });
      step('skrinshotlar: ' + process.env.SHOTS);
    }

    // 2) fon yadrosi
    const swOut = await sw.evaluate(async (s) => (await bitgCore.load(chrome.runtime.getURL('core/bitg_core.wasm'))).convert(s), SAMPLE);
    assert.equal(swOut, EXPECT);
    step('service worker yadrosi: ' + swOut);

    // 3) sahifa
    const page = await ctx.newPage();
    await page.goto(url);
    await page.bringToFront();
    const tabId = await sw.evaluate(async (u) => (await chrome.tabs.query({ url: u + '*' }))[0].id, url);
    const txt = (sel) => page.$eval(sel, (e) => ('value' in e && e.tagName !== 'DIV' ? e.value : e.textContent));

    assert.equal(await sw.evaluate((t) => togglePage(t), tabId), true);
    assert.equal(await txt('#h'), 'Özbekiston şaharlari');
    assert.equal(await txt('#p'), 'Toşkent — çoyxona va şahar.');
    assert.equal(await page.title(), 'Şahar sinovi');
    assert.equal(await txt('#code'), 'shahar');
    assert.equal(await txt('#pre'), 'chmod +x');
    assert.equal(await txt('#ta'), 'shahar');
    assert.equal(await txt('#inp'), 'choy');
    assert.equal(await txt('#ce'), 'chiroyli shahar');
    await page.evaluate(() => { const p = document.createElement('p'); p.id = 'late'; p.textContent = 'Keyin qo‘shilgan shahar'; document.body.append(p); });
    await page.waitForFunction(() => document.getElementById('late').textContent === 'Keyin qöşilgan şahar', null, { timeout: 5000 });
    step('sahifa: matn o‘girildi; kod, pre, textarea, input, contenteditable tegilmadi; keyin qo‘shilgan tugun ham');
    assert.equal(await sw.evaluate((t) => pageActive(t), tabId), true);
    assert.equal(await sw.evaluate((t) => togglePage(t), tabId), false);
    assert.equal(await txt('#h'), 'O‘zbekiston shaharlari');
    assert.equal(await txt('#p'), 'Toshkent — choyxona va shahar.');
    assert.equal(await page.title(), 'Shahar sinovi');
    step('sahifa: qayta bosilganda asl holiga qaytdi');

    // 4) belgilangan matn
    await page.focus('#inp');
    await page.$eval('#inp', (e) => e.setSelectionRange(0, e.value.length));
    await sw.evaluate((t) => convertSelection(t, 0), tabId);
    assert.equal(await txt('#inp'), 'çoy');
    await page.keyboard.press('Control+z');
    assert.equal(await txt('#inp'), 'choy');
    step('belgilangan: input joyida almashdi, Ctrl+Z qaytaradi');

    await page.focus('#ta');
    await page.$eval('#ta', (e) => e.setSelectionRange(e.value.length, e.value.length));
    await sw.evaluate((t) => convertSelection(t, 0), tabId);
    assert.equal(await txt('#ta'), 'şahar');
    step('belgilanmagan textarea: butun maydon o‘girildi');

    await page.evaluate(() => {
      const ce = document.getElementById('ce'); ce.focus();
      const r = document.createRange(); r.setStart(ce.firstChild, 9); r.setEnd(ce.firstChild, 15);
      const s = getSelection(); s.removeAllRanges(); s.addRange(r);
    });
    await sw.evaluate((t) => convertSelection(t, 0), tabId);
    assert.equal(await txt('#ce'), 'chiroyli şahar');
    step('contenteditable: faqat belgilangan "shahar" almashdi');

    await page.evaluate(() => {
      document.activeElement.blur();
      const r = document.createRange(); r.selectNodeContents(document.getElementById('sel'));
      const s = getSelection(); s.removeAllRanges(); s.addRange(r);
    });
    await sw.evaluate((t) => convertSelection(t, 0), tabId);
    assert.equal(await page.evaluate(() => navigator.clipboard.readText()), 'qişloq çoy');
    assert.equal(await txt('#sel'), 'qishloq choy');
    const toast = await page.evaluate(() => !!document.querySelector('[data-bitg-skip]'));
    assert.ok(toast);
    step('oddiy matn: buferga "qişloq çoy", sahifa o‘zgarmadi, xabarcha chiqdi');

    // 5) yozayotganda o'girish
    assert.equal(await sw.evaluate((t) => addSite('http://127.0.0.1/*', t), tabId), true);
    const reg = await sw.evaluate(() => chrome.scripting.getRegisteredContentScripts());
    assert.deepEqual(reg.map((r) => r.matches), [['http://127.0.0.1/*']]);
    await page.$eval('#ta', (e) => { e.value = ''; e.focus(); });
    await page.keyboard.type('choy shahar ', { delay: 30 });
    await page.waitForFunction(() => document.getElementById('ta').value === 'çoy şahar ', null, { timeout: 5000 });
    assert.equal(await page.$eval('#ta', (e) => e.selectionStart), 10);
    await page.keyboard.press('Backspace');
    assert.equal(await txt('#ta'), 'çoy shahar ');
    await page.keyboard.type('shahar.', { delay: 30 });
    await page.waitForTimeout(150);
    assert.equal(await txt('#ta'), 'çoy shahar shahar.');
    step('yozayotganda: "choy shahar " → "çoy şahar ", kursor joyida; ⌫ asl so‘zni qaytardi va u boshqa o‘girilmadi');

    await page.focus('#pw');
    await page.keyboard.type('shahar ', { delay: 20 });
    await page.waitForTimeout(150);
    assert.equal(await txt('#pw'), 'shahar ');
    await page.focus('#ce2');
    // "shahar" yuqorida ⌫ bilan qaytarilgan: shu sahifada boshqa o'girilmaydi, shuning uchun boshqa so'z
    await page.keyboard.type('chiroyli choyxona ', { delay: 30 });
    await page.waitForFunction(() => document.getElementById('ce2').textContent.replace(/ /g, ' ') === 'çiroyli çoyxona ', null, { timeout: 5000 })
      .catch(async (e) => { console.error('ce2:', JSON.stringify(await page.$eval('#ce2', (x) => [...x.childNodes].map((n) => n.nodeType + ':' + (n.data || n.outerHTML)))), JSON.stringify(await page.evaluate(() => { const s = getSelection(); return [s.anchorNode && s.anchorNode.nodeType, s.anchorOffset, s.anchorNode && s.anchorNode.data]; }))); throw e; });
    await page.keyboard.type('shahar', { delay: 30 });
    assert.equal((await txt('#ce2')).replace(/ /g, ' '), 'çiroyli çoyxona shahar');
    step('yozayotganda: parol maydoni tegilmadi; contenteditable "chiroyli choyxona " → "çiroyli çoyxona ", kursor oxirida');

    assert.equal(await sw.evaluate(() => removeSite('http://127.0.0.1/*')), true);
    assert.deepEqual(await sw.evaluate(() => chrome.scripting.getRegisteredContentScripts()), []);
    await page.$eval('#ta', (e) => { e.value = ''; e.focus(); });
    await page.keyboard.type('shahar ', { delay: 20 });
    await page.waitForTimeout(150);
    assert.equal(await txt('#ta'), 'shahar ');
    step('o‘chirilganda: ro‘yxatdan chiqdi va ochiq sahifada ham to‘xtadi');

    assert.deepEqual(external, []);
    step('tashqi tarmoq so‘rovi: 0');
    console.log('smoke: OK');
  } finally {
    await ctx.close();
    server.close();
    fs.rmSync(tmp, { recursive: true, force: true });
  }
})().catch((e) => { console.error(e); process.exit(1); });
