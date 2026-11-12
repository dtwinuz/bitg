# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

"""bitg ikonkalari yangi logodan ("Yumshoq b": to'q plitka, oq b, ko'k ö nuqtalari).

    python -I tools/make_icons.py   # crates/bitg-settings/assets/bitg.ico, bitg-off.ico (+ .png)

Manba: site/logo/png/bitg-icon-*.png (site/logo/build.py chizadi; kichik o'lchamlar qo'lda tekislangan).
O'chiq holat — rangsiz va xira (trayda farqi darhol ko'rinsin). Natija repoda saqlanadi — CI bu skriptni
ishlatmaydi.
"""

from pathlib import Path

from PIL import Image, ImageOps

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "site" / "logo" / "png"
OUT = ROOT / "crates" / "bitg-settings" / "assets"
SIZES = [16, 20, 24, 32, 40, 48, 64, 128, 256]


def frame(size: int) -> Image.Image:
    """Tayyor o'lcham bo'lsa — o'zi (piksel to'ri bo'yicha chizilgan), aks holda 1024 dan kichraytiriladi."""
    exact = SRC / f"bitg-icon-{size}.png"
    src = exact if exact.exists() else SRC / "bitg-icon-1024.png"
    im = Image.open(src).convert("RGBA")
    return im if im.size == (size, size) else im.resize((size, size), Image.LANCZOS)


def off(im: Image.Image) -> Image.Image:
    grey = ImageOps.grayscale(im.convert("RGB"))
    out = Image.merge("RGBA", (grey, grey, grey, im.getchannel("A").point(lambda a: a * 55 // 100)))
    return out


def save(name: str, f) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    imgs = [f(frame(n)) for n in SIZES]
    big = imgs[-1]
    big.save(OUT / name, sizes=[(n, n) for n in SIZES], append_images=imgs[:-1])
    big.save(OUT / name.replace(".ico", ".png"))


if __name__ == "__main__":
    save("bitg.ico", lambda im: im)
    save("bitg-off.ico", off)
    print("ok", OUT)
