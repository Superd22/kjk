"""Glue for the fold work: a sheet of KOTOR's crop, our paint before the folds and our paint with the folds, one row per head.

Usage: fold_compare.py <fold id> <out.jpg> <scale> <name> ...
"""
import json, sys
from pathlib import Path
from PIL import Image, ImageDraw

PAGES = Path.home() / ".claude/tools/likeness-fit/hq/paint/folds"
fold, out, scale, names = sys.argv[1], sys.argv[2], float(sys.argv[3]), sys.argv[4:]
rows = []
for n in names:
    ims = [Image.open(PAGES / fold / "tiles" / f"{n}{s}.jpg") for s in ("", "_before", "_ours")]
    ims = [i.resize((int(i.width * scale), int(i.height * scale))) for i in ims]
    rows.append((n, ims))
w, h = rows[0][1][0].size
sheet = Image.new("RGB", (3 * w, len(rows) * (h + 14)), (20, 20, 20))
d = ImageDraw.Draw(sheet)
for r, (n, ims) in enumerate(rows):
    y = r * (h + 14)
    d.text((3, y + 1), f"{n}: KOTOR | ours before | ours with folds", fill=(255, 255, 0))
    for i, im in enumerate(ims):
        sheet.paste(im, (i * w, y + 14))
sheet.save(out, quality=88)
