// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Interfeys matnlari (popup, kontekst menyu, xabarchalar). Standart til — o'zbekcha; tanlov chrome.storage.sync'da.
// Manifestdagi nom va tavsif _locales/ da (brauzer tili bo'yicha). O'zbekcha matn sayt kabi eski imloda:
// o‘ g‘ = U+2018, tutuq = U+2019.
(function (root) {
  'use strict';
  var S = {
    uz: {
      langName: 'O‘zbekcha',
      menuConvert: 'bitg: yangi alifboga o‘girish',
      menuPage: 'bitg: sahifani yangi alifboda ko‘rish',
      toastCopied: 'Yangi alifboda nusxalandi',
      toastReplaced: 'Yangi alifboga o‘girildi · Ctrl+Z — qaytarish',
      toastNothing: 'O‘giriladigan narsa yo‘q — matn allaqachon yangi alifboda',
      toastSelect: 'Avval matnni belgilang',
      toastStale: 'Matn o‘zgarib qoldi — qayta urinib ko‘ring',
      toastCopyFailed: 'Nusxalab bo‘lmadi — bitg oynasidan foydalaning',
      toastPageOn: 'Sahifa yangi alifboda. Asl holiga qaytarish — yana bir marta',
      toastPageOff: 'Sahifa asl holiga qaytdi',
      toastPageNone: 'Bu sahifada o‘giriladigan matn topilmadi',
      tagline: 'Yangi o‘zbek alifbosi',
      inLabel: 'Eski lotin yoki kirill matn',
      inPlaceholder: 'Matnni shu yerga yozing yoki qo‘ying: o‘zbek tili, shahar, choy',
      outLabel: 'Yangi alifboda',
      outEmpty: 'Natija shu yerda chiqadi',
      copy: 'Nusxa olish',
      copied: 'Nusxalandi',
      clear: 'Tozalash',
      converted: '{n} ta so‘z o‘girildi',
      unchanged: 'O‘zgarish yo‘q',
      checkAlt: '{n} ta so‘zni tekshiring',
      altTitle: 'Ikki xil o‘qiladi. Boshqa o‘qilishi: {alt}',
      protectedTitle: 'Tegilmadi: havola, pochta, inglizcha so‘z yoki istisno',
      pageSection: 'Shu sahifa',
      pageOn: 'Sahifani yangi alifboda ko‘rish',
      pageOff: 'Asl holiga qaytarish',
      pageHint: 'Faqat ko‘rinadigan matn: maydonlar, formalar va kodga tegilmaydi.',
      autotype: 'Yozayotganda o‘girish',
      autotypeHint: 'Probel yoki tinish belgisidan keyin so‘z o‘giriladi. Darhol ⌫ — asl so‘z qaytadi.',
      autotypeAsk: 'Yoqilganda brauzer faqat shu sayt uchun ruxsat so‘raydi.',
      autotypeDocs: 'Google Docs matnni canvas’da chizadi — yozayotganda o‘girish u yerda ishlamaydi. Matnni shu oynada o‘giring va qo‘ying.',
      restricted: 'Brauzer o‘z sahifalarida kengaytmalarga ruxsat bermaydi. Oddiy sayt oching.',
      shortcut: 'Belgilangan matn: {key}',
      shortcutNone: 'Belgilangan matn uchun tugma tayinlanmagan',
      shortcutEdit: 'O‘zgartirish',
      privacy: 'Oflayn ishlaydi · matn hech qayerga yuborilmaydi',
      langLabel: 'Interfeys tili',
      loadFailed: 'bitg yadrosi yuklanmadi'
    },
    ru: {
      langName: 'Русский',
      menuConvert: 'bitg: перевести в новый алфавит',
      menuPage: 'bitg: показать страницу в новом алфавите',
      toastCopied: 'Скопировано в новом алфавите',
      toastReplaced: 'Переведено в новый алфавит · Ctrl+Z — отменить',
      toastNothing: 'Нечего переводить — текст уже в новом алфавите',
      toastSelect: 'Сначала выделите текст',
      toastStale: 'Текст изменился — попробуйте ещё раз',
      toastCopyFailed: 'Не удалось скопировать — воспользуйтесь окном bitg',
      toastPageOn: 'Страница в новом алфавите. Вернуть — ещё раз',
      toastPageOff: 'Страница возвращена к исходному виду',
      toastPageNone: 'На странице нет текста для перевода',
      tagline: 'Новый узбекский алфавит',
      inLabel: 'Текст на старой латинице или кириллице',
      inPlaceholder: 'Напишите или вставьте текст: o‘zbek tili, shahar, choy',
      outLabel: 'В новом алфавите',
      outEmpty: 'Здесь появится результат',
      copy: 'Копировать',
      copied: 'Скопировано',
      clear: 'Очистить',
      converted: 'Переведено слов: {n}',
      unchanged: 'Без изменений',
      checkAlt: 'Проверьте слова: {n}',
      altTitle: 'Читается двояко. Другой вариант: {alt}',
      protectedTitle: 'Не тронуто: ссылка, почта, английское слово или исключение',
      pageSection: 'Эта страница',
      pageOn: 'Показать страницу в новом алфавите',
      pageOff: 'Вернуть исходный вид',
      pageHint: 'Только видимый текст: поля ввода, формы и код не затрагиваются.',
      autotype: 'Переводить при наборе',
      autotypeHint: 'Слово переводится после пробела или знака препинания. Сразу ⌫ — вернуть исходное.',
      autotypeAsk: 'При включении браузер запросит доступ только к этому сайту.',
      autotypeDocs: 'Google Docs рисует текст на canvas — перевод при наборе там не работает. Переведите текст в этом окне и вставьте.',
      restricted: 'Браузер не пускает расширения на свои страницы. Откройте обычный сайт.',
      shortcut: 'Выделенный текст: {key}',
      shortcutNone: 'Клавиша для выделенного текста не назначена',
      shortcutEdit: 'Изменить',
      privacy: 'Работает офлайн · текст никуда не отправляется',
      langLabel: 'Язык интерфейса',
      loadFailed: 'Не удалось загрузить ядро bitg'
    },
    en: {
      langName: 'English',
      menuConvert: 'bitg: convert to the new alphabet',
      menuPage: 'bitg: view page in the new alphabet',
      toastCopied: 'Copied in the new alphabet',
      toastReplaced: 'Converted to the new alphabet · Ctrl+Z to undo',
      toastNothing: 'Nothing to convert — the text is already in the new alphabet',
      toastSelect: 'Select some text first',
      toastStale: 'The text changed — please try again',
      toastCopyFailed: 'Could not copy — use the bitg window instead',
      toastPageOn: 'Page shown in the new alphabet. Run again to restore',
      toastPageOff: 'Page restored to the original',
      toastPageNone: 'No text to convert on this page',
      tagline: 'The new Uzbek alphabet',
      inLabel: 'Old Latin or Cyrillic text',
      inPlaceholder: 'Type or paste text: o‘zbek tili, shahar, choy',
      outLabel: 'In the new alphabet',
      outEmpty: 'The result appears here',
      copy: 'Copy',
      copied: 'Copied',
      clear: 'Clear',
      converted: '{n} words converted',
      unchanged: 'No changes',
      checkAlt: 'Check {n} words',
      altTitle: 'Has two readings. The other one: {alt}',
      protectedTitle: 'Left as is: link, e-mail, English word or exception',
      pageSection: 'This page',
      pageOn: 'View page in the new alphabet',
      pageOff: 'Restore original',
      pageHint: 'Visible text only: inputs, forms and code are never touched.',
      autotype: 'Convert while typing',
      autotypeHint: 'A word is converted after a space or punctuation. Press ⌫ right away to get the original back.',
      autotypeAsk: 'When enabled, the browser asks for access to this site only.',
      autotypeDocs: 'Google Docs draws text on a canvas, so convert-while-typing does not work there. Convert text in this window and paste it.',
      restricted: 'Browsers do not allow extensions on their own pages. Open a regular website.',
      shortcut: 'Selected text: {key}',
      shortcutNone: 'No shortcut assigned for selected text',
      shortcutEdit: 'Change',
      privacy: 'Works offline · your text never leaves the browser',
      langLabel: 'Interface language',
      loadFailed: 'Could not load the bitg core'
    }
  };
  var LANGS = ['uz', 'ru', 'en'];

  function pick(lang) { return S[lang] ? lang : 'uz'; }
  function t(lang, key, vars) {
    var s = S[pick(lang)][key];
    if (s === undefined) s = S.uz[key] || key;
    if (vars) s = s.replace(/\{(\w+)\}/g, function (m, k) { return vars[k] !== undefined ? String(vars[k]) : m; });
    return s;
  }
  /** Kontent skriptga beriladigan xabarchalar to'plami. */
  function toasts(lang) {
    var out = {};
    Object.keys(S.uz).forEach(function (k) { if (k.indexOf('toast') === 0) out[k] = t(lang, k); });
    return out;
  }

  var api = { LANGS: LANGS, STRINGS: S, pick: pick, t: t, toasts: toasts };
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.bitgI18n = api;
})(typeof globalThis !== 'undefined' ? globalThis : this);
