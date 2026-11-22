# pmhc04's and Bastila's eyes, round 6 (2026-11-21)

Written by the conversation kjk-31 for the lead kjk-48. Read it with round 5's note `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-pmhc04-eyes-round5-2026-11-21.md`. The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/56726613-db12-42bb-8bdd-853b3b1f6a68/scratchpad/r6/`, called `$R` below.

## Verdict

Nothing was published. Viewer 2 stays at `?b=48`, and no live file changed.

The sliders can now reach both targets, and both judges found the result clearly closer to KOTOR on pmhc04 and on Bastila. But the solve gets there by moving the whole eye forward and further apart. That buries the inner corner and casts a dark hollow under the eye in three-quarter view. No preset for either head passed the guard, so the brief's rule says not to publish.

What the full re-solve (`$R/s1`) gives, against live:
- pmhc04: the profile overlap went from 0.59 to 0.75 (target above 0.70). The dip under the eye went from 5.6 to 1.5 mm (target below 3). nose_skin_z went from 5.22 to 4.82 mm and outer_bulge from 5.97 to 2.47 mm (KOTOR 1.76 and 3.22).
- Bastila: the profile overlap went from 0.33 to 0.71. Seen from the temple side, her opening now reaches 22.2 mm, where live reaches 15.9 and KOTOR 25.8. Its outer end is pointed: 1 mm from the tip it is 1.15 mm tall, where live is 2.9 and KOTOR 0.9.
- Over all 57 heads, the median profile overlap went from 0.33 to 0.62, and 51 heads got closer.

What stops it:
- On pmhc04 the eye moved 2.5 mm forward (depth 15.4 mm, KOTOR 12.1, live 12.9). The space between the two eyes grew from 61.7 to 67.2 mm (KOTOR 60.0). The opening grew from 30.4 to 35.7 mm long (KOTOR 26.6). Bastila's eye moved 3.7 mm forward.
- The lower lid then stands further out over the cheek, and the skin below it falls away into shadow. The judges saw this as a hollow under the eye, though the skin is not concave.
- pmhc04's inner corner sinks 1.4 mm deeper. Bastila's outer_bulge gets worse (4.15 to 3.02 mm, KOTOR 6.61), and she gains 2 fold triangles. 42 heads fold more.
- Holding the eye's place at live loses the gains (solves s6, s7 and s8, below).

## Bastila's opening is read right

Her KOTOR mesh has a large lid piece, which runs on past the opening toward the temple. I drew her KOTOR eye with the eyeball in white, the lid piece in violet and the skin in grey, and the measure's opening dots in red (`$R/pics/tags_b_s.png`). From the front the dots follow the white edge exactly and stop where the white ends, not at the lid piece's tail. In profile and from the temple side, the measures read eyeball pixels only, so they read the real wedge under the lid piece. Every number on Bastila in this note can be trusted.

## What the pointed, far-back corner is

From 50 degrees toward the temple, KOTOR's opening runs round the eyeball's side almost to its edge, as a long, thin wedge (`$R/pics/h03_s.png`, the eyeball in white where it shows and coloured where skin hides it). Ours stops halfway, at a round end. Skin 1 to 3 mm outside the eyeball covers the rest. From the front the two openings are about as long, because the far part of KOTOR's corner lies on the eyeball's side, where the front view does not see it.

This holds on almost every head, not only Bastila. Seen from the temple side, the opening's median length is 21.7 mm on KOTOR and 15.6 mm on ours. Its height 1 mm from the tip is 1.0 mm on KOTOR and 2.3 mm on ours. pmhc04's KOTOR corner is also a thin wedge there (1.1 mm), not a blunt end. Its blunt look comes only from the profile.

## The new measures

Four rows were added to the eye measure (`eyes.rs`, 47 rows now). Each was checked on KOTOR's 57 heads in the way round 4 did.
- **temple_len:** how long the opening looks from 50 degrees toward the temple. A head's left and right eyes differ by 0.7 mm on the median head, against a spread between heads of 3.8 mm. The shortest heads are n_terenaadareh and Zherron, and the longest are old_a_f and old_b_f. Their pictures look right (`$R/pics/ext_tlen_s.png`).
- **temple_tip:** the opening's height 1 mm from its outer end, divided by its height 6 mm from it, seen from the same side. Left against right 0.02, spread between heads 0.18. A low value is a pointed end.
- **front_tip:** the same ratio seen from the front. Left against right 0.02, spread 0.16. Bastila's KOTOR value is 0.28, among the most pointed. pmhc04's is 0.40.
- **under_dip:** the deepest hollow in the skin under the eye, in mm. It is read along four lines across, 6, 9, 12 and 15 mm below the eyeball's centre, and two lines down, under the eye's middle and under its outer corner, from 6 to 18 mm below the centre. On each line it takes the largest gap under a straight bridge over the skin. Places where the eyeball or a lid piece lies in front of the skin are skipped, because there the line looks through the opening. Left against right 0.17 mm, spread 1.11 mm. KOTOR's lowest heads (pmha01, old_a_m) have smooth skin under the eye. Its highest (mod_hp_pfha01, pmba01) have a puffy lower lid with a crease below (`$R/pics/ext_dip_s.png`).

KOTOR's dip under the eye is 1.67 mm on pmhc04 and 1.63 mm on Bastila.

## The hollow under the eye

kjk-48 relayed that David means the area below the lower lid, down over the cheekbone. The measure above reads that whole area.
- Live pmhc04 dips 5.6 mm, the worst of all 57 heads (KOTOR 2.1 by the diag's reading). The dip is deepest just under the lower lid and fades by the cheekbone: 5.6, 3.7, 2.2 and 1.1 mm on the lines at 6, 9, 12 and 15 mm.
- Eye width makes most of it. At 0 instead of 5.54, the dip falls to 1.6 mm. Outer corner position at 0 instead of 3.56 takes it to 3.3 mm.
- The cheek sliders do not make it. Setting Cheekbone width, Cheekbone prominence, Cheek fullness or Lower cheek width to 0 changes it by 0.1 mm at most.
- The cause is how Eye width works. It turns the skin round the eyeball's centre, so the skin beside the outer corner swings back with the corner. The skin further toward the temple is not turned and stays forward, so a trough forms between them.
- In the solves, the measure kept the skin convex: pmhc04's dip fell to 1.5 mm. But a second, different hollow then appeared in three-quarter view, from the eye moving forward (see the verdict). under_dip does not see that one, because the skin is not concave there. It is a steep slope. lower_depth (how far the lower lid stands in front of the cheek) and depth do see it.

## Outer lid opening without notches

The notches sat 2 to 4 mm from the tip on the lower lid's edge. There the sink stepped from one vertex to the next (`$R/pics/z_base_s.png`, values 0 and 2). Longer fades did not remove them and lost the sharper tip.

The fix averages the slider's movement with each vertex's neighbours three times (`eye_curves::smooth_open`). Only vertices the slider moves take part, and the lid edges never move. The steps are gone at 2 on Zherron and pmhc04 (`$R/pics/z_s36.png`, two values each for three passes and six). The five heads of round 5's sheet are clean at 2 (`$R/pics/fr_s3_hi_s.png`).

Above 2 the slider gains nothing. Bastila's tip stays at 2.5 mm from the front, and folds jump to 55 heads at 2.5 and to all 57 at 3. The range is now 0 to 2. At 2 on every head, it adds 19 fold triangles to live's 93.

## Can the existing sliders make Bastila's corner?

Yes, but only with the eye's placement moving too. By hand on Bastila (`$R/pics/t_combo_s.png`):
- Outer lid opening 2, Outer corner position 3.9 and Outer corner depth -4: the opening seen from the temple side reaches 23 mm. It ends in a thin strip with small back-facing patches.
- Outer lid opening 2, Outer corner position 2.5 and Outer corner depth -3: a pointed wedge close to KOTOR's from the temple side and in profile, with one small back-facing patch.
- Outer corner roundness does nothing to any of the tip measures. That is why it was never in the solve.

So no new slider was added. Outer corner depth needs to go to -3, past round 3's clip of -2.

## The solves

Every solve is on all 57 heads, warm-started from the live page, with Outer corner depth clipped at -3. The new rows weigh 2, and under_dip weighs 2.5 (through `EYES_WEIGHTS`). "Placement" counts heads whose spacing, depth, width or height moved further from KOTOR by more than half its mark.

| Solve | What changes | Median overlap | Folds (live 93) | pmhc04 overlap, dip | Bastila overlap |
|---|---|---|---|---|---|
| s1 | all sliders free | 0.62 | 325 | 0.75, 1.5 mm | 0.71 |
| s2 | Eye width at most 3 | 0.60 | 194 | 0.56, 1.9 mm | 0.55 |
| s3 | Eye width at most 4.5 | 0.60 | 266 | 0.75, 1.5 mm | 0.42 |
| s4 | placement weighs 3 to 5 | 0.55 | 335 | 0.46, 2.5 mm | 0.41 |
| s6 | the eye's place, tilt, eyeball and inner corner held at live | 0.57 | 473 | 0.51, 3.7 mm | 0.45 |
| s7 | as s6, Eye width at most 4 | 0.56 | 390 | 0.54, 3.7 mm | 0.55 |
| s8 | Eye depth, prominence and spacing held at live | 0.60 | 499 | 0.56, 3.9 mm | 0.67 |

- Capping Eye width alone does not keep the eye in place. The solve then moves the eye apart and forward instead.
- I also pulled s1's and s2's values back toward live, to 75% and 50% of the change. At 50% of s1, folds fall to 71, below live, and 53 heads still gain overlap. pmhc04 then reaches 0.63 and Bastila 0.51, and both still fail the guard on their placement.
- The guard (`$R/guard6.py`) is round 4's plus: the dip under the eye, the length seen from the temple side, the placement, and on pmhc04 and Bastila any step back on nose_skin_z or outer_bulge. 18 heads pass it in at least one candidate. pmhc04 and Bastila pass it in none.

## The judges

Two Opus judges each saw one sheet per head. KOTOR was in the middle, live and s1 above and below at random, and the second judge had the sides swapped. The sheets show five views: the front, 50 degrees toward the temple, profile, a three-quarter view down over the cheekbone, and both eyes. Under them are the outlines in profile laid on KOTOR's. The sheets are `$R/judge/*_j.png`, and `key.json` holds which side is which.

Both judges found s1 closer on both heads, in both orders. For s1 they gave 4 for the outer corner from the front, 4 from the temple side, 3 to 4 in profile and 3.5 to 4 overall. For live they gave 2, 2, 2 to 3 and 2 to 3. Both named the same fault in s1: a dark hollow under the eye in the three-quarter view, where KOTOR's skin is smooth and full (2 against live's 3 to 4). One also saw a pale band of upper lid skin above the lashes on Bastila, in both versions.

My own look agrees (`$R/pics/look_pm.png`, `$R/pics/v_cmp.png`). s1's corner is long and pointed like KOTOR's. The eye looks longer and more open, and it sits more forward over a shadowed cheek.

## What changed

The code is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint/crates/character_customisation/head/head_sliders/src/`. Nothing is committed, and the live wasm was not rebuilt. The live data has neither the new rows nor Outer lid opening, so viewer 2 is unchanged.
1. `eyes.rs`: the four rows (`temple_view`, `under_dip`, `front_tip` in `measure_eye`) and their messages. `marks-pmhc-r4` also prints KOTOR's value per eye with `MARKS_LIST=1`. `make-pmhc-r5` makes Outer lid opening from 0 to 2 and smooths its field. Its hint now says the corner ends in a sharper point.
2. `eye_curves.rs`: `smooth_open`, 3 passes (`EYES_OPEN_SMOOTH` for trials).
3. `bench.rs`: `SOLVE_FIX="<slider>;..."`, for trials only. With `SOLVE_WARM`, it holds those sliders at the earlier page's values.
4. `gap_weight` is unchanged. The trials passed the weights by `EYES_WEIGHTS`. If the new rows are used, they need weights in the code.
5. Shared log: one line from round 6. Roadmap: a new eye line under "Now and next".
6. Diag: `$R/diag`, with its own target `$R/diagt`. New commands: `tags` (eyeball, lid piece and skin by colour, `TAGS_LIVE=1` for ours), `tip` (the opening's ends, with `TRIALS`), `fronts` (front close-ups of the outer corner at several values), `views` (the judging views), plus `HIDE_YAW` for `hide` and `HOLLOW_UNDER=1` and `HOLLOW_MIRROR=1` for `hollow`.
7. Scripts in `$R`: `trial6.sh` (a solve and all its checks), `evalb.sh` and `blend.py` (pulled-back values), `guard6.py`, `summ6.py` and `variant6.sh` (fades and smoothing of Outer lid opening).

## How to undo the round

Nothing live changed. To undo the code, copy `$R/orig/head_sliders_src/` over `head_sliders/src/` and `$R/orig/head_eyes_src/` over `head_eyes/src/`, if no other session has edited them since 11:10 (check the shared log). `head_eyes` did not change in this round.

## Traps I hit

- A solve can trade the eye's placement for the outline. The round 4 and 5 guards did not look at spacing, depth or width. Check `eye_audit props` against KOTOR before trusting an overlap gain.
- A dark band under the eye in three-quarter view can be a slope, not a hollow. under_dip reads only concave dips. lower_depth and depth catch the slope.
- KOTOR's eyeball can show below a line under the eye. A skin-only picture then looks into the socket, which gave dips of 37 mm until those places were skipped.
- zsh does not split an unquoted variable into words. A list of candidates in one variable reaches a script as one argument.
- In this shell, `cp` is GNU (`/bin/cp -c` clones on APFS) and `diff` is aliased to difftastic. `=====` in an echo is a zsh expansion.
- Editing a script while another run of it is going is safe with `sed -i`, which writes a new file. Rebuilding the diag while a trial uses it is not, so I built the judging diag into `$R/diagt2`.

## For a follow-up round

- The missing piece is a slider that sinks the skin beside and below the outer corner inside the eyeball's side, so that the eyeball's side shows as on KOTOR while the eye stays where it is. Round 4's tuck tried this and bared eyeball outside the lids. Outer lid opening shows how to keep the lid edges fixed while sinking skin, so the two ideas could be joined.
- Eye width should stretch the skin under the eye along the eyeball instead of turning it, so it stops cutting a trough.
- If David likes s1's corner enough to accept the eye moving, s1's pmhc04 and Bastila presets are in `$R/s1/web/eyes/page.json`. Their faults are the shadow under the eye and a buried inner corner.
- 18 other heads pass the guard in some candidate, with gains in overlap of up to 0.35 (mod_kyle_katarn 0.29 to 0.64, old_b_f 0.38 to 0.69). Their closed eyes and front close-ups are not checked yet.

## For David

- **Viewer 2 did not change.** It is still `?b=48`.
- **Bastila:** the measures read her real eye opening, not her large lid piece. Her KOTOR corner runs a long way round the side of the eyeball, as a thin pointed wedge. Ours stops halfway, at a round end. The same is true, less strongly, on almost every head.
- **What works:** the sliders can now make that pointed, far-reaching corner on both heads. Two judges, who did not know which picture was which, found the new eyes clearly closer to KOTOR on both. Outer lid opening no longer makes notches at the corner.
- **Why it is not live:** to get there, the solve pushes the whole eye about 3 mm forward and further apart. The lower lid then stands out over the cheek, and a dark hollow shows under the eye in three-quarter view. Keeping the eye in place loses most of the gain.
- **The hollow you pointed at on pmhc04** comes from Eye width, not from the cheek sliders. Without that trough the skin under the eye is close to KOTOR's.
- **The pictures that show it best:**
  - `/private/tmp/claude-501/-Users-david-perso-kjk/56726613-db12-42bb-8bdd-853b3b1f6a68/scratchpad/r6/judge/pmhc04_j.png` and `/private/tmp/claude-501/-Users-david-perso-kjk/56726613-db12-42bb-8bdd-853b3b1f6a68/scratchpad/r6/judge/p_bastilah_j.png` show live on top, KOTOR in the middle and the new solve at the bottom, in five views.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/56726613-db12-42bb-8bdd-853b3b1f6a68/scratchpad/r6/pics/tags_b_s.png` shows Bastila's KOTOR eye, with what the measure reads as her opening.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/56726613-db12-42bb-8bdd-853b3b1f6a68/scratchpad/r6/pics/h03_s.png` shows from the temple side how far KOTOR's opening reaches round the eyeball, against ours.
