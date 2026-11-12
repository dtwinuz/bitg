# AUR: `bitg` va `fcitx5-bitg`

Arch Linux (va Manjaro, EndeavourOS) uchun manbadan yig'iladigan paket. Bitta `PKGBUILD` ikkita paket beradi:

| Paket | Ichida |
|---|---|
| `bitg` | IBus dvigateli (`/usr/lib/ibus/ibus-engine-bitg`), `bitg` CLI |
| `fcitx5-bitg` | Fcitx5 qo'shimchasi (`/usr/lib/fcitx5/bitg.so`) |

O'rnatish (AUR'da e'lon qilingandan keyin):

```bash
yay -S bitg            # yoki: paru -S bitg; Fcitx5 uchun: yay -S fcitx5-bitg
ibus restart           # so'ng Sozlamalar → Klaviatura → «bitg (yangi alifbo)»
```

AUR yordamchisiz:

```bash
git clone https://aur.archlinux.org/bitg.git && cd bitg && makepkg -si
```

## Sinov

Har PR'da [`aur.yml`](../../.github/workflows/aur.yml) toza `archlinux:base-devel` konteynerida
`makepkg --syncdeps --cleanbuild` (shu jumladan `check()`), `namcap`, `.SRCINFO` mosligi, `pacman -U`,
CLI natijasi, IBus komponenti yo'li, Fcitx5 `.so` belgilari va `pacman -R` ni tekshiradi.

## Yangi versiya

```bash
cd packaging/aur
# 1) pkgver=X.Y.Z, pkgrel=1
# 2) sha256sums: curl -fsSL https://github.com/dtwinuz/bitg/archive/refs/tags/vX.Y.Z.tar.gz | sha256sum
makepkg --printsrcinfo > .SRCINFO      # Arch'da; yoki aur.yml artifact'idagi SRCINFO
```

PR → `aur` CI yashil → merge, keyin AUR'ga:

```bash
git clone ssh://aur@aur.archlinux.org/bitg.git aur-bitg     # birinchi marta: bo'sh repo yaratiladi
cp packaging/aur/PKGBUILD packaging/aur/.SRCINFO aur-bitg/
cd aur-bitg && git add PKGBUILD .SRCINFO && git commit -m "bitg X.Y.Z" && git push
```

AUR akkaunti va SSH kaliti [aur.archlinux.org](https://aur.archlinux.org/register) da; kalit
`~/.ssh/config` da `Host aur.archlinux.org` uchun. Split paket bo'lgani uchun `fcitx5-bitg` alohida repo
emas — u `bitg` pkgbase ichida avtomatik chiqadi.
