"""Renders random faces and stores what the detectors see, with the true slider values, to train the regressor.

Usage: python -I gen_data.py <out.npz> <count> <seed> [workers]
"""
import json, os, sys
import numpy as np
from multiprocessing import Pool

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

_state = {}


def _init():
    from head import Head
    from detect import Detector
    from recon3d import Recon
    from fit import DATA
    from render import paint
    _state["head"] = Head(os.path.join(DATA, "likeness"))
    _state["det"] = Detector(os.path.join(DATA, "model", "face_landmarker.task"))
    _state["recon"] = Recon()
    _state["colours"] = paint(_state["head"].base, json.load(open(os.path.join(HERE, "correspondence.json")))["landmarks"])


def features(det, recon, img):
    """The normalised MediaPipe points and 3DDFA_V2's identity numbers for one image."""
    from detect import IRIS_L, IRIS_R
    from fit import normalise
    d = det.detect(img)
    p = d.points * [1, -1]
    lm = normalise(p[:468], p[IRIS_R], p[IRIS_L]).ravel()
    rc = recon.run(img, d.points)
    return np.concatenate([lm, rc["alpha"] / 1e5]).astype(np.float32), d


def _one(seed):
    from calibrate import front, random_face
    h = _state["head"]
    rng = np.random.default_rng(seed)
    v, g, a, mix = random_face(h, rng)
    img = front(h, h.deform(h.target_weights(v, g, a, mix)), _state["colours"], perspective=bool(rng.random() < 0.5))
    try:
        f, _ = features(_state["det"], _state["recon"], img)
    except SystemExit:
        return None
    return f, v.astype(np.float32), np.array([g, a, *mix], dtype=np.float32)


def main(out, count, seed, workers=8):
    with Pool(int(workers), initializer=_init) as pool:
        rows = [r for r in pool.imap(_one, range(int(seed), int(seed) + int(count)), chunksize=8) if r is not None]
    np.savez_compressed(out, X=np.stack([r[0] for r in rows]), V=np.stack([r[1] for r in rows]), M=np.stack([r[2] for r in rows]))
    print(f"{len(rows)} faces written to {out}")


if __name__ == "__main__":
    main(*sys.argv[1:])
