# Xavfsizlik

bitg kiritish usuli sifatida siz yozayotgan matnni ko'radi, shuning uchun xavfsizlik muammolarini jiddiy qabul qilamiz.

## Qo'llab-quvvatlanadigan versiyalar

| Versiya | Holat |
|---|---|
| 0.3.x | ✅ tuzatishlar chiqadi |
| < 0.3 | ❌ yangilang |

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

## Yuklab olingan faylni tekshirish

v0.3.1 dan boshlab har bir reliz fayli (MSI, `.deb`, `.pkg`) uchun GitHub Actions imzolagan SLSA build provenance va
SBOM (SPDX) attestation'i bor. Fayl aynan shu repodagi teg'dan, GitHub Actions'da yig'ilganini tekshirish:

```bash
gh attestation verify bitg-0.3.1-x64.msi -R dtwinuz/bitg
```

Har relizda `bitg-<versiya>.spdx.json` ham bor: unda bitg ichidagi barcha kutubxonalar va ularning versiyalari.
