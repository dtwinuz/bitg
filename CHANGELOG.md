# O'zgarishlar tarixi

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versiyalar: [SemVer](https://semver.org/).

## [Unreleased]

### Qo'shildi
- **Linux panel belgisi: istisnolar va lug'at muharriri** ([#84](https://github.com/dtwinuz/bitg/issues/84)): menyudagi
  «Istisnolar va lugʼat…» `exceptions.txt` ni tahrirlaydi (atomik yozuv, IME jonli oladi). «Panel belgisini yopish»
  nomi aniqlashtirildi: kiritish usuli ishlashda davom etadi.
- **CLI `--exceptions FAYL`** ([#39](https://github.com/dtwinuz/bitg/issues/39)): `bitg`, `explain`, `file`, `diff` da
  `exceptions.txt` formatidagi istisnolar va lug'at qatorlari.
- README: «Windows'da muammo bo'lsa» bo'limi ([#51](https://github.com/dtwinuz/bitg/issues/51)).
- C ABI uchun tashqi integratsion test `crates/bitg-core/tests/ffi.rs` ([#38](https://github.com/dtwinuz/bitg/issues/38)).

### Tuzatildi
- Brendlar asl yozuvida qoladi: Shein, Bershka, Shiseido ([#66](https://github.com/dtwinuz/bitg/issues/66)),
  Chirey, Changhe, Tchibo ([#65](https://github.com/dtwinuz/bitg/issues/65), [#70](https://github.com/dtwinuz/bitg/pull/70) — @Abdulazim1771).
- Linux indikatori `settings.ini` ni IME bilan bir xil o'qiydi: BOM, katta-kichik harfli kalit, noto'g'ri qiymat
  standartni o'zgartirmaydi ([#84](https://github.com/dtwinuz/bitg/issues/84)).
- `.deb`: indikatorning GTK/Python bog'liqliklari `Depends` dan `Recommends` ga — IBus dvigateli va CLI ularsiz ham
  o'rnatiladi ([#84](https://github.com/dtwinuz/bitg/issues/84)).

### Xavfsizlik
- Windows zaxira yo'li (SendInput, matn bermaydigan dasturlar): dastur IME'ni so'z davomida o'chirsa (parol maydoni),
  har so'z oxirida qayta tekshiriladi va so'z o'girilmaydi; maxfiy maydon belgisi kelishi bilan yig'ilgan harflar
  xotiradan tashlanadi ([#41](https://github.com/dtwinuz/bitg/issues/41)).

## [0.4.0] — 2026-11-13

Foydalanuvchi uchun qisqasi — [NEWS.md](NEWS.md) (ilovadagi «Yangiliklar» shu fayldan).

### Qo'shildi
- **Shaxsiy lug'at** ([#77](https://github.com/dtwinuz/bitg/issues/77)): `exceptions.txt` da `so'z=yozuv` qatori so'zni
  aynan shunday yozadi (`shamollash=şamollaş`), qo'shimchali shakli ham (`shamollashdan` → `şamollaşdan`), registr
  yozilgandek. Bitta harfni saqlash: `bog'cha=bog'ça`. Uch platformada bitta fayl (`bitg_core::user`). Oynada
  «Istisnolar» o'rniga «Lug'at»: har o'/g'/sh/ch — yoqib-o'chiriladigan chip, fayldan qo'shish va nusxalash.
- **Ilova ichidan xabar berish** ([#78](https://github.com/dtwinuz/bitg/issues/78)): xato, taklif yoki noto'g'ri
  o'girish, ekran parchasi yoki rasm, ixtiyoriy diagnostika. `POST bitg.uz/api/feedback` — bot ichidagi HTTP oqim
  (`bot/src/web.rs`): tekshiruv, IP/o'rnatish/umumiy chegara, adminlarga Telegram orqali, chipta raqami.
- **Yangilanish** ([#79](https://github.com/dtwinuz/bitg/issues/79)): `bitg.uz/update/stable.json`; ilova kuniga bir marta
  tekshiradi, MSI faqat GitHub Releases'dan, hajm va SHA-256 mos kelsa `msiexec /passive`. Tarmoq — WinHTTP.
- **«Yangiliklar» sahifasi** ([#80](https://github.com/dtwinuz/bitg/issues/80)): `NEWS.md` dan versiyalar lentasi,
  yangilangandan keyin bir marta ochiladi (`whats_new`, `last_seen`).
- Qoidalar: «Ehtiyotkor / Tavsiya / Maxsus» tayyor holatlari; apostrof tiklash lug'at so'zini ham tuzatishi yozildi.
- **Ubuntu yuqori paneli** ([#81](https://github.com/dtwinuz/bitg/pull/81), [#75](https://github.com/dtwinuz/bitg/issues/75)):
  `bitg-indicator` (Ayatana AppIndicator) — bitg ikonkasi va menyu: yoqish/o'chirish, avtomatik o'girish, kirillni
  o'girish (`~/.config/bitg/settings.ini` ga yoziladi, IME darhol oladi), o'chiq holat uchun alohida ikonka.
  Ilovalar menyusida «bitg» launcher, GNOME/Unity sessiyasida avtostart. `.deb` bog'liqligi: `python3-gi`,
  `gir1.2-gtk-3.0`, `gir1.2-ayatanaappindicator3-0.1`.
- **Xorijiy ismlar ro'yxati kengaydi** ([#71](https://github.com/dtwinuz/bitg/pull/71)): `foreign_names.txt` 1 394 dan
  3 419 nomga (AQSh aholi ro'yxati familiyalari va NameDatabases, o'zbek kirill korpusida uchramaydiganlari;
  `tools/foreign_names_extra.txt`). `Chesney`, `Sanchez` kabi nomlar o'girilmaydi.
- **Brendlar va texnik atamalar** ([#69](https://github.com/dtwinuz/bitg/pull/69)): `names.txt` ga 55 ta nom
  (`fintech`, `edtech`, `biotech`, `Semrush`, `Hashnode`, `Sketchfab`, `Genshin`, `Launchpad` …) — o'girilmaydi.

- **crates.io:** `bitg-core` kutubxonasi crates.io'da (`cargo add bitg-core`; 0.3.1 v0.3.1 tegidan qo'lda chiqarildi).
  Keyingi relizlarda `release.yml` dagi `crates-io` job uni Trusted Publishing (OIDC, `crates-io` environment) bilan
  chiqaradi: repoda crates.io tokeni saqlanmaydi.
- **Chocolatey paketi** (`packaging/chocolatey`, `tools/release/packaging.py` yaratadi): `choco install bitg`. MSI GitHub
  Release'dan SHA-256 bilan olinadi; `/NoAutostart`, `/NoKeyboard` parametrlari. `packaging` workflow paketni haqiqiy
  Windows'da o'rnatib, parametrni tekshirib, o'chiradi; nashr — qo'lda (`publish`, `CHOCO_API_KEY`).
- Sayt, yuklab olish: «Terminalda bitta buyruq bilan» bo'limi (winget, Chocolatey, Homebrew, apt, AUR; katalogda hali
  yo'q kanal nusxalash tugmasisiz «tez orada» deb ko'rinadi) va har buyruq hamda SHA-256 yonida «Nusxalash» tugmasi.
- CI `coverage` job'i: tarmoq (branch) qamrovi ≥ 90 % va satrlar ≥ 95 % (hozir 94.8 % va 97.2 %) (`cargo llvm-cov --branch`, nightly sanasi
  qotirilgan; fayllar jadvali job summary'da). Yadro, CLI, IME va sozlamalar uchun yangi testlar.
- **Takrorlanadigan MSI va `.pkg`:** `.deb` kabi, bir commit'dan istalgan papkada bayt-mayt bir xil o'rnatuvchi.
  MSI: `/Brepro`, yo'l prefikslari, fayl vaqtlari commit vaqtida, ProductCode/PackageCode versiya va fayllardan
  (UUIDv5). `.pkg`: yo'l prefikslari (Rust, Swift `#file`), fayl vaqtlari, xar TOC vaqtlari
  (`tools/xar_normalize.py`). `macos` va `e2e` workflow'lari ikki marta yig'ib solishtiradi.
- `CONTRIBUTING.md`: birinchi hissa uchun qadamma-qadam yo'riqnoma (fork, branch, `commit -s`, PR).
- Xavfsizlik tekshiruvi 2026-11-13 (`docs/ASSURANCE.md` 8-bo'lim): 11 topilma, kritik va yuqori yo'q.

### Tuzatildi
- Havola, kod, `@user` va brend gap kontekstida inglizcha deb ovoz bermaydi: «fetch() chaqiruvi», «Shohjahon
  McDonald's» endi o'giriladi ([#76](https://github.com/dtwinuz/bitg/issues/76)). Baholash o'zgarmadi.
- TSF: `take_retype()` sharti soddalashtirildi ([#37](https://github.com/dtwinuz/bitg/issues/37)).

- **`bitg file`: zip-bomba himoyasi** ([#27](https://github.com/dtwinuz/bitg/issues/27)). O'giriladigan har XML
  qism ochilganda 256 MiB dan oshsa, o'girish xato bilan to'xtaydi (xotira to'lib qolmaydi). O'girilmaydigan
  qismlar avvalgidek ochilmasdan ko'chiriladi.

### Xavfsizlik
- **SLSA Build L3:** reliz paketlarini reusable `build.yml` yig'adi va imzolaydi. Kod kompilyatsiya qiladigan
  job'larda imzo tokeni (`id-token`) yo'q, attestation'ni alohida `attest` job yozadi. `release.yml` relizni faqat
  har paket uchun `gh attestation verify --signer-workflow build.yml --source-ref <teg>` o'tgandan keyin ochadi.
  Qo'lda ishga tushirilsa (`workflow_dispatch`) reliz zanjiri relizsiz sinaladi. bitg.uz deploy'i yangi imzolovchini
  talab qiladi (v0.3.1 uchun eskisi).
- `SECURITY.md`: qo'llab-quvvatlash muddati aniq yozildi (faqat eng so'nggi minor versiya).
- OpenSSF Baseline: har PR'da DCO (`Signed-off-by`) tekshiruvi (`dco.yml`); `cargo deny` endi crates.io'dan
  tashqari manbani ham rad etadi va relizdan oldin (`release.yml`) qayta yuradi. `SECURITY.md` ga SCA/SAST tuzatish
  muddatlari, VEX (`VEX.openvex.json`) va sirlar siyosati; `GOVERNANCE.md` ga kirish huquqi berish tartibi va loyiha
  repolari ro'yxati; `CONTRIBUTING.md` ga bog'liqlik siyosati. `.github/security-insights.yml`, `CITATION.cff`,
  `.zenodo.json` qo'shildi.

- Reliz job'i: token `.git/config`da qolmaydi, imzo kaliti faqat kompilyatsiya qilmaydigan qadamda, teg nomi
  skriptga `env` orqali beriladi.
- `input.dll` faqat System32 dan yuklanadi; zaxira yo'lda so'z buferi 40 belgidan oshmaydi.
- C ABI (`bitg-core`) va TSF COM eksportlarida panik ushlanadi: ilova yopilib qolmaydi ([#44](https://github.com/dtwinuz/bitg/issues/44)).
- `bitg file`: vaqtinchalik fayl oldindan bilib bo'lmaydigan nom bilan, faqat yangi fayl sifatida ochiladi, fayl
  ruxsatlari saqlanadi; Linux/macOS'da `exceptions.txt` atomik va faqat egasi o'qiydigan (0600) yoziladi ([#43](https://github.com/dtwinuz/bitg/issues/43)).
- bitg.uz deploy: reliz fayllari GitHub Release'dan olinadi va `gh attestation verify` (release.yml, teg) dan o'tmasa deploy to'xtaydi; SECURITY.md: MSI imzosiz, `.pkg` ad-hoc ([#42](https://github.com/dtwinuz/bitg/issues/42)).
- Windows: standart istisnolar ro'yxatiga kirish/UAC oynalari va parol menejerlari (KeePass, KeePassXC, 1Password, Bitwarden, Enpass) qo'shildi ([#41](https://github.com/dtwinuz/bitg/issues/41), qisman).
- Windows: sozlamalar va log papkasi `SHGetKnownFolderPath` dan olinadi, bitg yuklangan ilovaning muhitidagi `%APPDATA%` dan emas ([#45](https://github.com/dtwinuz/bitg/issues/45)).
- IBus va Fcitx5: kursor oldidagi matn faqat kerak bo'lganda va oxirgi 128 belgi olinadi (macOS kabi) ([#46](https://github.com/dtwinuz/bitg/issues/46)).
- CI: `cargo deny check advisories` (RustSec), Python paketlari xesh bilan qotirilgan ([#47](https://github.com/dtwinuz/bitg/issues/47)).

## [0.3.1] — 2026-11-13

Asosiysi: Windows'da bitg o'z-o'zidan inglizcha klaviaturaga o'tib ketmaydi ([#13](https://github.com/dtwinuz/bitg/issues/13)),
parol, PIN va email maydonlariga tegmaydi ([#26](https://github.com/dtwinuz/bitg/issues/26)),
har reliz fayli imzolangan kelib chiqish (provenance) bilan tekshiriladi.

### Qo'shildi
- **Takrorlanadigan `.deb`:** bir commit'dan istalgan papkada bayt-mayt bir xil paket (`SOURCE_DATE_EPOCH`, yo'l
  prefikslari); `linux` workflow har PR'da ikki marta yig'ib solishtiradi.
- **Reliz fayllarini tekshirish:** har MSI, `.deb` va `.pkg` uchun SLSA build provenance va SBOM attestation'i (`gh attestation verify <fayl> -R dtwinuz/bitg`), relizga `bitg-<versiya>.spdx.json` qo'shiladi.
- OpenSSF Scorecard (haftalik, README belgisi), `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1).
- **Fuzzing:** `fuzz/` (cargo-fuzz, libFuzzer) — `convert` va `session` target'lari, boshlang'ich korpus `eval/` dan; `fuzz` workflow haftalik va yadro o'zgargan PR'da.
- **Loyiha hujjatlari:** `GOVERNANCE.md` (ikki maintainer, qarorlar, davomiylik), `ROADMAP.md`, `ARCHITECTURE.md`,
  `docs/ASSURANCE.md` (xavfsizlik talablari, tahdid modeli, ma'lum kamchiliklar); `SECURITY.md` ga javob muddatlari;
  `CONTRIBUTING.md` ga kod standarti, review va DCO. README'da OpenSSF Best Practices belgisi.
- CI: test qamrovi (`cargo llvm-cov`, satrlar ≥ 90 %, hisobot job summary'da).
- **REUSE 3.3:** har kod faylida SPDX litsenziya va copyright sarlavhasi, qolgan fayllar `REUSE.toml` da, litsenziya
  matnlari `LICENSES/` da (Inter — OFL-1.1, SCOWL so'zlari — `LicenseRef-SCOWL`, o'zbekcha chastotalar — CC-BY-4.0);
  CI `reuse lint` bilan tekshiradi.
- Reliz attestation bundle'lari (`bitg-<versiya>.intoto.jsonl`, `.sbom.sigstore.json`) relizga qo'shiladi.
- **Homebrew:** `brew install --cask dtwinuz/tap/bitg` ([dtwinuz/homebrew-tap](https://github.com/dtwinuz/homebrew-tap);
  CI cask'ni haqiqiy macOS'da o'rnatib, o'chirib tekshiradi, har dushanba qayta).
- **winget:** `dtwinuz.bitg` manifesti microsoft/winget-pkgs ga yuborildi. `packaging` workflow uni Windows runner'da
  o'rnatib, `bitg.exe` ni ishlatib, o'chirib tekshiradi. Manifest ProductCode va UpgradeCode bilan, shuning uchun
  `winget upgrade` MSI'dan o'rnatilgan nusxani ham taniydi.
- `tools/release/packaging.py`: ikkala havola ham GitHub Release'ga, ProductCode MSI'ning o'zidan o'qiladi.
- **@bitguzbot: `/fikr` va `/yuklash`.** `/fikr` (yoki saytdagi `t.me/bitguzbot?start=fikr`) dan keyingi xabar,
  skrinshot bilan ham, bitg jamoasiga boradi; jamoa reply qilsa javob foydalanuvchining shu chatiga qaytadi.
  `/yuklash` Windows, Mac (brew), Linux (apt) va telefon yo'llarini beradi. Botga soxta Telegram server bilan
  uchidan-uchiga sinov (`bot/test/e2e.py`) qo'shildi va CI'da yuradi.
- **Reliz e'loni:** yangi versiya chiqqanda @bitguz ga qisqa post, @bitgtech ga CHANGELOG'dagi to'liq ro'yxat
  (`tools/release/announce.py`, `announce` workflow, @bitgdevbot orqali).

- `tools/telegram/setup.py`: kanal va botlar profili kod sifatida (nom, tavsif, rasm, qadalgan post), idempotent;
  har akkauntga logotipning o'z rangli varianti. @bitguz posti bitg'ning o'zi bilan yangi alifboga o'giriladi.
- **@bitgapp** (mobil ilova kanali): "tez kunda" posti, telefonda hozir ishlaydigan yo'llar (@bitguzbot, Tilmoch),
  binafsha logotip; @bitguz tavsifidan havola.
- **@bitguzbot qayta ishga tushsa ham `/fikr` yo'qolmaydi:** kutish holati va admin suhbatlari `STATE_DIR/state.json`
  da saqlanadi. Bot har 30 soniyada tiriklik belgisini yozadi; `bitg-stats` u 5 daqiqadan eskirsa @bitgdevbot orqali
  ogohlantiradi (kuzatuv jadvalida ham ko'rinadi).
- **@bitguz rejali postlari:** `deploy/stats/posts/YYYY-MM-DD-nom.html` o'sha kuni 10:00 dan keyin bir marta chiqadi
  (`bitg-stats`, @bitgdevbot). Birinchi seriya — sayt sahifalaridagi manbali faktlardan 8 ta post (alifbo, qonun
  sanalari, hujjatlar, kompyuter va telefonda yozish, tutuq belgisi, maktab, inglizcha so'zlar).

### O'zgardi
- Saytdagi "Telegram'da yozing" havolalari kanal (@bitguz, u yerga yozib bo'lmaydi) o'rniga @bitguzbot `/fikr` ga
  olib boradi. @bitguz faqat yangiliklar uchun qoldi, texnik kanal @bitgtech "Biz haqimizda" va `llms.txt` da.
- Statistika xulosasi va uptime ogohlantirishlari ommaviy bot emas, ishchi @bitgdevbot orqali boradi
  (`set-stats-env.sh` tokenni so'raydi; bir nechta admin chat — vergul bilan).

### Tuzatildi
- **Windows maxfiy maydonlar ([#26](https://github.com/dtwinuz/bitg/issues/26)):** bitg maydon turini (`InputScope`) o'zi o'qiydi va parol, PIN, email maydonlarida, shuningdek dastur klaviatura kiritishini o'chirganda so'zlarni o'zgartirmaydi.
- Docker image'lari digest bilan, `rustup-init` versiya va SHA-256 bilan qotirildi; Dependabot Docker'ni ham kuzatadi. `osv-scanner.toml`: Linux'da hech qachon yig'ilmaydigan glib zanjiri uchun asoslangan istisno.
- `bot/Cargo.lock` bitg-core 0.3.0 ga yangilandi: eskisi bilan bot Docker'da (`--locked`) yig'ilmas edi.
- `SECURITY.md`: qo'llab-quvvatlanadigan versiya 0.3.x; `release.yml` umumiy ruxsati `contents: read` ga tushirildi.
- **Windows: bitg ishga tushishda xato bersa ham tanlangan klaviatura o'zgarmaydi** ([#13](https://github.com/dtwinuz/bitg/issues/13)).
  Avval `ActivateEx` ichidagi xato yoki panic TSF'ga qaytardi, shunda Windows o'z-o'zidan oldingi klaviaturaga
  o'tib ketardi. Endi xato `tsf.log` ga `activate failed hr=…` deb yoziladi va bitg tanlangan holda qoladi.
- Sayt: CodeQL ogohlantirishlari tuzatildi (`web/app.js` — `innerHTML` DOM atributidan olinmaydi; `tools/site/og.cjs` —
  teglar to'liq olinadi, `&amp;` ikki marta ochilmaydi).

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

[0.3.1]: https://github.com/dtwinuz/bitg/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/dtwinuz/bitg/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/dtwinuz/bitg/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/dtwinuz/bitg/releases/tag/v0.1.0
