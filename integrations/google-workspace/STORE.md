# Google Workspace Marketplace materiallari

Editor add-on (Docs, Sheets, Slides) uchun listing matnlari, grafika rejasi, scope izohlari va nashr qadamlari.
Hech narsa hali nashr qilinmagan: Google Cloud loyihasi, OAuth consent screen va Marketplace listingini egasi o‘zi yaratadi.

Talablar manbasi: [Create a listing](https://developers.google.com/workspace/marketplace/create-listing),
[Configure the Marketplace SDK](https://developers.google.com/workspace/marketplace/enable-configure-sdk),
[App review](https://developers.google.com/workspace/marketplace/about-app-review),
[Editor add-on scopes](https://developers.google.com/workspace/add-ons/concepts/editor-scopes),
[Test editor add-ons](https://developers.google.com/workspace/add-ons/how-tos/testing-editor-addons).

Misollar haqiqiy yadro natijasidan (`test/engine.test.cjs`): `O'zbekiston shaharlari: Toshkent` →
`Özbekiston şaharlari: Toşkent`; `Ассалому алайкум! Эртага чойхонада кўришамиз` →
`Assalomu alaykum! Ertaga çoyxonada körişamiz`; `Shu hafta push qilaman: https://chat.example.uz/shahar` →
`Şu hafta push qilaman: https://chat.example.uz/shahar`.

## Umumiy

| Maydon | Qiymat |
|---|---|
| Nom (≤50) | `bitg — yangi o‘zbek alifbosi` (28). **OAuth consent screen’dagi App name bilan aynan bir xil bo‘lishi shart** (review talabi) |
| Til | o‘zbekcha (asosiy); ru va en listing tarjimalari pastda |
| Kategoriya | **Productivity** (yoki Education) |
| Narx | bepul; reklama, hisob, to‘lov yo‘q |
| Ko‘rinish | avval **Unlisted** (faqat to‘g‘ridan-to‘g‘ri havola), keyin Public. ⚠️ Marketplace SDK’da tanlangan visibility keyin **o‘zgartirib bo‘lmaydi** (Public/Private) — Public + “Unlisted” belgisi tanlang |
| O‘rnatish | Individual + Admin install |
| Integratsiya | Editor add-on: Docs ✔, Sheets ✔, Slides ✔ |
| Terms of service | https://bitg.uz/en/terms/ |
| Privacy policy | https://bitg.uz/en/privacy/ (pastdagi qo‘shimchani nashrdan OLDIN qo‘shing) |
| Support | https://github.com/dtwinuz/bitg/issues (yoki tech@bitg.uz) |
| Developer website | https://bitg.uz |
| Manba | https://github.com/dtwinuz/bitg/tree/main/integrations/google-workspace (MIT OR Apache-2.0) |

## Qisqa tavsif (≤200)

- **uz** (147): Eski lotin va kirill o‘zbek matnini yangi alifboga (Ö Ğ Ş Ç) o‘giradi: Docs, Sheets va Slides’da belgilangan joy yoki butun fayl. Format saqlanadi.
- **ru** (153): Переводит узбекский текст со старой латиницы и кириллицы на новый алфавит (Ö Ğ Ş Ç) в Docs, Sheets и Slides: выделение или весь файл. Формат сохраняется.
- **en** (153): Converts Uzbek text from old Latin and Cyrillic to the new 2026 alphabet (Ö Ğ Ş Ç) in Docs, Sheets and Slides: selection or whole file. Keeps formatting.

## To‘liq tavsif (≤16 000)

### uz

```
bitg — yangi o‘zbek lotin alifbosi (Ö Ğ Ş Ç) uchun. Google Docs, Sheets va Slides’dagi eski lotin (o‘, g‘, sh, ch) va kirill matnni bir bosishda yangi alifboga o‘giradi.

Oddiy «sh → ş» almashtirish emas. Ichida bitg yadrosi ishlaydi — bitg.uz konvertori, brauzer kengaytmasi va Windows/Mac/Linux klaviaturasi bilan bir xil dastur, lug‘at va qoidalar:
• inglizcha so‘z, havola va pochtaga tegmaydi: «push», «https://…/shahar» o‘zgarmaydi;
• har xil apostrofni tushunadi: o‘ oʻ o' o` o’;
• kirill matnni ham o‘giradi: «Ассалому алайкум» → «Assalomu alaykum».

Imkoniyatlar
• Docs: belgilangan matn yoki butun hujjat — barcha tablar, jadvallar, sarlavha, pastki qism va izohlar.
• Sheets: belgilangan kataklar yoki butun varaq. Formula, son va sanaga tegilmaydi.
• Slides: belgilangan matn yoki butun taqdimot, ma’ruzachi izohlari bilan.
• Format saqlanadi: faqat o‘zgargan harflar almashadi — qalin, kursiv, rang, havola va kommentlar joyida qoladi.
• Natija yoqmasa — Ctrl+Z.
• Panel: sozlamalar (kirill, inglizcha so‘zlar, tutuq belgisi) va sinab ko‘rish maydoni. Interfeys: o‘zbekcha, ruscha, inglizcha.

Maxfiylik
• Matn Google ichida, faqat ochiq fayl uchun ishlanadi — bitg serveriga yoki boshqa joyga yuborilmaydi, saqlanmaydi.
• Faqat ochiq turgan faylga ruxsat (“currentonly”): Drive’dagi boshqa fayllar, email yoki internetga ruxsat yo‘q.
• Analitika, reklama, hisob yo‘q. Kod ochiq: github.com/dtwinuz/bitg
```

### ru

```
bitg — для нового узбекского латинского алфавита (Ö Ğ Ş Ç). Переводит текст на старой латинице (o‘, g‘, sh, ch) и кириллице в Google Docs, Sheets и Slides на новый алфавит в один клик.

Это не простая замена «sh → ş». Внутри работает ядро bitg — та же программа, словарь и правила, что в конвертере bitg.uz, расширении браузера и клавиатуре для Windows/Mac/Linux:
• не трогает английские слова, ссылки и почту: «push», «https://…/shahar» не меняются;
• понимает любые апострофы: o‘ oʻ o' o` o’;
• переводит и кириллицу: «Ассалому алайкум» → «Assalomu alaykum».

Возможности
• Docs: выделенный текст или весь документ — все вкладки, таблицы, колонтитулы и сноски.
• Sheets: выделенные ячейки или весь лист. Формулы, числа и даты не затрагиваются.
• Slides: выделение или вся презентация, включая заметки докладчика.
• Форматирование сохраняется: меняются только нужные буквы — жирный, курсив, цвет, ссылки и комментарии остаются.
• Не понравилось — Ctrl+Z.
• Панель: настройки и поле для пробы. Интерфейс: узбекский, русский, английский.

Конфиденциальность
• Текст обрабатывается внутри Google только для открытого файла — не отправляется на сервер bitg или куда-либо ещё и не сохраняется.
• Доступ только к открытому файлу (“currentonly”): нет доступа к другим файлам Drive, почте или интернету.
• Без аналитики, рекламы и аккаунтов. Открытый код: github.com/dtwinuz/bitg
```

### en

```
bitg — for the new Uzbek Latin alphabet (Ö Ğ Ş Ç). Converts old Latin (o‘, g‘, sh, ch) and Cyrillic text in Google Docs, Sheets and Slides to the new alphabet in one click.

Not a plain “sh → ş” replace. It runs the bitg engine — the same program, dictionary and rules as the bitg.uz converter, the browser extension and the Windows/Mac/Linux keyboard:
• leaves English words, links and email addresses alone: “push” and “https://…/shahar” stay as they are;
• understands every apostrophe variant: o‘ oʻ o' o` o’;
• converts Cyrillic too: “Ассалому алайкум” → “Assalomu alaykum”.

Features
• Docs: the selection or the whole document — every tab, tables, headers, footers and footnotes.
• Sheets: selected cells or the whole sheet. Formulas, numbers and dates are not touched.
• Slides: the selection or the whole presentation, including speaker notes.
• Formatting is kept: only the letters that change are replaced, so bold, italic, colour, links and comments stay.
• Don’t like the result? Ctrl+Z.
• Panel with options and a try-it box. Interface in Uzbek, Russian and English.

Privacy
• Text is processed inside Google, for the open file only — it is never sent to a bitg server or anywhere else, and never stored.
• Access to the open file only (“currentonly”): no access to other Drive files, email or the internet.
• No analytics, ads or accounts. Open source: github.com/dtwinuz/bitg
```

## Grafika

| Asset | O‘lcham | Manba / reja |
|---|---|---|
| App icon | 32×32 va 128×128 (shart) | `extension/icons/icon-32.png`, `extension/icons/icon-128.png` (web/icon.svg dan, tayyor) |
| OAuth consent screen logo | 120×120 | `web/icon.svg` dan (`extension/tools/icons.cjs` ga 120 qo‘shib) — logo qo‘yilsa brand verification kerak |
| Card banner | 220×140 (shart) | qora `#1b1e26` fon, chapda logo, o‘ngda “sh → ş” birlashish; `web/og.png` uslubida |
| Screenshot 1–5 | 1280×800 (shart: kamida 1, Google xizmati bilan integratsiya ko‘rinsin), to‘g‘ri burchak, padding’siz | quyidagi reja |

Screenshot rejasi (haqiqiy Google oynasida, test deployment bilan olinadi):

1. **Docs + panel:** chapda “Oʻzbekiston Respublikasi…” eski matnli hujjat, o‘ngda bitg paneli — “Belgilanganni o‘girish”.
2. **Docs natija:** o‘sha hujjat yangi alifboda, qalin/havola saqlangan; status “Tayyor: N ta joy o‘zgartirildi”.
3. **Sheets:** kirill ustunli jadval → Kengaytmalar → bitg menyusi; natija va formulali ustun o‘zgarmagan.
4. **Slides:** slayd sarlavhasi va jadval yangi alifboda.
5. **Panel yaqindan:** sozlamalar va “Sinab ko‘rish” (ru yoki en interfeys).

Panel maketi (Node + Chrome, Google’siz) — lokal tekshiruvda 300 px kenglikda gorizontal aylantirishsiz chiqdi.

## Scope izohlari (OAuth consent screen va Marketplace SDK’ga)

Uch joyda bir xil ro‘yxat bo‘lishi shart: `src/appsscript.json`, OAuth consent screen, Marketplace SDK → OAuth Scopes.

| Scope | Justification (en, consent screen uchun) |
|---|---|
| `https://www.googleapis.com/auth/documents.currentonly` | Reads the text of the document the user has open and replaces old-alphabet characters in place when the user clicks “Convert”. No other document is accessed. |
| `https://www.googleapis.com/auth/spreadsheets.currentonly` | Reads text cells of the open spreadsheet (selection or current sheet) and writes the converted text back. |
| `https://www.googleapis.com/auth/presentations.currentonly` | Reads text of the open presentation (selection or all slides) and writes the converted text back. |
| `https://www.googleapis.com/auth/script.container.ui` | Shows the add-on menu and the sidebar inside Docs, Sheets and Slides. |

Tasnif (sensitive / non-sensitive) Google hujjatida yozilmagan — Cloud Console’da scope qo‘shilganda ko‘rinadi. Agar
birortasi “sensitive” chiqsa: verification’ga yuqoridagi izoh + YouTube (unlisted) demo video (consent → menyu →
o‘girish → natija, har uch muharrir) qo‘shiladi. Restricted scope yo‘q → security assessment kerak emas.

## Maxfiylik sahifasiga qo‘shimcha (nashrdan oldin)

`web/pages/en/privacy.html` va `web/pages/maxfiylik.html` ochiq PR #100 da o‘zgarmoqda — shu PR merge bo‘lgach qo‘shing.
Google API Services User Data Policy (Limited Use) bayonoti talab qilinadi:

```
Google Docs, Sheets and Slides add-on
• The add-on reads and changes only the file you have open, and only when you choose “Convert” (scopes: documents.currentonly, spreadsheets.currentonly, presentations.currentonly; script.container.ui for the menu and sidebar).
• Conversion runs inside Google Apps Script with the bitg engine built into the add-on. Text is not sent to bitg.uz or any other server and is not stored, logged or used for anything else. The add-on makes no network requests.
• The sidebar remembers only its own settings (interface language and three options) in your browser.
• bitg’s use and transfer of information received from Google APIs adheres to the Google API Services User Data Policy, including the Limited Use requirements.
```

## Nashr qadamlari (egasi bajaradi)

1. **Google Cloud loyihasi** (standart, masalan `bitg-workspace`): APIs → **Google Workspace Marketplace SDK** ni yoqing.
2. **OAuth consent screen:** User type **External**; App name `bitg — yangi o‘zbek alifbosi`; support email
   tech@bitg.uz; logo (120×120); App domain: homepage https://bitg.uz, privacy https://bitg.uz/en/privacy/,
   terms https://bitg.uz/en/terms/; Authorized domain `bitg.uz` (Search Console’da tasdiqlangan); 4 ta scope (yuqorida).
   Publishing status → **In production** (Testing holatida review rad etiladi) → verification’ga yuboring.
3. **Apps Script:** README’dagi clasp qadamlari (`build.mjs` → `clasp push -f`) → `bitgDiagnose` (`sync:true`) →
   Test deployments bilan qo‘lda sinov ro‘yxati. Project Settings → **Google Cloud Platform project** → 1-qadamdagi
   loyiha raqamini qo‘ying.
4. **Versiya:** Deploy → New deployment → Add-on → versiya raqamini yozib oling (Manage deployments → Configuration).
5. **Marketplace SDK → App Configuration:** visibility (Public, Unlisted belgisi bilan), Individual + Admin install,
   Editor add-on → Docs/Sheets/Slides, Script ID (Project Settings → IDs) va versiya, 4 ta scope, developer ma’lumotlari.
6. **Store Listing:** yuqoridagi matnlar (uz asosiy, ru/en tarjimalar), icon 32/128, banner 220×140, screenshotlar,
   terms/privacy/support URL → **Publish**. Review odatda bir necha kun.
7. Yangi versiya: `build.mjs` → `clasp push -f` → Manage deployments → Edit → New version → Marketplace SDK’da versiya
   raqamini yangilang.
