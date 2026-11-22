# The jaw edge, the throat angle and the neck width: prototype

This note was written on the night of 2026-11-21 by the conversation briefed by the head HQ (brief: `/private/tmp/claude-501/-Users-david-perso-kjk/52003e36-b8ca-4208-a733-a6d51b417e4e/scratchpad/jaw/brief-under-jaw-prototype.md`). The work serves B3 for the jaw and chin, and prepares B5. Nothing live changed. No preset was refitted. Nothing was committed.

## Verdict

The gate does not pass. The prototype is a clear win on pmhc04: all four judges chose it, in both rounds and in both row orders. It is not a win on Bastila. In both rounds, the two judges split on Bastila by row: each picked the same row, so the result is the judges' bias toward one row, not a verdict. Live won on Carth in both rounds and on pmhc03 in round 1. These are the two heads whose KOTOR throat is open. Every judge saw a crease, a step or a notch where the underside of the chin turns into the neck on the prototype. One fix softened it, and it is still there. Following the brief, I stopped after the one fix and the second round. Nothing is live, and no preset was refitted.

## What still looks broken, first

- **A crease along the jaw's lower edge.** All four judges in both rounds saw a step or crease where the underside of the chin turns into the neck, or a hard line along the jaw's lower edge. The fix of round 2 softened it but did not remove it. The enlarged three-quarter crop of pmhc04 still shows a pinched line along the edge.
- **A notch at the jaw's corners.** Judges saw a small notch near each jaw corner in the view from below on pmhc04 and Zherron in round 1.
- **A ridge down the middle of the throat** in the view from below on pmhc04. KOTOR's underside is flat there. The ridge is a vertex down the middle of our mesh that the live head already has; the lift makes it show more.
- **Bastila's throat cannot reach KOTOR's.** Our neck's rim sits right at the bottom of Bastila's throat, 70 mm behind the chin, so that part cannot move (decision 44). Her throat angle stays at 112 to 113 degrees, against KOTOR's 103.
- **The ledge does not move.** The ledge row (how far the jaw stands out over the neck) stays at 9.5 to 10.6 mm on every solve, against KOTOR's 13 to 18 mm on the heads that have one. Edge depth pushes the jaw out above the edge, but the measure barely reads it. The ledge row also fails check 1.
- **Capped ends.** Corner height at 30 mm leaves a hook at each jaw corner, and edge depth at 6 mm or more makes the jaw's corners flare like wings when the edge is not raised. One repair (a longer fade behind the corner) did not fix either. The ranges are capped at 22 mm and 5 mm, just above what the five solves use.
- **Neck width is weak.** The neck's rim lies about 38 mm under the chin at the front, so the neck can only narrow above it. At 50 percent it crumpled the neck, and the range is capped at 30 percent. Bastila, with corner height and neck width both at their top, has one turned triangle.
- **Carth and pmhc03 are better live.** Both have an open throat on KOTOR (128 and 132 degrees). Both judges in round 1 preferred live on them.

## The measure's three fixes (measured)

All in `under_jaw.rs` in the head-paint worktree.
1. **A taller window for the corner.** Each cross-section is now read from 24 mm above the chin's bottom, or from 12 mm above the last section's edge, whichever is higher, up to 57 mm. The corner ends when the edge rises more than 45 mm above the chin's bottom, or two sections in a row cannot be read. A section whose two sides disagree in height by 8 mm or more counts as unreadable. A first try also stopped the line when the edge climbed more than 6 mm between sections. That rule made the corner jump on subdivision (95th percentile wobble of the corner's place 32 mm), and it was removed.
2. **Hood and beard.** A stricter cover (3 mm instead of 6 mm, and one covered corner enough) tore real skin on Keanu and Jolee and still left Kreia's inner hood layer, which lies within 3 mm of our fit. It was dropped. Instead, the jaw-edge rows are hidden on a KOTOR head when more than a third of the edge's back half lies next to a vertex that is not skin. They are hidden on Kreia, pfha06, mod_keanu, mod_kyle_katarn and mod_narve_canderous.
3. **Short neck stubs.** The throat rows are read only when the front of KOTOR's neck reaches 34 mm below the chin's bottom without a gap. They are hidden on comm_b_m, n_admoffh, n_dorakh, old_b_m, old_w_f, pfha05 and pmba01. old_b_f cannot be measured at all (its line under the jaw is shorter than 40 mm).
4. **A new row**, `edge_back_height`: the height of the jaw's lower edge above the chin's bottom, 56 to 72 mm behind the chin's front. It is a fixed place, so it does not jump with the corner.

Check 1 after the fixes (wobble from subdivision, 95th percentile, against KOTOR's spread):

| Row | Before | After |
|---|---|---|
| Throat angle | 14.9 against 20.0 (passes) | 15.0 against 16.6 (passes) |
| Corner, behind the chin | 16.0 against 13.7 (fails) | 14.2 against 10.3 (fails) |
| Corner, above the chin's bottom | 14.6 against 12.5 (fails) | 27.4 against 16.9 (fails) |
| Ledge | 5.7 against 3.0 (fails) | 4.7 against 2.6 (fails) |
| Edge outline from the side | 5.5 against 4.5 (fails) | 10.4 against 5.7 (fails) |
| Edge outline from below | 5.3 against 3.6 (fails) | 6.5 against 3.1 (fails) |
| Width between the edges | 9.5 against 10.8 (passes) | 14.1 against 13.8 (passes narrowly) |
| Neck width | 10.5 against 13.2 (passes) | 10.7 against 13.3 (passes) |
| Jaw width less neck width | 13.5 against 18.3 (passes) | 16.0 against 18.9 (passes) |
| Edge height at the back (new) | | 3.8 against 4.6 (passes) |

The four rows that failed check 1 (the corner's place, the ledge and the two edge outlines) still fail. The solve therefore weighs them less, and the prototype is steered by the rows that pass: the throat angle, the neck's slope, the neck's width and the edge's height at the back. Check 3 is unchanged: the subdivided copy finds itself for 34 of 56 heads (32 before).

KOTOR's means after the fixes: throat angle 120 degrees, edge height at the back 19.3 mm, ledge 11.8 mm, neck width 72 mm. The live jaw presets' means: 136 degrees, 7.6 mm, 9.5 mm, 77.5 mm.

## The prototype

Code: `crates/character_customisation/head/head_sliders/src/jaw_edge.rs` (new), run as `sliders jawedge <command>`. Viewer 2 does not use it. It moves the built head after its sliders and smoothing, in the frame of the eyes, from the head's own chin. One zone weights every control: 0 on held vertices (the rim and below, the ears, the inside of the mouth) and on the lips, fading in over 14 mm from the held vertices and 8 mm from the lips, so the rim stays at 0.000 mm (decision 44).

The four values:
- **Jaw corner height**, 0 to 22 mm. The jaw's lower edge is a line on each side from the chin to a corner 88 mm behind the chin's front. It rises fast behind the chin and levels toward the corner, up to this height above the chin's bottom. On the back half of the line, the skin of the jaw's sides below the line is drawn up toward it. Its measure is `edge_back_height`. At 0 nothing moves. At 22 the jaw has a clear corner and a level line under it. At 30 (past the cap) a hook forms at each corner.
- **Jaw edge depth**, 0 to 5 mm. On the back half of the line, just above the edge, the jaw's sides are pushed out by this much, so the jaw stands out over the neck. Its measure is the ledge, which barely answers.
- **Throat angle**, -30 to 95. The underside between the jaw's edges is drawn up toward a ceiling by this percentage of its distance below it, from 15 mm behind the chin back. The ceiling is the chin's bottom in the middle and the edge's line at the sides. The lift eases in over 9 mm under the ceiling and never passes a place over another, so nothing folds. At 95 the underside runs level into a sharp turn down the neck, close to pmhc04. At -30 the throat is softer than live.
- **Neck width**, -15 to 30. The front of the neck below the jaw is drawn toward the middle by this percentage. Positive values make it thinner.

The corner height and the throat share one ceiling. The larger of their two shares is used at each place, so they do not add up into a ridge.

## Solves (measured, 5 heads, from viewer 2's live head)

`solve` searches the 5 jaw sliders alone ("old"), then the 5 jaw sliders and the 4 new values ("new"), from the live values, on the under-jaw measure. It holds the jaw measure where live has it, and costs every fold. The heads are built exactly as viewer 2 builds its default slider head: the five features' sets merged, built on `starts_all.bin` with each head's solved values.

Round 2 (after the fix), in the measure's units. Corner height is the measure's own corner row; edge back is the new row at a fixed place. "folds" counts turned triangles near the change.

```
                      throat_ang neck_slope corner_hei      ledge edge_width neck_width edge_back_
pmhc04
  kotor                     89.7       -3.2        8.0       14.3       77.3       69.7       19.1
  preset folds   0        127.6      -30.1      -18.8        9.8       58.6       70.8        1.4
  old    folds   0        121.2      -17.9       11.2       10.5       66.2       74.9       13.5
  new    folds   0        102.6       -5.8       11.2        9.9       84.9       75.9       18.1
   jaw [-5.5, 1.5, -12.0, 3.9, 20.1] proto [22.0, 3.3, 81.5, 30.0]  old jaw [-1.4, 8.9, -10.8, 3.9, 20.1]
p_bastilah
  kotor                    103.4        5.6       19.5        9.0       85.6       83.6       21.8
  preset folds   0        118.2      -10.9       -8.2        9.2       64.7       86.4        3.3
  old    folds   0        111.4       -9.7       -5.5        9.6       64.2       87.2        7.5
  new    folds   0        112.7       -5.9       20.2        9.6       80.0       86.4       18.9
   jaw [-2.4, 5.9, -7.0, 0.1, 9.6] proto [13.1, 2.8, 95.0, -15.0]  old jaw [-2.4, 10.6, -9.5, 2.5, 12.5]
p_carthh
  kotor                    131.7      -30.4       14.0       13.3       72.3       65.5       13.3
  preset folds   0        134.3      -32.3      -31.8        9.2       64.3       66.7        1.7
  old    folds   0        132.7      -30.7      -21.3        9.7       66.1       66.2        5.5
  new    folds   0        134.6      -33.0        9.2        9.6       74.0       65.6       10.1
   jaw [2.3, -1.6, -2.0, 2.2, 13.5] proto [3.1, 0.0, 10.0, 5.0]  old jaw [2.3, 1.9, -2.0, 4.2, 18.8]
n_zherronh
  kotor                    106.2       -2.3       19.8       10.2       90.9       75.7       19.1
  preset folds   0        137.7      -32.1      -28.5        9.2       59.3       67.8        8.2
  old    folds   0        133.3      -26.3      -16.0        9.7       64.6       72.6       14.2
  new    folds   0        111.3       -6.1       15.0       10.3       91.4       76.3       17.4
   jaw [-0.8, -0.2, -4.0, 1.1, 12.9] proto [19.1, 2.6, 95.0, 20.4]  old jaw [1.8, 6.0, -6.5, 3.2, 20.1]
pmhc03
  kotor                    128.0      -34.9       10.2       18.5       79.1       53.3       13.8
  preset folds   0        137.5      -23.3      -40.0        8.5       41.4       66.8       19.8
  old    folds   0        139.2      -28.3       16.5       10.0       76.5       65.9       16.5
  new    folds   0        136.8      -28.5       13.0       10.5       73.7       64.5       13.9
   jaw [8.5, 3.6, -1.6, 1.6, 18.9] proto [3.1, 1.6, 0.0, 30.0]  old jaw [10.0, 3.6, -1.6, 1.6, 13.6]
```

- The old jaw sliders alone bring the cost from 110 to 200 down to 44 to 137. With the prototype it falls to 6 to 38.
- The throat angle with the prototype: pmhc04 103 degrees (KOTOR 90, live 128), Bastila 113 (KOTOR 103, live 118), Zherron 111 (KOTOR 106, live 138). Carth and pmhc03 stay near live, which is near KOTOR.
- The edge's height at the back reaches KOTOR on every head (within 1 to 4 mm).
- The ledge does not move (see above). The width between the edges overshoots on pmhc04 (85 mm, KOTOR 77).
- Throat angle sits at its top (95) on Bastila and Zherron, and neck width at its top (30) on pmhc04 and pmhc03.

## Ends and pairs (seen)

Every value at both ends on the base head and the 5 stand-ins, and every pair at its four corners on the base head, pmhc04 and Bastila, in clay from the side, the front, three-quarter from below and from below. The fold count found one turned triangle in all of them (Bastila, corner height and neck width both at their top). Seen in the pictures: the throat at 95 draws a horizontal line across the front of the neck, which is the sharp throat seen from the front. Corner height and throat both at their top make a hard shelf with a small notch where the chin meets the jaw.

## Judges (two rounds, two fresh Fable judges each, KOTOR in the middle row, rows swapped for the second judge)

The order of the rows was drawn at random per head; the keys are in `judge1/key.json` and `judge2/key.json`.

Round 1 (before the fix):

| Head | Judge 1 | Judge 2 (rows swapped) | Result |
|---|---|---|---|
| pmhc04 | prototype, 7 to 4, high | prototype, 6 to 5, low | prototype |
| Bastila | live, 7 to 4 | prototype, 7 to 4 | split by row (both chose the bottom row) |
| Carth | live, 6.5 to 4 | live, 7 to 5 | live |
| Zherron | prototype, 6 to 4 | live, 6 to 5 | split by row (both chose the bottom row) |
| pmhc03 | live, 6.5 to 4 | live, 7 to 5 | live |

Judge 1 chose the bottom row on all five heads. Both named a step at the throat, a hard line along the jaw's lower edge and a notch at the jaw's corners on the prototype.

The fix: the lift eases in over 9 mm under the ceiling (was 4), the edge's push over 8 mm (was 5), the corner fades over 45 mm behind it (was 35), and the sides come in over 30 mm across (was 22). Then the five heads were solved again.

Round 2 (after the fix, two fresh judges, with enlarged side and three-quarter crops for pmhc04 and Bastila):

| Head | Judge 1 | Judge 2 (rows swapped) | Result |
|---|---|---|---|
| pmhc04 | prototype, 6 to 4, medium | prototype, 6 to 5, medium | prototype |
| Bastila | prototype, 6 to 5, low | live, 7 to 5, medium | split by row (both chose the top row) |
| Carth | live, 6 to 4 | live, 7 to 5 | live |
| Zherron | live, 6 to 4 | prototype, 7 to 5 | split by row (both chose the bottom row) |
| pmhc03 | prototype, 6 to 5, low | live, 7 to 5 | split by row (both chose the top row) |

On Bastila, judge 2 found the prototype's jaw edge and throat too sharp for her soft, rounded target, with a pinched crease under the jaw corner and a diagonal ridge into the neck. Both round 2 judges again named the notch or crease where the jaw meets the neck under the ear.

What I saw myself: on pmhc04 the prototype's profile is clearly closer to KOTOR's sharp throat, but the three-quarter view from below shows a pinched line along the edge and a ridge down the middle of the throat that KOTOR does not have. On Bastila, the prototype has a crisper jaw line from the front, like KOTOR's, but a crease runs from the jaw corner into the neck.

## Not checked

- The ends and pair corners were drawn in clay only, and before the round 2 fix. They were not drawn in viewer 2's light.
- The mouth was checked open in one lip shape (AA AE AH) in viewer 2, on the five heads, with the rig from the fit. The live rig of the slider head was not used, because these shapes keep the rig from the file.
- Only 5 heads were solved. The other 52 heads and the 57 presets were not touched.
- Check 2 of the measure (does it move with what David notices) was not run with David.

## Pictures

All under `/private/tmp/claude-501/-Users-david-perso-kjk/b96a544e-3958-4dd7-8cb5-b3d8bea7c7da/scratchpad/`.
- `david/clay_<head>.png` for pmhc04, p_bastilah, p_carthh, n_zherronh and pmhc03: KOTOR, live, the old sliders' solve and the prototype's solve, labelled, from the side, the front, three-quarter from below and from below. These are round 2.
- `ends/ends_<head>.png`: each value at both ends on the base head and the 5 stand-ins. `ends/pairs_<head>.png` and `ends/pairs_zoom.png`: every pair at its four corners on base, pmhc04 and Bastila. These were made before the round 2 fix.
- `p/sl5.png`: cross-sections of KOTOR (red), live (grey) and an early prototype (white).
- `v2/shots/` and `v2/shots2/`: viewer 2 in "In game" light, front, three-quarter, and three-quarter with the mouth open (lip shape AA AE AH), for KOTOR, live and the prototype.
- `judge1/j1`, `judge1/j2`, `judge2/j1`, `judge2/j2`: the blind sheets, with `key.json` beside them.
- `m/fix1_0.png`: the measure's points after the corner fix on six heads.

## How David can see it

Viewer 2 with a URL option. The default page does not change.
- http://127.0.0.1:8765/viewer2/index.html?shapes=data/jawproto/shapes.json&shape=jaw_proto&head=pmhc04&hair=none
- Change `head=` to p_bastilah, p_carthh, n_zherronh or pmhc03. Other heads have no file and fall back to the fit.
- The Shape group then lists "Jaw edge prototype", "Jaw solved with the old sliders", and "Live jaw (rebuilt here, for comparing)", next to the default "Made by the sliders". "Original" in the comparison buttons shows KOTOR's head. The "Neck" and "Side" cameras show the area.
- These shapes keep the rig from the file, not the live rig, so the mouth opens as on the fit.

## How to undo

- Viewer: delete `/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data/jawproto/`. Nothing else in viewer 2 changed.
- Code: delete `src/jaw_edge.rs`, the line `pub mod jaw_edge;` (and its `cfg` line) in `src/lib.rs`, and the `jawedge` block in `src/bin/sliders.rs`. To undo the measure's fixes, copy back `/private/tmp/claude-501/-Users-david-perso-kjk/b96a544e-3958-4dd7-8cb5-b3d8bea7c7da/scratchpad/under_jaw.rs.orig` over `src/under_jaw.rs`. `jaw.rs` is unchanged.
- Roadmap: the two lines under "Running now".

## Traps

- **The neck's rim is right under the jaw.** At the front of the neck it lies about 38 mm under the chin and 64 mm behind it. At the sides it lies 26 mm under the chin and 84 mm behind it, right under the jaw's corner. Any control that pulls skin in under the jaw's back half meets the rim's fade and tears. Push the jaw out above the edge instead, or lift.
- **Lift, do not tilt.** Turning the neck's front forward about the throat made the throat angle worse, because the rim holds the bottom of the neck. Drawing the underside up toward a ceiling works, and a share below 1 of the distance below it can never fold.
- **Two lifts side by side make a ridge.** A throat lift in the middle and a corner lift at the sides, with different strengths, made a rope down the side of the neck. One ceiling and the larger of the two shares fixed it.
- **A hard ceiling makes a crease.** The lift must ease in under the ceiling (9 mm now). The remaining crease along the edge comes from the same place.
- **The corner's place jumps.** Any rule that ends the line under the jaw on one section (a climb, a side mismatch) makes the corner's place jump between a head and its subdivided copy. Read heights at fixed places behind the chin instead.
- **The stricter cover tears skin.** One covered corner per triangle, at 3 mm, removed real skin on Keanu and Jolee. KOTOR's skin mask marks many necks as not skin, so it cannot be used on its own either; it is only used on the edge's back half.
- **The solver uses the throat and neck width to fix each other.** Lifting the underside widens the neck at the measure's height (24 mm under the chin), and the solver pushes neck width up to compensate. Watch for a thin, columnar neck in the front view.
- **Viewer 2 framing.** Its "Side" camera frames the whole head small. The `shoot.mjs` script (below) drives a headless Chrome over the DevTools protocol, hides the panels, pans and turns the camera. Never open a visible browser.
- **The jaw preset's values for the live head** come from each feature's `page.json` under `heads[i].solved.full.counts[last].values`, matched to the set's sliders by name, as `sliders.js` does.

## Tools

- `sliders jawedge probe|sheet|slices|solve|solve-sheet|viewer-bins <data> ...` (see the comments in `jaw_edge.rs`). `slices` draws cross-sections of KOTOR, live and the prototype across the jaw, which is the fastest way to see what a change does.
- `/private/tmp/claude-501/-Users-david-perso-kjk/b96a544e-3958-4dd7-8cb5-b3d8bea7c7da/scratchpad/v2/shoot.mjs`: headless viewer 2 screenshots (`node shoot.mjs <jobs.json> <out dir>`), and `mont.py` to stitch them.

## Next

The prototype is not ready for a full build. The next step should rebuild the edge so that it is crisp without a crease, probably by placing a row of vertices on the edge line, as the eye curve lays the visible rim on its curve. It should also find a ledge measure that passes check 1. Then judge again on pmhc04 and Bastila.
