#!/bin/zsh
# Builds one crate of the head-paint worktree for the browser (wasm32-unknown-unknown) and writes its module and JavaScript glue.
# Usage: build_wasm.sh <crate> [out dir]
# The out dir defaults to ~/.claude/tools/likeness-fit/hq/wasm/<crate>, which the local server serves at /wasm/<crate>/.
# Read ~/.claude/tools/likeness-fit/WASM-BUILD.md before changing this script, the wasm-bindgen tool or the served links.
set -e
CRATE=${1:?usage: build_wasm.sh <crate> [out dir]}
OUT=${2:-$HOME/.claude/tools/likeness-fit/hq/wasm/$CRATE}
WT=$HOME/perso/kjk/.claude/worktrees/head-paint
TC=$(sed -n 's/^channel = "\(.*\)"/\1/p' $WT/rust-toolchain.toml)
TOOLCHAIN=$HOME/.rustup/toolchains/$TC-aarch64-apple-darwin
export PATH="$TOOLCHAIN/bin:$PATH"
# The minimal toolchain's rust-lld looks for libLLVM.dylib next to itself and does not find it; the toolchain's lib folder has it.
export DYLD_FALLBACK_LIBRARY_PATH=$TOOLCHAIN/lib
cd $WT
cargo build --release -p $CRATE --lib --target wasm32-unknown-unknown
mkdir -p $OUT
$HOME/.claude/tools/likeness-fit/sliders/bin-tools/bin/wasm-bindgen --target web --no-typescript --out-dir $OUT target/wasm32-unknown-unknown/release/$CRATE.wasm
ls -la $OUT
