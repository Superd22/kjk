# The underside of the jaw and the jaw's corner: measure first

This note was written on the evening of 2026-11-21 by the conversation briefed by the head HQ (brief: `/private/tmp/claude-501/-Users-david-perso-kjk/52003e36-b8ca-4208-a733-a6d51b417e4e/scratchpad/jaw/brief-under-jaw-measure.md`). The work serves B1 to B3 for the jaw and chin zones, and it prepares B5. No sliders were built. Nothing live changed. Nothing was committed.

## Verdict

The premise holds, but mostly as a shared offset rather than as variety. Every KOTOR head has a jaw that stands out over the neck. On every KOTOR head, the line under the jaw runs level or rises toward the back, and most heads also have a clear throat corner. Our slider-made heads have none of these. The 5 jaw sliders cannot reach them, even when the solver may move every slider freely. The FLAME fits reach the throat angle and the line under the jaw on most heads. They do not reach the jaw's ledge over the neck.

The measure is only half proven:
- Five rows pass check 1 at the 95th percentile, though only narrowly: the throat angle, the slope of the neck, the neck's width, the width between the jaw's edges, and the outline from the chin into the throat. The throat outline passes clearly.
- The jaw corner's place, the ledge and the two outlines of the jaw's edge fail check 1 at the 95th percentile. Their median wobble is small, so they can show the offset between KOTOR and us. They cannot yet tell KOTOR's heads apart.
- Check 4 found three faults:
  - On comm_b_m and old_b_f, KOTOR's neck stub is too short, so their throat angle reads wrong.
  - Kreia's hood is not removed.
  - The corner's height stops at 22 mm, because that is the top of the reading window.

## What was built

All of it is in the head-paint worktree, crate `head_sliders`. Nothing uses it by default.
- `src/under_jaw.rs` (new). It holds the measure and the tools, run as `sliders underjaw <command>`:
  - `look <data> <prefix> [per]`: KOTOR's heads with the hair removed, from the side, the front, three-quarter from below and from below, with the measure's points. `UJ_HEADS=a,b` draws only the heads named.
  - `probe <data> <head>`: one head's rows.
  - `dump <data> <out.json>`: each head measured six ways. The six are KOTOR's bare mesh, the same mesh subdivided twice, our wrap, our FLAME fit, the jaw preset (from `web/jaw/page.json`) and the base head.
  - `reach <data> <out.json> <head>...`: a coordinate search over the 5 jaw sliders, within their ranges, starting from each head's jaw preset. It runs twice: once on this measure alone, and once while also holding the jaw measure where the preset has it.
  - `reach-sheet <data> <reach.json> <out.png>`: KOTOR, the preset, the solve and the FLAME fit, side by side.
  - `cut <data> <head|base> <y>`: a level cut, for debugging.
- `src/lib.rs`: one new line, `pub mod under_jaw;`.
- `src/bin/sliders.rs`: the `underjaw` key sends its commands to `under_jaw::tools::run`.
- `src/jaw.rs`: `cover` and `underside` are now `pub`. Nothing else changed.
- The statistics scripts are in `/private/tmp/claude-501/-Users-david-perso-kjk/7fae1cb7-a4ee-47b5-9bf7-3acaab4f4a13/scratchpad/scripts/`:
  - `ujstats.py` takes a dump and gives check 1, the spread, the gaps and check 3.
  - `reach.py` takes the output of `reach`.

To undo the work, delete `under_jaw.rs` and its two lines, and set `cover` and `underside` back to private.

## The measure

Everything is read in the head frame (`Frame::head()`), on KOTOR's own mesh with the hair and cloth removed by `jaw::cover`.
- **The chin.** The chin's front is the front-most point of the middle cut, 85 to 125 mm below the eyes. The chin's bottom is the lowest point of the middle cut within 25 mm behind the chin's front.
- **The jaw's edge.** The jaw is cut across every 4 mm, starting 8 mm behind the chin's front. Each cut is read from 24 mm above the chin's bottom to 45 mm below it. For each height, the outermost point of the cut on each side is taken, which is the cut seen from the front. The edge is the first height, going down, where that outline steps in faster than it goes down, measured over 2 mm. That is where the skin faces more down than out. The two sides are averaged.
  - The line under the jaw starts at the first readable cut within 20 mm of the chin.
  - It ends at the jaw's corner. The corner is the last cut before two cuts in a row cannot be read, or before the edge rises more than 25 mm above the chin's bottom.
- **The throat.** The line under the chin is fitted to the underside, 8 to 20 mm behind the chin's front. The line of the neck's front is fitted 20 to 32 mm below the chin's bottom. The throat angle is the angle between the two lines.
- **The rows:**
  - throat angle;
  - where the throat's corner lies (behind the chin's front, and below the chin's bottom);
  - the slope of the neck;
  - where the jaw's corner lies (behind the chin's front, and above the chin's bottom);
  - the height of the middle third of the line under the jaw;
  - the ledge, which is how far the edge's outline steps in from 3 mm above the edge to 6 mm below it, on the back half of the line;
  - the width between the edges on the back third of the line;
  - the neck's width 24 mm below the chin, over its front 35 mm;
  - the jaw's width less the neck's width.
- **The outlines:**
  - the edge from the side, chin to corner, resampled to 12 points;
  - the edge from below;
  - the line from the chin into the throat, which is 19 rays down the middle.

## The four checks (measured, 56 heads, Malak left out)

Wobble is the gap between KOTOR's mesh and the same mesh subdivided twice. Spread is the standard deviation over KOTOR's heads.

| Row | Wobble median / 95th percentile | KOTOR's spread | Base head | KOTOR's mean | Jaw presets' mean | FLAME fits' mean |
|---|---|---|---|---|---|---|
| Throat angle | 4.5 / 14.9° | 20.0° | 137.3° | 122.9° | 136.0° | 122.4° |
| Slope of the neck | 1.6 / 12.4° | 16.9° | −27.2° | −10.1° | −25.4° | −10.9° |
| Jaw's corner behind the chin | 4.0 / 16.0 mm | 13.7 mm | 68 | 70.0 | 69.9 | 71.4 |
| Jaw's corner above the chin's bottom | 1.0 / 14.6 mm | 12.5 mm | −41.8 | 13.8 | −14.7 | 6.0 |
| Middle of the line under the jaw | 1.2 / 4.1 mm | 3.0 mm | 15.0 | 11.2 | 11.6 | 12.0 |
| Ledge over the neck | 1.4 / 5.7 mm | 3.0 mm | 9.1 | 12.9 | 9.4 | 10.7 |
| Width between the jaw's edges | 2.9 / 9.5 mm | 10.8 mm | 56.8 | 75.5 | 61.8 | 71.1 |
| Neck's width | 2.3 / 10.5 mm | 13.2 mm | 101.9 | 72.3 | 77.5 | 81.0 |
| Outline, edge from the side | 2.2 / 5.5 mm | 4.5 mm | | | gap 11.8 | gap 3.1 |
| Outline, edge from below | 1.9 / 5.3 mm | 3.6 mm | | | gap 9.0 | gap 2.9 |
| Outline, chin into the throat | 0.4 / 1.0 mm | 3.1 mm | | | gap 1.6 | gap 0.6 |

Check 3 asks whether each head's nearest match is itself. The proportions are scaled by the spread.
- KOTOR's subdivided copy finds itself for 35 of 56 heads.
- Our wrap finds itself for 20 of 55 heads.
- The FLAME fit finds itself for 16 of 56 heads.
- The jaw presets find themselves for 1 of 56 heads.

Check 2, whether the measure sees what David notices, was not checked with David. It moves with what the HQ named. Our corner's height is negative, which means the line under the jaw drops into the neck. Our throat angle is 13 degrees more open than KOTOR's, and our ledge is 3.5 mm smaller.

Check 4 used the extremes sheet (`extremes_0.png`, listed below):
- Dodonna and pmhc04, the sharpest throats, are right.
- comm_b_m (165°) and old_b_f (179°) are wrong. Their KOTOR neck stubs are too short for the neck's line.
- Kreia's hood is still drawn, and it sets her ledge.
- mod_narve_canderous's modelled beard bends his line under the jaw.
- The corner's height stops at 22 mm on Bao-Dur, pfhc02 and Kyle Katarn. That is the top of the window, not their corner.

## What the existing sliders cannot reach (measured on 10 heads, seen on 4)

A coordinate search over the 5 jaw sliders started from each head's preset. "Free" moves the sliders as far as their ranges allow. "Held" also keeps the jaw measure where the preset has it.
- **The throat corner.** KOTOR's throat angle is 90° on pmhc04, 103° on Bastila, 106° on Zherron, 107° on comm_w_f, 113° on pfha02 and 117° on sith_app. The held solves stay at 116 to 133°. "Under the chin" sits at its top end, 20.1, on pmhc03, pmhc04 and sith_app. The FLAME fits reach 92 to 120°.
- **The jaw standing out over the neck.** KOTOR's ledge is 18.7 mm on pmhc03, 17.7 mm on pfha02, 14.3 mm on pmhc04 and 13.4 mm on Bastila. Every solve stays at 9 to 10 mm. No slider moves it. The FLAME fits also stay at 10 to 12 mm, so FLAME cannot make it either. This needs a new construction.
- **The line under the jaw rising to a corner.** On KOTOR, the line ends 8 to 22 mm above the chin's bottom. On our presets it ends 8 to 40 mm below it, because it runs down into the neck. The held solves stay at −7 to −23 mm. The free solve reaches it on Carth, but only with Jaw width at +15.5, which breaks the jaw. The FLAME fits reach it on Bastila, pmhc04 and sith_app, but not on Carth, pmhc03 or old_a_m.
- **The width of the jaw's back from below.** On KOTOR it is 91 mm on Zherron and 77 mm on pmhc04. The held solves give 66 and 65 mm.
- **The edge outline from the side.** After the held solve, it is 8 to 17 mm off on Bastila, Carth, Zherron and sith_app. The FLAME fit is 2 to 3 mm off on Bastila, Zherron and sith_app.
- **Why.** The jaw zone (`jaw.rs` `zone`) fades out over 25 mm toward the neck's rim. None of the 5 sliders has a part under the jaw behind the chin. The neck's rim must stay at 0.000 mm (decision 44), so any new control must fade before the rim.

## Proposal for the next step

These are ideas for the prototype conversation, ranked by what the reach test shows.
1. **A construction for the jaw's lower edge**, built like the eye curve. A curve on each side runs from the chin to the jaw's corner and up the back of the jaw. The skin above the curve is pushed out, and the skin under it is pulled in toward the neck. This makes the ledge and the corner. Its handles would be the corner's place (back and height), the edge's width at the back, and how sharp the edge is. Neither the sliders nor FLAME reach the ledge, so bumps alone will probably not do it.
2. **A "Throat angle" slider**, or a rebuilt "Under the chin". The throat corner should move back and up under the jaw, so that the underside of the chin can run level into a sharp turn down the neck. It must fade to 0 before the neck's rim.
3. **A neck-width slider** on the front of the neck, below the jaw. Our base neck is 102 mm wide over its front 35 mm, and KOTOR's mean is 72 mm.

Before the prototype, the measure needs three fixes:
- Read the corner's height over a taller window. The top of the window should probably be 40 mm above the chin's bottom, with the lip ruled out another way.
- Remove Kreia's hood with a stricter cover.
- Leave out the throat rows on heads whose neck stub ends less than 32 mm below the chin. These include comm_b_m, old_b_f, n_dorakh and pmba01.

## Pictures

All of these are in `/private/tmp/claude-501/-Users-david-perso-kjk/7fae1cb7-a4ee-47b5-9bf7-3acaab4f4a13/scratchpad/`.
- `look/k_0.png` to `look/k_5.png`: KOTOR's 57 heads with the hair removed, from the side, the front, three-quarter from below and from below.
- `look/m_0.png` to `look/m_9.png`: the same heads, 6 to a sheet, with the measure's points.
- `reach_sheet.png`, with crops `reach_sheet_0.png` to `reach_sheet_2.png`: KOTOR, the jaw preset, the held solve and the FLAME fit, from the side and from below. The heads are Bastila, Carth, pmhc03, pmhc04, Canderous of the mod, old_a_m, Zherron, pfha02, comm_w_f and sith_app.
- `extremes_0.png`: the check 4 extremes.

## Not done

- No judges were run. The guide asks for them at the prototype step, and this step built nothing to judge.
- The optional off-by-default option in `kotor_fit.py`, which would score the underside of the jaw, was not added.
- The left side was not compared with the right side as a separate wobble. Only subdivision was used.
- Nothing was shown in viewer 2.

## Traps

- **Necks.** KOTOR's neck stubs are short, and some end about 30 mm below the chin. Our neck is a full tube that widens toward the nape. A width cut must therefore stay on the front of the neck. Our base head read 150 mm wide before that fix.
- **The throat's corner.** The meeting point of two fitted lines runs off to infinity when the lines are nearly parallel. Read the angle, and read the corner's place only when the angle is under 160°.
- **The cuts near the chin.** The first cuts behind the chin's tip are often unreadable. A window that starts too high takes the lower lip for the jaw's edge.
- **The steepest step.** "The steepest step in a cut" picks the flat floor of the U under the chin. Use the first place where the outline steps in faster than it goes down.
- **Polar rays and corners.** Rays from one centre, and a depth picture's steepest fall, both put points all over the jaw. Both were tried and dropped.
- **The preset's values.** `page.json` holds each head's jaw preset under `heads[i].solved.full.counts["5"].values`.
