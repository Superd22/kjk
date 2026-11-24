# Fast slider solver: handoff (2026-11-20)

Brief: /private/tmp/claude-501/-Users-david-perso-kjk/78e955ef-9cc3-4aa3-9d80-4f3216d71481/scratchpad/brief-fast-solver.md
Scratch folder: /private/tmp/claude-501/-Users-david-perso-kjk/78e955ef-9cc3-4aa3-9d80-4f3216d71481/scratchpad/fs/
- `fs/d/`: my own data folder (input is a symlink to the live input; marks and slider sets are copies taken at the start).
  The eye set is the eye agent's 17-slider `sbf/d/eyes/sliders_solve.json` (1,022 fields).
- `fs/target/`: my own CARGO_TARGET_DIR.

## Where I am

- Started. Read the brief, rules, kit handoff, solve.rs, set.rs, bench.rs, heads.rs, the eye measure.
- Plan: new binary `src/bin/sliders_fast.rs` only (no shared-file change), using the crate's public API.
- 18:35 Found the main cost: `solve::Evaluator::new` smooths every slider movement and every shared field (1,022 for the eyes) per
  head and per solve: 10.6 s per head on the eye set, against 16 ms when the smoothing is done once and shared. The measure itself is
  9 to 11 ms (eyes). Same values bit for bit with the shared smoothing (profile on comm_a_f and mod_improved_bastila).
- Binary `src/bin/sliders_fast.rs` (auto-discovered, no Cargo.toml change): `profile`, `compare` (FAST_SOLVER=old|same|broyden),
  `presets` (quick mode). Builds go to `fs/target` (running) and `fs/target2` (newest).
- Running: `fs/cmp/run_eyes.sh` (eyes: same, broyden, old), log `fs/cmp/eyes.log`, results `fs/cmp/eyes_<solver>.json`.
- 18:50 Eyes, 57 heads, 17 sliders: old 147.0 s, new (shared smoothing, same search) 30.2 s, values identical bit for bit.
  Broyden updates: 28.5 s, no fewer measures, 16 heads worse by over 5% in cost. Dropped (kept behind FAST_SOLVER=broyden).
  Filtering triangles near the eyes: the window already holds only those (2,863 of 2,865). Dropped.
- bench.rs got a 4-line hook `SOLVE_HEAD` (logged in the shared log); `sliders_fast <f> grade` runs the whole bench with the new solve.
- Running: `fs/cmp/run_rest.sh` (nose, jaw, brow, cheeks: same then old), log `fs/cmp/rest.log`.
- 18:50 Four other features, old then new, values identical on every head: nose 55.7 s to 19.1 s, jaw 101.1 s to 34.5 s,
  brow 32.5 s to 14.0 s, cheeks 25.2 s to 2.8 s. Results in `fs/cmp/<f>_{old,same}.json`, `fs/cmp/diff.py` compares.
- Running: `fs/cmp/run_eye_presets.sh`: quick mode on the eyes with each solver into `fs/new` and `fs/old`, then made-eyes sheets
  `fs/cmp/made_new.png` and `made_old.png`. Next: `grade` timing on the eyes, crate tests, report.
- 18:56 Done. Eye quick mode: 15.6 s new, 63.9 s old (same values, identical made-eyes pictures `fs/cmp/old_vs_new_eyes.png`).
  Eye full grading with the new solve (`grade`): 162 s, against 1,093 s in the eye agent's bench1.log; every scorecard, pair, sample
  and fault line is the same as bench1.log. Crate tests pass. Report sent; nothing left running.
