# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Imzosiz xar (.pkg) faylidagi vaqtlarni SOURCE_DATE_EPOCH ga tenglaydi.

pkgbuild TOC ichiga arxiv yaratilgan vaqtni va ichki fayllarning (Bom, Payload, Scripts, PackageInfo)
ctime/mtime/atime qiymatlarini yozadi, ularni boshqarish uchun parametr yo'q. Shu sababli bir xil
manbadan ikki marta qurilgan .pkg farq qiladi. Bu skript TOC'dagi vaqtlarni almashtiradi, TOC'ni qayta
siqadi va uyum (heap) boshidagi TOC nazorat yig'indisini yangilaydi. Fayl ma'lumotlari (heap) va ularning
nazorat yig'indilari o'zgarmaydi.

Foydalanish: python3 -I tools/xar_normalize.py FILE.pkg   (SOURCE_DATE_EPOCH muhitdan)
"""

import hashlib
import os
import re
import struct
import sys
import time
import zlib

HEADER = struct.Struct(">4sHHQQI")
TIME_TAGS = ("creation-time", "ctime", "mtime", "atime")
DIGESTS = {1: ("sha1", 20), 2: ("md5", 16)}


def normalize(data: bytes, stamp: str) -> bytes:
    magic, size, version, toc_len, toc_raw_len, alg = HEADER.unpack_from(data)
    if magic != b"xar!":
        raise ValueError("xar arxiv emas")
    if alg not in DIGESTS:
        raise ValueError(f"TOC nazorat algoritmi qo'llab-quvvatlanmaydi: {alg}")
    toc = zlib.decompress(data[size:size + toc_len])
    if len(toc) != toc_raw_len:
        raise ValueError("TOC uzunligi mos emas")
    text = toc.decode("utf-8")
    if "<signature" in text:
        raise ValueError("imzolangan xar: vaqtni o'zgartirish imzoni buzadi")
    name, digest_len = DIGESTS[alg]
    m = re.search(r"<checksum style=\"[^\"]+\">\s*<offset>(\d+)</offset>\s*<size>(\d+)</size>", text)
    if not m or int(m.group(1)) != 0 or int(m.group(2)) != digest_len:
        raise ValueError("TOC nazorat yig'indisi heap boshida emas")
    for tag in TIME_TAGS:
        # pkgbuild creation-time'ni "Z"siz, fayl vaqtlarini "Z" bilan yozadi; shakl saqlanadi.
        value = stamp if tag != "creation-time" else stamp.rstrip("Z")
        text = re.sub(rf"<{tag}>[^<]*</{tag}>", f"<{tag}>{value}</{tag}>", text)
    # FinderCreateTime ichidagi <time> va <nanoseconds> ham vaqtinchalik fayl yaratilgan paytni saqlaydi.
    text = re.sub(r"<time>[^<]*</time>", "<time>1900-01-00T00:00:00</time>", text)
    text = re.sub(r"<nanoseconds>[^<]*</nanoseconds>", "<nanoseconds>0</nanoseconds>", text)
    # Arxiv ichidagi vaqtinchalik fayllar egasi va qurilmasi quruvchining mashinasiga bog'liq (o'rnatishga ta'siri
    # yo'q: o'rnatiladigan fayllar egasi Payload ichida, root:wheel).
    for tag, value in (("user", "root"), ("uid", "0"), ("group", "wheel"), ("gid", "0"), ("deviceno", "0")):
        text = re.sub(rf"<{tag}>[^<]*</{tag}>", f"<{tag}>{value}</{tag}>", text)
    # inode: qurilish mashinasidagi raqam; o'rniga tartib raqami (yagona bo'lib qoladi).
    counter = iter(range(1, 1 << 30))
    text = re.sub(r"<inode>\d+</inode>", lambda _: f"<inode>{next(counter)}</inode>", text)
    new_toc = text.encode("utf-8")
    packed = zlib.compress(new_toc, 9)
    heap = bytearray(data[size + toc_len:])
    heap[0:digest_len] = hashlib.new(name, packed).digest()
    header = HEADER.pack(magic, size, version, len(packed), len(new_toc), alg) + data[HEADER.size:size]
    return header + packed + bytes(heap)


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__.strip().splitlines()[-1], file=sys.stderr)
        return 2
    epoch = int(os.environ["SOURCE_DATE_EPOCH"])
    stamp = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(epoch))
    path = sys.argv[1]
    with open(path, "rb") as f:
        data = f.read()
    out = normalize(data, stamp)
    with open(path, "wb") as f:
        f.write(out)
    print(f"{path}: xar vaqtlari {stamp}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
