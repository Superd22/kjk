# The eye cleanup (2026-11-22)

Written by the conversation eye-cleanup (kjk-cd), started from the brief `eye-cleanup` by Eyes HQ (kjk-48). My scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/0bbd08f8-4441-481e-b56f-653a55b22f11/scratchpad`, called `$SP` below.

**In short.** Viewer 2 now has one eye path, the curve. The switches `?eyecurvelive`, `?eyecurve` and `?eyerim` are gone, and so are the tab "Eye curve (prototype)" and the old eye branches in viewer 2. The crates lost the corner carve, the tuck, Outer lid opening, the radial press, the seat and the trial environment variables. The built head is identical, number for number, on all 57 heads, open and closed. Every removed or changed file is in the backup folder `~/.claude/tools/likeness-fit-data/backups/eyes-legacy-2026-11-22/`, with a README that gives the restore steps. Nothing is committed.

## What is left that still looks legacy, and why it stays

- **The press, the lid smoother and the free stretch at the inner corner are not dead.** The live build still runs the press on every head before the curve. `SliderSet::build` calls `eye_curves::press` for any set with sliders of zone "eyes", and the curve set has them. The curve then rebuilds the ring round the eye from the pressed head. Removing the press would change every live head.
- **The eye paste is not dead.** The native `World::load` and the bench use it to make the eye, brow and cheek starts. `starts_all.bin` and the brow and cheek presets were solved on it.
- **The old 23 opening sliders and their making commands stay.** These are `web/eyes/sliders.json`, `page.json`, `made.*`, and the commands make3, make-whole, make-b, make-r4, make-r5, make-pmhc-r1, make-pmhc-r2, marks-pmhc-r4, guard-pmhc-r1 and place-whole. The slider page `/nose/` still shows the old eyes, and the native eye bench and `eye_audit` still work only on the old set. The bench does not run the curve. They can go once `/nose/` moves to the curve and a native curve solve exists.
- **The `/nose/` page is not switched to the live eye.** That is not a small change. `nose.js` has its own eye code: the old modules from `/wasm/head_sliders/` and `/wasm/head_eyes/`, the old slider files, the lid cover and the caruncle. Switching it needs the curve modules, the `*_curve.json` files, `init_curve`, `set_curve` and the pieces turned about their own axis, as viewer 2 does. So the old modules `hq/wasm/head_sliders/` and `hq/wasm/head_eyes/` stay, as do `web/eyes/sliders.json`, `eyes/page.json`, `brow/page.json` and `cheeks/page.json`.
- **head_eyes keeps the old lid piece (`build_eye`).** `/nose/`, the native eye measures and the `lid_piece` tool use it.
- **head_eye_curve is untouched.** Its browser class `CurveMaker` (in `src/web.rs`) and the tool `eye_curve_check` served only the preview, so they are dead in the browser. I did not touch the crate, because lid-sink-fine is working in it. It can go in a later step.
- **The trial variables in `eyes.rs` and in `bin/eye_audit.rs` stay** (EYES_PASTE, EYES_PASTE_MODE, EYES_WEIGHTS, EA_* and others). They are options of the native eye tools, which still work on the old set.
- **The "Caruncle" switch stays, as the brief asks.** Our curve eye has no caruncle, so on our head it does nothing. It still hides KOTOR's caruncle in comparisons.

## The inventory

Each item says where it was, whether it was dead, and what happened to it.

### Query parameters of viewer 2
- `?eyecurvelive=0/1` (`sliders.js`): overrode EYE_CURVE_LIVE for one page. Dead, since the curve is live. **Removed.**
- `?eyecurve=1` (`sliders.js`): the preview of the curve on top of the old eye. Dead. **Removed.**
- `?eyerim=0/1` (`paint_live.js`): overrode EYE_RIM_LIVE. With the switch on it only showed the old eye paint. **Removed.**
- `?eyes=shape` (`viewer2.js`): starts in "Shape only". Live. **Kept.**
- `?noseprofile=1`: not an eye parameter. nose-nostrils removed it at 00:30.
- No other eye parameter exists. There never was an `?eyeversion`. The version numbers live in constants, not in the address.

### Flags and version constants
- `EYE_CURVE_LIVE` (`sliders.js`, true): only selected the old path when false. **Removed.**
- `EYE_CURVE_VER` ("4", `sliders.js`): the cache version of the curve's modules, the `*_curve.json` files and the curve's data. It is still needed. **Kept, in one place, with a comment that says to raise it after a rebuild.**
- `CURVE_MODULE_VER` ("5", `sliders.js`): the preview module's version. **Removed** with the preview.
- `EYE_RIM_LIVE` and `EYE_RIM_VER` (`paint_live.js`): the rim switch and the trial module's version. **Removed.** The eye rim is always on for slider heads.
- `VER`, `EYES_MODULE_VER` and `SLIDERS_MODULE_VER` (`sliders.js`): still part of the live addresses (for example `?v=48c4`). **Kept unchanged.**
- `LID_SHADING_MAX` (`paint_live.js`): live. **Kept.**

### Viewer 2 code
- The tab "Eye curve (prototype)" and the switch of the same name in "Eye parts". **Removed:** `curveTab`, the `data-eye-curve` switch in `index.html` and its hint sentence.
- The preview's own lid piece (`curveEyes`), its kit (`curveKit`, `curveApply`, `curveSet`, `curveFitted`, `curveRefit`) and `curveValues`. **Removed.**
- The old caruncle (`caruncleMesh`), the Caruncle width and Upper lid cover readers (`caruncle()`, `cover()`) and `set_lower`. The live set has neither slider, so these were always 0 or skipped. **Removed.** The lid maker's own default for `lower` is 0, so nothing changed.
- The old lid turn about the head's x axis (`pieceGroup`, `turnEyes`). **Removed.** Every piece now turns about its own axis.
- The module root switch (`/wasm` or `/wasm/curve`) and the page file switch (`*.json` or `*_curve.json`). **Removed.** The live ones are fixed.

### Crate code (worktree head-paint, `crates/character_customisation/head/`)
- `head_sliders/src/eye_curves.rs`:
  - The corner carve (`HANDLES`, `handle_values`, `carve_sizes`, `carve_smooth`, `corner_carve`) ran only for sets with the six corner handle sliders, which no published set has. **Removed**, with its calls in `set.rs` and `solve.rs`.
  - The tuck (`tuck_reach`, `tuck_step`), the step of Outer corner reach. Never published. **Removed.**
  - `Curve::OuterOpen`, `outer_open_fades` and `smooth_open` (Outer lid opening). Never published. **Removed.**
  - The radial press (EYES_PRESS_MODE=radial), the seat (EYES_SEAT), the target "start" (EYES_PRESS_TARGET), the Gaussian smoother (EYES_GAUSS), `Press::corrections` and `Press::seat`. They were reached only through trial variables. **Removed.**
  - Every EYES_* trial variable of the file, 52 of them, among them EYES_SMOOTHER, EYES_SIDE_WIDEN, EYES_OPEN_FADES, EYES_PRESS, EYES_DEEP and EYES_CORNER_SPREAD. Each was native only and now is its default. **Removed.**
  - The press itself, the lid smoother and the free stretch. Live. **Kept.**
- `head_sliders/src/eyes.rs`:
  - The specs of Outer lid opening and Outer corner reach (SPECS 31 to 29), and `Eyes::drawn_step`. **Removed.**
  - The commands make-pmhc-r5, make-pmhc-r7 and refield-width-r7, which made the unpublished sliders. **Removed.**
  - press-diag, the radial press's diagnostic, and refield-r6, the one-time conversion of round 6 that ran on 2026-11-20 and needed EYES_SIDE_WIDEN. **Removed.**
  - The eye paste and the other commands. **Kept** (see above).
- `head_eyes/src/piece.rs`: `Settings.tip_axis`, `tip_axis_centred`, `TIP_END_RHO` and EYES_TIP_AXIS (off by default, superseded by the curve's own axis), and EYES_CURVE_OVERLAP (now the constant 0.4). **Removed.**
- `head_eye_curve`: **untouched**, as explained above.

### Served data and modules
- `hq/wasm/eyerim/`: the trial paint module, the old module g78 and the old `paint_live.js`. Nothing loads them. **Removed.**
- `hq/wasm/head_eye_curve/`: the preview module and `data/handles.json`, `ranges.json` and `refit.json` were read only by the preview. **Removed.** `data/group.bin`, `first.bin` and `hidden.bin` are read by the live head (`init_curve`). **Kept.**
- `/wasm/curve/`: the live modules. **Kept.**
- `web/eyes/sliders_uncapped.json`: nothing reads it. **Removed.**
- `web/brow/page_curve_before_a4.json`: the undo copy of the eye polish of 2026-11-21. **Removed** (it is in the backup).
- `web/eyes/page.json` against `page_curve.json`: `page.json` is the old set's presets, still read by `/nose/`. **Kept.**
- `lid_edges.json`: still read by viewer 2 (`eyeKit`, the eyeball centres) and by `/nose/`. **Kept.**
- `web/eyes/starts.bin`: read by the fold view's "Eye sliders only" mode. **Kept.**
- `kotor_eye_tris.json`: read by "Eye parts". **Kept.**

### Other pages
- `/nose/`: uses the old eyes. **Not switched** (see above).

## The query parameters that remain in viewer 2
`head`, `shape`, `shapes`, `eyes=shape`, `hair`, `hairpaint`, `skin`, `layers`, `beardgrid`, `hairgroups` and `hairpreset`. Only `eyes=shape` concerns the eyes.

## The backup and the restore steps
The folder is `~/.claude/tools/likeness-fit-data/backups/eyes-legacy-2026-11-22/`. Its `README.md` lists every file and how to put it back. In short:
1. Copy `viewer2/*` back into `hq/viewer2/`.
2. Copy `wasm/eyerim` and `wasm/head_eye_curve` back into `hq/wasm/`.
3. Copy `data/eyes/*` and `data/brow/*` back into `web/eyes/` and `web/brow/`.
4. Copy the five changed crate files back: `eye_curves.rs`, `eyes.rs`, `set.rs`, `solve.rs` and `head_eyes/src/piece.rs`.
5. Run `hq/viewer2/restage.sh`.

Other conversations have changed viewer 2's files and the crates since (nose-nostrils changed `sliders.js` at 00:30), so compare before you copy whole files.

## The proof that nothing changed

**Viewer 2, before and after the edits.** `$SP/proof/snap.py` loads viewer 2 at its plain address in headless Chrome and builds all 57 heads at their presets. For each head it hashes the built skin, and the eyes made from it open and closed. The eyes include the eyeballs, the lid pieces, the lash strips with their placement and the hidden skin faces. It also takes screenshots of 11 heads with paint on, open and closed: pmhc04, Bastila, Kreia, pmha04, Carth, Zherron, pfha05, Jolee, mod_hp_pfha01, pmha01 and Bao-Dur.
- I ran it twice before the edits (`$SP/proof/before1`, `before2`) and once after (`after1`).
- The geometry of all 57 heads is identical, open and closed, before and after. The two runs before were identical too.
- The 22 screenshots are identical, pixel for pixel, in the head view. The only change is the bottom of the side panel, where the Eye parts hint lost its sentence about the prototype. One shot (Jolee open) changed its framing between the two runs before. It comes from load timing. Three new shots of it equal one of the runs before exactly.
- The console errors are the same 42 before and after: the known 404s for `data/body/<head>.json` and `paint/live/groups/<head>.json`.

**The crate edits.** No module was rebuilt for the live page, so the crate edits change nothing the browser loads today. To prove that a later rebuild also changes nothing:
- The crates as they were before my edits build `head_sliders` and `head_eyes` byte for byte into the live modules in `hq/wasm/curve/`.
- The crates after my edits build modules with the same exported functions. `$SP/proof/modcmp.py` builds every head with the live modules and with the new ones, at its presets and at 3 sets of random slider values (half the sliders anywhere in their ranges). That makes 228 builds. The skin, the curve's arches, the lid pieces, the lash strips, their colour sources and shading, and the hidden faces are identical in every build. The largest difference in the skin is 0.
- The native crates compile and their tests pass (in the copy `$SP/ws`, with nose_base left out, see the notes).

**A browser check by a short sub-agent** (`$SP/check/`):
- The plain address shows the head.
- "Eye parts" has Shape only, Show all, Lid pieces, Caruncle and Eye paint, and no "Eye curve (prototype)". "Shape only" and `?eyes=shape` give the clay head with plain eyeballs.
- The tabs have no "Eye curve (prototype)". The Eyes tab has its 16 sliders.
- Moving "Upper lid height" rebuilds the eye with no error. "Closed" shuts the lids.
- The old parameters `?eyecurvelive=0`, `?eyecurve=1` and `?eyerim=0` now do nothing.
- No new console error appeared.

**The blink.** I took 40 frames of the Talk clip, cropped to the eyes: `$SP/proof/blink_montage.png`. The lids close fully and open again in several frames.

## Notes for later
- `/nose/` is the last user of the old eye. Once it moves to the curve, these can go: the old modules `hq/wasm/head_sliders/` and `head_eyes/`, the old eye files, and most of the old eye commands in `eyes.rs`.
- head_eye_curve's `CurveMaker` (`src/web.rs`) and the tool `eye_curve_check` can go once lid-sink-fine is done with the crate.
- The worktree's head_sliders does not build right now, because of nose-nostrils' new `nose_base.rs` (`Grid` lacks `Debug`). I checked my edits in a copy of the crates with that module left out (`$SP/ws/sync.sh`).
