# Brief: the lips' shape, round 2

You are a full conversation started by the head HQ on 2026-11-22. David (the user) is asleep. Before going, he said: "happy for you to fold anything that's clearly better in the defaults, happy for you to run a couple iterations otherwise on the ongoing works."

The first lips conversation stopped after 4 loops. Its result is not in viewer 2, because it was not clearly better. Your job is to fix the faults it named, in at most two loops. Fold the result into viewer 2's default only if it is clearly better.

## Read first

1. The last conversation's note, with its code, numbers, undo steps and traps: `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-lips-shape-2026-11-22.md`. The code is `lips.rs` in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, crate `head_sliders`, run as `sliders lips ...`. You own it.
2. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`, whole.
3. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: "For agents", "Now and next" and "Decisions". This serves B3 and B5 for the mouth.
4. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.

## What the last conversation found

- **Measured.** The profile measure passes checks 1, 3 and 4: 0.28 mm on the same head, against 1.07 mm between heads. There are 7 named lip sliders, solved on all 57 heads with a strict guard. The median gaps to KOTOR (middle, 10 mm to the side, front) fell from 0.58 / 0.71 / 2.46 mm to 0.41 / 0.59 / 1.52 mm. No head got worse on any outline. None of David's four candidates (sith_app, Bastila, Carth and pmhc03) meets all three pass marks (0.40 / 0.60 / 1.10 mm). The guard kept sith_app's lips as they were.
- **Broken:**
  1. The front measure cannot read the lip border on many of our smooth heads, because our lips have no crease at the border. This misled the solve on pfha05: it made her upper lip taller where KOTOR's is thin. An idea to test: on our mesh the lip border is known, from the vertex rows that the lip paint uses (`head_paint`, the lip border), so read it from those rows instead of from the shading.
  2. Our mouths are 9 mm narrower than KOTOR's at the median. The Mouth width slider cannot fix it, because the build's last smoothing spreads the corner move along the lips. The note suggests building mouth width as a step after the smoothing.
  3. Cupid's bow was dropped for the same reason.
  4. The corner sliders fold at their far ends on pmhc03, Carth and sith_app. Repair the ends before capping, as the guide says.
  5. sith_app's open mouth makes its front unreadable. Read it in a way that copes with an open mouth, or say why not.

## What to do

Fix 1 to 5, then solve all 57 heads again with the same strict guard. Always look at Bastila next to pmhc04 and next to David's four candidates, and at pfha05.

- At most two loops. Do not chase perfection.
- No judges. The measures decide.
- Look at your own close-ups in viewer 2's "In game" light, mouth closed and in two talk shapes, before you call anything clean.

## Folding

The result is clearly better when all of these hold:
- the guard passes with no head worse on any outline;
- the four candidates' gaps shrink from today's;
- pfha05's upper lip is no taller than KOTOR's;
- nothing folds at the solved values;
- your close-ups are clean.

If it is clearly better, make it viewer 2's default, with no URL option. Delete any preview code it does not use, and write how to undo it. If it is not, keep the default page unchanged and report it with the overlap sheets. Shipping into kjk itself waits.

## Rules

- Do the building yourself. Launch no sub-agents except for quick research that would block you.
- Rust for anything heavy (decision 53). For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`.
- Other conversations work in the same crate and in viewer 2:
  - `jaw-skull-build` owns `under_jaw.rs`, `jaw_edge.rs`, `throat_corner.rs`, `chin_neck.rs`, `skull.rs`, `bald.rs` and `jaw.rs`.
  - `nose-nostrils` owns `nose.rs` and `nose_profile.rs`.

  Do not edit their files. Keep shared changes small, and log them in `kjk-slider-kit-shared-changes.md`. The eye conversation owns the eye files.
- Do not commit, and do not change the main kjk checkout.
- David is asleep, so do not stop to ask him. Stop for the HQ only when something goes against this brief, or when a choice would change what gets built outside the lips, such as new topology.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ one report at the end. Start with the verdict and whether it was folded, and name what is broken first. Keep apart what was measured, what was seen and what was not checked. Include:
- the gaps per candidate and for pfha05, before and after;
- the overlap sheets as full paths;
- how to undo everything.

Write the note for the next conversation at `$TMPDIR/kjk-lips-round2-2026-11-22.md`, with its traps. Update the roadmap's "Now and next" when you start and when you finish.
