"""Glue for the fold work: builds the pages in hq/paint/folds/ from the Rust tools' JSON and tiles, and prints the scorecard.

Pages:
- index.html: the scorecard of every fold, and links.
- <fold>/index.html: for each of the 57 heads, KOTOR's own head next to ours with the folds and ours before them, seen from the
  front, with the measure of both.
- <fold>/measure.html: the wall of KOTOR's 57 crops sorted by the measure, with the found fold drawn or not, and the pairs of
  sides that the measure calls most alike and most different.

Usage: fold_pages.py
"""
import html, itertools, json, math
from pathlib import Path

DATA = Path.home() / ".claude/tools/likeness-fit-data/paint/folds"
PAGES = Path.home() / ".claude/tools/likeness-fit/hq/paint/folds"
FOLDS = [("nasolabial", "Nose-to-mouth fold"), ("marionette", "Mouth corner fold"), ("chin", "Chin crease"), ("under_eye", "Under-eye crease")]
SIDE = ["x < 0 side (the character's right)", "x > 0 side (the character's left)"]

STYLE = """<style>
:root { --bg: #16171a; --fg: #e8e6e1; --dim: #9a978f; --line: #33353a; --good: #6fbf73; --bad: #e0795f; --warn: #d9b45a; }
body { background: var(--bg); color: var(--fg); font: 14px/1.45 system-ui, sans-serif; margin: 0; padding: 16px; }
h1, h2 { font-weight: 600; } a { color: #8fb8ff; }
table { border-collapse: collapse; margin: 8px 0 20px; } td, th { border-bottom: 1px solid var(--line); padding: 4px 8px; text-align: left; vertical-align: top; }
.row { display: flex; gap: 6px; flex-wrap: wrap; margin-bottom: 18px; align-items: flex-start; }
.row img { max-width: 32vw; height: auto; display: block; }
figure { margin: 0; } figcaption { color: var(--dim); font-size: 12px; }
.wall { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 8px; }
.wall img { width: 100%; display: block; }
.note { color: var(--dim); max-width: 70em; }
.good { color: var(--good); } .bad { color: var(--bad); } .warn { color: var(--warn); }
</style>"""


def page(title, body):
    return f"<!doctype html><html><head><meta charset='utf-8'><meta name='viewport' content='width=device-width'><title>{html.escape(title)}</title>{STYLE}</head><body>{body}</body></html>"


def mean_track(m, a, b):
    t = m["track_mm"][a:b + 1]
    return sum(t) / len(t) if t else float("nan")


def compare(k, o):
    """What our side's measure says against KOTOR's."""
    if not k["present"] and not o["present"]:
        return {"verdict": "both without", "ok": True}
    if k["present"] and not o["present"]:
        return {"verdict": "KOTOR only", "ok": False}
    if o["present"] and not k["present"]:
        return {"verdict": "ours only", "ok": False}
    (ka, kb), (oa, ob) = k["run"], o["run"]
    lo, hi = max(ka, oa), min(kb, ob)
    place = abs(mean_track(k, lo, hi) - mean_track(o, lo, hi)) if lo <= hi else float("nan")
    return {
        "verdict": "both with",
        "ok": True,
        "depth": (k["depth_lstar"], o["depth_lstar"]),
        "core": (k["core_mm"], o["core_mm"]),
        "half": ((k["half_width_in_mm"], k["half_width_out_mm"]), (o["half_width_in_mm"], o["half_width_out_mm"])),
        "place": place,
        "overlap_mm": max(0, hi - lo + 1) * 0.25,
    }


def fmt_side(m):
    if not m["present"]:
        return "none"
    return f"{m['depth_lstar']:.1f} L* deep, {m['core_mm']:.0f} mm core, half widths {m['half_width_in_mm']:.1f} / {m['half_width_out_mm']:.1f} mm"


def median(v):
    v = sorted(x for x in v if x == x)
    return v[len(v) // 2] if v else float("nan")


def main():
    fits = json.load(open(DATA / "fits.json"))["heads"]
    kotor = json.load(open(DATA / "measure_kotor.json"))["heads"]
    names = sorted(fits)
    score = {}
    lines = []
    summary_rows = []
    for fold, label in FOLDS:
        rows = []
        for n in names:
            for s in (0, 1):
                k, o = fits[n][fold]["kotor"][s], fits[n][fold]["ours"][s]
                rows.append((n, s, k, o, compare(k, o)))
        score[fold] = rows
        both = [r[4] for r in rows if r[4]["verdict"] == "both with"]
        count = {v: sum(1 for r in rows if r[4]["verdict"] == v) for v in ("both with", "both without", "KOTOR only", "ours only")}
        dd = [abs(c["depth"][1] - c["depth"][0]) for c in both]
        dc = [abs(c["core"][1] - c["core"][0]) for c in both]
        dp = [c["place"] for c in both]
        dh = [abs(c["half"][1][0] - c["half"][0][0]) + abs(c["half"][1][1] - c["half"][0][1]) for c in both]
        within3 = sum(1 for x in dp if x == x and x <= 3.0)
        summary_rows.append((fold, label, count, median(dd), median(dc), median(dp), within3, len(both), median(dh), median([c["depth"][0] for c in both])))
    # The scorecard page.
    body = "<h1>Folds: KOTOR's and ours</h1><p class='note'>Each fold is measured on both sides of the 57 test heads: on KOTOR's paint and on ours. A side counts as having the fold when the measure finds a stretch at least 5 mm long that is at least 1.5 L* darker than the skin on both sides of it. Our fold sliders were fitted so that the measure of our paint matches KOTOR's (fold_fit). Pictures are front views of KOTOR's own head and of ours.</p>"
    body += "<table><tr><th>Fold</th><th>Both have it</th><th>Neither</th><th>KOTOR only</th><th>Ours only</th><th>Median depth gap</th><th>Median core length gap</th><th>Median place gap</th><th>Sides within 3 mm</th><th>Median half width gap (both sides added)</th></tr>"
    for fold, label, c, dd, dc, dp, w3, nb, dh, kd in summary_rows:
        body += f"<tr><td><a href='{fold}/index.html'>{label}</a> (<a href='{fold}/measure.html'>measure</a>)</td><td>{c['both with']}</td><td>{c['both without']}</td><td>{c['KOTOR only']}</td><td>{c['ours only']}</td><td>{dd:.1f} L* (KOTOR's median depth {kd:.1f})</td><td>{dc:.1f} mm</td><td>{dp:.1f} mm</td><td>{w3} of {nb}</td><td>{dh:.1f} mm</td></tr>"
        lines.append(f"{fold}: both {c['both with']}, neither {c['both without']}, KOTOR only {c['KOTOR only']}, ours only {c['ours only']}; median gaps: depth {dd:.2f} L* (KOTOR median {kd:.2f}), core {dc:.2f} mm, place {dp:.2f} mm ({w3}/{nb} within 3 mm), half widths {dh:.2f} mm")
    body += "</table><p class='note'>Counts are sides (114 per fold). The gaps are taken over the sides where both have the fold. The place gap is the mean sideways distance between KOTOR's found fold and ours over the stretch where both are found.</p>"
    (PAGES / "index.html").write_text(page("Folds scorecard", body))
    # One page per fold.
    for fold, label in FOLDS:
        rows = score[fold]
        order = sorted(names, key=lambda n: -sum(kotor[n][fold][s].get("score", 0) for s in (0, 1)))
        body = f"<h1>{label}</h1><p><a href='../index.html'>All folds</a> · <a href='measure.html'>The measure's wall and pairs</a></p><p class='note'>For each head, from the strongest KOTOR fold to the weakest: KOTOR's own head with its own texture, our head with the fitted folds, and our head before the folds (our current paint without the old hand-drawn wrinkle sets for this fold). All three are seen from the front through the same window, 10 pixels per millimetre.</p>"
        for n in order:
            sl = fits[n][fold]["sides"]
            bits = []
            for s in (0, 1):
                k, o = fits[n][fold]["kotor"][s], fits[n][fold]["ours"][s]
                c = compare(k, o)
                cls = "good" if c["ok"] else "bad"
                place = f", place gap {c['place']:.1f} mm" if c.get("place") == c.get("place") and "place" in c else ""
                bits.append(f"<b>{SIDE[s]}</b>: <span class='{cls}'>{c['verdict']}</span>{place}. KOTOR: {fmt_side(k)}. Ours: {fmt_side(o)}. Sliders: strength {sl[s]['strength']:.1f} L*, width {sl[s]['width_mm']:.1f} mm, sharpness {sl[s]['sharpness']:+.2f}, start {sl[s]['start_mm']:.1f} mm, length {sl[s]['length_mm']:.1f} mm, shift {sl[s]['shift_mm']:+.1f} mm.")
            body += f"<h2>{n}</h2><div class='row'><figure><img loading='lazy' src='tiles/{n}.jpg'><figcaption>KOTOR</figcaption></figure><figure><img loading='lazy' src='tiles/{n}_ours.jpg'><figcaption>Ours with folds</figcaption></figure><figure><img loading='lazy' src='tiles/{n}_before.jpg'><figcaption>Ours before</figcaption></figure></div><p class='note'>{'<br>'.join(bits)}</p>"
        (PAGES / fold / "index.html").write_text(page(label, body))
        # The measure page: the sorted wall and the pairs.
        body = f"<h1>{label}: the measure</h1><p><a href='index.html'>KOTOR next to ours</a> · <a href='../index.html'>All folds</a></p><p class='note'>The wall shows KOTOR's 57 heads sorted by the measure, the strongest fold first. The score is the mean of the fold's depth and edge contrast, in L*, times its length up to 20 mm over 20 mm, added over both sides. Green dots mark where the measure found the fold, yellow dots where it searched and found too little contrast. The plain wall is the same order without dots.</p>"
        body += "<h2>Sorted wall, with the found fold</h2><div class='wall'>"
        for i, n in enumerate(order):
            sc = sum(kotor[n][fold][s].get("score", 0) for s in (0, 1))
            body += f"<figure><img loading='lazy' src='tiles/{n}_track.jpg'><figcaption>{i + 1}. {n}, score {sc:.1f}</figcaption></figure>"
        body += "</div><h2>Sorted wall, plain</h2><div class='wall'>"
        for i, n in enumerate(order):
            body += f"<figure><img loading='lazy' src='tiles/{n}.jpg'><figcaption>{i + 1}. {n}</figcaption></figure>"
        body += "</div>"
        # Pairs: sides with the fold, compared on depth, edge, core length, half widths and place, each scaled by its spread.
        sides = [(n, s, kotor[n][fold][s]) for n in names for s in (0, 1) if kotor[n][fold][s]["present"]]
        feats = ["depth_lstar", "edge_lstar", "core_mm", "half_width_in_mm", "half_width_out_mm"]

        def vec(m):
            a, b = m["run"]
            return [m[f] for f in feats] + [mean_track(m, a, b)]

        vs = [vec(m) for _, _, m in sides]
        if len(vs) > 3:
            sds = [max(1e-6, (sum((v[i] - sum(x[i] for x in vs) / len(vs)) ** 2 for v in vs) / len(vs)) ** 0.5) for i in range(len(vs[0]))]
            pairs = []
            for (i, a), (j, b) in itertools.combinations(enumerate(sides), 2):
                if a[0] == b[0]:
                    continue
                d = math.sqrt(sum(((vs[i][k] - vs[j][k]) / sds[k]) ** 2 for k in range(len(sds))))
                pairs.append((d, a, b))
            pairs.sort(key=lambda p: p[0])
            for title, chosen in (("The 8 pairs the measure calls most alike", pairs[:8]), ("The 8 pairs the measure calls most different", pairs[-8:][::-1])):
                body += f"<h2>{title}</h2><p class='note'>Each pair is two sides of two different heads. The distance adds the differences in depth, edge, core length, both half widths and sideways place, each over its spread between heads.</p>"
                for d, a, b in chosen:
                    body += f"<div class='row'><figure><img loading='lazy' src='tiles/{a[0]}_track.jpg'><figcaption>{a[0]}, {SIDE[a[1]]}: {fmt_side(a[2])}</figcaption></figure><figure><img loading='lazy' src='tiles/{b[0]}_track.jpg'><figcaption>{b[0]}, {SIDE[b[1]]}: {fmt_side(b[2])}. Distance {d:.2f}</figcaption></figure></div>"
        (PAGES / fold / "measure.html").write_text(page(f"{label} measure", body))
    # The heads that look wrong by the numbers, for the report.
    for fold, label in FOLDS:
        bad = [(n, s, r[4]) for n, s, *_ in [] for r in []]
        wrong = [(n, s, c["verdict"]) for n, s, k, o, c in score[fold] if not c["ok"]]
        far = [(n, s, round(c["place"], 1)) for n, s, k, o, c in score[fold] if c["verdict"] == "both with" and c["place"] == c["place"] and c["place"] > 3.0]
        lines.append(f"{fold} mismatches: {wrong}")
        lines.append(f"{fold} placed more than 3 mm away: {far}")
    print("\n".join(lines))


main()
