// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Testlar uchun eng kichik soxta DOM: nodeType, parentNode/firstChild/nextSibling, getAttribute,
// className, isContentEditable (ota-bobodan meros). jsdom shart emas.
'use strict';

class FakeNode {
  constructor(nodeType) {
    this.nodeType = nodeType;
    this.parentNode = null;
    this.childNodes = [];
  }
  get firstChild() { return this.childNodes[0] || null; }
  get nextSibling() {
    if (!this.parentNode) return null;
    const s = this.parentNode.childNodes;
    return s[s.indexOf(this) + 1] || null;
  }
  append(...kids) {
    for (const k of kids) { k.parentNode = this; this.childNodes.push(k); }
    return this;
  }
}

class FakeText extends FakeNode {
  constructor(data) { super(3); this.data = data; }
}

class FakeElement extends FakeNode {
  constructor(tag, attrs = {}) {
    super(1);
    this.tagName = tag.toUpperCase();
    this.attrs = { ...attrs };
  }
  getAttribute(n) { return Object.prototype.hasOwnProperty.call(this.attrs, n) ? String(this.attrs[n]) : null; }
  get className() { return this.attrs.class || ''; }
  get type() { return this.attrs.type || 'text'; }
  get readOnly() { return 'readonly' in this.attrs; }
  get disabled() { return 'disabled' in this.attrs; }
  get isContentEditable() {
    for (let p = this; p && p.nodeType === 1; p = p.parentNode) {
      const ce = p.getAttribute('contenteditable');
      if (ce === 'false') return false;
      if (ce !== null) return true;
    }
    return false;
  }
}

/** h('div', {class: 'x'}, 'matn', h('b', {}, 'qalin')) */
function h(tag, attrs, ...kids) {
  const el = new FakeElement(tag, attrs || {});
  el.append(...kids.map((k) => (typeof k === 'string' ? new FakeText(k) : k)));
  return el;
}

class FakeDocument extends FakeNode {
  constructor(body) { super(9); this.append(body); }
}

module.exports = { h, FakeText, FakeElement, FakeDocument };
