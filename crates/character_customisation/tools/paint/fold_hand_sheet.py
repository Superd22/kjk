"""Glue for the fold work: the sheet of hand-set heads. Each head has one row: KOTOR's face and ours, seen whole as at conversation
distance, then KOTOR's fold window, ours, and the fold layer alone on grey skin.

Usage: fold_hand_sheet.py <fold id> <out.jpg> <name> ...
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

PAGES = Path.home() / ".claude/tools/likeness-fit/hq/paint/folds"
fold, out, names = sys.argv[1], sys.argv[2], sys.argv[3:]
rows = []
for n in names:
    d = PAGES / fold / "hand"
    face = [Image.open(d / f"{n}_{s}_face.jpg") for s in ("kotor", "ours")]
    win = [Image.open(d / f"{n}_{s}.jpg") for s in ("kotor", "ours")] + [Image.open(d / f"{n}_field.png").convert("RGB")]
    face = [i.resize((int(i.width * 0.75), int(i.height * 0.75))) for i in face]
    win = [i.resize((int(i.width * 0.48), int(i.height * 0.48))) for i in win]
    rows.append((n, face + win))
height = max(i.height for i in rows[0][1]) + 16
width = sum(i.width for i in rows[0][1]) + 4 * 6
sheet = Image.new("RGB", (width, height * len(rows)), (20, 20, 20))
dr = ImageDraw.Draw(sheet)
for r, (n, ims) in enumerate(rows):
    x, y = 0, r * height
    dr.text((3, y + 2), f"{n}: KOTOR face | ours | KOTOR fold window | ours | our fold alone", fill=(255, 255, 0))
    for im in ims:
        sheet.paste(im, (x, y + 16))
        x += im.width + 6
sheet.save(out, quality=90)
