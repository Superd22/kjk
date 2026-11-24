#!/bin/zsh
# Rebuilds the Likeness HQ site and refreshes the folder that the local web server serves.
# The server itself (python3 -m http.server 8765 --bind 127.0.0.1) is never stopped or started here.
# Usage: serve_local.sh [served folder]
set -e
HQ=${0:A:h}
SERVED=${1:-/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-local}
DATA=/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data
cd $HQ && ../.venv/bin/python -I build_site.py
# Replace the contents of the served folder, but keep the folder itself, because the server holds it open.
mkdir -p $SERVED
find $SERVED -mindepth 1 -maxdepth 1 -exec rm -rf {} +
cp -R $HQ/site/. $SERVED/
mkdir -p $SERVED/viewer
ln -s $HQ/viewer/viewer.js $SERVED/viewer/viewer.js
ln -s $DATA $SERVED/viewer/data
sed "s/{{build}}/$(date +%s)/" $HQ/viewer/standalone.html > $SERVED/viewer/index.html
# Head viewer 2 (the character creator preview). Its scripts, fit numbers and portraits are symlinks, so edits show on a reload.
mkdir -p $SERVED/viewer2
for f in viewer2.js heads.js fit.json thumbs sliders.js viewer_map.bin paint_live.js; do ln -s $HQ/viewer2/$f $SERVED/viewer2/$f; done
ln -s $DATA $SERVED/viewer2/data
sed "s/{{build}}/$(date +%s)/g" $HQ/viewer2/index.html > $SERVED/viewer2/index.html
echo "served folder refreshed: http://127.0.0.1:8765/"
