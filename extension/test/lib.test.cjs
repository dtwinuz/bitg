// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const L = require('../content/lib.js');
const { h, FakeText, FakeDocument } = require('./fake-dom.cjs');

const texts = (nodes) => nodes.map((n) => n.data);

test('textNodes: ko\'rinadigan matn olinadi, kod/forma/skript/contenteditable chetda', () => {
  const body = h('body', {},
    h('h1', {}, 'Shahar ', h('b', {}, 'choyxona')),
    h('p', {}, 'O‘zbekiston'),
    h('script', {}, 'var sh = 1;'),
    h('style', {}, '.ch{}'),
    h('pre', {}, h('span', {}, 'shell')),
    h('code', {}, 'chmod'),
    h('textarea', {}, 'shahar'),
    h('select', {}, h('option', {}, 'Toshkent')),
    h('div', { contenteditable: 'true' }, h('p', {}, 'muharrir')),
    h('div', { translate: 'no' }, 'brend'),
    h('div', { class: 'card notranslate' }, 'nomi'),
    h('div', { class: 'monaco-editor' }, h('div', {}, 'kod')),
    h('div', { 'data-bitg-skip': '' }, 'xabarcha'),
    h('p', {}, '   '),
    h('p', {}, '123 — 45'),
    h('footer', {}, h('div', {}, h('span', {}, 'chuqur')), 'oxiri')
  );
  new FakeDocument(body);
  assert.deepEqual(texts(L.textNodes(body)), ['Shahar ', 'choyxona', 'O‘zbekiston', 'chuqur', 'oxiri']);
});

test('textNodes: ildizning o\'zi chetdagi element bo\'lsa — bo\'sh; matn tuguni ildiz bo\'lsa — o\'zi', () => {
  assert.deepEqual(L.textNodes(h('code', {}, 'shahar')), []);
  const t = new FakeText('choy');
  assert.deepEqual(L.textNodes(t), [t]);
  assert.deepEqual(L.textNodes(new FakeText('  42 ')), []);
  assert.deepEqual(L.textNodes(null), []);
});

test('textNodes: contenteditable="false" ichi o\'giriladi, ichidagi editable chetda', () => {
  const body = h('body', {}, h('div', { contenteditable: 'false' }, 'ko‘rinish', h('span', { contenteditable: '' }, 'tahrir')));
  assert.deepEqual(texts(L.textNodes(body)), ['ko‘rinish']);
});

test('acceptTextNode / insideSkipped: MutationObserver qo\'shgan tugunlar ota-bobosi bilan tekshiriladi', () => {
  const inCode = new FakeText('shahar');
  const ok = new FakeText('shahar');
  const body = h('body', {}, h('pre', {}, h('span', {}, inCode)), h('div', {}, h('span', {}, ok)));
  new FakeDocument(body);
  assert.equal(L.acceptTextNode(inCode), false);
  assert.equal(L.acceptTextNode(ok), true);
  assert.equal(L.insideSkipped(ok.parentNode), false);
  assert.equal(L.acceptTextNode(new FakeText('  ')), false);
  assert.equal(L.acceptTextNode(h('p', {})), false);
});

test('findWordBefore: probel/tinish belgisidan oldingi so\'z, apostrof so\'z ichida', () => {
  assert.deepEqual(L.findWordBefore('o‘zbek ', 7), { start: 0, end: 6, word: 'o‘zbek', trigger: ' ' });
  assert.deepEqual(L.findWordBefore('salom shahar,', 13), { start: 6, end: 12, word: 'shahar', trigger: ',' });
  assert.deepEqual(L.findWordBefore("tog' ", 5), { start: 0, end: 4, word: "tog'", trigger: ' ' });
  assert.deepEqual(L.findWordBefore('ma’no.', 6), { start: 0, end: 5, word: 'ma’no', trigger: '.' });
  assert.deepEqual(L.findWordBefore('чойхона ', 8), { start: 0, end: 7, word: 'чойхона', trigger: ' ' });
  // ochuvchi qo'shtirnoq so'zga kirmaydi
  assert.deepEqual(L.findWordBefore("'shahar' ", 9), { start: 1, end: 8, word: "shahar'", trigger: ' ' });
  assert.deepEqual(L.findWordBefore('"choy" ', 6), { start: 1, end: 5, word: 'choy', trigger: '"' });
  // yangi qator ham trigger (textarea Enter)
  assert.equal(L.findWordBefore('choy\n', 5).word, 'choy');
});

test('findWordBefore: trigger yo\'q, havola, pochta, raqam, teg — null', () => {
  assert.equal(L.findWordBefore('shahar', 6), null);
  assert.equal(L.findWordBefore("o'", 2), null);
  assert.equal(L.findWordBefore(' ', 1), null);
  assert.equal(L.findWordBefore('  ', 2), null);
  assert.equal(L.findWordBefore('example.com ', 12), null);
  assert.equal(L.findWordBefore('ali@pochta ', 11), null);
  assert.equal(L.findWordBefore('3chi ', 5), null);
  assert.equal(L.findWordBefore('#shahar ', 8), null);
  assert.equal(L.findWordBefore('/usr/share ', 11), null);
  assert.equal(L.findWordBefore("'' ", 3), null);
  assert.equal(L.findWordBefore('choy ', 99), null);
  assert.equal(L.findWordBefore(null, 3), null);
});

test('findWordBefore: kursor matn o\'rtasida', () => {
  const s = 'bir shahar qoldi';
  assert.deepEqual(L.findWordBefore(s, 11), { start: 4, end: 10, word: 'shahar', trigger: ' ' });
});

test('shiftCaret: almashtirilgan oraliqdan keyin, ichida, oldida', () => {
  // "shahar " → "şahar ": so'z 1 belgiga qisqardi
  assert.equal(L.shiftCaret(7, 0, 6, 5), 6);
  assert.equal(L.shiftCaret(20, 4, 6, 5), 19);
  assert.equal(L.shiftCaret(3, 0, 6, 5), 5);
  assert.equal(L.shiftCaret(2, 4, 6, 5), 2);
  assert.equal(L.shiftCaret(4, 4, 6, 5), 4);
  // uzayish (kirill "ш" → "ş" bir xil, lekin "ё" → "yo")
  assert.equal(L.shiftCaret(10, 0, 4, 6), 12);
});

test('stillMatches: javob kelguncha matn o\'zgarmagan bo\'lsa', () => {
  const w = L.findWordBefore('salom shahar ', 13);
  assert.equal(L.stillMatches('salom shahar ', w), true);
  assert.equal(L.stillMatches('salom shahar va', w), true);
  assert.equal(L.stillMatches('salom shaha', w), false);
  assert.equal(L.stillMatches('salom shahar,', w), false);
  assert.equal(L.stillMatches('x salom shahar ', w), false);
});

test('relocate: oldingi so\'z kech almashib matn siljiganda so\'z qayta topiladi', () => {
  const w = L.findWordBefore('chiroyli shahar ', 16); // {start: 9}
  assert.equal(L.relocate('chiroyli shahar ', w, 16), w);
  assert.deepEqual(L.relocate('çiroyli shahar ', w, 15), { start: 8, end: 14, word: 'shahar', trigger: ' ' });
  assert.deepEqual(L.relocate('çiroyli shahar va', w, 17), { start: 8, end: 14, word: 'shahar', trigger: ' ' });
  // so'z tahrir qilingan yoki boshqa so'zning bir qismi — null
  assert.equal(L.relocate('çiroyli shaha ', w, 14), null);
  assert.equal(L.relocate('çiroyli ushahar ', w, 16), null);
  assert.equal(L.relocate('shahar ' + 'x'.repeat(100), w, 107), null);
});

test('undoRange: faqat "yangi so\'z + trigger" dan darhol keyin', () => {
  const last = { start: 6, conv: 'şahar', orig: 'shahar', trigger: ' ' };
  assert.deepEqual(L.undoRange('salom şahar ', 12, last), { start: 6, end: 11, text: 'shahar', caret: 13 });
  assert.equal(L.undoRange('salom şahar x', 13, last), null);
  assert.equal(L.undoRange('salom şahar ', 11, last), null);
  assert.equal(L.undoRange('salom sahar  ', 12, last), null);
  assert.equal(L.undoRange('salom şahar ', 12, null), null);
});

test('textPiece: belgilangan oraliqning tugundagi bo\'lagi', () => {
  assert.deepEqual(L.textPiece(10, true, 3, false, 0), { from: 3, to: 10 });
  assert.deepEqual(L.textPiece(10, false, 0, true, 4), { from: 0, to: 4 });
  assert.deepEqual(L.textPiece(10, true, 2, true, 6), { from: 2, to: 6 });
  assert.deepEqual(L.textPiece(10, false, 0, false, 0), { from: 0, to: 10 });
  assert.equal(L.textPiece(10, true, 10, false, 0), null);
  assert.equal(L.textPiece(10, true, 4, true, 4), null);
  assert.deepEqual(L.textPiece(5, true, 1, true, 99), { from: 1, to: 5 });
});

test('isTextControl / isAutotypeControl: parol, pochta, URL, readonly va kod muharriri chetda', () => {
  assert.equal(L.isTextControl(h('textarea', {})), true);
  assert.equal(L.isTextControl(h('input', {})), true);
  assert.equal(L.isTextControl(h('input', { type: 'search' })), true);
  for (const type of ['password', 'email', 'url', 'tel', 'number', 'hidden']) {
    assert.equal(L.isTextControl(h('input', { type })), false, type);
  }
  assert.equal(L.isTextControl(h('div', {})), false);
  assert.equal(L.isAutotypeControl(h('textarea', { readonly: '' })), false);
  assert.equal(L.isAutotypeControl(h('textarea', { disabled: '' })), false);
  for (const autocomplete of ['username', 'email', 'url', 'tel', 'tel-national', 'one-time-code', 'cc-number',
                              'section-login username']) {
    assert.equal(L.isAutotypeControl(h('input', { autocomplete })), false, autocomplete);
  }
  for (const inputmode of ['email', 'url', 'tel', 'numeric']) {
    assert.equal(L.isAutotypeControl(h('input', { inputmode })), false, inputmode);
  }
  assert.equal(L.isAutotypeControl(h('input', { autocomplete: 'name' })), true);
  assert.equal(L.isAutotypeControl(h('input', { inputmode: 'text' })), true);
  assert.equal(L.isTextControl(h('input', { autocomplete: 'username' })), true);
  const monaco = h('div', { class: 'monaco-editor' }, h('div', {}, h('textarea', {})));
  assert.equal(L.isAutotypeControl(monaco.firstChild.firstChild), false);
  const cm = h('div', { class: 'cm-editor' }, h('div', { class: 'cm-content', contenteditable: 'true' }));
  assert.equal(L.isAutotypeRich(cm.firstChild), false);
  assert.equal(L.isAutotypeRich(h('div', { contenteditable: 'true' })), true);
  assert.equal(L.isAutotypeRich(h('div', {})), false);
});

test('originPattern / restrictedUrl / canvasEditor', () => {
  assert.equal(L.originPattern('https://Kun.uz/news/1?x#y'), 'https://kun.uz/*');
  assert.equal(L.originPattern('http://localhost:8080/a'), 'http://localhost/*');
  assert.equal(L.originPattern('chrome://extensions'), null);
  assert.equal(L.originPattern('file:///C:/a.html'), null);
  assert.equal(L.restrictedUrl('chrome://newtab/'), true);
  assert.equal(L.restrictedUrl('about:addons'), true);
  assert.equal(L.restrictedUrl('https://chromewebstore.google.com/detail/x'), true);
  assert.equal(L.restrictedUrl('https://addons.mozilla.org/uz/firefox/'), true);
  assert.equal(L.restrictedUrl('https://kun.uz/'), false);
  assert.equal(L.canvasEditor('https://docs.google.com/document/d/abc/edit'), true);
  assert.equal(L.canvasEditor('https://docs.google.com/forms/d/abc'), false);
  assert.equal(L.canvasEditor('https://kun.uz/'), false);
});
