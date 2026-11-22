"""Lays the hair from a photo over a render of a share code, aligned on the pupils, to judge the face as it would read with that hair.

Usage: python -I hair_composite.py <share code file> <photo> <out.png> [--yaw Y --pitch P --distance D] [--tint]
"""
import argparse, base64, os, sys
import numpy as np
import cv2
import mediapipe as mp
from mediapipe.tasks.python import BaseOptions, vision
from PIL import Image, ImageOps

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from head import Head, read_share_code_full, SURF_DEF
from fit import DATA
from detect import Detector
from glrender import GLHead, BROW_KEYS


def hair_mask(rgb):
    seg = vision.ImageSegmenter.create_from_options(vision.ImageSegmenterOptions(
        base_options=BaseOptions(model_asset_path=os.path.join(DATA, "model", "selfie_multiclass.tflite")), output_confidence_masks=True))
    r = seg.segment(mp.Image(image_format=mp.ImageFormat.SRGB, data=np.ascontiguousarray(rgb)))
    return r.confidence_masks[1].numpy_view().copy()  # class 1 is hair


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("code_file")
    ap.add_argument("photo")
    ap.add_argument("out")
    ap.add_argument("--yaw", type=float, default=0.0)
    ap.add_argument("--pitch", type=float, default=0.0)
    ap.add_argument("--distance", type=float, default=0.62)
    ap.add_argument("--tint", help="colour the photo's hair with this hex colour, for a black and white photo")
    a = ap.parse_args()
    h = Head(os.path.join(DATA, "likeness"))
    code = open(a.code_file).read().strip()
    v, g, age, reg, surf = read_share_code_full(code, len(h.modifiers))
    gl = GLHead(h, 512, dist=a.distance)
    gl.set_eye_paint(surf["irisSize"], surf["pupilSize"])
    img, p5 = gl.render(v, g, age, reg, yaw=a.yaw, pitch=a.pitch, brow={key: surf[key] for key in BROW_KEYS + ["browColor", "browDensity"]})
    rgb = np.asarray(ImageOps.exif_transpose(Image.open(a.photo)).convert("RGB"))
    d = Detector(os.path.join(DATA, "model", "face_landmarker.task")).detect(rgb)
    src, dst = np.array([d.points[468], d.points[473]], dtype=np.float32), p5[:2]
    sv, dv = src[1] - src[0], dst[1] - dst[0]
    s = np.linalg.norm(dv) / np.linalg.norm(sv)
    ang = np.arctan2(dv[1], dv[0]) - np.arctan2(sv[1], sv[0])
    M = np.array([[s * np.cos(ang), -s * np.sin(ang), 0], [s * np.sin(ang), s * np.cos(ang), 0]], dtype=np.float32)
    M[:, 2] = dst[0] - M[:, :2] @ src[0]
    photo = cv2.warpAffine(rgb, M, (512, 512), borderValue=(0, 0, 0)).astype(np.float32)
    mask = cv2.warpAffine(hair_mask(rgb), M, (512, 512), borderValue=0)
    mask = cv2.GaussianBlur(np.clip((mask - 0.3) / 0.4, 0, 1), (5, 5), 0)[..., None]
    hair = photo
    if a.tint:
        c = np.array([int(a.tint[i:i + 2], 16) for i in (1, 3, 5)], dtype=np.float32)
        lum = photo.mean(2, keepdims=True) / 255
        hair = np.clip(lum * c * 2.2, 0, 255)
    out = img.astype(np.float32) * (1 - mask) + hair * mask
    Image.fromarray(np.concatenate([photo.astype(np.uint8), out.astype(np.uint8)], 1)).save(a.out)


if __name__ == "__main__":
    main()
