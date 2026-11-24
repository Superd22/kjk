# Brief: the jaw-to-neck and skull build, solved on all 57 heads

You are a full conversation started by the head HQ on 2026-11-21. David (the user) approved this step after seeing the chin-to-neck prototype in viewer 2 ("yes it looks better, let's go").

Your job is to make the throat sliders and the skull clean, then make the skull into sliders, then solve every KOTOR head once. You stop before anything goes live.

## Read first

1. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`, whole. This is checklist steps 6 to 9.
2. The notes of the conversations before you, newest first. They hold the code, the numbers, the undo steps and the traps:
   - `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-chin-neck-profile-2026-11-21.md`
   - `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-skull-refit-2026-11-21.md`
   - `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-bald-references-2026-11-21.md`
   - `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-throat-corner-2026-11-21.md`
   - `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-under-jaw-measure-2026-11-21.md`

   The code is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, crate `head_sliders`. You own `under_jaw.rs`, `jaw_edge.rs`, `throat_corner.rs`, `skull.rs`, `bald.rs` and the chin-to-neck code.
3. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: "For agents", "Now and next" and "Decisions". This serves B3 and B5 for the jaw, the chin and the skull, and A2.
4. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.

## Where things stand

- **The chin-to-neck measure is proven.** It is one outline from the chin's front, along the underside and down the neck, read in the middle, 12 mm to each side and in the side view. Its wobble is 0.6 to 1.1 mm, against 2.9 to 3.2 mm between heads.
- **The throat sliders** are Throat depth (45 to 76 mm), Throat corner height (−12 to 15 mm), Jaw corner rise (0 to 28 mm) and Throat roundness (2 to 25 mm). They work as one smooth field on top of the skull-refit head. Solved with the 5 jaw sliders on 10 heads, they cut the outline gap to KOTOR to 0.7 to 2.2 mm (pmhc04: 6.1 to 1.2, Bastila: 4.4 to 1.1).
- **The skull refit** moves the skull's vertices of 21 haired heads toward bald references. It is a free movement that no slider makes yet. pmhc04's skull is now 123 mm wide (reference 125 mm, before 155 mm).

## What is still broken, which you fix

1. **The corner line is jagged from below,** on heads with a sharp corner. The corner runs diagonally across the mesh's rows under the jaw.
   - First try to slide existing vertices into a row along the corner, with no new vertices. The slide must be computed from the slider values, the same on any head.
   - **Do not add vertices or change the mesh's topology without asking.** The base mesh is shared by every feature, the paint layout, the rig, the calibration and the eye conversation. If a new edge loop is the only clean answer, stop and send the HQ the case for it, with pictures of the sliding attempt.
2. **A small bump** where the corner meets the side of the neck, seen three-quarter on pmhc04.
3. **Two sliders hit their range ends.** Throat depth is at or near its top on 6 heads, and Throat roundness at its bottom on 5. Repair the far ends before you extend or cap them, following the guide.
4. **Jaesa and the Handmaiden.** The held jaw measure moved more on them than on other heads. Check their faces close up, and fix the cause.
5. **The ledge.** How far the jaw stands out over the neck, seen from the front, is not in the measure. KOTOR's is 13 to 19 mm on heads that have one, and ours is about 10. Add it to the measure if it passes check 1. Add a slider if the existing ones cannot reach it.
6. **The skull.** There are two parts:
   - Fix the shoulder where the held forehead meets the narrower temples (seen on pmhc04 and pfha06).
   - Fix pfha01's reference, which kept its fringe in the face box.
7. **Skull sliders.** Build named skull sliders: width, length, height and where the top of the head lies, as the skull-refit note proposes. Range them from the 36 KOTOR heads whose skull shows. The goal is that the solve reaches the refit skulls through sliders alone (decision 47). The forehead and the face must not move. The neck's rim stays at 0.000 mm (decision 44). The ears are out of scope.

## Then solve all 57 heads

Solve every KOTOR head once, with the throat sliders, the jaw sliders and the skull sliders. The targets are:
- the chin-to-neck outline;
- the jaw measures;
- the skull measures against KOTOR's own skull on the 36 heads that show one, or against the bald reference on the 21 haired heads.

Guard every head, as the guide says. A head takes its new values only when:
- no face zone gets worse;
- the nose's, brows' and cheeks' measures do not move beyond their wobble;
- nothing folds, at the solved values and at each slider's ends and pair corners;
- the skull stays inside KOTOR's hair.

Always look at Bastila next to pmhc04. Use close-ups in profile, three-quarter and from below, in viewer 2's "In game" light, with the hair off and with the mouth open in a talk shape.

No judges in this step. The measures decide, and David judges in viewer 2.

## Show David, then stop

- Put all 57 solved heads in viewer 2 behind a URL option, with the default page unchanged. Add live controls for the new sliders if you can, by building the construction for the browser as the other features are.
- Give links for pmhc04, Bastila and two heads that changed most.
- Draw outline overlays in profile for pmhc04 and Bastila: KOTOR, today's head and the new head.

Then stop. Change nothing live. List what going live needs:
- the presets file;
- the animation calibration again (decision 62);
- the part D hair caps placed again on the new scalps;
- a check of the paint's hairline.

## Rules

- Do the building yourself. Launch no sub-agents except for quick research that would block you.
- Rust for anything heavy (decision 53). For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`.
- Another conversation, `nose-profile`, works on the nose in the same crate (`nose.rs` and nose files). Do not edit its files, and leave the nose's presets as they are. The eye conversation owns the eye files.
- Do not change the default behaviour of a shared script or of the live page. Do not commit, and do not change the main kjk checkout.
- Start without asking for a go-ahead. Stop for David only when something goes against this brief, or when a choice would change what gets built, such as new topology.
- Run at most two or three loops of fixes on any one fault, then report it as unsolved.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ a report that starts with the verdict and names what is broken first. Keep apart what was measured, what was seen and what was not checked. Include:
- each fix above and its result;
- the skull sliders with their ranges and ends;
- the guard's result per head, and which heads kept their old values;
- the viewer 2 links, and the overlay pictures as full paths;
- what going live needs;
- how to undo everything.

Write the note for the next conversation at `$TMPDIR/kjk-jaw-skull-build-2026-11-21.md`, with its traps. Update the roadmap's "Now and next" when you start and when you finish.
