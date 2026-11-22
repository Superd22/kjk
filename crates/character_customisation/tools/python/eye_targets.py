"""Adds six eye and socket shape sliders per eye to the exported head, as new targets after the existing ones.

The lid sliders rotate the lid skin around the eyeball's centre, so the lid slides over the eye instead of cutting into it.
Usage: python -I eye_targets.py <head data dir> <out dir>
"""
import base64, json, os, struct, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from head import Head

EYE_R = 0.0123  # the page's eyeball radius
MM = 0.001


def smooth(a, b, t):
    x = np.clip((t - a) / (b - a), 0, 1)
    return x * x * (3 - 2 * x)


def rot(axis, ang):
    """Rotation matrices around one axis, one per angle."""
    c, s = np.cos(ang), np.sin(ang)
    o, z = np.ones_like(ang), np.zeros_like(ang)
    if axis == "x":
        return np.stack([np.stack([o, z, z], -1), np.stack([z, c, -s], -1), np.stack([z, s, c], -1)], -2)
    if axis == "y":
        return np.stack([np.stack([c, z, s], -1), np.stack([z, o, z], -1), np.stack([-s, z, c], -1)], -2)
    return np.stack([np.stack([c, -s, z], -1), np.stack([s, c, z], -1), np.stack([z, z, o], -1)], -2)


def lid_margin(base, centre):
    """The vertices along the edge of the lids, which touch the eyeball."""
    q = base - centre
    r = np.linalg.norm(q, axis=1)
    return np.nonzero((r > 0.008) & (r < 0.0145) & (q[:, 2] > 0))[0]


def stretch_opening(mesh, pos, centre, edge, far, wide, tall, shift=0.0):
    """Makes one eye opening wider and taller. The vertices on the edge of the lids slide over the eyeball, away from the middle of the opening:
    seen from the front, its distance from the middle is scaled, sideways by wide and up and down by tall.
    shift moves the whole opening sideways, in metres. The skin around follows smoothly. edge holds the welded edge vertices, and far marks the welded vertices that must not move."""
    from scipy.sparse.linalg import splu
    x = pos[mesh.first]
    q = x[edge] - centre
    r = np.linalg.norm(q, axis=1)
    # Seen from the front, the edge is scaled about its middle. In depth it stays on the eyeball where the eyeball reaches. An eye opening is
    # wider than the eyeball, so near the corners the lids leave the eyeball, and the edge stays a quarter of its radius in front of the centre.
    mid = (q[:, :2].max(0) + q[:, :2].min(0)) / 2
    xy = mid + (q[:, :2] - mid) * [wide, tall] + [shift, 0]
    z = np.sqrt(np.maximum(r ** 2 - (xy ** 2).sum(1), (0.25 * r) ** 2))
    value = np.zeros((mesh.n, 3))
    value[edge] = centre + np.c_[xy, z] - x[edge]
    fixed = far.copy()
    fixed[edge] = True
    free = ~fixed
    lu = splu(mesh.LtL[free][:, free].tocsc())
    for a in range(3):
        value[free, a] = lu.solve(-mesh.LtL[free][:, fixed] @ value[fixed, a])
    return pos + value[mesh.group]


def eye_deltas(base, centre, side):
    """The six shapes for one eye at full strength, in metres. The frame is mirrored for the right eye so +x is always the outer side."""
    sg = 1.0 if side == "L" else -1.0
    q = (base - centre) * [sg, 1, 1]
    r = np.linalg.norm(q, axis=1)
    d = r - EYE_R  # distance from the eyeball's surface
    region = (r > 0.008) & (r < 0.032) & (q[:, 2] > -0.012)
    margin = region & (r < 0.0145) & (q[:, 2] > 0)
    xi, xo = q[margin, 0].min(), q[margin, 0].max()  # inner and outer corners
    yi, yo = q[margin][q[margin, 0].argmin(), 1], q[margin][q[margin, 0].argmax(), 1]
    t = np.clip((q[:, 0] - xi) / (xo - xi), 0, 1)
    corner_line = yi + (yo - yi) * t
    upper = q[:, 1] > corner_line
    taper = np.sin(np.pi * t) ** 0.7  # the corners stay where they are for the lid heights
    lid = region * smooth(0.010, 0.0, d)
    out = {}

    def apply_rot(axis, ang):
        R = rot(axis, ang)
        return np.einsum("nij,nj->ni", R, q) - q

    out["upper-lid"] = apply_rot("x", -np.radians(14) * lid * taper * upper)  # minus: rotating about +x lifts the front upward when negative
    out["lower-lid"] = apply_rot("x", -np.radians(15) * lid * taper * ~upper)
    theta = np.arctan2(q[:, 0], q[:, 2])
    out["width"] = apply_rot("y", 0.18 * theta * lid)
    out["tilt"] = apply_rot("z", np.radians(9) * lid)
    out["shift"] = apply_rot("y", np.radians(8) * lid)  # slides the opening over the eyeball, toward the temple when positive
    band = region * np.exp(-((d - 0.007) / 0.003) ** 2) * upper * np.sin(np.pi * t) ** 0.5
    out["fold"] = np.stack([0 * band, -2.5 * MM * band, 0.8 * MM * band], 1)
    sock = region * smooth(0.026, 0.014, r) * smooth(0.001, 0.005, d)
    out["socket"] = np.stack([0 * sock, 0 * sock, -3.0 * MM * sock], 1)
    # fix the sign of the lid rotations by checking a margin vertex at the top moves up
    top = np.argmax(np.where(margin & upper, q[:, 1], -1))
    if out["upper-lid"][top, 1] < 0:
        out["upper-lid"] *= -1
    bottom = np.argmin(np.where(margin & ~upper, q[:, 1], 1))
    if out["lower-lid"][bottom, 1] < 0:
        out["lower-lid"] *= -1
    outer = np.argmax(np.where(margin, q[:, 0], -1))
    if out["tilt"][outer, 1] < 0:
        out["tilt"] *= -1
    if out["shift"][outer, 0] < 0:
        out["shift"] *= -1
    return {k: v * [sg, 1, 1] for k, v in out.items()}


SLIDERS = [  # key, name, label; the positive direction is the second word
    ("upper-lid", "eye-upper-lid", "Upper lid height (down / up)"),
    ("lower-lid", "eye-lower-lid", "Lower lid height (down / up)"),
    ("width", "eye-width", "Eye width (narrow / wide)"),
    ("tilt", "eye-tilt", "Tilt (outer corner down / up)"),
    ("fold", "eye-lid-fold", "Lid fold (deep / hooded)"),
    ("socket", "eye-socket", "Socket depth (shallow / deep)"),
    ("shift", "eye-shift", "Opening shift (nose / temple)"),
]


def main(src, out):
    h = Head(src)
    meta = json.load(open(os.path.join(src, "head.json")))
    blob = bytearray(open(os.path.join(src, "targets.bin"), "rb").read())
    e = h.eye_centres(h.base)
    D = {side: eye_deltas(h.base, e[side], side) for side in "LR"}
    # Sliders added later go after both eyes' earlier ones, so older share codes keep their order.
    for batch in (SLIDERS[:6], SLIDERS[6:]):
        for side in "LR":
            for key, name, label in batch:
                ids = []
                for sign, suffix in ((1, "pos"), (-1, "neg")):
                    d = D[side][key] * sign
                    idx = np.nonzero(np.linalg.norm(d, axis=1) > 1e-6)[0]
                    meta["targets"].append({"name": f"{name}-{side}-{suffix}", "offset": len(blob), "count": int(len(idx))})
                    blob += struct.pack("<I", len(idx)) + struct.pack(f"<{len(idx)}H", *idx.tolist())
                    if len(idx) % 2:
                        blob += b"\0\0"
                    q = np.clip(np.round(d[idx] * 50000), -32767, 32767).astype("<i2")
                    blob += q.tobytes()
                    while len(blob) % 4:
                        blob += b"\0"
                    ids.append(len(meta["targets"]) - 1)
                meta["modifiers"].append({"name": f"{name}-{side}", "label": label, "group": "eyes", "side": side,
                                          "pair": f"{name}-{'R' if side == 'L' else 'L'}", "neg": ids[1], "pos": ids[0]})
    os.makedirs(out, exist_ok=True)
    json.dump(meta, open(os.path.join(out, "head.json"), "w"), separators=(",", ":"))
    open(os.path.join(out, "targets.bin"), "wb").write(blob)
    open(os.path.join(out, "targets.b64.txt"), "w").write(base64.b64encode(bytes(blob)).decode())
    print(f"{len(meta['modifiers'])} modifiers, {len(meta['targets'])} targets")


if __name__ == "__main__":
    main(*sys.argv[1:3])
