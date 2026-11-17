// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

import { expect, test } from "@playwright/test";

const PAGES = {
  home: "#todayN", try: "#result .w", text: "#convOut", rules: '.sw[data-k="english_guard"]', kb: "#p-kb",
  apps: "#apps li", ex: "#rulesList li", news: "#timeline .ver", fb: "#fbForm", about: "#ver",
};

// namuna matnlar yadro yuklangach yoziladi: sinovdagi kiritish ulardan keyin bo'lsin
const ready = (page) => page.waitForFunction(() => document.querySelector("#convOut").value && document.querySelector("#result .w"));

let errors;
test.beforeEach(async ({ page }) => {
  errors = [];
  page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });
  page.on("pageerror", (e) => errors.push(String(e)));
});
test.afterEach(() => expect(errors).toEqual([]));

for (const [p, sel] of Object.entries(PAGES)) {
  test(`#/${p} ochiladi va to'la`, async ({ page }) => {
    await page.goto(`./#/${p}`);
    await expect(page.locator(`#p-${p}`)).toHaveClass(/\bon\b/);
    await expect(page.locator(sel).first()).toBeVisible();
    await expect(page.locator(".demo-pill")).toContainText("Demo");
  });
}

test("haqiqiy yadro: eski yozuv yangi alifboga, inglizcha tegilmaydi", async ({ page }) => {
  await page.goto("./#/try");
  await ready(page);
  await page.fill("#input", "O'zbekiston shaharlari: Toshkent. Please check the schedule, keyin choy ichamiz");
  await expect(page.locator("#result")).toHaveText("Özbekiston şaharlari: Toşkent. Please check the schedule, keyin çoy içamiz");
});

test("lug'at: qo'shish xotirada ishlaydi va qayta yuklanganda tiklanadi", async ({ page }) => {
  await page.goto("./#/ex");
  await ready(page);
  const n = await page.locator("#rulesList li").count();
  await page.fill("#exInput", "Shahrisabz");
  await page.dispatchEvent("#exInput", "input");
  await page.click("#exAdd");
  await expect(page.locator("#rulesList li")).toHaveCount(n + 1);
  await page.reload();
  await ready(page);
  await expect(page.locator("#rulesList li")).toHaveCount(n);
});

test("matn: yangi -> eski va kirill", async ({ page }) => {
  await page.goto("./#/text");
  await ready(page);
  await page.fill("#convIn", "Шаҳар чиройли");
  await expect(page.locator("#convOut")).toHaveValue("Şahar çiroyli");
  await page.click('#dir [data-v="old"]');
  await page.fill("#convIn", "Özbekiston");
  await expect(page.locator("#convOut")).toHaveValue("Oʻzbekiston");
});

test("xabar berish: yuboriladi va chipta raqami chiqadi", async ({ page }) => {
  await page.goto("./#/fb");
  await ready(page);
  await page.click('#fbType [data-v="idea"]');
  await page.fill("#fbText", "Mac uchun ham shunday oyna bo'lsa yaxshi bo'lardi.");
  await page.click("#fbSend");
  await expect(page.locator("#fbTicket")).toHaveText(/^DM-/);
});

test("hash havola menyu bilan sinxron", async ({ page }) => {
  await page.goto("./#/home");
  await page.click('nav button[data-page="news"]');
  await expect(page).toHaveURL(/#\/news$/);
  await page.goto("./#/kb");
  await expect(page.locator("#p-kb")).toHaveClass(/\bon\b/);
});
