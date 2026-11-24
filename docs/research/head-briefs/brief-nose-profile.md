# Brief: the nose seen from the side, and the kink on pmhc04 and the Handmaiden

You are a full conversation started by the head HQ on 2026-11-21. David (the user) asked for this.

David looked at pmhc04 (Male C4) in profile in viewer 2. Our nose has a kink in its profile: the bridge's line bends between the root and the tip. KOTOR's pmhc04 has a straight bridge. The same happens on the Handmaiden (`p_handmaidenbb`). David said that he understands why pmhc03 (Male C3) and old_w_m need a kink, because their KOTOR noses have one, but on pmhc04 and the Handmaiden it reads as a bug.

His screenshots, both in viewer 2:
- ours: `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/TemporaryItems/NSIRD_screencaptureui_g3d9Yv/Screenshot 2026-11-21 at 23.29.17.png`
- KOTOR's: `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/TemporaryItems/NSIRD_screencaptureui_cW7TT3/Screenshot 2026-11-21 at 23.29.23.png`

He asked whether we have a good side-profile nose shape measure, with an overlap of the outlines. If not, he wants one built, and then a further pass on the nose from the profile.

## Read first

1. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`, whole. This is checklist steps 3 to 8.
2. The nose round 2 report: `~/.claude/tools/likeness-fit-data/sliders/reports/report-nose-round2.md`, and its handoff, `kjk-nose-round2-handoff-2026-11-19.md` in the same folder. The nose's code is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, crate `head_sliders` (`src/nose.rs`, `src/maker.rs`, `src/checks.rs`, `src/bench.rs`), run as `sliders nose ...`. The page is at http://127.0.0.1:8765/nose/index.html.
3. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: "For agents", "Now and next" and "Decisions". This serves B3 and B5 for the nose.
4. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.

## What the HQ knows

- **The nose has 14 named sliders,** and David accepted them for feel on 2026-11-20. Five of them shape the profile: root height and root dip, bridge line (scooped to humped), bridge bend (whether the bridge bulges more near the eyes or near the tip), and bridge height.
- **The measure has a side-profile outline,** compared after its position and size are taken out. Its check 1 numbers were not written in the round 2 report, and checks 3 and 4 were not rerun after round 2. So we do not know whether it is proven, or whether it can see a local kink. An outline lined up and scaled as a whole can hide a kink of 1 mm.
- **Adding root dip and bridge bend cut the profile misses from 27 heads to 2.** A pair of sliders that fit the outline's average can still leave a bump between them.
- **The eye work may be the cause.** Another conversation rebuilt the eye opening as a curve on the eyeball. Its ring moves the skin round the eye as one smooth field, out to about 22 mm. The kink sits near the eyes' level, so the eye field may move the root of the nose. Check this first, by looking at the kink with the eye construction on and off. If it is the eye's field, report it and fix only the nose side. Do not edit the eye conversation's files: anything with eye, lid, ring or curve in its name. Tell the HQ what the eye side would need.

## What to do

1. **Find where the kink comes from, on pmhc04 and the Handmaiden.** Compare these:
   - KOTOR's mesh;
   - the FLAME fit;
   - the nose stand-in with no sliders;
   - the solved nose preset;
   - viewer 2's live head with the eye construction on and off.

   Draw each profile, enlarged, with the outlines over each other. Say which step adds the kink.
2. **Build and prove a side-profile shape measure for the nose.** Read it on KOTOR's own mesh, at the middle, from the brow's front down to under the tip. Read it finely enough to see a local bend.
   - Check 1: the wobble on KOTOR's subdivided copies, against KOTOR's spread.
   - Check 2: it must read a kink on our pmhc04 and the Handmaiden, and a kink on KOTOR's pmhc03 and old_w_m. It must read no kink on KOTOR's pmhc04 and the Handmaiden.
   - Check 3: does a head match itself?
   - Check 4: the extremes sheet.

   Draw our profile over KOTOR's for all 57 heads.
3. **Test the existing sliders.** Solve all 57 noses with the existing 14 sliders and the new measure added. Then report:
   - which heads lose or keep a kink they should not have;
   - which heads now miss elsewhere;
   - what is out of reach.

   Guard every head, as the guide says. A head takes a new preset only when it passes, and always look at Bastila next to pmhc04.
4. **Fix what is out of reach,** if anything is. Change or add a slider, following "One name, one main effect" and "Repair the far ends before you cap them". Do at most two loops of fixes.
5. **Show David.** Put the result in viewer 2 behind a URL option, with the default page unchanged. Give links for pmhc04, the Handmaiden, pmhc03 and old_w_m, and enlarged profile pictures of KOTOR, today's head and the new head with the outlines drawn over each other.

No judges in this step. The measure and David decide.

Then stop. Change nothing live, and do not refit the other features.

## Rules

- Do the building yourself. Launch no sub-agents except for quick research that would block you.
- Rust for anything heavy (decision 53). For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`.
- Another conversation, `chin-neck-profile`, works in `head_sliders` on the jaw (`under_jaw.rs`, `jaw_edge.rs`, `throat_corner.rs`, `skull.rs`, `bald.rs`). Do not edit those files. The eye conversation owns the eye files.
- Do not change the default behaviour of a shared script or of the live page. Do not commit, and do not change the main kjk checkout.
- Start without asking for a go-ahead. Stop for David only when something goes against this brief, or when a choice would change what gets built.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ a report that starts with the verdict: where the kink comes from, and whether it is fixed. Name what is broken first. Keep apart what was measured, what was seen and what was not checked. Include:
- the profile measure and its four checks;
- the before and after outline gaps per head;
- the viewer 2 links, and the profile pictures as full paths;
- how to undo everything.

Write the note for the next conversation at `$TMPDIR/kjk-nose-profile-2026-11-21.md`, with its traps. Update the roadmap's "Now and next" when you start and when you finish.
