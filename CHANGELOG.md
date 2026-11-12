# O'zgarishlar tarixi

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versiyalar: [SemVer](https://semver.org/).

## [Unreleased]

## [0.3.0] — 2026-11-12

### Tuzatildi
- **Ikki ma'noli so'zlar jim o'girilmaydi** ([#5](https://github.com/dtwinuz/bitg/issues/5)). Apostrofsiz yozilgan
  so'z ikki haqiqiy o'zakdan yasalgan bo'lsa (`ochmaydi` / `o'chmaydi`, `otdim` / `o'tdim`) va chastota hal qila
  olmasa, yozilganicha qoladi: `eşikni oçmaydi`, `telefonimni otdim`.
- **Xorijiy imlodagi nomlar saqlanadi** ([#6](https://github.com/dtwinuz/bitg/issues/6)): `Michelle`, `Chicago`,
  `Charlie Chaplin`, `Richard va Michelle` — `Miçelle`, `Çicago` emas. O'zbek imlosidagi nomlar avvalgidek
  o'giriladi (`Riçard Gir`, `Puşkin`, `Şohruh`). Xorijiy nom gap tilini belgilamaydi: `Chicagoda yaşaydi`.

### Qo'shildi
- **macOS 11+ (Apple M va Intel) — sinov.** InputMethodKit kiritish usuli, bitta universal `bitg-0.3.0-macos.pkg`
  (Apple imzosiz). Terminallarda o'chiq; menyudan yoqish/o'chirish, dasturda o'chirish va istisnolar.
  CI: arm64 va Intel Mac'da sinov va o'rnatish; haqiqiy IMK orqali `macos/test/e2e.swift` (qo'lda, real Mac'da).
- Saytdagi konvertor ikki ma'noli so'zni sariq belgilaydi; bosilsa boshqa o'qilishiga almashadi.
  `bitg explain` ham ko'rsatadi, yadroda — `convert_words` / `Word::alt`.
- `foreign_names.txt` (SCOWL proper names, 1 394 nom) va `tools/build_foreign_names.py`.
- **Sayt tizimni aniqlaydi** (`web/dl.js`): Mac'dan kirganga Mac `.pkg`, Linux'dan kirganga `.deb`, telefondan
  kirganga Tilmoch klaviaturasi va "havolani kompyuterga yuborish" birinchi chiqadi. iPad (o'zini Mac deb ataydi)
  telefon sifatida, Windows ARM ogohlantirish bilan. JS'siz sahifa avvalgidek ishlaydi.
- **Yuklab olgandan keyingi sahifa** `/yuklab-olish/rahmat/`: fayl shu yerdan yuklanadi, tanlangan tizim uchun
  3 qadam, SmartScreen/Gatekeeper oynasi chiqsa nima qilish. **Fikr sahifasi** `/fikr/` va Windows tray
  menyusida "Xato haqida xabar berish" (versiya GitHub shakliga o'zi yoziladi).
- **apt ombori** `https://bitg.uz/apt` (imzolangan, `sudo apt upgrade` bilan yangilanadi): `tools/apt/apt_repo.py`,
  kalit `deploy/apt/bitg.asc`. Ubuntu 24.04 va Debian 12 konteynerlarida o'rnatib sinaldi.
- **Statistika** (`deploy/stats/stats.py`, `bitg-stats` konteyneri): Caddy jurnalidan (IP /24 ga qisqartirilgan,
  30 kun) kunlik tashrif, tizim bo'yicha yuklab olish, apt faol qurilmalar, manbalar va 404; GitHub Releases soni;
  har kuni 9:00 da Telegram xulosa, dushanba — HTML hisobot; bitg.uz, www va TLS sertifikat kuzatuvi.
  Bosh sahifada ochiq hisoblagich 500 dan oshganda ko'rinadi.
- winget manifesti va Homebrew cask (`tools/release/packaging.py` -> `packaging/`), hali yuborilmagan.

### Sifat
- `eval/uz_noapos.tsv`: 81.30% → 81.20% (2439 → 2436). Uchta gap to'g'rilandi (`otdim`, `oçmaydi`, `otlikka`),
  10 ta haqiqatan ikki ma'noli so'z (`otmoqchi`, `oqdan`) endi taxmin qilinmaydi. `uz` 98.87%, `en` 99.73% —
  o'zgarmadi; tezlik o'zgarmadi.

## [0.2.0] — 2026-11-12

### Qo'shildi
- **Linux (Ubuntu 22.04+, Debian 12+).** IBus kiritish usuli `bitg_0.2.0_amd64.deb` paketida (`bitg` CLI ham shu paketda).
  Dastur kursor oldidagi matnni bersa, so'z joyida almashtiriladi. Bermasa, so'z tagi chizilgan holda (preedit)
  yoziladi. Backspace bilan qaytarish va istisno o'rganish Windows'dagidek ishlaydi. Parol, PIN va email
  maydonlarida hamda terminallarda bitg o'chiq.
- **Fcitx5 qo'shimchasi** (`fcitx5-bitg_0.2.0_amd64.deb`) — sinov bosqichida.
- `bitg-ime` crate: platformadan mustaqil yozish mantiqi va C ABI.
- `~/.config/bitg/settings.ini` va `exceptions.txt`: Windows bilan bir xil format, o'zgarish ~1 soniyada qo'llanadi.
- CI: haqiqiy `ibus-daemon` bilan uchidan-uchiga sinov (8 holat), `lintian`, paketni o'rnatish va o'chirish.
- Release: MSI bilan birga Linux paketlari va `.sha256` fayllari.

### O'zgarmadi
- Windows IME, sozlamalar ilovasi va o'rnatuvchi — v0.1.0 bilan bir xil kod.

## [0.1.0] — 2026-11-12

Birinchi ommaviy beta (Windows 10/11, x64).

### Qo'shildi
- `bitg-core`: 2026-yilgi alifbo qoidalari, inglizcha himoya (lug'at + kontekst), kirill → yangi lotin, teskari
  o'girish, brendlar ro'yxati, yozish sessiyasi, C ABI va WebAssembly.
- `bitg` CLI: `convert`, `old`, `cyr`, `explain`, `eval`, `file` (.docx, .xlsx, .txt, .md), `diff`, `--dry-run`.
- Windows TSF IME: matn o'zgarishini kuzatib so'zni joyida almashtiradi; matnni bermaydigan Qt dasturlari
  (Telegram) uchun zaxira yo'l; Backspace bilan qaytarish; klaviatura tartiblari (Tilmoch, Turkcha Q, AltGr, maxsus).
- Sozlamalar ilovasi (tray, jonli namuna) va belgilangan matnni o'girish (`Ctrl+Shift+Y`).
- MSI o'rnatuvchi (WiX 5): barcha foydalanuvchilar uchun, jim o'rnatish, `AUTOSTART` / `ADDKEYBOARD` xossalari.
- O'lchov: o'zbekcha gaplar 98,87 %, inglizcha gaplar o'zgarmay qolishi 99,73 % (har biri 3 000 gap).

[0.3.0]: https://github.com/dtwinuz/bitg/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/dtwinuz/bitg/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/dtwinuz/bitg/releases/tag/v0.1.0
