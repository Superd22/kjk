# Handoff of the paint HQ, morning of 2026-11-20

Written at 05:10 (CET) on 2026-11-20 by the paint lead session (kjk-5d), for a fresh agent that takes over the paint HQ with David.
This note is the lead's own. The skill `mattpocock-skills:handoff` refused to be called by an agent, so David can run it himself if he
wants its version.

## Your role

You are the "paint HQ" of kjk's head customization, part C of the roadmap. You plan with David, brief sub-agents, check their results
with your own eyes, and keep the roadmap current. Another lead session, kjk-07, leads shape and sliders.

## Read first

1. The roadmap, the one plan for all sessions: `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`. Read "For agents",
   part C, "Now and next" and decisions 49 to 59. Every result of the night is written there under "Running now", with its faults.
   The file is shared with kjk-07, so read a line right before you edit it and change paint lines only.
2. The memory note `~/.claude/projects/-Users-david-perso-kjk/memory/paint-subagent-rules.md`. It holds David's rules for
   sub-agents: Opus or Sonnet at medium effort and never above, pictures first, own judge and research agents, handoff at 800,000 tokens.
3. The writing rules in `/Users/david/perso/kjk/CLAUDE.md`. They apply to every text a person reads, including briefs and reports.

## Where the work is

- The code is in the worktree `/Users/david/perso/kjk/.claude/worktrees/head-paint`, in `crates/character_customisation/head/`:
  `head_paint` (settings, geometry, the frozen processor painter) and `head_paint_gpu` (the WGSL shaders and their Rust driver).
  Nothing is committed.
- The live page is http://127.0.0.1:8765/paint/. Its files are in `~/.claude/tools/likeness-fit/hq/paint/`. It loads browser module `g17`.
- Scripts and notes are in `~/.claude/tools/likeness-fit/paint/`. Data is in `~/.claude/tools/likeness-fit-data/paint/`.
- Build notes: `~/.claude/tools/likeness-fit/WASM-BUILD.md`. How to add a GPU layer: the top of `head_paint_gpu/src/layers/mod.rs`.
- cargo is not on the PATH. Prefix commands with `export PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH"`.

## What waits for David's eye

David has seen none of the night's results. Each one counts as done only when he has looked at the pictures and dragged the sliders.

| Work | Pictures | The lead's verdict |
|---|---|---|
| Pores and grain | http://127.0.0.1:8765/paint/pores/ | Clean, but a little airbrushed at face distance. |
| Paint round the eye | http://127.0.0.1:8765/paint/eyes/polish/p5_eye.jpg and http://127.0.0.1:8765/paint/eyes/index.html | A base. Better after the polish, still far from KOTOR on heavy makeup. |
| Procedural eyeball | http://127.0.0.1:8765/paint/eyes/ball/ | The best result of the night. The eyes read as KOTOR's. |
| Lip paint | http://127.0.0.1:8765/paint/lips/sheet6.jpg and http://127.0.0.1:8765/paint/lips/index.html | Good on glossy heads. Colours are a little muted and every head has one lip shape. |
| Forehead lines (earlier) | http://127.0.0.1:8765/paint/lines/sheet.jpg | A base that David already called "a good base for now". |

## State checked by the lead at 05:05

- The integration lock folder is released. No paint agent is running.
- `cargo test --release -p head_paint -p head_paint_gpu` passes: 110, 1 and 2 tests, with 1 ignored.
- The live page loads in headless Chrome on Bastila and Carth with every layer and the eyeball.
- A slider move repainted in 51 ms on Bastila once. It took 110 to 240 ms while the load average was above 50. The cost of the three
  new layers is not measured on an idle machine.

## Open faults, most important first

1. **The paint round the eye.** The upper lash line is a thick black band with a jagged edge on every head. Heavy makeup is weaker than
   KOTOR's and the ring is not closed under the eye. Only nine heads are set by hand after the polish, and the other 48 keep the first
   fit's values, because a refit with the new masks pushed 18 heads to the socket's maximum. The measure needs rework before a refit.
2. **The lash line and the shape session.** kjk-07 has a first rigid upper lid piece (`head_eyes/src/piece.rs`, page
   http://127.0.0.1:8765/lids/). On that page the lid lining is not drawn and the piece carries the lash strip. Judge the lash line
   again with that piece drawn before spending more work on it. The piece wants the painter's skin colour at a point, so that the closed
   lid matches the skin. Nothing is built for that on either side.
3. **The lips.** Colours drift toward a muted tan or mauve, and strong reds come out weak. Gloss is not measured and is set by hand on
   8 heads. Every head has the same lip shape template. comm_b_m is plainly wrong. The open mouth was never checked with the real talk
   shapes.
4. **The eyeball.** Irises are often darker and greyer than KOTOR's. comm_a_f2 and comm_w_f came out near black after the last refit.
   Pupils are too large on old_w_f, old_b_f and pfha05.
5. **The pores.** Faces are a little airbrushed at face distance, and the dark head shows almost no texture there. The forehead above
   32 mm over the eyes has no skin texture, because the layout is coarse there. Only the new layout of roadmap step C1 fixes that.
6. **Forehead lines.** Every line copies its neighbour's bends, the lines look embossed, and broken lines look like dashes. Frown
   lines and mouth corner lines are not started. David asked for these three first, on 2026-11-19.
7. **Older debt.** The four parked folds are not on the GPU. The fine lines cost about 45 ms. The shading turns dark skin white at a
   strength of 3 and darkens the chin groove. The processor painter (`?painter=cpu`) paints the old pores and no eyes, lips or eyeball.
8. **Never verified for any layer.** The look in kjk itself, a GTX 1050 Ti, side views, the blink, and byte equality between the
   native and browser builds.

## Suggested next steps, for David to choose

- Watch the live page with David and take his reactions per layer. This comes before any new build.
- Then one of these, in the lead's order of preference:
  1. Frown lines and mouth corner lines, with the forehead line faults, because David asked for them first.
  2. A second round on the paint round the eye, together with kjk-07's lid piece.
  3. A timing pass on an idle machine and on a GTX 1050 Ti (the skill `kjk-vast-bench` rents one).
  4. Brows and stubble, which no layer paints yet and which make every comparison with KOTOR harder to read.

## Traps

- Sub-agents cannot message the lead while they run. Their final report is their only channel. Resume one with SendMessage and its
  agent id.
- Agents that share the two crates need the lock folder `~/.claude/tools/likeness-fit/paint/integration.lock` and a private copy made
  with `paint/gpu/dev_copy.sh`. The script deletes what its target folder holds. One agent wiped another's copy that way tonight
  (`paint/NOTICE-eyes-agent-2026-11-20.md`).
- An agent that regenerates the page's `recipes.json` must merge only its own keys.
- Chrome's shader compiler is stricter than the native one. A shader that passes natively can fail in the browser.
- Chrome's shader compiler also built the paint pass wrongly twice on 2026-11-20, depending on the shape of the code and not on values.
  It painted black or green texels in dark skin, up to 3,268 on Bao-Dur, and native tests cannot see that. The two shapes that broke
  it: a function that returns a struct of arrays inlined a second time, and texel derivatives taken at the top of the paint pass and
  passed into a function. After every rebuild of the browser module, run
  `~/.claude/tools/likeness-fit/paint/gpu/native_browser_check.sh <out dir> [switches] [head]...`. It compares the native texture
  with Chrome's on six heads and exits with 1 if a texel differs by more than 2 steps. `WASM-BUILD.md` has a section on it.
- A patch must be made from inside one paint crate, never from the `head/` folder above them. On 2026-11-20 a patch made from that
  folder emptied the shape session's five crates (`paint/NOTICE-pores2-agent-2026-11-20.md`).
- On the morning of 2026-11-20, an agent of kjk-07 is editing viewer 2 (`hq/viewer2/viewer2.js` and viewer 2's `index.html`) to bring
  all sliders into it, with one tab per slider feature. It was told to keep the Skin group and the paint choices working, and not to
  touch `hq/paint/`, `head_paint`, `head_paint_gpu` or `hq/wasm/head_paint*`. A paint agent that must edit viewer 2's files re-reads
  them right before each edit.
- Never stop or restart the web servers on ports 8765 and 8766. Check pages in a headless browser, so that no window opens.
- The lead's scratchpad holds 12 GB of agent copies and pictures:
  `/private/tmp/claude-501/-Users-david-perso-kjk/c3910254-b661-450a-9003-645d2158b8cb/scratchpad/`. Everything that matters was
  copied out. The folders `dev`, `dev-eyes`, `dev-eyes2`, `dev-lips2`, `pores-agent-dev` and `pores-agent-old` can go once David agrees.

## Earlier handoffs of the same work

- Forehead lines and folds: `~/.claude/tools/likeness-fit/paint/handoff-lines-2026-11-20.md`.
- Eye paint, stage 1: `~/.claude/tools/likeness-fit/paint/handoff-eyes-2026-11-20.md`.
- The GPU move: `~/.claude/tools/likeness-fit/paint/gpu-status.md`.
- The pore research: `/Users/david/perso/kjk/docs/research/2026-11-20-pores-and-grain.md`.
- The iris research: `/Users/david/perso/kjk/docs/research/2026-11-20-procedural-iris.md`.
