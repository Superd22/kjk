# The head customisation tools

These are the tools that drive the crates in this folder's sibling
`head/`. They live here so that the work is tracked in one place. They are
not part of the mod, and nothing in kjk builds or runs them.

On the machine they were written on, they sit in
`~/.claude/tools/likeness-fit/`, and their data and pictures sit in
`~/.claude/tools/likeness-fit-data/`. The scripts still expect those
folders, so a fresh clone cannot run them without being pointed at the
data first.

## What is here

- `build_wasm.sh` builds one crate for the browser and writes its module
  and its JavaScript glue. `wasm/load.js` is what a page imports to load
  that module and to move typed arrays in and out.
- `viewer2/` is viewer 2, the page that shows a built head next to its
  KOTOR original. `index.html` is the page, `viewer2.js` draws the head,
  `sliders.js` runs the shape sliders, and `paint_live.js` runs the paint.
- `hq/` builds the Likeness HQ status page and the picture sheets, and
  serves them locally. The `template*.html` files are the page layouts.
- `sliders/` prepares the slider inputs and exports them for the browser.
- `paint/` builds the paint picture sheets, and holds the handover notes of
  the paint work.
- `python/` is the older Python pipeline: the FLAME fits, the eye and
  outline fits, the rig preview and the test renderer. Decision 22 and
  decision 53 say that anything that ships, and anything heavy to compute,
  is written in Rust. These scripts stay until they are ported.

## Working on two features at once

`sliders/serve_feature.sh <feature> <worktree>` gives one conversation a
viewer, a browser module and a slider data folder of its own, so that two
conversations in two worktrees do not overwrite each other's work. It takes
the next free port from 8770 upward, and a stack costs about 50 MB because
only that feature's own data is copied and the rest are symlinks.
`build_wasm.sh` takes the worktree as its third argument, or reads
`KJK_WORKTREE`, so each worktree builds its own module.

The whole flow, including what the HQ commits before the conversations
start and what it does at merge time, is the section "Working in parallel on
the head" in `docs/research/2026-11-19-head-roadmap.md`.

## What is deliberately not here

Three files that viewer 2 reads were left out, because they hold game data
or pictures made from it, and `AGENT.md` forbids both.

- `viewer2/kotor_eye_tris.json` lists, per KOTOR head, the triangles of its
  lid pieces and its eyeballs. Those indices come out of KOTOR's own
  meshes.
- `viewer2/thumbs/thumbs.webp` holds the thumbnails of the 57 test heads,
  which are pictures of extracted models.
- The three built `likeness-hq*.html` pages embed renders of KOTOR's heads
  and photographs of real people. Their templates and their builder are
  here, so the pages can be made again.

Viewer 2 will not draw KOTOR's eyes without the first file, and will show
no thumbnails without the second.
