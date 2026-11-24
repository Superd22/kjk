"""Round trip test. Builds random faces, renders them from the front, fits the renders and compares.

Usage: python -I test_roundtrip.py [count] [--out folder] [--perspective] [--oracle] [--true-macro]
"""
import argparse, json, os, sys
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from head import Head, share_code, read_share_code
from detect import Detector
from fit import DATA, RIDGE, Correspondence, fit, photo_frame
from calibrate import front, random_face, true_frame
from render import paint


def face_error(head, a, b):
    """Mean vertex distance over the front of the face, with each head scaled to an eye distance of 1, in percent.

    It is split into the photo plane and depth. A photo cannot show the absolute size of a head, so this ignores it.
    """
    lm = head.landmarks
    sel = (head.base[:, 2] > 0.09) & (head.base[:, 1] > lm["chin"][1] - 0.005) & (head.base[:, 1] < lm["forehead"][1]) & (np.abs(head.base[:, 0]) < 0.065)
    def centred(p):
        q = head.eye_centres(p)
        return (p[sel] - (q["L"] + q["R"]) / 2) / np.linalg.norm(q["L"] - q["R"])
    d = centred(a) - centred(b)
    return 100 * np.linalg.norm(d[:, :2], axis=1).mean(), 100 * np.abs(d[:, 2]).mean()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("count", type=int, nargs="?", default=12)
    ap.add_argument("--out", help="save each face beside its fit in this folder")
    ap.add_argument("--perspective", action="store_true", help="render with the page's camera distance instead of orthographic")
    ap.add_argument("--ridge", type=float, default=RIDGE)
    ap.add_argument("--seed", type=int, default=2)
    ap.add_argument("--oracle", action="store_true", help="give the fit the exact mesh landmarks instead of the detector's")
    ap.add_argument("--true-macro", action="store_true", help="give the fit the true gender and age")
    a = ap.parse_args()
    head = Head(os.path.join(DATA, "likeness"))
    det = Detector(os.path.join(DATA, "model", "face_landmarker.task"))
    corr = Correspondence()
    if a.oracle:
        corr.gain[:] = 1
        corr.weights[:] = 1
    colours = paint(head.base, json.load(open(os.path.join(HERE, "correspondence.json")))["landmarks"])
    rng = np.random.default_rng(a.seed)
    avg = head.deform(head.target_weights(np.zeros(len(head.modifiers))))
    rows = []
    for k in range(a.count + 1):
        if k == 0:
            v, g, ag, reg = np.zeros(len(head.modifiers)), 0.0, 0.0, (1 / 3,) * 3
        else:
            v, g, ag, reg = random_face(head, rng)
        truth = head.deform(head.target_weights(v, g, ag, reg))
        img = front(head, truth, colours, a.perspective)
        target = true_frame(head, truth, corr) if a.oracle else photo_frame(det.detect(img).points, corr)
        res = fit(head, target, corr, ridge=a.ridge, gender=g if a.true_macro else None, age=ag if a.true_macro else None)
        fv, fg, fa, freg = read_share_code(share_code(res["values"], res["gender"], res["age"], res["regions"]), len(head.modifiers))
        fitted = head.deform(head.target_weights(fv, fg, fa, freg))
        base_xy, base_z = face_error(head, avg, truth)
        fit_xy, fit_z = face_error(head, fitted, truth)
        rows.append((g, fg, base_xy, fit_xy, base_z, fit_z))
        label = "average face" if k == 0 else f"face {k}"
        print(f"{label:12} gender {g:+.2f}, fitted {fg:+.2f}. Face surface in the photo plane {base_xy:4.1f}% → {fit_xy:4.1f}%. Depth {base_z:4.1f}% → {fit_z:4.1f}%.")
        if a.out:
            os.makedirs(a.out, exist_ok=True)
            Image.fromarray(np.concatenate([img, front(head, fitted, colours, a.perspective)], 1)).resize((1024, 512)).save(os.path.join(a.out, f"face{k}.png"))
    r = np.array(rows[1:])
    if len(r):
        print(f"\nOver {len(r)} random faces, the fit removes {100 * (1 - r[:, 3].mean() / r[:, 2].mean()):.0f}% of the difference from the average face "
              f"in the photo plane and {100 * (1 - r[:, 5].mean() / r[:, 4].mean()):.0f}% in depth.")
        print(f"The mean gender error is {np.abs(r[:, 0] - r[:, 1]).mean():.2f} on the scale from −1 to 1.")


if __name__ == "__main__":
    main()
