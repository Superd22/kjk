# Bald references for pmhc04 and Bastila, and the whole head measured against them

This note was written on the night of 2026-11-21 by the conversation briefed by the head HQ (brief: `/private/tmp/claude-501/-Users-david-perso-kjk/52003e36-b8ca-4208-a733-a6d51b417e4e/scratchpad/jaw/brief-bald-references.md`). The work serves B1 and B2 for the skull, and A2. No sliders were built. Nothing live changed. Nothing was committed. No judges were run.

## Verdict

Our bald skull follows KOTOR's hair. On the 38 KOTOR heads whose skull shows, our skull matches KOTOR's to within half of KOTOR's spread on every skull row. On the heads with modelled hair, our skull lies at KOTOR's outer surface, hair included.
- On pmhc04, our skull is 32 mm wider than the reference, 12 mm higher and 10 mm longer. It is as wide as KOTOR's head with its hair on (157.5 mm against 158.2 mm), and its top is 5 mm above the top of KOTOR's hair.
- On Bastila, our skull is 22 mm wider than the reference.
- The cause is the FLAME fit's held skull. The cranium was pulled into a band under KOTOR's outer surface, hair included (`kotor_skull.py`). Viewer 2's live head has the same skull as the FLAME fit on every skull row, so the presets play no part.

## The references

- **No real bald version exists.** Bastila's game model has a face-mask "head" part, and her hair parts ("hair", "hairbangs") are the outside of her skull. pmhc04 models its hair as the skull's surface in both K1 and K2. The mods `mod_improved_bastila` and `mod_tcs_bastila` have no scalp under their hair either. The TCS mod adds a real ear mesh.
- **How they are built.** Our mesh (from each head's FLAME fit) is laid onto KOTOR's triangles that are not hair, along each vertex's normal, within 20 mm. The vertices that land on skin are real. The rest come from a 12-component shape model learned from the 38 heads whose skull shows (`SKULL_SHOWS` in `bald.rs`). The model is fitted to the real vertices, joined to them by a harmonic fill of the gap, and then kept at least 6 mm under KOTOR's hair. The 6 mm comes from the 25th percentile of the gap between hair caps and real skulls on Meetra, Jaesa, pfhb02 and the Exile mods (5.8 to 9.6 mm).
- **How much is filled.** pmhc04 has 2,131 real merged vertices and 1,584 filled ones. Bastila has 2,165 real and 1,550 filled.
- **How good the fill is.** Each target's hole was cut into each of the 38 bald heads and filled from the other 37. The root-mean-square gap along the normal is 3.6 mm on pmhc04's hole (worst head 6.1 mm), and 3.6 mm on Bastila's hole (worst head 5.8 mm).
- **pmhc04's hair** is found by its dark brown texture (lightness under 45, or under 52 with little red). Nothing inside the face is cut (between the brows, the eyes' outer corners and the chin), and the beard in front of the ears is kept.
- **The ears are not measured.** The ear area is closed smoothly from the skin around it and leaves small bits in the pictures. David said the ears come later.

## The measure (head frame, mm, read on KOTOR's own meshes)

Wobble is KOTOR's mesh against the same mesh subdivided twice, over the 38 bald heads. Spread is KOTOR's standard deviation over them. "Live minus KOTOR" is the median over the 38 bald heads.

| Row | Wobble median / 95th percentile | KOTOR's spread | Live minus KOTOR (bald heads) |
|---|---|---|---|
| Top of the head above the eyes | 1.0 / 2.2 | 6.4 | −3.0 |
| Chin's bottom below the eyes | 0.7 / 2.0 | 4.2 | +1.6 |
| Brow in front of the eyes | 0.3 / 1.4 | 3.3 | +0.5 |
| Back of the head behind the eyes | 0.1 / 2.6 | 10.5 | −1.3 |
| Height of the back of the head | 1.8 / 9.1 | 17.4 | +0.8 |
| Skull length | 0.7 / 2.7 | 11.4 | −0.2 |
| Skull width | 1.2 / 4.1 | 8.3 | +1.7 |
| Forehead slope (degrees) | 1.3 / 4.8 | 5.5 | +1.7 |
| Where the top lies, front to back (share of length) | 0.03 / 0.07 | 0.06 | +0.02 |
| How far forward the eyes sit (share of length) | 0.00 / 0.01 | 0.01 | 0.00 |

- Check 1 passes on every row except the back's height and the forehead slope at the 95th percentile.
- Check 3: the subdivided copy finds its own head for 35 of 38 heads.
- Check 4: the extremes are listed in the stats output. pfha02 (longest back) and pfhc02 (highest top) may be painted hair volumes rather than skulls. They are probably wrong members of `SKULL_SHOWS` and widen KOTOR's spread.
- Check 2 needs David.
- The neck rows (`nape_in`, `nape_slope`, `neck_depth`, `neck_under`, `crown_curve`) are computed but broken. KOTOR's neck stubs are open at the level they read. David dropped the back of the neck from this work, so they are not reported.

## Ranked differences, live against the reference, in KOTOR's spread

pmhc04 ("filled" means the reading rests on the filled skull, with its 3 to 6 mm uncertainty):
1. Skull width: 157.5 against 125.2 mm, +3.9 spreads, filled. Without the fill: ours equals KOTOR's width with the hair on (158.2).
2. Top of the head: 110 against 98 mm, +1.9 spreads, filled. Without the fill: ours is 5 mm above KOTOR's hair (105).
3. Skull length over face height: 1.82 against 1.66, +1.6 spreads, filled.
4. Back of the head: 170.4 against 160.1 mm, +1.0 spread, filled. Without the fill: ours equals the hair's back (171.5).
5. Forehead slope: 14.8 against 20.6 degrees, −1.05 spreads, filled. Ours is more upright, which is the opposite of the HQ's guess.
6. Face height: 112.6 against 117.0 mm, −1.0 spread, real. Our chin sits 4 mm high.

Bastila:
1. Skull width: 173.8 against 152.0 mm, +2.6 spreads, filled.
2. Height of the back of the head: 24 against 5 mm, +1.1 spreads, filled, on a row with a large wobble.
3. Where the top lies: 0.53 against 0.47 of the length, +1.1 spreads, filled.
4. Face height: 121.8 against 118.9 mm, +0.7 spreads, real.
5. Forehead slope: 7.8 against 10.8 degrees, −0.55 spreads, real.
- The top and the length match within half a spread.

Over the 19 heads with modelled hair, live minus KOTOR's outer surface is +3.6 mm in width, −1.0 mm at the top and +3.1 mm at the back (medians). Our skull follows the hair.

The jaw, the throat and the front of the neck are measured by `under_jaw.rs` on KOTOR's real skin. See `kjk-under-jaw-measure-2026-11-21.md`. pmhc04's throat angle is 90 degrees on KOTOR, and ours is about 128 degrees. Its ledge under the jaw is 14.3 mm, and ours is 9 to 10 mm.

## Pictures (scratchpad `/private/tmp/claude-501/-Users-david-perso-kjk/6938bbd2-56c1-41ef-8954-a9e7823dfa58/scratchpad/bald/`)

- `overlay_pmhc04.png` and `overlay_p_bastilah.png`: our outlines over KOTOR's reference, from the side and the front, with the hair off and on. Yellow is viewer 2's live head and cyan the FLAME fit. The live head is built as in `jaw_edge::tools::Heads`, so the eye curve is off. That changes nothing outside the eyes.
- `ref_pmhc04_v2.png` (after the eye fix) and `ref_sheet.png` (Bastila is right, pmhc04 is the older one): the references, real skin in clay, filled in orange, hair in blue.
- `look/k_0.png` to `look/k_5.png`: all 57 KOTOR heads with colour-found hair in blue and hair parts in orange.
- `measure.json` and `stats.json`: every row on every head.

## What was built

All of it is in the head-paint worktree, crate `head_sliders`. Nothing the viewer uses reads it.
- `src/bald.rs` (new). It holds the commands `sliders bald look|thickness|fill|show|measure|overlay <data> ...`. They need `BALD_HAIR=<scratchpad>/bald/hair`.
- `src/lib.rs` has one new line, `pub mod bald;`. `src/bin/sliders.rs` has the `bald` key.
- The build uses `CARGO_TARGET_DIR=<worktree>/target-bald`.
- The scripts are in the scratchpad's `bald/scripts/`. `prep_hair.py` writes each head's KOTOR hair parts from the viewer data's `hair/<name>.json`. `stats.py` computes the checks and the ranking.
- To undo, delete `bald.rs` and its two lines.

## Traps

- The colour test (`kotor_skin`) fails in both directions. On about 35 heads it calls painted short hair "not skin", although their outer surface is a real skull. On pmhc04 it calls the hair volume "skin". The skull-showing heads were chosen by eye from the `look` sheets.
- `kotor_v` holds only the face mesh objects. Bastila's hair is in her separate hair parts, which come from the viewer data.
- `bald.rs` uses `jaw_edge::tools::Heads`, which belongs to the jaw conversation. If its API changes, `bald.rs` breaks.
- The reference's width on pmhc04 (125 mm) is 3 spreads below the bald heads' mean. The fill is pressed by pmhc04's tight hair, so the exact number is uncertain. The finding does not depend on it, because ours equals the hair's width.

## Next step

Refit the skull of the 19 heads with modelled hair, holding the cranium under the hair rather than at it. This conversation's fill can be the target, or the hold can keep a measured hair thickness under the hair. Then re-measure with `sliders bald measure`. After that, skull sliders for width, length and height can be ranged from the 38 bald heads.
