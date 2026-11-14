// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Code.gs qayta yozish mantig'i soxta Docs/Sheets/Slides obyektlarida. Soxtalar Apps Script hujjatidagi
// metod imzolariga mos (Text.insertText/deleteText/getAttributes/setAttributes, RichTextValue, TextRange.clear),
// lekin Google'ning haqiqiy xulqini (masalan, qo'shilgan matn formatini) to'liq takrorlamaydi — shuning uchun
// Docs'da har ikki meros qoidasi sinaladi. Haqiqiy tekshiruv — README dagi qo'lda sinov ro'yxati.
'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const { load, loadPure, haveDist } = require('./gas.cjs');

// Soddalashtirilgan o'giruvchi (yadro emas): faqat yozish mantig'ini sinash uchun
const fakeConv = (s) => s.replace(/sh/g, 'ş').replace(/Sh/g, 'Ş').replace(/ch/g, 'ç').replace(/o['‘ʻ]/g, 'ö').replace(/g['‘ʻ]/g, 'ğ')
  .replace(/Тошкент/g, 'Toşkent');

// ------------------------------------------------------------------------------------------- Docs soxtasi
const KEYS = ['BOLD', 'ITALIC', 'LINK_URL', 'FOREGROUND_COLOR'];
class FakeText {
  constructor(runs, inherit = 'prev') {
    this.chars = [];
    for (const [s, attrs] of runs) for (const ch of s) this.pushChars(ch, attrs);
    this.inherit = inherit;
  }
  pushChars(s, attrs) { for (let i = 0; i < s.length; i++) this.chars.push({ ch: s[i], a: Object.assign({}, attrs || {}) }); }
  getType() { return 'TEXT'; }
  asText() { return this; }
  getText() { return this.chars.map((c) => c.ch).join(''); }
  getAttributes(o) {
    const a = this.chars[o].a;
    const out = {};
    for (const k of KEYS) out[k] = k in a ? a[k] : null;
    return out;
  }
  setAttributes(s, e, attrs) {
    for (let i = s; i <= e; i++) for (const k in attrs) if (attrs[k] !== null) this.chars[i].a[k] = attrs[k];
  }
  insertText(o, t) {
    assert.ok(o >= 0 && o <= this.chars.length, 'insert offset');
    const from = this.inherit === 'prev' ? this.chars[o - 1] : this.chars[o];
    const a = from ? from.a : {};
    this.chars.splice(o, 0, ...Array.from(t, (ch) => ({ ch, a: Object.assign({}, a) })));
  }
  deleteText(s, e) {
    assert.ok(s >= 0 && e < this.chars.length && s <= e, 'delete range');
    this.chars.splice(s, e - s + 1);
    assert.ok(this.chars.length > 0, 'Text element emptied');
  }
  /** "so'z" → shu so'zdagi belgilar atributi (hammasi bir xilmi). */
  attrOf(word) {
    const i = this.getText().indexOf(word);
    assert.ok(i >= 0, word);
    const set = new Set(this.chars.slice(i, i + word.length).map((c) => JSON.stringify(c.a)));
    assert.equal(set.size, 1, `${word}: aralash format`);
    return JSON.parse([...set][0]);
  }
}
class FakeContainer {
  constructor(type, kids) { this.type = type; this.kids = kids; }
  getType() { return this.type; }
  getNumChildren() { return this.kids.length; }
  getChild(i) { return this.kids[i]; }
  asBody() { return this; }
  asParagraph() { return this; }
  asTable() { return this; }
  asTableRow() { return this; }
  asTableCell() { return this; }
  asListItem() { return this; }
  asHeaderSection() { return this; }
  asFooterSection() { return this; }
  asFootnoteSection() { return this; }
}

for (const inherit of ['prev', 'next']) {
  test(`Docs: faqat o'zgargan bo'lak almashadi, qalin/havola saqlanadi (insertText merosi: ${inherit})`, () => {
    const g = loadPure();
    const t = new FakeText([
      ['Biz ', {}],
      ['shahar', { BOLD: true }],
      [' va ', {}],
      ['choyxona', { LINK_URL: 'https://example.uz/' }],
      [" o'zbek", { ITALIC: true }]
    ], inherit);
    const n = g.bitgDocsText_(t, 0, undefined, fakeConv);
    assert.equal(t.getText(), 'Biz şahar va çoyxona özbek');
    assert.equal(n, 3);
    assert.deepEqual(t.attrOf('şahar'), { BOLD: true });
    assert.deepEqual(t.attrOf('çoyxona'), { LINK_URL: 'https://example.uz/' });
    assert.deepEqual(t.attrOf('özbek'), { ITALIC: true });
    assert.deepEqual(t.attrOf('Biz'), {});
  });
}

test('Docs: qisman tanlov — faqat [start, end] o\'giriladi; butun element — ichidagi barcha Text', () => {
  const g = loadPure();
  const t1 = new FakeText([['shahar shahar shahar', {}]]);
  const t2 = new FakeText([['choy', {}]]);
  const t3 = new FakeText([['ichki shox', {}]]);
  const table = new FakeContainer('TABLE', [new FakeContainer('TABLE_ROW', [new FakeContainer('TABLE_CELL', [new FakeContainer('PARAGRAPH', [t3])])])]);
  const doc = {
    getSelection: () => ({
      getRangeElements: () => [
        { getElement: () => t1, isPartial: () => true, getStartOffset: () => 7, getEndOffsetInclusive: () => 12 },
        { getElement: () => new FakeContainer('PARAGRAPH', [t2]), isPartial: () => false },
        { getElement: () => table, isPartial: () => false }
      ]
    })
  };
  const r = g.bitgDocsSelection_(doc, fakeConv);
  assert.equal(t1.getText(), 'shahar şahar shahar');
  assert.equal(t2.getText(), 'çoy');
  assert.equal(t3.getText(), 'içki şox');
  assert.equal(r.changed, 4);
  assert.equal(r.items, 3);
  assert.equal(g.bitgDocsSelection_({ getSelection: () => null }, fakeConv).error, 'no_selection');
});

test('Docs: butun hujjat — tablar (ichki tab bilan), sarlavha, pastki qism, izoh; mundarija tegilmaydi', () => {
  const g = loadPure();
  const mk = (s) => new FakeText([[s, {}]]);
  const body1 = mk('shahar'), head = mk('sharq'), foot = mk('choy'), note = mk("o'q"), body2 = mk('ish'), toc = mk('shu');
  const tab = (body, extra, kids) => ({
    getType: () => 'DOCUMENT_TAB',
    asDocumentTab: () => ({
      getBody: () => new FakeContainer('BODY_SECTION', [new FakeContainer('PARAGRAPH', [body]), new FakeContainer('TABLE_OF_CONTENTS', [toc])]),
      getHeader: () => extra && new FakeContainer('HEADER_SECTION', [new FakeContainer('PARAGRAPH', [head])]),
      getFooter: () => extra && new FakeContainer('FOOTER_SECTION', [new FakeContainer('PARAGRAPH', [foot])]),
      getFootnotes: () => (extra ? [{ getFootnoteContents: () => new FakeContainer('FOOTNOTE_SECTION', [new FakeContainer('PARAGRAPH', [note])]) }] : null)
    }),
    getChildTabs: () => kids || []
  });
  const doc = { getTabs: () => [tab(body1, true, [tab(body2, false)])] };
  const r = g.bitgDocsAll_(doc, fakeConv);
  assert.deepEqual([body1, head, foot, note, body2, toc].map((t) => t.getText()), ['şahar', 'şarq', 'çoy', 'öq', 'iş', 'shu']);
  assert.equal(r.changed, 5);
});

test('Docs: element to\'liq almashsa ham bo\'sh qolmaydi', () => {
  const g = loadPure();
  const t = new FakeText([['Тошкент', { BOLD: true }]]);
  g.bitgDocsText_(t, 0, undefined, fakeConv);
  assert.equal(t.getText(), 'Toşkent');
  assert.deepEqual(t.attrOf('Toşkent'), { BOLD: true });
});

// ----------------------------------------------------------------------------------------- Sheets soxtasi
function rich(text, runs) {
  return {
    getText: () => text,
    getRuns: () => (runs || [[0, text.length, { b: false }, null]]).map(([s, e, style, url]) => ({
      getStartIndex: () => s, getEndIndex: () => e, getTextStyle: () => style, getLinkUrl: () => url
    }))
  };
}
function builder() {
  const st = { styles: [], links: [] };
  const b = {
    setText(t) { st.text = t; return b; },
    setTextStyle(s, e, style) { assert.ok(s >= 0 && e <= st.text.length && s < e); st.styles.push([s, e, style]); return b; },
    setLinkUrl(s, e, url) { st.links.push([s, e, url]); return b; },
    build() { return st; }
  };
  return b;
}
function fakeRange(values, formulas, richs) {
  const written = {};
  return {
    written,
    getValues: () => values,
    getFormulas: () => formulas || values.map((r) => r.map(() => '')),
    getRichTextValues: () => richs || values.map((r) => r.map((v) => (typeof v === 'string' ? rich(v) : null))),
    getCell: (r, c) => ({ setRichTextValue: (v) => { written[`${r},${c}`] = v; } })
  };
}

test('Sheets: faqat matnli, formulasiz, o\'zgargan kataklar; son, sana, formula tegilmaydi', () => {
  const g = loadPure({ SpreadsheetApp: { newRichTextValue: builder } });
  const d = new Date(0);
  const range = fakeRange(
    [['shahar', 42, d], ['=x', 'abc', 'choy'], ['', 'Тошкент', 'sharh']],
    [['', '', ''], ['', '', ''], ['', '', '=CONCAT("sh","arh")']]
  );
  const r = g.bitgSheetsRange_(range, fakeConv);
  assert.deepEqual(Object.keys(range.written).sort(), ['1,1', '2,3', '3,2']);
  assert.equal(range.written['1,1'].text, 'şahar');
  assert.equal(range.written['2,3'].text, 'çoy');
  assert.equal(range.written['3,2'].text, 'Toşkent');
  assert.equal(r.cells, 3);
});

test('Sheets: katak ichidagi format bo\'laklari va havola yangi koordinatalarga ko\'chadi', () => {
  const g = loadPure({ SpreadsheetApp: { newRichTextValue: builder } });
  const v = 'oddiy shahar havola';
  const range = fakeRange([[v]], null, [[rich(v, [[0, 6, 'plain', null], [6, 12, 'bold', null], [12, 13, 'plain', null], [13, 19, 'plain', 'https://bitg.uz/']])]]);
  g.bitgSheetsRange_(range, fakeConv);
  const w = range.written['1,1'];
  assert.equal(w.text, 'oddiy şahar havola');
  assert.deepEqual(w.styles, [[0, 6, 'plain'], [6, 11, 'bold'], [11, 12, 'plain'], [12, 18, 'plain']]);
  assert.deepEqual(w.links, [[12, 18, 'https://bitg.uz/']]);
  assert.equal(w.text.slice(6, 11), 'şahar');
  assert.equal(w.text.slice(12, 18), 'havola');
});

test('Sheets: tanlov (bir nechta diapazon) va bo\'sh tanlov', () => {
  const g = loadPure({ SpreadsheetApp: { newRichTextValue: builder } });
  const a = fakeRange([['shu']]), b = fakeRange([['choy', 'ish']]);
  const r = g.bitgSheetsSelection_({ getActiveRangeList: () => ({ getRanges: () => [a, b] }) }, fakeConv);
  assert.equal(r.items, 3);
  const none = g.bitgSheetsSelection_({ getActiveRangeList: () => null, getActiveRange: () => null }, fakeConv);
  assert.equal(none.error, 'no_selection');
});

// ----------------------------------------------------------------------------------------- Slides soxtasi
class FakeShapeText {
  constructor(s) { this.s = s; }
  range() {
    const self = this;
    return {
      asString: () => self.s,
      insertText(o, t) { assert.ok(o >= 0 && o <= self.s.length); self.s = self.s.slice(0, o) + t + self.s.slice(o); },
      clear(a, b) { assert.ok(a >= 0 && b <= self.s.length && a < b); self.s = self.s.slice(0, a) + self.s.slice(b); }
    };
  }
}
const shape = (txt) => ({ getPageElementType: () => 'SHAPE', asShape: () => ({ getText: () => txt.range() }) });

test('Slides: shakl, jadval, guruh, ma\'ruzachi izohi; qisman matn tanlovi', () => {
  const g = loadPure();
  const s1 = new FakeShapeText('shahar\n'), cellT = new FakeShapeText('choy\n'), inG = new FakeShapeText("o'q\n"), notes = new FakeShapeText('shu\n');
  const table = { getPageElementType: () => 'TABLE', asTable: () => ({ getNumRows: () => 1, getNumColumns: () => 1, getCell: () => ({ getText: () => cellT.range() }) }) };
  const group = { getPageElementType: () => 'GROUP', asGroup: () => ({ getChildren: () => [shape(inG)] }) };
  const slide = {
    getPageElements: () => [shape(s1), table, group],
    getNotesPage: () => ({ getSpeakerNotesShape: () => ({ getText: () => notes.range() }) })
  };
  const r = g.bitgSlidesAll_({ getSlides: () => [slide] }, fakeConv);
  assert.deepEqual([s1.s, cellT.s, inG.s, notes.s], ['şahar\n', 'çoy\n', 'öq\n', 'şu\n']);
  assert.equal(r.changed, 4);

  const part = new FakeShapeText('shahar shahar\n');
  const sel = {
    getSelectionType: () => 'TEXT',
    getTextRange: () => ({ getStartIndex: () => 7, getEndIndex: () => 13 }),
    getPageElementRange: () => ({ getPageElements: () => [shape(part)] }),
    getTableCellRange: () => null
  };
  g.bitgSlidesSelection_({ getSelection: () => sel }, fakeConv);
  assert.equal(part.s, 'shahar şahar\n');
  assert.equal(g.bitgSlidesSelection_({ getSelection: () => ({ getSelectionType: () => 'NONE' }) }, fakeConv).error, 'no_selection');
});

// -------------------------------------------------------------------- uchidan-uchiga: haqiqiy yadro + soxta Docs
test('convertAll(): haqiqiy yadro bilan soxta Google Docs hujjati', { skip: !haveDist }, () => {
  const t = new FakeText([['Ассалому алайкум! ', {}], ["O'zbekiston", { BOLD: true }], [' shaharlari: Toshkent, push.', {}]]);
  const docApp = {
    getUi: () => ({}),
    getActiveDocument: () => ({ getTabs: undefined, getBody: () => new FakeContainer('BODY_SECTION', [new FakeContainer('PARAGRAPH', [t])]), getHeader: () => null, getFooter: () => null, getFootnotes: () => [] })
  };
  const ctx = load({ DocumentApp: docApp });
  const r = ctx.convertAll({});
  assert.equal(t.getText(), 'Assalomu alaykum! Özbekiston şaharlari: Toşkent, push.');
  assert.deepEqual(t.attrOf('Özbekiston'), { BOLD: true });
  assert.equal(r.host, 'docs');
  assert.ok(r.changed >= 4);
});

test('convertSelection(): Sheets xostida (DocumentApp.getUi xato beradi)', { skip: !haveDist }, () => {
  const range = fakeRange([['Toshkent shahri', 7]]);
  const ctx = load({
    DocumentApp: { getUi: () => { throw new Error('not docs'); } },
    SpreadsheetApp: { getUi: () => ({}), newRichTextValue: builder, getActiveSpreadsheet: () => ({ getActiveRangeList: () => ({ getRanges: () => [range] }) }) }
  });
  const r = ctx.convertSelection({});
  assert.equal(r.host, 'sheets');
  assert.equal(range.written['1,1'].text, 'Toşkent şahri');
});
