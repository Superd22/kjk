# Notice from the eye paint agent, 2026-11-20 at 01:02 (CET)

At 01:02:12 the eye paint agent ran `dev_copy.sh` on the folder `dev` at the top of the paint lead's scratchpad
(`/private/tmp/claude-501/-Users-david-perso-kjk/c3910254-b661-450a-9003-645d2158b8cb/scratchpad/dev`), not knowing that another agent
was using that folder as its own copy. The script copies the worktree's `head_paint` and `head_paint_gpu` over that folder with
`rsync --delete`. Any change in `dev/head_paint` or `dev/head_paint_gpu` that was not yet in the worktree was lost.

The worktree itself was not touched. The pore agent built `pore_pictures` in that folder at 01:02:35, so it is probably the agent affected.
The pore agent should compare its shader `dev/head_paint_gpu/shaders/pores.wgsl` with what it expects, and write it again if needed.

The eye paint agent now works only in `scratchpad/dev-eyes`.

## Second notice, 03:06 (CET)

At 03:05 the eye paint agent rewrote `hq/paint/live/recipes.json` with `paint_live data` (holding the integration lock), to add its eye
values. `paint_live` read the lip agent's `lips/fit.json` of 03:00:40, so 13 heads got lip values from that newer fit instead of the ones
the page had before (comm_a_m, comm_b_f, mod_hp_pfha01, n_daraalah, n_dodonnah, n_dorakh, n_hussef, n_jedimalek, old_w_f, old_w_m,
p_joleeh, pmha01 and the forehead of none). The agent then ran `lips/merge_recipes.py` with that same `fit.json`, which changes nothing
more. If the lip agent meant the page to keep the older lip values, it should merge its intended fit again.
