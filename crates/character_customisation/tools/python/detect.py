"""Face landmark detection with MediaPipe, plus the photo checks every photo-to-face product makes."""
import math
from dataclasses import dataclass, field
import numpy as np

import mediapipe as mp
from mediapipe.tasks.python import BaseOptions, vision

IRIS_R, IRIS_L = 468, 473  # the subject's right and left iris centres


@dataclass
class Detection:
    points: np.ndarray  # 478 × 2, in pixels
    blend: dict
    yaw: float  # degrees
    pitch: float
    roll: float
    problems: list = field(default_factory=list)  # (severity, message)


class Detector:
    def __init__(self, model):
        self.det = vision.FaceLandmarker.create_from_options(vision.FaceLandmarkerOptions(
            base_options=BaseOptions(model_asset_path=model), output_face_blendshapes=True,
            output_facial_transformation_matrixes=True, num_faces=2))

    def _run(self, rgb):
        return self.det.detect(mp.Image(image_format=mp.ImageFormat.SRGB, data=np.ascontiguousarray(rgb[..., :3])))

    def detect(self, rgb):
        """Finds the face in the whole photo, or else in smaller and smaller crops, because the detector only finds faces that fill much of the frame."""
        H, W = rgb.shape[:2]
        r, ox, oy, w, h = self._run(rgb), 0, 0, W, H
        for frac in (0.5, 0.33, 0.2):
            if r.face_landmarks:
                break
            side = int(min(W, H) * frac)
            for y in range(0, H - side + 1, side // 2):
                for x in range(0, W - side + 1, side // 2):
                    r = self._run(rgb[y:y + side, x:x + side])
                    if r.face_landmarks:
                        ox, oy, w, h = x, y, side, side
                        break
                if r.face_landmarks:
                    break
        if not r.face_landmarks:
            raise SystemExit("No face was found in the photo.")
        faces = [np.array([[ox + p.x * w, oy + p.y * h] for p in f]) for f in r.face_landmarks]
        i = int(np.argmax([np.linalg.norm(f[IRIS_L] - f[IRIS_R]) for f in faces]))
        pts = faces[i]
        blend = {b.category_name: b.score for b in r.face_blendshapes[i]}
        R = np.array(r.facial_transformation_matrixes[i])[:3, :3]
        yaw = math.degrees(math.atan2(R[0, 2], R[2, 2]))
        pitch = math.degrees(math.asin(-max(-1, min(1, R[1, 2]))))
        roll = math.degrees(math.atan2(R[1, 0], R[0, 0]))
        d = Detection(pts, blend, yaw, pitch, roll)
        if len(r.face_landmarks) > 1:
            d.problems.append(("warn", "More than one face is in the photo. The fit uses the largest face."))
        check(d, rgb)
        return d


def check(d, rgb):
    p = d.problems
    if abs(d.yaw) > 12:
        p.append(("refuse", f"The face is turned {abs(d.yaw):.0f} degrees to the side. A front photo must be within 12 degrees."))
    elif abs(d.yaw) > 5:
        p.append(("warn", f"The face is turned {abs(d.yaw):.0f} degrees to the side. The face width and the nose position will be off."))
    # Level portraits read 5 to 10 degrees here, so the tilt limits are wider than the turn limits.
    if abs(d.pitch) > 20:
        p.append(("refuse", f"The face is tilted {abs(d.pitch):.0f} degrees up or down. A front photo must be within 20 degrees."))
    elif abs(d.pitch) > 13:
        p.append(("warn", f"The face is tilted {abs(d.pitch):.0f} degrees up or down. The vertical proportions will be off."))
    ipd = np.linalg.norm(d.points[IRIS_L] - d.points[IRIS_R])
    gap = np.linalg.norm(d.points[13] - d.points[14]) / ipd
    if d.blend.get("jawOpen", 0) > 0.3 or gap > 0.08:
        p.append(("refuse", "The mouth is open. The fit needs a closed, relaxed mouth."))
    smile = max(d.blend.get("mouthSmileLeft", 0), d.blend.get("mouthSmileRight", 0))
    if smile > 0.45:
        p.append(("warn", "The person is smiling. A smile widens the mouth and lifts the cheeks, so the fit will make them wider than they are."))
    if max(d.blend.get("eyeBlinkLeft", 0), d.blend.get("eyeBlinkRight", 0)) > 0.5:
        p.append(("warn", "An eye is closed or nearly closed. The eye shape will be off."))
    if ipd < 60:
        p.append(("warn", f"The eyes are only {ipd:.0f} pixels apart. Use a photo where they are at least 60 pixels apart."))
    grey = rgb[..., :3].astype(np.float64) @ [0.299, 0.587, 0.114]
    def patch(i):
        x, y = d.points[i].astype(int)
        r = max(3, int(ipd * 0.08))
        return grey[max(0, y - r):y + r, max(0, x - r):x + r].mean()
    # Upper cheeks, under the eyes and the forehead, where a beard does not reach.
    right = np.mean([patch(i) for i in (50, 117, 108)])
    left = np.mean([patch(i) for i in (280, 346, 337)])
    ratio = max(left, right) / max(1, min(left, right))
    if ratio > 2.2:
        p.append(("refuse", f"One side of the face is {ratio:.1f} times brighter than the other. The fit needs even front lighting."))
    elif ratio > 1.5:
        p.append(("warn", f"One side of the face is {ratio:.1f} times brighter than the other. Uneven lighting moves some landmarks."))
