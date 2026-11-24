# Procedural skin detail

This note was researched on 2026-11-19 for kjk's head paint track, which is part C of the roadmap (`2026-11-19-head-roadmap.md`).
It asks how pores, folds, wrinkles, freckles and other fine skin detail can be painted procedurally into a face's colour texture.
It then says which methods suit kjk's setup and ends with a design for the "fine detail and wrinkles" layer.

The earlier notes `2026-11-19-procedural-face-paint.md` and `2026-11-18-painted-facial-hair.md` are not repeated here.
The first one found no colour recipe for pores, freckles or moles and said MetaHuman uses scanned detail.
This note goes further on those points: it finds the measured numbers, the geometry methods that can be turned into colour, and the way tools and games expose them.

Terms used in this note.
- **Albedo** is the colour of a surface with no lighting in it.
- **A height map** (also called a displacement map) stores how far each point of the skin sits above or below its neighbours.
- **A normal map** stores the direction the surface faces at each pixel. KOTOR has no normal maps.
- **A cavity map** is a greyscale map that is dark in small dents such as pores and creases.
- **A pore** here means the visible opening of a hair follicle or sweat gland on the skin.
- **A furrow** is the narrow groove at the middle of a wrinkle. **A bulge** is the rounded ridge beside it.
- **A crease** is the general word for a wrinkle or a fold. **A fold** is a deeper crease that comes from sagging, gravity or fat compartments. **A wrinkle** is a shallower crease in the skin layer itself.
- **Langer's lines** are the directions in which skin is under the most tension, found on cadavers. **Relaxed skin tension lines** are the same idea measured on living people.
- **Worley noise** (also called cellular noise) gives, at each point in space, the distance to the nearest of many scattered feature points. F1 is the distance to the nearest point and F2 is the distance to the second nearest.
- **A Poisson-disk distribution** is a set of random points that keep a minimum distance from each other.
- **A solid texture** is a texture defined at every point of 3D space instead of on a 2D map, so it needs no UV layout.
- **A geodesic distance** is the shortest distance along the surface, not through the air.
- **A UV layout** is the map from the 3D surface to a flat texture.
- **Melanin** is the brown skin pigment. **Haemoglobin** is the red pigment of blood. kjk's base tone is built from these two (decision 26 of the roadmap).
- **FLAME** is the head model our customiser is built on.
- **Rmax** is the largest peak-to-valley height found on a measured skin profile.

Every claim carries a link.
Where I could not open a page myself and saw only a search result's summary of it, I say so.
Modding wikis, mod pages and forum posts are marked as such.
A sentence with "I infer" or "probably" is my own reasoning and not a fact from a source.
Papers I downloaded and read are in `/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/skin_detail_research/`.

## Summary

- **Our texture can only show the large end of skin detail.** One pixel of our face texture is 0.11 to 0.14 mm wide. Hair follicle openings sit 0.3 to 0.5 mm apart, which is 2 to 4 pixels. Visible pores sit 1.1 to 3.2 mm apart and are 1.4 to 4.9 pixels across. Micro wrinkles are 20 to 100 micrometres deep and far narrower than one pixel. So the layer should paint visible pores as small dots, follicle-level grain as fine noise, and creases as lines that are 1 mm or wider.
- **Pores have real numbers.** On the cheeks of 2,585 women, pores of at least 250 micrometres had a density of 10 to 80 per cm² and a size of 0.03 to 0.22 mm² ([Flament et al. 2015](https://www.dovepress.com/facial-skin-pores-a-multiethnic-study-peer-reviewed-fulltext-article-CCID)). Pores on the nose and forehead are more prominent than on the cheeks ([Jung et al. 2018](https://pubmed.ncbi.nlm.nih.gov/29393550/)). Follicle density falls from about 1,200 per cm² at the nose to about 460 per cm² beside the ear ([a textbook chapter quoting Pagnoni et al. 1994](https://vdoc.pub/documents/bioengineering-of-the-skin-skin-imaging-analysis-2nd-edition-dermatology-clinical-basic-science-3octl76tgrsg)).
- **The best pore method for us is 3D Worley noise with thinning.** It is a function of the 3D point, so it has no seams and no stretching. A feature point's own hash decides whether it is kept, and a region mask evaluated at that point sets the local density. The distance to the nearest point gives the dot. Because the feature points lie in space and not on the skin, the dots come out in many sizes, which looks natural (I infer this).
- **The best wrinkle method for us is a curve with a cross-section profile.** Bando, Kuratate and Nishita give the profile S(l) = −d (1 − l/w) exp(−l/w), where l is the distance from the furrow ([Bando et al. 2002](https://web.media.mit.edu/~bandy/wrinkle/PG02wrinkle.pdf)). It has a sharp furrow, round bulges and zero net volume. The curves come from landmarks and the distance l is measured in millimetres on the head, so the layer follows the sliders. The Disney micro-wrinkle paper reuses this profile ([Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)).
- **Age should switch wrinkle sets on one by one.** In 298 women aged 30 to 78, lower face sagging correlated most with age (0.764), then crow's feet (0.602), upper lip lines (0.558), glabellar lines (0.553), under-eye lines (0.546), mouth corner lines (0.520) and forehead lines (0.335) ([Flament et al. 2013](https://pmc.ncbi.nlm.nih.gov/articles/PMC3790843)). That order gives the order in which our sets appear.
- **Almost nobody publishes colour recipes.** Reallusion, Baldur's Gate 3, MetaHuman and Texturing.xyz all use scanned or authored images for pores and wrinkles, and the sliders only blend or scale them. The only fully procedural pipelines I found are in research (Disney, 2023) and in a Houdini talk (2024). Both produce geometry, not colour. The colour recipe in this note is therefore my own design, built on those methods.
- **Fitting to KOTOR is the real risk.** No source says how big KOTOR's painted creases and pores are. The design below keeps every amplitude and width as a parameter so that the fit to the 57 test heads can set them.

## What our texture can show

The face gets 53 to 84 pixels per mm² in our layout, as the task says.
One pixel is therefore 1 / √84 = 0.109 mm to 1 / √53 = 0.137 mm wide.

| Feature | Published size | Size in our pixels | Source |
|---|---|---|---|
| Follicle opening spacing | 460 to 1,200 follicles per cm², so 0.29 to 0.47 mm apart | 2 to 4 pixels apart | [Pagnoni et al. via a textbook chapter](https://vdoc.pub/documents/bioengineering-of-the-skin-skin-imaging-analysis-2nd-edition-dermatology-clinical-basic-science-3octl76tgrsg) |
| Visible pore spacing | 10 to 80 per cm², so 1.1 to 3.2 mm apart | 8 to 29 pixels apart | [Flament et al. 2015](https://www.dovepress.com/facial-skin-pores-a-multiethnic-study-peer-reviewed-fulltext-article-CCID) |
| Visible pore size | 0.03 to 0.22 mm², so 0.20 to 0.53 mm across | 1.4 to 4.9 pixels across | [Flament et al. 2015](https://www.dovepress.com/facial-skin-pores-a-multiethnic-study-peer-reviewed-fulltext-article-CCID) |
| Primary micro wrinkle depth | 20 to 100 micrometres | Far below one pixel | [Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf) |
| Secondary micro wrinkle depth | 5 to 40 micrometres | Far below one pixel | [Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf) |
| Micro wrinkle spacing | 1 to 2 wrinkles per mm on the scalp, about 10 per mm on the forearm | Less than one pixel apart on the forearm | [Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf) |
| Expression wrinkle width and depth parameters on a face | 2 to 12 mm | 15 to 110 pixels | [Bando et al. 2002](https://web.media.mit.edu/~bandy/wrinkle/PG02wrinkle.pdf) |

The pixel sizes in the third column are my own arithmetic from the sizes in the second column.

Three things follow, and they are my inference.
- Micro wrinkles cannot be drawn as lines. At most they can feed a fine noise that adds grain.
- Follicle-level pores cannot be drawn as dots either, because they would be 2 to 4 pixels apart and would turn into noise. Visible pores, which are the large and sparse ones, can be drawn as dots.
- Creases are wide enough to be drawn exactly.

If the head customiser shows the face close up, a pore of 2 pixels will look like a speck. A "pore size" control that exaggerates the pore radius lets the player and the KOTOR fit set the look.

## Pores

### What the dermatology papers measure

- **Visible pores on the cheeks.** Flament and colleagues photographed the cheeks of 2,585 women from Brazil, China, France, India and Japan with a polarised dermatoscope. A threshold of 250 micrometres matched expert scoring best (r = 0.76 for Caucasian and 0.82 for Asian women). Pore density ranged from 10 to 80 per cm² between ethnic groups, an eightfold range. Age had no significant effect on density within a group. Pore size ranged from 0.03 to 0.22 mm² on average between groups ([Flament et al. 2015](https://www.dovepress.com/facial-skin-pores-a-multiethnic-study-peer-reviewed-fulltext-article-CCID)).
- **Enlarged pores.** In the same study, the 30% of women with the largest pores had pores of 0.06 to 0.37 mm² on average, and the densities rose from 10 to 21 per cm² in one comparison. Brazilian and Indian women showed pore size increasing with age. Caucasian, Japanese and Chinese women plateaued after 50 ([Flament et al. 2015](https://www.dovepress.com/facial-skin-pores-a-multiethnic-study-peer-reviewed-fulltext-article-CCID)).
- **Regions.** In 434 polarised images, the enlarged pores of the nose and forehead were more prominent than those of the cheeks. Counts rose with age, and the rise was largest between the 30s and 40s. Pore counts correlated with wrinkle severity ([Jung et al. 2018](https://pubmed.ncbi.nlm.nih.gov/29393550/)). I could not read the numbers behind this abstract.
- **Follicle density.** Pagnoni, Kligman and colleagues counted follicles on the forehead, cheek, chin and nose of 12 women. The density followed a decreasing gradient from the centre of the face to the sides, and it did not correlate with sebum output ([Pagnoni et al. 1994](https://pubmed.ncbi.nlm.nih.gov/7857840)). A textbook chapter that quotes the paper gives 1,200 per cm² at the nose and 460 per cm² at the preauricular area. I read this on a document-sharing copy of the textbook, so it is a secondary source ([textbook chapter](https://vdoc.pub/documents/bioengineering-of-the-skin-skin-imaging-analysis-2nd-edition-dermatology-clinical-basic-science-3octl76tgrsg)).
- **Sebaceous glands.** A review gives 300 to 1,500 sebaceous glands per cm² on the face. I saw this only in a search result's summary ([Springer chapter](https://link.springer.com/content/pdf/10.1007/978-3-662-47398-6_33.pdf)).

I found no table of pore diameter by facial region.
The nose-to-ear ratio of 460 / 1,200 = 0.38 is the only per-region number I have.

### Pore shape and direction

- Skin pores and hair follicles are the starting points of micro wrinkles, and fine wrinkles always meet at pores ([Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)).
- Weiss and colleagues let pores stretch along the main wrinkle direction with a "skew" parameter between 0 and 1. They give the pore width a range of 0.01 to 0.5, in the paper's own units ([Weiss et al. 2023, table 1](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)).
- A Houdini tool by Ramon Schauer lets an artist draw guide curves that set the pore direction. I saw it only through an article that does not describe the maths ([80.lv](https://80.lv/articles/a-houdini-tool-for-procedural-pores-and-micro-wrinkles)).
- The TexturingXYZ tutorial on Adobe's Substance site, as summarised by a search result, uses several fill layers with different grain types, each masked to a region. For pores with a clear direction, such as those around the eyes, it uses a brush that follows a path ([Adobe Substance 3D magazine, search summary only](https://substance3d.adobe.com/magazine/using-texturingxyzs-human-skin-materials-in-substance-painter/)).

### Method P1, 3D Worley noise with thinning (recommended)

Worley's cellular texture basis function scatters feature points through 3D space at a given density and returns the distance to the nearest one ([Worley 1996](https://cedric.cnam.fr/~cubaud/PROCEDURAL/worley.pdf)).
Substance Designer has the same idea as its Voronoi node, which makes 3D Voronoi noise ([Adobe, Voronoi](https://substance3d.adobe.com/documentation/sddoc/voronoi-225972024.html)).
Substance Painter has added 3D Perlin and 3D Worley noises, which work in world space (from a search result's summary of the [release notes](https://experienceleague.adobe.com/en/docs/substance-3d-painter/using/release-notes/all-changes)).

The steps are these.
1. Choose a cell size `c` in millimetres. Split 3D space into cubes of edge `c`.
2. For the cube at integer coordinates `i`, hash `i` into one jittered feature point `q = (i + h1(i)) * c`. Hash it again to get `u` in [0, 1) and a size draw `s` in [0, 1).
3. At the pixel's 3D point `p`, loop over the 3×3×3 cubes around it. For each feature point `q`, compute the distance `d = |p − q|`.
4. Keep the feature point only if `u < f(q)`, where `f` is the region density factor in [0, 1] evaluated at `q`. This is called thinning. It makes the pore density follow a mask, as the nose-to-cheek gradient needs.
5. Give each kept point a radius `r = r0 * (0.7 + 0.6 * s)`, where `r0` is the mean pore radius in mm.
6. The pore darkness is the largest value over the kept points of `a * (1 − (d / r)²)²` for `d < r`, and zero elsewhere. The value `a` is the darkening amount.
7. Multiply the colour by `1 − darkness`, and shift the hue a little towards red and brown (the amount is for the KOTOR fit to set).

Inputs and ranges.
- Cell size `c` sets the density. If the points were kept at probability 1 and the pore sphere were thin, the density per cm² would be about 100 / c². In practice use the slab rule below.
- Mean pore radius `r0` is 0.10 to 0.26 mm for real visible pores, from the 0.03 to 0.22 mm² sizes of Flament et al. I infer that a larger value is wanted for KOTOR's painted look.
- Darkening `a` has no published value. I suggest starting at 0.10 to 0.35 and fitting it.

The slab rule (I infer this).
- The feature points lie in space, not on the skin. A sphere of radius `R` whose centre is at height `h` above a flat skin patch cuts the skin in a disc of radius √(R² − h²).
- So points close to the skin make big dots and points far from it make small dots. This gives a natural spread of sizes without a second random number.
- The expected number of dots per unit of skin area is the 3D point density times 2R. To get a wanted surface density `ρ_s` per mm², use a 3D density `ρ_3 = ρ_s / (2R)` points per mm³. The cell edge is then `c = (1 / ρ_3)^(1/3)`.
- The mean dot area is (2/3) π R², which is two thirds of the area of a dot made by a point lying exactly on the skin. Use `R` about 1.2 times the wanted dot radius to compensate.
- Where the skin is strongly curved, such as at the nostril wing, the rule is only approximate.

Direction (stretching) without seams.
- To stretch each pore along the local skin flow direction `t`, measure the distance with `d² = |Δ|² + (k² − 1)(Δ · t)²`, where `Δ = p − q`, and `t` and `k` are looked up at the feature point `q`, not at the pixel. A value of `k` below 1 stretches the dot along `t`, and `k` above 1 squeezes it.
- Because `t` belongs to the feature point, each pore stays a smooth ellipse. The only jump is where the nearest point changes, and there the dot value is close to zero if `r` is smaller than about half the cell size (I infer this).
- The flow direction `t` can be built from landmark curves, for example as the direction of the nearest curve's tangent, or from Langer or relaxed skin tension line maps ([DermNet](https://www.dermnetnz.org/topics/skin-tension-lines)).

Fits kjk. Yes. It is a function of the 3D point, it needs only masks built from landmarks, and it is cheap.

### Method P2, Poisson-disk points on the surface

Weiss and colleagues sample pores as graph nodes with Poisson-disk sampling in UV space. To avoid stretched spacing, they compute a radius scaling map from an approximation of the geodesic distance, and multiply the average node distance by it ([Weiss et al. 2023, section 3.1.1](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)).
Their table gives the average pore distance a range of 0.001 to 0.05. I take these to be UV units, but the paper's text does not say so.

This gives more even pores than Worley noise, since no two pores come closer than the disk distance.
It needs a point list built beforehand, with a spatial index, and the pores then have to be splatted into the texture.
It fits our baker less well than P1, because every pixel must find the pores near it.
It is the method to choose if the KOTOR fit shows that the pores must be evenly spaced.

Fits kjk. Partly. It works as a precomputed point set per head, but it needs a stretch correction and a spatial index.

### Method P3, scanned exemplar synthesis

Haro, Guenter and Essa grew a skin normal map across a face from a measured patch with texture synthesis ([Haro et al. 2001](https://diglib.eg.org/items/1bfc433a-8dc1-4c05-b14b-7251363cfd25)).
Graham and colleagues scanned skin patches of 24 mm by 16 mm at about 7 micrometres per pixel, and synthesised them per facial region, guided by the larger scan ([Graham et al. 2013](https://vgl.ict.usc.edu/Research/Microgeometry/Measurement_Based_Synthesis_of_Facial_Microgeometry.pdf)).
Golovinskiy and colleagues built a statistical model of detail that varies across the face, and used it to adjust apparent age ([Golovinskiy et al. 2006](https://pixl.cs.princeton.edu/pubs/Golovinskiy_2006_ASM)).

These methods need scanned data and a 2D layout.
Their detail is far finer than our pixels.
Fits kjk. No.

## Wrinkles and folds

### Where wrinkles go

- **Tension lines.** Langer's lines come from cadavers. Relaxed skin tension lines come from living people. On the face, the relaxed skin tension lines follow the facial muscles and cross Langer's lines at nearly a right angle ([DermNet](https://www.dermnetnz.org/topics/skin-tension-lines)). I read this in a search result's summary of the page. I found no published map of lines in a digital form that we could reuse.
- **Fine wrinkles follow a flow.** Weiss and colleagues call the main direction of the micro wrinkles the facial flow and cite Langer for it ([Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)).
- **Large wrinkles appear when the skin shrinks.** Bando and colleagues say skin is incompressible, so the excess skin buckles when it shrinks, and the wrinkles become more prominent with repeated wrinkling and with age. Both fine and large wrinkles deepen when the skin shrinks perpendicular to them ([Bando et al. 2002](https://web.media.mit.edu/~bandy/wrinkle/PG02wrinkle.pdf)).
- **Named creases.** A review of facial crease names lists the nasolabial fold, the creases around the lips, the mandibular fold (also called the marionette line), the transverse nasal line, the vertical glabellar line, the mental crease, the creases around the eyes, the forehead creases and the periauricular creases ([facial creases review](https://researchonline.ljmu.ac.uk/id/eprint/6404/1/facial%20creases%20review.pdf)). The same review says the nasolabial fold runs from the nose wing to the cheek beside the corner of the mouth, the mandibular fold runs from the corner of the mouth to the chin border, the crow's feet start at the eye corners and end on the side of the head, and forehead creases are almost exclusively horizontal.

### How deep and how wide

- **Cadaver depths.** In 55 embalmed cadavers aged 35 to 93, the mean Rmax was 0.24 mm for men and 0.32 mm for women on the forehead. At the outer eye corner it was 0.56 mm for men and 0.79 mm for women. The study found no link between age and wrinkle depth within the cadavers ([JAMA Dermatology cadaver study](https://jamanetwork.com/journals/jamadermatology/fullarticle/1105207)).
- **Crow's feet in life.** A silicone replica study of women aged 6 to 79 found the wrinkle depth at the eye corner rose from about 25 micrometres in the young, to eight times that by age 65. The number of lines doubled between ages 6 and 35 and stayed constant afterwards. I saw this only as a search result's summary, so I do not know the paper ([search summary](https://library.scconline.org/v038n03/2)).
- **Which sites deepen most.** A replica study of 136 Japanese women aged 18 to 83 found that depth increased with age most at the eye corner, then the glabella, the mouth corner, the nasolabial groove, the upper eyelid, the forehead and the lower eyelid. I saw this only through a search result's summary ([Takema et al., search summary](https://ilse.kobv.de/KobvIndexRecord/nat_lic_papers_NLZ243674554)).
- **Glabellar lines and age.** A fringe-projection study of men and women aged 20 to 70 found that glabellar lines did not show before age 40 in either sex. Men showed lines earlier and more severe. I saw this only through a search result's summary, and I do not know the paper.
- **Expressive wrinkle sizes in graphics.** Bando and colleagues used depth and width values of 2 to 12 mm for large wrinkles on a face scan, and 0.5 to 1 mm for the finger joints of a hand ([Bando et al. 2002](https://web.media.mit.edu/~bandy/wrinkle/PG02wrinkle.pdf)). These are about ten times the cadaver depths, so I infer they describe expression wrinkles on a mesh, not resting skin.
- **A rating scale.** Lemperle and colleagues rate wrinkles from 0 to 5 with photographs, and distinguish wrinkles in the skin layer from folds caused by laxity, gravity and bone loss ([facial creases review](https://researchonline.ljmu.ac.uk/id/eprint/6404/1/facial%20creases%20review.pdf)).

### Which wrinkles come with age

Flament and colleagues rated 298 Caucasian women aged 30 to 78 on standard photographic scales.
The table gives the Pearson correlation of each sign with age and with sun exposure ([Flament et al. 2013, tables 2 and 3](https://pmc.ncbi.nlm.nih.gov/articles/PMC3790843)).

| Sign | Correlation with age | Correlation with sun exposure |
|---|---|---|
| Lower face ptosis (sagging) | 0.764 | 0.498 |
| Crow's feet wrinkles | 0.602 | 0.478 |
| Upper-lip wrinkles | 0.558 | 0.531 |
| Glabellar wrinkles | 0.553 | 0.396 |
| Under-eye wrinkles | 0.546 | 0.395 |
| Eye bags | 0.529 | 0.303 |
| Mouth corner wrinkles | 0.520 | 0.402 |
| Forehead wrinkles | 0.335 | 0.282 |

The authors also describe an upper-lip texture with vertical lines and a grid-like microrelief, deep cheek folds without a fixed direction, wrinkles that run down from the corner of the lower lip to the chin, and jawline folds that start at the ear lobe.
Their conclusion is that UV exposure explains about 80% of the visible signs of facial ageing.

### Method W1, a curve with a cross-section profile (recommended)

This is the way Bando and colleagues model large wrinkles, and it is close to what Larboulette and Cani do ([Bando et al. 2002](https://web.media.mit.edu/~bandy/wrinkle/PG02wrinkle.pdf), [Larboulette and Cani 2004](https://hal.archives-ouvertes.fr/inria-00537455)).
The steps are these.
1. Place a curve for the wrinkle. Bando and colleagues draw a cubic Bezier curve in texture space. For kjk the curve would run through landmarks on the head and be a 3D spline.
2. For each pixel, find the distance `l` from the curve, measured perpendicular to it.
3. Apply the shape function `S(l) = −d (1 − l / w) exp(−l / w)`. The parameter `d` is the depth, `w` is the width, and `S` is the height change. At `l = 0` the height is `−d`, so the furrow is sharp. It crosses zero at `l = w` and rises to a gentle bulge beyond. I read this formula from the garbled equation text of the paper. It agrees with the paper's own condition that the integral of `S` over `l` from 0 to infinity is zero, which keeps the volume of the skin the same, and with the paper's figure.
4. Bando and colleagues then move the mesh vertices along their normals by `S`. We would instead use `S` to scale the colour.

A polynomial version over a bounded domain is given by Weiss and colleagues. With `x = l / w`, the shape with a bulge is `(x − 3)² (x − 1) / 9` for `x` from 0 to 3, and the shape without a bulge is `−(1 − 2x / 3)³` for `x` from 0 to 1.5 ([Weiss et al. 2023, equation 9](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)).
The bounded version costs nothing outside `3w`, which suits a per-pixel mask.

Making the line look natural.
- Weiss and colleagues found that straight lines look sterile. They offset the wrinkle's points with multi-scale Perlin noise looked up at the nearest point on the centre line, with a frequency of 100 to 2000 and a strength of 0.001 to 3 in their units ([Weiss et al. 2023, section 3.2.2 and table 1](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)).
- Bando and colleagues add a random perturbation of the direction at each segment ([Bando et al. 2002, section 5.2](https://web.media.mit.edu/~bandy/wrinkle/PG02wrinkle.pdf)).

Inputs and ranges.
- Depth `d` and width `w` in millimetres. For a colour layer the depth is only an amplitude, so it can be a darkening fraction.
- Length taper: the wrinkle's strength should fall to zero at both ends along the curve, for example with `smoothstep` on the arc length.
- Number of parallel lines and their spacing for forehead lines.
- A noise offset of 0.2 to 1 `w` along the curve.

Fits kjk. Yes. It needs only a distance to a landmark curve, which is a mask as in decision 32.

### Method W2, furrows grown from a direction field

Bando and colleagues grow fine furrows in texture space along a direction field.
Each furrow is grown from a random start point in segments. Each segment has a length `D`, a random rotation by an angle `θ`, a depth `I` and a width `Wh`. The furrow stops where it crosses another furrow.
A binary "occupancy image" keeps the furrows apart, so no new furrow starts in an occupied pixel.
The parameters depend on the size `m` of the direction vector, which is between 0 and 1. Stronger direction means longer, deeper and wider furrows with smaller random turning ([Bando et al. 2002, section 5](https://web.media.mit.edu/~bandy/wrinkle/PG02wrinkle.pdf)).

It works on a 2D image with a buffer, so it does not work per pixel at a 3D point.
It makes fine diamond-shaped furrows that are below our pixel size.
Fits kjk. No.

### Method W3, a pore graph carved by trail formation

Weiss and colleagues connect pores into a graph, and carve the wrinkles by a trail simulation.
The steps are these ([Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)).
1. Sample pores as nodes with Poisson-disk sampling, with a stretch correction.
2. Connect each node to its `k` nearest nodes to make candidate wrinkle edges.
3. Pick the edge direction with the probability `P[θ] ∝ α_s + exp(−|θ − θ̂|² / σ²)`, where `θ̂` is the main flow direction, `α_s` is a base strength from 0 to 1 and `σ = 0.3`.
4. Repeat: pick a random node and an outgoing edge with a probability that rewards short edges, the flow direction and continuing a wrinkle, and penalises crossings. Then raise the edge weight with `W ← W + Δt α_deposit (1 − W)`, where `Δt = 0.03` and `α_deposit` is 0.01 to 1.
5. After each batch of steps, shrink all weights by the factor `1 − Δt α_decay N / |E|`, with `α_decay = 0.5`.
6. Draw each edge with the shape function in the section above, scaled by its weight. Draw an extra deeper dent at each pore.

Their paper simulates 50,000 edges in parallel on a GPU, and reports 2 million pores and 10 million wrinkles for a full face.
It also shows that pores and wrinkles can be tied together, so that the creases of a heavily aged face start and end at the pores.
Fits kjk. No for the whole algorithm, because it needs a graph and a simulation. Its idea of seeding wrinkle lines from the pore points could be copied cheaply: a pore's darkening can stretch along the flow direction, as in P1.

### Method W4, wrinkle maps blended by masks and weights

Oat's animated wrinkle maps store two wrinkle normal maps: one for a stretched face and one for a compressed face.
A mask in one colour channel marks a region, and each mask has a weight between −1 and 1. Weight −1 gives the full first map, 0 gives neither and 1 gives the full second map.
The wrinkle map is added to the base normal map with `normalize(float3(w.xy + n.xy, w.z * n.z))` ([Oat 2007](https://www.advances.realtimerendering.com/s2007/Oat%20-%20Wrinkles%20-%20Course%20notes%20(SIGGRAPH%202007).pdf)).
Jimenez and colleagues extend this with a memory use of 96 KB and a shader cost of 0.31 ms on a GeForce 8600GT ([Jimenez et al.](https://graphics.unizar.es/papers/JimenezWrinkles_low.pdf)).
Raman and colleagues compute a "mesh tension" from expression scans to place wrinkles on identities that have no scan of that expression ([Raman et al. 2022](https://arxiv.org/pdf/2210.03529)).

These methods need authored or scanned wrinkle maps and tie them to expressions.
The idea of a weight per region that can be driven by one slider is useful, and it is what the age sets in my design do.
Fits kjk. The weights yes, the images no.

### Older and related work

- Boissieux and colleagues simulated skin ageing and wrinkles with cosmetics in mind. One summary says they applied generic masks of precomputed wrinkles as textures ([Boissieux et al. 2000](https://doi.org/10.1007/978-3-7091-6344-3_2)). A patent says this limits how well it predicts one person's own features ([US 8,290,257](https://patents.google.com/patent/US8290257)). I could not read the paper.
- Wu, Kalra and Magnenat-Thalmann simulated static and dynamic wrinkles ([Wu et al. 1996](https://dblp1.uni-trier.de/rec/conf/ca/WuKM96.html)). A later chapter of the group models expressive and ageing wrinkles from the principal strain of the skin and how long it stays deformed. I saw this only in a search result's summary.
- Golovinskiy and colleagues synthesise skin detail from scans and use it to adjust apparent age ([Golovinskiy et al. 2006](https://pixl.cs.princeton.edu/pubs/Golovinskiy_2006_ASM)).
- Kim and colleagues generate wrinkles from sketches, with the statistics taken from example faces. I saw this only in a search result's summary ([Kim et al. 2015](https://diglib.eg.org/handle/10.1111/cgf12551)).
- Schneider's thesis at Basel ages faces with a morphable model, detail transfer, wrinkle propagation and a stochastic pigment spot model ([Schneider 2019](https://edoc.unibas.ch/71615)). I could not open the thesis itself.
- I could not identify which paper by "Li et al." the task refers to. The only Li paper I met is an anisotropic Poisson-disk sampling method that Weiss and colleagues cite as "Li et al. [LWSF10]", and it is not about wrinkles. I have not guessed further.

## Other fine detail

### Freckles, age spots and moles

- **What they are.** Ephelides, the lay freckles, are largely genetic and are made darker by sunlight. Solar lentigines are induced by sun exposure and skin damage ([Praetorius et al. 2014](https://pubmed.ncbi.nlm.nih.gov/24517859/)).
- **Size and place.** Ephelides are about 1 to 2 mm across, though they can be larger ([Cleveland Clinic, a patient-information page](https://my.clevelandclinic.org/health/diseases/23091-freckles)). Solar lentigines are a few millimetres to several centimetres ([DermNet](https://dermnetnz.org/topics/brown-spots-and-freckles)). Ephelides appear on the mid face and sometimes more widely, from early childhood ([DermNet](https://dermnetnz.org/topics/brown-spots-and-freckles)).
- **Age and sun.** In the study of 298 women, pigmentation disorders tracked UV exposure at every age. The pigment signs the authors list include the malar area (the cheekbone), the outer cheek, and the lower face below the mouth corners ([Flament et al. 2013, table 4](https://pmc.ncbi.nlm.nih.gov/articles/PMC3790843)).
- **Density.** I found no published number of freckles per cm² and none for the size spread. I found no source for the size of moles either.
- **Game controls.** Reallusion's freckle layer has a type (nose, cheek or whole face), a pattern number, a random opacity between 0.5 and 1.0 and a random scale between 0.5 and 1.0, plus blur, blending and specularity ([Reallusion manual](https://manual.reallusion.com/character-creator-4/Content/ENU/4.0/15_Substance/Basic_Modifications_for_Head_Texture.htm)). Baldur's Gate 3 has freckle quantity and intensity controls ([BG3 wiki, a modding wiki](https://bg3.wiki/wiki/Character_Creation)). Baldur's Gate 3 stores freckles as melanin in the green channel of its HMVY skin tint texture ([BG3 modding wiki](https://bg3.wiki/wiki/Modding:Face_and_body_textures)).

Method F1, freckles as a thinned and clustered point set (my design).
1. Take the same 3D cell grid as in P1, with a larger cell edge of 3 to 6 mm.
2. Keep a feature point with a probability `u < f_sun(q) * amount`. The mask `f_sun` is high on the nose bridge, the cheekbones and the cheeks, and falls off towards the jaw, the forehead edge and the ears. It follows the "nose" and "cheek" types of the Reallusion manual.
3. To get clusters, make `amount` vary with a low-frequency 3D noise, so that freckles come in patches. This is a cheap stand-in for a clustered point process, and I infer it looks right.
4. Give each freckle a radius of 0.5 to 1.5 mm, with Reallusion's random scale of 0.5 to 1.0 as the spread, and an irregular edge by adding 3D noise to the distance `d`.
5. Give each freckle an opacity of 0.5 to 1.0 times an intensity slider.
6. Add the result to the melanin amount, not to the final colour, so the hue stays right for each skin tone. This follows Baldur's Gate 3's choice of keeping freckles in a melanin channel.

Moles are the same method with a much lower density, a hard edge, a radius of 1 to 3 mm and a very dark colour (the size is my design value and has no source).
Moles should be seeded points, not noise, so the player can place them.

### Blemishes, redness and mottling

- **Uneven tone.** Reallusion's skin noise templates "blend uneven skin colors", and its Variation slider has a range of 0.50 to 1.00 ([Reallusion templates](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_SkinGen/Skin/Skin-Details/Using-Skin-Details-Templates.htm), [Reallusion manual](https://manual.reallusion.com/character-creator-4/Content/ENU/4.0/15_Substance/Basic_Modifications_for_Head_Texture.htm)).
- **Capillaries and acne.** Reallusion's templates have a capillary effect for the face, and acne templates create a capillary effect ([Reallusion blemish templates](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_SkinGen/Skin/Blemish/Using-Blemish-Templates.htm)). Baldur's Gate 3 uses a vein map and a yellowing map with its haemoglobin and melanin maps ([Larian's lead character artist, 80.lv](https://80.lv/articles/larian-explained-why-there-are-no-face-sliders-in-baldur-s-gate-3)). The earlier note `2026-11-19-procedural-face-paint.md` covers that interview.
- **Redness by region.** Microvascular alterations of the cheekbone area correlate only weakly with sun exposure (0.257) and with age (0.086), and diffuse redness correlates 0.140 with sun exposure and 0.031 with age ([Flament et al. 2013, table 5](https://pmc.ncbi.nlm.nih.gov/articles/PMC3790843)). So redness is a separate control that does not follow the age slider.

Methods (my design, built from standard noise).
- **Mottling.** Multiply the haemoglobin and melanin amounts by `1 + m * n(p)`, where `n` is 3D gradient noise of wavelength 10 to 30 mm in [−1, 1]. Solid noise goes back to Perlin and Peachey ([Perlin 1985](https://history.siggraph.org/?p=107418), [Peachey 1985](https://history.siggraph.org/?p=107417)). The survey of noise functions describes the options, including Gabor noise, which has a controllable direction ([Lagae et al. 2010](https://hal.archives-ouvertes.fr/hal-00920177), [Lagae et al. 2009](https://dblp.org/rec/journals/tog/LagaeLDD09)).
- **Blotches and blemishes.** Threshold a lower-frequency noise, `smoothstep(t, t + e, n(p))`, and add the result to haemoglobin. A value of `t` near 0.7 gives a few round patches.
- **Fine capillaries.** Use the ridges of a noise, `1 − |n(p)|`, raised to a power of 8 to 16, to get thin branching lines in 3D. Mask them to the cheeks and the nose wings. This is the well-known ridged noise trick, so I infer it fits.
- **Scars.** A scar is a curve with the profile of W1 and a pale colour. It is placed by the player, so it is stored as a curve.

### Lip lines

- Flament and colleagues describe the upper lip with deep or shallow vertical lines, a thick padded look, and a grid-like microrelief ([Flament et al. 2013, table 3](https://pmc.ncbi.nlm.nih.gov/articles/PMC3790843)).
- The review of creases calls them upper and lower lip wrinkles, "smoker's lines" and "upper radial lip lines". It also says cadaver women aged 75 to 93 had more perioral wrinkles than men ([facial creases review](https://researchonline.ljmu.ac.uk/id/eprint/6404/1/facial%20creases%20review.pdf)).
- Baldur's Gate 3 keeps "tear and lip lines" in the blue channel of its MSK texture ([BG3 modding wiki](https://bg3.wiki/wiki/Modding:Face_and_body_textures)).

Method L1 (my design). For a pixel near the lip border, compute the arc length `a` along the border curve and the distance `l` from the border. Draw a line wherever `n1(a * freq)` is above a threshold, where `n1` is 1D noise and `freq` gives 1 line every 0.6 to 1.5 mm. Fade the lines with `l` over 3 to 8 mm. Give each line the W1 profile with `w` of 0.15 to 0.3 mm. The lines are so narrow that at our pixel size they read as a fine vertical hatching with a slightly darker tone.

## Tools and games

The table says what each tool or game exposes and what it does underneath, where that is documented.
The games in the earlier note `2026-11-19-procedural-face-paint.md` are not repeated unless they add something about pores or wrinkles.

| Tool or game | What the player or artist controls | What happens underneath | Source |
|---|---|---|---|
| Substance 3D Designer | Noise nodes with scale and disorder | Cells 1 is a Worley-style noise with a Euclidean distance. Crystal 1 is a Worley-style noise with a more angular distance, and has Scale and Disorder parameters. The Voronoi node makes 3D Voronoi noise on the GPU | [Cells 1](https://helpx.adobe.com/substance-3d-designer/substance-compositing-graphs/nodes-reference-for-substance-compositing-graphs/node-library/texture-generators/noises/cells-1.html), [Crystal 1](https://experienceleague.adobe.com/en/docs/substance-3d-designer/using/substance-graphs/nodes-reference-for-substance-graphs/node-library/texture-generators/noises/crystal-1), [Voronoi](https://substance3d.adobe.com/documentation/sddoc/voronoi-225972024.html) (the three pages were read through search result summaries) |
| Substance 3D Painter | Fill layers with grain types and hand-painted masks for each pore type | A TexturingXYZ tutorial stacks several grain types, masks each by region and changes the intensity with a smooth mask. Painter paints within the 0 to 1 UV range | [Adobe Substance 3D magazine](https://substance3d.adobe.com/magazine/using-texturingxyzs-human-skin-materials-in-substance-painter/) and the [Painter glossary](https://substance3d.adobe.com/documentation/spdoc/glossary-147095620.html), both through search result summaries |
| Houdini | Guide curves for pore direction, and stretch and compression from the animation | Schauer's tool is a SOP-level asset driven by guide curves. Klintberg and Haapaoja at SIGGRAPH 2024 present a fully procedural skin texture generator whose pores and wrinkles follow the animation's stretch and compression | [80.lv](https://80.lv/articles/a-houdini-tool-for-procedural-pores-and-micro-wrinkles), [SIGGRAPH history](https://history.siggraph.org/?p=160660) |
| Reallusion Character Creator 4 and SkinGen | Base (Variation 0.5 to 1.0, Tone 0 to 0.5, Normal Strength 0.5 to 1.0), Surface Normal for age and wrinkles, Freckles, Tan | Template images for skin detail and blemishes, and image layers for freckles with random opacity and scale. SkinGen's slider internals are not documented | [Reallusion manual](https://manual.reallusion.com/character-creator-4/Content/ENU/4.0/15_Substance/Basic_Modifications_for_Head_Texture.htm), [Head Skin Texture Layers](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_Substance/Head_Skin_Texture_Layers.htm) |
| MetaHuman | Skin tone, face texture index, freckle pattern with four sliders | Scanned detail textures picked by index. See the earlier note | [Procedural face paint](2026-11-19-procedural-face-paint.md) |
| Texturing.xyz VFace | Not a generator. It sells scanned data | Each face ships a 16K, 32-bit displacement with detail levels in the RGB channels, according to a forum user. The new SKAP service makes 16K displacement and cavity maps from a 2K to 8K input, and does not say how | [ArtStation listing](https://www.artstation.com/marketplace/p/GO2dR/texturingxyz-vface-setup-for-substance-painter), [Digital Production](https://digitalproduction.com/2026/03/10/texturing-xyz-puts-skap-online-pores-included) |
| Baldur's Gate 3 | An age lines option that "changes the depth and appearance of age lines", freckle quantity and intensity, scarring presets, vitiligo | Pores and cavities sit in the red channel of the CLEA texture. Freckles are in the green channel of HMVY. Wrinkles, scars and indents are in the NORMAL map. Pigment and vitiligo are in the green channel of MSK. Tear and lip lines are in its blue channel | [BG3 wiki, character creation](https://bg3.wiki/wiki/Character_Creation), [BG3 modding wiki](https://bg3.wiki/wiki/Modding:Face_and_body_textures). Both are fan-run wikis. Larian also chose to have no face sliders ([80.lv](https://80.lv/articles/larian-explained-why-there-are-no-face-sliders-in-baldur-s-gate-3)) |
| Skyrim and Fallout 4 (Bethesda) | An age setting that picks a complexion | Age complexions are texture images. A Skyrim mod page says the "40" and "50" images look like 60 and 70, and offers hand-painted replacements. A forum thread says Fallout 4's age menu applies normal maps | [Skyrim mod page](https://www.nexusmods.com/skyrim/mods/50870/), [Fallout 4 forum thread](https://forums.nexusmods.com/topic/10641153-in-need-of-wrinkles-make-upoverlay). Both are community sources |
| Black Desert | Wrinkle options for the brow, the eyes and the frown, which the game's wiki says adjust the age range | Not documented | [Black Desert wiki](https://www.naeu.playblackdesert.com/en-US/Wiki?wikiNo=5), through a search result summary |
| EA Sports FC 25 | Complexion, scars and other texture controls | The earlier note quotes EA saying its texture features are generated at runtime by a "Cryo Texture Compositor" | [Procedural face paint](2026-11-19-procedural-face-paint.md) |

What I infer from the table.
- Every shipped creator I could check uses authored or scanned images for the pore and wrinkle detail. The sliders select an image, scale its strength, or switch a layer on.
- Because of this, "age" in these games is a blend of images and not a growth process. A growth process is only found in research ([Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)).
- Baldur's Gate 3's channel layout is a useful model for what we bake: a cavity channel, a melanin channel, a wrinkle channel and a lip-line channel. I did not find how the shader uses them.

I found no primary source on how Black Desert, the Sims 4, Dragon's Dogma 2 or the EA creators implement their wrinkle sliders, and no public documentation of Houdini's own pore tools.

## On a 3D surface, then baked

Our layers are evaluated per pixel at the pixel's 3D point on the head, in millimetres.
The point is found from the texture layout, and the result is written into the pixel.
For a function of the 3D point, the two pixels on either side of a UV seam get the same value, since they sit at the same 3D point. I infer that the seam only needs the usual texture padding at its edge, and that this padding is not a layer problem.

| Method | Works on a 3D point alone | Needs a 2D space | Note |
|---|---|---|---|
| 3D gradient noise, 3D Worley noise (P1, mottling, F1) | Yes | No | Solid textures go back to [Perlin 1985](https://history.siggraph.org/?p=107418) and [Peachey 1985](https://history.siggraph.org/?p=107417), and need no UV layout. Worley's cells are defined in 3D ([Worley 1996](https://cedric.cnam.fr/~cubaud/PROCEDURAL/worley.pdf)). Gabor noise exists as solid noise as well ([Lagae et al. 2009](https://dblp.org/rec/journals/tog/LagaeLDD09)) |
| Euclidean distance to a landmark curve (W1) | Yes | No | Fine within a few millimetres of the curve. It is wrong where the skin folds back on itself, for example across the nasolabial fold, where a cheek point and a lip point can be close in the air and far along the skin |
| Geodesic distance to a curve | Computed once on the mesh | No, but needs a mesh | The heat method solves two sparse linear systems that can be factored once, and works from a curve as well as a point ([Crane et al. 2013](https://www.cs.cmu.edu/~kmcrane/Projects/HeatMethod/)). The result is a per-vertex field |
| Per-vertex fields baked by interpolation | Yes | No | A field stored on the vertices is continuous across UV seams, because the vertices are shared. I infer that a distance field interpolates well within a triangle, as long as the triangle is small compared with the wrinkle width |
| Poisson-disk points (P2), graph carving (W3) | Points yes, drawing no | Yes in the paper | Weiss and colleagues work in UV space and correct the stretch with a radius map from an approximate geodesic distance ([Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)) |
| Fine furrows grown on a height image (W2) | No | Yes | Bando and colleagues work in texture space with a planar or cylindrical projection, and share the boundary strip between body parts to avoid seams ([Bando et al. 2002](https://web.media.mit.edu/~bandy/wrinkle/PG02wrinkle.pdf)) |
| Patch synthesis from scans (P3) | No | Yes | Graham and colleagues unwrap the face and synthesise in the 4K map ([Graham et al. 2013](https://vgl.ict.usc.edu/Research/Microgeometry/Measurement_Based_Synthesis_of_Facial_Microgeometry.pdf)) |
| Wrinkle maps (W4) | No | Yes | They are authored images in the UV layout ([Oat 2007](https://www.advances.realtimerendering.com/s2007/Oat%20-%20Wrinkles%20-%20Course%20notes%20(SIGGRAPH%202007).pdf)) |

How others avoid seams and stretching.
- Solid textures avoid both, because no UV layout is involved.
- Weiss and colleagues scale the pore spacing by a stretch map so that pores are evenly spaced on the head, and refer to Li and colleagues for the anisotropic case ([Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)).
- Ptex gives each face its own texture and stores adjacency, so there are no UV seams to hide ([Burley and Lacewell 2008](https://diglib.eg.org/handle/10.2312/CGF.v27i4pp1155-1164)). We do not need this, because our baker can evaluate at any 3D point.

What I recommend for our baker.
1. Write every layer as a function of the 3D point `p` in millimetres, plus its surface normal `n` and a few per-vertex fields interpolated at the pixel.
2. Compute, once per head, a signed geodesic distance field to each wrinkle curve with the heat method, and store it per vertex. Use the Euclidean distance as a fallback where the mesh is too coarse. If the head mesh is as coarse as FLAME's, whose triangles are several millimetres wide (I infer this from its vertex count, which I did not re-check), interpolate the field and evaluate the profile on the interpolated value.
3. Use a normal-aware distance for creases that sit on a fold: `d_eff = |p − c| * (1 + k (1 − n_p · n_c))`, where `c` is the nearest curve point and `n_c` its normal. I infer this works, but it is untested.
4. Because a pore can be 2 pixels across, anti-alias the pore dots. Evaluate 4 samples per pixel, placed inside the pixel by barycentric interpolation of the triangle, and average them. The pixel footprint in millimetres varies across the layout (53 to 84 pixels per mm²), so a footprint-based smoothing of the pore profile is a cheaper alternative.
5. Evaluate the layer only inside the face mask. This keeps the cost low (see the design below).

## Recommended design for the fine detail and wrinkles layer

### What the layer outputs

For each pixel the layer returns four values.
1. A **cavity** value between 0 and 1, from pores and creases. The base colour is multiplied by `1 − cavity * k_cav`, with a slight shift towards red in the creases.
2. A **melanin change**, from freckles, moles and age spots. It is added to the melanin amount of the base tone, as in decision 26.
3. A **haemoglobin change**, from blemishes, capillaries, blotches and flushed creases.
4. A **height** in millimetres, from the pores and creases. The colour layer does not use it. It is kept so that a normal map can be made later, in the baker, from the derivative of the height with respect to the 3D tangent directions. The engine does not use normal maps today. Methods P1 and W1 give a height for free, and so do the Disney shape functions ([Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)). Freckles and mottling give no height.

The layer sits above the base tone and the shading in the stack, and below the lips, brows, makeup and decals (the order of part C of the roadmap).

### What the player sees

The names below are for the player.
The last column says which method moves.

| Control | Range | What it changes | Method and parameters |
|---|---|---|---|
| Age | 15 to 90 years | Turns the wrinkle sets on in the order of the table below, deepens them, and raises the number of pigment spots | W1 weights per set, F1 age spots |
| Wrinkle depth | 0 to 2 times | Strength of all creases at the current age | Multiplies the amplitude of W1 |
| Wrinkle sharpness | 0.5 to 2 times | How narrow the furrows are | Multiplies the width `w` of W1 |
| Frown or smile lines | −1 to 1 | Shifts the weight from the forehead and glabellar sets to the crow's feet and nasolabial sets | Weights of the W1 sets, in the way of Oat's −1 to 1 weights ([Oat 2007](https://www.advances.realtimerendering.com/s2007/Oat%20-%20Wrinkles%20-%20Course%20notes%20(SIGGRAPH%202007).pdf)) |
| Pore size | 0.5 to 3 times | Radius of the visible pores | `r0` of P1 |
| Pore amount | 0 to 1 | How many pores are visible | Maps to 10 to 80 per cm² on the cheek ([Flament et al. 2015](https://www.dovepress.com/facial-skin-pores-a-multiethnic-study-peer-reviewed-fulltext-article-CCID)) |
| Oily T-zone | 0 to 1 | How much more the nose and forehead show pores than the cheeks | The region factor `f` of P1 |
| Skin coarseness | 0 to 1 | Amplitude of the fine grain and darkening of the pores | `a` of P1 and the grain noise |
| Freckles | 0 to 1 | Amount of freckles | `amount` of F1 |
| Freckle size | 0.5 to 2 times | Radius of the freckles | Radius of F1 |
| Freckle area | nose, cheeks, whole face | Where freckles occur | The mask `f_sun` of F1, as in Reallusion's types ([Reallusion manual](https://manual.reallusion.com/character-creator-4/Content/ENU/4.0/15_Substance/Basic_Modifications_for_Head_Texture.htm)) |
| Moles | 0 to 8 | Count of moles, placed by seed or by the player | Seeded points of F1 |
| Blotchiness | 0 to 1 | Uneven red and brown tone | Mottling noise |
| Redness and veins | 0 to 1 | Flushed cheeks and nose, fine veins | Blemish threshold noise and ridged noise, masked to the cheeks and nose |
| Lip lines | 0 to 1 | Vertical lines on the lips | L1 |
| Seed | integer | Changes every random choice at once | Seed of all hashes |

The roadmap's decision 31 wants every slider-driven control to be fitted, and the roadmap's ordering for this layer is the fourth step of part C.
The controls are named so that a player can use them without knowing the methods.

### Wrinkle sets and their age curves

Each set is one or more landmark curves with the profile of W1.
The order follows the correlations of Flament and colleagues.
The onset ages are my design values, anchored on three published facts: crow's feet reach their final number by about age 35, glabellar lines are not seen before 40, and forehead lines correlate weakest with age.

| Set | Curve between landmarks (my reading of the crease names) | Weight starts at age | Weight is full at age | Published anchor |
|---|---|---|---|---|
| Crow's feet | Three to five lines fanning from the outer eye corner towards the temple | 25 | 60 | Correlation 0.602 ([Flament et al. 2013](https://pmc.ncbi.nlm.nih.gov/articles/PMC3790843)). Depth grows to eight times by age 65 (search summary of a [replica study](https://library.scconline.org/v038n03/2)) |
| Under-eye lines | One crescent along the lower eye rim, and the infra-orbital crease below it | 30 | 70 | Correlation 0.546 ([Flament et al. 2013](https://pmc.ncbi.nlm.nih.gov/articles/PMC3790843)) |
| Nasolabial folds | From the nose wing to beside the corner of the mouth | 25 | 70 | Starts at the nose wing and ends beside the corner of the mouth ([creases review](https://researchonline.ljmu.ac.uk/id/eprint/6404/1/facial%20creases%20review.pdf)) |
| Marionette lines | From the corner of the mouth down to the chin border | 40 | 80 | Correlation 0.520 for mouth corner lines ([Flament et al. 2013](https://pmc.ncbi.nlm.nih.gov/articles/PMC3790843)) |
| Glabellar lines | One to three vertical lines between the brows | 40 | 70 | Not seen before age 40 (search summary of a fringe-projection study) and correlation 0.553 ([Flament et al. 2013](https://pmc.ncbi.nlm.nih.gov/articles/PMC3790843)) |
| Forehead lines | Two to five horizontal lines above the brows | 30 | 75 | Weakest correlation, 0.335 ([Flament et al. 2013](https://pmc.ncbi.nlm.nih.gov/articles/PMC3790843)). Cadaver Rmax is 0.24 to 0.32 mm ([JAMA Dermatology](https://jamanetwork.com/journals/jamadermatology/fullarticle/1105207)) |
| Upper lip lines | Short vertical lines above the lip border (method L1) | 40 | 75 | Correlation 0.558 ([Flament et al. 2013](https://pmc.ncbi.nlm.nih.gov/articles/PMC3790843)) |
| Neck lines | Horizontal bands on the neck | 35 | 80 | I found no number for the neck |

The weight of a set is `smoothstep(start, full, age)`.
The depth and the number of lines of the set both scale with the weight, and each line's own random length and strength come from the seed.
The depth at full weight is `d_full`. For the forehead and the eye corner, the cadaver values of 0.24 to 0.32 mm and 0.56 to 0.79 mm are the only published ones, so they set the ratio between the sets: the crow's feet are about 2.3 times as deep as the forehead lines (0.56 / 0.24 and 0.79 / 0.32).
For a colour layer these depths only fix the relative darkening between sets.

Heavy ageing also needs sagging and folds (the table's lower face ptosis, with a correlation of 0.764). Those change the shape, which is the job of the shape sliders in part B, so this layer does not paint them.

### Evaluation order and cost

For each pixel in the face mask.
1. Compute the 3D point, the normal, the interpolated vertex fields and the region masks.
2. Evaluate the wrinkle sets, whose weights are above zero, as a distance to their curves. This is a few distances per set.
3. Evaluate the pore Worley noise with 27 cells. Where the cell edge is at least twice the pore radius, 8 cells are enough. Evaluate the freckle noise with 27 cells at the larger cell size.
4. Evaluate the mottling and the blemish noise, which are plain 3D noise calls.
5. Combine the values into the four outputs.

My estimate for the cost. With 4 million face pixels, 4 samples per pixel and about 60 hash calls per sample, there are about 1 billion hash calls, which should take a few seconds on one core and well under a second on many cores. I did not measure it. The baker runs once, when a slider changes, and the pore and freckle point sets do not change when only the age moves, so they can be cached.

### How to fit it to KOTOR's textures

This follows the method of part C: bake KOTOR's texture into our layout, and fit our parameters on each of the 57 heads.
The steps are my proposal.
1. **Find out whether KOTOR has pores at all.** Compute the energy of the bake's high-pass image (the image minus a blur of 1 to 3 pixels) in the cheek, nose and forehead regions. If it is close to zero, set the pore amount near zero for KOTOR's look, and keep the layer for the variety that mods show.
2. **Fit pore settings** by matching the energy and the number of dark dots per cm² that a blob detector finds, as the "pore amount" and "pore size". Compare nose, forehead and cheek to fit the T-zone factor.
3. **Fit the creases** with a ridge filter on the bake. Place the landmark curves on the ridges that the filter finds in the old heads (the roadmap's note on KOTOR meshes lists the wrinkles, nasolabial folds and under-eye bags of `old_*`, Vrook and Jorran, in `2026-11-18-kotor-head-meshes.md`). Then fit `d` and `w` from the average cross-section of the darkening along each curve, using the profile of W1.
4. **Fit the age curve** by giving each head an age value and checking that the sets turn on in the order of the table above. If KOTOR's old heads contradict the order, change the onset ages.
5. **Fit freckles and blemishes** with a blob detector on the melanin and haemoglobin maps, if those maps are computed for the bakes.
6. Measure the colour difference, sharpness and presence of each feature, as the roadmap asks, and look at pictures before trusting the numbers.

## What I could not find out

- **Pore size by facial region.** I found measured pore size and density only for the cheeks, and only the order (nose and forehead above cheeks) for the other regions. The only per-region number is the follicle density, 1,200 per cm² at the nose and 460 per cm² beside the ear, which I read in a textbook chapter and not in the original paper. I could not read the numbers behind the abstract of the 2018 region study.
- **Freckle and mole numbers.** I found no published freckle density per cm², no size distribution and no mole size. Every freckle parameter in this note is a design value or comes from a game's controls.
- **Wrinkle depth and width by age and site.** The cadaver study gives depths for two sites and no age trend. The replica studies that give an age trend (crow's feet depth rising eightfold by age 65, onset of glabellar lines at 40) came to me only as search result summaries, and I could not read the papers. I found no width numbers by site and no numbers for nasolabial folds, marionette lines or neck lines.
- **Colour of pores and creases.** No source gives how much darker or redder a pore or a crease is in a colour texture. Reallusion's and Baldur's Gate 3's images are authored. The darkening values in this note must come from the KOTOR fit.
- **How shipped creators map sliders to detail.** Reallusion SkinGen, Black Desert, the Bethesda creators and the EA creators publish no internals for their age, wrinkle, pore and freckle controls. Baldur's Gate 3's channel layout is documented only by a fan wiki. Texturing.xyz does not say how SKAP makes pores.
- **The Substance and Houdini tools.** I found no official skin generator in Substance Designer or Painter. The Houdini pore tool and the SIGGRAPH 2024 talk are known to me only from their abstracts, because the details are in a video and a paywalled talk.
- **Papers I could not read in full.** Boissieux et al. 2000, Wu et al. 1996, Larboulette and Cani 2004, Haro et al. 2001, Kim et al. 2015, Schneider's thesis, and the Takema replica study. I used abstracts, summaries and later papers that cite them.
- **"Li et al."** I could not tell which wrinkle paper the task meant. The only Li work I met is the anisotropic sampling paper that Weiss et al. cite.
- **Whether the 3D Worley slab rule looks right.** The disc-radius argument, the thinning at the feature point and the normal-aware distance are my own ideas. They are not published, and nobody has tested them on our head. The first step of the work should be a picture of them on the base head, next to a KOTOR bake.
- **Mesh coarseness.** I did not check the triangle size of our current head mesh, so I could not say whether per-vertex distance fields are fine enough for 1 to 3 mm wide creases.
