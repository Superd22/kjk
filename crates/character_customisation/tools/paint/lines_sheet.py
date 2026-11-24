"""Sheets of the forehead lines. Light glue only.

lines_sheet.py windows <out.jpg> <scale> <names...>  one row per head: KOTOR's forehead, ours before, ours with lines, the layer alone
lines_sheet.py sheet <out.jpg> <names...>            one row per head: KOTOR's forehead, ours, KOTOR's face, ours
lines_sheet.py strip <out.jpg> <name>                one row per slider, three values each
"""
import sys
from PIL import Image, ImageDraw, ImageFont

ROOT = "/Users/david/.claude/tools/likeness-fit/hq/paint/lines"
try:
    FONT = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 22)
except OSError:
    FONT = ImageFont.load_default()


def label(im, text):
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, 12 * len(text) + 12, 30], fill=(0, 0, 0))
    d.text((6, 3), text, fill=(255, 255, 0), font=FONT)
    return im


def grid(rows, out, gap=6):
    w = sum(t.width for t in rows[0]) + gap * (len(rows[0]) - 1)
    h = sum(r[0].height for r in rows) + gap * (len(rows) - 1)
    sheet = Image.new("RGB", (w, h), (20, 20, 20))
    y = 0
    for r in rows:
        x = 0
        for t in r:
            sheet.paste(t, (x, y))
            x += t.width + gap
        y += r[0].height + gap
    sheet.save(out, quality=90)


mode, out = sys.argv[1], sys.argv[2]
if mode == "windows":
    scale, names = float(sys.argv[3]), sys.argv[4:]
    rows = []
    for n in names:
        tiles = []
        for suffix, text in [("kotor.jpg", "KOTOR"), ("before.jpg", "ours before"), ("ours.jpg", "ours"), ("field.png", "lines alone")]:
            im = Image.open(f"{ROOT}/hand/{n}_{suffix}").convert("RGB")
            im = im.resize((int(im.width * scale), int(im.height * scale)), Image.LANCZOS)
            tiles.append(label(im, f"{n} {text}" if suffix == "kotor.jpg" else text))
        rows.append(tiles)
    grid(rows, out)
elif mode == "sheet":
    names = sys.argv[3:]
    rows = []
    for n in names:
        tiles = []
        for suffix, text in [("kotor.jpg", f"{n}, KOTOR"), ("ours.jpg", "ours"), ("kotor_face.jpg", "KOTOR"), ("ours_face.jpg", "ours")]:
            im = Image.open(f"{ROOT}/hand/{n}_{suffix}").convert("RGB")
            if "face" not in suffix:
                im = im.resize((int(im.width * 480 / im.height), 480), Image.LANCZOS)
            tiles.append(label(im, text))
        rows.append(tiles)
    grid(rows, out)
elif mode == "strip":
    name = sys.argv[3]
    import json
    rows = []
    sliders = [("amount", ["0.15", "0.5", "1"]), ("spacing", ["0", "0.5", "1"]), ("continuity", ["0", "0.5", "1"]),
               ("waviness", ["0", "0.5", "1"]), ("variation", ["0", "1", "2"]), ("depth", ["3 L*", "8 L*", "14 L*"]),
               ("sharpness", ["0", "0.5", "1"]), ("ridge", ["0", "0.4", "0.8"]), ("dip_mm", ["0 mm", "4 mm", "8 mm"]),
               ("reach", ["0.3", "0.65", "1"])]
    for s, values in sliders:
        rows.append([label(Image.open(f"{ROOT}/strip/{s}_{i}.jpg").convert("RGB"), f"{s} {v}") for i, v in enumerate(values)])
    grid(rows, out)
