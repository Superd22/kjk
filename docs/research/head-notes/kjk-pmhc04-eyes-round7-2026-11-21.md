# pmhc04's and Bastila's eyes, round 7 (2026-11-21)

Written by the conversation kjk-c0 for the lead kjk-48. The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/d171765e-638e-47eb-99a8-d8ef46a7501d/scratchpad/r7/`, called `$S` below. Round 6's folder is called `$R`.

## Verdict

Nothing was published. Viewer 2 stays at `?b=48`, and no live file changed.

- The new slider Outer corner reach works on Bastila. It gives her a long, thin, pointed outer corner seen from the temple, without moving the eye.
- On pmhc04 it does almost nothing. The back of pmhc04's opening lies behind the eyeball's centre. The closing lid turns about that centre, so it cannot cover an opening there, and the slider must stop short of it.
- The new Eye width works. It no longer cuts a trough under the eye. On pmhc04 at live values, the dip under the eye fell from 6.1 to 4.1 mm, with no extra folds over the 57 heads (97 fold triangles before, 101 after).
- A solve with the eye held in place reached pmhc04's outline target: overlap 0.59 to 0.72, dip under the eye 5.6 to 3.1 mm, depth 12.9 to 12.5 mm (KOTOR 12.1), and the space between the eyes 61.6 to 60.7 mm (KOTOR 60.0). The inner corner got less buried (nose_skin_z 5.22 to 4.63 mm, KOTOR 1.76).
- But both judges found the new pmhc04 worse than live in the pictures. Its eye is taller and rounder (height 8.5 mm, live 7.65, KOTOR 7.35), and its corner looks blunter. On Bastila, one judge gave a small lead to the new eye and the other called it a tie. So the brief's rule says not to publish.

## The slider Outer corner reach

It is a drawn slider, the same kind as Brow edge sharpness. It shapes the head after the smoothing and the press, so it reads each head's real eyeball and lid edges (`eye_curves::tuck_step`). Its value goes from 0 to 1.

- Round the outer corner (from 70% of the way along the eye), the skin between the two lid edges sinks 0.8 mm under the eyeball's surface. The eyeball then shows there from every side.
- The skin above, below and past the corner, up to 1.5 mm from the edges and fading out over 3 mm more, comes down to 0.15 mm outside the eyeball. It then no longer stands 1 to 3 mm out over the eyeball's side, and no eyeball is bared outside the lids.
- The lid edges in the outer part come down onto the eyeball along its radius, so they keep their place on it.
- The movement is averaged with the neighbours twice. A floor keeps the averaging from pulling skin outside the edges under the eyeball.
- The skin between the edges stops sinking 4 mm in front of the eyeball's centre (fading over 2 mm). Behind that line the closed lid cannot cover the eyeball. At 2 mm the closed pmhc04 eye showed 1.75 mm² of eyeball from the temple, at 3 mm 0, at 4 mm 0 (`DIAG_SHUT=1 shutcheck`).
- All reaches are set by `EYES_TUCK` for trials.

What it does, seen in the pictures (`$S/pics/reach_values.png`, front and temple side at 0, 0.5 and 1):
- At 0.5 both heads get a longer, thinner, pointed corner from the temple side and a clean point from the front.
- At 1 Bastila's opening ends in a round bulb past the lashes seen from the front, and pmhc04 gets a small ragged tab at the tip.
- With the lids closed (`$S/pics/reach_closed.png`), 0.5 closes cleanly. At 1 Bastila shows a skin lobe past the closed lid, and on Bastila with the candidate's values the closed eye bared 12.6 mm² of eyeball. So the high end is not safe yet. The range should end near 0.5, or the stop line must be set per head.

Measured on Bastila (candidate values, `$S/tune/`): at 0.5 the tip 1 mm from the end goes from 2.25 to 1.45 mm tall seen from the temple (KOTOR 0.9), and the opening seen from there goes from 18.35 to 18.55 mm long. Fold triangles go from 2 to 4.

## Eye width

Eye width used to turn the skin round the eyeball's centre. The skin under the outer corner swung back with the corner, and the skin further toward the temple did not, so a trough formed between them. Now the skin under the outer half of the eye that lies 2 to 5 mm or more outside the eyeball keeps its depth and only slides toward the temple (`eye_curves::width_slide`). The lids on the eyeball still turn as before.

- A first version slid the skin under the whole eye from 1 mm out. It folded the skin under the inner corner on many heads (97 to 366 fold triangles). The outer half only, from 2 to 5 mm, gives 101.
- Dip under the eye at live values, on the lines 6, 9 and 12 mm below the eyeball's centre: pmhc04 6.06, 3.97, 2.17 mm before and 4.09, 1.57, 0.72 after. Bastila changes little.
- The command `sliders eyes refield-width-r7 <data>` remakes the keys. Each key's amount is found from the stored field, which the old code matches exactly (misfit 0.0000 on all 9 keys).
- In the pictures (`$S/pics/width_slide.png`, KOTOR, live and the sliding Eye width), the skin under pmhc04's eye is a little lighter in three-quarter view. The change is small to the eye.

## The solves

All on 57 heads, warm from the live page, Outer corner depth clipped at -3, with the new rows weighted as in round 6 and with depth 4, spacing 4, lower_depth 3 and nose_skin 4 (`EYES_WEIGHTS`). Fold triangles below count only those wholly more than 0.5 mm outside the eyeball, because sunk skin can fold under the eyeball where no one sees it. Live has 89.

| Solve | What | Median overlap | Folds | pmhc04 overlap, dip | Bastila overlap |
|---|---|---|---|---|---|
| t3 | all free | 0.61 | 634 | 0.64, 2.6 mm | 0.51 |
| t4 | Eye depth, spacing and prominence held at live | 0.59 | 853 | 0.62, 3.1 mm | 0.39 |
| m4 | t4 pulled back per head to the largest share (0, 0.25, 0.5, 0.75 or 1) with no more folds and no more eyeball with the lids shut | | 66 | 0.61 | 0.48 |
| F2 | m4, with pmhc04 at t4's full values and Lower lid drop, outer at live | | 66 | 0.72, 3.1 mm | 0.48 |

- The folds come mostly from the other sliders the solve moves, not from Outer corner reach. t3's values with the reach at 0 still fold 605 triangles.
- The solve leaves Outer corner reach at 0 on pmhc04 and Bastila, and uses it on about 25 other heads.
- The placement held. On pmhc04 in F2: depth 12.5 mm (KOTOR 12.1, live 12.9) and spacing 60.7 mm (KOTOR 60.0, live 61.6). Bastila: depth 10.7 mm (KOTOR 11.6, live 11.4).
- Guard (`$S/guard7.py`, round 6's with folds counted as above): with every other head at live values and 27 heads taking F2's values (folder `$S/G`), only 3 heads fail. mod_exile_canon and mod_hp_pfha01 fold 2 more triangles, and pmha02's dip moves 0.4 mm from KOTOR's. Bastila fails on F2 (outer_bulge 4.15 to 3.60 mm against KOTOR's 6.61, and the eye's height moved). pmhc04 passes.
- The 27 heads were not checked in pictures, so none of them may take a new preset yet.

## The judges

Two Opus judges each saw one sheet per head, KOTOR in the middle, live and F2 above and below, with the order swapped for the second judge (`$S/judge/*_j_s.png` and `*_j2_s.png`, `key.json`: pmhc04 has F2 on top in the first order, Bastila has live on top).

- pmhc04: both judges preferred live, by about one point. They found the new eye taller and rounder, and its corner blunter from the temple side and in profile. One saw a softer, broader shadow under the eye on the new head, the other a heavier one.
- Bastila: one judge preferred the new eye by a small margin, for its long, pointed wedge from the temple side, but saw its outer corner tilt up into a cat-eye look. The other called it a tie: the new outer corner is closer, but live keeps a clearer crease under the eye.
- My own look agrees. The new pmhc04 eye reads larger and rounder from the front. Bastila's new corner is longer and more almond.

## What changed

Code in the worktree `head-paint`, `crates/character_customisation/head/head_sliders/src/`. Nothing is committed and the wasm was not rebuilt.
1. `eye_curves.rs`: `tuck_reach` and `tuck_step` (Outer corner reach), and `width_slide` used in `Curve::Width`. The slide changes Eye width's fields only when they are remade, so live data is unchanged.
2. `eyes.rs`: SPECS has 31 entries with Outer corner reach (group corners, measure temple_len, kind drawn). `Eyes::drawn_step` returns `tuck_step` for temple_len. Commands `make-pmhc-r7` (adds the slider to a data folder) and `refield-width-r7`.
3. Shared log: one line from round 7. Roadmap: a round 7 line under "Now and next".
4. Diag: `$S/diag` (built into `$S/diagt3`). `views` and `shutcheck` take `TAGS_SET`; `shutcheck` also gives the eyeball shown from the temple side with the lids closed; `DIAG_TUCK` applies the step directly.
5. Scripts in `$S`: `trial7.sh` (a solve and all its checks), `foldscan.sh` and `pickt.py` (per-head pull-back), `mixeval.sh`, `tune.sh` and `setv.py` (one head's values by hand), `summ7.py`, `guard7.py`, `tv.sh` (close-ups at several values of the slider).
6. Data: `$S/b7` is round 6's `a6` with Outer corner reach and the sliding Eye width. `$S/F2` and `$S/G` are the candidates above.

## How to undo the round

Nothing live changed. To undo the code, copy `$S/orig/head_sliders_src/` over `head_sliders/src/` and `$S/orig/head_eyes_src/` over `head_eyes/src/`, if no other session has edited them since 11:25 (check the shared log). `head_eyes` did not change in this round.

## Traps I hit

- Field sliders are baked into the data. A code change to a curve does nothing until its fields are remade (`refield-width-r7`). Refield from the original data, never from a folder already refielded, or the amounts are fitted to the new fields.
- The closed lid turns about the eyeball's centre. Behind that centre it moves up, away from the opening. Any opening behind the centre stays bare when the eye shuts.
- The fold check counts triangles that fold under the eyeball, where no one sees them. Count only those wholly outside it (`EA_WHERE=1`, the 7th column over 0.5 mm).
- A good outline overlap can come with a taller, rounder eye. The judges punish that. The solve needs the opening's height held near KOTOR's.
- Rebuilding a binary that a running trial uses is unsafe. Each code change went into a new target folder (`target2`, `target3`, `diagt2`, `diagt3`).
- In zsh, `echo ==` is an expansion, and `$v_` reads as the variable `v_`.

## For a follow-up round

- Hold the eye's opening height in the solve (weight `height` and `upper_rise`), then judge pmhc04 again. F2's pmhc04 values are in `$S/F2/web/eyes/page.json`.
- Give Outer corner reach a range of 0 to 0.5, or make its stop line follow each head's closed lid, then retry Bastila with it at 0.5.
- For pmhc04's far corner, either move pmhc04's eyeball back (KOTOR's eyeballs sit 5 to 10 mm further back) or let the closed lid piece cover openings behind the eyeball's centre.
- The 27 heads that pass the guard in `$S/G` need front close-ups and closed-eye pictures before any of them takes a new preset.
- Publishing the sliding Eye width alone is possible: it adds 4 fold triangles over all heads and lowers pmhc04's dip. It needs the wasm rebuilt, viewer versions raised, and the remade `web/eyes/sliders.json`.

## For David

- **Viewer 2 did not change.** It is still `?b=48`.
- **A new slider, Outer corner reach, makes the long, pointed outer corner without moving the eye.** On Bastila at half strength it gives a thin wedge that runs round the side of the eyeball, close to KOTOR's.
- **It cannot do the same for pmhc04 yet.** The back of pmhc04's eye opening lies behind the middle of the eyeball. When the eye shuts, the lid turns about that middle, so it cannot cover anything behind it, and the slider must stop short there.
- **Eye width no longer digs a hollow under the eye.** The skin under the outer half of the eye now slides sideways instead of swinging back. On pmhc04 the deepest dip under the eye went from 6.1 to 4.1 mm, with no extra skin folds.
- **The re-solve kept the eye in place but made pmhc04's eye too tall and round.** Both judges preferred the current pmhc04. Bastila's new eye was only slightly better or a tie, so nothing went live.
- **Pictures:**
  - `/private/tmp/claude-501/-Users-david-perso-kjk/d171765e-638e-47eb-99a8-d8ef46a7501d/scratchpad/r7/pics/reach_values.png` shows Outer corner reach at 0, 0.5 and 1 on pmhc04 and Bastila, from the front and from the temple side, with KOTOR on the left.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/d171765e-638e-47eb-99a8-d8ef46a7501d/scratchpad/r7/pics/reach_closed.png` shows the same with the eyes shut.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/d171765e-638e-47eb-99a8-d8ef46a7501d/scratchpad/r7/pics/width_slide.png` shows KOTOR, live and the new Eye width on both heads.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/d171765e-638e-47eb-99a8-d8ef46a7501d/scratchpad/r7/judge/pmhc04_j_s.png` and `/private/tmp/claude-501/-Users-david-perso-kjk/d171765e-638e-47eb-99a8-d8ef46a7501d/scratchpad/r7/judge/p_bastilah_j_s.png` are the judges' sheets. KOTOR is in the middle. On pmhc04 the new solve is on top. On Bastila it is at the bottom.
