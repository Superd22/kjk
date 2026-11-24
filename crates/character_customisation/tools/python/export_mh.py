"""Exports the MakeHuman base head and its face targets for the likeness prototype.

Usage: python3 -I export_mh.py <mpfb2 repo> <out dir>
"""
import gzip, json, os, re, struct, sys

repo, out = sys.argv[1], sys.argv[2]
data = os.path.join(repo, "src", "mpfb", "data")
os.makedirs(out, exist_ok=True)
CUT_Y = 5.3  # decimetres; keeps the head and the neck
GROUPS = ["head", "forehead", "eyebrows", "eyes", "nose", "cheek", "chin", "ears", "neck", "mouth", "asym"]

# ---- base mesh ----
verts, uvs, faces, grp, groups = [], [], [], None, {}
for line in open(os.path.join(data, "3dobjs", "base.obj")):
    if line.startswith("v "):
        verts.append([float(x) for x in line.split()[1:4]])
    elif line.startswith("vt "):
        uvs.append([float(x) for x in line.split()[1:3]])
    elif line.startswith("g "):
        grp = line.split()[1]
    elif line.startswith("f "):
        toks = [t.split("/") for t in line.split()[1:]]
        f = [(int(t[0]) - 1, int(t[1]) - 1 if len(t) > 1 and t[1] else -1) for t in toks]
        groups.setdefault(grp, []).append(f)

keep = lambda vi: verts[vi][1] > CUT_Y
split = {}       # (v, vt) -> new index
new_pos, new_uv, new_of = [], [], {}  # new_of: original vertex -> list of new indices
tris = []
def add(v, vt):
    k = (v, vt)
    if k not in split:
        split[k] = len(new_pos)
        new_pos.append([verts[v][0] / 10, verts[v][1] / 10, verts[v][2] / 10])
        new_uv.append(uvs[vt] if vt >= 0 else [0, 0])
        new_of.setdefault(v, []).append(split[k])
    return split[k]
for f in groups["body"]:
    if not all(keep(v) for v, _ in f):
        continue
    ids = [add(v, vt) for v, vt in f]
    for i in range(1, len(ids) - 1):
        tris += [ids[0], ids[i], ids[i + 1]]
eye_joint = {}
for side, g in (("L", "joint-l-eye"), ("R", "joint-r-eye")):
    ids = sorted({v for f in groups[g] for v, _ in f})
    eye_joint[side] = [add(v, -1) for v in ids]
print(f"head: {len(new_pos)} vertices, {len(tris)//3} triangles", file=sys.stderr)

# ---- targets ----
cat = json.load(open(os.path.join(data, "targets", "target.json")))
target_files = {}
for root, _, files in os.walk(os.path.join(data, "targets")):
    for fn in files:
        if fn.endswith(".target.gz"):
            target_files[fn[:-len(".target.gz")]] = os.path.join(root, fn)

def read_target(name):
    """Returns {original vertex: (dx, dy, dz) in metres} restricted to kept vertices."""
    d = {}
    with gzip.open(target_files[name], "rt") as fh:
        for line in fh:
            if line.startswith("#") or not line.strip():
                continue
            p = line.split()
            vi = int(p[0])
            if vi in new_of:
                d[vi] = (float(p[1]) / 10, float(p[2]) / 10, float(p[3]) / 10)
    return d

LABELS = {"cupidsbow": "Cupid's bow", "philtrum": "philtrum", "trans": "position", "scale-horiz": "width", "scale-vert": "height",
          "scale-depth": "depth", "angles": "corners", "ext": "extent", "lowerlip": "lower lip", "upperlip": "upper lip", "nubian": "nubian",
          "eyefold": "eye fold", "epicanthus": "epicanthus", "septumangle": "septum angle", "prognathism": "prognathism", "laugh-lines": "laugh lines",
          "rot": "rotation", "scale": "size", "bag": "bags", "nostrils": "nostrils", "point": "tip", "greek": "greek"}
SUFFIX = {"-decr-incr": "", "-down-up": " (down / up)", "-backward-forward": " (back / forward)", "-in-out": " (in / out)", "-concave-convex": " (concave / convex)",
          "-compress-uncompress": " (compress)", "-square-round": " (square / round)", "-pointed-triangle": " (pointed / triangle)", "-l-r": " (left / right)"}
def label(name, group):
    n, tail = name, ""
    for s, t in SUFFIX.items():
        if n.endswith(s):
            n, tail = n[:-len(s)], t
    pref = {"eyes": "eye-", "ears": "ear-", "cheek": "cheek-", "chin": "chin-", "nose": "nose-", "mouth": "mouth-", "head": "head-", "forehead": "forehead-", "eyebrows": "eyebrows-", "neck": "neck-", "asym": "asym-"}[group]
    if n.startswith(pref):
        n = n[len(pref):]
    if n.startswith("measure-"):
        n = n[len("measure-"):]
    for k, v in LABELS.items():
        n = re.sub(rf"(^|-){k}(-|$)", rf"\1{v}\2", n)
    n = n.replace("-", " ").strip()
    return ((n[:1].upper() + n[1:]) if n else name) + tail

modifiers, targets, tindex = [], [], {}
def tid(name):
    if not name:
        return -1
    if name not in tindex:
        tindex[name] = len(targets)
        targets.append(name)
    return tindex[name]
for g in GROUPS:
    for c in cat[g]["categories"]:
        o = c.get("opposites")
        if o is None:  # a one-sided shape, such as head-oval: 0 to 1 only
            modifiers.append({"name": c["name"], "label": label(c["name"], g), "group": g, "side": "C", "neg": -1, "pos": tid(c["targets"][0]), "oneSided": True})
        elif c.get("has_left_and_right"):
            modifiers.append({"name": c["name"] + "-L", "label": label(c["name"], g), "group": g, "side": "L", "pair": c["name"] + "-R", "neg": tid(o["negative-left"]), "pos": tid(o["positive-left"])})
            modifiers.append({"name": c["name"] + "-R", "label": label(c["name"], g), "group": g, "side": "R", "pair": c["name"] + "-L", "neg": tid(o["negative-right"]), "pos": tid(o["positive-right"])})
        else:
            modifiers.append({"name": c["name"], "label": label(c["name"], g), "group": g, "side": "C", "neg": tid(o["negative-unsided"]), "pos": tid(o["positive-unsided"])})
macro = {}
for sex in ("female", "male"):
    for age in ("young", "old"):
        macro[f"universal-{sex}-{age}"] = tid(f"universal-{sex}-{age}-averagemuscle-averageweight")
        for eth in ("african", "asian", "caucasian"):
            macro[f"{eth}-{sex}-{age}"] = tid(f"{eth}-{sex}-{age}")

# The regional averages carry body height and posture, which move the whole head by up to 15 cm.
# Each one is shifted back so the base of the neck, where the head meets the body, stays where it is.
neck_ring = [i for i, p in enumerate(new_pos) if p[1] < min(q[1] for q in new_pos) + 0.005]
macro_names = set()
for sex in ("female", "male"):
    for age in ("young", "old"):
        for eth in ("african", "asian", "caucasian"):
            macro_names.add(f"{eth}-{sex}-{age}")

blob = bytearray()
toffsets = []
for name in targets:
    d = read_target(name)
    rows = []
    if name in macro_names:
        full = [(0.0, 0.0, 0.0)] * len(new_pos)
        for vi, delta in d.items():
            for ni in new_of[vi]:
                full[ni] = delta
        t = [sum(full[i][k] for i in neck_ring) / len(neck_ring) for k in range(3)]
        rows = [(ni, f[0] - t[0], f[1] - t[1], f[2] - t[2]) for ni, f in enumerate(full)]
        rows = [r for r in rows if abs(r[1]) + abs(r[2]) + abs(r[3]) > 1e-7]
        d = {}
    for vi, (dx, dy, dz) in d.items():
        for ni in new_of[vi]:
            rows.append((ni, dx, dy, dz))
    toffsets.append({"name": name, "offset": len(blob), "count": len(rows)})
    blob += struct.pack("<I", len(rows))
    blob += struct.pack(f"<{len(rows)}H", *[r[0] for r in rows])
    if len(rows) % 2:
        blob += b"\0\0"
    q = []
    for _, dx, dy, dz in rows:
        q += [max(-32767, min(32767, round(dx * 50000))), max(-32767, min(32767, round(dy * 50000))), max(-32767, min(32767, round(dz * 50000)))]
    blob += struct.pack(f"<{len(q)}h", *q)
    while len(blob) % 4:
        blob += b"\0"
print(f"{len(targets)} targets, {len(modifiers)} modifiers, {len(blob)/1e6:.2f} MB", file=sys.stderr)

# ---- landmarks from the targets themselves, in metres ----
def centroid(ids):
    xs = [new_pos[i] for i in ids]
    return [sum(p[k] for p in xs) / len(xs) for k in range(3)]
def moved(name, side=None):
    d = read_target(name)
    ids = [ni for vi in d for ni in new_of[vi]]
    if side == "L":
        ids = [i for i in ids if new_pos[i][0] > 0.002]
    if side == "R":
        ids = [i for i in ids if new_pos[i][0] < -0.002]
    return ids
lm = {}
lm["eyeL"] = centroid(eye_joint["L"]); lm["eyeR"] = centroid(eye_joint["R"])
tip = max(moved("nose-point-up"), key=lambda i: new_pos[i][2]); lm["noseTip"] = new_pos[tip]
lm["noseBridge"] = centroid(moved("nose-hump-incr"))
lm["mouth"] = centroid(moved("mouth-scale-horiz-incr"))
lm["lipUpper"] = centroid(moved("mouth-upperlip-height-incr")); lm["lipLower"] = centroid(moved("mouth-lowerlip-height-incr"))
lm["cornerL"] = centroid(moved("mouth-angles-up", "L")); lm["cornerR"] = centroid(moved("mouth-angles-up", "R"))
lm["browL"] = centroid(moved("eyebrows-trans-up", "L")); lm["browR"] = centroid(moved("eyebrows-trans-up", "R"))
lm["chin"] = centroid(moved("chin-prominent-incr"))
lm["forehead"] = centroid(moved("forehead-trans-forward"))
lm["earL"] = centroid(moved("l-ear-scale-incr")); lm["earR"] = centroid(moved("r-ear-scale-incr"))
lm["cheekL"] = centroid(moved("l-cheek-bones-incr")); lm["cheekR"] = centroid(moved("r-cheek-bones-incr"))
lm["jawL"] = centroid(moved("chin-jaw-drop-incr", "L")); lm["jawR"] = centroid(moved("chin-jaw-drop-incr", "R"))
lm["neck"] = [0, 0.589, 0.0068]
lm["headTop"] = max(new_pos, key=lambda p: p[1])
ys = [p[1] for p in new_pos]
print("landmarks:", {k: [round(x, 3) for x in v] for k, v in lm.items()}, file=sys.stderr)

json.dump({
    "pos": [round(x, 5) for p in new_pos for x in p],
    "uv": [round(x, 5) for u in new_uv for x in u],
    "idx": tris,
    "eyeJoints": eye_joint,
    "modifiers": modifiers,
    "macro": macro,
    "targets": toffsets,
    "landmarks": lm,
}, open(os.path.join(out, "head.json"), "w"), separators=(",", ":"))
open(os.path.join(out, "targets.bin"), "wb").write(blob)
