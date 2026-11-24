# The jaw, throat and skull folded into viewer 2

This note was written on 2026-11-22 at about 02:10 by the conversation briefed with `/private/tmp/claude-501/-Users-david-perso-kjk/52003e36-b8ca-4208-a733-a6d51b417e4e/scratchpad/jaw/brief-jaw-skull-fold.md`. It serves B3 and B5 for the jaw, the chin and the skull, and A2. Nothing was committed. No judges were run.

The scratchpad is `/private/tmp/claude-501/-Users-david-perso-kjk/5a984c97-8d8f-4c99-853f-22536be7b508/scratchpad/` (called `S` below). `S/env.sh` and `S/sync.sh` set up the private build copy in `S/ws`.

## Verdict

The default changed. Viewer 2 now builds the throat construction in the browser, and its default heads carry the solved jaw, throat and skull values on all 57 heads. A new tab, "Throat and skull", has the 4 throat sliders and the 4 skull sliders with each head's preset. The haired heads' forehead slope was also solved again, and 13 heads took a new Forehead slope.

## What is still broken, first

- **The throat sliders' pair corners still turn triangles.** 48 of the 57 heads turn 1 to 17 triangles at some slider end or pair corner. No head turns one at its own values. One repair loop failed (see below), so the ranges were left as they were. The corners, with the number of heads and the most triangles on one head:
  - Throat corner height high and Throat roundness low: 25 heads, up to 8.
  - Throat corner height high and Jaw corner rise high: 18 heads, up to 17.
  - Throat depth high and Throat roundness low: 18 heads, up to 11.
  - Throat depth low and Throat corner height high: 14 heads, up to 9.
  - Throat depth high and Throat corner height low: 13 heads, up to 8.
  - Throat depth high and Throat corner height high: 9 heads, up to 10.
  - Every other end or corner: at most 8 heads each. The full list is `S/chk/tc0.json` (`by_case`).
- **pmhc04's forehead is still too upright.** The solve ran Forehead slope to the top of its range (+10.9). Read the brow feature's way, the forehead now leans back 21.0 degrees, against 17.8 before and 30.0 on the bald reference. The slider's range is set by the KOTOR fits, and it cannot reach 30 degrees on this head.
- **The ledge** was left alone, as the brief allows.
- **Three triangles turned at the mouth corners** (old_w_m 2, pmba01 1) when the lips conversation rewrote `lips/page_r2.json` at 01:34. My changes do not cause them: the same heads with the old brow presets turn the same three. This is for the lips conversation to judge.
- Three files were already missing on the default page before my change (the favicon, `paint/live/groups/<head>.json` and `viewer2/data/body/<head>.json`). They are the only console errors.

## Measured

- **The browser and Rust agree.** The 57 default heads built in headless Chrome through the page's own kit equal the native build of the same files within 0.00006 mm. The native build of today's files before my change also equals the live page before my change exactly.
- **The fold reaches the solved targets.** The chin-to-neck outline gaps of the default heads equal the last conversation's solved heads to 0.00001 mm. The side view's gap falls from 3.91 mm (median, before) to 0.96 mm. The skull's movement equals the solved one exactly. The forehead rays differ from the solved heads only because the brow presets of viewer 2 (`page_curve.json`) are not the ones the old solve read.
- **The skull profile gap** falls from 4.30 mm (median) to 2.09 mm.
- **The other features did not move.** Between the old and the new default, with the lips the same:
  - the nose and the eyes moved less than 0.001 of a mark on every head;
  - the cheeks moved at most 0.05 of a mark;
  - the jaw rows moved at most 0.47 of a mark (comm_b_f's chin projection);
  - the brow rows moved at most 0.96 of a mark, on Terena's temple width, which Skull width narrows on purpose;
  - no triangle turned on any head.
- **The lips** after the whole change: pmhc04 reads 0.51, 0.47 and 0.95 mm, and Bastila 0.30, 0.36 and 1.00 mm, the lips conversation's own numbers.
- **The Forehead slope solve** (`jawskull forehead-solve`, `FS_COST=slope`). It targets the brow feature's reading of the slope on the bald reference. On the 13 heads that changed, the median slope error fell from 9.4 to 0.0 degrees, and the whole skull profile's gap from 1.35 to 1.09 mm, worse on none. Only the brow's forehead slope row moved past its mark, which is the point. No triangle turned. The first loop targeted the front of the skull profile instead. Its gains (0.14 to 0.23 mm) were under the measure's wobble, and it made Bastila's forehead more upright, away from her reference, so it was dropped.
- **Speed.** A head builds in about 43 ms in Chrome, against 36 ms before.

## Seen

In viewer 2's "In game" light, with the hair off, next to KOTOR and the old default (`S/pic/c1/`, three columns: KOTOR, before and after):
- pmhc04 and Bastila from below, three-quarter from below, in profile, from the front, three-quarter from above and from behind, and talking (the shapes AA, OW and F, and a talking profile). The corner under the jaw is one straight line like KOTOR's, and the rig talks on the new shape.
- A dark notch at the neck's cut edge from below shows on pmhc04 before and after, so it is not new.
- The new foreheads of pmhc04, the Handmaiden, pmhc03, pmba01, Kreia, pfha06 and Jaesa in profile and three-quarter from above (`S/pic/fhc/`). They are clean and the changes are small.
- The new tab and its sliders in the page, with a slider moved through the page's own input.

## Not checked

- Close-ups of heads other than those named above.
- The throat's pair corners in pictures. Only counted.
- The paint's hairline and the hair caps on the new scalps.
- The animation calibration was not run again.

## Viewer 2

- http://127.0.0.1:8765/viewer2/index.html?head=pmhc04&hair=none
- http://127.0.0.1:8765/viewer2/index.html?head=p_bastilah&hair=none

## What changed

- Crate `head_sliders` in the head-paint worktree:
  - `set.rs`: the slider kind "throat", `throat_values`, `init_throat`, `kept_flags`. `build` ends with the throat when the set has throat sliders.
  - `web.rs`: `Builder.init_throat`.
  - `lib.rs`: `jaw_edge`, `throat_corner` and `chin_neck` build for the browser. Their `tools` are native only.
  - `chin_neck.rs`: `THROAT_KEYS`, `ThroatShape::from_parts`, `free_flags`, `vertex_count`, `section_probe`.
  - `under_jaw.rs`: `local_surface` moved out of `tools`.
  - `jaw_edge.rs`: `Heads::load_viewer`, `VIEWER_FILES`, `FOLD_FILES`, `values`, `build`.
  - `jaw_skull.rs`: the commands `fold-data`, `v2-bins`, `feature-diff`, `throat-corners`, `throat-probe` and `forehead-solve`.
- Module `hq/wasm/curve/head_sliders` built from the worktree. `SLIDERS_MODULE_VER` is "50".
- Data in `likeness-fit-data/sliders/web/`: `throat/` (free.bin, near.bin, near_w.bin), `skull/sliders.json` and `skull/page.json`, `jaw/page_throat.json`, `brow/page_fold.json`.
- `hq/viewer2/sliders.js`: the jaw loads `jaw/page_throat.json`, the brow `brow/page_fold.json`, a new feature "Throat and skull", the groups Throat and Skull, the throat sliders rest at their middle, and texts that said "five features". `viewer2.js`: three hints that said "44 sliders".

## What was deleted

- The viewer data `jawproto`, `throatproto`, `skullrefit`, `chinneck` and `jawskull` (moved to `~/.claude/tools/likeness-fit-data/backups/jaw-skull-prototypes-2026-11-22/viewer-data/`). They were reached through `?shapes=data/<name>/shapes.json`. The `?shapes=` option itself is generic and stays.
- The jaw edge prototype: `Values`, `NAMES`, `Proto::apply` and its constants, and the `jawedge` commands sheet, slices, solve, solve-sheet and viewer-bins. Its zone is kept as `ThroatZone`, because the throat construction uses it.
- The throat corner prototype: `Sharpen`, `Construction` and `Candidate`, and the `throatcorner` commands profiles, viewer-bins and sheet. `outline` is now a free function.
- The skull refit path: `with_refit`, the `chinneck bins` command, and the refit argument of `chinneck throat-bins` and `jawskull wire`.

## How to undo

- **The default:** in `hq/viewer2/sliders.js`, set the jaw's page back to `jaw/page.json` and the brow's to `brow/page_curve.json`, delete the `skull` line, and run `restage.sh`. Keep the lips line. Keep the new module: it builds sets without throat sliders exactly as before, and the lips need its "after" kind. The module before mine is in `S/backup/wasm_curve_head_sliders/`, but it has no "after" kind.
- **Only the forehead:** point the brow back at `brow/page_curve.json`.
- **The deleted prototypes:** move the folders back from the backups' `viewer-data/` into the viewer data folder. The five crate files before the deletion (with the port) are in the backups' `crates/`. The crate files before any of my changes are in `S/orig/src/`.
- **Roadmap:** the line "folded into viewer 2" under "Running now".

## Traps

- **Another conversation edits `sliders.js` at the same time.** The lips conversation added its feature on top of my edit within minutes. Read the file again before each edit, and never copy a backup over it.
- **Whoever rebuilds `hq/wasm/curve/head_sliders`** must build from the worktree's current `head_sliders`. A module without `init_throat` loses the throat, and the page logs an error.
- **The lips data changed while I measured.** A comparison across a change of someone else's data shows their change as yours. Build the baseline natively with the same files (`v2-bins`, `V2_EXTRA`, `V2_BROW`).
- **The preset margins of a limit must be read on what the limit acts on.** I measured the corner's middle against the outline's start, but limited the corner's control point, and four presets moved by up to 9 mm.
- **`throat-corners` builds each head once without the throat** (its throat sliders renamed `throat_off`) and applies the throat alone, so the 1,824 cases take about 2.5 s.
- **pfha01's bald reference** reads a forehead slope of 83 degrees. It was left out of the forehead solve.

## Next

- The throat's pair corners need a new idea. A cap inside the construction that keeps the presets barely helps. The two faults are different. A high corner pulls the skin next to the held chin up by 12 mm. A deep, sharp corner pulls a vertex on the neck's front up past the one above it.
- pmhc04's forehead needs a Forehead slope with more reach, or a slider that leans the upper forehead back.
- The animation calibration, the hair caps and the paint's hairline on the new scalps (the going-live steps of the last note).
