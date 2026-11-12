# Xavfsizlik

bitg kiritish usuli sifatida siz yozayotgan matnni ko'radi, shuning uchun xavfsizlik muammolarini jiddiy qabul qilamiz.

## Qo'llab-quvvatlanadigan versiyalar

| Versiya | Holat |
|---|---|
| 0.2.x | ✅ tuzatishlar chiqadi |
| < 0.2 | ❌ yangilang |

## Zaiflik haqida xabar berish

Ommaviy issue **ochmang**. GitHub'ning yopiq kanalidan foydalaning:
[**Report a vulnerability**](https://github.com/dtwinuz/bitg/security/advisories/new).

Xabarda: ta'sirlangan versiya va platforma, qayta chiqarish qadamlari, mumkin bo'lgan ta'sir. Imkon qadar tez
javob beramiz; tuzatish chiqqach, xohlasangiz, ismingizni advisory'da qayd etamiz.

## bitg nimani qilmaydi

- Yozilgan matnni tarmoqqa yubormaydi va saqlamaydi (faqat siz qo'shgan istisno so'zlar `exceptions.txt` ga yoziladi).
- Parol, PIN va email maydonlarida o'chiq turadi.
- Kiritish usuli (Windows TSF, Linux IBus/Fcitx5) klaviatura hook'i va clipboard ishlatmaydi. Clipboard faqat
  siz `Ctrl+Shift+Y` (belgilangan matnni o'girish) ni bosganda vaqtincha ishlatiladi va asl holiga qaytariladi.
