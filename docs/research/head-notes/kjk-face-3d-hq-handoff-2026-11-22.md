# Handoff: Face 3D Features HQ

This handoff was written on the morning of 2026-11-22. It is for a fresh conversation that takes over as the **Face 3D Features HQ** of the kjk project (repo `/Users/david/perso/kjk`).

The HQ leads the face's shape work outside the eyes for David (the user): the nose, the jaw, the throat, the skull, the lips, the brows and the cheeks. Another conversation, Eyes HQ, owns the eyes. This HQ does not build. It writes briefs, starts full conversations with the Conversation tool, reads their reports, looks at their pictures, decides within David's approvals, and summarises for David.

## Read these first

1. **The roadmap,** `docs/research/2026-11-19-head-roadmap.md`. Read "For agents" and decisions 44 and 62 to 73. Then read the block at the top of "Now and next", **"Face 3D features, state on the morning of 2026-11-22 (Face 3D Features HQ)"**. That block is the current state, the open faults, what waits for David, and the steps before shipping. It is not repeated here.
2. **The method,** `docs/research/2026-11-21-head-measures-and-sliders-guide.md`. Read all of it, including section 7, which holds tonight's lessons. Every brief points to it.
3. **The writing rules,** `CLAUDE.md` at the repo root. They apply to every reply and every brief. Write plain full sentences, one fact per sentence, and give real numbers.
4. **The memory index** at `~/.claude/projects/-Users-david-perso-kjk/memory/MEMORY.md`, which loads automatically. Two rules matter most for this HQ:
   - `fold-into-viewer-default.md`: clearly better results go straight into viewer 2's default. No URL-option previews.
   - `diagnosis-measures-not-judges.md`: a diagnosis measures against a reference and runs no judges.

## The notes of each step

Each step's note holds its code, numbers, undo steps and traps. Read a note before you brief the next step on its feature.

They are tracked in the repo at `docs/research/head-notes/`, which is where you should read them. Each one is also still in `$TMPDIR` (`/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/`), where it was written, but that folder is emptied when the machine restarts. It had only survived because the machine has been up for 51 days. Write a new note into `$TMPDIR` as the briefs ask, then copy it into `docs/research/head-notes/` and commit it when the step is finished.

| Feature | Notes, oldest first | State |
|---|---|---|
| Jaw, throat and skull | `kjk-under-jaw-measure-2026-11-21.md`, `kjk-under-jaw-prototype-2026-11-21.md`, `kjk-throat-corner-2026-11-21.md`, `kjk-bald-references-2026-11-21.md`, `kjk-skull-refit-2026-11-21.md`, `kjk-chin-neck-profile-2026-11-21.md`, `kjk-jaw-skull-build-2026-11-21.md`, `kjk-jaw-skull-fold-2026-11-22.md` | In viewer 2's default |
| Nose | `kjk-nose-profile-2026-11-21.md`, `kjk-nose-nostrils-2026-11-22.md` (with a Round 2 section) | In viewer 2's default |
| Lips | `kjk-lips-shape-2026-11-22.md`, `kjk-lips-round2-2026-11-22.md` | In viewer 2's default |
| Slider kit | `kjk-slider-kit-handoff-2026-11-20.md`, and the shared-changes log `kjk-slider-kit-shared-changes.md` | |

The 13 briefs this HQ wrote are tracked in the repo at `docs/research/head-briefs/`. They were written into a conversation's own scratch folder under `/private/tmp`, which a restart also empties, and the originals are still there in `/private/tmp/claude-501/-Users-david-perso-kjk/52003e36-b8ca-4208-a733-a6d51b417e4e/scratchpad/jaw/`. Copy their shape for new briefs. That shape is:
- what David said, quoted;
- what to read;
- what is known;
- numbered steps;
- the fold rule;
- the file ownership between conversations;
- what to report, plus a note in `$TMPDIR`.

## Where the code and the viewer are

- **Slider kit.** Crate `head_sliders`, in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, at `crates/character_customisation/head/head_sliders/src/`. The worktree's folder keeps the name `head-paint`, but since 2026-11-22 it is on the branch `character-customisation`. The feature files are:
  - `nose.rs`, `nose_profile.rs` and `nose_base.rs`;
  - `jaw.rs`, `jaw_skull.rs`, `jaw_edge.rs`, `under_jaw.rs`, `chin_neck.rs`, `throat_corner.rs`, `skull.rs` and `bald.rs`;
  - `lips.rs`;
  - `brow.rs` and `cheeks.rs`;
  - the eye files, which Eyes HQ owns.

  Run with `sliders <feature> <command>`.
- **Viewer 2.** http://127.0.0.1:8765/viewer2/index.html?head=pmhc04&hair=none. Its live sources are in the HQ site's `hq/viewer2/`, which is what the local server serves, and `sliders.js` lists the features. Run `restage.sh` after changes. The repo also tracks a copy at `crates/character_customisation/tools/viewer2/`; edit the live copy and copy a finished change into the repo.
- **Tools.** Python tools are in `~/.claude/tools/likeness-fit/`, run as `.venv/bin/python -I`. Their data is in `~/.claude/tools/likeness-fit-data/`. The old jaw and skull prototypes are backed up in `likeness-fit-data/backups/jaw-skull-prototypes-2026-11-22/`.
- **Trap.** Whoever rebuilds `hq/wasm/curve/head_sliders` must build it from the head-paint worktree, or the throat disappears.
- **Cargo.** For cargo, put `$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` first on the PATH.

## How David wants the work run

These all come from this conversation:
- **Pictures and measured overlaps.** David wants measurement and overlap: KOTOR's outline drawn over ours, from the side, the front and below. He reacts to profile views in viewer 2.
- **His approvals.** "Yes go" approves the next step that was proposed. Once he has said a result looks better, the HQ may fold it. While he is away, he allows a couple of extra loops and folding anything that is clearly better.
- **Things that need him.** Ask him before anything that changes the shared base mesh (new topology), and before anything that moves the neck's rim (never, decision 68).
- **Pronouns.** Refer to David by name in briefs. His pronouns have not been stated, so use they/them, or his name.
- **Conversation model.** Opus for build conversations. Sub-agents only as judges, and judges only when a prototype needs them.
- **Effort.** Two to four loops per step at most. Do not chase perfection.
- **Before relaying a report, look at its main picture yourself.** Shrink it with `sips -Z 1500 <png> --out <jpg> -s format jpeg`, then Read it. Say what you saw, apart from what was measured.

## Not done yet

- **The Likeness HQ status page is three days stale.** It is https://claude.ai/artifact/FJcX4XYxLytCHsBue2k6pw, and its sources and rebuild steps are in `~/.claude/tools/likeness-fit/hq/` (see the memory `likeness-hq.md`). Its memory says to update it in place after face work.

  It still carries the headline "status on 19 November 2026", and its footer still names the data files made on that day. Its zone grid marks B3, the Fix step, as "Not yet" for the forehead, the brows, the nose, the cheeks, the mouth, the chin and the jaw, when every one of those is in viewer 2's default. Its grid also has no row for the throat or for the skull, which are features now. The eyes row says B3 "Partly" and B4 "Paused".

  Read it, add the throat and the skull, update the nose, the jaw and chin, the skull, the lips, the brows and the cheeks from the roadmap's state block, change the headline date and the footer, and republish to the same URL. Its three built pages embed renders of KOTOR's heads and photographs of real people, so they stay out of the repo: only their templates and their builder are tracked.
- **Everything is committed and pushed** as of 2026-11-22. The branch is `character-customisation` and the draft pull request https://github.com/Superd22/kjk/pull/23 against David's own fork tracks it. It is not meant to be merged. Commit each finished step onto that branch. The commit gate does not pass yet, on purpose: the pull request body lists the 304 publish-check findings, the 3,829 fmt changes and the 278 clippy warnings that are still open, and David has deferred all of them.

## Likely next steps, once David has looked at viewer 2

1. Repair the far ends of the sliders that fold at pair corners: the throat pairs, Wing height with Nose length or Tip projection, and Nose length's bottom end.
2. Widen Forehead slope's range for pmhc04, or add a forehead control, so pmhc04 reaches 30 degrees.
3. Add a crease-top slider for pmhc04's nostril wing.
4. Close-ups of the heads that changed but were not looked at.
5. The steps before shipping into kjk: the calibration, the hair caps and the paint hairline.

## Suggested skills

- `artifact-design`, and the Artifact tool's `read` and publish actions, to update the Likeness HQ status page.
- The handoff skill cannot be called by an agent. `mattpocock-skills:handoff` sets `disable-model-invocation: true`, so David has to type `/mattpocock-skills:handoff` themselves. Ask them to, or write the next handoff by hand in the shape of this one.
- `mattpocock-skills:grilling`, if David wants a plan for the slider-end repairs stress-tested before briefing it.
