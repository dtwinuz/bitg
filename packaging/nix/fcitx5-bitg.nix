{
  lib,
  stdenv,
  bitg,
  pkg-config,
  fcitx5,
}:

stdenv.mkDerivation (finalAttrs: {
  pname = "fcitx5-bitg";
  inherit (bitg) version src;

  strictDeps = true;

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [
    bitg.dev
    fcitx5
  ];

  # Fcitx5 5.1+ headers use std::span; the later -std flag wins over the Makefile's -std=c++17
  env.CXXFLAGS = "-O2 -std=c++20";

  makeFlags = [
    "-C"
    "linux/fcitx5"
    "LIB=${bitg.dev}/lib/libbitg_ime.a"
  ];

  installPhase = ''
    runHook preInstall
    install -Dm755 linux/fcitx5/bitg.so -t $out/lib/fcitx5
    install -Dm644 linux/fcitx5/bitg-addon.conf $out/share/fcitx5/addon/bitg.conf
    substituteInPlace $out/share/fcitx5/addon/bitg.conf \
      --replace-fail "@VERSION@" "${finalAttrs.version}"
    install -Dm644 linux/fcitx5/bitg-im.conf $out/share/fcitx5/inputmethod/bitg.conf
    substituteInPlace $out/share/fcitx5/inputmethod/bitg.conf \
      --replace-fail "Icon=bitg" "Icon=fcitx-bitg"
    install -Dm644 web/icon.svg $out/share/icons/hicolor/scalable/apps/fcitx-bitg.svg
    runHook postInstall
  '';

  meta = {
    description = "Fcitx5 addon for bitg, the Uzbek input method for the 2026 Latin alphabet";
    homepage = "https://bitg.uz";
    changelog = "https://github.com/dtwinuz/bitg/blob/v${finalAttrs.version}/CHANGELOG.md";
    license = with lib.licenses; [
      mit
      asl20
    ];
    maintainers = [ ];
    platforms = lib.platforms.linux;
  };
})
