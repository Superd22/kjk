# pmhc04's eyes, round 1 (2026-11-21, night)

Written by the conversation kjk-c1 for the lead kjk-48. Read it together with the diagnosis note `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-pmhc04-eyes-diagnosis-2026-11-21.md`. The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/92cb9880-d2cc-470c-ba08-1791155b76ba/scratchpad/r1/`, called `$R` below.

## Verdict

Round 1 is live in viewer 2 as version ?b=46.

pmhc04's eyes improved. Its inner corner is no longer buried. The skin on the nose side of the inner corner now stands 3.1 mm in front of the eyeball's front. It stood 13.4 mm in front before, and KOTOR's stands 1.8 mm in front. The outer corner now sits 11.0 mm behind the eyeball's front. It sat 7.5 mm behind before, and KOTOR's sits 12.8 mm behind. These two targets of the plan are met.

The third target is missed. In profile, 19.6 mm² of pmhc04's eyeball now shows. That is up from 11.9 mm², but the target was over 30 mm², and KOTOR shows 53 mm². The lids still wrap the eyeball in profile, and the skin round the outer corner now bulges more (7.3 against 4.5, with KOTOR at 3.2). Round 2 has to fix both.

Three blind Sonnet judges looked at pmhc04. Two preferred the new eyes. The third looked at enlarged close-ups and preferred the new eyes with high confidence on all three of David's points: the inner corner, the outer corner's cushion and the profile. One preferred the old eyes at low confidence. Bastila and Kreia were judged worse by two judges, so they keep their old presets. Bastila is therefore not improved by this round.

## What changed

All code is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint/crates/character_customisation/head/`. Nothing is committed.

1. **Five new eye measures** (`head_sliders/src/eyes.rs`, `PROPORTIONS` now has 32 rows, computed in `measure_eye` and `profile_ball`). The solver reads them on the shown head, with our lid pieces and caruncles, as it reads the others. They are measured in the eyes' frame. The diagnosis's `probe` uses the head frame, so the numbers differ from the diagnosis's table. They agree in sign and in which heads are outliers.
   - `nose_skin`: the highest skin 2 to 6 mm on the nose side of the skin's own inner corner, within 2 mm above or below it, against the eyeball's front. The corner is that of the skin's opening, with our caruncle and KOTOR's lid piece left out. Without that, our caruncle moved the window toward the temple and our heads read 4 mm too low.
   - `inner_tip`: the inner corner's front (the eyeball, or our caruncle where it lies in front) against the eyeball's front. The plan suggested `inner_sink` (corner against the nose-side skin). I tried it first, but Inner corner depth barely moves it, because the skin beside the corner moves with the corner. The solve then set the slider the wrong way.
   - `outer_back`: the outer corner's column against the eyeball's front.
   - `ball_proud`: the eyeball's front against its mean depth along the opening's edge.
   - `profile_ball`: the eyeball's area that shows in profile from its own temple's side, in mm², at 0.2 mm pixels, counting only the eyeball's front half. The measure's window cuts the head's sides off, so the back half showed through on some KOTOR heads (spread 80 mm² instead of 16).
   - `Measured.marks.stray` gives, per eye, the eyeball area that shows in the front view but is not joined to the opening. It is not a proportion.
2. **Marks**: `eyes/marks.json` got five rows. Each spread is KOTOR's spread over the scored heads, and each mark is a quarter of it. I did not run check 1, so the marks ignore the measures' own wobble. Spreads: nose_skin 2.66, inner_tip 2.23, outer_back 2.49, ball_proud 0.90, profile_ball 16.2.
3. **New slider "Eye prominence"** (`head_sliders/src/eye_curves.rs`, `Curve::Prominence`, `prominence_fades`, `prominence_outer`, `prominence_shell_keep`). It is a field before the press, like Inner corner depth. It moves along `Frame::head`'s forward axis:
   - the eyeball's centre ring, by the whole amount;
   - the socket and the lining within 1 mm of the eyeball, wherever they are round it, wholly, fading to nothing 2.5 mm out;
   - the lids and the skin within 1 mm outside the opening, wholly, fading out over the next 12 mm. This weight is multiplied by a second one that is 1 up to 2 mm in front of the eyeball and fades out over the next 10 mm. The skin beside the nose, under the brow and on the cheek therefore stays.
   - Its range is 0 to 6 mm, every 0.5 mm, and its measure is profile_ball. Its spec and hint are in `SPECS` in `eyes.rs`. It was added to the data by the new command `sliders eyes make-pmhc-r1 <data>`.
4. **Inner corner depth** now has the measure inner_tip, so the solver uses it.
5. **The eyeball's size weighs less in the solve.** `feature.rs` gained `Feature::gap_weight(measure)`, default 1.0, and `solve.rs` `gaps()` multiplies each proportion's gap by it. The eyes give "ball" 0.3 (`EYES_BALL_WEIGHT` overrides it). pmhc04's Eyeball size went from +2.70 to -0.85.
6. **The guard** (`sliders eyes guard-pmhc-r1 <data> <old page.json> <heads...>`). The heads named keep their values from the old page, matched by slider name, with Eye prominence at 0. Their measure in the page is read again. Their full-mode shape in `made.bin` was copied from the old file with a small Python step. Their verdicts in `page.json` are still those of the new solve, and `eyes/audit.json` is the unguarded solve's (`$R/f4`).
7. Both shared-file changes are logged in `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-slider-kit-shared-changes.md`. The crate's tests pass.

pmhc04's values changed as follows: Eye prominence 0 to 4.36, Eye depth -4.83 to -5.34, Eyeball size 2.70 to -0.85, Eye width -2.48 to 5.99 (its top end), Inner corner position -4.98 to -2.63, Inner corner depth 0 to -2.32, Outer corner position 3.00 to 0.02, Upper lid fullness 0.72 to -2.00 (its bottom end), Eye tilt 0.23 to 1.51, and a few lid points by under 1.3.

## Viewer 2

- The URL is http://127.0.0.1:8765/viewer2/. Its data version is ?b=46 (`sliders.js` VER and SLIDERS_MODULE_VER 46). The slider page uses `nose.js` VER ?b=46 and `nose/index.html` loads nose.js?v=40.
- Live files: `web/eyes/{sliders.json, page.json, made.bin, made.json, starts.bin}` and `eyes/{sliders_solve.json, report.json, audit.json, marks.json}` under `~/.claude/tools/likeness-fit-data/sliders/`. The head_sliders browser module was rebuilt.
- A Sonnet agent checked viewer 2 in the browser after the publish. It saw no console errors after load, but its console watch started only after the page had loaded. pmhc04 ("Male C4") shows Eye prominence at +4.36 mm and Inner corner depth at -2.32 mm, on ranges of 0 to 6 mm and -3 to 3 mm. In the "Eyes" close-up, the eyes rendered with smooth lids and no holes or spikes. The agent also saw dark spiky shards on the scalp above the forehead and at the ears. This looks like the hair or cap pieces, which this round did not touch, and I did not look into it.
- 30 heads take the new presets. 27 heads keep their old ones, with Eye prominence at 0. Those 27 are the 25 heads that got more fold-check faults (listed in `$R/keep1.txt`), plus p_bastilah and p_kreiah.

## Measures before and after

The table tool is the diagnosis's diag (`$R/diag`, built on today's crate, with a new `stray` column). It measures viewer 2's shown head. "Before" is `$R/m_live.json` (live ?b=45), and "after" is `$R/m_f6.json` (what is live now). `python3 $R/compare.py <before> <after> [heads]` prints the table. The wobble is the median difference between KOTOR's left and right eye.

All 57 heads:

| Measure | Wobble | KOTOR median | Ours before | Ours after | Median gap to KOTOR before | After | Heads worse by more than the wobble |
|---|---|---|---|---|---|---|---|
| nose_skin_z | 1.08 | 5.01 | 10.07 | 8.59 | 4.23 | 3.53 | 0 |
| profile_ball | 3.92 | 43.3 | 15.3 | 18.9 | 26.7 | 23.0 | 1 |
| outer_back | 0.65 | -12.98 | -8.57 | -9.75 | 3.88 | 2.80 | 5 |
| ball_proud | 0.07 | 4.52 | 3.17 | 3.47 | 1.41 | 1.07 | 5 |
| outer_bulge | 0.67 | 5.43 | 5.33 | 5.80 | 1.73 | 1.67 | 9 |
| inner_tip_z | 0.51 | -6.28 | -4.95 | -5.76 | 1.81 | 1.51 | 4 |
| piece_reach | 0.10 | 2.93 | 1.82 | 1.82 | 1.64 | 1.82 | 8 |
| ball_front_abs | 1.24 | 14.74 | 12.77 | 13.89 | 1.56 | 1.56 | 6 |
| nose_skin_abs | 0.31 | 20.50 | 23.26 | 22.65 | 3.26 | 2.72 | 3 |

No head shows any eyeball outside the opening in the front view, before or after. The check is the `stray` column, and it is 0 on all 57 heads.

The eight heads of the sheets, given as KOTOR / before → after (the 27 kept heads show no change):

| Head | nose_skin_z | profile_ball | outer_back | ball_proud | outer_bulge | inner_tip_z | nose_skin_abs |
|---|---|---|---|---|---|---|---|
| pmhc04 | 1.8 / 13.4 → 3.1 | 53.4 / 11.9 → 19.6 | -12.8 / -7.5 → -11.0 | 4.1 / 2.5 → 3.7 | 3.2 / 4.5 → 7.3 | -5.6 / -4.1 → -8.9 | 16.6 / 25.0 → 15.0 |
| p_bastilah (kept) | 1.9 / 5.8 | 47.7 / 21.0 | -15.9 / -8.2 | 3.9 / 3.8 | 6.6 / 4.1 | -5.0 / -7.8 | 13.3 / 18.0 |
| p_carthh | 5.1 / 13.2 → 11.4 | 33.1 / 9.7 → 10.5 | -11.3 / -7.7 → -8.2 | 3.5 / 2.6 → 2.9 | 5.9 / 5.1 → 5.6 | -6.3 / -3.5 → -4.7 | 17.1 / 21.9 → 22.6 |
| p_kreiah (kept) | 1.7 / 7.1 | 60.1 / 12.2 | -9.9 / -6.6 | 3.1 / 3.2 | 1.4 / 4.0 | -3.9 / -5.7 | 13.0 / 18.8 |
| pfha05 (kept) | 3.2 / 4.6 | 39.2 / 19.7 | -14.6 / -10.2 | 5.8 / 3.9 | 5.1 / 6.0 | -10.2 / -8.0 | 22.0 / 22.0 |
| pmha01 | 4.1 / 7.2 → 5.1 | 48.9 / 15.8 → 23.0 | -11.6 / -8.4 → -11.0 | 4.5 / 3.1 → 3.6 | 3.4 / 4.5 → 6.6 | -9.5 / -7.0 → -8.4 | 22.6 / 24.7 → 22.5 |
| comm_a_m (kept) | 3.9 / 8.4 | 49.8 / 15.2 | -11.7 / -7.9 | 4.5 / 2.9 | 3.0 / 4.4 | -9.7 / -4.8 | 21.0 / 23.9 |
| n_zherronh | 0.5 / 10.1 → 4.6 | 28.6 / 8.0 → 12.2 | -7.8 / -6.0 → -7.3 | 2.8 / 2.5 → 2.4 | 3.7 / 4.4 → 4.1 | -4.7 / -3.9 → -5.7 | 16.1 / 24.2 → 21.3 |

Before the guard, Bastila's new preset read nose_skin_z -1.4, which overshoots KOTOR's 1.9, and profile_ball 22.9. Kreia's new preset read profile_ball 35.3 (KOTOR 60.1), but outer_bulge 7.0 (KOTOR 1.4).

## The fold check

The fold check is `eye_audit zones` on all 57 heads, using the kit count.
- Live before the round (`$R/d2_zones.log`): 260 faulty triangles. The inner corner has 19 on 7 heads, the lids 172 on 12 heads, the nose side 52 and other places 41.
- After the round, live now (`$R/f6/zones.log`): 112 faulty triangles. The inner corner has 13 on 6 heads, the lids 58 on 5 heads, the nose side 18 and other places 32.
- Under the guard, no head has more faulty triangles than before, and 23 heads have fewer, pmhc04 among them. In viewer 2's merged head (`eye_audit shown`) the count is 157 on f5, the round's result before Bastila and Kreia were also kept. I did not run it on f6.
- I did not run the full bench (held out, pairs and samples).

## Pictures

All are under `$R/pics/`.
- `before_after_8.png` shows, for each of the eight heads, three rows: KOTOR, before and after. The columns are the front, three-quarter from the temple side, three-quarter from the nose side and profile. The first four are in clay and the last four in viewer 2's "In game" light. `ba_pmhc_bast.png` is the pmhc04 and Bastila part.
- `before_8.png` (KOTOR and live) and `after_8.png` (after) are the raw sheets.
- `trial_prom.png`, `cmp_k.png` and `tr_k*.png` are the hand trials of Eye prominence on pmhc04.
- The judges' blind sheets are in `$R/judge/`, `$R/judge2/` (X and Y swapped) and `$R/judge3/` (pmhc04 enlarged). The key is `$R/judge_key.json`. The after sheet was rendered before Bastila and Kreia were kept, so their "after" rows show the presets that were rejected.

## The judges

All three judges were fresh, blind Sonnet agents at medium effort.
- **Judge 1** preferred the new eyes on pmhc04 (medium) and Zherron (low to medium). It preferred the old eyes on Bastila (medium: a cushion and creases round the outer corner, and a dent), Kreia (medium: a swollen brow roll and a hollow inner corner) and pmha01 (low). It called Carth and pfha05 the same. It picked the second row on 6 of 8 heads, including comm_a_m, whose two rows are identical, so it leans to the second position.
- **Judge 2** saw the rows swapped. It preferred the old eyes on Bastila (medium: a crease under the eye and a sharp wedge in profile) and Kreia (medium: a crease under the eye and a pinched inner corner). It preferred the new eyes on Zherron (medium) and pmha01 (low), and the old eyes on pmhc04 (low). It called Carth and pfha05 the same.
- **Judge 3** saw only pmhc04, enlarged. It preferred the new eyes on all three points, with high confidence. Its words: the old eye "has a deep dark hollow beside the nose", "a puffy bulge above and beside the outer corner", and in profile "the eyeball is tucked into a narrow slit". In the new eye, "a visible wedge of white eyeball protrudes with a clean lower-lid edge. It is still less exposed than the original." It saw two minor faults in the new eye: a small dark notch at the inner corner in the temple view, and a slight sag line under the lower lid.
- I looked at the sheets myself. pmhc04's inner corner is clearly more open and its caruncle smaller. Its profile shows a small wedge of eyeball. Bastila's new preset had a large pink caruncle.

## Heads that got worse

- On the judges: Bastila and Kreia were worse on both judges, so they keep their old presets. pmha01 was split between the judges, so it takes the new preset.
- On the measures, beyond their wobble on three or more measures: pfhb04 (profile_ball, ball_proud, outer_bulge), mod_improved_bastila (outer_back, outer_bulge, ball_front_abs), mod_meetra (outer_back, ball_proud, ball_front_abs, nose_skin_abs), n_admoffh (outer_bulge, inner_tip_z, piece_reach, inner_sink), p_handmaidenbb (inner_tip_z, piece_reach, ball_front_abs), and pmhc04 itself (outer_bulge, inner_tip_z, piece_reach). No judge looked at the first five.

## Choices made without David

- The guard. A head whose new preset has more fold-check faults than before keeps its old values (25 heads), and so do Bastila and Kreia, whom two judges found worse. This follows the rule "do not destroy other heads" rather than making a slider value for each case.
- Eye prominence runs from 0 to 6 mm, forward only. With -2 allowed, 26 heads were solved at -2. Setting the eyeball back folded the skin outside the eye, with 489 faulty triangles.
- There is no outer-corner hold-back in Eye prominence. Holding the lids and the skin near the outer corner back to 40% gave pmhc04 33 mm² in profile and all three targets. But it sheared the lids. At +3 mm the fold check found lid folds on 56 heads. The code keeps it behind `EYES_PROM_OUTER` and `EYES_PROM_SHELL`.
- The soft fades (1, 12, 2, 10 mm) replaced the plan's 4 mm and 8 mm. The hard fade folded the inner corner above +3 mm on 54 heads.
- The eyeball size's gap weight is 0.3.
- Inner corner depth's measure is inner_tip, not nose_skin_z or inner_sink.
- The brow and cheek presets were not re-solved. Nothing they depend on changed.

## How to undo the round

1. Copy back the live files from `$R/live_before/`. That means `web_eyes/*` to `~/.claude/tools/likeness-fit-data/sliders/web/eyes/` and `eyes/{sliders_solve,report,audit,marks}.json` to `.../sliders/eyes/`.
2. Copy back `$R/live_before/wasm/head_sliders/*` to `~/.claude/tools/likeness-fit/hq/wasm/head_sliders/`.
3. Raise the versions again rather than lowering them, so browsers reload: VER, SLIDERS_MODULE_VER, nose.js VER and nose.js?v. `$R/live_before/viewer/` holds the three files from before.
4. For the code, copy back `$R/orig/head_sliders_src/` over `head_sliders/src/`. Do this only if no other session has edited `feature.rs` or `solve.rs` since 01:42 (check the shared log). Otherwise remove only `gap_weight` by hand.

## Traps I hit

- In zsh, `$KEEP` does not split into words. Use `${=KEEP}`.
- `rm -rf` on a path built from variables is blocked. Make each data copy in a new folder instead.
- `cp -c` fails because GNU cp comes first on the PATH. `sed -i ''` needs `/usr/bin/sed`.
- `eye_audit zones` with a list of heads panicked ("head") on a name it could not find. Use `all`.
- A data folder whose page has no Eye prominence makes `eye_audit zones` panic when the set has it. Make the experiment folders from a solved folder.
- Our measures read the eyes' frame, and the diag reads the head frame. Targets must come from the diag (`pmhc_diag measure`). The solver works with its own numbers.
- A judge that sees two identical rows can still pick one. Swap the rows and ask a second judge before trusting a low-confidence pick.

## What round 2 should change

1. **The outer corner must sit back from the eyeball, the plan's round 2.** The one change that gave pmhc04 KOTOR's wedge of eyeball in profile was holding the outer lid back while the eyeball came forward (33 mm², with outer_bulge 1.8). It sheared the lids because the press still seats the lid edges on the eyeball and the lining moves with the eyeball. Round 2's "Outer corner depth", with the press letting the outer lid's end sit behind the eyeball's silhouette, is the way to get that profile without folds. Its measure is outer_back. Use profile_ball and outer_bulge as guards.
2. **The outer cushion got worse on pmhc04** (outer_bulge 4.5 to 7.3, KOTOR 3.2), and on 9 heads in all. Eye prominence brings the skin round the opening forward, and the outer skin follows. Add outer_bulge as a proportion, or as a guard in the solve.
3. **Bastila and Kreia.** Their new presets had a cushion or crease under the eye and round the outer corner. Bastila's nose_skin overshot (-1.4 against KOTOR's 1.9). Re-solve them after round 2's slider and judge again.
4. **Upper lid fullness and Eye width hit their range ends on pmhc04** (-2.00 and 5.99). Check whether the new measures pull them there.
5. **The 25 guarded heads** keep their old presets. After round 2, re-solve all heads and guard again with the fold check.
6. Run the full bench once, which I did not do.
