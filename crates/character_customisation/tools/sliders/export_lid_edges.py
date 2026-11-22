"""Writes the lid edge of each eye for the paint session (light work only: reading existing data).

For each eye: the vertices round the opening in order, as indices into the 4,750-vertex mesh, each one's lid and its share along the lid from
the inner corner, the corners, and how the eyeball's centre and radius are found on any head.

Usage, from ~/.claude/tools/likeness-fit: .venv/bin/python -I sliders/export_lid_edges.py
"""
import json, os, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "scan", "trials-2026-11-18"), os.path.join(HERE, "..", "scan"), os.path.join(HERE, "..")]
from kotor_fit import setup
from kotor_eyes import edge_frame
from head import EYE_RADIUS

DATA = os.path.expanduser("~/.claude/tools/likeness-fit-data")


def main():
    T = np.load(os.path.join(DATA, "flame", "targets.npz"))
    head, mesh, rig, corr, skin = setup(T)
    out = {"vertices": int(len(rig.base)), "eyeball_radius_m": float(EYE_RADIUS), "eyes": {}}
    for side in "LR":
        loop, lid, t, c, sign = edge_frame(head, mesh, rig.base, side)
        first = mesh.first[loop]
        copies = [np.nonzero(mesh.group == g)[0].tolist() for g in loop]
        out["eyes"][side] = {
            "edge": [int(v) for v in first],
            "edge_copies": copies,
            "lid": [("upper", "lower")[int(k)] for k in lid],
            "share_from_inner_corner": [round(float(s), 4) for s in t],
            "inner_corner": int(first[np.argmin(np.where(lid == 0, t, 9))]),
            "outer_corner": int(first[np.argmax(np.where(lid == 0, t, -9))]),
            "eyeball_centre_vertices": [int(j) for j in head.eye_joints[side]],
            "eyeball_centre_on_base_m": [float(x) for x in c],
            "temple_is_plus_x": bool(sign > 0),
        }
    path = os.path.join(DATA, "sliders", "lid_edges.json")
    json.dump(out, open(path, "w"), indent=1)
    for side, e in out["eyes"].items():
        print(side, len(e["edge"]), "edge vertices", e["lid"].count("upper"), "upper", e["lid"].count("lower"), "lower", "corners", e["inner_corner"], e["outer_corner"],
              "copies max", max(len(c) for c in e["edge_copies"]), "centre", [round(x, 4) for x in e["eyeball_centre_on_base_m"]], "centre vertices", len(e["eyeball_centre_vertices"]))
    print("wrote", path)


main()
