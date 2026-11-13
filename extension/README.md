# bitg brauzer kengaytmasi

Chrome, Edge va Firefox uchun bitta kod (Manifest V3). O'zbekcha matnni eski lotin va kirilldan yangi alifboga
(Ö Ğ Ş Ç) o'giradi. O'girishni **bitg yadrosi** bajaradi: `crates/bitg-core` WebAssembly'ga yig'iladi, bitg.uz
konvertori bilan bir xil fayl va bir xil ABI (`web/app.js` → `core()`). JS'da o'girish qoidasi yo'q.

| Imkoniyat | Qanday ishlaydi | Ruxsat |
|---|---|---|
| Popup o'giruvchi | yadro popup'ning o'zida, oflayn; o'girilgan so'zlar belgilanadi, ikki xil o'qiladigan so'z «tekshiring» | — |
| Kontekst menyu «bitg: yangi alifboga o'girish» | tahrir maydonida joyida almashtiradi (input, textarea, contenteditable; Ctrl+Z qaytaradi), oddiy matnda buferga nusxalaydi va xabarcha chiqaradi | `contextMenus`, `activeTab`, `scripting`, `clipboardWrite` |
| Tezkor tugma (standart Alt+Shift+Y) | xuddi menyu kabi, fokusdagi freymda | `activeTab`, `scripting` |
| «Sahifani yangi alifboda ko'rish» | ko'rinadigan matn tugunlarini o'giradi, qayta bosilsa asli tiklanadi; keyin qo'shilgan matn ham o'giriladi | `activeTab`, `scripting` |
| Yozayotganda o'girish (saytma-sayt, standart o'chiq) | probel yoki tinish belgisidan keyin so'z o'giriladi, kursor joyida qoladi; darhol ⌫ — asl so'z qaytadi va shu sahifada boshqa o'girilmaydi | ixtiyoriy host ruxsati faqat o'sha saytga |
| Interfeys tili | o'zbekcha (standart), ruscha, inglizcha | `storage` |

Tegilmaydi: `input`/`textarea`/`select` qiymatlari (sahifa o'girishida), `script`, `style`, `code`, `pre`, `kbd`,
`svg`, contenteditable, `translate="no"`, `.notranslate`, kod muharrirlari (Monaco, CodeMirror, Ace), parol, pochta,
URL va raqam maydonlari. Yozayotganda havola, pochta, `#teg` va raqamga yopishgan so'zlar ham o'tkazib yuboriladi.

**Google Docs/Slides/Sheets** matnni canvas'da chizadi: u yerda yozayotganda o'girish ishlamaydi (popup buni
ko'rsatadi). Matnni popup'da o'girib qo'yish kerak.

## Tuzilishi

```
extension/
  manifest.json          Chrome/Edge manifesti (Firefox'niki build vaqtida shundan yasaladi)
  background.js          fon: yadro, menyu, tezkor tugma, saytlar ro'yxati, kontent skriptni ro'yxatga olish
  core/core.js           wasm yuklovchi (web/app.js bilan bir xil chaqiruv)
  i18n.js                interfeys matnlari uz/ru/en
  content/lib.js         DOM'siz sof mantiq (testlanadi): matn tugunlari, so'z topish, kursor
  content/edit.js        maydonda almashtirish (execCommand insertText → setRangeText)
  content/selection.js   belgilangan matn       content/page.js   butun sahifa
  content/autotype.js    yozayotganda           content/toast.js  xabarcha (yopiq shadow DOM)
  popup/                 popup (bitg.uz tokenlari, qorong'i mavzu)
  _locales/{uz,ru,en}/   do'kon nomi va tavsifi
  icons/                 web/icon.svg dan (tools/icons.cjs)
  tools/build.mjs        yig'ish → dist/{chrome,firefox}/ va dist/{chrome,firefox}.zip
  test/                  node --test birlik testlari, smoke.cjs — Chromium'da tutun sinovi
```

## Yig'ish

Kerak: Rust (`wasm32-unknown-unknown` target), Node 20+. Boshqa bog'liqlik yo'q.

```sh
rustup target add wasm32-unknown-unknown
# sayt bilan bir xil buyruq (deploy/deploy.sh)
cargo build -p bitg-core --release --target wasm32-unknown-unknown --target-dir target/site
node extension/tools/build.mjs            # yoki: node extension/tools/build.mjs --cargo
```

Natija:

- `extension/dist/chrome/`, `extension/dist/firefox/` — «load unpacked» uchun;
- `extension/dist/chrome.zip` — Chrome Web Store va Edge Add-ons;
- `extension/dist/firefox.zip` — addons.mozilla.org.

Arxivlar takrorlanuvchan (fayllar tartiblangan, vaqt 1980-01-01, matn LF): bir xil wasm va manba → bir xil SHA-256.
Firefox manifesti farqi: `background.scripts` (Firefox MV3'da service worker yo'q), `browser_specific_settings.gecko`
(`bitg@bitg.uz`, `strict_min_version` 140 — `optional_host_permissions` va `data_collection_permissions` uchun).

## O'rnatish (ishlab chiqish)

- **Chrome / Edge:** `chrome://extensions` (`edge://extensions`) → «Developer mode» → «Load unpacked» →
  `extension/dist/chrome`.
- **Firefox:** `about:debugging#/runtime/this-firefox` → «Load Temporary Add-on» → `extension/dist/firefox/manifest.json`.

Kod o'zgarsa, `build.mjs` ni qayta ishga tushirib, kengaytmani «Reload» qiling.

## Testlar

```sh
node --test extension/test/*.test.cjs
```

- `lib.test.cjs` — matn tugunlarini tanlash (soxta DOM bilan), so'z topish, kursor siljishi, ⌫ bilan qaytarish;
- `core.test.cjs` — haqiqiy `bitg_core.wasm` (`target/site/...` yoki `BITG_WASM=`) core.js orqali;
- `package.test.cjs` — ruxsatlar ro'yxati, CSP, tarjimalar to'liqligi, do'kon chegaralari, ZIP.

Brauzerdagi tutun sinovi (popup, sahifa, belgilangan matn, yozayotganda; tashqi so'rovlar soni = 0):

```sh
NODE_PATH="$(npm root -g)/@playwright/cli/node_modules" CHROME=<chromium>/chrome.exe node extension/test/smoke.cjs
# SHOTS=<papka> — popup skrinshotlari (yorug' va qorong'i)
```

`CHROME` — to'liq Chromium (headless shell kengaytma yuklamaydi), masalan Playwright'ning `chromium-1248`.

Qo'lda tekshirish (har relizdan oldin, Chrome va Firefox'da):

1. Popup: `o‘zbek tili, shahar, choy` → `özbek tili, şahar, çoy`; kirill matn; «Nusxa olish».
2. Biror saytda matnni belgilab, o'ng tugma → «bitg: yangi alifboga o‘girish» → buferda yangi matn, xabarcha.
3. Textarea'da belgilab menyu yoki Alt+Shift+Y → joyida almashadi, Ctrl+Z qaytaradi.
4. Popup → «Sahifani yangi alifboda ko‘rish» → matn o'zgaradi, maydonlar va kod o'zgarmaydi; qayta bosish — asli.
5. Popup → «Yozayotganda o‘girish» → brauzer faqat shu saytga ruxsat so'raydi; `shahar ` → `şahar `; darhol ⌫ —
   `shahar ` qaytadi. O'chirilganda ruxsat ham olib tashlanadi (`chrome://extensions` → Details → Site access).
6. `chrome://extensions/shortcuts` da tugmani o'zgartirish.

## Reliz

1. `manifest.json` dagi `version` ni oshiring (do'konlar bir xil versiyani qayta qabul qilmaydi).
2. CI (`.github/workflows/extension.yml`) yashil bo'lsin; do'konga **CI'ning `extension` artifact'idagi**
   `chrome.zip` va `firefox.zip` yuklanadi. Linux'da bir xil Rust toolchain bilan qayta yig'ilsa SHA-256 bir xil
   (CI buni ikki marta yig'ib tekshiradi). Windows'da yig'ilgan wasm farq qiladi: panic joylari yo'lida `\`.
3. **Chrome Web Store** (Developer Dashboard) → yangi paket → `chrome.zip`. **Edge Add-ons** (Partner Center) →
   o'sha `chrome.zip`. **AMO** → `firefox.zip`; wasm mashina kodi bo'lgani uchun manba arxivi ham so'raladi:
   `git archive --format=zip -o bitg-src.zip HEAD` va ko'rib chiquvchi uchun yig'ish yo'riqnomasi (yuqoridagi
   «Yig'ish» bo'limi). Do'kon matnlari va ruxsat izohlari — [STORE.md](STORE.md).
