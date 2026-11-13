// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Tahrir maydonida matnni almashtirish. Avval `execCommand('insertText')`: brauzerning bekor qilish
// tarixiga (Ctrl+Z) tushadi va React/ProseMirror kabi muharrirlar odatdagi `input` hodisasini ko'radi.
// Ishlamasa — `setRangeText` va qo'lda `input` hodisasi.
(function () {
  'use strict';
  if (window.bitgEdit) return;
  var api = globalThis.browser || globalThis.chrome;

  function convert(text) {
    return api.runtime.sendMessage({ t: 'conv', s: text });
  }
  function convertMany(list) {
    return api.runtime.sendMessage({ t: 'convMany', a: list });
  }

  /** Aktiv element, shadow DOM ichiga ham kiradi. */
  function activeElement() {
    var el = document.activeElement;
    while (el && el.shadowRoot && el.shadowRoot.activeElement) el = el.shadowRoot.activeElement;
    return el;
  }

  /** input/textarea: `[s, e)` ni `text` bilan almashtiradi, keyin `text` belgilangan yoki kursor oxirida. */
  function replaceInControl(el, s, e, text) {
    if (document.activeElement !== el && typeof el.focus === 'function') el.focus({ preventScroll: true });
    el.setSelectionRange(s, e);
    var ok = false;
    try { ok = document.execCommand('insertText', false, text); } catch (err) { ok = false; }
    if (!ok || el.value.slice(s, s + text.length) !== text) {
      el.setRangeText(text, s, e, 'end');
      el.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertReplacementText', data: text }));
    }
  }

  /** contenteditable: berilgan oraliqni `text` bilan almashtiradi (bitta matn tuguni ichida). */
  function replaceRangeRich(range, text) {
    var sel = window.getSelection();
    sel.removeAllRanges();
    sel.addRange(range);
    var ok = false;
    try { ok = document.execCommand('insertText', false, text); } catch (err) { ok = false; }
    if (!ok) {
      range.deleteContents();
      var node = document.createTextNode(text);
      range.insertNode(node);
      range.setStartAfter(node);
      range.collapse(true);
      sel.removeAllRanges();
      sel.addRange(range);
      var host = node.parentElement && node.parentElement.closest('[contenteditable]');
      if (host) host.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertReplacementText', data: text }));
    }
  }

  /** Kursorni `n` belgi oldinga suradi (muharrir DOM'ni qayta qurgan bo'lsa ham ishlaydi). */
  function moveForward(n) {
    var sel = window.getSelection();
    for (var i = 0; i < n; i++) sel.modify('move', 'forward', 'character');
  }

  window.bitgEdit = {
    convert: convert,
    convertMany: convertMany,
    activeElement: activeElement,
    replaceInControl: replaceInControl,
    replaceRangeRich: replaceRangeRich,
    moveForward: moveForward
  };
})();
