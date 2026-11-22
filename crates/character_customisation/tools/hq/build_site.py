"""Builds the multi-page Likeness HQ site into site/ from template.html, blink.b64 and gallery_pics.json."""
import base64, json, re, pathlib, time

HQ = pathlib.Path(__file__).resolve().parent
SITE = HQ / "site"
BUILD = str(int(time.time()))
tpl = (HQ / "template.html").read_text()

ZONES = [("forehead", "Forehead"), ("brows", "Brows"), ("eyes", "Around the eyes"), ("nose", "Nose"),
         ("cheeks", "Cheeks"), ("mouth", "Mouth"), ("chin", "Chin"), ("jaw", "Jaw")]
STEPS = ["B1 Variety", "B2 Reproduce", "B3 Fix", "B4 Paint, now part C", "B5 Shippable"]

# pictures
(SITE / "img").mkdir(parents=True, exist_ok=True)
(SITE / "zones").mkdir(exist_ok=True)
def save(name, uri):
    (SITE / "img" / name).write_bytes(base64.b64decode(uri.split(",", 1)[1]))
    return "../img/" + name
pics = {"blink": save("eyes_blink.jpg", (HQ / "blink.b64").read_text().strip())}
for k, v in json.load(open(HQ / "gallery_pics.json")).items():
    pics[k] = save(k[2:] + ".jpg", v)
(SITE / "img" / "mouth_talk.jpg").write_bytes((HQ / "mouth_talk.jpg").read_bytes())
pics["mouth_talk"] = "../img/mouth_talk.jpg"
def fill(s):
    return re.sub(r"\{\{(\w+)\}\}", lambda m: pics[m.group(1)], s)

# css
m = re.search(r"<style>\n(.*?)</style>", tpl, re.S)
css = m.group(1) + """
/* sub-pages and navigation */
.back { font-family: var(--mono); font-size: 14px; }
.zone-links { margin: 0; padding-left: 18px; display: grid; gap: 6px; max-width: 72ch; }
.grid td a { color: inherit; text-decoration-color: var(--accent); text-underline-offset: 3px; }
.facts { margin: 0; padding-left: 18px; display: grid; gap: 8px; max-width: 72ch; }
.zone-body { display: grid; gap: 14px; }
/* the head viewer */
.viewer { display: grid; gap: 12px; background: var(--panel); border: 1px solid var(--line); border-radius: 10px; padding: 14px; }
.v-controls { display: flex; flex-wrap: wrap; gap: 10px 16px; align-items: center; font-family: var(--display); font-size: 15px; }
.v-controls select, .v-controls button { font: 14px var(--mono); color: var(--ink); background: var(--bg); border: 1px solid var(--line); border-radius: 6px; padding: 5px 8px; max-width: 100%; }
.v-controls select { max-width: 22em; }
.v-controls button { cursor: pointer; }
.v-controls button[aria-pressed="true"] { background: var(--accent); color: var(--panel); border-color: var(--accent); }
.v-controls button:disabled { opacity: 0.45; cursor: default; }
.v-controls button:focus-visible, .v-controls select:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
.v-group { display: inline-flex; gap: 4px; flex-wrap: wrap; }
.v-check { display: inline-flex; gap: 6px; align-items: center; }
.v-views { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
@media (max-width: 640px) { .v-views { grid-template-columns: 1fr; } }
.v-view { margin: 0; min-width: 0; }
.v-view canvas { width: 100%; display: block; border-radius: 8px; border: 1px solid var(--line); touch-action: none; }
.v-view figcaption { font-family: var(--display); font-weight: 700; color: var(--ink); }
.v-status { color: var(--muted); font-size: 14px; min-height: 1.2em; }
"""
(SITE / "style.css").write_text(css)

HEAD = """<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover">
<title>%s</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,500;12..96,700&family=Source+Serif+4:opsz,wght@8..60,400;8..60,600&family=IBM+Plex+Mono:wght@400;500&display=swap">
<link rel="stylesheet" href="../style.css">
"""

# grid rows
grid = re.search(r'<table class="board grid">.*?<tbody>(.*?)</tbody>', tpl, re.S).group(1)
rows = re.findall(r"<tr><td>(.*?)</td>(.*?)</tr>", grid, re.S)
state = {}
for name, rest in rows:
    state[name] = re.findall(r'<span class="pill (\w+)">(.*?)</span>', rest)

# details
dets = re.findall(r'<details class="zone"(?: open)?><summary>(.*?) <span class="num">(.*?)</span></summary><div class="body">\n(.*?)</div></details>', tpl, re.S)
body = {n: (s, b) for n, s, b in dets}
assert len(body) == 8 and len(state) == 8

footer_tpl = '<footer>\n    <p><a class="back" href="../index.html">Back to Likeness HQ</a></p>\n    <p>Numbers come from <code>kotor_fit_v6b.json</code>, <code>variety_v6.json</code>, <code>variety_v6b.json</code> and <code>judge_v6/identity.json</code> in the likeness-fit data folder, made on 19 November 2026.</p>\n  </footer>'

for slug, name in ZONES:
    st = state[name]
    var_pill, rep_pill = st[0][1], st[1][1]
    v_whole, v_zone = [x.strip() for x in var_pill.replace(" mm", "").split("·")]
    k_whole, k_zone, fit = [x.strip() for x in rep_pill.replace(" mm", "").split("·")]
    within = re.search(r"(\d+) of 57", body[name][0]).group(1)
    step_rows = "\n".join(
        f'          <tr><td class="name">{s}</td><td><span class="pill {c}">{t}</span></td></tr>' for s, (c, t) in zip(STEPS, st))
    text = fill(body[name][1]).replace("\n      ", "\n    ").strip()
    # extra sections of a zone page, kept in extra_<slug>.html next to this script
    extra_file = HQ / f"extra_{slug}.html"
    extra = fill(extra_file.read_text()) if extra_file.exists() else ""
    lead_file = HQ / f"lead_{slug}.html"
    lead = lead_file.read_text().replace("{{build}}", BUILD) if lead_file.exists() else ""
    page = f"""<!doctype html>
<html lang="en">
<head>
{HEAD % (name + " · Likeness HQ")}</head>
<body>
<div class="wrap">
  <header class="lede">
    <p><a class="back" href="../index.html">Back to Likeness HQ</a></p>
    <div class="eyebrow">kjk head customization · zone by zone · status on 19 November 2026</div>
    <h1>{name}</h1>
  </header>
{lead}

  <section aria-labelledby="steps-title">
    <h2 id="steps-title">Steps</h2>
    <div class="board-wrap">
      <table class="board">
        <thead><tr><th>Step</th><th>State</th></tr></thead>
        <tbody>
{step_rows}
        </tbody>
      </table>
    </div>
  </section>

  <section aria-labelledby="numbers-title">
    <h2 id="numbers-title">Numbers</h2>
    <ul class="facts">
      <li>Two KOTOR heads differ by <span class="num">{v_zone} mm</span> in this zone on average, with the zone lined up on itself, so that only its shape counts.</li>
      <li>Two of our fitted heads differ by <span class="num">{k_zone}</span> of that. Below 100%, our fits are more alike than KOTOR's heads. Above 100%, they differ a little more, partly from each fit's own error.</li>
      <li>Each fitted head lies <span class="num">{fit} mm</span> from its own KOTOR head in this zone on average.</li>
      <li><span class="num">{within} of 57</span> fitted heads lie within <span class="num">0.75 mm</span> of their own KOTOR head in this zone.</li>
    </ul>
  </section>

  <section aria-labelledby="notes-title">
    <h2 id="notes-title">What we see</h2>
    <div class="zone-body">
    {text}
    </div>
  </section>

{extra}
  <!-- ADD NEW SECTIONS HERE, before the footer. Each one is a <section aria-labelledby="..."> like the ones above. -->

  {footer_tpl}
</div>
</body>
</html>
"""
    (SITE / "zones" / f"{slug}.html").write_text(page)

# paint page (part C), from paint_body.html
from PIL import Image
S = pathlib.Path("/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/skin_tone")
BC = S.parent / "rbake" / "cmp"
L = pathlib.Path.home() / ".claude/tools/likeness-fit-data/kotor-heads/layout_v2"
def jpg(src, name, width, crop_h=None):
    im = Image.open(src).convert("RGB")
    if crop_h: im = im.crop((0, 0, im.width, crop_h))
    if im.width > width: im = im.resize((width, round(im.height * width / im.width)), Image.LANCZOS)
    im.save(SITE / "img" / name, "JPEG", quality=84, optimize=True)
    return "img/" + name
ppics = {"paint_tones": jpg(S / "tones.png", "paint_tones.jpg", 1330),
         "paint_patterns": jpg(S / "patterns.png", "paint_patterns.jpg", 1040, 780),
         "paint_layout": jpg(L / "layout_flat.png", "paint_layout.jpg", 1000),
         "paint_checker": jpg(L / "checker_front.png", "paint_checker.jpg", 800),
         "paint_seams": jpg(L / "seams.png", "paint_seams.jpg", 1600),
         "paint_bake_eye": jpg(BC / "mod_tcs_bastila_eye.png", "paint_bake_eye.jpg", 1600),
         "paint_bake_lips": jpg(BC / "mod_tcs_bastila_lips.png", "paint_bake_lips.jpg", 1600),
         "paint_layers": jpg(S.parent / "shading_layer" / "layers.png", "paint_layers.jpg", 1500)}
body = re.sub(r"\{\{(\w+)\}\}", lambda m: ppics[m.group(1)], (HQ / "paint_body.html").read_text())
paint = f"""<!doctype html>
<html lang="en">
<head>
{(HEAD % "Paint · Likeness HQ").replace("../style.css", "style.css")}</head>
<body>
<div class="wrap">
  <header class="lede">
    <p><a class="back" href="index.html">Back to Likeness HQ</a></p>
    <div class="eyebrow">kjk head customization · part C · status on 19 November 2026</div>
    <h1>Paint</h1>
  </header>

{body}
  <footer>
    <p><a class="back" href="index.html">Back to Likeness HQ</a></p>
    <p>The numbers come from the texture layout trials (<code>scan/trials-2026-11-18/README.md</code>, "Texture layout v2") and from the skin tone analysis, made on 19 November 2026.</p>
  </footer>
</div>
</body>
</html>
"""
(SITE / "paint.html").write_text(paint)

# main page
idx = tpl
idx = re.sub(r"<style>\n.*?</style>", '<link rel="stylesheet" href="style.css">', idx, flags=re.S)
for slug, name in ZONES:
    idx = idx.replace(f"<tr><td>{name}</td>", f'<tr><td><a href="zones/{slug}.html">{name}</a></td>', 1)
links = "\n".join(f'      <li><a href="zones/{s}.html">{n}</a></li>' for s, n in ZONES)
new = ('<h3>Each zone</h3>\n    <p>Each zone has its own page with its steps, its numbers and its pictures.</p>\n'
       f'    <ul class="zone-links">\n{links}\n    </ul>\n')
idx = re.sub(r'<h3>Each zone</h3>\n.*?(?=    <p><code>pmhc04</code>)', new, idx, flags=re.S)
idx = idx.replace("So far there are galleries for the mouth only.", "So far there are galleries for the mouth only, on the mouth page.")
assert "{{" not in idx and "<details" not in idx
(SITE / "index.html").write_text(idx)
print("built", len(list((SITE / "img").glob("*.jpg"))), "pictures")
