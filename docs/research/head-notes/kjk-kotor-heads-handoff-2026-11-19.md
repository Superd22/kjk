# Handoff: KOTOR head likeness work (kjk character creator)

Written 2026-11-19 at the end of a long session. The next session picks up the eye area, the cranium hold and the lash shell.

## The goal and the decisions

- The user wants a FLAME-based head slider system that can reproduce every human head in KOTOR 1 and 2, in KOTOR's style. Photos are no longer test material; only 3D meshes are. Recorded in memory `kotor-heads-as-target.md` and at the top of `/Users/david/perso/kjk/docs/research/2026-11-17-head-parts-and-controls.md`.
- Two things are needed: (1) FLAME reproduces KOTOR faces, (2) people can edit FLAME through a usable UI. Neither is proven. (1) is "done" only when: the fit's own check passes (it does), the cranium is scored, and the user judges likeness feature by feature **in the kjk engine**, with our head wearing the character's own KOTOR texture. The user agreed the grey shapes follow; that is not likeness.
- Deferred until (1) is proven: reducing the vertex count and naming the sliders. Both plans are written in the section "Deferred until the KOTOR shape match is proven" at the end of the reference note above.
- Look comparisons must be rendered in kjk, never in the test renderer (memory `side-by-side-in-engine.md`). Test-renderer sheets are method checks only.
- Downloads are allowed without asking, except a single file over 1 GB (memory `downloads-under-1gb.md`). Mod heads are private test targets; kjk will ship as a mod, so licences are fine (FLAME 2023 Open needs a CC BY credit).
- The user sometimes needs the Mac at full power. When asked to pause: message every running subagent to stop, and `pkill -f "trials-2026-11-18/kotor_"` plus `pkill -f "scan/wrap.py"`.
- Write all text by `/Users/david/perso/kjk/CLAUDE.md`'s rules (plain full sentences, one fact per sentence, numbers, no colons in titles). Ask after each step. Pictures numbered and labelled. Research subagents run on Opus or smaller. Commit or push only when asked; nothing from this work is committed.

## Where things live

- Tools: `~/.claude/tools/likeness-fit/` (run Python as `.venv/bin/python -I` from there; install packages with `uv pip install --python .venv/bin/python <pkg>`).
- Trial scripts and their README: `~/.claude/tools/likeness-fit/scan/trials-2026-11-18/` and its `README.md` (it documents every script below; read it first).
- Data: `~/.claude/tools/likeness-fit-data/kotor-heads/`.
- Research notes, untracked in the kjk repo `docs/research/`:
  - `2026-11-18-kotor-head-meshes.md` (inventory of the games' heads: 194 face meshes, 48 distinct shapes at 1.5 mm)
  - `2026-11-18-kotor-head-mods.md` (mod heads, what their archives contain, HD face textures)
  - `2026-11-18-painted-facial-hair.md` (how games paint or model brows, lashes, stubble, beards)
  - `2026-11-16-head-body-rig-contract.md` ("Eyes and blinking": lids and eyeballs are rigid meshes rotated about the eyeball's centre)
  - `2026-11-17-head-parts-and-controls.md` (reference note for every head part; has the deferred plans)

## The pipeline as it stands (v5)

1. `kotor_hairless.py` converts a KOTOR OBJ and wraps our mesh onto it: hair, edges and the mouth pocket are left out, and since v5 our lid edge goes onto KOTOR's eye slit while our lids stay on our own eyeball. Outputs `<name>_scan.npz` and `<name>_wrapped.npz`. Current targets: `kotor-heads/wrap_v5/` (57 heads: 45 game heads plus Kreia and Bastila, and 12 mod heads; list in `kotor_heads.txt`).
2. `kotor_fit.py` fits three slider sets (FLAME's 300 shapes, our 60 named controls, MakeHuman's 194 sliders) and scores per zone, mean and 95th percentile, plus proportions. `kotor_fit_check.py` proves the fit recovers slider-made targets (FLAME 0.013 mm, named 0.000, MakeHuman 0.024).
3. `kotor_paint.py` carries each KOTOR texture onto our fitted head and builds the per-feature sheet (`kotor_features_v5.png`, 10 characters).
4. `kotor_blink.py` checks that the lids close the way `build_rig.py` rigs them.
5. `kotor_v5_all.sh` reruns all of the above.
6. Other experiments: `kotor_why.py` (cause test), `kotor_detail.py` (smoothing test), `kotor_vertices.py` (vertex-count curve), `kotor_skull.py` (cranium hold, see below).

## Current numbers (FLAME, 57 heads)

- v5: face 0.70 mm (95th percentile 1.47). Around the eyes 0.93 mm (2.82). Nose 0.56, mouth 0.51, chin 0.80, jaw 0.81. Results in `kotor_fit_v5.json`. Average head without sliders: about 2.15 mm. Named controls: about 1.9 mm. MakeHuman: about 1.0 mm, partly bought with pointed skulls.
- Proportions: FLAME's lower face comes out a little bigger than KOTOR's (jaw +0.8 mm, brow-to-chin +0.7 mm). Eye spacing is within 0.22 mm on average.
- Pass mark proposed (not yet confirmed by the user): face within 0.5 mm, no zone worse than 0.75 mm, and our head closer to its own target than to any other KOTOR head.

## Open work, in the order agreed

1. **Eye openings.** FLAME cannot change the eye opening, because `flame_transfer.py` keeps our own lids rigid within 15.5 mm of each eye (`EYE_OWN`). Every fit keeps the same 11 mm by 26.5 mm opening. KOTOR's slits run 4.4 to 13.7 mm tall and 17.8 to 30.8 mm wide. Plan agreed: add our existing eye-opening controls (width, height and place, from `stretch_opening` in `eye_targets.py`, plus the 14 lid-curve points per eye, from `lid_curves` in `scan/eyes_layers.py`) to the FLAME slider set in the fit. Keep the lids on the eyeball, so the blink still works.
2. **Cranium hold.** Merge `kotor_skull.py`'s `fit_cranium` into `kotor_fit.fit` as an option, with a switch per head. It holds our upper skull in a band under KOTOR's outer surface, hair included. Results on the old targets (`kotor_skull.json`): FLAME cranium miss 2.42 → 0.32 mm, face 0.68 → 0.74 mm. On 47 of 57 heads the face cost is under 0.1 mm. Switch it off for heads with hair panels and no skull: Kreia (`p_kreiah`) and the `mod_hp_pfhb02` mod. Rerun on `wrap_v5`.
3. **Lash shell (the user's point, not yet decided whether it goes with step 1 or after).** KOTOR's separate lid pieces (`eyellid`, `eyerlid`) carry the lashes and liner as geometry. Of 185 heads with lid pieces, 67 are textured black and 74 dark, mostly the women. Bastila's is fully black and 45 mm wide, so it shows as a wing past her eye. Plan: a thin strip along our lid edge that moves with the lid bone, fitted to KOTOR's lid piece and painted from it. This should also fix the doubled lower liner, because our paint transfer only reads KOTOR's face mesh and misses the liner on the lid piece. Ask the user whether to do it together with step 1.
4. Then a new feature sheet for the user to look at, **before** any work on new KOTOR-learned shapes ("step 3" in the conversation: learning the leftover ~0.6 mm from the KOTOR heads, tested on heads they were not learned from).
5. Then the in-engine likeness proof. First find out whether kjk can load a replacement head model. Then export our fitted head as a KOTOR model with the rig contract's nodes, wearing the character's texture, and show it next to the original.

## Known faults and lessons from this session

- Every fault found so far was in the targets or the method, not the sliders. These were: an alignment that stretched mask-only heads, hair taken as skin, the chin pulled into the mouth pocket, an eyeball radius read from an 8 mm marker cube, torn lids, the lid lining tucked into the eyeball, and an 8% size mismatch between the MakeHuman and FLAME heads. Always check a new measure on slider-made targets (`kotor_fit_check.py`) and look at a sheet before believing a number.
- Blink: our lids blink by rotating lid-weighted skin about the eyeball's centre (`build_rig.py`). The lid weights must be computed on each head's own shape. MakeHuman's weights reused on FLAME heads tear the lids. With weights computed per head, the lids still dip up to 0.9 mm into the eyeball, and there is a small notch at the outer corner (`blink_v5.png`).
- Remaining eye faults on the v5 sheet: the openings are rounder and taller than KOTOR's, Bastila's and Kreia's lower liner shows doubled, the eyeball shows through at the outer corner on pfha02, and the upper lid overhangs in three-quarter views.
- MakeHuman's sliders at their limits turn noses inside out and make pointed skulls. Their low scores are misleading.
- An unfinished idea that failed: a hard limit holding all unseen skin under KOTOR's surface crumpled the necks. Only the soft, upper-cranium version in `kotor_skull.py` works.

## Suggested skills

- `mattpocock-skills:diagnosing-bugs` when a sheet shows something broken (torn lids and dark patches were all bugs in the pipeline).
- `dataviz` before making any chart or comparison sheet.
- `artifact-design` and the Artifact tool only if the user asks to update the Likeness HQ page (memory `likeness-hq.md`; it has not been updated for the KOTOR work).
- `run` or the kjk-specific workflow, when reaching the in-engine proof (step 5), together with memory `game-windows-aerospace.md` for keeping game windows off the user's screen.
