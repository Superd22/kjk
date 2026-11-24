"""Joins the pore pictures of `pore_pictures` into sheets (paint track, part C, pores and grain round). Glue only.

pores_sheet.py crops <data dir> <out.jpg> <scale> <columns> <names...>
    One row per head and region (cheek, forehead, nose). Each column is a folder under the data dir, such as kotor, before and after.
pores_sheet.py faces <data dir> <out.jpg> <columns> <names...>
    One row per head, the whole face from the front.
pores_sheet.py strip <data dir> <out.jpg> <scale> <name> <ids...>
    One row per control id, its three values from left to right, from the folder strip.
"""

import sys
from pathlib import Path

from PIL import Image, ImageDraw

GAP = 8
LABEL = 22


def open_scaled(path, scale):
    img = Image.open(path).convert("RGB")
    if scale != 1:
        img = img.resize((round(img.width * scale), round(img.height * scale)), Image.LANCZOS)
    return img


def grid(rows, captions, out):
    w = max(sum(i.width for i in r) + GAP * (len(r) + 1) for r in rows)
    h = sum(max(i.height for i in r) + LABEL + GAP for r in rows) + GAP
    sheet = Image.new("RGB", (w, h), (24, 24, 24))
    d = ImageDraw.Draw(sheet)
    y = GAP
    for r, caps in zip(rows, captions):
        x = GAP
        for img, cap in zip(r, caps):
            d.text((x, y + 4), cap, fill=(230, 230, 230))
            sheet.paste(img, (x, y + LABEL))
            x += img.width + GAP
        y += max(i.height for i in r) + LABEL + GAP
    sheet.save(out, quality=92)


def main():
    mode, data, out = sys.argv[1], Path(sys.argv[2]), sys.argv[3]
    rows, caps = [], []
    if mode == "crops":
        scale, cols, names = float(sys.argv[4]), sys.argv[5].split(","), sys.argv[6:]
        for n in names:
            for region in ("cheek", "forehead", "nose"):
                rows.append([open_scaled(data / c / f"{n}_{region}.jpg", scale) for c in cols])
                caps.append([f"{n} {region}: {c}" for c in cols])
    elif mode == "faces":
        cols, names = sys.argv[4].split(","), sys.argv[5:]
        for n in names:
            rows.append([open_scaled(data / c / f"{n}_face.jpg", 1) for c in cols])
            caps.append([f"{n}: {c}" for c in cols])
    elif mode == "strip":
        scale, name, ids = float(sys.argv[4]), sys.argv[5], sys.argv[6:]
        for i in ids:
            id_, _, values = i.partition("=")
            vals = values.split(",") if values else ["low", "middle", "high"]
            rows.append([open_scaled(data / "strip" / f"{name}_{id_}_{k}.jpg", scale) for k in range(3)])
            caps.append([f"{id_} = {v}" for v in vals])
    grid(rows, caps, out)


main()
