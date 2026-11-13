// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Manifest, tarjimalar va ZIP: do'kon cheklovlari va ruxsatlar ro'yxati o'zgarmasin.
'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('fs');
const path = require('path');
const zlib = require('zlib');
const I = require('../i18n.js');

const EXT = path.resolve(__dirname, '..');
const manifest = JSON.parse(fs.readFileSync(path.join(EXT, 'manifest.json'), 'utf8'));
const locales = Object.fromEntries(['uz', 'ru', 'en'].map((l) =>
  [l, JSON.parse(fs.readFileSync(path.join(EXT, '_locales', l, 'messages.json'), 'utf8'))]));

test('manifest: MV3, minimal ruxsatlar, tarmoqsiz CSP', () => {
  assert.equal(manifest.manifest_version, 3);
  assert.deepEqual([...manifest.permissions].sort(), ['activeTab', 'clipboardWrite', 'contextMenus', 'scripting', 'storage']);
  assert.deepEqual(manifest.optional_host_permissions, ['https://*/*', 'http://*/*']);
  assert.equal(manifest.host_permissions, undefined);
  assert.equal(manifest.content_scripts, undefined);
  assert.equal(manifest.web_accessible_resources, undefined);
  const csp = manifest.content_security_policy.extension_pages;
  assert.match(csp, /script-src 'self' 'wasm-unsafe-eval';/);
  assert.match(csp, /object-src 'self'/);
  assert.match(csp, /connect-src 'self'/);
  assert.equal(manifest.default_locale, 'uz');
});

test('_locales: har __MSG_x__ har tilda bor, do\'kon chegaralari (nom ≤45, tavsif ≤132)', () => {
  const keys = [...JSON.stringify(manifest).matchAll(/__MSG_(\w+)__/g)].map((m) => m[1]);
  assert.ok(keys.length >= 4);
  for (const [lang, msgs] of Object.entries(locales)) {
    for (const k of keys) assert.ok(msgs[k] && msgs[k].message, `${lang}: ${k}`);
    assert.ok(msgs.extName.message.length <= 45, `${lang} extName ${msgs.extName.message.length}`);
    assert.ok(msgs.extDescription.message.length <= 132, `${lang} extDescription ${msgs.extDescription.message.length}`);
  }
});

test('i18n.js: ru va en da uz dagi har kalit, {o\'rinbosar}lar bir xil', () => {
  const uz = I.STRINGS.uz;
  for (const lang of ['ru', 'en']) {
    for (const k of Object.keys(uz)) {
      assert.ok(typeof I.STRINGS[lang][k] === 'string', `${lang}.${k}`);
      const ph = (s) => (s.match(/\{\w+\}/g) || []).sort().join();
      assert.equal(ph(I.STRINGS[lang][k]), ph(uz[k]), `${lang}.${k}`);
    }
  }
  assert.equal(I.t('xx', 'copy'), uz.copy);
  assert.equal(I.t('en', 'converted', { n: 3 }), '3 words converted');
  assert.ok(Object.keys(I.toasts('ru')).every((k) => k.startsWith('toast')));
});

test('o\'zbekcha matn: apostrof o‘ g‘ = U+2018, ASCII apostrof yo\'q', () => {
  for (const s of Object.values(I.STRINGS.uz)) assert.ok(!s.includes("'"), s);
  for (const m of Object.values(locales.uz)) assert.ok(!m.message.includes("'"), m.message);
});

test('zip: takrorlanuvchan va standart o\'quvchi ochadi', async () => {
  const { zip, crc32 } = await import('../tools/zip.mjs');
  assert.equal(crc32(Buffer.from('123456789')), 0xcbf43926);
  const entries = [{ name: 'b.txt', data: Buffer.from('salom '.repeat(100)) }, { name: 'a/x.bin', data: Buffer.from([1, 2, 3]) }];
  const z1 = zip(entries), z2 = zip([...entries].reverse());
  assert.ok(z1.equals(z2));
  // markaziy katalogdan fayllarni o'qib, mazmunini tiklaymiz
  const eocd = z1.length - 22;
  assert.equal(z1.readUInt32LE(eocd), 0x06054b50);
  const count = z1.readUInt16LE(eocd + 10);
  let p = z1.readUInt32LE(eocd + 16);
  const got = {};
  for (let i = 0; i < count; i++) {
    const method = z1.readUInt16LE(p + 10), csize = z1.readUInt32LE(p + 20), nlen = z1.readUInt16LE(p + 28);
    const off = z1.readUInt32LE(p + 42), name = z1.toString('utf8', p + 46, p + 46 + nlen);
    const lnlen = z1.readUInt16LE(off + 26);
    const body = z1.subarray(off + 30 + lnlen, off + 30 + lnlen + csize);
    got[name] = method === 8 ? zlib.inflateRawSync(body) : body;
    p += 46 + nlen;
  }
  assert.deepEqual(Object.keys(got), ['a/x.bin', 'b.txt']);
  assert.ok(got['b.txt'].equals(entries[0].data));
  assert.ok(got['a/x.bin'].equals(entries[1].data));
});
