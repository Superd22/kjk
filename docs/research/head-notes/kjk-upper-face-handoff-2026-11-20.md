# Upper face sliders (brow ridge, forehead, cheekbones, cheeks): handoff

Started 2026-11-20 at about 03:40 by the agent that builds these sliders with the slider kit. Brief: the lead's message (rules in
the scratchpad `overnight-rules.md`). Read the kit handoff (`kjk-slider-kit-handoff-2026-11-20.md`) and the nose handoff first.

## Where I am

- 03:40. Read the rules, the roadmap, the kit handoff, the nose handoff and the two reports. Next: read `feature.rs` and `jaw.rs`,
  then draw KOTOR's 57 heads on the brow, forehead, cheekbones and cheeks.
- 03:55. Two features in my own files: `src/brow.rs` (key `brow`, "Brow and forehead") and `src/cheeks.rs` (key `cheeks`,
  "Cheekbones and cheeks"). `brow.rs` also holds the shared helpers both use (`upright`, `level`, `crest`, `side_at_depth`, `cover`,
  `measure_uncovered`). Registered in `lib.rs` and `feature::by_key` (logged in the shared log); `feature::all()` not yet.
- Check 1 (subdivided wobble median/95th against KOTOR's spread): sturdy are brow_projection 0.80/1.27 vs 4.38, brow_height (the
  ridge's lower edge, held, not a slider: it is the eye agent's socket) 0.30/1.14 vs 2.70, brow_middle 0.30/1.34 vs 4.98, temple_width
  0.47/1.56 vs 4.60, cheek_width 0.97/2.33 vs 6.44, cheekbone_projection 0.83/1.37 vs 3.28, cheek_fullness (cheek 45-55 mm below the
  eyes minus the cheekbone 15-30 mm below) 0.24/0.70 vs 3.34, cheek_side 0.64/1.69 vs 6.14. Weak: forehead_slope 1.28/4.19 vs 4.61,
  forehead_curve 0.69/3.12 vs 2.79, brow_sweep 0.94/2.42 vs 2.91. Noisy (cannot tell heads apart): brow_slant 1.49/8.0 vs 8.73. Dropped:
  brow arch (0.58/2.27 vs 1.92), the crest's height, cheekbone height (1.35/10.8 vs 7.0), the chord cheek fullness (spread 1.2 mm).
- Hidden by hair (brow): mod_hp_pfha01 all; mod_tcs_bastila brow; p_handmaidenbb, pmha04, pmha06 forehead; pfha06 forehead+temples.
- Next: zone sheets, then `make` and the clay sheets of the ends.
- 04:50. Round 1 sliders made (data folder `~/.claude/tools/likeness-fit-data/sliders/brow/` and `cheeks/`, sets in `web/brow/` and
  `web/cheeks/sliders.json`). Brow (bumps 9 mm, 7 apart): Brow depth (brow_projection; moves the brow with the lower forehead; was a
  trench when the ridge moved alone), Brow bulge (new measure: crest minus the forehead 15 mm above the lower edge; KOTOR spread only
  1.17 mm), Brow sweep, Forehead slope, Forehead roundness (now a parabola across, wobble 0.35/1.75 vs 2.47), Temple width. Dropped as
  a slider: "Brow between the eyes" (a knob at its high end); brow_middle stays a free measure. Cheeks (bumps 8 mm, 6 apart):
  Cheekbone width, Cheekbone prominence, Cheek fullness, Lower cheek width. All reach their asked spans except Brow bulge's low end
  (-0.2 of -1.2).
- Shared: `Guard::Lids` added (feature.rs, heads.rs); command `leaks` (sliders.rs) runs `brow::tools::leaks` (other features'
  measures at each slider end on the base head, written to `<feature>/leaks.json`).
- Clay sheets of the ends: scratchpad `upper/sheet4_brow_g*.jpg`, `upper/sheet2_cheeks_g*.jpg` (regrid with `regrid.py`).
- Running: `bench` and `leaks` for both (logs `upper/bench1_*.log`, `upper/leaks1_*.log`), with a copy of the binary
  `upper/sliders_r1`. Next: judge pictures (made-sheet on 8 heads) by fresh Sonnet judges, page by a Sonnet sub-agent.
- 05:50. Round 1 benched (brow 22/57 made vs 10 for stand-ins; cheeks 26/56 vs 9; lids 0.0000 mm; ears 0.06 mm; lips 0.49 mm).
  Two fresh judges (scratchpad `upper/judge1/`): ours softer and rounder than KOTOR's angular brow and cheek planes; brows stand out
  less; cheeks too full on gaunt/old heads; faults: Brow depth ends crease/ledge, Forehead roundness high end vertical seam, Cheek
  fullness/Lower cheek width low ends a local dent. Round 1 sources kept as `upper/brow_r1.rs`, `upper/cheeks_r1.rs`.
- Round 2 changes: brow_middle now relative to the ridge (held by Brow depth, so the middle follows); brow zone fades over 10 mm
  toward the socket (was 6); Forehead roundness frees the forehead profile; cheek fullness read 36-46 mm below the eyes (just under
  the cheekbone). Running `all` + `leaks` on the real folder with `upper/sliders_r2` (logs `upper/all_r2_*.log`).
- A Sonnet page agent is adding brow and cheeks to the page (feature.rs `all()` now has 4 features; starts_all.bin rewritten).
- 04:25 (real clock; earlier times in this note ran about 1.5 h ahead). DONE, two rounds. Round 2 on the real folder: brow 29/57
  made (stand-ins 11), cheeks 24/56 (stand-ins 10); lids 0.000 mm. Module rebuilt with round 2 measures, page version 22. Round 2
  judges (`upper/judge2/`): brow close on 5 and 3 of 8, cheeks 5 and 5 of 8. Roadmap line updated. Final report sent.
