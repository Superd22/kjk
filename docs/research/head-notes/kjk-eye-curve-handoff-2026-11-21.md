# Eye curve construction, handoff (2026-11-21)

Brief: /private/tmp/claude-501/-Users-david-perso-kjk/fa94c09e-e48c-4836-b534-90bfa0de7855/scratchpad/brief-eye-curve-construction.md
Scratch: /private/tmp/claude-501/-Users-david-perso-kjk/d55dbdf6-456c-4c14-ac07-1a61b76e9013/scratchpad ($S below)

## Where I am
- Read the corners note, the shape fit note, the research note and the memory notes.
- Copied the corners diag to $S/diag (own copy, links the worktree crates read only). Building it.
- Plan: prototype module $S/diag/src/curve.rs. Data = the live data folder (read only), so the whole-eye values and the rest of the head are live.
- 17:xx Prototype in $S/diag/src/curve.rs, command `cproto <data> <prefix> <heads|all>` (data = live folder, read only). Construction: the rim = layer 3 out from the lid loop (the live skin's fold, found on 5 heads) laid on the curve by share; layers 0-2 tucked under the lid just outside the curve at 0.06 mm; front ring stretched along rays from the curve's middle in the flat map (`CC_MODE=polar`), heights from the live skin's first outward hit (grid) blended down to 0.15 mm at the curve over 0.12 rad; inside component sunk under the ball. Hidden-group vertices must move too (they caused the blue/red spikes). Lid piece = upper arch swept about the centre line parallel to the tips' line.
- The curve is fed by direction (KOTOR's eyeballs are 1.0-1.7x ours in radius), so the "fed" score (ours vs KOTOR's directions on our eyeball) is the construction's fidelity; the raw KOTOR scores stay as in the earlier notes.
- 18:xx Step 1 frozen: binary $S/cc_step1, source copy $S/curve_step1.rs. Defaults: rim layer 3, under 0.03 rad/layer at 0.06 mm, front ring stretched along rays from the curve's middle (CC_SPINE 0.5), relax 40 with push-out along the curve's normal, heights from the live first-hit grid only within 0.25 rad of the curve (CC_HFADE), ring = within 15 mm of the live or the new rim. Outputs in $S/step1 (all_m.json, overview57.png, p_<head>.png, p_corners_<head>.png).
- Known faults: mod_exile_canon spike (triangles span the much larger KOTOR opening), pmha02 small steps at the outer upper lid, Kreia/pmha01 outer tips behind the equator read 5-10 deg short, pfha05 long inner spike reads blunt, fold check 24-150 per head vs 0 live (thin fans beside the nose / under the outer corner). A triangle push-out pass (CC_ROUNDS>1) made it worse; left off.
- Next: step 2 handles. Rust `cexport` (fed curves in flat mm), Python fit (scipy), Rust Target::from_handles + `chand` command for fits and ends.
- DONE. Final binary $S/cc_final (slope cap 1.5, lid piece 0.08 mm), results in $S/final1 (step 1) and $S/final2 (step 2, ends in $S/final2/ends). Note written: /var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-eye-curve-construction-2026-11-21.md. Messaged kjk-48. Nothing live touched.
