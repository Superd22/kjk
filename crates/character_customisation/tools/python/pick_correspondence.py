"""Picks, once, the mesh vertex that matches each chosen MediaPipe landmark.

It renders the average head from the front, runs the landmark detector on the render,
and takes the visible vertex nearest to each detected point. The result goes to
correspondence.json for review, and an overlay image shows each pair.

Usage: python -I pick_correspondence.py <head data dir> <face_landmarker.task> <overlay.png> [weight of the other MediaPipe points]
"""
import json, os, sys
import numpy as np
from PIL import Image, ImageDraw

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from head import Head
from render import Camera, paint, render
from fit import normalise
from detect import Detector

HERE = os.path.dirname(os.path.abspath(__file__))

# name, MediaPipe index, weight. Names use the character's sides, so "L" is the image's right.
LANDMARKS = [
    ("eyeOuterR", 33, 1), ("eyeInnerR", 133, 1), ("lidUpperR", 159, 0.7), ("lidLowerR", 145, 0.7),
    ("eyeOuterL", 263, 1), ("eyeInnerL", 362, 1), ("lidUpperL", 386, 0.7), ("lidLowerL", 374, 0.7),
    ("browInnerR", 107, 0.6), ("browPeakR", 105, 0.6), ("browTailR", 70, 0.4),
    ("browInnerL", 336, 0.6), ("browPeakL", 334, 0.6), ("browTailL", 300, 0.4),
    ("noseBridge", 168, 0.5), ("noseTip", 1, 1), ("noseBase", 2, 1), ("alaR", 129, 1), ("alaL", 358, 1),
    ("mouthCornerR", 61, 1), ("mouthCornerL", 291, 1), ("bowPeakR", 37, 0.8), ("bowPeakL", 267, 0.8),
    ("upperLipMid", 0, 0.8), ("lipLine", 13, 1), ("lowerLipBottom", 17, 0.8),
    ("chin", 152, 1), ("jawAngleR", 172, 0.7), ("jawAngleL", 397, 0.7), ("jawMouthR", 58, 0.7), ("jawMouthL", 288, 0.7),
    ("cheekboneR", 123, 0.5), ("cheekboneL", 352, 0.5), ("outlineEyeR", 234, 0.7), ("outlineEyeL", 454, 0.7),
    ("outlineNoseR", 93, 0.7), ("outlineNoseL", 323, 0.7), ("hairline", 10, 0.3),
]


def front_camera(eyes):
    return Camera((eyes["L"] + eyes["R"]) / 2 - [0, 0.03, 0], 0.30, 768)


def pick(h, det, pos, colours, dense):
    e = h.eye_centres(pos)
    cam = front_camera(e)
    img, zbuf = render(pos, h.tris, [e["L"], e["R"]], cam, colours=colours)
    pts = det.detect(img).points
    xy, depth = cam.project(pos)
    px = np.clip(xy.astype(int), 0, cam.size - 1)
    visible = depth >= zbuf[px[:, 1], px[:, 0]] - 0.0015
    # The head is symmetric, so each left and right pair shares one pick, mirrored.
    mirror = lambda q: np.array([cam.size - q[0], q[1]])
    mi_of = {name: mi for name, mi, _ in LANDMARKS}
    left = np.nonzero(visible & (pos[:, 0] > 1e-4))[0]
    centre = np.nonzero(visible & (np.abs(pos[:, 0]) <= 1e-4))[0]
    out = []
    for name, mi, w in LANDMARKS:
        side = name[-1] if name[-1] in "LR" and name[:-1] + ("R" if name[-1] == "L" else "L") in mi_of else None
        if side is None:
            vi = int(centre[np.linalg.norm(xy[centre] - pts[mi], axis=1).argmin()])
        else:
            target = (pts[mi_of[name[:-1] + "L"]] + mirror(pts[mi_of[name[:-1] + "R"]])) / 2
            vl = int(left[np.linalg.norm(xy[left] - target, axis=1).argmin()])
            vi = vl if side == "L" else int(np.linalg.norm(pos - pos[vl] * [-1, 1, 1], axis=1).argmin())
        out.append({"name": name, "mediapipe": mi, "vertex": vi, "weight": w})
    # Every other MediaPipe point, at a low weight, where a visible vertex lies close to it.
    used = {mi for _, mi, _ in LANDMARKS}
    vis = np.nonzero(visible)[0]
    for mi in range(468):
        if dense <= 0 or mi in used:
            continue
        d = np.linalg.norm(xy[vis] - pts[mi], axis=1)
        if d.min() <= 5:
            out.append({"name": f"mp{mi}", "mediapipe": mi, "vertex": int(vis[d.argmin()]), "weight": dense})
    return out


def measure_bias(h, det, pos, colours, out):
    """The detector's offset from each vertex on the average face, in the eye frame. The fit subtracts it from photos."""
    e = h.eye_centres(pos)
    cam = front_camera(e)
    img, _ = render(pos, h.tris, [e["L"], e["R"]], cam, colours=colours)
    pts = det.detect(img).points
    xy, _ = cam.project(pos)
    flip = lambda q: q * [1, -1]
    det_n = normalise(flip(pts[[o["mediapipe"] for o in out]]), flip(pts[468]), flip(pts[473]))
    mesh_n = normalise(flip(xy[[o["vertex"] for o in out]]), flip(cam.project(e["R"][None])[0][0]), flip(cam.project(e["L"][None])[0][0]))
    bias = det_n - mesh_n
    by_name = {o["name"]: k for k, o in enumerate(out)}
    for k, o in enumerate(out):
        n = o["name"]
        o["pick_error_px"] = round(float(np.linalg.norm(xy[o["vertex"]] - pts[o["mediapipe"]])), 2)
        if n.startswith("mp"):
            o["bias"] = [round(float(x), 4) for x in bias[k]]
        elif n[-1] in "LR" and n[:-1] + ("R" if n[-1] == "L" else "L") in by_name:
            other = bias[by_name[n[:-1] + ("R" if n[-1] == "L" else "L")]] * [-1, 1]
            o["bias"] = [round(float(x), 4) for x in (bias[k] + other) / 2]
        else:
            o["bias"] = [0.0, round(float(bias[k][1]), 4)]
    return img, pts, xy


def main(data_dir, model, overlay, dense=0.0):
    h = Head(data_dir)
    det = Detector(model)
    pos = h.deform(h.target_weights(np.zeros(len(h.modifiers))))
    # The first pick has no paint. The brows and lips are then painted from it, and the second pick sees them.
    out = pick(h, det, pos, None, dense)
    out = pick(h, det, pos, paint(h.base, out), dense)
    img, pts, xy = measure_bias(h, det, pos, paint(h.base, out), out)
    json.dump({"note": "Mesh vertices matched to MediaPipe face landmarks on the average head. The pupils use the eyeball centres and are not listed. "
               "The bias is where the detector puts the landmark on the average face, relative to the vertex, in eye distances. "
               "calibrate.py adds the gain and the axis weights.",
               "landmarks": out}, open(os.path.join(HERE, "correspondence.json"), "w"), indent=1)
    im = Image.fromarray(img).resize((1536, 1536))
    dr = ImageDraw.Draw(im)
    for o in out:
        if o["name"].startswith("mp"):
            continue
        a, b = pts[o["mediapipe"]] * 2, xy[o["vertex"]] * 2
        dr.ellipse([a[0] - 4, a[1] - 4, a[0] + 4, a[1] + 4], outline=(255, 40, 40), width=2)
        dr.ellipse([b[0] - 3, b[1] - 3, b[0] + 3, b[1] + 3], fill=(40, 120, 255))
        dr.text((b[0] + 6, b[1] - 6), o["name"], fill=(255, 255, 255))
    im.save(overlay)
    for o in out[:len(LANDMARKS)]:
        print(f'{o["name"]:15} mp {o["mediapipe"]:3}  vertex {o["vertex"]:4}  {o["pick_error_px"]:.1f}px')


if __name__ == "__main__":
    main(*sys.argv[1:4], *[float(x) for x in sys.argv[4:5]])
