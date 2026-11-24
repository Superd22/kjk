"""Adds the inputs that features after the nose need to the slider bench's input folder, once, without touching what prep_nose.py wrote.

It writes, under likeness-fit-data/sliders/input/extra/, and lists in input/manifest_extra.json:
- the base head's neck rim (42 vertices) and the vertices below it, the mouth bag, the 40 vertices of the lip line, and the ears, as
  the smoothing holds them (kotor_shape_smooth.py: more than 68 mm to the side, 680 to 780 mm high, 40 mm behind to 70 mm in front);
- for each KOTOR test head, which vertices of KOTOR's own face mesh are skin (not hair, not the eye socket, not an open edge), as the
  hairless wrap marked them, and the colour of KOTOR's texture under each of its triangles.

Usage: .venv/bin/python -I sliders/prep_extra.py
"""
import json, os
import numpy as np

DATA = os.path.expanduser("~/.claude/tools/likeness-fit-data")
LF = os.path.expanduser("~/.claude/tools/likeness-fit")
K = os.path.join(DATA, "kotor-heads")
IN = os.path.join(DATA, "sliders", "input")
OUT = os.path.join(IN, "extra")
manifest = {}


def put(name, a, dtype):
    a = np.ascontiguousarray(np.asarray(a).astype(dtype))
    path = os.path.join(OUT, name + ".bin")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    a.tofile(path)
    manifest["extra/" + name] = {"dtype": np.dtype(dtype).name, "shape": list(a.shape)}


def main():
    m = json.load(open(os.path.join(IN, "manifest.json")))
    base = np.fromfile(os.path.join(IN, "base.bin"), np.float64).reshape(-1, 3)
    cut = np.load(os.path.join(DATA, "likeness", "neck_cut.npz"))
    put("rim", cut["rim"], np.uint32)
    put("below", cut["below"], np.uint8)
    put("mouth_bag", np.load(os.path.join(DATA, "likeness", "mouth_bag.npy")), np.uint8)
    lips = np.zeros(len(base), np.uint8)
    lips[list(json.load(open(os.path.join(LF, "lip_vertices.json"))).values())] = 1
    put("lip_line", lips, np.uint8)
    x, y, z = base[:, 0], base[:, 1], base[:, 2]
    put("ears", (np.abs(x) > 0.068) & (y > 0.68) & (y < 0.78) & (z > -0.04) & (z < 0.07), np.uint8)
    for h in m["heads"]:
        name = h["name"]
        W = np.load(os.path.join(K, "wrap_v5", name + "_wrapped.npz"))
        sc = np.load(os.path.join(K, "wrap_v5", name + "_scan.npz"))
        nv = len(sc["v"])
        put(f"heads/{name}/kotor_skin", ~W["beard"][:nv], np.uint8)
        put(f"heads/{name}/kotor_colour", sc["colour"], np.float32)
    json.dump({"arrays": manifest}, open(os.path.join(IN, "manifest_extra.json"), "w"), indent=1)
    print("wrote", len(manifest), "arrays to", OUT)


if __name__ == "__main__":
    main()
