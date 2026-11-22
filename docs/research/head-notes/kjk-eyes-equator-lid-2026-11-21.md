# The eye corners on the equator and the heavy upper lid, step 1 (2026-11-21)

Written by the conversation started from the brief `eyes-equator-lid` for the lead kjk-48. The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/ea0d1303-868f-43c4-b8fe-71c29bfc78e9/scratchpad/q/`, called `$Q` below.

## Verdict

Step 1 stopped, so step 2 was not built. Nothing was published. Viewer 2 stays at `?b=48`. No file in the worktree, the live data or the shared kit changed.

- Step 0 showed that both claims of the brief hold only in part on KOTOR's 57 meshes. The details are in the next section.
- The prototype used only existing sliders, set by hand, plus round 7's Outer corner reach at 0.5. It raised the lower lid over the bottom of the iris on five heads, raised pmhc04's upper lid, and set Bastila's, Carth's and Kreia's outer corners back.
- Both blind judges preferred the prototype on pmhc04. On Bastila they split. One preferred the prototype by a moderate margin, and the other preferred live by a slim margin, because the prototype leaves a skin tab at Bastila's outer corner. The brief asks for a clear win on both heads, so I stopped.
- Both judges still found every candidate, live and prototype alike, too open and too round next to KOTOR on every head.

## What KOTOR's meshes show (step 0)

The command `equator` in my copy of the diagnosis tool reads each eye from the front, on KOTOR's head with its own eyeball and lid piece, and on viewer 2's head. The opening is every pixel where the eyeball or the caruncle is the front-most surface. The iris is drawn as in all our sheets, as the part of the eyeball within 0.42 radians of straight ahead. Each value is the mean of both eyes. The full output is `$Q/eq_live.json`, and `$Q/eqsum.py` prints the summary.

| Measure | What it reads | KOTOR median | KOTOR spread (p10 to p90) | Ours median |
|---|---|---|---|---|
| outer_dz | The outer corner's depth in front of the eyeball's centre, in radii | 0.15 | -0.02 to 0.39 | 0.24 |
| inner_dz | The inner corner's depth in front of the centre, in radii | 0.56 | 0.33 to 0.75 | 0.54 |
| outer_dx, inner_dx | Each corner's distance to the side of the centre, in radii | 0.89, 0.73 | | 0.96, 1.06 |
| r_over_half | The eyeball's radius against half the opening's length | 1.26 | 1.12 to 1.47 | 0.99 |
| iris_up_hidden | The share of the iris's upper half that the lids hide | 0.55 | 0.12 to 0.81 | 0.50 |
| iris_lo_hidden | The share of the iris's lower half that the lids hide | 0.15 | 0.02 to 0.52 | 0.01 |
| lid_bot | The opening's bottom below the centre, in iris radii | 0.79 | 0.47 to 1.04 | 1.01 |
| h_over_len | The opening's height on the centre's column against its length | 0.32 | 0.23 to 0.37 | 0.29 |

What this means:
- **The outer corner sits near the equator in KOTOR.** On 42 of 57 heads it lies within 0.3 radii of the centre's depth. Ours sits only 0.4 mm further forward at the median, and pmhc04's already sits on it (KOTOR 0.01 radii, ours 0.09). The 17 heads that miss it by more than 0.15 radii include Bastila (KOTOR -0.08, ours 0.46, about 8 mm), Kreia (0.20 against 0.51) and Carth (0.05 against 0.34).
- **The inner corner is not on the equator in KOTOR.** It sits 0.56 radii in front of the centre, and only 5 heads have it within 0.3 radii. Ours matches.
- **KOTOR's eyeball is not half the eye's length.** Its radius is 1.26 times half the length, and the eyeball sits closer to the inner corner than to the outer one. Ours is already 0.99 times. KOTOR's eyeballs are only 16 to 33 flat triangles, so their surface lies well inside the radius of their corners.
- **KOTOR's upper lid does not hide more of the iris than ours at the median.** KOTOR hides more on 17 heads and less on 25. On pmhc04 KOTOR hides 11% of the iris's upper half and ours 34%. On Bastila the opening's top lies 1.5 mm above the eyeball's centre in both.
- **Our openings are not taller for their length.** The ratio is 0.32 in KOTOR and 0.29 in ours.
- **The lower lid is the real global difference.** KOTOR's lower lid rides up over the bottom of the iris. Ours sits lower on 46 of 57 heads, by 0.26 iris radii at the median, and shows white under the iris. pmhc04 is the strongest case: KOTOR hides 44% of the iris's lower half and ours 4%.
- **Bastila's heavy lid is the lid piece, not the lid's height.** The diagnosis note measured it already: KOTOR's piece juts 8.6 mm in front of her eyeball and ours 1.9 mm.

The four checks:
- The wobble between the left and right eye is 10 to 30 times smaller than KOTOR's spread on every measure.
- The measures see what David sees: Bastila's corner is far forward, and pmhc04's and Kreia's irises show white below.
- KOTOR's spread is large on every measure.
- The extremes look right in `$Q/pics/extremes.png`, with one exception. The two heads with the most hidden upper iris (mod_narve_canderous and mod_tcs_bastila) are mods with odd geometry over the eyeball.

## The prototype (step 1)

The prototype is a set of hand-set values in `$Q/specP.txt`, read by the diagnosis tool's `set` variant. It runs round 7's Outer corner reach at 0.5 on every head. No code in the worktree changed.
- pmhc04: Upper lid, middle point +1.2, Lower lid drop, middle at -2.5, and Lower lid drop, inner and outer, each 0.8 higher.
- Bastila: Outer corner depth at -4 and Upper lid cover at 3.5.
- Carth: Outer corner depth at -3 and the middle of the lower lid 0.6 higher.
- Kreia: Outer corner depth at -4 and the lower lid 1.2 higher in the middle and 0.6 higher at the outer quarter.
- pmha01: the lower lid 1.0 higher in the middle and 0.5 higher at the outer quarter.
- Zherron: the lower lid 0.5 higher in the middle and Upper lid, middle point +0.8.

What it reached (`$Q/tP.json`, KOTOR against the prototype):
- On pmhc04, the lids now cross the iris where KOTOR's do (lower half hidden 0.44 against 0.43, upper 0.11 against 0.13). The opening becomes 6.1 mm tall against KOTOR's 7.05.
- Bastila's outer corner moves only 1 mm back (0.46 to 0.41 radii, KOTOR -0.08). Outer corner depth stops at -4 mm, and Outer corner reach stops 4 mm in front of the eyeball's centre.
- A thicker lid piece (its least edge thickness at 2.2 mm instead of 0.8) made Bastila's lid heavier, but it brought back a skin tab at the outer corner. I left it out of the judged candidate.

## The judges

Two Opus judges each saw 6 sheets, with KOTOR in the middle. Each sheet holds the eyes open in light, open in clay, closed, and glancing left and right, in five views, with no hair. The sides were drawn at random, and the second judge saw them swapped. The sheets are `$Q/judge/*_j1.png` and `*_j2.png`, and the sides are in `$Q/judge/key.json`.

| Head | Judge 1 | Judge 2 |
|---|---|---|
| pmhc04 | prototype, small margin | prototype, clear margin |
| p_bastilah | prototype, moderate margin | live, slim margin, because of the prototype's tab |
| pmha01 | prototype, clear margin | prototype, modest margin |
| p_carthh | prototype, small margin | tie |
| p_kreiah | prototype, small margin | prototype, clear margin |
| n_zherronh | live, tiny margin | tie, leaning to the prototype |

- Both judges said that both candidates are too round and too open on every head. They meant the whole iris showing, as on KOTOR's heads with a hooded lid.
- Both saw a black dash under the closed lower lid on Bastila and Carth, in live and in the prototype.
- Judge 2 noted the large pink caruncle at our inner corner, where KOTOR shows a pointed white corner.

## What changed

Nothing in the worktree, the live data, the shared kit or viewer 2. All the work is in `$Q`.
- `$Q/diag` is a copy of the rebuild's diagnosis tool. It adds the command `equator`, the function `set_values` (proto variants `set:<spec>:_:<reach>:<thickness>`, and `EQ_SET` for `equator`), and a lid piece thickness override. It still links the crates from the worktree.
- `$Q/eqsum.py` and `$Q/eqone.py` print the measures. `$Q/judge/make.py` makes the judges' sheets.
- The roadmap got one line under "Now and next" and decision 65.

## How to undo

Nothing needs undoing. To undo the roadmap, remove the line on the eye corners on the equator under "Now and next" and decision 65.

## Traps I hit

- KOTOR's eyeball meshes have only 16 to 33 triangles. Their vertices lie on the sphere of `kotor_eyes.json`, but their faces lie well inside it. A corner at 0.89 radii to the side lies at 0.15 radii in front of the centre, not at 0.46 as on a sphere.
- Our sheets draw every iris, KOTOR's too, at 0.42 radians round the eyeball's forward axis. Viewer 2 paints the iris itself, with a size in mm (Iris size, 11.8 mm by default) and centred in the middle of the opening (`head_paint/src/eyeball.rs`). So the iris measures above match the sheets the judges see, not viewer 2. KOTOR's iris in the game comes from its texture, which I did not read.
- Round 7's Outer corner reach with a heavy Upper lid cover or a thick lid piece leaves a skin tab at Bastila's outer corner, open and closed. With Outer corner depth at -4 the tab goes away.
- In my zsh shell, the head list must be an array (`H=(a b c)`, then `$H`).

## For a follow-up

- Add `lid_top` and `lid_bot` (where each lid crosses the iris, in iris radii) to the eye proportions, and re-solve with the opening's height held. Most of what the judges liked on pmhc04, pmha01 and Kreia came from these lid values. But first decide which iris these measures read: the sheets' iris or viewer 2's painted one.
- Bastila needs two builds. The first is a lid piece that is thicker and juts forward like KOTOR's, without the tab. The second is an outer corner that can reach the equator, with the closed lid still covering it.
- The black dash under Bastila's and Carth's closed lower lids is in live too. It is worth a fix of its own.

## For David

- **Viewer 2 did not change.** It is still `?b=48`.
- **The work stopped after the test pictures.** The test eye was better on pmhc04 for both judges, but not on Bastila. On Bastila one judge preferred it and the other preferred the current eye, because the test eye has a small flap of skin at the outer corner.
- **The brief's two ideas are only partly true in KOTOR.** KOTOR's outer eye corner does sit level with the middle of the eyeball, but ours already does on most heads, pmhc04 included. Bastila, Kreia and Carth are the exceptions. KOTOR's inner corner does not sit there.
- **KOTOR's upper lid does not cover more of the iris than ours on most heads.** The difference is the lower lid. In KOTOR the lower lid rides up over the bottom of the iris. Ours shows white under the iris on 46 of 57 heads, which makes the eyes look round and staring.
- **Bastila's heavy lid comes from KOTOR's lid piece.** It is a thick ridge that juts about 9 mm in front of her eyeball. Ours juts about 2 mm. A thicker piece made her lid heavier in a test, but it also made the flap of skin at the corner.
- **Pictures:**
  - `/private/tmp/claude-501/-Users-david-perso-kjk/ea0d1303-868f-43c4-b8fe-71c29bfc78e9/scratchpad/q/pics/pb_P.png` shows pmhc04 and Bastila: KOTOR, the current eye and the test eye, open and then closed.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/ea0d1303-868f-43c4-b8fe-71c29bfc78e9/scratchpad/q/pics/P_open_lit.png` and `/private/tmp/claude-501/-Users-david-perso-kjk/ea0d1303-868f-43c4-b8fe-71c29bfc78e9/scratchpad/q/pics/P_closed_lit.png` show all six test heads, open and closed.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/ea0d1303-868f-43c4-b8fe-71c29bfc78e9/scratchpad/q/pics/bast_var.png` shows Bastila with each change on its own, including the flap of skin.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/ea0d1303-868f-43c4-b8fe-71c29bfc78e9/scratchpad/q/pics/extremes.png` shows KOTOR's heads at the ends of each measure.
