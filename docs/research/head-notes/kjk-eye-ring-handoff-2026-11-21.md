# Eye ring handoff (2026-11-21)

Brief: /private/tmp/claude-501/-Users-david-perso-kjk/fa94c09e-e48c-4836-b534-90bfa0de7855/scratchpad/brief-eye-ring.md
My scratch folder: /private/tmp/claude-501/-Users-david-perso-kjk/a18ead03-cc34-498d-89ad-8295e8c70b3e/scratchpad

## Where I am

- Started. Reading the prototype note and the crate head_eye_curve.
- (about 45 tool calls) My copy of the shape tool is `diag/` in my scratch folder. The new ring is `curve::RING == 1` in `diag/src/curve.rs`: the rim's move in 3D, spread by a biharmonic field (`smooth_move`, conjugate gradients) over 22 mm, with an obstacle loop that holds vertices that land inside the opening or too near the eyeball. New command `cring` (David's views plus close-ups, old ring against new). `CC_RING=0|1` picks the ring in `cproto` and `cends`.
- At the fitted values, the folds drop from about 100 to 6 to 23 per head, and the ring measure is about the same. The dark wedge toward the nose and the creases under the eyes are gone on pmhc04. Next: the handle ends (inner corner place high and inner corner height low still make walls).
- (about 75 tool calls) Holding vertices that land inside the opening made the stair slabs, so the hold is now off (opt in with `CC_HOLD`). Heights get a smooth floor instead (`CC_RISE` 0.5 mm per mm from the curve, `CC_SOFT` 0.3). Ends on 8 heads in `ends2/`: folds 0 to 50 per case against 29 to 299 before. Fit sheets in `fit1/`. Left: outer corner place high, outer corner height high (Kreia, Jolee), the bright dot at the outer tip, the closed lid's slightly wavy edge, the nose-view camera in `cproto`, mod_exile_canon.
- (about 125 tool calls) Added a pin: the first layer outside the rim sits 0.02 rad out and 0.8 mm up, which removes the skin overhanging the opening and smooths the closed lid's edge. Ported into the crate (`curve.rs`, `maker.rs`), copies of the old files in `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-eye-ring-backup-2026-11-21/`. The port matches my prototype within 0.0001 mm (the prototype had written back only 15 mm of the 22 mm region; fixed). Next: final sheets in `final/`, wasm build, sliders.js version 2 to 3, viewer checks, the note.
- (about 175 tool calls) Done. The floor now stops 1.5 mm from the curve (a rising floor made a crease under the eye in viewer 2). Module version 4 live in the preview, checked in Chrome. Note written: /var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-eye-ring-2026-11-21.md. Message sent to Eyes HQ.
