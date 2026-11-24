# Audit of the nose, jaw, brow and cheeks: handoff (2026-11-20)

Brief: /private/tmp/claude-501/-Users-david-perso-kjk/78e955ef-9cc3-4aa3-9d80-4f3216d71481/scratchpad/brief-audit-four.md
Scratch folder: /private/tmp/claude-501/-Users-david-perso-kjk/78e955ef-9cc3-4aa3-9d80-4f3216d71481/scratchpad/a4/

## Where I am

- Started. Read the brief, the rules, the eye audit and round 4 notes, the kit note. Reading code next. Nothing changed yet.
- Code: heads.rs World::load pastes the eye area for brow and cheeks too (original a4/orig/heads.rs; logged). New audit-only binary
  src/bin/feature_audit.rs (presets, faults, judge, starts, sweep, pictures; FA_PASTE=1 pastes the feature's own area, rigid Kabsch
  fit over zone weight > 0.999, faded by the zone weight). Built with CARGO_TARGET_DIR=a4/target.
- Job 1 (presets, provisional marks, scored of 56): brow 27 on old starts, old presets on pasted starts (what the page shows now) 14,
  re-solved 20. Cheeks 48, 29, 34. Faults: brow 1 head after, cheeks 0. Folders a4/j1_before (EYES_PASTE=0), a4/j1_after.
  Live backups: a4/live_before/. Running: full bench in a4/j1_bench (the page needs held_out entries; the quick page lacks them).
- Job 2 running: a4/run_audit.sh, log a4/audit.log; today in a4/au, paste experiment in a4/ex.
- 20:35 Job 1 LIVE from the full bench (a4/j1_bench): brow 20/56 (held-out 20), 1 faulty head (Zherron), cheeks 34/56, 0 faulty.
  nose.js ?b=35, index.html v=30. Kreia's brow: our pasted stand-in can't be measured, preset all 0 (was solved before).
  Seam judge: grooves at outer eye corner in "page now" (old presets on pasted starts) 4 of 5 brow rows; after re-solve 3 of 5;
  Bastila after: lump above eye. Cheeks after: 1 of 5. Pictures a4/pics/seam_*.png (eye sliders not applied in them).
- Job 2 done: a4/au (today), a4/ex (feature paste), a4/analysis.json, a4/an.py. Worst-head judge and page check running.
- 20:50 Page check (headless): no errors; brow and cheek cards work in both modes; live sliders draw. Worst-head judge: TODAY beats
  PASTE or ties on every sheet. Bastila's brow after re-solve: Brow sweep flipped from -3.67 to +4.27, Brow depth 0.91 to -2.85 (lump).
  DONE; report sent. src/bin/feature_audit.rs stays (audit only, delete when no longer wanted).
