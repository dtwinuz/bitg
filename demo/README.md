# bitg jonli demo

Windows ilovasining haqiqiy sozlamalar oynasi ([`crates/bitg-settings/src/ui.html`](../crates/bitg-settings/src/ui.html))
brauzerda: oyna o'zgartirilmaydi, unga faqat xotiradagi IPC ([`mock-ipc.js`](mock-ipc.js)) va demo belgisi ulanadi.
O'girishni haqiqiy yadro qiladi (`bitg-core` → `wasm32-unknown-unknown`, sayt bilan bir xil buyruq). Statistika,
dasturlar ro'yxati va lug'at namuna ma'lumot; har o'zgarish faqat xotirada, sahifa qayta yuklanganda tiklanadi.

Jonli: <https://wonmor.io/demo/bitg/>

| Buyruq (`demo/` ichida) | Nima qiladi |
|---|---|
| `npm ci` | Playwright'ni o'rnatadi (`npx playwright install chromium` bir marta) |
| `npm run build` | `dist/` ni yozadi: `index.html`, `mock-ipc.js`, `demo.css`, `bitg_core.wasm` (cargo kerak; yoki `BITG_WASM=path`) |
| `npm run serve` | `dist/` ni `http://127.0.0.1:4178/demo/bitg/` da ochadi |
| `npm run test:demo` | smoke e2e: 10 sahifa, haqiqiy yadro, lug'at qo'shish/tiklanish, xabar berish, konsolda xato yo'q |
| `npm run site:screenshots` | README skrinshotlari: `docs/screenshots/*.jpg` (1440x900, telefon 390x844) |

`dist/` papkasi istalgan statik hostda istalgan papka ostida ishlaydi: barcha havolalar nisbiy, sahifalar `#/try`,
`#/kb`, `#/ex` kabi hash bilan. Ilovaning o'zi (`bitg-settings.exe`) va bitg.uz sayti bu papkaga bog'liq emas.

Klaviatura tartiblari `fixtures/layouts.json` da (`bitg-tsf` dagi tartiblardan generatsiya qilingan):
`cargo run -q --manifest-path demo/fixtures/gen/Cargo.toml > demo/fixtures/layouts.json`.
