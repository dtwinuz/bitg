# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

# bitg: IBus dvigateli + CLI + panel indikatori (bitg) va Fcitx5 qo'shimchasi (fcitx5-bitg).
# Tarmoqsiz yig'iladi: Source1 = `cargo vendor --locked` (packaging/rpm/make-srpm.sh yasaydi).
# Fayllar tarkibi linux/build-deb.sh dagi .deb bilan bir xil.

# Rust staticlib (libbitg_ime.a, o'zi Rust LTO bilan) C/C++ qobiqqa bog'lanadi: GCC -flto bilan aralashtirmaymiz
%global _lto_cflags %{nil}

Name:           bitg
Version:        0.4.0
Release:        1%{?dist}
Summary:        Uzbek input method for the new 2026 Latin alphabet

# bitg kodi: MIT OR Apache-2.0. Binarga kiritilgan lug'atlar: uz.tsv / sh_split.tsv (CC-BY-4.0),
# en.tsv / foreign_names.txt (SCOWL, LicenseRef-SCOWL) — REUSE.toml.
# /usr/bin/bitg ga statik bog'langan crate'lar (cargo tree -p bitg-cli -e normal --target x86_64-unknown-linux-gnu,
# proc-macro'lar — syn, quote, proc-macro2, unicode-ident, zeroize_derive — faqat yig'ishda, binarga kirmaydi):
#   cfg-if, crc32fast, equivalent, flate2, getrandom, hashbrown, indexmap, libc, typed-path, zeroize: MIT OR Apache-2.0;
#   constant_time_eq: CC0-1.0 OR MIT-0 OR Apache-2.0; memchr: Unlicense OR MIT; quick-xml, zip: MIT; zlib-rs: Zlib.
# bitg-ime (IBus/Fcitx5 ichidagi staticlib) bitg-core'dan boshqa crate ishlatmaydi.
License:        (MIT OR Apache-2.0) AND CC-BY-4.0 AND LicenseRef-SCOWL AND MIT AND Zlib AND (Unlicense OR MIT) AND (CC0-1.0 OR MIT-0 OR Apache-2.0)
URL:            https://bitg.uz
Source0:        https://github.com/dtwinuz/bitg/archive/refs/tags/v%{version}.tar.gz#/%{name}-%{version}.tar.gz
# make-srpm.sh: Source0 ichida `cargo vendor --locked vendor`, keyin tar.xz
Source1:        %{name}-%{version}-vendor.tar.xz

%if %{defined rust_arches}
ExclusiveArch:  %{rust_arches}
%endif

BuildRequires:  cargo >= 1.88
BuildRequires:  rust >= 1.88
BuildRequires:  gcc
BuildRequires:  gcc-c++
BuildRequires:  make
BuildRequires:  sed
BuildRequires:  pkgconfig
BuildRequires:  pkgconfig(ibus-1.0)
BuildRequires:  pkgconfig(Fcitx5Core)
BuildRequires:  pkgconfig(Fcitx5Utils)
BuildRequires:  desktop-file-utils

# /usr/share/ibus/component va hicolor papkalari egalari
Requires:       ibus >= 1.5.20
Requires:       hicolor-icon-theme
# bitg-indicator (python3 + GTK3 + Ayatana AppIndicator), autostart orqali GNOME/Unity'da ishga tushadi
Requires:       python3
%if 0%{?suse_version}
Requires:       python3-gobject-Gdk
Requires:       typelib(Gtk) = 3.0
Requires:       typelib(AyatanaAppIndicator3) = 0.1
%else
Requires:       python3-gobject
Requires:       gtk3
Requires:       libayatana-appindicator-gtk3
%endif
Suggests:       fcitx5-bitg

# vendor/ dan statik bog'langan crate'lar (Fedora "bundled" qoidasi)
Provides:       bundled(crate(cfg-if)) = 1.0.5
Provides:       bundled(crate(constant_time_eq)) = 0.6.1
Provides:       bundled(crate(crc32fast)) = 1.5.5
Provides:       bundled(crate(equivalent)) = 1.0.3
Provides:       bundled(crate(flate2)) = 1.1.11
Provides:       bundled(crate(getrandom)) = 0.4.3
Provides:       bundled(crate(hashbrown)) = 0.17.1
Provides:       bundled(crate(indexmap)) = 2.14.2
Provides:       bundled(crate(libc)) = 0.2.191
Provides:       bundled(crate(memchr)) = 2.8.3
Provides:       bundled(crate(quick-xml)) = 0.42.0
Provides:       bundled(crate(typed-path)) = 0.13.0
Provides:       bundled(crate(zeroize)) = 1.9.1
Provides:       bundled(crate(zip)) = 9.5.1
Provides:       bundled(crate(zlib-rs)) = 0.6.8

%description
bitg converts Uzbek text to the new Latin alphabet (o' g' sh ch to the new
letters) as you type, in any IBus application. English words, links and code
are left alone; Backspace right after a replacement restores the original word.
Works offline. Includes the bitg command line tool and a top-panel indicator
for GNOME/Unity (on/off, exceptions editor).

%package -n fcitx5-bitg
Summary:        Uzbek input method for the new 2026 Latin alphabet (Fcitx5)
License:        (MIT OR Apache-2.0) AND CC-BY-4.0 AND LicenseRef-SCOWL
Requires:       fcitx5
Requires:       hicolor-icon-theme

%description -n fcitx5-bitg
bitg converts Uzbek text to the new Latin alphabet (o' g' sh ch to the new
letters) as you type, in any Fcitx5 application. English words, links and code
are left alone; Backspace right after a replacement restores the original word.
Works offline.

%prep
%autosetup -n %{name}-%{version} -a 1
# crates.io o'rniga vendor/ (tarmoq yo'q); manbadagi .cargo/config.toml faqat Windows target'larini sozlaydi
cat >> .cargo/config.toml <<'EOF'

[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

%build
%{?set_build_flags}
export CARGO_HOME="$PWD/.cargo-home"
# strip va debuginfo'ni rpm o'zi qiladi (-debuginfo paketi), Cargo.toml dagi strip=true ni bosamiz
export CARGO_PROFILE_RELEASE_STRIP=false
export RUSTFLAGS="%{?build_rustflags}%{!?build_rustflags:-Copt-level=3 -Cdebuginfo=2}"
cargo build --frozen --offline --release %{?_smp_build_ncpus:-j%{_smp_build_ncpus}} -p bitg-ime -p bitg-cli
%make_build -C linux/ibus VERSION=%{version}
# Yangi Fcitx5 sarlavhalari std::span ishlatadi (C++20); Makefile'dagi -std=c++17 dan keyin kelgani yutadi
%make_build -C linux/fcitx5 CXXFLAGS="$CXXFLAGS -std=c++20"

%install
install -Dpm0755 target/release/bitg %{buildroot}%{_bindir}/bitg
install -Dpm0755 linux/ibus/ibus-engine-bitg %{buildroot}%{_libexecdir}/ibus-engine-bitg
install -d %{buildroot}%{_datadir}/ibus/component
sed -e 's/@VERSION@/%{version}/' -e 's|/usr/libexec/ibus-engine-bitg|%{_libexecdir}/ibus-engine-bitg|' \
  linux/ibus/bitg.xml > %{buildroot}%{_datadir}/ibus/component/bitg.xml
chmod 0644 %{buildroot}%{_datadir}/ibus/component/bitg.xml
install -Dpm0644 web/icon.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/bitg.svg
install -Dpm0644 linux/indicator/bitg-disabled.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/bitg-disabled.svg
install -Dpm0755 linux/indicator/bitg-indicator.py %{buildroot}%{_bindir}/bitg-indicator
install -Dpm0644 linux/indicator/bitg.desktop %{buildroot}%{_datadir}/applications/bitg.desktop
install -Dpm0644 linux/indicator/bitg-autostart.desktop %{buildroot}%{_sysconfdir}/xdg/autostart/bitg.desktop

install -Dpm0755 linux/fcitx5/bitg.so %{buildroot}%{_libdir}/fcitx5/bitg.so
install -d %{buildroot}%{_datadir}/fcitx5/addon %{buildroot}%{_datadir}/fcitx5/inputmethod
sed 's/@VERSION@/%{version}/' linux/fcitx5/bitg-addon.conf > %{buildroot}%{_datadir}/fcitx5/addon/bitg.conf
sed 's/^Icon=bitg$/Icon=fcitx-bitg/' linux/fcitx5/bitg-im.conf > %{buildroot}%{_datadir}/fcitx5/inputmethod/bitg.conf
chmod 0644 %{buildroot}%{_datadir}/fcitx5/addon/bitg.conf %{buildroot}%{_datadir}/fcitx5/inputmethod/bitg.conf
install -Dpm0644 web/icon.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/fcitx-bitg.svg

%check
export CARGO_HOME="$PWD/.cargo-home"
export CARGO_PROFILE_RELEASE_STRIP=false
export RUSTFLAGS="%{?build_rustflags}%{!?build_rustflags:-Copt-level=3 -Cdebuginfo=2}"
cargo test --frozen --offline --release %{?_smp_build_ncpus:-j%{_smp_build_ncpus}} -p bitg-ime
test "$(target/release/bitg "O'zbekiston shaharlari")" = 'Özbekiston şaharlari'
test "$(target/release/bitg 'Please check the schedule')" = 'Please check the schedule'
desktop-file-validate %{buildroot}%{_datadir}/applications/bitg.desktop
desktop-file-validate %{buildroot}%{_sysconfdir}/xdg/autostart/bitg.desktop

%files
%license LICENSE-MIT LICENSE-APACHE LICENSES/CC-BY-4.0.txt LICENSES/LicenseRef-SCOWL.txt
%doc README.md CHANGELOG.md
%{_bindir}/bitg
%{_bindir}/bitg-indicator
%{_libexecdir}/ibus-engine-bitg
%{_datadir}/ibus/component/bitg.xml
%{_datadir}/icons/hicolor/scalable/apps/bitg.svg
%{_datadir}/icons/hicolor/scalable/apps/bitg-disabled.svg
%{_datadir}/applications/bitg.desktop
%config(noreplace) %{_sysconfdir}/xdg/autostart/bitg.desktop

%files -n fcitx5-bitg
%license LICENSE-MIT LICENSE-APACHE LICENSES/CC-BY-4.0.txt LICENSES/LicenseRef-SCOWL.txt
%{_libdir}/fcitx5/bitg.so
%{_datadir}/fcitx5/addon/bitg.conf
%{_datadir}/fcitx5/inputmethod/bitg.conf
%{_datadir}/icons/hicolor/scalable/apps/fcitx-bitg.svg

%changelog
* Sat Nov 14 2026 dtwinuz <info@dtwin.uz> - 0.4.0-1
- Initial RPM package: IBus engine, CLI and panel indicator (bitg),
  Fcitx5 addon (fcitx5-bitg); offline build from vendored crates
