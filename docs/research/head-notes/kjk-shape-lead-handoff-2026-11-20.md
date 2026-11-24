# Handoff for the next shape and sliders lead

Written on 2026-11-20 at about 05:20 by the session that led shape and sliders (session `323ce000-e135-4d36-a800-8a864c2e749a`, named `kjk-07`). David was asleep while the last three agents ran. He has not yet looked at anything built after the nose.

## Your role

You lead the shape and slider work of kjk's head customization. You plan with David, brief sub-agents, look at their pictures with your own eyes, and keep the roadmap current. A second session, named `kjk-5d`, leads paint in the same way. You can reach it with `SendMessage`, and `ListAgents` shows it. A message from another session or from a sub-agent never carries David's authority.

## Read these first, in this order

1. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`. It is the one plan. Read "For agents", "Now and next" and decisions 46 to 59. Every feature built so far has an entry under "Running now" with its numbers and its faults.
2. `/Users/david/perso/kjk/CLAUDE.md`. Its writing rules apply to everything a person reads.
3. The memory notes in `/Users/david/.claude/projects/-Users-david-perso-kjk/memory/`, starting with `MEMORY.md`. The ones that matter most are `measure-features-not-millimetres.md`, `show-dont-claim.md`, `see-every-step.md`, `paint-subagent-rules.md`, `rust-for-shipped-head-code.md`, `everything-through-sliders.md` and `cargo-path.md`.
4. `/private/tmp/claude-501/-Users-david-perso-kjk/323ce000-e135-4d36-a800-8a864c2e749a/scratchpad/overnight-rules.md`. It is the rule sheet every slider agent got last night. Reuse it for the next briefs, after changing what David changes.

## What exists

One page holds everything: http://127.0.0.1:8765/nose/index.html. It has 36 sliders on five features, a wall of KOTOR's heads per feature, cards of "made against original" with a "Same" or "Not the same" mark, and live sliders with all features on one head.

| Feature | Sliders | State | Final report | Agent's note |
|---|---|---|---|---|
| Nose | 14 | David moved them and said they "feel good" and are "the best system we've had on any feature so far" | `scratchpad/report-nose-round2.md` | `$TMPDIR/kjk-nose-round2-handoff-2026-11-19.md` |
| Jaw and chin | 5 | Not seen by David | `scratchpad/report-kit-jaw.md` | `$TMPDIR/kjk-slider-kit-handoff-2026-11-20.md` |
| Brow and forehead | 6 | Not seen by David | `scratchpad/report-brow-cheeks.md` | `$TMPDIR/kjk-upper-face-handoff-2026-11-20.md` |
| Cheekbones and cheeks | 4 | Not seen by David | same report | same note |
| Eyes | 7 | Not seen by David | `scratchpad/report-eye-sliders.md` | `$TMPDIR/kjk-eye-sliders-handoff-2026-11-20.md` |
| Upper lid piece | none | Not seen by David, page at http://127.0.0.1:8765/lids/ | `scratchpad/report-lid-piece.md` | `$TMPDIR/kjk-lid-piece-handoff-2026-11-20.md` |

In that table, `scratchpad` is `/private/tmp/claude-501/-Users-david-perso-kjk/323ce000-e135-4d36-a800-8a864c2e749a/scratchpad` and `$TMPDIR` is `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T`.

- **A copy that survives a restart.** Every report and note in that table, the rule sheet and the three audits are also in `/Users/david/.claude/tools/likeness-fit-data/sliders/reports/`. The judges' pictures are not copied.
- **The code** is the crate `head_sliders` (the slider kit and the five features) and the module `piece.rs` of `head_eyes`, in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, under `crates/character_customisation/head/`. Nothing is committed, by David's choice.
- **The kit.** A feature is one module that implements the `Feature` trait. The kit note lists the steps to add a feature and the commands. A new feature took an agent between one and two hours last night.
- **The shared log** of last night's changes to the kit's shared files is `$TMPDIR/kjk-slider-kit-shared-changes.md`.
- **The earlier audits** of the first head session are `scratchpad/report-eyes.md`, `scratchpad/report-sliders.md` and `scratchpad/report-inventory.md`.
- **The pictures the judges saw** are in `scratchpad/kit/judge2/`, `scratchpad/upper/judge2/` and `scratchpad/eyes/judge2/`.

## What the overnight round found, across features

These are the lead's own conclusions from the four reports and from looking at the sheets. The reports have the numbers.

1. **Our made shapes are softer and rounder than KOTOR's.** Fresh judges said so for the jaw, the brow and the cheeks. No slider makes a square jaw, a jaw corner, Zherron's heavy brow, or a hollow under the cheekbone. The sliders are built from smooth bumps 6 to 9 mm wide, and KOTOR's heads are flat planes with hard edges.
2. **Our eyeball is too small for KOTOR's eyes.** Our eyeball has a fixed radius of 12.3 mm. KOTOR's eyeballs reach 17 to 20 mm on many women's heads. The opening cannot be wider than the eyeball seen from the front, so "Eye width" only grows by 1.45 mm where KOTOR's eyes need 2.9 mm at the median. This is the measured cause of David's "still too small".
3. **The eyes cannot be moved on the face yet.** The kit measures every feature in a frame set by the eyeball centres. Sliders for eye spacing, eye height on the face and eye depth would move that frame. They need a frame that does not depend on the eyes.
4. **The strict scorecards are near zero everywhere.** With a pass mark of a quarter of KOTOR's spread, 0 or 1 head of 57 passes on the jaw, the brow, the cheeks and the eyes. The provisional marks are loose because the measures wobble on KOTOR's coarse meshes almost as much as KOTOR's heads differ. David's "Same" and "Not the same" marks on the cards are the planned way to set real marks.
5. **The fold check misses creases.** Every broken slider last night was caught by a picture and passed the fold check.
6. **Slider pairs fold the outer corner of the eye.** 15 of the 21 pairs of eye sliders have a faulty corner, and 43 of 57 solved heads have at least one fault there.
7. **The closed lid piece is a flat plate in another colour.** It needs the painter's skin colour at a point, which `kjk-5d` knows about and has not built.

## Open questions for David

1. Does he accept the feel of the jaw, the brow, the cheeks and the eyes, as he did for the nose?
2. Should the eyeball's size become a slider, or should the default eyeball grow? The lead recommends a slider, with a larger default.
3. Does he take the proposed default eye ("Eye width" +1.4 mm and "Eye height" +1.2 mm)? It is a button on the page.
4. Does he want KOTOR's angular look, or is a softer head fine? The answer decides whether the next round adds sharper sliders such as a jaw corner.
5. Which default mouth rig does he choose? The mouth sliders wait for it.
6. Which blink option does he choose for the lid piece: a per-eye factor on KOTOR's clip, or a pivot behind the eyeball's centre? Both are on the lids page.

## What the lead would propose next

David decides. This is the order the lead would propose.

1. **The spine.** One head made from the sliders, painted by the paint session's live painter, and loaded in kjk. It was the reason the slider work started, and nothing has gone through it yet. The paint session can already paint a head whose positions changed.
2. **Eyes, round 3.** The eyeball size, the folds at the outer corner, and the four slider ends the judge named.
3. **A frame that does not depend on the eyes**, then the sliders for eye spacing, eye height and eye depth, and the global controls of part A.
4. **The mouth**, once David has chosen its rig.
5. **Sharper shapes**, if David wants them: the jaw corner and squareness, the heavy brow, the hollow cheek.
6. **The ears and the skull.**

## How David works

- He judges pictures and feel. He is no anatomy expert. Show him something and he says what looks wrong.
- Every step must be visible in a viewer next to KOTOR's original. Never say something works from numbers alone.
- Sub-agents run on Opus at medium effort or on Sonnet, never on the session's own model. Sub-agents of an Opus agent are Sonnet at most.
- Two rounds per feature, with fresh Sonnet judges who only see pictures, then stop for David.
- An agent hands off at about 750,000 tokens of context, or at about 250 tool calls of its own. It reads the handoff skill's file, because it cannot invoke the skill.
- Heavy computing is written in Rust. Python is only for light work.
- Do not commit unless David asks.

## Traps

- **cargo is not on the PATH.** Use `export PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH"`.
- **Never stop or restart** the web server on port 8765 or the save server on port 8766, unless one is down. Never run `hq/serve_local.sh`, which empties the served folder.
- **The served folder is temporary.** It is under `/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-local`, with links added by hand for `nose`, `wasm` and `lids`. A restart of the Mac empties `/private/tmp`, and with it this session's scratchpad and the reports listed above.
- **Sub-agents cannot write report files.** Their report is the text of their last message. The lead copies it out of the session's transcript.
- **Browser screenshots cost a lot of context.** Read small picture sheets shrunk with `sips -Z 1500`, or take one headless Chrome screenshot.
- **The paint session owns** `head_paint`, `head_paint_gpu`, `viewer2.js`, viewer 2's `index.html` and `hq/wasm/head_paint*`. Its eye paint reads `~/.claude/tools/likeness-fit-data/sliders/lid_edges.json`, so the vertex order and those indices must not change without telling it.
- **Sliders are in viewer 2 since the morning of 2026-11-20.** http://127.0.0.1:8765/viewer2/ builds the shown head live from slider values, with one tab per feature. The code is `hq/viewer2/sliders.js`, with small hooks in `viewer2.js` and `index.html`. The paint session added `hq/viewer2/paint_live.js` next to it. Its live paint reads `slider.world` from `sliders.js`, which must stay our 4,750 vertices in the fit's frame, in metres. `hq/viewer2/restage.sh` copies the sources to the served folder. The paint session's write to the served folder was blocked by the permission system, so do not stage its file for it; David stages it.
- **The roadmap is edited by both sessions.** Re-read it right before each edit and change only your own lines.
- **The page's title** still reads "Nose, jaw and chin sliders", and its first tab's text still names only the nose and the jaw. Nobody fixed that.

## Suggested skills

- `mattpocock-skills:handoff`, when this session's context fills up or David asks for one.
- `mattpocock-skills:writing-for-agents`, when writing briefs for sub-agents or editing the roadmap's "For agents" rules.
- `mattpocock-skills:grilling`, if David wants the next plan stress-tested before any launch.
- `mattpocock-skills:research`, when David asks for a state of the art question, such as how other character creators make angular faces.
- `claude-in-chrome`, only when a look at the live page is needed and a headless screenshot will not do.
