# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Sifat o'lchovi uchun to'plamlar (eval/out/*.tsv, repoga kiritilmaydi).

  en.tsv  — haqiqiy inglizcha gaplar (sh/ch/apostrof bor): o'zgarmasligi kerak.
  uz.tsv  — o'zbek kirill gaplari: kirish = eski lotin (kirilldan), kutilgan = yangi lotin (kirilldan).
            Ikkalasi kirilldan mustaqil olinadi — konvertor eski lotindan yangi lotinni "taxmin" qilishi
            kerak, ya'ni bu haqiqiy test (Ishoq, chat, tutuq...). Apostrof turlari tasodifiy almashtiriladi.

  python -I tools/build_eval.py --en <sentences.txt>... --uz <sentences.txt>... [--limit 3000]

Manba: Leipzig Corpora Collection (CC BY 4.0).
"""

import argparse
import os
import random
import re

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "eval", "out")

VOW = set("аеёиоуўэюяы")
BASE = {
    "а": "a", "б": "b", "в": "v", "г": "g", "д": "d", "ж": "j", "з": "z",
    "и": "i", "й": "y", "к": "k", "л": "l", "м": "m", "н": "n", "о": "o",
    "п": "p", "р": "r", "с": "s", "т": "t", "у": "u", "ф": "f", "х": "x",
    "ы": "i", "э": "e", "ё": "yo", "ю": "yu", "я": "ya", "қ": "q",
    "ҳ": "h", "ь": "",
}
OLD = {"ч": "ch", "ш": "sh", "щ": "shch", "ў": "o'", "ғ": "g'", "ъ": "'"}
NEW = {"ч": "ç", "ш": "ş", "щ": "şç", "ў": "ö", "ғ": "ğ",
       "ъ": "ʼ"}
OLD_APOS = ["'", "ʻ", "’", "`"]


def translit(text, table, apos=None):
    out = []
    chars = list(text)
    for i, ch in enumerate(chars):
        lower = ch.lower()
        upper = ch != lower
        prev = chars[i - 1].lower() if i > 0 else ""
        word_start = not (i > 0 and chars[i - 1].isalpha())
        if lower == "е":
            m = "ye" if (word_start or prev in VOW or prev in "ъь") else "e"
        elif lower == "ц":
            m = "ts" if prev in VOW else "s"
        elif lower in table:
            m = table[lower]
        elif lower in BASE:
            m = BASE[lower]
        else:
            out.append(ch)
            continue
        # eski lotin: s+h (сҳ) tutuq bilan ajratiladi — Is'hoq
        if table is OLD and lower == "ҳ" and prev in ("с", "ч"):
            m = "'h"
        if apos and "'" in m:
            m = m.replace("'", apos)
        if upper and m:
            nxt_upper = i + 1 < len(chars) and chars[i + 1].isalpha() and chars[i + 1].isupper()
            prev_upper = i > 0 and chars[i - 1].isalpha() and chars[i - 1].isupper()
            m = m.upper() if (nxt_upper or prev_upper) else m[0].upper() + m[1:]
        out.append(m)
    return "".join(out)


def sentences(paths):
    for p in paths:
        with open(p, encoding="utf-8", errors="replace") as fh:
            for line in fh:
                parts = line.rstrip("\n").split("\t")
                if len(parts) >= 2:
                    yield parts[1].strip()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--en", nargs="*", default=[])
    ap.add_argument("--uz", nargs="*", default=[])
    ap.add_argument("--limit", type=int, default=3000)
    ap.add_argument("--seed", type=int, default=2026)
    a = ap.parse_args()
    rnd = random.Random(a.seed)
    os.makedirs(OUT, exist_ok=True)

    risky = re.compile(r"sh|ch|'|’", re.I)
    en = [s for s in sentences(a.en) if risky.search(s) and "\t" not in s]
    rnd.shuffle(en)
    with open(os.path.join(OUT, "en.tsv"), "w", encoding="utf-8", newline="\n") as o:
        o.write("# inglizcha gaplar: o'zgarmasligi kerak (Leipzig, CC BY 4.0)\n")
        for s in en[: a.limit]:
            o.write(f"en\t{s}\t\n")

    cyr = re.compile("[Ѐ-ӿ]")
    lat = re.compile("[A-Za-z]")
    uz = [s for s in sentences(a.uz) if len(cyr.findall(s)) > 20 and not lat.search(s) and "\t" not in s]
    rnd.shuffle(uz)
    with open(os.path.join(OUT, "uz.tsv"), "w", encoding="utf-8", newline="\n") as o:
        o.write("# o'zbekcha: kirish = kirilldan eski lotin, kutilgan = kirilldan yangi lotin\n")
        for s in uz[: a.limit]:
            old = translit(s, OLD, rnd.choice(OLD_APOS))
            new = translit(s, NEW)
            o.write(f"uz\t{old}\t{new}\n")
    # Shoshib yozilgan matn: o'/g' dagi apostrof tushib qolgan ("ozbek", "togri") — tiklanishi kerak.
    noapos = dict(OLD, **{"ў": "o", "ғ": "g"})
    with open(os.path.join(OUT, "uz_noapos.tsv"), "w", encoding="utf-8", newline="\n") as o:
        o.write("# o'zbekcha, apostrofsiz: kirish = o'/g' apostrofsiz eski lotin, kutilgan = yangi lotin\n")
        for s in uz[a.limit: 2 * a.limit]:
            o.write(f"uz-noapos\t{translit(s, noapos, rnd.choice(OLD_APOS))}\t{translit(s, NEW)}\n")
    print(f"en={min(len(en), a.limit)} uz={min(len(uz), a.limit)}")


if __name__ == "__main__":
    main()
