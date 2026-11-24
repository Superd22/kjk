# The lips' shape, round 2

This note was written on the night of 2026-11-22 by the lips round 2 conversation, started by the head HQ. The brief is `/private/tmp/claude-501/-Users-david-perso-kjk/52003e36-b8ca-4208-a733-a6d51b417e4e/scratchpad/jaw/brief-lips-round2.md`. It serves B3 and B5 for the mouth. It ran two loops. No judges were run. Nothing was committed.

## Verdict

The result is clearly better by the brief's five conditions, so it is in viewer 2's default. Viewer 2 now has a seventh feature, "Lip shape", with 8 sliders and a preset for each of the 57 heads.

## What is still broken, first

- **Mouth width's ends turn a few triangles.** At -3 and at +6 (the range's ends), 1 or 2 thin triangles turn at a corner's tip on Carth, old_w_f and pmhc04. Narrowing past -3.5 and widening past 6.5 turn more. No solved head is near enough to these ends to turn one.
- **Mouth corner height at its top end (3.94) turns 2 triangles on Bastila**, 12 mm out on the lower lip.
- **Two heads cannot be trusted.** Our Kyle Katarn has nearly closed, flat lips, and its line between the lips sinks too little to read past 2.5 mm, so its width reads 5 mm. mod_kenobi has a beard over KOTOR's mouth. Both still pass the guard, which compares only outlines.
- **sith_app's open mouth is only 2 mm deep on KOTOR's mesh.** The line sits at the bottom of the gap, about 1.5 mm below its middle. Its outlines now read, but its front gap is still 1.51 mm, above the 0.90 mm pass mark.
- **The upper border is not the painted lip border.** It is where the lip has fallen back half way from its crest to the hollow above it, which often lies in the skin under the nose. The painted border is the lip paint's job, and its own sliders already fit it per head.
- **A small dark seam at one corner on pmhc03, and a light speck at Carth's corner in the open shape.** Today's heads have both, so they are not from this work.

## What changed in the measure (`lips.rs`)

The first pass's measure misread our heads badly. On 21 of our 57 live heads (every female head, and comm_a_m, comm_b_m, mod_keanu and mod_kenobi), it took the fold under the nose for the line between the lips, 15 to 18 mm too high. It did the same on 6 of KOTOR's heads (mod_exile_canon, mod_improved_bastila, mod_jaesa, old_a_f, p_kreiah and pfha02). That is why pfha05 went wrong in the first pass.

- **The line between the lips is searched within 4 mm of a guide.** On our heads, the guide is the lip paint's opening rows (`head_paint::lips::OPENING_UPPER` and `OPENING_LOWER`, which follow the vertices). On KOTOR's heads, it is the same rows on our smoothed fit of that head (`fit_smooth`). The reading itself stays on KOTOR's own mesh.
- **Each lip's border is where the lip has fallen back half way** from its crest to the hollow beside it, the deepest place within 10 mm above the upper crest or below the lower one. It is the same rule on KOTOR's and on our meshes. It reads on nearly every column of every head, where the old steepest-fall rule found no upper border on most of our smooth heads.
- **Reading our border from the paint's border rows was tried and not used.** The paint's border is KOTOR's painted border, and on KOTOR's heads it lies 2 to 6 mm inside the geometric border that the shading reads. Comparing one against the other gives a bias, not a likeness.
- **An open mouth** that sinks more than 2 mm is read at the middle of its gap.
- **A shallow stretch of the line up to 3 mm long no longer ends the mouth.** Our widths had swung from 12 to 49 mm. They now read 37 to 51 mm.

The checks, on all 57 KOTOR heads (`/private/tmp/claude-501/-Users-david-perso-kjk/bca58dc4-0301-4437-9c49-b6c3d281996e/scratchpad/checks_m1.json`):

| Outline | Wobble, subdivided twice, median (mm) | Left against right (mm) | Two KOTOR heads, 5th percentile / median (mm) |
|---|---|---|---|
| Middle profile | 0.30 | | 0.35 / 0.90 |
| Profile 10 mm | 0.37 | 0.56 | 0.55 / 1.20 |
| Front | 0.88 (it was 1.27) | 0.47 | 0.90 / 1.73 |

The pass marks are now 0.35, 0.55 and 0.90 mm. Check 3: the subdivided copy finds itself on 29, 34 and 32 of about 56 heads, and the right side finds its own left on 42 of 56 heads for the front. Our mouths are 8 mm narrower than KOTOR's at the median (KOTOR 51.5 mm), and our corners lie 4.5 mm less deep.

## What changed in the sliders

- **A new slider kind, "after"** (`set.rs`, `solve.rs`): a fixed movement added right after the build's smoothing. Mouth width and Cupid's bow are of this kind, so the smoothing no longer spreads the width along the lips or erases the bow. Every other set builds exactly as before.
- **Mouth width** stretches the lips evenly from the middle to the corners. Its backward turn follows the face's turn read over 4 mm and held from 3 mm short of the corner, so the corner's tip moves as one piece. It moves the width 1.9 to 2.0 mm per unit, and the corners sink 0.75 mm back per unit as they go out.
- **Cupid's bow** is back. Along the upper border, the lip comes forward at the two peaks and goes back at the dip. It moves the bow 0.3 to 1.5 mm per unit.
- **The corners' tearing was fixed.** 344 vertices of the mouth's lining near the corners are kept still by the smoothing, and the slider maker skipped them. The corners' skin moved away from its lining and tore holes. The lip sliders now move them. Mouth corner depth and Mouth corner height then turned no triangle at ±4 on pmhc03, Carth, sith_app, pfha05 and pmhc04.

| Slider | Range | Kind |
|---|---|---|
| Mouth width | -3 to 6 | after |
| Mouth corner depth | -1.13 to 5.28 | proportion |
| Mouth corner height | -4 to 3.94 | proportion |
| Upper lip fullness | -1.17 to 1.63 | proportion |
| Lower lip fullness | -0.92 to 1.91 | proportion |
| Upper lip height | -5 to 4.62 | proportion |
| Lower lip height | -2 to 6 | proportion |
| Cupid's bow | -1.84 to 2.6 | after |

The ranges are the solved values with 0.5 more on each side, within the ends drawn clean.

## The solve and the guard

The cost adds a steep charge for any outline worse than today's, so the solve looks for moves the guard keeps. The guard tries shares of the move from 1 down to 0.125, each also without the two lip heights, and the whole move with one slider left out. It keeps the cheapest try with no turned triangle outside the slit, no outline worse than today's, and no lip height past KOTOR's from the side today's head is on. 34 heads take their whole move, 20 take part of it and 3 keep today's lips. The final solve ran on today's head, with the jaw's `page_throat.json` and the skull feature.

Gaps to KOTOR (middle, 10 mm, front), in mm, measured with the new measure:

| Head | Today | Now |
|---|---|---|
| Bastila | 0.39, 0.43, 1.01 | 0.30, 0.36, 1.00 |
| pmhc04 | 0.73, 0.48, 1.34 | 0.51, 0.47, 0.95 |
| Carth | 0.68, 0.45, 0.87 | 0.47, 0.33, 0.51 |
| pmhc03 | 0.71, 0.55, 0.95 | 0.55, 0.39, 0.94 |
| sith_app | 0.34, 0.74, 1.60 | 0.28, 0.66, 1.51 |
| pfha05 | 0.59, 0.54, 0.78 | 0.25, 0.39, 0.48 |
| Median of 57 | 0.50, 0.59, 1.10 | 0.34, 0.45, 0.89 |

pfha05's upper lip is 12.45 mm high, against KOTOR's 12.60 and today's 12.20. The first pass's gaps are not comparable, because its measure misread 21 of our heads.

## What was seen

In viewer 2's "In game" light with the hair off, in three-quarter close-ups: Bastila and pfha05 with their paint, and pmhc04, pmhc03, Carth and sith_app with the paint off (pmhc04's painted beard hides the mouth). Each was seen closed and in lip shape 3 (AA AE AH), and Bastila, pmhc04 and sith_app also in lip shape 4 (OW OY). No tear, fold or hole showed. pmhc04's lower lip corner is a little angular in shape 4, and today's head is the same.

## Not checked

- The overlap sheet of all 57 heads was written but not looked at head by head. The other 51 heads that moved were not looked at in close-up.
- Pair corners of the sliders were not drawn.
- The jaw's and the nose's measures were not rerun on the solved heads. The solve costs any move of the nose's base.
- Check 2 is David's.

## Pictures

- Sheets (KOTOR, today, ours; front and side): `/private/tmp/claude-501/-Users-david-perso-kjk/bca58dc4-0301-4437-9c49-b6c3d281996e/scratchpad/sheets/p_bastilah.png`, `/private/tmp/claude-501/-Users-david-perso-kjk/bca58dc4-0301-4437-9c49-b6c3d281996e/scratchpad/sheets/pmhc04.png`, `/private/tmp/claude-501/-Users-david-perso-kjk/bca58dc4-0301-4437-9c49-b6c3d281996e/scratchpad/sheets/p_carthh.png`, `/private/tmp/claude-501/-Users-david-perso-kjk/bca58dc4-0301-4437-9c49-b6c3d281996e/scratchpad/sheets/pmhc03.png`, `/private/tmp/claude-501/-Users-david-perso-kjk/bca58dc4-0301-4437-9c49-b6c3d281996e/scratchpad/sheets/sith_app.png`, `/private/tmp/claude-501/-Users-david-perso-kjk/bca58dc4-0301-4437-9c49-b6c3d281996e/scratchpad/sheets/pfha05.png`.
- All 57 heads: `/private/tmp/claude-501/-Users-david-perso-kjk/bca58dc4-0301-4437-9c49-b6c3d281996e/scratchpad/overlap_all57.png` (KOTOR red, today yellow, ours green).
- Slider ends at the final ranges: `/private/tmp/claude-501/-Users-david-perso-kjk/bca58dc4-0301-4437-9c49-b6c3d281996e/scratchpad/ends_final.png`.

## How to undo

- Viewer 2: in `~/.claude/tools/likeness-fit/hq/viewer2/sliders.js`, remove the line `{ key: "lips", title: "Lip shape", ... }` and `["lips", "Lips"]` from GROUPS, then run `restage.sh`. The whole file before the change is `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T//kjk-lips-round2-backup-2026-11-22/sliders.js.before`, but copying it back would also undo any later change by another conversation.
- Data: delete `~/.claude/tools/likeness-fit-data/sliders/web/lips/sliders_r2.json` and `page_r2.json` after undoing viewer 2. That folder holds nothing else. The first pass's files stay in its own scratchpad.
- Code, in the head-paint worktree's crate `head_sliders`: `set.rs` and `solve.rs` (the "after" kind) and `Cargo.toml` (the native-only `head_paint` dependency). Originals are in `/private/tmp/claude-501/-Users-david-perso-kjk/bca58dc4-0301-4437-9c49-b6c3d281996e/scratchpad/orig/`. Removing the "after" kind from `set.rs` while viewer 2 loads the lips would build Mouth width and Cupid's bow before the smoothing, so undo viewer 2 first. The live module in `hq/wasm/curve/head_sliders` was rebuilt at 01:17 by another conversation and already holds the "after" kind.
- Logged in `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T//kjk-slider-kit-shared-changes.md` at 01:10, 01:40 (written there as 01:55 at first) and 01:53. The page before the 01:53 fix is `$TMPDIR/kjk-lips-round2-backup-2026-11-22/page_r2.before_strict.json`.

## Commands

```
export PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH"
cd ~/perso/kjk/.claude/worktrees/head-paint && cargo build --release -p head_sliders --target-dir target-lips --bin sliders
B=./target-lips/release/sliders; D=~/.claude/tools/likeness-fit-data/sliders
LIPS_PLAYER=1 $B lips make $D sliders.json     # the 8 sliders with the player ranges (without LIPS_PLAYER, the solve's ranges)
$B lips solve $D sliders.json out.json all
$B lips ends $D sliders.json out.png range <head>...   # each slider at its own ends; a number draws ±that value
$B lips borders $D          # per head: the line against the paint's opening, borders read, heights and widths
$B lips ourcols $D <head>   # one live head's columns
```
A working binary is at `/private/tmp/claude-501/-Users-david-perso-kjk/bca58dc4-0301-4437-9c49-b6c3d281996e/scratchpad/sliders_final`.

## Fix at 01:53: turned triangles inside the slit

The jaw-skull-fold conversation found 3 turned triangles at the mouth corners (old_w_m 2, pmba01 1). Its count compared the default before and after my 01:34 page rewrite, so it set my final presets against my loop-2 presets, not against a head without lips. pmba01 turns no triangle against its head without lips. The real fault was in my own count: the guard left out triangles inside the slit. Against the head without lips, 15 heads turned 1 to 9 triangles there, and old_w_m's lining looked jagged at its far corner in the open shape.

Those 15 heads were re-solved with every triangle counted (`LIPS_STRICT=1`): comm_a_m, comm_b_m, comm_w_m, mod_hp_pfhb02, mod_jaesa, n_hussef, n_traskh, n_zezkaiellh, old_b_m, old_w_m, p_bastilah, pmha01, pmha02, pmhc04 and sith_app. The other 42 heads kept their presets.

On the current default, with the jaw's page_throat.json, the brow's page_fold.json and the skull feature, `lips verify` finds no turned triangle on any of the 57 heads, the slit included, and no outline worse than today's. The medians are 0.35, 0.45 and 0.89 mm.

- old_w_m: Mouth width fell from 6.0 to 3.53. Gaps 0.55, 0.37, 1.88 today, now 0.45, 0.34, 1.25.
- Bastila: 0.30, 0.40, 0.96 (it was 0.30, 0.36, 1.00).
- pmhc04: 0.54, 0.47, 0.95.
- sith_app: 0.32, 0.63, 1.11 (front was 1.51).
- pmba01 kept its preset.

Seen in viewer 2's "In game" light with the paint and hair off: old_w_m and pmba01, closed and in lip shape 3. Both are clean. old_w_m's far corner shows the same small step in the open shape as its head without lips.

The sheets above were drawn before this fix. Bastila's, pmhc04's and sith_app's presets changed since.

## Traps

- **Other conversations break the crate's build while they edit it.** `chin_neck.rs` and `throat_corner.rs` failed to compile for a few minutes. A failed build leaves the old binary in place, and a script that goes on runs it. Check the build's result before using the binary.
- **The live head changes under you.** During this work, the jaw moved to `page_throat.json` and a skull feature appeared. `lips::tools::Live` must load what `sliders.js` loads.
- **A vertex the smoothing keeps is skipped by most slider makers.** The mouth's corners have such vertices, and a field that skips them tears the corners.
- **`window.closed` is a browser property.** A page helper named `closed` cannot be set.
- **The paint's border is not the shape's border.** See above.
- **The guard's slit rule hid real turns.** Count every triangle (`LIPS_STRICT=1`) when solving, and check with `lips verify` on the current default.

## Next

1. Look at all 57 heads in close-up, and the pair corners of Mouth width with the two corner sliders.
2. Reread the paint's lip fit on the new shapes, since the painted border moves with the vertices.
3. Give KOTOR's open mouths (sith_app) a reading of the gap's middle that works when the inside is shallow.
