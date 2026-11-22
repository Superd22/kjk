# Head editor sub-systems plan

This note records the plan agreed on 2026-11-17 for kjk's character head editor. It also records the test results that led to the plan, so the plan can be picked up in a later session.

The plan splits the head editor into sub-systems. A **sub-system** is one part of the editor that shapes one region of the head with its own controls and its own rules. Each sub-system is built and scored on its own against a large set of 3D heads. The sub-systems are then checked to merge well.

Three earlier notes in this folder cover the background.

- The likeness note (`docs/research/2026-11-16-likeness-recreation.md`) explains what people use to recognise a face.
- The sculpting note (`docs/research/2026-11-16-character-sculpting.md`) explains how a drag on the head becomes a shape change.
- The curve note (`docs/research/2026-11-17-curve-based-face-editing.md`) covers curve editing in games, film tools and research.

## Terms used in this note

A **scan** is a 3D model of a real person's head, made with cameras or a laser.

The **free head** is our own mesh wrapped onto a scan with every vertex free to move. It is the closest our mesh can get to that person.

The **distance** is the mean gap between our head's surface and the scan's surface, in millimetres. The skull is left out, because scans carry hair. Where stated, the ears are left out too.

A **curve** is a smooth line on the skin. A **control point** is a point on a curve that the player can move. The skin between the curves follows them through a smooth fill.

The **ICT shapes** are the 100 identity shapes of USC's ICT FaceKit, which has an MIT licence. They were learned from scans of real people.

## What the tests showed

All tests used the same twelve scans. They are the Lincoln life mask from the Smithsonian, the Lee Perry-Smith head, and ten people from Meta's Multiface set.

| Head | Distance left |
|---|---|
| Average head, nothing moved | 1.52 mm |
| 194 MakeHuman sliders | 1.02 mm |
| 100 ICT shapes | 0.99 mm |
| MakeHuman sliders and ICT shapes together | 0.85 mm, and the face becomes lumpy |
| 19 curves with 121 control points, on the average head | 0.86 mm |
| 29 curves with 193 control points, on the average head | 0.80 mm |
| 29 curves with every point of each line exact | 0.59 mm |
| 100 FLAME shapes | 0.91 mm |
| 300 FLAME shapes with a weak prior | 0.72 mm, with about 80 weights at their limit |
| 300 FLAME shapes with a strong prior | 0.82 mm, and the faces stay clean |
| Free head | 0.15 mm |

The rows for the sliders and the ICT shapes include the ears. The rows for the curves leave them out.

Seven findings shape the plan.

1. The mesh is not the limit. The free head lies 0.15 mm from the scans.
2. About 1 mm is what a hundred or so fixed shapes reach on a new person. More range does not help.
3. The ICT shapes give clean faces, but every fit drifts toward one young, slim face.
4. Curves on the clean average head look saner than curves on top of the ICT fit. The ICT fit brings its own oddities, such as staring eyes.
5. Curves that follow the mesh's edges and curves that cut smoothly across its faces score the same with the same control points. Smooth curves are still the right design, because they are better lines to follow.
6. One smooth skin with curves on it breaks at the lips. Three lip curves sit within 5 mm of each other, and the lips tore open on 2 of the 12 people.
7. The targets were not clean. The scans carry hair, moustaches and stubble as part of the surface, and their eyes are closed or masked.

The **FLAME shapes** are the 300 identity shapes of FLAME 2023 Open from the Max Planck Institute, learned from about 3,800 scanned heads. Their licence is CC BY 4.0, so shapes derived from them can ship with attribution. The licence bans pornographic use and fake or defamatory content. The FLAME rows in the table include the ears. A **prior** here is how strongly each shape's weight is held near zero.

The FLAME shapes were tested after the plan was written. They are the first fixed shapes that make heavy faces and softer female faces, which the ICT shapes could not. They are a candidate for the layer under the sub-systems.

The tests measure capacity, which is how close the controls can get when every control is set perfectly. They do not show likeness. The scans are anonymous people, and every head was rendered with the same eyes, eyebrows and skin.

## The plan

### Step 1. The benchmark

Every sub-system is scored on the same set of heads, so the benchmark comes first.

- The set needs a few hundred heads. It must include old, heavy and female faces, which the twelve scans lack.
- Each head gets masks for hair, facial hair and closed eyes. A sub-system is never scored on a masked area.
- Each sub-system is scored only inside its own region, with a measure that fits it.
- Each score is compared with three known results on the same heads. These are the 194 sliders, the 100 ICT shapes and the free head.

The dataset is being chosen. The candidates and their licences are in `docs/research/2026-11-17-head-scan-datasets.md`.

One rule applies to every dataset. kjk's own MIT licence does not change what a dataset's licence allows. Measuring our editor against a dataset is one use. Shipping shapes that were learned from a dataset is a different use, and many research datasets forbid it. Only data whose licence allows it, such as the ICT shapes, may end up in the repository.

### Step 2. The contract between sub-systems

The contract is fixed before any part is built, because it is what makes the parts merge.

1. Each sub-system owns a region of the head. The regions are the lips, each eye, the nose tip with the wings, the ears, and the skull. The broad curve net owns everything else.
2. The broad curve net goes first. It sets the jaw, the cheeks, the chin, the brow and the forehead. It carries the regions of the other parts along with it, so a wider jaw moves the mouth with it.
3. Each part then shapes its own region on top. Its effect fades to nothing across a narrow band at the edge of its region, so no seam can appear.
4. Each part keeps its own promises. The lips never cross. The eyelids stay on the eyeball. The nostrils stay open.

### Step 3. The lips, as the pilot

The lips come first. They are where the general curve net breaks, and a lip outline can be traced from a photo.

- The controls are the upper edge with its two peaks and its dip, the lower edge, the line between the lips, and the two corners. They form one linked shape. Each lip also has its fullness, which is how far it rolls forward.
- The expected size is 25 to 30 numbers with the two sides mirrored.
- The scores to beat on the mouth zone are 1.09 mm for the ICT shapes, 0.68 mm for the general curve net, and 0.41 mm for perfect lines.
- A second test sets the lips from an outline traced on a front photo.

### Result of the lips pilot

The lips need two layers. The **paint layer** draws the upper edge and the lower edge of the lips as colour. The **shape layer** sets where the line between the lips sits and how far each lip rolls forward.

Until this test every head carried the same painted lips, so all mouths looked alike. Moving the skin until that fixed paint matched a photo's lip outline gave a raised muzzle and dark smears. Painting the lips from 18 numbers gave clean mouths with the person's lip character. The 18 numbers are the height of the upper lip and of the lower lip at 9 places between the mouth corners. The paint is drawn from the head's own line between the lips, so it follows the head.

The test used 5 front photos, which is too few to score finely. The base layer still places the mouth 2 to 3 mm too low on those photos. Estimating the head's tilt did not fix that. The three photos with a slight smile are the worst, so expression is the likely cause.

The same split into paint and shape probably applies to the eyebrows and to the eyes.

### Step 4. The other parts

| Order | Part | Note |
|---|---|---|
| 1 | Lips | The pilot. |
| 2 | Broad curve net | Mostly built. The lip, eye and nose curves must be taken out of it. |
| 3 | Nose tip and wings | Can be scored on the scans we have. |
| 4 | Eyes | Needs heads with open eyes, or photos. The current scans cannot score it. |
| 5 | Ears and skull | Sliders are probably enough. |

Once the benchmark and the contract exist, the parts no longer depend on each other. Several agents can then build them in parallel.

### Step 5. The merge test

All parts are applied together on every benchmark head. Three things are checked.

1. The whole-face score is at least as good as the parts' own scores promised.
2. No seam is visible at the edge of any region in the Likeness Lab renderer.
3. Extreme settings of one part do not break a neighbouring part.

## What was built

The parts that were built, with every control, its range, its score and its faults, are documented in `docs/research/2026-11-17-head-parts-and-controls.md`. That note is the current reference. The steps above are kept as the record of the plan.

## Decisions already taken

- There will be no creator that blends presets, and no photo projection onto the texture.
- Eyebrows are painted into the texture.
- Many controls are acceptable. Some can be hidden by default.
- Scripts must work on any photo or scan. They are never tuned to one person.
- Hair caps must fit all 14 headgear items.

## Open questions

- Which dataset becomes the benchmark, and what its licence allows.
- Whether the base head is ICT's neutral head carried onto our mesh, or the MakeHuman average. The curve tests used ICT's neutral head.
- How a player sets the curves that no photo shows, such as the cheek slices. The photo-only test reached 1.26 mm from 1.52 mm.
- How facial animation behaves on a head whose lips were reshaped.

## Where the work lives

The work is outside the repository.

- The scripts are in `~/.claude/tools/likeness-fit/scan/`. The curve tests are `curves.py` and `curves_smooth.py`.
- The scans and all results are in `~/.claude/tools/likeness-fit-data/`. The curve sheets are in its `scans/curves/` folder.
- The viewer of the curve net is at https://claude.ai/artifact/LqW3Jd9HQ49WsXTMcx3DZr.
- The Likeness Lab page is at https://claude.ai/artifact/W9xpoEuYARkRqFDGXDazoA. It has not been changed by this work.
