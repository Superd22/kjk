# Brief: the lips' shape, from the front and the side, while David is away

You are a full conversation started by the head HQ on 2026-11-21. David (the user) asked for a pass on the lips' shape while he is away from the keyboard. He wants **measurement and overlap**: our lips' outlines drawn over KOTOR's, and scored, **from the front and from the side**.

His candidate heads are:
- sith_app (the Sith apprentice);
- p_bastilah (Bastila);
- p_carthh (Carth);
- pmhc03 (Male C3).

His words on effort: "happy with like 4ish loops, but if we can have something good enough in 2 that's fine, don't chase perfection."

## Read first

1. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`, whole. Follow its checklist (section 6), steps 1 to 9.
2. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: "For agents", "Now and next" and "Decisions". Read in particular the mouth entries: "The mouth's shape (B3)", "The mouth when it talks (A1)" and "Mouth v2". This serves B3 and B5 for the mouth. The mouth's shape was deferred on 2026-11-19 until after the paint. David is now asking for it.
3. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.
4. The slider kit's handoff, which explains how a new feature is added: `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-slider-kit-handoff-2026-11-20.md`. The kit is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, crate `head_sliders`. Look at `nose.rs` and `jaw.rs` as models.

## What is known

- **Variety.** On 2026-11-19, David judged that KOTOR's mouths vary enough (B1). The mouth is the zone where KOTOR varies least, 1.0 mm between two heads, lined up on itself. It is also where our fits keep the least of that variety, 87 to 88%.
- **Learned shapes.** 20 learned mouth shapes plus a corrected average head cut the mouth's error from 0.46 to 0.41 mm on heads they were not learned from. They raise the variety kept from 88% to 91%. Ten hand-made mouth controls gained only 0.006 mm. The scripts are `~/.claude/tools/likeness-fit/scan/trials-2026-11-18/kotor_mouth_*.py`, and the results are in `~/.claude/tools/likeness-fit-data/kotor-heads/mouth_b3/`. They are millimetre fits, not named sliders, and they are not proven likeness measures.
- **The lip line.** The jagged lip line and the soft lip border come from our base mesh's vertex spacing, about 2 mm. Paint draws the lip border (`head_paint`, the lips layer). Paint follows the shape, because it is built from the surface.
- **Moving lips affects animation.** The face rig's weights (Mouth v2, the calibration) and the live rig (`RigKit::rig`) follow the head's shape. A lip slider must keep the opening between the lips, and the mouth must still talk cleanly.
- **The jaw's chin sliders already move the lips** by up to 0.77 mm.

## What to do

1. **Look at KOTOR's lips first,** on all 57 heads, from the front and from the side, with the hair off and in clay. Say in plain words how they vary.
2. **Build the measure** as a new kit feature. It needs at least these parts:
   - the lips' outline from the front;
   - the profile from the nose's base, over both lips, to the chin's fold, at the middle and a little to each side;
   - the proportions: mouth width, the height of each lip, how far each lip stands out, the corners' height and depth, and the Cupid's bow.

   Read KOTOR's side on KOTOR's own mesh, as the guide says. Prove it with checks 1, 3 and 4. Check 2 is David's, so leave it for him. Draw the overlap pictures of KOTOR's outlines over ours, front and side, for all 57 heads. Mark the four candidates.
3. **Test the existing sliders** and the learned shapes on the four candidates, with the new measure. Name what is out of reach.
4. **Build named lip sliders** for what is out of reach, with one name and one main effect each. Draw every slider's ends and pair corners in close-ups in viewer 2's "In game" light, with the mouth closed and in two talk shapes. Keep the neck's rim, the nose, the jaw's measures and the eyes still. The lips' opening must stay open where it is open and closed where it is closed.
5. **Solve all 57 heads,** with the four candidates as the main targets. Guard every head, as the guide says.

## Loops

- **Loop 1** is steps 1 to 5.
- **Each later loop** fixes what the measure and your own close-ups show is worst.
- **Stop after loop 2** if the four candidates are within their pass marks on both the front and side outlines, no head got worse, and the close-ups show no folds or creases. That counts as good enough.
- **Otherwise go on, to at most 4 loops,** then stop whatever the result.

Do not chase perfection.

No judges. David asked for measurement and overlap. The measures decide, and David judges when he is back.

## Show David, then stop

Put the result in viewer 2 behind a URL option, with the default page unchanged. Give a link per candidate with `hair=none`. Make one overlap sheet per candidate: KOTOR, today's head and ours, front and side, with the outlines drawn over each other. Also check that the lip paint still sits on the new lips.

Change nothing live.

## Rules

- Do the building yourself. Launch no sub-agents except for quick research that would block you.
- Rust for anything heavy (decision 53). For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`, and is fine for reading files once.
- Two other conversations work in the same crate. `jaw-skull-build` owns `under_jaw.rs`, `jaw_edge.rs`, `throat_corner.rs`, `chin_neck.rs`, `skull.rs`, `bald.rs` and `jaw.rs`. `nose-profile` owns `nose.rs` and `nose_profile.rs`. The eye conversation owns the eye files. Do not edit any of them. Add your own files. If you need a change in shared kit files (`feature.rs`, `maker.rs`, `bench.rs`, `lib.rs`, `sliders.rs`), keep it small and additive.
- Do not change the default behaviour of a shared script or of the live page. Do not commit, and do not change the main kjk checkout.
- David is away. Do not stop to ask him. Stop for the HQ only when something goes against this brief, or when a choice would change what gets built outside the lips, such as new topology.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ one report at the end. Start with the verdict, and name what is broken first. Keep apart what was measured, what was seen and what was not checked. Include:
- the measure and its checks;
- the sliders with their ranges and ends;
- each loop and what it changed;
- the outline gaps per candidate, before and after, front and side;
- the viewer 2 links, and the overlap sheets as full paths;
- how to undo everything.

Write the note for the next conversation at `$TMPDIR/kjk-lips-shape-2026-11-21.md`, with its traps. Update the roadmap's "Now and next" when you start and when you finish.
