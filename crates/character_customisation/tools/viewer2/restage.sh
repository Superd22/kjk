#!/bin/zsh
# Copies index.html into the served folder with a fresh build number, for edits between full serve_local.sh runs.
S=${1:-/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-local}
sed "s/{{build}}/$(date +%s)/g" ${0:A:h}/index.html > $S/viewer2/index.html
