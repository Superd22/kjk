# Eye curve go-live, handoff (2026-11-21)

Written by the conversation started from the brief `eye-golive` by Eyes HQ (kjk-48).
Brief: /private/tmp/claude-501/-Users-david-perso-kjk/fa94c09e-e48c-4836-b534-90bfa0de7855/scratchpad/brief-eye-golive.md
My scratch folder: /private/tmp/claude-501/-Users-david-perso-kjk/512af72c-ee70-4214-b51f-252490db0619/scratchpad

## Where I am

- Read the brief and the three notes. Starting to read the code.
- 20:35: switch built (sliders.js EYE_CURVE_LIVE false; ?eyecurvelive=1 to try). Modules in hq/wasm/curve/. Data *_curve.json written by $SP/py/gen_curve_data.py. New build equals the preview within 0.0001 mm (py/vcheck.py). Real lid piece on the curve works (head_eyes build_eye_on_curve), runs 3 mm past the tips, closes at the corners. Lower lid pin step now [0.05 rad, 0.05 mm] (curve.rs PIN_LOWER): the pale line is narrower, not gone. Pictures: py/shot.py <dir> shape|paint <heads>. Next: pmha04 crease, then sheets on 8 heads, paint check, 57 overview.
- 20:58: lid piece on the curve settled (head_eyes piece.rs: upper arch run 3 mm past the tips via past_tips, lower arch not extended, thins over 30% at each end, every column within 1 mm past the tips sets the closing angle, overlap 0.4). PIN_LOWER [0.04, 0.3]. pmha04: full refit move with Inner corner place 16.68 ($SP/overrides.json, read by gen_curve_data.py). head_sliders wasm builds from a snapshot ($SP/build_sliders.sh) because another session's under_jaw.rs breaks the worktree wasm build; target dir worktree/target-golive. vcheck2 on 57 heads done ($SP/vcheck2.json). Sheets of 10 heads running into $SP/final. Next: paint check, overview 57, note, roadmap, message kjk-48.
- 21:03: lid piece fixes for long eyes (Kreia): extension along the axis, CURVE_END_RHO, lash fade. Final runs in background: $SP/final (shot.py shape/paint, 10 heads), $SP/ov (ov.py all), $SP/blink (blink.py tlknorm all). After them: overview montage, paint check (paint session gets a message if needed: the paint reads the loop of lid_edges.json, which with the curve lies under the lid about 1 mm outside the opening), note, roadmap, message kjk-48 and wait.
- 21:35: note written (kjk-eye-golive-2026-11-21.md), roadmap line and decision 67 added. Paint sheets rerunning after the piece colour fix. Next: message kjk-48 with pictures and switch/undo, wait up to 30 min, switch only on "go".
- 21:50: message sent to Eyes HQ (uds:/tmp/cc-socks/17379.sock) asking go or no-go. Waiting up to 30 min. On go: set EYE_CURVE_LIVE true in hq/viewer2/sliders.js, run hq/viewer2/restage.sh, check http://127.0.0.1:8765/viewer2/?head=pmhc04 in a browser (Eyes tab shows the ten handles), report.
- 21:55: Eyes HQ said no go yet. Done its asks: narve and tcs_bastila on same-sex medians (gen_curve_data.py UNREADS), paint request sent to Beard HQ (not the owner; routed back to Eyes HQ, section 'For the paint session' in the note), pale line second try with no gain, kept [0.04, 0.3]. Waiting for go.
- 22:05: LIVE. Switch on, restaged, checked at the plain URL (console: only the old body 404). Sub-agent judging the live pictures. Roadmap updated. Last: final message to Eyes HQ, then stop.
