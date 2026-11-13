// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Kengaytma ikonkalari (16/32/48/128 PNG) web/icon.svg dan — natija extension/icons/ ga commit qilinadi.
//
//   NODE_PATH="$(npm root -g)/@playwright/cli/node_modules" [CHROME=.../chrome.exe] node extension/tools/icons.cjs
'use strict';
const fs = require('fs');
const path = require('path');
const { chromium } = require('playwright-core');

const ROOT = path.resolve(__dirname, '..', '..');
const SVG = fs.readFileSync(path.join(ROOT, 'web', 'icon.svg'), 'utf8');
const OUT = path.join(ROOT, 'extension', 'icons');
const SIZES = [16, 32, 48, 128];

(async () => {
  const browser = await chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : {});
  const page = await browser.newPage({ deviceScaleFactor: 1 });
  fs.mkdirSync(OUT, { recursive: true });
  for (const size of SIZES) {
    const svg = SVG.replace(/width="\d+" height="\d+"/, `width="${size}" height="${size}"`);
    await page.setViewportSize({ width: size, height: size });
    await page.setContent(`<!doctype html><style>html,body{margin:0;background:transparent}svg{display:block}</style>${svg}`);
    await page.screenshot({ path: path.join(OUT, `icon-${size}.png`), omitBackground: true, clip: { x: 0, y: 0, width: size, height: size } });
    console.log(`icons/icon-${size}.png`);
  }
  await browser.close();
})().catch((e) => { console.error(e); process.exit(1); });
