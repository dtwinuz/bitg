// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Engine.gs: qo'lda UTF-8, qayta yozish rejasi (planEdits_/mapPos_) va dist/Wasm.gs dagi haqiqiy yadro.
// dist yo'q bo'lsa yadro testlari o'tkazib yuboriladi (avval: node tools/build.mjs).
'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const { load, loadPure, haveDist, DIST } = require('./gas.cjs');

const pure = loadPure();

test('UTF-8: kodlash/ochish TextEncoder bilan bir xil (kirill, ʼ, emoji, juftsiz surrogat)', () => {
  const enc = new TextEncoder();
  const samples = ['', 'abc', 'oʻzbek gʻisht', 'Ассалому алайкум ўғқҳ', 'Özbekiston ŞĞÇ', '😀 ok 𝔸', 'a\ud800b', '\udfffx'];
  for (const s of samples) {
    const mine = Buffer.from(pure.bitgUtf8Encode_(s));
    assert.deepEqual(mine, Buffer.from(enc.encode(s)), JSON.stringify(s));
    assert.equal(pure.bitgUtf8Decode_(mine), new TextDecoder().decode(enc.encode(s)));
  }
  const big = 'şahar, çoy 😀 '.repeat(50000);
  assert.equal(pure.bitgUtf8Decode_(pure.bitgUtf8Encode_(big)), big);
});

test('flags: panel sozlamalari ffi.rs BITG_* bitlariga', () => {
  assert.equal(pure.bitgFlags_({}), 0);
  assert.equal(pure.bitgFlags_(undefined), 0);
  assert.equal(pure.bitgFlags_({ english: false }), 1);
  assert.equal(pure.bitgFlags_({ cyrillic: false }), 2);
  assert.equal(pure.bitgFlags_({ asciiTutuq: true }), 4);
  assert.equal(pure.bitgFlags_({ english: false, cyrillic: false, asciiTutuq: true }), 7);
});

const PAIRS = [
  ['shahar', 'şahar'],
  ['o‘zbek tili, shahar, choy', 'özbek tili, şahar, çoy'],
  ["O'zbekiston  shaharlari:\tToshkent\n", 'Özbekiston  şaharlari:\tToşkent\n'],
  ['Ассалому алайкум!', 'Assalomu alaykum!'],
  ['abc', 'abc'],
  ['', ''],
  ['bir ikki', 'bir  ikki'], // bo'sh joylar mos emas → bitta bo'lak
  ['x 😀sh', 'x 😀ş'],
  ['😀a', '😁a'], // surrogat juftining yarmi almashmaydi
  ['ёлка', 'yolka']
];

test('planEdits_: qo\'llangan reja aynan natijani beradi, bo\'laklar tartibli va kesishmaydi', () => {
  for (const [a, b] of PAIRS) {
    const edits = pure.planEdits_(a, b);
    assert.equal(pure.applyEdits_(a, edits), b, a);
    for (let i = 0; i < edits.length; i++) {
      assert.ok(edits[i].start <= edits[i].end);
      if (i) assert.ok(edits[i - 1].end <= edits[i].start);
      // surrogat juftlari bo'linmaydi
      const lo = (s, k) => k > 0 && k < s.length && /[\udc00-\udfff]/.test(s[k]);
      assert.ok(!lo(a, edits[i].start) && !lo(a, edits[i].end), JSON.stringify([a, edits[i]]));
    }
  }
  assert.deepEqual(JSON.parse(JSON.stringify(pure.planEdits_('ko‘rishamiz shu', 'körişamiz şu'))), [
    { start: 1, end: 7, text: 'öriş' },
    { start: 12, end: 14, text: 'ş' }
  ]);
});

test('mapPos_: formatlash chegaralari yangi matnga ko\'chadi', () => {
  // "bold shahar" — "shahar" qalin (6..12) → "bold şahar" da 5..10
  const a = 'bold shahar end';
  const edits = pure.planEdits_(a, 'bold şahar end');
  assert.equal(pure.mapPos_(edits, 0), 0);
  assert.equal(pure.mapPos_(edits, 5), 5);
  assert.equal(pure.mapPos_(edits, 11), 10);
  assert.equal(pure.mapPos_(edits, a.length), a.length - 1);
  // monoton
  let prev = -1;
  for (let p = 0; p <= a.length; p++) {
    const m = pure.mapPos_(edits, p);
    assert.ok(m >= prev);
    prev = m;
  }
  // sof qo'shish (start === end)
  const ins = [{ start: 2, end: 2, text: 'XY' }];
  assert.equal(pure.mapPos_(ins, 1), 1);
  assert.equal(pure.mapPos_(ins, 2), 2);
  assert.equal(pure.mapPos_(ins, 3), 5);
});

test('yadro (dist/Wasm.gs): sinxron yo\'l, eski lotin va kirill → yangi alifbo', { skip: !haveDist && `dist yo'q: ${DIST}` }, () => {
  const ctx = load();
  // Apps Script kabi: TextEncoder yo'q — qo'lda UTF-8 yo'li ishlaydi
  assert.equal(require('vm').runInContext('typeof TextEncoder + typeof TextDecoder', ctx), 'undefinedundefined');
  const t0 = Date.now();
  const core = ctx.bitgCoreSync_();
  assert.ok(core, String(ctx.BITG_CORE_ERROR_));
  const ms = Date.now() - t0;
  const cases = [
    ['o‘zbek tili, shahar, choy', 'özbek tili, şahar, çoy'],
    ["O'zbekiston shaharlari: Toshkent", 'Özbekiston şaharlari: Toşkent'],
    ['Oʻzbekiston gʻalabasi', 'Özbekiston ğalabasi'],
    ['O`zbekiston g`alabasi', 'Özbekiston ğalabasi'],
    ['Ассалому алайкум! Эртага чойхонада кўришамиз', 'Assalomu alaykum! Ertaga çoyxonada körişamiz'],
    ['Shu hafta push qilaman: https://chat.example.uz/shahar', 'Şu hafta push qilaman: https://chat.example.uz/shahar'],
    ['', '']
  ];
  for (const [src, want] of cases) assert.equal(core.convert(src), want, src);
  // flags: kirill o'chirilsa kirill qoladi
  assert.equal(core.convert('Тошкент shahar', 2), 'Тошкент şahar');
  console.log(`  yadro yuklash (base64+gunzip+compile): ${ms} ms`);
});

test('yadro: asinxron yo\'l (WebAssembly.instantiate) va bitgWithCore_', { skip: !haveDist }, async () => {
  const ctx = load();
  const core = await ctx.bitgCoreAsync_();
  assert.equal(core.convert('choy'), 'çoy');
  // sinxron yo'l rad etilsa — Promise qaytadi
  const ctx2 = load({ WebAssembly: { Module: function () { throw new RangeError('sync compile disabled'); }, instantiate: WebAssembly.instantiate, Instance: WebAssembly.Instance } });
  const r = ctx2.bitgWithCore_((c) => c.convert('shahar'));
  assert.equal(typeof r.then, 'function');
  assert.equal(await r, 'şahar');
});

test('yadro + reja: har xil matnda planEdits_ natijani aynan qayta quradi', { skip: !haveDist }, () => {
  const ctx = load();
  const core = ctx.bitgCoreSync_();
  const texts = [
    'O‘zbekiston Respublikasi Prezidentining qarori. Toshkent shahri, 2026-yil 14-noyabr.',
    'Ассалому алайкум, азиз дўстлар! Шаҳрисабз ва Чирчиқ шаҳарлари.',
    "Ko'cha, g'isht, so'z, ma'no, sh'ir — har xil apostrof: o`g`il, oʻgʻil, o’g’il.",
    'Mixed: Hello world, shop, push; Шахар; shahar.\n\tYangi qator  ikki  bo‘sh joy.',
    'Emoji 😀 va chorshanba 🎉, “qo‘shtirnoq” ichida.'
  ];
  for (const s of texts) {
    const out = core.convert(s);
    assert.notEqual(out, s);
    assert.equal(ctx.applyEdits_(s, ctx.planEdits_(s, out)), out, s);
  }
});

test('preview(): panelda sinab ko\'rish', { skip: !haveDist }, () => {
  const ctx = load();
  assert.deepEqual(JSON.parse(JSON.stringify(ctx.preview('Toshkent', {}))), { out: 'Toşkent' });
  assert.equal(ctx.preview('x'.repeat(30000), {}).out.length, 20000);
});
