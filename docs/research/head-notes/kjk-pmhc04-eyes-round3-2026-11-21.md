# pmhc04's eyes, round 3 (2026-11-21, night)

Written by the conversation kjk-72 for the lead kjk-48. Read it with round 2's note `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-pmhc04-eyes-round2-2026-11-21.md`. The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/08f6f3c6-221e-4d46-9c70-3f8501a95db0/scratchpad/r3/`, called `$X` below. Round 2's folder is called `$W`, as in its note.

## Verdict

Part A is done and live in viewer 2 as version ?b=47. Part B was tried and is not published.

Round 3 publishes round 2's work: the press that lets the outer corner sit behind the eyeball, and the slider Outer corner depth. The eye presets were solved again with three changes, so that round 2's regressions do not come back. On pmhc04, against round 1 (the old live version):
- More eyeball shows in profile. profile_ball went from 19.6 to 34.9 mm². KOTOR shows 53.4. Round 2 had reached 25.6.
- The skin round the outer corner bulges less. outer_bulge went from 7.3 to 6.2 mm. KOTOR's is 3.2. Round 2 had reached 4.7, but only with Outer corner depth at -4, which folds the skin (see "The fold check").
- The skin beside the nose stands a little further in front of the eyeball. nose_skin_z went from 3.1 to 4.2 mm. KOTOR's is 1.8. Round 2 had 5.7. The change from round 1 is about the measure's own wobble (1.08 mm).
- The opening is as long and narrow as round 1's again. Inner corner position is -2.12, close to round 1's -2.63. Round 2 had -5.64.
- outer_back is -11.1 mm (round 1 -11.0, round 2 -9.7, KOTOR -12.8).

Two blind judges agree that the new profile is closer to KOTOR (3 of 5 against 2 of 5, in both orders). They found the inner and the outer corners the same as before. My own look agrees: the profile and the temple three-quarter view show a clear wedge of eyeball, and the opening is long again. The pink caruncle at the inner corner is unchanged, and both judges name it as the main remaining fault on every head.

14 heads take new presets. 43 heads keep their round 1 values, Bastila, Kreia and Zherron among them. No head has more folds than live, no head shows stray eyeball (pmhc04 shows 0.03 mm², under a pixel), and no head got worse by more than the wobble on three or more measures.

## What changed

All code is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint/crates/character_customisation/head/head_sliders/src/`. Nothing is committed. The crate's 5 library tests pass.

1. **Three proportions weigh more in the eye solve** (`eyes.rs`, `Eyes::gap_weight`). nose_skin weighs 2.0, outer_bulge 2.5 and profile_ball 1.5. The eyeball's size keeps 0.3, and the rest keep 1.0. Without these weights, the solve gave up the skin beside the nose and the outer bulge to match the opening's outline better. `EYES_WEIGHTS="key=w;key=w"` overrides them for trials (native only). Other features are unchanged.
2. **Two trial options in the solve** (`bench.rs`, `solve_head`), native only, both off unless their environment variable is set:
   - `SOLVE_WARM=<page.json>` adds a third start: each head's values from an earlier page. The best of the three starts by cost wins.
   - `SOLVE_CLIP="<slider>=<low>"` raises a solved value below its low to that low afterwards, without a new descent, and measures the head again.
3. **The published solve** used the weights, `SOLVE_WARM` with round 1's live page, and `SOLVE_CLIP="Outer corner depth=-2"`, on a copy of round 2's guarded folder `$W/w3` (`$X/trial3.sh`). The result is `$X/t11`. The guarded result that went live is `$X/f2`.
4. Round 2's press changes are now live, because the head_sliders wasm was rebuilt from the crate. The crate and the live wasm agree.
5. `head_eyes/src/piece.rs` is unchanged. The visor trial is kept aside in `$X/piece_visor_trial.rs` (see Part B).
6. Shared log: two lines at 03:13 in `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-slider-kit-shared-changes.md`.

pmhc04's new values: Eye depth -3.73, Eye prominence 2.98, Eye width 5.62, Upper lid fullness -1.52, Inner corner position -2.12, Inner corner depth -2.85, Outer corner position 0.73, Outer corner depth -2.00, Eye tilt 1.79, Eyeball size -0.93, Eye opening height -0.38. Upper lid fullness is no longer at its low end (-2.0).

## How the fix was found (Part A)

- A higher weight on nose_skin alone (3.0) moved the solve to a different place. Eye width fell to 1.86, the nose skin reached 4.5 and the outer bulge 6.3.
- Limiting Inner corner position to -3 alone sent the solve to yet another place, with the nose skin at 6.2.
- Round 1's values with Outer corner depth set by hand at -4 met almost every goal on pmhc04 (nose skin 2.0, profile 29, outer bulge 4.9). The solver had simply never started there. That is why `SOLVE_WARM` exists.
- With the warm start but no new weights, the solver moved Outer corner depth back to 0. By its own measure, outer_tip already matched KOTOR there, so the outer bulge needed its own weight.
- With the weights and the warm start, pmhc04 reached Outer corner depth -4, profile 38.8 and outer bulge 5.4. But -4 folded 10 triangles at the outer corners. At -3 the folds went, but a sliver of eyeball showed past the skin at the outer tip in the front view (0.41 mm²). At -2 the corner is clean. Hence the clip at -2.
- A solve with Outer corner depth's range set to -3 (instead of the clip) landed in another place again, with the bulge at 6.9. The solve's landing place is sensitive, which is why the clip is applied after the solve and not as a range.

## Viewer 2

Live as ?b=47. sliders.js VER ?b=47 and SLIDERS_MODULE_VER 47, nose.js VER ?b=47, nose/index.html nose.js?v=41. The head_sliders wasm was rebuilt with `~/.claude/tools/likeness-fit/sliders/build_wasm.sh head_sliders`. The head_eyes wasm was not rebuilt.

A Sonnet agent checked viewer 2 in the browser after the publish. It saw no console errors. pmhc04 ("Male C4") shows Eye prominence +2.98 mm, Outer corner depth -2.00 mm and Inner corner position -2.12 mm. In the eyes' close-up from the front and from the side, the eyes render cleanly, with no holes, no spikes and no eyeball through the skin. It could not confirm the module version from the network, only from the served file.

## How to publish

Done. The files from before are in `$X/live_before/` (web_eyes, eyes, wasm/head_sliders, viewer).

## Measures before and after

The tool is round 2's diag (`$W/diag`), which renders our lid pieces and reads the head frame. "Before" is round 1's `m_f6.json` (live until tonight), "after" is `$X/f2/m_all.json`. `python3 $W/compare.py <before> <after> [heads]` prints the table (`$X/compare_f2.txt`).

All 57 heads, after the guard:

| Measure | Wobble | KOTOR median | Before | After | Median gap before | After | Heads worse by more than the wobble |
|---|---|---|---|---|---|---|---|
| nose_skin_z | 1.08 | 5.01 | 8.59 | 8.33 | 3.53 | 3.41 | 1 |
| profile_ball | 3.92 | 43.3 | 18.9 | 19.7 | 23.0 | 21.9 | 0 |
| outer_back | 0.65 | -12.98 | -9.75 | -10.15 | 2.80 | 2.42 | 0 |
| ball_proud | 0.07 | 4.52 | 3.47 | 3.56 | 1.07 | 1.03 | 1 |
| outer_bulge | 0.67 | 5.43 | 5.80 | 5.98 | 1.67 | 1.73 | 2 |
| inner_tip_z | 0.51 | -6.28 | -5.76 | -5.90 | 1.51 | 1.49 | 2 |

The heads of the sheets, as KOTOR / before → after:

| Head | nose_skin_z | profile_ball | outer_back | outer_bulge | inner_tip_z |
|---|---|---|---|---|---|
| pmhc04 | 1.8 / 3.1 → 4.2 | 53.4 / 19.6 → 34.9 | -12.8 / -11.0 → -11.1 | 3.2 / 7.3 → 6.2 | -5.6 / -8.9 → -9.1 |
| pmha01 | 4.1 / 5.1 → 5.2 | 48.9 / 23.0 → 22.7 | -11.6 / -11.0 → -10.8 | 3.4 / 6.6 → 6.3 | -9.5 / -8.4 → -8.8 |
| p_bastilah (kept) | 1.9 / 5.8 → 5.8 | 47.7 / 21.0 → 21.0 | -15.9 / -8.2 → -8.2 | 6.6 / 4.1 → 4.1 | -5.0 / -7.8 → -7.8 |
| n_zherronh (kept) | 0.5 / 4.6 → 4.6 | unchanged | unchanged | unchanged | unchanged |

Unguarded, Bastila's new preset was better on the measures (profile 40.8, nose skin 3.5), but it had 3 fold triangles against 2 live, so the rule kept her old values.

## The fold check

`eye_audit zones` on all 57 heads, kit count:
- Live before (round 1): 112 triangles (121 in all) on 28 heads (`$W/z_live.log`).
- Round 3 unguarded (`$X/t11/zones.log`): 289 triangles. 34 heads had more than live.
- Round 3 guarded (`$X/f2/zones.log`): 102 triangles (108 in all) on 27 heads. No head has more than live.

## The guard

A head keeps its round 1 values (with Outer corner depth at 0) when its new preset has any of these:
- more fold triangles than live (34 heads);
- stray eyeball in the front view (comm_b_m, mod_hp_pfhb02, old_a_f, old_b_m, old_w_f, p_joleeh);
- worse by more than the wobble on three or more measures (comm_b_m, mod_tcs_bastila, n_zezkaiellh, p_carthh, pmhc03, sith_app, mod_meetra, mod_hp_pfhb02, p_kreiah, pmha04, n_traskh, old_b_m, mod_narve_canderous);
- Bastila, as in round 1;
- Zherron, whose inner corner both judges found worse.

The 43 kept heads are in `$X/keep_f2.txt`. Their made.bin shapes were copied from live with round 2's `copymade.py`. The 14 heads with new presets are comm_a_f, mod_exile_canon, mod_hp_pfha01, mod_improved_bastila, mod_keanu, mod_kyle_katarn, mod_layne, n_daraalah, p_baodurh, pfha06, pmba01, pmha01, pmha02 and pmhc04.

## The judges

I followed the brief's method. Each head had its own sheet, with KOTOR in the middle row and before and after above and below it, drawn at random per head (`$X/judge/head{1..4}.png`, key in `$X/judge/key.json`). The second judge saw the same sheets with the sides swapped (`$X/judge/swap/`). Each judge rated each side from 1 to 5 against KOTOR on the inner corner, the outer corner and the profile. Both were Sonnet at medium effort.

Agreed verdicts, after decoding:
- pmhc04: inner corner the same (2 and 2), outer corner the same (3 and 3). The profile split: each judge gave the top row 3 and the bottom row 2. So it showed only a bias to the top row.
- Zherron: the inner corner got worse (both judges). Zherron therefore keeps its round 1 values.
- pmha01 and Bao-Dur: no agreed difference.

Because pmhc04's profile split, I asked two fresh Opus judges about the profile alone, with the three profiles enlarged side by side (`$X/judge/prof1.png`, `prof2.png`, sides swapped). Both rated the new eye 3 of 5 and the old eye 2 of 5. They described the new one as a medium half-lens of eyeball clearly past the lids, blunter and about half the size of KOTOR's wedge. They described the old one as a thin crescent that barely shows.

## Part B, the lid piece as a visor

Not published. I added a trial setting to the open lid piece (`PIECE_VISOR`, native only, kept in `$X/piece_visor_trial.rs`). It moves the piece's outer face, inner face and edge outward from the blink axis, by the full amount at the edge and fading to 0 at the piece's top, which turns the piece forward about its top.

On pmhc04:
- At 1 mm, piece_reach stays 2.01 mm. At 2 mm it reaches 3.06 mm, past KOTOR's 2.40.
- The eyeball seen in profile shrinks, from 34.9 to 33.8 mm² at 1 mm and 30.4 mm² at 2 mm, because the flared edge hides it.
- The slices (`$X/pics/slab02.png`, top 0 mm, bottom 2 mm) show only a small lip at the edge. The piece stays a thin shell lying on the eyeball. KOTOR's piece is a wedge whose top meets the skin far forward of the eyeball.

So the visor does not clearly help. As the diagnosis already noted, pmhc04's piece reach was close to KOTOR's (2.0 against 2.4 mm). What differs is that our lid skin stands as far forward as the piece. A real fix needs the skin's lid edge set back behind the piece's top, with the piece made as a wedge. That is a construction change in both `piece.rs` and the press, and the blink must be checked on every head. I did not start it. piece.rs is back to its version from before tonight.

## Pictures

All are under `$X/pics/` and `$X/judge/`.
- `before_after_r3.png`: pmhc04 and Bastila, four rows each: KOTOR, round 1 (was live), round 2 (not published) and round 3 (live). The columns are the front, three-quarter from the temple side, three-quarter from the nose side and profile, in clay and then in viewer 2's light.
- `$X/judge/head1.png`: pmhc04's judge sheet in viewer 2's light. Row A is round 3, the middle row is KOTOR, and row B is round 1.
- `$X/judge/prof1.png`: pmhc04's profile enlarged. A is round 3, the middle is KOTOR, and B is round 1.
- `tr1.png`: trials on pmhc04, with round 2's values and with round 1's values plus Outer corner depth.
- `stray_zoom.png`: the sliver of eyeball at the outer tip with Outer corner depth at -3, against -2.
- `slab02.png`: the slices of pmhc04 with the visor at 0 and at 2 mm.

## Heads that got worse

After the guard, no head is worse by more than the wobble on three or more measures. One head is worse on nose_skin_z, one on ball_proud, two on outer_bulge and two on inner_tip_z, each by a little more than the wobble. No judge looked at heads other than pmhc04, Zherron, pmha01 and Bao-Dur.

## Choices made without David

- The three new weights (nose_skin 2.0, outer_bulge 2.5, profile_ball 1.5).
- The solve started also from round 1's values. Without that start, the solver never found the good point for pmhc04.
- Outer corner depth is clipped at -2 on the presets, because -4 folds and -3 shows a sliver of eyeball at the outer tip on pmhc04. The slider itself still runs from -4 to 2 for anyone moving it by hand.
- The guard is stricter than round 2's. It also keeps heads with stray eyeball, heads worse on three or more measures, and Zherron.
- Part B is not published.

## How to undo the round

1. Copy back the live files from `$X/live_before/`: `web_eyes/*` to `~/.claude/tools/likeness-fit-data/sliders/web/eyes/`, and `eyes/*` to `.../sliders/eyes/`.
2. Copy back `$X/live_before/wasm/head_sliders/*` to `~/.claude/tools/likeness-fit/hq/wasm/head_sliders/`. That wasm is round 1's press.
3. Raise the versions again rather than lowering them, so browsers reload (VER, SLIDERS_MODULE_VER, nose.js VER and nose.js?v). `$X/live_before/viewer/` holds the three files from before.
4. For the code, undo round 2 as its note says, and remove round 3's parts by hand: in `eyes.rs`, the `EYES_WEIGHTS` block and the `match` on nose_skin, outer_bulge and profile_ball in `gap_weight`, and the last two lines of its doc comment; in `bench.rs`, the `SOLVE_WARM` and `SOLVE_CLIP` blocks in `solve_head` (and `let mut s` back to `let s`). Then rebuild the wasm. The crate and the live wasm must agree.

## Traps I hit

- In zsh, `echo =====X` fails with "not found", because a word starting with `=` is expanded as a command path. Use printf.
- The solver's measures and the diag's measures differ (eyes frame against head frame). On round 2, the solver read outer_bulge 6.1 mm where the diag read 4.7. Judge targets with the diag, but understand the solve with `page.json`'s "measured".
- The solve's landing place jumps between distant points when a weight or a range changes. Compare whole value sets, not one slider.
- `eye_audit faultsat` with `EA_SET="<slider>=<value>"` checks folds for a hand-set value on top of a page's preset in under a second. It is much faster than a full zones run for trying one head.
- `pmhc_diag trial` with `TRIALS` on a data folder measures and draws hand-set values. It is the quickest way to see what the solver missed.
- A judge shown a profile split across two rows of a large sheet still picks by row. Enlarged crops side by side, with the sides swapped, gave an agreed answer.

## Is round 4 needed?

Only if David asks for it. The next faults, in order of what the judges name:
1. **The pink caruncle at the inner corner.** KOTOR has none, and every judge gave every head 2 of 5 there. The slider Caruncle width exists (-2.5 to more), but it has no measure, so the presets leave it at 0. A measure of how much pink shows in the front view, solved against 0 for KOTOR, would let the presets shrink it.
2. **The lid as a visor.** This needs the skin's lid edge set back behind a wedge-shaped piece, not a flared shell (see Part B).
3. **The outer bulge.** pmhc04 is at 6.2 mm, against KOTOR's 3.2. Going further needs Outer corner depth past -2 without folds or a sliver at the tip, which means fixing the press near the outer tip.

## For David in the morning

- **Viewer 2:** http://127.0.0.1:8765/viewer2/index.html, version ?b=47. Choose the head "Male C4", which is pmhc04.
- **What to look at on pmhc04:**
  - In profile, from the temple side: a wedge of white eyeball now shows past the lids. Before, it was a thin crescent.
  - In three-quarter view from the temple side: the outer corner is a little flatter, and the eyeball runs further back toward the temple.
  - From the front: the eye is long and narrow again, as in round 1. Round 2 had made it rounder.
  - At the inner corner, from the nose side: the pink caruncle wedge is still there, the same as in round 1.
- **Bastila:** her eyes did not change tonight. Her new preset was better on the numbers, but it added a fold, so she keeps round 1's values.
- **The sheets that show it best:**
  - `/private/tmp/claude-501/-Users-david-perso-kjk/08f6f3c6-221e-4d46-9c70-3f8501a95db0/scratchpad/r3/pics/before_after_r3.png` shows KOTOR, round 1, round 2 and round 3 for pmhc04 and Bastila.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/08f6f3c6-221e-4d46-9c70-3f8501a95db0/scratchpad/r3/judge/prof1.png` shows pmhc04's profile enlarged: round 3 on the left, KOTOR in the middle and round 1 on the right.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/08f6f3c6-221e-4d46-9c70-3f8501a95db0/scratchpad/r3/judge/head1.png` shows pmhc04 in viewer 2's light: round 3 on top, KOTOR in the middle and round 1 at the bottom.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/08f6f3c6-221e-4d46-9c70-3f8501a95db0/scratchpad/r3/pics/slab02.png` shows why the visor trial was not published.
- **Still wrong:**
  - The pink caruncle at the inner corner.
  - The eyeball in profile is about two thirds of KOTOR's, and its front is rounded where KOTOR's ends in a point.
  - The skin round the outer corner still bulges more than KOTOR's (6.2 against 3.2 mm).
  - The lid piece is a thin shell on the eyeball, where KOTOR's juts forward like a visor.
- **Next step:** a measure for the caruncle's pink, so that the presets can shrink it with the existing slider Caruncle width. After that, rebuilding the lid piece as a wedge with the skin's edge tucked behind it.
