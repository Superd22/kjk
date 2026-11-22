#!/bin/zsh
# The animation calibration (roadmap decision 62), in one command. It is run on demand after a fit, not on every fit.
# 1. head_rig's calibrate tool learns where the face bones and teeth go (regressor.json) from KOTOR's own bones and teeth on the fitted
#    heads, then the weights rule of the mouth, the brows, the cheeks and the neck from KOTOR's motion, with the bones placed by that regressor.
# 2. head_rig's mouth_rig tool rigs every head of viewer 2 with that rule: bones, teeth, the lip line and the skin weights.
# 3. The three files of the rule are copied to the viewer data for the live rig of slider heads.
# 4. Viewer 2's rigs.json points to the new rig, and calibration.json lets the viewer warn when a head is fitted again later.
#
# Usage: calibrate.sh [viewer data dir]
# The template (calibration/template.json) only changes with the head mesh: run calibration_template.py again then.
set -e
DATA=${1:-/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data}
CAL=~/.claude/tools/likeness-fit-data/calibration
REPO=~/perso/kjk/.claude/worktrees/head-paint
export PATH="$HOME/.rustup/toolchains/$(sed -n 's/^channel = "\(.*\)"/\1/p' $REPO/rust-toolchain.toml)-aarch64-apple-darwin/bin:$PATH"
(cd $REPO && cargo build --release -q -p head_rig)
$REPO/target/release/calibrate $DATA $CAL/template.json $CAL --brows 30 --neck 30 --leave-out mod_hp_pfhb02
$REPO/target/release/mouth_rig --weights $CAL/weights_model.json --lips $CAL/lips.json $DATA $CAL/regressor.json rigs/calibrated mouth/teeth_v2.png > /dev/null
cp $CAL/calibration.json $DATA/calibration.json
# The browser computes the rig of a head made by the sliders (viewer 2, head_rig in WebAssembly) from the same files.
mkdir -p $DATA/calibration
cp $CAL/weights_model.json $CAL/regressor.json $CAL/lips.json $DATA/calibration/
cat > $DATA/rigs.json <<'EOF'
{
 "variants": [
  {
   "id": "calibrated",
   "label": "Calibrated",
   "dir": "rigs/calibrated",
   "perHead": true,
   "note": "Made by head_rig's mouth_rig tool with the weights rule of the animation calibration (head_rig's calibrate tool)."
  }
 ]
}
EOF
echo "viewer 2 now uses rigs/calibrated"
