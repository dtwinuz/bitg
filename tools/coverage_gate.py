# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

"""cargo llvm-cov --branch --json --summary-only natijasini tekshiradi.

Satr va tarmoq (branch) qamrovi chegaradan past bo'lsa — exit 1. Jadval Markdown'da stdout'ga
(CI uni job summary'ga yozadi).

    python3 -I tools/coverage_gate.py cov.json --min-lines 95 --min-branches 90
"""

import argparse
import json
import sys


def pct(part):
    return 100.0 * part["covered"] / part["count"] if part["count"] else 100.0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("report")
    ap.add_argument("--min-lines", type=float, required=True)
    ap.add_argument("--min-branches", type=float, required=True)
    args = ap.parse_args()

    with open(args.report, encoding="utf-8") as f:
        data = json.load(f)["data"][0]

    rows = []
    for item in data["files"]:
        name = item["filename"].replace("\\", "/")
        name = name[name.find("crates/"):] if "crates/" in name else name
        s = item["summary"]
        rows.append((name, s["lines"], s["branches"]))
    rows.sort(key=lambda r: r[0])

    print("| File | Lines | Branches |")
    print("|---|---:|---:|")
    for name, lines, branches in rows:
        b = f"{pct(branches):.1f} % ({branches['covered']}/{branches['count']})" if branches["count"] else "n/a"
        print(f"| `{name}` | {pct(lines):.1f} % | {b} |")

    total = data["totals"]
    lines, branches = pct(total["lines"]), pct(total["branches"])
    print(f"| **Total** | **{lines:.2f} %** | **{branches:.2f} %** "
          f"({total['branches']['covered']}/{total['branches']['count']}) |")

    failed = []
    if lines < args.min_lines:
        failed.append(f"lines {lines:.2f} % < {args.min_lines} %")
    if branches < args.min_branches:
        failed.append(f"branches {branches:.2f} % < {args.min_branches} %")
    if failed:
        print("\ncoverage below the gate: " + "; ".join(failed), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
