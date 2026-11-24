# Brief: bald references for Bastila and pmhc04, then whole-head measurements

You are a full conversation started by the head HQ on 2026-11-21. David (the user) looked at our pmhc04 next to KOTOR's in profile in viewer 2. He said that ours looks "like a fucking alien" and that he cannot describe why. He wants measurements, not opinions. To measure the whole head, he needs a reference for **Bastila (`p_bastilah`) and pmhc04 without hair**. Your job is to get those two references, then measure our heads against them.

David's screenshots, from viewer 2 (different cameras and zoom, so do not measure them):
- ours: `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/TemporaryItems/NSIRD_screencaptureui_wBVFnN/Screenshot 2026-11-21 at 22.14.35.png`
- KOTOR's: `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/TemporaryItems/NSIRD_screencaptureui_rod1gZ/Screenshot 2026-11-21 at 22.14.48.png`

## Fixed by David

- **No judges in this step.** David said: "we don't need subjectivity yet, we need measurements." Launch no judge sub-agents. Your own pictures are still required, as the guide says, for checking that the measures read the right thing.
- **The neck's rim never moves** (decision 44, restated on 2026-11-21 for compatibility with KOTOR's bodies).

## Read first

1. `/Users/david/perso/kjk/docs/research/2026-11-21-head-measures-and-sliders-guide.md`, whole. Follow its rules on measures (section 1) and its checklist steps 1 to 4. Skip anything about judges.
2. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`: "For agents", "Now and next" and "Decisions". This serves B1 and B2 for parts that are not scored today (the skull, the ears, the nape, the forehead under the hair), and A2 (our head stays inside KOTOR's volume).
3. `/Users/david/perso/kjk/CLAUDE.md` for how to write anything David reads.
4. Memories that apply: downloads under 1 GB need no asking. The original game decomps and the KOTOR installs are on this Mac, and the Steam installs are on the Windows PC (`ssh David@192.168.192.130`, D:\SteamLibrary).

## What the HQ knows

- The fit (`~/.claude/tools/likeness-fit/scan/trials-2026-11-18/kotor_fit.py`) scores only the face zones. The ears, the skull, the back of the head and the neck are not scored. Nothing compares the head's overall proportions. On pmhc04 the forehead is 1.62 mm off (about 0.6 mm on an average head), and the face 0.74 mm (pass mark 0.5 mm).
- KOTOR models the hair as the outside of the head, with no scalp under it. `kotor_hairless.py` removes the hair by texture colour, and leaves a hole. The skull study, `~/.claude/tools/likeness-fit-data/kotor-heads/kotor_skull.json`, counts our skull vertices with KOTOR skin under them. pmhc04 has 40, and p_bastilah 185. About 40 heads have 140 to 260. The colour test may drop scalps whose hair is only painted on a real skull shape. Check that.
- Two Bastila mod heads, `mod_improved_bastila` (196) and `mod_tcs_bastila` (184), may carry more skull than the game's head. Look at what they really have under the hair.
- The HQ's guesses from the screenshots, which are not measured: our bald skull reaches as far back as KOTOR's head with its hair on; our ear looks about 1.7 times as tall as KOTOR's and sits higher; our forehead slopes back more.

## What to do

1. **Find a real bald reference first.** In order:
   - Look in the game files of both games for the same head without hair: a variant, a cutscene model, a model used under masks or helmets, or a separate hair part that can be taken off.
   - Look in the mod heads already on this Mac.
   - Search for mods that make Bastila or pmhc04 bald, or that model a scalp under their hair, and download them.

   Say what you found and where.
2. **If no real reference exists, build one, and say how much of it is made up.** Remove the hair and keep every piece of real skin. Fill the hole with a skull shape learned from the KOTOR heads whose skull shows. Join it smoothly to the real skin, and keep it inside the hair mesh with a hair thickness you measure on heads where both can be seen. Mark which vertices are real KOTOR and which are filled in, and draw them in two colours.
3. **Show David the two references before measuring.** Show each one in profile, front, three-quarter and from behind, with KOTOR's hair on and off. If you can, add a viewer 2 view behind a URL option, with the default page unchanged. Send the HQ a short message with the pictures, then go on with step 4 while you wait. Mark every result that rests on a filled-in part.
4. **Measure the whole head.** Do this on Bastila and pmhc04 against their references, and on every KOTOR head whose skull shows. Build each measure on KOTOR's own mesh and prove it with the guide's checks 1, 3 and 4. Check 2 needs David, so leave it for him. The measures:
   - the skull's length, width and height against the height of the face;
   - the forehead's slope up to the top of the head;
   - where the face sits on the skull, front to back and up and down;
   - the ear's size and place, against the eye and the jaw;
   - the nape and the back of the head.

   Draw our head and KOTOR's reference from exactly the same camera, lined up the way the fit lines them up, with our outline over KOTOR's in profile and from the front. Use the shape that viewer 2 shows by default, and the FLAME fit if they differ, and say which is which. Put each measure's wobble next to KOTOR's spread.
5. **Rank the differences** by size against KOTOR's spread, on pmhc04, on Bastila and over all heads that can be measured. For each, say where it comes from: the FLAME fit, the held skull, the base head, or the live presets.

Then stop. Build no sliders and change nothing live. Name the next step.

## Rules

- Do the building yourself. Launch no sub-agents in this step, except for research such as a mod search if it would block you.
- Rust for anything heavy (decision 53). The slider kit is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, crate `head_sliders`, and it renders clay sheets. For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH. Python runs as `.venv/bin/python -I` from `~/.claude/tools/likeness-fit/`, and is fine for reading files once.
- Another conversation, `throat-corner`, is working in `head_sliders` on the jaw (`jaw_edge.rs` and a new throat file). Do not edit its files. Add your own.
- Do not change the default behaviour of a shared script or of the live page. Do not commit, and do not change the main kjk checkout.
- Start without asking for a go-ahead. Stop for David only when something goes against this brief.
- At about 750,000 tokens of context, write a handoff and tell the HQ.

## What to report

Send the HQ a report that starts with the verdict: in one or two plain sentences, what is measurably different between our head and KOTOR's. Keep apart what was measured, what was seen and what was not checked. Include:
- where each reference came from, and how much of it is filled in;
- the ranked differences with their numbers, wobble and KOTOR's spread;
- the pictures as full paths, with the profile overlay of pmhc04 first;
- the proposed next step.

Write the note for the next conversation at `$TMPDIR/kjk-bald-references-2026-11-21.md`, with its traps. Add a line to the roadmap's "Now and next" when you start, and update it when you finish.
