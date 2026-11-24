"""Joins eye pictures into a wall. Glue only.

eyes_wall.py <out.jpg> <cols> <dir>:<suffix> [<dir>:<suffix> ...] -- <name>...
Each head becomes one cell; with several sources the cell holds them side by side (for example KOTOR then ours).
Names may be given as name=label to print another label."""
import sys
from PIL import Image, ImageDraw

args = sys.argv[1:]
out, cols = args[0], int(args[1])
k = args.index("--")
sources = [a.split(":") for a in args[2:k]]
names = args[k + 1:]
cells = []
for n in names:
    name, _, lab = n.partition("=")
    ims = [Image.open(f"{d}/{name}_{s}.png").convert("RGB") for d, s in sources]
    w = sum(i.width for i in ims) + 4 * (len(ims) - 1)
    c = Image.new("RGB", (w, ims[0].height), (20, 20, 20))
    x = 0
    for i in ims:
        c.paste(i, (x, 0)); x += i.width + 4
    d = ImageDraw.Draw(c)
    t = lab or name
    d.rectangle([0, 0, 7 * len(t) + 6, 13], fill=(0, 0, 0))
    d.text((3, 1), t, fill=(255, 230, 60))
    cells.append(c)
cw, ch = max(c.width for c in cells), max(c.height for c in cells)
rows = (len(cells) + cols - 1) // cols
W = Image.new("RGB", (cols * cw + (cols - 1) * 8, rows * ch + (rows - 1) * 8), (10, 10, 10))
for i, c in enumerate(cells):
    W.paste(c, ((i % cols) * (cw + 8), (i // cols) * (ch + 8)))
W.save(out, quality=88)
print(out, W.size)
