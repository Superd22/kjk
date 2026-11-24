# Detail maps for the custom heads

This note was researched on 2026-11-20 for kjk's head paint track, which is part C of the roadmap (`2026-11-19-head-roadmap.md`).
David asked whether the custom heads should get a normal map, a specular or gloss map, or another map that is not physically based, so that the fine skin detail answers to the game's light.
He also asked what kjk's renderer supports today.

Every claim carries a `path:line` in the kjk repository, a link, or a path in a clone made for this note.
The clones are in `/private/tmp/claude-501/-Users-david-perso-kjk/c3910254-b661-450a-9003-645d2158b8cb/scratchpad/research-maps/`: reone (commit 33f8996), xoreos (6774724), Odyssey (7ac4b01) and K12 (ef0e18d).
Odyssey is the reconstruction of the original `swkotor.exe` that builds byte-identical, and K12 is the decompilation of the Android ports.
The repository rules forbid copying their code and forbid using original addresses in kjk, so this note only describes what they do.
Where a claim comes from a wiki or a forum, the text says so.
Where I estimate a cost, the text says "estimate".

Terms used below.
- A **normal map** stores, for each texel, the direction the skin surface faces. It is "tangent-space" when that direction is stored relative to the surface, so it follows the mesh as it bends and animates.
- A **tangent** is a direction along the surface that matches the texture's horizontal axis. A mesh needs one per vertex to use a tangent-space normal map.
- A **gloss mask** or **specular mask** says, per texel, how strongly the surface mirrors a light.
- A **cavity map** says how much each texel sits in a small dent, so dents can be darkened.
- A **detail normal tile** is a small normal map that repeats many times across the face.
- A **matcap** is a picture of a lit sphere that is looked up by the direction the surface faces on screen, so it needs no lights.

## Summary

- kjk's character shader samples exactly three textures from a mesh's own files: the colour texture, a lightmap and an environment map. It samples no normal map, no bump map and no gloss map, and the mesh pipeline carries no tangents (`crates/game/src/odyssey.wgsl:243-256`, `crates/game/src/scene.rs:3154-3162`).
- A custom head today gets no highlight at all from any light. The only shine in the shader is the environment map, and the fight lights' highlight, and both are switched on only for a surface that has an environment map (`odyssey.wgsl:1816`, `odyssey.wgsl:1861-1865`). Room lights, lightmaps, the area's ambient and the light probes only shade with a diffuse term.
- The original game does support bump maps. Its Windows renderer has a per-pixel diffuse pass and a per-pixel specular pass for a bump-mapped mesh, and the specular pass is masked by one minus the colour texture's alpha, exactly as kjk masks its environment map. So the gloss mask in the alpha channel is the original's own method, and KOTOR 1 and 2 mod heads already use it (section 2).
- The cheapest change with a visible result is a gloss mask in the alpha channel together with a highlight from the area's lights. It needs no tangents, no extra texture and no extra memory. The pores and wrinkles should follow with one tangent-space normal map for the wrinkles and one repeated detail tile for the pores. A cavity map, a matcap and a separate occlusion map are not worth adding.
- A normal map shows nothing under kjk's flat ambient light. It changes the picture only under a directional light, and a head in a conversation has few of them. The gloss highlight and the normal map should therefore be built together, in the order given in the recommendation.

## 1. What kjk's renderer does today

### The textures a character material takes

- The material binds, in order, a parameter block, the colour texture with its sampler, a lightmap, a cube environment map, the light list, the shadow list, the boxes, the animated slots, a flat sphere-mapped environment texture, the shadow tile atlas and the light probes. There is no binding for a normal, bump, gloss or detail texture (`odyssey.wgsl:242-257`, `crates/game/src/material.rs:848-873`).
- A custom head therefore has one texture of its own, the 2,048-pixel colour texture. Its alpha channel is used as coverage unless the material has an environment map (`odyssey.wgsl:1881`).
- The texture reader parses `bumpmaptexture` and `bumpyshinytexture` from a TXI file, which is the small text file that sits beside a KOTOR texture and sets its options. It stores the names (`crates/asset_odyssey/src/texture.rs:36-41`, `:107-108`). The bump name only changes how the surface is sorted: a bump-mapped texture's alpha is not read as coverage (`crates/game/src/material.rs:689-691`, `:716`, `:725`). The name `bumpyshinytexture` is treated as an environment map name, the same as `envmaptexture` (`texture.rs:107`). No bump texture is ever loaded for drawing. I searched for `bumpmap` over the other crates and found no use.

### How a character's skin is lit

A character mesh has no lightmap, so the shader builds its light from these parts (`odyssey.wgsl:1652-1760`).
1. The area's flat ambient, `ambient_colour * area_ambient * room_kept` (`odyssey.wgsl:1684`). It has no direction, so it does not use the normal at all. Under Enhanced lighting a model of an area takes the light probes' light instead, which weighs six coloured sides by the normal (`odyssey.wgsl:570-600`, `:1675-1683`).
2. Each ordinary light in the list adds `strength * facing * kept` times the material's diffuse colour, where `facing` is the normal's dot product with the direction to the light (`odyssey.wgsl:1737-1751`). This is the old fixed-function diffuse sum, and it has no specular part.
3. Each fight's light, such as a blaster bolt or a saber, adds its light with a wrapped diffuse term, where the surface turned away from the light still takes 30% (`odyssey.wgsl:224`, `:1444-1457`).
4. The sum is clamped to 1 and multiplied by the colour texture (`odyssey.wgsl:1754`, `:1810`).
5. Under Enhanced lighting the fight lights are added afterwards as linear light (`odyssey.wgsl:1847-1870`).

The fragment shader lights with `in.world_normal`, which is the vertex normal interpolated across the triangle (`odyssey.wgsl:1663`).
The normal-prepass for ambient occlusion also uses the geometric normal (`crates/game/src/odyssey_prepass.wgsl:102-124`), so a perturbed normal would not need a change there.

### Which shine exists

- The environment map. When the material has one (`material.rs:1153-1155`), the shader mirrors the view direction about the interpolated normal, reads the cube or the sphere texture, and adds it as `colour + shine * (1 - texel.a)` (`odyssey.wgsl:1816-1846`). So the colour texture's alpha is a mask, and 255 means no shine.
- The fight lights' highlight, `fight_highlights`. It is a Blinn-Phong highlight with an exponent of 60 and a gain of 6 (`odyssey.wgsl:232-233`, `:1536-1562`). It runs only under Enhanced lighting, only on a surface with an environment map, and only for lights of the kind "fight" (`odyssey.wgsl:1847`, `:1861-1865`, `:1546`). It is masked by the same `1 - texel.a`.
- Room lights, lamps and the sun produce no highlight in any mode. Only the main sum above reads them.

So a custom head, which has no environment map, shows no highlight from any light.
Its "sheen" on the forehead and nose, and its lip gloss, exist only as painted colour.
During a conversation, no fight light is present, so even a head that did have an environment map would only show the fixed cube reflection, which does not move with the room's lamps.

### Tangents

- The meshes the scene builds carry position, normal, two sets of texture coordinates, and for skinned meshes joint indices and weights (`crates/game/src/scene.rs:1484-1487`, `:3154-3162`). I found no use of `ATTRIBUTE_TANGENT` or of tangent generation anywhere in `crates`.
- Bevy 0.19.1 supports tangents in its default vertex shader. When a mesh has them, the shader outputs `world_tangent`, and it skins them through the blended joint matrix (`bevy_pbr-0.19.1/src/render/mesh.wgsl:86-92`, in the cargo registry). It can generate them with `Mesh::generate_tangents`, which uses the mikktspace method (`bevy_mesh-0.19.1/src/mesh.rs:1762`).
- kjk's own sway vertex shader already passes tangents on when a mesh has them (`crates/game/src/dangly.wgsl:65-70`).
- So adding tangents costs nothing in the pipeline. The mesh needs an extra four floats per vertex, and the generator must call the tangent builder once per head. Our custom head is generated in Rust, so it can write its tangents directly.

### The prepass and the rest

The depth and motion prepass draws the same triangles from the same vertices, and its fragment shader does not read the colour texture except to cut out alpha (`material.rs:1461-1500`).
No detail map needs a change there.

## 2. What the original KOTOR does with these maps

### The original engine

I read the Odyssey reconstruction of `swkotor.exe`.
- The texture class knows three kinds of bump map: `IsBumpMap`, `IsDiffuseBumpMap` and `IsSpecularBumpMap`, with a diffuse and a specular intensity each, and a bump scale (`dev/libsource/AURORA/aurtexman.cpp:420-480`, Odyssey).
- A material has `HasBumpMap`, `HasBumpyShinyMap` and `BumpMapIsSpecular` (`aurmaterial.cpp:193`, `:593`, Odyssey).
- A bump-mapped mesh is drawn with a bump pass only when the mesh carries tangent bases, written `HasTangentBases`. A skinned mesh is tested the same way, so characters can be bump mapped (`aurskin.cpp:357`, `:1025`, `:1118`, Odyssey).
- The bump pass needs special graphics hardware: NVIDIA register combiners and a vertex program, or an ATI fragment shader (`aurglext.cpp:648-668`, Odyssey). Without it the engine draws a fallback (`RenderBumpmapFallback`, named in `cmake/order/swkotor.txt:2081`, Odyssey).
- There are separate vertex programs for the bumpy-shiny pass, its skinned form, and the diffuse, specular and one-pass bump forms (`aurvertexprog.cpp:341-440`, Odyssey). Each one turns the light, half vector or eye vector into the surface's tangent space.
- The K12 decompilation rebuilds the PC bump pass as a fragment shader (`resources/shaders/k2/kotorbump.frag`, K12). Its header says it follows the Knights of the Old Republic II PC executable's programs. It does these things.
  - The normal is read from the bump texture as `rgb * 2 - 1`, in tangent space.
  - The diffuse term is the light's diffuse colour times `max(N.L, 0)` times an attenuation, added to the vertex light, and multiplied by the colour texture.
  - The specular term is the light's specular colour times `max(N.H, 0)` to the 16th power, only where the half vector faces out of the surface. It is multiplied by `1 - base.a`, so the colour texture's alpha masks it.
  - The bumpy-shiny pass reads a bump map and an environment cube, reflects the eye about the perturbed normal, and looks the cube up.
- I did not find the document that K12's header cites (`docs/prep/bump_k2pc_v2263.md`). It is not in the clone. So the exact formula above is K12's reading of the executable and not something I confirmed in the executable.
- The same file shows that a plain environment map is added as `color += (1 - color.a) * cubecolor` (`resources/shaders/k2/kotor.frag:58-63`, K12). This matches kjk's `colour + shine * (1 - texel.a)`.

### The open reimplementations

- reone reads `bumpmaptexture`, `bumpyshinytexture`, `envmaptexture` and `bumpmapscaling` from the TXI (`src/libs/graphics/format/txireader.cpp:66-76`, reone). If the bump texture is grayscale it is treated as a height map. The shader takes its screen-space derivatives and builds a normal from them. If it is coloured it is read as a normal map (`src/libs/scene/node/mesh.cpp:434-442`, `glsl/i_normalmap.glsl`, reone). The resulting normal feeds only the diffuse term. reone has no specular term for bump maps, and it adds the environment map as `envmap * (1 - alpha)` (`glsl/f_oit_model.glsl:109-113`, reone).
- xoreos stores the TXI names and ignores the bump map when drawing KOTOR models. Its environment map uses the same `1 - alpha` masking idea (`src/graphics/aurora/modelnode.cpp:479-482`, xoreos). This matches kjk's own comment that xoreos ignores the bump map (`material.rs:689-692`).
- xoreos's Jade Empire loader reads a material flag named `bumpMapIsSpecular` (`src/graphics/aurora/model_jade.cpp:724`, xoreos), but does not draw with it. This is the only evidence I found that BioWare's next engine let the bump map carry specular. It shows the flag exists. It does not show what Jade Empire's faces look like.

### Do KOTOR heads use these maps

- I could not read the stock KOTOR 1 or 2 archives on this Mac, because the games are installed on David's Windows PC only. So I could not count how many of the 57 stock heads have a `bumpmaptexture` or `envmaptexture` line. This needs one run of the scene's own tally on the Windows install (`crates/game/src/scene.rs:1371-1382`).
- I did read the TXI files of the mod heads that are on this Mac, in `~/.claude/tools/likeness-fit-data/kotor-heads/mods`. Seven of them have a TXI with a line, and all seven say exactly `envmaptexture CM_SpecMap`. None has a bump map.
- Those head textures carry an alpha channel that is not opaque. For the three I measured (`PFHB05.tga`, `PFHA01.tga` and `P_CandH01.tga`), 1% to 4% of the pixels have an alpha below 250, and the lowest alpha is 163 to 217. I drew where it sits for `PFHB05` and `P_CandH01`: it is on the lips, the eyeballs and the teeth, and on nothing else. So the mod authors use the original method as intended: a small shine mask on the lips and the wet parts of the eye, with a weak strength of at most (255 - 163) / 255 = 36%.
- The textures in the folder `improved_bastila` and `vp_source` carry an alpha with a minimum of 0, but their TXI has no environment line, so I do not know what that alpha does there.

The head textures of the 57 stock heads that the fit tools measured (`2026-11-20-pores-and-grain.md`, section 4) have no pores in them.
That is consistent with an engine that, by default, gives faces only a colour texture and an optional mask.

## 3. What each candidate map would buy on a face

The distances.
At conversation distance a head fills about a tenth to a fifth of the screen height.
In a dialogue close-up it fills most of it.
On a 1,080-pixel screen, a head 24 cm tall that fills 900 pixels shows about 3.75 pixels per millimetre (estimate).
Our texture gives the face zones 53 to 84 texels per square millimetre at 2,048 pixels (`2026-11-19-head-roadmap.md`, step C1), which is about 7 to 9 texels per millimetre.
So in the close-up the texture is about twice as dense as the screen, and the screen is the limit. A pore, which is a fraction of a millimetre wide, is one or two screen pixels in the close-up and below one pixel at conversation distance.

### A tangent-space normal map

- What the sources say. Graham and colleagues measured that skin's micro-relief is a specular effect: the diffuse and subsurface normals stay at 4K, and only the specular normal and the specular albedo go to 16K (`2026-11-20-pores-and-grain.md`, section 1). The GPU Gems 3 chapter on skin says that specular light "shows the most dramatic, high-frequency changes in the skin surface", that diffuse light is scattered and so softens surface detail, and that "the true normal information should be used for both specular and diffuse computations" ([d'Eon and Luebke, GPU Gems 3, chapter 14](https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-14-advanced-techniques-realistic-real-time-skin)).
- What it buys. Under a directional light and a highlight, wrinkles and pores respond when the light or the head moves. A baked dark line cannot do that.
- What it does not buy here. Under the area's flat ambient, the normal is never used (`odyssey.wgsl:1684`), so the map is invisible there. It shows only under an ordinary light, a fight light, a probe's light or a highlight. The pores will therefore be visible mainly through the highlight on the forehead, nose and cheeks.
- What it needs. Tangents on the mesh (section 1), one more texture binding, a tangent-space normal built from the paint's own furrow network, and a seam-aware layout, since the texture layout splits vertices at seams.
- Epic's own guidance for its digital humans is the same split: wrinkles in the face's normal map, and "Pore details should be done with a tiling texture" ([Unreal, Creating Human Skin](https://dev.epicgames.com/documentation/unreal-engine/creating-human-skin-in-unreal-engine), quoted in `2026-11-20-pores-and-grain.md`, section 1).

### A gloss or specular mask in the alpha channel

- What it buys. This reuses the path kjk already has. A mask in the alpha channel marks where the head is shiny: the lips, the wet eye corner, the tip and ridge of the nose, and the forehead's oily patch. It would replace the painted sheen and the painted lip gloss, which are fixed in the colour and cannot follow a light.
- Evidence that it works on KOTOR faces. The mod heads of section 2 do exactly this, and Valve's engine did it in 2004: in Source the alpha channel of the normal map, or of the base texture with `$basemapalphaphongmask`, is the phong mask ([Valve Developer Community, a wiki, via search result; the page itself returned an error to me](https://developer.valvesoftware.com/wiki/$phong)).
- What it needs from the renderer. Today the mask works only with the environment map, and the highlight only for fight lights under Enhanced lighting (section 1). To show a highlight in a conversation, the shader needs a highlight term inside the main light loop, for the ordinary lights (`odyssey.wgsl:1706-1752`). It also needs a way to switch the mask on for a head without an environment map. The original used 16 as the specular exponent for a bump surface (K12 reading); kjk's fight highlight uses 60. Skin wants a lower exponent and a lower strength than a metal, so both should be per-region settings.
- What it needs from the texture generator. The alpha channel, holding 255 minus the shine, with a slider for each region. A texture where alpha is below 255 is no longer coverage for the head (`odyssey.wgsl:1881`), which is what the head wants.
- The cost to the look. The alpha channel can hold only one number per texel. A single mask cannot give the lips and the skin different exponents. A second channel of the mask, or a per-region exponent from the paint's region map, solves that.
- Source's other ideas help here. Its phong has an exponent map, a boost and a Fresnel range, and a wiki note says it needs at least `$phong`, `$phongboost`, and an exponent ([Valve Developer Community pages seen through search results, a wiki](https://developer.valvesoftware.com/wiki/$phong)). A Fresnel term raises the highlight at grazing angles, which is what makes skin read as skin. GPU Gems 3 says Phong "fails to capture increased specularity at grazing angles" and uses a Kelemen and Szirmay-Kalos model with a precomputed Beckmann texture and Schlick's Fresnel approximation instead.

### A cavity or occlusion map

- What it buys. A cavity map darkens the pores and wrinkles, and in a physically based pipeline it darkens the specular light in the dents. Unity's digital human shader uses its cavity map only for smoothness and for specular occlusion (`2026-11-20-pores-and-grain.md`, section 1). At MetaHuman's low levels of detail, where the micro normals are gone, ambient occlusion is multiplied into the base colour "to help fake shadowing" ([MetaHuman materials and textures](https://dev.epicgames.com/documentation/metahuman/metahuman-materials-and-textures)).
- Verdict for kjk. A separate cavity map has no light to answer to in kjk, because the ambient is flat. What we paint today, the dark pore core, is the cavity, and it can stay baked in the colour. This is the case MetaHuman ships at its lowest level. The one change worth making is to cut the cavity's strength once a detail normal exists, so that pores are not drawn twice.

### A detail normal tile repeated over the face

- What it buys. Pores and skin grain without spending the 2,048-pixel texture on them. A tile of about 1.4 cm on a side at 2,048 pixels is sold for exactly this, and one such tile covers only 70 to 130 of our texels (`2026-11-20-pores-and-grain.md`, section 1). Unity's digital human repeats its detail normal 25 times over the texture square, with a strength of 0.3 and a per-region mask (same section).
- What it needs from the renderer. One more texture, a second sampling of the texture coordinates multiplied by a tiling factor, and a way to blend it with the base normal. [Barré-Brisebois and Hill, "Blending in Detail"](https://blog.selfshadow.com/publications/blending-in-detail/), compare the methods. Linear and overlay blending are rejected there. UDN costs 5 instructions and "is described as a better option even when GPU cycles are scarce". Reoriented normal mapping costs 8, 6 without the final normalise, and keeps more detail.
- What it needs from the texture generator. A small tile, generated once, from the same furrow network the paint already draws, and a mask per region from the region map of pore density and size. The tile does not depend on the head, so it is a shared asset, not a per-head one.
- The cost to the look. A tile repeats, so on a face of 24 cm a tile of 1.4 cm repeats about 17 times across, and a repeated pattern can show if the tile has a strong feature. The standard cure is two tiles of different sizes.

### A matcap or another trick that is not physically based

- A matcap looks up a stored lit-sphere picture by the screen-space normal. It needs one texture and almost no code, and it gives a convincing soft sheen in a fixed light.
- It would not answer to kjk's lights. A head turns, the room's lamps are anywhere, and a fight light moves. A matcap's light is stuck to the camera. It also adds no new fine detail unless it is combined with a normal map. I do not recommend it. If the highlight in section 3 proves too costly on the lowest cards, a fixed highlight from the area's brightest light direction, computed once per head, is a cheaper fallback that still answers to the room.

### What it gives on the wet eye and the lips

- Lips. A gloss mask with a medium exponent gives a highlight that moves with the light and the head. A normal map adds the lips' fine vertical lines. The lip gloss slider already exists and writes colour (`2026-11-19-head-roadmap.md`, the lip paint entry), so it can instead write the mask.
- Wet eye. The eye is a separate mesh with its own texture. The mod heads give the eyeballs a shine mask too (section 2), which suggests the same path. I did not measure how the procedural eyeball in `head_paint_gpu` would take a highlight, and I did not verify how the eye mesh is lit in kjk.
- Skin grain at conversation distance. Both maps are close to invisible there. A pore is below one pixel. The visible gain is the highlight's shape, which breaks up over the grain.

### Games that shipped faces with few maps

- Source. Its character materials use a colour texture, a normal map whose alpha is the phong mask, and optionally an exponent map (Valve wiki as above). I could not open Valve's SIGGRAPH 2006 course notes ([Shading in Valve's Source Engine](https://media.steampowered.com/apps/valve/2006/SIGGRAPH06_Course_ShadingInValvesSourceEngine.pdf)), because the tool returned the compressed file, so I do not quote them.
- Star Wars The Old Republic. A community wiki on the game's extracted files says its glossiness texture holds the specular colour in RGB and the shininess in alpha, its `_n` texture holds the normal map split across the green and alpha channels, and a head material can add complexion, face paint, age and wrinkle maps ([SWTOR-Slicers wiki, a community wiki, not a developer source](https://github.com/SWTOR-Slicers/WikiPedia/wiki/SWTOR-materials-and-texture-files)). It also says the diffuse alpha, where it exists, masks the eyes, nostrils and mouth on heads. That is the same idea as the KOTOR alpha mask.
- Mass Effect, Dragon Age Origins and Jade Empire. I found no first-party source on their face texture sets. Mass Effect ran on Unreal Engine 3 (Epic's own blog), but none of what I found describes its skin maps. The only statement about Jade Empire is the `bumpMapIsSpecular` flag above.
- The real-time skin literature. The GPU Gems 3 chapter paints the specular roughness and intensity "in a texture" and says Weyrich's measured values "can be easily painted onto a face using a low-resolution two-channel map". It also says white specular colour is right for skin in linear space. This supports a low-resolution gloss mask: the specular parameters need far fewer texels than the detail.

## 4. What it would cost in kjk

These costs are estimates. No frame time was measured on a GTX 1050 Ti, the project's minimum card (`2026-11-18-minimum-gpu-target.md`, "Open questions"), and I ran no benchmark.

### Memory

A 2,048 by 2,048 texture with four 8-bit channels takes 16.8 MB, and 22.4 MB with its chain of smaller sizes.
The minimum card has 4 GB (`2026-11-18-minimum-gpu-target.md`, "Recommendation").

| Map | Format | Size with its sizes chain (estimate) |
|---|---|---|
| Gloss mask in the alpha channel | no new texture | 0 MB |
| Normal map for wrinkles at 2,048 | two 8-bit channels, third rebuilt | 11.2 MB per head |
| Same, block compressed (BC5) | one byte per texel | 5.6 MB per head |
| Detail tile at 512 | two 8-bit channels | 0.7 MB, shared by every head |
| Cavity map at 2,048 | one 8-bit channel | 5.6 MB per head |

Heads on screen at once are few, so even uncompressed the normal map adds under 50 MB for four heads.
The paint is made on the GPU by `head_paint_gpu`, so compression would be a second pass.
Leaving it uncompressed is acceptable (estimate).

### Changes in the shader and the material

1. Gloss mask and highlight. In `odyssey.wgsl`, add a highlight in the light loop at line 1706, using the existing light list, the existing `HIGHLIGHT_POWER` style of Blinn-Phong and the existing visibility test. The mask is `1 - texel.a`, as today. The material needs a new flag so that a head with no environment map still uses the alpha as a mask and not as coverage (`odyssey.wgsl:1881`, `material.rs:1372`). The changes are one flag, one function and about ten lines of arithmetic for each ordinary light. A light that casts a shadow also calls `light_visibility` for its highlight, which is the costly part (`odyssey.wgsl:874-897`). The highlight could reuse the visibility already computed for the diffuse term at line 1748 instead of asking again.
2. Normal map. The material gets one more texture binding after binding 18 (`material.rs:1069` onward builds the bind group layout by hand), and a flag. The fragment shader reads the map, builds the tangent-space frame from `in.world_normal` and `in.world_tangent`, and uses the result for the diffuse `facing` and the highlight, but not for ray starts or shadow tests, which should keep the geometric normal (`odyssey.wgsl:1689`). The mesh gets the tangent attribute.
3. Detail tile. A second binding and a second texture-coordinate multiply, plus a blend of two normals, 5 to 8 instructions ([Barré-Brisebois and Hill](https://blog.selfshadow.com/publications/blending-in-detail/)).
4. Prepass. No change (section 1).
5. Mesh. Tangents for the custom head only, written by the Rust head code or by Bevy's builder. KOTOR's own heads keep none and stay on the old path.

### Cost per pixel

- The fragment shader already loops over up to 32 lights, each with a shadow trace, and runs the fight lights and the room's shadows (`odyssey.wgsl:1706-1752`). A normal map adds one texture read and about 20 arithmetic instructions for the frame, and the detail tile adds one read and about 8 more (estimate). Compared with the light loop this is small.
- The gloss highlight is a per-light cost. With three or four lights near a character it adds about 10 instructions each (estimate). If it reuses the visibility already traced, it adds no new shadow work.
- A head fills a small share of the screen in play and a large share in a close-up, so the new cost falls where the pixels are few, except in the close-up. I do not know how the close-up behaves on a GTX 1050 Ti. It should be measured with the Vast.ai method of the `kjk-vast-bench` skill before it ships.
- The original engine set a higher bar. Its bump pass needed special hardware and fell back when it was missing (section 2), so a setting that turns the new maps off is in line with the original. The loose-parts direction in memory also asks that costly changes be a setting.

## Recommendation

In order of value for effort.

1. **A gloss mask in the colour texture's alpha, with a highlight from the area's lights.** Paint the lips, the eye corners, the nose and the forehead's patch into alpha (255 means matte). Add the highlight to the light loop and a flag so a head without an environment map reads the alpha as a mask. This needs no tangent and no new texture. It is also the one change that matches how the original engine and the mod heads work, so the head sits well next to KOTOR's own. It replaces the painted sheen and lip gloss, so remove those from the colour when the highlight is on. Keep a fixed-strength painted version for the original lighting mode, so that Classic lighting still looks as KOTOR does.
2. **A repeated detail normal tile for the pores and grain.** It costs 0.7 MB for all heads and has a mask per region from the pore region map. Its effect shows only in highlights and under directional light, so it comes second, after the highlight exists. When it is on, lower the pore depth and drop the light rim of each pore in the colour, so the pore is drawn once.
3. **A per-head tangent-space normal map for wrinkles and folds.** The fold layer already draws the furrows, so the generator can write their height and turn it into a normal. It needs tangents on the head mesh and one more binding. It is the most work, and it gives the least that the detail tile does not already give at conversation distance. It matters in the close-up, where an expression wrinkles the forehead. Build it only if the close-up looks flat after steps 1 and 2.
4. **Do not add** a separate cavity map, an occlusion map or a matcap.

What stays in the colour: the skin tone, the regional tints, freckles, moles, make-up, the dark core of a pore, the dark line of a wrinkle, and the lip colour.
Keeping the dark core and the dark line follows MetaHuman's low level of detail, and it follows the earlier note's rule that a pore is a small darkening, soft, and slightly red (`2026-11-20-pores-and-grain.md`, section 4).
What moves into maps: the shine of the lips, the eye, the nose and the forehead (mask), the pore relief and the skin grain (detail tile), and later the deep folds (normal map).

The head must still sit next to KOTOR's heads, which have only a colour texture and sometimes a weak alpha mask.
With the highlight switched off, or under Classic lighting, steps 1 to 3 change nothing, and the head falls back to its painted colour, as KOTOR's do.
The highlight's strength should be tuned on the lips of the mod heads, whose alpha reaches 36% shine at most.

Before building, three checks are cheap.
- Count the stock heads' `envmaptexture` and `bumpmaptexture` lines on the Windows install with the scene's tally, to learn how many KOTOR heads already carry a mask.
- Render one head with a hand-made alpha mask and the existing environment map, to see how far the old path goes without any code.
- Bench the highlight on the GTX 1050 Ti class card before choosing its default.

## What I could not find out

- Whether any stock KOTOR 1 or 2 head uses `bumpmaptexture` or `envmaptexture`. The game files are on the Windows PC only, and I did not run the scene tally there.
- The exact bump-pass formula of the retail executable. I read K12's reconstruction and the Odyssey names and structure. K12's header cites a note that is not in its clone, and the Odyssey vertex programs are register-level code that I did not translate.
- How many lights the original's bump pass used for one mesh. The names `GetBumpMapLights` and `EnableBumpMapLights` exist (`cmake/order/swkotor.txt:2599-2631`, Odyssey), and I did not read their limit.
- Which stock or mod meshes in KOTOR carry tangent bases. The tangent flag is a property of the model file, and I did not parse any.
- First-party sources on the face texture sets of Mass Effect, Dragon Age Origins, Jade Empire and The Old Republic. I found only the Jade `bumpMapIsSpecular` flag, a community wiki for The Old Republic, and nothing for the other two.
- Valve's own text on phong masks. The developer wiki returned an error, and the SIGGRAPH 2006 course notes came back as a compressed file that I could not read. The Source claims rest on a search summary of the wiki.
- Any frame time on the GTX 1050 Ti, the GTX 1650 or integrated graphics. All costs above are estimates.
- How the procedural eyeball would take a highlight, and how the eye mesh is lit in kjk.
- Whether the visible gain of a normal map at conversation distance survives kjk's TAA and MSAA. I did not render anything.
