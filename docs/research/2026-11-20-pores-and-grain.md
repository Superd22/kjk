# Pores and grain

This note was researched on 2026-11-20 for kjk's head paint track, which is part C of the roadmap (`2026-11-19-head-roadmap.md`).
David looked at the painted heads and found that the pores are "just random noise".
He asked two things.
- Is there a better state of the art for drawing pores and the fine grain of the skin?
- Is our rule of more pores on the nose and forehead and fewer on the cheeks the real distribution?

The note starts where `2026-11-19-procedural-skin-detail.md` stops.
That note already gives the basic sizes, the 3D Worley method (P1), Poisson-disk points (P2), scanned exemplars (P3), the Disney trail simulation (W3) and the tool survey.
This note does not repeat them.
It takes that note's list of things it could not find out as its first list of questions.

Terms used in this note.
- **A pore** is the opening of a hair follicle on the skin. Every follicle has an opening, but most are too small to see.
- **Follicle density** counts every follicle opening, visible or not. It is measured by gluing a slide to the skin and counting the plugs pulled out (cyanoacrylate biopsy).
- **Visible pore density** counts only the openings above a size threshold. Each study picks its own threshold. The common ones are an area of 0.02 mm², an area of 0.04 mm², or a diameter of 250 µm (about 0.05 mm²).
- **The grain** is the fine relief between the pores. It is a network of shallow furrows that cut the skin's surface into small plateaus.
- **Primary lines** are the deeper furrows of that network, and **secondary lines** are the shallower ones that cross them.
- **A plateau** is one of the small raised fields enclosed by the furrows. Japanese papers call it a crista cutis, or "hill".
- **Anisotropy** is how much the furrows share one direction. The anisotropy index used below is the share of the relief's spectral power that lies within 10 degrees of its strongest direction. It is 11% for a pattern with no direction and 100% for parallel lines.
- **The flow** is the main direction of the fine furrows at a point of the face. `pores.rs` already computes one for the fine lines, in `PoreLandmarks::flow`.
- **F1 and F2** are the distances from a point to its nearest and second nearest feature point in Worley noise. **F2 − F1** is small near the border between two cells, so it draws the cell walls.
- **High-pass** means an image minus a blurred copy of itself. It keeps only the small details.
- **Skew** is the third moment of a set of values. A band made of dark dots on a plain field has a negative skew, and noise that is as often light as dark has a skew near zero.
- **L\*** is the lightness of the CIELAB colour space, from 0 (black) to 100 (white).

Every claim carries a link.
Where only an abstract or a summary could be read, the sentence says so.
Numbers read off a plotted figure, and not from a table, are marked "read from a figure".
A sentence with "I infer", "my estimate" or "probably" is my own reasoning and not a fact from a source.
The papers that were downloaded are in `/private/tmp/claude-501/-Users-david-perso-kjk/c3910254-b661-450a-9003-645d2158b8cb/scratchpad/papers/`, in the folders `derm`, `relief` and `pipelines` and at the top level.

## Summary

**Is there a better way than our noise?** Yes.
- Our grain is two octaves of Perlin noise with no link to the pores. Perlin noise is, by design, random noise with no structure. So what David sees is exactly what the code draws.
- With the shipped default settings, the pores are almost invisible. The default has no pores on the cheeks at all (`t_zone` 1.0) and makes each pore 1.2 L\* *lighter* than the skin (`depth` −1.2). So on the default head the visible fine texture is nearly all the Perlin grain.
- Every serious method published since 2019 treats pores and grain as one structure. The pores are the nodes of a network, and the fine furrows run between neighbouring pores along the flow. This is how Disney's research does it ([Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)), how Weta grew the pores of Gemini Man ([Weta patent US 11,461,968](https://patents.google.com/patent/US11461968)), and what dermatology describes, with pores "at the intersections of the furrows" ([Arakawa et al. 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/3/41_3_173/_pdf)).
- Real pores are not round. On the cheek their long axis is 1.7 to 2.4 times their short axis, and they line up along one direction that runs from the outer eye corner down towards the ear ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf), [Mizukoshi 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/4/41_4_262/_article/-char/ja/)). Our pores are round.
- The full Disney method needs a graph simulation and cannot run per pixel. Its result can be approximated per pixel. Scatter nodes on a 3D grid, as the pores are scattered today, and measure distances in a metric stretched along the flow. Draw short furrows between each pixel's nearest nodes, and draw the visible pores on a share of those nodes. This costs about the same number of hash calls as today.
- One limit is hard. The real furrow network on an adult cheek has a spacing of 0.24 to 0.28 mm ([Hayashi et al. 1989](https://www.jstage.jst.go.jp/article/sccj1979/23/1/23_1_43/_pdf), read from a figure), which is only 2 pixels in our texture. At its true size the cheek's network cannot be drawn as lines. It can only be drawn at a coarser scale, as the deeper families of lines on the forehead, the temples and older skin, or exaggerated, as painters do.

**Is our distribution right?** Partly.
- The order "nose and forehead above the cheeks" is supported. Follicle density falls from the middle of the face to its sides ([Pagnoni et al. 1994](https://pubmed.ncbi.nlm.nih.gov/7857840/), abstract only). Enlarged pores are more prominent on the nose and forehead than on the cheeks ([Jung et al. 2018](https://pubmed.ncbi.nlm.nih.gov/29393550/), abstract only).
- "The cheek" is two regions, and our mask treats it as one. The upper cheek beside the nose has pores about twice the area of those on the lower cheek of the same face ([Yamashita et al. 2010](https://www.jstage.jst.go.jp/article/sccj/44/3/44_216/_pdf/-char/en)). Almost every cheek study, including the 2,585-woman study, measures this upper inner zone ([Flament et al. 2015](https://pmc.ncbi.nlm.nih.gov/articles/PMC4337418)). It has 10 to 90 visible pores per cm².
- Our default of no pores at all on the cheeks contradicts every study.
- The chin changes with age. It is like the lower cheek in the teens and has more and larger pores than the middle of the cheek by the 60s ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf), read from a figure). Our mask puts the chin in the T-zone at every age.
- The temples and the skin around the eyes have few visible pores. No study measures them directly, so this is an inference from the follicle gradient and from the eyelid's tiny follicles. The lip's red part has no follicles at all ([Ackerley et al. 2017](https://pmc.ncbi.nlm.nih.gov/articles/PMC5451466)). Our mask already clears the lips.

**The top three changes**, explained under "Recommendations" below.
1. Replace the Perlin grain with a furrow network anchored on the pores and stretched along the flow.
2. Replace the T-zone Gaussian with a region map of pore density and pore size, built from the measured numbers.
3. Stretch each pore along the flow, give it a dark and slightly red core with a faint light rim, and remove grain detail finer than two pixels.

## 1. Techniques

The 2026-11-19 note covers Worley noise, Poisson-disk points, scanned exemplars and the Disney trail simulation.
This section adds what was missing from it: the methods' details about pores, the region-by-region findings, and the methods that can be turned into a per-pixel function.

### Graham and colleagues 2013, scanned microstructure per region

Graham and colleagues pressed skin against a 24 mm × 16 mm opening and photographed it at about 7 µm per pixel under polarised gradient light. They turned the normals into displacement maps ([Graham et al. 2013, sections 1 and 3](https://vgl.ict.usc.edu/Research/Microgeometry/Measurement_Based_Synthesis_of_Facial_Microgeometry.pdf)).
They scanned the forehead, temple, cheek, nose and chin of a Caucasian man and an Asian woman.
They write that "the skin microstructure is highly differentiated across both individuals and facial regions".

The paper's text says nothing more per region. Its figure 3 shows the patches, and the research agent read them as follows (my reading of a figure, not the authors' words).
- On the forehead, the furrows run mostly in one direction on one subject and form a grid of straight lines on the other.
- On the temple, both subjects show a clear polygon network, with dark pores at many of the corners.
- On the cheek, the relief is fine and low in contrast, with only a weak direction.
- On the nose, large, round, deep pores dominate. On one subject, small ridges radiate out from each pore.
- On the chin, one subject shows two families of long diagonal furrows that cross each other.

The synthesis grows a 4K face displacement map to 16K, region by region, with Image Analogies. An artist splits the face into forehead, temples, nose, cheeks and chin, and the overlapping regions are blended linearly. The matching window is 5 × 5 pixels at 4K and 13 × 13 pixels at 16K, and a weight α of about 0.5 balances the exemplar against the face's own mid frequencies ([Graham et al. 2013, section 4](https://vgl.ict.usc.edu/Research/Microgeometry/Measurement_Based_Synthesis_of_Facial_Microgeometry.pdf)).

Two points matter for us.
- The diffuse albedo and the subsurface normals stay at 4K. Only the specular normal and specular albedo go to 16K. So in their pipeline the microstructure is a specular effect, not part of the skin's colour.
- When they blurred the 16K patches, the fitted specular roughness rose towards the value fitted at 4K. They conclude that "at sufficiently high resolution, the surface microgeometry variation is responsible for the appearance of specular roughness". Detail finer than the texture's pixel therefore belongs in roughness, which we do not have.

It needs scans and an offline texture synthesis. It cannot run per pixel from hashes.

### Nagano and colleagues 2015, microstructure under stretch

Nagano and colleagues measured skin patches of 8 mm at 6 µm per pixel while stretching and compressing them ([Nagano et al. 2015](https://vgl.ict.usc.edu/Research/SkinStretch/skinstretch_SIG15p.pdf)).
Their observations are useful even without their filter.
- Skin that stretches and compresses a lot is rougher than skin that stays still. The forehead and the skin around the eyes are rougher. The tip of the nose is smoother.
- At rest, the forehead's microstructure is "relatively isotropic". When the brow is raised, horizontal ridges less than 0.1 mm across form. When the brow is knitted, vertical ones form.

The method convolves the displacement map with a 1D kernel along each strain axis, `k = (1 − α) δ + α G(σ)`. On the forehead, the fitted values are `α = min(1, 15.4 r − 13.8, 3.09 (r − 1))` and `σ = (38.2 r − 26.5)` µm for a stretch ratio `r` of 1 or more, and `σ = (70.5 r − 46.9)` µm below 1 ([Nagano et al. 2015, equations 6 to 9](https://vgl.ict.usc.edu/Research/SkinStretch/skinstretch_SIG15p.pdf)).
The σ values of 6.5 to 24 µm are far below our pixel. The method needs a displacement map and runs per frame on a GPU. For us only the observations matter.

### Golovinskiy and colleagues 2006, detail statistics per region

Golovinskiy and colleagues scanned 149 people aged 15 to 83 at a sample spacing of about 0.5 mm ([Golovinskiy et al. 2006](https://pixl.cs.princeton.edu/pubs/Golovinskiy_2006_ASM/facePaper.pdf)).
- They cut the 1024 × 1024 displacement image into 16 × 16 tiles. For each tile, they split the detail with a steerable pyramid of 4 scales and 4 directions, plus a high-pass band. That gives 17 bands.
- They found that the histograms of the same tile on different faces differ mainly in width. So each band of each tile is described only by its standard deviation, which makes 4,352 numbers per face.
- To make a new face, they match each band's standard deviation, tile by tile, with bilinear blending between tiles. One matching pass per iteration was enough.

Findings.
- Roughness grows faster after 45. Around that age, directional wrinkles appear at the eye corners, the mouth, parts of the cheeks and the forehead.
- Men aged 20 to 30 have more high-frequency detail than women of the same age, mostly on the chin and the lower cheeks. The authors put this down to stubble and hair follicles.
- Plain Perlin noise added to a face looked "too uniform", and noise matched by the pyramid "does not produce long, correlated structures".

The method is an iterative pyramid synthesis and cannot run per pixel. Its idea of a band strength and a direction per region is cheap to copy, as a region map of amplitudes.

### Ishii and colleagues 1993, hierarchical Voronoi furrows

This is the classic procedural model of the furrow network. The paper itself is not online, so everything here comes from later papers that describe it.
- Ishii and colleagues "divided skin surface into polygons by hierarchical Voronoi division, and rendered each edge of these polygons as a furrow" ([Bando et al. 2002, section 2](https://web.media.mit.edu/~bandy/wrinkle/PG02wrinkle.pdf)).
- Each polygon gets a curved surface that represents the ridge, and the model "partially took wrinkle directions into account" ([Bando's bachelor thesis, p. 1](https://web.media.mit.edu/~bandy/thesis/bachelor.pdf)).
- Weiss and colleagues say that this model suits "skin areas exhibiting isotropic stress, e.g. the forearm, less so for areas with anisotropic stress" ([Weiss et al. 2023, section 2](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)).

No secondary source gives Ishii's seed densities, depths or ridge profile.

A documented anisotropic version is the sketch by Miyata, Itoh and Shimada ([Miyata et al. 2001](https://www.andrew.cmu.edu/user/shimada/papers/01-siggraph-sketch-miyata.pdf)).
1. Pack elliptic particles of the wanted size, stretch and direction.
2. Connect their centres into a mesh and take its dual, which gives Voronoi-like cells stretched along the flow.
3. Turn each cell into a tapered prism, skew its top corners along the flow, smooth it, and add fractal noise for small bumps.

It took about 20 seconds for a 512 × 512 texture in 2001. It is mesh-based, but its key idea, cells measured in a metric stretched along the flow, works per pixel.

### Weiss and colleagues 2023 and 2024, details about pores

The 2026-11-19 note describes the trail simulation. These are the details about pores it did not give ([Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf), [supplementary](https://studios.disneyresearch.com/app/uploads/2023/07/SkinMicroWrinkles-Supplementary.pdf)).
- **Wrinkles end at pores.** "Fine wrinkles on the human skin always meet at pores", and the model enforces it, because every wrinkle is an edge between two pore nodes.
- **Not every node shows as a pore.** A node's depth comes from the edges that meet at it: `w_v = max(w_max + α_dmin, w_max + α_blend (w_sum − w_max))`, where `w_max` is the strongest edge at the node and `w_sum` the sum of its edges. The pore's radius is `w_v × α_pore-width`. So a pore where several strong furrows meet is deep and wide, and a node with weak edges is barely there.
- **The pore's shape** is the same profile as a wrinkle, as a function of the distance from its centre. With `x` the distance divided by the width, the profile with a rim is `(x − 3)² (x − 1) / 9` for `x` from 0 to 3. It is a dent from 0 to 1 and a low rim from 1 to 3.
- **Pores stretch along the flow.** A parameter `α_skew` from 0 to 1 stretches each pore along the main direction. The fits to scans used large values, and the face of figure 16 uses 0.741.
- **Pore spacing.** The face results use a node spacing of 0.002, 0.001 and 0.0005 of the UV square on three levels. The maps are 16K with a mean pixel of 0.02 mm, so the UV square spans about 330 mm. My estimate of the spacings is therefore 0.66 mm, 0.33 mm and 0.16 mm.
- **What matters most.** In the supplementary's sensitivity study, the pore spacing `α_d` matters far more than any other parameter. It ranks at 100%, and the next ones rank at 2% to 9%. The direction comes next.

The follow-up paper ran a user study ([Weiss et al. 2024](https://studios.disneyresearch.com/app/uploads/2024/06/Stylize-My-Wrinkles-Bridging-the-Gap-from-Simulation-to-Reality-Paper.pdf)).
- 58 people judged 56 pairs of grey renders of the forehead, cheek, nose and crow's feet of 4 people.
- On a PageRank score where lower is better, the simulation with a different preset per region scored 0.168. One setting for the whole face scored 0.209. Copying the nearest scanned patch scored 0.435.
- Settings tuned for the forehead made "the rest of the face lack realism", most clearly on the nose and the cheek.
- Two levels of network were best. One level lacked detail, and a third level added nothing visible. The second level's spacing is half the first's.

### Weta's pores for Gemini Man

Weta grew the young Will Smith's pores procedurally.
In an interview, Weta's Guy Williams says the system "used flow maps, because pores obviously have a flow and structure to them", that "pores are sort-of shaped like diamonds", and that wrinkles then formed "along pore lines" ([Before & After interview](https://beforesandafters.com/2019/10/11/procedural-pores-modeling-teeth-messy-fighting-and-other-new-things-weta-digital-did-to-turn-will-smith-into-a-23-year-old/)). This is a journalist's interview, not a talk.
The patents describe the same system ([US 11,055,911](https://patents.google.com/patent/US11055911), [US 11,461,968](https://patents.google.com/patent/US11461968)).
1. Place pores from a density, depth, diameter, shape and distribution, which can differ per face region.
2. Let artists draw flow lines.
3. Join each pore to nearby pores, within a distance or a fixed number of neighbours, by curves that follow the flow.
4. Turn those curves into furrows and then into a displacement.

The patents give no numbers.

### Activision's sinusoid tile

Activision's GDC 2013 talk observed that the microstructure in Graham's scans "look[s] pretty much sinusoidal". They built a 1024 × 1024 height tile from "100 radial sinusoidal generators", each a sine wave around a random centre with a random frequency, and turned it into a tiling normal map ([Jimenez, von der Pahlen and colleagues, GDC 2013 slides with notes](https://www.iryoku.com/downloads/Next-Generation-Character-Rendering-v6.pptx), [Digital Ira and Beyond course notes](https://vgl.ict.usc.edu/Publications/2014/Digital%20Ira%20and%20Beyond%20Creating%20Photoreal%20Real-Time%20Digital%20Characters.pdf)).
The pores themselves were a cavity map from ZBrush, used to darken the specular reflection.
A sum of a few radial sinusoids around hashed centres is cheap per pixel. It gives a ripple, not a network, so I infer that it suits the sub-pixel band less well than the cell walls described below.

### Other recent work

- **Huynh and colleagues 2018** train a network to predict displacement from a 1K albedo ([Huynh et al. 2018](https://openaccess.thecvf.com/content_cvpr_2018/papers/Huynh_Mesoscopic_Facial_Geometry_CVPR_2018_paper.pdf)). It learned to tell pores, stubble, wrinkles and moles apart "based on their location on the face". It needs a network.
- **FLEX 2025** replaces Weiss's simulation with a graph neural network fitted to one photograph ([Du et al. 2025](https://icewired-yy.github.io/papers/face_micro_detail_synthesis/pdf/Facial%20Microscopic%20Structure%20Synthesis%20from%20a%20Single%20Unconstrained%20Image.pdf)). Its authors note that the nose tip shows raised bumps from sebaceous filaments, which a model of dents alone cannot make.
- **VISAGE** (2026) uses Blender-style nodes. Its pores are Perlin noise at scale 350, and its wrinkles are a Voronoi distance-to-edge texture on coordinates stretched by 1.5 and 3.0 ([VISAGE, arXiv 2611.04712](https://arxiv.org/html/2611.04712), read through a fetch tool only). The authors admit that "pore-level detail as carried by 4K textures remains out of reach." This is the cheap per-pixel approach, and it is close to what we have.
- **Iglesias-Guitian and colleagues 2015** model ageing skin's optics, not its relief. They quote a peak-to-valley roughness of 83 µm in the young and 151 µm in the elderly ([Iglesias-Guitian et al. 2015](https://graphics.unizar.es/papers/Iglesias_eg15.pdf)).
- **Klintberg and Haapaoja 2024** present a procedural skin generator in Houdini whose pores and wrinkles react to stretch ([SIGGRAPH 2024 talk](https://history.siggraph.org/?p=160660)). Only the abstract is public.

### What shipped games do

Every shipped pipeline I could check from primary sources keeps pores in a tiling detail normal map or a cavity map, and not in the colour.
- Epic's documentation says: "The Normal Map that is used for the face should only contain details about wrinkles. Pore details should be done with a tiling texture" ([Unreal, Creating Human Skin](https://dev.epicgames.com/documentation/unreal-engine/creating-human-skin-in-unreal-engine)).
- MetaHuman's base colour holds "skin texture, skin accents, make-up, freckles". At its LOD 2, where the micro normals are gone, "AO is applied to the base color to help fake shadowing" ([MetaHuman materials and textures](https://dev.epicgames.com/documentation/metahuman/metahuman-materials-and-textures)). This is the shipped case closest to ours.
- Unity's digital human shader repeats its detail normal 25 times across the UV square by default, with a strength of 0.3, and masks it per region with one channel of a mask map ([Unity digital human package](https://github.com/Unity-Technologies/com.unity.demoteam.digital-human), `ShaderLibrary/DigitalHuman Skin (2019).shadergraph`). Its cavity map changes only the smoothness and the specular occlusion.
- Texturing.xyz sells micro skin tiles of about 1 × 1 cm at 1024 pixels, and of 1.4 × 1.4 cm at 2048 pixels. They hold cavity, height, normal and roughness maps and no colour map ([MicroSkin 1.0](https://texturing.xyz/collections/microskin/products/microskin-face-cheek-01), [MicroSkin 2.0](https://texturing.xyz/collections/microskin-2-0/products/microskin-forearm-26)). One of their tiles would cover only 70 to 130 of our pixels.

### Which parts survive in our texture

One pixel of our texture is 0.11 to 0.14 mm.
A feature needs about two pixels, 0.22 to 0.28 mm, to show at all.

| Feature | Real size | In our pixels | Can we draw it? |
|---|---|---|---|
| Visible pore, Japanese cheeks at the 0.02 mm² threshold | 0.025 to 0.06 mm², so 0.18 to 0.28 mm across ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf), [Yamashita et al. 2010](https://www.jstage.jst.go.jp/article/sccj/44/3/44_216/_pdf/-char/en)) | 1.3 to 2.5 | Only as a soft speck |
| Enlarged pore, Caucasian and Japanese cheeks | 0.14 to 0.22 mm², so 0.42 to 0.53 mm across ([Flament et al. 2015, table 4](https://pmc.ncbi.nlm.nih.gov/articles/PMC4337418)) | 3 to 5 | Yes |
| Spacing between enlarged pores | 60 to 90 per cm², so 1.05 to 1.3 mm apart ([Flament et al. 2015](https://pmc.ncbi.nlm.nih.gov/articles/PMC4337418)) | 8 to 12 | Yes |
| Furrow spacing on an adult cheek | 0.235 to 0.275 mm ([Hayashi et al. 1989](https://www.jstage.jst.go.jp/article/sccj1979/23/1/23_1_43/_pdf), read from a figure) | about 2 | No, only as a texture |
| Plateaus on an adult cheek | 160 to 180 per cm² ([Masuda's thesis, chapter 4](https://muroran-it.repo.nii.ac.jp/record/8950/files/B80.pdf), read from a figure) | about 6 pixels across | At the limit |
| Closed polygons on an adult forearm | 1.0 per mm² ([Trojahn et al. 2015](https://doi.org/10.1111/srt.12158), abstract only) | about 8 across | Yes |
| Furrow width on the cheek | 40 to 43 µm ([Masuda's thesis](https://muroran-it.repo.nii.ac.jp/record/8950/files/B80.pdf)) | 0.3 to 0.4 | No, only as a darker average |
| Disney's coarsest network level | about 0.66 mm (my estimate from the UV units) | about 5 | Yes |

The pixel counts are my own arithmetic.
I infer three rules from this table.
- Pores and the coarser level of the network can be drawn as shapes.
- The fine cheek network can be drawn only as a texture at its two-pixel limit, or at an exaggerated size.
- Anything finer than two pixels must not be drawn per pixel, because it turns into exactly the random noise David sees. It can only change the average colour.

## 2. The real distribution of pores

### What the numbers count

The studies use different thresholds, and the counts move a lot with the threshold.

| Study | Threshold | Method | Link |
|---|---|---|---|
| Flament 2015, 2,585 women in 5 countries | Diameter of 250 µm or more, about 0.05 mm² | Polarised dermatoscope at 25 µm per pixel | [full text](https://pmc.ncbi.nlm.nih.gov/articles/PMC4337418) |
| Yamashita 2010, 90 Japanese women | 0.04 mm² or more counts as enlarged | Video microscope at 20 times, 1 cm² field | [full text](https://www.jstage.jst.go.jp/article/sccj/44/3/44_216/_pdf/-char/en) |
| Nishijima 2001, 98 Japanese women | 0.02 mm² or more | Replica, 6 × 6 mm field | [full text](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf) |
| Iida 2016, 23 Japanese adults | 0.02 mm² or more, measured 40 µm below the surface | Replica under a confocal microscope | [full text](https://www.jstage.jst.go.jp/article/koshohin/40/1/40_1/_pdf/-char/ja) |
| Lee 2022, 101 Korean women | 0.02 mm² or more | 3D fringe projection at 20 µm | [full text](https://pmc.ncbi.nlm.nih.gov/articles/PMC9292708) |

### Region by region

| Region | Follicle density (all openings) | Visible pore density | Visible pore size | Shape |
|---|---|---|---|---|
| Nose | Highest at the nose tip in a Korean study of 24 women ([1998 study](https://search.bvsalud.org/gim/resource/en/wpr-162757), search summary only). No primary number per cm² was found. The 1,200 per cm² of the earlier note comes from a textbook | "More prominent" than on the cheeks ([Jung et al. 2018](https://pubmed.ncbi.nlm.nih.gov/29393550/), abstract only). About 70 to 75% of Japanese women from their 20s on have prominent nose pores, against about 25% of teenagers ([Mizukoshi 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/4/41_4_262/_article/-char/ja/), read from a figure) | No measured area found | Round and deep in Graham's scans (my reading of a figure). Often plugged by a hardened plug of sebum in the 20s and 30s ([Mizukoshi 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/4/41_4_262/_article/-char/ja/)) |
| Forehead | 285 ± 84 per cm², the densest of 9 body sites ([Ackerley et al. 2017](https://pmc.ncbi.nlm.nih.gov/articles/PMC5451466)). Highest of the sites studied ([Otberg et al. 2004](https://pubmed.ncbi.nlm.nih.gov/14962084/), abstract only). Lower in Asian and African-American people than in white people ([Mangelsdorf et al. 2006](https://pubmed.ncbi.nlm.nih.gov/16679817/), abstract only) | "More prominent" than on the cheeks ([Jung et al. 2018](https://pubmed.ncbi.nlm.nih.gov/29393550/), abstract only) | No measured area found | Not measured |
| Upper cheek beside the nose | Not measured separately | 10 to 80 per cm² at 250 µm in all women, and 20 to 90 per cm² in women with enlarged pores ([Flament et al. 2015](https://pmc.ncbi.nlm.nih.gov/articles/PMC4337418)). About 12 per cm² at 0.02 mm² on the upper cheek of Japanese women, with no change with age ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf), read from a figure) | 0.033 to 0.062 mm², about twice the lower cheek of the same face ([Yamashita et al. 2010, table 1](https://www.jstage.jst.go.jp/article/sccj/44/3/44_216/_pdf/-char/en)). 0.14 to 0.37 mm² in women with enlarged pores outside China ([Flament et al. 2015](https://pmc.ncbi.nlm.nih.gov/articles/PMC4337418)) | Long axis 1.8 times the short one, rising to 2.1 with age ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf), read from a figure) |
| Middle of the cheek | Not measured | About 25 per cm² at 0.02 mm², with no change with age ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf), read from a figure). 52 per cm² on average in Korean women ([Lee et al. 2022](https://pmc.ncbi.nlm.nih.gov/articles/PMC9292708)) | 0.035 rising to 0.044 mm² with age ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf)). Depth of 140 µm on average, from 91 to 211 µm ([Iida et al. 2016](https://www.jstage.jst.go.jp/article/koshohin/40/1/40_1/_pdf/-char/ja)) | 1.9 rising to 2.2 |
| Lower cheek beside the mouth corner | Not measured | About 17 rising to 33 per cm² with age ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf), read from a figure) | 0.018 to 0.033 mm² ([Yamashita et al. 2010](https://www.jstage.jst.go.jp/article/sccj/44/3/44_216/_pdf/-char/en)). 0.025 rising to 0.035 mm² with age ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf)) | About 2.1 |
| Chin | Not measured | About 14 per cm² in the teens, rising to about 58 per cm² by 70 (r = 0.668) ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf), read from a figure) | About 0.03 rising to 0.06 mm² | About 2.4 to 2.5, the most stretched of the four sites, with no change with age |
| Around the eyes | Very small hair follicles, more on the outer side of the eyelid than the inner side ([Montagna and Ford 1969](https://jamanetwork.com/journals/jamadermatology/fullarticle/531162), abstract only) | Not measured. One small trial scored the outer eye area as having fewer visible pores than the forehead and nose ([trial of 30 patients](https://pmc.ncbi.nlm.nih.gov/articles/PMC13282088)), which is weak evidence | Not measured | Not measured |
| Lips | The red part of the lip has no hair follicles ([Ackerley et al. 2017](https://pmc.ncbi.nlm.nih.gov/articles/PMC5451466)) | None | None | None |
| Temples | Probably low. The lateral forehead had the lowest follicle count in the Korean study, and follicle density falls from the middle of the face to its sides ([Pagnoni et al. 1994](https://pubmed.ncbi.nlm.nih.gov/7857840/), abstract only). This is an inference | Not measured | Not measured | Not measured |

Sebum output follows a similar order.
In 49 Japanese men, the nose made about 160 µg/cm², about twice the forehead in winter and four times the upper cheek. The chin made about 57 to 70 µg/cm² and the eye corner about 40 to 52 µg/cm² ([Onishi et al. 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/2/41_2_94/_article/-char/ja/), read from a figure).
Pore size grows with sebum output ([Roh et al. 2006](https://pubmed.ncbi.nlm.nih.gov/17034515/), abstract only).

One large study disagrees.
A study of selfies of 1,000,568 Chinese people, scored by software, ranked the cheek worst and the nose least ([Zhou et al. 2024](https://pmc.ncbi.nlm.nih.gov/articles/PMC11411701)). The nose was also where the software agreed least with dermatologists. I infer that flash photography of the nose's curved, shiny surface is the likely cause, but it is still a real counterpoint.

### Shape and direction

- **Pores become elliptical with age.** On the cheek of 248 Japanese women, the long-to-short axis ratio rose from 1.69 in the teens to 1.85 in the 50s ([Mizukoshi 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/4/41_4_262/_article/-char/ja/)). In Korean women, the mean ratio was 2.20, from 1.78 at age 10 to 3.92 in the oldest group, partly because neighbouring pores merge into short grooves ([Lee et al. 2022](https://pmc.ncbi.nlm.nih.gov/articles/PMC9292708)).
- **Pores line up along one direction.** On the upper cheek of older skin, the furrows and pores line up from the outer eye corner down towards the area below the ear, the direction in which the cheek is stretched ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf)). The pores of a 58-year-old man "appear to flow diagonally" ([Iida et al. 2016](https://www.jstage.jst.go.jp/article/koshohin/40/1/40_1/_pdf/-char/ja)). In a person with conspicuous pores, the long axes cluster in one direction. In a person with inconspicuous pores, they are scattered and the pores are rounder ([Koda et al. 2016](https://www.jstage.jst.go.jp/article/sccj/50/4/50_329/_pdf/-char/en), 2 people only).
- **Pores are rounder where the skin is firmer.** Roundness correlates with cheek elasticity at r = 0.521 ([Iida et al. 2016](https://www.jstage.jst.go.jp/article/koshohin/40/1/40_1/_pdf/-char/ja)).
- **A pore has a rim.** The surface around a conspicuous pore shows a hollow and a broken ring of raised skin ([Mizukoshi and Takahashi 2014](https://pubmed.ncbi.nlm.nih.gov/23750904/), abstract only). Conspicuous cheek pores sit in a funnel-shaped dip of the surface ([Nishijima 2009 review](https://www.jstage.jst.go.jp/article/sccj/43/1/43_1_3/_pdf), in Japanese).
- **No study measures a teardrop shape.** The Japanese term "sagging pores" describes pores that are elliptical or linked in a line.

### Sex, age and ethnicity

- **Sex.** Men's pores are larger. In 1,000,568 Chinese selfies, men scored worse than women in every region, by 26% on the nose and 4% on the cheek ([Zhou et al. 2024](https://pmc.ncbi.nlm.nih.gov/articles/PMC11411701)). Male sex predicted pore size in 60 Korean adults ([Roh et al. 2006](https://pubmed.ncbi.nlm.nih.gov/17034515/), abstract only). In 600 Indian adults, pore density was the same in both sexes, but men's pores were larger up to age 55 ([2018 Mumbai study](https://pubmed.ncbi.nlm.nih.gov/29344961/), abstract only). In 23 Japanese adults, my average of the paper's table gives men a pore area of about 6.5 mm² per 45 mm² against 4.6 mm² for women, which the authors call a trend that is not significant ([Iida et al. 2016](https://www.jstage.jst.go.jp/article/koshohin/40/1/40_1/_pdf/-char/ja)).
- **Age.** Children aged about 10 had 22 visible pores per cm² of 0.048 mm². Adults had 50 to 76 per cm² of 0.12 to 0.28 mm² ([Lee et al. 2022](https://pmc.ncbi.nlm.nih.gov/articles/PMC9292708)). On the cheek, size rises until about 40 and then levels off in Caucasian, Japanese and Chinese women, while density hardly changes ([Flament et al. 2015](https://pmc.ncbi.nlm.nih.gov/articles/PMC4337418)). On the upper and middle cheek, pores stop growing at about 40 and stretch instead. On the chin and lower cheek, they keep growing in size and number into the 60s ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf)). Counts rise most between the 30s and the 40s ([Jung et al. 2018](https://pubmed.ncbi.nlm.nih.gov/29393550/), abstract only).
- **Ethnicity.** On the cheek, Chinese women have about 21 pores per cm² of 0.06 mm², covering 1.2 to 1.5% of the skin. Japanese and Caucasian women have 60 to 72 per cm² of 0.14 to 0.22 mm², covering 10 to 13%. Indian and Brazilian women have 71 to 91 per cm² of 0.17 to 0.37 mm², covering 14 to 23% ([Flament et al. 2015, tables 3 and 4](https://pmc.ncbi.nlm.nih.gov/articles/PMC4337418)). These figures are for the women who reported enlarged pores.

### Verdict on our rule

The rule "more pores on the nose and forehead, fewer on the cheeks" is right in its order but wrong in three details.
1. The upper inner cheek beside the nose belongs with the high group. Its pores are twice the area of the lower cheek's, and it is the zone that all the cheek studies measure.
2. The chin belongs with the low group in young people and with the high group in old people.
3. The cheeks should never have zero pores. The lowest measured density on any cheek is about 10 per cm², in Chinese women.

## 3. The grain

### What it is made of

The skin's surface relief is classified into four orders of lines. The classification goes back to Hashimoto 1974, which I could not read. Later papers quote it as follows ([Zahouani et al. 2014](https://iopscience.iop.org/article/10.1088/1742-6596/483/1/012012/pdf), [Gougeon et al. 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC9838641)).

| Order | Depth | What it is |
|---|---|---|
| Primary lines | 20 to 100 µm (30 to 100 µm in Gougeon) | The deepest and widest furrows. The only lines visible to the naked eye |
| Secondary lines | 5 to 40 µm | Shallower lines that run across or diagonal to the primary lines |
| Tertiary lines | about 0.5 µm | The borders of single skin cells |
| Quaternary lines | about 0.05 µm | The texture on each skin cell |

The primary and secondary lines form a criss-cross pattern of polygonal plateaus ([Quatresooz et al. 2006](https://onlinelibrary.wiley.com/doi/full/10.1111/j.1467-2494.2006.00342.x)).
The plateaus are "mainly triangles or quadrangles" ([Gougeon et al. 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC9838641)), or triangles, trapezoids and rhombuses ([Igarashi et al. 2005](https://cave.cs.columbia.edu/old/publications/pdfs/Igarashi_CUTR05.pdf)).
Japanese papers describe two levels. Shallow furrows enclose small hills, and deeper furrows group several hills into larger polygonal fields ([Masuda's thesis, section 2.1.2](https://muroran-it.repo.nii.ac.jp/record/8950/files/B80.pdf), in Japanese).
On the young back of the hand, the lines form "a highly ordered pattern of triangles". In the old, the order is lost because the secondary lines are gone ([Grove et al. 1981](https://library.scconline.org/is-cacheable/1708703862255/Nonintrusive-Tests-Age-Associated-Changes.pdf)).

Bando and colleagues add that fine furrows "run locally in two directions", that a furrow ends where it meets another furrow of the same direction, and that the polygons they enclose are "roughly diamond-shaped" ([Bando et al. 2002](https://web.media.mit.edu/~bandy/wrinkle/PG02wrinkle.pdf)).
Weiss and colleagues say that where the skin's stress has one main direction, the primary wrinkles "form rectangles aligned with that direction", and the secondary wrinkles "criss-cross those primary wrinkles in a more random, diagonal pattern". Where the stress is the same in all directions, the wrinkles form "relatively regular triangular patterns" ([Weiss et al. 2023, section 1](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)).

### Measured sizes on the face

Cheek, 103 Japanese women and girls aged 5 to 85, 3D confocal replicas ([Masuda's thesis, chapter 4](https://muroran-it.repo.nii.ac.jp/record/8950/files/B80.pdf), values read from figures).

| Age | Plateaus per cm² | Furrow length per mm² | Mean furrow width | Mean furrow depth |
|---|---|---|---|---|
| 5 to 9 | about 360 | about 6.0 mm | about 41 µm | about 16 µm |
| 10 to 19 | about 280 | about 5.4 mm | about 40 µm | about 16 µm |
| 20 to 39 | about 180 | about 4.1 mm | about 43 µm | about 19 µm |
| 40 to 59 | about 160 | about 3.8 mm | about 42 µm | about 20 µm |
| 60 to 90 | about 175 | about 3.9 mm | about 41 µm | about 22.5 µm |

The correlations with age are −0.523 for the plateau count, −0.572 for the furrow length, +0.484 for the furrow depth and +0.684 for the pore area.
The author compares the change to a sieve whose mesh gets coarser with age.
For a network of lines with no main direction, the mean spacing between lines is π divided by twice the line length per area. That gives about 0.26 mm in children and about 0.40 mm in adults (my arithmetic).

Cheek, 90 Japanese women aged 20 to 85, Fourier analysis of replicas ([Hayashi et al. 1989](https://www.jstage.jst.go.jp/article/sccj1979/23/1/23_1_43/_pdf), values read from figures).

| Age | Mean furrow spacing | Anisotropy index |
|---|---|---|
| 20s | about 0.235 mm | about 17.6% |
| 30s | about 0.25 mm | about 17.4% |
| 40s | about 0.268 mm | about 18.3% |
| 50s and 60s | about 0.275 mm | about 19.0% |
| 70s and 80s | about 0.24 mm | about 20.8% |

On the cheek of 65 East Asian women aged 18 to 30, the peak-to-valley height of the hill-and-furrow pattern was 24.5 µm and its crest-to-crest spacing was 259.5 µm on average, ranging from 103 to 395.5 µm ([Omotezako et al. 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12319487/)).

Other sites.
- **Forearm.** The furrows in the main direction are 33 µm deep in children, 41 µm in adults and 60 µm in the aged. Some people over 80 have furrows deeper than 100 µm ([Corcuff et al. 1983](https://library.scconline.org/cdn-1708704470772/Skin-Relief-Aging.pdf)). Closed polygons fall from 3.1 per mm² in children to 1.0 per mm² in adults ([Trojahn et al. 2015](https://doi.org/10.1111/srt.12158), abstract only). The anisotropy index rises from about 25% at 20 to about 40% at 75 ([Zahouani et al. 2014](https://iopscience.iop.org/article/10.1088/1742-6596/483/1/012012/pdf), read from a figure).
- **Crow's feet.** The number of furrows doubles from age 6 to 35 and then stays the same. Their depth starts at about 25 µm and grows eightfold, to about 200 µm in the 60s ([Corcuff et al. 1987](https://library.scconline.org/cdn-1708705344415/Impact-Aging-Microrelief-Peri-Orbital.pdf)). The 2026-11-19 note saw this only as a search summary. This note confirms it from the paper.
- **Cheek, forehead and eye corner compared.** By in-vivo confocal microscopy, the cheek's furrows were about 96 µm deep and the forehead's and eye corner's about 81 µm ([Fujiwara and Ushiki 2015](https://www.jstage.jst.go.jp/article/koshohin/39/3/39_173/_pdf), read from a figure). These depths are 4 to 5 times those of the replicas, because the two methods measure different things.
- **Temple against forearm.** Furrow density falls with age at both sites, more at the temple. Anisotropy rises at both, more at the temple, which gets more sun ([Lagarde et al. 2005](https://doi.org/10.1111/j.1600-0846.2005.00096.x), abstract only).
- **Exposed against covered skin.** The pattern is strongly changed on exposed skin of people aged 68 to 75 and only slightly on covered skin ([Lavker et al. 1980](https://doi.org/10.1093/geronj/35.3.348), abstract only). Covered body sites have a finer texture, and the face a coarser one ([Masuda's thesis](https://muroran-it.repo.nii.ac.jp/record/8950/files/B80.pdf)).
- **Season.** In winter the cheek's furrows are significantly wider and the hills fewer, in a survey of 1,777 Japanese women ([Arakawa et al. 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/3/41_3_173/_pdf)).

### How the lines meet the pores

- Magnified skin shows furrows running lengthwise and crosswise, the hills between them, and "pores scattered at the intersections of the furrows" ([Arakawa et al. 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/3/41_3_173/_pdf), in Japanese).
- Hairs grow from the deep furrows, and sweat pores open on the hills ([Masuda's thesis](https://muroran-it.repo.nii.ac.jp/record/8950/files/B80.pdf), [Akimoto et al.](https://www.jstage.jst.go.jp/article/jacc/59/0/59_708/_pdf)).
- "Many pores are at the centre where furrows meet." On the cheek, small extra grooves join neighbouring furrows at the pore ([Fujiwara and Ushiki 2015](https://www.jstage.jst.go.jp/article/koshohin/39/3/39_173/_pdf)).
- An image-analysis method for skin replicas thins the furrows away and keeps "intersections and pores" together as one class ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf)).

So the pores sit on the corners of the network.
No study measures which share of the corners carries a pore.
With 160 to 360 plateaus per cm² on the cheek and 10 to 90 visible pores per cm², roughly one corner region in 2 to 20 carries a visible pore (my arithmetic).

### Direction

- On the forehead, the deep furrows run mostly across the face, perpendicular to its midline ([Fujiwara and Ushiki 2015](https://www.jstage.jst.go.jp/article/koshohin/39/3/39_173/_pdf)).
- On the cheek and at the outer eye corner, the furrows run in many directions ([Fujiwara and Ushiki 2015](https://www.jstage.jst.go.jp/article/koshohin/39/3/39_173/_pdf)).
- On the upper cheek of older skin, the furrows line up from the outer eye corner down towards below the ear ([Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf)).
- At the crow's feet, the furrows fan out from the eye corner ([Corcuff et al. 1987](https://library.scconline.org/cdn-1708705344415/Impact-Aging-Microrelief-Peri-Orbital.pdf)).
- With age, the second direction disappears. On the forearm, almost every person over 70 has only one direction left ([Corcuff et al. 1983](https://library.scconline.org/cdn-1708704470772/Skin-Relief-Aging.pdf)).
- Zahouani's group says that the deepest family of lines, deeper than 60 µm, follows Langer's lines ([Zahouani et al. 2014](https://iopscience.iop.org/article/10.1088/1742-6596/483/1/012012/pdf)). They state it but do not test it on the face. Corcuff and colleagues doubt that Langer's lines have ever been shown convincingly ([Corcuff et al. 1987](https://library.scconline.org/cdn-1708705344415/Impact-Aging-Microrelief-Peri-Orbital.pdf)).

Our `flow()` already runs across the forehead and out from each eye. Below the eyes it runs straight down the face. The sources point to a diagonal from the outer eye corner towards the ear on the upper cheek instead (I infer this from the three cheek findings above).

### How the techniques of question 1 make the grain

- Ishii's hierarchical Voronoi and Miyata's stretched cells draw the network as cell walls. They give polygons whose corners can carry pores.
- Weiss's graph draws only the edges between pores, so every furrow ends at a pore. The 2024 user study found two levels enough.
- Bando's growth model draws furrows in two directions that end on each other.
- Golovinskiy's statistics and Graham's exemplars copy the measured texture per region. They need scans.
- Activision's tile is a sum of radial sine waves. It gives a ripple without polygons.
- Our code uses Perlin noise, which gives neither polygons, nor a direction, nor a link to the pores.

## 4. How pores and grain show in colour alone

### The physics

- **Dark means deep.** Beeler and colleagues recover pores from photographs by assuming that small dark details are dents. Their reason is that "in small concavities, such as pores, part of the incoming light is blocked and the point thus appears darker" ([Beeler et al. 2010, section 3.3](https://studios.disneyresearch.com/wp-content/uploads/2019/03/High-Quality-Single-Shot-Capture-of-Facial-Geometry.pdf)). They push the surface in by `δ_µ = η Σ w_i (µ(X) − µ(X_i)) (…) / Σ w_i` with `η` = 0.2, where `µ` is the high-pass of the image. They note that larger dark patches, such as pigment spots, have no high frequencies and do not change the geometry. Read in reverse, a pore or furrow in a diffuse-lit picture is a small local darkening, and a broad darkening is colour.
- **The shadow in a pore is red.** Under diffuse light, the skin's red light scatters furthest under the surface, so the red channel's shading is the smoothest. Rendering each channel with its own normals shows "additional reddish light within the concavities" ([Ma et al. 2007](https://vgl.ict.usc.edu/Research/FaceScanning/EGSR2007_SGI_high.pdf)). So a pore's dark spot is less dark in red than in green and blue, which makes it slightly redder than a grey shadow (my inference from that finding).
- **Polarised video images of the cheek** show pores as reddish round spots and furrows as long thin shadows ([Arakawa et al. 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/3/41_3_173/_pdf)).
- **Plugs make pores dark.** In the 20s and 30s, most prominent nose pores are plugged by a hardened plug that is about 73% protein ([Mizukoshi 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/4/41_4_262/_article/-char/ja/)). I found no primary source that measures the colour of these plugs.
- **Some pore features are light.** The nose tip shows raised sebaceous filaments ([Du et al. 2025](https://icewired-yy.github.io/papers/face_micro_detail_synthesis/pdf/Facial%20Microscopic%20Structure%20Synthesis%20from%20a%20Single%20Unconstrained%20Image.pdf)). Foundation powder that collects in pores makes them look "brighter" ([Iida et al. 2014](https://www.jstage.jst.go.jp/article/sccj/48/1/48_19/_pdf), abstract only). A pore's raised rim catches light ([Mizukoshi and Takahashi 2014](https://pubmed.ncbi.nlm.nih.gov/23750904/), abstract only).
- **The grain is shading, with a small colour part.** Every replica method shows the furrows by shadows under grazing light at 26 to 38 degrees ([Corcuff et al. 1983](https://library.scconline.org/cdn-1708704470772/Skin-Relief-Aging.pdf), [Corcuff et al. 1987](https://library.scconline.org/cdn-1708705344415/Impact-Aging-Microrelief-Peri-Orbital.pdf)). On skin-coloured casts, a deep, fine texture raised the diffuse reflectance from about 35% to about 40% ([Masuda's thesis, chapter 5](https://muroran-it.repo.nii.ac.jp/record/8950/files/B80.pdf), read from a figure). Dry, flaking skin adds a white diffuse reflection ([a book chapter](https://www.taylorfrancis.com/books/9780429114564/chapters/10.3109/9781420005516-38), abstract only).
- **More fine contrast reads as dry skin.** Boosting the high frequencies of cheek lightness in photographs raised the variance of L\* from 23.4 to 30.2, and people judged the skin significantly less moist ([2024 preprint](https://www.biorxiv.org/content/10.1101/2024.07.16.603839.full)). I infer that a strong grain makes our heads look dry and rough.
- **Soft pores read as real.** A scan vendor raises the subsurface scattering "until the diffuse component of the skin pores almost vanishes" and calls hard pore shading the "sandy dry skin look" of many 3D renders ([3D Scan Store, Skin Shading in Marmoset Toolbag](https://3dscanstore.com/blog/skin-shading-in-marmoset-toolbag)). This is a vendor's tutorial.

### How painted and scanned colour maps show them

- A raw cross-polarised photograph still holds the pores' shadows. Texturing.xyz runs a "dedicated skin cleaning tool called Hyperclean, which removes all the undesired occlusion, shadow, lighting information and hair" before selling its albedo maps ([Texturing.xyz, VFace albedo](https://texturing.xyz/pages/introduction-vface-albedo)). So a modern scanned albedo has its pores removed on purpose, because the normal map puts them back.
- At 4K, ICT's polarised diffuse maps still had enough "naturally occurring texture" to align single pores across most of the face ([Digital Ira and Beyond](https://vgl.ict.usc.edu/Publications/2014/Digital%20Ira%20and%20Beyond%20Creating%20Photoreal%20Real-Time%20Digital%20Characters.pdf)).
- Weyrich and colleagues give a skin's albedo the grain of another skin by matching its statistics. They use a steerable pyramid of 7 levels and 4 directions in a decorrelated colour space and match 256-bin histograms, over the whole face or per region ([Weyrich et al. 2006](https://www.merl.com/publications/docs/TR2006-071.pdf)). Their maps are 2048 × 2048, like ours.
- For a game with only a diffuse map, the Polycount wiki says that "light, shadow and highlights can be used to simulate the effects of more complex shaders" ([Polycount, Diffuse map](http://wiki.polycount.com/wiki/Diffuse_map)). A cavity map is "small-scale ambient occlusion" ([Polycount, Cavity map](http://wiki.polycount.com/wiki/Cavity_map)).
- MetaHuman multiplies occlusion into the base colour once its normal detail is switched off ([MetaHuman materials and textures](https://dev.epicgames.com/documentation/metahuman/metahuman-materials-and-textures)).
- One artist interviewed by Texturing.xyz uses the curvature map to make cavity masks and to "push color info to my pores in the actual albedo map" ([Jared Chavez interview](https://texturing.xyz/blogs/all/jared-chavez-making-of-lankester-merrin-in-ue4)).
- Epic notes that "darkened wrinkles accentuate the normal map's texture" in the diffuse map ([Unreal 4.27, Digital Humans](https://dev.epicgames.com/documentation/en-us/unreal-engine/digital-humans?application_version=4.27)).

### What KOTOR's own textures say

The fit tool `fit_skin_detail` already measured KOTOR's pore band, which is the lightness minus its blur at 2.5 pixels, on all 57 heads. I read its output, `fit_all/fit_skin_detail.json` in the 2026-11-19 session's scratchpad.
- KOTOR's pore band has a median skew of 0.19 over the 57 heads. Real pores would make it clearly negative, as our own pores do (about −5).
- On the cheeks, the median head has 48 dark spots and 47 light spots per cm² in that band. On the nose it has 20 and 21. Dark and light spots are equally common.
- The fit therefore gave 36 of the 57 heads a negative pore depth, meaning light pores, and gave the median head no cheek pores at all. These are the shipped defaults.
- The band is strongest around the eyes (1.47 L\* median standard deviation) and on the forehead (1.24 L\*), and weakest on the cheeks (0.30 L\*).

I infer from this that the pore band of most KOTOR heads holds no pores. It holds upscaling and compression noise and the edges of painted wrinkles, which are as often light as dark. The roadmap already says that the 512-pixel textures hold no pores. The fit has taken its pore settings from that noise. The look to reach is the mod textures, and they should set the pore settings.

### What this means for our colour

I infer the following rules from the sources above.
1. A pore is a small darkening, strongest at its centre, and slightly redder than the skin around it.
2. A pore may have a faint lighter rim, much weaker than the dark core.
3. The furrows of the grain are thin darkenings along the cell walls. Their plateaus are flat or very slightly lighter. So the grain's lightness has a negative skew, like the pores' and unlike Perlin noise.
4. Detail finer than two pixels cannot be drawn and should only shift the mean colour. A deeper, finer texture is slightly lighter and whiter on average.
5. The contrast should be soft. Hard, high-contrast specks read as dry, sandy skin.

## 5. Our implementation against the sources

`pores.rs` and `noise.rs` were read in the `head-paint` worktree on 2026-11-20. The defaults are those of `controls.rs`, `default_recipe()`.

### What the sources support

- Every value is a function of the 3D point in millimetres, so the layer has no seams and no stretching. This is the solid-texture approach, and the per-pixel cost is a few dozen hash calls.
- The thinning of feature points by a density mask evaluated at the feature point keeps each pore whole and lets the density follow a region map.
- The slab rule gives a spread of dot sizes, and the unit test checks the density on the skin against the wanted density.
- The density control ranges over 0 to 150 per cm² and the size over 0.1 to 0.6 mm in radius. The measured range of 10 to 90 per cm² and 0.08 to 0.35 mm in radius falls inside. The default radius of 0.22 mm, with the ball 1.2 times larger, gives a mean dot of about 0.15 mm². That matches the enlarged pores of Caucasian and Japanese women ([Flament et al. 2015](https://pmc.ncbi.nlm.nih.gov/articles/PMC4337418)).
- The lips get no pores, as the red lip has no follicles. The eye openings are cleared as well.
- The pore profile `(1 − d²/r²)²` is widened by 0.06 mm and kept at the same total darkness, so a two-pixel pore does not flicker.
- The layer changes only the lightness and keeps the chroma. This is a safe default, but see "What is missing".

### What the sources contradict

- **The grain is Perlin noise.** Two octaves of gradient noise with a feature size of 0.25 mm make a pattern with no cells, no direction, no link to the pores, and a skew near zero. Real grain is a network of dark lines whose corners carry the pores ([Arakawa et al. 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/3/41_3_173/_pdf), [Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf)). Golovinskiy and colleagues found plain Perlin noise on a face "too uniform" ([Golovinskiy et al. 2006](https://pixl.cs.princeton.edu/pubs/Golovinskiy_2006_ASM/facePaper.pdf)).
- **The grain is finer than the texture can hold.** At the default size of 0.25 mm, the first octave's features are about 2 pixels and the second octave's are about 1 pixel. Detail at one pixel cannot form shapes, so it shows as speckle. This is the most direct cause of "random noise".
- **The pores are round.** Measured cheek pores have a long axis 1.7 to 2.4 times their short axis, lined up along one direction ([Mizukoshi 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/4/41_4_262/_article/-char/ja/), [Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf)). The code already has `flow()` and the 2026-11-19 note gave the formula to stretch pores along it, but the pores do not use it.
- **The T-zone mask is three Gaussians.** It has a 9 mm reach around the nose's ridge, 26 mm around the forehead's middle and 12 mm around the chin. It leaves out the upper inner cheek, which has the largest cheek pores, and it gives the chin T-zone pores at every age.
- **The default removes every cheek pore.** The default `t_zone` of 1.0 means the cheeks have none. Every study finds 10 to 90 visible pores per cm² there.
- **The default pores are lighter than the skin.** The default `depth` of −1.2 L\* makes a light speck. A light pore is possible on a nose tip or with powder, but a typical pore is a darkening ([Beeler et al. 2010](https://studios.disneyresearch.com/wp-content/uploads/2019/03/High-Quality-Single-Shot-Capture-of-Facial-Geometry.pdf)). The value comes from fitting noise in the 512-pixel textures, as section 4 shows.
- **Size and density are the same everywhere.** Only the count follows the mask. Measured pore size on the upper cheek is twice that on the lower cheek ([Yamashita et al. 2010](https://www.jstage.jst.go.jp/article/sccj/44/3/44_216/_pdf/-char/en)).

### What is missing

- **A furrow network.** There is no network that joins the pores, so pores and grain look like two unrelated noises. The fine lines of `wrinkles.rs` are scattered strokes 5 to 11 mm long and 0.4 mm wide, which is a much larger scale than the grain. They are not tied to the pores either.
- **Region presets.** Disney's user study found that a preset per region beats one setting for the face (PageRank 0.168 against 0.209, lower is better) ([Weiss et al. 2024](https://studios.disneyresearch.com/app/uploads/2024/06/Stylize-My-Wrinkles-Bridging-the-Gap-from-Simulation-to-Reality-Paper.pdf)). Our grain has one amplitude and one size for the whole face.
- **Age and sex.** Pore size, pore stretch, chin pores, grain coarseness and grain direction all change with age. The grain gets coarser (360 down to about 170 plateaus per cm² on the cheek), deeper and more one-directional ([Masuda's thesis](https://muroran-it.repo.nii.ac.jp/record/8950/files/B80.pdf), [Hayashi et al. 1989](https://www.jstage.jst.go.jp/article/sccj1979/23/1/23_1_43/_pdf)). Men's pores are larger, and men have more fine detail on the chin and lower cheeks ([Zhou et al. 2024](https://pmc.ncbi.nlm.nih.gov/articles/PMC11411701), [Golovinskiy et al. 2006](https://pixl.cs.princeton.edu/pubs/Golovinskiy_2006_ASM/facePaper.pdf)). The pore layer reads none of this. The wrinkle layer already has an age.
- **Colour in the pore.** The pore changes only lightness. A slight shift towards red in the core would follow the measured reddish light in concavities ([Ma et al. 2007](https://vgl.ict.usc.edu/Research/FaceScanning/EGSR2007_SGI_high.pdf)).
- **A rim.** Real conspicuous pores have a broken raised rim, and Disney's pore profile has one.
- **Plugged pores.** In young adults, many nose pores hold a dark plug. The layer has no way to make some pores darker than others except by their slab cut.
- **A band limit.** No part of the layer checks the pixel's size. Any detail finer than two pixels is drawn and turns into speckle.

## Recommendations

They are ranked by how much they should improve the look for their cost.
Each must be shown in the viewer on our head next to the KOTOR original and a mod head before it counts as done, as the roadmap asks.
The cost today is about 54 hash calls per pixel for the pores (27 cells, two hashes each) and 16 for the grain (two octaves of 8 corners).

### 1. A furrow network on the pores instead of the Perlin grain

**What changes in `pores.rs`.**
- Add a second cell grid for the network, with a cell edge `s` of 0.4 to 0.8 mm. One jittered node per cube, as for the pores today.
- Measure every distance in a metric stretched along the flow. The stretched distance is `d² = |Δ|² + (k² − 1)(Δ · t)²`, where `Δ` is the vector between two points and `t` is the flow at the pixel. A `k` below 1 makes distances along `t` count less, so the network's long furrows run along the flow. This is the per-pixel form of Miyata's stretched cells ([Miyata et al. 2001](https://www.andrew.cmu.edu/user/shimada/papers/01-siggraph-sketch-miyata.pdf)) and of Weiss's preferred edge direction.
- At the pixel, find the four nearest nodes over the 27 cubes in that metric. Draw a furrow along each segment between the nearest node `q1` and each of the other three, as long as the segment is shorter than about 1.5 `s`. A segment between two neighbouring nodes is an edge of Weiss's pore graph, so every furrow starts and ends at a node, as the sources say ([Weiss et al. 2023](https://studios.disneyresearch.com/app/uploads/2023/07/GraphBasedSynthesisForSkinMicroWrinkles.pdf), [Arakawa et al. 2007](https://www.jstage.jst.go.jp/article/sccj1979/41/3/41_3_173/_pdf)). Only nodes close to the skin should count, so weight each segment by how close both of its ends lie to the skin's surface, using the distance along the normal.
- Draw each furrow as a darkening with the profile `1 − smoothstep(0, 1, e / w)`, where `e` is the pixel's distance to the segment and `w` is half a furrow width. Give each segment its own strength from a hash of its two nodes, so that some furrows are deeper than others, as primary and secondary lines are.
- A simpler variant draws the cell walls instead, with the wall distance `e = (Δ2 · Δ2 − Δ1 · Δ1) / (2 |q2 − q1|)` between the nearest node `q1` and the second nearest `q2`. This is the Ishii pattern. Its walls meet at the cells' corners, not at the nodes, so the pores would then belong on the corners. I recommend the segment version, because it puts the pores where the sources put them.
- Make the visible pores a thinned subset of the network's nodes. A node shows as a pore with the probability `density(region) × s² / 100`, so that the pores per cm² still follow the density control. Then the network and the pores share one grid, and every pore sits where furrows meet. This replaces the pore grid of today. With Weiss's rule, a pore where several strong furrows meet can be made deeper and wider.
- Remove the Perlin grain, or keep it only below a strength of about 0.3 L\* for faint unevenness.
- Apply the band limit of recommendation 3 to the walls. Where `s` is under about 2.5 pixels, fade the walls' contrast out and darken the mean colour a little instead.

**Sliders.**
- "Grain" keeps its name and its range, and now sets how dark the furrows are, in L\*.
- "Grain size" becomes the cell spacing `s`, with a range of 0.3 to 1.0 mm. The measured adult cheek network is 0.24 to 0.4 mm apart and Disney's coarsest level is about 0.66 mm (my estimate). The mod textures should set the default. I expect a value of 0.5 to 0.7 mm, because the true cheek size is only 2 pixels.
- A new "Grain direction" slider from 0 to 1 sets how much the cells stretch along the flow, through `k` from 1 down to about 0.5. The age slider could drive it, since the anisotropy index rises with age on both the cheek and the forearm.

**Cost.** One more 27-cube search with one hash per cube, a sort of the four nearest nodes, and three point-to-segment distances. That is about 27 hashes and about 30 distance computations, which replace the 16 hashes of the Perlin grain. If the pores move onto the network's grid, their own 27-cube search goes away, and the total falls below today's.

**Picture that proves it.** Crops of 30 × 30 mm of the cheek, the forehead and the nose, enlarged 4 times. Show our old grain, our new network, a KOTOR mod head, and a polarised close-up of real skin side by side. Also give, per region, the skew of the pore band (it should turn negative) and the anisotropy index of its spectrum (it should be about 17% to 21% on the cheek at middle age, as [Hayashi et al. 1989](https://www.jstage.jst.go.jp/article/sccj1979/23/1/23_1_43/_pdf) measured).

### 2. A region map for pore density and pore size

**What changes in `pores.rs`.**
- Replace `t_zone()` with two region maps built from the landmarks. One gives a density factor and the other a size factor, each from 0 to 1.
- Use these density factors, relative to the nose. They are my design values, built from the ratios in section 2: nose 1.0, upper inner cheek beside the nose 0.8, middle of the forehead 0.7, middle of the cheek 0.5, lower cheek beside the mouth 0.35, chin 0.3 at age 15 rising to 0.8 at age 70, temples and the outer edge of the forehead 0.15, skin around the eyes 0.1, lips 0. Never let the cheeks fall below about 0.15 of the nose.
- Use these size factors: nose 1.0, upper inner cheek 1.0, middle of the cheek 0.7, lower cheek and the skin beside the mouth 0.5, chin 0.5 rising to 0.8 with age, forehead 0.7, temples and eyes 0.4. The factor of two between the upper and lower cheek comes from [Yamashita et al. 2010](https://www.jstage.jst.go.jp/article/sccj/44/3/44_216/_pdf/-char/en). The rest are my design values.
- The upper inner cheek zone needs a new landmark point on the cheek beside the nose wing.
- Evaluate both maps at the feature point, as the mask is evaluated today.

**Sliders.**
- "Pores per cm²" stays and now sets the density at the nose.
- "Fewer pores on the cheeks" becomes "Pores away from the nose", from 0 (the same everywhere) to 1 (the measured gradient) to 2 (twice as steep). The default should be 1 and not today's 1.0 of the old meaning, which removes every cheek pore.
- Age moves the chin's density and size, from the wrinkle layer's age slider.

**Cost.** The map costs a few distances to landmark points per candidate point, which is the same as `t_zone()` today. No new hash calls.

**Picture that proves it.** A map of the pore density on the unwrapped head, coloured by pores per cm², next to the table of section 2. Then the full face at 1:1 next to a mod head, with the visible dots per cm² on the nose, upper cheek, lower cheek, forehead and chin counted by the blob detector of `fit_skin_detail`.

### 3. Oriented pores with a core and a rim, and a band limit

**What changes in `pores.rs`.**
- Stretch each pore along the flow at its feature point, with the distance `d² = |Δ|² + (k² − 1)(Δ · t)²` that the 2026-11-19 note already gives. A `k` of 0.45 to 0.6 gives the measured axis ratios of 1.7 to 2.2. Draw a random `k` per pore around that mean, so that the ratios spread as measured.
- Change the profile to a darker core with a faint rim. Use Weiss's shape with the rim, `(x − 3)² (x − 1) / 9` for `x` from 0 to 3, with the rim's lightening scaled down to about a quarter of the core's darkening. The rim's strength is my design value, because no source gives it.
- Add a small shift towards red in the core. The size of the shift has no source and should be fitted on a mod head.
- Give a small share of the nose's pores, for example 10% to 20%, a darker core, to stand for plugged pores. This is my design value.
- Add a band limit to the whole layer. Pass the pixel's size in millimetres to `lightness_shift`. Fade out any feature whose size is under 2 pixels, and widen the soft margin of the pore profile to half a pixel of the actual pixel size instead of the fixed 0.06 mm. Remove the grain's second octave, which is at one pixel.

**Sliders.**
- A new "Pore stretch" slider from 0 to 1 sets the mean `k`, from round to an axis ratio of about 2.5. Age can raise it, as measured on the upper and middle cheek.
- "Pore darkness" keeps its range of −30 to 30 L\*, but its default should be refitted on the mod heads and should be positive.

**Cost.** One flow evaluation per kept candidate point. Since the flow is smooth, it can be cached per pixel, which makes it cheap. The band limit is free.

**Picture that proves it.** A 10 × 10 mm crop of the upper cheek at 8 times, with pores coloured by their measured axis ratio and angle. Then the same crop in colour, next to a polarised dermatoscope image and a mod head.

### 4. Fit the pore settings on the mod heads, not the 512-pixel heads

**What changes.** Nothing in `pores.rs`. In `fit_skin_detail`, read the mod textures and the 1,024-pixel player heads, and leave the 512-pixel heads out of the pore fit. Add a check that refuses to fit pores where the pore band's skew is above about −0.5, because such a band holds no dark dots.

**Sliders.** The defaults of "Pore darkness", "Pores away from the nose" and "Grain" come from the new fit.

**Cost.** None at run time.

**Picture that proves it.** The bands picture that `fit_skin_detail` already writes, for three mod heads, with the pore band's skew and the dark-to-light spot ratio printed under each.

### 5. Age and sex drive the fine detail

**What changes in `pores.rs`.** Read the recipe's age, and a sex value if the head has one.
- Pore size grows by about 1.5 times from age 15 to 40 and then holds, on the cheeks and nose. On the chin and lower cheek it keeps growing to 70. This follows [Lee et al. 2022](https://pmc.ncbi.nlm.nih.gov/articles/PMC9292708) and [Nishijima et al. 2001](https://www.jstage.jst.go.jp/article/sccj1979/35/2/35_2_141/_pdf).
- Pore stretch rises with age on the upper and middle cheek.
- The network's cells grow by about 1.4 times from childhood to adulthood (360 down to about 170 plateaus per cm²) ([Masuda's thesis](https://muroran-it.repo.nii.ac.jp/record/8950/files/B80.pdf)). Its direction strength rises with age.
- For a male head, raise pore size by about 1.2 times and the network's strength on the chin and lower cheek. The factor is my design value from the Chinese selfie study's 4% to 26% gap.

**Sliders.** No new slider. The age slider of the wrinkle layer drives these values. Every value stays inside the player's existing controls.

**Cost.** A few multiplications per candidate point.

**Picture that proves it.** One head at ages 20, 40 and 70, in crops of the cheek and chin, next to the matching KOTOR heads of those ages.

## What I could not find out

- **Per-region pore numbers for the nose, forehead, temples and around the eyes.** I found pore counts and sizes for the upper, middle and lower cheek and the chin, but none in mm² or per cm² for the nose or forehead. The only primary forehead number is a follicle density of 285 per cm² ([Ackerley et al. 2017](https://pmc.ncbi.nlm.nih.gov/articles/PMC5451466)). Pagnoni 1994, Otberg 2004, Jung 2018, Roh 2006, Kim 2013 and Sugiyama-Nakagiri 2008 sit behind bot checks, and I read only their abstracts. Murakami 2006, a 3D analysis of pores in several facial regions cited by Iida 2016, could not be found online.
- **The 1,200 per cm² follicle density on the nose.** It still comes only from a textbook. I could not confirm it.
- **Which share of the furrow network's corners carries a pore.** Dermatology describes pores at the corners, but no study counts it.
- **Hashimoto 1974 and Ishii 1993.** The four orders of lines and the hierarchical Voronoi model are known to me only through later papers. I have no seed densities, depths or ridge profiles from Ishii.
- **The colour of a pore.** No source measures how much darker or redder a pore is in a cross-polarised albedo, or the brightness of its rim. The red tint is an inference from Ma and colleagues. Every colour value in the recommendations needs the fit.
- **Whether the cell walls look right at 2 to 5 pixels.** The stretched cell network, the pores on its nodes and the band limit are my design. They are not published as a per-pixel method, and nobody has tested them on our head.
- **Directional statistics on the face.** I found no study that tests facial furrows against Langer's lines or relaxed skin tension lines with circular statistics. The flow directions in this note come from a few descriptions.
- **The mod textures' own pore numbers.** I did not measure pores on the KOTOR mod heads. Deadly Stream refused connections, so I could not read the mod pages either. One upscale mod states that it does "not attempt to … add extra detail (like … skin pores)" ([Deadly Stream, search summary only](https://deadlystream.com/files/file/2659-4x-upscale-character-textures-model-fixes/)).
- **Talks I could not read.** The Klintberg and Haapaoja talk (abstract only), the Weta talk and patents (no numbers), the Square Enix GDC 2017 skin talk, the Disney patent US 12,651,397 (first page only), and the tiling values of MetaHuman's micro normal maps.
- **Lagarde 2005 and Trojahn 2015.** Their temple and forearm numbers are behind a paywall. I read only their abstracts.
