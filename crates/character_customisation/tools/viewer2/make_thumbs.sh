#!/bin/zsh
# Renders the portrait sheet of head viewer 2 (thumbs/thumbs.webp and thumbs/thumbs.json) with headless Chrome.
# The local server must be running at http://127.0.0.1:8765 with viewer2 staged (serve_local.sh). Rerun after the default paint or the fits change.
set -e
HERE=${0:A:h}
PROFILE=$(mktemp -d)
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new --user-data-dir=$PROFILE --no-first-run --mute-audio \
  --use-angle=swiftshader --enable-unsafe-swiftshader --window-size=400,400 --virtual-time-budget=600000 \
  --dump-dom "http://127.0.0.1:8765/viewer2/index.html?thumbs=1" 2>/dev/null > $PROFILE/dom.html
rm -rf $PROFILE/Default
python3 -I - $PROFILE/dom.html $HERE/thumbs <<'PY'
import base64, html, json, re, sys
dom = open(sys.argv[1]).read()
m = re.search(r'<pre id="thumbs-out">(.*?)</pre>', dom, re.S)
if not m: sys.exit("no thumbnails in the page; check the console of ?thumbs=1")
d = json.loads(html.unescape(m.group(1)))
img = d.pop("image")
open(sys.argv[2] + "/thumbs.webp", "wb").write(base64.b64decode(img.split(",", 1)[1]))
d["file"] = "thumbs.webp"
json.dump(d, open(sys.argv[2] + "/thumbs.json", "w"))
print("wrote", len(d["heads"]), "portraits with paint", d["paint"])
PY
rm -rf $PROFILE
