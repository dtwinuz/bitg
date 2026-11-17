# bitg.uz — yo'nalish kartasi

Subject noun:      ikki harf bitta bo'ladigan bosish (sh → ş, o' → ö)
States:            terish → so'z tugadi → birlashdi; himoya (inglizcha, havola); ⌫ bilan qaytish; tartib almashishi
Register:          product (productivity-ui-designer uslubi: tinch sirtlar, 1px chiziq, bitta aksent)
Accent:            #2f62c0 (och) / #8fb0ea (to'q) — logodagi ö nuqtalari
Type:              Inter (latin-ext: ö ğ ş ç ʼ; kirill) — bitta oila, og'irlik bilan ierarxiya
Signature:         3D klaviatura + "birlashish": ikkinchi harf birinchisiga singib, yangi harf chiqadi (WAAPI)
Second moment:     haqiqiy bitg yadrosi (Rust → WASM) bilan sinov maydoni va butun sahifani eski/yangi alifboga o'girish
Differentiators:   1) yozayotganda o'zi o'giradi, 2) inglizcha/havola/kodga tegmaydi, 3) ⌫ bilan qaytadi
Deliberately off:  to'qima raqam va sharh, brend logolari, uzun paragraf, har bo'limga fade-up, gradient blob

Matn eski imloda yozilgan (qidiruv tizimlari shuni indekslaydi, odamlar shunday qidiradi); "Yangi" tugmasi uni bitg
yadrosi (`toNew`) bilan o'giradi, tanlov eslab qolinadi. Demo va misollar `data-no-conv`; uning ichidagi UI yorlig'i
`data-conv`. Apostrof: o‘ g‘ = U+2018, tutuq = U+2019.
Demo natijalari `bitg` CLI va WASM chiqishidan olingan (README'dagi misollar bilan bir xil yadro).

## /qilingan-ishlar/ — sahifa kartasi

Subject noun:      bitta tugma bosish (ikki bosish orasidagi vaqt ichida bitg so'zni o'girib bo'ladi)
States:            100 000 µs (odam keyingi tugmani bosguncha) → kattalashtirish → 2,7 µs (bitg ishi) → 7 kunlik ish jurnali
Signature:         "Bir bosish ichida" — scroll bilan 100 ms dan 25 µs gacha logarifmik kattalashadigan chizg'ich (sticky, rAF)
Second moment:     7 kun: kunlik commit ustunlari + reliz belgilari (SVG, build vaqtida chiziladi; CSP style-src 'self')
Differentiators:   1) tezlik raqamlari bilan, 2) har ish dalil havolasi bilan, 3) holat ochiq (jonli / ko'rib chiqilmoqda / kutilmoqda)
Deliberately off:  to'qima raqam, "foydalanuvchilar soni", kartochka to'ri, har bo'limga fade-up; inline style yo'q (CSP)
Ma'lumot:          tools/site/ishlar.json (manba havolasi bilan); raqamlar git/gh/README'dan, buyrug'i JSON'da

## redesign.bitg.uz — sahifa kartasi (web/redesign/)

Subject noun:      klaviatura tugmasi va undagi ikki harfning bitta harfga birlashishi
States:            yozish → probel → birlashdi (ö ğ ş ç tugmasi yonadi); inglizcha/havola tegilmaydi; 13 joyda ishlaydi
Register:          product (anthropic.com tili: iliq qog'oz fon, serif displey, qora "slate" bo'limlar, bitta aksent)
Accent:            #2f62c0 / #8fb0ea — logodagi ö nuqtalari; token ranglari yadro sabablari (c/e/p/x) bo'yicha
Type:              Source Serif 4 (displey), Hanken Grotesk + Onest kirill (matn), JetBrains Mono (ma'lumot) — wonmor.io shriftlari
Signature:         three.js 3D klaviatura (kb3d.mjs -> kb3d.js): hero yozuvi bilan sinxron bosiladi, prujina fizikasi
Second moment:     haqiqiy yadro (bitg_core.wasm) bilan sinov maydoni — so'zlar sababi bo'yicha bo'yaladi, tezlik brauzerda o'lchanadi
Differentiators:   1) hamma joyda (kompyuter, brauzer, ofis, AI, telefon, Telegram, dasturchi), 2) tushunadi, 3) dalilli ishonch
Deliberately off:  to'qima raqam va sharh, AI-render rasm, inline style (CSP), har bo'limga fade-up (faqat sarlavhalar)
Ma'lumot:          raqamlar tools/site/ishlar.json va README'dan; hero misollari bitg_run chiqishi; skrinshotlar docs/screenshots
