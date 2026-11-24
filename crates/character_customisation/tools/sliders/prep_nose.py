"""Prepares the inputs of the nose slider bench (the Rust crate head_sliders, tool nose_bench), once.

It writes, under likeness-fit-data/sliders/input/:
- the base head, its triangles, the vertices the smoothing keeps, the merged vertices and how the hidden insides (nostrils, mouth)
  follow the skin, and each vertex's zone;
- for each of the 57 KOTOR test heads, in its fit's frame (metres): the stand-in before smoothing (the saved fit with its neck rim on
  KOTOR's ring, which the smoothing turns into fits_v6_smooth), our mesh wrapped onto the KOTOR head (only used to learn the shape
  sliders), KOTOR's own face mesh, and the eye centres;
- extra KOTOR meshes that are copies or near copies of a test head, lined up on it, for the measure's check that copies score alike.

Usage: .venv/bin/python -I sliders/prep_nose.py
"""
import json, os, sys
import numpy as np
LF = os.path.expanduser("~/.claude/tools/likeness-fit")
T18 = os.path.join(LF, "scan", "trials-2026-11-18")
sys.path[:0] = [LF, os.path.join(LF, "scan"), T18]
from scipy.spatial import cKDTree
import kotor_fit as KF
import kotor_neck as KN
from kotor_shape_smooth import Builder
from wrap import similarity

DATA = os.path.expanduser("~/.claude/tools/likeness-fit-data")
K = os.path.join(DATA, "kotor-heads")
OUT = os.path.join(DATA, "sliders", "input")
VIEW = "/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data"
manifest = {}


def put(name, a, dtype):
    a = np.ascontiguousarray(np.asarray(a).astype(dtype))
    path = os.path.join(OUT, name + ".bin")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    a.tofile(path)
    manifest[name] = {"dtype": np.dtype(dtype).name, "shape": list(a.shape)}


def read_obj(path, wanted):
    """A KOTOR face mesh as kotor_scan.py reads it: KOTOR's axes turned so y is up and the face looks at +z, copies at one place merged."""
    v, faces, current = [], [], None
    for line in open(path):
        if line.startswith("v "):
            v.append([float(x) for x in line.split()[1:4]])
        elif line.startswith(("o ", "g ")):
            current = line.split()[1].lower()
        elif line.startswith("f ") and current in wanted:
            idx = [int(t.split("/")[0]) - 1 for t in line.split()[1:]]
            faces += [[idx[0], idx[k], idx[k + 1]] for k in range(1, len(idx) - 1)]
    v, f = np.array(v), np.array(faces)
    used = np.unique(f)
    v, f = KN.turn(v[used]), np.searchsorted(used, f)
    _, first, inv = np.unique(np.round(v, 6), axis=0, return_index=True, return_inverse=True)
    v, f = v[first], inv.ravel()[f]
    return v, f[(f[:, 0] != f[:, 1]) & (f[:, 1] != f[:, 2]) & (f[:, 2] != f[:, 0])]


def icp(src, dst, rounds=40):
    """Rigid alignment of src's points onto dst's points, nearest point to nearest point, ignoring the worst 20% of the pairs."""
    tree = cKDTree(dst)
    s, R, t = 1.0, np.eye(3), np.zeros(3)
    for _ in range(rounds):
        p = src @ R.T + t
        d, j = tree.query(p)
        ok = d <= np.percentile(d, 80)
        _, R2, t2 = similarity(p[ok], dst[j[ok]], scale=False)
        R, t = R2 @ R, t @ R2.T + t2
    return R, t


def main():
    build = Builder()
    head, base = build.sm, build.base
    from head import Head
    from wrap import Mesh
    from lips_photo import Rig
    H = Head(os.path.join(DATA, "likeness"))
    mesh = Mesh(H)
    rig = Rig(H, mesh, np.load(os.path.join(DATA, "flame", "targets.npz")))
    KF.fit.head, KF.fit.mesh = H, mesh
    corr = {o["name"]: o["vertex"] for o in json.load(open(os.path.join(LF, "correspondence.json")))["landmarks"]}
    skin = ~rig.hidden[mesh.group]
    put("base", rig.base, np.float64)
    put("tris", H.tris, np.uint32)
    put("keep", build.keep, np.uint8)
    put("group", mesh.group, np.uint32)
    put("first", mesh.first, np.uint32)
    put("hidden", rig.hidden, np.uint8)
    put("near", rig.near, np.uint32)
    put("near_w", rig.near_w, np.float64)
    put("skin", skin, np.uint8)
    z = KF.zones(H, rig.base, skin, corr)
    put("zone", [0 if x is None else KF.ZONES.index(x) + 1 for x in z], np.uint8)
    e = H.eye_centres(rig.base)
    put("base_eyes", np.array([e["L"], e["R"]]), np.float64)
    put("landmarks", np.array([corr[k] for k in ("noseBridge", "noseTip", "noseBase", "alaL", "alaR", "lipLine", "chin")]), np.uint32)

    common = json.load(open(os.path.join(VIEW, "common.json")))
    titles = {h["name"]: h.get("title") or "" for h in common["heads"]}
    names = sorted(os.path.basename(f)[:-8] for f in os.listdir(os.path.join(K, "fits_v6")) if f.endswith("_fit.npz"))
    heads = []
    worst = 0.0
    for name in names:
        F = np.load(os.path.join(K, "fits_v6", name + "_fit.npz"))
        FS = np.load(os.path.join(K, "fits_v6_smooth", name + "_fit.npz"))
        ring = KN.ring_in_fit(name, os.path.join(K, "fits_v6"))
        pre = build.place(F["pos"], ring)
        check = build.sm.smooth(pre, "implicit", 3.0)
        worst = max(worst, float(np.abs(check - FS["pos"]).max()))
        s, R, t = float(F["s"]), F["R"], F["t"]
        wf = os.path.join(K, "wrap_v5", name + "_wrapped.npz")
        true, _, on_scan = KF.target(rig, H, wf)
        W = np.load(wf)
        sc = np.load(wf.replace("_wrapped.npz", "_scan.npz"))
        nv = len(sc["v"])
        kv = W["scan_v"][:nv] * s @ R.T + t
        put(f"heads/{name}/standin_pre", pre, np.float32)
        put(f"heads/{name}/fit_smooth", FS["pos"], np.float32)
        put(f"heads/{name}/wrap", true * s @ R.T + t, np.float32)
        put(f"heads/{name}/on_scan", on_scan, np.uint8)
        put(f"heads/{name}/kotor_v", kv, np.float64)
        put(f"heads/{name}/kotor_f", sc["f"], np.uint32)
        put(f"heads/{name}/eyes", F["eyes"], np.float64)
        heads.append({"name": name, "title": titles.get(name, ""), "mod": name.startswith("mod_"), "female": name in KN.FEMALE,
                      "player": name in KN.PLAYER})
    print(f"stand-ins: smoothing them again gives the saved smoothed fits to within {worst * 1000:.4f} mm")

    # Copies and near copies of test heads, for the measure's third check.
    twins = [("pmhc01", "k2", "pmhc01"), ("pmha01", "k2", "pmha01"), ("pmha05", "k2", "pmha05"), ("pmhc03", "k2", "pmhc03"),
             ("pfha02", "k2", "pfha02"), ("pfhb04", "k2", "pfhb04"), ("pfhc02", "k2", "pfhc02"), ("pfha05", "k2", "pfha05"),
             ("pmhc04", "k2", "pmhc04"), ("old_a_f", "k2", "old_a_f"), ("old_w_f", "k2", "old_w_f"), ("n_traskh", "k2", "n_traskh"),
             ("sith_app", "k2", "sith_app"), ("comm_a_m", "k1", "n_shaardanh"), ("comm_a_m", "k1", "n_tatcoma_m"),
             ("comm_w_m", "k1", "n_mekelh"), ("old_a_m", "k1", "n_jorakh"), ("comm_a_f", "k1", "n_tatcomw_f")]
    out = []
    for sib, game, other in twins:
        path = os.path.join(K, game, "obj", other + ".obj")
        if not os.path.exists(path):
            print("missing", path)
            continue
        v, f = read_obj(path, {"head", "head_geo", "head2"})
        if len(f) == 0:
            print("no face mesh in", path)
            continue
        wf = os.path.join(K, "wrap_v5", sib + "_wrapped.npz")
        W, sc = np.load(wf), np.load(wf.replace("_wrapped.npz", "_scan.npz"))
        to_fit = KN.model_to_fit(sib, os.path.join(K, "fits_v6"))
        # line the copy up on the test head's own mesh in KOTOR's model space, then carry it as the test head is carried
        R, t = icp(v[v[:, 2] > np.percentile(v[:, 2], 40)], sc["v"])
        vf = to_fit(v @ R.T + t)
        d = cKDTree(to_fit(sc["v"])).query(vf)[0]
        key = f"{other}_{game}" if other == sib else other
        put(f"twins/{key}/kotor_v", vf, np.float64)
        put(f"twins/{key}/kotor_f", f, np.uint32)
        out.append({"name": key, "of": sib, "game": game, "model": other, "median_gap_mm": float(np.median(d) * 1000),
                    "same_topology": bool(len(v) == len(sc["v"]) and len(f) == len(sc["f"]))})
        print(f"{key:16s} on {sib:12s}: median gap {np.median(d) * 1000:.3f} mm, {len(v)} against {len(sc['v'])} vertices")

    # the viewer's 5,852 vertices, each a copy of one of our 4,750
    FS = np.load(os.path.join(K, "fits_v6_smooth", "p_bastilah_fit.npz"))["pos"]
    import base64
    h = json.load(open(os.path.join(VIEW, "heads", "p_bastilah.json")))
    vp = np.frombuffer(base64.b64decode(h["ours"]["pos"]), np.float32).reshape(-1, 3)
    d, vmap = cKDTree(FS).query(vp)
    print(f"viewer vertices: {len(vp)}, farthest from their copy {d.max() * 1000:.4f} mm")
    put("viewer_map", vmap, np.uint32)

    json.dump({"arrays": manifest, "heads": heads, "twins": out, "zones": KF.ZONES}, open(os.path.join(OUT, "manifest.json"), "w"), indent=1)
    print("wrote", OUT)


if __name__ == "__main__":
    main()
