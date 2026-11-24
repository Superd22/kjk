# Eye refit handoff (2026-11-21)

Conversation started by Eyes HQ (kjk-48) from the brief /private/tmp/claude-501/-Users-david-perso-kjk/fa94c09e-e48c-4836-b534-90bfa0de7855/scratchpad/brief-eye-refit.md.
My scratch folder: /private/tmp/claude-501/-Users-david-perso-kjk/e584df1f-e66d-4e3c-9618-478e0dca7eea/scratchpad (called $P).

## Where I am
- Read the brief and the three notes (ring, construction, shape fit).
- Copied the ring conversation's diag tool to $P/diag to add the refit solver.
- Found: mod_hp_pfha01's KOTOR curve read spikes into the brow (a slit). Added a wide second despike pass in $P/diag/src/shape.rs (DESPIKE_NARROW=1 turns it off).
- Found: mod_hp_pfha01 and mod_hp_pfhb02 have handle fits that miss by 4 to 6 mm in v4 (ranges.json "loss"), so their v4 eyes are wrong. The refit starts them from the medians.
- Found: pmha02's hollow above the upper lid is already in live, not made by the ring.
- Wrote the command `rfit` in $P/diag (source also in $P/rfit.rs). First run: $P/fit/f1.json on the 14 test heads.
- KEY FINDING: our eyeball centres sit 1 to 8 mm from KOTOR's (pmhc04 2.7 mm higher; pfha05 8 mm forward). The shape note's measure lines both up on the centre, so v4's handles were fitted to a curve moved off its place in the face. In the head's frame v4 is worse than live on pmhc04 (curve 1.57 -> 2.72 mm). rfit now measures in the head frame (RF_CENTRED=1 gives the old way) and holds the eye centre within 0.75 mm of live's.
- Second run: $P/fit/f2.json / f2.log (head frame). f1 was the centred version, superseded.
- Terena Adare: the sharp ridge on the nose bridge is in live already (v4/r_n_terenaadareh.png), so it is the nose feature's, not the eye work's.
- pmha02's live eye sits ~3.5 mm higher than KOTOR's while brow, nose and ears line up ($P/sheets/wcmp_pmha02.png). The hold now lets the centre move along the line from live's centre to KOTOR's (plus 1 mm).
- Third run, all 57 heads: $P/fit/f3.json / f3.log, with Outer corner place capped at 25.8 and Outer corner height at 4.6 (just above every KOTOR fit).
- Sheet command `rsheet <data> <prefix> <fit.json> <heads>` (rows KOTOR, LIVE, V4, REFIT, REFIT CLOSED, REFIT LINES). Summary: python3 $P/summ.py <fit.json> [v].
- Viewer 2 headless screenshots: uv run -q --with playwright python3 -I $P/shot.py <url> <prefix> <wait> "Eyes,Front,Side" (the Chrome extension was not connected).
- Edited (logged in the shared change log 18:50): crate PIN_STEP 0.03, PIN_RISE 0.5, wasm rebuilt; sliders.js v5 with refit.json support. refit.json NOT yet written. Backups in /var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-eye-refit-backup-2026-11-21/. Diag runs must now use CC_FLOORW=1.5 CC_PIN_RISE=0.5 CC_PIN_STEP=0.03.
- Export: python3 $P/export.py <fit.json> <refit.json>.
- f3 (all heads, LM only): median curve live 2.01 / v4 2.57 / refit 1.40 mm; ring 1.39 / 1.58 / 1.30. LM stalled on several heads.
- Added a coarse pattern search after LM (RF_PATTERN passes, default 3): pmha02 cost 40 -> 9.8, Jolee 32 -> 9.9, pmhc04 15 -> 4 (f4t.json).
- Guard fold margin now RF_FOLDTOL=8 (pmha04's unguarded fit was clean in pictures, $P/sheets/s4fit_pmha04.png, but had 8 more counted folds). Brow edge sharpness no longer solved.
- Fifth run, all heads: $P/fit/f5.json (running; diag env CC_FLOORW=1.5 CC_PIN_RISE=0.5 CC_PIN_STEP=0.03 + the outer caps).
- f5 is the result: median opening gap live 2.01 / v4 2.57 / refit 1.00 mm, ring 1.39 / 1.61 / 1.23. Table: $P/score_table.md. Sheets: $P/sheets5/s_<head>.png, overview $P/sheets5/overview.png (ov_0..3 quarters).
- PUBLISHED to the preview (19:40): data/refit.json and capped ranges.json. Viewer 2 check ($P/vcheck.py): preview on equals the refit within 0.0002 mm; off equals live; off-on-off identical.
- Viewer 2 shots: $P/shot2.py -> $P/v2/v2_<head>.png (running for the 14 test heads).
- Outer corner ends at the new caps: $P/ends/outer_p_kreiah.png, no fold seen.
- mod_hp_pfha01: KOTOR's curve read is still broken (spikes through slits); its score is not trustworthy.
- LEFT TO DO: look at the v2 shots, write the note /var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-eye-refit-2026-11-21.md, message kjk-48.
- DONE: note written to /var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-eye-refit-2026-11-21.md; sliders.js "Known faults" hint updated; messaging kjk-48 and stopping.
