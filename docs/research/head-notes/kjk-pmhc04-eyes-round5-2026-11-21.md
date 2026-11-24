# pmhc04's eyes, round 5 (2026-11-21)

Written by the conversation kjk-0f for the lead kjk-48. Read it with round 4's note `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-pmhc04-eyes-round4-2026-11-21.md`. The scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/0c5f07bb-30b6-44f4-a2bd-6443c7170ee3/scratchpad/r5/`, called `$R` below.

## Verdict

Nothing was published. Viewer 2 stays at `?b=48`, and no live file changed.

Round 5 built the slider the brief asked for, named Outer lid opening. It works in profile: the eyeball shows further back, as a wedge, and the closed eye still covers it. But it was not good enough to publish, for two reasons:
- From a value of about 1.5, the slider gives the outer corner small notches seen from the front. On Zherron, n_terenaadareh and mod_jaesa they are clear at 2. The guard's measures did not catch them. I found them in close-ups, and one judge saw Zherron's.
- pmhc04's best new preset raised the outline overlap from 0.595 to 0.699. Its eyeball showed 9.35 mm behind its front instead of 8.30 (KOTOR 13.35). outer_bulge went from 5.97 to 3.24 mm (KOTOR 3.22), and nose_skin_z went from 5.22 to 5.17. But the two Opus judges split, each preferring the top row. One found the new profile closer (4 against 3), the other found it a blunt, trapezoid end (2 against 3). The brief asks for a clear improvement, so I stopped.

Bastila, the second check, could not take the slider at all. Already at 0.5 her outer corner seen from the front moves away from KOTOR's (outer_bulge 4.15 to 3.58 mm, KOTOR's 6.61).

## What hides the back of the opening (found this round)

Round 4 said that the skin beside the outer corner hides the eyeball's side. The exact cause is the lid's roll:
- The lid edges in `lid_edges.json` lie on the eyeball (0.3 mm off it on pmhc04) and run back to 15 mm behind its front. The opening is already long enough.
- In the outer third, the lid skin just beside each edge folds back over the opening, 1.5 to 3 mm off the eyeball. This is the roll that `head_eyes/src/piece.rs` describes: from the edge the skin runs down to the lid's margin, then out and up. Near the outer tip the upper and lower rolls meet over the opening. Seen from the temple, that roll covers the back of the opening.
- The roll is in the base mesh. It is there on every head, also with every eye slider at 0.

The diag's commands `edges` and `who` show it (`$R/pics/edges_live.png`).

## The slider Outer lid opening

It is a field before the press (`eye_curves.rs`, `Curve::OuterOpen`), so the same value gives the same result on any head. The value is in mm, from 0 to 3.
- Lid skin that lies inside the opening (between the two lids' edges, by its angle on the eyeball) sinks toward the eyeball's centre by up to the value, but no deeper than 0.3 mm under the eyeball's surface. The eyeball then hides it from every side.
- It works from 75% along the eye, fully from 90%. It fades out from 84% of the way to the tip over 12%, so the roll stays at the tip and closes the end.
- The sink grows over 1.5 mm in from each lid's edge. The edges themselves and the skin beyond them never move, so no eyeball shows outside the lids.
- These fades came from a sweep (`$R/variant.sh`). Sharper fades lengthened the wedge more but left a skin tab at the tip and folded every head (412 fold triangles at 2, against 93 on live). The chosen ones add 29 fold triangles over all 57 heads at 2.

The lid piece (`piece.rs`) now ignores roll vertices that lie under the eyeball's surface when it finds the lid's margin. Without this, the piece's outer end hung down into the opened gap. On live heads nothing is sunk, so nothing changes (outlines of all 57 heads equal within 0.008).

The blink: with the chosen fades the closed eye covers the larger opening. Over all 57 heads on the assembled page, the eyeball seen from the front with the lids closed went from 2.89 to 2.03 mm² in total, and none showed in profile.

## The solves

Every solve is on all 57 heads, warm-started from the live page, with the round 4 outline weights now in `gap_weight`.

| Page | Median overlap | Better / worse | pmhc04 | Heads with more folds |
|---|---|---|---|---|
| Live (`?b=48`) | 0.325 | | 0.595 | |
| Full solve (s2) | 0.619 | 52 / 3 | 0.707 | 39 |
| Full solve, Eye width capped at live (s2e) | 0.553 | 48 / 5 | 0.669 | 26 |
| Live values plus the opening alone, at most 2 (c2) | 0.360 | 38 / 0 | 0.595 | 0 |
| Assembled page (f), each head's best passing candidate | 0.360 | 25 / 0 | 0.699 | 0 |

- The full solve brings the outline much closer, but most heads fold. On pmhc04 the folds come from Eye width rising from 5.54 to 6.0.
- On pmhc04 the new slider barely moves the overlap. It lengthens the wedge (8.65 to 9.35 mm) and fixes outer_bulge (5.60 to 3.24 mm). The overlap gain comes from the other sliders.
- The assembled page (`$R/f`) passed round 4's guard on every head: no new folds, no stray eyeball, no head worse on three measures, no outline worse, no closed eye worse. It is the page I would have published.

## The judges

Two Sonnet judges saw seven heads, KOTOR in the middle, before and after at random, the sides swapped for the second (`$R/judge/`, key in `key.json`). Both found the new preset closer or equal on every head. Kreia, pmhc01, old_a_m and Zherron were clearly closer in both orders. pmhc04 was a tie for one judge and a split for the other.

Two Opus judges then saw pmhc04's profile and temple view enlarged (`$R/judge/prof1.png`, X is after; `prof2.png`, X is before). Each preferred row X. That is the row bias rounds 3 and 4 saw, and no clear answer.

My own close-ups (`$R/pics/ba_0.png` to `ba_2.png`) show clean corners in the temple and profile views on all 25 changed heads. The front close-ups (`$R/pics/front_levels.png`, values 0, 1, 1.5 and 2) show the notches from 1.5.

## What changed

The code is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint/crates/character_customisation/head/`. Nothing is committed. The crate's 5 library tests pass. The live wasm was not rebuilt, and the live data has no Outer lid opening, so the code changes have no effect on viewer 2.
1. `head_sliders/src/eye_curves.rs`: `Curve::OuterOpen` and `outer_open_fades` (`EYES_OPEN_FADES` for trials).
2. `head_sliders/src/eyes.rs`: SPECS has 30 entries, with Outer lid opening (measure profile_len). New command `sliders eyes make-pmhc-r5 <data>` adds it to a data folder. `gap_weight` now gives the nine outline rows weight 2, as round 4's published solve used.
3. `head_eyes/src/piece.rs`: `sunk`, used where the margin is found.
4. `head_sliders/src/bin/eye_audit.rs`: `faultsat` with `EA_WHERE=1` also prints how far each folded triangle lies outside the eyeball.
5. Shared log: one line from round 5.
6. Diag: `$R/diag` with its own target `$R/diagt`. New commands `edges`, `who`, `pieceshow` and `hollow`, and `DIAG_OPEN` or `open=` in TRIALS.

## How to undo the round

Nothing live changed. To undo the code, copy `$R/orig/head_sliders_src/` over `head_sliders/src/` and `$R/orig/head_eyes_src/` over `head_eyes/src/`, if no other session has edited them since this round (check the shared log).

## Traps I hit

- The diag's `probe` hangs on closed eyes, so `tcorner` with `DIAG_SHUT=1` never ends. Draw closed eyes with `corner` or `sheet`.
- A field is computed on the base head, where the eye is much shorter than on pmhc04 (the tip 7.5 mm behind the front, against 15). Weights by depth behind the front do nothing there. Use the share along the eye.
- Moving the roll up and down cancels on the opening's middle line, where the two rolls meet. Sinking works there, sliding does not.
- Pages store values under the count of solved sliders (21 on live, 22 now). A kept head's entry must move to the new count's key (`$R/apply_guard5.py`).
- Slider names with commas still break comma-split lists. `EA_SET` splits on semicolons and is safe.
- The guard's measures do not see a notch at the outer corner seen from the front. Close-ups from the front at the chosen value are needed.

## For a follow-up round

- Make the sink smoother where it meets the lid edge in the front view, then retry values up to 2. The notch sits where the sunk roll meets the unsunk roll near the tip.
- The full solve's outline gain is large (median 0.33 to 0.62). It is held back by folds from Eye width at its high end. A solve with Eye width limited, or a fold term in the solve, would let many heads keep that gain.
- pmhc04's preset candidate is in `$R/f` (values in `$R/f/web/eyes/page.json`). It is ready if David likes it in pictures.

## The hollow round pmhc04's eye

kjk-48 asked about a concave hollow round pmhc04's eye. I measured it with the diag's `hollow` command. Along five lines across the socket seen from the front, it takes the skin's depth and finds the deepest dip below a straight bridge over it (the line's convex hull).
- The hollow is under the eye. On the line 7 mm below the eyeball's centre, pmhc04 dips 5.1 mm, against KOTOR's 1.6. The brow line, 11 mm above, dips 1.5 mm against 0.4.
- Eye width makes most of it. Setting Eye width from 5.54 to 0 takes the dip under the eye from 5.1 to 2.6 mm. Outer corner position (3.56 to 0) takes it to 3.3, and Outer corner depth (-2 to 0) to 4.3. With every eye slider at 0 it is 2.0.
- Outer lid opening does not deepen it (5.12 at 2, against 5.14).
- A follow-up could add the dip under the eye as a measure with KOTOR's value as its target, so the solve keeps the skin under the eye convex. Another way is to make Eye width stretch the skin under the eye along the eyeball instead of pulling it in.

## For David

- **Viewer 2 did not change.** It is still `?b=48`.
- **What was tried:** a new slider, Outer lid opening, sinks the skin that rolls over the back of the eye opening under the eyeball. In profile the white of the eye then reaches further back, like KOTOR's long wedge. The lid piece still closes over it.
- **Why it is not live:** from the front, the outer corner gets small notches on some heads at the values that help most. On pmhc04 the two judges could not agree that the profile looked better.
- **The pictures that show it best:**
  - `/private/tmp/claude-501/-Users-david-perso-kjk/0c5f07bb-30b6-44f4-a2bd-6443c7170ee3/scratchpad/r5/pics/ba_0.png` shows KOTOR, before and after for nine heads, in the temple view and in profile, with the outlines.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/0c5f07bb-30b6-44f4-a2bd-6443c7170ee3/scratchpad/r5/pics/front_levels.png` shows six heads from the front at values 0, 1, 1.5 and 2, where the notches appear.
  - `/private/tmp/claude-501/-Users-david-perso-kjk/0c5f07bb-30b6-44f4-a2bd-6443c7170ee3/scratchpad/r5/judge/prof1.png` shows pmhc04's profile enlarged. X is the new preset, the middle is KOTOR, and Y is live.
