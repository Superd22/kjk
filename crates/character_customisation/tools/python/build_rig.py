"""Rigs the Likeness Lab head with KOTOR's face bones, for playing KOTOR's own face clips on it.

The bones keep KOTOR's names, hierarchy and rest rotations. Their rest positions are KOTOR's, mapped onto this head through
matching features (eyes, lips, mouth corners, brows, cheeks), so a clip's position offsets and absolute rotations apply as they do in the game.
The bones are stored in KOTOR's axes (+Y forward, +Z up, the character's left at -X). TO_OUR turns them into the head's axes.

Usage: python -I build_rig.py <head data dir> <kotor rest json or bundle.json> <out rig.json>
"""
import json, math, os, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from head import Head

TO_OUR = np.array([[-1, 0, 0], [0, 0, 1], [0, 1, 0]], dtype=float)  # its own inverse
FACE = ["necklwr_g", "neck_g", "hturn_g", "head_g", "talkdummy", "f_um_g", "f_lmc_g", "f_rmc_g", "f_jaw_g", "f_llm_g", "f_rlm_g", "f_tonguetip_g",
        "f_mdbrw_g", "f_lbrw_g", "f_rbrw_g", "f_llweye_g", "f_rlweye_g", "f_lns_g", "f_rns_g", "eyela", "eyera", "eyellid", "eyerlid"]
MOUTH_Y = 0.6615  # the line where the lips meet, in metres
BLINK_DEG = 29.0  # KOTOR's blink turns the lid from 1.4 to about 30 degrees


def smooth(a, b, t):
    x = np.clip((np.asarray(t, float) - a) / (b - a), 0, 1)
    return x * x * (3 - 2 * x)


def qmat(q):
    x, y, z, w = q
    return np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                     [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                     [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])


def kotor_world(nodes):
    """World position and rotation of each node in the KOTOR head model's space."""
    by = {n["name"]: n for n in nodes}
    out = {}

    def get(name):
        if name in out:
            return out[name]
        n = by[name]
        if n["parent"] in by:
            pp, pr = get(n["parent"])
        else:
            pp, pr = np.zeros(3), np.eye(3)
        out[name] = (pp + pr @ np.array(n["pos"]), pr @ qmat(n["rot"]))
        return out[name]
    for name in by:
        get(name)
    return out


def fit_map(kw, h):
    """Per-axis scale and offset from KOTOR head space (turned into our axes) to our head, fitted on matching features."""
    L = h.landmarks
    e = h.eye_centres(h.base)
    lo = np.array(L["lipLower"])
    pairs = [("eyela", e["L"]), ("eyera", e["R"]), ("f_um_g", L["lipUpper"]), ("f_lmc_g", L["cornerL"]), ("f_rmc_g", L["cornerR"]),
             ("f_llm_g", lo + [0.013, 0, 0]), ("f_rlm_g", lo - [0.013, 0, 0]), ("f_lbrw_g", L["browL"]), ("f_rbrw_g", L["browR"]),
             ("f_lns_g", L["cheekL"]), ("f_rns_g", L["cheekR"])]
    src = np.array([TO_OUR @ kw[k][0] for k, _ in pairs])
    dst = np.array([v for _, v in pairs], float)
    scale, off = np.zeros(3), np.zeros(3)
    for a in range(3):
        A = np.c_[src[:, a], np.ones(len(src))]
        (s, t), *_ = np.linalg.lstsq(A, dst[:, a], rcond=None)
        if a == 0:  # the head is symmetric, so x only scales
            s, t = (src[:, 0] @ dst[:, 0]) / (src[:, 0] @ src[:, 0]), 0.0
        scale[a], off[a] = s, t
    res = {k: float(np.linalg.norm(src[i] * scale + off - dst[i]) * 1000) for i, (k, _) in enumerate(pairs)}
    return scale, off, res


class Layers:
    """Skin weights painted in layers. Painting a bone with alpha a scales what is there by 1 - a and gives the bone a."""

    def __init__(self, n, bones):
        self.bones = bones
        self.w = np.zeros((n, len(bones)))
        self.w[:, bones.index("head_g")] = 1

    def paint(self, bone, alpha):
        a = np.clip(alpha, 0, 1)[:, None]
        self.w *= 1 - a
        self.w[:, self.bones.index(bone)] += a[:, 0]


def regions(h, pos):
    """The weight painting, in the head's own axes and metres."""
    L = h.landmarks
    x, y, z = pos[:, 0], pos[:, 1], pos[:, 2]
    ax = np.abs(x)
    e = h.eye_centres(h.base)
    cx = L["cornerL"][0]
    front = smooth(0.045, 0.085, z)  # in front of the jaw's hinge
    out = {}
    out["necklwr"] = 1 - smooth(0.54, 0.575, y)
    out["neck"] = 1 - smooth(0.60, 0.645, y + 0.25 * np.clip(z - 0.06, 0, None))
    # The jaw: everything below the mouth line in front of the hinge. The cut is sharp across the lips and soft over the cheeks.
    width = 0.0015 + 0.02 * smooth(cx - 0.004, cx + 0.025, ax)
    centre = MOUTH_Y - 0.012 * smooth(cx, cx + 0.03, ax)
    below = 1 - smooth(centre - width, centre + width, y)
    chin_under = smooth(0.585, 0.63, y + 0.4 * np.clip(z - 0.10, 0, None))
    out["jaw"] = below * front * chin_under
    lipzone = smooth(0.125, 0.14, z) * (1 - smooth(cx + 0.002, cx + 0.012, ax))
    out["lower"] = lipzone * (y < MOUTH_Y) * (1 - smooth(MOUTH_Y - 0.012, MOUTH_Y - 0.004, MOUTH_Y - y))
    out["upper"] = lipzone * (y >= MOUTH_Y) * (1 - smooth(MOUTH_Y + 0.010, MOUTH_Y + 0.017, y))
    dc = {s: np.sqrt((x - sg * cx) ** 2 + ((y - L["cornerL"][1]) * 1.3) ** 2) for s, sg in (("L", 1), ("R", -1))}
    for s in "LR":
        out["corner" + s] = np.exp(-(dc[s] / 0.009) ** 2) * smooth(0.10, 0.125, z)
    for s, sg in (("L", 1), ("R", -1)):
        b = L["brow" + s]
        out["brow" + s] = np.exp(-(((x - b[0]) / 0.022) ** 2 + ((y - b[1]) / 0.013) ** 2)) * smooth(0.10, 0.12, z) * smooth(e[s][1] + 0.004, e[s][1] + 0.012, y)
        c = L["cheek" + s]
        out["cheek" + s] = np.exp(-(((x - c[0]) / 0.02) ** 2 + ((y - c[1] + 0.008) / 0.02) ** 2)) * smooth(0.08, 0.10, z) * 0.8
        r_eye = np.linalg.norm(pos - e[s], axis=1)
        out["under" + s] = np.exp(-(((x - e[s][0]) / 0.016) ** 2 + ((y - e[s][1] + 0.021) / 0.009) ** 2)) * smooth(0.10, 0.115, z) * 0.9 * smooth(0.0145, 0.019, r_eye)
    out["browM"] = np.exp(-((x / 0.012) ** 2 + ((y - L["browL"][1] + 0.004) / 0.012) ** 2)) * smooth(0.12, 0.135, z)
    return out


def lid_weights(h, pos, side):
    """The upper lid's skin, which turns around the eyeball's centre. The shape follows eye_targets.py's upper lid."""
    from eye_targets import EYE_R
    c = h.eye_centres(h.base)[side]
    sg = 1.0 if side == "L" else -1.0
    q = (pos - c) * [sg, 1, 1]
    r = np.linalg.norm(q, axis=1)
    d = r - EYE_R
    region = (r > 0.008) & (r < 0.032) & (q[:, 2] > -0.012)
    margin = region & (r < 0.0145) & (q[:, 2] > 0)
    xi, xo = q[margin, 0].min(), q[margin, 0].max()
    yi, yo = q[margin][q[margin, 0].argmin(), 1], q[margin][q[margin, 0].argmax(), 1]
    t = np.clip((q[:, 0] - xi) / (xo - xi), 0, 1)
    upper = q[:, 1] > yi + (yo - yi) * t
    w = region * smooth(0.009, 0.0, d) * np.sin(np.pi * t) ** 0.6 * upper
    # how far the upper lid must turn to meet the lower one, about the eye's sideways axis
    top = np.argmax(np.where(margin & upper, q[:, 1], -1))
    bot = np.argmin(np.where(margin & ~upper, q[:, 1], 1))
    close = math.degrees(math.atan2(q[top, 1], q[top, 2]) - math.atan2(q[bot, 1], q[bot, 2]))
    return w, close


def build(head_dir, rest_path):
    h = Head(head_dir)
    src = json.load(open(rest_path))
    nodes = src["head"]["nodes"] if "head" in src else src["nodes"]
    kw = kotor_world(nodes)
    scale, off, res = fit_map(kw, h)
    e = h.eye_centres(h.base)
    world_our = {k: scale * (TO_OUR @ kw[k][0]) + off for k in FACE}
    for k, s in (("eyela", "L"), ("eyera", "R"), ("eyellid", "L"), ("eyerlid", "R")):
        world_our[k] = e[s]
    by = {n["name"]: n for n in nodes}
    bones = []
    for k in FACE:
        n = by[k]
        parent = n["parent"] if n["parent"] in FACE else ""
        wk = TO_OUR @ world_our[k]
        if parent:
            pk = TO_OUR @ world_our[parent]
            local = kw[parent][1].T @ (wk - pk)
        else:
            local = wk
        bones.append({"name": k, "parent": parent, "world": [round(float(v), 5) for v in wk], "local": [round(float(v), 5) for v in local], "rot": [round(float(v), 5) for v in n["rot"]]})
    names = [b["name"] for b in bones]
    lay = Layers(h.n, names)
    R = regions(h, h.base)
    lay.paint("necklwr_g", R["necklwr"])
    lay.paint("neck_g", R["neck"] * (1 - R["necklwr"]))
    lay.paint("f_jaw_g", R["jaw"])
    lay.paint("f_llm_g", R["lower"] * smooth(-0.006, 0.006, h.base[:, 0]))
    lay.paint("f_rlm_g", R["lower"] * (1 - smooth(-0.006, 0.006, h.base[:, 0])))
    lay.paint("f_um_g", R["upper"])
    lay.paint("f_lmc_g", R["cornerL"])
    lay.paint("f_rmc_g", R["cornerR"])
    for s, bs in (("L", "l"), ("R", "r")):
        lay.paint(f"f_{bs}ns_g", R["cheek" + s])
        lay.paint(f"f_{bs}lweye_g", R["under" + s])
        lay.paint(f"f_{bs}brw_g", R["brow" + s])
    lay.paint("f_mdbrw_g", R["browM"])
    lids = {}
    for s, b in (("L", "eyellid"), ("R", "eyerlid")):
        w, close = lid_weights(h, h.base, s)
        lay.paint(b, w)
        lids[b] = round(close * 0.97 / BLINK_DEG, 3)
    for s, b in (("L", "eyela"), ("R", "eyera")):
        joints = h.eye_joints[s]
        a = np.zeros(h.n)
        a[joints] = 1
        lay.paint(b, a)
    W = lay.w
    # at most four bones a vertex, as KOTOR's skins
    order = np.argsort(-W, axis=1)[:, :4]
    top = np.take_along_axis(W, order, 1)
    top /= top.sum(1, keepdims=True)
    head_i = names.index("head_g")
    skin = []
    for v in range(h.n):
        pairs = [(int(order[v, k]), round(float(top[v, k]), 3)) for k in range(4) if top[v, k] > 0.004]
        if len(pairs) == 1 and pairs[0][0] == head_i:
            continue
        pairs[0] = (pairs[0][0], round(1 - sum(w for _, w in pairs[1:]), 3))  # the rounded weights still sum to 1
        skin.append([v] + [x for p in pairs for x in p])
    inside = mouth_inside(h)
    return {"axes": "kotor", "toOur": TO_OUR.astype(int).tolist(), "keyScale": [round(float(s), 4) for s in np.abs(TO_OUR @ scale)],
            "defaultBone": head_i, "bones": bones, "lidScale": lids, "skin": skin, "mouthInside": inside}, res, scale


def mouth_inside(h):
    """The mouth bag's vertices, with how dark each one is painted (0 to 1).

    The bag is found by spreading over the mesh from its deepest point behind the lips, through vertices that do not face forward.
    It also passes vertices deep behind the lips, which hold the back of the bag. It stops at the lips' front, so the skin around the mouth stays as it is.
    """
    pos = h.base
    t = h.tris
    _, group = np.unique(np.round(pos, 4), axis=0, return_inverse=True)  # split copies of one vertex count as one
    group = group.ravel()
    ng = group.max() + 1
    gp = np.zeros((ng, 3))
    gp[group] = pos
    fn = np.cross(pos[t[:, 1]] - pos[t[:, 0]], pos[t[:, 2]] - pos[t[:, 0]])
    gn = np.zeros((ng, 3))
    for k in range(3):
        np.add.at(gn, group[t[:, k]], fn)
    gn /= np.maximum(np.linalg.norm(gn, axis=1, keepdims=True), 1e-12)
    nbr = [set() for _ in range(ng)]
    for tri in group[t]:
        for i in range(3):
            nbr[tri[i]].add(tri[(i + 1) % 3])
            nbr[tri[(i + 1) % 3]].add(tri[i])
    L = h.landmarks
    cx = L["cornerL"][0]
    near = (np.abs(gp[:, 0]) < cx + 0.006) & (np.abs(gp[:, 1] - MOUTH_Y) < 0.03) & (gp[:, 2] > 0.09)
    seeds = np.nonzero(near & (np.abs(gp[:, 0]) < 0.002) & (gn[:, 2] < -0.8))[0]
    deep = L["lipLower"][2] - 0.002  # the back of the bag faces forward too, but sits well behind the lips' front
    seen = np.zeros(ng, bool)
    stack = list(seeds)
    seen[seeds] = True
    while stack:
        g = stack.pop()
        for o in nbr[g]:
            if not seen[o] and near[o] and (gn[o, 2] < 0.3 or (gp[o, 2] < deep and abs(gp[o, 0]) < cx * 0.75 and abs(gp[o, 1] - MOUTH_Y) < 0.01)):
                seen[o] = True
                stack.append(o)
    dark = seen * np.maximum(smooth(0.3, -0.1, gn[:, 2]), smooth(deep + 0.002, deep - 0.002, gp[:, 2]))
    dv = dark[group]
    idx = np.nonzero(dv > 0.05)[0]
    return [[int(i), round(float(dv[i]), 2)] for i in idx]


def main(head_dir, rest_path, out):
    rig, res, scale = build(head_dir, rest_path)
    json.dump(rig, open(out, "w"), separators=(",", ":"))
    print("fit residuals (mm):", {k: round(v, 1) for k, v in res.items()})
    print("scale (our x, y, z):", np.round(scale, 3), "lid scale:", rig["lidScale"], "skinned vertices:", len(rig["skin"]), "mouth inside:", len(rig["mouthInside"]))


if __name__ == "__main__":
    main(*sys.argv[1:4])
