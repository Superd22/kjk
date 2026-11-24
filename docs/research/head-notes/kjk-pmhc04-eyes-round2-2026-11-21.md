# pmhc04's eyes, round 2 (2026-11-21, night)

Written by the conversation kjk-d2 for the lead kjk-48. Read it with round 1's note `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-pmhc04-eyes-round1-2026-11-21.md`. The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/ad70993a-91df-408e-93d2-fcd64cb31370/scratchpad/r2/`, called `$W` below.

## Verdict

Round 2 is built but not published. Viewer 2 still runs round 1 (?b=46). The guarded result is ready in `$W/w3` and can be published in a few minutes (see "How to publish").

pmhc04 improved on two of the plan's targets and slipped on a third:
- The skin round the outer corner bulges less. outer_bulge went from 7.3 to 4.7 mm. KOTOR's is 3.2 mm, and the target was under 4.5.
- More eyeball shows in profile. profile_ball went from 19.6 to 25.6 mm². KOTOR shows 53, and the target was over 30. Outer corner depth cannot get pmhc04 past about 26 mm². At -5 and -6 mm the area stays at 26 and the folds grow.
- The skin beside the nose stands further in front of the eyeball again. nose_skin_z went from 3.1 to 5.7 mm. It is still under the target of 6, but round 1 had brought it to 3.1. The solve moved Inner corner position from -2.63 to -5.64, and the pink caruncle wedge grew back.
- outer_back moved away from KOTOR, from -11.0 to -9.7 (KOTOR -12.8). The reason is in "What changed", point 2.

The two blind judges cancel out. Both picked the second row on every head, in both orders, so they show only the known bias toward the second row. I looked at the enlarged pmhc04 sheet myself. The new eye shows a clearer wedge of eyeball in profile, and the outer corner is flatter. But the opening looks rounder and more bulging than KOTOR's long, narrow eye, and the caruncle sits in a small hollow again. I call that mixed. The brief allows a publish only if pmhc04 improves, so I did not publish.

Bastila keeps her live values under the guard. Her new preset has 14 faulty triangles against 2 live. In pictures, her new preset shows more eyeball in profile (profile_ball 21 to 38, KOTOR 48).

## What changed

All code is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint/crates/character_customisation/head/head_sliders/src/`. Nothing is committed. The crate's tests pass.

1. **The press frees the outer corner** (`eye_curves.rs`). Two changes:
   - Edge points within 8% of the edge from the outer tip are only pushed out of the eyeball, never pulled in onto it (`PressEye.outer`, `outer_free`, `EYES_OUTER_FREE=0` gives round 1's press). So the lid's end may lie behind the eyeball's silhouette.
   - The skin within 4 mm of the outer tip follows the tip's own correction (`outer_reach`, `EYES_OUTER_REACH=0` gives round 1's press).
   - These change every head's build near the outer corner, also at slider value 0. On the live presets the fold check moved from 121 to 82 faulty triangles after the guard, and no head got worse.
2. **Two new measures** (`eyes.rs`, PROPORTIONS now has 34 rows).
   - `outer_bulge`: the highest skin 1 to 5 mm beyond the outer corner, within 3 mm above or below it, against the corner's column. It is the diagnosis's measure, now a proportion the solver matches. KOTOR's spread is 3.58 mm and its mark 0.90.
   - `outer_tip`: the skin's own depth over the first millimetre beyond the outer corner, against the eyeball's front. KOTOR's spread is 2.67 mm and its mark 0.67.
   - Why outer_tip is needed: outer_back reads the eyeball's depth in the opening's last column seen from the front. A corner set straight back along the head's forward axis changes nothing in the front view, so outer_back cannot see Outer corner depth. outer_back mostly reads how far round the eyeball the opening reaches, and our eyeball is smaller than KOTOR's (13.1 against 14.7 mm on pmhc04). That is why it pulls Eye width and Outer corner position up.
3. **New slider "Outer corner depth"** (`Curve::CornerDepth { inner: false }`, measure outer_tip, -4 to 2 mm every 0.5). It moves the outer corner's tip, the lids within about 9 mm of it and the skin within about 7 mm beyond it back along the head's forward axis. The lining that wraps the eyeball stays (`EYES_OUTER_LINING`). Made by the new command `sliders eyes make-pmhc-r2 <data>`, which also extends `eyes/marks.json`.
   - Moving the lining with the corner folded the socket wall beside the eyeball (2320 faulty triangles with every head at -6). Sliding the lining along the eyeball made it worse (3365). Leaving it in place gave 392 at -3, 504 at -4, 780 at -5 and 1356 at -6, against 362 at 0. Longer fades made it worse. So the range stops at -4.
   - At -6, comm_b_f shows a visible notch at the outer corner from the temple side (`$W/pics/tr_cbf.png`). At -3 the corner is clean.
4. `bin/eye_audit.rs`: `faultsat` prints each fold's place with `EA_WHERE=1`. It is audit only.
5. Shared log updated at 02:40 in `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-slider-kit-shared-changes.md`.

pmhc04's new values (unguarded and guarded are the same, because pmhc04 got no new fold): Eye depth -5.34 (its low end), Eye prominence 5.81, Eye width 4.39, Upper lid fullness -1.97, Inner corner position -5.64, Inner corner depth -2.16, Outer corner position -0.09, Outer corner depth -3.98 (its low end), Eye tilt -3.71, Eyeball size 1.02, Eye opening height -0.19.

## Why Upper lid fullness and Eye width sat at their ends (task 3)

- **Upper lid fullness** reads the lid 3 mm above the opening against the skin 7 mm above it. Eye prominence brings the lid forward and fades out before the skin under the brow. So the lid stands out more, and the solve answers with fullness at its low end. On pmhc04, the lid stands 1.0 mm in front of that skin even at -1.97, and KOTOR's stands 2.8 mm behind it. The new measures did cause this, through Eye prominence. I did not fix it. Fullness is a blend of about 90 learned fields with keys from -2 to 4.5, so its range cannot be stretched by hand. A fix is either a lower key for fullness, made the way it was made first, or an Eye prominence that carries the skin under the brow a little.
- **Eye width** is no longer at its end in round 2 (5.99 before, 4.39 now). It was pulled there by outer_back, as explained in "What changed", point 2. The new measures outer_bulge and outer_tip pull the other way.

## Viewer 2

Nothing changed. It runs ?b=46 (round 1). The head_sliders wasm was not rebuilt.

Warning: the crate in the worktree now holds round 2's press. If anyone rebuilds the head_sliders wasm from it, every head's outer corner is built with the new press, while the live data has no Outer corner depth. The fold check says that is safe on the live presets (82 against 121 faulty triangles), but it was not looked at in pictures with live data. Undo the code, or publish round 2, before the next wasm rebuild.

## How to publish, if the lead or David decides to

1. Copy `$W/w3/web/eyes/{sliders.json,page.json,made.bin,made.json,starts.bin}` to `~/.claude/tools/likeness-fit-data/sliders/web/eyes/`, and `$W/w3/eyes/{sliders_solve,report,audit,marks}.json` to `.../sliders/eyes/`.
2. Rebuild the head_sliders wasm into `~/.claude/tools/likeness-fit/hq/wasm/head_sliders/` the way round 1 did.
3. Raise viewer 2's `sliders.js` VER and SLIDERS_MODULE_VER to 47, `nose.js` VER to ?b=47, and `nose/index.html` to nose.js?v=41. Log it.
4. The files from before round 2 are in `$W/live_before/` (web_eyes, eyes, wasm, viewer).

## Measures before and after

The tool is round 1's diag, rebuilt on round 2's crate (`$W/diag`). "Before" is round 1's `m_f6.json` (live), "after" is `$W/m_w3.json` (guarded) and `$W/m_w2.json` (unguarded). `python3 $W/compare.py <before> <after> [heads]` prints the table.

All 57 heads, after the guard (only 18 heads take a new preset):

| Measure | Wobble | KOTOR median | Before | After | Median gap before | After | Heads worse by more than the wobble |
|---|---|---|---|---|---|---|---|
| nose_skin_z | 1.08 | 5.01 | 8.59 | 8.44 | 3.53 | 3.43 | 4 |
| profile_ball | 3.92 | 43.3 | 18.9 | 18.9 | 23.0 | 22.8 | 3 |
| outer_back | 0.65 | -12.98 | -9.75 | -10.15 | 2.80 | 2.42 | 3 |
| ball_proud | 0.07 | 4.52 | 3.47 | 3.56 | 1.07 | 0.99 | 4 |
| outer_bulge | 0.67 | 5.43 | 5.80 | 5.96 | 1.67 | 1.73 | 6 |
| inner_tip_z | 0.51 | -6.28 | -5.76 | -5.97 | 1.51 | 1.43 | 6 |

Without the guard (all 57 heads re-solved), the median gap to KOTOR shrinks on profile_ball (23.0 to 19.7), outer_back (2.80 to 1.78), nose_skin_z (3.53 to 2.90) and inner_tip_z (1.51 to 1.08). But up to 25 heads get worse by more than the wobble on one measure or another, and 39 heads get more folds.

No head shows any eyeball outside the opening in the front view after the guard.

The heads of the sheets, as KOTOR / before → after (guarded):

| Head | nose_skin_z | profile_ball | outer_back | outer_bulge | inner_tip_z |
|---|---|---|---|---|---|
| pmhc04 | 1.8 / 3.1 → 5.7 | 53.4 / 19.6 → 25.6 | -12.8 / -11.0 → -9.7 | 3.2 / 7.3 → 4.7 | -5.6 / -8.9 → -7.0 |
| n_zherronh | 0.5 / 4.6 → 3.7 | 28.6 / 12.2 → 13.7 | -7.8 / -7.3 → -7.2 | 3.7 / 4.1 → 4.5 | -4.7 / -5.7 → -4.8 |
| p_bastilah (kept; unguarded in brackets) | 1.9 / 5.8 (4.6) | 47.7 / 21.0 (38.1) | -15.9 / -8.2 (-11.2) | 6.6 / 4.1 (5.2) | -5.0 / -7.8 (-6.7) |
| p_kreiah (kept; unguarded in brackets) | 1.7 / 7.1 (6.0) | 60.1 / 12.2 (29.8) | -9.9 / -6.6 (-10.1) | 1.4 / 4.0 (4.7) | -3.9 / -5.7 (-7.5) |
| pmha01 (kept; unguarded in brackets) | 4.1 / 5.1 (6.5) | 48.9 / 23.0 (22.6) | -11.6 / -11.0 (-9.7) | 3.4 / 6.6 (4.2) | -9.5 / -8.4 (-7.3) |

## The fold check

`eye_audit zones` on all 57 heads, kit count:
- Live (round 1, old binary): 121 faulty triangles on 28 heads (`$W/z_live.log`).
- Round 2 unguarded (`$W/w2/zones.log`): 660 faulty triangles on 44 heads. Without Outer corner depth in the solve it was 387 (`$W/zB.log`), close to round 1's unguarded 326. So most new folds come from the re-solve and a part from the new slider.
- Round 2 guarded (`$W/w3/zones.log`): 82 faulty triangles on 25 heads. No head has more than live. Six heads have fewer: pmha02 (28 to 0), mod_tcs_bastila (15 to 10), mod_hp_pfhb02 (3 to 1), n_zherronh (2 to 0), old_w_m (1 to 0) and pfhc02 (2 to 1).
- The 39 kept heads are listed in `$W/keep2.txt`. Their made.bin shapes were copied from live (`$W/copymade.py`).

## The bench

`sliders eyes bench` on the unguarded set (`$W/wb_bench.log`, 612 s): 44 of 57 solved KOTOR heads have a fault, 79 of 210 pairs have a faulty corner (150 corners of 840), 34 of 64 KOTOR-like samples and 25 of 64 box samples have a fault. Round 1 ran no bench, so there is nothing to compare it with.

## Pictures

All are under `$W/pics/` and `$W/judge/`.
- `before_after.png`: for pmhc04, Bastila, Zherron, Kreia, pmha01 and Carth, four rows each: KOTOR, before (live), after (guarded) and after unguarded. The columns are the front, three-quarter from the temple side, three-quarter from the nose side and profile, in clay and then in viewer 2's light. `ba_pmhc_bast.png` is the pmhc04 and Bastila part.
- `tr_ocd.png`: Outer corner depth by hand on pmhc04 at 0, -3 and -6. `tr_cbf.png`: the same on comm_b_f, the head with the most folds.
- `slab.png`: the slices for pmhc04, Bastila, pmha01, comm_a_m, Zherron and Carth, KOTOR against round 2.
- The judges' blind sheets are `$W/judge/head{1,2,3}.png` and `$W/judge2/` (rows swapped). The key is `$W/judge/key.txt`.

I found no gap between the lid's end and the eyeball from the temple side at the values the solve uses (down to -4). At -6 there is a notch.

## The judges

Two fresh blind Sonnet judges at medium effort, with enlarged sheets.
- Judge 1 (head1 X = after, Y = before) picked Y on all three heads: the old pmhc04 (medium), Bastila's new unguarded preset (medium), and Zherron's new preset (low to medium). It said the new pmhc04 has "a heavier, bulging cushion under and beside the eye" and a bigger pink wedge.
- Judge 2 (rows swapped) also picked Y on all three: the new pmhc04 (medium, "more eyeball past the lids, similar to the original"), the old Bastila (medium) and the old Zherron (low to medium).
- Both judges therefore only show a bias to the second row. Each head has one vote each way. I did not run a third judge.

## Heads that got worse

After the guard, by more than the wobble on three or more measures: n_hussef, old_w_m, pfhc02, pmhc04 (nose_skin_z, outer_back, ball_proud, nose_skin_abs), n_jedimalek, old_b_m, mod_improved_bastila and p_baodurh. No judge looked at the heads other than pmhc04, Bastila and Zherron.

## Choices made without David

- Outer corner depth's measure is a new one, outer_tip, not outer_back. outer_back cannot see a corner set straight back (see "What changed").
- outer_bulge is a proportion of the solve, not a separate guard.
- The lining stays when the outer corner goes back, and the range stops at -4 mm, because of the folds.
- The guard is round 1's rule: a head with more faulty triangles than live keeps its live values. 39 heads keep them, Bastila and Kreia among them.
- Round 2 is not published, because pmhc04 did not clearly improve.

## How to undo the round

Nothing live changed. To undo the code, copy `$W/orig/head_sliders_src/` over `head_sliders/src/`, if no other session has edited those files since 02:18 (check the shared log). Otherwise remove by hand the `outer`, `outer_free`, `outer_reach`, `outer_depth_fades` and `outer_lining_keep` parts of `eye_curves.rs`, the two proportions, the Outer corner depth spec and `make_pmhc_r2` in `eyes.rs`, and the `EA_WHERE` block in `bin/eye_audit.rs`.

## Traps I hit

- A copy of a data folder must not link the source's log files. `$W/mkdata.sh` now skips `*.log`. My first copies wrote their fold logs into another folder's log through the link.
- In the faultsat command, stdout and stderr must be split: the fold places go to stdout and the totals to stderr.
- `sed -i` with `$( )` in a heredoc script failed silently. Rewrite small scripts whole.
- The solve takes about 75 s, the fold check about 30 s, and the bench about 10 minutes.

## Is round 3 needed?

Yes, if David still sees fault 4. The slices in `$W/pics/slab.png` show the same thing as in the diagnosis. KOTOR's lid piece juts forward like a visor in front of the top of the eyeball on pmhc04, pmha01, comm_a_m, Carth and Bastila. Ours are thin shells that lie on the eyeball, concentric with the skin, on every head. Rounds 1 and 2 did not touch that.

Two things from this round also matter for round 3 or for a round 2b:
- profile_ball is stuck near 26 mm² on pmhc04 because the lids still wrap the eyeball above and below. Freeing the outer end was not enough. A visor-like lid piece, with the skin's lid edge tucked behind it, would also let more eyeball show in profile.
- The rounder, bulging opening on pmhc04 and the regrown caruncle come from the solve (Eye prominence 5.81, Inner corner position -5.64, Upper lid fullness at its end). A lower key for Upper lid fullness and a weight on nose_skin would likely help more than a new slider.
