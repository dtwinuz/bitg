"""Xorijiy imlodagi kishi va joy nomlari (ch/sh bor, o'zbek korpusida yo'q) — `crates/bitg-core/src/data/foreign_names.txt`.

Manba: SCOWL 2020.12.07 `*-proper-names.N` (N <= 80), ruxsat beruvchi litsenziya (Kevin Atkinson), va
`tools/foreign_names_extra.txt` (AQSh aholi ro'yxati familiyalari, NameDatabases — jamoat mulki; sarlavhasiga qarang).
O'zbek lug'atida (`uz.tsv`) bor nom (Richard, Pushkin, Sasha) olinmaydi: ular o'zbek imlosida ham shunday
yoziladi va o'giriladi (`names.txt` siyosati). Bu ro'yxatdagi nom (Michelle, Rachel, Sanchez) faqat
inglizcha imloda uchraydi — "Miçelle" hech bir imloda to'g'ri emas.

  python -I tools/build_foreign_names.py --scowl <scowl-2020.12.07>
"""

import argparse
import glob
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "..", "crates", "bitg-core", "src", "data")
EXTRA = os.path.join(HERE, "foreign_names_extra.txt")
MAX_LEVEL = 80
REGIONS = ("english", "american", "british", "canadian", "australian")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--scowl", required=True)
    a = ap.parse_args()

    uz = set()
    with open(os.path.join(DATA, "uz.tsv"), encoding="utf-8") as f:
        for line in f:
            if not line.startswith("#"):
                uz.add(line.split("\t", 1)[0])

    pat = re.compile(r"(%s)-proper-names\.(\d+)$" % "|".join(REGIONS))
    names = set()
    for path in glob.glob(os.path.join(a.scowl, "final", "*-proper-names.*")):
        m = pat.match(os.path.basename(path))
        if not m or int(m.group(2)) > MAX_LEVEL:
            continue
        with open(path, encoding="latin-1") as f:
            for w in f:
                w = w.strip()
                if w.endswith("'s"):
                    w = w[:-2]
                if not w or not w.isascii() or not w.isalpha() or not w[0].isupper():
                    continue
                k = w.lower()
                if ("ch" in k or "sh" in k) and k not in uz:
                    names.add(k)
    with open(EXTRA, encoding="utf-8") as f:
        for k in f:
            k = k.strip()
            if k and not k.startswith("#") and ("ch" in k or "sh" in k) and k not in uz:
                names.add(k)

    out = os.path.join(DATA, "foreign_names.txt")
    with open(out, "w", encoding="utf-8", newline="\n") as f:
        f.write("# Xorijiy imlodagi kishi va joy nomlari (ch/sh bor, o'zbek korpusida yo'q): bosh harf bilan "
                "yozilsa o'girilmaydi.\n")
        f.write("# SCOWL 2020.12.07 *-proper-names (daraja <= %d), Copyright 2000-2018 Kevin Atkinson va boshqalar "
                "(ruxsat beruvchi litsenziya)\n" % MAX_LEVEL)
        f.write("# tools/foreign_names_extra.txt: AQSh aholi ro'yxati familiyalari (US Census Bureau) va NameDatabases "
                "(jamoat mulki)\n")
        f.write("# tools/build_foreign_names.py bilan qurilgan\n")
        for k in sorted(names):
            f.write(k + "\n")
    print(out, len(names))


if __name__ == "__main__":
    main()
