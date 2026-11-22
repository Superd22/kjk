"""Glue for the fold work: review sheets with two heads per row, each as KOTOR | ours with folds | ours before.

Usage: fold_review.py <fold id> <out prefix> <scale> <heads per sheet>
"""
import json, sys
from pathlib import Path
from PIL import Image, ImageDraw

DATA = Path.home() / ".claude/tools/likeness-fit-data/paint/folds"
PAGES = Path.home() / ".claude/tools/likeness-fit/hq/paint/folds"
fold, prefix, scale, per = sys.argv[1], sys.argv[2], float(sys.argv[3]), int(sys.argv[4])
kotor = json.load(open(DATA / "measure_kotor.json"))["heads"]
names = sorted(kotor, key=lambda n: -sum(kotor[n][fold][s].get("score", 0) for s in (0, 1)))
for start in range(0, len(names), per):
    chunk = names[start:start + per]
    tiles = []
    for n in chunk:
        ims = [Image.open(PAGES / fold / "tiles" / f"{n}{s}.jpg") for s in ("", "_ours", "_before")]
        tiles.append((n, [i.resize((int(i.width * scale), int(i.height * scale))) for i in ims]))
    w, h = tiles[0][1][0].size
    cols = 2
    rows = (len(tiles) + 1) // 2
    sheet = Image.new("RGB", (cols * (3 * w + 8), rows * (h + 13)), (20, 20, 20))
    d = ImageDraw.Draw(sheet)
    for i, (n, ims) in enumerate(tiles):
        x0, y0 = (i % 2) * (3 * w + 8), (i // 2) * (h + 13)
        d.text((x0 + 2, y0), f"{start + i + 1}. {n}: KOTOR | ours | before", fill=(255, 255, 0))
        for j, im in enumerate(ims):
            sheet.paste(im, (x0 + j * w, y0 + 13))
    sheet.save(f"{prefix}_{start // per}.jpg", quality=85)
