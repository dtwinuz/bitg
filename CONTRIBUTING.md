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

## Pull request

- Bitta PR — bitta mavzu; commit sarlavhasi qisqa va inglizcha yoki o'zbekcha (lotin).
- CI (`ci`, `e2e`, `linux`) yashil bo'lishi kerak. Merge faqat squash.
- Yangi xulq — test bilan.

## Litsenziya

Yuborgan hissangiz loyiha litsenziyasi (MIT yoki Apache-2.0, foydalanuvchi tanlovi) ostida tarqatiladi.
