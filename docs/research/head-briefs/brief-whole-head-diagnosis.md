# Brief: why our pmhc04 looks alien, a whole-head diagnosis

You are a full conversation started by the head HQ on 2026-11-21. David (the user) looked at our pmhc04 next to KOTOR's in profile in viewer 2. He said that ours looks "like a fucking alien" and that he cannot even describe why. It is not only the line under the jaw, which another conversation (`throat-corner`) is working on. Your job is to find out, with pictures first and then measures, what makes our whole head read as alien. Then name it in plain words David can react to.

David's screenshots, from viewer 2:
- ours, bald with our paint and beard: `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/TemporaryItems/NSIRD_screencaptureui_wBVFnN/Screenshot 2026-11-21 at 22.14.35.png`
- KOTOR's, with its hair: `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/TemporaryItems/NSIRD_screencaptureui_rod1gZ/Screenshot 2026-11-21 at 22.14.48.png`

They use different cameras and zoom, so do not measure them.

## Read first

1. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`, whole. This is checklist steps 1 to 4: look, check premises, build and prove measures. You do not build sliders.
2. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: "For agents", "Now and next" and "Decisions". This serves B1 and B2 for zones that are not scored today (the skull, the ears, the nape), and A2 (our head must stay inside KOTOR's volume, hair included).
3. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.

## What the HQ knows

- **What the fit leaves out.** The fit (`~/.claude/tools/likeness-fit/scan/trials-2026-11-18/kotor_fit.py`) scores only the face zones. The ears, the back of the head, the skull and the neck are not scored. Nothing compares the head's overall proportions.
- **pmhc04's face scores.** pmhc04 is one of our worse fits. The forehead is 1.62 mm off, against about 0.6 mm on an average head. The whole face is 0.74 mm off, against a pass mark of 0.5 mm (`~/.claude/tools/likeness-fit-data/kotor-heads/kotor_fit_v7.json`).
- **KOTOR has no scalp under its hair.** KOTOR models the hair as the outside of the head. `kotor_hairless.py` already removes the hair, by the texture's colour, and leaves a hole there with no skull under it. The skull study is `kotor-heads/kotor_skull.json`. It counts how many of our skull vertices have KOTOR skin under them:
  - About 40 heads have 140 to 260, because they are bald, buzz-cut or have painted hair. Their skull can be measured.
  - pmhc04 has 40. sith_app, p_baodurh, n_mastkavarh and n_zezkaiellh have 12 to 22. mod_meetra and mod_jaesa have 0 or 1.
  - **Check this premise.** The hair test is by colour, so it may also drop scalps where the hair is only painted on a real skull shape. If it does, those heads' skulls are measurable after all.
- **The HQ's guesses from the screenshots,** which are not measured:
  - our bald skull reaches as far back as KOTOR's head with its hair on, so the skull may be too big for the face;
  - our ear looks about 1.7 times as tall as KOTOR's, sits higher (its top level with the eye, where KOTOR's sits below), and is crumpled;
  - our forehead slopes back more.

## What to do

1. **Pictures first.** Draw our head and KOTOR's from exactly the same camera, in profile, front, three-quarter and from behind. Line them up the way the fit lines them up. Use no hair on ours, and KOTOR's hair both on and off. Lay our outline over KOTOR's in profile and from the front. Do this for pmhc04, Bastila, Carth, pmhc03, pmba01, old_a_m and three heads whose skull is skin. Use the shape that viewer 2 shows by default, and the FLAME fit if they differ. Say which is which.
2. **Name what reads as alien.** Look at the pictures yourself, then ask two blind Fable judges a plain question: "What makes the left head look less human than the right one?". Give them only the pictures.
3. **Measure the whole head.** Build measures on KOTOR's own meshes and prove them with the guide's four checks:
   - the skull's length, width and height against the height of the face;
   - the forehead's slope;
   - where the face sits on the skull;
   - the ear's size and its place, against the eye and the jaw;
   - the nape.

   On heads with modelled hair, measure only whether our skull fits inside the hair. Report it as an outer limit, not a target. Learn the skull's real proportions from the heads that show their skull. Put each measure's wobble next to KOTOR's spread.
4. **Rank the faults** by how much they explain the alien look, on pmhc04 and on all heads. For each, say whether it comes from the fit (FLAME, the held skull), from the base head, or from the live presets.

Then stop. Build no sliders and change nothing live. Name the next step: which control or fix, and in which order.

## Rules

- Do the building yourself. Launch sub-agents only as judges of your pictures, on Fable, given only what to judge and the picture paths. Do browser checks, code review and research yourself.
- Rust for anything heavy (decision 53). The slider kit is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, crate `head_sliders`, and it renders clay sheets. For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`.
- Another conversation, `throat-corner`, is working in `head_sliders` on the jaw (`jaw_edge.rs` and a new throat file). Do not edit its files. Add your own.
- The neck's rim never moves (decision 44, restated by David on 2026-11-21 for compatibility with KOTOR's bodies).
- Do not change the default behaviour of a shared script or of the live page. Do not commit, and do not change the main kjk checkout.
- Start without asking for a go-ahead. Stop for David only when something goes against this brief.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ a report that starts with the verdict: in one or two plain sentences, what makes our head look alien. Keep apart what was measured, what was seen and what was not checked. Include:
- the ranked faults with their numbers;
- which heads can and cannot give a skull;
- the pictures as full paths, with the profile overlay of pmhc04 first;
- the proposed next step.

Write the note for the next conversation at `$TMPDIR/kjk-whole-head-diagnosis-2026-11-21.md`, with its traps. Add a line to the roadmap's "Now and next" when you start, and update it when you finish.
