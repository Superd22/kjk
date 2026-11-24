# Solver merge and eye audit: handoff (2026-11-20)

Brief: /private/tmp/claude-501/-Users-david-perso-kjk/78e955ef-9cc3-4aa3-9d80-4f3216d71481/scratchpad/brief-eye-audit.md
Scratch folder: /private/tmp/claude-501/-Users-david-perso-kjk/78e955ef-9cc3-4aa3-9d80-4f3216d71481/scratchpad/ea/

## Where I am

- Started. Read the brief, the rules, the fast solver note, the eye fix note, the stage B notes, the research. Reading code next.
- 19:05 Job 1 done in code: solve.rs has `Smoothed` and `Evaluator::with`; bench.rs `solve_head` uses a shared `Smoothed`, `SOLVE_HEAD`
  removed, `presets` added; `sliders <f> presets <data>`; `sliders_fast.rs` moved to ea/removed/. Broyden dropped.
  Proof: ea/m (copy of fs/d), `ea/run_merge.sh`, `ea/cmp_merge.py`: all 5 features, 57 of 57 heads identical values to fs/cmp/<f>_old.json.
  Times (quiet Mac): nose 4.6 s, jaw 8.3 s, brow 4.2 s, cheeks 1.8 s, eyes 12.6 s (old: 55.7, 101.1, 32.5, 25.2, 147.0 s under load).
  Tests pass (5), wasm lib checks. Kit handoff commands updated. Running: full eye bench in ea/mb for its time (ea/bench_eyes.log).
- Next: Job 2, the eye audit, in ea/a (scratch data folder).
- 19:07 Full eye bench through `sliders eyes bench` (ea/mb): 113.6 s, same fault and sample lines as sbf/bench1.log.
- Job 2: new binary `src/bin/eye_audit.rs` (audit only, remove when done): commands presets, starts, sweep, iris, pictures; EA_PASTE=1
  pastes the base head's eye area into every start (rigid per eye, mean core translation, fade 1 to 1.7 times the 17 x 11 mm oval).
  Data folders: ea/today (live eye set, today's starts) and ea/exp (pasted). Script ea/run_audit.sh, log ea/audit.log.
  Finding while reading: the start shape already has the eye zone's own shape taken out (zone.own_shape: base + harmonic fill), except the
  eyeball-centre vertices (zone weight 0), which keep the fit's place. The solver's Evaluator does not apply the press; place-whole does.
- 19:30 All measures done. Results: ea/today/eyes/{audit,starts_audit,iris,sweep}.json, ea/exp/eyes/{audit,iris,sweep}.json.
  Analysis scripts ea/an1.py (sliders, faults, measures reached), an2.py (start shapes), an3.py (iris), an4.py (sweeps), an5.py (residual
  features). Pictures ea/pics/close.png and front.png (pfha06 comm_w_f pmha06 comm_a_m pfha02 p_bastilah). A Sonnet judge looks at them.
  Headlines: experiment faults 37 heads / 808 triangles vs today 54 / 1376; Eye width gain 0.31 mm per mm (at an end on 29 of 57);
  KOTOR's lid piece lowers the visible top by over 1 mm on 26 of 55 heads (Bastila 2.9 mm); our opening sits about 2.7 mm higher round
  the eyeball than KOTOR's on Bastila; outer_reach missed on 50 of 57, inner_reach (no slider) on 41; press changes width by 0.9 mm median.
- Next: judge's answer, then the report. Nothing live was changed.
- 19:40 Judge (Sonnet, pictures only): the experiment is cleaner on all 6 rows and closer to KOTOR; both still show more iris than KOTOR.
  Eye width at its low end (22 heads) is a stand-in for a missing inner corner slider: those heads need the inner corner 2.65 mm nearer the
  eye's centre, and Outer corner position (+1.43) puts the outer corner back. Report sent. Done; nothing running. src/bin/eye_audit.rs stays
  (audit only, delete when no longer wanted).
