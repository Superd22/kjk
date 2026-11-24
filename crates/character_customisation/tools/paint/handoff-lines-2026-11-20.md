# Handoff: from the fold layer to the line generator (forehead, frown, mouth corner lines)

Written on 2026-11-19 by the fold agent, for a fresh agent building the line generator. You have the lead's brief and the roadmap. This
note holds what you cannot find by looking.

Folder names used below:
- CRATE is `/Users/david/perso/kjk/.claude/worktrees/head-paint/crates/character_customisation/head/head_paint`
- TOOLS is `/Users/david/.claude/tools/likeness-fit`
- DATA is `/Users/david/.claude/tools/likeness-fit-data`
- SCRATCH is `/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad` (read only; write nothing there)

## State when I stopped

- The crate builds (`cargo build --release -p head_paint --bins`), and all 97 library tests pass.
- I did not start a line module. Nothing of the line generator exists.
- `CRATE/src/folds.rs` compiles with 6 tests. The folds are off by default: `Folds::NONE`, and `paint()` paints no folds.

## Building and running

- cargo is not on the PATH. Prefix every command with
  `export PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH";`
- Run cargo from the worktree root `/Users/david/perso/kjk/.claude/worktrees/head-paint`, with `-p head_paint`:
  - build: `cargo build --release -p head_paint --bin <name>`
  - tests: `cargo test --release -p head_paint --lib <module>`
- Ignore the warnings `rust-objcopy ... libLLVM.dylib` and "stripping debug info failed". They are harmless.
- `Cargo.toml` lists some bins explicitly, but cargo still picks up every new `src/bin/*.rs` on its own. Do not edit `Cargo.toml`, which belongs to the live page agent.
- Shared tool code lives in `CRATE/src/bin/shared/*.rs` and is included with
  `#[path = "shared/x.rs"] mod x;`. The include order matters, because each module names the ones before it as `crate::...`:
  1. `bundle`
  2. `fold_common`
  3. `fold_measure`
  4. `fold_paint`
- Python runs as `TOOLS/.venv/bin/python -I`. The system python3 has no numpy.

## Inputs, and where they live

- **Layout bundle.** `SCRATCH/build-smooth-default/prep/common/` (`bundle.json` and `bundle.bin`). It holds:
  - `uv`: 5,716 layout vertices;
  - `tris`: the layout's triangles;
  - `vmap`: the mesh vertex of each layout vertex;
  - `old_tris`: the mesh's triangles, in another order than `tris`, so never pair them by index;
  - `base`: the base head, 4,750 by 3, in metres.
  Read it with `fold_common::Common::load()`.
- **Fitted heads.** `SCRATCH/build-smooth-default/prep/heads/<name>/`. They hold:
  - `fitted`: positions in metres, in a frame with x lateral (the layout's left is x < 0), y up and z forward;
  - KOTOR's own head in the same frame: `kv` and `kf`, with `kuv` per face corner (read the texture at v as `1 - v`), `kmat`, and `khair` (skip hair faces);
  - the KOTOR texture paths, in the manifest's `textures`.
  Read the positions with `Common::fitted(name)`.
- **KOTOR bakes in our layout.** `SCRATCH/viewer-data/tex/v2/<name>_ours.jpg`, 2,048 pixels. Unpainted pixels are grey (123) or black. One pixel is 0.11 to 0.14 mm on the face.
- **The head list.** `fold_common::head_names()` (57 heads), read from `SCRATCH/build-smooth-default/fit_shading.json`.
- **Our current recipe per head.** `fold_paint::recipe_of(shading_fit, detail_fit, name)`, with the fits loaded by `recipe_fits()`. The detail fit is `SCRATCH/skin_detail/fit_all/fit_skin_detail.json`.
- `recipe_of` turns off the old wrinkle sets named in `fold_paint::REPLACED`: under_eye, nasolabial, marionette and chin. For the new lines, add `forehead`, `glabella` and the mouth-corner set to that rule. The wrinkle set ids are listed in `wrinkles.rs` `SETS`. Note that `marionette` is the old set for the lines that run down from the mouth corners.

## Pictures: front views of KOTOR's head and ours through the same window

Everything is in `CRATE/src/bin/shared/fold_common.rs`, in `KotorFront`:
- `KotorFront::open(name, across, up, forward)` builds KOTOR's own head with its own texture.
- `KotorFront::ours(&common, &positions, texture_rgb, across, up, forward)` builds our head with a painted texture.
- Both project onto the face's plane (across, up). The frame comes from the landmarks: `FoldHead.landmarks.frame`.
- Each pixel reads the frontmost face along forward.

The two views share a window as follows:
1. `q = ours.project(point_3d).0` gives the window's centre in the plane.
2. `tile(q, w_mm, h_mm)` renders it at 10 pixels per mm.
3. `tile_at(q, w, h, per_mm)` renders it at another scale, averaging 3 by 3 samples per pixel. For a whole face at conversation distance, use 110 by 120 mm at 4 pixels per mm.
4. `front_point(q)` gives the 3D point of our surface there. Use it to draw a layer alone on grey skin.

The fold windows (centre point, width, height) are in `shared/fold_measure.rs`, `tile_window(kind, head)`. A forehead window would be centred above the brows, about 90 by 40 mm.

The template tool is `CRATE/src/bin/fold_hand.rs`. For each head it:
1. reads hand-set sliders from `DATA/paint/folds/hand.json`;
2. paints the full texture with `paint_with_folds`;
3. writes KOTOR and ours through the fold's window, both whole faces, our face without folds, and the layer alone (`_field.png` and `_field_face.png`) to `TOOLS/hq/paint/folds/<fold>/hand/`.

It takes about 25 s for 5 heads on 5 threads. Painting one 2,048-pixel texture takes about 0.5 s.

The sheet glue is in `TOOLS/paint/`:
- `fold_hand_sheet.py <fold> <out.jpg> <names...>` makes one row per head: KOTOR's face, ours, KOTOR's window, ours, the layer alone.
- `fold_review.py` and `fold_compare.py` make sheets of KOTOR, ours and before.

Pages under `TOOLS/hq/paint/` are served at `http://127.0.0.1:8765/paint/...`. For example, `http://127.0.0.1:8765/paint/folds/index.html` answers. Put your pages in a new folder there.

Picture rules I learned the hard way:
- Judge from front views, never from layout crops. The layout's patch seams look like hard rectangles in a crop and are not on the head.
- Render the layer alone on grey skin (`_field.png`). Every geometry bug I had showed there first.

## Landmarks, paths and distance per pixel (reuse these)

All of this is in `CRATE/src/folds.rs`.

- **Anchors.** `ANCHORS` holds each landmark as three mesh vertices and their weights; `Anchors::point(&anchor, positions_m)` returns millimetres.
  - The landmarks are: the nose wing, the mouth corner, the chin middle, the lower lip middle, the eye inner and outer corners, the lower lid and the nose tip.
  - They were made by `CRATE/src/bin/fold_anchors.rs` from layout pixels on the base head (its `PICKS`), with the other side found by mirroring in 3D.
  - Add brow anchors (brow inner end, brow peak, brow outer end, forehead middle, hairline middle) the same way. Edit `PICKS`, run `fold_anchors <out dir> <names>` and paste the printed constants. It also draws the anchors on bakes so you can check them.
  - Trap: the old `pores::landmarks::EYE_INNER` pixel (746,790) lies on the nose bridge, not at the eye corner. Pick pixels from a gridded bake picture.
- **Frame.** `Landmarks::new(positions)` gives `frame.across` (from the x < 0 side to the x > 0 side), `up` and `forward`. `frame.out(side)` points away from the face's middle.
- **Surface queries.** `SurfaceIndex::new(layout_tris, per_layout_positions_m)` builds a 4 mm grid. `closest(p_mm)` returns the triangle, barycentric weights, point, normal and distance within 12 mm. `uv_of(hit, uv, tris)` gives the layout point. Build it per head through `FoldHead::new(layout_tris, vmap, positions)`.
- **Paths.** `PathShape` holds three `Control`s, each a `Mark` plus an offset outward and up in mm, and an `extend_mm`.
  - `Landmarks::controls` places the control points.
  - `SurfacePath::lay` draws a quadratic through them, lays each point on the surface with `closest`, resamples every 0.25 mm, and gives each point a normal and an `across` vector. The `across` vector is the cross of the normal, averaged over ±2 mm, with the tangent, flipped to lean outward.
  - `SurfacePath::at(s)` interpolates along the path.
- **Distance per pixel.** `FoldField::new(folds, head)`:
  1. builds the drawn polyline every 0.25 mm, shifted along `across` and re-laid with `closest`;
  2. registers each segment in a 3 mm `HashMap` grid out to the profile's reach.

  `lightness_shift(p_mm)` then:
  1. finds, for each line, the nearest segment: the distance, the signed distance (by `dot(offset, across)`), the share along the line, and the height off the skin (`dot(offset, normal)`);
  2. applies the cross-section, the taper along the line, a 15% `line_noise` variation along it, and a fade where the point stands 1.5 to 3.5 mm off the skin (the side of the nose wing).

  It uses no threads and is deterministic. For thin lines, copy this structure, or `wrinkles.rs` `WrinkleField`, which is the same idea.
- **Unit test pattern.** The `folds.rs` tests (`test_head`, `flat_face`, `place_landmarks`) build a flat 2.5 mm grid with the anchor vertices placed. They prove that moving landmark vertices moves the path, and that a 0.1 mm nudge moves it less than 0.2 mm. Reuse them for "a small change of the head gives a small change of the lines".
- **Flow.** `pores.rs` `PoreLandmarks::flow(p)` already gives a direction field: across the forehead above the eyes, radial around the eyes and down elsewhere. It is a starting point for lines grown along a flow.

## The hook in `paint.rs`

I did not add a field to `Recipe`, because `controls.rs` and `paint_live.rs` (the live page agent's files) build `Recipe` literals and would break.

What exists instead:
- `HeadDetail` has `pub folds: Option<FoldHead>`. `HeadDetail::new` builds it, and it is `None` on meshes without the anchors' vertices, such as small test meshes.
- `DetailPaint::with_folds(recipe, folds, layers, surface)` builds an optional `FoldField`, only when `layers.wrinkles` is on and `!folds.is_none()`.
- `wrinkle_shift` adds the folds' shift to the wrinkle sets.
- `paint_with_folds(recipe, folds, ...)` paints with folds. `paint()` calls it with `Folds::NONE`, so it is byte for byte as before.

Copy this pattern for the lines, for example with `paint_with_detail(recipe, folds, lines, ...)`. Add any new geometry to `HeadDetail` as an `Option`.

`CRATE/src/bin/fold_fit.rs` asserts on every head that sliders at zero paint the texture byte for byte as before. Keep a check like that.

Turning the old sets off: set `wrinkles.sets[s].strength = 0.0` for the replaced ids, as `fold_paint::recipe_of` does. `wrinkles.rs` itself stays unchanged.

## Traps I hit and what fixed them

1. **A surface normal beside the nose wing points sideways.** A shift along `across` then sent a fold into the face, and its start direction pointed backwards, so the whole fold was skipped. The fix was two things: normals averaged along the path, and the start direction taken from the unshifted path.
2. **Across directions taken in the face's front plane instead of on the surface** broke the measure on sloped cheeks. Keep surface-based across vectors.
3. **A wide falloff on the concave side of a curved line.** The nearest-segment point jumps from one part of the line to another there, which gives hard straight edges. Keep the reach well under the curve's radius on the concave side. Thin lines (0.3 to 1 mm) will not hit this, but wide shading bands will.
4. **Hard starts.** Do not cut points that lie "before the start". Let the taper fade both ends: a fade-in over 10% or 3 mm and a fade-out over 25% or 4 mm. Ends that fade over less than the line's width look like cut bars.
5. **Two halves of one line meeting in the middle** (the chin crease) doubled up. The fix weights each half by `smoothstep(-1, 1, a)`, where `a` is the distance along the half's start direction, and moves both halves straight down instead of along their own across vector. A forehead line that crosses the middle should be one path, not two halves.
6. **Results that sit at slider limits** were fits to the wrong thing, such as moustache edges. Reject them, and look at them.
7. **The bake is 0.3 to 1.7 mm off near the eyes.** Read KOTOR near the eyes on its own head (`KotorFront::lightness`), not on the bake. Frown lines sit between the brows, at the edge of that zone.
8. **Open question at the hand-off.** On old_a_m with hand-set folds, our whole-face front view showed a lighter "muzzle" patch with straight edges. Yet the painted texture's pixels in the middle of the upper lip were identical with and without folds, and the field was 0 there. So the patch is in the front-view rendering or in a region I did not sample, not in the field. Check `KotorFront::ours` and `tile_at` before trusting whole-face views.

## What I learned about KOTOR's painted creases

- **The hair mask.** `Lightness::hair()` in `fold_common.rs`:
  1. takes the energy of L* minus its blur over 5 pixels;
  2. averages it over 13 pixels;
  3. marks pixels above 4 times the face's median, then grows the mask by 6 pixels.
  It catches stubble, beards and brows, and also sharp lip and eye edges. Without it, beards and moustaches were read as folds.
- **Brows** will dominate any line measure on the forehead and between the brows. Mask them, for example with the hair mask.
- **Contrast.** "Darker than both sides" (the darker flank minus the core) rejects one-sided edges such as beards and lips. Measuring flanks at two scales (1.2 to 3 mm and 2.5 to 6 mm) lets broad soft creases count. Averaging along the line over ±1 mm kills short hair strokes.
- **Cross-sections.** These were measured over 43 to 82 sides, as medians, on the bake (512 to 1,024-pixel sources). Every fold is a soft dark valley with no light ridges, with half widths of 2 to 2.8 mm on each side. The nose-to-mouth fold is a step: the contrast against the lighter side is 6.8 L* against a depth of 4.0 L*.
- **Thin lines were not measured.** Forehead lines on Kreia and the old heads look like valleys about 1 mm wide. Check them on KOTOR's own head front view before choosing a cross-section.
- **Fine lines.** The old fit found that a line 0.18 mm wide keeps too little of its variance through a 1.5 to 24 pixel band to be measured. Lines must be at least about 0.5 mm wide to read at all.
- **The shading layer** already darkens real valleys. Under the chin it darkens 2 to 4.5 L* where KOTOR paints nothing, on 25 sides. The lead takes that up with the shading, so do not compensate inside a line layer.

## Where the four parked folds stand

1. **Generator.** `folds.rs` round 2 has 7 sliders per side: strength, highlight, width, edge, start, length and shift. The cross-section is a step: a dark outer side, a light band half as wide on the inner side, and a soft border (`step()`). The off-skin fade fixes the hard corner by the nose wing. The tests pass.
2. **Hand-set values.** The five-head values for the nose-to-mouth fold (strength 6 to 10, highlight 5 to 6, width 5, edge 1.2, length 32, shift 3 to 4) are in `DATA/paint/folds/hand.json`. The sheet `fold_hand_sheet.py` makes from them did not yet read as KOTOR's fold to my eye, and trap 8 was unresolved.
3. **Measure and fitter.** The measure (`shared/fold_measure.rs`) has no light-side reading and no mirror-based error bar. `fold_fit.rs` still fits the round 1 sliders, mapped onto the new ones with highlight 0. Round 1 results are in `DATA/paint/folds/fits.json`, `measure_kotor.json` and `placement.json`. The round 1 pages are at `http://127.0.0.1:8765/paint/folds/index.html`.
4. **Open from round 2.** Steps 2 to 4 of the lead's round 2 brief are untouched: the light-side measure, the error bar from mirrored textures, the refit of all 57 heads, and the other three folds.
