# The eye paint on the eye curve (2026-11-21)

Written by the conversation kjk-7a, started by Eyes HQ (kjk-48) from the brief eye-paint. My scratch folder is /private/tmp/claude-501/-Users-david-perso-kjk/55e67acb-94c4-4cfa-98ee-cf774da9f2c8/scratchpad.

**It is live** in viewer 2 at its plain address. GPU_MODULE is g79. `?eyerim=0` shows the old eye paint for one page.

## What still looks wrong, first

- The closed lids of Kreia and Bao-Dur are lighter than KOTOR's lid strips. Those strips are mostly KOTOR's black liner, so they are not a fair target for the colour of a closed lid. I could not see KOTOR's own closed lids, because the Talk clip moves the head out of frame and my blink capture found no closed frame.
- pfhb04, Jaesa and sith_app close to a grey or black lid. They wear their own eye shadow on the lid. That is the same as before.
- pmhc04's lower lash line is now a thin dark line at the visible edge. KOTOR shows a reddish lower rim there. This comes from the preset (lower lash 0.82, red rims 0), which was fitted while the paint read the old loop. A refit of the eye paint presets on the rim is the next step.
- A hard vertical seam in the shading near the temple shows on pmha04 and pmhc04. It shows the same in live, so this change did not cause it.

## What is fixed

- The eye paint reads each lid's edge from the curve's rim, the visible edge of the opening. That covers the lash lines, the liner and its wing, the lid rims, the crease, the socket, the eye shadow and the iris placement. On Bastila the thick black ring under the eye is gone, and the liner hugs the edge. On Kreia the black band under the eye is thinner.
- The closed lid is no longer near black. The lid piece wears the skin just above the rim, deep in the fold under the brow. A strong shading layer (strength up to 2.7 on Kreia) painted that skin near black. The lid pieces' paint now takes the shading at no more than strength 1, the shared KOTOR pattern. Rendered in viewer 2's light, pmhc04's closed lid went from (0.11, 0.06, 0.05) to (0.18, 0.12, 0.10). KOTOR's lid strip is (0.21, 0.12, 0.10). pmha04 stays at 0.45, against KOTOR's 0.44.

## The checks

- Sheets, with rows KOTOR, live open, new open, live closed and new closed. The columns are both eyes from the front, three-quarter from each temple, a close front view and the lower lid. They cover pmhc04, Bastila, Kreia, pmha04, Bao-Dur, pfhb04, Carth and Meetra: /private/tmp/claude-501/-Users-david-perso-kjk/55e67acb-94c4-4cfa-98ee-cf774da9f2c8/scratchpad/r2/sheet_<head>.png.
- The overview of all 57 heads, with columns KOTOR, live open, new open, live closed and new closed: /private/tmp/claude-501/-Users-david-perso-kjk/55e67acb-94c4-4cfa-98ee-cf774da9f2c8/scratchpad/ov/ov_p0.png to ov_p2.png, cut into pages q_p*_*.png. No head looks worse.
- The tests of head_paint's eyes pass, with one new test for the rim.

## What changed

- head_paint eyes.rs: RIMS (the rim's vertices, from the curve's own topology), LidEdge, eye_shapes_on. eyeball.rs: iris_centres_on. The old functions are unchanged.
- head_paint_gpu: PreparedHead.eye_edge, painter set_eye_edge, the eye layer reads the head's edge, and the web GpuPainter.set_eye_rim.
- viewer2 paint_live.js: EYE_RIM_LIVE, ?eyerim, set_eye_rim for slider heads while the curve is live, and LID_SHADING_MAX for the lid pieces' paint.
- The paint page (../paint/) never calls set_eye_rim, so it paints exactly as before.

## Undo

- Quick: set EYE_RIM_LIVE to false in hq/viewer2/paint_live.js and run hq/viewer2/restage.sh. With the rim off, the new module paints exactly as g78 did.
- Full: copy hq/wasm/eyerim/old_g78/head_paint_gpu* back into hq/wasm/head_paint_gpu/, set GPU_MODULE back to g78 (old_g78/gpu_version.js), copy old_viewer2/paint_live.js back, and restage. The originals of the crate files are in /private/tmp/claude-501/-Users-david-perso-kjk/55e67acb-94c4-4cfa-98ee-cf774da9f2c8/scratchpad/orig/.

## Traps

- The rim's vertex lists come from rimgen (/private/tmp/claude-501/-Users-david-perso-kjk/55e67acb-94c4-4cfa-98ee-cf774da9f2c8/scratchpad/rimgen), which runs head_eye_curve's topo on the base head. If the curve's roll layer (SIZES.roll, 3) changes, regenerate RIMS.
- If a new build of head_paint_gpu is made, raise GPU_MODULE.
