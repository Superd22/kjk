# Brief: a sharper bend where the underside of the jaw meets the neck

You are a full conversation started by the head HQ on 2026-11-21. David (the user) looked at our head in profile in viewer 2. The underside of the jaw turns into the front of the neck through a long, soft, rounded bend. On KOTOR's heads that bend is much tighter, close to a corner. David asked why we cannot simply make that bend more abrupt. Your job is to answer that with a measure and a prototype slider, "Throat corner sharpness".

David's screenshot of the soft bend is `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/TemporaryItems/NSIRD_screencaptureui_NXEAjx/Screenshot 2026-11-21 at 22.05.37.png`. If it is gone, the bend is easy to see in profile in viewer 2 with `hair=none`.

## Fixed by David

- **The neck's rim never moves** (decision 44). David restated it on 2026-11-21: we keep compatibility with KOTOR's bodies. Everything you build fades to 0.000 mm before the rim.
- This is a change to the bend only. The angle between the underside and the neck, the line under the jaw and the line of the neck stay where they are. Only the skin inside the bend moves.

## Read first

1. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`, whole. Follow its checklist (section 6), steps 3 to 6.
2. The two notes before you. They hold the measure, the code, the failed prototype and its traps:
   - `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-under-jaw-measure-2026-11-21.md`
   - `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-under-jaw-prototype-2026-11-21.md`
3. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: "For agents", "Now and next" and "Decisions". This work serves B3 for the jaw, and prepares B5.
4. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.

## Why the last prototype is not the answer

The prototype in `jaw_edge.rs` (`sliders jawedge`) changed the throat angle. It did that by lifting the whole underside of the jaw toward a ceiling. All four judges saw a crease along the jaw's lower edge, a notch at the jaw's corners and a ridge down the throat. It won on pmhc04 and lost on Carth and pmhc03, whose KOTOR throats are open. Its measure reads the throat angle, the corner and the ledge, but never how tight the bend is. So it was blind to what David saw.

## What to do

1. **Step 0, the premise.** Measure the bend's sharpness on KOTOR's 57 own meshes, on the live presets, and on the prototype's solved heads. Read it in the profile at the middle of the throat, and also in sections across the width. One way is the distance from the meeting point of the two tangent lines (the line under the jaw and the line of the neck) to the skin. Another is the bend's radius. You choose, by the four checks.
   - **Trap:** KOTOR's meshes are low-poly. Two flat facets always meet in a sharp edge. Do not let the measure reward facets. Read KOTOR on its subdivided copies as well (check 1), and compare with KOTOR's mesh resolution.
   - If KOTOR's bend is not clearly tighter than ours across most heads, beyond the measure's wobble, stop and report.
2. **Build the prototype slider "Throat corner sharpness"** on code that viewer 2 does not use. It moves the skin inside the bend toward the tangent lines' meeting point, section by section across the throat. It fades out toward the jaw's corners and the ears, and before the neck's rim. It must not move the lips or the inside of the mouth. The skin must stay smooth, with no crease and no ridge.
   - Make it work on top of the live head, so it is a change David can judge alone.
   - Try it on top of the prototype's Throat angle as well, and report what that pair does on pmhc04.
3. **Set values by hand, then solve.** Do it on pmhc04, Bastila, Carth, Zherron and pmhc03. Carth and pmhc03 have open throats on KOTOR, and must not get worse. Draw the slider at both ends and at its corners with Throat angle and Jaw corner height, on the base head and the stand-ins. Use close-ups in profile, three-quarter and from below, in viewer 2's "In game" light and in clay, with the hair off. Check the mouth open in a talk shape too.
4. **Judge.** Use two blind Fable judges, with KOTOR in the middle row, live and the prototype above and below at random per head, and the rows swapped for the second judge. When they split by row, show enlarged profile crops side by side, with the sides swapped, to two more judges. Give each judge only what to judge and the picture paths. The gate passes with a clear win on both pmhc04 and Bastila, and no loss on Carth or pmhc03. Fix once, judge again, then stop whatever the result.

Then stop. Nothing goes live, and no presets are refitted. Give David a viewer 2 link behind a URL option, like the last prototype's `?shapes=data/jawproto/shapes.json&shape=jaw_proto`, with the default page unchanged. Ideally the link has a live control for the slider, so that David can drag it himself.

## Rules

- Do the building yourself. Launch sub-agents only as judges of your pictures, as above. Do browser checks, slider checks, code review and research yourself.
- Rust for anything heavy (decision 53). The kit is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, crate `head_sliders`. For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`.
- Do not change the default behaviour of a shared script or of the live page. Do not commit, and do not change the main kjk checkout.
- Start without asking for a go-ahead. Stop for David only when something goes against this brief, or when a choice would change what gets built.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ a report that starts with the verdict and names what is broken first. Keep apart what was measured, what was seen and what was not checked. Include:
- step 0's numbers: KOTOR's bend against ours, with the wobble;
- the slider's range and what its ends look like;
- the judges' verdicts by head, in both orders;
- the viewer 2 link and the picture paths;
- how to undo everything.

Write the note for the next conversation at `$TMPDIR/kjk-throat-corner-2026-11-21.md`, with its traps. Update the roadmap's "Now and next" when you start and when you finish.
