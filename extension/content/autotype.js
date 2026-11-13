// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Yozayotganda o'girish (foydalanuvchi o'zi yoqqan saytlarda, ixtiyoriy host ruxsati bilan).
// So'z probel yoki tinish belgisidan keyin yadroga yuboriladi; javob kelganda matn hali o'sha joyda bo'lsa
// almashtiriladi, kursor o'z joyida qoladi. Darhol Backspace — asl so'z qaytadi va shu sahifada boshqa o'girilmaydi.
(function () {
  'use strict';
  if (window.bitgAutotype) { window.bitgAutotype.enabled = true; return; }
  var L = window.bitgLib, E = window.bitgEdit;
  var api = globalThis.browser || globalThis.chrome;
  var self = { enabled: !L.canvasEditor(location.href) };
  window.bitgAutotype = self;
  var last = null;        // oxirgi o'girish (Backspace bilan qaytarish uchun)
  var kept = new Set();   // qaytarilgan so'zlar: shu sahifada boshqa o'girilmaydi

  function target(e) {
    var t = e.composedPath ? e.composedPath()[0] : e.target;
    return t && t.nodeType === 1 ? t : null;
  }

  async function onControl(el, e) {
    if (el.selectionStart !== el.selectionEnd) return;
    var w = L.findWordBefore(el.value, el.selectionStart);
    if (!w || kept.has(w.word.toLowerCase())) return;
    var out = await E.convert(w.word);
    if (typeof out !== 'string' || out === w.word || !self.enabled) return;
    w = L.relocate(el.value, w, el.selectionStart);
    if (!w) return;
    var a = el.selectionStart, b = el.selectionEnd, dir = el.selectionDirection;
    E.replaceInControl(el, w.start, w.end, out);
    el.setSelectionRange(L.shiftCaret(a, w.start, w.word.length, out.length),
      L.shiftCaret(b, w.start, w.word.length, out.length), dir || 'none');
    last = { el: el, start: w.start, conv: out, orig: w.word, trigger: w.trigger };
  }

  async function onRich() {
    var sel = window.getSelection();
    if (!sel || !sel.isCollapsed || !sel.anchorNode || sel.anchorNode.nodeType !== 3) return;
    var w = L.findWordBefore(sel.anchorNode.data, sel.anchorOffset);
    if (!w || kept.has(w.word.toLowerCase())) return;
    var out = await E.convert(w.word);
    if (typeof out !== 'string' || out === w.word || !self.enabled) return;
    // javob kutilganda muharrir tugunni almashtirgan yoki oldingi so'z almashib matn siljigan bo'lishi mumkin:
    // so'z kursor turgan tugundan qayta topiladi, kursorgacha masofa saqlanadi
    sel = window.getSelection();
    if (!sel || !sel.isCollapsed || !sel.anchorNode || sel.anchorNode.nodeType !== 3) return;
    var node = sel.anchorNode, caret = sel.anchorOffset;
    w = L.relocate(node.data, w, caret);
    if (!w) return;
    var r = document.createRange();
    r.setStart(node, w.start);
    r.setEnd(node, w.end);
    E.replaceRangeRich(r, out);
    E.moveForward(caret - w.end);
    last = { el: null, rich: true, conv: out, orig: w.word, trigger: w.trigger };
  }

  document.addEventListener('input', function (e) {
    if (!self.enabled || e.isComposing) return;
    if (e.inputType !== 'insertText' && e.inputType !== 'insertLineBreak') return;
    var el = target(e);
    if (!el) return;
    if (L.isAutotypeControl(el)) onControl(el, e).catch(function () {});
    else if (e.inputType === 'insertText' && L.isAutotypeRich(el)) onRich().catch(function () {});
  }, true);

  document.addEventListener('keydown', function (e) {
    if (!last || !self.enabled) return;
    if (e.key === 'Shift' || e.key === 'Control' || e.key === 'Alt' || e.key === 'Meta') return;
    var prev = last;
    last = null;
    if (e.key !== 'Backspace' || e.ctrlKey || e.altKey || e.metaKey || e.isComposing) return;
    if (!prev.rich) {
      var el = target(e);
      if (el !== prev.el || el.selectionStart !== el.selectionEnd) return;
      var u = L.undoRange(el.value, el.selectionStart, prev);
      if (!u) return;
      e.preventDefault();
      E.replaceInControl(el, u.start, u.end, u.text);
      el.setSelectionRange(u.caret, u.caret);
      kept.add(prev.orig.toLowerCase());
      return;
    }
    var sel = window.getSelection();
    if (!sel || !sel.isCollapsed || !sel.anchorNode || sel.anchorNode.nodeType !== 3) return;
    var node = sel.anchorNode, offset = sel.anchorOffset;
    var tail = prev.conv + prev.trigger;
    var start = offset - tail.length;
    var ur = L.undoRange(node.data, offset, { start: start, conv: prev.conv, orig: prev.orig, trigger: prev.trigger });
    if (start < 0 || !ur) return;
    e.preventDefault();
    var r = document.createRange();
    r.setStart(node, ur.start);
    r.setEnd(node, ur.end);
    E.replaceRangeRich(r, ur.text);
    E.moveForward(prev.trigger.length);
    kept.add(prev.orig.toLowerCase());
  }, true);

  api.runtime.onMessage.addListener(function (m) {
    if (!m || m.t !== 'autotype' || m.pattern !== L.originPattern(location.href)) return;
    self.enabled = !!m.on && !L.canvasEditor(location.href);
  });
})();
