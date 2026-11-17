# bitg uchun Nix / NixOS

Bu papkada nixpkgs uslubidagi retseptlar bor:

| Fayl | Nima beradi |
|---|---|
| `package.nix` | `bitg`: CLI (`bin/bitg`), IBus dvigateli (`libexec/ibus-engine-bitg`, `share/ibus/component/bitg.xml`), `dev` chiqishida `libbitg_ime.a` + `bitg_ime.h` |
| `fcitx5-bitg.nix` | `fcitx5-bitg`: Fcitx5 qo'shimchasi (`lib/fcitx5/bitg.so`, `share/fcitx5/addon/bitg.conf`, `share/fcitx5/inputmethod/bitg.conf`) |
| `../../flake.nix` | shu repo'ning o'zidan yig'ish (`Cargo.lock` orqali, `cargoHash` kerak emas) |

`package.nix` manbani `fetchFromGitHub` bilan `v<versiya>` tegidan oladi; `flake.nix` esa shu checkout'dan.

## NixOS: IBus

```nix
{ pkgs, ... }:
{
  i18n.inputMethod = {
    enable = true;
    type = "ibus";
    ibus.engines = [ pkgs.bitg ];
  };
}
```

Keyin IBus sozlamalaridan (yoki GNOME: Sozlamalar → Klaviatura → Kiritish manbalari) "bitg (yangi alifbo)" ni qo'shing.

## NixOS: Fcitx5

```nix
{ pkgs, ... }:
{
  i18n.inputMethod = {
    enable = true;
    type = "fcitx5";
    fcitx5.addons = [ pkgs.fcitx5-bitg ];
  };
}
```

Keyin `fcitx5-configtool` da "bitg" kiritish usulini qo'shing.

## Flake orqali (nixpkgs'ga kirmasdan oldin)

```sh
# CLI ni sinab ko'rish
nix run github:dtwinuz/bitg -- "O'zbekiston shaharlari"    # Özbekiston şaharlari

# profilga o'rnatish
nix profile install github:dtwinuz/bitg
```

NixOS konfiguratsiyasida (flake):

```nix
{
  inputs.bitg.url = "github:dtwinuz/bitg";

  outputs = { nixpkgs, bitg, ... }: {
    nixosConfigurations.mypc = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./configuration.nix
        ({ pkgs, ... }: {
          i18n.inputMethod = {
            enable = true;
            type = "ibus";
            ibus.engines = [ bitg.packages.${pkgs.system}.bitg ];
            # Fcitx5 uchun: type = "fcitx5"; fcitx5.addons = [ bitg.packages.${pkgs.system}.fcitx5-bitg ];
          };
        })
      ];
    };
  };
}
```

Paketlar: `packages.<system>.default` (= `bitg`), `bitg`, `fcitx5-bitg`; tizimlar: `x86_64-linux`, `aarch64-linux`.

## Lokal sinov

```sh
nix build .#bitg .#fcitx5-bitg
./result/bin/bitg "O'zbekiston shaharlari"

# nixpkgs usuli (fetchFromGitHub + cargoHash)
nix-build -E 'with import <nixpkgs> {}; callPackage ./packaging/nix/package.nix {}'
```

## Yangi versiya chiqqanda

1. `package.nix` da `version` ni o'zgartiring.
2. `hash` va `cargoHash` ni `lib.fakeHash` ga qo'ying, `nix-build` xatosidagi haqiqiy qiymatlarni yozing
   (yoki nixpkgs ichida `nix-update bitg`).
3. `flake.lock` ni vaqti-vaqti bilan `nix flake update` bilan yangilang.

## nixpkgs'ga yuborish ro'yxati

- [ ] `maintainers/maintainer-list.nix` ga o'z yozuvingizni qo'shing (alohida commit: `maintainers: add <ism>`),
      keyin `maintainers = with lib.maintainers; [ <ism> ];`.
- [ ] Fayllar: `pkgs/by-name/bi/bitg/package.nix` (= `package.nix`) va
      `pkgs/by-name/fc/fcitx5-bitg/package.nix` (= `fcitx5-bitg.nix`).
- [ ] `nixfmt` (RFC 166 uslubi) bilan formatlang; CI `nixfmt --check` qiladi.
- [ ] `nix-build -A bitg -A fcitx5-bitg` va `nixpkgs-review pr <raqam>` o'tsin.
- [ ] Commit nomlari: `bitg: init at 0.4.0`, `fcitx5-bitg: init at 0.4.0`.
- [ ] PR shablonidagi "Things done" bandlarini belgilang (sandbox = true, x86_64-linux, kamida bitta binar ishga tushirilgan).
- [ ] Ixtiyoriy: `ibus-engines.bitg = bitg;` taxallusi (`pkgs/tools/inputmethods/ibus-engines` ro'yxatiga o'xshash) va
      `passthru.updateScript = nix-update-script { };`.
