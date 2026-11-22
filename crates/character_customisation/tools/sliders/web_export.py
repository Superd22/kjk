"""Copies what the nose page needs from existing data, after nose_bench has run (light work only: reading and cropping files).

- KOTOR's own textured heads, from viewer 2's data, cut down to the part around the nose and put in the frame of the eyes (mm), one
  file per head in web/kotor/;
- our mesh's texture layout (layout v2, the one the KOTOR bakes use) and its drawn triangles, in web/layout.json;
- a link to viewer 2's textures as web/tex.

With --feature jaw, only the jaw's crops are written, in web/kotor_jaw/ (|x| < 90, -175 < y < -20, z > -110).
With --feature brow or --feature cheeks, only that feature's crops are written, in web/kotor_brow/ (|x| < 95, -20 < y < 105,
z > -85) or web/kotor_cheeks/ (|x| < 95, -95 < y < 25, z > -85).

With --feature eyes, only the eyes' crops are written, in web/kotor_eyes/ (|x| < 80, -40 < y < 40, z > -85).

Usage: .venv/bin/python -I sliders/web_export.py [--feature jaw|brow|cheeks|eyes]
"""
import base64, json, os, sys
import numpy as np

DATA = os.path.expanduser("~/.claude/tools/likeness-fit-data")
WEB = os.path.join(DATA, "sliders", "web")
VIEW = "/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data"
b64 = lambda a: base64.b64encode(np.ascontiguousarray(a).tobytes()).decode()


def main():
    feature = sys.argv[sys.argv.index("--feature") + 1] if "--feature" in sys.argv else "nose"
    jaw = feature != "nose"
    report = json.load(open(os.path.join(DATA, "sliders", "report.json")))
    folder = "kotor" if feature == "nose" else "kotor_" + feature
    os.makedirs(os.path.join(WEB, folder), exist_ok=True)
    for h in report["heads"]:
        f = h["frame"]
        O, A, s = np.array(f["origin"]), np.array(f["axes"]), f["scale"]
        d = json.load(open(os.path.join(VIEW, "heads", h["name"] + ".json")))
        out = []
        for m in d["kotor"]["materials"]:
            p = np.frombuffer(base64.b64decode(m["pos"]), np.float32).reshape(-1, 3).astype(float)
            uv = np.frombuffer(base64.b64decode(m["uv"]), np.float32).reshape(-1, 2)
            q = ((p - O) @ A.T) * s
            t = q.reshape(-1, 3, 3)
            if feature == "brow":
                keep = ((np.abs(t[:, :, 0]) < 95) & (t[:, :, 1] > -20) & (t[:, :, 1] < 105) & (t[:, :, 2] > -85)).any(1)
            elif feature == "cheeks":
                keep = ((np.abs(t[:, :, 0]) < 95) & (t[:, :, 1] > -95) & (t[:, :, 1] < 25) & (t[:, :, 2] > -85)).any(1)
            elif feature == "eyes":
                keep = ((np.abs(t[:, :, 0]) < 80) & (t[:, :, 1] > -40) & (t[:, :, 1] < 40) & (t[:, :, 2] > -85)).any(1)
            elif jaw:
                keep = ((np.abs(t[:, :, 0]) < 90) & (t[:, :, 1] > -175) & (t[:, :, 1] < -20) & (t[:, :, 2] > -110)).any(1)
            else:
                keep = ((np.abs(t[:, :, 0]) < 80) & (t[:, :, 1] > -110) & (t[:, :, 1] < 40) & (t[:, :, 2] > -60)).any(1)
            if not keep.any():
                continue
            out.append({"texture": m.get("texture"), "pos": b64(t[keep].reshape(-1, 3).astype(np.float32)),
                        "uv": b64(uv.reshape(-1, 3, 2)[keep].reshape(-1, 2).astype(np.float32))})
        json.dump(out, open(os.path.join(WEB, folder, h["name"] + ".json"), "w"))
    if jaw:
        print("wrote", os.path.join(WEB, folder))
        return
    L = np.load(os.path.join(DATA, "likeness", "layout_v2.npz"))
    cut = np.load(os.path.join(DATA, "likeness", "neck_cut.npz"))
    drawn = cut["drawn"].astype(bool)
    json.dump({"uv": b64(L["uv"].astype(np.float32)), "tris": b64(L["tris"][drawn].astype(np.uint32)), "vmap": b64(L["vmap"].astype(np.uint32)),
               "n": int(len(L["uv"]))}, open(os.path.join(WEB, "layout.json"), "w"))
    link = os.path.join(WEB, "tex")
    if not os.path.islink(link):
        os.symlink(os.path.join(VIEW, "tex"), link)
    print("wrote", WEB)


if __name__ == "__main__":
    main()
