# Slider kit and jaw and chin: handoff

Started 2026-11-20 by the agent that turned the nose code into a kit and builds the jaw and chin with it. Read the nose handoff
(`$TMPDIR/kjk-nose-round2-handoff-2026-11-19.md`) for the crate's history, traps and servers. This note says what changed since.

## Where I am (2026-11-20, about 01:30)

- **The kit exists and the nose came through it unchanged.** Crate `head_sliders` (worktree head-paint). New modules: `feature.rs`
  (the `Feature` trait, `Measured`, `Spec`, generic outlines and `Scale`), `nose.rs` (the nose as a feature, its 14 specs), `jaw.rs`,
  `maker.rs` (was `named.rs`, now generic), `checks.rs` (check 1 and the marks, checks 3 and 4), `bench.rs` (all commands), and
  `round1/` (round 1's solve, verdict and learn, frozen for `nose_bench`). `nose_r2.rs` was replaced by `src/bin/sliders.rs`
  (its old copy is in the scratchpad `kit/old_src/`). `measure.rs` got `Window` and windowed surfaces; its bins stay as before for the
  nose so the nose's numbers are bit for bit the same.
- **Proof the nose is unchanged:** `sliders nose all <scratch folder>` gave the same `marks.json`, `sliders_r2.json`, `made_r2.bin`,
  `starts.bin`, and the same `page_r2.json`, `report_r2.json`, `made_r2.json`, `named.json` apart from timings (key `nose` is now
  `measured`). The first comparison found 1e-13 differences, from changed height bins; the bins were restored. The second full
  comparison is still to be rerun after that fix (see the commands below and `kit/jcmp.py` in the scratchpad).
- **Extra inputs:** `~/.claude/tools/likeness-fit/sliders/prep_extra.py` writes `input/extra/` and `input/manifest_extra.json`
  (rim, below, mouth bag, lip line, ears, and per head KOTOR's skin mask and face colours). `Input::open` merges the extra manifest.
- **The jaw measure (jaw.rs), version 3.** A depth picture from in front and 40 degrees above; the lower face is what joins the chin
  without a step steeper than 5 in 1; its edge along rays gives the front outline (19 points every 5 degrees around the place 60 mm
  below the eyes) and, with depth, the jawline from the side. Proportions: jaw width (face outline 82 to 88 mm below the eyes, in front
  of 75 mm behind the eyes), jaw shape (squareness at mid height), jawline slope (side), chin length (bottom), chin width (level cuts, 6
  mm behind the front), chin projection (front of the chin), angle under the chin. Outlines: front, side, chin profile (from the
  hollow under the lip to where it turns under). Dropped: corner landmark from a chord (unstable), chin point shape (KOTOR's spread
  2.7 mm under its wobble), corner height (no steady definition yet).
- **Hair and cloth:** KOTOR vertices not skin (hairless wrap's mask) and more than 6 mm outside our fit; 4 such triangles on the jaw's
  sides or chin hide those rows. Hidden on: Kreia, pfha06, pmha06, mod_keanu, mod_narve_canderous, mod_tcs_bastila (sides), and
  mod_hp_pfha01 (all). Malak: measure refused ("metal jaw"), left out of the scorecard and the ranges.
- **Next:** make the jaw sliders, look at their ends in clay, judge with a fresh agent, bench, page (by a sub-agent), roadmap, report.

## Commands

```
export PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH"
cd /Users/david/perso/kjk/.claude/worktrees/head-paint && cargo build --release -p head_sliders
cd ~/.claude/tools/likeness-fit && .venv/bin/python -I sliders/prep_extra.py     # once
./target/release/sliders nose all            # the nose: marks, make, bench, sheet (about 10 minutes)
./target/release/sliders jaw all             # the jaw and chin
./target/release/sliders all-features starts # web/starts_all.bin for the page's live view of every feature
#  other commands: marks, needs, make, bench, sheet, kotor-sheet, probe <data> <head>, probe-sheet <data> <head...>
```

## Where I am (2026-11-20, about 01:55)

- **Round 1 of the jaw was built and judged** (scratch folder `kit/data1`). Judge (fresh Sonnet, pictures only) found: chin projection's
  high end makes a beak-like shelf, chin width creases at both ends, jaw width's high end folds at the jaw angle, jaw shape and jawline
  slope are nearly dead (their measures barely answer to bumps), and made jaws cluster narrow and V-shaped (Canderous, Zherron missed).
  The provisional scorecard said 54 of 56 because the jaw's pass marks are loose (KOTOR's jaw meshes wobble).
- **Round 2 fixes (the last round):** bumps 9 mm apart 7 mm (were 6 and 5); jaw shape replaced by "Lower jaw width" (outline 98 to
  104 mm below the eyes, in front of 40 mm behind them); jawline slope kept as a measure but its slider dropped; pogonion read only up to
  18 mm above the chin's bottom; jaw sliders no longer carry the inside of the mouth (it moved 3.8 mm in round 1); spans cover 95% of
  the needs (`span_quantile`); the jaw's optimiser aims at a quarter of KOTOR's spread (`solve_by_spread`); a strict scorecard (a
  quarter of the spread) and the stand-ins alone are reported next to the provisional one.
- Running: `sliders jaw all` on the real data folder (log `kit/jaw_all2.log`), and the nose identity check in `kit/data3`.
- A Sonnet sub-agent is rebuilding the page (feature choice, jaw cards, all features live, card marks saved as `card_marks`).

## Where I am (2026-11-20, about 03:00)

- Round 2 ran on the real data folder (`sliders jaw all`). All six sliders reached most of their span after one more correction:
  bumps of 9 mm made jaw width, chin length and the new lower jaw width dead, so bumps went back to 6 mm (5 apart), and jaw width and
  lower jaw width hold each other loosely.
- The second fresh judge found "Lower jaw width" broken at both ends (a gash at the low end, a shelf at the high end) and it put ledges
  on solved heads (Kreia, pmha01). It was taken out of the page's set: the full set is kept as `jaw/sliders_with_lower_jaw_width.json`,
  `web/jaw/sliders.json` holds the other 5, and the bench was rerun on those 5 (log `kit/jaw_bench3.log`).
- The nose through the kit: marks.json and starts.bin are byte for byte the same; every verdict, count and flag is the same; numbers
  differ by at most 3.5e-6 relative (a solved gap) and made positions by at most 6e-8 m. The source of those 13th-digit differences
  in the slider maker was not found. The real data folder's nose files were not rewritten.

## How to add a third feature with the kit (for example the mouth or the brows)

1. Write `src/<feature>.rs` with a unit struct and a `static`, like `jaw.rs`. Implement `feature::Feature`:
   - `proportions()` and `outlines()`: what the feature is, as named measures (`measure::Proportion`, `feature::OutlineDef`).
   - `window()` (the part of the face the measure reads, in mm in the frame of the eyes) and `region()` (smaller, for check 3).
   - `measure(surface)`: read the measures from cuts (`Surface::front`, `Surface::cut`) or a depth picture (see `jaw.rs`), returning
     `Measured`. Put drawable points in `marks["dots"]` so `kotor-sheet` and `probe-sheet` show them.
   - `measure_kotor(...)` only if hair or cloth can cover the feature on KOTOR's own mesh (see `jaw.rs`'s `cover`).
   - `zone(mesh, base)`: weight 1 on the feature, fading to 0, and 0 on every vertex the smoothing holds (`mesh.keep` and not hidden).
   - `part_of(p, base)` and `groups()`; `specs()`: one `Spec` per slider (measure, name, hint, group, parts, repair parts, free, loose).
   - `bumps()` (size and spacing; 6 mm and 5 mm suited the jaw, 4 and 3.5 the nose), `check_bump()`, `shots()`, `miss_words()`.
   - Optional: `guards()` for the leak table's own lines, `carries_hidden()` false when the inside of the mouth must not move,
     `span_quantile()` below 1 and `solve_by_spread()` true when KOTOR's measure is noisy.
2. Register it in `lib.rs` and in `feature::by_key` and `feature::all`.
3. Look first: `sliders <f> kotor-sheet` (KOTOR's 57 meshes with the measure's dots), `sliders <f> zone-sheet`, `sliders <f> probe-sheet
   <data> <head>...` (KOTOR, subdivided, ours). Then `sliders <f> marks` prints each measure's wobble against KOTOR's spread.
4. `sliders <f> all` makes the sliders, benches them and draws `<f>_sheet.png`. `sliders <f> made-sheet <data> <head>...` draws KOTOR
   next to ours. Judge the pictures with a fresh agent before believing the scorecard.
5. `sliders all-features starts` rewrites `web/starts_all.bin` with every feature's own shape taken out. The page reads the new
   feature's files from `web/<f>/`; it needs a feature choice entry, camera presets and slider groups for it.
With the measure as the only real work, a third feature should take about half a day: two hours for a measure that holds up in
check 1, an hour of making and benching on this loaded Mac, and the rest for pictures, a judge and the page.

## Final state (2026-11-20, about 03:20)

- Jaw and chin on the real data folder: `jaw/marks.json`, `jaw/checks.json`, `jaw/named.json`, `jaw/report.json`, `jaw_sheet.png`,
  `web/jaw/{sliders.json, page.json, made.bin, made.json, starts.bin}`, `web/kotor_jaw/` (57 crops), `web/starts_all.bin`.
  `web/jaw/sliders.json` has 5 sliders; the 6-slider set with "Lower jaw width" is `jaw/sliders_with_lower_jaw_width.json`.
- Page version 19 (`VER`, `load.js?v=`, `nose.js?v=`). The module was rebuilt at 03:14 because the 01:16 build still had the round 1
  jaw measure (the live view read "width near the chin" as 13 mm). Card marks save as `answers/card_marks.json`; none are saved yet.
- Fresh judges' pictures: `scratchpad/kit/judge1/` and `judge2/`. Both judges' reports are summarised in the final report.
- Not done: a working slider for the jaw's corner, squareness or the jawline's slope; the chin width's narrow end still pinches; the
  scorecard's provisional marks are loose and the strict ones (a quarter of KOTOR's spread) make 1 of 56.
