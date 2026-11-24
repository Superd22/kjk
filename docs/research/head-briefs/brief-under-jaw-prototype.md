# Brief: a prototype of the jaw edge, the throat angle and the neck width

You are a full conversation started by the head HQ on 2026-11-21. David (the user) wants the part of the head that joins the jaw to the neck to look like KOTOR's. A first conversation measured it. Your job is the prototype gate: steps 6 of the guide's checklist, after three fixes to the measure. David approved this step and its three controls.

## Read first

1. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`. It is the method. Read it whole, especially "A prototype gate before a full build", "Test every slider at both ends", "Keep the skin convex and smooth", "Fresh blind judges" and "Honest reports name what is broken first".
2. The note of the measure conversation, with its full table and traps: `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-under-jaw-measure-2026-11-21.md`.
3. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: "For agents", "Now and next" and "Decisions". This work serves B3 for the jaw and chin, and prepares B5.
4. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.
5. The jaw slider report, `~/.claude/tools/likeness-fit-data/sliders/reports/report-kit-jaw.md`, and the kit's handoff, `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-slider-kit-handoff-2026-11-20.md`.

## What the measure found

The code is `crates/character_customisation/head/head_sliders/src/under_jaw.rs` in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, run as `sliders underjaw look|probe|dump|reach|reach-sheet|cut`. Its pictures are in `/private/tmp/claude-501/-Users-david-perso-kjk/7fae1cb7-a4ee-47b5-9bf7-3acaab4f4a13/scratchpad/`, with `reach_sheet.png` as the main one.

Measured, on 56 heads with Malak left out:

| | KOTOR (mean) | Our jaw presets (mean) |
|---|---|---|
| Throat angle, between the jaw's underside and the neck | 123° | 136° |
| Jaw corner, against the chin's lowest point | 14 mm above | 15 mm below |
| The ledge under the jaw | 12.9 mm | 9.4 mm |
| Width of the front of the neck | 72 mm | 102 mm |

- The FLAME fits reach the throat angle (mean 122°) but not the ledge (10.7 mm).
- A solve over the 5 jaw sliders on 10 heads cannot reach the throat corner (pmhc04 needs 90°, Bastila 103°; our solves stay at 116° to 133°). It cannot reach the ledge (13 to 19 mm needed, 9 to 10 mm reached). It cannot reach the width of the jaw's back from below (Zherron 91 mm, pmhc04 77 mm; ours 65 to 66 mm).
- The cause: the jaw zone fades out over 25 mm toward the neck's rim, so no slider has a part under the jaw behind the chin.

## What to do

1. **Fix the measure first.** There are three fixes:
   - Make the corner's reading window taller, because the corner height stops at the window's top (22 mm) on Bao-Dur, pfhc02 and Kyle Katarn.
   - Make the hood cover stricter, so that Kreia's hood no longer sets her ledge.
   - Leave out the throat rows on heads whose KOTOR neck stub is too short (comm_b_m and old_b_f, and any others you find).

   Then rerun check 1 and check 3. Say whether the four rows that failed check 1 now pass: the corner's place, the ledge, and the two outlines of the jaw's edge. If the corner and ledge rows still fail, use the rows that pass, and say so.
2. **Build the prototype.** Build it on code that viewer 2 does not use. There are three controls:
   - **The jaw edge.** A curve from the chin to the jaw corner. The skin above it is pushed out and the skin under it is pulled in, which makes the corner and the ledge. Its handles could be the corner's height, how far back it sits, and how sharp the edge is. You decide, but each handle must be named in plain words and must set one measure.
   - **The throat angle.** One slider.
   - **The neck width.** One slider for the front of the neck.

   All three must fade to 0.000 mm before the neck's rim (decision 44). They must not move the inside of the mouth or the lips, and they must keep the skin convex and smooth.
3. **Set values by hand, then solve.** Set values by hand on pmhc04, Bastila, Carth, Zherron and pmhc03, then solve those heads with the old and the new sliders. Draw every new slider at both ends, and every pair at its four corners. Do this on the base head and on the stand-ins. Use close-ups in profile, three-quarter and from below, in viewer 2's "In game" light as well as in clay, with the hair off. Check the mouth open in a talk shape too, because the jaw bone moves this area.
4. **Judge.** Run two blind judges on Fable. Show KOTOR in the middle row, with live and the prototype above and below it at random per head. Show the second judge the rows swapped. Give each judge only what to judge and the picture paths. The gate passes only with a clear win on both pmhc04 and Bastila. Fix once after the judges, judge again, and then stop whatever the result.

Then stop. Do not put anything live in viewer 2 and do not refit the 57 presets. Both belong to the next step, after David has seen the prototype. Give David a way to see the prototype next to KOTOR. A picture sheet is the minimum. A viewer 2 page variant that loads only behind a URL option is better, as long as the default page does not change.

## Rules

- Do the building yourself. Launch sub-agents only as judges of your pictures, as above. Do browser checks, slider checks, code review and research yourself.
- Rust for anything heavy (decision 53). For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`.
- Do not change the default behaviour of a shared script or of the live page. Do not commit, and do not change the main kjk checkout.
- Start without asking for a go-ahead. Stop for David only when something goes against this brief, or when a choice would change what gets built.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ a report that starts with the verdict and names what is broken first. Keep apart what was measured, what was seen and what was not checked. Include:
- the measure's fixes and its new check results;
- each new control with its range and what its ends look like;
- the judges' verdicts, both orders, by head;
- the pictures as full paths, and how David can see the prototype;
- how to undo everything.

Write the note for the next conversation at `$TMPDIR/kjk-under-jaw-prototype-2026-11-21.md`, with its traps. Update the roadmap's "Now and next" when you start and when you finish.
