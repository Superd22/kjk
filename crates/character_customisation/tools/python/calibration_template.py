"""Writes the template that head_rig's calibrate tool reads: the base head in viewer 2's vertex order, its landmarks, and which side of the
mouth opening each vertex lies on. Run it again only when the head mesh changes.

Usage: .venv/bin/python -I calibration_template.py <upperness.npy> <out.json>

upperness.npy holds, per vertex of the base head, the upper lip score (1 on the upper lip's side of the opening, 0 on the lower lip's) and
how strongly that side rule applies, from geodesic distances over the mesh. It was made by rigfix.upperness() during the mouth work, and a
copy is in likeness-fit-data/backups/mouth-v2-scripts-2026-11-20/.
"""
import json, os, sys
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from head import Head
from build_rig import MOUTH_Y

D = os.path.expanduser("~/.claude/tools/likeness-fit-data")
h = Head(os.path.join(D, "likeness"))
L = np.load(os.path.join(D, "likeness", "layout_v2.npz"))
viewer_to_base = np.array(sorted(set(zip(L["tris"].ravel().tolist(), h.tris.ravel().tolist())))).T[1]
rig = json.load(open(os.path.join(D, "likeness", "rig.json")))
uk = np.load(sys.argv[1])
assert len(uk) == h.n, "upperness.npy belongs to another mesh"
inside = {v for v, _ in rig["mouthInside"]}
eye = set(np.r_[h.eye_joints["L"], h.eye_joints["R"]].tolist())
out = {
    "n": int(len(viewer_to_base)),
    "pos": h.base[viewer_to_base].round(7).tolist(),
    "landmarks": h.landmarks,
    "mouthY": MOUTH_Y,
    "upper": uk[viewer_to_base, 0].round(5).tolist(),
    "side": uk[viewer_to_base, 1].round(5).tolist(),
    "mouthInside": [int(v) for v in np.nonzero([b in inside for b in viewer_to_base])[0]],
    "eyeJoints": [int(v) for v in np.nonzero([b in eye for b in viewer_to_base])[0]],
}
json.dump(out, open(sys.argv[2], "w"))
print("wrote", out["n"], "vertices,", len(out["mouthInside"]), "inside the mouth,", len(out["eyeJoints"]), "on the eyes")
