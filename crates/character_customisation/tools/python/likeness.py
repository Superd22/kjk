"""Face recognition scoring with InsightFace's ArcFace, and the lineup test.

The lineup test fits every photo in a folder, renders each fitted head without hair, and asks ArcFace
which photo each render looks most like. A render that picks its own photo counts as recognised.

Usage: python -I likeness.py <photo folder> [--method 2d|...] [--out folder]
"""
import argparse, glob, json, os, sys
import numpy as np
import cv2
import onnxruntime as ort
from PIL import Image, ImageOps

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from fit import DATA

MODELS = os.path.join(DATA, "models", "arcface")
TEMPLATE = np.array([[38.2946, 51.6963], [73.5318, 51.5014], [56.0252, 71.7366], [41.5493, 92.3655], [70.7299, 92.2041]], dtype=np.float32)


def five_points(points):
    """Image-left eye, image-right eye, nose tip, image-left mouth corner, image-right mouth corner, from MediaPipe points."""
    return np.array([points[468], points[473], points[1], points[61], points[291]], dtype=np.float32)


def align(rgb, pts5, size=112):
    m, _ = cv2.estimateAffinePartial2D(pts5, TEMPLATE * size / 112, method=cv2.LMEDS)
    return cv2.warpAffine(rgb, m, (size, size), borderValue=0)


class Embedder:
    def __init__(self):
        self.s = ort.InferenceSession(os.path.join(MODELS, "w600k_r50.onnx"), providers=["CPUExecutionProvider"])
        self.inp = self.s.get_inputs()[0].name

    def __call__(self, rgb, pts5):
        face = align(rgb, pts5).astype(np.float32)
        blob = ((face - 127.5) / 127.5).transpose(2, 0, 1)[None]
        e = self.s.run(None, {self.inp: blob})[0][0]
        return e / np.linalg.norm(e)


class GenderAge:
    def __init__(self):
        self.s = ort.InferenceSession(os.path.join(MODELS, "genderage.onnx"), providers=["CPUExecutionProvider"])
        self.inp = self.s.get_inputs()[0].name

    def __call__(self, rgb, points):
        x0, y0 = points.min(0)
        x1, y1 = points.max(0)
        c, half = np.array([(x0 + x1) / 2, (y0 + y1) / 2]), max(x1 - x0, y1 - y0) * 0.75
        m = np.array([[96 / (2 * half), 0, 48 - c[0] * 96 / (2 * half)], [0, 96 / (2 * half), 48 - c[1] * 96 / (2 * half)]], dtype=np.float32)
        face = cv2.warpAffine(rgb, m, (96, 96)).astype(np.float32)
        out = self.s.run(None, {self.inp: face.transpose(2, 0, 1)[None][:, ::-1].copy()})[0][0]  # the model reads BGR without scaling
        return ("male" if out[1] > out[0] else "female"), float(out[2] * 100)


def mesh_five(head, pos, cam):
    """The same five points on a render, from the mesh itself."""
    corr = {o["name"]: o["vertex"] for o in json.load(open(os.path.join(HERE, "correspondence.json")))["landmarks"]}
    e = head.eye_centres(pos)
    pts3 = np.array([e["R"], e["L"], pos[corr["noseTip"]], pos[corr["mouthCornerR"]], pos[corr["mouthCornerL"]]])
    return cam.project(pts3)[0].astype(np.float32)


def render_for_id(head, values, gender, age, regions, colours, size=512):
    from render import Camera, render
    pos = head.deform(head.target_weights(values, gender, age, regions))
    e = head.eye_centres(pos)
    cam = Camera((e["L"] + e["R"]) / 2 - [0, 0.02, 0], 0.22, size, distance=0.62)
    img, _ = render(pos, head.tris, [e["L"], e["R"]], cam, colours=colours, light=(-0.2, 0.3, 1.0))
    return img, mesh_five(head, pos, cam)


def render_mean(head, emb, colours, path=os.path.join(DATA, "models", "render_mean.npy"), count=200):
    """The mean ArcFace embedding of random renders. Subtracting it, and the photos' mean, removes the gap between the two domains."""
    if os.path.exists(path):
        return np.load(path)
    from calibrate import random_face
    rng = np.random.default_rng(777)
    E = []
    for _ in range(count):
        v, g, a, m = random_face(head, rng)
        img, p5 = render_for_id(head, v, g, a, m, colours)
        E.append(emb(img, p5))
    mu = np.mean(E, 0)
    np.save(path, mu)
    return mu


def centred_similarity(R, P, mu_r, mu_p):
    r = R - mu_r
    p = P - mu_p
    return (r / np.linalg.norm(r, axis=1, keepdims=True)) @ (p / np.linalg.norm(p, axis=1, keepdims=True)).T


def lineup_scores(S):
    """S[i, j] is the similarity of render i to photo j. Returns the rank of the right photo for each render (1 is best), raw and with each photo's column centred."""
    ranks = lambda M: np.array([1 + int((M[i] > M[i, i]).sum()) for i in range(len(M))])
    return ranks(S), ranks(S - S.mean(0, keepdims=True))


def fit_photo(method, head, det, rgb, ga, extra):
    """Returns shape values for one photo with the chosen method."""
    from fit import Correspondence, fit, photo_frame
    d = det.detect(rgb)
    sex, years = ga(rgb, d.points)
    gender, age = (1.0 if sex == "male" else -1.0), float(np.clip((years - 25) / 65, 0, 1))
    regions = (1 / 3,) * 3
    if method == "average":
        return np.zeros(len(head.modifiers)), gender, age, regions, d
    if method == "2d":
        corr = Correspondence()
        res = fit(head, photo_frame(d.points, corr), corr, gender=gender, age=age, regions=regions)
        return res["values"], gender, age, regions, d
    if method == "regressor":
        values = extra["regressor"].predict(rgb, d)
        return values, gender, age, regions, d
    raise ValueError(method)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("folder")
    ap.add_argument("--method", default="2d", choices=["average", "2d", "regressor"])
    ap.add_argument("--out")
    a = ap.parse_args()
    from head import Head
    from detect import Detector
    from render import paint
    head = Head(os.path.join(DATA, "likeness"))
    det = Detector(os.path.join(DATA, "model", "face_landmarker.task"))
    emb, ga = Embedder(), GenderAge()
    colours = paint(head.base, json.load(open(os.path.join(HERE, "correspondence.json")))["landmarks"])
    extra = {}
    if a.method == "regressor":
        from regressor import Regressor
        extra["regressor"] = Regressor()
    names, P, Rr, renders = [], [], [], []
    for path in sorted(glob.glob(os.path.join(a.folder, "*.jpg")) + sorted(glob.glob(os.path.join(a.folder, "*.png")))):
        rgb = np.asarray(ImageOps.exif_transpose(Image.open(path)).convert("RGB"))
        try:
            values, g, age, reg, d = fit_photo(a.method, head, det, rgb, ga, extra)
        except SystemExit:
            continue
        img, p5 = render_for_id(head, values, g, age, reg, colours)
        names.append(os.path.splitext(os.path.basename(path))[0])
        P.append(emb(rgb, five_points(d.points)))
        Rr.append(emb(img, p5))
        renders.append(img)
    P, Rr = np.array(P), np.array(Rr)
    S = centred_similarity(Rr, P, render_mean(head, emb, colours), P.mean(0))
    raw, centred = lineup_scores(S)
    print(f"{len(names)} people. Method {a.method}.")
    print(f"Recognised first time: {(raw == 1).sum()} of {len(names)}. Chance is about 1.")
    print(f"Mean rank of the right photo: {raw.mean():.1f} out of {len(names)}. Chance is {(len(names) + 1) / 2:.1f}. "
          f"With each photo's column centred as well: {centred.mean():.1f}.")
    print(f"Mean similarity of a render to its own photo {np.diag(S).mean():.3f}, to the other photos {(S.sum() - np.trace(S)) / (S.size - len(S)):.3f}.")
    for n, rk in zip(names, raw):
        print(f"  {n:32} rank {rk}")
    if a.out:
        os.makedirs(a.out, exist_ok=True)
        for n, img, rk in zip(names, renders, centred):
            Image.fromarray(img).save(os.path.join(a.out, f"{n}_rank{rk}.png"))


if __name__ == "__main__":
    main()
