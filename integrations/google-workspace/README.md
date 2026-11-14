# bitg — Google Docs, Sheets va Slides uchun add-on

[O‘zbekcha](#ozbekcha) · [English](#english)

<a id="ozbekcha"></a>
## O‘zbekcha

Google Docs, Sheets va Slides ichida eski lotin (oʻ gʻ sh ch, har xil apostrof: `'` `‘` `ʻ` `` ` `` `’`) va kirill
matnni **yangi 2026 alifbosiga** (Ö Ğ Ş Ç) o‘giradigan Editor add-on. Menyu: **Kengaytmalar → bitg**.

| Muharrir | Belgilangan | Hammasi |
|---|---|---|
| Docs | belgilangan matn (qisman qator ham), jadval, ro‘yxat | barcha tablar (ichki tablar ham), sarlavha, pastki qism, izohlar |
| Sheets | belgilangan kataklar (bir nechta diapazon ham) | joriy varaqning to‘ldirilgan qismi |
| Slides (beta) | belgilangan matn, shakl, jadval katagi, slaydlar | barcha slaydlar + ma’ruzachi izohlari |

Panel (Kengaytmalar → bitg → Panelni ochish): sozlamalar (kirill, inglizcha so‘zlar himoyasi, tutuq belgisi),
sinab ko‘rish maydoni, o‘zbek / rus / ingliz interfeysi.

### Qanday ishlaydi (dvigatel tanlovi)

O‘girish **bitg yadrosi** bilan — bitg.uz va brauzer kengaytmasidagi aynan o‘sha `bitg_core.wasm` (Rust → WebAssembly).
Qoidalar JavaScript’da qayta yozilmagan. Uch variant ko‘rib chiqildi:

| Variant | Qaror |
|---|---|
| **(a) wasm Apps Script V8 ichida** | ✅ **Tanlandi.** Google’ning o‘zi V8 runtime’da `WebAssembly` borligini yozadi ([V8 runtime](https://developers.google.com/apps-script/guides/v8-runtime): “The WebAssembly API is the only built-in feature that operates in a non-blocking manner”). Matn Google serveridagi shu ijrodan chiqmaydi, `UrlFetchApp` yo‘q → `script.external_request` scope kerak emas. Menyu panelsiz ham ishlaydi, katta hujjat brauzerga ko‘chirilmaydi. |
| (b) wasm panel (iframe) ichida, brauzerda | Zaxira. Matn Google → brauzer → Google yo‘lini bosadi (2 marta ko‘chiriladi), “butun hujjat” uchun panel ochiq bo‘lishi shart, wasm har ochilishda ~1,6 MB HTML bilan yuklanadi. |
| (c) bitg.uz HTTP (`/mcp`, `convert_text`) | ❌ Matn uchinchi tomon serveriga ketadi, qo‘shimcha `script.external_request` scope, tarmoq/limit (120 so‘rov/daqiqa) va bitg.uz ishlamay qolishiga bog‘liqlik. bitg serverida boshqa o‘girish endpointi yo‘q (`bot/src/web.rs`: faqat `/api/health`, `/api/feedback`, `/mcp`). |

**wasm qanday joylanadi.** Apps Script loyihasi ikkilik fayl saqlamaydi. `tools/build.mjs` wasm’ni gzip (3,5 MB → 1,2 MB)
va base64 qilib `dist/Wasm.gs` ga ~512 KB bo‘laklarda yozadi; `Engine.gs` uni `Utilities.base64Decode` + `Utilities.ungzip`
bilan ochib, `new WebAssembly.Module` bilan sinxron kompilyatsiya qiladi. Tashqi URL’dan yuklash (SRI bilan ham)
rad etildi: sayt `bitg_core.wasm` ni har deploy’da almashtiradi (doimiy “pinned” manzil yo‘q) va tarmoq scope’i kerak bo‘lardi.
Apps Script’da `TextEncoder`/`TextDecoder` yo‘q — UTF-8 `Engine.gs` da qo‘lda.

Agar Google sinxron kompilyatsiyani rad etsa, `bitgWithCore_` avtomatik ravishda rasmiy asinxron naqshga
(`WebAssembly.instantiate`) o‘tadi.

**Formatlash saqlanadi.** Hujjat butunlay qayta yozilmaydi: `planEdits_` faqat o‘zgargan bo‘laklarni topadi
(`ko‘rishamiz` → faqat `o‘rish` → `öriş`), shuning uchun qalin/kursiv, rang, havola, izoh va kommentlar joyida qoladi.
Sheets’da yozish `RichTextValue` bilan: katak ichidagi format bo‘laklari va havolalar ko‘chiriladi, matn son yoki
formulaga aylanib qolmaydi. Formulali, son va sana kataklari o‘tkazib yuboriladi.

### Ruxsatlar (OAuth scopes)

| Scope | Nega |
|---|---|
| `documents.currentonly` | faqat **ochiq turgan** hujjatni o‘qish/yozish |
| `spreadsheets.currentonly` | faqat ochiq jadval |
| `presentations.currentonly` | faqat ochiq taqdimot |
| `script.container.ui` | menyu va panelni ko‘rsatish |

Drive, tarmoq (`script.external_request`), email yoki boshqa fayllarga ruxsat **yo‘q**. `@OnlyCurrentDoc` ham qo‘yilgan.

### Yig‘ish va test

```sh
# repo ildizidan
cargo build -p bitg-core --release --target wasm32-unknown-unknown --target-dir target/site
node integrations/google-workspace/tools/build.mjs        # → integrations/google-workspace/dist/
node --test integrations/google-workspace/test/*.test.cjs
```

Lokal testlar (Node, Google’siz): haqiqiy wasm bilan o‘girish (TextEncoder’siz, Apps Script kabi), sinxron va
asinxron yuklash, UTF-8, qayta yozish rejasi, soxta Docs/Sheets/Slides obyektlarida format saqlanishi, panel mantig‘i.
**Faqat Google ichida tekshiriladi:** `WebAssembly` va sinxron kompilyatsiyaning haqiqiy mavjudligi va vaqti,
`insertText`/`setAttributes` ning haqiqiy format xulqi, Ctrl+Z, katta hujjatda vaqt chegarasi (6 daqiqa), panel ko‘rinishi.

### Google’da sinash (clasp, nashrsiz)

1. `npm i -g @google/clasp` → `clasp login` (Apps Script API yoqilgan bo‘lsin: <https://script.google.com/home/usersettings>).
2. `cd integrations/google-workspace` → `clasp create --type standalone --title "bitg" --rootDir dist`
   (yoki mavjud loyiha uchun `.clasp.json.example` → `.clasp.json`, `scriptId` ni qo‘ying). `clasp create` `dist/` ichiga
   `.clasp.json` yozsa — uni `integrations/google-workspace/` ga ko‘chiring (`rootDir: "dist"`).
3. `node tools/build.mjs` → `clasp push -f`.
4. Apps Script muharririda `bitgDiagnose` funksiyasini **Run** qiling → Executions jurnalida `"sync":true` va `sample`
   (`Özbekiston şaharlari: Toşkent, Assalomu alaykum`) bo‘lishi kerak. `sync:false` bo‘lsa — `syncError` ni yozing.
5. **Deploy → Test deployments → Editor add-on → Add test** → Latest Code → test hujjatini tanlang → **Save test** →
   **Execute**. Docs, Sheets va Slides uchun alohida test qo‘shing.
6. Qo‘lda sinov ro‘yxati: qalin so‘z ichidagi `sh` → qalin `ş`; havola saqlanadi; jadval va sarlavha; kirill; Sheets’da
   formula/son/sana tegilmaydi, katak ichidagi qisman qalin saqlanadi; Slides matn qutisi va jadval; Ctrl+Z;
   ~100 sahifalik hujjatda vaqt.

Nashr (Marketplace, unlisted yoki public) — [STORE.md](STORE.md).

### Fayllar

| Fayl | Vazifa |
|---|---|
| `src/appsscript.json` | manifest: V8, 4 ta tor scope |
| `src/Code.gs` | menyu, panel, Docs/Sheets/Slides’ga qayta yozish |
| `src/Engine.gs` | wasm yuklash, UTF-8, `planEdits_`/`mapPos_` |
| `src/Sidebar.html` | panel (uz/ru/en, bitg.uz tokenlari) |
| `tools/build.mjs` | `dist/` + generatsiya qilingan `Wasm.gs` (git’da emas) |
| `test/*.test.cjs` | Node testlari |
| `.clasp.json.example` | clasp sozlamasi namunasi |

---

<a id="english"></a>
## English

An Editor add-on for Google Docs, Sheets and Slides that converts Uzbek text in old Latin (oʻ gʻ sh ch with any
apostrophe variant) and Cyrillic to the **new 2026 Latin alphabet** (Ö Ğ Ş Ç). Menu: **Extensions → bitg**.

- **Docs:** selection (including part of a line), or the whole document: every tab (and child tab), headers, footers, footnotes.
- **Sheets:** selected cells (multiple ranges too), or the used range of the current sheet.
- **Slides (beta):** selected text / shapes / table cells / slides, or every slide including speaker notes.

**Engine.** The same `bitg_core.wasm` as bitg.uz and the browser extension, run **inside the Apps Script V8 runtime**
(Google documents `WebAssembly` support in the [V8 runtime guide](https://developers.google.com/apps-script/guides/v8-runtime)).
Text never leaves that execution: there is no `UrlFetchApp` call and no network scope. The wasm is gzipped and
base64-embedded into a generated `Wasm.gs` (Apps Script cannot hold binary files); loading it from a URL was rejected
because the site replaces the file on every deploy and it would need `script.external_request`. If synchronous
compilation is ever refused, the code falls back to the documented async `WebAssembly.instantiate` pattern.
Running the wasm in the sidebar (option b) and calling bitg.uz/mcp (option c) were considered and rejected — see the
table in the Uzbek section.

**Formatting is preserved:** only the changed fragment of each word is replaced (`planEdits_`), so bold/italic,
colour, links and comments stay. In Sheets, writes use `RichTextValue` (per-run styles and links are remapped; text never
turns into a number or formula); formula, number and date cells are skipped.

**Scopes:** `documents.currentonly`, `spreadsheets.currentonly`, `presentations.currentonly`, `script.container.ui`.
No Drive, network, or email access.

**Build and test locally:**

```sh
cargo build -p bitg-core --release --target wasm32-unknown-unknown --target-dir target/site
node integrations/google-workspace/tools/build.mjs
node --test integrations/google-workspace/test/*.test.cjs
```

The Node tests cover the real wasm without `TextEncoder` (as in Apps Script), sync and async loading, the edit plan, and
the write-back logic against fake Docs/Sheets/Slides objects. Only Google can confirm: real `WebAssembly`
availability and timing, the exact formatting behaviour of `insertText`/`setAttributes`, undo, execution-time limits on
very large files, and the sidebar inside the editor.

**Try it in Google without publishing:** install clasp (`npm i -g @google/clasp`, `clasp login`, enable the Apps Script
API), `clasp create --type standalone --title "bitg" --rootDir dist` (or copy `.clasp.json.example` to `.clasp.json`),
`node tools/build.mjs && clasp push -f`, run `bitgDiagnose` once in the editor (expect `"sync":true`), then
**Deploy → Test deployments → Editor add-on → Add test → Latest Code → pick a test file → Save → Execute**.

Publishing (unlisted or public Marketplace listing): see [STORE.md](STORE.md).
