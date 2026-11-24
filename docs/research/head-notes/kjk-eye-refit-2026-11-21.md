# The heads refitted for the eye curve (2026-11-21)

Written by the conversation kjk-cb, started from the brief `eye-refit` by Eyes HQ (kjk-48). My scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/e584df1f-e66d-4e3c-9618-478e0dca7eea/scratchpad`, called `$P` below.

Nothing live changed. No live preset was published, and nothing is committed. Only the opt-in eye curve preview in viewer 2 changed.

## Verdict

**What still looks broken, first.**
- **pmha04 still has a crease between the brow and the upper lid.** It is shallower than in live and in version 4, but it is still a dark crease from the front. In KOTOR, the brow comes straight down onto the lid. The fold guard kept only half of this head's move.
- **mod_hp_pfha01's eye is not KOTOR's shape.** KOTOR's outline of that eye cannot be read: slits in its mesh above the eye send the outline up into the brow. A wider filter did not fix it. The refit eye is a clean almond shape, and version 4's twisted opening is gone. But KOTOR's eye there is larger and rounder, and this head's scores cannot be trusted.
- **Terena Adare's nose** has a sharp spike at its top, between the eyes. The same spike is in live, with the preview off. So it is the nose feature's own fault, not the eye work's, and I left it.
- **A thin pale line** still runs along both lids. It is thinner than in version 4. It is clearest round a closed eye in viewer 2's light, where the edge of the closed lid shows as a light ring.
- **mod_hp_pfhb02** is clean now, but its opening is still 2.95 mm from KOTOR's, which is more than live (2.46 mm).
- **Not checked**: eye paint, glances and the caruncle with the preview on. The four quarters of the overview were checked at a small size only. The close-ups in viewer 2's light cover the 14 test heads.

**What is better**, seen in the pictures:
- **The eye now sits where KOTOR's eye sits in the face.** Version 4 fitted the handles to KOTOR's outline after moving it onto our eyeball's centre. Our eyeball centres sit 1 to 8 mm from KOTOR's, so version 4's openings sat in the wrong place on many heads. On pmha02, for example, the eye sat 3.5 mm too high while the brow, the nose and the ears lined up with KOTOR. The refit measures in the head's own frame.
- **The hollow below the brows on pmha02 is gone.** The brow now comes down onto a narrower opening, as in KOTOR.
- **The mod heads mod_hp_pfha01 and mod_hp_pfhb02 are no longer broken.** Version 4 had twisted, spiral openings on both. Their handle fits had missed KOTOR's outline by 4 to 6 mm. Both now have clean openings that close cleanly.
- **mod_exile_canon** has rounder eyes, close to KOTOR's, open and shut.
- **Terena Adare's eyes** moved from 4.86 mm (version 4) to 0.69 mm from KOTOR's opening.
- **The outer corner's high ends.** Kreia shows no diagonal fold and no spot at the new high ends.
- Over all 57 heads, the median gap between our opening and KOTOR's fell from 2.57 mm (version 4) and 2.01 mm (live) to 1.00 mm. The skin ring's median gap fell from 1.61 mm (version 4) and 1.39 mm (live) to 1.23 mm.

## The sliders dropped and kept

While the preview is on, 23 old eye sliders are set to 0. The ten handles now draw the opening on the eyeball, so these sliders have nothing left to do:
- **Eye width** and **Eye opening height**. The handles set the opening's length and height.
- **Upper lid, inner point**, **Upper lid, middle point**, **Upper lid, outer point**, **Lower lid drop, inner**, **Lower lid drop, middle**, **Lower lid drop, outer** and **Lower lid flatness**. The lid handles set the arches.
- All ten corner sliders: **Inner corner position**, **depth**, **height**, **angle** and **roundness**, and the same five for the outer corner. The corner handles set the tips and their shape. A corner's depth now follows from where the tip sits on the eyeball.
- **Eye tilt** and **Socket angle**. The handles draw the curve in the eye's own frame, and the corner heights tilt the opening.
- **Upper lid cover** and **Caruncle width**. The preview hides the caruncle and uses the curve's own lid piece, so neither does anything.

The live slider set has 29 eye sliders. Outer lid opening, Outer corner reach, round 7's sliding Eye width and the corner carve were never published, so there was nothing to drop for them.

These sliders are kept and solved:
- **The whole-eye sliders**: Eye depth, Eye height, Eye spacing, Eyeball size and Eye prominence.
- **Upper lid fullness**. It shapes the skin above the lid, which the ring keeps from live.
- **The brow and cheek sliders that touch the skin round the eye**: Brow depth, Brow bulge, Brow between the eyes, Cheekbone prominence and Cheek fullness.

Brow edge sharpness was solved in a trial but taken out. The solve pushed it to its high end on pmhc04, away from the brow's own fit.

## The solve

The command is `rfit` in my copy of the shape tool (`$P/diag`, source also in `$P/rfit.rs`). It builds each head the way viewer 2's preview does: the sliders first, then the curve construction.

**The targets.** The head is measured against KOTOR's head in the same frame.
- The curve: 72 points of KOTOR's opening, measured both ways to our opening.
- The skin ring: KOTOR's skin points within 15 mm of its opening, each measured to our outer skin and weighted by area. This ring includes the area below the brows, so a hollow there is scored.
- The corners: the two tips and the opening's height at four distances behind each tip.

The difference from the shape note's measure is this: KOTOR's outline and skin stay where they are in the head. They are not moved onto our eyeball's centre.

**The eye's position.** The eyeball's centre may move anywhere along the line from its live place to KOTOR's centre, give or take 1 mm. Anywhere else costs as much as the curve. Holding the centre at KOTOR's alone would have been wrong, because KOTOR's centres are not where its openings are. pfha05's centre, for example, sits 8 mm behind ours.

**The method.** Levenberg-Marquardt over the 21 values (the 11 sliders and the 10 handles), with a weak pull back to the start, as in the eye solver. It is followed by three passes of a coarse search. In that search, each value tries steps of 0.3 and 0.1 of its spread both ways. The measure is too noisy at the solver's small steps. Without the coarse search, the solve stopped early on Jolee, pmha02 and pmha04 (Jolee's cost fell from 31 to 30.9 only, against 9.9 with it).

**The start.** The dropped sliders are at 0, and the handles are version 4's. On the two mod_hp heads, version 4's handles missed by more than 1 mm, so they start from the median handles.

**The fold guard.** It has two parts, as in the shape note.
- During the solve, a penalty counts triangles near the eye that turn away from the start.
- After the solve, the guard tries 1, 0.75, 0.5 and 0.25 of the move. It keeps the largest share that has at most 8 more counted folds than the start or version 4, whichever has more, and at most 0.5 mm² more bare eyeball when shut, from the front or from the temple side.

The margin of 8 folds is wider than the shape note's. The counted folds rise with the new lid edge even where the pictures are clean (pmha04's full move, `$P/sheets/s4fit_pmha04.png`). 47 heads keep the full move, 4 keep 0.75 of it and 6 keep half.

**The handle ranges.** Outer corner place now ends at 25.8 instead of 26.74, and Outer corner height at 4.6 instead of 5.43. The new ends lie just above every head's fit. The old ends caused the fold and the spot.

A head takes 1.5 to 3 minutes. The 57 heads run in parallel, 16 at a time.

## The faults

| Fault | Result |
|---|---|
| Hollow below the brows on pmha02 | Fixed. The eye sat 3.5 mm too high. It now sits lower, and the brow comes down onto the lid. |
| Hollow below the brows on pmha04 | Better, not fixed. A shallower crease remains. The guard kept half the move. |
| mod_exile_canon | Fixed. Clean, with rounder eyes like KOTOR's. |
| mod_hp_pfha01 and mod_hp_pfhb02 | The twisted openings are fixed. pfha01's eye is not KOTOR's shape, because KOTOR's outline of it cannot be read. |
| Terena Adare's eyes | Fixed. Her eyes are 0.69 mm from KOTOR's opening, down from 4.86 mm. |
| Terena Adare's nose | Not the eye work's fault. The spike between the eyes is in live too. It belongs to the nose feature. |
| Fold from the outer tip at Outer corner height's high end | Fixed by the new end (4.6). Kreia shows no fold there (`$P/ends/outer_p_kreiah.png`). |
| Spot at Outer corner place's high end | Fixed by the new end (25.8). The bright patch beside the outer tip that remains is also in the fitted row, so it comes from the light. |
| Pale edge along the lids | Thinner, not gone. The first layer outside the rim now rises 0.5 mm instead of 0.8 mm and steps out 0.03 radians instead of 0.02 (`PIN_RISE` and `PIN_STEP` in the crate). At 0.45 mm and 0.045 radians, a wobbly line appeared on the upper lid. |

## The scores against KOTOR

All three columns are measured in the head's frame. Each value is the mean of both eyes, in mm. The whole table for 57 heads is in `$P/score_table.md`. The raw results are in `$P/fit/f5.json`.

| Head | Opening gap: live / v4 / refit | Skin ring gap: live / v4 / refit | Share of the move kept |
|---|---|---|---|
| pmhc04 | 1.53 / 2.76 / 1.00 | 1.67 / 1.92 / 1.18 | 1 |
| p_bastilah | 2.01 / 1.33 / 0.67 | 1.14 / 1.27 / 1.06 | 1 |
| pmha01 | 0.96 / 1.08 / 0.49 | 1.41 / 1.73 / 1.34 | 1 |
| pmha02 | 2.77 / 3.79 / 1.08 | 2.05 / 2.41 / 2.01 | 1 |
| pmha04 | 4.22 / 4.83 / 3.20 | 1.39 / 1.48 / 1.02 | 0.5 |
| p_kreiah | 1.89 / 1.71 / 0.65 | 1.23 / 1.33 / 1.25 | 1 |
| p_carthh | 0.98 / 1.07 / 0.78 | 1.02 / 1.16 / 1.04 | 1 |
| n_zherronh | 2.60 / 2.70 / 0.40 | 1.59 / 1.34 / 0.96 | 1 |
| pfha05 | 2.28 / 2.84 / 1.74 | 1.02 / 1.04 / 0.77 | 1 |
| p_joleeh | 3.60 / 3.44 / 1.29 | 1.75 / 2.25 / 1.56 | 1 |
| n_terenaadareh | 4.52 / 4.86 / 0.69 | 2.59 / 2.39 / 0.79 | 1 |
| mod_exile_canon | 1.59 / 3.16 / 1.01 | 2.02 / 2.07 / 1.48 | 1 |
| mod_hp_pfha01 (KOTOR misread) | 6.22 / 7.52 / 4.55 | 2.15 / 2.81 / 2.87 | 1 |
| mod_hp_pfhb02 | 2.46 / 3.40 / 2.95 | 3.15 / 3.71 / 2.40 | 1 |
| **Median of 57 heads** | **2.01 / 2.57 / 1.00** | **1.39 / 1.61 / 1.23** | |

On the opening, the refit is worse than live on 1 head (mod_hp_pfhb02). On the skin ring, it is worse than live by more than 0.05 mm on 8 heads: comm_a_m, comm_w_f, mod_hp_pfha01, mod_improved_bastila, old_a_f, p_handmaidenbb, pfha06 and sith_app. On those 8, the opening improved a lot (for example pfha06, 2.87 to 1.27 mm), and the ring got 0.1 to 0.5 mm worse.

## The sheets

- **Viewer 2's light, shape only, hair off**: `$P/v2/v2_<head>.png` for the 14 test heads. The rows are KOTOR's original, the refit open and the refit closed. The columns are the eyes from the front, three-quarter from the left and from the right, three-quarter from slightly above, and profile.
- **Clay sheets**: `$P/sheets5/s_<head>.png` for all 57 heads. The rows are KOTOR, LIVE, V4, REFIT, REFIT CLOSED, and REFIT LINES (KOTOR's opening in yellow where it is in the head, ours in cyan). The columns are the same five views.
- **The overview of all 57 heads**: `$P/sheets5/overview.png`, and its four quarters `$P/sheets5/ov_0.png` to `ov_3.png`. Per head: KOTOR, V4, REFIT and REFIT CLOSED from the front, then KOTOR, V4 and REFIT three-quarter from above.
- **The outer corner ends** at the new high ends: `$P/ends/e_<head>.png` for Kreia, pmhc04, Bastila and pmha01. `$P/ends/outer_p_kreiah.png` is the crop of the two rows in question.
- **The pale edge trials**: `$P/pale/pale_cmp.png`. The three pairs of rows are the old values (0.8 mm, 0.02 rad), the chosen values (0.5 mm, 0.03 rad) and the rejected values (0.45 mm, 0.045 rad).

## Viewer 2

- URL: http://127.0.0.1:8765/viewer2/?head=pmhc04&eyes=shape&hair=none&eyecurve=1
- The curve module is version **"5"** (`CURVE_MODULE_VER` in `hq/viewer2/sliders.js`).
- With the preview on, the head starts from the refit. Its values are in `hq/wasm/head_eye_curve/data/refit.json`.
  - The ten handles are the refit's.
  - Each slider that the refit dropped or solved moves by the refit's change from the head's solved value. So a slider moved by hand in another tab still moves the head.
  - "Back to the fitted values" returns to the refit's handles.
- With the preview off, the values reach the builder unchanged. I checked this in viewer 2 on pmhc04, mod_exile_canon and pmha02:
  - The preview off equals live within 0.0001 mm.
  - Turning the preview on, then off again, gives exactly the same positions.
  - The preview on equals my tool's refit within 0.0002 mm.
- The Chrome extension was not connected. The checks and pictures came from a headless Chromium driven by Playwright: `$P/vcheck.py` and `$P/shot2.py`.
- All edits are logged in `kjk-slider-kit-shared-changes.md` (18:50 and 19:40).

## How it would go live

1. Port the curve construction into `head_sliders` as a step after the whole-eye sliders. It would replace the press, the corner carve and the lid sliders' fields for the opening. Give the lid piece builder (`head_eyes::piece`) the curve and the piece's axis.
2. Remove the 23 dropped sliders from the eye set. Add the ten handles as sliders, with the ranges in `data/ranges.json`.
3. Publish the refit's values from `refit.json` into the eye, brow and cheek presets (`page.json` of each). The refit changed Brow depth, Brow bulge, Brow between the eyes, Cheekbone prominence and Cheek fullness, so the brow and cheek pages change too. Their own measures should be checked first, because the refit scored only the skin within 15 mm of the eye.
4. Run the eye audit and the fold view against the new build. Their counts are made for the old lid edge.

## How to undo

Copies of everything I replaced are in `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-eye-refit-backup-2026-11-21/`:
- `crate/src` and `crate/Cargo.toml`: copy them back over `crates/character_customisation/head/head_eye_curve/` in the head-paint worktree.
- `wasm/`: copy `head_eye_curve.js`, `head_eye_curve_bg.wasm` and `data/ranges.json` back into `hq/wasm/head_eye_curve/`, and delete `data/refit.json`.
- `viewer2/sliders.js`: copy it back into `hq/viewer2/`. Then set `CURVE_MODULE_VER` to "6", so browsers drop the cached module.

Deleting `refit.json` alone brings back version 4's handles with the live sliders. The module still reads that file, and it carries on without it.

## Traps

- **Do not line KOTOR up on the eyeball's centre for a fit.** Our centres sit 1 to 8 mm from KOTOR's. That measure is useful for comparing shapes, but a solve fitted to it puts the opening in the wrong place in the face. That is what went wrong in version 4.
- **The solver stalls on this measure.** The curve's march and the nearest-point matching are noisy at small steps. Keep the coarse search (`RF_PATTERN`, 3 passes by default).
- **The counted folds rise with the new lid edge** even where the pictures are clean. The guard's margin is 8 (`RF_FOLDTOL`). Judge by the pictures.
- **My diag tool must be run with `CC_FLOORW=1.5 CC_PIN_RISE=0.5 CC_PIN_STEP=0.03`** to build what viewer 2 shows. Without them it builds the old floor and the old pale edge.
- **KOTOR's outline is misread on mod_hp_pfha01.** mod_narve_canderous reads as nearly closed, and mod_tcs_bastila's upper arch wanders (from the shape note). Do not trust those heads' scores.
- In viewer 2, the camera buttons are "Eyes", "Front" and "Side". A drag on the canvas turns the view. `[data-lid=closed]` shuts the preview's lids.
- In zsh, use `${=list}` to split a list of heads.

## Commands

- Build: `PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH" cargo build --release --offline` in `$P/diag`.
- Solve: `CC_FLOORW=1.5 CC_PIN_RISE=0.5 CC_PIN_STEP=0.03 RF_HCAP="Outer corner place=10.48:25.8;Outer corner height=-5.6:4.6" $P/diag/target/release/pmhc_diag rfit ~/.claude/tools/likeness-fit-data/sliders <out.json> all`.
  - Options: `RF_FREE`, `RF_ZERO`, `RF_W` (curve, ring, corners), `RF_TOL`, `RF_POS`, `RF_FOLD`, `RF_FOLDTOL`, `RF_PRIOR`, `RF_PATTERN`, `RF_ROUNDS`.
  - `RF_CENTRED=1` gives the old centred measure. `RF_HOLD=live` holds the centre near live only.
- Summaries: `python3 $P/table.py <out.json>` and `python3 $P/summ.py <out.json> v`.
- Sheets: `rsheet <data> <prefix> <out.json> <heads>` (`RS_LABEL=fit` for the unguarded fit, `RS_RAKE=1`, `RS_WIDE=1`). Overview: `uv run -q --with pillow python3 -I $P/overview.py <sheet dir> <out.png> <heads>`.
- Export to viewer 2: `python3 $P/export.py <out.json> <refit.json>`.
- Viewer 2 check: `uv run -q --with playwright python3 -I $P/vcheck.py <rdump dir> <heads>`, after `rdump <data> <dir> <out.json> <heads>`.
- Viewer 2 pictures: `uv run -q --with playwright --with pillow python3 -I $P/shot2.py <out dir> <heads>`.
