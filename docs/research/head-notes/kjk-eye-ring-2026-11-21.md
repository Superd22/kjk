# The skin round the eye curve, rebuilt (2026-11-21)

Written by the conversation kjk-6a, started from the brief `eye-ring` by Eyes HQ (kjk-48). My scratch folder is `/private/tmp/claude-501/-Users-david-perso-kjk/a18ead03-cc34-498d-89ad-8295e8c70b3e/scratchpad`, called `$M` below. The data folder is `L=~/.claude/tools/likeness-fit-data/sliders`. The handles are the prototype's fit, `$S/step2/handles.json` and `ranges.json`, where `$S` is the prototype's scratch folder `/private/tmp/claude-501/-Users-david-perso-kjk/d55dbdf6-456c-4c14-ac07-1a61b76e9013/scratchpad`.

Nothing live changed. No preset, slider or live module was touched, and nothing is committed. Only the opt-in preview changed.

## Verdict

**What still looks broken, first.**
- **Outer corner height at its high end (5.43)** still leaves a diagonal fold running down from the outer tip, open and closed. It is clearest on Kreia (`$M/final2/ends/e_p_kreiah.png`, row 9). Kreia's fit is -3.45, so this end moves her tip 9 mm. On the other seven heads it is a faint line.
- **Outer corner place at its high end (26.74)** puts the tip about 125 degrees from straight ahead, behind the eyeball's equator. The tip is reached, but a small bright spot shows there in the temple close-ups. It is the lower lid's edge turning toward the light, not a tear.
- **A thin pale edge** now runs along both lids, about 0.8 mm wide. It comes from pinning the first layer of skin outside the rim (see the method). It reads like KOTOR's own pale lower-lid band, and the live head has a similar band. David may still want it thinner.
- **The ring measure against KOTOR did not improve.** It is better on Zherron (1.99 to 1.71) and pmha01 (1.15 to 1.07), about the same on pmhc04, Carth and Jolee, and worse on Bastila (0.82 to 1.08), Kreia (1.52 to 1.66) and pfha05 (6.60 to 7.21). The new skin is smoother, but it is not yet shaped toward KOTOR's. That is the refit's job, or a later ring step.
- **The fold check** still counts 4 to 43 folded triangles per head at the fit (live has 0 to 2), and up to 73 at Kreia's worst end. In the pictures I could not find them. They are mostly the thin rim band where it meets the pinned layer.

**What is fixed**, seen in the pictures:
- On all 8 heads at the fitted values: no alcove, no step round the eye, no spike or flap on the nose bridge, no notch between the brows, and no crease on the cheek. The old ring showed a dark wedge or slit from each inner corner toward the nose on pmhc04, Bastila, Kreia and pfha05. It is gone.
- At every handle end: the slabs beside the nose (inner corner place high), the stair under the inner corner (inner corner height low), the ledges of the flattest arches and the cracked inner tip are gone.
- mod_exile_canon's flap across the opening is gone, on its handles and on KOTOR's own curve.
- The closed lid's lower edge is smooth. It was jagged.
- The skin no longer hangs over the opening near the tips, so the opening follows the handles' curve.
- The nose-side view of `cproto` looks from outside the head.

## The method and why

The old ring moved each vertex along a ray in the eyeball's flat map, took its height from a sampled live surface near the curve, and capped the slope at 1.5 mm per mm. The cap let go 9 to 15 mm out, which made the step and the alcove. The sampled surface jumps between skin layers beside the nose, which made the spike and the notch.

The new ring (`build_eye` in `crates/character_customisation/head/head_eye_curve/src/curve.rs`):
1. The rim (the third layer out from the lid loop) is laid on the curve at 0.15 mm, and the layers under it go back against the eyeball, as before.
2. **The rim's move spreads over the region round the eye as one smooth field.** The region is every vertex within 22 mm of the live or the new rim. The field is the biharmonic move in 3D, with mean value weights from the live mesh. It equals the rim's move at the rim and is zero at the region's edge and beyond. Because the Laplacian is also held small on the first ring outside the region, the move fades out with no step and no kink. It is solved by conjugate gradients in `smooth_move`, in about 5 ms per head in release.
3. **The first layer outside the rim is pinned**: it sits 0.02 radians outward of its rim neighbours' places on the curve and 0.8 mm higher. Without this, the front of the live lid, which overhangs its rim as seen from the eyeball's centre, hung over the opening and hid the tips.
4. **A height floor near the curve**: within 1.5 mm of the curve, the skin rises at least 0.5 mm per mm, by a smooth maximum 0.3 mm wide. Further out the floor is level. A floor that kept rising made a crease under the eye in viewer 2, so it stops at 1.5 mm.

Why this method:
- A move in 3D keeps the skin's own shape. The nose and the brow translate with the moved skin instead of being rebuilt from a sampled surface, so nothing jumps between layers.
- A biharmonic move blends with a continuous slope into the face that does not move, which is what "no step" needs.
- The skin pulls in from as far as 22 mm when the curve moves out, because the whole region moves.
- I tried a move in the flat map's angles first. Skin high above the eyeball beside the nose then moved much further than the lid, and it made a wall. A move in millimetres does not.
- I tried holding every vertex that landed inside the opening at the opening's edge. That piled the layers onto one line and made the stair slabs. Margins that grow with each layer gave 2 to 4 times more folds. Pinning the one layer next to the rim was the fix.

## Test 1, at the fitted values

The sheets are made by the new command `cring` (in `$M/diag`, my copy of the shape tool). Each sheet has the rows KOTOR, LIVE, OLD RING (the first preview) and NEW RING. Each row is drawn in the key light and then in a raking light from above.
- `$M/final2/fit/r_<head>.png`: both eyes from the front, three-quarter from each side seen from 15 degrees above, the nose bridge from 25 degrees above, and the profile.
- `$M/final2/fit/r_<head>_close.png`: one eye at 14 px per mm from the front, from above, three-quarter from the temple and from the nose (both from above), and the inner corner from above.
- `$M/final2/closed/`: the same sheets with the lids shut.
- `$M/final2/fit/p_<head>.png`: `cproto`'s sheet (front, temple, nose and profile, against KOTOR and live), with the nose view fixed.
- `$M/final/fit/pair0.png` to `pair3.png` put two heads side by side (these are from just before the floor limit; the floor only changes the skin within 5 mm under the eye).
- In viewer 2's light, pmhc04 three-quarter from above: `$M/final2/viewer2/pmhc04_new_3q_above.png`, next to `pmhc04_live_3q_above.png` and `pmhc04_kotor_3q_above.png`. The crease that the first floor made is in `pmhc04_v3_crease_under_eye_fixed_since.png`.

Result: clean on all 8 heads. The fold counts at the fit fell from 66 to 162 per head to 4 to 43. Measures (`python3 $S/summ.py $M/final2/fit/p_m.json`, old ring in `$M/final/old_m.json`):

| Head | Curve against the fed curve, old then new (mm) | Ring against KOTOR, old then new | Folds, old then new | Bare eyeball when shut, front (mm²), old then new |
|---|---|---|---|---|
| pmhc04 | 0.12, 0.13 | 1.35, 1.40 | 97, 6 | 0.23, 0.26 |
| p_bastilah | 0.15, 0.12 | 0.82, 1.08 | 102, 25 | 0.89, 0.16 |
| pmha01 | 0.18, 0.16 | 1.15, 1.07 | 94, 19 | 0.36, 0.14 |
| p_kreiah | 0.12, 0.13 | 1.52, 1.66 | 157, 43 | 0.25, 0.08 |
| p_carthh | 0.15, 0.12 | 0.99, 1.04 | 88, 19 | 0.17, 0.12 |
| n_zherronh | 0.11, 0.09 | 1.99, 1.71 | 66, 4 | 0.11, 0.12 |
| pfha05 | 0.34, 0.33 | 6.60, 7.21 | 95, 16 | 0.46, 0.52 |
| p_joleeh | 0.31, 0.30 | 1.46, 1.44 | 162, 11 | 0.82, 0.20 |

## Test 2, every handle at both ends

`cends` writes `$M/final2/ends/e_<head>.png` and `.json` for the 8 heads. The first row is the fit, then each handle at its low and its high end. The columns are the front and the temple three-quarter, open and closed, then the inner corner from the front and the outer corner from the temple side, open and closed.

Result: clean, with no slab, flap, ledge, crack or tear, except the two outer corner ends listed at the top. The worst fold count per head over all ends fell from 165 to 299 to 23 to 73.

## The viewer 2 preview

- URL: http://127.0.0.1:8765/viewer2/?head=pmhc04&eyes=shape&hair=none&eyecurve=1
- The crate's module is rebuilt into `~/.claude/tools/likeness-fit/hq/wasm/head_eye_curve/`, version **"4"**. The only change in viewer 2 is `CURVE_MODULE_VER` "2" to "4" in `hq/viewer2/sliders.js`, then `restage.sh`. Logged in the shared change log at 18:15.
- The preview stays opt-in. With it off, the head is unchanged: on pmhc04, Bastila, pmha01, Kreia and Jolee, turning the preview on changes the positions, and turning it off gives back exactly the same positions (a hash of every position). I checked this in Chrome.
- The crate matches my prototype within 0.0001 mm on 5 heads (`eye_curve_check`). A build takes 4 to 10 ms.

## How to undo

Copies of everything I replaced are in `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-eye-ring-backup-2026-11-21/`:
- `crate/src` and `crate/Cargo.toml`: copy them back over `crates/character_customisation/head/head_eye_curve/`.
- `wasm/`: copy `head_eye_curve.js` and `head_eye_curve_bg.wasm` back into `hq/wasm/head_eye_curve/`.
- `viewer2/sliders.js`: copy it back into `hq/viewer2/`, then run `hq/viewer2/restage.sh`. Or set `CURVE_MODULE_VER` to "5" with the old module, so browsers drop the cached one.

## Traps

- The prototype's old command wrote back only the vertices within 15 mm, while the new field moves 22 mm. That left a 0.7 mm step at 15 mm until I fixed it. Any copy of the ring must write back the whole region.
- Holding vertices at the opening's edge makes stairs. Pin the one layer next to the rim instead.
- A height floor that keeps rising with distance makes a crease under the eye that only shows in viewer 2's light, not in the clay sheets. Look in viewer 2.
- A move in the flat map's angles makes walls beside the nose. Move in millimetres.
- The fold check counts against the live head's lid. The new lid's edge differs from live, so the count stays above live even where the pictures are clean. Judge by the pictures.
- In the viewer, a JavaScript call that returns a URL is blocked by the browser tool. Return booleans.
- In zsh, use `${=heads}` to split a list.

## What the refit conversation needs to know

- Build with `CC_FLOORW=1.5` in my diag tool, or with the crate, to get what viewer 2 shows. My diag's defaults without that variable keep the floor rising; the crate has it built in.
- The commands: `cring $L <prefix> <heads>` (with `CC_HANDLES`, `CC_SET`, `CC_CLOSED`, `CC_TINT`), `cends`, `cproto` (`CC_RING=0` gives the old ring), `cdump $L <dir> <heads>` for `eye_curve_check`. The binary is `$M/diag/target/release/pmhc_diag`; the source is also in `$M/final2/proto_curve.rs` and `proto_main.rs`.
- The sizes are constants at the top of `curve.rs`: `SPREAD` 22 mm, `PIN_STEP` 0.02 rad, `PIN_RISE` 0.8 mm, `RISE` 0.5, `SOFT` 0.3 mm, `FLOOR` 1.5 mm. The pale lid edge comes from `PIN_RISE`; 0.15 left dark specks on the lower lid, and 0.4 left a crease above the upper lid.
- The ring does not pull the skin toward KOTOR's shape. It keeps the live face and blends it to the curve. If the refit wants the skin round the eye to match KOTOR (the ring measure), that needs sliders for it, or a target in the field.
- Outer corner height high and outer corner place high are the weak ends. If the refit lands near them, check the outer corner close-ups.
