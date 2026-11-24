"""Moves the face and brow sliders to raise a face recognition network's similarity between the render and the photo.

The search network is ArcFace (InsightFace w600k_r50). The judge network is glintr100, which the search never sees.
Usage:
  python -I idfit.py photo.jpg [--gender G --age A] [--evals N]   prints a share code
  python -I idfit.py --lineup <folder> [--evals N] [--method idfit|2d|average]
"""
import argparse, glob, json, os, sys, time
import numpy as np
import cv2
import cma
import onnxruntime as ort
from PIL import Image, ImageOps

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from fit import DATA, SKIP_GROUPS, Correspondence, fit, photo_frame
from head import Head, share_code
from detect import Detector
from glrender import GLHead, BROW_KEYS, BROW_DEFAULT
from likeness import TEMPLATE, five_points, GenderAge

NETS = {"search": os.path.join(DATA, "models", "arcface", "w600k_r50.onnx"), "judge": os.path.join(DATA, "models", "judge", "antelopev2", "glintr100.onnx")}
BROW_BOUNDS = {"browInner": (-1, 1), "browOuter": (-1, 1), "browArchPos": (-1, 1), "browArch": (-1, 1), "browThickIn": (0, 1), "browThickOut": (0, 1), "browLength": (-1, 1), "browSpacing": (-1, 1)}


class Net:
    def __init__(self, path):
        self.s = ort.InferenceSession(path, providers=["CPUExecutionProvider"])
        self.inp = self.s.get_inputs()[0].name

    def __call__(self, crops):
        blob = ((np.stack(crops).astype(np.float32) - 127.5) / 127.5).transpose(0, 3, 1, 2)
        e = self.s.run(None, {self.inp: blob})[0]
        return e / np.linalg.norm(e, axis=1, keepdims=True)


def crop(rgb, pts5):
    m, _ = cv2.estimateAffinePartial2D(pts5, TEMPLATE, method=cv2.LMEDS)
    return cv2.warpAffine(rgb, m, (112, 112), borderValue=0)


def unit(v):
    return v / np.linalg.norm(v, axis=-1, keepdims=True)


class Space:
    """The searched sliders: one value per symmetric face shape, then the brow shape numbers."""

    def __init__(self, head):
        self.h = head
        self.mods = []
        for i, m in enumerate(head.modifiers):
            if m["group"] in SKIP_GROUPS or m["group"] == "asym" or m["side"] == "R":
                continue
            self.mods.append((i, head.by_name[m["pair"]] if m.get("pair") else None, bool(m.get("oneSided"))))
        self.n_shape = len(self.mods)
        self.lo = np.array([0.0 if one else -1.0 for _, _, one in self.mods] + [BROW_BOUNDS[k][0] for k in BROW_KEYS])
        self.hi = np.array([1.0] * self.n_shape + [BROW_BOUNDS[k][1] for k in BROW_KEYS])

    def encode(self, values, brow=None):
        b = dict(BROW_DEFAULT, **(brow or {}))
        return np.array([values[i] for i, _, _ in self.mods] + [b[k] for k in BROW_KEYS])

    def decode(self, z):
        v = np.zeros(len(self.h.modifiers))
        for (i, j, _), x in zip(self.mods, z[:self.n_shape]):
            v[i] = x
            if j is not None:
                v[j] = x
        return v, dict(zip(BROW_KEYS, z[self.n_shape:]))


class Scorer:
    def __init__(self, head, size=256):
        self.h = head
        self.gl = GLHead(head, size)
        self.nets = {k: Net(p) for k, p in NETS.items()}
        self.space = Space(head)
        self.mu_r = {k: self._render_mean(k) for k in self.nets}

    def _render_mean(self, key, count=300):
        path = os.path.join(DATA, "models", f"gl_render_mean_{key}.npy")
        if os.path.exists(path):
            return np.load(path)
        from calibrate import random_face
        rng = np.random.default_rng(4242)
        crops = []
        for _ in range(count):
            v, g, a, m = random_face(self.h, rng)
            z = self.space.encode(v)
            z[self.space.n_shape:] = rng.uniform(self.space.lo[self.space.n_shape:], self.space.hi[self.space.n_shape:])
            vv, brow = self.space.decode(z)
            img, p5 = self.gl.render(vv, g, a, m, brow=brow)
            crops.append(crop(img, p5))
        mu = np.concatenate([self.nets[key](crops[i:i + 50]) for i in range(0, count, 50)]).mean(0)
        np.save(path, mu)
        return mu

    def render_crops(self, Z, gender, age, regions):
        out = []
        for z in Z:
            v, brow = self.space.decode(z)
            img, p5 = self.gl.render(v, gender, age, regions, brow=brow)
            out.append(crop(img, p5))
        return out

    def similarity(self, key, crops, target):
        """target is the photo's embedding with the photo mean removed, normalised."""
        return unit(self.nets[key](crops) - self.mu_r[key]) @ target


def photo_targets(scorer, rgb, points, mu_p):
    c = crop(rgb, five_points(points))
    return {k: unit(scorer.nets[k]([c])[0] - mu_p[k]) for k in scorer.nets}


def search(scorer, target, z0, gender, age, regions, evals=3000, sigma=0.2, ridge=0.02, seed=1):
    sp = scorer.space
    es = cma.CMAEvolutionStrategy(np.clip(z0, sp.lo + 1e-3, sp.hi - 1e-3), sigma, {"bounds": [sp.lo, sp.hi], "popsize": 24, "maxfevals": evals, "verbose": -9, "seed": seed})
    best = (-np.inf, z0)
    while not es.stop():
        Z = es.ask()
        s = scorer.similarity("search", scorer.render_crops(Z, gender, age, regions), target)
        loss = [-si + ridge * float(np.sum(np.square(z[:sp.n_shape]))) for si, z in zip(s, Z)]
        es.tell(Z, loss)
        i = int(np.argmax(s))
        if s[i] > best[0]:
            best = (float(s[i]), np.array(Z[i]))
    return best[1]


def photo_mean(scorer, folder):
    path = os.path.join(DATA, "models", "photo_mean.json")
    if os.path.exists(path):
        return {k: np.array(v) for k, v in json.load(open(path)).items()}
    det = Detector(os.path.join(DATA, "model", "face_landmarker.task"))
    E = {k: [] for k in scorer.nets}
    for p in sorted(glob.glob(os.path.join(folder, "*"))):
        rgb = np.asarray(ImageOps.exif_transpose(Image.open(p)).convert("RGB"))
        try:
            d = det.detect(rgb)
        except SystemExit:
            continue
        c = crop(rgb, five_points(d.points))
        for k in scorer.nets:
            E[k].append(scorer.nets[k]([c])[0])
    mu = {k: np.mean(v, 0) for k, v in E.items()}
    json.dump({k: v.tolist() for k, v in mu.items()}, open(path, "w"))
    return mu


def start_point(head, scorer, det, rgb, ga, method):
    d = det.detect(rgb)
    sex, years = ga(rgb, d.points)
    gender, age = (1.0 if sex == "male" else -1.0), float(np.clip((years - 25) / 65, 0, 1))
    regions = (1 / 3,) * 3
    values = np.zeros(len(head.modifiers))
    if method in ("2d", "idfit"):
        corr = Correspondence()
        values = fit(head, photo_frame(d.points, corr), corr, gender=gender, age=age, regions=regions)["values"]
    z = scorer.space.encode(values)
    if method in ("regressor", "regressor+idfit"):
        global _REG
        if "_REG" not in globals():
            from regressor import Regressor
            _REG = Regressor()
        n = scorer.space.n_shape
        z[:n] = np.clip(_REG.predict_shape(rgb, d, gender, age), scorer.space.lo[:n], scorer.space.hi[:n])
    return d, gender, age, regions, z


def lineup(folder, method, evals, out):
    head = Head(os.path.join(DATA, "likeness"))
    scorer = Scorer(head)
    det = Detector(os.path.join(DATA, "model", "face_landmarker.task"))
    ga = GenderAge()
    mu_p = photo_mean(scorer, folder)
    names, T, Z, M = [], {k: [] for k in scorer.nets}, [], []
    for p in sorted(glob.glob(os.path.join(folder, "*"))):
        rgb = np.asarray(ImageOps.exif_transpose(Image.open(p)).convert("RGB"))
        try:
            d, g, a, reg, z0 = start_point(head, scorer, det, rgb, ga, method)
        except SystemExit:
            continue
        tg = photo_targets(scorer, rgb, d.points, mu_p)
        t0 = time.time()
        z = search(scorer, tg["search"], z0, g, a, reg, evals=evals) if method.endswith("idfit") else z0
        names.append(os.path.splitext(os.path.basename(p))[0])
        for k in T:
            T[k].append(tg[k])
        Z.append(z)
        M.append((g, a, reg))
        print(f"{names[-1]}: fitted in {time.time() - t0:.0f} s", file=sys.stderr, flush=True)
    for k in scorer.nets:
        crops = sum((scorer.render_crops([z], *m) for z, m in zip(Z, M)), [])
        R = unit(scorer.nets[k](crops) - scorer.mu_r[k])
        S = R @ np.array(T[k]).T
        ranks = np.array([1 + int((S[i] > S[i, i]).sum()) for i in range(len(S))])
        print(f"{k} network ({'used by the search' if k == 'search' else 'never seen by the search'}). "
              f"Recognised first time: {(ranks == 1).sum()} of {len(names)}. Mean rank {ranks.mean():.1f} of {len(names)}, chance {(len(names) + 1) / 2:.1f}. "
              f"Own-photo similarity {np.diag(S).mean():.3f}, others {(S.sum() - np.trace(S)) / (S.size - len(S)):.3f}.")
        if k == "judge":
            for n, r in zip(names, ranks):
                print(f"  {n:32} rank {r}")
    if out:
        os.makedirs(out, exist_ok=True)
        for n, z, m in zip(names, Z, M):
            v, brow = scorer.space.decode(z)
            img, _ = scorer.gl.render(v, *m, brow=brow)
            Image.fromarray(img).save(os.path.join(out, f"{n}.png"))
        json.dump({n: {"z": z.tolist(), "macro": [m[0], m[1], list(m[2])]} for n, z, m in zip(names, Z, M)}, open(os.path.join(out, "fits.json"), "w"))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("photo", nargs="?")
    ap.add_argument("--lineup")
    ap.add_argument("--method", default="idfit", choices=["average", "2d", "idfit", "regressor", "regressor+idfit"])
    ap.add_argument("--evals", type=int, default=3000)
    ap.add_argument("--out")
    a = ap.parse_args()
    if a.lineup:
        lineup(a.lineup, a.method, a.evals, a.out)


if __name__ == "__main__":
    main()
