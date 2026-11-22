#!/bin/zsh
# Builds one crate of a kjk worktree for the browser (wasm32-unknown-unknown) and writes its module and JavaScript glue.
# Usage: build_wasm.sh <crate> [out dir] [worktree]
# The out dir defaults to ~/.claude/tools/likeness-fit/hq/wasm/<crate>, which the local server serves at /wasm/<crate>/.
# The worktree defaults to ~/perso/kjk/.claude/worktrees/head-paint, which is where the head work lives. A conversation working in
# its own worktree passes that worktree here, or sets KJK_WORKTREE, and passes an out dir of its own, so that two conversations
# building in parallel do not overwrite each other's module. See "Working in parallel on the head" in the head roadmap.
# Read ~/.claude/tools/likeness-fit/WASM-BUILD.md before changing this script, the wasm-bindgen tool or the served links.
set -e
CRATE=${1:?usage: build_wasm.sh <crate> [out dir] [worktree]}
OUT=${2:-$HOME/.claude/tools/likeness-fit/hq/wasm/$CRATE}
WT=${3:-${KJK_WORKTREE:-$HOME/perso/kjk/.claude/worktrees/head-paint}}
[[ -f $WT/rust-toolchain.toml ]] || { echo "build_wasm.sh: $WT is not a kjk worktree (no rust-toolchain.toml)" >&2; exit 1; }
TC=$(sed -n 's/^channel = "\(.*\)"/\1/p' $WT/rust-toolchain.toml)
TOOLCHAIN=$HOME/.rustup/toolchains/$TC-aarch64-apple-darwin
export PATH="$TOOLCHAIN/bin:$PATH"
# The minimal toolchain's rust-lld looks for libLLVM.dylib next to itself and does not find it; the toolchain's lib folder has it.
export DYLD_FALLBACK_LIBRARY_PATH=$TOOLCHAIN/lib
cd $WT
echo "build_wasm.sh: crate $CRATE, worktree $WT, out $OUT"
cargo build --release -p $CRATE --lib --target wasm32-unknown-unknown
mkdir -p $OUT
$HOME/.claude/tools/likeness-fit/sliders/bin-tools/bin/wasm-bindgen --target web --no-typescript --out-dir $OUT target/wasm32-unknown-unknown/release/$CRATE.wasm
ls -la $OUT
