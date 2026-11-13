# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

"""tools/demo/demo.ps1 kadrlaridan README uchun GIF.

    python tools/demo/make_gif.py <demo papka> docs/media/demo.gif

Kadr davomiyligi frames.tsv dagi haqiqiy vaqtdan olinadi (yozish tezligi buzilmaydi). Hamma kadr bitta umumiy
palitrada, ditheringsiz: matn tiniq qoladi va kadrlar orasida rang "sakramaydi". Bir xil ketma-ket kadrlar
bittaga qo'shiladi (Pillow), keyingi kadrlarda faqat o'zgargan to'rtburchak yoziladi.
"""

from __future__ import annotations

import argparse
import hashlib
import sys
from pathlib import Path

from PIL import Image

COLORS = 48  # oq fon, kulrang chiziqlar, matn antialiasing va bitta aksent uchun yetarli
MIN_MS = 20  # brauzerlar 20 ms dan qisqa GIF kadrini 100 ms deb ko'rsatadi


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("src", type=Path, help="demo.ps1 -Out papkasi (frames/, frames.tsv)")
    ap.add_argument("out", type=Path, help="GIF fayl")
    ap.add_argument("--hold-ms", type=int, default=2200, help="oxirgi kadr qancha turadi (aylanishdan oldin)")
    args = ap.parse_args()

    rows = []
    for line in (args.src / "frames.tsv").read_text(encoding="utf-8").splitlines():
        if line.strip():
            n, ms = line.split("\t")
            rows.append((int(n), int(ms)))
    if len(rows) < 10:
        print(f"kadr juda kam: {len(rows)}", file=sys.stderr)
        return 1

    frames = [Image.open(args.src / "frames" / f"{n:05d}.png").convert("RGB") for n, _ in rows]
    durations = [max(MIN_MS, rows[i + 1][1] - rows[i][1]) for i in range(len(rows) - 1)] + [args.hold_ms]

    # Umumiy palitra: birinchi, o'rta va oxirgi kadr yonma-yon (bo'sh oyna, yarim matn, to'liq matn).
    w, h = frames[0].size
    probe = Image.new("RGB", (w * 3, h))
    for i, f in enumerate((frames[0], frames[len(frames) // 2], frames[-1])):
        probe.paste(f, (w * i, 0))
    palette = probe.quantize(colors=COLORS, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    indexed = [f.quantize(palette=palette, dither=Image.Dither.NONE) for f in frames]

    args.out.parent.mkdir(parents=True, exist_ok=True)
    indexed[0].save(
        args.out,
        save_all=True,
        append_images=indexed[1:],
        duration=durations,
        loop=0,
        optimize=True,
        disposal=1,
    )
    data = args.out.read_bytes()
    total = sum(durations) / 1000
    print(f"{args.out}: {w}x{h}, {len(frames)} kadr, {total:.1f} s, {len(data) / 1024:.0f} KiB, "
          f"sha256 {hashlib.sha256(data).hexdigest()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
