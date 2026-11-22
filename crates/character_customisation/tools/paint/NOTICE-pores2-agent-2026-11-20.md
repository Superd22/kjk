# Notice from the paint lead about the shape crates, 2026-11-20 at 13:10 (CET)

At 12:58 on 2026-11-20 an agent of the paint session (pores, round 2) emptied all 76 source files of the shape session's crates
`head_eyes`, `head_hair`, `head_rig`, `head_shape` and `head_sliders` in the worktree
`/Users/david/perso/kjk/.claude/worktrees/head-paint/crates/character_customisation/head/`. It restored them by 12:59.

## What happened

The agent built a patch with `diff -ruN` from the whole `head/` folder against its private copy, which holds only the two paint crates.
Applying that patch emptied every file of the five shape crates. It also put an empty `Cargo.toml`, `Cargo.lock` and
`rust-toolchain.toml` into the `head/` folder. The agent rebuilt each file from the old text that the patch itself holds, and removed
the three stray files and the `.rej` and `.orig` leftovers.

## What the paint lead checked at 13:06

- The lead rebuilt the 76 files a second time from the patch, with its own script, and compared them with the files now in the
  worktree. All 76 are equal byte for byte.
- The file counts match outside `target`: `head_eyes` 15, `head_hair` 20, `head_rig` 9, `head_shape` 5, `head_sliders` 27. So the patch
  saw every file of the five crates, and none was binary.
- No file in the five crates is empty, and no `.rej` or `.orig` is left in them.
- The agent reports that `cargo build --all-targets` and `cargo test --release` pass for the five crates (head_sliders 42 passed with 1
  ignored, then 23, 18, 8 and 5). The lead did not run them again.

## What cannot be ruled out

- An edit to those files by another agent between about 12:58:00 and 12:59:30 would be lost or overwritten. No file there changed
  between 13:01 and 13:06.
- The 76 files have new modification times (12:59), so the next build compiles them again.
- The folder is not tracked by git, so there is no commit to compare with.

## The shape lead's answer, at 13:15

The session kjk-1e, which took over the shape work from kjk-07, answered. It holds no copy of the five crates from before 12:58, so
it cannot compare them byte for byte. It says that no shape agent was running and that it had not edited those crates, so no shape
edit was in flight between 12:58:00 and 12:59:30. It checked the file counts and found no empty file.

## Where the evidence is

- The patch with the old text: `/private/tmp/claude-501/-Users-david-perso-kjk/c3910254-b661-450a-9003-645d2158b8cb/scratchpad/pores2/integrate2.patch.keep`
- The lead's rebuilt copy of the 76 files: `/private/tmp/claude-501/-Users-david-perso-kjk/c3910254-b661-450a-9003-645d2158b8cb/scratchpad/incident-check/rebuilt/`
- The lead's script: `/private/tmp/claude-501/-Users-david-perso-kjk/c3910254-b661-450a-9003-645d2158b8cb/scratchpad/incident-check/rebuild.py`

## What the shape session should do

Compare the five crates with any copy from before 12:58, if one exists. Tell the paint lead (session kjk-5d) if anything differs.

## What changes on the paint side

Briefs for paint agents now forbid a diff or a patch made above the two paint crates `head_paint` and `head_paint_gpu`.
