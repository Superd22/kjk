# Procedural lip paint

Date: 2026-11-20

This note looks at how games and character creators paint lips, and it proposes a lip layer for our head that is computed on the GPU from the distance of each pixel to the lip border curve of the 3D head. The head texture is 2,048 pixels wide and one pixel covers about 0.12 mm of face. At that scale 1 mm is about 8.3 pixels.

It builds on two earlier notes, `2026-11-19-procedural-face-paint.md` and `2026-11-19-procedural-skin-detail.md`. Those notes already cover the lip lines method (their method L1), the Skyrim tint list and the Baldur's Gate 3 channel layout. This note does not repeat them in full.

How to read the sources. The search tool and the page reader could not open several pages. The Epic MetaHuman documentation pages came back empty because they need a browser. The Nexus Mods pages, the Reallusion skin guide PDF and the Imai and Tsumura abstract returned an error or binary data. For those I rely on a search result's summary of the page, and I say so. A statement marked "inference" is mine and is not stated by any source.

## Summary of what is most useful for us

1. **No source publishes a lip mask made from the mesh.** Every creator I could read uses either a painted mask in UV space or a fixed set of lip shapes. I found no published method that derives the vermilion border from landmarks or from the 3D surface, and none that changes how sharp the border is. Our distance-to-border design is therefore our own, and nothing I found contradicts it.
2. **The shipped creators agree on the controls.** They give the player a colour, a strength (opacity), a gloss or roughness control and a choice of shape. A few add a second "accent" or "liner" layer. Starfield, EVE Online, Fallout 4 and Reallusion all do this (details below).
3. **Natural lip colour is a skin pigment story.** Baldur's Gate 3 stores hemoglobin, melanin and veins in separate texture channels, and medical sources say the vermilion is red because it has many surface blood vessels and few melanocytes. Our design should therefore derive the natural lip colour from the skin tone, with more blood and less melanin, and keep lipstick as a separate layer with any colour.
4. **Lipstick should keep the lip's own shading.** Published virtual makeup methods convert to CIELAB or HCL and keep the original luminance, so the lip shape does not turn into a flat colour. I recommend we do the same.
5. **Lip detail has little published support.** The only documented items are Baldur's Gate 3's lip lines channel, Reallusion's "contour" and "corner" layers, and a few painting tutorials. Everything else in the design below (the white roll band, the mouth line, the corner darkening, the upper lip shade) is my own design.
6. **I found no documentation that KOTOR, Mass Effect or Oblivion artists painted a darker upper lip and a lit lower lip.** I could only confirm that hand painted textures in general carry painted lighting.
7. **The inner lip seam is best solved by evaluating the same 3D distance on the inner lip vertices.** This is my inference. The only related source is Reallusion's warning that high blending values can create a UV seam.

## 1. Character creators and their lip controls

### Epic MetaHuman

- The lip controls are in the Materials, Makeup panel. The documentation says the Lips section has a number of premade lipstick styles, and that after choosing one the player sets color, roughness, opacity and metalness ([Epic, makeup material tools](https://dev.epicgames.com/documentation/metahuman/makeup-material-tools), from a search result's summary).
- The Unreal Engine API struct for lip makeup lists the same fields, which are a type, a color, a roughness, an opacity and a metalness ([Epic API, FMetaHumanCharacterLipsMakeupProperties](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/MetaHumanCharacter/FMetaHumanCharacterLipsMakeupPro-), from a search result's summary). That is four sliders plus one colour picker and one style list. I found no lip liner field.
- The docs describe opacity oddly. The summary says "Lower values result in higher opacity", so the control may be inverted. I could not confirm this.
- The Skin panel has an Accents tab with named face regions. Epic's region list includes Lips. Each accent has Redness, Saturation and Lightness values, each limited to 0 to 1 in the C++ API, and the changes are always symmetric ([Epic, skin material tools](https://dev.epicgames.com/documentation/metahuman/skin-material-tools), [API struct for the accent properties](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/MetaHumanCharacterEditor/FMetaHumanCharacterSkinMaterials), both from search result summaries). This is the natural lip colour control, and it is separate from lipstick.
- The lip shape comes from the style choice. I found no statement about whether it follows the mesh. A MetaHuman head has one fixed UV layout, so the styles are most likely masks in UV space (inference).

### Reallusion Character Creator 4 and SkinGen

- The manual's Lips page names four lip layers. They are Base, which gives the lips a basic shade that suggests vitality, Contour and Corner, which both help imply the shape of the lips, and Highlight, which adds highlights so the lips look moist ([Reallusion, Lips](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/16_Makeups/Lips.htm)). This is the only documented source of "corner" and highlight layers I found.
- The skin guide says specular and gloss can be adjusted for all sections of the lip, and that blending and opacity set whether the makeup is sharp or soft. It warns that too high a blending value may cause an unwanted UV seam ([Reallusion, skin design guide, from a search result's summary](https://reallusion.com/tw/character-creator/Character_Creator-Character_Skin_Design_Guide.pdf)). This shows that Reallusion has one softness control per layer and that its masks live in UV space.
- Lip makeup templates come in two kinds. Lip Makeups cover the whole lip area and Shape templates emphasize the outline ([Reallusion, using lip makeup templates](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_SkinGen/Makeup/Lip/Using-Lip-Makeup-Templates.htm)).
- The skin guide lists three lip normal textures, and a custom one can be chosen with "Input" ([same guide, from a search result's summary](https://reallusion.com/tw/character-creator/Character_Creator-Character_Skin_Design_Guide.pdf)).
- The head shader also has area masks for roughness and subsurface scattering. The NMUIL mask holds nose, mouth, upper eyelids and inner eyelids in four channels. The CFULC mask holds cheeks, forehead, upper lip and chin ([Reallusion, area mask textures](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_Digital_Human_Shader/Head/Texture_Area_Mask.htm)).
- That is four layers, each with a colour, a strength, a blend value and gloss. The shapes are masks in UV space.

### Skyrim, Fallout 4 and mods

- Skyrim has a tint type called Lip Color, number 1 in a list of 14 ([UESP, race record format](https://en.uesp.net/wiki/Skyrim_Mod:Mod_File_Format/RACE), as quoted in the earlier note). Each tint has a mask file, a list of colour presets and a strength from 0 to 1. The Creation Kit describes tint layers as textures blended together, with a preset colour and a slider that governs how much of it blends in ([UESP, Race](https://ck.uesp.net/wiki/Race), from a search result's summary).
- The Fallout 4 LooksMenu Tint API lets modders add head tints for makeup, dirt, scars or tattoos. The tint types are Mask, ChargenDetail, OnOff and TakesSkinTone ([LooksMenu on Nexus Mods](https://www.nexusmods.com/fallout4/mods/30163), from a search result's summary). Its masks and diffuse textures must be exactly 1024 by 1024 pixels, and specular maps 512 by 512.
- One makeup pack for that API has 1 lipstick and 2 lip liners ([Nexus Mods, from a search result's summary](https://www.nexusmods.com/fallout4/mods/12631?tab=description)). Another mod adds separate upper and lower lipstick entries ([Nexus Mods](https://www.nexusmods.com/fallout4/mods/30163?tab=posts)).
- A forum answer says vanilla lipstick is a palette tint with a list of colours rather than a mask tint with a free colour ([Nexus forum](https://forums.nexusmods.com/topic/12536794-help-with-replacing-fo4-makeup-textures), from a search result's summary).
- The Skyrim warpaints in RaceMenu are texture images, and one pack's makeup textures do not line up on Fallout 4 because the UV map differs ([Nexus Mods, from a search result's summary](https://www.nexusmods.com/skyrim/mods/72955?tab=docs)). This shows the weakness of UV masks. They break when the layout changes. A mask computed from the mesh does not.
- The shape does not follow the mesh in either game. It is a painted mask.

### Cyberpunk 2077

- The character creator has a Lip Makeup Style with 4 options (off, default, glossy, matte), a Lip Makeup with 20 numbered options and a Lip Makeup Color ([Cyberpunk modding wiki, character creator cheat sheet](https://wiki.redmodding.org/cyberpunk-2077-modding/for-mod-creators-theory/references-lists-and-overviews/cheat-sheet-character-creator)). The colours and styles are entries in the mesh's appearance list. In other words each combination is a ready-made appearance and not a free colour.
- That is 3 choices and no free sliders. The lip shape does not follow the mesh.

### Starfield

- The creator has Lipstick Base, which sets colour and intensity, and Lipstick Accent, which adds liner and similar accents with its own colour and intensity ([Insider Gaming, from a search result's summary](https://insider-gaming.com/starfield-character-creation-explained/), [EIP guide](https://eip.gg/starfield/guides/character-creation)). I could not open the pages, so I take this from the search summary.
- That is two layers, each with a colour and an intensity.

### EVE Online

- The EVE University wiki lists Lipstick with colour, glossiness, opacity and a choice of styles, such as top lip only or half of the bottom lip ([EVE University, character creator](https://wiki.eveuniversity.org/Character_Creator), from a search result's summary).
- A 2011 review advises keeping makeup opacity below about 60 percent, because heavy colour looks garish ([Engadget](https://www.engadget.com/2011-02-20-eve-evolved-making-a-better-avatar.html), from a search result's summary). I use this in the slider ranges below.

### Baldur's Gate 3

- The CLEA texture keeps the lips in its blue channel, next to cavities and pores (red), eyebrows (green) and ambient occlusion (alpha). The MSK texture keeps tear and lip lines in its blue channel. The HMVY texture holds hemoglobin (red), melanin and freckles (green), veins (blue) and yellowing (alpha) ([BG3 modding wiki, face and body textures](https://bg3.wiki/wiki/Modding:Face_and_body_textures)). The page has no lip colour section, so how the shader uses these channels is not stated.
- The game has a Makeup atlas whose red channel holds the makeup options ([same page](https://bg3.wiki/wiki/Modding:Face_and_body_textures)).
- I did not find the in-game lip control list.

### Dragon Age The Veilguard

- The creator has "over 30 options like glitter, blush, eyeliner intensity and color, and more" ([GamingBolt](https://gamingbolt.com/dragon-age-the-veilguard-character-creator-complexions-qunari-horn-materials-and-more-detailed)). The article does not name a lip control. The earlier note cites Game Informer for lips among the makeup options.

### The Sims 4

- Lipstick is a category in Create a Sim, and creators ship it as swatches, often 8 to 24 colours ([CurseForge, example lipstick](https://www.curseforge.com/sims4/create-a-sim/lipstick-46-hq), from a search result's summary). A forum thread reports that swatch colours on the display do not match the colours on the lips ([EA forums](https://forums.ea.com/idea/the-sims-4-bug-reports-en/collection-issues-with-swatches-cas/5036207)). I found no official slider list.

### Black Desert, Saints Row

- The earlier note lists Black Desert's skin, makeup, blush and lips controls with placement and opacity ([official wiki, via the earlier note](https://www.naeu.playblackdesert.com/Wiki?wikiNo=5)). I did not find Saints Row lip controls.

### What the creators have in common

| Control | Who has it |
|---|---|
| Colour | MetaHuman, Starfield, EVE, Skyrim, BG3 (colour presets), Reallusion |
| Strength (opacity or intensity) | MetaHuman, Starfield, EVE, Skyrim, Reallusion |
| Gloss or roughness | MetaHuman, EVE, Reallusion, Cyberpunk (matte, default, glossy) |
| Metalness | MetaHuman |
| Liner or accent layer | Starfield, Fallout 4 mods, Reallusion (contour) |
| Shape choice | MetaHuman, EVE, Cyberpunk, Reallusion |
| Natural redness separate from lipstick | MetaHuman (accents) |

Most creators give 3 to 6 controls per lip layer. None gives a control for the sharpness of the border except Reallusion, whose blending value serves that purpose. In every case the shape is a mask in UV space, or a choice of fixed shapes. None follows the mesh.

## 2. How the lip mask is made

- **UV space masks are the standard.** Skyrim tint masks, Fallout 4 tint masks, the BG3 lips channel, Reallusion's layers and the MetaHuman styles are all images in UV space (sources in section 1).
- **No distance fields or vertex colours.** I found no creator that documents a distance field or vertex colours for lips. The Blender Artists thread uses vertex paint masks with mix nodes, but that is a hobbyist example ([Blender Artists](https://blenderartists.org/t/shading-the-lips-beginner-help/1366410), from a search result's summary).
- **Softening and sharpening one mask.** A Renderosity thread suggests blurring the lip mask and then sharpening it with nodes, so one mask gives an adjustable edge. It also says the mask must stay inside the lip material zone, or a hard seam appears ([Renderosity, 2009 Poser discussion](https://api.renderosity.com/forums/threads/2763856/print), from a search result's summary). This is the only published way of changing border sharpness that I found, and it is a forum post.
- **Landmarks in virtual try on.** Patents and hobby projects find the lip region with landmarks and a mask with values from 0 to 1. One project found it helped to blur the mask so the colour bleeds slightly past the lip border ([jwdinius, virtual makeup](https://jwdinius.github.io/blog/2020/virtualmakeup), [patent 11501479](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11501479), both from search result summaries). This is photo work and not a 3D head, but it matches our case of a mask from curves.
- **Decals.** I found no lip decal system. Epic has no lip decal in the MetaHuman docs I could read.
- **Conclusion.** The recommended design is ours. A signed distance to the border curve gives one number per pixel. A smooth step over that number gives the mask, and a second smooth step gives each band such as the white edge and the liner. All widths are in millimetres, so they hold if the texture size changes.

## 3. How the colour is blended

- **Overlay for lip masks.** The earlier note records that the LooksMenu documentation uses Overlay in its example entries for lip masks ([LooksMenu](https://www.nexusmods.com/fallout4/mods/30163), from a search result's summary).
- **Multiply layers in hand painting.** A hand painted skin guide puts the lip base on a Multiply layer using the base blood colour, with adjustable opacity ([Instructables](https://www.instructables.com/Hand-Painted-Game-Texture-Character-Skin), from a search result's summary).
- **Multiply changes the base.** A Blender user notes that layering lip colour with Multiply alters the base colour ([Blender Artists](https://blenderartists.org/t/help-understanding-vertex-paint-layers-and-mix-node/1190539), from a search result's summary).
- **Luminance kept in CIELAB or HCL.** One virtual makeup patent converts the lip image and the lipstick colour to CIELAB so luminance is separate from colour, and an alpha value sets how much of the lip pattern remains ([patent 11039675](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11039675), from a search result's summary). A second patent converts to an HCL space and scales the colour so the lip keeps its broad shape instead of becoming flat ([patent 9603437](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/9603437), from a search result's summary). A third applies a style transfer to luminance only, to give matte, moist or glossy finishes ([patent 11000107](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11000107), from a search result's summary).
- **No shipped game documents its colour space.** Epic and Reallusion say the blend maths is not documented (the earlier note says this for Reallusion).
- **Natural lips follow skin tone.** The sources give the reason and not a game method.
  - The vermilion is red because of many superficial blood vessels and few melanocytes ([StatPearls, lip anatomy](https://www.ncbi.nlm.nih.gov/books/NBK507900/), from a search result's summary). The colour runs from reddish pink to brown depending on ethnicity, and pigmentation is common in darker skinned patients (same summary).
  - A 2023 study of lip pigments estimated melanin and blood volume separately in the skin, the vermilion and the mucosa, and found they differ ([Imai and Tsumura, Color and Imaging Conference 2023](https://library.imaging.org/cic/articles/31/1/27), from a search result's summary). I could not read the numbers.
  - The Jimenez model describes skin colour as the amounts of two chromophores, melanin and hemoglobin ([Jimenez and others, practical appearance model](https://www.iryoku.com/skincolor/)). It does not mention lips.
  - BG3 stores hemoglobin and melanin as separate texture channels (BG3 modding wiki above), and MetaHuman has per region redness accents.
- **Inference for our design.** Natural lip colour should be computed from the same two skin pigments as the face. The lip gets more hemoglobin and, for darker skin, somewhat less melanin than the cheek. Lipstick sits above that and is an overlay or a replacement in a luminance preserving space, and so it does not depend on skin tone.

## 4. Lip detail

- **Vertical lip lines.** BG3 has a lip lines channel. The earlier note has the method L1 for generating lines from noise along the border arc length. Medical papers describe the upper lip with vertical lines and a grid like micro relief ([Flament and others 2013](https://pmc.ncbi.nlm.nih.gov/articles/PMC3790843), as quoted in the earlier note).
- **White roll.** Medical sources describe a pale ridge of 2 to 3 mm along the border between the red lip and the skin ([Taylor and Francis, white roll](https://taylorandfrancis.com/knowledge/Medicine_and_healthcare/Anatomy/White_roll), from a search result's summary). At our scale that is about 17 to 25 pixels wide. I found no game that paints it as a separate layer.
- **Cupid's bow and philtrum.** The upper border curves up to two peaks at the base of the philtrum ridges, with a downward point between them ([NIH, anatomy of the lips](https://elementsofmorphology.nih.gov/anatomy-oral.shtml), from a search result's summary). This is shape and belongs to the mesh and to the border curve we measure from.
- **Dark line where the lips meet and mouth corners.** Painting tutorials describe it. One digital painting lesson picks a dark brown and replaces the centre line, then softens it with a light beige ([Tuts+, realistic lips](https://design.tutsplus.com/tutorials/learn-to-paint-beautiful-realistic-lips-in-adobe-photoshop--cms-23119), from a search result's summary). Reallusion's Corner layer is the closest shipped control, and it only says the layer helps imply the shape.
- **Highlight on the lower lip.** Painting tutorials put a gloss on the lower lip with vertical, curved strokes. One comic tutorial places a saturated pink red along the top edge of the lower lip ([Tuts+](https://design.tutsplus.com/tutorials/learn-to-paint-beautiful-realistic-lips-in-adobe-photoshop--cms-23119), [Pencil Kings, from a search result's summary](https://www.pencilkings.com/?p=3250)). Reallusion's Highlight layer exists to make lips look moist.
- **Procedural methods.** I found no published procedural method for any of these. The profiles below are my design.
  - A line or band is a smooth step of the distance to the border or to the lip seam curve.
  - Lines along the lip use 1D noise of the arc length, as in L1.
  - A highlight is a thin band of the distance from the lower lip centre line, with its brightness raised only in the gloss and specular channels.
  - Mouth corner darkening is a radial falloff from the two corner points.

## 5. Painted lighting in older face textures

- **I found no tutorial or postmortem about KOTOR, Mass Effect or Oblivion lips.** A search for developer talks about Mass Effect and Oblivion faces returned only modding forums and unrelated material.
- **What I could confirm in general.** Hand painted texturing puts the lighting into the diffuse map, and the maps "don't mesh that well with dynamic lighting" ([Reallusion forum, from a search result's summary](https://forum.reallusion.com/293491/Forum13.aspx); [O'Reilly, digital character creation](https://www.oreilly.com/library/view/digital-character-creation/9781000883466/ch8.xhtml)). A Dishonored 2 head texturing account says the lips, nose, areas around the eyes and ears are painted redder ([80.lv, Dishonored 2 characters](https://80.lv/articles/crafting-the-characters-of-dishonored-2-a-detailed-guide-and-strategy), from a search result's summary).
- **Hand painted layering.** One tutorial uses a Multiply shadow layer at about 75 percent opacity after the lighting is painted, and separate layers for lip base and highlights ([Instructables](https://www.instructables.com/Hand-Painted-Game-Texture-Character-Skin), from a search result's summary).
- **Mass Effect.** A modding guide says some Mass Effect diffuse maps are grey and take their colour from a separate tint map, and that the faces have no matching specular map ([DAO tools guide](https://www.tapatalk.com/groups/daitools/guide-texture-information-for-aspiring-modders-t358.html), from a search result's summary).
- **Our own data is the better check.** The KOTOR face textures can be opened and compared with our layer, and the earlier KOTOR head notes already have that data (`2026-11-18-kotor-head-meshes.md`). I recommend measuring the mean brightness of the upper and lower lip in the original textures before choosing the default values of the upper lip shade and the lower lip light. This is a suggestion and not a finding.

## 6. Inner lip and mouth interior

- I found no primary source that documents the inner lip texturing of a shipped game.
- **Mesh practice.** A modelling guide builds the inner mouth by duplicating the last lip edge loop and pulling it back, and calls the mouth corner the hardest part to model ([Autodesk Softimage, modeling the mouth](https://download.autodesk.com/global/docs/softimage2014/en_us/files/face_modeling_ModelingtheMouthArea.htm), from a search result's summary). A Polycount thread asks whether the inner mouth needs its own UV island so it can take a separate material, and one answer says their pipeline uses separate dental textures ([Polycount](https://polycount.com/discussion/comment/2716292), from a search result's summary).
- **The seam risk.** Reallusion warns that high blending values can cause a UV seam, which shows the problem exists for UV masks ([Reallusion skin guide, from a search result's summary](https://reallusion.com/tw/character-creator/Character_Creator-Character_Skin_Design_Guide.pdf)).
- **Inference for our design.**
  - Compute the distance on the 3D surface positions that the texture is baked from. A point on the inner lip and the matching point on the outer lip then get a continuous value even when they sit in separate UV islands.
  - Let the lip colour continue over the inner lip and fade to the mucosa colour of the mouth interior over a few millimetres. The mouth interior should use the same natural lip colour with more hemoglobin and no lipstick, so no seam appears when the mouth opens.
  - Make the lipstick fade to zero a little before the wet line, because real lipstick stops at the wet line.
  - The earlier note's table says Euclidean distance to a curve is wrong where the skin folds back. The inner lip and the outer lip fold back at the wet line, so use the distance along the lip surface or a per vertex value from the mesh and not the 3D straight distance there.

## Recommended design

The layer sits above the base skin tone and the pores, and below the makeup and decals, as in the roadmap. Each pixel needs three numbers from the head. The first is the signed distance `d` to the lip border curve in millimetres, positive inside the red lip. The second is the position along the border `a` in millimetres from the left corner. The third is the distance `m` to the lip seam curve, where the lips meet. All three can be baked into a small data texture or computed from curves in the shader.

These are 13 sliders and one colour picker, named for what a player sees.

| # | Slider | Range | What the player sees | Method |
|---|---|---|---|---|
| 1 | Lip redness | 0 to 1 | More blood in the natural lip colour | Adds hemoglobin to the skin pigment pair, inside the border |
| 2 | Lip darkness | 0 to 1 | Browner, deeper natural lips | Adds melanin, which is stronger on darker skin |
| 3 | Lip edge softness | 0.1 to 1.5 mm | A crisp or blurred border between lip and skin | Width of the smooth step on `d`. At 0.12 mm per pixel, 0.1 mm is under one pixel, so give the lower bound as 0.25 mm in practice |
| 4 | Lip edge paleness | 0 to 1 | A pale thin rim around the lips | White roll band 0 to 2.5 mm outside the border, which is about 20 pixels |
| 5 | Upper lip shade | 0 to 1 | The upper lip is darker than the lower lip | Multiplies the upper lip luminance down, to a default set from KOTOR measurements |
| 6 | Lower lip shine | 0 to 1 | A soft light on the lower lip | Raises specular and gloss on a band of the lower lip, and a little albedo |
| 7 | Lip lines | 0 to 1 | Fine vertical lines on the lips and above them | Method L1, with arc length `a` |
| 8 | Mouth line darkness | 0 to 1 | A dark line where the lips meet | Smooth step of `m`, about 0.3 to 0.6 mm wide |
| 9 | Mouth corner darkness | 0 to 1 | Shadow at the corners of the mouth | Radial falloff from the two corner points |
| 10 | Lipstick strength | 0 to 0.9 | How much lipstick covers the lip | Mix strength. EVE reviewers suggest staying under 60 percent for natural looks, so show 0.6 as the "natural" mark |
| 11 | Lipstick colour (picker) | any | The lipstick colour | Replaces hue and chroma in a luminance preserving space, such as CIELAB, and keeps the natural lip luminance |
| 12 | Lipstick gloss | 0 to 1 | Matte, satin or wet look | Lowers roughness and raises specular inside the lipstick mask |
| 13 | Lip liner strength | 0 to 1 | A darker outline in the lipstick or a different colour | A band 0 to 0.8 mm inside the border, using the picker colour darkened |
| 14 | Lip liner width | 0.2 to 1.2 mm | A thin or thick liner | Width of the liner band |

This table has 14 rows. If we want fewer, drop slider 9 (it can be a fixed fraction of 8) and slider 14 (fix the liner width at 0.6 mm), which gives 12.

How the pieces combine, in order:

1. Compute natural lip colour from the skin's hemoglobin and melanin, using sliders 1 and 2.
2. Apply the white edge (4), the upper lip shade (5), the lines (7), the mouth line (8) and the corners (9) as multipliers on luminance.
3. Mix in lipstick (10, 11), with luminance from step 2 kept, so that the lines and shade show through the lipstick as they do on real lips.
4. Add the liner (13, 14) and then the gloss (6, 12) into the roughness and specular channels.

Things to check once built:

- Show the result next to the KOTOR original in the viewer, as the repository rules require, on at least a pale head and a dark head.
- Check the border at 2,048 pixels with softness 0.25 mm. That is only 2 pixels, so the mask needs an anti aliased smooth step and sRGB aware blending.
- Check the inner lip with the mouth open at the widest viseme.

## What I could not find

- The exact lip controls of The Sims 4, Black Desert, Saints Row, Baldur's Gate 3 and Dragon Age The Veilguard from a first hand source. The base pages were closed or JavaScript only.
- The exact names of sliders in Reallusion's lip section, and whether the layers have separate gloss sliders. The manual page names only the four layers.
- The Epic MetaHuman documentation text itself. I used the API summaries and search result summaries, and the inverted opacity wording is unconfirmed.
- Any published method that finds the vermilion border from the mesh or landmarks in a game or character creator, and any method that sets border sharpness other than a forum post.
- The blend modes and colour space of Skyrim and Fallout 4 lipstick. The earlier note states Overlay from the LooksMenu documentation, and I could not reopen it.
- The numbers from Imai and Tsumura on melanin and blood volume in the lip.
- Any texture painting tutorial, postmortem or developer talk about painted lip shading in KOTOR, Mass Effect or Oblivion.
- Any source on inner lip texturing in shipped games.
