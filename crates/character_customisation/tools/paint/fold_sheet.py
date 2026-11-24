"""Glue for the fold work: composes labelled picture sheets from the tiles that the Rust fold tools write.

Usage: fold_sheet.py <fold id> <out.jpg> <tile suffix> <scale> <columns> [<name> ...]
With no names, every head is shown, sorted by the measure (the strongest fold first).
The tile suffix is "" for the plain crops, "_track" for the crops with the found fold drawn on them.
"""
import json, sys
from pathlib import Path
from PIL import Image, ImageDraw

DATA = Path.home() / ".claude/tools/likeness-fit-data/paint/folds"
PAGES = Path.home() / ".claude/tools/likeness-fit/hq/paint/folds"


def score(side):
    return side.get("score", 0.0)


def main():
    fold, out, suffix, scale, cols = sys.argv[1], sys.argv[2], sys.argv[3], float(sys.argv[4]), int(sys.argv[5])
    heads = json.load(open(DATA / "measure_kotor.json"))["heads"]
    names = sys.argv[6:] or sorted(heads, key=lambda n: -sum(score(s) for s in heads[n][fold]))
    tiles = []
    for n in names:
        im = Image.open(PAGES / fold / "tiles" / f"{n}{suffix}.jpg")
        im = im.resize((int(im.width * scale), int(im.height * scale)))
        sides = heads[n][fold]
        label = f"{n}  " + " | ".join(
            (f"{s['depth_lstar']:.1f}/{s['edge_lstar']:.1f} {s['length_mm']:.0f}mm" if s["present"] else "none") for s in sides)
        tiles.append((im, label))
    w, h = tiles[0][0].size
    rows = (len(tiles) + cols - 1) // cols
    sheet = Image.new("RGB", (w * cols, (h + 14) * rows), (20, 20, 20))
    d = ImageDraw.Draw(sheet)
    for i, (im, label) in enumerate(tiles):
        x, y = (i % cols) * w, (i // cols) * (h + 14)
        sheet.paste(im, (x, y + 14))
        d.text((x + 3, y + 1), label, fill=(255, 255, 0))
    sheet.save(out, quality=88)


main()
