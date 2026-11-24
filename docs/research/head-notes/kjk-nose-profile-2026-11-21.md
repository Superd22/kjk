# The nose seen from the side: the kink on pmhc04 and the Handmaiden (2026-11-21)

This note was written by the conversation `nose-profile`, started by the head HQ. The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/159b5fd0-5315-4d72-803e-88198c5439e1/scratchpad/np`, called `$NP` below. It serves roadmap items B3 and B5 for the nose.

Nothing is live and nothing is committed. The result is in viewer 2 only behind `?noseprofile=1`.

## Verdict

**The kink comes from the nose's own presets, not from the eye construction.** The solved values of the 14 nose sliders put a narrow bump at the eye line and leave a hollow 4 to 7 mm below it. On pmhc04, Root dip at −1.97 pushes the skin out by 2.6 mm at the eye line, and Bridge line at −0.87 pushes it out again from 8 mm below. Between the two, the profile sinks 1.7 mm behind KOTOR's. The FLAME fit and the nose stand-in have no kink. With the eye curve on or off, the profile below the eye line moves by less than 0.1 mm.

**The kink is fixed in the preview.** Two sliders were added (Root place and Upper bridge), and the 57 noses were solved again against a new side-profile measure. In viewer 2's own build:
- pmhc04's bridge gap to KOTOR went from 0.66 to 0.10 mm, and its root corner went from 5.0 mm below the eye line to 5.6 mm above it (KOTOR: 5.9 mm above).
- The Handmaiden's bridge gap went from 0.68 to 0.15 mm, and her root corner from 4.6 mm below to 8.5 mm above (KOTOR: 7.6 mm above).
- pmhc03 and old_w_m keep their sharp root corner. pmhc03's corner turns 58 degrees, where KOTOR's turns 66 and live turned 49.

**What is still wrong or unchecked, first.**
- **Root place sits at an end of its range on 13 of 57 heads.** Its range is −4 to +4 mm. A first version with a narrower field and a range of ±6 mm bunched the skin into a 1.7 mm notch at its low end on pmhc04. The broader field was not tested past ±4. Its far ends need repairing before the range can grow.
- **The 14 existing sliders alone cannot remove the kink.** With the profile weighted heavily, pmhc04's root corner stays 11 mm too low. pmhc03 cannot be reached at all without Root place.
- **Five heads keep their live preset** because the guard refused the new one. They are comm_b_f (1 new folded triangle), mod_jaesa (gain under 0.1 mm), n_daraalah (2 new folded triangles), n_zezkaiellh (2 new folded triangles and three nose rows worse) and pfha02 (15 folded triangles, against 2 live).
- **Bastila's root corner is sharper** than KOTOR's: 35 degrees against 27 (live 29). Her bridge gap still improves, from 0.43 to 0.28 mm. I looked at her outline picture only, not at her in viewer 2.
- **The fold guard counts on the nose's own stand-in**, not on viewer 2's head with every feature. The fold counts of viewer 2's build were not checked.
- **Not checked:** the blink and the mouth with the new presets, Root place and Upper bridge at their ends in viewer 2's light (only in clay), pairs of sliders at their corners, the slider page `/nose/`, and any judge (none were run, as the brief asked).

## Where the kink comes from (measured)

The profiles were read on the midline every 0.25 mm on KOTOR's mesh, the FLAME fit (`fit_smooth`), the nose stand-in, the stand-in with the nose preset, and viewer 2's head with the eye curve on and off (built by viewer 2's own modules in Node, `$NP/live/dump.mjs`).

pmhc04, depth out of the face in mm at a few heights (mm above the eye line):

| Height | KOTOR | FLAME fit | Stand-in | Stand-in + preset | Viewer 2, old eyes | Viewer 2, eye curve |
|---|---|---|---|---|---|---|
| +5 | 30.85 | 31.27 | 29.58 | 30.99 | 31.37 | 31.30 |
| 0 | 32.24 | 31.37 | 29.78 | 32.41 | 32.37 | 32.34 |
| −4 | 33.36 | 32.07 | 31.38 | 31.72 | 31.70 | 31.70 |
| −8 | 34.24 | 33.48 | 34.13 | 34.16 | 34.14 | 34.17 |

The preset is in front of KOTOR at the eye line and 1.6 mm behind it at −4 mm. That is the kink. The Handmaiden shows the same pattern (−2.2 mm at −4 mm). Picture: `$NP/final/diagnosis_steps.png`.

The eye side needs nothing for this fault. The eye's field moves the profile only above the eye line, by about 0.5 mm near the brow.

## The measure (measured)

`src/nose_profile.rs` in the worktree's `head_sliders`. It reads the midline profile on KOTOR's own mesh in the head's frame, blurred by 0.75 mm, from the brow's front down to the base of the nose. Every 1 mm along KOTOR's profile it reads the distance to our profile, signed, with nothing taken out: the head's frame is shared, so this is the overlay David sees. Points above 10 mm over the eye line are left out of the score, because the brow's shape decides them. It gives:
- the bridge gap, the root mean square of the gaps from the brow (or 10 mm above the eye line) to 6 mm above the tip;
- the whole gap, down to the base of the nose;
- the swing, the largest change of the gap within 8 mm along the bridge, which is what a kink adds;
- the root corner, the largest forward turn between straight lines fitted 8 mm above and 8 mm below a point, and its height.

**Check 1** (KOTOR against its twice-subdivided copy, median and 95th percentile, against two different KOTOR heads):
- bridge gap 0.24 and 0.56 mm, against 3.43 mm between two heads;
- whole gap 0.30 and 0.56 mm, against 4.05 mm;
- swing 0.65 and 1.49 mm, against 2.91 mm;
- root corner turn 2.8 and 14.8 degrees, against a spread of 16.8 degrees;
- root corner height 0.86 and 3.53 mm, against a spread of 3.72 mm.
The gaps are sturdy. The swing is usable but noisy. The corner's turn and height are weak at the 95th percentile, so they describe the kink and are not solved on.

**Check 2:**
- Our pmhc04 reads a swing of 2.03 mm and our Handmaiden 2.74 mm, above the 95th percentile of the wobble (1.49 mm). Their root corners read 11 and 12 mm too low.
- KOTOR's pmhc03 and old_w_m read sharp corners of 66 and 50 degrees. KOTOR's pmhc04 reads 38 degrees and the Handmaiden 27, with their corners where the bridge starts.
- The separation between old_w_m (50) and pmhc04 (38) is smaller than the corner's 95th percentile wobble (14.8 degrees). The corner number alone is not a sure kink detector. The gaps and the picture are.

**Check 3:** Loop's rounded copy of each KOTOR head is nearest to its own head for 49 of 57.

**Check 4:** `$NP/final/check4_extremes.png`. The most alike pairs are mod_keanu and mod_narve_canderous (0.46 mm), old_b_f and pfhb04, and mod_tcs_bastila and p_bastilah. The most different pairs all involve Terena Adare (16 to 17 mm). Because nothing is taken out, the most different pairs are mostly a difference of place, not of shape.

## The sliders

- **Root place** (group root, −4 to +4 mm): the skin round the top of the nose slides up or down the face, most at 1 mm below the eye line, fading over 14 mm up and down and 10 mm to each side. Hint: "Seen from the side, how high the dip at the top of the nose sits between the eyes. A higher value moves the dip up toward the brow, so the bridge starts higher."
- **Upper bridge** (group bridge, −3 to +5 mm): the skin from the eye line to 6 mm below it moves straight out of the face, fading with cosine edges to nothing 8 mm above and 18 mm below, and over about 8 mm to each side. Hint: "Seen from the side, how far the top of the bridge stands out, just under the eyes. A higher value fills the hollow under the top of the nose, so the bridge runs straight from between the eyes."
- Both are fixed movements read on the base head, so a value gives the same movement on every head.
- Folded triangles near the nose over the 57 heads (measured on the nose stand-in): Root place has 11 heads with a fold at its low end, 10 at the solved values and 11 at its high end. Upper bridge has 11, 10 and 9. The 10 at the solved values already fold the same way live.
- The clay ends sheet is `$NP/s2c/ends.png`. At that size I saw no crease.

Loop 1 added Root place alone, with a narrower field. It fixed pmhc03 but left a 1.7 mm notch on pmhc04. Loop 2 broadened Root place, capped it at ±4 and added Upper bridge.

## The solve and the guard

`sliders noseprofile solve` solves each head with the 16 sliders against the kit's 17 nose rows plus the fine profile's gaps (weight: a 0.15 mm gap on every point costs one row at its mark). It measures the head as viewer 2 builds it (`NP_VIEWER`, viewer 2's head with the nose at 0 plus the nose sliders' movement). It starts from live and from six other starts. A head keeps the largest share of its move (1, 0.75, 0.5, 0.25) that passes these tests:
- no more folded triangles near the nose than live;
- fewer than three of the 17 rows worse than live by more than half their mark;
- a bridge gap at least 0.1 mm smaller and a swing at most 0.25 mm larger.

45 heads take the full move, 6 three quarters, 1 half, and 5 keep live.

**Before and after, in viewer 2's own build** (`$NP/beforeafter.json`): the median bridge gap goes from 0.54 to 0.17 mm, and the 90th percentile from 0.90 to 0.41 mm. The median swing goes from 1.39 to 0.69 mm. 52 heads improve by more than 0.05 mm and none gets worse. Root corners more than 4 mm below KOTOR's go from 10 heads to 2 (mod_hp_pfha01 and Kreia).

With the 14 old sliders only, the same solve kept 18 heads at live, and the median bridge gap reached 0.30 mm.

## Viewer 2

- New: http://127.0.0.1:8765/viewer2/?head=pmhc04&noseprofile=1, http://127.0.0.1:8765/viewer2/?head=p_handmaidenbb&noseprofile=1, http://127.0.0.1:8765/viewer2/?head=pmhc03&noseprofile=1, http://127.0.0.1:8765/viewer2/?head=old_w_m&noseprofile=1, http://127.0.0.1:8765/viewer2/?head=p_bastilah&noseprofile=1
- Today: the same addresses without `&noseprofile=1`.
- Checked in Chrome on pmhc04: with the option the Nose tab has 16 sliders and the bridge is straight. Without it there are 14 and the old bend shows (`$NP/final/viewer2_pmhc04_new.png`, `viewer2_pmhc04_today.png`).

## Pictures

- `$NP/final/profile_<head>.png` for pmhc04, p_handmaidenbb, pmhc03, old_w_m and p_bastilah: KOTOR, today and new in clay, each with KOTOR's outline (thick black), today's (red) and the new one (green).
- `$NP/final/sheet57.png`: all 57 outlines, with the bridge gap and the swing today and new.
- `$NP/final/check4_extremes.png` and `$NP/final/diagnosis_steps.png`.

## How to undo

- Viewer 2: copy `$NP/sliders.js.before` over `~/.claude/tools/likeness-fit/hq/viewer2/sliders.js` and run `~/.claude/tools/likeness-fit/hq/viewer2/restage.sh`. Or leave it: without `?noseprofile=1` nothing changes.
- Data: delete `web/sliders_r2_profile.json` and `web/page_r2_profile.json` in `~/.claude/tools/likeness-fit-data/sliders/`.
- Code: delete `src/nose_profile.rs` and remove its line from `lib.rs` and its dispatch from `src/bin/sliders.rs`. The build folder `target-noseprof` in the worktree can be deleted.
- Roadmap: my lines in "Now and next".

## How to run it again

```
cd /Users/david/perso/kjk/.claude/worktrees/head-paint
export PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH"
CARGO_TARGET_DIR=target-noseprof cargo build --release -p head_sliders --bin sliders
D=~/.claude/tools/likeness-fit-data/sliders
# viewer 2's heads, live and with the nose at 0 (Node, viewer 2's own module)
(cd $NP/live && node dump.mjs 1 live_curve.bin && node dump.mjs 1 nose0_curve.bin jaw,brow,cheeks,eyes)
# the set: Root place, then Upper bridge
NP_RP_HEIGHT=14 NP_RP_LOW=-4 NP_RP_HIGH=4 ./target-noseprof/release/sliders noseprofile make $D $NP/s2c/rp.json
NP_BASE_SET=$NP/s2c/rp.json NP_NEW=upper NP_RP_LOW=-3 NP_RP_HIGH=5 ./target-noseprof/release/sliders noseprofile make $D $NP/s2c/sliders.json
# the solve
NP_VIEWER=$NP/live/nose0_curve.bin NP_MARK=0.15 SLIDERS_SET=$NP/s2c/sliders.json NOSE_VALUES=$NP/live/live_curve.json ./target-noseprof/release/sliders noseprofile solve $D $NP/s3/solve.json
# the checks and before/after on viewer 2's builds
(cd $NP/live && NOSE_PROFILE=1 node dump.mjs 1 new_curve.bin)
./target-noseprof/release/sliders noseprofile checks $D $NP/checks2.json live=$NP/live/live_curve.bin new=$NP/live/new_curve.bin
```
The page file `web/page_r2_profile.json` holds, for each head of `solve.json`, its `values` as `heads[].solved.full.counts["16"].values`, with `counts: [16]` and `sliders` listing the 16 names in the set's order.

## Traps

- **The nose presets were solved on the nose stand-in, but viewer 2 builds on `starts_all.bin` with every feature.** Solving on the stand-in gave pmhc04 0.14 mm in the solve and 0.37 mm in viewer 2. Measure the head viewer 2 shows (`NP_VIEWER`).
- **The kit's side-profile outline takes out position and size**, so it cannot see a 1.5 mm local kink. The new gaps keep both.
- **The brow's front decides the top of the profile.** Scoring from the brow's front made the Handmaiden's 2.3 mm brow miss dominate. The score starts 10 mm above the eye line.
- **A sliding field bunches skin.** Root place slides skin along the face. Narrow and at −5.5 mm, it made a notch at the lower edge of its fade.
- **The solve has several minima.** Root place went the wrong way from the live start. The solve now tries seven starts.
- **A slider's contribution read at a fixed height on a sloped profile looks like a step** when the slider slides skin vertically. Read the overlay, not the ablation table, for sliding sliders.
- **The corner number wobbles** (95th percentile 14.8 degrees, 3.5 mm). Do not solve on it.

## What comes next

- Repair Root place's ends so its range can pass ±4 mm, and look at both new sliders at their ends in viewer 2's light.
- Look at Bastila's sharper corner in viewer 2.
- If David likes the preview: make the two sliders part of the nose's kit set (with a measure each), and solve the presets in the bench.

## Per head (viewer 2's own build, mm)
| Head | Share of the move kept | Bridge gap today | Bridge gap new | Swing today | Swing new |
|---|---|---|---|---|---|
| comm_a_f | 1.0 | 0.64 | 0.21 | 1.51 | 0.89 |
| comm_a_f2 | 1.0 | 0.79 | 0.23 | 1.89 | 0.97 |
| comm_a_m | 1.0 | 0.46 | 0.25 | 1.37 | 1.17 |
| comm_b_f | 0.0 | 0.38 | 0.38 | 0.91 | 0.91 |
| comm_b_m | 1.0 | 0.30 | 0.12 | 1.67 | 0.46 |
| comm_w_f | 1.0 | 0.61 | 0.16 | 1.45 | 0.63 |
| comm_w_m | 1.0 | 0.54 | 0.14 | 1.90 | 0.49 |
| mod_exile_canon | 1.0 | 0.25 | 0.09 | 0.83 | 0.34 |
| mod_hp_pfha01 | 0.75 | 0.76 | 0.40 | 1.37 | 0.80 |
| mod_hp_pfhb02 | 1.0 | 0.42 | 0.12 | 0.71 | 0.42 |
| mod_improved_bastila | 1.0 | 0.40 | 0.14 | 1.19 | 0.60 |
| mod_jaesa | 0.0 | 0.23 | 0.23 | 1.00 | 1.00 |
| mod_keanu | 1.0 | 0.59 | 0.12 | 0.98 | 0.45 |
| mod_kenobi | 1.0 | 0.52 | 0.16 | 1.11 | 0.76 |
| mod_kyle_katarn | 1.0 | 0.57 | 0.16 | 2.01 | 0.58 |
| mod_layne | 1.0 | 0.56 | 0.12 | 2.36 | 0.44 |
| mod_meetra | 1.0 | 0.33 | 0.09 | 1.31 | 0.38 |
| mod_narve_canderous | 1.0 | 0.56 | 0.15 | 1.62 | 0.83 |
| mod_tcs_bastila | 1.0 | 0.94 | 0.17 | 2.88 | 0.63 |
| n_admoffh | 1.0 | 0.41 | 0.26 | 1.44 | 0.97 |
| n_daraalah | 0.0 | 0.26 | 0.26 | 0.83 | 0.83 |
| n_dodonnah | 1.0 | 0.48 | 0.12 | 0.66 | 0.44 |
| n_dorakh | 1.0 | 0.45 | 0.10 | 1.34 | 0.41 |
| n_hussef | 1.0 | 0.22 | 0.08 | 0.72 | 0.35 |
| n_jedimalek | 1.0 | 0.65 | 0.16 | 2.01 | 0.53 |
| n_mastkavarh | 1.0 | 0.47 | 0.13 | 1.58 | 0.44 |
| n_terenaadareh | 0.75 | 2.33 | 0.87 | 2.41 | 1.97 |
| n_traskh | 1.0 | 0.86 | 0.13 | 1.03 | 0.65 |
| n_zezkaiellh | 0.0 | 1.27 | 1.27 | 2.26 | 2.26 |
| n_zherronh | 1.0 | 0.69 | 0.12 | 2.13 | 0.41 |
| old_a_f | 1.0 | 0.46 | 0.15 | 1.37 | 0.75 |
| old_a_m | 1.0 | 1.54 | 0.42 | 1.50 | 1.12 |
| old_b_f | 1.0 | 0.49 | 0.18 | 1.39 | 0.95 |
| old_b_m | 1.0 | 0.42 | 0.09 | 1.30 | 0.38 |
| old_w_f | 1.0 | 0.46 | 0.14 | 1.07 | 0.53 |
| old_w_m | 1.0 | 0.64 | 0.28 | 1.35 | 0.99 |
| p_baodurh | 0.5 | 0.58 | 0.34 | 1.75 | 1.02 |
| p_bastilah | 1.0 | 0.43 | 0.28 | 1.38 | 1.12 |
| p_carthh | 1.0 | 0.27 | 0.11 | 0.97 | 0.44 |
| p_handmaidenbb | 1.0 | 0.68 | 0.15 | 2.74 | 0.55 |
| p_joleeh | 1.0 | 0.41 | 0.21 | 1.53 | 0.79 |
| p_kreiah | 1.0 | 0.54 | 0.20 | 1.49 | 0.70 |
| pfha02 | 0.0 | 0.70 | 0.70 | 0.85 | 0.85 |
| pfha05 | 0.75 | 0.43 | 0.20 | 1.27 | 0.70 |
| pfha06 | 1.0 | 0.57 | 0.14 | 2.10 | 0.57 |
| pfhb04 | 1.0 | 0.47 | 0.15 | 1.25 | 0.79 |
| pfhc02 | 1.0 | 0.44 | 0.22 | 1.27 | 1.08 |
| pmba01 | 1.0 | 0.62 | 0.21 | 1.17 | 0.82 |
| pmha01 | 1.0 | 0.56 | 0.17 | 0.84 | 0.69 |
| pmha02 | 1.0 | 0.89 | 0.32 | 2.31 | 1.85 |
| pmha04 | 0.75 | 1.16 | 0.53 | 2.17 | 0.64 |
| pmha05 | 0.75 | 0.52 | 0.23 | 1.33 | 0.55 |
| pmha06 | 0.75 | 0.93 | 0.45 | 1.51 | 0.62 |
| pmhc01 | 1.0 | 0.53 | 0.24 | 1.76 | 0.96 |
| pmhc03 | 1.0 | 0.64 | 0.24 | 2.68 | 1.02 |
| pmhc04 | 1.0 | 0.66 | 0.10 | 2.03 | 0.37 |
| sith_app | 1.0 | 0.40 | 0.10 | 1.52 | 0.39 |
