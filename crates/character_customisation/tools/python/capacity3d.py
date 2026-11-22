"""Measures how much of a real face's 3D shape the sliders can reach, with no photo fitting in the way.

3DDFA_V2 reconstructs each photo as a 3D face. The sliders are then solved against that 3D face directly,
vertex against vertex, with almost no ridge penalty. What is left over is shape the sliders cannot make.

Usage: python -I capacity3d.py <photo or folder>...
"""
import os, sys
import numpy as np
from PIL import Image, ImageOps

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from head import Head
from detect import Detector
from recon3d import Recon
from fit import DATA, Problem
import fit3d

IPD_MM = 63.0
ZONES = ("forehead and brows", "eyes", "nose", "cheeks", "mouth", "jaw and chin")


def zone_of(p):
    x, y = abs(p[0]), p[1]
    if y > 0.22:
        return 0
    if y > -0.2 and x > 0.2:
        return 1
    if x < 0.28 and y > -0.78:
        return 2
    if y > -0.72:
        return 3
    if y > -1.2 and x < 0.5:
        return 4
    return 5


def main():
    paths = []
    for a in sys.argv[1:]:
        paths += sorted(os.path.join(a, f) for f in os.listdir(a)) if os.path.isdir(a) else [a]
    head = Head(os.path.join(DATA, "likeness"))
    det = Detector(os.path.join(DATA, "model", "face_landmarker.task"))
    recon = Recon()
    reg = fit3d.Registration()
    probs = {"symmetric": Problem(head, reg, False), "with asymmetry": Problem(head, reg, True)}
    mw = head.macro_weights(0, 0, (1 / 3,) * 3)
    zones = np.array([zone_of(p) for p in probs["symmetric"].average])
    print(f"{len(reg.verts)} vertices. Vertices per zone: " + ", ".join(f"{z} {np.sum(zones == i)}" for i, z in enumerate(ZONES)))
    rows = []
    for path in paths:
        try:
            img = np.asarray(ImageOps.exif_transpose(Image.open(path)).convert("RGB"))
            rc = recon.run(img, det.detect(img).points)
        except Exception as e:
            print(f"{os.path.basename(path)}: skipped ({e})")
            continue
        row = {}
        for name, prob in probs.items():
            target = fit3d.targets(prob, recon, reg, rc)
            before = np.linalg.norm(target - prob.average, axis=1) * IPD_MM
            x, r, _ = prob.solve(mw, target, 1e-4, iters=4)
            after = np.linalg.norm(r, axis=1) * IPD_MM
            row[name] = (before, after, x)
        b, a, x = row["symmetric"]
        a2 = row["with asymmetry"][1]
        rows.append((b, a, a2))
        print(f"{os.path.basename(path)[:28]:28} average face {b.mean():4.2f} mm off, sliders {a.mean():4.2f} mm, with asymmetry {a2.mean():4.2f} mm. "
              f"{np.sum(x > 0.98)} of {len(x)} slider sides at the limit.")
    if not rows:
        return
    B, A, A2 = (np.array(c) for c in zip(*rows))
    print(f"\nOver {len(rows)} faces, mean distance per vertex in mm (eye distance taken as {IPD_MM:.0f} mm):")
    print(f"{'zone':20} {'average face':>12} {'sliders':>8} {'+asym':>6} {'reached':>8}")
    for i, z in enumerate(ZONES):
        m = zones == i
        print(f"{z:20} {B[:, m].mean():12.2f} {A[:, m].mean():8.2f} {A2[:, m].mean():6.2f} {100 * (1 - A[:, m].mean() / B[:, m].mean()):7.0f}%")
    print(f"{'whole face':20} {B.mean():12.2f} {A.mean():8.2f} {A2.mean():6.2f} {100 * (1 - A.mean() / B.mean()):7.0f}%")
    print(f"Worst single vertex after the fit, median over faces: {np.median(A.max(1)):.2f} mm.")


if __name__ == "__main__":
    main()
