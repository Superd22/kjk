# Brief: the profile from the chin to the neck, measured and solved

You are a full conversation started by the head HQ on 2026-11-21. David (the user) approved this step.

David looked at pmhc04 (Male C4) and Bastila in profile in viewer 2. He said that he still hates the transition from the chin and the underside of the jaw to the neck, on both heads. Your job has two parts:
1. Build and prove one profile outline measure for that transition.
2. Put the throat construction on top of the refitted skull, turn its values into named sliders, and solve them toward that outline.

Then stop for David. The skull sliders and the full solve of all 57 heads come after, in another step.

## Read first

1. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`, whole. This is checklist steps 4 to 7, stopping before the full solve.
2. The notes of the four conversations before you. They hold the code, the numbers and the traps:
   - `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-under-jaw-measure-2026-11-21.md`, the under-jaw measure in `under_jaw.rs`;
   - `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-under-jaw-prototype-2026-11-21.md`, the failed lift in `jaw_edge.rs`;
   - `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-throat-corner-2026-11-21.md`, the construction in `throat_corner.rs`;
   - `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-skull-refit-2026-11-21.md`, the skull refit in `skull.rs`.

   All of the code is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, crate `head_sliders`. You now own all four files. No other conversation is working in them.
3. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: "For agents", "Now and next" and "Decisions". This serves B3 and B5 for the jaw and chin.
4. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.

## What is known

- **Proven measures** (they pass check 1):
  - the throat angle (pmhc04: ours 128°, KOTOR 90°; Bastila: about 113° against 103°);
  - the sharpest 10 mm of the bend (KOTOR turns 75°, ours 22°, sharper on KOTOR on all 45 heads that can be read);
  - the outline from the chin into the throat (wobble 0.4 mm median and 1.0 mm at the 95th percentile, against KOTOR's spread of 3.1 mm).
- **Unproven measures**, which fail check 1: the jaw corner's place, the ledge, and the outlines of the jaw's edge from the side and from below. These are the parts David reacts to. On our presets the line under the jaw ends 8 to 40 mm below the chin's bottom, where KOTOR's ends 8 to 22 mm above it. Our ledge is about 10 mm, against 13 to 19 mm on KOTOR.
- **The throat construction** (`throat_corner.rs`, `Construction` and `Corner`) builds, in each section across the jaw, an outline from under the chin level to a corner under the ear, then down to the neck's front 34 mm below the chin, with the corner rounded. It has five values: strength, corner depth, corner height, jaw corner rise and radius. On pmhc04 at hand-set values it reached a 77° turn, against KOTOR's 91°. David said its profile was "way better". The judges found a ledge across the throat in three-quarter view from below, a notch at the jaw's corner and bumps on pmhc03's neck. Its note suggests placing a row of vertices on the corner line rather than drawing skin onto it.
- **The skull refit** (`skull.rs`, `sliders skull refit`) moves the skull's vertices of 21 haired heads toward bald references. David said the skull is better. It is a free movement of vertices, which no slider makes yet. That is the next step, not yours.
- **Fable judges chose by position** on every stacked sheet in two conversations today, even with the sides swapped.

## What to do

1. **Build the chin-to-neck profile measure.** Make it one outline, from the chin's front, along the underside of the jaw, round the corner, and down the front of the neck to the rim. Read it at the middle and at sections toward the jaw's corners, for example 0, 15, 30 and 45 mm from the middle. Compare outlines after taking out what the guide says to take out. Read KOTOR on its own meshes, and hide heads whose neck stub is too short or whose jaw is covered, as the under-jaw measure does.
   - Prove it with checks 1, 3 and 4.
   - Draw our outline over KOTOR's in profile for all 57 heads. Use both today's live head and the skull-refit head.
   - You are done when the measure passes. If the corner and the ledge only pass as part of the whole outline, say so.
2. **Combine the two prototypes.** Put the throat construction on top of the skull-refit head. Place a row of vertices along the corner line, or say why not.
   - Fix the ledge across the throat, the notch at the jaw's corner and pmhc03's bumps.
   - Keep the neck's rim at 0.000 mm (decision 44). Leave the lips and the inside of the mouth still, and keep the skin smooth.
3. **Turn the five values into named sliders.** Give each a plain name and one main effect, following the guide's "One name, one main effect". Strength is probably not a player slider. Check that the result does not depend on the order of the other jaw sliders. Draw every slider at both ends, and every pair at its corners. Use close-ups in profile, three-quarter and from below, in viewer 2's "In game" light, with the hair off and with the mouth open in a talk shape.
4. **Solve.** Solve pmhc04, Bastila, Carth, Zherron, pmhc03 and five more heads toward the new outline measure with the new sliders and the existing jaw sliders. Hold the face, the chin's front and the rim. No values are set by hand. Report the outline gap before and after per head.
5. **Show David.** Put the result in viewer 2 behind a URL option, with the default page unchanged. Add live controls for the new sliders if you can. Give links for pmhc04 and Bastila with `hair=none`, and profile pictures of KOTOR, today's head and the new head with the outlines drawn over each other.

No judges in this step. The measure and David decide.

Then stop. Do not build skull sliders, do not solve all 57 presets, and change nothing live.

## Rules

- Do the building yourself. Launch no sub-agents except for quick research that would block you.
- Rust for anything heavy (decision 53). For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`.
- Do not change the default behaviour of a shared script or of the live page. Do not commit, and do not change the main kjk checkout.
- Start without asking for a go-ahead. Stop for David only when something goes against this brief, or when a choice would change what gets built.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ a report that starts with the verdict and names what is broken first. Keep apart what was measured, what was seen and what was not checked. Include:
- the measure and its checks;
- the sliders with their ranges and ends;
- the outline gaps per head, before and after;
- the viewer 2 links, and the profile pictures as full paths;
- how to undo everything.

Write the note for the next conversation at `$TMPDIR/kjk-chin-neck-profile-2026-11-21.md`, with its traps. Update the roadmap's "Now and next" when you start and when you finish.
