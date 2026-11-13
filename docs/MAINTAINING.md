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
| Actions secrets (`BITG_SIGN_PFX`, `CHOCO_API_KEY`, `BITG_DEVBOT_TOKEN`) | Settings → Secrets and variables → Actions | admin; qiymatlarni faqat almashtirish mumkin, o'qib bo'lmaydi |
| Homebrew tap | [dtwinuz/homebrew-tap](https://github.com/dtwinuz/homebrew-tap) | org a'zosi |
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
| `packaging/`, `tools/release/` | winget, Chocolatey, Homebrew, AUR paketlari |

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

1. PR: `Cargo.toml` dagi `version` va `CHANGELOG.md` (`[Unreleased]` → `[0.3.2] - YYYY-MM-DD`). Review, merge.
2. Teg:
   ```bash
   git switch main && git pull
   git tag v0.3.2
   git push origin v0.3.2
   ```
3. `release` workflow'i o'zi bajaradi:
   - teg va `Cargo.toml` versiyasi bir xilligini tekshiradi;
   - `.deb` (IBus va Fcitx5), macOS `.pkg` va Windows MSI ni yig'adi;
   - SLSA provenance va SBOM attestation'ini yaratadi;
   - GitHub Release'ni ochadi.

   Keyin `announce` workflow'i Telegram'ga e'lon yuboradi.
4. Tekshirish:
   ```bash
   gh release download v0.3.2 -R dtwinuz/bitg -D dist
   gh attestation verify dist/bitg-0.3.2-x64.msi -R dtwinuz/bitg
   ```

Workflow yiqilsa, tegni o'chirmang: xatoni PR bilan tuzating va keyingi patch versiyani (`v0.3.3`) chiqaring.

## 6. Paketlar (relizdan keyin)

```bash
gh release download v0.3.2 -R dtwinuz/bitg -D dist --clobber
python tools/release/packaging.py      # winget, Chocolatey va Homebrew fayllarini yangilaydi
```

Natijani PR qilib yuboring. `packaging` workflow'i paketni Windows'da o'rnatib sinaydi. Keyin:

| Kanal | Qanday |
|---|---|
| winget | `packaging/winget/manifests/d/dtwinuz/bitg/<v>/` → [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) ga PR ("Update: dtwinuz.bitg to <v>"). Oldingi versiya PR'i hali ochiq bo'lsa, kuting |
| Chocolatey | Actions → `packaging` → Run workflow, `publish = true` |
| Homebrew | `packaging/homebrew/Casks/bitg.rb` → `dtwinuz/homebrew-tap` ga PR |
| AUR | `packaging/aur/PKGBUILD` va `.SRCINFO` ni yangilang (`aur` workflow'i Arch'da yig'ib sinaydi), so'ng AUR akkauntidan yuboriladi |

## 7. bitg.uz sayti va apt ombori

`deploy/deploy.sh <ssh-host>` saytni yig'adi va serverga uzilishsiz chiqaradi. Ishlashi uchun kerak:

- serverga SSH kirish;
- apt ombori imzo kaliti;
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
