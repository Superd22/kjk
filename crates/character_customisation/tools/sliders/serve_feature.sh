#!/bin/zsh
# Gives one feature's conversation a viewer stack of its own, so that two conversations can work on two features at the same time
# without overwriting each other's browser module, slider data or page. It is the setup step of "Working in parallel on the head"
# in docs/research/2026-11-19-head-roadmap.md.
#
# Usage: serve_feature.sh <feature> <worktree> [port] [viewer2 source]
#   <feature>         the feature's key, as `sliders <feature>` takes it: nose, jaw, brow, cheeks, eyes, lips, skull.
#   <worktree>        the kjk worktree this conversation works in, such as ~/perso/kjk/.claude/worktrees/eyes-round8.
#   [port]            the port to serve on, or the first port to try. The script takes the next free port upward, because David's
#                     own viewer holds 8765 and the answers server holds 8766. The default start is 8770.
#   [viewer2 source]  where the page's scripts come from. The default is the shared hq/viewer2, whose files the HQ owns. Pass a copy
#                     of your own when your feature needs to change sliders.js, and hand the change to the HQ at merge time.
#
# What it builds, under hq/parallel/<feature>/:
#   data/   a slider data folder for `sliders <feature> <command> <data folder>`. `input/` and every shared file of `web/` are
#           symlinks to the real ones, so the shared inputs stay shared and current. Only `web/<feature>/` is a folder of its own,
#           seeded from the shared one, so this conversation's presets and pages do not touch anyone else's.
#   wasm/   where `build_wasm.sh <crate> <here>/curve/<crate> <worktree>` writes this conversation's browser modules.
#   served/ the symlink farm the server serves, with the four mount names the page's own paths expect: /viewer2 for the page,
#           /nose for the slider data, /wasm for the browser modules, and /lids and /paint for the lid edges and the paint.
#
# Nothing here is copied except `web/<feature>/`, so a second stack costs a few megabytes, not the 171 MB of the data folder.
set -e
FEATURE=${1:?usage: serve_feature.sh <feature> <worktree> [port] [viewer2 source]}
WT=${2:?usage: serve_feature.sh <feature> <worktree> [port] [viewer2 source]}
PORT=${3:-8770}
# Take the next free port upward, so several conversations can each run their own stack without agreeing on numbers.
while nc -z 127.0.0.1 $PORT 2>/dev/null; do PORT=$((PORT+1)); done
HQ=$HOME/.claude/tools/likeness-fit/hq
VSRC=${4:-$HQ/viewer2}
SHARED=$HOME/.claude/tools/likeness-fit-data/sliders
# The 829 MB of KOTOR reference meshes and textures the page draws. Shared, and read only here.
VIEWERDATA=/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data
ROOT=$HQ/parallel/$FEATURE

[[ -f $WT/rust-toolchain.toml ]] || { echo "serve_feature.sh: $WT is not a kjk worktree (no rust-toolchain.toml)" >&2; exit 1; }
[[ -d $SHARED/web ]] || { echo "serve_feature.sh: no shared slider data at $SHARED/web" >&2; exit 1; }

rm -rf $ROOT
mkdir -p $ROOT/data/web $ROOT/wasm/curve $ROOT/served/viewer2 $ROOT/served/nose

# The data folder: everything shared by symlink, this feature's own outputs as a real folder.
ln -s $SHARED/input $ROOT/data/input
for e in $SHARED/web/*(N); do
  n=${e:t}
  [[ $n == $FEATURE ]] && continue
  ln -s $e $ROOT/data/web/$n
done
mkdir -p $ROOT/data/web/$FEATURE
[[ -d $SHARED/web/$FEATURE ]] && cp -R $SHARED/web/$FEATURE/. $ROOT/data/web/$FEATURE/

# This conversation's own browser modules. The page loads head_sliders and head_eyes from /wasm/curve, and head_rig and
# head_paint_gpu from /wasm. The two under curve/ are copied, because this conversation rebuilds them; the rest are symlinks to the
# shared ones, and so is the loader.
for c in $HQ/wasm/curve/*(N/); do
  [[ ${c:t} == old_* ]] && continue
  cp -R $c $ROOT/wasm/curve/${c:t}
done
ln -s $HQ/wasm/load.js $ROOT/wasm/load.js
for c in $HQ/wasm/*(N/); do
  [[ ${c:t} == curve ]] && continue
  ln -s $c $ROOT/wasm/${c:t}
done

# The served tree, with the mount names the page's own paths expect.
for f in viewer2.js heads.js fit.json thumbs sliders.js viewer_map.bin paint_live.js kotor_eye_tris.json; do
  [[ -e $VSRC/$f ]] && ln -s $VSRC/$f $ROOT/served/viewer2/$f
done
ln -s $VIEWERDATA $ROOT/served/viewer2/data
sed "s/{{build}}/$(date +%s)/g" $VSRC/index.html > $ROOT/served/viewer2/index.html
ln -s $HQ/nose/index.html $ROOT/served/nose/index.html
ln -s $HQ/nose/nose.js $ROOT/served/nose/nose.js
ln -s $ROOT/data/web $ROOT/served/nose/data
ln -s $ROOT/wasm $ROOT/served/wasm
# The page also reads the lid edges from /lids and the paint's recipes and textures from /paint. Both are shared and read only here.
ln -s $HQ/lids $ROOT/served/lids
ln -s $HQ/paint $ROOT/served/paint

cat <<TXT
serve_feature.sh: stack for "$FEATURE" ready at $ROOT

  Serve it (leave it running):
    python3 -m http.server $PORT --bind 127.0.0.1 -d $ROOT/served

  Look at it:
    http://127.0.0.1:$PORT/viewer2/index.html?head=pmhc04&hair=none

  Solve and bench into your own data folder:
    sliders $FEATURE presets $ROOT/data

  Rebuild your own browser module after a code change:
    ~/.claude/tools/likeness-fit/sliders/build_wasm.sh head_sliders $ROOT/wasm/curve/head_sliders $WT

  Refresh the page's HTML after editing it:
    $HQ/viewer2/restage.sh $ROOT/served

  At merge time the HQ reruns "sliders all-features starts" on the merged tree, rebuilds into the shared
  hq/wasm, bumps VER in hq/viewer2/sliders.js once, and copies web/$FEATURE back into $SHARED/web.
TXT
