<!--
SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# RPM paketi (Fedora, openSUSE)

`bitg.spec` ikki paket yig'adi:

| Paket | Ichida |
|---|---|
| `bitg` | `/usr/bin/bitg` (CLI), IBus dvigateli `/usr/libexec/ibus-engine-bitg` + `/usr/share/ibus/component/bitg.xml`, panel indikatori `/usr/bin/bitg-indicator`, `bitg.desktop`, `/etc/xdg/autostart/bitg.desktop`, ikonlar |
| `fcitx5-bitg` | Fcitx5 qo'shimchasi `%{_libdir}/fcitx5/bitg.so`, `addon/` va `inputmethod/` conf, `fcitx-bitg.svg` |

Tarkib `linux/build-deb.sh` dagi `.deb` bilan bir xil.

## Manbalar

- `Source0` — GitHub teg tarball'i `v<versiya>.tar.gz` (ichida `bitg-<versiya>/`).
- `Source1` — `bitg-<versiya>-vendor.tar.xz`: `cargo vendor --locked` natijasi. `%prep` uni
  `.cargo/config.toml` orqali crates.io o'rniga ulaydi, `cargo build --frozen --offline` tarmoqsiz ishlaydi.

Ikkalasini `make-srpm.sh` yasaydi va `.src.rpm` yo'lini chiqaradi.

## Lokal yig'ish (Fedora)

```sh
sudo dnf install rpm-build cargo git-core tar xz curl 'dnf5-command(builddep)'
packaging/rpm/make-srpm.sh                 # teg tarball'i GitHub'dan
packaging/rpm/make-srpm.sh --from-git      # teg hali yo'q: `git archive HEAD`
sudo dnf builddep target/rpm/bitg-*.src.rpm
rpmbuild --rebuild target/rpm/bitg-*.src.rpm
sudo dnf install ~/rpmbuild/RPMS/$(uname -m)/{bitg,fcitx5-bitg}-[0-9]*.rpm
```

Faqat SRPM yig'ishda tarmoq kerak (tarball va crate'lar). `rpmbuild --rebuild` tarmoqsiz.

## COPR

COPR loyihasi SRPM'dan yig'adi: `make-srpm.sh` chiqargan faylni yuklang
(`copr-cli build <loyiha> target/rpm/bitg-*.src.rpm`). Build ichida tarmoq yo'q bo'lsa ham ishlaydi,
chunki hamma crate `Source1` ichida.

## Yangi versiya

1. `bitg.spec`: `Version`, `Release: 1%{?dist}`, `%changelog` ga yozuv.
2. `Cargo.lock` o'zgargan bo'lsa: `Provides: bundled(crate(...))` va `License` qatorlarini
   `cargo tree -p bitg-cli -e normal --prefix none -f '{p} {l}'` chiqishiga moslang.
3. `make-srpm.sh <versiya>` (spec'dagi versiyadan farq qilsa to'xtaydi).

## Eslatma

- `%global _lto_cflags %{nil}`: Rust staticlib (`libbitg_ime.a`) o'zi Rust LTO bilan yig'iladi,
  GCC `-flto` bilan aralashtirilmaydi (AUR'dagi `options=(!lto)` bilan bir xil).
- Fcitx5 sarlavhalari yangi distributivlarda C++20 talab qiladi: `CXXFLAGS` ga `-std=c++20` qo'shiladi.
- Lug'atlar binarga kiritilgan, shuning uchun `License` da CC-BY-4.0 va LicenseRef-SCOWL ham bor (`REUSE.toml`).
