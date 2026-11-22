"""Writes KOTOR's own eyeballs and upper lid pieces for the eye sliders (run once).

For each of the 57 test heads, it reads the viewer's KOTOR mesh (`lids/vd/heads/<name>.json`, in the fit's frame, metres) and its skin
weights (`lids/vd/kskin/<name>.json`). The vertices whose main bone is 19 or 20 are the eyeballs (`eyela` on +x, `eyera` on -x), and 21
or 22 the upper lid pieces. Each eyeball gets a sphere fitted by least squares. The output is `sliders/eyes/kotor_eyes.json`: per head,
per eye (+x first), the sphere's centre and radius and the lid piece's and the eyeball's triangles, all in metres.
"""
import base64, json, struct, sys
from pathlib import Path
import numpy as np

DATA = Path.home() / ".claude/tools/likeness-fit-data"
VD = DATA / "lids/vd"
OUT = DATA / "sliders/eyes/kotor_eyes.json"


def floats(b64):
    raw = base64.b64decode(b64)
    return np.frombuffer(raw, dtype="<f4")


def sphere(p):
    a = np.c_[2 * p, np.ones(len(p))]
    f = (p ** 2).sum(1)
    c, *_ = np.linalg.lstsq(a, f, rcond=None)
    centre = c[:3]
    return centre, float(np.sqrt(c[3] + centre @ centre))


def main():
    manifest = json.loads((DATA / "sliders/input/manifest.json").read_text())
    out = {}
    for h in manifest["heads"]:
        name = h["name"]
        head = json.loads((VD / f"heads/{name}.json").read_text())
        skin = json.loads((VD / f"kskin/{name}.json").read_text())
        pos = np.concatenate([floats(m["pos"]).reshape(-1, 3) for m in head["kotor"]["materials"]])
        idx = np.concatenate([np.frombuffer(base64.b64decode(m["idx"]), dtype=np.uint8).reshape(-1, 4) for m in skin["materials"]])
        w = np.concatenate([floats(m["w"]).reshape(-1, 4) for m in skin["materials"]])
        main_bone = idx[np.arange(len(idx)), w.argmax(1)]
        tris = pos.reshape(-1, 3, 3)
        tri_bone = main_bone.reshape(-1, 3)
        face = np.frombuffer((DATA / f"sliders/input/heads/{name}/kotor_v.bin").read_bytes(), "<f8").reshape(-1, 3)
        frame_eyes = np.frombuffer((DATA / f"sliders/input/heads/{name}/eyes.bin").read_bytes(), "<f8").reshape(2, 3)
        on_face = (np.linalg.norm(tris.reshape(-1, 1, 3) - face[None], axis=2).min(1) < 1e-5).reshape(-1, 3)
        is_lid = np.isin(tri_bone, [21, 22]).all(1)
        middle = tris.mean(1)
        eyes = []
        for side, lid in [(0, 21), (1, 22)]:
            # the eyeball: triangles that are neither face nor lid, near this eye and on its side of the face (some heads skin their
            # eyeballs to the head bone, and one puts both eyes on one bone)
            near = np.linalg.norm(middle - frame_eyes[side], axis=1) < 0.022
            same_side = np.sign(middle[:, 0] - frame_eyes.mean(0)[0]) == np.sign(frame_eyes[side][0] - frame_eyes.mean(0)[0])
            bt = tris[~on_face.any(1) & ~is_lid & near & same_side]
            lt = tris[(tri_bone == lid).all(1)]
            if len(bt) == 0:
                eyes.append(None)
                continue
            centre, r = sphere(np.unique(bt.reshape(-1, 3), axis=0))
            eyes.append({"centre": centre.tolist(), "radius": r, "ball": bt.round(7).tolist(), "lid": lt.round(7).tolist()})
        out[name] = eyes
        e = [x for x in eyes if x]
        print(name, " ".join(f"r {x['radius']*1000:.1f} mm, lid {len(x['lid'])} tris" for x in e), file=sys.stderr)
    OUT.write_text(json.dumps(out))


main()
