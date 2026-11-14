# Loyihani yuritish (maintainer qo'llanmasi)

Bu hujjat bitg'ni bosh maintainer'siz ham davom ettirish uchun: PR ko'rib chiqish, merge, reliz, paketlar va sayt.
Rollar va qarorlar tartibi: [GOVERNANCE.md](../GOVERNANCE.md). Kod tuzilishi: [ARCHITECTURE.md](../ARCHITECTURE.md).

**Asosiy qoida:** har kim faqat o'z GitHub akkaunti bilan ishlaydi. Boshqa odamning tokeni yoki paroli
so'ralmaydi va berilmaydi. Har bir maintainer o'z akkauntida 2FA'ni yoqib qo'yadi (ilova yoki xavfsizlik kaliti, SMS emas).

## 1. Kirish huquqi

| Nima | Qayerda | Kerakli rol |
|---|---|---|
| Repo, PR, issue, reliz | [github.com/dtwinuz/bitg](https://github.com/dtwinuz/bitg) | `dtwinuz` org owner yoki repo admin |
| `main` qoidalari (ruleset) | Settings → Rules → `main` | admin |
| Actions secrets: repo darajasida sir saqlanmaydi; environment'lar (faqat `main` va `v*` teglar): `announce` → `BITG_DEVBOT_TOKEN`, `chocolatey` → `CHOCO_API_KEY`, `winget` → `WINGET_TOKEN` (hali qo'yilmagan), `aur` → `AUR_SSH_KEY`. Kod imzosi uchun `SIGNPATH_API_TOKEN` (sir) va `SIGNPATH_ORGANIZATION_ID` (repo o'zgaruvchisi) joyi tayyor, imzo keyinga qoldirilgan ([CODE_SIGNING.md](CODE_SIGNING.md)) | Settings → Secrets and variables → Actions | admin; qiymatlarni faqat almashtirish mumkin, o'qib bo'lmaydi |
| Homebrew tap | [dtwinuz/homebrew-tap](https://github.com/dtwinuz/homebrew-tap) | org a'zosi |
| Scoop bucket | [dtwinuz/scoop-bucket](https://github.com/dtwinuz/scoop-bucket) | org a'zosi |
| Xavfsizlik xabarlari | Security → Advisories (yopiq) | admin |

## 2. Kod xaritasi

| Papka | Nima |
|---|---|
| `crates/bitg-core` | O'girish yadrosi: eski lotin/kirill → yangi lotin, inglizcha so'zlarni himoya. Barcha platformalar shuni ishlatadi |
| `crates/bitg-ime` | Platformaga bog'liq bo'lmagan kiritish usuli mantiqi va C ABI (Linux va macOS qobiqlari uchun) |
| `crates/bitg-tsf` | Windows kiritish usuli (TSF COM DLL) va sozlamalar fayli (`config.rs`) |
| `crates/bitg-settings` | Windows sozlamalar oynasi (WebView2, `ui.html`) |
| `crates/bitg-cli` | `bitg` buyruq qatori: matn, fayl (`.docx`, `.xlsx`), `eval` |
| `linux/ibus`, `linux/fcitx5` | Linux qobiqlari (C / C++), `linux/build-deb.sh` |
| `macos/` | macOS qobig'i (Swift, InputMethodKit), `macos/build.sh` |
| `installer/` | Windows MSI (WiX 5), `installer/build.ps1` |
| `eval/` | Aniqlik o'lchovi to'plamlari (`uz.tsv`, `uz_noapos.tsv`, `en.tsv`) |
| `fuzz/` | libFuzzer nishonlari |
| `web/`, `deploy/` | bitg.uz sayti va deploy skriptlari |
| `packaging/`, `tools/release/` | winget, Chocolatey, Homebrew, AUR paketlari; `channels.py` — kanallar monitori |

## 3. PR ko'rib chiqish

1. **Files changed** ni o'qing. Tushunarsiz joy bo'lsa, qatorga izoh yozing: savol ham review hisoblanadi.
2. Tekshiring:
   - CI yashil: `ci` (test, clippy, fmt, coverage, advisories), `linux` (IBus va Fcitx5 sinovi), `macos`, `fuzz`;
   - yangi xulq yoki tuzatilgan xato uchun test bor;
   - `bitg-core` o'zgargan bo'lsa, PR'da `eval` natijasi oldin/keyin yozilgan va yomonlashmagan;
   - xavfsizlik talablari ([ASSURANCE.md](ASSURANCE.md) R1–R6): yozilgan matn logga tushmaydi, FFI/COM
     chegarasidan panik chiqmaydi;
   - foydalanuvchi sezadigan o'zgarish `CHANGELOG.md` ga yozilgan;
   - commitlarda `Signed-off-by` bor (DCO).
3. **Review changes → Approve** (yoki **Request changes** va sababi).
4. **Squash and merge.** Repoda faqat shu usul yoqilgan, merge'dan keyin branch o'zi o'chadi.

Muallif o'z PR'ini o'zi tasdiqlamaydi: har o'zgarishni muallifdan boshqa odam ko'radi (GOVERNANCE.md).

## 4. Issue va xavfsizlik xabarlari

- Yangi issue'ga 14 kun ichida javob beriladi: yorliq (`bug`, `enhancement`, `good first issue`), kerak bo'lsa savol.
- Windows xatolarida avval so'raladi: `winver` natijasi, bitg versiyasi, `%LOCALAPPDATA%\bitg\tsf.log`.
- Xavfsizlik xabari ([SECURITY.md](../SECURITY.md)) 7 kun ichida tasdiqlanadi. Tuzatish yopiq advisory'da
  tayyorlanadi, reliz bilan birga e'lon qilinadi.

## 5. Reliz

Reliz to'liq GitHub Actions'da yig'iladi, shaxsiy kompyuter kerak emas.

1. PR: `Cargo.toml` dagi `version` va `CHANGELOG.md` (`[Unreleased]` → `[0.4.1] - YYYY-MM-DD`). Versiya yozilgan hamma joy `Cargo.toml` bilan bir xilligini tekshiring:
   `python tools/release/check_versions.py`. Review, merge.
2. Teg:
   ```bash
   git switch main && git pull
   git tag v0.4.1
   git push origin v0.4.1
   ```
3. `release` workflow'i o'zi bajaradi:
   - teg va `Cargo.toml` versiyasi bir xilligini tekshiradi;
   - `build.yml` orqali `.deb` (IBus va Fcitx5), Arch `.pkg.tar.zst` (`packaging/aur/PKGBUILD`, shu commit manbasidan),
     macOS `.pkg` va Windows MSI ni yig'adi;
   - SLSA provenance va SBOM attestation'ini yaratadi (imzolovchi `build.yml`, SLSA Build L3);
   - attestation'ni `gh attestation verify --signer-workflow` bilan tekshiradi, o'tmasa reliz ochilmaydi;
   - GitHub Release'ni ochadi.

   Reliz zanjirini teg qo'ymasdan sinash: Actions → release → Run workflow (versiya = `Cargo.toml` dagi). Hammasi
   yig'iladi va tekshiriladi, lekin reliz ochilmaydi.

   Keyin `announce` workflow'i Telegram'ga e'lon yuboradi.
4. Tekshirish:
   ```bash
   gh release download v0.4.1 -R dtwinuz/bitg -D dist
   gh attestation verify dist/bitg-0.4.1-x64.msi -R dtwinuz/bitg \
     --signer-workflow dtwinuz/bitg/.github/workflows/build.yml --source-ref refs/tags/v0.4.1
   ```

Workflow yiqilsa, tegni o'chirmang: xatoni PR bilan tuzating va keyingi patch versiyani (`v0.4.2`) chiqaring.

## 6. Paketlar (relizdan keyin)

```bash
gh release download v0.4.1 -R dtwinuz/bitg -D dist --clobber
python tools/release/packaging.py      # winget, Chocolatey va Homebrew fayllarini yangilaydi
```

Natijani PR qilib yuboring: `packaging` workflow'i winget manifesti va Chocolatey paketini haqiqiy Windows'da o'rnatib sinaydi (repodagi nusxa — sinov va hujjat). Kanallar:

| Kanal | Qanday |
|---|---|
| winget | **avtomatik**: `release.yml` → `stores.yml` Komac bilan [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) ga PR ochadi (MSI attestation tekshiruvidan keyin). Token: environment `winget` → `WINGET_TOKEN` (classic, faqat `public_repo`). **Hozir o'chiq:** `WINGET_TOKEN` birinchi paket (microsoft/winget-pkgs#478645, 0.3.0) merge bo'lgach qo'yiladi; shundan keyin Actions → `stores` → Run workflow ni v0.4.0 bilan qo'lda yurgizing. Qayta yuborish ham shu yo'l bilan |
| Chocolatey | **avtomatik**: `stores.yml` relizdagi MSI'dan paket yasaydi, Windows'da o'rnatib-o'chirib sinaydi va environment `chocolatey` → `CHOCO_API_KEY` bilan yuboradi (community.chocolatey.org moderatsiyasi) |
| Homebrew | **avtomatik**: `dtwinuz/homebrew-tap` dagi `autobump` (har 3 soatda) yangi relizni topadi, `.pkg` sha256 va attestation'ini tekshiradi, macOS'da o'rnatib sinab, cask'ni yangilaydi. Darhol kerak bo'lsa: tap → Actions → `autobump` → Run workflow |
| Scoop | **avtomatik**: `dtwinuz/scoop-bucket` dagi `autobump` (har 3 soatda) MSI sha256 va attestation'ini tekshiradi, Windows'da haqiqiy Scoop bilan o'rnatib-o'chirib (`bin/test.ps1`) manifestni yangilaydi. Manifest MSI'ni ochib qo'ymaydi (`#/bitg-setup.msi_`), `msiexec` bilan o'rnatadi: TSF tizimda ro'yxatdan o'tishi kerak, shuning uchun bitta UAC so'rovi bor |
| Microsoft Store | **to'xtab turibdi**: Store MSI/EXE ilovasini faqat Microsoft Trusted Root dasturidagi sertifikat bilan imzolangan bo'lsa qabul qiladi. MSI hozircha imzosiz (`.signpath/`), imzo paydo bo'lgach Partner Center'da "MSI or EXE app" sifatida ro'yxatga olinadi (havola relizdagi MSI'ga, versiya bo'yicha). MSIX (Store o'zi imzolaydi) yo'l emas: MSIX paketidan tizim kiritish usulini (TSF, HKLM COM) ro'yxatdan o'tkazish hujjatlanmagan |
| pacman (bitg.uz/arch) | **sayt deploy'ida** (keyingi relizdan: v0.4.0 da Arch paketi yo'q): `deploy.sh` relizdagi Arch paketlarini (attestation tekshiruvidan keyin) apt kaliti bilan imzolab, `tools/arch/pacman_repo.py` orqali ombor yasaydi. Foydalanuvchi: `bitg.uz/yuklab-olish/#pacman`. Format PR'da `aur` workflow'ida haqiqiy pacman bilan sinaladi (o'rnatish va buzilgan paketni rad etish) |
| AUR | **o'chiq**: AUR akkaunti ochilib, repo o'zgaruvchisi `AUR_ENABLED=true` qo'yilmaguncha `aur` job'i o'tkazib yuboriladi (yangi akkaunt ochish vaqtincha yopiq). Yoqilgach `stores.yml` PKGBUILD'ni teg manbasiga yangilaydi, Arch konteynerida `check()` bilan yig'adi, `.SRCINFO` ni yozadi va environment `aur` → `AUR_SSH_KEY` (faqat AUR uchun alohida kalit; ochiq qismi AUR akkauntida) bilan push qiladi |

Hamma kanal yetib keldimi — `channels` workflow (har kuni, `tools/release/channels.py`): har kanalning ochiq
manbasidan versiyani o'qiydi va so'nggi reliz bilan solishtiradi. Kanal o'z muddatidan (Homebrew, Scoop, crates.io —
12 soat; sayt, apt, pacman — 48 soat; Chocolatey, winget — 10 kun moderatsiya) ortda qolsa yoki manba javob bermasa,
`channels` yorlig'li bitta issue ochiladi va yangilanib turadi; hammasi joyiga tushgach o'zi yopiladi. Hali
ochilmagan kanal (birinchi paket ko'rib chiqilmoqda, AUR akkaunti yo'q) xato hisoblanmaydi. Qo'lda:
`python tools/release/channels.py` yoki Actions → `channels` → Run workflow.

## 7. bitg.uz sayti, apt va pacman omborlari

`deploy/deploy.sh <ssh-host>` saytni yig'adi va serverga uzilishsiz chiqaradi. Ishlashi uchun kerak:

- serverga SSH kirish;
- apt ombori imzo kaliti (pacman ombori ham shu kalit bilan imzolanadi);
- faqat `origin/main` holati.

Reliz fayllarini skript GitHub Release'dan oladi va attestation'dan o'tmasa to'xtaydi. Orqaga qaytarish
buyrug'ini skript xato bo'lganda o'zi chiqaradi.

Hozir server kirishi va apt kaliti faqat bosh maintainer'da (GOVERNANCE.md, "Continuity" jadvali, "pending").
Ular yo'qolsa: `deploy/` dan yangi server ko'tariladi, apt uchun yangi kalit yaratiladi
(`tools/apt/genkey.sh`). Shunda foydalanuvchilar yangi kalitni qayta import qiladi va bu README'da e'lon qilinadi.

## 8. Muntazam ishlar

| Qachon | Nima |
|---|---|
| Har oy | Dependabot PR'lari: action/kutubxona versiyasi chiqqaniga kamida 2 hafta bo'lgan bo'lsin, CI yashil bo'lsa merge |
| Har oy | Scorecard natijasi (README'dagi nishon) va [bestpractices.dev/projects/16620](https://www.bestpractices.dev/projects/16620) formasi hali to'g'rimi |
| Har reliz | `docs/ASSURANCE.md` dagi topilmalar jadvali yangilanganmi |
