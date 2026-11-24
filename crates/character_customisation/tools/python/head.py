"""Loads the MakeHuman head exported for the Likeness Lab and deforms it exactly as the page does."""
import base64, json, os, re, struct
import numpy as np

Q = 1 / 50000  # metres per quantised delta unit
EYE_RADIUS = 0.0123
REGIONS = ("african", "asian", "caucasian")

# The share code version and the paint bytes the page expects after the modifiers, in SURF_DEF order, at the page's defaults.
# read_page() replaces both with what a given copy of the page says.
VERSION = 4
MODIFIERS_IN = {1: 180, 2: 180, 3: 192}  # how many sliders each older version stored
SURF_DEF = [
    ("undertone", "s", 0), ("shading", "u", 0.5), ("blush", "u", 0.2), ("freckles", "u", 0), ("foreheadLines", "u", 0), ("crowsFeet", "u", 0), ("nasolabial", "u", 0), ("underEyeDark", "u", 0.15),
    ("moleCount", "i", 0), ("moleSeed", "i", 7), ("scarSide", "i", 0), ("scarX", "u", 0.5), ("scarY", "u", 0.55), ("scarAngle", "u", 0.3), ("scarLength", "u", 0.5),
    ("eye", "i", 0), ("irisSize", "u", 0.5), ("limbalRing", "u", 0.5), ("scleraTone", "u", 0.1), ("linerTop", "u", 0.5), ("linerBottom", "u", 0.2), ("lashes", "u", 0.4),
    ("browColor", "i", 1), ("browInner", "s", 0), ("browOuter", "s", 0), ("browArchPos", "s", 0), ("browArch", "s", 0), ("browThickIn", "u", 0.5), ("browThickOut", "u", 0.4), ("browLength", "s", 0), ("browSpacing", "s", 0), ("browDensity", "u", 0.85), ("browSparse", "u", 0),
    ("lip", "i", 0), ("lipTint", "u", 0.3), ("lipUpperThick", "u", 0.5), ("lipLowerThick", "u", 0.5), ("bowDepth", "u", 0.4), ("bowWidth", "u", 0.5), ("lipWidth", "s", 0), ("lipEdge", "u", 0.3), ("cornerDark", "u", 0.3), ("lipHighlight", "u", 0.3),
    ("hairPaint", "i", 1), ("hairline", "s", 0), ("temples", "u", 0.2), ("widowsPeak", "u", 0), ("hairGrey", "u", 0), ("hairDensity", "u", 1),
    ("beardMoustache", "u", 0), ("beardChin", "u", 0), ("beardCheeks", "u", 0), ("beardSideburns", "u", 0), ("beardNeck", "u", 0), ("beardDensity", "u", 0.6), ("beardGrey", "u", 0),
    ("pupilSize", "u", 0.5),
]


class Head:
    def __init__(self, data_dir):
        meta = json.load(open(os.path.join(data_dir, "head.json")))
        bin_path = os.path.join(data_dir, "targets.bin")
        blob = open(bin_path, "rb").read() if os.path.exists(bin_path) else base64.b64decode(open(os.path.join(data_dir, "targets.b64.txt")).read())
        self.base = np.array(meta["pos"], dtype=np.float64).reshape(-1, 3)
        self.n = len(self.base)
        self.tris = np.array(meta["idx"], dtype=np.int64).reshape(-1, 3)
        self.eye_joints = {s: np.array(meta["eyeJoints"][s]) for s in ("L", "R")}
        self.modifiers = meta["modifiers"]
        self.macro_index = meta["macro"]
        self.landmarks = meta["landmarks"]
        self.target_names = [t["name"] for t in meta["targets"]]
        self.targets = []  # (vertex indices, deltas in metres)
        for t in meta["targets"]:
            off = t["offset"]
            count = struct.unpack_from("<I", blob, off)[0]
            idx = np.frombuffer(blob, dtype="<u2", count=count, offset=off + 4).astype(np.int64)
            d_off = off + 4 + count * 2 + (2 if count % 2 else 0)
            d = np.frombuffer(blob, dtype="<i2", count=count * 3, offset=d_off).reshape(-1, 3).astype(np.float64) * Q
            self.targets.append((idx, d))
        self.by_name = {m["name"]: i for i, m in enumerate(self.modifiers)}

    def dense(self, ti):
        """The full per-vertex delta of one target."""
        idx, d = self.targets[ti]
        out = np.zeros((self.n, 3))
        np.add.at(out, idx, d)
        return out

    def macro_weights(self, gender, age, regions):
        w = {}
        g = (gender + 1) / 2
        es = sum(regions) or 1
        for sex, ws in (("female", 1 - g), ("male", g)):
            for ag, wa in (("young", 1 - age), ("old", age)):
                k = ws * wa
                if k < 1e-4:
                    continue
                w[self.macro_index[f"universal-{sex}-{ag}"]] = w.get(self.macro_index[f"universal-{sex}-{ag}"], 0) + k
                for r, rw in zip(REGIONS, regions):
                    ti = self.macro_index[f"{r}-{sex}-{ag}"]
                    w[ti] = w.get(ti, 0) + k * rw / es
        return w

    def target_weights(self, values, gender=0.0, age=0.0, regions=(1 / 3, 1 / 3, 1 / 3), exag=1.0):
        w = self.macro_weights(gender, age, regions)
        for i, m in enumerate(self.modifiers):
            v = values[i] * exag
            if v > 0 and m["pos"] >= 0:
                w[m["pos"]] = w.get(m["pos"], 0) + v
            elif v < 0 and m["neg"] >= 0:
                w[m["neg"]] = w.get(m["neg"], 0) - v
        return w

    def deform(self, weights):
        pos = self.base.copy()
        for ti, wt in weights.items():
            if wt:
                idx, d = self.targets[ti]
                np.add.at(pos, idx, d * wt)
        return pos

    def eye_centres(self, pos):
        return {s: pos[j].mean(axis=0) for s, j in self.eye_joints.items()}


def read_page(path):
    """Takes the share code version and the paint defaults from the Likeness Lab page source."""
    global VERSION, SURF_DEF
    html = open(path, encoding="utf-8").read()
    VERSION = int(re.search(r"function encode\(\) \{\s*const b = \[(\d+),", html).group(1))
    block = html[html.index("const SURF_DEF = ["):]
    block = block[:block.index("];")]
    SURF_DEF = [(k, kind, float(d)) for k, kind, d in re.findall(r"\['(\w+)', '([usi])', (-?[\d.]+)\]", block)]
    SURF_DEF = [(k, kind, int(d) if kind == "i" else d) for k, kind, d in SURF_DEF]


def share_code(values, gender, age, regions, skin=1, hair=1, symmetry=True):
    es = sum(regions) or 1
    b = [VERSION, skin, hair, 1 if symmetry else 0, round(gender * 127) + 128, round(age * 255)] + [round(r / es * 255) for r in regions]
    b += [round(max(-1, min(1, v)) * 127) + 128 for v in values]
    for _, kind, d in SURF_DEF:
        b.append(d if kind == "i" else round(d * 255) if kind == "u" else round(d * 127) + 128)
    return base64.b64encode(bytes(b)).decode().rstrip("=")


def read_share_code(code, n_modifiers):
    values, gender, age, regions, _ = read_share_code_full(code, n_modifiers)
    return values, gender, age, regions


def read_share_code_full(code, n_modifiers):
    """Also returns the paint numbers. Sliders the code does not store stay at 0."""
    s = code.strip()
    b = base64.b64decode(s + "=" * (-len(s) % 4))
    if b[0] not in (1, 2, 3):
        raise ValueError("unknown share code version")
    stored = MODIFIERS_IN.get(b[0], n_modifiers)
    gender, age = (b[4] - 128) / 127, b[5] / 255
    regions = (b[6] / 255, b[7] / 255, b[8] / 255)
    values = np.zeros(n_modifiers)
    for i in range(min(stored, n_modifiers)):
        values[i] = (b[9 + i] - 128) / 127
    surf, k = {}, 9 + stored
    for key, kind, d in SURF_DEF:
        v = b[k] if k < len(b) else None
        surf[key] = d if v is None else (v if kind == "i" else v / 255 if kind == "u" else (v - 128) / 127)
        k += 1
    return values, gender, age, regions, surf
