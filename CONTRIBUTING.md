# Hissa qo'shish

Rahmat! bitg — ochiq loyiha, xato haqida xabar, lug'atga so'z yoki kod bilan yordam berishingiz mumkin.

## Xato yoki noto'g'ri o'girish

[Issue oching](https://github.com/dtwinuz/bitg/issues/new/choose) va quyidagilarni yozing:

- nima yozdingiz va nima chiqdi (matn bilan, skrinshot ixtiyoriy);
- qaysi dastur (Word, Chrome, Telegram…) va tizim (Windows 11, Ubuntu 24.04…);
- bitg versiyasi.

Noto'g'ri o'girilgan so'zni `bitg explain "<so'z>"` bilan tekshirsangiz, natijani ham qo'shing.

## Kod

Talablar: Rust (stable, `rust-version` — `Cargo.toml` da). Windows qismi uchun MSVC, Linux qismi uchun
`libibus-1.0-dev` va (ixtiyoriy) `libfcitx5core-dev`.

```bash
cargo test --workspace --locked          # barcha testlar
cargo clippy --workspace --all-targets   # ogohlantirishsiz bo'lishi kerak
cargo fmt --all
cargo run -p bitg-cli -- eval eval/uz.tsv   # aniqlik o'lchovi (core'ni o'zgartirsangiz)
```

- `bitg-core` ga o'zgarish: `eval/uz.tsv` va `eval/en.tsv` natijasi yomonlashmasligi kerak — PR'da oldin/keyin raqamini yozing.
- Linux: `linux/build-deb.sh`, sinov — `dbus-run-session -- python3 linux/test/ibus_e2e.py`.
- Windows: `pwsh installer/build.ps1` (MSI).

## Kod standarti

| Til | Standart | Tekshiruv (CI) |
|---|---|---|
| Rust | [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/), `rustfmt` standart uslubi | `cargo fmt --check`, `cargo clippy -- -D warnings` |
| Python | PEP 8, faqat standart kutubxona, `python3 -I` bilan ishga tushadi | `py_compile`, CodeQL |
| C / C++ (Linux qobiqlari) | GNOME/IBus va Fcitx5 uslubi, ogohlantirishsiz | `-Wall -Wextra` (`linux/*/Makefile`), CodeQL |
| JavaScript (sayt) | ES2020, framework'siz, `innerHTML` ga tashqi matn yo'q | CodeQL |
| Workflow'lar | har action commit SHA bilan, `permissions` minimal | `actionlint`, Scorecard |

Xavfsiz kod: [docs/ASSURANCE.md](docs/ASSURANCE.md) dagi talablar (R1–R6) — masalan, yozilgan matn logga
yozilmaydi, COM/FFI chegarasidan panic chiqmaydi.

## Pull request

- Bitta PR — bitta mavzu; commit sarlavhasi qisqa va inglizcha yoki o'zbekcha (lotin).
- CI (`ci`, `e2e`, `linux`) yashil bo'lishi kerak. Merge faqat squash.
- Yangi xulq — test bilan; tuzatilgan xato — uni qaytaradigan regression test bilan.
- Test qamrovi (Linux'da kompilyatsiya bo'ladigan kod): satrlar ≥ 95 %, tarmoqlar (branch) ≥ 90 % — CI `coverage`
  job'i tekshiradi (`tools/coverage_gate.py`). Lokal: `cargo +nightly llvm-cov --workspace --branch`.
  Faqat Windows/macOS qobig'i (TSF COM, IMK) real ilovada `e2e` va `linux` workflow'lari bilan sinanadi.
- Har PR'ni muallifdan boshqa maintainer ko'rib chiqadi ([GOVERNANCE.md](GOVERNANCE.md)). Reviewer tekshiradi:
  testlar, `eval/` natijasi (yadro o'zgarsa), xavfsizlik talablari, hujjat (`README`, `CHANGELOG`).

### Developer Certificate of Origin (DCO)

Har commit oxirida `Signed-off-by: Ism <email>` bo'lsin (`git commit -s`). Bu bilan siz
[DCO 1.1](https://developercertificate.org/) ni tasdiqlaysiz: hissani yuborishga haqqingiz bor va u loyiha
litsenziyasi ostida tarqatiladi.

## Litsenziya

Yuborgan hissangiz loyiha litsenziyasi (MIT yoki Apache-2.0, foydalanuvchi tanlovi) ostida tarqatiladi.

Loyiha [REUSE](https://reuse.software/) talabiga mos: har kod faylining boshida SPDX sarlavha bor, qolgan fayllar
`REUSE.toml` da. Yangi kod fayliga sarlavha qo'shing (`reuse annotate --copyright "dtwinuz and bitg contributors"
--license "MIT OR Apache-2.0" <fayl>`) — CI `reuse lint` bilan tekshiradi. Litsenziya matnlari: `LICENSES/`.
