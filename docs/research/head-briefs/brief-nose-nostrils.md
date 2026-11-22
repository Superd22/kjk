# Brief: the nostrils' height, and the nose from below and above

You are a full conversation started by the head HQ on 2026-11-22. David (the user) asked for this.

## First, fold the nose profile into viewer 2's default

The last nose conversation fixed the kink in the nose's profile. Its result is behind the URL option `?noseprofile=1`. David liked it: "the side profile looks good". He also set a new rule: "from now on, let's fold our effort directly into the viewer when they're clearly better rather than add an url encoded option, we have too much dead code."

So, before anything else:
1. Make the nose profile presets (52 heads, with Root place and Upper bridge) viewer 2's default.
2. Remove the `noseprofile` option and any preview code that only it used.
3. Check that the default page loads with no console errors, and that pmhc04 shows the new nose.
4. Write how to undo it.

Its note, with the undo steps and the traps, is `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-nose-profile-2026-11-21.md`.

## Then, the nostrils

David looked at pmhc04 (Male C4): "one thing missing is like the 'height' of the nostril, not in position but in actual size. The side profile looks good thanks to the tip being low enough, but the nostrils are not low enough."

So on pmhc04, our nostril wings are too short from top to bottom. This is about their size, not where they sit. The tip matches KOTOR's in profile, but the wings' lower edge does not come down as far as KOTOR's.

David's idea: "We probably need outlines from top and bottom as well, with the shape of the hole of the nostrils included. Not that we care about the holes themselves, but it helps distinguish between the end of the tip and the exterior."

## Read first

1. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`, whole. This is checklist steps 1 to 9.
2. The nose notes and reports:
   - `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-nose-profile-2026-11-21.md`
   - `~/.claude/tools/likeness-fit-data/sliders/reports/report-nose-round2.md`
   - `~/.claude/tools/likeness-fit-data/sliders/reports/kjk-nose-round2-handoff-2026-11-19.md`

   The code is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, crate `head_sliders`, files `nose.rs` and `nose_profile.rs`, run as `sliders nose ...`.
3. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: "For agents", "Now and next" and "Decisions". This serves B3 and B5 for the nose.
4. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.

## What is known

- The nose measure has an outline of the base seen from below. In round 2, old_w_m, pmhc01 and pmhc03 missed it by 1.27 to 1.52 mm against a mark of 1.19 mm. Whether it includes the nostril holes, and whether it reads the wings' height, is not known. There is no outline seen from above.
- The 16 nose sliders now include wing width and wing flare. No slider sets the wings' height.
- KOTOR's meshes have about 400 vertices on the nose, so the nostril hole may be only a few triangles. Read it on KOTOR's own mesh and on its subdivided copy, and check that it does not wobble more than KOTOR's noses vary.

## What to do

1. **Look first.** Draw KOTOR's 57 noses and ours from below, from above and in profile, close up. Mark the nostril holes. Say in plain words how KOTOR's wing height and nostril shape vary, and what our pmhc04 misses.
2. **Build and prove the measures:**
   - the outline from below, with the nostril holes;
   - the outline from above;
   - the wing height in profile, which is the distance from the top of the wing's crease to its lower edge.

   The holes are there to tell the tip from the wings, not to be matched for their own sake. Prove each with the guide's checks 1, 3 and 4. Check 2: the measure must read pmhc04's nostrils as too short.
3. **Test the existing 16 sliders** with the new measures on pmhc04, Bastila, pmhc03 and old_w_m. Name what is out of reach.
4. **Add or change sliders** for what is out of reach, probably a wing height slider. Follow "One name, one main effect" and "Repair the far ends before you cap them". Draw ends and pair corners in close-ups in viewer 2's "In game" light, from below and in three-quarter.
5. **Solve all 57 noses** and guard every head. Always look at Bastila next to pmhc04. The side profile must not get worse.

At most two or three loops of fixes. Do not chase perfection.

No judges. The measures decide, and David judges in viewer 2.

## Show David

If the result is clearly better (the guard passes, pmhc04's nostrils are closer, and no head is worse), make it viewer 2's default. Delete any code it replaces, and write how to undo it. If it is not clearly better, keep the default page unchanged and report it with pictures. Give enlarged pictures of pmhc04, Bastila, pmhc03 and old_w_m from below, from above and in profile, with KOTOR's outlines over ours.

Then stop. Shipping into kjk itself waits for the going-live steps.

## Rules

- Do the building yourself. Launch no sub-agents except for quick research that would block you.
- Rust for anything heavy (decision 53). For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`.
- Two other conversations work in the same crate and in viewer 2:
  - `jaw-skull-build` owns `under_jaw.rs`, `jaw_edge.rs`, `throat_corner.rs`, `chin_neck.rs`, `skull.rs`, `bald.rs` and `jaw.rs`.
  - `lips-shape` owns the lip files it adds.

  Both may also change viewer 2's default. Do not edit their files. When you change a shared viewer file such as `hq/viewer2/sliders.js`, keep the change small, and log it in the roadmap's shared-changes log. The eye conversation owns the eye files.
- Do not commit, and do not change the main kjk checkout.
- Start without asking for a go-ahead. Stop for David only when something goes against this brief, or when a choice would change what gets built outside the nose, such as new topology.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ a report that starts with the verdict and names what is broken first. Keep apart what was measured, what was seen and what was not checked. Include:
- the fold of the nose profile into the default;
- the measures and their checks;
- the sliders;
- the gaps per head before and after;
- the pictures as full paths;
- how to undo everything.

Write the note for the next conversation at `$TMPDIR/kjk-nose-nostrils-2026-11-22.md`, with its traps. Update the roadmap's "Now and next" when you start and when you finish.
