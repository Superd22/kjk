# The throat corner: measure and prototype

This note was written on the night of 2026-11-21 by the conversation briefed by the head HQ. The brief is `/private/tmp/claude-501/-Users-david-perso-kjk/52003e36-b8ca-4208-a733-a6d51b417e4e/scratchpad/jaw/brief-throat-corner.md`. The HQ changed the brief during the work: the corner's height became part of the main target. The work serves B3 for the jaw and chin, and prepares B5. Nothing live changed. No preset was refitted. Nothing was committed.

## Verdict

Step 0 passes clearly. KOTOR's bend under the jaw is far tighter than ours on every head that can be read.

The prototype does not pass the gate. In profile it does what David asked: the underside runs back from the chin almost level, to a corner under the ear, and the neck drops from there. From three-quarter below, the judges see a step or ledge across the throat at the bend, and a notch at the jaw's corner. In the second round, scored on their own, the prototype scored lower than live on pmhc04, Carth and pmhc03. It tied on Zherron and was half a point ahead on Bastila.

## What still looks broken, first

- **A step or ledge across the throat at the bend, seen three-quarter from below.** Both judges of round 2 named it on all five heads. On pmhc03 they called it a hard ledge with a lump below it.
- **A notch at the jaw's corner in the side view** on Carth, pmhc03 and pmhc04. This is where the corner line meets the side of the neck.
- **Bumps on the front of the neck below the bend on pmhc03.** The construction pulls the neck's front back hard there, and the neck below it is held by the rim.
- **The first round's judges and its tie-break judges gave no verdict.** The first two judges chose the bottom row on all five heads, in both row orders. The two tie-break judges chose the right side on all ten pictures, in both orders. Every judge chose by position.
- **The base head at the far end of the corner's depth (76 mm)** has a rounded lump on the side of the neck under the ear.
- **The sharpness slider alone does almost nothing on live heads** (see below).

## Step 0: the bend on KOTOR's 57 meshes (measured)

The measure reads the profile in sections parallel to the middle of the face, at 0, ±8 and ±16 mm. In each section it walks along the lowest skin going back from the chin, until the skin falls more steeply than 45 degrees, then down the front-most skin of the neck. The walk starts 12 mm behind the chin's front and ends 34 mm below the chin's bottom. The reading is how far the skin turns over its sharpest 10 mm, in degrees. A corner turns almost all the way in 10 mm. A long, soft bend turns only a little.

At the middle of the throat:

| | Median | Spread |
|---|---|---|
| KOTOR's own mesh | 75° | |
| KOTOR subdivided twice | 93° | 20° |
| Our FLAME fit | 37° | |
| Live (viewer 2's slider heads) | 22° | 7° |
| Base head | 23° | |

- KOTOR is sharper than live on 45 of the 45 heads where both can be read. On every one of them, the gap is larger than the median wobble.
- The median gap is 72°.
- The wobble between KOTOR's mesh and its subdivided copy has a median of 9° and a 95th percentile of 47°. The 95th percentile is larger than KOTOR's spread, so the measure cannot yet tell KOTOR's heads apart. It can show the offset between KOTOR and us, which is three times its 95th percentile wobble.
- The left side against the right side, at 8 mm from the middle, differs by a median of 3° on KOTOR.
- **Facets are not the cause.** The subdivided copy reads sharper than the raw mesh, not softer. KOTOR's median edge in the throat is 21 mm long, and ours is 8.6 mm. KOTOR's corner runs along one row of its vertices.
- The total turn from the chin to the neck is about the same on both: 70° on KOTOR and 58° on live, at the middle. **Our bend is not more open. It is spread over 40 to 75 mm of skin, where KOTOR's is spread over a few mm.**
- KOTOR's corner sits close to the chin's bottom on the five heads: between 5 mm below it and 6 mm above it. It lies 55 to 70 mm behind the chin's front. Carth and pmhc03 have a sharp corner on KOTOR too, at 67° and 98°. Their throats look open only because their necks lean.
- The prototype of the jaw edge (Throat angle and Jaw corner height) already raised the sharpest turn to 63° on pmhc04 and Bastila, and to 70° on Zherron. It left Carth at 15° and pmhc03 at 16°.

Data: `tc2.json` (all heads, all sections) and `scripts/tcturn.py` in the scratchpad below.

## What was built

All of it is in the head-paint worktree, crate `head_sliders`, file `src/throat_corner.rs` (new), run as `sliders throatcorner <command>`. Viewer 2 does not use it.

- **The measure:** `bend_at` and `bend_with` (the two lines and their corner, as the under-jaw measure fits them), `profile` and `turn_at` (the sharpest 10 mm).
- **"Throat corner sharpness" (`Sharpen`), the brief's slider.** In each section across the throat, it draws the skin inside the angle between the measure's two lines toward their meeting point. Its strength runs from -0.5 to 1. It never moves the rim and stays clean at both ends, with no folds. **On live heads it does almost nothing.** It moves pmhc04 from 22° to 38°, Carth from 15° to 22°, and Bastila, Zherron and pmhc03 not at all. The reason is that our underside has no straight part. Both of the measure's lines lie on our long curve, so they meet almost on the skin, and there is nothing to draw in. Tangents taken nearer the ends of the bend were tried too. Our underside curves up into the chin, so the line taken there runs above the skin behind it.
- **The construction (`Construction`, values in `Corner`), the main candidate.** In each section across the jaw, an outline runs from the skin under the chin straight back to a corner, then straight down to the neck's front 34 mm below the chin. The corner is rounded to a set radius. The skin is drawn onto that outline along rays from one place behind and above the throat, so skin cannot pass other skin. The move eases in at both ends of the outline. The construction has five values:
  - **Strength**, 0 to 1. At 0 nothing moves.
  - **Corner depth**, 45 to 76 mm behind the chin's front. At the jaw's corners, 45 mm from the middle, the corner always lies 84 mm back, which is KOTOR's mean jaw corner.
  - **Corner height**, -12 to +15 mm above the chin's bottom.
  - **Jaw corner rise**, 0 to 28 mm, which is how much higher the corner lies under the jaw's corners.
  - **Radius** of the bend, 1 to 25 mm. This is the construction's sharpness.

  It is at full strength within 28 mm of the middle, and fades out by 50 mm.
- **`follow_hidden`.** The mesh has hidden vertices under the jaw (for example 989, 992, 1000, 3341, 3342, 3352, 4039). The build places each from its 6 nearest skin vertices. The zone of `jaw_edge::Proto` gives them a weight of 0, so they never moved, and they made the notches at about 18 mm from the middle. **The last prototype's notches probably had the same cause.** The construction now moves each hidden vertex as its 6 skin vertices move.
- Tools: `dump`, `profiles` (sections of KOTOR, KOTOR subdivided, live and a candidate, with the lines and the sharpest 10 mm), `sheet` (clay rows of KOTOR, live and candidates; `TC_TINT=1` tints by movement and by the zone's weight), `viewer-bins`, `zeros` (the vertices under the jaw that the zone holds at 0).
- `jaw_edge::smoothstep` is now `pub`. Nothing else in `jaw_edge.rs` changed.

## Values set by hand (round 2)

Strength 1 and radius 5 on all five heads.

| Head | Corner depth | Corner height | Jaw corner rise | Sharpest 10 mm (KOTOR, live, candidate) |
|---|---|---|---|---|
| pmhc04 | 66 | 4 | 10 | 91°, 22°, 77° |
| Bastila | 68 | 0 | 14 | 62°, 29°, 86° |
| Carth | 58 | -4 | 11 | 67°, 15°, about 54° |
| Zherron | 68 | -2 | 14 | 57°, 18°, about 70° |
| pmhc03 | 55 | 0 | 10 | 98°, 19°, about 45° |

The values are in `c/m2.json` in the scratchpad. Nothing was solved: the brief's solve step was not reached.

## Ends (seen, in clay)

Each value at both ends on the base head and the 5 stand-ins, and four pair corners. The fold check counted no turned triangles, except at a corner depth of 82 mm before the fix: 3 to 8 triangles on the base head, pmhc04, Bastila and Carth, with a hooked crease at the jaw's corners. Fixing the corner at 84 mm under the jaw's corners and capping the depth at 76 mm removed them. At 76 mm, the base head still has a lump on the side of the neck under the ear. A radius of 1 mm shows a small dent where the corner line meets the side of the neck. The ends were drawn before the round 2 fix and were not drawn again after it.

## Judges

Round 1 used the brief's method: two Fable judges, KOTOR in the middle row, live and the candidate above and below at random, with the rows swapped for the second judge. Judge 1 chose the bottom row on all five heads, and judge 2 chose the bottom row on all five. Both chose the candidate on pmhc04 and Carth with one order and live with the other, so there is no verdict. The tie-break showed enlarged side and three-quarter views side by side, with the sides swapped. Judge 3 chose the right side on all 10 pictures, and judge 4 did the same. Again there is no verdict.

What the judges said about the candidate, whatever its place: a pinch or dark crease along the bend, and a small step just under the chin. About live: a rounded bulge under the chin and a soft bend.

The fix: gathering off (it was crowding vertices onto the corner, which the judges saw as a pinched ridge), a gentler lift toward the jaw's corners (full strength to 28 mm instead of 36, a lower jaw corner rise), and a radius of 5 mm instead of 3 to 4.

Round 2 changed the method, because every judge so far had chosen by position. Each picture shows the target on top and one attempt below, and each picture is scored on its own. The ten pictures (live and the fix, five heads) were shuffled, and the second judge saw them in the reverse order. Scores out of 10, judge A then judge B:

| Head | Live | Fix |
|---|---|---|
| pmhc04 | 5, 6 | 4, 5 |
| Bastila | 5, 5 | 5, 6 |
| Carth | 5, 5 | 4, 4 |
| Zherron | 5, 4 | 5, 4 |
| pmhc03 | 5, 5 | 3, 4 |

## What I saw myself

- In viewer 2's light, in the whole-head profile, the candidate on pmhc04 has KOTOR's level underside and a corner under the ear, where live slopes from the chin down to the neck. Picture: `v2/pmhc04_profile_kotor_live_candidate.png`.
- From below and three-quarter below, the corner is one clean line across the throat after the hidden vertex fix. The line is stronger and more even than KOTOR's own.
- The mouth open (lip shape 2, three-quarter view) showed no tearing on pmhc04 and Bastila. It was seen at small size only.

## Not checked

- Nothing was solved. The values were set by hand on five heads.
- The ends and pairs were not drawn again after the round 2 fix.
- The mouth open was not checked close up, and only with the fit's rig: these shape files keep the rig from the file.
- The pair of the candidate with Throat angle and Jaw corner height was not built. The construction replaces both: it lifts the underside and makes the corner itself. The brief's pair of the sharpness slider with the old prototype's Throat angle was measured on pmhc04 only: 63° with Throat angle alone, and 79° with the sharpness added.
- Viewer 2 has no live control for the values. The link shows fixed shapes.
- In the comparison "Original", `hair=none` does not remove KOTOR's own hair.

## How David can see it

- http://127.0.0.1:8765/viewer2/index.html?shapes=data/throatproto/shapes.json&shape=throat_corner&head=pmhc04&hair=none
- Change `head=` to p_bastilah, p_carthh, n_zherronh or pmhc03.
- The Shape group lists "Throat corner prototype", "Throat corner sharpness alone" and "Live jaw (rebuilt here, for comparing)". "Original" in the comparison buttons shows KOTOR's head. The default page does not change.

## Pictures

All in `/private/tmp/claude-501/-Users-david-perso-kjk/677340bf-c277-4b30-9166-6eeb93af4a90/scratchpad/`:
- `v2/pmhc04_profile_kotor_live_candidate.png` and `v2/p_bastilah_profile_kotor_live_candidate.png`: the profile like David's screenshots, in viewer 2's "In game" light. KOTOR is on the left, live in the middle and the candidate on the right.
- `v2/pmhc04_3q_kotor_live_candidate.png`, `v2/p_bastilah_3q_kotor_live_candidate.png`: three-quarter.
- `v2/shots/`: every viewer 2 shot, with the mouth open in `*_open.png`.
- `pm1.png`: sections at the middle of KOTOR (red), KOTOR subdivided (orange), live (grey) and the candidate (white).
- `prof2.png`: KOTOR against live sections, with the sharpest 10 mm in blue.
- `j1/`, `j2/`, `j3/`: the judges' pictures, with `key.json` in each.
- `ends/`: the ends and pairs.

## How to undo

- Viewer: delete `/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data/throatproto/`.
- Code: delete `src/throat_corner.rs`, the two lines `#[cfg(not(target_arch = "wasm32"))]` and `pub mod throat_corner;` in `src/lib.rs`, and the `throatcorner` block in `src/bin/sliders.rs`. Make `smoothstep` in `src/jaw_edge.rs` private again.
- Roadmap: the line under "Running now".

## Traps

- **Our bend is long, not open.** Two lines fitted at fixed places (8 to 20 mm behind the chin, 20 to 32 mm below it) both lie on our curve. Their corner then lies almost on the skin. The gap and the radius read from it see nothing, and a slider built on them has nothing to move. Read the turn along the skin instead.
- **The ray walk must start deep in the neck.** A centre near the underside missed pmhc04's corner. The walk along the lowest skin, then the front-most skin, is more reliable. Read directions over 2 mm chords, or the switch between the two reads as a turn and a turn back.
- **Hidden vertices.** The zone of `jaw_edge::Proto` holds every hidden vertex at 0. Under the jaw that leaves stuck patches, which make notches. Move them with their 6 nearest skin vertices (`follow_hidden`).
- **The silhouette in profile comes from the whole jaw.** A construction only on the middle 30 mm of the throat barely changes the side view. It must reach the jaw's corners.
- **Do not set the corner further back than the jaw's corner at the sides.** Beyond 84 mm it makes a hooked crease there.
- **Fable judges choose by position.** All four judges of round 1 chose one position on every picture, in both orders, including on enlarged side-by-side crops. Scoring each attempt on its own against the target, with the pictures shuffled, gave different scores per head. Use that method from the start.
- **Gathering vertices onto the corner** reads as a pinched ridge. KOTOR gets its clean edge from a row of vertices that already lies on the corner.
- **The worktree is shared.** Another conversation added `bald.rs` to the same crate during this work, and the build failed for some minutes until it was fixed.

## Next

The idea holds in profile, which is what David asked about. The ledge across the throat from three-quarter below is what loses the judges. I see three ways forward:
1. Lay a row of vertices on the corner line, as the eye curve lays the rim on its curve, instead of drawing skin to it along rays.
2. Give the move's ease toward the neck a longer reach, so the neck below the corner is not left bulging.
3. Ask David to judge the profile in viewer 2 before spending another round, because the judges were unreliable here.
