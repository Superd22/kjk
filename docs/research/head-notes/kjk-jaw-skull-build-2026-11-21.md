# The jaw-to-neck and skull build, solved on all 57 heads

This note was written on the night of 2026-11-21 to 2026-11-22 by the conversation briefed by the head HQ. The brief is `/private/tmp/claude-501/-Users-david-perso-kjk/52003e36-b8ca-4208-a733-a6d51b417e4e/scratchpad/jaw/brief-jaw-skull-build.md`, with two changes the HQ sent during the work: one forehead-to-skull profile measure, and folding the result into viewer 2's default. The work serves B3 and B5 for the jaw, the chin and the skull, and A2. Nothing live changed. Nothing was committed. No judges were run.

The scratchpad is `/private/tmp/claude-501/-Users-david-perso-kjk/0d22d722-33db-4847-9b34-38ddd4a6d394/scratchpad/` (called `S` below).

## Verdict

All 57 heads are built from slider values alone: the 5 jaw sliders, 4 throat sliders and 4 new skull sliders. Every head passes its own guard, except that Kreia keeps only a quarter of her skull values. The result is in viewer 2 behind a URL option. It is not viewer 2's default yet, because the throat construction runs only in Rust (see "Folding into the default").

## What is still broken, first

- **The throat sliders' pair corners turn triangles.** At two sliders' ends at once, 39 of the 44 readable heads turn 2 to 21 triangles. The worst pairs are Throat corner height high with Jaw corner rise high, and Throat depth high with Throat roundness low. No head turns a triangle at its own solved values. Three repairs were tried, and then I stopped as the brief asks:
  - the low end of Throat roundness was capped at 6 mm;
  - the high end of Throat corner height was capped at 12 mm;
  - the outline under the jaw's corners may rise at most 32 mm above the chin's bottom.
- **The ledge is not measured.** A front-view reading (the jaw's half-width less the neck's, 25 mm under the chin) fails check 1. The same head measured twice differs by up to 6.3 mm (95th percentile), and its two sides by up to 10.4 mm. The spread between heads is 10.5 mm. Read this way, live's ledge is only 1.9 mm under KOTOR's at the median, so no slider was added.
- **pmhc04's forehead leans back less than KOTOR's.** The skull sliders do not move the forehead's front. See "The forehead slope".
- **pmhc04's skull is narrow and egg-shaped from the front.** That is the bald reference's shape (125 mm wide), not a step.
- **pfha01's new reference** has a filled forehead with no real skin under its fringe. It is the least certain target. Its brow-style forehead slope reads 83°, which is not believable.
- **One head turns triangles at the skull sliders' ends.** pmhc03 turns 3 at the high end of Skull length.
- **Kreia** keeps a quarter of her skull values. At her full values the skull stands outside her hood at 15 places, against 11 today.

## The fixes

1. **The jagged corner line: fixed by sliding existing vertices. No vertex was added.** In `ThroatShape::apply` (`chin_neck.rs`), two steps follow the smooth field:
   - The snap lays each pulled vertex onto the outline along its ray, as far as the outline pulls it. Before, the vertices nearest the corner stopped 5 to 10 mm short of it, each by a different amount.
   - The slide finds a row of the mesh's vertices across the throat: the cheapest path over the edges, nearest the corner, where every step goes at least 2 mm further across. Each section is then remapped along its outline so the row lands on the corner. The remap keeps the vertices' order and fades out within 26 mm.
   - The snap is applied only when a row is found. Otherwise a sharp corner across the mesh's rows shows as a sawtooth.
   - In viewer 2 close-ups from below and three-quarter, pmhc04, Bastila and pmhc03 show one straight corner line, like KOTOR's. Pictures: `S/w/v2/*_belowz.png` and `*_threez.png`. The columns are KOTOR, old, snap alone, and snap with slide.
2. **The bump where the corner meets the neck's side.** The hidden vertices under the jaw now move with the construction. Those below 100 mm under the eyes are laid on the outline like skin, and the rest follow their 6 skin neighbours. The fold guard no longer finds anything there. It was not checked again close up in viewer 2's light after the final solve.
3. **The sliders at their range ends.** With the snap, the corner is truly sharp, so the solver no longer runs to the ends. Over the 44 readable heads, Throat roundness runs from 6 to 25 mm (median 16), and Throat depth from 50 to 84 mm (median 67).
   - The new ranges are: depth 45 to 84 mm, corner height −12 to 12 mm, jaw corner rise 0 to 28 mm, and roundness 6 to 25 mm.
   - Depth's top is 84 mm because the corner already lies 84 mm back under the jaw's corners.
4. **Jaesa and the Handmaiden.** No nose, brow, cheek or jaw proportion moved beyond its mark on any head, these two included. Their faces look unchanged in three-quarter (`S/fin/v5/jh.png`). The cause was the old soft corner: the jaw sliders bent to make up for a corner the construction could not reach.
5. **The ledge.** Not added, because it fails check 1 (see above).
6. **The skull.**
   - **The shoulder at the temples.** Skull width now fades in from the eyes' height to 60 mm above them, and to nothing in front of the forehead's sides. The temples narrow with the skull, and the corner seen from the front on pmhc04 and in three-quarter on pfha06 is gone (`S/fin/v4/`).
   - This moves the brow feature's temple width on purpose, so that row is left out of the guard. The brief describes the fault as "the held forehead meets the narrower temples", so the temples are meant to narrow.
   - **pfha01's reference.** The face box of its hair cut now ends 18 mm above the eyes instead of 32 (`bald.rs`, `painted_hair`). The fringe is cut and the forehead is filled. The new references are in `S/sk/ref`.

## The forehead-to-skull profile (the HQ's change 1)

- **The measure.** The middle cut of the head is read along 61 rays from one place inside the skull (80 mm behind the eyes, 20 mm above them), every 3° from straight ahead over the top to straight back. The radii are not lined up or scaled, so a step at the hairline reads as a miss. Code: `skull_profile` in `jaw_skull.rs`.
- **Check 1 passes.** The same head measured twice (KOTOR against its subdivided copy, 36 bald heads) differs by a median of 1.22 mm (95th percentile 1.88). Two bald KOTOR heads differ by a median of 6.22 mm (5th percentile 2.90).
- **Check 3 passes.** Each subdivided copy finds its own head for 36 of 36.
- **What it found.** Live misses KOTOR by 4.5 mm (median) even on the bald heads. The overlay `S/sk/ov0.png` shows our top and back lying 4 to 5 mm inside KOTOR's on Carth, comm_a_m and Zherron.
- **The forehead slope.**
  - The two readings agree across heads: the skull measure's slope (brow to 45 mm above it) and the brow feature's way (a line fitted 30 to 62 mm above the eyes) have a correlation of 0.85 over 57 heads.
  - On pmhc04's target they disagree: 20° by the skull reading and 30° by the brow reading. Ours is 15° by both today, and 17° and 18° after the skull sliders.
  - So pmhc04's forehead is too upright by 3° to 12°, and the skull sliders cannot fix it because they hold the forehead's front.
  - **The brow side needs a change** (not made, `brow.rs` untouched). The brow preset's Forehead slope should be solved against the bald reference's forehead on the 21 haired heads. Today the brow measure hides the forehead where KOTOR's hair covers it.

## The skull sliders

They are four fixed, smooth movements of the base head's vertices, `skull_moves` in `jaw_skull.rs`. A head's skull values add the smoothed movements (`skull_world_fields`). They are ordinary linear sliders, so they would run in the browser as they are.

| Slider | What one unit does | Range (the fits' lowest and highest values) |
|---|---|---|
| Skull width | Widens the skull 1 mm at its widest. It fades in from the eyes' height to 60 mm above them, and to nothing within 25 mm of the middle and in front of the forehead's sides. | −33 to 26 |
| Skull length | Moves the back of the head 1 mm back. It fades in from 90 to 150 mm behind the eyes, so it does not move the ears, and to nothing down the nape. | −19 to 11 |
| Skull height | Raises the top 1 mm. It grows with the square of the height above 40 mm. | −19 to 30 |
| Crown place | Moves the top 1 mm forward. It has the same fade as Skull height. | −25 to 31 |

- **The solve** (`skull-solve`) targets KOTOR's skull on the 36 skull-showing heads and the bald reference on the 21 haired heads. It fits the profile and the skull's width.
- **The profile gap** falls from 4.5 to 2.3 mm (median) on the bald heads, and from 3.5 to 1.6 mm on the haired heads.
- **pmhc04** goes from 7.0 to 1.0 mm, and is 125.1 mm wide against the reference's 125.0. The refit reached 2.1 mm.
- **Bastila** goes from 2.6 to 2.2 mm, and is 151.9 mm wide against 151.9.
- **The heads that changed most** are Canderous of the mod (up to 33 mm, his top is raised 28 mm) and pmha02 (28 mm, crown 31 mm forward). Their KOTOR skulls are really that high and that far forward (`S/sk/ov1.png`).

## The solve and the guard on all 57 heads

- **Throat and jaw** (`throat-solve`, `S/ts/all3/`):
  - All 44 heads with a readable outline take their solved jaw and throat values. The outline gap falls from 4.3 mm (median, live) to 1.2 mm, with the worst head at 2.3 mm. pmhc04 goes from 6.1 to 1.3 mm, Bastila from 4.4 to 0.8, Jaesa to 1.2 and the Handmaiden to 1.6.
  - The 13 heads whose outline cannot be read keep their jaw preset and take the median throat values (67, 0.8, 10, 16). On n_dorakh and old_b_m the median turned triangles, so they take a slightly deeper corner.
- **Skull** (`final`, `S/fin/f9/final.json`): 56 heads take their full skull values. Kreia takes a quarter.
- **What the guard checks on each head:**
  - The nose, brow (except temple width), cheek and jaw proportions stay within their marks of the head without the new values.
  - No triangle turns at the solved values.
  - On the haired heads, the skull stands outside KOTOR's hair by more than 2 mm at no more places than today. It now does so at fewer places on every haired head: pmhc04 goes from 44 to 0, Terena from 126 to 27 and pmhc03 from 70 to 31.
- **The ends and pair corners** of every throat and skull slider were counted on every head, with the results listed above.
- **What was seen** in viewer 2's "In game" light with the hair off: pmhc04, Bastila, pmha02, Canderous of the mod, Jaesa and the Handmaiden, in profile, front, three-quarter, zoomed from below, zoomed three-quarter and talking (`S/fin/v2/`, `S/fin/v5/`).

## Viewer 2

The default page is unchanged.
- http://127.0.0.1:8765/viewer2/index.html?shapes=data/jawskull/shapes.json&shape=jaw_skull&head=pmhc04&hair=none
- http://127.0.0.1:8765/viewer2/index.html?shapes=data/jawskull/shapes.json&shape=jaw_skull&head=p_bastilah&hair=none
- http://127.0.0.1:8765/viewer2/index.html?shapes=data/jawskull/shapes.json&shape=jaw_skull&head=pmha02&hair=none
- http://127.0.0.1:8765/viewer2/index.html?shapes=data/jawskull/shapes.json&shape=jaw_skull&head=mod_narve_canderous&hair=none

`shape=jaw_skull_live` shows today's head rebuilt from the same files. All 57 heads are there. These shapes keep the rig from the fit. There are no live controls.

Pictures:
- `S/out/outline_pmhc04_bastila.png`: the chin-to-neck outlines in profile, KOTOR in red, today in yellow and new in green, for the four readings. The labels give today's (T) and the new (N) gap.
- `S/out/skull_pmhc04_bastila.png`: the forehead-to-skull profile, KOTOR or the reference in red, today in yellow and new in green.

## Folding into the default (the HQ's change 2): not done

Viewer 2's default builds each head live from slider values in WebAssembly (`web.rs` `Builder`, `sliders.js`). The skull sliders would work there as they are. The throat construction would not. It needs the mesh's merged vertices, their neighbours and the hidden vertices' nearest skin in the browser, and a new kind of slider whose four values feed one construction. That is a port of `ThroatShape`, `chin`, the outline helpers of `throat_corner.rs` and `jaw_edge::Proto`'s weight into code the browser builds, plus new data files and a new feature file in `sliders.js`.

I did not start it, for two reasons. It is a change to the live page, and the brief also asked not to change live. The pair corners also still fold, and a live slider would let a player reach them. The superseded prototypes (`jawproto`, `throatproto`, `skullrefit`, `chinneck`) were not deleted, because their deletion goes with the fold.

## What going live needs

- The presets file: the jaw values change on 44 heads. The new throat values are in `S/ts/all3/values.json` and the skull values in `S/fin/f9/final.json` (`skull` per head).
- The browser port of the throat construction, and the throat and skull sliders as features in `sliders.js`.
- The animation calibration again (decision 62), because the jaw, the throat and the skull change.
- The part D hair caps placed again on the new scalps.
- A check of the paint's hairline on the new scalps.
- The brow preset's Forehead slope solved against the bald reference on the haired heads.

## How to undo

- **Code**, in the head-paint worktree's crate `head_sliders`. The originals are in `S/orig/`.
  - Delete `src/jaw_skull.rs`, its two lines in `src/lib.rs` and the `jawskull` block in `src/bin/sliders.rs`.
  - Copy back `src/chin_neck.rs` (snap, slide, hidden vertices, ranges and `SIDE_TOP`), `src/throat_corner.rs` (`path_length` was made public) and `src/bald.rs` (pfha01's face box).
- **Viewer:** delete `/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data/jawskull/`.
- **The private build copy:** `S/ws/` and `S/sync.sh`. It copies the five head crates and patches a missing `Debug` derive in the nose conversation's `nose_base.rs`, in the copy only.
- **Roadmap:** the line under "Running now".

## Traps

- **The shared crate breaks while other conversations edit it.** `nose_base.rs` was mid-edit, and `head_sliders` gained a `head_paint` dependency. Build from a private copy (`S/sync.sh`, then cargo in `S/ws`).
- **The smooth field never reaches a sharp corner.** The vertices nearest the corner stopped 5 to 10 mm inside it. Snap them onto the outline, but only with a row on the corner, or the corner shows as a sawtooth.
- **Rows of the mesh under the jaw.** Only one row spans the throat from side to side on most heads. The row nearer the corner in the middle ends about 18 mm out, joined to the next row only by edges that run along the throat, not across it. A row that steps along such an edge lands two vertices on one place. Require every step to go 2 mm across.
- **Hidden vertices under the jaw** placed from their skin neighbours land inside a sharp corner and turn the triangles around them.
- **A guard that counts ends and pair corners per head** sends heads back to their old values for a fault of the ranges. Guard each head on its own values, and report the ends apart.
- **The skull length field sheared the ears** when it started 40 mm behind the eyes. Spreading the movement over the ear area made it worse. Starting the field behind the ears fixed it.
- **A leftover `max(1.0)`** in the width field moved the whole head, jaw included. The guard caught it as jaw width −13 mm.
- **"Outside KOTOR's hair" must compare with today.** Today's skull already stands outside the hair on most haired heads.
- **Live's bald skulls are 4 to 5 mm inside KOTOR's top and back** on many bald heads. The earlier row measure reported this only as "top 3 mm low".

## Next

David looks at the four links. If he likes them, the next conversation ports the throat construction to the browser, repairs the pair corners, makes the result viewer 2's default, and deletes the superseded prototypes.
