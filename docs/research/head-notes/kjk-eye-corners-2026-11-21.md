# Eye corners: measured, given handles, and solved on all 57 heads (2026-11-21)

Written by the conversation started from the brief `eye-corners` for the lead kjk-48 (Eyes HQ). The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/57efa471-d38e-4773-9d9a-64f6ff7ae461/scratchpad`, called `$C` below. The shape fit note's folder is called `$F`.

Nothing is published. The live data, viewer 2 and the live wasm are unchanged. Nothing is committed. The code changes are in the worktree `head-paint`, and the originals are in `$C/orig/`.

## Verdict

- **The corner measures work.** A corner's angle tells KOTOR's heads apart 5 to 11 times better than it confuses a head's left corner with its right one. The pictures of the extremes show the measure tracing the bare eyeball's edge. They also show what is wrong with ours: KOTOR's corners are nearly all straight points, while our inner corners end square and our outer corners end round.
- **The outer corner can now reach KOTOR's place.** Two things do it. First, a new step carves the skin round the outer corner to the lid edges' own outline. Our lid edges already reached about where KOTOR's tip is, but the skin's roll covered the last 3 to 6 mm. Second, three new sliders per corner move the tip round the eyeball, up and down, and change how the end closes.
- **The blink no longer leaves the corners bare.** The lid now turns about a line through the eyeball's centre, parallel to the line through the two tips. A line through the tips themselves made the closed lid huge and floating, so I did not keep it.
- **Solved on all 57 heads** with the curve, the ring and the corners as targets and the corrected fold guard. The guard kept the whole move on all 54 heads that the measure reads. Over those 54 heads, the median curve gap fell from 2.16 to 0.78 mm, the ring from 2.02 to 1.35 mm, and the outer tip's gap from 6.1 to 0.9 mm. The fits have fewer folded triangles than live (29 on 3 heads, against 84 on 10), and with the lids closed almost no eyeball shows.
- **What is not good yet.**
  - The solve moves the whole eye to shape the outline: Eye spacing by a median of 2.9 mm and Eye depth by 1.9 mm. Bastila's sheet shows her eye shifted toward the temple.
  - The closed lid has a lump at its outer end on several heads.
  - The inner corner carve tears the skin, so it is off. The inner tip still sits 5.4 mm (median) from KOTOR's, and the inner corners stay blunt. Inner tip shape does nothing without the carve.
  - Nothing is ready to publish until David has looked at the pictures.
- **The folds that blocked Bastila, Zherron and pfha05 were not real.** All seven lay on triangles that are never drawn. The guard now skips those.

## 1. The corner measures

The code is in the diag copy `$C/diag`: `Eye::corner` and `CornerShape` at the end of `src/shape.rs`, `corners_json`, `corner_res` and the `corners` part of `compare` in `src/main.rs`.

**How a corner is read.** The tip is the curve point furthest from straight ahead, within 45 degrees of the temple direction for the outer corner, or of the nose direction for the inner corner. Then the curve is marched again round the tip, every 0.5 degrees over 35 degrees on each side. Near the outer tip that is about one point every 0.1 mm. The opening's upper and lower edges are read 0.25, 0.5, 1, 2, 3 and 4 mm behind the tip. They are read five ways:
- on the eyeball's surface, unrolled about the tip, with "behind" pointing back toward straight ahead;
- in the four views, front, 50 degrees toward the temple, 50 degrees toward the nose, and profile. In a view, the tip is the point of the curve furthest from the opening's middle, and "behind" points back to that middle.

The inner corner cannot be seen from the temple side or in profile, and the outer corner cannot be seen from the nose side. Those views give no value.

**The two numbers per corner and view.**
- The angle at which the upper and lower edges meet, taken from the edges 2 mm behind the tip, in degrees.
- The bluntness: the opening's height 0.5 mm behind the tip divided by its height 2 mm behind it. A straight point gives 0.25. A round end gives about 0.5. A square end gives 1.

The solve uses the tip's place and the edges themselves at 0.5, 1, 2 and 3 mm on the surface, in mm, because those stay smooth when the tip changes. The angle and the bluntness are reported.

**The proof** (`$C/m/all.json`, `python3 $C/cproof.py $C/m/all.json`, 54 heads, the three misread mods left out as in the shape note):

| Corner, view | KOTOR's median | KOTOR's range | Left against right | Head against head | Ratio | Live median |
|---|---|---|---|---|---|---|
| Inner, surface, angle | 55° | 24 to 92° | 1.5° | 16.8° | 11 | 54° |
| Inner, front, angle | 69° | 39 to 111° | 1.9° | 11.2° | 6 | 63° |
| Inner, nose side, angle | 52° | 21 to 87° | 1.5° | 16.0° | 11 | 55° |
| Inner, surface, bluntness | 0.26 | 0.21 to 0.59 | 0.02 | 0.03 | 1.5 | 0.90 |
| Outer, surface, angle | 44° | 24 to 81° | 1.4° | 14.1° | 10 | 71° |
| Outer, front, angle | 85° | 42 to 120° | 3.4° | 17.4° | 5 | 86° |
| Outer, temple side, angle | 46° | 23 to 81° | 1.7° | 15.0° | 9 | 71° |
| Outer, profile, angle | 45° | 23 to 89° | 1.5° | 14.6° | 10 | 76° |
| Outer, surface, bluntness | 0.28 | 0.18 to 0.54 | 0.02 | 0.03 | 1.5 | 0.54 |

What this shows:
- The angle tells KOTOR's heads apart 5 to 11 times better than it confuses a head's left corner with its right one.
- KOTOR's corners are nearly all straight points. The median bluntness is 0.26 at the inner corner and 0.28 at the outer one, and two heads differ by only 0.03. So bluntness does not tell heads apart. It tells KOTOR from us: our inner corners end square (0.90) and our outer corners end round (0.54).
- A few KOTOR corners are blunt: Bastila's and mod_meetra's inner corners (0.47), mod_jaesa's outer corner (0.54). Bastila's matches what David saw from the nose side.
- The tips also tell heads apart. KOTOR's inner tip sits 9 mm in front of the eyeball's centre (median), and live's 1.4 mm. KOTOR's outer tip sits 1.3 mm in front, and live's 6.6 mm.

**The pictures.** `$C/pics/corner_extremes.png` shows KOTOR's corners on n_hussef, old_a_f, mod_exile_canon, n_mastkavarh (the most pointed), p_bastilah, mod_meetra, mod_jaesa, mod_keanu, n_admoffh (the bluntest and widest) and pmhc03. Each row has the inner corner from the front and the nose side, then the outer corner from the front, the temple side and in profile. The dense corner curve is drawn in yellow and follows the bare eyeball's edge on every head but one. pmhc03's inner corner misreads (angle 9°, bluntness 0.93): the curve catches a thin slit there. Leave pmhc03's inner corner out.

`$C/pics/s_bast.png` shows Bastila against live in close-up. Live's outer end is a U, and KOTOR's is a straight wedge.

## 2. The handles and the carve

### What the code does

The code is in `head_sliders/src/eye_curves.rs`, in the section "The corner handles", with the functions `corner_carve`, `handle_values`, `carve_sizes` and `carve_smooth`. `SliderSet::build` (`set.rs`) and `Evaluator::positions` (`solve.rs`) call `corner_carve` right after the press. They do so only when the slider set holds at least one handle slider. Sets without one build exactly as before.

The handles are drawn sliders whose value is their amount. They are read together, so one step gets all six values.

1. **The tips move.** A tip's place moves it round the eyeball, away from the eye's middle. A tip's height moves it up. Both are in mm along the eyeball's surface. The lid edges from the eye's middle to the tip stretch evenly to follow it. The skin round them follows, fading out between 2 and 8 mm from the lids, and between 3 and 8 mm outward from the eyeball. The outer tip stops 100 degrees round from straight ahead, which is 10 degrees past the eyeball's widest point seen from the front.
2. **The end is shaped.** A tip's shape changes the last 3 mm of the opening. Below 0, the opening's height near the tip shrinks, so the end becomes a longer, sharper point. Above 0, the height grows, up to the height 3 mm behind the tip, so the end becomes blunter. At 1 it is almost square.
3. **The outline is smoothed.** It is averaged along the eye over 3 mm, less so within 2 mm of each tip. Without this, a dip in the lid edges' own line, which the skin's roll used to hide, shows as a V notch in the upper lid (mod_keanu).
4. **The skin is carved round the outer corner.** The carve starts at 10% of the way from the eye's middle to the outer tip and grows over the next 60%. It works along the eyeball's radius:
   - Skin inside the outline goes under the eyeball's surface, at most 0.8 mm under. The eyeball then shows there from every side.
   - Skin at the outline lies 0.15 mm outside the eyeball. Skin further out rises 1 mm for every mm away from the outline. This holds for 1.5 mm and fades out over 3 mm more. Such skin is pulled down when it stands higher, and lifted when it lies lower.
   - The carve stops 1 mm past the tip, fading over 4 mm more.
   - The inner corner is not carved (mode 0 in `EYES_CARVE`). Both of the ways tried at the inner corner tore the skin. Sinking only gave 495 folded triangles over 57 heads. The full carve made the jagged tear along Zherron's upper inner lid seen in `$C/pics/s10.png`.

With every handle at 0, the carve still opens the outer corner along the lid edges. That alone, over 54 heads, brought the outer tip from 6.1 to 3.8 mm off KOTOR's (median), and the inner tip from 7.8 to 3.4 mm. The second number was measured while the inner carve was still on.

`EYES_CARVE` (15 numbers) and `EYES_CARVE_SMOOTH` set the sizes for trials. The defaults are `0.75,0.15,0.1,0.6,1.5,3,0.8,0.15,1,1,4,3,100,0,2` and 3.

### The blink

The code is in `head_eyes/src/piece.rs`. `Frame` can now turn, and `Settings` has `tip_axis` and `tip_axis_centred`. There is a new `turn_about`, and `EyePiece.axis` holds the real axis. `eyes.rs` closes the pieces with `turn_about`, which gives the same result as before for the old axis.

- `EYES_TIP_AXIS=1` turns the lid about the line through the two tips. The tips stay put, but the line runs 3 to 6 mm in front of the eyeball's centre. A rigid lid turning about it must stand far off the eyeball to clear it, so the closed lid becomes a huge, floating shell (`$C/pics/s5.png`). I did not keep this one.
- `EYES_TIP_AXIS=2` turns the lid about a line through the eyeball's centre, parallel to the line through the tips. The lid then hugs the eyeball. The outer tip lies close to that line, so the lid sweeps over the opening near it. This is the one used for every result in this note.
- The default is still off (0), so nothing changes for any other session. To publish, the default must become 2 on the wasm build too, and viewer 2 must turn the piece about `info.axis` instead of x (see "How to publish").

### The six sliders

They are in the group "corners". `$C/py/addh.py <data>` adds them to a data folder's `web/eyes/sliders.json`, with these hints.

| Name | Range | Hint |
|---|---|---|
| Inner tip place | -4 to 6 mm | Moves the inner corner's tip round the eyeball. At a positive value the tip goes further toward the nose and back round the eyeball. At a negative value it comes forward. The lids stretch evenly from the middle of the eye to follow it, and the skin round the corner is shaped to the new opening. The value is in mm along the eyeball's surface. |
| Inner tip height | -3 to 3 mm | Moves the inner corner's tip up or down on the eyeball. The lids stretch evenly from the middle of the eye to follow it, and the skin round the corner follows. The value is in mm along the eyeball's surface. |
| Inner tip shape | -1 to 1 | Sets how the inner corner closes. (It does nothing while the inner carve is off. Leave it out of the page until the inner carve works.) |
| Outer tip place | -4 to 8 mm | Moves the outer corner's tip round the side of the eyeball. At a positive value the tip goes further toward the ear, up to the eyeball's widest point seen from the front and a little past it. At a negative value it comes forward. The lids stretch evenly from the middle of the eye to follow it, and the skin round the corner is shaped to the new opening. When the eye closes, the lid still covers the whole opening. The value is in mm along the eyeball's surface. |
| Outer tip height | -3 to 3 mm | Moves the outer corner's tip up or down on the eyeball. The lids stretch evenly from the middle of the eye to follow it, and the skin round the corner follows. The value is in mm along the eyeball's surface. |
| Outer tip shape | -1 to 1 | Sets how the outer corner closes. At a negative value the lids meet in a sharper, longer point. At a positive value the end of the opening grows blunter, and at 1 it is almost square. Only the last 3 mm of the opening change. |

"Depth" is not a separate slider. A tip that stays on the eyeball's surface has only two free numbers. Moving it round the eyeball is what moves it back or forward, so "place" is the depth.

### What each handle looks like at its ends

The sheets are `$C/pics/ends/out_p_kreiah.png`, `out_p_bastilah.png`, `in_p_kreiah.png` and `in_p_bastilah.png`. Each row has the inner corner from the front and the nose side, and the outer corner from the front, the temple side and in profile. Each value has an open row and a closed row, with KOTOR's corner in yellow and ours in cyan.
- **Outer tip place.** At -4 the outer end is short and round. At +8, Kreia's opening reaches KOTOR's tip from the temple side and in profile, and the closed lid covers it.
- **Outer tip height.** The tip goes down or up. At -3 on Kreia, the upper lid near the tip gets a step.
- **Outer tip shape.** At -1 the end is a sharp point. At +1 it is square, but its edge is jagged, and the closed eye shows a small ring at the end. Keep the high end to about 0.5.
- **Inner tip place.** At -4, Bastila's inner tip comes forward onto KOTOR's. At +6 it goes deep toward the nose.
- **Inner tip height.** The tip goes down or up, and the closed eye covers it in each case. The nose-side close-ups are hard to read, because the cut that removes the nose also shows the inside of the head.

### Which old sliders the handles replace

I solved the 8 heads twice (`$C/m/fitA8.json`, `$C/m/fitB8.json`, `python3 $C/py/fsumm.py <file>`):
- **A:** the handles with the old corner sliders held at 0. Those are Inner corner position, depth, height and roundness, Outer corner position, depth, height and roundness, Outer lid opening and Outer corner reach.
- **B:** the handles together with the old corner sliders.

B beats A on 7 of 8 heads. For example, Bastila's curve gap is 0.91 mm with B and 1.18 mm with A, and pmhc04's outer tip gap is 0.8 mm with B and 1.8 mm with A. So in practice the old sliders are not redundant. The solver uses them together with the handles. On all 57 heads it pushes Outer corner roundness to its low end (-1.5) on most heads, next to Outer tip shape at -0.56.

Two old sliders are made redundant by the carve:
- **Outer corner reach**: the carve does what it did, and the closed lid now covers the result. It stayed at 0.
- **Outer lid opening**: the solver left it at 0 on almost every head (median change 0.01).

Both can be removed. Inner tip shape stays out until the inner carve works.

## 3. The folds

**Bastila, Zherron and pfha05** (`$C/pics/s7.png` and `$C/pics/s8.png`, command `foldshow`). The fit note's folds were the guard's added folded triangles. I listed them with their place and area. All seven lie on triangles of a hidden mesh group, which is never drawn:
- two on Bastila, below the inner corners;
- three on Zherron, beside the nose;
- two on pfha05, above the outer brows.

Close-ups at 30 pixels per mm, from the front, the temple side, the nose side and below, show nothing on any of the three. **Verdict: not real.** The solve's own fold penalty already skipped hidden triangles, but the guard did not. The guard in `eyefit` and the new `foldcount` now skip them. With that change, the old fits of Bastila, Zherron and pfha05 would have kept their whole move.

**Does the fold penalty hold the inner corners back?** I solved pmhc04 and pmha01 again with the old sliders and without the penalty (`$C/m/fit_nofold.json`).
- pmhc04: no. The inner corner's curve gap is 1.35 mm without the penalty and 1.33 mm with it.
- pmha01: partly. The gap is 0.67 mm without the penalty and 1.12 mm with it.

pmhc04's inner corner was held back by the missing control, not by the penalty. With the handles, pmhc04's inner tip now lands 0.4 mm from KOTOR's.

**The new folds.** I counted the drawn folded triangles over all 57 heads (`foldcount`). Each count is against the smoothed start, for triangles more than 0.5 mm outside the eyeball. The trials ran on the new data, with the tilted blink.

| Trial | Folded triangles | Heads with folds |
|---|---|---|
| Live (no carve) | 84 | 10 |
| Handles and tilted blink, no carve | 84 | 10 |
| Full carve at both corners | 434 | 49 |
| Inner corner sinking only, outer full | 495 | 52 |
| Inner off, outer from 30% of the way | 226 | 44 |
| Inner off, outer from 10% growing over 60% (chosen) | 107 | 8 |
| The 57 new fits | 29 | 3 |

Zherron's inner carve folds can be seen: a jagged tear along the upper inner lid (`$C/pics/s10.png`). That is why the inner carve is off. The 4 folds most heads gained with the outer carve from 30% are the crease of a hooded upper lid on mod_keanu (`$C/pics/s12.png`). They are not a tear, but the V notch next to them is a real fault. The outline smoothing fixed the notch (`$C/pics/s14.png`).

## 4. The solve on all 57 heads

**The data.** `$C/d8` is a copy of the shape note's `b7` with the six handles added. Its live values are the live page's, with every handle at 0. The blink turns about the centred, tilted line (`EYES_TIP_AXIS=2`).

**The solve** (`$C/pmhc_diag_v7 eyefit`, command line in "Commands"). It is the shape note's solve, with three targets of equal weight: the curve, the ring, and the corners (`corner_res`). Both eyes and both corners count. For each, the targets are the tip's place and the opening's edges 0.5, 1, 2 and 3 mm behind the tip on the eyeball's surface, all in mm. The solve moves every eye slider except Caruncle width, Upper lid cover, Outer corner reach and Inner tip shape. The fold penalty is on (weight 10). The guard skips hidden triangles. It took 16 minutes for all heads.

**The result** (`$C/m/fitC_all.json`, `$C/m/table57.json` for every head). The guard kept the whole move on all 54 heads that the measure reads. The columns are:
- the curve's mean gap and the ring's mean gap, in mm;
- the outline overlap from the front and from the temple side;
- each tip's distance from KOTOR's, in mm;
- the mean gap of each corner's edges 0.5 to 3 mm behind the tip, in mm;
- the bluntness and the angle on the eyeball's surface, with KOTOR's in brackets.

Each cell gives KOTOR against live, then KOTOR against the new fit. Both eyes are averaged.

| Head | Curve | Ring | Overlap front / temple | Inner tip | Inner edges | Outer tip | Outer edges | Bluntness inner / outer (KOTOR) | Angle inner / outer (KOTOR) |
|---|---|---|---|---|---|---|---|---|---|
| pmhc04 | 1.75 → 0.54 | 1.54 → 1.02 | 0.54 / 0.53 → 0.91 / 0.85 | 5.1 → 0.4 | 0.45 → 0.22 | 4.0 → 0.6 | 0.58 → 0.51 | 0.56 / 0.66 → 0.40 / 0.35 (0.26 / 0.30) | 62 / 73 → 81 / 75 (78 / 50) |
| p_bastilah | 1.97 → 0.92 | 1.41 → 1.12 | 0.82 / 0.61 → 0.85 / 0.79 | 8.7 → 8.4 | 1.32 → 1.19 | 10.7 → 1.2 | 1.06 → 0.72 | 0.99 / 0.53 → 0.93 / 0.45 (0.45 / 0.28) | 47 / 84 → 49 / 73 (92 / 37) |
| pmha01 | 0.98 → 0.70 | 1.07 → 1.21 | 0.73 / 0.68 → 0.81 / 0.77 | 3.3 → 3.0 | 0.30 → 0.28 | 8.3 → 3.1 | 0.76 → 0.63 | 0.59 / 0.54 → 0.57 / 0.31 (0.26 / 0.28) | 49 / 70 → 48 / 69 (38 / 30) |
| p_kreiah | 1.65 → 0.90 | 1.44 → 1.35 | 0.60 / 0.43 → 0.78 / 0.72 | 8.4 → 8.2 | 1.00 → 0.55 | 11.8 → 2.8 | 0.90 → 0.96 | 1.01 / 0.49 → 0.86 / 0.52 (0.26 / 0.25) | 58 / 77 → 48 / 77 (59 / 31) |
| p_carthh | 1.51 → 0.54 | 1.15 → 0.99 | 0.80 / 0.66 → 0.88 / 0.81 | 5.9 → 5.5 | 1.25 → 0.59 | 7.5 → 1.4 | 0.43 → 0.46 | 0.97 / 0.44 → 0.94 / 0.35 (0.39 / 0.32) | 56 / 54 → 37 / 60 (55 / 30) |
| n_zherronh | 2.67 → 0.40 | 2.09 → 1.03 | 0.67 / 0.54 → 0.85 / 0.87 | 7.5 → 6.1 | 1.30 → 0.92 | 5.3 → 0.4 | 0.49 → 0.54 | 0.93 / 0.61 → 0.94 / 0.35 (0.29 / 0.27) | 48 / 75 → 34 / 84 (51 / 59) |
| pfha05 | 6.84 → 1.77 | 6.71 → 1.92 | 0.57 / 0.40 → 0.77 / 0.71 | 10.6 → 10.2 | 0.67 → 0.69 | 7.3 → 1.6 | 0.46 → 0.49 | 0.72 / 0.53 → 0.89 / 0.28 (0.59 / 0.29) | 37 / 66 → 43 / 79 (24 / 54) |
| p_joleeh | 4.31 → 0.49 | 1.72 → 1.34 | 0.78 / 0.60 → 0.91 / 0.86 | 5.0 → 0.7 | 0.51 → 0.39 | 6.4 → 0.5 | 0.55 → 0.46 | 0.62 / 0.55 → 0.66 / 0.34 (0.26 / 0.28) | 68 / 64 → 54 / 64 (50 / 37) |
| Median of 54 | 2.16 → 0.78 | 2.02 → 1.35 | 0.61 / 0.54 → 0.83 / 0.81 | 7.8 → 5.4 | 0.89 → 0.48 | 6.1 → 0.9 | 0.61 → 0.52 | 0.88 / 0.54 → 0.65 / 0.30 (0.26 / 0.28) | |

What the numbers say:
- The outer corner is now within 0.4 to 1.6 mm of KOTOR's tip on six of the eight heads. pmha01 is 3.1 mm off and Kreia 2.8 mm. Its end is about as pointed as KOTOR's on six heads.
- The inner corner reaches KOTOR's tip only where the tip had to come forward or up: pmhc04 0.4 mm, Jolee 0.7 mm. Where KOTOR's inner tip lies far in front, as on Bastila, Kreia and pfha05 (8 to 10 mm), it does not, because there is no inner carve. Our inner ends stay square (0.86 to 0.94) on Bastila, Carth, Zherron and Kreia.
- Zherron's curve gap falls from 2.67 to 0.40 mm, and pfha05's from 6.84 to 1.77 mm.
- The angles stay off. The solve does not target them directly, and the edges at 2 mm move only a little.
- **The price.** The solve moves the whole eye to shape the outline. The median change is 2.9 mm for Eye spacing and 1.9 mm for Eye depth. Eye placement is measured elsewhere and is not in this solve. Hold Eye spacing, depth and height at live before any of this goes live.


## 5. Pictures for David

Everything is shape only: no lid pieces, no lash strip, no caruncle, a plain eyeball and no hair. Each `_s.png` is a 1600 px copy. The folder is `$C/out/`.

- `shape_<head>.png`, for pmhc04, p_bastilah, pmha01, p_kreiah, p_carthh, n_zherronh, pfha05 and p_joleeh. The rows are:
  1. KOTOR.
  2. Live, with KOTOR's outline in yellow and ours in cyan.
  3. Live's ring map and live closed, with the live blink.
  4. The new fit with both outlines.
  5. The new fit's ring map and the new fit closed, with the tilted blink.

  The columns are front, temple 3q, nose 3q and profile.
- `corners_<head>.png`, for the same heads. The rows are KOTOR, live, live closed, the new fit and the new fit closed. The columns are the inner corner from the front and the nose side, then the outer corner from the front, the temple side and in profile. KOTOR's corner curve is yellow and ours is cyan. Each tile is labelled with the corner's angle and bluntness.
- `overview_57.png` (`_s` is too small to read, so open the full one). Each head has one block: KOTOR from the front and the temple 3q, live from the front and the temple 3q, live closed from the temple 3q, then the same three for the new fit.
- The handles at their ends: `$C/pics/ends/`. The measure's extremes: `$C/pics/corner_extremes.png`. The folds: `$C/pics/s7.png`, `s8.png`, `s10.png`, `s12.png` and `s14.png`.

What I saw in them:
- Bastila's new outline lies on KOTOR's in all four views, including the long outer wedge in profile. But her eye sits further toward the temple than live, which is the Eye spacing change.
- The new fit's closed lid has a lump at its outer end on Bastila and some other heads. That is the lid piece's end, built about the tilted line. It is a real fault to fix before publishing.
- mod_narve_canderous's fit is broken: a strip of bare eyeball. It is one of the three heads the measure misreads, so leave it out.

To look in viewer 2, the data must be published first. These pictures come from the diag's renderer only. Before anything goes live, David should see the heads in viewer 2 (http://127.0.0.1:8765/viewer2/?eyes=shape&hair=none).


## 6. How to publish, and how to undo

**Prepared data.** `$C/publish/web_eyes/` holds `sliders.json`, which is `d8`'s with the six handles, and `page.json`. The page has the handles in its slider list and each head's new values under the count 34. All 57 heads are fitted, and heads whose guard share is below 1 would keep the guarded values (there are none). It was made by `python3 $C/py/mkpage.py $C/d8 $C/m/fitC_all.json $C/publish/web_eyes`.

**Fix first, before anything goes live:**
1. Hold Eye spacing, Eye depth and Eye height at live in the solve (`FIT_SKIP`), and solve again (16 minutes).
2. Fix the closed lid's lump at the outer end.
3. Drop Inner tip shape, Outer corner reach and Outer lid opening from the page.
4. Cap Outer tip shape at 0.5.

**Then the steps to go live:**
1. In `head_eyes/src/piece.rs`, make `tip_axis_default()` return 2 on every build, the wasm one included.
2. In viewer 2's `sliders.js` `pieceGroup`, replace `group.rotation.x = ...` with a turn about the piece's own axis: `group.quaternion.setFromAxisAngle(new THREE.Vector3(...info.axis), closed ? info.closed_angle * Math.PI / 180 : 0)`.
3. Rebuild the head_sliders wasm.
4. Remake `web/eyes/made.bin` and `made.json` with the eye bench, as the earlier rounds did. I did not run that.
5. Copy the two json files into the live `web/eyes/`, and raise viewer 2's version numbers.

**Undo.** Nothing live changed. To undo the code, copy `$C/orig/head_sliders_src/` over `head_sliders/src/` and `$C/orig/head_eyes_src/` over `head_eyes/src/`. Check the shared log first, in case another session has edited those files since 15:30. My changed files are also kept in `$C/changed/`. The 5 library tests pass with the changes.


## 7. What is still out of reach

- **The inner corner's shape and forward place.** Carving the skin at the inner corner tears it. The inner handles can move the tip, but they cannot open or close skin there. KOTOR's inner tip sits 8 to 10 mm in front of ours on Bastila, Kreia and pfha05, and our inner ends stay square. The next try should open the deep pocket's lining in a smoother way, for example by moving the caruncle pocket's vertices along the eyeball rather than along its radius.
- **The angles.** The solve targets the edges, not the angles, and the angles stay 10 to 40 degrees off on several heads. A target on the angle itself, or an angle handle with a wider reach, is needed.
- **The eye's place.** The solve trades it for the outline. It must be held.
- **The closed lid's end.** It has a lump where the piece ends near the tilted line.
- **The female heads.** pfha05 gets to 1.77 mm, but only by moving the eyeball. That needs the shape note's step 0 on the female heads' eyeballs.
- **The base head and glances.** I did not check the base head's closed eye or the glances. The closed eye is checked on all 57 heads in numbers (`foldcount`) and in pictures for the 8 heads and the overview.


## 8. Traps

- The carve runs whenever a set holds any handle slider, also with every handle at 0. Data with the handles is a different head from live even at 0.
- A lid turning about the line through the tips makes a huge, floating closed lid. Use the line through the eyeball's centre (`EYES_TIP_AXIS=2`).
- Sinking skin inside the outline folds more than the full carve does. Raising skin at the inner pocket tears it.
- Our lid edges' own line can have dips that the skin's roll used to hide. Carve to a smoothed outline.
- The guard used to count folds on hidden triangles. `foldshow` prints whether a fold is in a hidden group.
- A head's eye moves in the solve because the measure is lined up on the eyeball's centre. Hold the whole-eye sliders.
- The sheets combine two data folders: the old one for live (old blink, no `EYES_TIP_AXIS`) and the new one for the fit (`EYES_TIP_AXIS=2`). `$C/sheets.sh` does both.
- There is no PIL or ImageMagick here. `$C/py/stack` runs Pillow through `uv`.

## Commands

The binaries are `$C/pmhc_diag_v8` (the latest diag) and `$C/pmhc_diag_v7` (the one the 57-head solve used).
- `eyeshape <data> <out.json> all` now also writes `kotor_corners` and `live_corners`. `python3 $C/cproof.py` gives the proof, and `python3 $C/py/summ.py <json> [heads]` gives the scores.
- `EYES_TIP_AXIS=2 FIT_W=1,1,1 FIT_SKIP="Outer corner reach;Inner tip shape" pmhc_diag_v7 eyefit $C/d8 <out.json> <heads>`. `FIT_ZERO="<slider>;..."` holds sliders at 0. `python3 $C/py/fsumm.py <out.json>` prints the fits.
- `cornersheet <data> <prefix> <heads>`. It takes `FIT_VALUES`, `CORNER_CANDS="label:slider=v;..|.."`, `SHEET_NO_LIVE`, `SHEET_NO_KOTOR`, `CORNER_KOTOR_ONLY` and `CORNER_PX`.
- `foldshow <data> <prefix> <heads>`, with `FIT_VALUES`, `FOLD_AT=x,y,z` and `FOLD_ALL`. `foldcount <data> all`, with `SHAPE_VALUES`.
- `eyeoverview` with `OV_ROWS`, `OV_KOTOR`, `OV_LABEL` and `SHAPE_VALUES`. `$C/sheets.sh <fitvals> <out> <heads>` makes the per-head sheets.

