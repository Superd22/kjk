"""A small software renderer for front views of the head, used to test the fit without a real person."""
import numpy as np

SKIN = np.array([0.86, 0.68, 0.58])


def smooth_normals(pos, tris):
    key = np.round(pos, 4)
    _, group = np.unique(key, axis=0, return_inverse=True)
    group = group.ravel()
    a, b, c = pos[tris[:, 0]], pos[tris[:, 1]], pos[tris[:, 2]]
    fn = np.cross(b - a, c - a)
    acc = np.zeros((group.max() + 1, 3))
    for k in range(3):
        np.add.at(acc, group[tris[:, k]], fn)
    n = acc[group]
    return n / np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-12)


def eye_mesh(centre, radius, rings=24, segs=32):
    """A sphere with an iris and a pupil facing +z."""
    th = np.linspace(0, np.pi, rings + 1)
    ph = np.linspace(0, 2 * np.pi, segs + 1)
    T, P = np.meshgrid(th, ph, indexing="ij")
    nrm = np.stack([np.sin(T) * np.cos(P), np.sin(T) * np.sin(P), np.cos(T)], -1).reshape(-1, 3)
    pos = centre + radius * nrm
    ang = np.degrees(np.arccos(np.clip(nrm[:, 2], -1, 1)))
    col = np.where(ang[:, None] < 12, [0.05, 0.04, 0.04], np.where(ang[:, None] < 30, [0.33, 0.22, 0.14], [0.92, 0.9, 0.88]))
    tris = []
    w = segs + 1
    for i in range(rings):
        for j in range(segs):
            p = i * w + j
            tris += [[p, p + w, p + 1], [p + 1, p + w, p + w + 1]]
    return pos, nrm, col, np.array(tris)


class Camera:
    """Orthographic when distance is None, otherwise a pinhole at that distance in front of the centre."""

    def __init__(self, centre, width_m, size=768, distance=None):
        self.c = np.asarray(centre, dtype=np.float64)
        self.size = size
        self.ppm = size / width_m
        self.distance = distance

    def project(self, p):
        rel = p - self.c
        if self.distance is None:
            s = np.full(len(rel), self.ppm)
            depth = rel[:, 2]
        else:
            dz = self.distance - rel[:, 2]
            s = self.ppm * self.distance / dz
            depth = -dz
        x = self.size / 2 + rel[:, 0] * s
        y = self.size / 2 - rel[:, 1] * s
        return np.stack([x, y], 1), depth


def render(head_pos, tris, eyes, cam, colours=None, light=(-0.35, 0.45, 1.0)):
    """Returns an RGB uint8 image. eyes is a list of eyeball centres."""
    nrm = smooth_normals(head_pos, tris)
    col = np.tile(SKIN, (len(head_pos), 1)) if colours is None else colours
    P, N, C, T = [head_pos], [nrm], [col], [tris]
    off = len(head_pos)
    for e in eyes:
        ep, en, ec, et = eye_mesh(e, 0.0123)
        P.append(ep); N.append(en); C.append(ec); T.append(et + off)
        off += len(ep)
    pos, nrm, col, tris = np.concatenate(P), np.concatenate(N), np.concatenate(C), np.concatenate(T)
    xy, depth = cam.project(pos)
    L = np.asarray(light, dtype=np.float64)
    L /= np.linalg.norm(L)
    shade = 0.32 + 0.75 * np.clip(nrm @ L, 0, 1)
    vcol = np.clip(col * shade[:, None], 0, 1)
    S = cam.size
    zbuf = np.full((S, S), -np.inf)
    img = np.tile(np.array([0.55, 0.57, 0.6]), (S, S, 1))
    for t in tris:
        p = xy[t]
        x0, y0 = np.floor(p.min(0)).astype(int)
        x1, y1 = np.ceil(p.max(0)).astype(int)
        x0, y0, x1, y1 = max(x0, 0), max(y0, 0), min(x1, S - 1), min(y1, S - 1)
        if x0 > x1 or y0 > y1:
            continue
        (ax, ay), (bx, by), (cx, cy) = p
        den = (by - cy) * (ax - cx) + (cx - bx) * (ay - cy)
        if abs(den) < 1e-12:
            continue
        gx, gy = np.meshgrid(np.arange(x0, x1 + 1) + 0.5, np.arange(y0, y1 + 1) + 0.5)
        w0 = ((by - cy) * (gx - cx) + (cx - bx) * (gy - cy)) / den
        w1 = ((cy - ay) * (gx - cx) + (ax - cx) * (gy - cy)) / den
        w2 = 1 - w0 - w1
        inside = (w0 >= 0) & (w1 >= 0) & (w2 >= 0)
        if not inside.any():
            continue
        z = w0 * depth[t[0]] + w1 * depth[t[1]] + w2 * depth[t[2]]
        sub = zbuf[y0:y1 + 1, x0:x1 + 1]
        win = inside & (z > sub)
        if not win.any():
            continue
        sub[win] = z[win]
        c = w0[..., None] * vcol[t[0]] + w1[..., None] * vcol[t[1]] + w2[..., None] * vcol[t[2]]
        img[y0:y1 + 1, x0:x1 + 1][win] = c[win]
    return (img * 255).astype(np.uint8), zbuf


def _in_polygon(pts, poly):
    x, y = pts[:, 0], pts[:, 1]
    inside = np.zeros(len(pts), bool)
    for (x0, y0), (x1, y1) in zip(poly, np.roll(poly, -1, 0)):
        cross = ((y0 > y) != (y1 > y)) & (x < (x1 - x0) * (y - y0) / (y1 - y0 + 1e-12) + x0)
        inside ^= cross
    return inside


def paint(base, corr):
    """Vertex colours with dark brows and red lips, placed from the landmark vertices on the undeformed head.

    The detector needs to see brows and lips to place them, as it would on a photo.
    """
    v = {o["name"]: base[o["vertex"], :2] for o in corr}
    xy = base[:, :2]
    front = base[:, 2] > 0.09
    col = np.tile(SKIN, (len(base), 1))
    brow = np.zeros(len(base), bool)
    for s, sx in (("R", -1), ("L", 1)):
        line = [v[f"browInner{s}"], v[f"browPeak{s}"], v[f"browTail{s}"]]
        line = [p - [0, 0.0035] for p in line]
        d = np.full(len(base), np.inf)
        for a, b in zip(line, line[1:]):
            ab = b - a
            t = np.clip(((xy - a) @ ab) / (ab @ ab), 0, 1)
            d = np.minimum(d, np.linalg.norm(xy - (a + t[:, None] * ab), axis=1))
        half = 0.0042 - 0.0018 * np.clip((xy[:, 0] * sx - abs(line[0][0])) / (abs(line[2][0]) - abs(line[0][0]) + 1e-9), 0, 1)
        brow |= front & (d < half)
    col[brow] = [0.22, 0.16, 0.12]
    upper = [v["mouthCornerR"], v["bowPeakR"], v["upperLipMid"], v["bowPeakL"], v["mouthCornerL"], v["lipLine"]]
    lower = [v["mouthCornerR"], v["lipLine"], v["mouthCornerL"], v["lowerLipBottom"]]
    lips = front & (_in_polygon(xy, np.array(upper)) | _in_polygon(xy, np.array(lower)))
    col[lips] = [0.72, 0.42, 0.40]
    return col
