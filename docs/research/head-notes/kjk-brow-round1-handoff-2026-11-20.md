# Brow round 1 (repair Brow bulge and Brow sweep): handoff

The brow lead does this round itself (David: no sub-agents for building). Scratch folder:
/private/tmp/claude-501/-Users-david-perso-kjk/c0e6bd7c-e741-42d9-bf56-a0d142515190/scratchpad/b1/
(`orig/brow.rs` = brow.rs before, `live_before/` = live brow data before, `d/` = scratch data folder with links to the live files
except `brow/` and `web/brow/`, `target/` = own CARGO_TARGET_DIR).

## Where I am

- 23:55 Audit with `feature_audit brow sweep d <57 heads>` (an_sweep.py summarises). Live presets: 6 heads unmeasurable (comm_a_m,
  n_admoffh, n_terenaadareh, old_a_m, pmha01, pmha06: "no brow ridge"), Brow sweep runs backward over one step on 8 heads, Brow bulge
  barely moves at its low end on 5 heads.
- 00:05 Cause (new audit command `feature_audit brow ridge <slider> <heads>`, fn `brow::ridge_report`): at 38 to 50 mm from the middle
  the outer brow and the socket's rim are one flat face on those heads, so the cut never falls 3 mm behind the crest and `ridge_at`
  returned None. That failed the whole measure, and it switched Brow sweep's outer end between 50 mm and nearer places (the jumps).
- 00:10 Fix in brow.rs `ridge_at`: the edge is the cut's lowest place when the fall never reaches 3 mm (continuous with a fall that
  just reaches it). Sweep after: every head readable, no backward step, gains 0.64 to 1.73 overall.
- KOTOR marks re-read in d (`sliders brow marks d`): brow_sweep wobble 0.76/2.19 (was 0.94/2.42), spread 3.95 (was 3.74), mark 4.35
  (was 4.86). Other rows moved by under 0.2.
- Brow bulge: the measure is crest depth minus the forehead 15 mm above the edge, so it cannot go below 0. The low end is a floor, not
  a fault: no head misses on bulge at the low end. Real problem: high end (Jolee needs 5.4, Malek 4.9, Kavar at the end) and the pinch
  between the brows at the high end (report-brow-cheeks).
- Shared changes logged: solve.rs `Evaluator::surface`.
- Running: `sliders brow make d` (full remake in the scratch folder, to compare). Plan: keep the 4 other sliders as live, take Brow
  sweep and Brow bulge only (SLIDERS_ONLY), presets, pictures, then publish.
- 00:20 Kit: Feature::span_quantile_of and span_cap_of (feature.rs, bench.rs, logged). brow.rs: bulge and sweep reach every head, bulge
  cap 5 spreads; sweep frees forehead_profile. Remade only those two (SLIDERS_ONLY), merged into the live set (sliders_r1b.json).
  Presets: 32 of 57 made (was 27), sweep at an end 1 (was 4), beyond 0 (was 4); bulge at an end 16 (its floor), beyond only 3 heads by
  0.3 mm at the low end.
- 00:30 LIVE ?b=41 (module 41). Backups in b1/live_before/ (brow, web_brow, wasm, page). Before-data folder for pictures: b1/o.
- 00:35 Blind Sonnet judge (judge_a.png X=after, judge_b.png X=before; key b1/judge_key.txt): after won pmha04; before won Jolee
  (diagonal crease above the brow at bulge 6.0) and Bastila (groove across the lower forehead at sweep -6.0); 6 the same.
- STOPPED for David's look. Open: Bastila's groove, Jolee's crease, the rope and pinch at bulge's high end. To undo: copy
  b1/live_before back and set the versions to 40/34.
- Round 2 (David: "brows look better", keep ?b=41; then fix Bastila's groove and Jolee's crease). feature_audit pictures: a label
  "name@Slider=value,..." sets sliders over the preset. Scratch folder b1/d2.
  - Bastila: the groove grows with Brow sweep below about -3; on the base head the low end is smooth; with every other slider at 0 it
    mostly goes away, and no single other slider at 0 removes it (bast_pairs.png). Freeing forehead_curve for sweep: 33 of 57 made,
    groove the same (bast_sw2.png). Reverted.
  - Jolee: the diagonal crease grows with Brow bulge above about 4.2 (jl.png). Bulge from "ridge" only (middle as repair): crease the
    same, 31 of 57, 14 too flat between the eyes (jl2.png). Reverted. Code = live (r1b) again.
- Round 2 slider: "Brow between the eyes" (brow_middle; parts middle+ridge, repair forehead, free forehead_profile, loose bulge, sweep,
  height, line). Range -2.92 to 3.75 (made to span). Scratch d3. 44 of 57 made (was 32); too flat between the eyes 1 head (was 13).
  Leak: nose root_dip -0.88 / +0.75 spreads at the ends. Ends sheet mid_ends.png (no knob), heads mid_heads.png (straighter heavy
  brows, closer to KOTOR). LIVE ?b=42 (module 42); backups of ?b=41 in b1/live_r1/. A Sonnet viewer check is running.
- Height slider skipped (David agreed): only comm_a_f2 and Zherron miss on brow height after round 2, and the edge sits 4 to 10 mm above
  the eye centre, inside the eye paste.
- Brow edge sharpness, measure trial (code kept in b1/brow_roll_trial.rs, NOT in the crate; brow.rs is back to the live r2 version):
  brow_roll = height between the cut's fall of A and B mm behind the crest, over x 22..X. Best: A=2, B=6, X=34: wobble 0.52/1.27,
  spread 1.70 (as weak as brow_bulge). Sharpest KOTOR: n_daraalah, n_jedimalek, n_admoffh, p_joleeh, pmha04, mod_kenobi; softest:
  old_a_f, n_hussef, mod_exile_canon, pmhc03, pfha06, mod_jaesa (roll_sharp.png, roll_soft.png). Malek, admoffh, Jolee and Jaesa look
  right; Daraala, Kenobi, pmha04 and pfha06 are read on hair over the brow; old_a_f does not look soft.
  Blocker: brow bumps are 9 mm Gaussians, too broad to make an edge that turns within 2 to 4 mm.
- Eyes lead's terms for any brow change near the eye: full effect down to 8 mm above the eye centre, fade to 0 between 8 and 6 mm,
  0 below 6 mm, measured from the eye centre after the whole-eye sliders; check field() reach in eye_curves.rs; fold view with lid
  sliders at their ends on Bastila, Kreia, Zherron; close-ups open and closed.
- STOPPED by David (2026-11-21). The brow edge slider goes to a new conversation started by the eyes sliders session.
