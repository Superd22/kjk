"""Fits the face sliders to a 3D face that 3DDFA_V2 reconstructs from one photo.

The fit compares the photo's face with 3DDFA_V2's reconstruction of the average head, so the two models'
differences cancel out. Only the difference between this face and the average face reaches the sliders.
"""
import json, os, sys
import numpy as np
from scipy.spatial import cKDTree

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from fit import Problem
from render import Camera, paint, render

REGISTRATION = os.path.join(HERE, "registration3d.json")
EYE_R, EYE_L = range(36, 42), range(42, 48)  # 68-point eye contours, image-left eye first


class Registration:
    def __init__(self, path=REGISTRATION):
        r = json.load(open(path))
        self.verts = np.array(r["mh_verts"])
        self.bfm_verts = np.array(r["bfm_verts"])
        self.avg_alpha = np.array(r["avg_alpha"], dtype=np.float32)
        self.gain = np.ones((len(self.verts), 3))
        self.weights = np.ones((len(self.verts), 3))
        self.names = [f"v{v}" for v in self.verts]


def bfm_frame(recon, ident):
    kp = recon.bfm.keypoints[::3] // 3
    er, el = ident[kp[list(EYE_R)]].mean(0), ident[kp[list(EYE_L)]].mean(0)
    return (ident - (er + el) / 2) / np.linalg.norm(el - er)


def build(head, det, recon, corr_landmarks, spacing=0.004):
    """Registers the average head's front vertices to 3DDFA_V2's reconstruction of a render of it."""
    pos = head.deform(head.target_weights(np.zeros(len(head.modifiers))))
    e = head.eye_centres(pos)
    cam = Camera((e["L"] + e["R"]) / 2 - [0, 0.03, 0], 0.30, 768)
    img, zbuf = render(pos, head.tris, [e["L"], e["R"]], cam, colours=paint(head.base, corr_landmarks))
    pts = det.detect(img).points
    rc = recon.run(img, pts)
    xy, depth = cam.project(pos)
    px = np.clip(xy.astype(int), 0, cam.size - 1)
    visible = depth >= zbuf[px[:, 1], px[:, 0]] - 0.0015
    d, j = cKDTree(rc["image"][:, :2]).query(xy)
    keep = np.nonzero(visible & (d < 1.0) & (head.base[:, 2] > 0.06))[0]
    # One vertex per cell of a grid on the front of the face, so the solve is not dominated by dense areas.
    cells = {}
    for i in keep:
        cells.setdefault(tuple(np.floor(head.base[i, :2] / spacing).astype(int)), i)
    mh = np.array(sorted(cells.values()))
    json.dump({"note": "MakeHuman vertices and the 3DDFA_V2 vertices they match on the average head.",
               "mh_verts": mh.tolist(), "bfm_verts": j[mh].tolist(), "avg_alpha": rc["alpha"].tolist()}, open(REGISTRATION, "w"))
    return len(mh)


def targets(prob, recon, reg, rc):
    avg_ident = (recon.bfm.u + recon.bfm.w_shp @ reg.avg_alpha[:, None]).reshape(3, -1, order="F").T
    delta = bfm_frame(recon, rc["identity"])[reg.bfm_verts] - bfm_frame(recon, avg_ident)[reg.bfm_verts]
    return prob.average + delta


def fit3d(head, recon, reg, rc, gender=0.0, age=0.0, regions=(1 / 3,) * 3, ridge=1.0, asym=False):
    prob = Problem(head, reg, asym)
    target = targets(prob, recon, reg, rc)
    mw = head.macro_weights(gender, age, regions)
    x, r, cost = prob.solve(mw, target, ridge, iters=3)
    return {"values": prob.values(x), "gender": gender, "age": age, "regions": regions, "residual": r, "prob": prob}
