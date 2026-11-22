# Drawn and procedural hair

This note was researched on 2026-11-19 for kjk's head customiser.
It asks whether a player could draw a hairstyle, or shape one from a few controls, instead of picking one of a fixed set of hair meshes.
It also asks how such a hairstyle would be turned into the thing KOTOR can draw: a low-polygon mesh with a texture.

Terms used in this note.

- A **strand** is one hair, stored as a curve made of a few points.
- A **guide strand** is a strand that a person draws or a program places. Many other strands are then made by copying or blending the guides.
- A **wisp** is a bundle of neighbouring strands that run together, as hair does in real life. A **clump** is the same idea seen as an effect: strands pulled towards a shared centre.
- A **card** is a flat strip of polygons with a hair texture and transparency. A **shell** is a copy of the head surface pushed outward, which shows a slice of a hair texture.
- A **hair cap** is a thin mesh that follows the scalp and carries the hair that lies flat on the head.
- An **orientation field** (also called a vector field) gives, for every point in space or on the scalp, the direction in which hair grows there. A strand is made by starting at a root on the scalp and repeatedly stepping along the field.
- A **style curve** is a 3D curve drawn by the user that forces the field to point along it.
- The **scalp** is the part of the head surface where hair roots grow.

Every claim carries a link.
A sentence that starts with "I infer" or that says "probably" is my own conclusion and not a fact from a source.
A source that is not a paper, an official manual or a studio's own text is marked as a forum post, a mod page, a press article or a community wiki.

## Summary

- **A player can literally draw hair, and one shipped program does it.** VRoid Studio lets the user draw hair bundles with a brush on a guide mesh around the head. Each stroke becomes a ribbon of hair. A retouch brush redraws a bundle, and control points change its length and its wave. The same program also builds hair from a few sliders (hair count, cross-section, twist, length, thickness). VRoid's manual is the best primary source for the "draw it" idea ([VRoid Help, Hair Editor](https://vroid.pixiv.help/hc/en-us/articles/360012339194-Hair-Editor), [VRoid Help, How do I customize hairstyles](https://vroid.pixiv.help/hc/en-us/articles/900005678786-How-do-I-customize-hairstyles)).
- **The classic method for a drawn hairstyle needs only a few curves.** Fu, Wei, Tai and Quan (2007) let the user draw 3D style curves. A sparse linear system then fills a 3D grid with growth directions, and strands are traced from the scalp. The paper reports hairstyles made in 3 to 5 minutes from 4 to 11 stream curves plus a few other curves ([Fu et al., Sketching Hairstyles](https://hongbofu.people.ust.hk/doc/sbim07_hairstyle_sketching.pdf)). The method is small enough to write in Rust, and it can run on a coarse grid.
- **The neural methods are not a fit for kjk.** DeepSketchHair, SketchHairSalon, Perm and HAAR all need trained networks. Their training data comes from hairstyle collections gathered from online repositories and, for one of them, from the Sims modding scene. Only SketchHairSalon has public code, and its data is limited to non-commercial research ([SketchHairSalon repository](https://github.com/chufengxiao/SketchHairSalon)).
- **A strand hairstyle can be turned into a few cards automatically.** DiffHairCard (2025) and Strands2Cards (2025) cluster strands, fit one card to each cluster, and bake a texture atlas. DiffHairCard needs about 46 minutes for 100 cards on an RTX 3090 and 7 GB of memory ([Zheng et al.](https://arxiv.org/abs/2505.18805)). That is far too slow for a player's creator screen. The steps still show how to build a fast, simple version ourselves (see "Turning strands into cards and shells").
- **Hair that fits under headgear has one published recipe.** DreamWorks' Hair Smash stores, for each guide strand, its distance to the hat. It pushes strands that cross the hat towards the scalp and then forbids every strand from going further from its guide than that distance ([O'Hagan et al., SIGGRAPH 2015](https://history.siggraph.org/wp-content/uploads/2022/10/2015-Talks-O_Hagan_Hair-Smash.pdf)). Games mostly do something simpler. They swap in a short "hat" version of the hairstyle or hide the hair.
- **The recommendation** is a hair-envelope creator. The player draws or sets a few style curves and a few sliders on a guide mesh that sits inside the headgear volume. Our code fills an orientation field, grows guide strands, and bakes them into one hair cap, a handful of cards and one procedural strand texture. The details are in the last two sections.

## Sketch-based hair modelling

### What the research line looks like

Most sketch-based methods answer the same question. The user gives a few strokes, so how does the program fill in the thousands of strands that the strokes do not show?
There are four answers in the papers.

1. Solve a smooth field from the strokes, then grow strands in it (Fu 2007).
2. Treat the strokes as a query into a database of finished hairstyles, then blend the matches (Hu 2015, HairBrush 2019).
3. Train a neural network that maps strokes to a field (DeepSketchHair 2020) or to strands (StrandDesigner 2025).
4. Train a network that draws a picture of hair and never builds 3D hair (SketchHairSalon 2021).

### Table of the methods

| Method | What the user gives | What comes out | How sparse input is filled | Code |
|---|---|---|---|---|
| Wither, Bertails and Cani, 2007 | A side view of the head, with example strands drawn on it. A photo of real hair can also be annotated. | Strands with mechanical properties, an adjusted scalp shape and a hair volume | Physical hair model (super-helices) whose parameters are inferred from the strands | Not found |
| Fu, Wei, Tai and Quan, 2007 | 3D style curves drawn on the head: stream curves, dividing curves and ponytails | A 3D grid of directions, then tens of thousands of strands | A sparse linear system with the style curves as constraints | Not found |
| Hu, Ma, Luo and Li, 2015 | A photo and a few strokes over it | 10,000 strands | Retrieval from 686 example hairstyles, blended into one field | Not found |
| HairBrush, Xing et al., 2019 | Guide strokes drawn in virtual reality | Textured polygon strips, convertible to strands | A network suggests strips, each example hairstyle gives blend shapes, and the result is deformed | Not found |
| DeepSketchHair, Shen et al., 2020 | A hair outline and a few direction strokes on a bust | A 128 by 128 by 96 field, then strands | Three networks (S2ONet, O2VNet, V2VNet) | Not found |
| SketchHairSalon, Xiao et al., 2021 | Coloured strokes over a face image | A 512 by 512 hair picture and a matte | Two image networks | Public, license not stated |
| Perm, He et al., 2025 | Not sketches. It is a model of hair, driven by 512-number codes | Strands from a 256 by 256 scalp texture | Generator networks and PCA | Not linked by the paper |
| StrandDesigner, 2025 | Sketches | 3D strands | Latent diffusion with strand upsampling | Promised, not released |

### Fu et al., Sketching Hairstyles (2007)

This is the method I would implement. The paper is short and complete.

The system has four parts. They are a head mesh, a style-curve set, a vector field on a uniform 3D grid around the head, and the strands ([Fu et al.](https://hongbofu.people.ust.hk/doc/sbim07_hairstyle_sketching.pdf)).

**What the user draws.** The paper has three kinds of style curve.

- A **stream curve** shows the general flow. Every grid point near the curve is forced to point along the curve's tangent. To stop hair running past the end of the curve, the program adds a few extra segments at the end whose vectors shrink from unit length to zero.
- A **dividing curve** is drawn on the scalp and makes a parting. Hair on the two sides flows in opposite directions. In a local frame at each point of the curve, the constraints follow the tangents of a Gaussian that passes through the curve point. The Gaussian's two shape numbers, sigma and h, default to 1 and 0.5, in units of the grid step. A second set of vectors with a magnitude of 0.3 marks the discontinuity and stops hair from crossing the parting.
- A **ponytail** is a bundle of four curves: a spine, a cross-section and two profile curves. It adds vectors that point towards the centre of the first cross-section, with a magnitude of 0.6 by default, and that sets how tight the ponytail is.

**How depth is found.** A 2D stroke has no depth. The paper takes the depth from the scalp surface under the stroke, or from a "supporting surface" built around an existing curve and seen from the view in which it was drawn. The user edits a curve by drawing a hint stroke on that surface. The hint stroke is read as a cut, a concatenation or an insertion, depending on where it starts and ends.

**How the field is solved.** For each grid vertex *i* there is a direction vector *t_i*. The program minimises the sum over all vertices of the squared Laplacian of *t_i*, plus omega squared times the squared distance between each constrained vertex and its target direction. The Laplacian is the average over the 1-ring neighbours (the 6 neighbours in a cubic grid) of the neighbour minus the vertex. The constraint weight omega is 100. The paper turns this into normal equations (DᵀD + WᵀW) t = Aᵀb and solves each of the x, y and z components separately.

**Published numbers.**

- Low-resolution grids have about 25,000 unknowns and suit smooth hairstyles. High-resolution grids have about 50,000 unknowns and suit ponytails.
- A direct solve of 50,000 unknowns takes more than 20 seconds. The paper avoids that by changing the Cholesky factor incrementally when the constraint set changes, using the update and downdate method of Davis and Hager. A re-solve then takes "a few seconds". Setting up the system takes tens of seconds.
- Strand roots are sampled uniformly on the scalp with a little randomness. The scalp is parameterised by two angles, theta from 0 to 180 and phi from 0 to 360, and the hair density is set by the sampling steps of the two angles.
- Each strand grows in steps of 0.5 times the grid step. It stops when the field's magnitude falls below 0.05 or when the point leaves the bounding volume.
- The hairstyle in the paper's Figure 1 used 11 stream curves, 4 dividing curves and 1 ponytail, and it took 5 minutes. Other examples took 3 minutes.

**Scalp penetration.** A flow line can dive into the scalp. The paper first lets the user draw freely. Then, for every grid point near the scalp whose direction points inwards, it replaces the direction with its projection on the tangent plane of the nearest scalp point. It adds those points as soft constraints with a magnitude of 0.3. This is done once.

**Limits stated by the authors.** Smooth interpolation gives hair that "may lack local variations". They suggest adding curl and wave as a later step.

**Why it fits kjk.** It needs no data and no network. All inputs are curves, which a creator can show as handles. I infer that a coarse grid of about 24 cells per side (about 14,000 cells, or 41,000 unknowns over three components) could be solved with conjugate gradients in a few milliseconds to tens of milliseconds in Rust, so the incremental Cholesky update would not be needed. The paper does not say this. It is my estimate from its own sizes.

### Wither, Bertails and Cani, Realistic Hair from a Sketch (2007)

The user sketches example strands over a side view of a head, or annotates a side-view photo of real hair. The program infers each strand's geometric and mechanical properties, adjusts the scalp shape and builds a hair volume. The paper reports results from straight to curly and from short to long ([IP Paris abstract](https://researchportal.ip-paris.fr/en/publications/realistic-hair-from-a-sketch/), [HAL record](https://hal.archives-ouvertes.fr/inria-00337444)).
I could not download the full text, so I cannot give its equations or numbers. It is aimed at physics-based hair, which kjk does not need.

### Hu et al., Single-View Hair Modeling Using a Hairstyle Database (2015)

The input is one photo, a few strokes and a database of finished hairstyles. The database holds 343 hairstyles, collected from the Sims community sites The Sims Resource and Newsea, doubled to 686 by mirroring. Each hairstyle was a set of triangle meshes, and each mesh was a wisp. The authors resampled each mesh into strands, built a 3D orientation field from the strands, and grew 10,000 strands from the scalp ([Hu et al.](https://linjieluo.github.io/assets/pdf/15-SIG-SingleViewHairModelingUsingAHairstyleDatabase.pdf)).

The strokes are matched against the strands of each example. The best examples are combined into one orientation field with a Markov random field, which weighs the strokes and the local consistency of the examples. The strands are then refined by fitting piecewise helices to the photo's 2D directions.

The paper reports grids of 2 mm and, for interactive retrieval, 8 mm. A whole result took about 20 minutes. The 2D deformation step alone took about 10 minutes for 10,000 strands.

I draw two points from this paper. First, a hair mesh of the kind that games already use can be turned back into an orientation field, so a field can serve as a common middle format between cards and strands. Second, a blend of database hairstyles is a method that kjk has already decided against, because of the decision not to blend presets.

### HairBrush, Xing et al. (2019)

HairBrush is a virtual-reality tool in which the user draws high-level guide strips. A trained network suggests strips from sparse strokes. Each hairstyle in the dataset has variations that act as blend shapes, and the system fits them with a global blend and a local deformation. The user can accept, change or ignore the suggestions. The hair is stored as "textured polygonal strips" that "can be converted to strands". In the one example the page gives, a novice finished a hairstyle in 10 minutes with 382 suggested strips and 71 drawn ones ([USC ICT, HairBrush](https://vgl.ict.usc.edu/Research/HairBrush)).
The page names no dataset size and no code. This is the closest research system to what a card-based game needs, because its output is already cards.

### DeepSketchHair, Shen et al. (2020)

The user draws a hair outline and a few strokes that show the growth direction, over a 3D bust. Three networks follow ([Shen et al.](https://arxiv.org/abs/1908.07198), full text read).

- **S2ONet** turns the sketch into a dense 2D orientation map at 128 by 128 pixels.
- **O2VNet** turns the 2D map into a 3D orientation field of 128 by 128 by 96 cells.
- **V2VNet** updates the 3D field when the user draws from a new view.

Strands are then grown from the field. The training set has 653 strand-level hairstyles, taken partly from public online repositories and partly from another paper's authors, all aligned to one bust. Training took about one week on four GTX 1080 Ti cards.
In the pilot study, five novices drew for about 30 seconds, the networks took 0.2, 0.3 and 0.2 seconds, and strand synthesis took about 5 seconds. The users found the 2D input limiting when they wanted to change hair in 3D. I found no code link.

### SketchHairSalon, Xiao et al. (2021)

SketchHairSalon produces a picture, not 3D hair. Its first network predicts a hair matte from a sketch. Its second network draws the hair image from the sketch and the matte, at 512 by 512 pixels. It includes tools that complete repeated strokes and braids ([Xiao et al.](https://arxiv.org/abs/2109.07874)).
The repository holds code, trained models and a dataset of thousands of annotated sketch and image pairs. The dataset is "available for non-commercial research purposes only". The README names no software license, and it asks for Python 3.6.12 and torch 1.0.0 ([repository](https://github.com/chufengxiao/SketchHairSalon)).
Its use for kjk is small. It could in theory paint a hair texture from strokes, but it was trained on photos of real hair, which do not look like KOTOR's hand-painted textures.

### Neural strand models, Perm, HAAR and StrandDesigner

These do not take drawn strokes in a form that suits a creator. I list them because they show a useful data layout, the scalp texture.

- **Perm** stores a hairstyle as a 256 by 256 texture over the scalp, where each texel holds a whole strand of 100 points as 64 PCA coefficients. The first 10 coefficients give the global shape (the "guide"). The other 54 give the curl. The guide texture is only 32 by 32, so it holds about 400 guide strands. A small network upsamples it to 256 by 256 by blending the 4 nearest guides ([Perm](https://arxiv.org/abs/2407.19451)). Cutting the PCA coefficients from 64 to 10 smooths a hairstyle. This is a slider-like control. The paper trained on 343 original hairstyles (augmented to 21,054 samples) and a private curly set, and I found no code link.
- **HAAR** is a latent diffusion model in a shared scalp UV space. It generates guide strands from text and then upsamples to up to about 100,000 strands. It was trained on synthetic hairstyles made from a small set of artist-made ones ([HAAR](https://arxiv.org/abs/2312.11666), [project page](https://haar.is.tue.mpg.de/)).
- **StrandDesigner** is a sketch-conditioned strand generator. Its code is promised and not yet released ([StrandDesigner](https://arxiv.org/abs/2508.01650)).

I infer from the Perm layout that a hand-built version, with no network, is possible. We could store a hairstyle as a low-resolution scalp texture of guide strands and as a few shape numbers for curl. That is what the next section builds.

## Procedural hair from a few parameters

### Guide strands with interpolation and clumping

The usual production method is this. Place a few thousand guide strands. For every other strand, pick a root on the scalp, find the nearest guides and blend their shapes. Then add clumping and noise.
Epic's MetaHuman hairstyle generator is the best documented example of this method with named controls ([Epic, MH Groom Hairstyle Generator](https://dev.epicgames.com/documentation/metahuman/mh-groom-hairstyle-generator)). It runs inside Houdini and is not a runtime tool. Its page lists these groups of controls, without ranges.

- **Hairline.** Forehead style, shape and height, corner and temple positions. Sideburn style, length and height. Nape style, width and height.
- **Parting and regions.** Part line position and rotation, with a noise amplitude and frequency so that the line is not straight. The sizes of the top, bangs and back regions, with a blur between regions.
- **Follicles.** A global density ratio, a seed, a per-region density, and balding at the forehead and crown (influence, position, size, noise).
- **Guides.** A guide density. Master controls for length, volume and variation for long and short hair. Regional orientation: parting, pulled back or pulled forward. A bangs variant. Blending across the parting.
- **Crops and fades.** A planar crop with a height and two rotation angles. Mohawk, side fades and hairline fades with a length and falloff.
- **Guide styling.** Tufts (density, thickness), curls (size, frequency, position), roots volume and tips bend.
- **Hair styling.** Clumping (influence, size, noise, seed), hair noise (amount, frequency) and a coil mode with four levels (none, soft, normal, strong).
- **Look.** A root-to-tip colour gradient, width with randomness and tip thinning.
- **A lattice** that deforms the guides after the fact.

Epic's styling tools page says that strands are then made "by interpolating from the guides", and lists clumping at four levels (guide, primary, secondary, tertiary) and frizz, flyaways, spread or lift, and length ([Epic, Styling tools](https://dev.epicgames.com/documentation/metahuman/styling-tools)). The pages give no algorithm and no numbers for any of it.

For kjk this list is a ready catalogue of sliders. Its hairline, parting, length, volume, curl and bangs controls are close to the "length, part, volume, curl and fringe" that the user asked about.

### Wisps and generalized cylinders

A **generalized cylinder** is a tube whose centre line is a curve and whose cross-section can change along it. A wisp is often modelled as one.
Choe and Ko's statistical wisp model builds a wisp from one master strand and many member strands ([Choe and Ko 2005](https://dblp.org/rec/journals/tvcg/ChoeK05); the method is described in the authors' patent, [US 7,418,371 B2](https://patentimages.storage.googleapis.com/5a/f5/f5/8adc093fa566e4/US7418371B2.pdf)). I read the patent text for the details below.

- A strand is a Catmull-Rom spline through control points p1 to pn. The spline parameter s runs from 0 at the root to 1 at the tip.
- A wisp has three parameters. The **length distribution** l(u) is a probability density of a member strand's length relative to the master strand. The **deviation radius function** r(s) is the largest distance a member strand may stray from the master at parameter s. It sets the shape of the generalized cylinder. The **fuzziness** rho runs from 0.1 (low variance) to 1 (high variance) in the patent's figures.
- A member strand copies each control point of the master and adds a displacement d_k. Each d_k is computed from the previous one and a random noise vector that stays inside a sphere of radius r_k. The patent draws the noise length uniformly in the range 0 to the smaller of the distance to the sphere and rho times r_k, so that points do not pile up on the sphere's surface.
- The master strands share one pattern. A new master strand is made from a prototype strand with an example-based Markov chain. The strand is split into an outline (straight segments joined by 3-axis joints) and a details part. The details are copied window by window from the prototype, with a Gibbs distribution whose temperature the user controls.
- The roots of the wisps lie on the scalp. The roots of member strands are random inside the Voronoi cell of their wisp.
- Styling rotates the joints of the outline to line up with a force field. The joint angle maximises E_phi plus kappa times E_B, where E_phi is how well the segment lines up with the force field and E_B is how well it lines up with its rest direction. Constraint fields (a point, a trajectory or a direction) are mixed into the force field with a weight.
- Hair collision is checked per wisp segment. Each segment adds a density to a field. If the density at a point plus the new density passes a threshold, the segment is turned away along the gradient.

The patent shows that a few wisp parameters give many looks. I flag that it is a patent. It was granted in 2008. I did not find out whether it is still in force.

Kim and Neumann's multiresolution hair modelling (2002) rests on the observation that neighbouring strands form clusters at several scales. The designer starts with a few clusters, subdivides them, and can edit at any level down to one strand. It has high-level curl, scale and copy-paste tools ([SIGGRAPH History Archive abstract](https://history.siggraph.org/?p=118452)). I could not open the full text, so I have no numbers from it.

### Hair as a scalp-anchored field or a volume field

Fu et al. already gave the method (above). The older field methods are described in their related-work section. Hadap and Magnenat-Thalmann (2000) treated hair as streamlines of a fluid flowing around the head. Yu (2001) added vector-field primitives that are summed. Choe and Ko (2005) used several styling fields, one at a time. Fu et al. say that the field-summing methods need several minutes to redo the field after a change, which is why they chose a solved field.

A second use of a field is to keep it on the scalp surface only. I infer that for a flat-lying hair cap, a 2D field on the scalp's UV layout can replace the 3D grid. The strands then follow a 2D flow and the cap shape is a height field. No source covers this. It is a simplification of Fu's method for hair that lies on the skull.

### Hair meshes (Yuksel, Schaefer and Keyser, 2009)

A **hair mesh** is a layered mesh that encloses the hair volume ([Yuksel et al.](https://people.engr.tamu.edu/schaefer/research/hair.pdf)). It lets an artist sketch hair as a polygon surface. The method is the closest published match to "draw the envelope and fill it".

1. The root layer is the scalp mesh. Each face of the root layer becomes one bundle of strands.
2. The user extrudes faces to make layers. Each face of layer k has exactly one matching face on layer k plus 1, and the two form a prism. A strand stops at the last layer of its bundle.
3. To grow a strand, the program takes a root point in a root face and computes its mean value coordinates (barycentric weights for a polygon). It applies the same weights on every layer and joins the points with a C1 Catmull-Rom spline.
4. The artist moves only the outer, visible vertices. The inner vertices are placed by minimising the sum, over each layer step and each edge, of the squared difference between two neighbouring layer offsets, with the outer vertices held fixed. The paper solves this with conjugate gradients, starting from the current positions, so it converges in a few iterations.
5. Style is added afterwards. Noise and trigonometric functions move the strand points. The paper defines them in the strand's barycentric coordinates in the mesh, so a small change to the mesh does not change the style. The paper also shows hair meshes generating 200 wisp curves, which Choe and Ko's model then expands.

The paper's examples (buns, knots, a bun with a few minutes' work) take "as short as a couple of minutes". The paper's physics example used 5 layers and 150 vertices and ran at 92 frames per second on a 2.14 GHz Core 2 Duo ([Yuksel et al.](https://people.engr.tamu.edu/schaefer/research/hair.pdf)). Yuksel's page states that hair meshes are covered by US and international patents and asks that anyone who wants to use them commercially or in research contact him ([project page](https://cemyuksel.com/research/hairmeshes/)). That is a risk for kjk and has to be settled before copying the method.

### Envelopes grown from the scalp with a height field

I found no paper that describes a hair "envelope" given only as a height field over the scalp for hair on a game character. The nearest published ideas are these.

- Kim and Neumann's "thin shell volume" model (2000) is cited by Yuksel et al. as hair given a thickness as a surface. I could not open the paper.
- DiffHairCard builds a **hair cap** by taking the scalp faces next to the roots and extruding them by a small distance, and bakes the first part of each strand into it ([Zheng et al.](https://arxiv.org/abs/2505.18805)).
- Shell fur (Lengyel et al.) is already covered in `2026-11-18-painted-facial-hair.md`.

I infer that an envelope of this kind is easy to define for KOTOR. The cap is the scalp offset outward by a height value per vertex. The player's sliders change that height, and the same field sets the hair length for the cards that hang from it.

### Noise-driven curl and wave

Two sources give the recipe.

- Yuksel et al. move strand points with noise and sine functions, defined in a strand's own coordinate frame so that edits upstream do not shake the style ([Yuksel et al.](https://people.engr.tamu.edu/schaefer/research/hair.pdf)).
- Epic's generator gives curl size, curl frequency, a curl position along the strand, a coil mode with four levels, hair noise amount and frequency, and a "preserve tight tips" switch ([Epic](https://dev.epicgames.com/documentation/metahuman/mh-groom-hairstyle-generator)).

A simple version that a programmer can write is this. For a strand with a tangent frame (T, N, B) and length parameter s, add `amp(s) * (cos(2*pi*f*s + phase) * N + sin(2*pi*f*s + phase) * B)` to each point. The amplitude amp(s) can be a function of s, so curls are tight at the tips and loose at the roots. This formula is mine and is the standard helix. It is not copied from a paper.

### Parametric styles as a few numbers

VRoid's procedural hair groups are the best shipped example of hair as numbers. The user adds a "procedural hair" group and sets, among others, the hair count, the cross-section, the twist, the length, the thickness and the width. The group is a set of bundles in a fixed shape, and "you can change the hairstyle design at once by adjusting the parameters" ([VRoid Help, Hair Editor](https://vroid.pixiv.help/hc/en-us/articles/360012339194-Hair-Editor), [VRoid Help, customize hairstyles](https://vroid.pixiv.help/hc/en-us/articles/900005678786-How-do-I-customize-hairstyles)).
The manual's stray-hair guide shows the typical controls: a curve for the hair shape, "guide parameters" named height and offset, "length", "number of hairs", "control points" and "center (X)" ([VRoid Help, stray hair with procedural hair](https://vroid.pixiv.help/hc/en-us/articles/360013259773-How-to-model-stray-hair-Procedural-Hair-ver)).

Reallusion's Character Creator 4 takes a different route. Its hair is made of parts that the user swaps, such as the fringe, the back length and the ponytail type. Beards and brows can be fine-tuned with a mesh editor. A "hair patch" with a morphable scalp can soften the hairline or fill holes ([Reallusion, 3D hair and beard system](https://www.reallusion.com/Character-Creator/hair.html)). This is a library of pieces, not a generator.

## Turning strands into cards and shells

### Epic's hair card generator

Epic's tool is a dataflow graph with the nodes GetGroomAsset, GroomAssetToCollection, BuildCardSettings, GenerateCardsClumps, GenerateCardsGeometry, GenerateCardsTextures and CardAssetTerminal. The page calls it "Experimental" ([Epic, Hair Card Generator](https://dev.epicgames.com/documentation/en-us/unreal-engine/hair-card-generator-for-grooms-in-unreal-engine)).

- The input is a groom exported from a DCC tool as an Alembic file, with card groups set up.
- For each level of detail, the user sets the number of clumps, the number of flyaways, the number of triangles and the number of textures. A multi-card clump uses "a group of three hair cards that represent a volume of hair". A flyaway is a single-strand card.
- Triangles are shared among card groups by each card's strand count and coverage.
- The user can turn on adaptive subdivision, or set a maximum number of vertical segments. The page's example uses 2 for a buzz cut. A texture compression option shortens the texture along the strand direction.
- The page gives no default counts and no texture sizes.

### SideFX Houdini

The Hair Card Generate node groups nearby curves into clusters and makes one polygon card per cluster, with a width and orientation that match the curves. A second tool, Hair Card Texture, makes materials and a render node. It includes default hair strips with four regions of a few hundred strips each. Each card is matched to the strip whose length and width ratio fits it best, to limit stretching. Short hairs near the top use the left-side strips and long hairs near the bottom use the right-side ones. The bake is a render of the strand groom onto the cards ([SideFX, Hair cards](https://sidefx.com/docs/houdini/fur/haircards.html)). The page gives no numbers beyond these.

### DiffHairCard (Zheng et al., 2025)

This paper has the most complete published recipe. It is still a research system and runs offline. Every number below is from the paper's text ([Zheng et al.](https://arxiv.org/abs/2505.18805)).

1. **Cluster the strands.** Each strand has 32 samples. The distance between two strands is the sum of squared distances between matching samples. The strands are clustered with k-means, and each cluster's centre is a mean strand. The models use 40,000 strands.
2. **Start a card on each cluster.** A frame is built along the mean strand with the Bishop method (a frame that does not twist, which is more stable than the Frenet frame). The card's half-width at each sample is the largest absolute projection of the cluster's strands on the binormal. The card is a strip of quads, with 31 quads for a standard card, 15 for a cross card and 48 for an afro. The root normal is chosen among 20 candidates spread over a half-circle, by the smallest projection error.
3. **Store strand samples in card coordinates.** Each strand sample is projected on its card and stored as a 2D coordinate. The paper keeps this as an "explicit texture" so that the optimisation does not alias.
4. **Share textures.** Card textures are rendered, compared with the LPIPS image distance and merged by k-means to a target texture count.
5. **Optimise.** A differentiable renderer (Nvdiffrast) draws the strands from 64 viewpoints on a sphere, at 128 by 128 pixels for the card orientation and 256 by 256 for the joint optimisation. The losses are tangent, depth and silhouette ("dice"). Two regularisers keep tangents consistent and keep cards out of the head, with weights listed in the paper (for example 10, 10, 5, 3 and 100,000 for straight hair). It converges in roughly 200 epochs.
6. **Pack the atlas.** The default atlas is 2048 by 2048 pixels and holds 4 by 8 = 32 textures of 512 by 256 pixels. The textures are tangent, depth, alpha and ambient occlusion.
7. **Options.** A crossed card places a second card at 90 degrees. A hair cap extrudes the scalp faces by a small distance and bakes the first sub-segment of each strand into it.

**Published numbers.** The paper's figures use 100 to 800 cards with 32 textures. It says that the MetaHuman lowest level of detail has 351 cards and 20 textures. 100 cards with 32 textures took about 46 minutes, and 400 cards about 1 hour, on an AMD Threadripper 3970X with an RTX 3090. Memory use was about 7 GB for 100 cards and 10 GB for 400 cards. In Unreal, a 50,000-strand model cost 2.4 ms per frame and 512 cards cost 0.8 ms. The paper beat Unreal's own card tool on three styles, for instance 17.78 against 19.99 PSNR on a bun. I found no code link for it.

**What this means for KOTOR.** The paper does not say how many triangles a card has. If each of 31 quads has two triangles, a card has 62 triangles and 100 cards have 6,200 triangles. I infer this from the paper's numbers. KOTOR's whole head model has about 500 to 900 triangles ([2026-11-18-kotor-head-meshes.md](2026-11-18-kotor-head-meshes.md)). So a KOTOR-style result needs far fewer and coarser cards, for example a few tens of cards of 2 to 6 quads each. That is my estimate and not a published figure.

### Strands2Cards (Tojo et al., 2025)

Strands2Cards clusters strands into wisps, skins and aligns the strands of each wisp into a normalised pose in UV space, and builds a shared texture for similar wisps. It then fits polygon strips to the clusters with a differentiable renderer that optimises transparent, cluster-coloured coverage masks. It was tested on 342 hair models from the USC-HairSalon database, and the demo ran at 30 frames per second on a MacBook Pro M3 Pro ([project page](https://kenji-tojo.github.io/publications/strns2cards)). The code is at [kenji-tojo/strns2cards](https://github.com/kenji-tojo/strns2cards), and the page gives no license. A repository listing of the abstract says it converts a model of more than 100,000 strands in under 20 seconds ([MBZUAI listing](https://irep.mbzuai.ac.ae/items/042f079d-4815-4160-ac21-34f2df9a4967)). That is much faster than DiffHairCard, but I could not read the paper's own timing table.

### Blender and other tools

Several Blender add-ons turn curves into cards. They include a free Geometry Nodes setup by Daniel Bystedt ([CG Channel, press](https://www.cgchannel.com/?p=156171)) and the paid add-ons GBH Tool and Curve-BStrands ([listing search, press and store pages](https://www.cgchannel.com/tag/hair-cards-from-curves/)). I did not check how any of them works inside. They show that the curve-to-card step is routine in artist tools, with a card laid along each curve and twisted to face outward.

### What I could not find for volumes

I found no published method that turns a hair density volume directly into a game hair mesh, for example with marching cubes. The nearest equivalents are the hair mesh (Yuksel) and the hair cap (DiffHairCard). I infer that a marching-cubes pass would give one closed shell without the strand texture that makes cards look like hair, so a volume is a poor starting point.

## Games and tools that let the player shape hair

Each entry says what the player actually controls.

| Product | What the player controls | Source quality |
|---|---|---|
| VRoid Studio | Draws hair bundles with a brush on a guide mesh that looks like a white cage around the head. Retouches a bundle by stroking again. Moves control points to change length and wave. Adds procedural groups with sliders. Clones and flips groups. Paints the hair texture in layers. Saves a hairstyle as a preset. | Official help pages |
| Reallusion Character Creator 4 | Swaps hair parts (fringe, back length, ponytail), edits root and end colours, and edits beard and brow meshes. | Official product page |
| MetaHuman (Houdini tools) | An artist, not the player, sets hairline, parting, density, length, volume, curl and clumping. | Official docs |
| Dreams (PS4) | Free sculpting, for example hair sculpted with surface snap and the curve tool, per the art director. This is a general sculpting tool, not a hair tool. | Official blog |
| Minecraft | Paints a skin. The outer skin layer is 1 pixel bigger than the inner layer on the head and can be transparent. It is used for glasses, hats and accessories. The wiki does not name hair. | Community wiki |
| Saints Row (2022) | 63 hairstyles, 22 facial hair options, colours and a glossiness slider. | Press and a guide, no official list |
| Black Desert Online | Press says players can adjust how hair sits and blend colours. I found no primary list of hair controls. | Press only |
| Dragon's Dogma 2, Monster Hunter Wilds, The Sims 4 | Presets and colours as far as I could find. Dragon's Dogma 2 fits beards and brows with a wrap deformer when the face is built (see the earlier notes). | No primary source for the hair menu |

### VRoid Studio in detail

VRoid is the only product I found where a player draws hair. Its manual describes these tools ([VRoid Help, Hair Editor](https://vroid.pixiv.help/hc/en-us/articles/360012339194-Hair-Editor), [VRoid Help, customize hairstyles](https://vroid.pixiv.help/hc/en-us/articles/900005678786-How-do-I-customize-hairstyles)).

- The **brush tool** draws hair on a guide mesh around the head. The grid lines that appear show where a stroke can go. The user can move the guide mesh's green control dots to change its shape. This guide mesh works like the hair envelope of a hair mesh.
- The **retouch tool** redraws a bundle. "Trace the parts you'd like to fix and their shape will change/be overwritten according to the direction of the stroke." A stroke that starts at the tip extends the length, which is how very long hair is made. Holding R while using the brush or select tool switches to it.
- The **select tool** picks a bundle and shows its **control points**, which change the length and undulation of the bundle.
- A **procedural group** makes bundles of a set count and shape from sliders. It can be converted to a freehand group, after which the sliders no longer apply and only the control points can be edited.
- Groups can be duplicated and flipped about the centre line. The manual uses this for pigtails.
- Hair has materials with a base and a highlight texture that the user edits in a texture editor. The hair texture can be painted in layers.
- The manual advises drawing "divided" strands so that hair overlaps, and to place the guides to get better overlap ([VRoid Help, Hair-modeling tips](https://vroid.pixiv.help/hc/en-us/articles/360013208914-Hair-modeling-Tips)).

**Polygon numbers.** VRoid says that its chat services support up to 20,000 polygons and that its default uniform is about 16,000 polygons, so it asks users to fit hair into about 4,000. The same page gives three measured results for one hairstyle: 5,496 polygons with smoothness 15 and a diamond cross-section, 1,304 with smoothness 7 and an unfilled-triangle cross-section, and 1,400 when the bangs kept smoothness 15 and the back used a larger interval ([VRoid Help, decreasing hair polygons](https://vroid.pixiv.help/hc/en-us/articles/360013210674-How-to-decrease-the-number-of-hair-polygons)). So VRoid's hair is a tube or ribbon mesh whose cost is set by smoothness, cross-section, thickness and interval, which suits KOTOR's budget only after strong reduction.

**Limits.** VRoid is closed source and I found no code. A user on the Steam forum reports that making a ponytail was difficult ([Steam discussion, a user post](https://steamcommunity.com/app/1486350/discussions/0/5251727781286694661)). So even this tool is not self-evident to new users, and the sliders and presets matter.

### What I could not confirm

I could not find a primary description of the hair controls of Black Desert Online, Monster Hunter Wilds, Saints Row or The Sims 4. The press coverage for Saints Row and Black Desert is listed in the table and is not official. No shipped game that I found lets a player draw a hairstyle with free strokes, except VRoid Studio.

## Fitting hair inside headgear

### What the published work says

**Hair Smash.** DreamWorks' technique is the one published recipe for hair under a hat that applies to strand hair ([O'Hagan et al.](https://history.siggraph.org/wp-content/uploads/2022/10/2015-Talks-O_Hagan_Hair-Smash.pdf)). I read the whole one-page talk.

1. For each combination of hairstyle, head, body and hat, export 15 percent of the render curves at one reference frame.
2. Run a collision step in Houdini that finds the hair segments that pass through the hat and "iteratively" pushes them towards the scalp. It has falloff options so that the push fades with distance and so that the hair can stay close to the hat.
3. Store each smashed curve with a point attribute that holds its closest distance to the hat.
4. At render time, move the smashed curves with the animated skin using a wrap deformer, so that each curve follows one transform. The hair generator treats them as "motion guides", with a before and an after curve. Any hair that follows a guide deforms by the same amount, with a shear that lays the hair on the skin.
5. Also, force every hair to stay within the stored collider distance of its guide. The paper says this "guarantee[s]" that no hair passes through the hat, though the generator never sees the hat.
6. For low-detail tube hair, reduce the tube radius so it does not exceed the collider distance.

The numbers are 2 minutes per variation against about 2 weeks if the artists groom each variation, and about 15 MB per variation for one reference frame against about 15 MB per frame. The studio used it on three feature films and made hundreds of variations automatically.

**A related patent.** A patent on "temporally coherent hair deformation" describes fixing the jump when a hat moves and a hair goes from just missing it to touching it, by giving each hair one deformation direction ([US 8,169,438](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/8169438)). I read only the search summary of it.

**Hair meshes against collisions.** Yuksel et al. note that the hair mesh surface can be used to avoid hair crossing nearby objects when the style noise is small, and that Kim and Neumann's earlier method can also avoid intersections at strand level ([Yuksel et al.](https://people.engr.tamu.edu/schaefer/research/hair.pdf)).

### What games do

I found no first-party description of this from a studio. The following come from forum posts and mod pages, so treat them as testimony and not as fact.

- Many games "make a special hairstyle that fits under all hats, and swap the normal hair to that when a hat is equipped". A reply on the Unreal Engine forum also says to make several versions of each hairstyle with the parts beyond the hat line removed ([Unreal forum post](https://forums.unrealengine.com/t/put-a-helmet-on-a-hairy-head/2456372)).
- A second Unreal forum thread asks how to hide hair cards under a cap with a mask ([Unreal forum post](https://forums.unrealengine.com/t/create-a-mask-to-hide-haircuts-cards-under-a-cap-or-a-hat/1767140)).
- Sims 4 modders make "hat chops", which are copies of a hairstyle with the hair under the hat removed ([Sims 4 Studio tutorial, forum post](https://sims4studio.com/thread/26873/make-hat-chops-ts4-hair)).
- DAZ users use a tool or a deformer to pull hair in under a hat ([DAZ forum post](https://www.daz3d.com/forums/discussion/188416/how-to-crush-hair-so-it-doesn-t-poke-through-hats-and-other-headgear)).
- Dragon's Dogma 2 uses a wrap deformer with a falloff map. The vertex colour of the map separates rigid parts (helmets) from soft ones (some hair) ([Autodesk article, already in `2026-11-16-character-creation-tech.md`](https://blogs.autodesk.com/media-and-entertainment/2025/04/10/ideas-and-ingenuity-behind-capcoms-in-house-re-engine-tools-for-dragons-dogma-2/)).

I found no published way to make a long hairstyle compress by itself under any of several hats. The three approaches I found are a swapped variant, a hidden or cut part, and a collision push (Hair Smash).

### A method for kjk

This is my own design, built from Hair Smash and from VRoid's guide mesh. The sources do not give it as such.

1. **Build one headgear volume.** Take the inside surface of each of the 14 headgear items in the head's frame. Keep, at every direction from the head centre, the smallest radius over all items. Call this the **headgear envelope**. Then scale it down by a margin, for example 1 to 2 mm.
2. **Use the envelope as the guide mesh.** The creator draws and sets everything on a guide mesh that equals the scalp offset outward, never past the headgear envelope. A player cannot place a control point outside it. The cap and short hair always fit, by construction.
3. **Treat long hair as outside the envelope.** Hair that hangs below the helmet line, such as long hair on the back, is by nature outside it. For each guide strand, compute the signed distance of each control point to the headgear envelope. Push points that are inside a headgear item towards the scalp along the shell normal, with a falloff along the strand, as Hair Smash does. Store the distance per guide point.
4. **Bake two versions.** Bake one hairstyle for the bare head and one smashed version for use when headgear is worn. KOTOR's headgear is a separate model on the head's `gogglehook`. So the game swaps the hair model when headgear is equipped, as the games above do.
5. **Hide, if needed.** If the smashed version still shows through a helmet, mask it with the texture alpha near the cap's edge, as the Unreal forum threads suggest.

I did not check how KOTOR itself treats hair under helmets, and I did not find a source for it (see the gaps).

## What would suit kjk

### The setup in this note

- The language is Rust, so a method must be small and have no network.
- The look is KOTOR's: a head of 500 to 900 triangles, with hair either as the top of the face mesh or as a small separate mesh. In KOTOR 1, 20 of 101 head models add a separate hair mesh, and in KOTOR 2, 46 of 183 do ([2026-11-18-kotor-head-meshes.md](2026-11-18-kotor-head-meshes.md)).
- The result is baked to a cap, cards and a texture.
- Everything must be a slider or a direct edit, and hair must fit inside the 14 headgear items.

### Ranking of the approaches

1. **First, a guide mesh with style curves and sliders (Fu 2007 and VRoid).** The player draws stream curves and a dividing curve on a guide mesh, sets a few sliders (length, volume, curl, bangs, parting position), and sees the strands. Our own code solves the field and grows the strands. This is the only approach that is both a literal drawing tool and fully editable, and it needs no data. VRoid shows that players accept it. Fu et al. show that 3 to 5 minutes is enough for a style. The guide mesh is also the thing that keeps the hair inside the headgear envelope.
2. **Second, a hair cap and a few cards made by a small card builder.** After the strands exist, cluster them by their roots and directions, lay one card along each cluster's mean strand, and bake a strand texture. This follows Houdini's, Epic's and DiffHairCard's steps, with the optimiser removed. The cap is the scalp offset outward, as in DiffHairCard. I infer that, because KOTOR's own triangle budget is so low, a simple bake with k-means and a fixed texture strip will be good enough, and the differentiable renderer (46 minutes per run) is not needed.
3. **Third, a slider-only generator for the first pass.** The MetaHuman hairstyle generator's list of controls (hairline, parting, length, volume, curl, clumping, fades) is the model for the sliders. Sliders give the starting field that the player's curves then change.
4. **Fourth, the wisp model for detail.** Choe and Ko's master and member strands, with a length distribution, a deviation radius and a fuzziness value, give clumps and flyaways cheaply. It adds local variation that Fu's smooth field lacks. Check the patent first.
5. **Fifth, hair meshes.** The idea of drawing the hair as a layered polygon surface is the closest to how KOTOR hair is already made. Yuksel states that it is patented, so I rank it below Fu's method. I did not search for patents on Fu's method, so that ranking only reflects the patent notice I did see.
6. **Last, anything neural.** DeepSketchHair, Perm, HAAR, StrandDesigner and SketchHairSalon need Python, GPU training and datasets that are non-commercial or scraped. They would also fall against the decision to ship paint, blink, sliders, bake and skinning in Rust.

### What to try first

1. Write the Fu-style field solver in Rust on a grid of about 24 cells a side. Measure how long a conjugate-gradient solve takes for a stream curve that is moved by hand. If it takes more than about 50 ms, add the incremental update or lower the grid size.
2. Make the guide mesh from the scalp offset by a height value and clamp it to the headgear envelope of the 14 items.
3. Grow strands from roots on the scalp, and bake them to a hair cap plus 10 to 40 cards. Count triangles against KOTOR's own hair meshes, and compare in the head viewer next to the originals.
4. Add curl with the helix formula above, and one clump slider with the wisp rule.
5. Apply the Hair Smash step to the long-hair guides and bake the second, smashed version for headgear.

## What I could not find out

- **The full text of several papers.** I could not open Wither et al. (2007), Kim and Neumann (2002) and (2000), the full Choe and Ko paper (only the authors' patent), the Hair Smash talk's slides, or the full text of HAAR. Their numbers are missing from this note.
- **Code for the sketch methods.** I found no code for Fu et al., Hu et al., HairBrush, DeepSketchHair or DiffHairCard. SketchHairSalon and Strands2Cards have repositories, and neither states a license in the page I read.
- **Patent status.** I did not find out whether the Choe and Ko patent or the hair mesh patents are still in force, or whether Fu et al. or others hold patents.
- **VRoid's internals.** The manual describes the tools, but I found no description of how a stroke becomes a ribbon, what the guide mesh is, or how many vertices a bundle has. VRoid's output model is closed.
- **Player hair controls in other games.** I found no primary source for the hair controls of Black Desert Online, Monster Hunter Wilds, Saints Row, The Sims 4 or Dragon's Dogma 2.
- **How KOTOR treats hair under headgear.** I did not look at the game's files and I found no document on it.
- **Published numbers for low-triangle hair cards.** The papers I found are about modern hair with hundreds of cards. I found no paper or talk about hair cards of the size KOTOR uses, which is a few hundred triangles in all.
- **A published way to turn a hair density volume into a mesh.** None found.
- **Timing of a coarse Fu-style solver.** The estimate in this note is mine and has not been measured.
