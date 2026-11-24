"""Fits the eye shape sliders to the outline of the eye openings in a front photo.

The photo's outline is MediaPipe's 16-point contour of each eye. MediaPipe follows real eyes well, except that its
inner corner runs onto the pink bump of the tear duct (the caruncle), which the head does not model. So that end of
the contour is pulled back to where the white of the eye ends in the pixels.
The head's outline is the edge of its lids, projected from the mesh at the given pose.
Both outlines are put in the same frame, with the pupils at (0, 0) and (1, 0). A search moves the eye sliders until
the two outlines are as close as possible, point to curve both ways, with the corners also counted on their own.
The iris size is measured from the iris edge in the pixels.

Usage:
  python -I eye_fit.py <share code file> <photo> <out code file> --distance D --pitch P --yaw Y [--debug out.png] [--stability]
  python -I eye_fit.py <share code file> --selftest --distance D --pitch P --yaw Y
"""
import argparse, os, sys
import numpy as np
import cv2
import cma
from PIL import Image, ImageOps

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import head as head_module
from head import Head, read_share_code_full, share_code, EYE_RADIUS
from fit import DATA
from detect import Detector
from glrender import GLHead, VERT, perspective, look_at
from eye_targets import lid_margin

SIZE = 1024
# MediaPipe's eye contours, from the outer corner over the upper lid to the inner corner (index 8), then back along the
# lower lid. R is the subject's right eye (image left).
MP_EYE = {
    "R": [33, 246, 161, 160, 159, 158, 157, 173, 133, 155, 154, 153, 145, 144, 163, 7],
    "L": [263, 466, 388, 387, 386, 385, 384, 398, 362, 382, 381, 380, 374, 373, 390, 249],
}
MP_IRIS = {"R": (468, [469, 470, 471, 472]), "L": (473, [474, 475, 476, 477])}
# Socket depth is left out: it mostly moves the lids in depth, so a front outline barely sees it (the round trip
# recovered it with a mean error of 0.25), and the search used it at ±1 as a fudge.
NEW = ["Upper lid height (down / up)", "Lower lid height (down / up)", "Eye width (narrow / wide)",
       "Tilt (outer corner down / up)", "Lid fold (deep / hooded)", "Opening shift (nose / temple)"]
SHORT = ["upper lid", "lower lid", "width", "tilt", "fold", "shift"]
# The two Position sliders are left out: they move the eyes in the face, which the landmark fit (fit.py) owns.
# When they were allowed, the search set both to ±1 on three of the four photos.
OLD = ["Size", "Corner1 (down / up)", "Corner2 (down / up)", "Epicanthus (in / out)"]
LIMIT = 1.0
SIGMA = 0.01  # outline distances are counted in units of 1% of the pupil distance (about 0.6 mm)
PRIOR = 0.2  # the cost of one slider at ±1, against a mean squared outline error of SIGMA² costing 1
CORNER = 1.0  # the weight of each corner's squared distance against the mean of all outline points
OLD_GAIN = 0.15  # the older sliders are kept only when they lower the loss by this share
IRIS_OFFSET = 0.003  # the pixel iris measurement reads this much low on renders, as radius over pupil distance
NOISE = 0.005  # the round trip's noise on the outline points, half a percent of the pupil distance (about 0.3 mm)

MASK_FRAG = """
#version 330
uniform float value; out vec4 frag;
void main() { frag = vec4(value, value, value, 1.0); }
"""


class EyeGL(GLHead):
    """GLHead plus the head's lid edge as projected points."""

    def __init__(self, *a, **k):
        super().__init__(*a, **k)
        h = self.h
        ec = h.eye_centres(h.base)
        self.loop = {}
        for side in "LR":  # the lid edge of each eye, ordered around the eye by angle in the front view
            ids = lid_margin(h.base, ec[side])
            q = h.base[ids] - ec[side]
            self.loop[side] = ids[np.argsort(np.arctan2(q[:, 1], q[:, 0]))]
        sub = np.unique(np.r_[self.loop["L"], self.loop["R"], h.eye_joints["L"], h.eye_joints["R"]])
        self.where = {v: i for i, v in enumerate(sub)}
        self.Tsub = self.T.reshape(len(self.T), -1, 3)[:, sub, :].reshape(len(self.T), -1).astype(np.float64)
        self.base_sub = h.base[sub].astype(np.float64)

    def camera(self, joints_l, joints_r):
        """The page's camera, as render() places it."""
        el, er = joints_l.mean(0), joints_r.mean(0)
        target = (el + er) / 2 - np.array([0, 0.03, 0])
        eye = target + np.array([0, 0, self.dist])
        fov = np.degrees(2 * np.arctan(np.tan(np.radians(self.fov) / 2) * 0.62 / self.dist))
        return perspective(fov, 1, 0.05, 10) @ look_at(eye, target)

    def project(self, mvp, P):
        h = np.c_[P, np.ones(len(P))] @ mvp.T
        n = h[:, :2] / h[:, 3:]
        return np.stack([(n[:, 0] + 1) / 2 * self.size, (1 - n[:, 1]) / 2 * self.size], 1)

    def outlines(self, values, gender, age, regions, yaw=0.0, pitch=0.0):
        """The lid edge of both eyes and the two pupils (subject's right first), in render pixels at the pose.

        Only the lid edge and eye joint vertices are deformed, so this is much faster than a render."""
        w = self.weights_vector(values, gender, age, regions).astype(np.float64)
        pos = self.base_sub + (w @ self.Tsub).reshape(-1, 3)
        ix = lambda ids: [self.where[v] for v in ids]
        jl, jr = ix(self.h.eye_joints["L"]), ix(self.h.eye_joints["R"])
        if yaw or pitch:  # as render() turns the head, around the point between the eyes
            c = (pos[jl].mean(0) + pos[jr].mean(0)) / 2
            cy, sy = np.cos(np.radians(yaw)), np.sin(np.radians(yaw))
            cp, sp = np.cos(np.radians(-pitch)), np.sin(np.radians(-pitch))
            Ry = np.array([[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]])
            Rx = np.array([[1, 0, 0], [0, cp, -sp], [0, sp, cp]])
            pos = (pos - c) @ (Ry @ Rx).T + c
        mvp = self.camera(pos[jl], pos[jr])
        pupils = self.project(mvp, np.array([pos[jr].mean(0), pos[jl].mean(0)]))
        return {s: self.project(mvp, pos[ix(self.loop[s])]) for s in "LR"}, pupils


def load_rgb(path):
    im = ImageOps.exif_transpose(Image.open(path))
    if im.mode in ("RGBA", "LA", "P"):
        im = im.convert("RGBA")
        bg = Image.new("RGBA", im.size, (128, 128, 128, 255))
        im = Image.alpha_composite(bg, im)
    return np.asarray(im.convert("RGB"))


def similarity(src, dst):
    """The 2 × 3 similarity that maps the two src points onto the two dst points."""
    src, dst = np.asarray(src, float), np.asarray(dst, float)
    sv, dv = src[1] - src[0], dst[1] - dst[0]
    s = np.linalg.norm(dv) / np.linalg.norm(sv)
    ang = np.arctan2(dv[1], dv[0]) - np.arctan2(sv[1], sv[0])
    M = np.array([[s * np.cos(ang), -s * np.sin(ang), 0], [s * np.sin(ang), s * np.cos(ang), 0]])
    M[:, 2] = dst[0] - M[:, :2] @ src[0]
    return M


def apply(M, P):
    return P @ M[:, :2].T + M[:, 2]


UNIT = np.array([[0.0, 0.0], [1.0, 0.0]])  # the comparison frame: the subject's right pupil at 0, the left at 1


def whiteness(rgb):
    """Lightness minus twice the colourfulness, in Lab. The white of the eye scores high and the pink caruncle low.
    On a black and white photo this is the plain lightness."""
    lab = cv2.cvtColor(rgb, cv2.COLOR_RGB2LAB).astype(np.float32)
    return lab[..., 0] - 2.0 * np.hypot(lab[..., 1] - 128, lab[..., 2] - 128)


def refine_iris(grey, c0, r0):
    """The iris circle found in the pixels: the centre and radius whose left and right arcs best follow a dark-to-light edge.

    Only the arcs within 40 degrees of horizontal count, because the lids usually cover the top and bottom of the iris.
    The search covers centres within 40% of the starting radius, and radii from 85% to 140% of it. Radii start at 85%,
    so the edge between a dark pupil and a light iris is not taken for the iris edge.
    """
    g = cv2.GaussianBlur(grey, (0, 0), max(0.5, r0 * 0.04))
    ang = np.radians(np.r_[np.linspace(-40, 40, 17), np.linspace(140, 220, 17)])
    dirs = np.stack([np.cos(ang), np.sin(ang)], 1)
    offs = np.linspace(-0.4, 0.4, 17) * r0
    radii = np.linspace(0.85, 1.4, 23) * r0
    d = max(1.0, r0 * 0.08)
    cx, cy, rr = np.meshgrid(c0[0] + offs, c0[1] + offs, radii, indexing="ij")

    def ring(dr):
        return np.stack([cx[..., None] + (rr[..., None] + dr) * dirs[:, 0], cy[..., None] + (rr[..., None] + dr) * dirs[:, 1]], -1)

    def smp(q):
        f = q.reshape(-1, q.shape[-2], 2).astype(np.float32)
        return cv2.remap(g, f[..., 0], f[..., 1], cv2.INTER_LINEAR).reshape(q.shape[:-1])

    rise = smp(ring(d)) - smp(ring(-d))
    score = np.mean(np.minimum(rise, np.percentile(rise, 99)), axis=-1)
    i, j, k = np.unravel_index(np.argmax(score), score.shape)
    return np.array([cx[i, j, k], cy[i, j, k]]), float(rr[i, j, k])


def trim_caruncle(white, con, pupil, iris_r):
    """MediaPipe's eye contour with its inner corner pulled back to where the white of the eye ends.

    Along the line from the iris's inner edge to MediaPipe's inner corner, the white of the eye is bright and the
    caruncle is darker or pinker. The new corner is the first point past the brightest stretch where the brightness has
    fallen halfway from the white's level to the level at MediaPipe's corner. The lid points beyond the new corner are
    dropped, and the outline closes at the new corner.
    When the white is not clearly brighter than the corner (by 15% of the brightness range around the eye), nothing is
    trimmed. Returns the outline and the share of the stretch from the iris to the corner that was kept.
    """
    inner = con[8]
    u = (inner - pupil) / np.linalg.norm(inner - pupil)
    start = pupil + u * iris_r
    if np.linalg.norm(inner - start) <= 1 or (inner - start) @ u <= 0:
        return cut_contour(con, inner, u), 1.0
    t = np.linspace(0, 1, 60)
    line = start + t[:, None] * (inner - start)
    v = cv2.remap(white, line[:, 0:1].astype(np.float32), line[:, 1:2].astype(np.float32), cv2.INTER_LINEAR)[:, 0]
    v = np.convolve(np.pad(v, 2, mode="edge"), np.ones(5) / 5, mode="valid")
    top_i = int(np.argmax(v[:45]))
    top, end = np.percentile(v[:45], 90), v[-3:].mean()
    r = int(3 * iris_r)
    y, x = int(pupil[1]), int(pupil[0])
    lo, hi = np.percentile(white[max(0, y - r):y + r, max(0, x - r):x + r], [5, 95])
    if top - end < 0.15 * (hi - lo):
        return cut_contour(con, inner, u), 1.0
    below = np.nonzero(v[top_i:] < (top + end) / 2)[0]
    keep = 1.0 if len(below) == 0 else max(float(t[top_i + below[0]]), 0.3)
    return cut_contour(con, start + keep * (inner - start), u), keep


def cut_contour(con, corner, u):
    """The closed outline from MediaPipe's contour, ending at the given inner corner: the lid points beyond it, toward
    the nose, are dropped. Returns the outline (starting at the outer corner) with its inner and outer corners."""
    lim = corner @ u
    up = [p for p in con[1:8] if p @ u < lim]
    lo = [p for p in con[9:16] if p @ u < lim]
    poly = np.array([con[0]] + up + [corner] + lo)
    return dict(poly=poly, inner=corner, outer=con[0])


def photo_outlines(rgb, d):
    """MediaPipe's eye contours with the caruncle trimmed, in the pupil frame; the raw contours in the same frame;
    the share kept by each trim; and the iris radius measured in the pixels, over the pupil distance."""
    pts = d.points
    ipd = np.linalg.norm(pts[473] - pts[468])
    white = whiteness(rgb)
    grey = cv2.cvtColor(rgb, cv2.COLOR_RGB2LAB)[..., 0].astype(np.float32)
    M = similarity(pts[[468, 473]], UNIT)
    out, raw, keep, radii = {}, {}, {}, []
    for side in "LR":
        c, ring = MP_IRIS[side]
        _, r = refine_iris(grey, pts[c], np.linalg.norm(pts[ring] - pts[c], axis=1).mean())
        radii.append(r)
        con = pts[MP_EYE[side]]
        o, keep[side] = trim_caruncle(white, con, pts[c], r)
        out[side], raw[side] = {k: apply(M, np.atleast_2d(x)).reshape(np.shape(x)) for k, x in o.items()}, apply(M, con)
    return out, raw, keep, float(np.mean(radii) / ipd)


def seg_dist(P, Q):
    """The distance from each point of P to the closed polyline through Q."""
    A, B = Q, np.roll(Q, -1, axis=0)
    v = B - A
    w = P[:, None, :] - A[None]
    t = np.clip((w * v[None]).sum(-1) / np.maximum((v * v).sum(-1), 1e-12)[None], 0, 1)
    return np.min(np.linalg.norm(w - t[..., None] * v[None], axis=-1), axis=1)


def corners(loop, side):
    """The inner and outer corners of an outline: its points furthest toward the nose and toward the temple."""
    x = loop[:, 0] if side == "L" else -loop[:, 0]  # grows toward the temple
    return loop[np.argmin(x)], loop[np.argmax(x)]


def outline_error(photo, head, per_eye=False):
    """Per eye: the mean squared point-to-curve distance from the photo's points to the head's outline, the same from
    the head's points to the photo's outline, and the squared distances between the two inner and two outer corners.
    In units of SIGMA²."""
    tot, parts = 0.0, {}
    for side in "LR":
        P, Q = photo[side], head[side]
        ci, co = corners(Q, side)
        dc = np.sum((P["inner"] - ci) ** 2) + np.sum((P["outer"] - co) ** 2)
        e = (np.mean(seg_dist(P["poly"], Q) ** 2) + np.mean(seg_dist(Q, P["poly"]) ** 2) + CORNER * dc) / SIGMA ** 2
        parts[side] = e
        tot += e
    return (tot, parts) if per_eye else tot


class Problem:
    def __init__(self, h, gl, v, g, age, reg, pose):
        self.h, self.gl, self.v, self.g, self.age, self.reg, self.pose = h, gl, v.copy(), g, age, reg, pose
        eyes = [i for i, m in enumerate(h.modifiers) if m["group"] == "eyes"]
        self.new = [[i for i in eyes if h.modifiers[i]["label"] == lab] for lab in NEW]
        self.old = [[i for i in eyes if h.modifiers[i]["label"] == lab] for lab in OLD]
        for idx in self.new:
            self.v[idx] = 0  # the fit starts the new sliders from 0 and searches them

    def values(self, z, groups):
        vv = self.v.copy()
        for zi, idx in zip(z, groups):
            vv[idx] = zi
        return vv

    def head_outlines(self, vv, pose=None):
        loops, pupils = self.gl.outlines(vv, self.g, self.age, self.reg, **(pose or self.pose))
        M = similarity(pupils, UNIT)
        return {s: apply(M, q) for s, q in loops.items()}


def fit(prob, photo, groups, z0, evals=1000, seed=3, pose=None):
    def loss(z):
        return outline_error(photo, prob.head_outlines(prob.values(z, groups), pose)) + PRIOR * float(np.sum(np.square(z)))

    lo, hi = -LIMIT * np.ones(len(z0)), LIMIT * np.ones(len(z0))
    es = cma.CMAEvolutionStrategy(np.clip(z0, lo + 1e-3, hi - 1e-3), 0.3, {"bounds": [lo, hi], "popsize": 14, "maxfevals": evals, "verbose": -9, "seed": seed})
    while not es.stop():
        Z = es.ask()
        es.tell(Z, [loss(z) for z in Z])
    z = np.array(es.result.xbest)
    return z, loss(z), loss(np.array(z0))


def run_fit(prob, photo, log=print, try_old=True, pose=None):
    """The six new sliders first. Then the older eye sliders as well, kept only if they lower the loss by 15%."""
    z_new, l1, l0 = fit(prob, photo, prob.new, np.zeros(len(NEW)), pose=pose)
    log(f"  six new sliders: loss {l0:.2f} at 0 -> {l1:.2f}")
    if try_old:
        z_all, l2, _ = fit(prob, photo, prob.new + prob.old, np.r_[z_new, np.zeros(len(OLD))], pose=pose)
        log(f"  with the older sliders too: loss {l1:.2f} -> {l2:.2f}")
        if l2 < l1 * (1 - OLD_GAIN):
            return prob.new + prob.old, z_all, l2
    return prob.new, z_new, l1


def iris_size_for(ratio, ipd_mm):
    """The page's iris size whose iris radius is ratio × the pupil distance: radius = 12.3 mm × sin(acos(0.88 − 0.1 × size))."""
    r = ratio * ipd_mm
    return float((0.88 - np.cos(np.arcsin(np.clip(r / (EYE_RADIUS * 1000), 0, 1)))) / 0.1)


def describe(z, names):
    return ", ".join(f"{n} {x:+.2f}" for n, x in zip(names, z))


def debug_image(rgb, d, photo, raw, head, gl, vv, prob, zoom=3):
    """Eye crops of the photo with MediaPipe's points (yellow), the trimmed photo outline (red) and the fitted head's
    lid edge (blue), above the same crops of the render, then the two faces. Everything is aligned on the pupils."""
    img, p5 = gl.render(vv, prob.g, prob.age, prob.reg, **prob.pose)
    warped = cv2.warpAffine(rgb, similarity(d.points[[468, 473]], p5[:2]), (SIZE, SIZE), flags=cv2.INTER_CUBIC)
    Mu = similarity(UNIT, p5[:2])  # the pupil frame to render pixels
    cols = []
    for s in ("R", "L"):
        c = apply(Mu, photo[s]["poly"]).mean(0)
        x0, y0 = int(c[0] - 75), int(c[1] - 40)
        crops = [cv2.resize(np.ascontiguousarray(im[y0:y0 + 80, x0:x0 + 150]), None, fx=zoom, fy=zoom, interpolation=cv2.INTER_CUBIC) for im in (warped, img)]
        for k, cr in enumerate(crops):
            polys = ([(photo[s]["poly"], (255, 60, 60))] if k == 0 else []) + [(head[s], (60, 200, 255))]
            for P, col in polys:
                q = (apply(Mu, P) - [x0, y0]) * zoom
                cv2.polylines(cr, [np.round(q * 4).astype(np.int32)], True, col, 2, cv2.LINE_AA, shift=2)
            if k == 0:
                for p in (apply(Mu, raw[s]) - [x0, y0]) * zoom:
                    cv2.circle(cr, (int(p[0]), int(p[1])), 3, (255, 230, 0), -1)
        cols.append(np.concatenate(crops, 0))
    top = np.concatenate(cols, 1)
    ys = int(p5[:2, 1].mean())
    face = np.concatenate([warped[ys - 250:ys + 350, 212:812], img[ys - 250:ys + 350, 212:812]], 1)
    face = cv2.resize(face, (top.shape[1], int(face.shape[0] * top.shape[1] / face.shape[1])))
    return np.concatenate([top, face], 0)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("code_file")
    ap.add_argument("photo", nargs="?")
    ap.add_argument("out", nargs="?")
    ap.add_argument("--distance", type=float, default=2.0)
    ap.add_argument("--pitch", type=float, default=0.0)
    ap.add_argument("--yaw", type=float, default=0.0)
    ap.add_argument("--no-old", action="store_true", help="fit the six new sliders only")
    ap.add_argument("--stability", action="store_true", help="also fit with the yaw moved by ±1 degree and the pitch by ±2 degrees")
    ap.add_argument("--selftest", action="store_true", help="fit head outlines with known eye sliders instead of a photo")
    ap.add_argument("--debug")
    a = ap.parse_args()
    log = lambda *x: print(*x, file=sys.stderr)
    h = Head(os.path.join(DATA, "likeness"))
    v, g, age, reg, surf = read_share_code_full(open(a.code_file).read(), len(h.modifiers))
    gl = EyeGL(h, SIZE, dist=a.distance)
    pose = dict(yaw=a.yaw, pitch=a.pitch)
    prob = Problem(h, gl, v, g, age, reg, pose)
    if a.selftest:
        return selftest(prob, gl, surf, log)
    rgb = load_rgb(a.photo)
    d = Detector(os.path.join(DATA, "model", "face_landmarker.task")).detect(rgb)
    photo, raw, keep, ratio = photo_outlines(rgb, d)
    log(f"caruncle trim: kept {100 * keep['R']:.0f}% (right eye) and {100 * keep['L']:.0f}% (left eye) of the stretch from the iris to MediaPipe's inner corner")
    groups, z, l = run_fit(prob, photo, log, not a.no_old)
    vv = prob.values(z, groups)
    names = SHORT + OLD
    _, parts = outline_error(photo, prob.head_outlines(vv), per_eye=True)
    log("fitted: " + describe(z, names))
    log(f"  remaining outline error (loss per eye): right {parts['R']:.2f}, left {parts['L']:.2f}")
    if a.stability:
        base = np.zeros(len(names))
        base[:len(z)] = z
        R = [base]
        for dy, dp in ((-1, 0), (1, 0), (0, -2), (0, 2)):
            p2 = dict(yaw=a.yaw + dy, pitch=a.pitch + dp)
            _, z2, _ = run_fit(prob, photo, lambda *x: None, not a.no_old, pose=p2)
            full = np.zeros(len(names))
            full[:len(z2)] = z2
            R.append(full)
            log(f"  yaw {p2['yaw']:+.1f}, pitch {p2['pitch']:+.1f}: " + describe(full[:len(z2)], names))
        log("  largest move from the main fit: " + describe(np.abs(np.array(R) - base).max(0), names))
    e = h.eye_centres(gl.positions(gl.weights_vector(vv, g, age, reg)))
    ipd_mm = np.linalg.norm(e["L"] - e["R"]) * 1000
    true_ratio = ratio + IRIS_OFFSET
    size = iris_size_for(true_ratio, ipd_mm)
    log(f"iris: measured radius / pupil distance {ratio:.4f}, corrected {true_ratio:.4f}, eye distance {ipd_mm:.1f} mm, "
        f"iris radius {true_ratio * ipd_mm:.2f} mm, iris size {size:.2f}")
    surf["irisSize"] = float(np.clip(size, 0, 1))
    head_module.SURF_DEF = [(k, kind, surf[k]) for k, kind, _ in head_module.SURF_DEF]
    open(a.out, "w").write(share_code(vv, g, age, reg))
    if a.debug:
        gl.set_eye_paint(surf["irisSize"], surf["pupilSize"])
        Image.fromarray(debug_image(rgb, d, photo, raw, prob.head_outlines(vv), gl, vv, prob)).save(a.debug)


CASES = [
    [0.6, -0.4, 0.0, 0.0, 0.0, 0.0],
    [-0.6, 0.5, 0.4, 0.0, 0.5, -0.4],
    [0.0, 0.0, -0.6, 0.6, 0.0, 0.3],
    [0.4, 0.3, 0.3, -0.5, -0.6, 0.5],
    [-0.3, -0.5, -0.3, 0.3, 0.6, -0.6],
]


def mp_like(head, rng, noise):
    """16 points on a head outline, spread the way MediaPipe spreads its eye contour: the two corners, then seven points
    on each lid at evenly spaced positions across the eye. Gaussian noise is added to every point."""
    out = {}
    for side in "LR":
        Q = head[side]
        ci, co = corners(Q, side)
        A, B = Q, np.roll(Q, -1, axis=0)
        up, lo = [], []
        for x in np.linspace(co[0], ci[0], 9)[1:-1]:
            m = (A[:, 0] - x) * (B[:, 0] - x) <= 0
            dx = B[m, 0] - A[m, 0]
            ys = A[m, 1] + (x - A[m, 0]) / np.where(dx != 0, dx, 1) * (B[m, 1] - A[m, 1])
            up.append([x, ys.min()])  # image y grows downward, so the upper lid has the smaller y
            lo.append([x, ys.max()])
        P = np.array([co] + up + [ci] + lo[::-1]) + rng.normal(0, noise, (16, 2))
        out[side] = dict(poly=P, inner=P[8], outer=P[0])
    return out


def selftest(prob, gl, surf, log):
    """Head outlines at known slider values, resampled like MediaPipe's contour with noise, then fitted from 0."""
    rng = np.random.default_rng(1)
    errs = []
    for k, zt in enumerate(CASES):
        truth = prob.head_outlines(prob.values(np.array(zt), prob.new))
        z, _, _ = fit(prob, mp_like(truth, rng, NOISE), prob.new, np.zeros(len(NEW)))
        log(f"case {k + 1}:")
        log(f"  true:      {describe(zt, SHORT)}")
        log(f"  recovered: {describe(z, SHORT)}")
        errs.append(np.abs(z - zt))
    log("mean absolute error per slider: " + describe(np.mean(errs, 0), SHORT))
    # the pixel iris measurement on renders, started from the drawn pupil with a radius 10% too large
    for s in (0.0, 0.5, 1.0):
        gl.set_eye_paint(s, surf["pupilSize"])
        img, p5 = gl.render(prob.v, prob.g, prob.age, prob.reg, **prob.pose)
        grey = cv2.cvtColor(img, cv2.COLOR_RGB2LAB)[..., 0].astype(np.float32)
        e = prob.h.eye_centres(gl.positions(gl.weights_vector(prob.v, prob.g, prob.age, prob.reg)))
        ipd_mm = np.linalg.norm(e["L"] - e["R"]) * 1000
        ipd_px = np.linalg.norm(p5[1] - p5[0])
        true_r = EYE_RADIUS * 1000 * np.sin(np.arccos(0.88 - s * 0.1)) / ipd_mm
        r = np.mean([refine_iris(grey, p5[i], true_r * ipd_px * 1.1)[1] for i in (0, 1)]) / ipd_px
        log(f"iris size {s}: true radius / pupil distance {true_r:.4f}, measured {r:.4f}")


if __name__ == "__main__":
    main()
