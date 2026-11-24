"""Checks the brow fit on renders: draws the head with known brow numbers, fits as if the render were a photo,
and prints how far each number comes back.

Usage: python -I roundtrip_brows.py <share code file> <out dir> [--yaw Y --pitch P --distance D]
"""
import argparse, os, sys
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from head import Head, read_share_code_full
from fit import DATA
from detect import Detector
from glrender import GLHead, BROW_KEYS, BROW_DEFAULT
import outline_fit as of

CASES = [
    dict(browInner=0.0, browOuter=0.0, browArchPos=0.0, browArch=0.0, browThickIn=0.5, browThickOut=0.4, browLength=0.0, browSpacing=0.0, lift=0.0),
    dict(browInner=-0.4, browOuter=-0.5, browArchPos=0.2, browArch=-0.5, browThickIn=0.6, browThickOut=0.3, browLength=0.3, browSpacing=-0.3, lift=-0.4),
    dict(browInner=0.5, browOuter=0.3, browArchPos=-0.3, browArch=0.6, browThickIn=0.3, browThickOut=0.2, browLength=-0.4, browSpacing=0.4, lift=0.3),
    dict(browInner=-0.2, browOuter=0.6, browArchPos=0.5, browArch=0.3, browThickIn=0.8, browThickOut=0.7, browLength=0.6, browSpacing=0.1, lift=0.0),
    dict(browInner=0.3, browOuter=-0.8, browArchPos=-0.5, browArch=-0.2, browThickIn=0.2, browThickOut=0.1, browLength=-0.2, browSpacing=-0.5, lift=0.5),
]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("code_file")
    ap.add_argument("out_dir")
    ap.add_argument("--distance", type=float, default=2.0)
    ap.add_argument("--pitch", type=float, default=1.0)
    ap.add_argument("--yaw", type=float, default=3.0)
    a = ap.parse_args()
    os.makedirs(a.out_dir, exist_ok=True)
    h = Head(os.path.join(DATA, "likeness"))
    v, g, age, reg, surf = read_share_code_full(open(a.code_file).read(), len(h.modifiers))
    gl = GLHead(h, of.SIZE, dist=a.distance)
    det = Detector(os.path.join(DATA, "model", "face_landmarker.task"))
    pose = dict(yaw=a.yaw, pitch=a.pitch)
    lift = [i for i, m in enumerate(h.modifiers) if m["group"] == "eyebrows" and m["label"].startswith("Position (down / up)")]
    # every fit starts from the page's default brows and the brow position slider at 0
    start = dict(surf, **{k: BROW_DEFAULT[k] for k in BROW_KEYS})
    v0 = v.copy()
    v0[lift] = 0
    errs, shape = [], []
    for n, case in enumerate(CASES):
        vt = v.copy()
        vt[lift] = case["lift"]
        brow = dict({k: case[k] for k in BROW_KEYS}, browColor=surf["browColor"], browDensity=0.85)
        img, _ = gl.render(vt, g, age, reg, brow=brow, **pose)
        path = os.path.join(a.out_dir, f"case{n}.png")
        Image.fromarray(img).save(path)
        r = of.fit_photo(gl, h, v0, g, age, reg, start, img, pose, det, log=lambda t: None)
        m = gl.brow_mask(r["values"], g, age, reg, r["brow"], **pose)
        Image.fromarray(of.draw_debug(r["warped"], r["sides"], r["mask"], m, gl.render(r["values"], g, age, reg, brow=dict(r["brow"], browDensity=0.85), **pose)[0])).save(os.path.join(a.out_dir, f"case{n}_fit.png"))
        # how far the fitted stroke's outline lies from the true one, column by column, in pupil distances
        mt = gl.brow_mask(vt, g, age, reg, dict(brow, browDensity=1.0), **pose) > 0.5
        mf = m > 0.5
        cols = np.flatnonzero(mt.any(0) & mf.any(0))
        edge = lambda mm: (np.argmax(mm[:, cols], 0), of.SIZE - 1 - np.argmax(mm[::-1, cols], 0))
        (tt, tb), (ft, fb) = edge(mt), edge(mf)
        ipd = np.linalg.norm(r["pupils"][1] - r["pupils"][0])
        outline = (np.abs(tt - ft).mean() + np.abs(tb - fb).mean()) / 2 / ipd
        ends = np.abs(np.flatnonzero(mt.any(0))[[0, -1]] - np.flatnonzero(mf.any(0))[[0, -1]]).mean() / ipd
        iou = (mt & mf).sum() / max((mt | mf).sum(), 1)
        shape.append((outline, ends, iou))
        truth = np.array([case[k] for k in BROW_KEYS + ["lift"]])
        e = r["z"] - truth
        errs.append(e)
        print(f"case {n}: shape cost {r['cost']:.2f}, edges off by {outline:.3f} and outer ends by {ends:.3f} pupil distances on average, overlap {100 * iou:.0f}%")
        for k, t, z0, z in zip(BROW_KEYS + ["lift"], truth, r["z0"], r["z"]):
            print(f"  {k:14} true {t:+.2f}  start {z0:+.2f}  fitted {z:+.2f}  error {z - t:+.2f}")
    shape = np.array(shape)
    print(f"mean over cases: edges off by {shape[:, 0].mean():.3f}, ends by {shape[:, 1].mean():.3f} pupil distances, overlap {100 * shape[:, 2].mean():.0f}%")
    errs = np.abs(np.array(errs))
    print("mean absolute error per number:")
    for k, e in zip(BROW_KEYS + ["lift"], errs.mean(0)):
        print(f"  {k:14} {e:.2f}")


if __name__ == "__main__":
    main()
