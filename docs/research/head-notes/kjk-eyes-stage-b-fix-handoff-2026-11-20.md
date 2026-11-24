# Eyes stage B fix round: handoff (2026-11-20)

Brief: /private/tmp/claude-501/-Users-david-perso-kjk/78e955ef-9cc3-4aa3-9d80-4f3216d71481/scratchpad/brief-eyes-stage-b-fix.md
Stage B note: /var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-eyes-stage-b-handoff-2026-11-20.md
Scratch folder: /private/tmp/claude-501/-Users-david-perso-kjk/78e955ef-9cc3-4aa3-9d80-4f3216d71481/scratchpad/sbf/

## Where I am

- Started. Read the brief, stage B note, stage B brief, notes and rules. Reading the code next. Nothing changed yet.
- Press cause found: the stand-ins' lid edges sit 1 to 4 mm off the eyeball (jagged, up to 3 mm between neighbouring edge vertices), and the old press forced them onto the ball over 3 graph rings, so the skin tore and folded. Also `fault-places` read the page's values by count 24 (missing), so it measured all-zero values.
- New press (eye_curves.rs): keeps each edge vertex's distance from the ball as it is on the start (identity at zero values), smooth blend of corrections (2.5 mm along the edge), whole correction within 2 mm and fade over 7 mm by straight-line distance, 3 passes, corner tips pressed too. set.rs passes `start` to `press` (shared log). Seat of the eyeball (EYES_SEAT=1) tried and left off: it pokes the ball through the skin.
- Lining keeps a corner's turn further than the skin (deep_weight in eye_curves.rs) for Eye width and the corner sliders.
- Old binary with the old press: sbf/sliders_old (build_old.sh). Diag: `sliders eyes press-diag <data>` (PRESS_SOLVED=1), `closeup <data> heads` (CLOSE_ZERO, CLOSE_SET).
- Running: bench in sbf/d with 17 solve sliders (outer corner position, height, angle added), log sbf/bench1.log. Then place-whole.
- 18:20 Re-solve 1 (sbf/d, 17 sliders incl. outer angle): 56/57 bench, 55/57 window check after place-whole. Outer corner angle alone faults 43 heads (787 triangles). Re-solve 2 running without it (sbf/d1, log bench2.log). place-whole and fault-places read counts by the page's slider count now (bug fixed).
- Lead asked (18:10) for a flush press: the press is back to the base thickness, corrections averaged along the edge over 3 mm, whole within 4 mm, fade over 18 mm, corner fade 12% of the edge, 2 passes. Live ?b=31.
- Re-solve 2 (sbf/d1, 16 solve sliders with Outer corner position and height; Outer corner angle left at 0 because it alone faults 43 heads): bench 50/57 with a fault, after place-whole 54/57 (window check; 53 without the press). Outer corner position within its mark 2/57 before, 7/57 after. Live ?b=32 (page, made, starts, sliders_solve); stage B files in likeness-fit-data/sliders/eyes/stage_b/.
- Still wrong: pfha05, old_a_f, pfha02 tear at the outer corner at their presets (Eye width at its max 7.78 with large spacing and Outer corner position). Closed lid gaps not fixed (fix 5), inner corner gap not looked at (fix 6), upper lid shelf not measured (fix 3).
- Sheets: sbf/close_before_final.png, sbf/close_after_final.png, sbf/*_before_after.png, sbf/flush_z1.png (zero values), sbf/ends_pfha05.png.
- Running: Sonnet judge and Sonnet viewer check.
- Judge and viewer check done. Female A5 (pfha05) is still torn at the outer corner and under the lower lid at its preset (Eye width 7.78, Eye tilt 12, spacing 6.3), and its closed lid splits at the outer corner. Bastila, Carth, Kreia, comm_a_f and n_dodonnah are better (thinner upper lid edge, flush lids). Stopped and reported. Next step: repair Eye width's and Outer corner position's high ends at the outer corner, then the lid piece's closing at the outer corner.
