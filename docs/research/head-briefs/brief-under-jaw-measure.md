# Brief: the underside of the jaw and the jaw corner, measure first

You are a full conversation started by the head HQ on 2026-11-21. David (the user) wants to work on the part of the head that joins the jaw and the chin to the neck. Our heads are weak there. Your job is the first step: look at KOTOR, prove a measure, and find out what the existing sliders cannot reach. You do not build new sliders in this conversation.

## Read first

1. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`. It is the method for this work. Follow its checklist (section 6), steps 1 to 5. Its traps are real. Read it whole.
2. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: its "For agents" rules, "Now and next" and "Decisions". This work serves B1, B2 and B3 for the jaw and chin zones, and prepares B5.
3. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.
4. The jaw and chin slider report: `~/.claude/tools/likeness-fit-data/sliders/reports/report-kit-jaw.md`. The slider kit's handoff, with its commands: `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-slider-kit-handoff-2026-11-20.md`.

## What the HQ found today

**Pictures.** KOTOR next to our slider-made jaws, for nine heads (Carth, Canderous of the mod, pmhc03, old_a_m, Zherron, Bastila, pfha02, comm_w_f and sith_app): `/private/tmp/claude-501/-Users-david-perso-kjk/52003e36-b8ca-4208-a733-a6d51b417e4e/scratchpad/jaw/look_0.jpg` and `look_1.jpg` in the same folder. They were made with `sliders jaw made-sheet <data> <head>` and `SLIDERS_SHEET=<out.png>`, one head per call, because several heads in one call panicked.
- From the side, KOTOR's line under the jaw runs back from the chin almost level, then turns sharply into the neck at the jaw's corner. Ours runs as one straight slope from the chin down to the neck, with no corner and no throat angle. It is clearest on Carth, Canderous, old_a_m, Zherron and sith_app.
- From below, KOTOR's jaw has a clear outline, shaped like a V or a U. Ours fades into the neck.
- From the front, our neck looks as wide as our jaw. On KOTOR the jaw stands out wider than the neck.

**Numbers, on the FLAME fits** (`~/.claude/tools/likeness-fit-data/kotor-heads/fits_v6_smooth`). The fit's score in `kotor_fit.py` (`zones`) stops 4 mm below the chin's lowest point and skips anything more than 20 mm behind the eyes. So the underside of the jaw and the jaw corner are never scored. The HQ's probe scripts are `probe.py` and `variety.py` in the same scratchpad folder, with the regions in `regions.npz`. These are surface distances in mm. The guide says they are not a likeness measure, so treat them as a pointer only.

| Area | Our distance from KOTOR | KOTOR's spread between heads | Variety kept | Same pairs |
|---|---|---|---|---|
| Jaw (scored) | 0.41 mm | 2.29 mm | 97% | 0.98 |
| Chin (scored) | 0.51 mm | 2.50 mm | 98% | 0.97 |
| Jaw corner (not scored) | 0.61 mm | 3.17 mm | 94% | 0.98 |
| Under the jaw, down to 35 mm below the chin (not scored) | 1.22 mm | 3.79 mm | 97% | 0.87 |

KOTOR's heads differ more under the jaw than in any scored zone. KOTOR's meshes barely reach the neck below that (4.5% of our upper-neck vertices have KOTOR skin under them).

**The sliders.** The 5 jaw sliders (jaw width, chin length, chin width, chin projection, under the chin) do not shape this area. The kit dropped a jaw-corner landmark because it jumped by up to 62 mm. The "Under the chin" angle measure cannot tell KOTOR's heads apart (wobble 8.0° at the 95th percentile against a spread of 6.3°). The jawline slope from the side has no slider. Seven heads have hair, a hood or a modelled beard over the jaw, and Malak's jaw is metal (see the report).

## What to do

Follow the guide's checklist, steps 1 to 5.
1. **Look at KOTOR.** Draw the jaw and the throat on all 57 KOTOR heads in profile, three-quarter and from below, with the hair off. Say in plain words how KOTOR's jaw corner and the line under the jaw vary.
2. **Step 0, the premise.** The premise is that KOTOR's heads have a jaw corner and a throat angle that vary between heads by more than a measure's wobble. Check it on KOTOR's own 57 meshes. If it does not hold, stop and report.
3. **Build the measure** in the slider kit (`head_sliders`, Rust, a new or extended feature). The HQ's idea is to read the profile outline: where the line under the jaw meets the back edge of the jaw, and the angle between the jaw's underside and the neck. A from-below outline of the jaw's edge may also be needed. These are ideas only, and the guide's four checks decide. Write down each measure's wobble, KOTOR's spread and the extremes sheet.
4. **Test the existing sliders.** Solve a few heads, including Bastila, Carth and pmhc03, with only the existing sliders and the new measure. Name what is out of reach. Optionally, also add an option to `kotor_fit.py`, off by default, that scores the underside of the jaw. Then see how close FLAME gets there when it is asked to.

Then stop. Do not prototype or build new sliders. That is the next conversation.

## Rules

- The neck's rim never moves (decision 44). Anything you build must keep it at 0.000 mm.
- Do not change anything live in viewer 2, and do not change the default behaviour of a shared script. New behaviour goes behind an option that is off by default.
- Do the building yourself. Launch sub-agents only as judges of your pictures: two on Fable, each given only what to judge and the picture paths, never the history, the code or this brief. Do browser checks, slider checks, code review and research yourself.
- Rust for anything heavy (decision 53). For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. The kit lives in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`.
- Do not commit, and do not change the main kjk checkout.
- Start without asking for a go-ahead. Stop for David only when something goes against this brief, or when a choice would change what gets built.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ a report that starts with the verdict and names what is broken first. Keep apart what was measured, what was seen and what was not checked. Include:
- the measure and its four checks;
- the pictures, as full paths;
- what the existing sliders cannot reach, by head;
- a proposal for the sliders or construction of the next step.

Write a note for the next conversation at `$TMPDIR/kjk-under-jaw-measure-2026-11-21.md`, with its traps. Add a line to the roadmap's "Now and next" when you start, and update it when you finish.
