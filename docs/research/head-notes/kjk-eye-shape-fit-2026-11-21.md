# The eye's shape measured directly, and what the existing sliders can reach (2026-11-21)

Written by the conversation started from the brief `eye-shape-fit` for the lead kjk-48 (Eyes HQ). The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/808ab3f7-c432-4a58-ba68-a470f082f09d/scratchpad/fit/`, called `$F` below.

Nothing live changed. No file in the worktree, the live data, the shared kit or viewer 2 was touched, and nothing was committed. All the code is in a copy of the shape-only diagnosis tool, `$F/diag`. It still links the worktree's crates, read only. No judges were used.

## Verdict

- The new measure works. It reads the eye's shape as two things: the curve where the eyeball comes through the skin, and the skin in a 15 mm ring round that curve. Both are read in a frame centred on the eyeball's centre. It tells KOTOR's heads apart about ten times better than it confuses a head's left eye with its right. Its extremes look right in the pictures.
- With only this target, the existing solver brings the curve close to KOTOR's on every one of the 8 heads tried. The mean gap falls from 1.5 to 6.8 mm (live) to 0.4 to 0.9 mm on seven heads, and to 1.7 mm on pfha05. Seen from the front, the fits match KOTOR's outline on all 8 heads.
- One part of the curve is out of reach on every head: **the far outer corner seen from the temple side and in profile.** KOTOR's outer tip lies 0.5 to 4 mm in front of the eyeball's centre (6.4 mm on pfha05). The fits' outer tip stops 1.5 to 5 mm further forward than KOTOR's. Kreia is short by 5.2 mm, Bastila by 3.1 mm and pmhc04 by 1.6 mm.
- A smaller miss is at the **inner corner** of pmhc04 and pmha01. Our inner corner sits 2 to 3 mm lower and 2 to 3 mm further back than KOTOR's. That is the "buried inner corner" seen in earlier rounds.
- The ring gets closer too (mean 1.1 to 2.1 mm down to 0.9 to 1.3 mm, pfha05 6.7 down to 1.8 mm). What is left is the skin beside the nose on pmhc04 and pmha01 (ours lies 0.9 and 1.7 mm deeper than KOTOR's), and the upper lid on Bastila and pfha05 (1.1 and 2.0 mm deeper).
- The fold guard keeps the full move only on pmha01, Kreia and Carth. It keeps a quarter of the move on pmhc04 and half on Jolee. On Bastila, Zherron and pfha05 it keeps none, because the fits fold 2 or 3 more triangles than live. On Zherron those folds cannot be seen in the pictures, and the fit's closed eye shuts cleanly. Nothing here is ready to publish. These are fits that prove what the sliders can reach.

## The measure

All of it is in `$F/diag/src/shape.rs` and in the functions `kotor_reads`, `our_reads`, `compare` and `ring_values` near the end of `$F/diag/src/main.rs`.

**The frame.** Each eye is read in its own frame. The eyeball's centre is at the origin. u points toward the temple (the left eye is mirrored), v points up, and w points forward. The units are the head frame's millimetres, with no scaling. KOTOR's head and ours are lined up on the eyeball's centre, as the brief asks. So the measure ignores where the eye sits in the head. Eye depth, spacing and height are measured elsewhere.

**The curve.** For 72 directions round the forward axis, a march goes out from straight ahead across the eyeball's surface. It stops at the first point of the eyeball that is no longer bare. A point counts as bare when both of these hold:
- no skin lies within 3 mm outside it along the eyeball's radius;
- at least one of 15 views can see it past the skin. The views are 30 degrees toward the nose, straight on, and 30, 60 and 90 degrees toward the temple, each level, from 30 degrees above and from 30 degrees below. Each view is an orthographic depth buffer of the skin at 0.15 mm a pixel.

The same rule reads KOTOR's head (its eyeball mesh against its skin) and ours (our sphere against our skin). On KOTOR this is the curve where the eyeball meets the skin. On ours it is where our skin meets the eyeball. The lid pieces, the lash strip and the caruncle are left out on both heads.

Three things I tried first did not work:
- Rays along the radius alone, with no views, stop short at our inner corner. There the radius points at the skin beside the nose, while the eyeball is bare when seen from the front.
- The edge loop of `lid_edges.json` lies well outside our bare eyeball. With the lid pieces off, our skin rolls inward past those vertices before it meets the eyeball.
- The views alone see under the lids from below and through gaps in the skin.

**The ring.** These are points on the skin about 0.6 mm apart (0.36 mm² each), within 15 mm of the curve. A point is kept only if it is outside the eyeball and on skin that faces the outside. Skin faces the outside when a ray along its normal meets no other skin and no eyeball within 25 mm. The gap is measured both ways. Each of KOTOR's points is measured to our nearest outer skin, and each of ours to KOTOR's. Each way counts for half. The gap is signed: + means our skin stands further out than KOTOR's.

**The scores.**
- Curve: the mean of both ways' nearest distances between the two curves, the worst distance, and the outline overlap (intersection over union) seen from the front, from 50 degrees toward the temple and in profile.
- Curve regions, by the direction round the forward axis: outer corner (within 30 degrees of the temple direction), upper arch (30 to 150), inner corner (150 to 210), lower arch (210 to 330).
- Ring: the mean absolute gap and its 90th percentile.
- Ring regions, in quarters: upper lid, beside the nose, under the eye, toward the temple. Each region has a signed mean and an absolute mean.

The profile overlap is a noisy number. The curve is a thin, twisted shape when seen from the side. Use it to compare heads with each other, not as an absolute score.

**KOTOR mesh fixes.**
- All of KOTOR's face triangles are used. The `kotor_skin` mask drops pmhc04's lower lid (`SKIN_ONLY=1` turns it back on).
- An eyeball triangle with a corner more than 1.4 times the 25th-percentile radius from the centre is dropped. That removes pfhc02's stray grey sliver, which is part of its eyeball mesh, not its skin.
- A curve point whose angle from straight ahead jumps more than 15 degrees past the median of its four neighbours is replaced by the line between its good neighbours. Slits in the skin cause those jumps.

## How the measure was proven

The proof is in `$F/m_all.json` and `$F/proof.py`. The overview of all 57 heads is `$F/ov.png`. Each head gets three pictures: KOTOR from the front and temple 3q with its curve in yellow, then live from the front with KOTOR's curve moved to our eyeball's centre in yellow and our curve in cyan.

- **Left against right.** On the 54 heads where the measure reads well, KOTOR's left eye against its own right eye differs by a median of 0.23 mm on the curve and 0.49 mm on the ring. Two different heads differ by a median of 3.22 mm on the curve and 2.78 mm on the ring. That makes the curve 14 times and the ring 6 times more sensitive to the head than to the side.
- **Telling heads apart.** For 48 of 54 heads, the right eye's nearest match among all heads' left eyes is the head's own left eye, on the curve and on the ring alike. pfhc02 fails, because its two KOTOR eyeballs really differ: one has radii from 11 to 33 mm, the other from 16.7 to 18.5 mm. pmba01 and pmhc03 are close to other heads.
- **The pictures.** In `$F/ov.png` the yellow curve traces KOTOR's bare eyeball on every head, from the front and from the temple side, except three mods. mod_hp_pfha01 has a slit above the eye that sends the curve up into the brow. On mod_tcs_bastila the upper curve wanders. On mod_narve_canderous the eye reads as closed. These three are left out of the proof. n_jedimalek's upper arch sits on the edge of a dark crease and is borderline. pfhc02's sliver is handled, but its curve is a little large at the outer corner.
- **The extremes.** `$F/proof/extremes.png` shows the best and worst live heads. The best are n_jedimalek (curve 0.56 mm) and pmha01 (0.98 mm), and their curves lie on top of KOTOR's. The worst are pfha05 (6.84 mm) and comm_a_f (5.99 mm). On those two, KOTOR's curve is much larger and reaches far back, and the ring is deep blue everywhere. KOTOR's skin there stands 6 to 8 mm further out from the eyeball's centre than ours. The female heads are the worst matches in live: pfha05, pfha02, pfha06, comm_a_f, comm_a_f2 and old_a_f all lie between 5 and 7 mm.

## Scores, KOTOR against live

These come from the live data (`~/.claude/tools/likeness-fit-data/sliders`), averaged over both eyes. The table holds the curve mean, the worst gap, the overlap (front / temple / profile), the curve regions (inner / outer / upper / lower), the ring mean, and the ring's signed regions (upper lid / beside the nose / under the eye / toward the temple). All 57 heads are in `$F/m_all.json`. The median over 54 heads is 2.16 mm for the curve and 2.02 mm for the ring.

| Head | Curve | Worst | Overlap | Curve regions | Ring | Ring signed |
|---|---|---|---|---|---|---|
| pmhc04 | 1.75 | 5.2 | 0.54 / 0.53 / 0.18 | 2.0 / 1.9 / 2.1 / 1.2 | 1.69 | +0.1 / -2.1 / -0.6 / -0.6 |
| p_bastilah | 1.97 | 7.6 | 0.82 / 0.61 / 0.14 | 1.3 / 3.9 / 1.3 / 1.9 | 1.23 | -1.4 / -0.5 / +0.4 / +0.7 |
| pmha01 | 0.98 | 5.8 | 0.73 / 0.68 / 0.13 | 1.3 / 1.7 / 0.6 / 0.8 | 1.20 | -0.4 / -1.9 / -0.1 / -0.6 |
| p_kreiah | 1.65 | 9.0 | 0.60 / 0.43 / 0.10 | 1.7 / 3.5 / 1.1 / 1.1 | 1.45 | +0.4 / -0.6 / -1.7 / -0.0 |
| p_carthh | 1.51 | 4.0 | 0.80 / 0.66 / 0.07 | 0.8 / 2.3 / 1.4 / 1.6 | 1.14 | -0.2 / -0.5 / +0.9 / -0.0 |
| n_zherronh | 2.67 | 5.2 | 0.67 / 0.54 / 0.07 | 2.7 / 3.1 / 2.5 / 2.6 | 2.09 | +1.9 / +1.5 / +1.7 / +1.9 |
| pfha05 | 6.84 | 9.4 | 0.57 / 0.40 / 0.01 | 8.0 / 6.5 / 6.9 / 6.4 | 6.96 | -7.7 / -7.1 / -6.6 / -6.8 |
| p_joleeh | 4.31 | 4.9 | 0.78 / 0.60 / 0.03 | 4.2 / 4.5 / 4.2 / 4.4 | 1.77 | +0.1 / +0.5 / +1.1 / +0.0 |

## The power test

**The data.** It runs on a copy of round 7's `b7` (`$F/b7`, made with round 7's `mkdata.sh`). That copy has all 31 eye sliders, including Outer corner reach and the sliding Eye width. Its start values are the live page's. The start differs a little from live, because Eye width slides.

**The solve.** The command is `eyefit`. It uses the eye solver's own method: Levenberg-Marquardt with finite differences of 0.05 spread, the same damping, up to 30 rounds, and the sliders' own ranges. Each slider's spread is a quarter of its range, and there is a weak pull toward 0 (prior 0.01). It starts from the head's shown values. It moves the 29 eye sliders except Caruncle width and Upper lid cover, because those two only shape the caruncle and the lid piece. The target is only the curve and the ring:
- per eye, the 72 curve points' differences in mm, so that the curve's squared terms add up to its mean squared gap;
- KOTOR's ring points' signed distance to our outer skin, weighted by area.

The curve and the ring have equal weight. No other eye measure is in the solve.

**The fold guard.** It is in two parts.
- Inside the solve, there is a penalty. It counts triangles in the eye's window that turn more than 72 degrees from the smoothed start, and neighbours that met at under 45 degrees and now meet at more than 72 degrees. It only counts triangles more than 0.5 mm outside the eyeball. Its weight is 10 (`FIT_FOLD`). Without it, pmhc04's fit folded 44 triangles.
- After the solve, a pull-back. It tries 1, 0.75, 0.5 and 0.25 of the move and keeps the largest share that passes. A share passes when it has no more folded triangles more than 0.5 mm outside the eyeball than the start has (the `faultsat` rule), and no more than 0.5 mm² more bare eyeball with the lids closed, from the front or from 50 degrees toward the temple.

Results (`$F/fit1.json`, printed by `python3 $F/summ.py $F/fit1.json`). Each cell gives live (b7 start), then the fit, then the share the guard kept:

| Head | Curve mm | Overlap front | Overlap temple | Ring mm | Guard share | Extra folds in the fit |
|---|---|---|---|---|---|---|
| pmhc04 | 1.75 → 0.67 | 0.54 → 0.83 | 0.53 → 0.87 | 1.54 → 1.25 | 0.25 | 1 |
| p_bastilah | 1.97 → 0.85 | 0.82 → 0.89 | 0.61 → 0.84 | 1.41 → 1.08 | 0 | 2 |
| pmha01 | 0.98 → 0.74 | 0.73 → 0.81 | 0.68 → 0.80 | 1.07 → 1.07 | 1 | 0 |
| p_kreiah | 1.65 → 0.88 | 0.60 → 0.83 | 0.43 → 0.72 | 1.44 → 0.93 | 1 | 0 |
| p_carthh | 1.51 → 0.55 | 0.80 → 0.85 | 0.66 → 0.83 | 1.15 → 0.86 | 1 | 0 |
| n_zherronh | 2.67 → 0.43 | 0.67 → 0.85 | 0.54 → 0.83 | 2.09 → 0.95 | 0 | 3 |
| pfha05 | 6.84 → 1.73 | 0.57 → 0.73 | 0.40 → 0.69 | 6.71 → 1.83 | 0 | 2 |
| p_joleeh | 4.31 → 0.52 | 0.78 → 0.90 | 0.60 → 0.85 | 1.72 → 1.02 | 0.5 (curve 2.31) | (start has 2) |

Several fits rest on a slider's limit. Kreia's fit sits at the low end of Eye depth (-5.34) and Eye spacing (-6.40), and at the high end of Outer corner angle (35). pmha01's Outer corner angle goes to 14. Most fits move Outer corner depth to -2 to -3, toward its low end of -4. Several fits move Eye height and Eye spacing a lot. That is because the measure lines up on the eyeball's centre: the solver moves the eyeball inside the skin to shape the curve.

### The pictures

Each head has one sheet in `$F/sheets/fit_<head>.png` (with `_s.png` copies shrunk to 1600 px). The rows are:
1. KOTOR, shape only.
2. Live (b7 start).
3. Live's ring map and live closed.
4. The fit.
5. The fit's ring map and the fit closed.
6. On pmhc04 and Jolee only, the guarded share.

The columns are front, temple 3q, nose 3q and profile. KOTOR's curve is in yellow, moved to our eyeball's centre, and ours is in cyan. On the ring maps, red means our skin stands out, blue means it lies deeper, and full colour is 3 mm. The hair is off and the eyes are open, apart from the closed tiles.

### Can the existing sliders reach KOTOR's shape?

The outer tip is given as w, its distance in front of the eyeball's centre. "Short" means the fit's tip is further forward than KOTOR's.

| Head | Inner corner | Outer corner | Upper arch | Lower arch | Beside the nose | Under the eye | Upper lid | Toward the temple |
|---|---|---|---|---|---|---|---|---|
| pmhc04 | No. The fit's tip is 3.5 mm lower and 2 mm further back (gap 1.3 mm) | Yes from the front. Short by 1.6 mm from the temple side (w 3.1 against 1.5) | Yes (0.65) | Yes (0.33) | No. 0.9 mm too deep | Yes | Yes (0.6 deep) | Yes |
| Bastila | Yes from the front. 2 mm too far back | No. Short by 3.1 mm (w 4.9 against 1.8). The long wedge seen from the temple stays out of reach | Yes (0.47) | Yes (0.59) | Yes | Yes | No. 1.1 mm too deep | Yes (0.7 deep) |
| pmha01 | No. 1.9 mm lower and 3.4 mm further back | Short by 2.5 mm (w 5.1 against 2.6) | Yes | Yes | No. 1.7 mm too deep | Yes | Yes | Yes |
| Kreia | Yes | No. Short by 5.2 mm (w 5.7 against 0.5), and the tip is 2.4 mm less wide. This is the worst on any head | Yes | Yes | Yes | Yes (0.4 deep) | Yes | No. 0.9 mm too deep |
| Carth | Yes | Short by 1.7 mm (w 5.5 against 3.8) | Yes | Yes | Yes | Yes | Yes | Yes |
| Zherron | Yes | Short by 2.4 mm (w 4.3 against 1.9) | Yes | Yes | Yes | Yes | Yes (0.7 high) | Yes |
| pfha05 | Partly. A deep wedge at the inner corner (gap 2.8 mm) | No. Short by 3.6 mm (w 10.0 against 6.4), and the tip is 3.5 mm less wide | Partly (1.6) | Yes | Yes | No. 1.1 mm too deep | No. 2.0 mm too deep | No. 1.8 mm too deep |
| Jolee | Yes | Short by 2.2 mm (w 5.3 against 3.1) | Yes | Yes | Yes | Yes (0.65 deep) | Yes | Yes (0.66 deep) |

The closed eyes:
- The fits of pmhc04, Kreia, Carth and Zherron close cleanly.
- Bastila's fit closes with the same small pink caruncle tip as live.
- pfha05's fit shows small slivers of eyeball at the outer corner when closed, which live also does.
- Jolee's full fit leaves 0.96 mm² of eyeball bare from the front. Her guarded half closes cleanly.

## What control is missing

1. **An outer corner that reaches toward the eyeball's equator** while the closed lid still covers it. Every fit stops 1.5 to 5 mm in front of KOTOR's outer tip. The sliders that should do it are capped:
   - Outer corner depth stops at -4 mm, and the fits press toward it.
   - Outer corner reach stops sinking 4 mm in front of the eyeball's centre, so that the closed lid still covers the corner.
   - Outer corner reach also does almost nothing below 1. At 0.5 it changes the curve by less than 0.01 mm. At 1 it improves Kreia's outer corner from 1.57 to 1.02 mm and Bastila's from 1.40 to 1.27 mm. Because its finite-difference step sees no change, the solver leaves it at 0 on every head.

   This confirms round 7's finding and the equator note's step 0, now with a proven measure. The control needs a closing lid that can cover an opening behind the eyeball's centre.
2. **The inner corner's height and depth** on pmhc04 and pmha01. The fit's inner tip stays 2 to 3.5 mm lower and 2 to 3.4 mm further back than KOTOR's, even with Inner corner depth, height and position free. The fold penalty may be what holds them back there. That was not tested, and it is the first thing to try.
3. **The skin beside the nose**, on pmhc04 and pmha01. It stays 0.9 to 1.7 mm deeper than KOTOR's. No eye slider lifts that skin on its own.
4. **The female heads.** On pfha05, KOTOR's skin stands about 7 mm further out from the eyeball's centre all round, and its outer tip is 18 mm wide. The fit gets within 1.8 mm on the ring, but it uses up the ranges. This suggests the female heads' eyeballs, or their sockets, are placed differently from ours. It needs a step 0 on the 57 meshes before anyone builds anything.

## Commands

The binaries are `$F/diag/target/release/pmhc_diag`, which the solve used, and `$F/diag/target2/release/pmhc_diag`, which has the overlay fix and `SHAPE_VALUES`. Build with `PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH" CARGO_TARGET_DIR=target2 cargo build --release --offline` in `$F/diag`.

- `pmhc_diag eyeshape <data> <out.json> <heads|all>` gives the scores, KOTOR against shown, plus KOTOR's left against right, the curves, and with `all` the 57×57 cross table. `SHAPE_VALUES=<fitvals.json>:<label>` and `SHAPE_SET="<slider>=<v>;..."` override the values. `COVER_K`, `COVER_O` (default 3 mm) and `RADIAL=1` (rays only) are for trials.
- `pmhc_diag eyeoverview <data> <out.png> <heads|all>` draws the overview.
- `FIT_VALUES=<json> pmhc_diag eyesheet <data> <prefix> <heads>` draws the per-head sheets. The json is `{head: {label: values over the merged set}}`. `$F/fitvals1.json` is the one for this round.
- `pmhc_diag eyefit <data> <out.json> <heads>` runs the solve and the guard. `FIT_W="curve,ring"`, `FIT_ROUNDS`, `FIT_FOLD` and `FIT_SKIP` tune it. It takes 2 to 3 minutes per head, and the 8 heads run in parallel.
- `python3 $F/proof.py $F/m_all.json` prints the proof and the live ranking. `python3 $F/summ.py $F/fit1.json` prints the fits. `python3 $F/tips.py <eyeshape json>` prints the corner tips.

## Traps

- The KOTOR row must use all of KOTOR's face triangles. The `kotor_skin` mask removes pmhc04's lower lid.
- pfhc02's grey sliver is an eyeball triangle, not a thin skin triangle. A skin sliver filter (`sliver()`) finds nothing there. It drops only 2 triangles, both on n_daraalah.
- Our `lid_edges.json` edge loop is not the edge of our bare eyeball once the lid pieces are off.
- The curve is lined up on the eyeball's centre. Overlays must draw KOTOR's curve moved to our centre, or the picture shows a placement gap that the measure does not score. The first sheets had this wrong.
- Outer corner reach acts in a threshold-like way. A finite-difference solver will never move it from 0. Test it by hand at 1.
- The guard's fold rule counts against the smoothed start, not the built live head. Live itself already has 2 folds on pmha01 and Jolee.
- The power test's "live" row is b7, which has the sliding Eye width, so it differs a little from viewer 2's live.
