// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Haqiqiy bitg_core.wasm (sayt bilan bir xil yig'ilgan) core.js orqali: natija CLI va ci.yml Smoke bilan bir xil.
// wasm yo'q bo'lsa test o'tkazib yuboriladi (BITG_WASM=<yo'l> yoki target/site/... standart joy).
'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('fs');
const path = require('path');
const bitgCore = require('../core/core.js');

const WASM = process.env.BITG_WASM ||
  path.resolve(__dirname, '..', '..', 'target', 'site', 'wasm32-unknown-unknown', 'release', 'bitg_core.wasm');
const have = fs.existsSync(WASM);

test('yadro: eski lotin va kirill → yangi alifbo', { skip: !have && `wasm yo'q: ${WASM}` }, async () => {
  const core = await bitgCore.load(fs.readFileSync(WASM));
  const cases = [
    ['o‘zbek tili, shahar, choy', 'özbek tili, şahar, çoy'],
    ["O'zbekiston shaharlari: Toshkent", 'Özbekiston şaharlari: Toşkent'],
    ['Ассалому алайкум! Эртага чойхонада кўришамиз', 'Assalomu alaykum! Ertaga çoyxonada körişamiz'],
    ['shahar', 'şahar'],
    ['', '']
  ];
  for (const [src, want] of cases) assert.equal(core.convert(src), want, src);
});

test('yadro: havola, pochta va inglizcha so\'z himoyalanadi; toks sabablari', { skip: !have }, async () => {
  const core = await bitgCore.load(fs.readFileSync(WASM));
  const r = core.toNew('Shu hafta push qilaman: https://chat.example.uz/shahar');
  assert.ok(r.out.includes('https://chat.example.uz/shahar'), r.out);
  assert.ok(r.out.startsWith('Şu hafta push'), r.out);
  const reasons = Object.fromEntries(r.toks.map((k) => [k[0], k[1]]));
  assert.equal(reasons.Shu, 'c');
  assert.equal(reasons.push, 'e');
});

test('yadro: katta matn va ko\'p chaqiruv (xotira o\'sishi buferni eskirtirmaydi)', { skip: !have }, async () => {
  const core = await bitgCore.load(fs.readFileSync(WASM));
  const big = 'o‘zbek tili, shahar, choy. '.repeat(20000);
  const out = core.convert(big);
  assert.equal(out, 'özbek tili, şahar, çoy. '.repeat(20000));
  for (let i = 0; i < 2000; i++) assert.equal(core.convert('choy'), 'çoy');
});
