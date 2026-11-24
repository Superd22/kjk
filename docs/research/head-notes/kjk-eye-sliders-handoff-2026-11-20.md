# Eye sliders: handoff (night of 2026-11-20)

The task: the eye sliders for kjk's head customisation, built with the slider kit in `head_sliders` (worktree head-paint), in the
nose's and the jaw's fashion. The brief is the lead's message; the rules are the scratchpad's `overnight-rules.md`. Two rounds, then
stop and report.

## Where I am

- 2026-11-20 about 03:40: read the rules, the roadmap, the kit handoff, the nose handoff's traps, the jaw report, the eye audit
  (sections 2, 3, 5, 8) and the lid piece report. Now reading the kit's code (`feature.rs`, `jaw.rs` done; `maker.rs`, `measure.rs`,
  `bench.rs` next).

## Where I am (about 04:20)

- **Feature `eyes`** in `head_sliders/src/eyes.rs` (my file). Registered in `lib.rs` and `feature::by_key` (logged in the shared log).
  The measure: a depth picture every 0.1 mm round each eye, seen from the front; the opening is where the eyeball lies in front of
  everything else. On KOTOR the eyeball is KOTOR's own low-poly eyeball mesh (a fitted sphere gave a wobbly lower lid) and KOTOR's lid
  piece is drawn with the skin. On ours the eyeball is a 12.3 mm sphere at the frame's eye centre, sized with `Surface::scale` (a new
  field, logged). 11 proportions and 2 outlines, averaged over the two eyes. Per-eye values are in `marks["per_eye"]`.
- **KOTOR's eyeballs and lid pieces**: `~/.claude/tools/likeness-fit/sliders/prep_eyes.py` (new, run once) writes
  `likeness-fit-data/sliders/eyes/kotor_eyes.json` from the lid page's viewer data (`lids/vd`). Eyeballs are found geometrically (not
  face, not lid, near the eye), because comm_a_m and mod_kenobi skin them to the head bone. 3 heads have no lid pieces.
- **Check 1** (median/95th wobble against KOTOR's spread): width 0.50/1.30 vs 2.73, height 0.30/1.10 vs 1.69, tilt 0.54/2.64 vs 3.17,
  upper arch 0.10/0.49 vs 1.24, lower curve 0.38/1.08 vs 1.39, outer corner 2.5/10.4 vs 13.4 deg, upper lid depth 0.9/3.8 vs 4.2.
  Fail: the peak positions, the inner corner's angle, the lower lid's depth. Mod Canderous: no opening found (nearly shut eye).
- **Default eye** (`sliders eyes default-eye`): the stand-ins and the smoothed fits have an opening 23.6 mm wide and 7.2 mm tall
  (frame mm), KOTOR's seen eyes 26.7 by 8.8 (median of 114). Median need: width +2.9 mm, height +1.2 mm. The base head itself is
  24.65 by 9.25.
- Tools in `eyes::tools` (my file), run through `sliders eyes <command>`: `kotor-eyes` (sheet KOTOR next to ours with eyeballs and
  pieces), `default-eye`. The hook is 3 lines in `src/bin/sliders.rs` (logged).

## Where I am (about 05:00)

- 7 specs in `eyes.rs`: Eye width, Eye height, Eye tilt, Upper lid arch, Lower lid curve, Upper lid fullness, Outer corner shape.
- First make failed for the lower lid: the zone gave weight 0 to every vertex the smoothing holds (the jaw's rule), and many of the
  lid margin's front-skin vertices are held, so the margin was frozen and its neighbours flipped. Now only the 16 eyeball-centre
  vertices have weight 0. Note: the kit's fault check counts any hidden vertex above 0.6 m as a "nostril", so the eye lining's flips
  show as "a nostril turned over".
- New commands in `eyes::tools`: `ends` (each slider's ends, open and closed, with eyeball and piece), `lid-safety` (writes
  `eyes/lid_safety.json`), `default-sheet <name>=<value> ... -- <heads>`, `made-eyes <heads>` (needs the bench's `web/eyes/page.json`).
- Trial make of height and lower lid running (log in scratchpad `eyes/make2.log`).

## Where I am (about 05:40)

- The lids moved straight in y and cut into the eyeball or left it. New shared hook `Feature::shape_bump` (empty by default, called in
  `maker::bumps`, logged): the eyes take out the radial part of each bump's movement within 0.8 mm of the eyeball, fading to 2.5 mm.
  Trial ends (height, lower lid) are now clean. Full make running (log `eyes/make5.log` in the scratchpad), then: `ends`,
  `lid-safety`, `bench`, `made-eyes`, judges, page.

## Where I am (round 2 started)

- Round 1 done and judged. Files kept in `likeness-fit-data/sliders/eyes/round1/`. Round 1: 7 sliders; width reached only -1.95 to
  +1.45 (asked about ±5.5) because the opening cannot be wider than our 12.3 mm eyeball seen from the front (26.3 mm in the base
  frame). Scorecard provisional 36 of 57 made (all 7, full), 0 strict. Faults: ends clean, but 17 of 21 pairs and 46 of 57 solved
  heads have flips 13 to 16 mm from the eyeball centre (lid roll and margin, mostly the outer corner). Lid checks: stand-ins as they
  are 97 of 116 eyes pass; the smoothed fits 71 of 116; ends 69 (width high) to 101.
- Judges: ends judge flags height low, arch low (lash strip floats off), outer corner low (spike), tilt low (crease), fullness low
  (tent when closed) and high (heavy ridge), width high (corner crease). Likeness judge: 1 close, 5 somewhat close, 2 different
  (pfha02, pfhb04); "ours too open" is the iris size (KOTOR's eyeballs up to 20 mm radius), the measured heights match.
- Round 2: bumps 4.0/3.5 mm (were 3.0/2.5), remake running (`eyes/make6.log`), then `sliders eyes cap` on the flagged ends, bench,
  ends, lid-safety, made-eyes, judges again. Page: a Sonnet sub-agent adds the eyes to the nose page (eyeballs and pieces via
  head_eyes' LidMaker, "Proposed default eyes" = width +1.4, height +1.2).
- After round 2: `sliders all-features starts` (feature::all() now includes the eyes), roadmap line, report.

## Where I am (round 2 done, judging)

- Round 2: bumps 4.0/3.5, then capped (`sliders eyes cap`; uncapped set in `web/eyes/sliders_uncapped.json`): Eye height low -2.4,
  Upper lid arch low -1.8, Outer corner shape low -25, Eye tilt low -4.5, Upper lid fullness -2.0 to 4.5. Bench: 37 of 57 made (7
  sliders, provisional), 0 strict; stand-ins alone 13 of 57 on proportions only. Pairs 15 of 21 faulty; solved heads 43 of 57 with a
  fault. Lid checks in `eyes/lid_safety.json`. starts_all.bin rewritten with 5 features. Page done by a sub-agent (nose.js?v=23).
  Roadmap line updated. Waiting for the round 2 judges, then the final report.

## Final state

- Two rounds done and judged; stopped as the rules ask. Round 2 judges: ends still to repair are Upper lid fullness high (+4.5,
  heavy ridge), Lower lid curve high (+2.5, crease under the eye), Upper lid arch low (-1.8, lash strip sticks out), Eye height low
  (-2.4, closed lid does not seal). Likeness: 2 close (pmha01, old_w_m), 3 somewhat close (Bastila, Carth, Kreia), 3 different
  (Zherron, pfha02, pfhb04). Proposed default (width +1.4, height +1.2) is a small step closer but shows the eyeball's side at the
  outer corner on some heads. Final report sent to the lead.
