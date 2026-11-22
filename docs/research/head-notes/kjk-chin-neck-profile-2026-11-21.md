# The profile from the chin to the neck: measure, throat sliders and solve

This note was written on the night of 2026-11-21 by the conversation briefed by the head HQ. The brief is `/private/tmp/claude-501/-Users-david-perso-kjk/52003e36-b8ca-4208-a733-a6d51b417e4e/scratchpad/jaw/brief-chin-neck-profile.md`. The work serves B3 and B5 for the jaw and chin. Nothing live changed. Nothing was committed. No judges were run.

## Verdict

The outline measure passes checks 1, 3 and 4, and check 2 on the one fault David named. The throat construction was rebuilt as a smooth field on top of the skull-refit head. It has four named sliders. Solved on 10 heads with no value set by hand, the outline gap to KOTOR falls from 1.4 to 8.0 mm (live) to 0.7 to 2.2 mm. The 5 jaw sliders alone only reach 1.4 to 6.0 mm. No solved head, slider end or pair corner turns a triangle. David has not looked yet.

## What is still broken, first

- **The corner line is jagged seen from below** on pmhc04 and other heads with a sharp corner. The corner runs diagonally across the mesh's rows, so it steps from one row to the next. KOTOR's corner is one straight row of vertices. No row of vertices was laid on the corner line (see "Row of vertices" below).
- **A small bump remains on the side of the neck** under the jaw's corner, seen three-quarter on pmhc04, where the corner line meets the side of the neck.
- **Two sliders sit at the end of their range on several heads.** Throat depth is at or near its top (76 mm) on Bastila, Jaesa, Zherron, the Handmaiden, comm_b_f and pmhc04. Throat roundness is at its bottom (2 mm) on pmhc04, Carth, pmhc03, sith_app and pfha02. The ranges should probably be widened. The top of 76 mm was set by the last conversation to avoid a hooked crease in the old construction, and it was not tested again with the smooth field.
- **The solve moves the face a little on two heads.** The held jaw measure costs 8.4 on Jaesa and 4.2 on the Handmaiden (1 means 1.5 units off on one row), so their jaw sliders moved further from live than on the other heads. Their faces were not checked close up.
- **The ledge is not measured.** A profile outline cannot see how far the jaw stands out over the neck from the front. The jaw's corner only counts through the side view, as part of the whole outline.

## The measure (`chin_neck.rs`, `sliders chinneck`)

One outline runs from the chin's front, down round the chin, back along the underside of the jaw, round the throat's corner, and down the front of the neck to 34 mm below the chin's bottom. It is a walk round the outer edge. It follows the front-most skin going down, the lowest skin going back once the front recedes faster than 45 degrees, and the front-most skin going down the neck once the underside drops faster than 45 degrees.

- It is read in four ways: the middle section, the sections 12 mm and 24 mm from the middle on each side, and the side view. The side view is the silhouette of everything within 55 mm of the middle, which is what David sees in profile and which carries the jaw's corners.
- Place and size are taken out. Each outline is moved so that the chin's front and bottom lie at 0, and scaled so that the chin's bottom lies 120 mm under the eyes.
- The chin's front is the front-most place of the middle profile 2 to 25 mm above the chin's bottom. The first version took Bastila's lower lip for her chin.
- The gap between two outlines is the mean distance from each one's points to the other line, both ways, in mm. The whole gap weighs the middle, 12 mm and the side view equally. The 24 mm section gets no weight.
- KOTOR is read on its own bare mesh. Every reading is hidden on the 5 heads where hair, a hood or a beard lies on the jaw's back half: Kreia, pfha06, Keanu, Kyle Katarn and Canderous of the mod. The outline is unreadable on the 7 short neck stubs (comm_b_m, n_admoffh, n_dorakh, old_b_f, old_b_m, old_w_f, pfha05 and pmba01 are among them). The middle reads on 44 heads, the side view on 44, 12 mm on 37 and 24 mm on 19.

The checks, with wobble from KOTOR's mesh against the same mesh subdivided twice, and from a section's left side against its right side:

| Reading | Wobble, median and 95th percentile (mm) | Left against right (mm) | Two different KOTOR heads, median (5th percentile) | Check 3 | Live's gap to KOTOR, median (95th percentile) |
|---|---|---|---|---|---|
| Middle | 0.62 / 1.30 | | 2.93 (1.23) | the subdivided copy finds itself for 36 of 41 heads | 4.1 (6.7) |
| 12 mm | 1.09 / 1.90 | 0.99 / 2.29 | 3.16 (1.65) | 25 of 35; right side finds its own left for 26 of 37 | 5.3 (8.0) |
| 24 mm | 1.11 / 2.09 | 1.60 / 4.55 | 4.27 (2.29) | 14 of 17; 12 of 18 | 5.7 (9.4) |
| Side view | 0.71 / 1.36 | | 2.95 (1.24) | 35 of 41 | 3.9 (6.6) |

- Check 1 passes on the middle, 12 mm and the side view. The 24 mm section fails it, because its two sides differ by up to 4.5 mm.
- Check 3 passes clearly on the middle and the side view.
- Check 4 was seen. The most alike pairs (comm_a_f and comm_a_f2 at 0.4 mm, Bastila and pfha02 at 0.9 mm) do look alike. The most different (Jolee against pmha01 at 10.9 mm) look different. The worst live fits (Jaesa 8.0, the Handmaiden 7.0, the Exile, comm_b_f) all show the fault David named: our underside slopes down into the neck where KOTOR's runs level to a corner.
- Check 2: the old prototype that David called "way better" on pmhc04 cuts its gap from 6.1 to 1.9 mm. On pmhc03, which the judges scored lower, it raises the gap from 1.4 to 1.8 mm. pmhc04's live gap (6.1 mm) is the 38th of 49 and Bastila's (4.4 mm) is the median.
- The corner and the ledge: the corner passes only as part of the whole outline. The ledge is not in this measure.

## The construction (`ThroatShape` in `chin_neck.rs`)

It uses the old construction's outline (`throat_corner::Construction::outline`, now `pub`). Each free skin vertex asks for a move onto the outline along a ray from one place behind and above the throat. The chin and the face above the outline's start are held, as is skin far inside the outline (the back of the neck). Skin past the neck's end of the outline is free. The moves are then solved as one smooth field over the mesh's edges (stiffness 1, 400 sweeps), with the rim, the lips, the ears and the inside of the mouth fixed at 0. This removed the step across the throat and most of pmhc03's lump. The strength is always 1, so it is not a slider. The construction runs after every other slider on the built head, so the order of the other sliders cannot change it.

The sliders (`THROAT_SLIDERS`):

| Slider | Low end | High end | Start | What it does |
|---|---|---|---|---|
| Throat depth | 45 mm | 76 mm | 60 | How far back under the chin the throat's corner lies. A deeper throat has a longer underside. |
| Throat corner height | -12 mm | 15 mm | 0 | How high the corner lies against the bottom of the chin. Low values slope the underside down into the neck. |
| Jaw corner rise | 0 mm | 28 mm | 10 | How much higher the line under the jaw runs toward the jaw's corners than under the chin. |
| Throat roundness | 2 mm | 25 mm | 8 | The radius of the bend where the underside turns into the neck. Low values make a sharp corner. |

The start values are not the base head's measured values. They were set by me. Nothing has been solved on the base head.

The other jaw sliders: on pmhc04 with the throat on, moving each jaw slider to either end keeps the outline gap between 1.2 and 4.0 mm. Without the throat it is 5.1 to 9.5 mm. The largest changes come from Chin projection and Chin length, which move the chin, and the chin is part of the outline.

## Row of vertices on the corner line

Not built. The corner line runs diagonally across the rows of our mesh under the jaw, so no row of vertices lies along it. Sliding the nearest vertices onto it is the "gathering" the last conversation tried, and all four judges saw it as a pinched ridge. A real row needs new topology under the jaw. It would also fix the jagged line seen from below. This is the next thing to try if David likes the profile.

## Solve (`sliders chinneck solve`)

A coordinate search with a coarse look at each throat slider first. It holds the jaw measure where the live preset has it, holds the chin's front and bottom (1 mm costs 1), and costs every turned triangle. The fit term is (whole gap / 0.5 mm) squared.

| Head | Live gap (mm) | Jaw sliders alone | With the throat | Throat depth | Corner height | Jaw corner rise | Roundness |
|---|---|---|---|---|---|---|---|
| pmhc04 | 6.1 | 5.6 | 1.2 | 74 | 6.0 | 0.0 | 2.0 |
| Bastila | 4.4 | 3.4 | 1.1 | 76 | 5.2 | 14.2 | 4.4 |
| Carth | 2.0 | 1.8 | 1.2 | 59 | -3.0 | 0.0 | 2.0 |
| Zherron | 3.9 | 3.1 | 1.2 | 75 | 0.0 | 1.2 | 2.4 |
| pmhc03 | 1.4 | 1.4 | 1.3 | 59 | 1.5 | 14.9 | 2.0 |
| Jaesa | 8.0 | 6.0 | 1.5 | 76 | 14.6 | 15.9 | 10.2 |
| the Handmaiden | 7.0 | 5.2 | 1.7 | 74 | 14.8 | 22.5 | 14.8 |
| comm_b_f | 6.8 | 4.8 | 1.8 | 75 | 10.5 | 9.6 | 5.2 |
| sith_app | 4.8 | 4.0 | 2.2 | 62 | 3.2 | 7.1 | 2.0 |
| pfha02 | 4.5 | 3.5 | 0.7 | 70 | 6.6 | 0.0 | 2.0 |

Values and gaps are in `solve10.json`. The jaw values are in `solved10.json` (throat on) and `old10.json` (jaw sliders alone).

## Viewer 2

- http://127.0.0.1:8765/viewer2/index.html?shapes=data/chinneck/shapes.json&shape=chin_neck&head=pmhc04&hair=none
- http://127.0.0.1:8765/viewer2/index.html?shapes=data/chinneck/shapes.json&shape=chin_neck&head=p_bastilah&hair=none
- `shape=chin_neck_jaw` shows the jaw sliders alone, and `shape=chin_neck_live` the live head with the skull refit. Only the 10 solved heads differ. The others show the skull refit's head. There are no live controls for the throat sliders, because the construction is only in Rust. The default page is unchanged.

## Pictures

All in `/private/tmp/claude-501/-Users-david-perso-kjk/c2b598dd-98d5-4f6e-971f-c44f5bf89000/scratchpad/cn/`:
- `all57_r0.png` (middle) and `all57_r3.png` (side view): KOTOR's 57 bare heads with KOTOR's outline in red, live in yellow and the skull-refit head in green. Live and the refit are the same here, because the refit does not move the throat.
- `extremes_pairs.png` and `extremes_fits.png`: check 4.
- `outline_pmhc04.png`, `outline_p_bastilah.png`: the four readings large, KOTOR red, live yellow, solved green.
- `ov_solve.png` (side view) and `outline_10_middle.png` (middle): the 10 solved heads.
- `clay_pb.png`, `clay_solve_a.png`, `clay_solve_b.png`: clay from the side, the front, three-quarter below and below, KOTOR, live and solved.
- `ends_pmhc04.png`, `ends_p_bastilah.png`: each slider at both ends. `pairs_*.png`: every pair at its 4 corners (small).
- `v2/sheets/<head>_<view>.png` for pmhc04, p_bastilah, pmhc03 and p_carthh: viewer 2 in its "In game" light with the hair off, KOTOR, live and solved side by side. The views are `profile`, `side` (close-up), `below`, `three` (three-quarter from below) and `talk` (lip shape 3, live and solved only). The single shots are in `v2/shots/`.

## Not checked

- No judges, as the brief asked.
- The pair corners were only counted (no turned triangle on any of 48 heads) and seen at a small size.
- The talk shape was seen at a small size only, and these shapes keep the rig from the fit, not a rig built on the new throat.
- The 47 heads that were not solved, and the base head with the start values.
- The faces of Jaesa and the Handmaiden after their jaw sliders moved.

## How to undo

- Viewer: delete `/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data/chinneck/`.
- Code, in the head-paint worktree's crate `head_sliders`: delete `src/chin_neck.rs`, its two lines in `src/lib.rs` and the `chinneck` block in `src/bin/sliders.rs`. In `src/jaw_edge.rs`, remove `Heads::live_with`. In `src/throat_corner.rs`, make `Construction::outline`, `ray_hit` and `at_length` private again.
- The build is in `target-chin` in the worktree. Delete it to free 455 MB.
- Roadmap: the line under "Running now".

## Traps

- **zsh does not split an unquoted variable.** `-- $H` passed all heads as one name, and `throat-bins` wrote nothing without an error. Use an array.
- **The hidden vertices under the jaw carry visible skin.** Drawing only triangles whose vertices are not hidden leaves holes under the jaw. `skull show` does that, and it is fine there because it looks at the skull.
- **KOTOR's neck is narrow.** At 30 mm from the middle the section misses KOTOR's neck on most heads, so the sections are 12 and 24 mm.
- **The chin's front.** The front-most place 85 to 125 mm under the eyes is Bastila's lower lip. Read it within 25 mm above the chin's bottom.
- **The live head and the skull refit are the same in the throat.** The refit's move is added after the throat is built (`with_refit`), and it is zero there.
- **The solver hits the ends.** Watch Throat depth's top and Throat roundness's bottom before ranging the sliders from fits.

## Next

David looks at pmhc04 and Bastila in viewer 2. If he likes the profile, the next steps are a row of vertices on the corner line (new topology under the jaw), wider ranges for Throat depth and Throat roundness, a browser version of the construction for live controls, and then the full solve of the 57 heads with the skull sliders.
