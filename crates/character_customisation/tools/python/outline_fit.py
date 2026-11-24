"""Fits the painted brows to the brows cut out of a front photo.

The photo is warped into the render's frame at the given pose, aligned on the pupils. Each brow is then read
column by column. A dynamic-programming search follows the darkest continuous band near MediaPipe's brow line,
and never comes closer than 0.1 pupil distances to the upper lid. In each column the top edge is where the profile
brightens fastest going up from that band, and the bottom edge is where it brightens fastest going down.

A column is marked unknown when the light there is too dark to tell brow from hair or shadow. A column with
clear skin and no dark band is marked bare, so the fit learns where the brow ends. A bottom edge is kept only
where the skin under the brow is clearly lighter than the brow. Under a brow in shadow, the fit uses the top
edge alone.

Every threshold comes from the photo itself. Lengths are fractions of the distance between the pupils, and
darkness is measured against the local skin brightness, so a brow in shadow and a brow in light read alike.

The renderer draws the painted brow stroke alone. A search moves the eight brow numbers and the brow position
slider until the stroke's top and bottom edges and its ends match the photo's, with a small pull toward the
starting values.

Usage: python -I outline_fit.py <share code file> <photo> <out code file> --distance D --pitch P --yaw Y [--debug out.png]
"""
import argparse, os, sys
import numpy as np
import cv2
import cma
from PIL import Image, ImageOps

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import head as head_module
from head import Head, read_share_code_full, share_code
from fit import DATA
from detect import Detector
from glrender import GLHead, BROW_KEYS

# MediaPipe's brow points, upper and lower edge, from the inner end to the outer end
BROW_R_UP, BROW_R_LO = [107, 66, 105, 63, 70], [55, 65, 52, 53, 46]  # the subject's right brow, image left
BROW_L_UP, BROW_L_LO = [336, 296, 334, 293, 300], [285, 295, 282, 283, 276]
LID_R, LID_L = [33, 246, 161, 160, 159, 158, 157, 173, 133], [263, 466, 388, 387, 386, 385, 384, 398, 362]  # upper lid edges
SIZE = 512
BROW, BARE, UNKNOWN = 1, 0, -1


def warp_photo(rgb, points, p5):
    """The similarity that puts the photo's pupils on the render's pupils, and the warped photo."""
    src, dst = np.array([points[468], points[473]], dtype=np.float32), p5[:2]
    sv, dv = src[1] - src[0], dst[1] - dst[0]
    s = np.linalg.norm(dv) / np.linalg.norm(sv)
    ang = np.arctan2(dv[1], dv[0]) - np.arctan2(sv[1], sv[0])
    M = np.array([[s * np.cos(ang), -s * np.sin(ang), 0], [s * np.sin(ang), s * np.cos(ang), 0]], dtype=np.float32)
    M[:, 2] = dst[0] - M[:, :2] @ src[0]
    return M, cv2.warpAffine(rgb, M, (SIZE, SIZE), borderValue=(0, 0, 0))


def brow_columns(pts, ipd, mid_x):
    """For each side, the columns to read and MediaPipe's brow centre line over them."""
    sides = []
    for up, lo, lid in ((BROW_R_UP, BROW_R_LO, LID_R), (BROW_L_UP, BROW_L_LO, LID_L)):
        c = (pts[up] + pts[lo]) / 2
        coef = np.polyfit(c[:, 0], c[:, 1], 2)
        inner, outer = c[0, 0], c[-1, 0]
        sgn = np.sign(outer - inner)
        x0 = inner - sgn * 0.15 * ipd
        if sgn * (x0 - mid_x) < 0.03 * ipd:
            x0 = mid_x + sgn * 0.03 * ipd
        x1 = outer + sgn * 0.25 * ipd
        xs = np.arange(int(round(min(x0, x1))), int(round(max(x0, x1))) + 1)
        xs = xs[(xs >= 0) & (xs < SIZE)]
        # the line between the inner and outer MediaPipe points, then held level past them, so it does not curl
        xc = np.clip(xs, min(inner, outer), max(inner, outer))
        L = pts[lid][np.argsort(pts[lid][:, 0])]
        sides.append(dict(xs=xs, centre=np.polyval(coef, xc), inner=inner, outer=outer, sgn=sgn, lid=np.interp(xs, L[:, 0], L[:, 1])))
    return sides


def shoulder(q):
    """The start of a profile up to where it stops rising: the first bright level next to a dark band."""
    e, tol = 0, 0.02 * max(q.max(), 1.0)
    while e + 1 < len(q) and q[e + 1] > q[:e + 1].max() - tol:
        e += 1
    return q[:e + 1]


def steepest(q):
    """Where q rises fastest, with a sub-pixel step."""
    g = np.diff(q)
    j = int(np.argmax(g))
    off = 0.0
    if 0 < j < len(g) - 1:
        den = g[j - 1] - 2 * g[j] + g[j + 1]
        if den < 0:
            off = 0.5 * (g[j - 1] - g[j + 1]) / den
    return j + 0.5 + off


def dark_path(rel, xs, cy, lid, up, down, seen, ipd):
    """The darkest continuous line along the brow, one row per column, found by dynamic programming.

    Each row scores its darkness against the local skin, less a mild cost for its distance from MediaPipe's line.
    Rows on the upper lid, or less than 0.1 pupil distances above it, are ruled out, so a dark eye socket is not taken.
    A step between columns costs in proportion to its height, so the line follows one dark band and does not
    jump between the brow, the fringe above and the lid below.
    """
    offs = np.arange(-int(up), int(down) + 1)
    n, k = len(xs), len(offs)
    D = np.zeros((n, k))
    for i, (x, c) in enumerate(zip(xs, cy)):
        if seen[i]:
            rows = np.clip(np.round(c + offs).astype(int), 0, rel.shape[0] - 1)
            D[i] = rel[rows, x] - 0.25 * (offs / (0.15 * ipd)) ** 2
            D[i, c + offs > lid[i] - 0.1 * ipd] = -1.0  # never on the lid or within 0.1 pupil distances above it
    step = 0.6 / ipd  # cost of a one-pixel step between neighbouring columns
    jump = step * np.abs(offs[:, None] - offs[None, :])
    acc, back = D[0].copy(), np.zeros((n, k), int)
    for i in range(1, n):
        tot = acc[None, :] - jump
        back[i] = np.argmax(tot, 1)
        acc = D[i] + tot[np.arange(k), back[i]]
    j = int(np.argmax(acc))
    out = np.zeros(n, int)
    for i in range(n - 1, -1, -1):
        out[i] = j
        j = back[i, j]
    return np.round(cy + offs[out]).astype(int)


def photo_brows(warped, pts, ipd, mid_x):
    """The photo's brows, per column: a label (brow, bare or unknown) and the top and bottom edge of the brow."""
    grey = cv2.cvtColor(warped, cv2.COLOR_RGB2GRAY).astype(np.float32)
    grey = cv2.GaussianBlur(grey, (0, 0), max(0.6, ipd * 0.008))
    k = max(5, int(ipd * 0.35)) | 1
    # The local skin brightness: the grey with every dark mark thinner than the disc closed over.
    bg = cv2.morphologyEx(grey, cv2.MORPH_CLOSE, cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (k, k)))
    bg = cv2.GaussianBlur(bg, (0, 0), ipd * 0.04)
    rel = np.clip((bg - grey) / np.maximum(bg, 1.0), 0, 1)
    sides = brow_columns(pts, ipd, mid_x)
    # Reference skin brightness: the bright end of the background along both brows.
    ref = np.percentile(np.concatenate([bg[np.clip(s["centre"].astype(int), 0, SIZE - 1), s["xs"]] for s in sides]), 80)
    mask = np.zeros(grey.shape, np.uint8)
    up, down = 0.22 * ipd, 0.15 * ipd
    for s in sides:
        xs, cy = s["xs"], s["centre"]
        n = len(xs)
        label, top, bot, depth = np.full(n, BARE), np.full(n, np.nan), np.full(n, np.nan), np.zeros(n)
        seen = np.array([bg[int(np.clip(c, 0, SIZE - 1)), x] >= 0.45 * ref for x, c in zip(xs, cy)])
        path = dark_path(rel, xs, cy, s["lid"], up, down, seen, ipd)
        cand = []
        for i, x in enumerate(xs):
            if not seen[i]:
                label[i] = UNKNOWN  # too dark to tell brow from hair or shadow
                continue
            a = int(max(0, cy[i] - up - 0.25 * ipd))
            p = grey[a:int(min(SIZE, cy[i] + down + 0.25 * ipd)) + 1, x]
            m = int(np.clip(path[i] - a, 1, len(p) - 2))
            # then slide down to the darkest point of that dark band
            while 0 < m < len(p) - 1 and min(p[m - 1], p[m + 1]) < p[m]:
                m += -1 if p[m - 1] < p[m + 1] else 1
            # The skin above is the first shoulder over the brow, where the profile stops rising, within 0.25
            # pupil distances, so a fringe further up is not mistaken for it.
            ab = shoulder(p[max(0, m - int(0.25 * ipd)):m + 1][::-1])
            s_up = ab.max()
            # the brow's dip: its depth below the skin above it, as a fraction of that skin's brightness
            depth[i] = (s_up - p[m]) / max(s_up, 1.0)
            if len(ab) < 3 or s_up - p[m] < 0.04 * s_up:
                label[i] = UNKNOWN  # no skin above the dark band: hair or shadow covers the brow's top
                continue
            # Each edge is where the profile brightens fastest between the darkest point and the shoulder.
            top[i] = a + m - steepest(ab)
            # The bottom edge counts only where the skin below the brow is clearly lighter than the brow, at least
            # a quarter as much as the skin above. Under a brow in shadow the dark runs on into the eye socket.
            # The skin below is the first shoulder under the brow, where the profile stops rising, so the bright
            # lid further down is not mistaken for it. It must lie within 0.3 pupil distances of the top edge.
            below = shoulder(p[m:max(m + 1, int(top[i] - a + 0.3 * ipd)) + 1])
            if len(below) >= 3 and below.max() - p[m] >= 0.25 * (s_up - p[m]):
                bot[i] = a + m + steepest(below)
            cand.append(i)
        # A column holds brow when its dip is at least 35% as deep as the side's deepest dips.
        if cand:
            strong = np.percentile(depth[cand], 90)
            for i in cand:
                if depth[i] >= 0.35 * strong:
                    label[i] = BROW
                    if not 0.03 * ipd <= bot[i] - top[i] <= 0.30 * ipd:
                        bot[i] = np.nan
                else:
                    top[i] = bot[i] = np.nan
        score = np.where(label == BROW, depth, 0)
        # Split the brow columns into stretches. A stretch bridges gaps of up to 0.12 pupil distances, but only
        # 0.03 pupil distances of bare skin. The brow runs from the first to the last stretch that is at least
        # 0.12 pupil distances long, or is the strongest one, so a hair or a mark past the brow's end stays out.
        gap = int(0.12 * ipd)
        runs, cur, last = [], None, None
        for i in np.flatnonzero(label == BROW):
            if cur is None or i - last > gap + 1 or (label[last + 1:i] == BARE).sum() > 0.03 * ipd:
                cur = [i, i]
                runs.append(cur)
            cur[1], last = i, i
        if runs:
            best = max(runs, key=lambda r: np.nansum(score[r[0]:r[1] + 1]))
            kept = [r for r in runs if r is best or r[1] - r[0] >= 0.12 * ipd]
            r0, r1 = min(r[0] for r in kept), max(r[1] for r in kept)
            for i in np.flatnonzero(label == BROW):
                if i < r0 or i > r1:
                    label[i], top[i], bot[i] = UNKNOWN, np.nan, np.nan
            for i in range(r0, r1 + 1):
                if label[i] != BROW:
                    label[i] = UNKNOWN
            # Smooth each edge with a running median over 0.1 pupil distances. An edge more than 0.04 pupil
            # distances from that median is a wrinkle, a hair or noise, and is dropped.
            w = max(1, int(0.05 * ipd))
            for arr in (top, bot):
                med = arr.copy()
                for i in range(r0, r1 + 1):
                    win = arr[max(r0, i - w):i + w + 1]
                    if np.isfinite(arr[i]) and np.isfinite(win).sum() >= 3:
                        med[i] = np.nanmedian(win)
                off = np.abs(arr - med) > 0.04 * ipd
                arr[off] = np.nan
                sm = arr.copy()
                v = max(1, int(0.02 * ipd))
                for i in range(r0, r1 + 1):
                    if np.isfinite(arr[i]):
                        sm[i] = np.nanmedian(arr[max(r0, i - v):i + v + 1])
                arr[:] = sm
            for i in range(r0, r1 + 1):
                if label[i] == BROW and np.isnan(top[i]):
                    label[i], bot[i] = UNKNOWN, np.nan
        s.update(label=label, top=top, bot=bot, depth=depth)
    # A bottom edge more than 1.4 times the median thickness of both brows below the top is shadow under the
    # brow, not brow. One less than 0.3 times that is noise.
    th = np.concatenate([s["bot"] - s["top"] for s in sides])
    if np.isfinite(th).sum() >= 5:
        typical = np.nanmedian(th)
        for s in sides:
            t = s["bot"] - s["top"]
            s["bot"][(t > 1.4 * typical) | (t < 0.3 * typical)] = np.nan
            s["typical"] = typical
    for s in sides:
        top, bot, label, xs = s["top"], s["bot"], s["label"], s["xs"]
        for i, x in enumerate(xs):
            if label[i] == BROW:
                mask[int(np.ceil(top[i])):int(np.floor(bot[i] if not np.isnan(bot[i]) else top[i] + 1)) + 1, x] = 1
    return sides, mask, rel


def render_columns(m, sides):
    """The rendered stroke's top and bottom edge in each of the photo's columns, at the 0.5 level, with sub-pixel steps."""
    out = []
    for s in sides:
        col = m[:, s["xs"]]
        on = col > 0.5
        has = on.any(0)
        first = np.argmax(on, 0)
        last = SIZE - 1 - np.argmax(on[::-1], 0)
        cols = np.arange(len(s["xs"]))

        def edge(i_in, i_out):
            a, b = col[i_out, cols], col[i_in, cols]
            f = np.clip((0.5 - a) / np.maximum(b - a, 1e-6), 0, 1)
            return i_out + (i_in - i_out) * f

        top = np.where(has, edge(first, np.maximum(first - 1, 0)), np.nan)
        bot = np.where(has, edge(last, np.minimum(last + 1, SIZE - 1)), np.nan)
        out.append((has, top, bot))
    return out


def shape_cost(sides, rendered, ipd):
    """The mean per-column mismatch: edge distances where both have brow, a fixed cost where only one has.

    A photo column whose bottom edge is unsure (the brow runs into shadow below) is matched on its top edge, and its
    thickness is pulled with half the weight toward the median thickness of the columns whose bottom edge is sure.
    """
    unit = 0.04 * ipd
    total, n = 0.0, 0
    for s, (has, top, bot) in zip(sides, rendered):
        lab = s["label"]
        both = (lab == BROW) & has
        dt = np.abs(top[both] - s["top"][both]) / unit
        db = np.abs(bot[both] - s["bot"][both]) / unit
        # Where the photo's bottom edge is unsure, the stroke is held loosely to the brows' usual thickness.
        guess = 0.5 * np.abs(bot[both] - s["top"][both] - s.get("typical", 0.1 * ipd)) / unit
        db = np.where(np.isnan(db), guess, db)
        total += np.minimum(dt + db, 4).sum()
        total += 3 * (((lab == BROW) & ~has).sum() + ((lab == BARE) & has).sum())
        n += (lab >= 0).sum()
    return total / max(n, 1)


def fit_brows(gl, v, g, age, reg, surf, sides, ipd, pose, lift, evals=600, prior=0.6, seed=3):
    """Searches the eight brow numbers and the brow position slider. Returns the best numbers and the start."""
    lo = np.array([-1, -1, -1, -1, 0, 0, -1, -1, -1.0])
    hi = np.array([1, 1, 1, 1, 1, 1, 1, 1, 1.0])
    span = hi - lo

    def apply(z):
        vv = v.copy()
        vv[lift] = z[-1]
        return vv, dict(zip(BROW_KEYS, z[:-1]), browColor=surf["browColor"], browDensity=1.0)

    z0 = np.array([surf[k] for k in BROW_KEYS] + [v[lift[0]]])

    def loss(z):
        vv, brow = apply(z)
        m = gl.brow_mask(vv, g, age, reg, brow, **pose)
        return shape_cost(sides, render_columns(m, sides), ipd) + prior * np.mean(((z - z0) / span) ** 2)

    start = loss(z0)
    es = cma.CMAEvolutionStrategy(np.clip(z0, lo + 1e-3, hi - 1e-3), 0.3, {"bounds": [lo, hi], "popsize": 14, "maxfevals": evals, "verbose": -9, "seed": seed})
    while not es.stop():
        Z = es.ask()
        es.tell(Z, [loss(z) for z in Z])
    z = es.result.xbest
    return z, z0, start, loss(z), apply


def draw_debug(warped, sides, mask, m, img):
    """Left: the photo with the cut's edges in red, unknown columns in yellow and bare ones in green on MediaPipe's
    brow line, and the fitted stroke's outline in blue. Right: the render. Below: both enlarged on the brows."""
    vis = warped.copy()
    for s in sides:
        for x, lab, cy, t, b in zip(s["xs"], s["label"], s["centre"], s["top"], s["bot"]):
            if lab == UNKNOWN:
                vis[int(cy), x] = [255, 220, 0]
            elif lab == BARE:
                vis[int(cy), x] = [80, 255, 80]
            else:
                vis[int(round(t)), x] = [255, 40, 40]
                if np.isfinite(b):
                    vis[int(round(b)), x] = [255, 40, 40]
    edge = cv2.Canny((m > 0.5).astype(np.uint8) * 255, 50, 150) > 0
    vis[edge] = [60, 160, 255]
    ys, xs = np.nonzero(mask | (m > 0.5))
    top_row = np.concatenate([vis, img], 1)
    if not len(ys):
        return top_row
    y0, y1, x0, x1 = max(0, ys.min() - 30), min(SIZE, ys.max() + 30), max(0, xs.min() - 30), min(SIZE, xs.max() + 30)
    f = SIZE / (x1 - x0)
    # enlarge the photo without the marks, then draw them at the enlarged scale so they stay one pixel wide
    zoom = cv2.resize(warped[y0:y1, x0:x1], None, fx=f, fy=f, interpolation=cv2.INTER_CUBIC)
    for s in sides:
        for x, lab, cy, t, b in zip(s["xs"], s["label"], s["centre"], s["top"], s["bot"]):
            X = int((x - x0) * f)
            put = lambda y, c: cv2.line(zoom, (X, int((y - y0) * f)), (X + int(f), int((y - y0) * f)), c, 2)
            if lab == UNKNOWN:
                put(cy, (255, 220, 0))
            elif lab == BARE:
                put(cy, (80, 255, 80))
            else:
                put(t, (255, 40, 40))
                if np.isfinite(b):
                    put(b, (255, 40, 40))
    mz = cv2.resize((m[y0:y1, x0:x1] > 0.5).astype(np.uint8) * 255, (zoom.shape[1], zoom.shape[0]), interpolation=cv2.INTER_LINEAR)
    cs, _ = cv2.findContours((mz > 127).astype(np.uint8), cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_NONE)
    cv2.drawContours(zoom, cs, -1, (60, 160, 255), 2)
    rz = cv2.resize(img[y0:y1, x0:x1], None, fx=f, fy=f, interpolation=cv2.INTER_CUBIC)
    bottom = np.concatenate([zoom, rz], 1)
    return np.concatenate([top_row, bottom], 0)


def fit_photo(gl, h, v, g, age, reg, surf, rgb, pose, detector, evals=600, log=print):
    """Cuts the brows out of the photo and fits the stroke to them. Returns the new sliders and brow numbers."""
    _, p5 = gl.render(v, g, age, reg, **pose)
    d = detector.detect(rgb)
    log(f"MediaPipe reads the face as turned {d.yaw:+.1f} and tilted {d.pitch:+.1f} degrees")
    M, warped = warp_photo(rgb, d.points, p5)
    pts = np.c_[d.points, np.ones(len(d.points))] @ M.T
    ipd = np.linalg.norm(pts[473] - pts[468])
    sides, mask, _ = photo_brows(warped, pts, ipd, (p5[0, 0] + p5[1, 0]) / 2)
    for name, s in zip(("image left", "image right"), sides):
        lab = s["label"]
        log(f"{name}: {np.sum(lab == BROW)} brow, {np.sum(lab == BARE)} bare, {np.sum(lab == UNKNOWN)} unknown columns, "
            f"{np.sum(np.isfinite(s['bot']))} with a bottom edge")
    lift = [i for i, m in enumerate(h.modifiers) if m["group"] == "eyebrows" and m["label"].startswith("Position (down / up)")]
    z, z0, c0, c1, apply = fit_brows(gl, v, g, age, reg, surf, sides, ipd, pose, lift, evals=evals)
    log(f"shape cost {c0:.2f} -> {c1:.2f}")
    for k, a0, a1 in zip(BROW_KEYS + ["lift"], z0, z):
        log(f"  {k:14} {a0:+.2f} -> {a1:+.2f}")
    vv, brow = apply(z)
    return dict(values=vv, brow=brow, z=z, z0=z0, warped=warped, sides=sides, mask=mask, cost=c1, pupils=pts[[468, 473]])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("code_file")
    ap.add_argument("photo")
    ap.add_argument("out")
    ap.add_argument("--distance", type=float, default=2.0)
    ap.add_argument("--pitch", type=float, default=0.0)
    ap.add_argument("--yaw", type=float, default=0.0)
    ap.add_argument("--evals", type=int, default=600)
    ap.add_argument("--debug")
    a = ap.parse_args()
    h = Head(os.path.join(DATA, "likeness"))
    v, g, age, reg, surf = read_share_code_full(open(a.code_file).read(), len(h.modifiers))
    gl = GLHead(h, SIZE, dist=a.distance)
    gl.set_eye_paint(surf["irisSize"], surf["pupilSize"])
    pose = dict(yaw=a.yaw, pitch=a.pitch)
    rgb = np.asarray(ImageOps.exif_transpose(Image.open(a.photo)).convert("RGB"))
    det = Detector(os.path.join(DATA, "model", "face_landmarker.task"))
    r = fit_photo(gl, h, v, g, age, reg, surf, rgb, pose, det, a.evals, log=lambda t: print(t, file=sys.stderr))
    vv, brow = r["values"], r["brow"]
    for k in BROW_KEYS:
        surf[k] = float(brow[k])
    head_module.SURF_DEF = [(k, kind, surf[k]) for k, kind, _ in head_module.SURF_DEF]
    open(a.out, "w").write(share_code(vv, g, age, reg))
    if a.debug:
        img, _ = gl.render(vv, g, age, reg, brow=dict(brow, browDensity=surf["browDensity"]), **pose)
        m = gl.brow_mask(vv, g, age, reg, brow, **pose)
        Image.fromarray(draw_debug(r["warped"], r["sides"], r["mask"], m, img)).save(a.debug)


if __name__ == "__main__":
    main()
