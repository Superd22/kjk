"""Forehead crops of KOTOR bakes, to pick heads with clear forehead lines. Usage: lines_pick_sheet.py <out.jpg> <names...>"""
import sys
from PIL import Image, ImageDraw
S = "/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data/tex/v2"
out, names = sys.argv[1], sys.argv[2:]
tiles = []
for n in names:
    im = Image.open(f"{S}/{n}_ours.jpg").convert("RGB").crop((520, 280, 1332, 760)).resize((406, 240))
    ImageDraw.Draw(im).text((4, 4), n, fill=(255, 255, 0))
    tiles.append(im)
cols = 4
rows = (len(tiles) + cols - 1) // cols
sheet = Image.new("RGB", (cols * 406, rows * 240))
for i, t in enumerate(tiles):
    sheet.paste(t, ((i % cols) * 406, (i // cols) * 240))
sheet.save(out, quality=88)
