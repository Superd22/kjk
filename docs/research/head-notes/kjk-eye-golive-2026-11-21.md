# The eye curve, ready to go live (2026-11-21)

Written by the conversation kjk-60, started from the brief `eye-golive` by Eyes HQ (kjk-48). My scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/512af72c-ee70-4214-b51f-252490db0619/scratchpad`, called `$SP` below.

**It is live.** Eyes HQ said go after David approved, and at about 22:00 I set `EYE_CURVE_LIVE` to true in viewer 2's `sliders.js` and ran `restage.sh`. Viewer 2 now builds every head's eye opening on the curve, at its plain address, for example http://127.0.0.1:8765/viewer2/?head=pmhc04. `?eyecurvelive=0` shows the old eyes for one page. Nothing is committed.

**Undo:** set `EYE_CURVE_LIVE` back to false in `~/.claude/tools/likeness-fit/hq/viewer2/sliders.js` and run `~/.claude/tools/likeness-fit/hq/viewer2/restage.sh`.

**The check after the switch.** At the plain address, on pmhc04, Bastila, Kreia and pmha04:
- the curve is on, and the Eyes tab has 16 sliders: Eye depth, Eye prominence, Eye height, Eye spacing, Eyeball size, Upper lid fullness and the ten handles;
- the lid closes and opens again;
- the only console error is a 404 for `data/body/<head>.json`, which the page has with the switch off too;
- the pictures, open and closed, in shape only and with paint, are in `$SP/live/shape_<head>.png` and `paint_<head>.png`.
- a short sub-agent check of those pictures found every closed eye fully covered, and no hole, tear, flap or spike. It noted that the closed lid's upper edge stands a little proud of the skin on Bastila and Kreia, with a faint step at Kreia's outer end. That is the lid piece's look, the later step.

## Verdict

**What still looks wrong, first.**
- **The pale line along the lower lid is thinner, not gone.** I found what draws it, but I could not remove it without making specks. See "The pale line".
- **pmha04 still has a shadow under the brow.** The crease is shallower than in the preview and much shallower than in live, but the brow still overhangs the upper lid more than in KOTOR. The groove between the brows seen from the front is in live too.
- **The closed lid stands out as a smooth plate.** It is 0.8 mm thick in the middle, and the lash strip hangs just below the lower lid when the eye is shut. This is the lid piece's look, which the brief leaves for a later step. It is clearest on pfha05. A small dark lash fragment shows near pfha05's outer corner when closed.
- **Tiny slivers at pmha04's tips.** At the closest close-ups, a sliver of eyeball about 0.3 mm wide can show at pmha04's outer tip when closed.
- **The paint's lid lines are about 1 mm off.** The paint reads each lid's edge from the loop in `lid_edges.json`. With the curve, that loop lies under the lid, about 1 mm outside the visible opening. On Bastila the lower liner then reads as a thick black ring. This is for the paint session (see "The paint check").
- **The closed lid is nearly black with paint on, on pmhc04 and Kreia.** Live pmhc04's closed lid is just as dark. The piece wears the skin of the upper lid, and the paint darkens that skin (the eye-socket tint). On a curve the piece now takes its colour from 2.5 mm above the rim, past the paint's liner, but the lid skin there is still dark. This is for the paint session, or for the lid piece's later look step.
- **Terena Adare's nose spike** is still there. It is in live too, so it is the nose feature's fault.
- **The slider page (`/nose/`, tab "3. Live sliders") is not switched.** It still shows the old eye sliders. Only viewer 2 has the switch.

**What works**, seen in the pictures:
- The real lid piece and lash strip are built on the curve, and they close cleanly on every head. In the Talk clip's blink, 56 of the 57 heads show no bare eyeball at the most shut frame. The 57th, n_mastkavarh, shows none when set to Closed. Its sampled frames missed the peak of the blink.
- Both corners close on the test heads, Kreia's long eye included.
- The new build equals the preview David approved within 0.2 mm (the median head), apart from the changes listed below.
- The nose, the jaw and chin, the brow, the cheeks and the paint layers are built by the same code from the same values. See "The other features".

## What was fixed

### pmha04's crease
The crease is the brow's skin hanging over a recessed upper lid. I moved one slider at a time in viewer 2 (`$SP/sh1/probe1.png`, `probe2.png`). Upper lid fullness and the brow sliders change it most. The refit's guard had kept only half of pmha04's move, and the full move fills the lid out more (Upper lid fullness 4.15 instead of 3.41, Brow depth -1.53 instead of -2.14). But the full move puts Inner corner place at 18.26, the top of its range, which leaves a gap at the inner tip. So pmha04 now has the full move with Inner corner place kept at the guarded 16.68. These values are in `$SP/overrides.json`.

Pictures in viewer 2's light: `$SP/sh1/c.png` compared live, the preview and the new values. The final ones are `$SP/final/shape_pmha04.png` and `paint_pmha04.png`.

### The pale line
The pale line is a short wall of skin. It runs between the lid's rim on the curve and the first layer of skin outside the rim, which the construction pins 0.5 mm higher (`PIN_RISE`). Along the lower lid, that wall faces up into the key light. I proved it by raising that step to 1.5 mm, which made the band huge (`$SP/sh1/` trial pictures).

The lower lid now has its own step: 0.04 radians outward and 0.3 mm up (`PIN_LOWER` in `head_eye_curve/src/curve.rs`). The upper lid keeps 0.03 radians and 0.5 mm. A flatter step (0.05 radians, 0.05 mm) left specks along pmha04's lower lid where the eyeball showed through. The band is now a little thinner than in the preview on pmhc04 and Bastila, but it is still there. What is left is the lit skin rising off the eyeball, which any lid has.

A second try, asked by Eyes HQ, made the lower step wider (0.08 radians, 0.3 mm) to soften the wall's slope. On pmhc04, Bastila and pmha04 it made no clear difference (`$SP/sh1/pl0_<head>.png` against `pl1_<head>.png`), so I kept 0.04 radians and 0.3 mm. I stopped there, at the 20 minutes Eyes HQ gave.

### mod_hp_pfha01, mod_tcs_bastila and mod_narve_canderous
KOTOR's outline of these three heads' eyes cannot be read. On mod_hp_pfha01, slits in the mesh send it into the brow. On mod_narve_canderous it reads as nearly closed, which made the refit's eye a narrow slit. On mod_tcs_bastila the upper arch wanders, and the refit moved its Brow depth from -1.86 to 5.08. Eyes HQ asked for the same treatment for all three.

All three are left out of the curve's targets. Their handles are medians of the refits of readable heads of the same sex. Every other slider (the whole-eye sliders, Upper lid fullness, and the brow and cheek sliders) keeps its live solved value.
- mod_hp_pfha01 and mod_tcs_bastila take the medians of the five readable female heads: pfha02, pfha05, pfha06, pfhb04 and pfhc02. I did not use mod_hp_pfhb02, because one of its eyes is misread. The handles are 11.09, -3.55, 21.81, 0.66, 5.98, 0.39, 5.05, 0.55, -0.20 and 0.16.
- mod_narve_canderous takes the medians of the eight player male heads: pmba01, pmha01, pmha02, pmha04, pmha05, pmha06, pmhc01 and pmhc04. I did not use pmhc03, whose inner corner is misread. The handles are 12.57, -1.48, 19.91, -0.75, 3.31, 0.46, 3.46, 0.55, -0.24 and -0.03.
- The lists are in `$SP/py/gen_curve_data.py` (`UNREADS`).

## The lid piece and the blink

`head_eyes::piece` now builds the real lid piece and lash strip on the curve (`build_eye_on_curve`).
- The piece turns about the line through the eyeball's centre parallel to the line through the curve's two tips.
- Its open edge follows the upper arch, and its closed edge goes 0.4 mm past the lower arch.
- It runs on 3 mm past each tip along the blink axis, at the tip's angle, so it covers the corners when closed. Only the upper arch runs on. The lower arch held at the tips keeps the closing angle sane.
- It thins to 12% of its thickness over the last 30% of its length at each end, so it stays under the skin near the tips.
- The columns from tip to tip, and up to 1 mm past them, set the closing angle.
- The lash strip fades out over the last 1.5 mm inside each tip.
- The piece wears the skin of the upper lid from 2.5 mm above the rim upward, mirrored about the arch (`CURVE_MIRROR_START`). Nearer the rim the paint has its liner and lash band.

Viewer 2 turns each piece about the axis the piece gives (`info.axis`), when closed and in the animations, through the same `turnEyes` code the preview used. The caruncle is off.

Without the curve (the switch off), `build_eye` is unchanged.

**The checks.**
- The eight test heads plus pmha04 and mod_hp_pfha01, open and closed, with close-ups of both corners and the lower lid: `$SP/final/shape_<head>.png` (shape only, lid pieces on) and `paint_<head>.png` (everything on). The rows are KOTOR, ours open and ours closed. The columns are both eyes from the front, three-quarter from the temple, from above, the profile, the inner corner, the outer corner, the lower lid, and the other eye.
- The overview of all 57 heads: `$SP/ov/overview.png`, in quarters `$SP/ov/ov_0.png` to `ov_3.png`. Per head: KOTOR from the front, ours open, ours closed, ours three-quarter.
- The blink: `$SP/blink/blink_tlknorm_<head>.png` for every head, with the open frame and the most shut frame of the Talk clip, and the count of eyeball pixels. The log is `$SP/blink/log.txt`.
- The glances are in the Talk clip. I checked them only for bare eyeball at the blink.

## The paint check

With the paint on, on pmhc04, Bastila, pmha01, Kreia, Carth, Zherron, pfha05, Jolee, pmha04 and mod_hp_pfha01 (`$SP/final/paint_<head>.png`):
- The iris sits in the middle of the opening on every head. The paint centres it halfway between the lid loop's corners and its highest and lowest points. The loop moved outward evenly with the curve, so its middle stayed in place.
- **The lid lines are off.** `head_paint/src/eyes.rs` (`eye_shapes`) reads each lid's edge from the loop in `lid_edges.json`. The curve puts that loop under the lid, about 1 mm outside the visible opening. The visible rim is the third ring of vertices out from the loop. So the lash bands and the liner start about 1 mm from where they should. On Bastila the lower liner reads as a thick black ring under the eye (`$SP/sh1/new_p_bastilah.png` against `live_p_bastilah.png`).
- What the paint session needs: when the curve is on, read the lid's edge from the visible rim. That is the third ring out from the loop, or the curve's arches, which head_sliders now gives as `Builder.curve_info(eye)` (`upper` and `lower`, metres).
- I did not edit the paint code. Eyes HQ is asked to pass this to the paint session.

## The other features

`$SP/py/vcheck2.py` builds all 57 heads in viewer 2 three ways: live, the preview and the new default. The results are in `$SP/vcheck2.json`.
- **New against the preview:** a median of 0.20 mm within 25 mm of the eyes and 0 beyond 40 mm. The differences are the lower lid's step, pmha04 and mod_hp_pfha01. On mod_hp_pfha01 they reach 13 mm, because its brow, cheeks and whole-eye values went back to live.
- **New against live:** the refit changed 5 brow and cheek presets, so the new default moves the face beyond 60 mm from the eyes by 1.26 mm on the median head and by 8.5 mm on mod_tcs_bastila.
- **New with live brow and cheek presets, against live:** at most 0.46 mm between 40 and 60 mm from the eyes, and at most 1.26 mm beyond 60 mm (pfha05). That remainder comes from the whole-eye sliders (Eye spacing, Eye height, Eye depth, Eyeball size, Eye prominence), whose fields reach far. So the nose, jaw, brow and cheek sliders build exactly as before, and every change outside the eyes comes from the 5 preset values the refit solved and the eye's own sliders.
- The nose and jaw files are untouched. The brow and cheek sets are untouched. Only their presets have new copies (`page_curve.json`).
- The paint layers are the same code. With the switch off, every new branch in viewer 2 is skipped (`curveLive`), and the old files and modules load as before. I did not hash the switch-off build against a build from before my edits.

## What changed in which files

All edits are logged in `kjk-slider-kit-shared-changes.md` (20:25 and 21:03). Copies of every file before my edits are in `$SP/orig/`.

**The worktree** (`/Users/david/perso/kjk/.claude/worktrees/head-paint`, crates under `crates/character_customisation/head/`):
- `head_eye_curve/src/maker.rs`: `Lid` keeps the curve's arches (`upper`, `lower`), `Maker::vertex_count`.
- `head_eye_curve/src/web.rs`: `CurveMaker.arch`.
- `head_eye_curve/src/curve.rs`: `PIN_LOWER` and its use for the first layer outside the lower rim.
- `head_sliders/Cargo.toml`: depends on `head_eye_curve`.
- `head_sliders/src/set.rs`: `SliderSet::build` ends with the curve when the set has sliders of kind "curve" (`curve_handles`, `init_curve`, `with_curve_lid`). Sets without them build as before.
- `head_sliders/src/web.rs`: `Builder.init_curve`, `Builder.curve_info`.
- `head_eyes/src/piece.rs`: `Arches`, `arch_chain`, `past_tips`, `build_eye_on_curve`, the curve branches of `plan` and `mesh`, and the constants `CURVE_*`.
- `head_eyes/src/web.rs`: `LidMaker.set_curve`.

**The modules**, built into `~/.claude/tools/likeness-fit/hq/wasm/curve/` only: `head_sliders`, `head_eyes` and `head_eye_curve`. The live modules in `hq/wasm/head_sliders`, `head_eyes` and `head_eye_curve` are untouched.

**The data**, new files beside the live ones in `~/.claude/tools/likeness-fit-data/sliders/web/`:
- `eyes/sliders_curve.json`: the 6 kept eye sliders and the 10 handles (kind "curve", with plain names and hints, groups "lids" and "corners"). The 23 dropped sliders are gone from it.
- `eyes/page_curve.json`, `brow/page_curve.json`, `cheeks/page_curve.json`: the refit's presets for 57 heads, with pmha04, mod_hp_pfha01, mod_tcs_bastila and mod_narve_canderous as above.
- They are written by `$SP/py/gen_curve_data.py` from the live pages, `refit.json` and `$SP/overrides.json`.

**Viewer 2** (`~/.claude/tools/likeness-fit/hq/viewer2/`):
- `sliders.js`: the switch `EYE_CURVE_LIVE` (false) and `EYE_CURVE_VER` ("3"). With it on: the modules from `/wasm/curve/`, the `*_curve.json` files, `init_curve`, the lid pieces put on the curve and turned about their own axis, no caruncle, curve handles shown over their own range, and the old preview tab hidden. Also a getter `HeadSliders.orbit` for scripted pictures.
- `viewer2.js`: a slider's rest value is `meta[k].zero`, which is 0 for every old slider and the middle of the range for a curve handle. "Set to zero" and heads without presets use it.

**The ten handles in the Eyes tab**, with their hints:
- Inner corner place, Inner corner height, Outer corner place, Outer corner height, Inner corner shape and Outer corner shape, in the group "Corners".
- Upper lid height, Upper lid peak place, Lower lid depth and Lower lid low point place, in the group "Lids".
- They replace Eye width, Eye opening height, the six lid points, Lower lid flatness, the ten corner sliders, Eye tilt, Socket angle, Upper lid cover and Caruncle width.
- The Eyes tab keeps Eye depth, Eye prominence, Eye height, Eye spacing, Eyeball size and Upper lid fullness.

## The switch and the undo

**To switch it live** (done at about 22:00):
1. In `~/.claude/tools/likeness-fit/hq/viewer2/sliders.js`, set `const EYE_CURVE_LIVE = false;` to `true`.
2. Run `~/.claude/tools/likeness-fit/hq/viewer2/restage.sh`, so that browsers load the new `sliders.js` (the page asks for it with a build number).
3. Open http://127.0.0.1:8765/viewer2/?head=pmhc04 and check that the Eyes tab shows the ten handles.

**To undo:**
1. Set `EYE_CURVE_LIVE` back to `false`.
2. Run `restage.sh` again.

Nothing else needs to change. The old modules, slider files and presets were never touched. `?eyecurvelive=0` or `=1` overrides the switch for one page, either way.

**To remove the work completely:** delete `hq/wasm/curve/` and the four `*_curve.json` files, copy `$SP/orig/viewer2/sliders.js` and `viewer2.js` back, and copy the three crates from `$SP/orig/crates/` back into the worktree.

## The traps

- **The worktree's `head_sliders` does not build for the browser right now.** Another session's new `head_sliders/src/under_jaw.rs` (since 20:41) uses native-only modules. I built the browser module from a snapshot of the head crates with that file left out of the browser build: `$SP/build_sliders.sh`. Its target folder is `target-golive` in the worktree. `head_eyes` builds normally with `build_wasm.sh head_eyes <out dir>`.
- **The modules in `hq/wasm/curve/` are loaded with version `48c3` (head_sliders) and `22c3` (head_eyes).** Raise `EYE_CURVE_VER` after any rebuild, or browsers keep the old module.
- **The curve sliders' value 0 is not a rest value.** A handle at 0 is no eye at all. Use `meta[k].zero`. The Rust side holds each handle within its range.
- **The paint's lid edge is the loop of `lid_edges.json`, which is under the lid with the curve.** Anything that reads the lid edge from that loop is about 1 mm off.
- **The lid piece on a curve is built along the blink axis.** Near the tips of a long eye, the arch runs across the axis, not along it. Anything that runs past the tips must go along the axis at the tip's angle (`past_tips`). An extension along the arch's own direction dropped Kreia's real tip and opened a wedge at her inner corner.
- **Do not extend the lower arch.** Its extension curls back inside the tips and blows up the closing angle.
- **Pictures:** `$SP/py/shot.py <out dir> shape|paint <heads>`, `$SP/py/ov.py <out dir> all`, `$SP/py/blink.py <out dir> tlknorm <heads>`, `$SP/py/probe.py <out.png> <head> "<Slider=value;...>"` (with `PAINT=1`, `CLOSED=1` and `Q=` for live). `$SP/py/cols.py <head>` prints the lid piece's columns. KOTOR's own hair still shows in its side views, because `hair=none` hides only our hair.
- **Bench solves do not run the curve.** `SliderSet::build` runs it once `init_curve` is called, which only the browser's `Builder` does. The solver's `Evaluator` and `World::load` do not, so a future preset solve of the eye sliders would need that.

## For the paint session

I sent this to Beard HQ on 2026-11-21. Beard HQ does not own the eye paint and told David about it. Eyes HQ is asked to start a conversation for the eye paint.

- With the eye curve on (`?eyecurvelive=1`, later the default), the loop of 28 vertices in `lid_edges.json` is no longer the visible lid edge. The curve lays that loop under the lid, about 1 mm outside the visible opening. The visible rim is the third ring of vertices out from the loop.
- `head_paint/src/eyes.rs` (`eye_shapes`, `EyeLines`) reads the lids' edges from that loop. So the lash bands and the liner start about 1 mm from the visible edge. On Bastila the lower liner reads as a thick black ring under the eye (`$SP/sh1/new_p_bastilah.png` against `live_p_bastilah.png`).
- The iris centre is still right, because the loop moved outward evenly.
- What it needs: when the curve is on, read the lid's edge from the visible rim. That is the third ring out from the loop, or the curve's arches, which head_sliders' browser `Builder.curve_info(eye)` returns (`upper` and `lower`, points in metres, from the inner tip to the outer one).
- A smaller point: the upper lid's skin carries the dark eye-socket tint, so the closed lid piece, which wears that skin, is nearly black on pmhc04 and Kreia. Live pmhc04 is the same.
