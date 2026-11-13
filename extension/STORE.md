# Do'kon materiallari

Chrome Web Store, Microsoft Edge Add-ons va addons.mozilla.org (AMO) uchun matnlar, ruxsat izohlari va maxfiylik
javoblari. Paketlar: `extension/dist/chrome.zip` (Chrome va Edge), `extension/dist/firefox.zip` (AMO) —
[README.md](README.md#reliz).

Misollar `bitg` CLI natijasidan olingan (o'sha yadro): `Ozbekiston gozal, goshtli osh` → `Özbekiston gözal, göştli oş`;
`Shu hafta push qilaman: https://chat.example.uz/shahar` → `Şu hafta push qilaman: https://chat.example.uz/shahar`;
`Ассалому алайкум, қўшни` → `Assalomu alaykum, qöşni`; `Photoshop va Chevrolet` o'zgarmaydi.

## Umumiy

| Maydon | Qiymat |
|---|---|
| Nom (≤45) | uz: `bitg — yangi o‘zbek alifbosi (Ö Ğ Ş Ç)` (38) · ru: `bitg — новый узбекский алфавит (Ö Ğ Ş Ç)` (40) · en: `bitg — new Uzbek alphabet (Ö Ğ Ş Ç)` (35) |
| Asosiy til | o'zbekcha (`default_locale: uz`); ru va en tarjimalari bor |
| Kategoriya | Chrome: **Productivity → Tools** · Edge: **Productivity** · AMO: **Language Support** (qo'shimcha: Other) |
| Sayt | https://bitg.uz |
| Yordam | https://github.com/dtwinuz/bitg/issues |
| Maxfiylik siyosati | https://bitg.uz/maxfiylik/ (pastdagi «Maxfiylik sahifasiga qo'shimcha» bo'limini nashr qilishdan oldin qo'shing) |
| Litsenziya / manba | MIT OR Apache-2.0 · https://github.com/dtwinuz/bitg/tree/main/extension |
| Firefox ID | `bitg@bitg.uz` (o'zgarmasin: yangilanishlar shu ID bilan bog'lanadi) |

Narxi: bepul. Reklama, hisob, to'lov yo'q.

## Qisqa tavsif (≤132)

Manifestdagi `extDescription` (`_locales/*/messages.json`) bilan bir xil:

- **uz** (119): Eski lotin va kirill matnni yangi o‘zbek alifbosiga (Ö Ğ Ş Ç) o‘giradi: belgilangan matn, sahifa, yozayotganda. Oflayn.
- **ru** (120): Переводит узбекский текст со старой латиницы и кириллицы на новый алфавит (Ö Ğ Ş Ç): выделение, страница, набор. Офлайн.
- **en** (123): Converts Uzbek text from old Latin and Cyrillic to the new 2026 alphabet (Ö Ğ Ş Ç): selection, whole page, typing. Offline.

## To'liq tavsif

### uz

```
bitg — yangi o‘zbek lotin alifbosi (Ö Ğ Ş Ç) uchun. Eski lotin (o‘, g‘, sh, ch) va kirill matnni bir bosishda yangi alifboga o‘giradi.

Oddiy «sh → ş» almashtirish emas. Ichida bitg yadrosi ishlaydi — bitg.uz konvertori va Windows/Mac/Linux klaviaturasi bilan bir xil dastur, lug‘at va qoidalar bilan:
• inglizcha so‘z, havola, pochta va brendlarga tegmaydi: «push», «https://…/shahar», «Photoshop» o‘zgarmaydi;
• tushib qolgan apostrofni tiklaydi: «Ozbekiston gozal» → «Özbekiston gözal»;
• kirill matnni ham o‘giradi: «Ассалому алайкум, қўшни» → «Assalomu alaykum, qöşni»;
• ikki xil o‘qiladigan so‘zni belgilab, «tekshiring» deydi.

Imkoniyatlar
• Oyna: matnni yozing yoki qo‘ying — natija darhol, o‘girilgan so‘zlar belgilangan, bitta tugma bilan nusxa.
• O‘ng tugma → «bitg: yangi alifboga o‘girish»: maydonda (xat, izoh, forma) matn joyida almashadi, Ctrl+Z qaytaradi; oddiy sahifa matni buferga nusxalanadi.
• Tezkor tugma Alt+Shift+Y (o‘zgartirish mumkin).
• «Sahifani yangi alifboda ko‘rish»: butun sahifa matni yangi alifboda, yana bir bosish — asl holi. Maydonlar, formalar va kodga tegilmaydi.
• Yozayotganda o‘girish — siz tanlagan saytlarda: probeldan keyin so‘z o‘giriladi, kursor joyida qoladi, darhol ⌫ bosilsa asl so‘z qaytadi. Standart holatda o‘chiq; yoqilganda brauzer faqat o‘sha sayt uchun ruxsat so‘raydi.
• Interfeys: o‘zbekcha, ruscha, inglizcha. Qorong‘i mavzu.

Maxfiylik
• Internetga ulanmaydi: matn brauzeringizda o‘giriladi, hech qayerga yuborilmaydi va saqlanmaydi.
• Analitika, reklama, hisob yo‘q. Saqlanadigan yagona narsa — sozlamalar (til va avto-o‘girish yoqilgan saytlar).
• Kod ochiq: github.com/dtwinuz/bitg

Eslatma: Google Docs matnni canvas’da chizadi, u yerda yozayotganda o‘girish ishlamaydi — matnni oynada o‘girib qo‘ying.
```

### ru

```
bitg — для нового узбекского латинского алфавита (Ö Ğ Ş Ç). Переводит текст со старой латиницы (o‘, g‘, sh, ch) и кириллицы в новый алфавит одним нажатием.

Это не простая замена «sh → ş». Внутри работает ядро bitg — та же программа, словари и правила, что в конвертере на bitg.uz и в клавиатуре для Windows/Mac/Linux:
• не трогает английские слова, ссылки, почту и бренды: «push», «https://…/shahar», «Photoshop» остаются как есть;
• восстанавливает пропущенный апостроф: «Ozbekiston gozal» → «Özbekiston gözal»;
• переводит и кириллицу: «Ассалому алайкум, қўшни» → «Assalomu alaykum, qöşni»;
• отмечает слова с двумя прочтениями — «проверьте».

Возможности
• Окно: напишите или вставьте текст — результат сразу, изменённые слова подсвечены, копирование одной кнопкой.
• Правая кнопка → «bitg: перевести в новый алфавит»: в поле ввода (письмо, комментарий, форма) текст заменяется на месте, Ctrl+Z отменяет; обычный текст страницы копируется в буфер.
• Горячая клавиша Alt+Shift+Y (можно изменить).
• «Показать страницу в новом алфавите»: весь текст страницы в новом алфавите, повторное нажатие — исходный вид. Поля, формы и код не затрагиваются.
• Перевод при наборе — на выбранных вами сайтах: слово переводится после пробела, курсор остаётся на месте, сразу ⌫ — вернуть исходное слово. По умолчанию выключено; при включении браузер запрашивает доступ только к этому сайту.
• Интерфейс: узбекский, русский, английский. Тёмная тема.

Конфиденциальность
• Не выходит в интернет: текст переводится в браузере, никуда не отправляется и не сохраняется.
• Без аналитики, рекламы и аккаунтов. Хранятся только настройки (язык и сайты с переводом при наборе).
• Открытый код: github.com/dtwinuz/bitg

Примечание: Google Docs рисует текст на canvas, перевод при наборе там не работает — переведите текст в окне и вставьте.
```

### en

```
bitg is built for the new Uzbek Latin alphabet (Ö Ğ Ş Ç). It converts text written in the old Latin script (o‘, g‘, sh, ch) or in Cyrillic to the new alphabet in one click.

This is not a simple "sh → ş" replace. It runs the bitg core — the same engine, dictionaries and rules as the converter on bitg.uz and the bitg keyboard for Windows, Mac and Linux:
• leaves English words, links, e-mail addresses and brands alone: "push", "https://…/shahar" and "Photoshop" stay as they are;
• restores missing apostrophes: "Ozbekiston gozal" → "Özbekiston gözal";
• converts Cyrillic too: "Ассалому алайкум, қўшни" → "Assalomu alaykum, qöşni";
• flags words that can be read two ways so you can check them.

Features
• Popup: type or paste text and get the result instantly, with converted words highlighted and one-click copy.
• Right-click → "bitg: convert to the new alphabet": in a text field (mail, comment, form) the text is replaced in place and Ctrl+Z undoes it; ordinary page text is copied to the clipboard.
• Keyboard shortcut Alt+Shift+Y (customisable).
• "View page in the new alphabet": shows the whole page in the new alphabet, click again to restore. Inputs, forms and code are never touched.
• Convert while typing, on the sites you choose: a word is converted after you press space, the cursor stays put, and pressing ⌫ right away brings the original word back. Off by default; when you turn it on, the browser asks for access to that one site only.
• Interface in Uzbek, Russian and English. Dark mode.

Privacy
• No network access: text is converted inside your browser and is never sent anywhere or stored.
• No analytics, ads or accounts. The only stored data are your settings (language and the sites where convert-while-typing is on).
• Open source: github.com/dtwinuz/bitg

Note: Google Docs draws text on a canvas, so convert-while-typing does not work there; convert the text in the popup and paste it.
```

## Skrinshotlar

Chrome/Edge: 1280×800 (yoki 640×400), PNG, 3–5 ta; AMO: istalgan o'lcham, 1280×800 tavsiya. Har til uchun alohida
(interfeys tilini popup'da almashtirib). Popup'ning o'zi `SHOTS=<papka> node extension/test/smoke.cjs` bilan
olinadi; to'liq kadr uchun brauzer oynasida qo'lda.

1. Popup: `Shu hafta chorshanba kuni push qilaman, linkni https://meet.example.uz/abc ga tashlayman. O‘zbekiston go‘zal!` — o'girilgan so'zlar belgilangan, `push` va havola tegilmagan (yorug' mavzu).
2. Xuddi shu, qorong'i mavzu.
3. Kontekst menyu: Gmail/Telegram Web yozish maydonida belgilangan matn → «bitg: yangi alifboga o‘girish» va natija.
4. Yangiliklar sayti (masalan kun.uz yoki gazeta.uz maqolasi) — «Sahifani yangi alifboda ko‘rish» oldin/keyin (ikki kadr yonma-yon).
5. Yozayotganda o'girish: popup'dagi kalit yoqilgan, maydonda `şahar ` paydo bo'lgan lahza; brauzerning «faqat shu sayt» ruxsat oynasi.

Promo rasmlar (Chrome): kichik plitka 440×280 (majburiy) — logo, «Ö Ğ Ş Ç» va «Eski → yangi alifbo»; marquee
1400×560 (ixtiyoriy). Ikonka: `icons/icon-128.png`.

## Yagona maqsad (Chrome «Single purpose»)

> Converting Uzbek text from the old Latin and Cyrillic scripts to the new Uzbek Latin alphabet (Ö Ğ Ş Ç) — in the
> popup, in selected text, on the current page on demand, and optionally while typing on sites the user enables.

## Ruxsatlar izohi

| Ruxsat | Nima uchun (do'kon formasiga) |
|---|---|
| `contextMenus` | Adds the "bitg: convert to the new alphabet" item for selected text and the "view page in the new alphabet" item. |
| `activeTab` | When the user clicks the menu item, the toolbar button or the shortcut, the extension gets temporary access to that one tab to read the selection and write the converted text back. No access to any tab otherwise. |
| `scripting` | Injects the conversion script into the active tab only after a user action (activeTab), and registers the convert-while-typing script only on sites the user explicitly enabled. |
| `storage` | Stores settings only: interface language and the list of sites where convert-while-typing is on (`chrome.storage.sync`). No text is ever stored. |
| `clipboardWrite` | Copies the converted text to the clipboard when the selection is not editable (ordinary page text), and from the popup's Copy button. The extension never reads the clipboard. |
| `optional_host_permissions` `https://*/*`, `http://*/*` | Not granted at install. Requested for a single site (`https://example.com/*`) only when the user turns on "Convert while typing" for that site, and removed when they turn it off. Needed so the typing script runs on that site without a click each time. |

Remote code: **No.** Hamma kod paket ichida; yadro `core/bitg_core.wasm` — `crates/bitg-core` dan yig'ilgan
WebAssembly (sayt bilan bir xil). CSP: `script-src 'self' 'wasm-unsafe-eval'; object-src 'self'; connect-src 'self'`
— kengaytma sahifalari tashqi manzilga so'rov yubora olmaydi. `wasm-unsafe-eval` faqat paketdagi wasm'ni
kompilyatsiya qilish uchun.

## Maxfiylik amaliyoti

**Chrome Web Store → Privacy practices:**

- Collected data: hech bir toifa belgilanmaydi (Personally identifiable information, Health, Financial, Authentication,
  Personal communications, Location, Web history, User activity, Website content — **yo'q**). Matn faqat brauzer
  xotirasida o'giriladi, saqlanmaydi va uzatilmaydi.
- Uch tasdiq: ✓ not sold to third parties · ✓ not used for purposes unrelated to the single purpose · ✓ not used for
  creditworthiness or lending.

**Edge Partner Center:** Privacy policy URL — https://bitg.uz/maxfiylik/; «Does your product access, collect or
transmit personal information?» — **No**.

**AMO:** `data_collection_permissions: { required: ["none"] }` manifestda (build.mjs qo'shadi). «This add-on does not
collect or transmit any data». Manba kodi so'raladi (wasm — kompilyatsiya qilingan kod): `git archive` arxivi va
README «Yig'ish» bo'limi; ko'rib chiquvchi Linux'da `cargo build -p bitg-core --release --target wasm32-unknown-unknown --target-dir target/site` va
`node extension/tools/build.mjs` bilan bir xil `firefox.zip` oladi.

## Maxfiylik sahifasiga qo'shimcha

https://bitg.uz/maxfiylik/ hozir Windows dasturi, konvertor, bot va saytni yoritadi; do'konga topshirishdan oldin
`web/pages/maxfiylik.html` (va ru/en nusxalari) ga shu bo'lim qo'shilishi kerak:

> **Brauzer kengaytmasi (Chrome, Edge, Firefox)**
> - Tarmoq yo'q. bitg yadrosi kengaytma ichida (WebAssembly), matn brauzeringizda o'giriladi va hech qayerga
>   yuborilmaydi; kengaytmaning xavfsizlik siyosati (CSP) tashqi so'rovlarni taqiqlaydi.
> - Matn saqlanmaydi. Brauzer sinxron xotirasida faqat sozlamalar: interfeys tili va «yozayotganda o'girish» yoqilgan
>   saytlar ro'yxati.
> - Sahifaga faqat siz bosganda kiradi (kontekst menyu, tugma yoki tezkor tugma) — o'sha bitta tab uchun. Yozayotganda
>   o'girish faqat siz yoqqan saytda ishlaydi; o'chirilganda brauzer ruxsati ham olib tashlanadi.
> - Bufer faqat yozish uchun: belgilangan matn tahrirlanmaydigan joyda bo'lsa, o'girilgani buferga nusxalanadi.
>   Kengaytma buferni o'qimaydi.
> - Analitika, reklama, cookie yo'q.

## Do'kon hisoblari bo'yicha qo'lda qadamlar

- Chrome Web Store developer hisobi (bir martalik $5 to'lov), dtwinuz nomidan; **2-bosqichli tasdiq** majburiy.
  Dashboard'da «Privacy» va «Distribution» (barcha mintaqalar), «Store listing» uz/ru/en.
- Microsoft Partner Center — Edge Add-ons dasturi (bepul).
- addons.mozilla.org hisobi (bepul); `bitg@bitg.uz` ID birinchi yuklashda bog'lanadi.
- Har do'konda kategoriya, skrinshotlar va promo plitkani yuklash; birinchi ko'rib chiqish odatda bir necha kun.
