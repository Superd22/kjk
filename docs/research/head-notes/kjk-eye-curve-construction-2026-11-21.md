# The eye opening built from a curve on the eyeball (2026-11-21)

Written by the conversation started from the brief `eye-curve-construction` for the lead kjk-48 (Eyes HQ). The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/d55dbdf6-456c-4c14-ac07-1a61b76e9013/scratchpad`, called `$S` below.

Nothing live changed. The worktree, the live data, the shared kit, viewer 2 and the live wasm are untouched, and nothing is committed. All the code is in my own copy of the shape-only diagnosis tool, `$S/diag`. It links the worktree's crates read only. The prototype is the module `$S/diag/src/curve.rs`. Every number and picture below comes from the live data folder (`~/.claude/tools/likeness-fit-data/sliders`), read only. So each head keeps its live whole-eye values and the rest of its live head.

## Verdict

**Step 1.** The construction can make KOTOR's opening shapes cleanly on most heads, with a few clear faults.

- It reproduces the curve it is fed within 0.08 mm (the median over 54 heads). It does so in every view: the outline overlap is 0.98 from the front, 0.98 from the temple side and 0.92 in profile.
- The pictures show a clean opening, clean skin round it and a clean closed eye on most heads. That holds for 7 of the 8 test heads in the open views.
- Against KOTOR's own curve, the median gap falls from 2.16 mm (live) to 1.46 mm. The outline overlap from the front rises from 0.61 to 0.80, and from the temple side from 0.54 to 0.79. The rest of the gap is the eyeball's size. KOTOR's eyeballs are 1.0 to 1.7 times larger than ours, the female heads most of all, and the whole-eye sliders stay at live.
- What is broken is listed in "What is broken or unproven" below. In short:
  - mod_exile_canon has a skin flap across the opening.
  - pmha02 has small steps on its outer upper lid.
  - Outer tips that lie behind the eyeball's equator (Kreia, pmha01) are reached, but the skin round them hides them.
  - The construction folds many more triangles than live by the fold check's count, although most of them cannot be seen.
  - The closed lid's edge is slightly jagged in close-ups.

**Step 2.** Ten handles describe KOTOR's curves well.

- The fit loses little. The median gap to the fed curve grows from 0.08 mm (step 1, the direct curve) to 0.18 mm. The gap to KOTOR's own curve stays the same (1.46 against 1.47 mm).
- **The handles' ends are not all clean.** This is the test the brief sets, and the construction fails it on some handles.
  - The tip moves give slabs and flaps at their ends. Those are Inner corner place at its high end, Inner corner height at both ends, Outer corner height at its high end, and Outer corner place at its high end.
  - The flattest arches give ledges (Upper lid height low, Lower lid depth low).
  - A very pointed inner corner cracks.
  - The middle of every range, and most low ends, are clean.

My reading is this. The construction is sound where the new curve stays within a few mm of the live lid rim. It breaks where the curve moves far from it, because the live skin round that place was never shaped for an opening there. The next step should fix that before any preset is solved. See "How to make it the default".

## 1. The construction and why

**What I found in the live mesh first.** Each eye's lid edge loop (`lid_edges.json`, 28 vertices) is not where the skin meets the eyeball. Counting layers out from the loop on 5 heads, the skin goes out and back. Layers 1 and 2 run back toward the eye, under the lid. The fold, which is the visible lid rim, is layer 3 on every head checked. The flat-map triangles turn over between layers 2 and 3 (`cfold` command). So the loop itself sits under the lid against the eyeball, and the skin rolls over it. That is the "roll" earlier notes talked about.

**The construction, per eye, in the eye's frame** (`build_eye` in `curve.rs`):

1. **The curve.** It is a closed curve of directions from the eyeball's centre. In step 1 it is KOTOR's curve. KOTOR's 72 points and the dense corner curves are taken by their direction and set on our eyeball. In step 2 it is drawn from the handles.
2. **The rim on the curve.** Layer 3 (28 vertices) is laid on the curve, 0.15 mm outside the eyeball. Each vertex goes to its share of its lid's length, from the inner tip to the outer tip.
3. **Under the lid.** Layers 0 to 2 (the loop and the roll) are laid just outside the curve, at 0.03 radians more per layer, 0.06 mm above the eyeball. So they lie between the eyeball and the lid, out of sight. Each takes its place along the rim from its neighbours one layer further out.
4. **The front skin** (every layer after the rim, up to 15 mm from the live or the new rim).
   - Its place on the eyeball is the live place, stretched along rays from the curve's middle. A vertex keeps its distance outside the live rim, now measured from the new curve. The stretch fades out over 0.8 radians and over the ring's outer 40%.
   - Skin that dove into the live inner pocket keeps its depth, mirrored outward.
   - Then 40 rounds of smoothing (mean value weights from the live mesh) relax the moved part. Every vertex is held at least 0.015 radians per layer outside the curve, pushed out along the curve's normal.
5. **The front skin's height.**
   - Within 0.25 radians of the curve, it comes from the live skin's surface: the first outward-facing skin along each direction from the eyeball's centre, sampled on a grid. It is brought down to 0.15 mm at the curve.
   - Further out, each vertex keeps its own live height.
   - Everywhere, the skin may rise from the curve no steeper than 1.5 mm per mm, by a smooth minimum. This cap was added for step 2's ends. It softens the slabs, but lowers the skin a little.
6. **Inside the loop.** The vertices inside the loop (the socket lining, 280 per eye) follow the loop vertex they are reached from. That follow fades over 6 steps. They go 1 mm under the eyeball's surface where they lie within 120 degrees of straight ahead.
7. **The blink built from the curve** (`lid_piece`).
   - The lid piece is the upper arch, made 4 mm longer past each tip, swept about a line through the eyeball's centre. The line is parallel to the line through the two tips.
   - The piece sits 0.08 mm above the eyeball. Closed, it reaches past the lower arch everywhere by 6 degrees. The angle is 50 to 60 degrees on the test heads, against KOTOR's 35.
   - Open, the piece lies above the upper arch, under the skin. Closed, any part of it outside the curve lies under the skin, which is at least 0.15 mm up. So only the part over the opening shows.
   - This keeps the rig contract: one rigid piece turning about the eyeball's centre.

**Why this method and not another.**

- I first tried a smooth 3D move (biharmonic) from the old loop to the curve. It left the roll over the opening, and then tore holes in the lower lid. Those pictures are `$S/s1/a_pmhc04_s.png` and `$S/s1/h_pmhc04_s.png`.
- A harmonic layout in the flat map unrolled the roll, but it pulled the skin beside the nose over the inner corner, and it made lumps.
- What worked was three things. First, the rim at the fold. Second, a stretch that keeps every vertex outside the curve. Third, heights from the live skin only near the curve.
- Three traps cost the most time:
  - The hidden-group vertices must move with the rest, or they leave spikes.
  - Pushing vertices out along rays from the middle piles them up at a long pointed inner corner. Pushing them out along the curve's normal does not.
  - The sampled live surface jumps between layers beside the nose. Used far from the curve, it made a long ledge from the inner corner to the nose.

## 2. Step 1's verdict, with pictures

The pictures are in `$S/final1/` (each has a `_s.png` copy at 1600 px). All of them are shape only: no lid piece when open, no lash strip, no caruncle, plain eyeballs and no hair.

- `p_<head>.png` for pmhc04, p_bastilah, pmha01, p_kreiah, p_carthh, n_zherronh, pfha05 and p_joleeh.
  - The rows are KOTOR, live, the prototype, and the prototype closed.
  - The columns are the front, three-quarter from the temple, three-quarter from the nose, and profile.
  - The nose view is now a working one. The skin is cut 3 mm on the nose's side of the inner tip, and the eyeball is drawn only up to 6 degrees past the opening.
  - KOTOR's curve, set on our eyeball, is yellow on the prototype row. Ours is cyan.
- `p_corners_<head>.png` has close-ups of both corners: the inner corner from the front and the nose side, and the outer corner from the front, the temple side and in profile. Its rows are KOTOR, the prototype open, and the prototype closed.
- `overview57.png` covers all 57 heads. Each head has KOTOR from the front and the temple side, then the prototype, then the prototype closed. Open the full one, because the small copy is unreadable.

What I see in them, broken things first:

- **mod_exile_canon is broken.** A flap of skin stands across the opening, and the closed eye shows it. KOTOR's opening is much larger than the live one there. Some skin triangles span the new opening even though all their corners lie outside it. A pass that pushed such triangles out made things worse, with two spikes on exile_canon and slabs on pmha02, so it is off (`CC_ROUNDS`).
- **Three heads fail because KOTOR's own curve is misread there.** mod_hp_pfha01 has a slit that sends the curve into the brow. mod_narve_canderous reads closed. On mod_tcs_bastila the curve wanders. The shape fit note found the same three. One eye of mod_hp_pfhb02 is misread too (its inner tip reads at 110 degrees), and pfha05's second eye is doubtful.
- **pmha02** has small steps on the upper lid near the outer corner, open and closed.
- **The outer tips behind the equator.** KOTOR's outer tip lies 97 to 104 degrees from straight ahead on Kreia and pmha01. The rim is laid there, but the thin wedge is hidden by the skin round it in every view. So the measure reads our tip 5 to 6 mm short. A slope limit on the skin did not help.
- **pfha05's long, thin inner point** reads blunter than KOTOR's, because the front triangles there are wider than the point.
- **Thin fans of triangles** beside the nose and under the outer corner. They are faint in the pictures but real. The fold check counts 69 to 144 folded triangles on the 8 heads (median 80 over 54 heads), against 0 to 2 live. In the coloured check (`$S/s3/fo_p_kreiah_s.png`) they are the squeezed triangles beside the nose and under the outer corner.
- **The closed eye.**
  - It shuts on every test head.
  - In close-ups, the lid piece's lower edge is slightly jagged where the skin's flat triangles dip under the piece.
  - Bastila and Jolee leave about 1 to 1.5 mm² of eyeball bare from the front, at the inner corner.
  - The median over 54 heads is 0.31 mm² from the front.

What is good: the opening's outline lies on KOTOR's in every view on pmhc04, Bastila, Kreia, Carth, Zherron, Jolee and pmha01. The corners are pointed where KOTOR's are. The skin round the eye is smooth and keeps the live head's brow, cheek and nose.

**The measures** (`$S/final1/all_m.json`, printed by `python3 $S/summ.py $S/final1/all_m.json` and `python3 $S/cmp12f.py`). "Fed" means against KOTOR's curve set on our eyeball. Each cell is the mean of both eyes. The four regions are the inner corner, the outer corner, the upper arch and the lower arch.

| Head | Curve, live against KOTOR (mm) | Curve, prototype against KOTOR | Prototype against the fed curve | Regions against the fed curve | Ring, live then prototype | Overlap front / temple, live then prototype | Folds, live then prototype |
|---|---|---|---|---|---|---|---|
| pmhc04 | 1.75 | 0.54 | 0.07 | 0.10 / 0.13 / 0.03 / 0.06 | 1.69, 1.33 | 0.54 / 0.53, 0.92 / 0.91 | 0, 69 |
| p_bastilah | 1.97 | 1.33 | 0.12 | 0.07 / 0.20 / 0.15 / 0.10 | 1.23, 0.82 | 0.82 / 0.61, 0.80 / 0.81 | 0, 97 |
| pmha01 | 0.98 | 0.65 | 0.09 | 0.07 / 0.17 / 0.11 / 0.04 | 1.20, 1.16 | 0.73 / 0.68, 0.89 / 0.89 | 0, 81 |
| p_kreiah | 1.65 | 0.67 | 0.08 | 0.06 / 0.18 / 0.10 / 0.02 | 1.45, 1.51 | 0.60 / 0.43, 0.91 / 0.89 | 0, 144 |
| p_carthh | 1.51 | 1.26 | 0.08 | 0.14 / 0.03 / 0.08 / 0.05 | 1.14, 1.00 | 0.80 / 0.66, 0.82 / 0.84 | 0, 94 |
| n_zherronh | 2.67 | 2.58 | 0.05 | 0.15 / 0.03 / 0.04 / 0.03 | 2.09, 1.99 | 0.67 / 0.54, 0.65 / 0.64 | 0, 70 |
| pfha05 | 6.84 | 6.58 | 0.12 | 0.17 / 0.03 / 0.08 / 0.18 | 6.96, 6.58 | 0.57 / 0.40, 0.42 / 0.41 | 0, 80 |
| p_joleeh | 4.31 | 4.12 | 0.16 | 0.17 / 0.15 / 0.17 / 0.14 | 1.77, 1.44 | 0.78 / 0.60, 0.54 / 0.54 | 2, 136 |
| Median of 54 | 2.16 | 1.46 | 0.08 | 0.10 / 0.08 / 0.08 / 0.03 | 2.02, 1.62 | 0.61 / 0.54, 0.80 / 0.79 | 80 in all on 9 heads, then a median of 80 per head |

Zherron, pfha05 and Jolee barely improve against KOTOR's raw curve. That is not the construction. Their KOTOR eyeballs differ in size from ours (0.85, 1.68 and 0.83 times), and the curve is carried by direction. Against the fed curve they are within 0.05 to 0.16 mm.

## 3. The handles

Each handle is one number per head, used for both eyes, mirrored. The curve is drawn in the flat map of the eyeball: the angle from straight ahead, along the direction round the forward axis, times the eyeball's radius. That is in mm along the eyeball's surface. Each arch is a bump on the chord from the inner tip to the outer tip. The bump is flat at its peak, and near each tip it is shaped by that corner's shape handle (`Target::from_handles` in `curve.rs`, `arch` and `shape_q`).

The ranges come from the fits to the KOTOR heads. They are the lowest and the highest fitted value with a 10% margin. Six heads are left out: the three misread mods, plus mod_hp_pfhb02, pfhc02 and pmhc03, whose curves are misread on one eye or at one corner. The values are in `$S/step2/handles.json` and the ranges in `$S/step2/ranges.json`.

| Name | Range (median) | Hint | What it looks like at its ends (pmhc04 and Bastila) |
|---|---|---|---|
| Inner corner place | 6.0 to 18.3 mm (11.1) | Moves the inner corner's tip toward the nose and round the eyeball. At a higher value the eye reaches further toward the nose. | Low: a shorter eye, clean. High: the skin beside the nose makes a slab, and the opening stops short of the tip. **Not clean.** |
| Inner corner height | -7.3 to 2.9 mm (-1.7) | Moves the inner corner's tip down or up on the eyeball. | Low: the lower lid makes a ledge, and skin covers the lower inner corner. High: a small slab above the inner corner. **Not clean at either end.** |
| Outer corner place | 10.5 to 26.7 mm (19.4) | Moves the outer corner's tip toward the ear and round the side of the eyeball. | Low: a short, round eye, clean. High: a small notch at the outer tip, and on pmhc04 the tip passes behind the skin. **Not clean.** |
| Outer corner height | -5.6 to 5.4 mm (-0.3) | Moves the outer corner's tip down or up on the eyeball. | Low: clean on pmhc04, with a small spur at the tip on Bastila. High: a flap of skin over the upper outer opening. **Not clean.** |
| Upper lid height | 2.0 to 9.6 mm (5.1) | Sets how high the upper lid's edge stands above the line between the two tips. | Low: a narrow eye, but the flattened lid leaves a ledge near the tips. High: a tall, round opening, clean. |
| Upper lid peak place | 0.20 to 0.79 (0.42) | Sets where along the eye the upper lid is highest, from near the inner corner to near the outer corner. | Clean at both ends on pmhc04 and Bastila. Raised fold counts on pfha05 at the low end. |
| Lower lid depth | 0.3 to 7.5 mm (3.5) | Sets how far the lower lid's edge sits below the line between the two tips. | Low: a straight lower lid, with ledges at both corners. **Not clean.** High: a deep lower lid, clean. |
| Lower lid low point place | 0.19 to 0.85 (0.54) | Sets where along the eye the lower lid is lowest. | Clean, apart from a small wedge at the inner corner on pmhc04 at the high end. |
| Inner corner shape | -1.5 to 0.9 (0.0) | Sets how the inner corner closes. At a negative value it is a long, sharp point. At a positive value it is blunt. | Low: the point is sharp but cracks at its tip (a dark notch). High: blunt, clean. |
| Outer corner shape | -0.5 to 0.7 (0.0) | Sets how the outer corner closes. At a negative value it is a sharp point. At a positive value it is blunt. | Clean at both ends. |

The ends sheets are `$S/final2/ends/e_<head>.png` for the 8 heads, with the row counts in `e_<head>.json`. The first row is the head's fit. Then each handle has a row at its low end and a row at its high end, with the others at the fit. The columns are:
- the front and the temple 3q, open;
- the same two views closed;
- the inner corner close-up from the front, open and closed;
- the outer corner close-up from the temple side, open and closed.

The yellow line is the curve the handles draw, and cyan is what the measure reads. The halves I looked at are `v_<head>_a.png` and `v_<head>_b.png` in the same folder.

## 4. What the handle fit loses

The fit is `$S/py/fit_handles.py`. It uses scipy's bounded least squares on both eyes at once, with the curve's points both ways and the two tips weighted twice. It runs in 15 seconds. The loss is measured on the built heads (`$S/final2/fit_m.json`, `python3 $S/cmp12f.py`), against the fed curve, in mm.

| Head | Direct curve (step 1) | Handles | Regions with handles (inner / outer / upper / lower) |
|---|---|---|---|
| pmhc04 | 0.07 | 0.12 | 0.15 / 0.24 / 0.10 / 0.08 |
| p_bastilah | 0.12 | 0.15 | 0.10 / 0.24 / 0.15 / 0.12 |
| pmha01 | 0.09 | 0.18 | 0.24 / 0.32 / 0.13 / 0.12 |
| p_kreiah | 0.08 | 0.12 | 0.12 / 0.21 / 0.12 / 0.06 |
| p_carthh | 0.08 | 0.15 | 0.20 / 0.15 / 0.08 / 0.17 |
| n_zherronh | 0.05 | 0.11 | 0.22 / 0.09 / 0.09 / 0.07 |
| pfha05 | 0.12 | 0.34 | 0.39 / 0.24 / 0.17 / 0.55 |
| p_joleeh | 0.16 | 0.31 | 0.34 / 0.33 / 0.35 / 0.21 |
| Median of 54 | 0.08 | 0.18 | 0.24 / 0.24 / 0.16 / 0.12 |

The largest losses are on heads whose KOTOR curve is misread at one place:
- mod_hp_pfhb02, at 2.8 mm, is misread on one eye;
- pfhc02, at 1.2 mm, has the grey sliver on its lower arch;
- pmhc03, at 0.7 mm, has its inner corner misread.

After those come n_admoffh (0.50 mm) and pfha05 (0.34 mm). pfha05's long inner point and its lower lid's sharp turn are what the arch shape cannot follow. With the handles, the folded triangle count is about the same as step 1's (median 86 against 80). The bare eyeball with the eye closed is lower (0.15 mm² against 0.31 mm²).

## 5. What is broken or unproven

- **The ends of the tip moves and of the flattest arches are not clean.** This is the main open fault, and it is the test the brief sets. When the curve moves far from the live rim, the live skin there has the wrong height and the wrong amount of skin for an opening. The stretch can only slide vertices along the surface. The slope cap of 1.5 mm per mm softens the walls into slopes, but it does not remove them, and it lowers the ring a little (from 1.56 to 1.62 mm against KOTOR).
- **mod_exile_canon**, where skin triangles span a new opening that is much larger than the live one.
- **The fold check counts 70 to 160 folded triangles per head**, against 0 to 2 live. In the pictures they are thin squeezed fans, not tears. I have not proven that none of them shows in viewer 2's light.
- **The closed lid's edge is slightly jagged** in close-ups, and 1 to 1.5 mm² of eyeball shows at Bastila's and Jolee's inner corners when closed. The closing angle is 50 to 60 degrees, more than KOTOR's 35, because the piece is the upper arch swept down past the lower one.
- **Outer tips behind the equator** cannot be seen past our skin. KOTOR's larger eyeballs show them.
- **The eyeball's size.** The curve is carried by direction, so on heads with a much larger KOTOR eyeball (all the female heads, 1.3 to 1.7 times), the opening has KOTOR's angles but not KOTOR's millimetres. The shape note's step 0 on the female eyeballs is still open.
- **Not checked.** I did not check the base head, glances, the caruncle or the lid pieces' look with paint. Viewer 2 is not covered either, because nothing is published.

## 6. How the next step would make it the default

1. **Fix the far moves first**, before any preset is solved.
   - Let the curve's move also drive the front skin's amount and height, not only its place. One way is to let the stretch bring skin from further out when the curve moves outward, and to lower the live heights round the new curve by a smooth field instead of a per-vertex cap.
   - Then render the ends again on the 8 heads until every row is clean. `cends` does this in 20 seconds for all 8 heads.
2. Fix the triangles that span a much larger new opening (mod_exile_canon). One option is to subdivide nothing, but to move rim vertices along the curve by arc length rather than by the live share, so the front ring's first layer follows the new length.
3. Port `curve.rs` into `head_sliders` as a new step after the whole-eye sliders. It would replace the press, the corner carve, the free stretch, the eye paste and the bump sliders for the opening. It runs in about a second per head here, in release, without any tuning. It will need the same speed in the wasm build.
4. Give the lid piece builder the curve. `head_eyes::piece` would take the upper arch and the axis, and viewer 2's `pieceGroup` would turn the piece about `info.axis`, as the corners note already describes.
5. Add the ten handles to the slider set with the ranges above, and drop the old opening sliders from the page only after David has seen the ends.
6. Then solve the presets with the handles as the only opening sliders, holding Eye spacing, depth and height at live.

## 7. Traps

- The lid edge loop in `lid_edges.json` is under the lid. The visible rim is the third layer out from it.
- Hidden-group vertices must move with the skin round them. Leaving them behind makes spikes across the opening, which look like a construction fault.
- KOTOR's eyeballs are not our size. Compare against the fed curve (KOTOR's directions on our eyeball) to judge the construction, and against KOTOR's raw curve to judge the likeness.
- Push vertices out of the curve along its normal, not along rays from its middle. Rays pile vertices up at long pointed corners.
- Use the sampled live surface only near the curve. Far from it, the first-hit surface jumps between skin layers beside the nose.
- A lid piece lower than about 0.08 mm above the eyeball loses to the eyeball's facets and doubles the bare area. A higher one pokes through the skin's flat triangles.
- In zsh, `$heads` does not split into words. Use `${=heads}`.
- The `CC_ROUNDS` triangle push pass makes things worse. Leave it at 1.

## Commands

The binaries are `$S/cc_final` (all results above) and `$S/cc_step1` (the first step 1 run, without the slope cap). The source is `$S/curve_final.rs` and `$S/main_final.rs`, the same as `$S/diag/src`. Build with `PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH" cargo build --release --offline` in `$S/diag`. It takes about 20 seconds. The data folder is `L=~/.claude/tools/likeness-fit-data/sliders`.

- `cproto $L <prefix> <heads|all>` builds each head on KOTOR's curve and writes the sheets and `<prefix>_m.json`.
  - `CC_NO_SHEETS=1` writes the numbers only.
  - `CC_HANDLES=<handles.json>` builds from the handles, and `CC_SET="Inner corner place=12;..."` sets handles on top.
  - `CC_TINT=1` colours the parts: green is the front ring, yellow the rim, blue under the lid, red inside the loop.
  - `CC_FOLDS=1` colours the folded triangles red.
- `cov $L <out.png> <heads|all>` draws the overview, and takes `CC_HANDLES`.
- `cexport $L <fed.json>` writes the fed curves for the fit, then `uv run --with numpy --with scipy python3 -I $S/py/fit_handles.py fed.json handles.json ranges.json`.
- `cends $L <prefix> handles.json ranges.json <heads>` writes the ends sheets and their counts.
- `cfold $L <heads>` prints the layers' angles and turned-over shares (the rim finding). `ctopo $L <head>` prints the loop's topology.
- The sizes can be set from the environment: `CC_GAP`, `CC_REACH`, `CC_ROLL` (the rim's layer), `CC_SINK`, `CC_BLEND`, `CC_UNDER`, `CC_SPINE`, `CC_FADE`, `CC_EPS`, `CC_POCKET`, `CC_RELAX`, `CC_HFADE`, `CC_SLOPE2`, `CC_LIFT`, `CC_MODE` (`polar`, `flat` or `disp`).
