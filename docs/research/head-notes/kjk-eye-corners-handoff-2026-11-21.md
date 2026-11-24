# Eye corners, handoff (2026-11-21)

Brief: /private/tmp/claude-501/-Users-david-perso-kjk/fa94c09e-e48c-4836-b534-90bfa0de7855/scratchpad/brief-eye-corners.md
Scratch: /private/tmp/claude-501/-Users-david-perso-kjk/57efa471-d38e-4773-9d9a-64f6ff7ae461/scratchpad ($C below)

## Where I am
- Step 0: reading inputs (shape-fit note, rounds 5-7 notes). Nothing built yet.
- 15:10 Corner measure built in diag copy $C/diag (shape.rs `Eye::corner`, `corners_json`): dense march ±35° round each tip at 0.5°, opening's edges 0.25..4 mm behind the tip on the surface (unrolled) and in 4 views; angle (from 2 mm) and bluntness h(0.5)/h(2). Proof on 54 heads: L-R vs head-head ratio 5-11 for angle; KOTOR blunt median 0.26-0.28 (points), ours 0.54 (outer) and 0.90 (inner). Extremes sheet $C/pics/corner_extremes.png. pmhc03 inner misreads.
- 15:40 Code (worktree, originals in $C/orig): eye_curves.rs `corner_carve` + `HANDLES` + `handle_values` (always-on carve after the press when the set has the handle sliders); set.rs build and solve.rs positions call it. piece.rs: Frame.rot, Settings.tip_axis / tip_axis_centred (EYES_TIP_AXIS=1 tips' line, =2 through the centre parallel to it), `turn_about`; eyes.rs our_scene uses turn_about. Data $C/d8 = b7 + 6 handle sliders ($C/py/addh.py).
- Findings: carve at 0 opens the outer corner along the lid edges (long wedge like KOTOR from the temple). Axis through the tips' line makes a huge floating closed piece; axis through the centre parallel to it (=2) closes cleanly on Bastila, small slivers on pmhc04.
- 16:00 Fold verdict: the guard's folds on Bastila, Zherron, pfha05 (fit note) are all on hidden-group triangles (never drawn): not real. Guard in diag eyefit/foldcount now skips hidden triangles. Fold penalty test: without it pmhc04's inner corner gap 1.35 vs 1.33 (not the cause); pmha01 0.67 vs 1.12 (partly).
- Carve trials (foldcount over 57, live 84 folds on 10 heads): full carve both corners 434; inner carve folds visibly (Zherron's inner upper lid tears) -> inner carve OFF (mode 0); outer carve from 0.1 rise 0.6 + outline smoothing 3 mm: 107 folds on 8 heads, no notch. Defaults now [0.75,0.15,0.1,0.6,1.5,3,0.8,0.15,1,1,4,3,100,0,2], smooth 3.
- 8-head solves: A (handles replace old corner sliders) worse than B (handles + old ones). So the old ones are not redundant in practice.
- 15:30 running: eyefit all 57 on d8, EYES_TIP_AXIS=2, FIT_W 1,1,1, skip Outer corner reach + Inner tip shape -> $C/m/fitC_all.json (log fitC_all.log, binary pmhc_diag_v7).
- Next: fsumm, fitvals, sheets.sh for the 8 heads, overview of 57, note, message kjk-48.
- 15:50 DONE. 57-head solve finished (54 heads at full share; medians curve 2.16->0.78, ring 2.02->1.35, outer tip 6.1->0.9, inner tip 7.8->5.4; folds 29 on 3 heads vs live 84 on 10). Sheets in $C/out, publish data in $C/publish/web_eyes. Note: /var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-eye-corners-2026-11-21.md. Stopped building in head_sliders/head_eyes.
