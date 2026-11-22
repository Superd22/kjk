# Eye makeup in character creators and digital human tools

Date: 2026-11-20. Scope: how shipped creators build eyeliner, eyeshadow and periocular paint, and what makeup artists call the parts of the eye.

Source marks used below. "Primary" means I read the page itself. "Search summary" means I only saw a search engine summary of the page, so treat it as unconfirmed. "Not found" means I looked and no source gave the fact.

## What this means for our eye layer

These are proposals for the kjk eye paint layer. Each one rests on the findings further down.

1. Store each effect as a greyscale mask plus a colour, then blend with alpha. Both the Reallusion SkinGen manual and the 3D makeup papers use a mask with an opacity, and the papers write the blend as `M_b * M_a + (1 - M_a) * A_b` (links in the sections on SkinGen and papers).
2. Give every effect only three player controls, which are colour, amount (opacity) and one shape slider. MetaHuman, Dragon Age and Black Desert all expose colour and strength, and none of the three exposes a free-hand drawing control.
3. Make the liner shape sliders wing length, wing angle and thickness. Black Desert's official guide lists shape, length and colour for eye lines. No shipped creator I could read documents a wing angle slider, so this one is our own addition.
4. Use these distances from the eyelid edge curve. Put the crease about 8 mm above the upper lash line, because adult measurements average 8.2 mm for men and 7.98 mm for women (Eze et al., linked below). Scale all distances by the head's own eye width. The horizontal eye opening averages about 32.7 mm in that study.
5. Use these blend modes. Use normal alpha for liner and lash lines. Use multiply for the eye socket, the under eye darkness and the crease shadow. Use alpha for eye shadow, with an optional overlay for a second tone. SkinGen offers Normal, Multiply, Overlay and Softlight, and Sims 4 creators use Multiply or Overlay for makeup layers.
6. Keep the fixed layer order of SkinGen. Foundation comes first, then eye makeup, then eyelashes. In our case that means skin, then socket and under eye darkness, then shadow and crease, then liner, then lash lines on top.
7. Compute each mask from the signed distance to the eyelid edge curve, in millimetres on the 3D head, and smooth the edges with a falloff. Do not use a fixed UV stencil. Skyrim's vanilla face tint masks are 256 by 256 pixels (replacement packs use 1024 by 1024), which is why they look soft at close range.
8. Offer the socket and under eye darkness as skin accents with their own sliders, apart from makeup. MetaHuman lists dark circles under the eyes in its skin Accents, not in its makeup panel. A character can then have dark circles without wearing any makeup.

## Makeup artist vocabulary and sizes

The parts of the eye below come from Makeup Geek's page on the anatomy of eye makeup. That page gives no measurements in millimetres or centimetres. Source, primary: [Makeup Geek, Anatomy of Eye Makeup](https://www.makeupgeek.com/content/articles/anatomy-of-eye-makeup).

- The lid is the largest area for shadow.
- The crease is the folded skin just above the lid, where medium to dark matte shadow adds depth.
- The transition area sits just above the crease, and light to medium matte shades blend harsh lines there.
- The outer V is the outer corner of the eye. It runs from the lash line up to the crease and forms a sideways V.
- The inner corner is next to the tear duct and takes light, bright shades.
- The highlight area is the brow bone just under the brow.
- The lower lash line is the area just under the bottom lashes, and it takes medium to dark shades.
- The waterline is the inner rim of the lower lid. The page says to use a pencil or gel liner there and never a brush.

Other makeup sources, all primary but without millimetres:

- A tightline is liner applied to the upper waterline, the thin strip where the lash roots meet the lid. Source: [W Magazine, tightlining](https://www.wmagazine.com/beauty/tightlining-eyeliner-makeup-trend) (known to me from a search summary only, the page was not opened).
- The wing should point toward the end of the eyebrow. The author places a dot between the outer eye corner and the end of the brow, draws the wing up to the dot, and joins it back to the lash line to form a triangle. The page gives no length. Source: [Unidays, winged eyeliner](https://www.myunidays.com/US/en-US/blog/article/conquer-the-art-of-the-winged-eyeliner).
- Other guides say to follow the angle of the lower lash line and not to go past the end of the brow. Source, search summary only: [Mecca](https://www.mecca.com/mecca-memo/makeup/winged-eyeliner-eye-shapes-tutorial/) and [Milk Makeup](https://milkmakeup.com/blogs/makeup/how-to-do-winged-eyeliner).
- Not found: a makeup source that gives a wing length in millimetres.

Anatomy measurements that can stand in for the missing makeup sizes:

- The distance from the upper lash line to the lid crease, measured in downgaze, averages 8.2 mm (standard deviation 2.5) for adult Nigerian men and 7.98 mm (standard deviation 2.2) for women. The same study measured a horizontal eye opening of 32.8 mm for men and about 32.7 mm for women, and a vertical opening of 10.6 mm. Source, primary: [Eze et al. 2012, Med Princ Pract](https://pmc.ncbi.nlm.nih.gov/articles/PMC5586704).
- The same paper says its crease values are lower than American Caucasian values and higher than Turkish, Japanese and Brazilian values. Crease height therefore differs by ancestry, so the crease distance should be a slider and not a constant.
- A review quoted by a search summary puts the crease 8 to 11 mm above the lid margin in white men and 6 to 9 mm in white women. I could not open the cited paper, so this is a search summary only: [paojournal.com](https://paojournal.com/index.php/pjo/article/download/73/62/74).

## MetaHuman

MetaHuman has a makeup panel and a separate skin Accents panel. I could not read the Epic pages directly because they render with scripts, so everything in this section is a search summary.

- The makeup panel has Foundation, Eyes, Blush and Lips. The eye section offers premade styles that combine eyeliner and eyeshadow. Source, search summary: [Makeup material controls](https://dev.epicgames.com/documentation/metahuman/makeup-material-controls).
- The eyeshadow has Roughness (matte or glossy), Opacity and Metalness. The eyeshadow colour is the Primary Color and the eyeliner colour is the Secondary Color. Source, search summary: [Makeup material controls](https://dev.epicgames.com/documentation/metahuman/makeup-material-controls).
- The summary noted that the Japanese page describes Metalness for lipstick and the English page for eyeshadow, so that detail is in doubt.
- The shape of the liner and shadow is therefore a pick from a style list, not a slider. The mask is a texture inside the style, and I did not find how it is blended.
- The skin Accents panel sets Redness, Saturation and Lightness per face region, always symmetrically, and the docs list dark circles under the eyes among its uses. Source, search summary: [Skin material controls](https://dev.epicgames.com/documentation/metahuman/skin-material-controls).
- The API struct for makeup settings exists as `FMetaHumanCharacterMakeupSettings` ([API page](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/MetaHumanCharacter/FMetaHumanCharacterMakeupSetting-)), but the page did not load, so I have no field list.

## Character Creator and SkinGen from Reallusion

All of this is primary from the CC4 and SkinGen manuals.

- Makeup layers sit on top of skin layers and affect only the face. They are, in order, Foundation Makeup, Eye Makeup, Eyelash, Lip Makeup, Eyebrow and Miscellaneous. Source: [Skin and Makeup Layers Concepts](https://manual.reallusion.com/character-creator-4/content/enu/4.0/15_skingen/Basic-Concepts/Skin-and-Makeup-Layers-Concepts.htm).
- Eye makeup comes from a template library with two sub-libraries, Eyeliner and Eyeshadow. A full look uses one of each. Source: [Using Eye Makeup Templates](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_SkinGen/Makeup/Eye/Using-Eye-Makeup-Templates.htm).
- In SkinGen each tool is split into sections. The lip tool has four, which are Base, Heart, Contour and Highlight. Layers can be shown, hidden, reordered and merged, and each has an opacity and a blend mode. Source: [SkinGen function page](https://www.reallusion.com/Character-Creator/skingen-function.html).
- The blend modes are Normal, Multiply, Overlay and Softlight, and the page says they behave like Photoshop's. Every tool has a global layer mask. Global settings usually include Color, Opacity, Normal Strength, Roughness and Metallic. Custom image masks need the paid SkinGen Premium plug-in. Source: same page.
- The older makeup editor lists Color, Opacity, Blending (softens the masked colour into the skin) and Specularity. Source: [Basic Modifications for Makeup](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/16_Makeups/Basic_Modifications_for_Makeup.htm).
- The eyeshadow system has 7 eyeshadow sections and 1 powder layer for glitter. This number is a search summary only, from the SkinGen product page.
- The SkinGen manual pages for eye makeup do not list sections, colours or blend modes for the eye tool. Not found.

## Skyrim and RaceMenu

- RaceMenu has tabs for Makeup (face), Body Paint, Hand Paint and Feet Paint. A user picks a slot, sets a visible colour, then opens a texture browser. Source, search summary: [RaceMenu on Nexus](https://www.nexusmods.com/skyrim/mods/29624).
- RaceMenu reads higher resolution tint masks when SKSE is installed, and the ini value `iTintTextureResolution` is set to 2048 or 4096. Source, search summary: the same Nexus page and linked guides.
- Replacement face tints are 1024 by 1024 pixels, against 256 by 256 for the originals, and are compressed with BC1. Source, primary: [High Quality FaceTints](https://dovaya.github.io/mods/high-quality-facetints).
- Eyeliner is a tint slot, because a mod page says it swaps the texture behind the "EYELINER" slider. Source, search summary: [Nexus mod 115564](https://www.nexusmods.com/skyrim/mods/115564).
- Not found for Skyrim: documentation of the exact names EyeSocket Upper and EyeSocket Lower, and any blend mode field. The Creation Kit wiki page for tint mask types returned an error.
- A search result says the Fallout 4 tint definition format lists Eyeliner, EyeSocketUpper, EyeSocketLower and CheekColorLower, with a BlendOp such as Overlay. That is Fallout 4, not Skyrim, so it is only a hint. Source, search summary: [Fallout 4 tint guide, Nexus mod 30163](https://www.nexusmods.com/fallout4/mods/30163).
- Practical reading. The tint system gives one greyscale mask per area (liner, upper socket, lower socket), one player colour per mask, and one alpha strength. This is the closest shipped model to our plan of separate socket and liner masks.

## Baldur's Gate 3

- The official wiki says the Makeup option can set eye makeup, and lip colour, gloss and metallic tint. It lists no eye makeup colours and no intensity slider. Source, primary: [BG3 wiki, Character Creation](https://bg3.wiki/wiki/Character_Creation).
- Mod pages refer to numbered presets such as makeup 5 and makeup 9, and say some eyeliner styles warp on some head presets. Source, search summary: [Nexus BG3 mod 14424](https://nexusmods.com/baldursgate3/mods/14424).
- A fixed texture per preset that warps on different faces is the failure our per pixel distance masks avoid.
- Not found: the blend mode and the texture format.

## Cyberpunk 2077

- The RED modding wiki lists Eye Makeup as "off, 1-20" and a separate Eye Make-Up Color, and gives no description of how it is built. Source, primary: [Cheat Sheet, Character Creator](https://wiki.redmodding.org/cyberpunk-2077-modding/for-mod-creators-theory/references-lists-and-overviews/cheat-sheet-character-creator).
- A search summary says launch had 8 styles and 9 colours, and that patch 2.2 lists 16 styles. These counts disagree with the 20 on the wiki, so I would not rely on them. Source, search summary: [PCGamesN](https://www.pcgamesn.com/cyberpunk-2077/character-creator).
- The player controls are a style from a list and a colour. Not found: the mask construction and the blend.

## The Sims 4

- Sims 4 eyeshadow and eyeliner are textures with an alpha channel. A creator keeps a white greyscale texture and saves one coloured file per swatch, and each swatch is imported as a colour swatch. Source, primary: [Sims 4 Studio eyeshadow tutorial](https://sims4studio.com/thread/513/eyeshadow-tutorial-using-photoshop-sims4studio?page=1).
- Creator guides put the shadow on a layer set to Overlay or Multiply. Source, search summary: [Sims 4 Studio threads](https://sims4studio.com/thread/21317/eyeshadow-cas).
- The player picks a style and a swatch colour. There is no amount slider in the base game that I could confirm.
- The official EA documentation was not found, so this is community knowledge.

## Black Desert

- The official guide puts eye makeup, eyebrows, blush and lips in a Makeup section. It has a separate "Eye lines" option for the upper and lower lines, which can be adjusted for shape, length and colour. Source, primary: [Black Desert guide](https://www.naeu.playblackdesert.com/en-US/Wiki?wikiNo=5).
- This is the only shipped creator I found that exposes liner length as a player control. The guide does not say what length means in geometry. A Japanese guide says black is the default colour, which is a search summary only: [chimolog](https://chimolog.co/blackdesert-charactermaking-kotsu-makeup/).
- Not found: the mask or mesh method.

## Dragon Age The Veilguard

- Game director Corinne Busche says makeup has more than 30 options, including eyeliner intensity, colour, glitter, eye shadow, lips and blush. Source, primary: [Game Informer](https://gameinformer.com/exclusive/2024/06/13/a-deep-dive-into-dragon-age-the-veilguards-expansive-character-creator).
- The article gives no detail on how the makeup is built or blended. Not found.
- Of all shipped creators here, this is the one that names an eyeliner intensity slider and a glitter option. Glitter is an extra effect that a later version of our layer could add.

## Papers on makeup in UV space

- Makeup Prior Models (Yang et al., CVPR 2024) store makeup as a four channel UV map. The first three channels are the colour and the fourth is an alpha matte between 0 and 1. The compositing is `A_m = M_b * M_a + (1 - M_a) * A_b`, where `A_b` is the bare skin albedo. The paper mentions eyeshadow (including gradient eyeshadow), lipstick and blush and does not mention eyeliner. Source, primary: [arXiv 2403.17761](https://arxiv.org/html/2403.17761v1).
- Makeup Extraction via Illumination-Aware Decomposition (Yang, Taketomi, Kanamori, Eurographics CGF 2023) uses the same alpha blend and builds a PCA model of makeup textures. It adjusts alpha as `clamp(A + sigma, 0, 1)` to fade makeup, with sigma between 0 and 1. The dataset has 3,070 makeup images. Source, primary: [ar5iv 2302.13279](https://ar5iv.labs.arxiv.org/html/2302.13279).
- TinyBeauty (2024) treats eyeliner as a sharp contour. It adds a Sobel edge loss inside an eye mask, and removing that loss made the eyeliner less clear. Source, primary: [arXiv 2403.15033](https://arxiv.org/html/2403.15033v1).
- Lesson for us. A learned model needs an edge term to keep eyeliner crisp. Our computed distance mask gets a crisp edge for free, as long as the falloff width at the liner edge is about one texel.
- Not found: any paper that parameterises eyeliner or eyeshadow by wing length, angle or thickness. The makeup transfer work I found copies shapes from reference photos. Landmark based patents warp eye makeup templates onto the eye, per a search summary only: [US 11039675](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11039675).

## Gaps

- No GDC talk or engine source was readable for any of these creators. Most mask and blend details for BG3, Cyberpunk, Black Desert, The Sims 4 and Veilguard are not public in the pages I could read.
- The MetaHuman documentation could not be read directly, so every MetaHuman fact is a search summary.
- Neither the Skyrim Creation Kit tint mask list nor the RaceMenu source was reachable, so the EyeSocket and Eyeliner slot names are unconfirmed for Skyrim.
- No makeup source gives millimetre sizes for a wing, a tightline or a smudge. The only millimetre numbers are anatomy measurements.
