# Eyelids and eyelashes

This note was written on 2026-11-19. It records where the work on our head's eye openings, blink and eyelashes stands, and what is left. The work is paused so the rest of the face can come first. Everything here was done in the likeness-fit trial scripts (`~/.claude/tools/likeness-fit/scan/trials-2026-11-18/`), and nothing is in the kjk repo yet.

## The goal

The head system must reproduce every human head in KOTOR 1 and 2 (see `2026-11-17-head-parts-and-controls.md`). The KOTOR heads are only test targets. What ships is a character customizer, so nothing may be stored for one KOTOR head. Every eye a player can make must open, blink and wear lashes correctly.

The order of work is fixed. First we prove that our sliders reproduce every KOTOR head. The values the fits use then give each control's real range. The player's sliders must cover those ranges and stop at their ends. Random slider settings are not used as tests.

## Decisions

- **The eye opening is set by our own controls, added next to FLAME's shapes.** FLAME cannot change the opening, because our lids are kept rigid within 15.5 mm of each eye's centre.
- **The blink is computed on each head.** The closed lids are worked out from the head's own lid edges. The old rig turned the lid skin about the eyeball's centre with fixed weights, and that failed on every FLAME head, the base head included. Whether the engine computes the closed shape or the sliders carry it is decided later.
- **The lashes and the liner go on a "lash strip".** This is a thin strip of mesh along our lid edge that closes with the lid. Its own sliders, outside FLAME, set how far it extends and its colours. KOTOR has a similar piece, the separate lid meshes `eyellid` and `eyerlid`, but those are made by hand for each head and are only test targets for us.

## Where it stands

### The eye opening

Each eye has 26 opening controls:
- one for the opening's width, one unit widening it by 1 mm;
- one for its place sideways, one unit moving it 1 mm toward the temple;
- twelve for the height of the lid edge, at seven stops along the lids (the two corners and five places on each lid);
- twelve for the depth of the lid edge at the same stops.

In front of the eyeball, a change of height or width slides the edge along the eyeball. Beside the eyeball, where KOTOR's corners often lie, the edge moves freely and the depth controls set how far back it lies. After a fit, any lid vertex inside the eyeball is moved straight forward until it is 0.2 mm outside it.

Three changes to the fit were needed:
- Each lid edge vertex is pulled onto the place where the wrap put it on KOTOR's slit. Without this pull, the fit used the new controls to stretch the skin around the eye, and opened the eyes 7 to 12 mm too tall.
- The target is no longer rescaled during the fit. With a free scale, the fit shrank one head (pfha02) to 90% of its size, and KOTOR's slit then no longer fitted round our eyeball, which has a fixed size.
- The lid controls only act on our lids and the skin within 24 mm of each eye's centre.

Results on the 57 KOTOR heads (`kotor_fit_v6.json`):

| Measure | FLAME alone | FLAME with the opening controls |
|---|---|---|
| Distance from our lid edge to KOTOR's slit | 5.85 mm | 0.87 mm |
| Opening height difference | 2.66 mm | 0.07 mm |
| Opening width difference | 3.15 mm | 0.99 mm |
| Around the eyes | 0.89 mm | 0.64 mm |
| Whole face | 0.70 mm | 0.64 mm |

### The blink

`kotor_close.py` closes the lids of any head in three steps.
1. Each point of the upper lid edge, and the lower lid edge's point at the same place across the eye, move toward a meeting line round the eyeball. The meeting line lies a quarter of the way up from the lower edge, so the upper lid does three quarters of the move.
2. Each edge point's move gives an angle about the sideways axis through the eyeball's centre. These angles are spread smoothly over the skin out to 24 mm from the eye's centre, and each vertex turns by its own angle. A turn keeps a vertex at its distance from the centre, so the lid slides over the eyeball.
3. No lid vertex may end up inside the eyeball.

Results on the 57 KOTOR fits (`close_v6.json`):
- Every head closes. The largest gap left between the lids is under 1 mm on every head.
- No lid skin goes inside the eyeball. The base head is the only exception, at 0.38 mm, and its own open lids already sit that far inside.
- Between 11 and 160 triangles turn over per head. Most of them are in the inside of the lid margin, which is hidden once the eye is closed.

The comparison sheet is `blink_sheet_v6.png`, with nine heads open, half closed and closed next to KOTOR's own blink. KOTOR's lid pieces are turned 17 and 34 degrees about each eyeball's centre, as the game's clips turn them. The shape of our blink works on all nine heads.

## What is wrong or missing

1. **The paint on the closing lid is wrong.** When the upper lid comes down, it shows the paint that was taken from where the open lid sat. That includes the liner and the lash line, so the closed lid looks smeared and dark. This is worst on Bastila, pfha02 and Meetra. The lid skin needs plain skin paint, taken from above the eye, and the liner must move onto the lash strip.
2. **The lash strip does not exist yet.** Until it does, Bastila's and Kreia's lower liner shows doubled, because our paint transfer only reads KOTOR's face mesh and misses its lid pieces. Bastila's liner wing, which sticks out 45 mm wide past her eye, cannot be made at all.
3. **Some opening controls are too narrow.** The depth controls at the inner corner hit their limit on 31 to 36 of the 57 heads. The opening's sideways place hits its limit on 10 to 13 heads. Their ranges must be widened.
4. **One head fails.** On the mod head `mod_hp_pfha01` the lid edge lies 7.18 mm from KOTOR's slit. The next worst head is 2.1 mm off. This head has hair panels and no skull, so its target may be the fault.
5. **A small dent above the outer corner when closed.** The corner stays still while the lid next to it turns about 30 degrees, and the skin between them is pulled into a wedge.
6. **The opening is still 1 mm narrow on average.** Both corners stop about 1 mm short of KOTOR's.
7. **The blink sheet missed Meetra's lid pieces.** This mod head names them differently, so they did not move in the sheet.

## Next steps, when the work resumes

1. Paint the lid skin from the skin above the eye, so a closed lid shows skin.
2. Build the lash strip and its sliders. Fit them to KOTOR's lid pieces, which also fixes the doubled liner.
3. Widen the depth and sideways ranges of the opening controls, and look at `mod_hp_pfha01`.
4. Fix the dent above the outer corner.
5. Choose how the game gets the closed shape: computed in the engine, or carried by the sliders.
6. Show the blink in kjk itself, with our head exported as a KOTOR model wearing the character's texture.

## Files

All scripts are in `~/.claude/tools/likeness-fit/scan/trials-2026-11-18/`, and all results in `~/.claude/tools/likeness-fit-data/kotor-heads/`.

| File | What it is |
|---|---|
| `kotor_eyes.py` | The 26 opening controls per eye, the push out of the eyeball, and the opening measures |
| `kotor_fit.py` | The fit. `KOTOR_RIGID=1` holds the target's scale. `KOTOR_SETS` picks the slider sets. The set "FLAME + openings" adds the opening controls and the pull on the lid edge (`KOTOR_EDGE`, 1 by default). `fit_flame` gives that fit to the other scripts when `KOTOR_OPENINGS=1` |
| `kotor_close.py` | The computed blink and its checks |
| `kotor_blink_sheet.py` | The textured sheet of KOTOR's blink next to ours |
| `kotor_blink.py`, `kotor_paint.py` | Use `fit_flame`, so they follow `KOTOR_OPENINGS` |
| `kotor_fit_v6.json`, `.png`, `.log` | The fit of the 57 heads with the opening controls, with each control's value per head |
| `close_v6.json`, `.png`, `.log` | The blink check on the base head and the 57 fits |
| `blink_sheet_v6.png` | The nine-head blink sheet |
