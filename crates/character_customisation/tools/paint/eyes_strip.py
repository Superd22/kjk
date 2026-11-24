"""Joins the slider strip pictures into one sheet: one row per slider, its three values side by side. Glue only.

eyes_strip.py <strip dir> <head> <out.jpg> <id>=<label>:<a>,<b>,<c> ..."""
import sys
from PIL import Image, ImageDraw
d, head, out = sys.argv[1:4]
rows = []
for spec in sys.argv[4:]:
    cid, rest = spec.split("=", 1)
    label, vals = rest.rsplit(":", 1)
    ims = [Image.open(f"{d}/{head}_{cid}_{k}.png").convert("RGB").resize((336, 120)) for k in range(3)]
    r = Image.new("RGB", (260 + 3 * 340, 124), (20, 20, 20))
    dr = ImageDraw.Draw(r)
    dr.text((8, 50), label, fill=(255, 230, 60))
    for k, (im, v) in enumerate(zip(ims, vals.split(","))):
        r.paste(im, (260 + k * 340, 2))
        dr.rectangle([260 + k * 340, 2, 260 + k * 340 + 7 * len(v) + 6, 15], fill=(0, 0, 0))
        dr.text((263 + k * 340, 3), v, fill=(255, 255, 255))
    rows.append(r)
W = Image.new("RGB", (rows[0].width, sum(r.height for r in rows)), (10, 10, 10))
y = 0
for r in rows:
    W.paste(r, (0, y)); y += r.height
W.save(out, quality=88)
print(out, W.size)
