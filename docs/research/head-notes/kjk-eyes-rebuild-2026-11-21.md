# The eye rebuild with a larger eyeball set further back, step 1 (2026-11-21)

Written by the conversation kjk-d8 for the lead kjk-48. The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/cca6a419-40ae-4274-8444-32656ff8bf71/scratchpad/e1/`, called `$E` below.

## Verdict

Step 1 failed, so step 2 was not built. Nothing was published. Viewer 2 stays at `?b=48`. No file in the worktree, the live data or the shared kit changed.

- The brief's premise does not hold on the two heads that matter. KOTOR's own eyeball meshes (`eyes/kotor_eyes.json`) show that pmhc04's eyeball centre sits 1.5 mm in front of ours, not behind it, and Bastila's sits in the same place as ours. Both of KOTOR's eyeballs are only about 1 mm larger than ours.
- The idea also works against its own goal. If an eyeball grows and moves back so that its front stays in place, the eye's corners have to come forward to stay on it. The eyeball then shows as a shorter wedge in profile, not a longer one. The pictures confirm this.
- Neither prototype was clearly better than live in the pictures. The two blind judges preferred live on pmhc04, and they called Bastila a tie.

## What KOTOR's eyeballs really are

The command `balls` in my copy of the diagnosis tool reads each eyeball's centre and radius in the head's frame, in mm, for ours (live, `?b=48`) and for KOTOR's own eyeball mesh. The +x eye is used. The full list is `$E/balls_live.txt`, and `$E/stats.py` prints the summary.

| Head | Our centre depth | KOTOR's centre depth | Our radius | KOTOR's radius |
|---|---|---|---|---|
| pmhc04 | -0.2 | 1.3 | 12.2 | 13.2 |
| p_bastilah | -3.1 | -3.2 | 15.4 | 16.5 |
| pmha01 | 4.4 | 4.7 | 12.7 | 13.6 |
| p_carthh | -2.1 | -0.9 | 12.9 | 13.3 |
| p_kreiah | -2.1 | -2.3 | 13.6 | 15.2 |
| n_zherronh | 3.0 | 4.1 | 13.0 | 11.0 |

Over all 57 heads, KOTOR's centre lies 0.8 mm further back than ours at the median. A tenth of the heads have it 7.5 mm or more further back, and another tenth have it 2.3 mm or more further forward. KOTOR's radius is 1.6 mm larger at the median, and 6.4 mm larger on the tenth with the largest eyeballs. So "5 to 10 mm further back" is true only for a few heads with very large eyeballs, and not for pmhc04 or Bastila.

KOTOR's outer corner sits at about the depth of its eyeball's centre, on the line the blink turns about. On pmhc04 it lies 0.4 mm in front of KOTOR's centre, and on Bastila 0.6 mm in front. The eyeball's radius there is close to half the eye's length, so the corners lie on the eyeball's equator.

## Why a larger eyeball further back cannot give the long wedge

Seen from the front, the outer corner lies about 13 mm to the side of the eyeball's centre on pmhc04. A corner must lie on the eyeball's surface or outside it. On an eyeball with a 12 mm radius it can sit at the equator, level with the centre. On an eyeball with a 16 mm radius whose centre is 4 mm further back, the surface at 13 mm to the side lies 6.7 mm in front of the old equator. So the corner has to come forward by about 6.7 mm, and the wedge seen in profile gets shorter. The longest wedge comes from the smallest eyeball whose equator reaches the corners. KOTOR's eyeballs are sized like that.

## The prototypes

The prototype is a step in my copy of the diagnosis tool, `reseat` in `$E/diag/src/main.rs`. Viewer 2 does not use it. It runs after the head is built and before the lid piece is made. It gives each eyeball a new centre and radius. Each piece of skin within 3 mm of the opening's edge moves in depth only, fading out by 12 mm from the edge. It keeps its place seen from the front and its gap to the eyeball, so the opening's outline from the front does not change. The lid piece and the blink are then built from the new eyeball as usual.

- **A, the brief's construction.** The eyeball is 2 mm or 4 mm larger, and its centre is 2 mm or 4 mm further back, so that its front stays in place. At 4 mm the eyeball and the socket lining came out through the temple. Cutting the eyeball down to its front 80 degrees and fixing the lining's move removed that. At 2 mm the profile wedge on pmhc04 and Bastila is shorter than live, and Bastila gets a small tab at the outer corner (`$E/pics/s_p3.png`).
- **B, KOTOR's own eyeball.** Each eyeball takes KOTOR's centre and radius for that head. The profile wedge grows on pmhc04 and Kreia, but pmhc04's eye looks larger and rounder from the front. Zherron's eyeball is 2 mm smaller than ours, so the skin at its outer corner tears, and the closed lid leaves a gap there.
- **B with round 7's Outer corner reach** at 0.5 and 1 did not change pmhc04. At 1 it gives Bastila the same rounded bulb that round 7 saw.

Pictures, with KOTOR, live and B on the six heads in five views (front, three-quarter from the temple, three-quarter from the nose, temple side and profile), with no hair:
- `$E/pics/s_B_open_lit.png`: the eyes open, in light.
- `$E/pics/s_B_open_clay.png`: the eyes open, in clay.
- `$E/pics/s_B_closed_lit.png`: the eyes closed.
- `$E/pics/s_B_left_lit.png` and `$E/pics/s_B_right_lit.png`: the eyes glancing left and right.
- `$E/pics/s_p3.png`: pmhc04 and Bastila with A at 2 mm, B, and B with Outer corner reach.

## The judges

Two Opus judges each saw 8 sheets, with KOTOR in the middle. Each sheet holds the eyes open in light, open in clay, closed, and glancing. The sides were drawn at random, and the second judge saw them swapped. The sheets are `$E/judge/*_j1.png` and `*_j2.png`, and the sides are in `$E/judge/key.json`.

| Head | Pair | Judge 1 | Judge 2 |
|---|---|---|---|
| pmhc04 | B against live | live, 3 to 2 | live, 3 to 2 |
| pmhc04 | A against live | tie | tie, leaning A |
| p_bastilah | B against live | tie, leaning B | tie |
| p_bastilah | A against live | tie, leaning A | tie |
| pmha01 | B against live | live, 3 to 2 | live |
| p_carthh | B against live | B, because live's closed lid shows a dark dash | B |
| p_kreiah | B against live | B | live |
| n_zherronh | B against live | live, 3 to 1 | live, 3 to 1 |

- Both judges found B's pmhc04 eye too tall and round, with a fold or slit at the outer corner when it is closed.
- Both found Zherron's B eye torn at the outer corner.
- On Bastila, both judges said that every candidate is far rounder than KOTOR's and lacks KOTOR's heavy lid over the top of the iris. That fault is the same in live and in both prototypes.

## What changed

Nothing in the worktree, the live data, the shared kit or viewer 2. All the work is in `$E`.
- `$E/diag` is a copy of round 7's diagnosis tool, built into `$E/diagt`. It adds the commands `balls` and `proto`, the step `reseat`, glances (the iris recoloured toward a turned direction) and `cap_balls`. It still links the crates from the worktree.
- `$E/crop.py` re-packs a sheet, and `$E/judge/make.py` makes the judges' sheets.
- The roadmap got one line under "Now and next" and decision 64.

## How to undo

Nothing needs undoing. To undo the roadmap, remove the line on the eye rebuild under "Now and next" and decision 64.

## Traps I hit

- In zsh, a list held in a plain variable does not split into words. Use an array, `H=(a b c)` and `$H`.
- The socket lining behind the eyeball must move like the front point in line with it. If it is mapped onto the back of the new sphere, it jumps 10 mm back at the equator and comes out through the temple.
- Our eyeball is drawn as 0.6 of a sphere. A larger eyeball set back pokes through the temple unless it is cut to its front.
- The diagnosis note's "KOTOR's eyeballs sit 5 to 10 mm further back" came from the "ball" measure, which reads KOTOR's front as a radius about our centre. It is not KOTOR's real eyeball.

## For a follow-up

- The construction that matches KOTOR's eyeballs has the eyeball's equator reach the eye's corners. That means an eyeball radius of about half the eye's length, with the corners level with the eyeball's centre, so the blink's turning line runs through them. A closing lid that turns about that line covers an opening that ends there. Read each head's half length against its eyeball radius before trying it.
- The fault both judges see on every head, Bastila above all, is that our openings are too tall and round, with no heavy lid over the iris. That is the lid piece and the opening's height, not the eyeball.
- Zherron's eyeball in KOTOR is smaller than ours, so a slider that shrinks the eyeball needs the corner skin to follow it.

## For David

- **Viewer 2 did not change.** It is still `?b=48`.
- **The rebuild was stopped after the test pictures.** Neither version looked clearly better than the current eyes, and the judges preferred the current pmhc04.
- **KOTOR's eyeballs are not set further back than ours on pmhc04 or Bastila.** I measured KOTOR's own eyeball meshes. pmhc04's eyeball in KOTOR sits 1.5 mm further forward than ours and is 1 mm larger. Bastila's sits in the same place as ours and is 1 mm larger. Only a few heads with very large eyeballs have them far back.
- **A larger eyeball set back makes the profile wedge shorter, not longer.** The front of the eyeball stays in place, so the corners of the eye must come forward to stay on its surface. The pictures show this.
- **Using KOTOR's own eyeball on each head gives a slightly longer wedge on pmhc04 and Kreia.** But pmhc04's eye then looks larger and rounder from the front, and Zherron's outer corner tears.
- **Both judges saw the same fault on every head, Bastila most of all.** Our eyes are too tall and round, and they lack KOTOR's heavy upper lid over the top of the iris. That is the next thing to fix, and it is the lid, not the eyeball.
- **Pictures:**
  - `/private/tmp/claude-501/-Users-david-perso-kjk/cca6a419-40ae-4274-8444-32656ff8bf71/scratchpad/e1/pics/s_B_open_lit.png` shows KOTOR, the current eye and KOTOR's own eyeball on six heads, with the eyes open.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/cca6a419-40ae-4274-8444-32656ff8bf71/scratchpad/e1/pics/s_B_closed_lit.png` shows the same heads with the eyes closed.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/cca6a419-40ae-4274-8444-32656ff8bf71/scratchpad/e1/pics/s_p3.png` shows pmhc04 and Bastila with the larger eyeball set back.
