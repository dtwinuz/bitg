// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Demo smoke: demo/dist ni /demo/bitg/ ostida ochib, har sahifa to'lganini va konsolda xato yo'qligini tekshiradi.
// `npm run test:demo` (demo/ ichida).
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "tests",
  timeout: 30_000,
  reporter: [["list"]],
  use: { baseURL: "http://127.0.0.1:4391/demo/bitg/", viewport: { width: 1280, height: 800 } },
  webServer: { command: "node scripts/serve.mjs 4391", url: "http://127.0.0.1:4391/demo/bitg/", reuseExistingServer: false },
});
