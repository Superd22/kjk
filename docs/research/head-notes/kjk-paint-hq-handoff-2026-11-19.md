# Handoff for the paint lead session

Written on 2026-11-19, late evening, by the session that leads shape and sliders (session `323ce000-e135-4d36-a800-8a864c2e749a`).

## Your role

You lead the paint work on kjk's head customization, which is part C of the roadmap. You plan with David, brief subagents, check their results with your own eyes, and keep the roadmap current. Another session leads shape and sliders in the same way. David talks to both.

Paint means everything that ends up in the head's texture: skin tone, shading, sheen, pores, wrinkles, lips, brows, eye makeup, facial hair, decals and painted hair. Tonight it also takes the placeholder lash strip, for the reason given below.

## Read these first, in this order

1. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`. It is the one plan for every session and agent. Read all of it. Part C is the paint plan. "Done today" holds the result and known faults of every paint step. Decisions 14 to 32 are the paint decisions. Decisions 46 to 52 were taken tonight and change how work is judged.
2. `/Users/david/perso/kjk/CLAUDE.md`. Its writing rules apply to everything a person reads, including your replies, briefs' reports and roadmap edits.
3. The memory notes in `/Users/david/.claude/projects/-Users-david-perso-kjk/memory/`. Start with `MEMORY.md`. The ones that matter most for paint are `measure-features-not-millimetres.md`, `see-every-step.md`, `show-dont-claim.md`, `everything-through-sliders.md`, `rust-for-shipped-head-code.md`, `research-subagents-opus.md`, `lash-strip-and-blink.md` and `cargo-path.md`.
4. The paint research in `/Users/david/perso/kjk/docs/research/`: `2026-11-19-procedural-face-paint.md`, `2026-11-19-procedural-skin-detail.md`, `2026-11-18-painted-facial-hair.md` and `2026-11-19-eyelids-and-lashes.md`.
5. Three audits written tonight, in `/private/tmp/claude-501/-Users-david-perso-kjk/323ce000-e135-4d36-a800-8a864c2e749a/scratchpad/`:
   - `report-eyes.md`. Read sections 2, 3b, 5 and 8. It explains why the eyes are now mostly a paint job.
   - `report-inventory.md`. Sections 1 and 4 map every pipeline step and every file, with their locations.
   - `report-sliders.md`. Section 1 lists every paint number per layer, and risk 4 covers paint.

## What the other lead session is doing

- **The direction changed tonight.** The project had fitted each KOTOR head with an optimiser and scored average millimetres. Nobody had ever moved a slider. The shape session now builds sliders one feature at a time, starting with the nose. Decisions 46 to 52 record this.
- **One Opus agent is running on the nose.** It writes a new Rust crate, `head_sliders`, next to the other head crates, and its own page next to viewer 2. It puts scripts in `~/.claude/tools/likeness-fit/sliders/` and data in `~/.claude/tools/likeness-fit-data/sliders/`. It was told not to edit `head_paint`, `viewer2.js`, viewer 2's `index.html` or the shared viewer data.
- **The eyes come after the nose**, as sliders. No more eye shape rounds run until then.

David may still have agents alive from the first head session (`10035d93-2b63-4fbe-8d90-9e81fd5ef380`). Ask him which ones before you launch anything that writes to the viewer's data.

## How the two sessions stay out of each other's way

- **You own** the crate `head_paint`, the paint textures and `ours.json` in the viewer's data, the paint switches in `viewer2.js`, and the paint entries of the roadmap.
- **The shape session owns** `head_sliders`, `head_shape`, `head_eyes`, `head_rig`, the shapes and rigs in the viewer's data, and the lid edge's shape.
- **Shared files.** Both sessions edit the roadmap and may edit the workspace `Cargo.toml` in the worktree. Re-read either file right before editing it, and change only your own lines.
- **Builds.** Both build in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, so a build sometimes waits for the other's lock.
- **The web server** on 127.0.0.1:8765 is shared. Never stop or restart it.
- **Nothing is committed.** David has not settled where the code will live, so do not commit unless he asks.

## Where paint stands

The roadmap's "Done today" entries have the numbers. This is only the map.

| Layer | State | Roadmap entry to read |
|---|---|---|
| KOTOR's paint baked into our layout (the targets) | Done for 57 heads | "The per-pixel bake (C)" |
| Texture layout v2 | Done | "Next, in this order", item 3 |
| Base tone from pigments | Done, with five controls | "The base tone from pigments" and "Base tone, second round" |
| Shading and sheen | Done, with two known flaws | "Shading, sheen and paint layer switches (C2)" |
| Pores and grain | Works | "Fine skin detail, first version" |
| Wrinkles and folds | Built, but it does not match KOTOR yet | Same entry |
| Painted hair | Done for 5 heads, inside the hair work | "Hair, second round and D2" |
| Regional tints, such as beard shadow | Analysed, not built | "Next, in this order", item 1 |
| Eye outline: lash line, liner and crease shadow | Designed, not built | See below |
| Lash strip | Not built | Decision 6 |
| Lips | Not built | "Next, in this order", item 4 |
| Brows, facial hair, decals | Not built | Part C, "The layers" |
| Iris and eye white | Each head copies KOTOR's own eye texture | `report-eyes.md`, section 7 |

Viewer 2 is at http://127.0.0.1:8765/viewer2/index.html. Its Skin group switches between our paint, KOTOR's paint carried over, and no paint, with one switch per layer.

## What tonight's decisions mean for paint

1. **The eyes are waiting on you.** An audit of six eye shape rounds found that a KOTOR eye is mostly paint. KOTOR's skin just above the eye is 24 lightness points darker than the cheek, and ours is 7 to 9. No liner, lash line or lash strip exists in any crate. The only picture in which our eyes read as Carth, Bastila and Kreia had KOTOR's own paint on our head. David agreed to stop the shape rounds (decision 46). The eye outline paint is therefore the most valuable paint work right now.
2. **Its design already exists.** The Eyes v2 agent wrote it at 16:03 UTC. It is in `handbacks.txt` in the audits' folder above, at lines 2028 to 2033. It has four parts, each computed from the distance to our lid edge: an upper lash line, a thinner lower lash line, a liner wing past the outer corner, and a crease shadow 4 to 8 mm above the edge.
3. **A placeholder lash strip goes with it.** The strip is a thin dark band of mesh along the upper lid edge. Without it, David cannot judge eye size, because the strip covers part of the opening. A plain version is enough for now. Build both on the default smoothed head, not on Eyes v2 to v6.
4. **Compute from the head, never from a KOTOR head.** The sliders will move the lid edge later. The eye outline and the strip must follow the lid edge of any head. Nothing may be stored per KOTOR head (decision 32 and the note `lash-strip-and-blink.md`).
5. **The bar for eyes.** They are judged mostly at conversation distance. They must also hold up in the editor, but they need not be perfect at 5 cm. David finds the default eyes the most natural, and also too small. The size is the shape session's job.
6. **Measure features, not averages (decision 49).** David said earlier agents "gave me mm differences but we never proved we were measuring the right things". Paint has the same habit: a median colour difference per pixel, or a share of a band explained. The wrinkle fit showed the cost, because a crease 1 mm off counted against itself and every depth shrank to zero. Score paint per feature instead. For example: is the darkening above the eye there, and how dark is it? Does the lip border sit in the right place, and is it sharp? Does the liner wing have the right length and angle? Before using a measure, prove it with the four checks of decision 49.
7. **A head is only its slider values (decision 47).** That covers paint too. The same paint value must give the same result on any head, and a starting head's paint is a saved list of values.
8. **Who proves what (decision 48).** Agents prove that the paint controls can make each KOTOR face. David judges whether using them is fun. He is not an expert in anatomy or painting. Show him a result and he says what looks wrong.

## An order I would suggest

David decides. This is the order I would propose to him.

1. The eye outline paint and the placeholder lash strip, on the default head, fitted to the 57 KOTOR bakes and judged in viewer 2 at conversation distance.
2. The lips (C3). Every head needs them, and the mouth looks unfinished without lip colour.
3. Wrinkles, second round. The skin detail agent's own plan is to detect the creases in each KOTOR texture, fit each curve's position and shape per head, and only then fit the depth. Judge on Kreia first.
4. Regional tints, starting with beard shadow, which is 15% of how KOTOR's tints vary.
5. Brows, then a procedural iris and eye white, so that eye colour becomes a slider and no KOTOR pixels are copied.

## How paint is built and shown

`report-inventory.md`, section 1, steps 8 to 11 and 15, has the commands' files and data. The points that are easy to miss are these.

- **cargo is not on the PATH.** See the memory note `cargo-path.md`.
- **Paint is pre-baked.** The viewer never paints live. `paint_heads` bakes every combination of the five layer switches, so each head has 32 textures and the set takes about 607 MB. Each new switch doubles that. Painting live in the browser would remove the problem, and the shape session is setting up the Rust build for the browser now, so ask it before choosing.
- **The browser build is being set up by the nose agent.** It compiles `head_sliders` and the smoothing to WebAssembly for its own page. It was asked to make the setup reusable: one build script that takes a crate name, a small JavaScript loader another page can import, and a README with the commands and timings. The README's path will be in the nose agent's final report, so ask the shape session for it. `head_paint` needs changes of its own before it runs in the browser. It starts threads in `src/closest.rs`, and the local server does not send the headers that shared-memory threads need. One texture at 2,048 pixels takes about 0.7 s natively on one thread, so live painting probably means a small texture while a slider moves and the full one on release.
- **Much of the data lives in a temporary folder.** The viewer's data, the paint recipes and `zones.json` are under `/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/`. A restart of the Mac empties it. David knows and has chosen not to move it yet. Put new scripts and data under `~/.claude/tools/likeness-fit/` and `~/.claude/tools/likeness-fit-data/` instead.
- **`zones.json` is read from a backup folder**, `viewer-data-old-layout`. Do not delete that folder.
- **The bake is less reliable near the eyes.** Reading KOTOR's lash bar on the bake gave edges 0.3 to 1.7 mm off. For targets close to the lid edge, read KOTOR's own head with its own texture.
- **The shading and sheen were computed on the default smoothed head only.** Other shapes in the viewer wear paint made for the default.
- **Known paint flaws.** `pfha02`'s forehead and nose are too white. `mod_kenobi` has a light stripe on the nose ridge. The upper lip lines take Canderous's painted stubble for creases. The fine creases read as scratches on Male B1.

## How David works

The memory notes hold this in full. In short:

- He needs to see every step in the viewer, on our head next to the KOTOR original. Numbers alone do not count.
- Never say something works when only numbers or a subagent's report say so. Say what was measured and what was seen.
- Build subagents run on Opus. Research and small viewer changes have run on Sonnet. Subagents never run on the session's own model.
- He asks for research with `/mattpocock-skills:research`, and wants primary sources and specific techniques.
- Before launching, he often wants to plan together first. When he says "plan, don't launch", do not launch.

## Open questions for David

1. Should paint move to live painting in the browser now, or stay pre-baked for a while?
2. Does he agree with the order above, starting with the eye outline and the placeholder lash strip?

One question is already answered. David said on 2026-11-19 to avoid Python for everything that is heavy to compute. Fits, optimisers, measures over many heads, bakes and solves are written in Rust, whether they ship or not. Python stays for light work, such as reading files and gluing steps together. This is decision 53 in the roadmap. Put it in every brief.

## Suggested skills

- `mattpocock-skills:research`, when David asks for a state of the art question to be researched in the background.
- `mattpocock-skills:writing-for-agents`, when writing subagent briefs or editing the roadmap's "For agents" rules.
- `claude-in-chrome`, to open viewer 2 in a new tab and take screenshots, so you look at results yourself before reporting them.
- `mattpocock-skills:grilling`, if David wants a paint plan stress-tested before launching it.
