"""Renders a share code beside a photo and over it, so a person or an agent can adjust sliders by eye.

Usage:
  python -I tweak.py <share code file> <photo> <out.png> [--set "group/label/side=value" ...] [--save-code new_code.txt] [--list]
The --set names match the --list output. A side is L, R or C. With symmetry, setting L also sets R.
Brow shape numbers can be set as surface/browInner=0.3 and so on.
"""
import argparse, base64, os, sys
import numpy as np
import cv2
from PIL import Image, ImageOps

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import head as head_module
from head import Head, share_code, read_share_code_full, SURF_DEF
from fit import DATA
from detect import Detector
from glrender import GLHead, BROW_KEYS


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("code_file")
    ap.add_argument("photo")
    ap.add_argument("out")
    ap.add_argument("--set", action="append", default=[])
    ap.add_argument("--save-code")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--yaw", type=float, default=0.0, help="turn the head, in degrees, positive toward the character's left")
    ap.add_argument("--pitch", type=float, default=0.0, help="tilt the head, in degrees, positive lifts the chin")
    ap.add_argument("--distance", type=float, default=0.62, help="camera distance in metres. The page uses 0.62; studio portraits are often 1.5 to 3")
    a = ap.parse_args()
    h = Head(os.path.join(DATA, "likeness"))
    code = open(a.code_file).read().strip()
    v, g, age, reg, surf = read_share_code_full(code, len(h.modifiers))
    if a.list:
        for i, m in enumerate(h.modifiers):
            print(f"{m['group']}/{m['label']}/{m['side']} = {v[i]:+.2f}")
        for key in BROW_KEYS + ["browColor", "browDensity"]:
            print(f"surface/{key} = {surf[key]}")
        print(f"macro/gender = {g:+.2f}, macro/age = {age:.2f}")
        return
    for s in a.set:
        name, val = s.rsplit("=", 1)
        val = float(val)
        if name.startswith("surface/"):
            surf[name.split("/", 1)[1]] = val
            continue
        if name == "macro/gender":
            g = val
            continue
        if name == "macro/age":
            age = val
            continue
        grp, rest = name.split("/", 1)
        label, side = rest.rsplit("/", 1)
        hits = [i for i, m in enumerate(h.modifiers) if m["group"] == grp and m["label"] == label and m["side"] == side]
        if not hits:
            raise SystemExit(f"No slider is called {name}. Use --list.")
        i = hits[0]
        v[i] = val
        if h.modifiers[i].get("pair"):
            v[h.by_name[h.modifiers[i]["pair"]]] = val
    gl = GLHead(h, 512, dist=a.distance)
    gl.set_eye_paint(surf['irisSize'], surf['pupilSize'])
    img, p5 = gl.render(v, g, age, reg, yaw=a.yaw, pitch=a.pitch, brow={key: surf[key] for key in BROW_KEYS + ["browColor", "browDensity"]})
    rgb = np.asarray(ImageOps.exif_transpose(Image.open(a.photo)).convert("RGB"))
    d = Detector(os.path.join(DATA, "model", "face_landmarker.task")).detect(rgb)
    src = np.array([d.points[468], d.points[473]], dtype=np.float32)
    dst = p5[:2]
    # similarity from the photo's pupils to the render's pupils
    sv, dv = src[1] - src[0], dst[1] - dst[0]
    s = np.linalg.norm(dv) / np.linalg.norm(sv)
    ang = np.arctan2(dv[1], dv[0]) - np.arctan2(sv[1], sv[0])
    c, si = s * np.cos(ang), s * np.sin(ang)
    M = np.array([[c, -si, 0], [si, c, 0]], dtype=np.float32)
    M[:, 2] = dst[0] - M[:, :2] @ src[0]
    photo = cv2.warpAffine(rgb, M, (512, 512), borderValue=(0, 0, 0))
    overlay = (0.5 * photo + 0.5 * img).astype(np.uint8)
    Image.fromarray(np.concatenate([photo, img, overlay], 1)).save(a.out)
    vals = dict(surf)
    if a.save_code:
        head_module.SURF_DEF = [(key, kind, vals[key]) for key, kind, _ in SURF_DEF]
        open(a.save_code, "w").write(share_code(v, g, age, reg))
        head_module.SURF_DEF = SURF_DEF


if __name__ == "__main__":
    main()
