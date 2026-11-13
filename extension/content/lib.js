// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// DOM'siz sof mantiq: matn tugunlarini tanlash, kursor oldidagi so'zni topish, kursorni siljitish.
// Kontent skriptlar (`window.bitgLib`) va `node --test` (module.exports) bir xil faylni ishlatadi.
(function (root, factory) {
  'use strict';
  var api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else if (!root.bitgLib) root.bitgLib = api;
})(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  'use strict';

  // Ichidagi matnga tegilmaydigan elementlar: kod, skript, forma boshqaruvlari, grafika.
  var SKIP_TAGS = new Set([
    'SCRIPT', 'STYLE', 'NOSCRIPT', 'TEMPLATE', 'TEXTAREA', 'INPUT', 'SELECT', 'OPTION', 'OPTGROUP',
    'DATALIST', 'CODE', 'PRE', 'KBD', 'SAMP', 'VAR', 'TT', 'SVG', 'MATH', 'IFRAME', 'OBJECT', 'EMBED',
    'CANVAS', 'VIDEO', 'AUDIO', 'HEAD', 'TITLE'
  ]);
  // Kod muharrirlari (Monaco, CodeMirror, Ace) — yashirin textarea/contenteditable bilan ishlaydi.
  var CODE_EDITOR_CLASS = /(^|\s)(monaco-editor|CodeMirror|cm-editor|cm-content|ace_editor|ql-code-block|hljs)(\s|$)/;
  var APOS = "'‘’ʻʼ`";
  var WORD_CHAR = /[\p{L}\p{M}'‘’ʻʼ`]/u;
  var TRIGGER = /[\s.,!?;:)\]}"»”…]/u;
  // So'z oldida shulardan biri bo'lsa — bu havola, pochta, yo'l, teg yoki raqamning davomi: tegilmaydi.
  var GLUED_BEFORE = /[\p{N}@/\\.:_#&=+%~$]/u;
  var LETTER = /\p{L}/u;

  function tagOf(el) {
    var n = el && (el.tagName || el.nodeName);
    return n ? String(n).toUpperCase() : '';
  }
  function attr(el, name) {
    return el && typeof el.getAttribute === 'function' ? el.getAttribute(name) : null;
  }
  function classOf(el) {
    if (!el) return '';
    var c = el.className;
    if (typeof c === 'string') return c;
    if (c && typeof c.baseVal === 'string') return c.baseVal; // SVG
    return attr(el, 'class') || '';
  }

  /** Element (va butun ichki daraxti) sahifa o'girishidan chetda qoladimi. */
  function skipElement(el) {
    if (!el) return false;
    if (SKIP_TAGS.has(tagOf(el))) return true;
    if (el.isContentEditable) return true;
    var ce = attr(el, 'contenteditable');
    if (ce !== null && ce !== 'false') return true;
    if (attr(el, 'translate') === 'no') return true;
    if (attr(el, 'data-bitg-skip') !== null) return true;
    var cls = classOf(el);
    if (/(^|\s)notranslate(\s|$)/.test(cls) || CODE_EDITOR_CLASS.test(cls)) return true;
    return false;
  }

  /** Tugunning biror ota-bobosi chetda qoladimi (MutationObserver qo'shgan tugunlar uchun). */
  function insideSkipped(node) {
    for (var p = node && node.parentNode; p; p = p.parentNode) {
      if (p.nodeType === 9 || p.nodeType === 11) return false; // document / shadow root
      if (skipElement(p)) return true;
    }
    return false;
  }

  /** Matn tuguni o'girishga arziydimi: kamida bitta harf bo'lsin. */
  function hasLetters(s) {
    return typeof s === 'string' && LETTER.test(s);
  }

  /** Matn tuguni sahifa o'girishiga olinadimi (to'liq tekshiruv: ota-bobolar ham). */
  function acceptTextNode(node) {
    if (!node || node.nodeType !== 3) return false;
    if (!hasLetters(node.data)) return false;
    return !insideSkipped(node);
  }

  /**
   * `root` ichidagi o'giriladigan matn tugunlari (hujjat tartibida). Chetdagi element butun ichki daraxti
   * bilan o'tkazib yuboriladi. `firstChild/nextSibling` bilan iterativ: haqiqiy DOM va testdagi soxta tugunlar.
   */
  function textNodes(root) {
    var out = [];
    if (!root) return out;
    if (root.nodeType === 3) return hasLetters(root.data) ? [root] : out;
    if (root.nodeType === 1 && skipElement(root)) return out;
    var n = root.firstChild;
    while (n) {
      var down = false;
      if (n.nodeType === 3) {
        if (hasLetters(n.data)) out.push(n);
      } else if (n.nodeType === 1 && !skipElement(n) && n.firstChild) {
        down = true;
      }
      if (down) { n = n.firstChild; continue; }
      while (n && n !== root && !n.nextSibling) n = n.parentNode;
      if (!n || n === root) break;
      n = n.nextSibling;
    }
    return out;
  }

  function insideCodeEditor(el) {
    for (var p = el; p; p = p.parentNode) {
      if (p.nodeType !== 1) continue;
      if (CODE_EDITOR_CLASS.test(classOf(p))) return true;
      if (attr(p, 'data-bitg-skip') !== null) return true;
    }
    return false;
  }

  /** Yozayotganda o'girish mumkin bo'lgan oddiy matn maydoni (parol, pochta, URL emas). */
  function isTextControl(el) {
    var t = tagOf(el);
    if (t === 'TEXTAREA') return true;
    if (t !== 'INPUT') return false;
    var type = String(attr(el, 'type') || el.type || 'text').toLowerCase();
    return type === 'text' || type === 'search';
  }
  function isAutotypeControl(el) {
    if (!isTextControl(el) || el.readOnly || el.disabled) return false;
    return !insideCodeEditor(el);
  }
  function isAutotypeRich(el) {
    if (!el || !el.isContentEditable) return false;
    return !insideCodeEditor(el);
  }

  /**
   * Kursor oldida probel yoki tinish belgisi bo'lsa, undan oldingi so'z.
   * Qaytadi `{start, end, word, trigger}` yoki null. Apostrof so'zning bir qismi (o' g' tutuq).
   */
  function findWordBefore(text, caret) {
    if (typeof text !== 'string' || caret < 2 || caret > text.length) return null;
    var trigger = text[caret - 1];
    if (!TRIGGER.test(trigger)) return null;
    var end = caret - 1, s = end;
    while (s > 0 && WORD_CHAR.test(text[s - 1])) s--;
    // boshidagi apostrof — ko'pincha ochuvchi qo'shtirnoq
    while (s < end && APOS.indexOf(text[s]) >= 0) s++;
    if (s >= end) return null;
    if (s > 0 && GLUED_BEFORE.test(text[s - 1])) return null;
    var word = text.slice(s, end);
    if (!LETTER.test(word)) return null;
    return { start: s, end: end, word: word, trigger: trigger };
  }

  /** `[start, start+oldLen)` oralig'i `newLen` uzunlikka almashganda kursorning yangi o'rni. */
  function shiftCaret(caret, start, oldLen, newLen) {
    if (caret >= start + oldLen) return caret + (newLen - oldLen);
    if (caret > start) return start + newLen;
    return caret;
  }

  /** O'girish natijasini qo'yishdan oldin: matn hali o'sha joyda turibdimi (asinxron javob kechikishi). */
  function stillMatches(text, w) {
    return typeof text === 'string' && text.slice(w.start, w.end) === w.word && text[w.end] === w.trigger &&
      (w.start === 0 || !WORD_CHAR.test(text[w.start - 1]));
  }

  /**
   * Javob kelguncha so'z siljigan bo'lsa (oldingi so'z kechroq almashib, uzunlik o'zgardi) — uni kursor oldidan
   * qayta topadi. Qaytadi yangi `{start, end, word, trigger}` yoki null (so'z endi yo'q yoki tahrir qilingan).
   */
  function relocate(text, w, caret) {
    if (stillMatches(text, w)) return w;
    if (typeof text !== 'string') return null;
    var needle = w.word + w.trigger;
    var s = text.lastIndexOf(needle, Math.max(0, caret - needle.length));
    // faqat kursorga yaqin (oxirgi 64 belgi ichida) va so'z boshidan boshlangan moslik
    if (s < 0 || s + needle.length > caret || caret - (s + needle.length) > 64) return null;
    if (s > 0 && (WORD_CHAR.test(text[s - 1]) || GLUED_BEFORE.test(text[s - 1]))) return null;
    return { start: s, end: s + w.word.length, word: w.word, trigger: w.trigger };
  }

  /**
   * Backspace bilan qaytarish: kursor aynan "yangi so'z + trigger" dan keyin turibdimi.
   * `last = {start, conv, orig, trigger}`; qaytadi almashtiriladigan oraliq yoki null.
   */
  function undoRange(text, caret, last) {
    if (!last) return null;
    var tail = last.conv + last.trigger;
    if (caret !== last.start + tail.length) return null;
    if (text.slice(last.start, caret) !== tail) return null;
    return { start: last.start, end: last.start + last.conv.length, text: last.orig, caret: last.start + last.orig.length + last.trigger.length };
  }

  /**
   * Belgilangan oraliq ichiga tushgan matn tuguni bo'lagi `{from, to}` yoki null.
   * Tugun oraliqning boshlanish/tugash konteyneri bo'lsa, o'sha offset olinadi; aks holda butun tugun.
   */
  function textPiece(len, isStart, startOffset, isEnd, endOffset) {
    var from = isStart ? Math.min(startOffset, len) : 0;
    var to = isEnd ? Math.min(endOffset, len) : len;
    return to > from ? { from: from, to: to } : null;
  }

  /** Brauzer manzilidan avto-o'girish ruxsati uchun naqsh (faqat http/https). */
  function originPattern(url) {
    var m = /^(https?):\/\/([^/?#:]+)/i.exec(url || '');
    return m ? m[1].toLowerCase() + '://' + m[2].toLowerCase() + '/*' : null;
  }

  /** Brauzer o'z sahifalariga kengaytma skriptini qo'yishga ruxsat bermaydi. */
  function restrictedUrl(url) {
    if (!/^(https?|file):/i.test(url || '')) return true;
    return /^https:\/\/(chrome\.google\.com\/webstore|chromewebstore\.google\.com|microsoftedge\.microsoft\.com\/addons|addons\.mozilla\.org)/i.test(url);
  }

  /** Google Docs/Slides matnni canvas'da chizadi — DOM'da tahrir qilinadigan matn yo'q. */
  function canvasEditor(url) {
    return /^https:\/\/docs\.google\.com\/(document|presentation|spreadsheets)\//i.test(url || '');
  }

  return {
    skipElement: skipElement,
    insideSkipped: insideSkipped,
    hasLetters: hasLetters,
    acceptTextNode: acceptTextNode,
    textNodes: textNodes,
    insideCodeEditor: insideCodeEditor,
    isTextControl: isTextControl,
    isAutotypeControl: isAutotypeControl,
    isAutotypeRich: isAutotypeRich,
    findWordBefore: findWordBefore,
    shiftCaret: shiftCaret,
    stillMatches: stillMatches,
    relocate: relocate,
    undoRange: undoRange,
    textPiece: textPiece,
    originPattern: originPattern,
    restrictedUrl: restrictedUrl,
    canvasEditor: canvasEditor
  };
});
