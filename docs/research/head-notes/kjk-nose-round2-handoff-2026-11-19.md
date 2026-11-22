# Nose sliders, round 2: handoff

This note was written on 2026-11-19 by the agent that built round 1 of the nose sliders and started round 2. It is for a fresh agent that has the round 2 brief and none of that agent's memory. It says what the code does not show quickly. Read the roadmap's "For agents" section and `~/.claude/tools/likeness-fit/WASM-BUILD.md` before you start, because both set rules.

## State at handoff

- The code builds. `cargo build --release -p head_sliders`, its 3 tests, a full run of `nose_bench` and the browser build all work.
- The page at http://127.0.0.1:8765/nose/index.html still shows round 1. Its data in `~/.claude/tools/likeness-fit-data/sliders/web/` and its module in `~/.claude/tools/likeness-fit/hq/wasm/head_sliders/` were made before round 2 started, and nothing has overwritten them.
- The code already holds round 2's first pieces: a revised measure with 12 proportions, `check.rs`, `named.rs`, sliders made of keys, and a bench step that makes the named sliders. A full bench run now changes the page's data to the new measure (see "Traps").
- None of this is committed, as the rules require.

## Map of the files

### The Rust crate `head_sliders`

It is in `/Users/david/perso/kjk/.claude/worktrees/head-paint/crates/character_customisation/head/head_sliders/`. The workspace picks it up through its `crates/character_customisation/*/*` member pattern. Its own `Cargo.toml` adds `wasm-bindgen = "0.2.129"` for the browser target only. The workspace `Cargo.toml` was not changed. `Cargo.lock` changed only to add the crate.

| File | What it does |
|---|---|
| `src/lib.rs` | The module list and small vector helpers (`V3`, `add`, `sub`, `dot`, `cross`, `length`, `normalize`) |
| `src/mesh.rs` | `HeadMesh`: the base head, triangles, the vertices the smoothing keeps, merged vertices (`group`, `first`), and hidden vertices (nostril and mouth insides) with their 6 nearest skin vertices. `expand` spreads a movement given per merged vertex to every vertex and moves hidden vertices with their skin. `input::Input` reads the files `prep_nose.py` wrote (native builds only) |
| `src/zone.rs` | `Zone::nose`: the nose's weight per merged vertex, 1 on the nose and fading to 0 over 7 mm. `own_shape` is a movement minus its harmonic fill from the zone's edge, times the weight. It is what a nose slider may move, and what is taken out of a fit to make a stand-in |
| `src/measure.rs` | The measure (see "How the measure works") |
| `src/set.rs` | `SliderSet` as plain data (serde JSON), building a head (`unsmoothed`, then `build` with the smoothing), and 3 tests: a head is its start plus its values, up and back returns the same head in any order, and the set survives its JSON. Round 2 added `Key`, `Field`, `SliderSet.fields` and `Slider.hint`, `.measure` and `.keys` |
| `src/solve.rs` | `Evaluator`: one head's smoothed start and each slider's smoothed movement, cut to the vertices near the nose, so one measure costs about 1 ms. Round 2 made it handle keyed sliders. `gaps` gives the residual rows: each proportion's gap over its pass mark, then each outline's point gaps. `solve` is Levenberg and Marquardt over all sliders, with values bounded at 5 spreads (`BOUND`) |
| `src/learn.rs` | Round 1 learning (see "Round 1") |
| `src/check.rs` | Round 2. `Checker` counts faults of a built head against the head it started from, near the nose: triangles turned over, folds, and nostril triangles turned over |
| `src/named.rs` | Round 2. The named sliders from smooth local fields (see "Round 2") |
| `src/web.rs` | The browser API. Class `Builder(base, triangles, keep, slidersJson)` has `build(start, values)` and `measure(positions, eyes)`, which returns JSON |
| `src/bin/nose_bench.rs` | The bench: it reads the inputs, runs check 1 of the measure, makes the round 2 sliders, learns round 1, solves all heads, runs checks 3 and 4, the leak table, faults and samples, and writes the report and the page's data |

### Python, light work only

All of it is in `~/.claude/tools/likeness-fit/sliders/`.

| File | What it does |
|---|---|
| `prep_nose.py` | Run once. It writes `likeness-fit-data/sliders/input/`: the base head, triangles, keep mask, merged vertices, hidden vertices and their weights, zones, and landmarks. For each of the 57 heads, in its fit's frame (metres), it writes the stand-in before smoothing (the `fits_v6` fit with its neck rim placed), the smoothed fit, our wrap, KOTOR's own face mesh (`kotor_v`, `kotor_f`) and the eye centres. It also writes 18 extra KOTOR meshes that are copies or near copies of a test head (`twins/`), lined up by rigid fitting, and the map from the viewer's 5,852 vertices to our 4,750. A `manifest.json` lists every array with its type and shape |
| `web_export.py` | Run after the bench. It crops KOTOR's textured heads from viewer 2's data to the nose and puts them in the frame of the eyes (`web/kotor/<name>.json`). It writes our texture layout (`web/layout.json`, layout v2 with the neck cut's drawn triangles) and links `web/tex` to viewer 2's textures |
| `answers_server.py` | The small server on port 8766 that saves the page's answers (see "Traps") |
| `build_wasm.sh` | The browser build for any crate. The paint session uses it too, so keep it working as it is |

### Data

All of it is in `~/.claude/tools/likeness-fit-data/sliders/`.
- `input/`: written by `prep_nose.py`.
- `report.json`: the bench's full report (about 35 MB).
- `named.json`: what each round 2 slider does at each key.
- `answers/noticing.json`: David's strip clicks. He saw width at 0.7 mm, length at 1.25 mm and projection at 0.45 mm. He did not answer the bridge-line strip.
- `web/`: what the page reads.
  - `page.json` is the slim report.
  - `sliders.json` is the round 1 set with its ranges, and `sliders_r2.json` is the round 2 set.
  - `base.bin`, `triangles.bin`, `keep.bin` and `starts.bin` (57 stand-ins) are 32-bit floats and integers.
  - `made.bin` and `made.json` hold the made noses' positions near the nose, per head, mode and count.
  - `heads.json`, `layout.json`, `kotor/` and `tex` (a link).

### The page

- The sources are `~/.claude/tools/likeness-fit/hq/nose/index.html` and `nose.js`.
- The shared loader is `~/.claude/tools/likeness-fit/hq/wasm/load.js`, and the module is in `hq/wasm/head_sliders/`.
- The link `hq/nose/data` points to `likeness-fit-data/sliders/web`.
- The served folder `viewer-local` (in the first session's scratchpad) has the links `nose` and `wasm`. `WASM-BUILD.md` says how to recreate them.
- `nose.js` has a version string (`VER = "?b=11"`) and imports `/wasm/load.js?v=11`. Bump both after rebuilding data or the module, or the browser keeps old copies.
- The page has four tabs: the wall (with the checks below it), the noticing strip, live sliders, and made against original (with the samples and the leak table below it).

## How to run the whole chain

```
# once, or when the fits or wraps change
cd ~/.claude/tools/likeness-fit && .venv/bin/python -I sliders/prep_nose.py

# the bench (about 40 s on a quiet machine, up to 3 minutes when other sessions load it)
cd /Users/david/perso/kjk/.claude/worktrees/head-paint
export PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH"
cargo build --release -p head_sliders --bin nose_bench && ./target/release/nose_bench
#   NOSE_CHECK1_ONLY=1   prints check 1 of the measure against KOTOR's spread and stops (a few seconds)
#   NOSE_MAKE_ONLY=1     makes the round 2 sliders, writes web/sliders_r2.json and named.json, and stops (about 12 s)
#   a folder argument    runs on another data folder, which must hold input/ (it writes report.json and web/ there)

# the page's KOTOR crops and layout, after the bench
cd ~/.claude/tools/likeness-fit && .venv/bin/python -I sliders/web_export.py

# the module for the browser
~/.claude/tools/likeness-fit/sliders/build_wasm.sh head_sliders

# then bump VER and the load.js version in hq/nose/nose.js
```

## How the measure works

Everything is in `src/measure.rs`.

- **The frame.** The origin is midway between the eye centres. x runs along the eye line, y is up, and z points out of the face. Lengths are scaled so the eyes are 63 mm apart. Our heads and the KOTOR heads they copy share their eye centres: the fits' `eyes`, which are where our eyeballs sit in KOTOR's slit. So the frame is identical on both sides of a comparison. KOTOR's mesh is put in the fit's frame by the wrap's similarity and the fit's `R` and `t` (in `prep_nose.py`). That is a frame, not a score of the fit.
- **Cuts.** The measure cuts the surface with planes. A cut is read as the front-most depth every 0.25 mm and blurred lightly. The midline cut gives the profile, and level cuts at a height y give z(x). `Surface` bins triangles by 1 mm of height, so a level cut only reads its bin.
- **Landmarks.**
  - The tip is the front-most profile point. Its height is the weighted middle of the profile within 1 mm of the front, because a round tip's maximum slides.
  - The base is the subnasale. Going down from the tip, the profile's lowest point before it rises again by 0.5 mm, then the front of the upper lip, then the deepest point under the tip-to-lip chord.
  - The wings come from level cuts between the base and 4 mm above the tip. In each cut, the steepest place of each side is taken, if its slope is over 1. The widest pair, blurred over 5 rows, marks the wings.
- **Proportions in round 1 (8).** These are the width at the wings, the bridge width (4 mm behind the front at 35% of the way to the tip, averaged over 3 heights), the tip width, the length (eyes to base), the projection (tip minus the cheek 3 mm beside the wing), the root (profile depth at the eye line), the tip turn (the angle from the base to the tip), and the bridge line (the mean distance in front of the chord from a quarter of the way down to the tip).
- **Round 2 changes, already in the code (12 proportions).**
  - The tip width is now the width of the lower nose seen from below, 5 mm behind its front.
  - Four measures were added. The flare is half of wing width minus tip width. The bridge height is the profile at mid-bridge minus the face 16 mm to each side. The tip roundness from the side and from below are each the radius of a circle fitted to the part within 5 mm of the front.
  - The "below" envelope is the front-most z over the level cuts from the base to 2 mm above the tip.
  - Wing height was tried and dropped, because its wobble was larger than KOTOR's spread. The flare is redundant: it equals (wing width − tip width) / 2. Remove it, or never hold it while moving the widths.
  - The code still computes an unused wing-height centroid. Remove that too.
- **Outlines (3).** Each is compared after its position and size are taken out (`outline_gaps(o, ...)`).
  - The side profile runs from 4 mm above the eye line to the base, 40 points by arc length.
  - The front outline is half widths 4 mm behind the front at 20 heights, from 15% of the way to the tip down to 3 mm above the tip. It is compared as widths over their mean.
  - The base from below is the "below" envelope across the wings, 31 points.
- **Pass marks.** These are provisional: a quarter of KOTOR's spread, or twice the 95th percentile of check 1's subdivided wobble if that is larger.
- **What is fragile.** These are the numbers from the last check 1, as the median and the 95th percentile against KOTOR's spread.
  - Sturdy: the width at the wings (0.6 and 1.7 against 5.0 mm), the length (0.25 and 1.0 against 4.5 mm), the root (0.9 against 4.9 mm), the bridge height (1.1 against 3.8 mm), the bridge line (0.4 against 0.57 mm) and the profile outline (0.54 against 1.68 mm).
  - Weak: the tip turn (3.1 against 5.3 degrees), the tip width (1.7 against 3.1 mm), the roundness from the side (1.2 against 2.05 mm) and from below (2.0 against 3.1 mm), and the front outline (0.6 against about 1.0 mm).
  - Tip shape is ambiguous on KOTOR's meshes of about 400 vertices. Our wrap's 95th percentile on the wing width is 3.6 mm, because the wrap cuts the alar crease.
  - The measure's 0.25 mm sampling quantizes the length and the wing row.
- **Check 3.** Copies whose nose lies within 1 mm of a test head pass all 11 round 1 measures. Copies whose nose BioWare moved by 4 to 6 mm mostly fail some.

## Round 1: learning, solving and ranges

- **Stand-ins.** Each head's stand-in is its fit before smoothing, minus the fit's own nose shape (`zone.own_shape`). A made head is the stand-in plus the sliders' movements, then the build smoothing (implicit, strength 3, ramp 4 mm, from `head_shape`). The smoothing is linear, so the `Evaluator` precomputes smoothed movements.
- **Learning** (`learn.rs`). The examples are our wraps' nose shapes (`own_shape` of the wrap) for the 45 game heads, with their KOTOR-measured proportions.
  - The 4 proportion sliders are least-squares regressions of the shapes on 4 centred proportions (wing width, length, projection, bridge width). Each is scaled to 1 mm of its proportion on the base head by measuring at ±2. Because they hold only the other 3 named proportions, they carry KOTOR's correlations. This is the cause of David's complaint that "wide goes with flat".
  - The shape sliders are the leading singular vectors of what is left, without centring, in spreads.
  - Held out means one set per game head, learned without it. Mods are always held out.
- **Solving.** `solve` runs from two starts, each proportion's own gap and zero, and keeps the better. It takes 0.23 to 0.52 s per head on one thread.
- **Ranges.** A slider's range is the lowest and highest value over the 57 solved noses, with 12 sliders: learned values for game heads, held out for mods. No check is made of how the nose looks at the ends.
- **Round 1 results** (old measure, provisional marks). Held out, the noses made were 11, 43, 44, 54 and 54 of 57 at 4, 6, 8, 10 and 12 sliders.
  - 45 of the 57 solved noses had at least one fault.
  - 35 of 64 noses sampled like KOTOR had one, and 60 of 64 drawn anywhere in the ranges.
  - The faults are where the wings meet the cheek.
  - With the new measure, a bench run in a scratch folder gave 51 of 57 held out at 12 sliders.

## Round 2: what was started

### The design: named sliders from smooth local fields (`named.rs`)

- **Bumps.**
  - Bump centres sit on the base head's nose, 3.5 mm apart, on the right side and the midline (zone weight over 0.6, skin only, z over 0).
  - Each centre gives 3 fields: across, up and out. The across field is skipped on the midline.
  - A field is a 4 mm Gaussian plus its mirror. For the across direction the mirror moves the opposite way, so every field is symmetric.
  - Each field is multiplied by the zone weight and spread to hidden vertices with `expand`.
  - There are 305 fields.
- **Parts.** Each bump belongs to a part of the base nose (`part_of`).
  - The root is above 20% of the tip's height. The bridge runs from there down to 8 mm above the tip.
  - Below that, the wings are more than 9 mm from the middle. The tip is the front within 10 mm of the tip's depth, and the base is the rest.
  - A slider only uses the bumps of its parts (`Spec.parts`).
- **Bending.** `bending` is the Gram matrix of the fields' uniform Laplacians in mm, normalized to a mean diagonal of 1.
- **Each slider** (`Maker::make`). Starting at 0, it steps by half a KOTOR spread toward each end. At each step it does the following:
  1. It measures the response of all residual rows to each allowed bump, by central differences of 0.3 mm. That is about 0.25 s.
  2. It solves the smoothest mix change that reaches the asked value of its own measure. This is an equality constraint solved in closed form with a Lagrange multiplier. The other proportions are held with weight 30 and the outlines with weight 6, in pass-mark units, and the drift of the held rows is pulled back.
  3. It builds the head and counts faults with `Checker`. If a step makes a fault, it retries up to 3 times with the bending 4 times stiffer, then stops that side.
- **Keys.** Each accepted step becomes a key whose value is the measured change, not the asked one, so the slider's unit is exactly its measure. The slider blends between keys, so it may bend along its range. Past the last key it stays at the last key's movement. A head is still a pure function of the values: the same values give the same head, in any order of edits.
- **The 11 specs.** They are in `SPECS`, each with a name and a hint: wing width, bridge width, tip width, nose length, tip projection, root height, tip turn, bridge line, bridge height, tip roundness and tip shape from below. Their `free` lists are empty, so every slider holds every other measure.

### Done

- The measure's new proportions.
- `check.rs`.
- Keyed sliders through `set.rs` and the `Evaluator`. The browser `Builder` reads keyed sets too, because they are the same `SliderSet`.
- `make_named` in the bench: it makes all 11 sliders in about 12 s and writes `web/sliders_r2.json` and `named.json`.

### The first results, not yet looked at

These figures are from `NOSE_MAKE_ONLY=1`. Each gives the range reached, against the span asked from the KOTOR noses.
- Root height: −9.1 to 14.1 mm, against −9.1 to 15.7. It stopped at the top on 3 turned-over triangles.
- Tip turn: −12.7 to 11.2 degrees, against −14.9 to 12.6.
- Bridge height: −12.3 to 6.3 mm, against −12.4 to 6.4.
- Bridge width: −3.3 to 9.7 mm, against −3.3 to 9.7.
- Tip projection: −11.1 to 10.6 mm. It stopped at the low end on a fold.
- Bridge line: −2.3 to 1.1 mm.
- Wing width: −6.6 to 13.3 mm, against −9.7 to 19.2. It stopped growing at the low end, and a fold stopped it at the high end.
- Nose length: −5.3 to 8.8 mm, against −6.0 to 29.2. Faults stopped it at the high end. The 29 mm is mod_hp_pfha01's very long nose.
- Tip width: −6.1 to 8.1 mm.
- Tip roundness: only −2.6 to 0.6 mm, against −4.1 to 7.8. The measure stopped growing.
- Tip shape from below: only −0.9 to 1.7 mm, against −6.6 to 9.5. The measure stopped growing.

The roundness sliders barely work. Either their measures are too noisy to steer, or holding every other measure leaves them no room. Try freeing the tip width for them, or define roundness differently. A step that ends very close to its span is reported as "the measure stopped growing"; that message is wrong in that case.

### Half done, and what I meant to do next

- **The bench's solve loop does not use the round 2 set yet.** I meant to add a mode "named" (count 11) to the per-head solving, next to "held_out" and "learned". Its start values would be each KOTOR measure minus the stand-in's. Its ranges would be the solved values clamped to the keys. The report would list the heads whose value sits at a key end, which is the cost of each cap.
- **The rest of the bench still runs on round 1.** I meant to point `finish` at the round 2 set for the leak table, the per-slider faults and the samples. I meant to add a check of every pair of sliders at their four corners for faults. And I meant to put both sets' scorecards side by side.
- **The page knows nothing of round 2 yet.** It needs these changes:
  - a choice between `sliders.json` (round 1) and `sliders_r2.json` in the live view, with one `Builder` per set;
  - slider panels built from the set's own JSON, so they show each slider's name, hint and range;
  - the solved values of mode "named";
  - the `SHORT` chip labels in `nose.js`, which list 11 measures and must become 15 (12 proportions and 3 outlines).
- **The staircase questions of the brief are not started.** The plan was this:
  - Each question shows two noses side by side on the same head, the reference and one moved by the round 2 slider of one measure. That slider changes one measure by a known amount.
  - There are two buttons, "Same nose" and "Different nose". The changed nose goes on a random side, and the change is wider or narrower at random.
  - The change starts at about one KOTOR spread and halves or grows by David's answers, for about 6 questions per measure.
  - The questions run on the clay base head and on a painted KOTOR head. A painted head is our head with that head's stand-in, its solved values and the bake `tex/v2/<name>_ours.jpg`.
  - Answers go to a new file through the 8766 server (`/save/<name>`; names may only use lowercase letters and underscores). The pass mark is the largest change he still calls the same nose.
- **Not started either:** the judging of each slider at both ends (front, side and below in clay), and the roadmap update.

## Traps, and how I got round them

- **cargo is not on the PATH.** Put `~/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first.
- **Warnings that are not errors.** Every native release build warns "stripping debug info with rust-objcopy failed" (libLLVM.dylib). It is harmless.
- **The browser link step needs a library path.** `rust-lld` cannot find `libLLVM.dylib`, and the build only works with `DYLD_FALLBACK_LIBRARY_PATH` set to the toolchain's `lib`. `build_wasm.sh` does it. Its first argument is the crate and its second an optional output folder. Use an output folder in your scratchpad to test a build without replacing the served module.
- **Formatting.** `cargo fmt -p head_sliders` reformats to the repo's width, which breaks text-based edits written for the old layout. Re-read a file after formatting.
- **The 8765 server went down once,** at about 22:20 UTC, and not because of me. I started it again with `cd viewer-local && nohup /usr/bin/python3 -m http.server 8765 --bind 127.0.0.1 &`, and noted this in `WASM-BUILD.md`. Restart it only when it is down.
- **The save server.** `answers_server.py` runs on 8766, started with nohup from `~/.claude/tools/likeness-fit/sliders/`. It allows the origin `http://127.0.0.1:8765`, saves to `answers/<name>.json` with a copy stamped with the time, and serves `/load/<name>`. If it is down, start it the same way. The page then keeps answers in the browser's local storage only.
- **The browser's cache.** It keeps old `nose.js`, `load.js`, the module and the data. Bump the versions, as described under "The page". The third argument of `loadCrate` is the module's version.
- **A frozen module.** A module namespace is frozen. `load.js` returns a copy with `loadMs` added. Do not try to add fields to the module itself.
- **JavaScript strings.** A plain `"` inside a double-quoted JavaScript string broke the whole page once. Check the script's syntax with `node --check` on a copy named `.mjs`.
- **A full bench run changes what the page reads.** It rewrites `web/page.json` and `made.bin` with the new measure's 12 proportions and 15 chips, but `nose.js` still expects 11 chips. Either update the page first, or run the bench on a scratch folder (a folder that holds a link to `input/`).
- **The base landmark's sign.** The chord-distance sign for the base and the bridge line was wrong once, and the tip was taken as the base. Both are fixed. If a head reports "base too close to the tip", look at its midline cut first.
- **The wings.** The widest pair of creases (most concave bend) landed on cheek or lip rows. Steepest sides with a slope over 1 fixed that, which I checked on a sheet of all 57 cuts.
- **Check 1's subdivision.** Loop's subdivision rounds the low-poly nose by 1 to 2 mm, so it cannot be used to test the noise. The interpolating butterfly rule, applied twice, is the noise test. Loop's rule is kept as the harsher "rounded" test.
- **The copies.** Their meshes needed a rigid fit (`icp` in `prep_nose.py`) from the copy's front half onto the whole sibling. Fitting toward the front only went wrong. Some copies have the same mesh but an edited nose (Shaardan, Tanis). Look at the gap near the nose (`nose_gap_mm`), not the median gap.
- **Slow solving.** The loaded machine (load average 40 to 90) made solves 5 times slower. Timings must say the load.
- **Faults.** The fault count against the base head is not a fair yardstick for fits: the earlier fits show 4,565 faults against the base. Count faults against the head's own start.
- **Smoothing the learning fields** at strength 10 halved the faults on solved noses. It did not help the samples and cost 3 made noses, so it is off (`NOSE_FIELD_SMOOTHING`).

## What I would do differently

- I would make the sliders named and local from the start, as round 2 asks, and use KOTOR data only for the list of measures and the ranges. Learned directions powered the scorecard but read badly to a person.
- I would check each slider at both ends visually before solving anything.
- I would define the tip's shape measures on a denser, rounded copy of KOTOR's mesh, or accept that KOTOR's ~400 vertices cannot carry tip shape, and say so instead of measuring it.
- I would keep the page generic over the number of measures, so a measure change does not break the chips.
- I would write the bench as subcommands rather than one long `main` with environment switches.

## Suggested skills

- `claude-in-chrome`, to look at the page. Open a new tab, do not take over David's tabs, and close the tab when done.
- `mattpocock-skills:diagnosing-bugs`, if a measure misreads a head.
- `simplify`, on `nose_bench.rs`, which grew long.
- `artifact-design` is not needed. The page is local and served from this Mac, not published.

## Where I am (second agent, 2026-11-19, about 23:55)

- **New tool `nose_r2`** (`src/bin/nose_r2.rs`) owns round 2. Commands: `needs` (what each measure needs from each head's stand-in), `make` (makes the sliders into `web/sliders_r2.json` and `named.json`; `NOSE_ONLY=key,key` remakes some into `named_only.json` without touching the set), `sheet` (`NOSE_SHEET=out.png`, each slider at low, 0 and high from the front, the side and below), `bench` (solves the 57 heads held out and with ranges from every head at 4 to 12 sliders, safety at the ends, pairs and 64 samples, leak table, writes `report_r2.json`, `web/page_r2.json`, `web/made_r2.bin`, `web/made_r2.json`; `NOSE_SHEETS=folder` draws the samples). It reads `marks.json`, which `NOSE_MARKS_ONLY=1 nose_bench` writes.
- **New library modules**: `heads.rs` (the 57 heads, their stand-ins and stand-in noses, `parallel`), `render.rs` (clay pictures in Rust, `image` crate, native only), `verdict.rs` (`judge`, moved out of `nose_bench`). `nose_bench` no longer makes round 2 sliders.
- **The measure**: flare is now half of (wing width minus the mean width of the nose's sides above the wing row, up to 4 mm above the tip). The tip's roundness from the side and bluntness from below are now width-free: the cap 2.5 mm deep against 0.707 times the cap 5 mm deep (a parabola scores 0). The base is looked for at most 25 mm below the tip (old_w_m's stand-in had its lip taken for the base). The cheek beside the wing is now read at half the blurred wing width plus 3 mm. Check 1: flare 0.17/0.61 against a spread of 0.90; the two tip shapes are still as noisy as KOTOR's spread (KOTOR's meshes cannot carry tip shape), so their provisional marks are lenient.
- **The reach fix**: the spans asked are now what each head needs from its own stand-in (`needs`), not from the base head. That alone removed most of the "too short" ranges (the 29 mm length was against the base head; against its stand-in the mod needs 9.8 mm). The maker also holds outlines with each axis's size taken out (`outline_shape_gaps`), uses a 1 mm finite difference (0.3 mm saw the measures' 0.25 mm steps), pulls back only 30% of the held rows' drift per step, halves the step before giving up, and has repair levels (loosened holds, then extra parts), recorded per key as `level`. Every slider now reaches its asked span or the needs of all but one or two heads.
- **Next**: the page (a subagent), the visual checks, the report.

## Where I am (second agent, 2026-11-20, about 00:30)

- 14 measures now: two added, `root_dip` (the profile at the eye line behind the line from the brow, 6 mm above the eyes, to mid-bridge) and `bend` (the bridge line's lower half minus its upper half). `bridge_height` is now the profile's depth at mid-bridge alone (sampling the face 16 mm aside made the slider push the cheeks into lumps). The cheek for projection is read halfway between the tip and the base (the widest wing row jumped on flat noses and made projection jump by 4 mm).
- 14 named sliders in `SPECS`, grouped root, bridge, tip, wings, base. With them only 2 profile misses remain (were 27 with 12 sliders).
- The page was rebuilt by a subagent: round choice in the live view and in "Made against original", grouped sliders with hints, scorecards side by side, a "Same or different nose" tab (answers to `answers/same_or_different.json`), round 2 leak table. `VER` is `?b=13`, `index.html` loads `nose.js?v=14`, data is fetched with no-cache. The module was rebuilt at 00:11 with the 14-measure code.
- Running: `nose_r2 bench`, then the full `nose_bench` (round 1 rescored with the new measure). Then: look at the page and the eight heads, roadmap, report.

## Final state (second agent, 2026-11-20)

- Round 2 is done and reported. `web/sliders_r2.json` (14 sliders), `web/page_r2.json`, `made_r2.*` and `report_r2.json` match each other. Round 1 was rescored with the new measure by a full `nose_bench` run (52 of 57 held out at 12 sliders).
- 72 answers to "Same or different nose" were saved between 00:12 and 00:17 in `answers/same_or_different.json`. They cover the 12 measures that existed then; root dip and bridge bend were never asked. The answers are noisy: several measures have a "different" answer at a smaller change than a "same" answer.
- Open faults: the slider ends fold many stand-ins (tip projection low end on 45 of 57), wing flare cannot go below −0.40 mm (21 heads want up to 0.7 mm less), the tip pinches into a ridge at low tip width and bluntness, root height's high end makes a fin, pmha01 held out misses by 16 degrees of tip turn because its length need sits outside the held-out range.
- Clippy shows 7 style warnings in the crate (loops that index, one function with 8 arguments, one `unwrap_or_default`).
