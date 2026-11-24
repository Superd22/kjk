# Handoff: from the eye paint (stage 1) to the procedural eyeball (stage 2)

Written on 2026-11-20 at about 03:00 by the eye paint agent of the paint session, for a fresh agent who builds stage 2 of the eye brief:
a procedural iris, pupil and eye white, so that eye colour becomes a slider and no KOTOR pixels are copied. You have the lead's brief.
This note holds what you cannot find by looking.

Folder names used below:
- WT is `/Users/david/perso/kjk/.claude/worktrees/head-paint/crates/character_customisation/head`
- TOOLS is `/Users/david/.claude/tools/likeness-fit`
- DATA is `/Users/david/.claude/tools/likeness-fit-data`
- DEV is `/private/tmp/claude-501/-Users-david-perso-kjk/c3910254-b661-450a-9003-645d2158b8cb/scratchpad/dev-eyes`, my private copy of
  the two crates. Make your own copy with `TOOLS/paint/gpu/dev_copy.sh <a new empty folder>`. Never run it on a folder another agent uses:
  I did that once on the folder `scratchpad/dev` and wiped the pore agent's copy (see `TOOLS/paint/NOTICE-eyes-agent-2026-11-20.md`).

## What stage 1 left

- The eye paint layer is integrated and on the live page (http://127.0.0.1:8765/paint/, the group "Eye paint", 23 sliders).
- Settings and lid geometry are in `WT/head_paint/src/eyes.rs`. The GPU half is `WT/head_paint_gpu/src/layers/eyes.rs` and `shaders/eyes.wgsl`
  (bindings 12 and 13). The paint pass calls `eyes_apply` last, always, and the layer's uniform says whether it is on.
- The tools are `WT/head_paint_gpu/src/bin/eye_pictures.rs` (front views of KOTOR and ours with a plain eyeball drawn in) and
  `eye_measure.rs` (measure, fit and time). Their shared code is `src/bin/eyes_shared/common.rs`.
- Pictures are under `TOOLS/hq/paint/eyes/` (page `index.html`). Data is under `DATA/paint/eyes/`: `measure_kotor.json`,
  `measure_ours.json`, `fits.json` (used), `fits_raw.json` (before the widths were dropped), `makeup.json` (hand-set makeup) and
  `hand.json` (the six hand-tuned heads of the first rounds).
- `paint_live` reads `fits.json` and `makeup.json` when it writes the page's `recipes.json`.

## Where the eyeball is today

- Our head's eyeballs are not in the paint layout. The page draws them as their own mesh with a texture per head copied from KOTOR's eye
  (`hq/paint/paint.js` around line 276: `o.eyes.pos`, `o.eyes.colour`, `o.eyes.tris`, and `eyeInfo.file` from `DATA + "eyes.json"`; the
  textures are `viewer-data/tex/eyes/<name>...`, 512 pixels per eye, as `report-eyes.md` section 7 says).
- Kreia's eye texture answers 404 on the page, so her eyeballs show white on both sides (seen in headless Chrome at 02:30).
- `lid_edges.json` gives the 8 vertices whose mean is each eyeball's centre. The radius is 12.3 mm on every head.
- The research note `docs/research/2026-11-20-procedural-iris.md` (kjk repo) holds a 10-step per-pixel recipe and a slider list. It says
  the visible iris is about 11.8 mm across, which is a half angle of about 28.7 degrees on a 12.3 mm sphere.

## Suggested plan

1. Decide where the eyeball's texture is painted. The paint's GPU painter only paints the skin layout. Either add a second small target to
   `GpuPainter` (a 512 by 512 eye texture painted from the sphere's angles, which is the simplest), or paint the eyeball per pixel in the
   page's own three.js shader. The rule "the shader is the implementation" and decision 58 point to WGSL in `head_paint_gpu`.
2. Read KOTOR's eye textures to learn the iris colours and sizes of the 57 heads (they are read only for targets, never shipped).
3. Build the iris, pupil, limbal ring and white, with sliders in the recipe (`controls.rs`, a new group or the "Eye paint" group).
4. Show it in `eye_pictures`: `common.rs` `ball_colour` draws the plain eyeball today; replace it with the real one for ours.
5. Integrate under the lock, as stage 1 did (see the steps below).

## How stage 1 integrated (repeat it)

1. `mkdir TOOLS/paint/integration.lock` with an `owner` file. Remove it when done.
2. Apply your edits to the worktree files as they are now, not by copying whole files: other agents change the same files.
3. A new field in `Recipe` or `Layers` breaks every literal of them in both crates and the tools. `cargo build -p head_paint -p
   head_paint_gpu --bins` lists them. The tests `every_number_of_the_recipe_has_one_control_with_a_unique_id` and
   `the_switches_map_to_the_layers` count the controls and switches.
4. `cargo test --release -p head_paint -p head_paint_gpu`, then `gpu_check <TOOLS/hq/paint/live> <out> 2048 11110,10000,01000,00100
   <6 heads>`.
5. `TOOLS/sliders/build_wasm.sh head_paint_gpu` and `head_paint`, then bump `GPU_MODULE` in `hq/paint/paint.js` (it is `g10` now).
6. Regenerate the page's recipes with `paint_live data` into a temporary folder, check that only your keys changed, then copy
   `recipes.json` into `hq/paint/live/`. The sources are written in `recipes.json` itself.
7. `node TOOLS/paint/gpu/eyes_shot.mjs <out> p_bastilah p_carthh p_kreiah` checks the page headless.

## Traps

- cargo is not on the PATH: prefix `export PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH";`.
- `eye_pictures` takes `<out dir> <label> <names>`; `all` as the name means the 57 heads. `--set` wins over `--values`.
- The front views read our texture through the layout. Some small triangles at the lid corners show pale skin from elsewhere in the
  texture, with jagged edges, whatever the paint does. It is in the layout or the bake, not in the eye layer.
- Our upper lid's lining hangs 1 to 3 mm in front of the eyeball. The eye layer paints the whole lining with the lash colour, so the
  visible edge is dark whatever its width.

## Suggested skills

- None is needed to start. `mattpocock-skills:diagnosing-bugs` helps if a shader paints nothing.

## Stage 1 faults still open (the judge's second round, 03:00)

These are worth a short polish round before or after stage 2. The judge saw them on the 57 pairs at `hq/paint/eyes/pairs.jpg`.
- The lower half is too weak on most heads: liner under the eye, darkness and bags under the eye, red lower rims. The fit's measure of
  the skin under the eye reads our own lower lid's band on our side, so it stops early. Measure our side from the lower lid's line instead
  of the eyeball's bottom, or give the under-eye a second, lower band (bags).
- The heads with heavy smoky or ringed eyes (mod_jaesa, mod_meetra, Terena, sith_app, Zherron) come out at about a third of KOTOR's
  darkness. The socket is capped at 22 L* in `fits.json`, and the shadow mask sits above the lid fold on our head.
- The socket band can read as a brow shadow: it sits high and wide.
- Wings are too long and thin. Bastila's is now 5 mm by hand.
- Pale chunks at some inner corners of the lower lid, from the layout (see Traps).
