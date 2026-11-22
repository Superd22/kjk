"""Writes walls of KOTOR's eyes sorted by one eye measure, to check that the order matches what the eye sees. Glue only.

eyes_sorted.py <measure json> <eye picture dir> <out dir> <key>..."""
import json, subprocess, sys, os
m = json.load(open(sys.argv[1]))
pics, out = sys.argv[2], sys.argv[3]
here = os.path.dirname(os.path.abspath(__file__))
for key in sys.argv[4:]:
    rows = [(v[key], n) for n, v in m.items() if v and v.get(key) is not None]
    rows.sort()
    names = [f"{n}={n} {val:.1f}" for val, n in rows]
    subprocess.run([sys.executable, "-I", f"{here}/eyes_wall.py", f"{out}/sorted_{key}.jpg", "6", f"{pics}:eyes", "--", *names], check=True)
