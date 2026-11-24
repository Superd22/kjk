#!/bin/zsh
# Builds the Likeness HQ site and stages it, with the head viewer and its data, in a folder the Artifact tool may publish from.
# Usage: stage.sh <viewer data dir> <staging dir>
# The viewer data comes from scan/trials-2026-11-18/kotor_viewer_export.py. Publish <staging dir>/index.html with root <staging dir>.
set -e
HQ=${0:A:h}
cd $HQ && ../.venv/bin/python -I build_site.py
rm -rf $2 && cp -R $HQ/site $2
mkdir -p $2/viewer/data && cp -R $1/. $2/viewer/data/ && cp $HQ/viewer/viewer.js $2/viewer/
echo "staged $(find $2 -type f | wc -l | tr -d ' ') files, $(du -sh $2 | cut -f1)"
