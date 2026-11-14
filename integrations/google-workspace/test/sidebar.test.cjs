// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Sidebar.html skripti soxta `document` va soxta google.script.run bilan: til tanlash, matnlar, sozlamalar,
// server chaqiruvlari va natija xabarlari. Haqiqiy panel ko'rinishi faqat Google ichida tekshiriladi.
'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const HTML = fs.readFileSync(path.join(__dirname, '..', 'src', 'Sidebar.html'), 'utf8');
const SCRIPT = HTML.match(/<script\b[^>]*>([\s\S]*?)<\/script\s*>/i)[1];

/** HTML dagi id va data-* atributli elementlardan eng kichik soxta DOM. */
function fakeDom() {
  const els = [];
  const re = /<(\w+)([^>]*)>/g;
  let m;
  while ((m = re.exec(HTML))) {
    const attrs = {};
    m[2].replace(/([\w-]+)(?:="([^"]*)")?/g, (_, k, v) => { attrs[k] = v === undefined ? '' : v; });
    const listeners = {};
    els.push({
      tag: m[1], attrs, textContent: '', className: '', disabled: false, value: '',
      checked: 'checked' in attrs,
      getAttribute: (k) => (k in attrs ? attrs[k] : null),
      setAttribute: (k, v) => { attrs[k] = String(v); },
      addEventListener: (ev, fn) => { (listeners[ev] = listeners[ev] || []).push(fn); },
      fire(ev) { for (const fn of listeners[ev] || []) fn({ currentTarget: this }); }
    });
  }
  const html = els.find((e) => e.tag === 'html');
  return {
    documentElement: html,
    getElementById: (id) => els.find((e) => e.attrs.id === id) || null,
    querySelectorAll: (sel) => {
      const k = sel.match(/^\[([\w-]+)\]$/)[1];
      return els.filter((e) => k in e.attrs);
    }
  };
}

function boot({ host = 'docs', language = 'uz-UZ', saved = {}, reply } = {}) {
  const ctx = {};
  vm.createContext(ctx);
  vm.runInContext(SCRIPT, ctx); // `google` yo'q — avtomatik ishga tushmaydi
  const doc = fakeDom();
  const calls = [];
  const store = Object.assign({}, saved);
  const run = (fn, args, ok, err) => {
    calls.push([fn, JSON.parse(JSON.stringify(args))]);
    if (fn === 'hostInfo') return ok({ host });
    const r = reply ? reply(fn, args) : { changed: 2 };
    if (r instanceof Error) err(r);
    else ok(r);
  };
  const sb = ctx.bitgSidebar({
    document: doc, run, language,
    storage: { get: (k) => (k in store ? store[k] : null), set: (k, v) => { store[k] = v; } }
  });
  sb.init();
  return { ctx, doc, calls, store, sb };
}

test('panel: uz standart, server xostiga qarab "butun" tugmasi', () => {
  const { doc, sb } = boot({ host: 'sheets' });
  assert.equal(doc.getElementById('sel').textContent, 'Belgilanganni o‘girish');
  assert.equal(doc.getElementById('all').textContent, 'Butun varaqni o‘girish');
  assert.equal(doc.documentElement.getAttribute('lang'), 'uz');
  assert.equal(sb.state().host, 'sheets');
  // barcha data-i18n kalitlari har uch tilda bor
  for (const lang of ['uz', 'ru', 'en']) {
    const { doc: d } = boot({ language: lang, host: 'slides' });
    for (const el of d.querySelectorAll('[data-i18n]')) assert.ok(el.textContent, `${lang}: ${el.getAttribute('data-i18n')}`);
    assert.ok(d.getElementById('all').textContent);
  }
});

test('panel: brauzer tili, saqlangan til va til tugmalari', () => {
  assert.equal(boot({ language: 'ru-RU' }).sb.state().lang, 'ru');
  assert.equal(boot({ language: 'en-GB' }).sb.state().lang, 'en');
  assert.equal(boot({ language: 'de-DE' }).sb.state().lang, 'uz');
  assert.equal(boot({ language: 'ru', saved: { lang: 'en' } }).sb.state().lang, 'en');
  const { doc, sb, store } = boot();
  doc.querySelectorAll('[data-lang]').find((b) => b.getAttribute('data-lang') === 'ru').fire('click');
  assert.equal(sb.state().lang, 'ru');
  assert.equal(store.lang, 'ru');
  assert.equal(doc.getElementById('sel').textContent, 'Преобразовать выделенное');
});

test('panel: sozlamalar serverga uzatiladi va eslab qolinadi', () => {
  const { doc, calls, store } = boot({ saved: { optEn: '0' } });
  const tut = doc.getElementById('optTutuq');
  tut.checked = true;
  tut.fire('change');
  assert.equal(store.optTutuq, '1');
  doc.getElementById('sel').fire('click');
  doc.getElementById('all').fire('click');
  assert.deepEqual(calls.slice(1), [
    ['convertSelection', [{ cyrillic: true, english: false, asciiTutuq: true }]],
    ['convertAll', [{ cyrillic: true, english: false, asciiTutuq: true }]]
  ]);
  assert.equal(doc.getElementById('status').textContent, 'Tayyor: 2 ta joy o‘zgartirildi.');
  assert.equal(doc.getElementById('status').className, 'ok');
});

test('panel: natija xabarlari (belgilanmagan, o\'zgarmadi, xato) va sinab ko\'rish', () => {
  const replies = { convertSelection: { error: 'no_selection', changed: 0 }, convertAll: { changed: 0 }, preview: { out: 'Toşkent' } };
  const { doc } = boot({ reply: (fn) => replies[fn] });
  doc.getElementById('sel').fire('click');
  assert.equal(doc.getElementById('status').textContent, 'Avval matn (yoki kataklar) ni belgilang.');
  assert.equal(doc.getElementById('status').className, 'err');
  doc.getElementById('all').fire('click');
  assert.equal(doc.getElementById('status').textContent, 'O‘giriladigan matn topilmadi.');
  doc.getElementById('src').value = 'Toshkent';
  doc.getElementById('prev').fire('click');
  assert.equal(doc.getElementById('out').textContent, 'Toşkent');
  assert.equal(doc.getElementById('prev').disabled, false);

  const { doc: d2 } = boot({ reply: () => new Error('Exceeded maximum execution time') });
  d2.getElementById('all').fire('click');
  assert.equal(d2.getElementById('status').textContent, 'Xato: Exceeded maximum execution time');
  assert.equal(d2.getElementById('all').disabled, false);
});
