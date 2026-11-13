// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Fon: yadro (WASM) shu yerda bir marta yuklanadi, kontent skriptlar matnni xabar bilan yuboradi.
// Chrome/Edge — service worker (importScripts), Firefox — manifestdagi background.scripts ro'yxati.
// Tarmoq so'rovi yo'q: wasm kengaytma ichidan olinadi, CSP connect-src 'self'.
/* global bitgCore, bitgI18n */
'use strict';
if (typeof importScripts === 'function' && typeof bitgCore === 'undefined') {
  importScripts('core/core.js', 'i18n.js');
}

const api = globalThis.browser || globalThis.chrome;
const AUTOTYPE_ID = 'bitg-autotype';
const SELECTION_FILES = ['content/lib.js', 'content/toast.js', 'content/edit.js', 'content/selection.js'];
const PAGE_FILES = ['content/lib.js', 'content/toast.js', 'content/edit.js', 'content/page.js'];
const AUTOTYPE_FILES = ['content/lib.js', 'content/edit.js', 'content/autotype.js'];

function core() {
  return bitgCore.load(api.runtime.getURL('core/bitg_core.wasm'));
}

async function settings() {
  const s = await api.storage.sync.get({ lang: 'uz', sites: [] });
  return { lang: bitgI18n.pick(s.lang), sites: Array.isArray(s.sites) ? s.sites : [] };
}

// ---------- kontekst menyu ----------
async function buildMenus() {
  const { lang } = await settings();
  await api.contextMenus.removeAll();
  api.contextMenus.create({ id: 'convert', title: bitgI18n.t(lang, 'menuConvert'), contexts: ['selection', 'editable'] });
  api.contextMenus.create({ id: 'page', title: bitgI18n.t(lang, 'menuPage'), contexts: ['page'] });
}

// ---------- belgilangan matn ----------
async function convertSelection(tabId, frameId) {
  const { lang } = await settings();
  const msg = bitgI18n.toasts(lang);
  const one = typeof frameId === 'number';
  const targets = one ? [{ tabId, frameIds: [frameId] }] : [{ tabId, allFrames: true }, { tabId }];
  for (const target of targets) {
    try {
      await api.scripting.executeScript({ target, files: SELECTION_FILES });
      await api.scripting.executeScript({
        target,
        func: (m, strict) => window.bitgSelection.run(m, strict),
        args: [msg, !one]
      });
      return true;
    } catch (e) {
      // ruxsatsiz begona freym bo'lsa — faqat asosiy freym bilan qayta urinish
    }
  }
  return false;
}

// ---------- butun sahifa ----------
async function togglePage(tabId) {
  const { lang } = await settings();
  const target = { tabId, frameIds: [0] };
  await api.scripting.executeScript({ target, files: PAGE_FILES });
  const [res] = await api.scripting.executeScript({
    target,
    func: (m) => window.bitgPage.toggle(m),
    args: [bitgI18n.toasts(lang)]
  });
  return !!(res && res.result);
}

async function pageActive(tabId) {
  try {
    const [res] = await api.scripting.executeScript({
      target: { tabId, frameIds: [0] },
      func: () => !!(window.bitgPage && window.bitgPage.active())
    });
    return !!(res && res.result);
  } catch (e) {
    return false;
  }
}

// ---------- yozayotganda o'girish (saytma-sayt) ----------
async function grantedSites() {
  const { sites } = await settings();
  const out = [];
  for (const o of sites) {
    try { if (await api.permissions.contains({ origins: [o] })) out.push(o); } catch (e) { /* noto'g'ri naqsh */ }
  }
  return out;
}

async function syncAutotype() {
  const sites = await grantedSites();
  const existing = await api.scripting.getRegisteredContentScripts({ ids: [AUTOTYPE_ID] });
  if (!sites.length) {
    if (existing.length) await api.scripting.unregisterContentScripts({ ids: [AUTOTYPE_ID] });
    return sites;
  }
  const def = { id: AUTOTYPE_ID, matches: sites, js: AUTOTYPE_FILES, runAt: 'document_idle', allFrames: true };
  if (existing.length) await api.scripting.updateContentScripts([def]);
  else await api.scripting.registerContentScripts([def]);
  return sites;
}

async function setSites(fn) {
  const { sites } = await settings();
  const next = [...new Set(fn(sites))].sort();
  await api.storage.sync.set({ sites: next });
  return syncAutotype();
}

// tabs ruxsatisiz manzil ko'rinmaydi: kontent skript naqshni o'z manzili bilan o'zi solishtiradi
async function broadcast(message) {
  const tabs = await api.tabs.query({});
  await Promise.all(tabs.map((t) => api.tabs.sendMessage(t.id, message).catch(() => {})));
}

async function addSite(pattern, tabId) {
  if (!pattern || !(await api.permissions.contains({ origins: [pattern] }))) return false;
  await setSites((s) => s.concat(pattern));
  // sahifani qayta yuklamasdan shu tabda darhol ishlasin
  if (typeof tabId === 'number') {
    try { await api.scripting.executeScript({ target: { tabId, allFrames: true }, files: AUTOTYPE_FILES }); } catch (e) { /* freym */ }
  }
  return true;
}

async function removeSite(pattern) {
  await setSites((s) => s.filter((x) => x !== pattern));
  try { await api.permissions.remove({ origins: [pattern] }); } catch (e) { /* allaqachon yo'q */ }
  await broadcast({ t: 'autotype', on: false, pattern });
  return true;
}

// ---------- xabarlar ----------
function handle(m, sender) {
  switch (m && m.t) {
    case 'conv':
      return core().then((c) => c.convert(String(m.s || '')));
    case 'convMany':
      return core().then((c) => (Array.isArray(m.a) ? m.a.map((s) => c.convert(String(s))) : []));
    // quyidagilar faqat kengaytmaning o'z sahifasidan (popup), kontent skriptdan emas
    case 'page':
      return sender.tab ? null : togglePage(m.tabId);
    case 'pageActive':
      return sender.tab ? null : pageActive(m.tabId);
    case 'siteAdd':
      return sender.tab ? null : addSite(m.pattern, m.tabId);
    case 'siteRemove':
      return sender.tab ? null : removeSite(m.pattern);
    default:
      return null;
  }
}

api.runtime.onMessage.addListener((m, sender, reply) => {
  if (sender.id !== api.runtime.id) return false;
  const p = handle(m, sender);
  if (!p) return false;
  Promise.resolve(p).then(reply, () => reply(null));
  return true;
});

api.contextMenus.onClicked.addListener((info, tab) => {
  if (!tab || typeof tab.id !== 'number') return;
  if (info.menuItemId === 'convert') convertSelection(tab.id, info.frameId);
  else if (info.menuItemId === 'page') togglePage(tab.id).catch(() => {});
});

api.commands.onCommand.addListener(async (command, tab) => {
  if (!tab || typeof tab.id !== 'number') [tab] = await api.tabs.query({ active: true, currentWindow: true });
  if (!tab || typeof tab.id !== 'number') return;
  if (command === 'convert-selection') convertSelection(tab.id, null);
  else if (command === 'toggle-page') togglePage(tab.id).catch(() => {});
});

api.runtime.onInstalled.addListener(() => { buildMenus(); syncAutotype(); });
api.runtime.onStartup.addListener(() => { buildMenus(); syncAutotype(); });
api.storage.onChanged.addListener((changes, area) => {
  if (area === 'sync' && changes.lang) buildMenus();
});
// foydalanuvchi ruxsatni brauzer sozlamalarida olib tashlasa — ro'yxatdan ham chiqadi
api.permissions.onRemoved.addListener(async (p) => {
  const gone = (p && p.origins) || [];
  if (gone.length) {
    await setSites((s) => s.filter((x) => !gone.includes(x)));
    for (const pattern of gone) await broadcast({ t: 'autotype', on: false, pattern });
  }
});
