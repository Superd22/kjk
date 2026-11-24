"""Writes the eye paint's page, hq/paint/eyes/index.html, from the pictures and measures already made. Glue only."""
import json, os, html
HQ = os.path.expanduser("~/.claude/tools/likeness-fit/hq/paint/eyes")
DATA = os.path.expanduser("~/.claude/tools/likeness-fit-data/paint/eyes")
names = sorted(n[:-9] for n in os.listdir(f"{HQ}/fitted") if n.endswith("_eyes.png"))
fits = json.load(open(f"{DATA}/fits.json"))
makeup = json.load(open(f"{DATA}/makeup.json"))
mk = json.load(open(f"{DATA}/measure_kotor.json"))
mo = json.load(open(f"{DATA}/measure_ours.json"))
keys = [("upper_dark", "Upper band, L* darker"), ("lower_dark", "Lower band, L* darker"), ("socket", "Socket, L* darker"), ("under", "Under the eye, L* darker"), ("rim_red", "Lower rim, a* redder"), ("lid_red", "Lid skin, a* redder")]
def num(v):
    return "" if v is None else f"{v:.1f}"
rows = []
for n in names:
    k, o = mk.get(n) or {}, mo.get(n) or {}
    cells = "".join(f"<td>{num(k.get(key))} / {num(o.get(key))}</td>" for key, _ in keys)
    sliders = ", ".join(f"{a[4:]} {b:g}" for a, b in sorted({**fits.get(n, {}), **makeup.get(n, {})}.items()))
    rows.append(f"""<div class="pair" id="{n}"><h3>{n}</h3><div class="imgs"><figure><img loading="lazy" src="kotor/{n}_eyes.png"><figcaption>KOTOR</figcaption></figure><figure><img loading="lazy" src="fitted/{n}_eyes.png"><figcaption>Ours, fitted</figcaption></figure></div>
<div class="imgs small"><figure><img loading="lazy" src="kotor/{n}_face.png"></figure><figure><img loading="lazy" src="fitted/{n}_face.png"></figure><figure><img loading="lazy" src="kotor/{n}_eye.png"></figure><figure><img loading="lazy" src="fitted/{n}_eye.png"></figure></div>
<table><tr>{''.join(f'<th>{html.escape(l)}<br>KOTOR / ours</th>' for _, l in keys)}</tr><tr>{cells}</tr></table><p class="s">Sliders: {html.escape(sliders)}</p></div>""")
page = f"""<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Eye paint</title>
<style>body{{background:#16181c;color:#ddd;font:14px/1.45 system-ui,sans-serif;margin:16px;max-width:1500px}}a{{color:#8cf}}img{{max-width:100%;display:block}}
.imgs{{display:grid;grid-template-columns:1fr 1fr;gap:6px}}.small{{grid-template-columns:repeat(4,1fr);margin-top:6px}}figure{{margin:0}}figcaption{{font-size:12px;color:#aaa}}
.pair{{border-top:1px solid #333;padding:10px 0}}table{{border-collapse:collapse;font-size:12px;margin-top:6px}}td,th{{border:1px solid #333;padding:2px 6px;font-weight:normal}}.s{{font-size:12px;color:#aaa}}</style></head><body>
<h1>Eye paint</h1>
<p>Each pair shows KOTOR's eyes on its own head with its own texture, and our head with the eye paint at the values fitted to that head. The eyeballs are drawn the same plain brown way on both sides, so that only the paint round the eyes differs. The views are taken from the front: both eyes at 8 pixels per millimetre, the whole face as at conversation distance, and one eye as in the editor.</p>
<p>Other pictures: <a href="sheet.jpg">the six heads set by hand, KOTOR next to ours</a>, <a href="strip.jpg">each slider at three values</a>, and the walls of KOTOR's eyes sorted by each measure, which prove the measure: <a href="look/sorted_upper_dark.jpg">upper band</a>, <a href="look/sorted_lower_dark.jpg">lower band</a>, <a href="look/sorted_socket.jpg">socket</a>, <a href="look/sorted_under.jpg">under the eye</a>, <a href="look/sorted_rim_red.jpg">lower rim redness</a> and <a href="look/sorted_lid_red.jpg">lid redness</a>. The wall of all 57 KOTOR eyes is <a href="look/wall_kotor.jpg">here</a>.</p>
<p>The table under each pair gives the measure on KOTOR and on ours after the fit. The widths of the lash bands were not fitted, because the measure could not read them, and stay at their defaults. Makeup was set by hand on 8 heads.</p>
<p>Heads: {" ".join(f'<a href="#{n}">{n}</a>' for n in names)}</p>
{''.join(rows)}
</body></html>"""
open(f"{HQ}/index.html", "w").write(page)
print(len(names), "heads")
