---
sarlavha: "Alifboni yozish paytida almashtirish: bitg qanday ishlaydi"
qayerga: "Telegram (Telegraph), kun.uz / daryo.uz IT bo'limi, uzbek dev jamoalari"
holat: qoralama
---

<!-- Qoralama. Raqamlar README.md ("Sifat") va ARCHITECTURE.md dan. E'lon qilishdan oldin `bitg eval` ni qayta
     yurgizib, raqam va sanani yangilang. -->

# Alifboni yozish paytida almashtirish: bitg qanday ishlaydi

2026-yil 21-oktabrda O'RQ-1184 qonuni qabul qilindi. Lotin alifbosida to'rtta harf almashdi:
**Oʻ → Ö, Gʻ → Ğ, Sh → Ş, Ch → Ç**. Qonun 2027-yil aprelida kuchga kiradi. Klaviaturada bu harflar yo'q, ko'pchilik
esa har kuni inglizcha ham yozadi.

[bitg](https://bitg.uz) — ochiq kodli kiritish usuli. Odatdagidek `o'zbek shahar` deb yozasiz, probel bosilishi bilan
so'z `özbek şahar` bo'ladi. Word, Telegram, brauzer — hamma dasturda.

## Oddiy almashtirish nega ishlamaydi

`sh → ş` qoidasi inglizcha so'zlarni buzadi: *check*, *shell*, *push*. Shuning uchun bitg so'zni faqat uning
**o'zbekcha ekaniga dalil bo'lsa** o'zgartiradi:

- o'zbek korpusidan yig'ilgan lug'at va qo'shimchalarni ajratish (*shaharlarimizdan* ham taniladi);
- inglizcha so'zlar ro'yxati (SCOWL), brend va xorijiy nomlar — ular yozilganicha qoladi;
- so'zning shakli: havola, email, `@user`, `#teg`, `snake_case`, `push()`, `shahar123` — tegilmaydi;
- gap konteksti: `Menga chat yozing` da *chat* inglizcha, `Menga choy yozing` da *choy* → *çoy*.

Yana bir qiyin joy — apostrof. Matnlarda eski `ʻ` uchun kamida 12 xil belgi ishlatiladi (`'`, `’`, `‘`, `ʻ`, `` ` `` …).
bitg ularni bitta deb ko'radi va tirnoqdan ajratadi: `'Yaxshi tong'` — tirnoq, `tog'` — ğ, `Is'hoq` → `Ishoq`.

## Bitta yadro, har tizimga yupqa qobiq

O'girishni Rust'dagi bitta yadro qiladi. Windows'da u TSF matn xizmati: klaviaturani "ushlamaydi", matn o'zgarishini
kuzatadi va tugagan so'zni joyida almashtiradi. Telegram kabi matnini bermaydigan dasturlarda zaxira yo'l ishlaydi.
Linux'da IBus va Fcitx5, macOS'da InputMethodKit, saytda esa o'sha yadro WebAssembly ko'rinishida.

Parol, PIN, email maydonlari va terminalda bitg o'chiq turadi. O'girilgan so'zdan keyin darhol Backspace bosilsa,
asl so'z qaytadi.

## Raqamlar (2026-11-12, `bitg eval`)

| To'plam | Natija |
|---|---|
| 3000 o'zbekcha gap (eski lotin → yangi) | **98.87%** |
| Apostrofsiz yozilgan 3000 gap, butun gap to'g'ri | **81.20%** |
| Sh/ch/apostrofli 3000 inglizcha gap — o'zgarmasligi kerak | **99.73%** |
| Tezlik | so'ziga 1.4–2.7 µs, 1.1 MB xotira |

Sinov to'plamlari repoda (`eval/`), lug'at qurishda ishlatilmagan — har kim o'zi tekshira oladi.

## Ishonch

Kiritish usuli siz yozadigan har dasturga yuklanadi, shuning uchun:

- internetga ulanmaydi, logda yozgan matningiz yo'q;
- relizlar faqat GitHub Actions'da yig'iladi: SHA-256, SLSA provenance, SBOM;
- kod ochiq (MIT yoki Apache-2.0), fuzzing va OpenSSF tekshiruvlaridan o'tadi.

## O'rnatish

- **Windows 10/11:** [bitg.uz/yuklab-olish](https://bitg.uz/yuklab-olish/)
- **Ubuntu/Debian:** `.deb` yoki apt ombori; **Arch:** `packaging/aur`
- **macOS (sinov):** `brew install --cask dtwinuz/tap/bitg`
- **Brauzerda sinash:** [bitg.uz](https://bitg.uz)

Noto'g'ri o'girilgan so'z ko'rsangiz — [issue](https://github.com/dtwinuz/bitg/issues/new/choose) yoki
[bitg.uz/fikr](https://bitg.uz/fikr/): so'zni va u turgan gapni yozing.
