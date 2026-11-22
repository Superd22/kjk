# pmhc04's eyes, round 4 (2026-11-21, morning)

Written by the conversation kjk-b5 for the lead kjk-48. Read it with round 3's note `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-pmhc04-eyes-round3-2026-11-21.md`. The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/123bc42e-dd62-40a5-afb2-78e3aa2d0b67/scratchpad/r4/`, called `$R` below.

## Verdict

Part of round 4 is live in viewer 2 as version ?b=48. The slider the brief asked for, Outer corner opening, was tried and is not built.

What is live:
- A new measure of the eyeball's outline seen in profile. It follows kjk-48's correction: the target is the outline's shape, not its area. It passed the four checks.
- Eye presets solved again with that measure. Five heads take new presets: pmhc04, Zherron, Jolee, pmha04 and n_admoffh. The other 52 heads keep their live values.

On pmhc04, against ?b=47:
- The eyeball's outline in profile overlaps KOTOR's more. Overlap divided by union went from 0.46 to 0.60.
- The eyeball shows further back in profile, 8.3 mm behind its front instead of 7.3 mm. KOTOR's shows 13.4 mm.
- profile_ball went from 34.9 to 36.1 mm². KOTOR's is 53.4.
- outer_bulge went from 6.2 to 6.0 mm. KOTOR's is 3.2.
- nose_skin_z went from 4.2 to 5.2 mm. KOTOR's is 1.8. The rise is just under the measure's wobble of 1.1 mm, but it is a step the wrong way.
- With the lids closed, 0.67 mm² of eyeball shows from the front, against 1.27 mm² before.

Both Opus judges agree that pmhc04's new outer end is closer to KOTOR's. On the profile, one gave a tie and the other preferred the new eye. Both say the new eye still stops about 30% short of KOTOR's long, pointed wedge.

## The outer corner, as found (task 1)

The pictures are in `$R/pics/corner_live.png` (eight heads, KOTOR against live) and `$R/pics/cuts.png` (cross-sections).

- Seen in profile, KOTOR's eyeball shows as a long wedge. On the median head it reaches 11.7 mm behind the eyeball's front, and on pmhc04 13.4 mm. It ends in a narrow point far back.
- Ours shows as a short half-lens. On the median head it reaches 5.4 mm, and on pmhc04 7.3 mm. It ends at a near-vertical edge.
- Our lids are not the cause. On pmhc04 the two lid edges stay 2 to 7 mm apart up to 13 mm behind the eyeball's front, and the outer tip sits 14 mm back, past the eyeball's middle.
- The cause is the skin beside the outer corner. On our heads it wraps round the eyeball's side about 1 to 3 mm outside it. Seen from the temple, it hides the back half of the opening (`$R/pics/hide2.png`, yellow).
- On KOTOR the skin beside the outer corner runs into the eyeball, so the eyeball's side sticks out past the skin (`$R/pics/cuts.png`, top views).

So the "blunt, open end" in the brief is not a gap between the lids that we lack. Our profile is short because the head's side covers the eyeball. KOTOR's profile is long because its eyeball pokes out of the side of the head.

## The measure (task 2)

The diag reads the eyeball's pixels in profile from the temple side and aligns two outlines on the eyeball's front. It scores their overlap divided by their union. The solver reads the same outline as 9 proportions in `eyes.rs`:
- the outline's height at 2, 5, 8 and 11 mm behind the eyeball's front (profile_h2 to profile_h11);
- the height of its middle there, against the eyeball's centre (profile_m2 to profile_m11);
- how far behind the front it still shows (profile_len).

The four checks:
- **It ignores what nobody notices.** The left and right eyes of one KOTOR head overlap 0.85 on the median head. On each of the 9 proportions, the median gap between KOTOR's left and right eye is 4 to 8 times smaller than KOTOR's spread. For example, profile_h5's gap is 0.4 mm against a spread of 1.7 mm.
- **It sees what David notices.** On live, our outline overlaps KOTOR's only 0.32 on the median head, with a range of 0.11 to 0.62. Our eyeball shows to 5.4 mm against KOTOR's 11.7. pmhc04 was already one of the better heads after round 3 (0.46).
- **It tells KOTOR's heads apart.** Two different KOTOR heads overlap 0.47 on the median pair, against 0.85 for two eyes of one head.
- **Its extremes look right** (`$R/pics/extremes_s.png`). comm_a_f and pmhc01 score lowest. KOTOR's lid piece covers the top of comm_a_f's eyeball, and pmhc01's eyeball barely shows on ours. n_daraalah and Meetra score highest, and they do look alike. old_a_f has KOTOR's longest wedge.

One head misreads. pfhc02's KOTOR eye has a strap across it, and its two eyes overlap only 0.04. Its outline is not a fair target.

## The slider Outer corner opening (task 3): tried, not built

I prototyped it in the diag (`$R/diag/src/main.rs`, functions `tuck` and `thin`). It was never put in the crate. The pictures are `$R/pics/t_tuck*.png` and `$R/pics/t_thin_s.png`.

1. **The tuck** moves the skin round the outer corner toward the head's middle, to just under the eyeball's side. The eyeball then hides that skin, which works like a cut without changing the mesh. It does make the eyeball show far further back: up to 15 mm, and profile_ball up to 72 mm². But the bared eyeball sits outside the lids, behind the outer tip, as a separate grey lobe with a band of lid margin and lashes across it. That is the torn hole the brief forbids. It would also stay bare when the lids close. Limiting the tuck to skin in front of the tip, averaging it with neighbouring vertices, and keeping the lid margin out of it all made it smoother. None removed the lobe or the band.
2. **Thinning the lids** brings the skin near the outer lid edges toward the eyeball's centre, so the lids lie thin on the eyeball. It does not help, and profile_ball falls from 34.9 to 31.3 mm². The skin moves along the line of sight, so it still covers the same place.
3. **The existing sliders** Outer corner angle 35, Outer corner height 1.5 and Outer corner position 2 together give the blunt, open end the brief describes, and an overlap of 0.64 on pmhc04 (`$R/pics/t_exist_s.png`). But Outer corner angle alone folds the skin on 43 heads (stage B notes), and the eyeball still shows only 7.8 mm back.

What would work is a construction change, not a cut. The outer part of the opening and the lids next to it must face the temple, so that the eyeball's side shows between the lids from the side, as on KOTOR. See "Is round 5 needed?".

## The blink (task 4)

The diag now builds the head with the lids closed (`DIAG_SHUT=1`). Its command `shutcheck` measures how much eyeball shows from the front and in profile with the lids shut.
- On all 57 heads, no eyeball shows in profile with the lids shut, before or after.
- From the front, the new presets show less or the same eyeball as live on every head except Kyle Katarn. His new preset showed 0.53 mm², where live shows none: a dark sliver under the outer corner (`$R/pics/shut_cmp.png`). Kyle therefore keeps his live values.
- pmhc04 shows 0.67 mm² closed, against 1.27 live. Its small sliver at the outer corner was there before.
- Heads already over 0.05 mm² on live, unchanged by this round: Layne 0.98, mod_hp_pfha01 0.31, old_a_f 0.28, pmha02 0.27, n_daraalah 0.21, n_hussef 0.06.

## What changed

The code is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint/crates/character_customisation/head/head_sliders/src/`. Nothing is committed. The crate's 5 library tests pass.

1. `eyes.rs`: PROPORTIONS has 43 rows. Nine new rows hold the profile outline (see "The measure"). `profile_ball` now returns a `Profile` with the area and the outline. `measure_eye` takes it. The messages for the new rows are in `gap_message`'s match.
2. `eyes.rs`: a new command `sliders eyes marks-pmhc-r4 <data>` adds the new rows to `eyes/marks.json`. Each row has KOTOR's spread and a mark of a quarter of it. It also prints the left-against-right wobble.
3. `gap_weight` is unchanged in code. The published solve set the weights with `EYES_WEIGHTS="profile_ball=1.5;profile_h2=2;...;profile_len=2"`, so every outline row weighs 2 and the area 1.5. These weights should go into `gap_weight` before the next eye solve, or the next solve will weigh the outline at 1. I did not do it, so that the crate matches what is live.
4. The solve script is `$R/trial4.sh`. It is round 3's solve, with a warm start from the live page and Outer corner depth clipped at -2, on a copy of live data with the new marks (`$R/a0`).
5. The guard is in `$R/guard.py` and `$R/apply_guard.py`. A kept head takes its whole live page entry, with its measures padded to 43 by empty values. Its made.bin shape is copied from live with round 2's `copymade.py`. The published folder is `$R/g5`. The kept heads are listed in `$R/keep_g5.txt`.
6. The diag is a copy at `$R/diag` with its own target `$R/diagt`. Round 3's diag is unchanged. New commands: `corner`, `tcorner`, `shape`, `hide`, `cuts`, `loop` and `shutcheck`, plus `DIAG_SHUT`, and the prototypes `tuck` and `thin` (see the comments in `main.rs`).
7. Live files: `web/eyes/{page.json,made.bin}` and `eyes/{marks,report,audit}.json`. `sliders.json`, `starts.bin`, `made.json` and `sliders_solve.json` are unchanged. The head_sliders wasm was rebuilt with `~/.claude/tools/likeness-fit/sliders/build_wasm.sh head_sliders`. viewer2 `sliders.js` VER is ?b=48 and SLIDERS_MODULE_VER is 48. `nose.js` VER is ?b=48, and `nose/index.html` loads nose.js?v=42.
8. Shared log: two lines from round 4 in `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-slider-kit-shared-changes.md`.

pmhc04's values, before and after:
- Eye depth -3.73 to -2.83.
- Eye prominence 2.98 to 2.69.
- Eye spacing 0.62 to 2.24.
- Eye tilt 1.79 to 1.32.
- Upper lid fullness -1.52 to -0.20.
- Lower lid drop, middle 0.10 to -0.96.
- Lower lid drop, outer 1.47 to 0.65.
- Outer corner position 0.73 to 3.56.
- Outer corner height -0.74 to 0.08.
- Upper lid cover 1.46 to 0.43.
- Eye width (5.54), Eyeball size (-0.89), Inner corner position (-1.98) and Outer corner depth (-2.00) barely moved.

## The solves

Each solve is on all 57 heads, against live. "Overlap" is the outline's overlap divided by union, against KOTOR.

| Solve | Weights | Median overlap | Heads better / worse | pmhc04 | Bastila | Heads with more folds |
|---|---|---|---|---|---|---|
| Live (?b=47) | | 0.32 | | 0.46 | 0.33 | |
| s1 | outline 1, area 0.5 | 0.42 | 39 / 8 | 0.44 | 0.51 | 13 |
| s2 | outline 1, area 1.5 | 0.47 | 45 / 3 | 0.55 | 0.49 | 22 |
| s3 (published, guarded) | outline 2, area 1.5 | 0.58 | 50 / 4 | 0.60 | 0.36 | 21 |

Unguarded, s3 brings the outline much closer on almost every head. Most of those heads fold more, so the guard keeps them.

## The guard

A head keeps its live values when its new preset has any of these:
- more fold triangles than live (44 heads);
- stray eyeball in the front view (mod_hp_pfhb02, old_a_f and old_b_m);
- worse by more than the wobble on three or more of the diag's measures (26 heads);
- an outline overlap more than 0.03 below live (n_daraalah and pfha06);
- a worse closed eye (Kyle Katarn);
- an outline the judges found worse (Bastila);
- a KOTOR outline that cannot be trusted (pfhc02, whose new eyeball also bulged far out).

After the guard, on all 57 heads:
- No head has more fold triangles than live. n_admoffh went from 22 to 14.
- No head shows stray eyeball.
- No head is worse on three or more measures.
- No head's outline got worse.
- No closed eye got worse.

The five new heads, outline overlap before and after:
- pmhc04 0.46 to 0.60
- Zherron 0.37 to 0.59
- Jolee 0.21 to 0.61
- pmha04 0.36 to 0.60
- n_admoffh 0.30 to 0.40

## The judges

Each head had its own sheet, with KOTOR in the middle row and before and after above and below it, drawn at random (`$R/judge/head1.png` to `head7.png`, key in `$R/judge/key.json`). Under the three rows, each sheet shows the three profiles enlarged and both outlines laid on KOTOR's. The second judge saw the sheets with the sides swapped (`$R/judge/swap/`). Both were Sonnet at medium effort. They scored the outer end, the profile and the whole eye from 1 to 5.

Agreed verdicts, after decoding:
- Zherron, Jolee and Kyle Katarn: the new preset is closer, by both judges, on the profile and on the whole eye.
- pmha04: one judge found it closer, the other found it even.
- n_admoffh: the same, by both judges.
- Bastila: both judges found the profile worse after. The new eyeball shows above KOTOR's outline. She keeps her live values.
- pmhc04: the judges split, each preferring row B. That is the row bias round 3 also saw.

For pmhc04 I then asked two Opus judges about the profile and the temple view alone, enlarged, with the sides swapped (`$R/judge/prof1.png` and `prof2.png`):
- On the outer end, both judges preferred the new eye: 3 against 2 in both orders.
- On the profile, one judge gave a tie (2 and 2) and the other preferred the new eye (3 against 2).
- Both said the new eye's wedge is longer and narrows more, but still stops about 30% short of KOTOR's point and ends rounded.

My own look agrees (`$R/pics/before_after_s.png`). The new pmhc04 profile is a longer wedge that narrows toward the back. The front view is close to before.

## Viewer check

A Sonnet sub-agent loaded viewer 2 at ?b=48, and I looked myself. There are no console errors. The eye panel shows its 29 sliders. Bastila and pmhc04 render with both eyes. pmhc04 shows dark, spiky pieces of hair on the top of its bald head and at the ears. Its head thumbnail, made before this round, shows the same pieces, so they are not from round 4. They look like stray hair, and the hair work may want to know.

## Pictures

All are under `$R/pics/` and `$R/judge/`.
- `before_after_s.png`: eight heads, with KOTOR, before and after in profile and from the temple side, then the outlines before and after laid on KOTOR's. pfhc02 is in it but was not published.
- `corner_live.png`: the outer corner enlarged, KOTOR against live, on eight heads, with the outlines.
- `hide2.png`: what hides the eyeball in profile. Skin within 1 mm of the eyeball is green, 1 to 3 mm yellow, and further red.
- `cuts_s.png`: cross-sections of pmhc04 and pmha01, from above and across the depth, KOTOR against live.
- `t_tuck5_s.png`, `t_thin_s.png` and `t_exist_s.png`: the slider prototypes and the existing sliders on pmhc04.
- `shut_cmp.png`: closed eyes, Kyle Katarn and pmhc04, before and after.
- `extremes_s.png`: the measure's extreme heads.

## Choices made without David

- The outline measure replaced the area as the main profile target, as kjk-48 asked. The area keeps its round 3 weight of 1.5. Each outline row weighs 2.
- The new slider was not built, because every form of it bared eyeball outside the lids.
- The guard is stricter than round 3's. It also keeps heads whose outline got worse or whose closed eye got worse.
- Bastila, Kyle Katarn and pfhc02 keep their live values.
- I published, though pmhc04's nose_skin_z rose by 1.0 mm, just under its wobble.

## How to undo the round

1. Copy back the live files from `$R/live_before/`: `web_eyes/*` to `~/.claude/tools/likeness-fit-data/sliders/web/eyes/`, and `eyes/*` to `.../sliders/eyes/`.
2. Copy back `$R/live_before/wasm/head_sliders/*` to `~/.claude/tools/likeness-fit/hq/wasm/head_sliders/`.
3. Raise the versions again rather than lowering them, so browsers reload (VER, SLIDERS_MODULE_VER, nose.js VER and nose.js?v). `$R/live_before/viewer/` holds the three files from before.
4. For the code, copy `$R/orig/head_sliders_src/eyes.rs` over `eyes.rs`, if no other session has edited it since 08:40 (check the shared log). Then rebuild the wasm. The crate and the live wasm must agree.

## Traps I hit

- A slider name with a comma ("Upper lid, outer point") breaks the diag's `TRIALS` lists, which split on commas.
- The diag's `Scene` cannot be cloned. Build KOTOR's scene again where it is needed.
- Moving skin sideways, along the line of sight, changes nothing seen from that side. Thinning the lids failed for this reason.
- The fold logs list heads in their own order. Compare them by name, never line by line. I misread n_admoffh this way at first.
- Judges on stacked sheets still prefer a row. Enlarged side-by-side pictures with the sides swapped gave an agreed answer, as in round 3.

## Is round 5 needed?

Yes, if David wants KOTOR's long wedge in profile. This round got pmhc04 to about two thirds of KOTOR's length with the existing sliders. No existing slider can go further.

Round 5 would rebuild the outer part of the eye so that the eyeball's side shows between the lids from the temple, as on KOTOR:
- The lids' last third, toward the outer corner, would run back along the eyeball's side, open toward the temple, instead of wrapping round it.
- The skin beyond the outer corner would sit inside the eyeball's side, not 1 to 3 mm outside it.
- The closed lid piece must then cover the larger opening. That needs a change in `head_eyes/src/piece.rs` and in the press, with the blink checked on all 57 heads.

I would build it as a field before the press, like Outer corner depth, and measure it with profile_len and the outline overlap. Cutting skin is not the answer here. Every cut I tried bared eyeball outside the lids.

Smaller items for round 5:
- Put the outline weights into `gap_weight` (see "What changed", point 3).
- The 44 heads whose solve folded would gain a lot from the outline measure if their folds can be fixed.
- Bastila's new eyeball showed too much above KOTOR's outline.

## For David

- **Viewer 2:** http://127.0.0.1:8765/viewer2/index.html, version ?b=48. Choose the head "Male C4", which is pmhc04.
- **What to look at on pmhc04:**
  - In profile, from the temple side: the white of the eyeball now runs further back and narrows toward the back, closer to KOTOR's long wedge. It still stops about a third short of KOTOR's point.
  - From the temple side in three-quarter view: the opening closes more gently toward the outer corner.
  - From the front: the eye is nearly the same as yesterday.
- **Other heads that changed:** Zherron, Jolee, pmha04 (a human male) and n_admoffh. The judges found Zherron and Jolee clearly closer to KOTOR.
- **Bastila did not change.** Her new preset showed more eyeball in profile, but too much of it sat above KOTOR's outline.
- **Your skin-cutting idea:** I tried it. Cutting away the skin beside the outer corner does show more eyeball in profile, but the eyeball then shows outside the lids, as a bare lobe behind the corner. It would also stay bare when the eye closes. KOTOR's look comes from lids that run back along the eyeball's side, not from a hole. That is the next round's job.
- **The pictures that show it best:**
  - `/private/tmp/claude-501/-Users-david-perso-kjk/123bc42e-dd62-40a5-afb2-78e3aa2d0b67/scratchpad/r4/pics/before_after_s.png` shows KOTOR, before and after for the changed heads, with the outlines laid on KOTOR's.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/123bc42e-dd62-40a5-afb2-78e3aa2d0b67/scratchpad/r4/judge/prof1.png` shows pmhc04's profile enlarged. A is after, the middle is KOTOR, and B is before.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/123bc42e-dd62-40a5-afb2-78e3aa2d0b67/scratchpad/r4/pics/t_tuck5_s.png` shows why skin cutting was not used.
