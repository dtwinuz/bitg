// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// README skrinshotlari: demo/dist ni ochib har sahifani (to'ldirilgan holatda) docs/screenshots/ ga yozadi.
// `npm run site:screenshots` (demo/ ichida). 1440x900 JPEG q78 + telefon 390x844.
import { mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "@playwright/test";
import { BASE, serve } from "./serve.mjs";

const OUT = join(dirname(fileURLToPath(import.meta.url)), "../../docs/screenshots");
mkdirSync(OUT, { recursive: true });
const server = await serve();
const url = `http://127.0.0.1:${server.address().port}${BASE}`;
const browser = await chromium.launch();

async function open(page, hash, theme) {
  await page.goto(url + "#/" + hash);
  if (theme) await page.evaluate((t) => { localStorage.setItem("bitg.theme", t); }, theme);
  await page.waitForFunction(() => document.querySelector("#result .w") && document.querySelector("#convOut").value);
  await page.waitForTimeout(900);
}
async function shot(page, name) {
  await page.waitForTimeout(500);
  await page.screenshot({ path: join(OUT, name + ".jpg"), type: "jpeg", quality: 78 });
  console.log("docs/screenshots/" + name + ".jpg");
}
const go = (page, p) => page.click(`nav button[data-page="${p}"]`).then(() => page.waitForTimeout(700));

const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1, colorScheme: "light" });
const page = await ctx.newPage();
await open(page, "home");
await page.waitForTimeout(2600); // zarrachalar harfga yig'ilsin
await shot(page, "home");

await go(page, "try");
await shot(page, "try");

await go(page, "text");
await shot(page, "text");

await go(page, "rules");
await shot(page, "rules");

await go(page, "kb");
await page.click('.sw[data-k="keymap"]').catch(() => {});
await page.waitForTimeout(400);
await shot(page, "keyboard");

await go(page, "apps");
await shot(page, "apps");

await go(page, "ex");
await page.fill("#exInput", "Shahrisabz");
await page.dispatchEvent("#exInput", "input");
await shot(page, "dictionary");

await go(page, "news");
await page.click("#updBtn");
await page.waitForTimeout(1500);
await shot(page, "news");

await go(page, "fb");
await page.click('#fbType [data-v="conversion"]');
await page.fill("#fbTyped", "Chilonzor tumanida");
await page.fill("#fbGot", "Chilonzor tumanida");
await page.fill("#fbExp", "Çilonzor tumanida");
await page.fill("#fbText", "Tuman nomi lug'atda istisno bo'lib qolgan ekan, olib tashladim.").catch(() => {});
await page.click("#fbShot");
await page.waitForTimeout(1500);
await shot(page, "feedback");

await go(page, "about");
await shot(page, "about");

// qorong'i mavzu: Sinash sahifasi
const dark = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: "dark" });
const dp = await dark.newPage();
await open(dp, "try");
await shot(dp, "try-dark");

// telefon
const phone = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true });
const pp = await phone.newPage();
await open(pp, "try");
await shot(pp, "phone");

await browser.close();
server.close();
