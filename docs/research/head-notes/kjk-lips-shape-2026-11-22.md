# The lips' shape: measure, lip sliders and solve

This note was written on the night of 2026-11-21 to 2026-11-22 by the lips-shape conversation, started by the head HQ. The brief is `/private/tmp/claude-501/-Users-david-perso-kjk/52003e36-b8ca-4208-a733-a6d51b417e4e/scratchpad/jaw/brief-lips-shape.md`. The HQ changed the brief during the work. A result goes into viewer 2's default only when it is clearly better, and otherwise it stays out of the viewer. The work serves B3 and B5 for the mouth. Nothing live changed. Nothing was committed. No judges were run.

## Verdict

The result is not clearly better, so it is not in viewer 2. Viewer 2's default page is unchanged, and the staged test copy was deleted.

The profile measure works and passes checks 1, 3 and 4. Seven lip sliders exist, and the solve brings the median outline gaps down on all 57 heads with no head worse. But the front measure misreads the lips' borders on some heads, and the solve then shapes those lips the wrong way. pfha05 is the clear case. KOTOR's pfha05 has a thin upper lip and a full lower lip. The solve made our upper lip taller and our lower lip thinner, while its front gap fell from 4.65 to 4.29 mm. The mouth width, the largest real difference, is still out of reach.

## What is still broken, first

- **The front border is not readable on many of our heads.** The upper border is the place above the upper lip's crest where the skin falls back most steeply. Our lips are smooth, and on many live heads no place falls back steeply enough (0.3 mm per mm), so our upper border is missing. The front outline is then only the lower half, and the gap compares a half outline with a whole one. This misled the solve on pfha05 and probably on old_w_f.
- **Our mouths are 9 mm narrower than KOTOR's at the median**, and the solve could not fix it. The slider Mouth width moves the corners out, but the build's last smoothing spreads the move along the lips. So widening the mouth makes the profile 10 mm from the middle worse, and the guard refuses it. Its far ends also turn a row of thin triangles running down from each corner (from about +4), and narrowing past about -4 opens a hole at each corner.
- **The corner sliders fold at their ends on heads whose corners lie further in than the base head's** (pmhc03, Carth, sith_app). Mouth corner height at -4 turns 23 to 27 triangles there. Two repairs did not help: a field that moves the corner as one piece 6 mm each way, and letting the mouth's inside follow the lips for 10 mm. The solve avoids these ends, and no solved head turns a triangle.
- **Cupid's bow was dropped.** The build's last smoothing erases a bump as narrow as the bow. The bow measure moved the wrong way, by -0.35 mm per unit on Bastila.
- **sith_app cannot be scored on the front.** KOTOR's sith_app has an open mouth, and the measure cannot find the upper lip's border above the gap. Carth's mouth width cannot be trusted either, because his slit is so thin that its ends are found too close in (KOTOR 30 mm).
- **The subdivided copy closes KOTOR's thin slits.** So check 1's wobble from subdivision is large for the width and the front outline (width 7.5 mm). The left side against the right side is the better check for those (front 0.49 mm, width 3.0 mm).

## The measure (`src/lips.rs`, `sliders lips`)

Each head is read in upright cuts across the face, every 0.5 mm from the middle out to each corner. A cut is the front-most surface from below the chin to above the nose.

- **The line between the lips** is where the cut sinks deepest behind the lips within 7 mm above and 9 mm below. The fold above the chin sinks too, so the highest place that sinks at least half as deep as the deepest is taken. Where KOTOR's lips part and the cut sees nothing, the gap is filled as a deep V. The line is read on the cut without smoothing, so a thin slit still sinks.
- **The corners** are where the line stops sinking by at least 0.3 mm.
- **Each lip's crest** is its most forward place. **Each lip's border** is where the lip falls back most steeply, above the upper crest or below the lower one, on the cut smoothed by 1 mm. The borders are smoothed along the mouth over 2 mm each way, because our lips' vertices lie about 2 mm apart.
- **The outlines** are three readings. The first is the profile down the middle. The second is the profile 10 mm to each side. The third is each half of the lips seen from the front (upper border out to the corner and back along the lower border). Every outline is lined up on the middle's line between the lips (height) and upper lip crest (depth). The profiles run from 11 mm above the line to 13 mm below it. The line's dip is capped 3 mm behind the less prominent lip.
- **The proportions** are mouth width, the height of each lip, how far each lip stands out (the upper lip from the nose's base, the lower from the fold above the chin), the corners' height and depth against the middle, the Cupid's bow and the line's depth (capped, not scored).
- KOTOR is read on its own bare mesh (`under_jaw::tools::kotor_bare`).

The checks, on all 57 KOTOR heads (`checks_final.json`):

| Reading | Wobble, subdivided twice, median / 95th percentile (mm) | Left against right (mm) | Two KOTOR heads, 5th percentile / median (mm) | Check 3 |
|---|---|---|---|---|
| Middle profile | 0.28 / 1.99 | | 0.40 / 1.07 | the subdivided copy finds itself on 33 of 57 heads |
| Profile 10 mm | 0.44 / 2.27 | 0.58 | 0.60 / 1.39 | 29 of 55; the right side finds its own left on 23 of 57 |
| Front | 1.27 / 7.96 | 0.49 | 1.10 / 2.64 | 27 of 55; the right side finds its own left on 41 of 57 |

- Check 1 passes on the middle and the 10 mm profiles. The front passes left against right, and fails against the subdivided copy, because subdivision closes KOTOR's thin slits.
- Check 3 is weak on the profiles (33 of 57 and 29 of 55). KOTOR's mouths are alike in profile.
- Check 4 was seen on the sheets of all 57 heads. KOTOR's outlines trace the lips on most heads. They fail on mod_kenobi (a beard over the mouth) and sith_app (an open mouth).
- Check 2 is David's. It was not done.
- The pass marks are the gap between the 5% most alike pairs of KOTOR heads: 0.40 mm for the middle, 0.60 mm at 10 mm and 1.10 mm for the front.

## The existing sliders and the learned shapes

On the four candidates, every nose, jaw and cheek slider was set free with the lips as the only target. Neither profile moved, and neither did the width or the corners' depth. Only the front border moved a little, through the nose's wings. The 20 learned mouth shapes were not tested. They live on the 2,375-vertex mouth crop of the FLAME fit, not on the kit's 4,750-vertex head, and they are unnamed shapes.

## The lip sliders

They are ordinary sliders (kind "proportion", zone "mouth"), fixed movements of the base mesh made by `lips::fields::make`. Viewer 2 could load them as a sixth feature with one line in `sliders.js` and no new browser code. The live rig then follows them. The ranges below cover the solved values with 0.5 more on each side, within the tested ends.

| Slider | Range | What it does | Measured on Bastila, per unit |
|---|---|---|---|
| Mouth width | -1 to 4.75 | The corners move out along the mouth, and only the outer half of the mouth stretches. | width +0.88 mm, corner depth -0.50 |
| Mouth corner depth | -6 to 6 | The corners sink back into the cheeks. | corner depth +0.93 mm |
| Mouth corner height | -2.5 to 4 | The corners move up or down. | corner height +0.56 mm, width -1.25 |
| Upper lip fullness | -2.53 to 5 | The upper lip comes forward. | +0.82 mm |
| Lower lip fullness | -4 to 5 | The lower lip comes forward. | +0.26 mm (weak) |
| Upper lip height | -6 to 6 | The upper border moves up, and the skin up to the nose's base gives way. | +0.70 mm |
| Lower lip height | -6 to 6 | The lower border moves down. | +0.61 mm |

## The loops

- **Loop 1.** Eight sliders, and the solve on all 57 heads. The median front gap went from 2.35 to 1.36 mm. Mouth width was unused. Bastila got worse at 10 mm and on the front. Lower lip fullness also moved skin near the eyes, because its field had no fade above the line. That was fixed with a vertical fade and a mouth window on every field.
- **Loop 2.** Every fold sits in the slit between the lips, so the fold count now leaves out triangles in the slit (within 1.5 mm of the line and facing up or down, or behind the lips). The width spreads further, Cupid's bow was dropped, the lips' heights got wider ranges, the outlines count twice in the cost, and each head is guarded. No head was worse, and no head folded.
- **Loop 3.** The width moves only the outer half of the mouth. The results were about the same as loop 2.
- **Loop 4.** The guard became strict, so no outline may get worse at all. This is the final solve, `solve_all4.json`. 28 heads take their whole move, 12 take part of it, and 17 keep today's lips.

The gaps, today against loop 4, in mm (middle, 10 mm, front):

| Head | Today | Loop 4 |
|---|---|---|
| Bastila | 0.40, 0.44, 1.48 | 0.33, 0.39, 1.46 |
| Carth | 0.68, 0.45, 1.29 | 0.45, 0.26, 0.66 |
| pmhc03 | 0.80, 0.66, 2.11 | 0.43, 0.52, 0.91 |
| sith_app | 0.34, 0.74, 3.29 | unchanged (the guard kept today's lips) |
| Median of 57 | 0.58, 0.71, 2.46 | 0.41, 0.59, 1.52 |

Bastila passes both profiles and fails the front (1.46 against 1.10). Carth passes the 10 mm profile and the front, and fails the middle (0.45 against 0.40). pmhc03 passes the 10 mm profile and the front, and fails the middle (0.43 against 0.40). sith_app passes the middle only.

## What was seen in viewer 2

These were checked in the "In game" light with the hair off, on a staged copy that is now deleted.
- **Bastila**, front: the lower lip is a little fuller, and the lip paint follows the new border.
- **pmhc03**, three-quarter: the lower lip is thinner, like KOTOR's. The corners are clean. With lip shape 3 (AA AE AH) the lips part and the teeth show. With lip shape 11 (B M P) the lips stay closed. A small dark seam shows at the corner, and today's head has it too.
- **old_w_f**: the lips stand out more, as KOTOR's do. No fold showed.
- **pfha05**: the upper lip got taller and the lower lip thinner, the opposite of KOTOR's (see above).

## Pictures

All are in `/private/tmp/claude-501/-Users-david-perso-kjk/d99ab7db-ea3d-4e5d-adf7-4bf44f1bc288/scratchpad/lips/`:
- `sheets/p_bastilah.png`, `sheets/p_carthh.png`, `sheets/pmhc03.png`, `sheets/sith_app.png`: one sheet per candidate. The columns are KOTOR, today and ours, and the rows are the front and the side, in clay. KOTOR's outlines are red, today's yellow and ours green.
- `overlap_all57.png`: all 57 heads on KOTOR's clay, front and side, with KOTOR in red, today in yellow and ours in green, and the gaps in each label.
- `look_0.png` to `look_3.png`: KOTOR's 57 mouths and today's, in clay, front, three-quarter and side.
- `ends_4.png`, `ends_8.png`, `ends_c3_crop.png`, `e_width.png`: slider ends (older slider versions; the corner faults look the same).

## Not checked

- No judges, as the brief asked.
- Pair corners of the sliders were not drawn.
- Close-ups were seen on 5 heads only. The other 35 heads that moved were not looked at.
- The talk shapes were seen on pmhc03 only.
- The jaw's and the nose's measures were not rerun on the solved heads. The lip fields stop 3 mm above the nose's base and 5 mm under the fold above the chin, and the solve costs any move of the nose's base.

## How to undo

- Viewer 2: nothing to undo. The staged copy `viewer-local/viewer2-lips/` was deleted. The lip data that was briefly in `likeness-fit-data/sliders/web/lips/` was moved to the scratchpad's `web_lips_unpublished/`.
- Code, in the head-paint worktree's crate `head_sliders`: delete `src/lips.rs`, the two lines `#[cfg(not(target_arch = "wasm32"))] pub mod lips;` in `src/lib.rs`, and the `lips` block in `src/bin/sliders.rs`. These changes are logged in `$TMPDIR/kjk-slider-kit-shared-changes.md`.
- The build is in `target-lips` in the worktree. Delete it to free the space.
- Roadmap: the lips entry under "Running now".

## Commands

```
export PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH"
cd ~/perso/kjk/.claude/worktrees/head-paint && cargo build --release -p head_sliders --target-dir target-lips
S=<scratchpad>/lips; D=~/.claude/tools/likeness-fit-data/sliders; B=./target-lips/release/sliders
$B lips look $D out.png [head...]                      # KOTOR and today in clay
$B lips checks $D out.json today=<bins> ours=<bins>     # the four checks' numbers
$B lips overlay $D out.png 4 today=<bins|live> -- all   # outlines over KOTOR's clay
$B lips make $D sliders.json                            # the lip sliders
$B lips gain|ends|solve|bins|sheet|try|power|cols ...   # see the comments in lips.rs
```

## Traps

- **Other conversations break the crate's build while they edit it.** `eye_curves.rs` and `nose_base.rs` failed to compile for ten minutes. Keep a copy of a working binary.
- **The nose now loads `sliders_r2_profile.json` and `page_r2_profile.json`**, and the eyes load `sliders_curve.json` and `page_curve.json`. `jaw_edge::tools::Heads` still builds the old nose and the old eyes. `lips::tools::Live` builds what viewer 2 shows.
- **Viewer 2's own head has 5,852 vertices**, not the kit's 4,750. `viewer_map.bin` maps them.
- **The build's last smoothing erases narrow bumps** (the Cupid's bow) and spreads moves along the lips (the width). A narrow lip feature needs a drawn step after the smoothing (`set::DrawnStep`), which needs a line in `feature.rs` and a rebuilt browser module.
- **The fold count must leave out the slit.** Thin triangles inside the slit turn over under any move of one lip against the other, and no one sees them.
- **A missing border makes the front gap lie.** Check that both borders read on our head before trusting the front gap.

## Next

1. Fix the front border so it reads on every one of our heads, for example by a fixed share of the lip's depth from the crest rather than the steepest place, or from the lip paint's border. Then rerun the checks and the solve, and look at pfha05 and old_w_f first.
2. Build the mouth width as a construction that moves the corners after the smoothing, so the lips do not stretch.
3. When both work, fold the sliders into viewer 2 with the one line in `sliders.js` and the `web/lips/` files, if the result is clearly better.
