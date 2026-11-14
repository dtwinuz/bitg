{
  lib,
  stdenv,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  glib,
  ibus,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "bitg";
  version = "0.4.0";

  src = fetchFromGitHub {
    owner = "dtwinuz";
    repo = "bitg";
    tag = "v${finalAttrs.version}";
    hash = "sha256-qy3GGGZLPbRovHUGubIOhbyOXF77HPChfd/LzMT7yoo=";
  };

  cargoHash = "sha256-NO4BNkjPj1Ad0uhMCs/+vQcpGfdscEUIx6xneEfWWm8=";

  outputs = [
    "out"
    "dev"
  ];

  strictDeps = true;

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [
    glib
    ibus
  ];

  # libbitg_ime.a (staticlib) is linked into the IBus engine here and into the Fcitx5 addon (fcitx5-bitg)
  cargoBuildFlags = [
    "-p"
    "bitg-ime"
    "-p"
    "bitg-cli"
  ];

  cargoTestFlags = [
    "-p"
    "bitg-ime"
  ];

  postBuild = ''
    make -C linux/ibus \
      VERSION=${finalAttrs.version} \
      LIB=../../target/${stdenv.hostPlatform.rust.cargoShortTarget}/release/libbitg_ime.a
  '';

  postInstall = ''
    # cargoInstallHook copies every staticlib to $out/lib; only the IME one is wanted, in $dev
    rm -rf $out/lib
    install -Dm644 target/${stdenv.hostPlatform.rust.cargoShortTarget}/release/libbitg_ime.a -t $dev/lib
    install -Dm644 linux/include/bitg_ime.h -t $dev/include

    install -Dm755 linux/ibus/ibus-engine-bitg -t $out/libexec
    install -Dm644 web/icon.svg $out/share/icons/hicolor/scalable/apps/bitg.svg
    install -Dm644 linux/ibus/bitg.xml -t $out/share/ibus/component
    substituteInPlace $out/share/ibus/component/bitg.xml \
      --replace-fail "@VERSION@" "${finalAttrs.version}" \
      --replace-fail "/usr/libexec/ibus-engine-bitg" "$out/libexec/ibus-engine-bitg" \
      --replace-fail "/usr/share/icons" "$out/share/icons"
  '';

  doInstallCheck = true;

  installCheckPhase = ''
    runHook preInstallCheck
    test "$($out/bin/bitg "O'zbekiston shaharlari")" = "Özbekiston şaharlari"
    test "$($out/bin/bitg 'Please check the schedule')" = "Please check the schedule"
    runHook postInstallCheck
  '';

  meta = {
    description = "Uzbek input method and converter for the 2026 Latin alphabet";
    longDescription = ''
      bitg converts Uzbek text to the new Latin alphabet (o' g' sh ch to
      ö ğ ş ç) as you type. English words, links and code are left alone,
      and Backspace right after a replacement restores the original word.
      This package contains the bitg command line tool and the IBus engine;
      the Fcitx5 addon is packaged as fcitx5-bitg.
    '';
    homepage = "https://bitg.uz";
    changelog = "https://github.com/dtwinuz/bitg/blob/v${finalAttrs.version}/CHANGELOG.md";
    license = with lib.licenses; [
      mit
      asl20
    ];
    maintainers = [ ];
    platforms = lib.platforms.linux;
    mainProgram = "bitg";
    # checked by the NixOS option i18n.inputMethod.ibus.engines
    isIbusEngine = true;
  };
})
