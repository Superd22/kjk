# Brief: refit the skull under KOTOR's hair

You are a full conversation started by the head HQ on 2026-11-21. David (the user) approved this step.

On the KOTOR heads with modelled hair, our bald head's skull sits on the outside of KOTOR's hair. That makes the face look small on a head that is far too big. David called our pmhc04 "a fucking alien". Your job is to refit the skull of those heads so that it sits under the hair, where a real skull would be. You change nothing else, and nothing goes live.

## Read first

1. The note of the conversation that found this, with its table, code and traps: `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-bald-references-2026-11-21.md`. Its code is `bald.rs` in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, crate `head_sliders`, run as `sliders bald ...`.
2. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`, whole. Especially "Change only what was asked", "Refit after a construction changes", "Guard every other head" and "Honest reports name what is broken first".
3. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: "For agents", "Now and next" and "Decisions". This serves B2 for the skull, and A2.
4. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.

## What is known (measured by the last conversation)

- On the 38 KOTOR heads whose skull shows, our skull is within half of KOTOR's spread on every skull measure. The base head and the sliders are fine.
- On the 19 heads with modelled hair, our bald skull lies at KOTOR's outer surface, hair included. The median gap is +3.6 mm in width, −1.0 mm at the top and +3.1 mm at the back. The cause is the FLAME fit (`~/.claude/tools/likeness-fit/scan/trials-2026-11-18/kotor_fit.py`). It pulls every on-scan vertex to KOTOR's surface, and its hair test (`kotor_hairless.py`, by texture colour) fails both ways. On pmhc04 it called the hair volume skin. On about 35 heads it called painted short hair "not skin", though that surface is a real skull. Viewer 2's live head has the same skull as the fit, so the presets play no part.
- On pmhc04, against its bald reference, the skull is 157.5 mm wide against 125.2 mm, and its top is 110 mm against 98 mm. It is 10 mm too long at the back, and the forehead is 6° more upright. On Bastila the skull is 22 mm too wide.
- The bald references of pmhc04 and Bastila keep KOTOR's real skin and fill the skull from the 38 skull-showing heads. The fill stays at least 6 mm under the hair. Tested by leaving one head out, it lands about 3 to 4 mm from a real skull, with a worst case near 6 mm.
- pfha02 and pfhc02 may be painted hair volumes rather than skulls. Decide by looking, and move them to the haired set if so.

## What to do

1. **Build a bald reference for each of the 19 haired heads,** the same way as for pmhc04 and Bastila. Never cut inside the face, between the brows, the eyes' outer corners and the chin. Leave the ears out, because they are later work. Look at a sheet of all 19 before going on. Fix any head where the cut removed real skin or kept hair.
2. **Refit the skull only,** in a new output folder, behind an option that is off by default. Do not change the default behaviour of `kotor_fit.py` or any shared script. Change only what was asked: the face zones keep their fit. The guard below checks it.
   - **The target.** On each haired head, the skull's target is the bald reference: the filled skull where there was hair, and KOTOR's real skin everywhere else.
   - **The weight.** The filled skull carries 3 to 6 mm of uncertainty. So give it a weaker pull than real skin, and keep the skull smooth.
   - **The rim.** The neck's rim never moves (decision 44).
   - **The back of the neck** is out of scope. David said it is fixed by the body seam.
3. **Guard.** Measure every head before and after. A head takes the new skull only when all of these hold:
   - its skull measures land within half of KOTOR's spread of its reference (or say by how much they miss);
   - no face zone gets worse by more than its wobble;
   - the jaw and throat measures (`sliders underjaw`) do not move;
   - the skull stays inside KOTOR's hair everywhere;
   - nothing folds.

   Also measure the 38 bald heads before and after, and confirm they do not change.
4. **Show David.** Put the refitted heads in viewer 2 behind a URL option, the way the jaw prototype used `?shapes=data/jawproto/shapes.json&shape=jaw_proto`. The default page must not change. Give links for pmhc04 and Bastila with `hair=none`, and say whether KOTOR's hair still sits over the new skull without a gap that shows. Draw the profile and front overlays of pmhc04 and Bastila again, with the old head, the new head and the reference.

No judges in this step. The measures decide, and David judges in viewer 2.

Then stop. Do not put the new skulls live, do not refit the slider presets, and do not rerun the animation calibration. List what going live would need, for the next step:
- the presets;
- the calibration (decision 62);
- the hair caps and cards of part D, which sit on the scalp;
- the paint layout.

## Rules

- Do the building yourself. Launch no sub-agents except for quick research that would block you.
- Rust for anything heavy (decision 53). For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`, and is fine for reading files and for glue.
- Another conversation, `throat-corner`, works in `head_sliders` on the jaw (`jaw_edge.rs` and a throat file). Do not edit its files.
- Do not commit, and do not change the main kjk checkout.
- Start without asking for a go-ahead. Stop for David only when something goes against this brief, or when a choice would change what gets built.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ a report that starts with the verdict and names what is broken first. Keep apart what was measured, what was seen and what was not checked. Include:
- the 19 references and any heads moved between the two sets;
- the before and after numbers per head, with the guard's result;
- the viewer 2 links, and the overlay pictures as full paths;
- what going live would need;
- how to undo everything.

Write the note for the next conversation at `$TMPDIR/kjk-skull-refit-2026-11-21.md`, with its traps. Update the roadmap's "Now and next" when you start and when you finish.
