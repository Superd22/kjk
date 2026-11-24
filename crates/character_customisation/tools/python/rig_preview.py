"""Poses the rigged head with KOTOR face clips and renders a contact sheet, as the Likeness Lab page does.

Usage: python -I rig_preview.py <head data dir> <rig.json> <bundle.json> <out.png> [frames]
frames is a list like "talk@3,tlknorm@0.6,tlklaugh@1.2". The default sheet shows the 16 talk shapes and a few clip frames.
"""
import json, math, os, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from head import Head
from glrender import GLHead
from build_rig import qmat

SHAPES = ["rest", "IH IY", "EH ER EY", "AA AE AH", "OW OY", "UH UW W", "D DH S Y Z", "CH JH SH ZH", "F V", "G HH K NG", "T TH", "B M P", "L N", "R", "AW AY", "AO"]


def sample(keys, t, n):
    if not keys:
        return None
    if t <= keys[0][0]:
        return np.array(keys[0][1:1 + n])
    if t >= keys[-1][0]:
        return np.array(keys[-1][1:1 + n])
    for a, b in zip(keys, keys[1:]):
        if b[0] > t:
            f = (t - a[0]) / max(b[0] - a[0], 1e-6)
            va, vb = np.array(a[1:1 + n]), np.array(b[1:1 + n])
            if n == 4 and va @ vb < 0:
                vb = -vb
            v = va + (vb - va) * f
            return v / np.linalg.norm(v) if n == 4 else v


def qmul(a, b):
    ax, ay, az, aw = a
    bx, by, bz, bw = b
    return np.array([aw * bx + ax * bw + ay * bz - az * by, aw * by - ax * bz + ay * bw + az * bx, aw * bz + ax * by - ay * bx + az * bw, aw * bw - ax * bx - ay * by - az * bz])


def qpow(q, k):
    q = q if q[3] >= 0 else -q
    ang = 2 * math.acos(min(1, q[3]))
    s = math.sin(ang / 2)
    if s < 1e-8:
        return np.array([0, 0, 0, 1.0])
    axis = q[:3] / s
    return np.r_[axis * math.sin(ang * k / 2), math.cos(ang * k / 2)]


class Rig:
    def __init__(self, rig):
        self.r = rig
        self.bones = rig["bones"]
        self.index = {b["name"]: i for i, b in enumerate(self.bones)}
        self.A = np.array(rig["toOur"], float)
        self.ks = np.array(rig["keyScale"])
        self.rest = self.world({})

    def world(self, pose):
        """pose maps a bone name to (offset or None, rotation or None), in KOTOR's axes."""
        out = []
        for b in self.bones:
            off, rot = pose.get(b["name"], (None, None))
            rq = np.array(b["rot"]) if rot is None else np.array(rot)
            if rot is not None and b["name"] in self.r["lidScale"]:
                rest = np.array(b["rot"])
                d = qmul(rq, rest * [-1, -1, -1, 1])
                rq = qmul(qpow(d, self.r["lidScale"][b["name"]]), rest)
            p = np.array(b["local"]) + (0 if off is None else self.ks * np.array(off))
            M = np.eye(4)
            M[:3, :3] = qmat(rq)
            M[:3, 3] = p
            if b["parent"]:
                M = out[self.index[b["parent"]]] @ M
            out.append(M)
        return out

    def skin_mats(self, pose):
        W = self.world(pose)
        S = np.eye(4)
        S[:3, :3] = self.A
        return [S @ w @ np.linalg.inv(w0) @ S for w, w0 in zip(W, self.rest)]

    def skin(self, pos, pose):
        mats = self.skin_mats(pose)
        h = np.c_[pos, np.ones(len(pos))]
        out = (h @ mats[self.index["head_g"]].T)[:, :3]  # a vertex the skin list leaves out follows head_g alone
        for row in self.r["skin"]:
            v = row[0]
            acc, ws = np.zeros(4), 0.0
            for k in range(1, len(row), 2):
                acc += row[k + 1] * (mats[row[k]] @ h[v])
                ws += row[k + 1]
            out[v] = acc[:3] / ws
        return out, mats


def clip_pose(clip, t):
    pose = {}
    for name, tr in clip["tracks"].items():
        pose[name] = (sample(tr["p"], t, 3), sample(tr["r"], t, 4))
    return pose


def main(head_dir, rig_path, bundle_path, out, frames=None):
    from PIL import Image, ImageDraw
    h = Head(head_dir)
    rig = Rig(json.load(open(rig_path)))
    b = json.load(open(bundle_path))
    g = GLHead(h, size=300, dist=0.36)
    base = g.positions(g.weights_vector(np.zeros(len(h.modifiers)), 0.6, 0.2, (1 / 3,) * 3)).astype(np.float64)
    margin_axes = {}
    orig_axes, orig_off = g.eyeball_axes, g.opening_offset
    centres0 = h.eye_centres(base)
    for s in "LR":
        margin_axes[s] = orig_axes(base, s, centres0[s])
    if frames:
        items = [f.split("@") for f in frames.split(",")]
    else:
        items = [["talk", str(s / 30)] for s in range(16)] + [["tlknorm", "0.6"], ["tlklaugh", "0.9"], ["tlklaugh", "2.4"], ["tlksad", "1.5"], ["tlksad", "5.0"], ["tlkplead", "1.5"], ["tlkforce", "2.0"], ["tsl_rolleyes", "1.0"]]
    tiles = []
    for name, at in items:
        t = float(at)
        pose = clip_pose(b["clips"][name], t)
        if name == "talk":
            # talk keys only the mouth; the rest of the face is at rest
            pass
        pos, mats = rig.skin(base, pose)
        gaze = {}
        for s, eb in (("L", "eyela"), ("R", "eyera")):
            M = mats[rig.index[eb]]
            fwd = M[:3, :3] @ np.array([0, 0, 1.0])
            gaze[s] = fwd[:2] * 0.0123
        g.eyeball_axes = lambda p, s, c: margin_axes[s]
        g.opening_offset = lambda p, s, c, _o=orig_off: _o(base, s, centres0[s]) + gaze[s][0]
        img, _ = g.render(None, pos=pos.astype(np.float32))
        im = Image.fromarray(img)
        d = ImageDraw.Draw(im)
        label = f"talk {int(round(t * 30))} {SHAPES[int(round(t * 30))]}" if name == "talk" else f"{name} {t:g}s"
        d.rectangle([0, 0, 300, 18], fill=(0, 0, 0))
        d.text((4, 3), label, fill=(255, 255, 255))
        tiles.append(im)
    cols = 6
    rows = (len(tiles) + cols - 1) // cols
    sheet = Image.new("RGB", (cols * 300, rows * 300), (16, 18, 22))
    for i, im in enumerate(tiles):
        sheet.paste(im, ((i % cols) * 300, (i // cols) * 300))
    sheet.save(out)
    print("wrote", out)


if __name__ == "__main__":
    main(*sys.argv[1:6])
