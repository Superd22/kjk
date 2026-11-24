# Brief: fold the jaw-to-neck and skull build into viewer 2's default

You are a full conversation started by the head HQ on 2026-11-22. David (the user) is asleep. Before going, he said: "happy for you to fold anything that's clearly better in the defaults, happy for you to run a couple iterations otherwise on the ongoing works." He also said: "let's fold our effort directly into the viewer when they're clearly better rather than add an url encoded option, we have too much dead code."

The last conversation built the jaw-to-neck throat sliders and 4 skull sliders, and solved all 57 heads from slider values alone. David had already called the chin-to-neck profile and the refitted skull clearly better. The HQ looked at the outlines: on pmhc04 and Bastila the new outline lies on KOTOR's. The result is still behind a URL option, because the throat construction runs only in Rust.

Your job:
1. Port the throat construction to the browser.
2. Make the build viewer 2's default.
3. Delete the dead prototypes.
4. Try once more on the faults below.

## Read first

1. The last conversation's note, with its code, guard per head, ranges, pictures, undo steps and traps: `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-jaw-skull-build-2026-11-21.md`. The code is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, crate `head_sliders`. You own `under_jaw.rs`, `jaw_edge.rs`, `throat_corner.rs`, `chin_neck.rs`, `skull.rs`, `bald.rs` and `jaw.rs`.
2. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`.
3. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: "For agents", "Now and next" and "Decisions".
4. `~/.claude/tools/likeness-fit/WASM-BUILD.md`, for how the other features are built for the browser, and the shared-changes log `kjk-slider-kit-shared-changes.md`.
5. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.

## What to do, in order

1. **Port the throat construction to the browser** the way the other features run there. The skull sliders already would. The browser and Rust must agree on the 57 solved heads, within a small number you report.
2. **Make the 57 solved heads viewer 2's default.** This means the throat, jaw and skull sliders and their presets, with live controls for the new sliders in the sliders tab. Check that:
   - the default page loads with no console errors;
   - the live rig still follows the sliders: talk shapes on pmhc04 and Bastila, in the "In game" light;
   - the other features' presets did not change: the nose from `nose-nostrils`, the brows, the cheeks and the eyes.
3. **Delete the dead prototypes:** the URL options and data of `jawproto`, `throatproto`, `skullrefit`, `chinneck` and `jawskull`, and any code that only they used. Write how to undo it. Log shared changes in the shared-changes log.
4. **One or two loops on the faults left,** then stop whatever the result:
   - **The throat sliders' pair corners** turn 2 to 21 triangles on 39 of 44 heads. No head turns one at its own values. Try once more to repair them, following "Repair the far ends before you cap them". If it does not work, leave the ranges as they are, and list the corners for David.
   - **pmhc04's forehead is too upright.** Solve the brow preset's Forehead slope (in `brow.rs`'s feature, whose code you may now change for this) against the bald reference and the forehead-to-skull profile. Guard every head, and change only Forehead slope.
   - **The ledge measure** failed check 1. Leave it unless a steady reading is quick.

## Rules

- **Clearly better** means: no head worse beyond its wobble on any measure, nothing folds at the solved values, and your close-ups are clean. If the fold breaks something you cannot fix, revert the default to what it was, and report.
- Do the building yourself. Launch no sub-agents except for quick research that would block you. No judges.
- Rust for anything heavy (decision 53). For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`.
- `lips-round2` works in the same crate (`lips.rs`) and may also change viewer 2's default. Do not edit its files. Keep shared changes small and logged. The eye conversation owns the eye files.
- Do not commit, and do not change the main kjk checkout. Shipping into kjk itself waits for the going-live steps.
- David is asleep, so do not stop to ask him. Stop for the HQ only when something goes against this brief, or when a choice would change what gets built, such as new topology.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ one report at the end. Start with the verdict and whether the default changed, and name what is broken first. Keep apart what was measured, what was seen and what was not checked. Include:
- the viewer 2 links for pmhc04 and Bastila;
- what was deleted;
- how to undo everything.

Write the note for the next conversation at `$TMPDIR/kjk-jaw-skull-fold-2026-11-22.md`, with its traps. Update the roadmap's "Now and next" when you start and when you finish.
