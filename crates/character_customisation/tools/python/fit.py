"""Fits the Likeness Lab head to a front photo and prints a share code.

The share code sets the face proportions and the face averages (gender, age and region).
The paint and the hair keep the page's defaults and are finished by hand in the page.

Usage: python -I fit.py <photo> [options]
The share code goes to stdout. The report goes to stderr.
"""
import argparse, json, os, sys
import numpy as np
from PIL import Image, ImageOps
from scipy.optimize import lsq_linear

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import head as head_module
from head import Head, share_code, REGIONS
from detect import Detector, IRIS_L, IRIS_R

DATA = os.path.join(os.path.dirname(HERE), "likeness-fit-data")
SKIP_GROUPS = {"ears", "neck"}  # no front landmark sees them
RIDGE = 0.05


class Correspondence:
    """The landmark pairs, and what calibrate.py measured about the detector for each one.

    The detector reports a landmark at bias + average + gain × (true position − average), plus noise.
    The weights are 1 / noise², so a landmark the detector places poorly counts for less.
    """

    def __init__(self, path=os.path.join(HERE, "correspondence.json")):
        c = json.load(open(path))["landmarks"]
        self.names = [o["name"] for o in c]
        self.mp = np.array([o["mediapipe"] for o in c])
        self.verts = np.array([o["vertex"] for o in c])
        self.bias = np.array([o["bias"] for o in c])
        self.gain = np.array([o.get("gain", [1, 1]) for o in c], dtype=float)
        self.weights = np.array([o["axis_weight"] if "axis_weight" in o else [o["weight"]] * 2 for o in c], dtype=float)


def normalise(pts, eye_r, eye_l):
    """Moves the eye midpoint to the origin, levels the eyes and divides by the eye distance. y points up."""
    mid = (eye_r + eye_l) / 2
    axis = eye_l - eye_r
    ipd = np.linalg.norm(axis)
    c, s = axis / ipd
    rot = np.array([[c, s], [-s, c]])
    return ((pts - mid) @ rot.T) / ipd


def photo_frame(points, corr):
    """The photo landmarks in the eye frame, moved by the detector's known offset so they sit where the mesh vertices would."""
    p = points * [1, -1]  # image y points down
    return normalise(p[corr.mp], p[IRIS_R], p[IRIS_L]) - corr.bias


class Problem:
    """The landmark vertices and eye joints of every target, so a solve only touches a few dozen vertices."""

    def __init__(self, head, corr, asym):
        self.h = head
        self.corr = corr
        self.joints_r = head.eye_joints["R"]
        self.joints_l = head.eye_joints["L"]
        self.sub = np.concatenate([corr.verts, self.joints_r, self.joints_l])
        self.nv = len(corr.verts)
        lookup = {v: i for i, v in enumerate(self.sub)}
        n_sub = len(self.sub)
        self.dims = corr.gain.shape[1]  # 2 for photo landmarks, 3 for a reconstructed face
        self.base = head.base[self.sub, :self.dims]
        self.tdelta = []
        for idx, d in head.targets:
            out = np.zeros((n_sub, self.dims))
            for k, vi in enumerate(idx):
                j = lookup.get(int(vi))
                if j is not None:
                    out[j] += d[k, :self.dims]
            self.tdelta.append(out)
        # Each unknown is one side of one modifier, or of a left and right pair when the face is symmetric.
        self.units = []  # (modifier indices, sign, target indices)
        seen = set()
        for i, m in enumerate(head.modifiers):
            if m["group"] in SKIP_GROUPS or (m["group"] == "asym" and not asym) or m["name"] in seen:
                continue
            mods = [i]
            if not asym and m.get("pair"):
                mods.append(head.by_name[m["pair"]])
                seen.add(m["pair"])
            for sign, key in ((1, "pos"), (-1, "neg")):
                ts = [head.modifiers[j][key] for j in mods]
                if all(t >= 0 for t in ts):
                    self.units.append((mods, sign, ts))
        self.U = np.stack([sum(self.tdelta[t] for t in ts) for _, _, ts in self.units])  # units × vertices × 2
        self.average = self.frame(self.positions(head.macro_weights(0, 0, (1 / 3,) * 3), np.zeros(len(self.units))))

    def positions(self, macro_w, x):
        p = self.base.copy()
        for ti, w in macro_w.items():
            p += w * self.tdelta[ti]
        return p + np.tensordot(x, self.U, 1)

    def frame(self, p):
        nr = len(self.joints_r)
        eye_r = p[self.nv:self.nv + nr].mean(0)
        eye_l = p[self.nv + nr:].mean(0)
        if self.dims == 3:
            return (p[:self.nv] - (eye_r + eye_l) / 2) / np.linalg.norm(eye_l - eye_r)
        return normalise(p[:self.nv], eye_r, eye_l)

    def detected(self, f):
        """Where the detector would report the landmarks of a head whose true landmarks are f."""
        return self.average + self.corr.gain * (f - self.average)

    def jacobian(self, macro_w, x, eps=0.02):
        p0 = self.positions(macro_w, x)
        f0 = self.detected(self.frame(p0)).ravel()
        J = np.empty((len(f0), len(x)))
        for j in range(len(x)):
            J[:, j] = (self.detected(self.frame(p0 + eps * self.U[j])).ravel() - f0) / eps
        return f0, J

    def solve(self, macro_w, target, ridge, iters=3, x0=None):
        x = np.zeros(len(self.units)) if x0 is None else x0.copy()
        W = np.sqrt(self.corr.weights).ravel()
        for _ in range(iters):
            f0, J = self.jacobian(macro_w, x)
            A = np.vstack([W[:, None] * J, np.sqrt(ridge) * np.eye(len(x))])
            b = np.concatenate([W * (target.ravel() - f0 + J @ x), np.zeros(len(x))])
            x = lsq_linear(A, b, bounds=(0, 1), method="bvls").x
        r = self.detected(self.frame(self.positions(macro_w, x))) - target
        cost = float(np.sum(self.corr.weights * r ** 2) + ridge * np.sum(x ** 2))
        return x, r, cost

    def values(self, x):
        v = np.zeros(len(self.h.modifiers))
        for (mods, sign, _), xv in zip(self.units, x):
            for m in mods:
                v[m] += sign * xv
        return np.clip(v, -1, 1)


def fit(head, target, corr, asym=False, ridge=RIDGE, gender=None, age=None, regions=None):
    """Solves the shape values. Gender, age or regions left as None are searched on a grid."""
    prob = Problem(head, corr, asym)
    genders = [gender] if gender is not None else np.linspace(-1, 1, 9)
    ages = [age] if age is not None else [0.0, 0.25, 0.5]
    mixes = [tuple(regions)] if regions is not None else [(1 / 3, 1 / 3, 1 / 3), (0.7, 0.15, 0.15), (0.15, 0.7, 0.15), (0.15, 0.15, 0.7)]
    best = None
    for g in genders:
        for a in ages:
            for mix in mixes:
                x, r, cost = prob.solve(head.macro_weights(g, a, mix), target, ridge, iters=1)
                if best is None or cost < best[0]:
                    best = (cost, g, a, mix)
    _, g, a, mix = best
    # A finer pass around the best grid point.
    if gender is None:
        for g2 in np.clip(g + np.linspace(-0.2, 0.2, 5), -1, 1):
            x, r, cost = prob.solve(head.macro_weights(g2, a, mix), target, ridge, iters=1)
            if cost < best[0]:
                best = (cost, g2, a, mix)
        _, g, a, mix = best
    x, r, cost = prob.solve(head.macro_weights(g, a, mix), target, ridge, iters=3)
    r0 = prob.average - target
    return {"values": prob.values(x), "gender": float(g), "age": float(a), "regions": mix, "residual": r, "residual_average_face": r0, "prob": prob, "x": x}


def report(head, corr, res, det, caricature, out):
    names = corr.names
    w = out.write
    for sev, msg in det.problems:
        w(f"{'Refused' if sev == 'refuse' else 'Warning'}. {msg}\n")
    w(f"The head is turned {det.yaw:+.1f} degrees, tilted {det.pitch:+.1f} degrees and rolled {det.roll:+.1f} degrees.\n")
    pr, pl = sorted([det.points[IRIS_R], det.points[IRIS_L]], key=lambda q: q[0])
    w(f"The pupils are at {pr[0]:.1f},{pr[1]:.1f} {pl[0]:.1f},{pl[1]:.1f} in photo pixels. Paste this into the Likeness Lab to align the photo.\n\n")
    g = res["gender"]
    w(f"Face averages. Gender {g:+.2f} (−1 is female, 1 is male). Age {res['age']:.2f} (0 is young, 1 is old). ")
    w("Regions " + ", ".join(f"{n} {v:.2f}" for n, v in zip(REGIONS, res["regions"])) + ".\n")
    if caricature != 1:
        w(f"Every shape value is multiplied by {caricature:.2f} before it goes into the share code.\n")
    v = res["values"] * caricature
    order = np.argsort(-np.abs(v))
    w("\nThe ten largest shape values:\n")
    for i in order[:10]:
        m = head.modifiers[i]
        side = {"L": " (left)", "R": " (right)"}.get(m["side"], "")
        if abs(v[i]) < 0.005:
            break
        w(f"  {v[i]:+.2f}  {m['group']} / {m['label']}{side}\n")
    w("\nResidual per landmark, in percent of the distance between the pupils (before the fit, then after):\n")
    r, r0 = np.linalg.norm(res["residual"], axis=1), np.linalg.norm(res["residual_average_face"], axis=1)
    for k in np.argsort(-r):
        if names[k].startswith("mp"):
            continue
        w(f"  {names[k]:15} {100 * r0[k]:5.1f}  →  {100 * r[k]:5.1f}\n")
    w(f"\nMean residual {100 * r0.mean():.1f}% for the average face and {100 * r.mean():.1f}% after the fit.\n")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("photo")
    ap.add_argument("--data", default=os.path.join(DATA, "likeness"), help="the folder with head.json and targets.bin")
    ap.add_argument("--model", default=os.path.join(DATA, "model", "face_landmarker.task"))
    ap.add_argument("--gender", type=float, default=0.0, help="from −1 (female) to 1 (male). The default is 0")
    ap.add_argument("--age", type=float, default=0.0, help="from 0 (about 25 years) to 1 (about 90 years), as in MakeHuman. The default is 0")
    ap.add_argument("--regions", default="1,1,1", help="the African, Asian and Caucasian weights, such as 0,0,1. The default is an equal mix")
    ap.add_argument("--fit-averages", action="store_true",
                    help="search the gender, the age and the regions instead of taking them from the options. "
                         "This works on renders of the head but drifts to the extremes on real photos")
    ap.add_argument("--asym", action="store_true", help="fit the left and right sides separately and use the asymmetry shapes")
    ap.add_argument("--ridge", type=float, default=RIDGE, help="how strongly the fit prefers small values")
    ap.add_argument("--caricature", type=float, default=1.0, help="multiply every shape value before writing the code")
    ap.add_argument("--page", help="the Likeness Lab page source, to take the share code version and the paint defaults from it")
    ap.add_argument("--force", action="store_true", help="fit even when the photo checks refuse the photo")
    ap.add_argument("--overlay", help="save the photo with the detected and the fitted landmarks drawn on it")
    a = ap.parse_args()

    if a.page:
        head_module.read_page(a.page)
    img = np.asarray(ImageOps.exif_transpose(Image.open(a.photo)).convert("RGB"))
    det = Detector(a.model).detect(img)
    refused = [m for s, m in det.problems if s == "refuse"]
    head = Head(a.data)
    corr = Correspondence()
    target = photo_frame(det.points, corr)
    regions = [float(x) for x in a.regions.split(",")]
    regions = tuple(r / sum(regions) for r in regions)
    if a.fit_averages:
        res = fit(head, target, corr, asym=a.asym, ridge=a.ridge)
    else:
        res = fit(head, target, corr, asym=a.asym, ridge=a.ridge, gender=a.gender, age=a.age, regions=regions)
    report(head, corr, res, det, a.caricature, sys.stderr)
    if a.overlay:
        draw_overlay(img, det, corr, target, res, a.overlay)
    if refused and not a.force:
        sys.stderr.write("\nNo share code was written because the photo was refused. Pass --force to fit it anyway.\n")
        sys.exit(2)
    print(share_code(res["values"] * a.caricature, res["gender"], res["age"], res["regions"], symmetry=not a.asym))


def draw_overlay(img, det, corr, target, res, path):
    from PIL import ImageDraw
    im = Image.fromarray(img)
    dr = ImageDraw.Draw(im)
    p = det.points * [1, -1]
    er, el = p[IRIS_R], p[IRIS_L]
    axis = el - er
    ipd = np.linalg.norm(axis)
    c, s = axis / ipd
    rot = np.array([[c, s], [-s, c]])
    to_img = lambda q: ((q * ipd) @ rot + (er + el) / 2) * [1, -1]
    fitted = to_img(target + res["residual"] + corr.bias)
    rad = max(2, ipd / 40)
    for a, b in zip(det.points[corr.mp], fitted):
        dr.line([tuple(a), tuple(b)], fill=(255, 220, 0), width=1)
        dr.ellipse([a[0] - rad, a[1] - rad, a[0] + rad, a[1] + rad], outline=(255, 40, 40), width=2)
        dr.ellipse([b[0] - rad, b[1] - rad, b[0] + rad, b[1] + rad], fill=(40, 140, 255))
    im.save(path)


if __name__ == "__main__":
    main()
