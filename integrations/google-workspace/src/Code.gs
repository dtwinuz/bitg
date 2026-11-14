// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * @OnlyCurrentDoc
 *
 * bitg — yangi o'zbek alifbosi: Google Docs, Sheets va Slides uchun Editor add-on.
 * Eski lotin (oʻ gʻ sh ch, har xil apostrof) va kirill matnni yangi alifboga (Ö Ğ Ş Ç) o'giradi.
 *
 * Yadro — Engine.gs (bitg_core.wasm shu ijro ichida). Bu fayl: menyu, panel va hujjatga qayta yozish.
 * Faqat o'zgargan bo'laklar almashtiriladi (planEdits_), shuning uchun formatlash, havolalar va izohlar qoladi.
 * Ochiq funksiyalar (pastki chiziqsiz) — menyu va google.script.run uchun; qolganlari xususiy (`_`).
 */

var BITG_I18N_ = {
  uz: {
    menuSel: 'Belgilanganni yangi alifboga o‘girish',
    menuAll: 'Butun hujjatni o‘girish',
    menuAllSheet: 'Butun varaqni o‘girish',
    menuAllSlides: 'Butun taqdimotni o‘girish',
    menuPanel: 'Panelni ochish',
    done: 'Tayyor: {n} ta joy o‘zgartirildi.',
    none: 'O‘giriladigan matn topilmadi.',
    noSel: 'Avval matn (yoki kataklar) ni belgilang.'
  },
  ru: {
    menuSel: 'Выделенное — в новый алфавит',
    menuAll: 'Весь документ',
    menuAllSheet: 'Весь лист',
    menuAllSlides: 'Вся презентация',
    menuPanel: 'Открыть панель',
    done: 'Готово: изменено мест — {n}.',
    none: 'Текст для преобразования не найден.',
    noSel: 'Сначала выделите текст (или ячейки).'
  },
  en: {
    menuSel: 'Convert selection to the new alphabet',
    menuAll: 'Convert whole document',
    menuAllSheet: 'Convert whole sheet',
    menuAllSlides: 'Convert whole presentation',
    menuPanel: 'Open panel',
    done: 'Done: {n} place(s) changed.',
    none: 'No text to convert was found.',
    noSel: 'Select some text (or cells) first.'
  }
};

function bitgLang_() {
  var loc = '';
  try {
    loc = String(Session.getActiveUserLocale() || '');
  } catch (e) {
    // AuthMode.NONE yoki cheklangan kontekst — standart til
  }
  loc = loc.toLowerCase();
  if (loc.indexOf('ru') === 0) return 'ru';
  if (loc.indexOf('en') === 0) return 'en';
  return 'uz';
}

function bitgT_(key, n) {
  var s = (BITG_I18N_[bitgLang_()] || BITG_I18N_.uz)[key];
  return n === undefined ? s : s.replace('{n}', String(n));
}

/** Qaysi muharrir ichida ishlayapmiz: 'docs' | 'sheets' | 'slides'. */
function bitgHost_() {
  try {
    DocumentApp.getUi();
    return 'docs';
  } catch (e) { /* Docs emas */ }
  try {
    SpreadsheetApp.getUi();
    return 'sheets';
  } catch (e) { /* Sheets emas */ }
  SlidesApp.getUi();
  return 'slides';
}

function bitgUi_(host) {
  if (host === 'docs') return DocumentApp.getUi();
  if (host === 'sheets') return SpreadsheetApp.getUi();
  return SlidesApp.getUi();
}

// ------------------------------------------------------------------------------------------------ menyu

function onOpen(e) {
  var host = bitgHost_();
  var ui = bitgUi_(host);
  var all = host === 'sheets' ? 'menuAllSheet' : host === 'slides' ? 'menuAllSlides' : 'menuAll';
  ui.createAddonMenu()
    .addItem(bitgT_('menuSel'), 'menuConvertSelection')
    .addItem(bitgT_(all), 'menuConvertAll')
    .addSeparator()
    .addItem(bitgT_('menuPanel'), 'showSidebar')
    .addToUi();
}

function onInstall(e) {
  onOpen(e);
}

function showSidebar() {
  var html = HtmlService.createHtmlOutputFromFile('Sidebar').setTitle('bitg — yangi alifbo');
  bitgUi_(bitgHost_()).showSidebar(html);
}

function menuConvertSelection() {
  return bitgToast_(convertSelection({}));
}

function menuConvertAll() {
  return bitgToast_(convertAll({}));
}

/** Menyudan chaqirilganda natijani foydalanuvchiga ko'rsatadi. */
function bitgToast_(res) {
  if (res && typeof res.then === 'function') return res.then(bitgToast_);
  var msg = res.error === 'no_selection' ? bitgT_('noSel') : res.error ? res.error : res.changed ? bitgT_('done', res.changed) : bitgT_('none');
  var host = res.host || bitgHost_();
  if (host === 'sheets') SpreadsheetApp.getActiveSpreadsheet().toast(msg, 'bitg', 5);
  else bitgUi_(host).alert(msg);
  return res;
}

// ------------------------------------------------------------------------------------------ panel API

/** Panel uchun: muharrir turi. */
function hostInfo() {
  return { host: bitgHost_() };
}

/** Panelda sinab ko'rish: matnni o'giradi (hujjatga tegmaydi). */
function preview(text, opts) {
  var s = String(text || '').slice(0, 20000);
  return bitgWithCore_(function (core) {
    return { out: core.convert(s, bitgFlags_(opts)) };
  });
}

/** Belgilangan matn / kataklar / shakllarni o'giradi. */
function convertSelection(opts) {
  var host = bitgHost_();
  var flags = bitgFlags_(opts);
  return bitgWithCore_(function (core) {
    var conv = function (s) { return core.convert(s, flags); };
    var r = host === 'docs' ? bitgDocsSelection_(DocumentApp.getActiveDocument(), conv)
      : host === 'sheets' ? bitgSheetsSelection_(SpreadsheetApp.getActiveSpreadsheet(), conv)
        : bitgSlidesSelection_(SlidesApp.getActivePresentation(), conv);
    r.host = host;
    return r;
  });
}

/** Butun hujjat (Docs: barcha tablar, sarlavha/pastki qism, izohlar) / joriy varaq / butun taqdimot. */
function convertAll(opts) {
  var host = bitgHost_();
  var flags = bitgFlags_(opts);
  return bitgWithCore_(function (core) {
    var conv = function (s) { return core.convert(s, flags); };
    var r = host === 'docs' ? bitgDocsAll_(DocumentApp.getActiveDocument(), conv)
      : host === 'sheets' ? bitgSheetsAll_(SpreadsheetApp.getActiveSpreadsheet(), conv)
        : bitgSlidesAll_(SlidesApp.getActivePresentation(), conv);
    r.host = host;
    return r;
  });
}

/** Tashxis: Apps Script loyihasida bir marta qo'lda ishga tushiring (Executions jurnaliga yoziladi). */
function bitgDiagnose() {
  var t0 = Date.now();
  var bytes = bitgWasmBytes_();
  var t1 = Date.now();
  var sync = bitgCoreSync_();
  var t2 = Date.now();
  var info = {
    webAssembly: typeof WebAssembly,
    textEncoder: typeof TextEncoder,
    wasmBytes: bytes.length,
    decodeMs: t1 - t0,
    syncCompileMs: t2 - t1,
    sync: !!sync,
    syncError: BITG_CORE_ERROR_ ? String(BITG_CORE_ERROR_) : null
  };
  if (sync) info.sample = sync.convert('O‘zbekiston shaharlari: Toshkent, Ассалому алайкум');
  console.log(JSON.stringify(info));
  return info;
}

// ------------------------------------------------------------------------------------------------ Docs

/** Bitta Text elementning [start, endExclusive) qismini o'giradi; o'zgargan joylar sonini qaytaradi. */
function bitgDocsText_(text, start, end, conv) {
  var full = text.getText();
  if (end === undefined) end = full.length;
  var src = full.substring(start, end);
  if (!src) return 0;
  var edits = planEdits_(src, conv(src));
  for (var i = edits.length - 1; i >= 0; i--) {
    var e = edits[i];
    var at = start + e.start;
    var del = e.end - e.start;
    // o'chirilayotgan birinchi belgi formatini olamiz (sof qo'shishda — oldingi belgi)
    var src0 = del > 0 ? at : Math.max(at - 1, 0);
    var attrs = full.length ? bitgDefined_(text.getAttributes(src0)) : null;
    if (e.text) {
      // avval qo'shamiz, keyin o'chiramiz: element hech qachon bo'sh qolmaydi
      text.insertText(at, e.text);
      if (attrs) text.setAttributes(at, at + e.text.length - 1, attrs);
    }
    if (del > 0) text.deleteText(at + e.text.length, at + e.text.length + del - 1);
  }
  return edits.length;
}

/** getAttributes() dagi null qiymatlar — "o'rnatilmagan"; ularni qayta yozmaymiz. */
function bitgDefined_(attrs) {
  var out = {};
  var any = false;
  for (var k in attrs) {
    if (attrs[k] !== null && attrs[k] !== undefined) {
      out[k] = attrs[k];
      any = true;
    }
  }
  return any ? out : null;
}

var BITG_DOCS_CAST_ = {
  BODY_SECTION: 'asBody',
  HEADER_SECTION: 'asHeaderSection',
  FOOTER_SECTION: 'asFooterSection',
  FOOTNOTE_SECTION: 'asFootnoteSection',
  PARAGRAPH: 'asParagraph',
  LIST_ITEM: 'asListItem',
  TABLE: 'asTable',
  TABLE_ROW: 'asTableRow',
  TABLE_CELL: 'asTableCell'
};

/** Element ichidagi barcha Text elementlar (mundarija o'tkazib yuboriladi — u sarlavhalardan qayta quriladi). */
function bitgDocsTexts_(el, out) {
  var type = String(el.getType());
  if (type === 'TEXT') {
    out.push(el.asText());
    return out;
  }
  var cast = BITG_DOCS_CAST_[type];
  if (!cast) return out;
  var c = el[cast]();
  for (var i = 0; i < c.getNumChildren(); i++) bitgDocsTexts_(c.getChild(i), out);
  return out;
}

function bitgDocsSelection_(doc, conv) {
  var sel = doc.getSelection();
  if (!sel) return { error: 'no_selection', changed: 0 };
  var changed = 0;
  var items = 0;
  var res = sel.getRangeElements();
  for (var i = 0; i < res.length; i++) {
    var re = res[i];
    var el = re.getElement();
    if (re.isPartial()) {
      changed += bitgDocsText_(el.asText(), re.getStartOffset(), re.getEndOffsetInclusive() + 1, conv);
      items++;
    } else {
      var texts = bitgDocsTexts_(el, []);
      for (var j = 0; j < texts.length; j++) changed += bitgDocsText_(texts[j], 0, undefined, conv);
      items += texts.length;
    }
  }
  return { changed: changed, items: items };
}

/** Hujjatning barcha bo'limlari: tablar (bo'lsa, ichki tablar bilan), tana, sarlavha, pastki qism, izohlar. */
function bitgDocsSections_(doc) {
  var parts = [];
  function addTab(dt) {
    parts.push(dt.getBody());
    if (dt.getHeader()) parts.push(dt.getHeader());
    if (dt.getFooter()) parts.push(dt.getFooter());
    var fns = dt.getFootnotes() || [];
    for (var i = 0; i < fns.length; i++) parts.push(fns[i].getFootnoteContents());
  }
  if (typeof doc.getTabs === 'function') {
    var walk = function (tabs) {
      for (var i = 0; i < tabs.length; i++) {
        if (String(tabs[i].getType()) === 'DOCUMENT_TAB') addTab(tabs[i].asDocumentTab());
        walk(tabs[i].getChildTabs());
      }
    };
    walk(doc.getTabs());
  } else addTab(doc);
  return parts;
}

function bitgDocsAll_(doc, conv) {
  var changed = 0;
  var items = 0;
  var parts = bitgDocsSections_(doc);
  for (var i = 0; i < parts.length; i++) {
    var texts = bitgDocsTexts_(parts[i], []);
    for (var j = 0; j < texts.length; j++) changed += bitgDocsText_(texts[j], 0, undefined, conv);
    items += texts.length;
  }
  return { changed: changed, items: items };
}

// ---------------------------------------------------------------------------------------------- Sheets

/**
 * Bitta diapazon: faqat matnli, formulasiz va o'zgargan kataklar yoziladi. Yozish RichTextValue bilan —
 * katak ichidagi qalin/kursiv/havola bo'laklari yangi koordinatalarga ko'chiriladi, matn son yoki formulaga
 * aylanib qolmaydi ("=..." bilan boshlansa ham).
 */
function bitgSheetsRange_(range, conv) {
  var values = range.getValues();
  var formulas = range.getFormulas();
  var rich = range.getRichTextValues();
  var cells = 0;
  var changed = 0;
  for (var r = 0; r < values.length; r++) {
    for (var c = 0; c < values[r].length; c++) {
      var v = values[r][c];
      if (typeof v !== 'string' || !v || formulas[r][c]) continue;
      var out = conv(v);
      if (out === v) continue;
      var edits = planEdits_(v, out);
      var b = SpreadsheetApp.newRichTextValue().setText(out);
      var rt = rich[r] && rich[r][c];
      var runs = rt ? rt.getRuns() : [];
      for (var k = 0; k < runs.length; k++) {
        var s = mapPos_(edits, runs[k].getStartIndex());
        var e = mapPos_(edits, runs[k].getEndIndex());
        if (e <= s) continue;
        b.setTextStyle(s, e, runs[k].getTextStyle());
        var url = runs[k].getLinkUrl();
        if (url) b.setLinkUrl(s, e, url);
      }
      range.getCell(r + 1, c + 1).setRichTextValue(b.build());
      cells++;
      changed += edits.length;
    }
  }
  return { cells: cells, changed: changed };
}

function bitgSheetsRanges_(ranges, conv) {
  var res = { changed: 0, items: 0 };
  for (var i = 0; i < ranges.length; i++) {
    var r = bitgSheetsRange_(ranges[i], conv);
    res.changed += r.changed;
    res.items += r.cells;
  }
  return res;
}

function bitgSheetsSelection_(ss, conv) {
  var list = ss.getActiveRangeList();
  var ranges = list ? list.getRanges() : [];
  if (!ranges.length && ss.getActiveRange()) ranges = [ss.getActiveRange()];
  if (!ranges.length) return { error: 'no_selection', changed: 0 };
  return bitgSheetsRanges_(ranges, conv);
}

function bitgSheetsAll_(ss, conv) {
  return bitgSheetsRanges_([ss.getActiveSheet().getDataRange()], conv);
}

// ---------------------------------------------------------------------------------------------- Slides

/**
 * Slides matni: `getFull` har safar shakl matnini qayta oladi (TextRange tahrirdan keyin eskiradi);
 * [start, end) — shu matn ichidagi qism. Qo'shilgan matn uslubi qo'shni matndan olinadi (Slides qoidasi).
 */
function bitgSlidesText_(getFull, start, end, conv) {
  var full = getFull().asString();
  if (end === undefined) end = full.length;
  var src = full.substring(start, end);
  if (!src) return 0;
  var edits = planEdits_(src, conv(src));
  for (var i = edits.length - 1; i >= 0; i--) {
    var e = edits[i];
    var at = start + e.start;
    var del = e.end - e.start;
    if (e.text) getFull().insertText(at, e.text);
    if (del > 0) getFull().clear(at + e.text.length, at + e.text.length + del);
  }
  return edits.length;
}

/** Sahifa elementidagi barcha matnlar: shakl, jadval kataklari, guruh ichidagilar. */
function bitgSlidesElement_(pe, conv) {
  var type = String(pe.getPageElementType());
  var n = 0;
  if (type === 'SHAPE') {
    var shape = pe.asShape();
    n += bitgSlidesText_(function () { return shape.getText(); }, 0, undefined, conv);
  } else if (type === 'TABLE') {
    var t = pe.asTable();
    for (var r = 0; r < t.getNumRows(); r++) {
      for (var c = 0; c < t.getNumColumns(); c++) {
        var cell = t.getCell(r, c);
        if (cell.getMergeState && String(cell.getMergeState()) === 'MERGED') continue; // asosiy katakda
        n += bitgSlidesText_(function () { return cell.getText(); }, 0, undefined, conv);
      }
    }
  } else if (type === 'GROUP') {
    var kids = pe.asGroup().getChildren();
    for (var i = 0; i < kids.length; i++) n += bitgSlidesElement_(kids[i], conv);
  }
  return n;
}

function bitgSlidesPage_(slide, conv) {
  var n = 0;
  var els = slide.getPageElements();
  for (var i = 0; i < els.length; i++) n += bitgSlidesElement_(els[i], conv);
  var notes = slide.getNotesPage && slide.getNotesPage().getSpeakerNotesShape();
  if (notes) n += bitgSlidesText_(function () { return notes.getText(); }, 0, undefined, conv);
  return n;
}

function bitgSlidesSelection_(pres, conv) {
  var sel = pres.getSelection();
  var type = sel ? String(sel.getSelectionType()) : 'NONE';
  var changed = 0;
  if (type === 'TEXT') {
    var tr = sel.getTextRange();
    var pes = sel.getPageElementRange() ? sel.getPageElementRange().getPageElements() : [];
    var shape = pes.length === 1 && String(pes[0].getPageElementType()) === 'SHAPE' ? pes[0].asShape() : null;
    if (shape && tr) {
      changed = bitgSlidesText_(function () { return shape.getText(); }, tr.getStartIndex(), tr.getEndIndex(), conv);
    } else if (sel.getTableCellRange()) {
      var tcs = sel.getTableCellRange().getTableCells();
      for (var k = 0; k < tcs.length; k++) {
        var tc = tcs[k];
        changed += bitgSlidesText_(function () { return tc.getText(); }, 0, undefined, conv);
      }
    } else return { error: 'no_selection', changed: 0 };
  } else if (type === 'TABLE_CELL') {
    var cells = sel.getTableCellRange().getTableCells();
    for (var i = 0; i < cells.length; i++) {
      var cl = cells[i];
      changed += bitgSlidesText_(function () { return cl.getText(); }, 0, undefined, conv);
    }
  } else if (type === 'PAGE_ELEMENT') {
    var els = sel.getPageElementRange().getPageElements();
    for (var j = 0; j < els.length; j++) changed += bitgSlidesElement_(els[j], conv);
  } else if (type === 'PAGE') {
    var pages = sel.getPageRange().getPages();
    for (var p = 0; p < pages.length; p++) changed += bitgSlidesPage_(pages[p].asSlide(), conv);
  } else return { error: 'no_selection', changed: 0 };
  return { changed: changed, items: 1 };
}

function bitgSlidesAll_(pres, conv) {
  var changed = 0;
  var slides = pres.getSlides();
  for (var i = 0; i < slides.length; i++) changed += bitgSlidesPage_(slides[i], conv);
  return { changed: changed, items: slides.length };
}
