"""Round trip test for the 3D fit. Same random faces and error measure as test_roundtrip.py.

Usage: python -I test3d.py [count] [--ridge R] [--seed S] [--out folder]
"""
import argparse, json, os, sys
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from head import Head
from detect import Detector
from recon3d import Recon
from fit import DATA
from calibrate import front, random_face
from render import paint
from test_roundtrip import face_error
import fit3d


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("count", type=int, nargs="?", default=12)
    ap.add_argument("--ridge", type=float, default=1.0)
    ap.add_argument("--seed", type=int, default=2)
    ap.add_argument("--out")
    a = ap.parse_args()
    head = Head(os.path.join(DATA, "likeness"))
    det = Detector(os.path.join(DATA, "model", "face_landmarker.task"))
    recon = Recon()
    reg = fit3d.Registration()
    colours = paint(head.base, json.load(open(os.path.join(HERE, "correspondence.json")))["landmarks"])
    avg = head.deform(head.target_weights(np.zeros(len(head.modifiers))))
    rng = np.random.default_rng(a.seed)
    rows = []
    for k in range(a.count):
        v, g, ag, reg_mix = random_face(head, rng)
        truth = head.deform(head.target_weights(v, g, ag, reg_mix))
        img = front(head, truth, colours)
        rc = recon.run(img, det.detect(img).points)
        res = fit3d.fit3d(head, recon, reg, rc, g, ag, reg_mix, ridge=a.ridge)
        fitted = head.deform(head.target_weights(res["values"], g, ag, reg_mix))
        b = face_error(head, avg, truth)
        f = face_error(head, fitted, truth)
        rows.append((*b, *f))
        print(f"face {k + 1:2}. Photo plane {b[0]:4.1f}% → {f[0]:4.1f}%. Depth {b[1]:4.1f}% → {f[1]:4.1f}%.")
        if a.out:
            os.makedirs(a.out, exist_ok=True)
            Image.fromarray(np.concatenate([img, front(head, fitted, colours)], 1)).resize((1024, 512)).save(os.path.join(a.out, f"face{k + 1}.png"))
    r = np.array(rows)
    print(f"\nOver {len(r)} faces, the 3D fit removes {100 * (1 - r[:, 2].mean() / r[:, 0].mean()):.0f}% of the difference in the photo plane "
          f"and {100 * (1 - r[:, 3].mean() / r[:, 1].mean()):.0f}% in depth.")


if __name__ == "__main__":
    main()
