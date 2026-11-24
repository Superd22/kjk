"""Measures how the landmark detector responds to known faces, and stores it in correspondence.json.

It renders random faces from the front, with painted brows and lips, and compares each detected
landmark with the true one. For each landmark and each axis it stores the gain (how far the
detected point moves when the true point moves by one) and a weight of 1 / noise².

Usage: python -I calibrate.py [count]
"""
import json, os, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from head import Head
from detect import Detector
from fit import DATA, Correspondence, Problem, normalise, photo_frame
from render import Camera, paint, render

SEED = 1000  # the round trip test uses small seeds, so the two never share a face


def random_face(head, rng):
    v = np.zeros(len(head.modifiers))
    for i, m in enumerate(head.modifiers):
        if m["group"] == "asym" or m["side"] == "R":
            continue
        r = rng.uniform(-0.55, 0.55)
        v[i] = abs(r) if m.get("oneSided") else r
    for i, m in enumerate(head.modifiers):
        if m["side"] == "R":
            v[i] = v[head.by_name[m["pair"]]]
    return v, float(rng.uniform(-1, 1)), float(rng.uniform(0, 0.6)), tuple(rng.dirichlet([3, 3, 3]))


def front(head, pos, colours, perspective=False):
    e = head.eye_centres(pos)
    cam = Camera((e["L"] + e["R"]) / 2 - [0, 0.03, 0], 0.30, 768, distance=0.62 if perspective else None)
    return render(pos, head.tris, [e["L"], e["R"]], cam, colours=colours)[0]


def true_frame(head, pos, corr):
    e = head.eye_centres(pos)
    return normalise(pos[corr.verts, :2], e["R"][:2], e["L"][:2])


def main(count=60):
    path = os.path.join(HERE, "correspondence.json")
    doc = json.load(open(path))
    for o in doc["landmarks"]:
        o.pop("gain", None)
        o.pop("axis_weight", None)
    json.dump(doc, open(path, "w"), indent=1)
    head = Head(os.path.join(DATA, "likeness"))
    det = Detector(os.path.join(DATA, "model", "face_landmarker.task"))
    corr = Correspondence(path)
    colours = paint(head.base, doc["landmarks"])
    average = Problem(head, corr, False).average
    rng = np.random.default_rng(SEED)
    T, D = [], []
    for k in range(count):
        v, g, a, reg = random_face(head, rng)
        pos = head.deform(head.target_weights(v, g, a, reg))
        T.append(true_frame(head, pos, corr) - average)
        D.append(photo_frame(det.detect(front(head, pos, colours)).points, corr) - average)
        print(f"\\r{k + 1}/{count}", end="", file=sys.stderr)
    print(file=sys.stderr)
    T, D = np.array(T), np.array(D)
    # The floor of 0.5 keeps the fit from multiplying a small photo difference more than twice.
    # Renders and photos differ, and a low gain measured on renders may not hold for photos.
    gain = np.clip(np.einsum("nkc,nkc->kc", T, D) / np.maximum(np.einsum("nkc,nkc->kc", T, T), 1e-9), 0.5, 1.2)
    noise = np.maximum(np.sqrt(((D - gain * T) ** 2).mean(0)), 0.006)
    weight = (0.02 / noise) ** 2  # a landmark with 2% noise weighs 1
    for o, gk, wk in zip(doc["landmarks"], gain, weight):
        o["gain"] = [round(float(x), 3) for x in gk]
        o["axis_weight"] = [round(float(x), 3) for x in wk]
    doc["calibration"] = f"{count} random faces, seed {SEED}"
    json.dump(doc, open(path, "w"), indent=1)
    for o in doc["landmarks"]:
        if not o["name"].startswith("mp"):
            print(f'{o["name"]:15} gain {o["gain"][0]:.2f} {o["gain"][1]:.2f}  weight {o["axis_weight"][0]:5.2f} {o["axis_weight"][1]:5.2f}')


if __name__ == "__main__":
    main(*[int(x) for x in sys.argv[1:2]])
