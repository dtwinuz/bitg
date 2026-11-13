# Xavfsizlik

bitg kiritish usuli sifatida siz yozayotgan matnni ko'radi, shuning uchun xavfsizlik muammolarini jiddiy qabul qilamiz.

## Qo'llab-quvvatlanadigan versiyalar

| Versiya | Holat |
|---|---|
| 0.3.x | ✅ tuzatishlar chiqadi |
| < 0.3 | ❌ yangilang |

Xavfsizlik tuzatishlari faqat eng so'nggi minor versiyaga (hozir 0.3.x) chiqadi. Yangi minor versiya (masalan 0.4.0)
chiqqan kuni oldingisi qo'llab-quvvatlanmaydigan bo'ladi: tuzatish uchun yangi versiyaga o'ting. 1.0 dan oldin
eski versiyalarga alohida tuzatish (backport) chiqarilmaydi.

## Zaiflik haqida xabar berish

Ommaviy issue **ochmang**. GitHub'ning yopiq kanalidan foydalaning:
[**Report a vulnerability**](https://github.com/dtwinuz/bitg/security/advisories/new).

Xabarda: ta'sirlangan versiya va platforma, qayta chiqarish qadamlari, mumkin bo'lgan ta'sir.

## Xabar kelgandan keyin (Vulnerability response)

| Qadam | Muddat |
|---|---|
| Xabar olinganini tasdiqlash | 7 kun ichida |
| Tekshirish va xavf darajasi (CVSS) | 14 kun ichida |
| Tuzatish va reliz (yuqori/kritik) | 60 kun ichida, iloji bo'lsa tezroq |
| GitHub Security Advisory va CVE (kerak bo'lsa) | tuzatilgan reliz bilan birga |

Xabar beruvchi tuzatish ustida yopiq advisory'da ishtirok etadi. Tuzatish chiqqach, xohlasangiz, ismingiz
advisory'da va `CHANGELOG.md` da minnatdorchilik bilan qayd etiladi. Xavfsizlik asoslamasi va ma'lum kamchiliklar:
[docs/ASSURANCE.md](docs/ASSURANCE.md).

## Avtomatik tekshiruvlar topgan muammolar (SCA va SAST)

Har PR va `main` ga har push quyidagilardan o'tadi. `main` ruleset'i ular yashil bo'lmaguncha merge'ni to'sadi.

| Turi | Vosita | Nimani to'sadi |
|---|---|---|
| SCA (bog'liqliklar) | `cargo deny check advisories licenses sources` (`ci.yml`, `advisories` job; qoidalar [deny.toml](deny.toml)) | RustSec'dagi zaif, unsound yoki qarovsiz paket; ruxsat ro'yxatida yo'q litsenziya; crates.io'dan tashqari registry yoki git manba |
| SCA | Dependabot (security updates + oylik yangilash), OSV (Scorecard) | zaif versiya topilsa avtomatik PR |
| SAST (kod) | CodeQL (Rust, C/C++, Python, JavaScript, Actions), `cargo clippy -D warnings` | xavfsizlik ogohlantirishi (code scanning qoidasi) |

Tuzatish muddatlari (topilgan kundan boshlab):

| Daraja | SCA (zaif bog'liqlik) | SAST (kod zaifligi) |
|---|---|---|
| Kritik / yuqori | 14 kun, va har holda keyingi relizdan oldin | merge to'siladi; `main` da topilsa 14 kun |
| O'rta | 30 kun | 30 kun |
| Past / ma'lumot (masalan "qarovsiz") | 90 kun yoki keyingi minor reliz | 90 kun |
| Litsenziya buzilishi | merge to'siladi, paket qo'shilmaydi | — |

- **Relizdan oldin** barcha SCA topilmalari hal qilingan bo'lishi kerak: `release.yml` teg vaqtida `cargo deny` ni
  qayta yurgizadi va u o'tmasa reliz yaratilmaydi. Yagona istisno — bitg'ga ta'sir qilmasligi asoslangan topilma
  (pastga qarang).
- **Istisno** faqat ID, sabab va qayta ko'rib chiqish sharti bilan qo'shiladi: `deny.toml` / `osv-scanner.toml` da
  va [VEX.openvex.json](VEX.openvex.json) da `not_affected` bayonoti bilan. CodeQL ogohlantirishini yopish
  ("dismiss") ham sababi yozilgan izoh bilan va maintainer tomonidan bo'ladi.
- Zararli paketdan himoya: bog'liqlik faqat crates.io'dan, `Cargo.lock` bilan (`--locked`) olinadi; yangi bog'liqlik
  qo'shadigan PR'ni maintainer alohida ko'rib chiqadi ([CONTRIBUTING.md](CONTRIBUTING.md), "Bog'liqliklar").

## Ta'sir qilmaydigan zaifliklar (VEX)

Bog'liqliklarda topilgan, lekin bitg yetkazadigan fayllarga ta'sir qilmaydigan zaifliklar
[VEX.openvex.json](VEX.openvex.json) da (OpenVEX) sababi bilan yozilgan. Istisno o'chirilsa yoki yangisi qo'shilsa,
VEX hujjati o'sha PR'da yangilanadi.

## Sirlar va kalitlar

| Nima | Qayerda saqlanadi | Kim foydalanadi | Almashtirish |
|---|---|---|---|
| Reliz imzosi (SLSA provenance, SBOM) | uzoq muddatli kalit yo'q: GitHub Actions OIDC + Sigstore, har relizda yangi qisqa muddatli sertifikat | faqat `build.yml` workflow'i | kerak emas |
| CI sirlari (Telegram e'lon boti tokeni; kod-imzo sertifikati qo'shilsa u ham) | GitHub Actions secrets | faqat shu sirni ishlatadigan job (`announce.yml`, `build.yml`) | har 12 oyda, maintainer ketganda yoki sizib chiqish shubhasida darhol |
| Shaxsiy kirish tokenlari (PAT) | maintainer'ning o'zida, repoda emas | maintainer | muddati ko'pi bilan 1 yil; fine-grained, minimal huquq |
| apt ombori imzo kaliti | shifrlangan oflayn zaxira | bosh maintainer | har 12 oyda ko'rib chiqiladi; sizib chiqsa darhol yangi kalit va e'lon |

- Sirlar repoga yozilmaydi: GitHub secret scanning va push protection yoqilgan, sir bilan push rad etiladi.
- Workflow'lar sukut bo'yicha faqat `contents: read` huquqi bilan ishlaydi; sir faqat kerakli job'ga beriladi.
  Fork'dan kelgan PR'lar sirlarni ko'rmaydi (`pull_request` ishlatiladi, `pull_request_target` emas).
- Maintainer loyihadan ketsa, uning kirish huquqi olinadi va u ko'rgan barcha sirlar almashtiriladi
  ([GOVERNANCE.md](GOVERNANCE.md)).

## bitg nimani qilmaydi

- Yozilgan matnni tarmoqqa yubormaydi va saqlamaydi (faqat siz qo'shgan istisno so'zlar `exceptions.txt` ga yoziladi).
- Linux'da parol, PIN va maxfiy maydonlarda o'chiq turadi; macOS'da tizim xavfsiz kiritish paytida kiritish
  usullarini o'chiradi; Windows'da terminallarda, shuningdek maydon turi parol, PIN yoki email bo'lsa
  (`InputScope`) yoki dastur klaviatura kiritishini o'chirgan bo'lsa o'chiq turadi.
- Kiritish usuli (Windows TSF, Linux IBus/Fcitx5) klaviatura hook'i va clipboard ishlatmaydi. Clipboard faqat
  siz `Ctrl+Shift+Y` (belgilangan matnni o'girish) ni bosganda vaqtincha ishlatiladi va asl holiga qaytariladi.

## Yuklab olingan faylni tekshirish

v0.3.1 dan boshlab har bir reliz fayli (MSI, `.deb`, `.pkg`) uchun GitHub Actions imzolagan SLSA build provenance va
SBOM (SPDX) attestation'i bor. Fayl aynan shu repodagi teg'dan, GitHub Actions'da yig'ilganini tekshirish:

```bash
gh attestation verify bitg-0.3.1-x64.msi -R dtwinuz/bitg
```

Har relizda `bitg-<versiya>.spdx.json` ham bor: unda bitg ichidagi barcha kutubxonalar va ularning versiyalari.

v0.3.2 dan boshlab paketlarni alohida reusable workflow (`build.yml`) yig'adi va imzolaydi (SLSA Build L3). Faylni
aynan shu workflow, aynan shu teg uchun yig'ganini qat'iy tekshirish:

```bash
gh attestation verify bitg-0.3.2-x64.msi -R dtwinuz/bitg \
  --signer-workflow dtwinuz/bitg/.github/workflows/build.yml --source-ref refs/tags/v0.3.2 --deny-self-hosted-runners
```

v0.3.1 uchun `--signer-workflow` qiymati `dtwinuz/bitg/.github/workflows/release.yml`.

Internetsiz (GitHub API'siz) tekshirish uchun relizdagi bundle fayli:

```bash
gh attestation verify bitg-0.3.1-x64.msi -R dtwinuz/bitg --bundle bitg-0.3.1.intoto.jsonl
```

bitg.uz dagi fayllar ham aynan shu reliz fayllari: deploy skripti ularni GitHub Release'dan oladi va attestation
tekshiruvidan o'tmasa to'xtaydi.

**Imzo haqida:** Windows MSI kod-imzo sertifikati bilan imzolanmagan (SmartScreen "noma'lum nashriyotchi" deydi),
macOS `.pkg` faqat ad-hoc imzolangan (Developer ID emas, notarizatsiya yo'q). Shuning uchun faylning haqiqiyligini
yuqoridagi `gh attestation verify` va `.sha256` bilan tekshiring.

bitg kodi muntazam fuzz qilinadi (`fuzz/`, libFuzzer): ixtiyoriy matnda yadro panic bermasligi va IME tahriri
so'zdan tashqariga chiqmasligi tekshiriladi.
