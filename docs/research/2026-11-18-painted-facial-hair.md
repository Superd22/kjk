# Painted facial hair

This note was researched on 2026-11-18 for kjk's head slider work.
It asks how games draw eyebrows, eyelashes, eyeliner, stubble and beards from settings that a player chooses in a character creator.
It then asks which of those methods suits KOTOR's heads.

**Painted hair** in this note means hair that exists only as colour (and sometimes a normal or roughness change) on the skin of the head.
**Geometry hair** means hair that has its own polygons. It comes in three kinds.

- **Cards** are flat strips of polygons with a hair texture and transparency.
- **Strands** are thousands of thin curves, one per hair or per small group of hairs.
- **Shells** are copies of the skin surface pushed outward in a few layers, each layer showing a different slice of hair dots.

Every claim carries a link.
Many game details come only from modding wikis, mod pages and forum posts, because studios rarely publish how their character creators work.
Each such source is marked as a modding wiki, a mod page or a forum post.
A sentence that starts with "This suggests" or "probably" is my own inference and not a fact from a source.

## Summary

- **The split in shipped games.** Long beards and moustaches are geometry in almost every recent game. Eyebrows are split. Baldur's Gate 3 paints them into a mask channel of the head texture, while Cyberpunk 2077, Skyrim, Dragon's Dogma 2 and MetaHuman give them their own mesh. Eyeliner and eyeshadow are painted everywhere. Stubble is painted almost everywhere, either as part of the skin texture or as a separate density slider. Eyelashes are geometry in the high-end games that document them. MetaHuman draws them as a groom, and inZOI makes them part of the face mesh. On Mac and on phones MetaHuman falls back to cards.
- **The player's controls.** Most games give a preset list for each kind of hair, plus a colour picker. A few games add continuous sliders. Elden Ring has a stubble slider that is separate from the beard style. Reallusion Character Creator 4 lets the user move, rotate, scale and corner-warp the painted eyebrow. Black Desert, Monster Hunter Rise and Monster Hunter Wilds let the player move, scale and rotate makeup and brows. No shipped game that I found draws the eyebrow from a curve with shape numbers, as kjk's prototype does.
- **How hair follows the face shape.** Painted hair follows the face for free, because it lives in the head's texture coordinates. Its drawback is that a stretched face stretches the hair too. Geometry hair needs extra work. Skyrim ships a morph file for every beard. Cyberpunk 2077 ships one for every eyebrow mesh. Dragon's Dogma 2 fits each beard and eyebrow to the face with a wrap deformer when the face is built.
- **KOTOR and its mods keep facial hair painted.** Only two KOTOR heads have a modelled beard. The most downloaded KOTOR beard mod ships only textures.
- **The recommendation** is to keep all of kjk's facial hair as paint drawn by the head's shader. Eyebrows, eyeliner, lashes, stubble and beards should each be a shape made from a few numbers on the head's own landmarks, filled with hair-sized stamps and a direction field, and given a small normal and shadow so that it does not look flat under moving light. Shells and cards are not needed for the KOTOR look. The details and trade-offs are in the last section.

## Shipped games

### Table of the games

The column "Source quality" says whether the evidence is official (the studio or the engine maker), press, a modding wiki, a mod page or a forum post.

| Game | Eyebrows | Lashes and eyeliner | Stubble | Beards and moustaches | Player controls | How hair follows the face | Source quality |
|---|---|---|---|---|---|---|---|
| Baldur's Gate 3 | Painted. A mask in the green channel of the head's CLEA texture. | Eye makeup is painted from a makeup atlas. | No official stubble. A mod adds it as a tattoo. | Meshes, built from hair cards. | Presets and colours. | Paint follows the UVs. Beard meshes are fitted per head. | Modding wiki, mod page, forum post |
| Cyberpunk 2077 | A separate mesh per eyebrow style, with a diffuse, a normal and an alpha texture. | Makeup is a set of extra meshes with textures. | Paint, through mods. | Separate beard meshes, for the masculine body only. | Presets and colours. | Each eyebrow mesh has its own morph target file. | Modding wiki, mod page |
| Starfield | Shape, width, position and colour sliders. | Not checked. | Not checked. | Presets and a colour slider, for one body type. | Presets and sliders. | A modder says that the same eyebrow looks thicker or more arched on different faces. | Guide, mod page |
| Dragon's Dogma 2 | A separate mesh. | Separate menu sections for lashes and eyeliner. | Not checked. | Separate meshes. | Sliders for brow spacing, depth and angle. | A wrap deformer fits the mesh to the face when it is built. | Official (Autodesk interview), guides |
| Elden Ring | 17 shapes, plus colour, luster, root darkness and white hairs. | 4 lash shapes and a colour. Eyeliner and eyeshadow are sliders with colours. | A separate slider from thin to thick. | 12 beard styles with the same colour settings as the hair. | Presets, sliders, colour pickers. | Not documented. | Fan wiki |
| Black Desert | Eyebrow length and style. | Lashes and eyeliner in the makeup section. | Not checked. | A moustache option for male characters. | Move, rotate and size tools for each part, with fine sliders. | Not documented. | Official wiki, guides |
| Monster Hunter Rise and Wilds | Presets. | Presets. | Not checked. | 28 facial hair types in Rise. | Up to 3 makeup or face paint designs in Wilds, each with size, position and angle. | Not documented. | Guide sites |
| The Sims 4 | A CAS category "Facial Hair, Eyebrows". Probably a texture. | Not checked. | A mod paints it on the "Mouth Details" skin layer. | Meshes with their own UVs, whose border vertices must match the head. Some recolours are texture-only. | Presets, colour swatches. | The beard mesh shares the head's border vertices. | Mod pages, forum post |
| Skyrim with RaceMenu | A head part mesh (type "Eyebrows"). RaceMenu can paint a brow overlay on top. | Eyeliner is a tint mask. | Paint, through overlays. | Head part meshes (type "Facial Hair"). | Presets, colours, and texture overlays the player picks. | Each head part needs a TRI morph file. RaceMenu bakes overlays into a tint mask. | Fan wiki, mod pages |
| Fallout 4 with LooksMenu | Male brows are a face tint category. | Makeup is a face tint category for women. | Not checked. | Head part meshes. | Presets and tint colours. | Tints follow the UVs. | Mod page |
| Mass Effect Legendary Edition | Painted. A mod restores a missing texture that made custom eyebrows and facial hair look sparse. | Makeup presets. | A stubble texture, replaced by mods. | Facial hair textures, plus updated hair models. | Presets and colours. | Paint follows the UVs. | Press, mod pages |
| MetaHuman (Unreal) | A groom (strands or cards). | A groom. Eyeliner and eyeshadow are one painted preset with two colours. | Part of the scanned skin textures. Skins can be filtered by stubble density. | Grooms. 10,000 strands at LOD 0, cards at LOD 2 to 4, nothing from LOD 5. | Presets, colour pickers, opacity sliders. | Grooms are bound to the face mesh. A pipeline option bakes a groom into the face texture at far LODs. | Official |
| inZOI | A MetaHuman groom exported as a static mesh. | Lashes are attached to the face mesh. | Not checked. | Static meshes. | Not checked. | The modding tool attaches the meshes to the face with auto rigging. | Official modding docs |
| Reallusion Character Creator 4 | Painted, in three layers (hair, shadow, replace), each with a normal. | Painted makeup layers. | Painted through SkinGen. | Mesh beards and brows in the facial hair system. | Many sliders, including move, rotate, scale and corner warp of the eyebrow. | Paint follows the UVs. Mesh hair conforms to facial movement. | Official manual |

### Notes on each game

**Baldur's Gate 3.**
The BG3 modding wiki lists the head's CLEA texture as "Cavities/Pores" in red, "Eyebrows" in green, "Lips" in blue and ambient occlusion in alpha ([bg3.wiki, Face and body textures](https://bg3.wiki/wiki/Modding:Face_and_body_textures), modding wiki).
The same page lists a makeup atlas whose red channel holds the makeup options, and a tattoo atlas with three tattoo designs, one per channel.
So BG3 paints eyebrows as a grey mask and lets the shader colour them.
A Steam forum poster describes mixing the skin colour and an eyebrow or lip colour through the green and blue channels ([Steam forum](https://steamcommunity.com/app/1086940/discussions/0/3823047564749772277), forum post).
The modding wiki lists beard meshes in race folders, and says beard material presets control "Roughness, HairSoupleness and BeardIntesity" ([bg3.wiki, Hair Meshes](https://bg3.wiki/wiki/Modding:Hair_Meshes), modding wiki).
Hair models are hair cards, which the wiki calls "long strips of mesh, or planes, with a hair texture applied" ([bg3.wiki, Hair Mod Mesh Setup](https://bg3.wiki/wiki/Modding:Hair_Mod_Mesh_Setup), modding wiki).
BG3 has no stubble option. A mod adds stubble by replacing a face tattoo, so the player can choose its colour and intensity. Its author warns that some heads distort the stubble because of how the texture stretches over the mesh ([Nexus Mods 9479](https://www.nexusmods.com/baldursgate3/mods/9479), mod page, read through a search summary because the page refused a direct fetch).
PC Gamer reports that BG3's beards clip through the face as it moves, on Gale and on custom characters ([PC Gamer](https://www.pcgamer.com/games/rpg/theres-a-mod-to-fix-gales-beard-making-baldurs-gate-3-playable-at-last/), press).

**Cyberpunk 2077.**
The REDmodding wiki lists the head's parts by file prefix. `hb_` is the beard "for male V only", `heb_` the eyebrows, and `hx_` holds makeup, freckles, blemishes, tattoos and scars. A head draws between 3 and 14 meshes ([REDmodding wiki, Cheat Sheet Head](https://wiki.redmodding.org/cyberpunk-2077-modding/for-mod-creators/references-lists-and-overviews/cheat-sheet-head), modding wiki).
"Every eyebrow option has an associated texture file, normal map, and alpha map" (same page).
The eyebrow guide says each eyebrow material uses a diffuse, a normal and a secondary diffuse texture, that the vanilla eyebrow textures are 512x256, and that a `.morphtarget` file names the eyebrow mesh ([REDmodding wiki, CCXL Eyebrows](https://wiki.redmodding.org/cyberpunk-2077-modding/for-mod-creators-theory/core-mods-explained/archivexl/archivexl-character-creator-additions/ccxl-eyebrows/), modding wiki).
So Cyberpunk's eyebrows are a thin mesh over the brow, painted with an alpha texture and a normal map, and moved by a morph target when the face changes.
A stubble mod notes that stubble is loaded separately from the head ([Nexus Mods 6344](https://www.nexusmods.com/cyberpunk2077/mods/6344), mod page).

**Starfield.**
The eyebrows have shape, width and position controls and a colour slider. The facial hair slider and its colour slider exist only for body type 1 ([EIP Gaming guide](https://eip.gg/starfield/guides/character-creation), guide site).
The author of a hand-drawn eyebrow replacer warns that the same eyebrow can look thicker, uneven or more arched depending on the face and eye shape ([Nexus Mods 5107](https://www.nexusmods.com/starfield/mods/5107), mod page, read through a search summary).
This suggests that the brows are drawn in the head's texture space and stretch with the face.

**Dragon's Dogma 2.**
Capcom's technical art director told Autodesk that the team "implemented Maya's wrap deformer into the engine" for the beard, the eyebrows and the helmets. Each part is placed on the face, reshaped by the wrap deformer "at the right time for the scaling of the face", and then the deformer is removed ([Autodesk, RE ENGINE tools for Dragon's Dogma 2](https://blogs.autodesk.com/media-and-entertainment/2025/04/10/ideas-and-ingenuity-behind-capcoms-in-house-re-engine-tools-for-dragons-dogma-2/), official interview).
The face shapes come from about 80 scanned people in one topology (same source).
The eyebrow sliders include spacing, depth and angle, each from -100 to 100 ([GameRiv](https://gameriv.com/dragons-dogma-2-character-creation/), guide site).

**Elden Ring.**
The eyebrow menu has 17 shapes, a colour, luster, root darkness and white hairs. The facial hair menu has 12 beard styles with the same settings, plus a separate "Stubble" slider from "Thin" to "Thick". The lash menu has 4 shapes and a colour. Eyeliner and upper and lower eyeshadow are each a slider from light to dark with a colour ([Elden Ring wiki, Character Creator](https://eldenring.wiki.gg/wiki/Character_Creator), fan wiki).
The stubble slider being separate from the beard style is the closest shipped match to a stubble density setting.

**Black Desert.**
The official guide says players can adjust details such as eyebrow length, and that the controls are of three kinds, moving, rotating and size. The player clicks a part to move it directly and then uses a control bar for fine changes ([Black Desert official wiki](https://www.naeu.playblackdesert.com/en-US/Wiki?wikiNo=5), official).
Another guide says the makeup section holds eyebrow colour and style, lashes, eyeliner and lips, with a moustache option for male characters ([Gamepressure guide](https://www.gamepressure.com/blackdesert/character-creation/z18711), guide site).

**Monster Hunter Rise and Wilds.**
In Wilds the player can wear up to three makeup or face paint designs, and each one can be adjusted in size, position and angle ([Game8, Wilds](https://game8.co/games/Monster-Hunter-Wilds/archives/482570), guide site, read through a search summary because the page needs a paid account).
In Rise there are 28 facial hair types and 41 makeup types, and the makeup sliders run from 0 to 100 for vertical and horizontal position and size ([Game8, Rise](https://game8.co/games/Monster-Hunter-Rise/archives/315433), guide site).

**The Sims 4.**
A Sims 4 Studio forum thread explains that a goatee is a separate mesh with its own weights, UVs, vertex paint and textures. Its border vertices must line up with the head's vertices, and it uses a specific green vertex paint to stop clipping ([Sims 4 Studio forum](https://sims4studio.com/thread/21929/solved-facial-hair-clips-head), forum post).
Some beard downloads are "texture overrides" that "contain no mesh" ([ModTheSims](https://db.modthesims.info/d/568511/get-together-full-beard-texture-overrides.html), mod page).
The "Automatic Beard Shadows" mod paints stubble on the "Mouth Details" skin layer, so it works with any facial hair ([CurseForge](https://www.curseforge.com/sims4/mods/automatic-beard-shadows), mod page).
Eyebrows sit in the "Facial Hair, Eyebrows" catalogue section ([ModTheSims, Handsome Brows](https://db.modthesims.info/d/539398/handsome-brows-male.html), mod page). The 9.7 KB recolour file suggests that they are textures, but no source says so.

**Skyrim with RaceMenu.**
A head part record can be of type "Facial Hair" (4) or "Eyebrows" (6) ([UESP, HDPT format](https://en.m.uesp.net/wiki/Skyrim_Mod:Mod_File_Format/HDPT), fan wiki, read through a search summary).
A Nexus guide on making a moustache says the part points at a NIF mesh, and its TRI morph file "auto-populates if the name's right" ([Nexus Mods wiki, Deforming a Skyrim SE mustache](https://wiki.nexusmods.com/index.php/Deforming_a_Skyrim_SE_mustache:_TRIs_%26_NIFs), modding wiki, read through a search summary).
So Skyrim's brows and beards are meshes, and each one carries the morphs it needs to follow the face sliders.
RaceMenu adds texture overlays. When the head is exported, RaceMenu "generates the head mesh and a tintmask texture" with the chosen overlays baked in at their colour and transparency ([Nexus Mods 29624, RaceMenu](https://www.nexusmods.com/skyrim/mods/29624), mod page).
Eyeliner is one of the vanilla tint masks, which a mod replaces with smoother masks ([Nexus Mods 3424](https://www.nexusmods.com/skyrim/mods/3424), mod page).

**Fallout 4 with LooksMenu.**
The LooksMenu tint API lists fixed tint categories. The first is "Brows for male, Makeup for female", followed by markings, blemishes, damage, grime, face tattoos and face paint. Face detail textures such as eyebrows must be BC3 (DXT5) ([Nexus Mods 12631, LooksMenu](https://www.nexusmods.com/fallout4/mods/12631?tab=description), mod page).

**Mass Effect Legendary Edition.**
Kotaku quotes BioWare saying that "customization options and character appearances have also been improved with updated textures and hair models" ([Kotaku](https://kotaku.com/mass-effect-legendary-edition-makes-the-first-game-requ-1846899558), press).
A mod "restores a missing texture" that made "ALL custom Shepard eyebrows and facial hair options" look sparse in Mass Effect 2 Legendary Edition ([Nexus Mods 7](https://www.nexusmods.com/masseffectlegendaryedition/mods/7), mod page, read through a search summary).
This suggests that the remaster kept Shepard's brows and short facial hair as textures on the head.
Another mod replaces the default stubble option and needs the Mass Effect Modder texture tool ([Nexus Mods 303](https://www.nexusmods.com/masseffectlegendaryedition/mods/303), mod page).

**MetaHuman.**
Epic's LOD table gives facial hair (beards and moustaches) 10,000 strands and 15,000 card vertices at LOD 0, 5,000 strands at LOD 1, cards only at LOD 2 to 4 (3,000, 1,000 and 500 vertices), and nothing at LOD 5 to 7. Mac uses cards only, and iOS and Android use cards from LOD 3 ([Epic, platform support and LOD specifications](https://dev.epicgames.com/documentation/metahuman/platform-support-and-lod-specifications-for-metahumans), official).
The LODSync component lists eyebrows, eyelashes, moustache and beard as separate components ([Epic, LODSync](https://dev.epicgames.com/documentation/metahuman/lodsync-component-for-unreal-engine), official).
The groom pipeline has a `GroomTextureMinLOD` setting, described as the best LOD "that this groom will be baked onto the face" ([Epic API, UMetaHumanDefaultGroomPipeline](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/MetaHumanDefaultPipeline/UMetaHumanDefaultGroomPipeline), official, read through a search summary because the page body did not load). MetaHuman Crowd has a matching "baked groom texture" that is bound onto the face material at the listed LODs ([Epic API, FMetaHumanCrowdGroomBakedTexture](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/MetaHumanCrowd/FMetaHumanCrowdGroomBakedTexture-), official, read through a search summary).
This is the texture fallback for far LODs. I did not find a page that says whether the default MetaHuman bakes eyebrows this way.
Stubble is not a groom. The skin tools let the user filter face textures "based on the density of wrinkles, stubble, freckles, and moles" ([Epic, Skin material tools](https://dev.epicgames.com/documentation/metahuman/skin-material-tools), official).
The makeup panel combines eyeliner and eyeshadow in one preset style. The primary colour sets the eyeshadow and the secondary colour the eyeliner, with roughness, opacity and metalness ([Epic, Makeup material tools](https://dev.epicgames.com/documentation/metahuman/makeup-material-tools), official).
Forum users report eyebrows that disappear up close because the brow LODs switch between strands and cards ([Unreal forum](https://forums.unrealengine.com/t/eyebrows-disappear-close-up-to-metahumans/681472), forum post).

**inZOI.**
The inZOI mod kit exports eyebrows, beard and moustache from MetaHuman as static meshes at LOD 0 only. "Eyelashes are attached to the face mesh", and a Blender step "attaches beard and eyebrows to the face (auto rigging)" ([inZOI ModKit, Facial Hair overview](https://mod-docs.playinzoi.com/docs/modkit-docs/modkit/how-to/caz/facialhair/01overview), official).

**Reallusion Character Creator 4.**
This is not a game, but it is the most detailed parametric painted eyebrow that I found.
The eyebrow has three layers. The hair layer draws the brow from a pattern texture, mirrored to the other side by default. The shadow layer darkens the skin under the brow "to simulate the shadow cast by each hair". The replace layer covers the skin under the brow with a solid colour ([Reallusion manual, Eyebrow appearance settings](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_SkinGen/Makeup/Eyebrow/Settings-in-Appearance-Editor-Eyebrow.htm), official).
The hair layer has sliders for sharpening each hair, cutting off the whole brow, cutting off single hairs, edge blur, normal strength per hair, a micro normal noise, roughness, and metallic. It can be moved, rotated and scaled per side, and warped by its four corners (same source).
Reallusion's facial hair system for beards and brows conforms "to any facial movements" ([Reallusion, Hair and beard system](https://www.reallusion.com/Character-Creator/hair.html), official).

**Other games.**
Revelation Mobile splits the face into up to 36 modules with 4 to 6 settings each. The eyebrow's tip, middle and end can each be moved ([Game Developer](https://www.gamedeveloper.com/design/how-to-help-non-professional-players-better-customize-their-character-faces-), press). This is the closest shipped control to kjk's curve-based eyebrow.
Dragon Age: The Veilguard draws head hair with up to 50,000 strands per character ([Microsoft Game Dev, BioWare strand hair](https://developer.microsoft.com/en-us/games/articles/2024/12/bioware-dragon-age-the-veilguard-strand-hair-technology/), official). Forum users report that many of its skin "complexions" already include stubble ([EA forum](https://forums.ea.com/discussions/dragon-age-the-veilguard-discussion-en/feedback-stubble-in-character-complexions/5035696), forum post).

## Techniques for parametric hair painting

### Eyebrows drawn from a curve with a signed distance field

A **signed distance field** (SDF) gives, for every point, its distance to a shape, negative inside and positive outside.
An eyebrow band can be drawn as the set of points closer than a varying half-width to a curve.
Inigo Quilez publishes exact 2D distance functions, including `sdBezier` for a quadratic Bézier curve and `sdUnevenCapsule` for a segment whose two ends have different radii ([Inigo Quilez, 2D distance functions](https://iquilezles.org/articles/distfunctions2d/)).
An uneven capsule chain along kjk's brow ridge gives a band whose thickness changes from the inner end to the outer end, which is what kjk's taper and belly settings describe.
Valve showed in 2007 that a distance field stored at low resolution still gives sharp, antialiased edges when it is magnified ([Green, Improved Alpha-Tested Magnification for Vector Textures and Special Effects, SIGGRAPH 2007](https://steamcdn-a.akamaihd.net/apps/valve/2007/SIGGRAPH2007_AlphaTestedMagnification.pdf)).
This matters for KOTOR's 256x256 head textures, because a distance evaluated in the shader stays sharp however close the camera comes.

A solid band looks like a marker stroke, not like hair.
So the band should only be the mask. The hairs inside it come from the next technique.

### Hair stamps splatted along a direction field

**Texture bombing** divides texture space into a grid. Each cell places one small image at a random offset, picked with a random number per cell. A second random value can pick the image, its colour or its rotation, and a priority value decides which image wins where two overlap ([GPU Gems, chapter 20, Texture Bombing](https://developer.nvidia.com/gpugems/gpugems/part-iii-materials/chapter-20-texture-bombing)).
The chapter warns that each image needs a border of at least two pixels from the edge of its region, or sampling causes artifacts (same source).
For an eyebrow, each stamp is one short hair, rotated to follow a **direction field**. The direction field says which way the hair grows at each point. On a real brow it points up and out near the nose and turns to point along the brow toward the temple.
The curve already gives the direction along the brow, so the field can be a blend between "up" and "along the curve", controlled by one grooming number.
The same stamps can draw stubble dots, moustache hairs and beard hairs with other fields.

Commercial tools already make facial hair this way for artists.
HairTG makes "procedural eyebrow, eyelashes, mustache designs" as parametric decals in Substance Painter ([HairTG Facial Hair demo](https://eyosido.gumroad.com/l/htg-surface-fh-demo), product page).
Olivier Lau's Substance Designer generator builds strands from up to 10 control points, and each strand is drawn with a bitmap or procedural pattern ([CG Channel](https://www.cgchannel.com/2018/03/check-out-this-neat-hair-generator-for-substance-designer/), press).

### Stubble from a density mask and tiling noise

Stubble needs two things. One is a mask that says where hair grows and how dense it is. The other is a pattern of dots that looks random at every scale.
Elden Ring shows that one stubble slider, separate from the beard style, is enough for players ([Elden Ring wiki](https://eldenring.wiki.gg/wiki/Character_Creator), fan wiki).
Reallusion's eyebrow replace layer uses a noise with its own colour, grain scale from 10 to 150, normal strength and roughness ([Reallusion manual](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_SkinGen/Makeup/Eyebrow/Settings-in-Appearance-Editor-Eyebrow.htm), official).
Heitz and Neyret's histogram-preserving blending tiles a small example texture without visible repetition. It runs over 20 times faster than an equal-quality procedural noise ([Heitz and Neyret, High-Performance By-Example Noise, HPG 2018](https://eheitzresearch.wordpress.com/722-2/)).
For short stubble, texture bombing with one dot per cell and a density threshold is simpler and gives the same result.
A character artist describes faking stubble with an intense pore layer multiplied by -1 ([80 Level](https://80.lv/articles/video-game-characters-sculpting-texturing-rigging), interview).

### Shells and fins for short hair

Lengyel, Praun, Finkelstein and Hoppe's "Real-Time Fur over Arbitrary Surfaces" draws fur as semi-transparent concentric shells of a texture, with fins along the silhouette ([Lengyel et al., 2001](https://www.hhoppe.com/fur.pdf)).
Kenwright's course notes describe the simple form. Noise dots are scattered on a transparent texture, and the same texture is drawn on each shell pushed out along the normal ([Kenwright, Fur and hair shells](https://www.xbdev.net/physics/workshops/Workshop_15_Fur_&_Hair_Shells.pdf)).
Epic's 2011 Samaritan demo listed "Short Hair / Beard" as a feature ([Epic and NVIDIA, GDC 2011](https://www.nvidia.com/content/pdf/gdc2011/gdc2011epicnvidiacomposite.pdf), official). The slides do not say how it was drawn.
A Frostbite course talk says that games "usually resort to hair cards or shells" ([Frostbite, SIGGRAPH 2019 hair course](https://www.advances.realtimerendering.com/s2019/hair_presentation_final.pdf), official).
Shells give a full beard a soft, raised edge in profile. They cost one extra draw of the face area per shell, and they would add volume that KOTOR's flat painted beards never had.

### Baking a groom into a texture

Ready at Dawn rendered groomed hair offline in several passes and composited them into "a library of sharable general use hair textures" for The Order: 1886 ([Phail-Liff and Andreason, Melton and Mustaches, SIGGRAPH 2015](https://history.siggraph.org/wp-content/uploads/2022/10/2015-Talks-Phail-Liff_Melton-and-Mustaches-The-Character-Art-and-Shot-Lighting-Pipelines-of-The-Order-1886.pdf), official talk).
MetaHuman bakes a groom onto the face texture for far LODs, as described above.
A Polycount user warns that a baked groom has the same limit as painted hair, because it depends on the head's texture resolution and UV layout ([Polycount](https://polycount.com/discussion/comment/2761198), forum post, read through a search summary because the page refused a direct fetch).
For kjk, baking is useful in the other direction. kjk can bake its procedural hair once into a per-character texture, so that the runtime cost is one texture read.

### Normal and shadow cues so that painted hair is not flat

Painted hair looks flat under a moving light because it has no height and no direction.
Three cues fix most of this.

- **A per-hair normal.** Cyberpunk 2077 ships a normal map with every eyebrow ([REDmodding wiki](https://wiki.redmodding.org/cyberpunk-2077-modding/for-mod-creators/references-lists-and-overviews/cheat-sheet-head), modding wiki). Reallusion has a normal strength per hair and a micro normal noise ([Reallusion manual](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_SkinGen/Makeup/Eyebrow/Settings-in-Appearance-Editor-Eyebrow.htm), official). With stamps, each hair can carry a small cylinder normal across its width.
- **A shadow under the hair.** Reallusion's shadow layer darkens the skin just under the brow ([same source](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_SkinGen/Makeup/Eyebrow/Settings-in-Appearance-Editor-Eyebrow.htm)). In a shader this is the same band, widened, blurred and moved down by a millimetre.
- **An anisotropic highlight.** Hair shines in a stretched line across its growth direction. Scheuermann's real-time hair shader uses Kajiya-Kay lighting with two specular highlights shifted along the hair tangent ([Scheuermann, Practical Real-Time Hair Rendering and Shading, SIGGRAPH 2004](https://history.siggraph.org/wp-content/uploads/2022/12/2004-Talks-Scheuermann_Practical-Real-Time-Hair-Rendering-and-Shading.pdf), official talk). The direction field from the stamps gives this tangent for free.

### Making geometry hair follow a changed face

kjk does not need geometry hair, but the methods are worth knowing if a modelled beard is ever added.
Skyrim ships a TRI morph file with each beard and brow ([Nexus Mods wiki](https://wiki.nexusmods.com/index.php/Deforming_a_Skyrim_SE_mustache:_TRIs_%26_NIFs), modding wiki).
Cyberpunk 2077 names each eyebrow mesh in a `.morphtarget` file ([REDmodding wiki](https://wiki.redmodding.org/cyberpunk-2077-modding/for-mod-creators-theory/core-mods-explained/archivexl/archivexl-character-creator-additions/ccxl-eyebrows/), modding wiki).
Dragon's Dogma 2 runs a wrap deformer once when the face is built and then removes it ([Autodesk](https://blogs.autodesk.com/media-and-entertainment/2025/04/10/ideas-and-ingenuity-behind-capcoms-in-house-re-engine-tools-for-dragons-dogma-2/), official interview).
FaceGen gives each accessory its own statistical shape model, driven by the same numbers as the head (see `docs/research/2026-11-16-character-creation-tech.md`).

## Research papers and open source work

| Work | Year | What it does | Code and licence | Use for kjk |
|---|---|---|---|---|
| [EMS, 3D Eyebrow Modeling from Single-view Images](https://arxiv.org/abs/2309.12787) (Li et al., SIGGRAPH Asia) | 2023 | Grows 3D eyebrow fibres from one photo. Three networks find the roots, predict the growth direction and decide where each fibre ends. Trained on 400 artist-made eyebrows. | [Code on GitHub](https://github.com/kevinlee09/EMS) under CC BY-NC 4.0, so non-commercial only. Needs Blender 3.6. | Its idea of roots, a direction field and lengths maps well onto stamps. The code cannot ship in MIT-licensed kjk. |
| [3D Reconstruction and Semantic Modeling of Eyelashes](https://diglib.eg.org/items/9401cf1d-b06a-4dad-af5e-49ea7e59aa20/full) (Kerbiriou, Avril, Marchal, Computer Graphics Forum) | 2024 | The first data-driven generative model of eyelashes from semantic features, with a dataset of 57 subjects. | The paper is CC BY-NC-ND 4.0. I found no code repository. | Its semantic features could name kjk's lash sliders. |
| [Intuitive, Interactive Beard and Hair Synthesis with Generative Models](https://vgl.ict.usc.edu/Research/HairSynthesis) (Olszewski et al., CVPR) | 2020 | Paints facial hair into a photo from a mask and coloured guide strokes, in under one second. | The page offers only the paper. | Not usable. It works on photos, not on texture space. |
| [Facial Hair Tracking for High Fidelity Performance Capture](https://studios.disneyresearch.com/2022/07/24/facial-hair-tracking-for-high-fidelity-performance-capture/) (Winberg et al., SIGGRAPH) | 2022 | Reconstructs and tracks 3D facial hair and skin from a multi-camera rig, from stubble to full beards. | No code. | Background only. |
| [Coupled 3D Reconstruction of Sparse Facial Hair and Skin](https://la.disneyresearch.com/publication/coupled-3d-reconstruction-of-sparse-facial-hair-and-skin/) (Beeler et al., SIGGRAPH) | 2012 | The earlier multi-view facial hair method. | No code. | Background only, and older than the 2015 window. |
| [FFHQ-UV](https://openaccess.thecvf.com/content/CVPR2023/html/Bai_FFHQ-UV_Normalized_Facial_UV-Texture_Dataset_for_3D_Face_Reconstruction_CVPR_2023_paper.html) (Bai et al., CVPR) | 2023 | Builds evenly lit face textures in UV space from single photos, with eyebrows painted in. | [Code](https://github.com/csbhr/FFHQ-UV) under MIT. The textures come from [FFHQ](https://github.com/NVlabs/ffhq-dataset), whose photos allow only non-commercial use. | A source of real eyebrow and stubble shapes in UV space, to fit kjk's brow numbers against. Private test data only. |
| [Simulating realistic eyelash looks](https://history.siggraph.org/?p=139419) (Sacha et al., SIGGRAPH) | 2006 | Generates lashes in real time from a small set of meta-parameters, including mascara. | No code found. | Older than the window, but the same idea as kjk's four lash numbers. |
| [High-Performance By-Example Noise](https://eheitzresearch.wordpress.com/722-2/) (Heitz and Neyret, HPG) | 2018 | Tiles a small example texture without visible repetition. | The page offers the paper, slides and video. | A way to tile a stubble dot texture. |

No paper that I found generates painted eyebrow, stubble or beard textures from a few parameters for a game.
The neural work reconstructs strands from photos or edits photos.
The parametric work in production is in tools, namely Reallusion's eyebrow layers and the HairTG decals.

## Style fit with KOTOR's heads

### What KOTOR itself does

KOTOR's face mesh has about 250 face vertices. Its eyebrows, stubble, beards, the lip line and most of the shading are painted into the texture ([2026-11-18 KOTOR head meshes](2026-11-18-kotor-head-meshes.md), summary).
Only two heads in both games have a modelled beard, Jorran and Batono (same note, the section on heads furthest from the average).
Most KOTOR 1 heads use a 256x256 texture and most KOTOR 2 heads a 512x512 one ([2026-11-18 KOTOR head mods](2026-11-18-kotor-head-mods.md), "High-resolution face textures").

### What KOTOR modders do

The KOTOR community adds facial hair as paint, not as meshes.
"Bearded PMHC01 Head Replacement" ships only `.tga` files, and has 14.3k downloads and 608 endorsements on Nexus ([2026-11-18 KOTOR head mods](2026-11-18-kotor-head-mods.md), row 42; [Deadly Stream page](https://deadlystream.com/files/file/485-bearded-pmhc01-head-replacement/)).
"Male Head 2k Re-skin - Mullet Man" adds a beard by hand-editing the texture ([Nexus Mods, KOTOR 1323](https://www.nexusmods.com/kotor/mods/1323)).
The Fens series of retextures gives Carth more stubble ([Nexus Mods, KOTOR 1192](https://www.nexusmods.com/kotor/mods/1192)).
The new high-poly head mods remodel the face, not the facial hair (same note, summary item 3).

### What remasters did

Mass Effect Legendary Edition updated "textures and hair models" ([Kotaku](https://kotaku.com/mass-effect-legendary-edition-makes-the-first-game-requ-1846899558), press). Mods for it suggest that Shepard's brows and short facial hair stayed textures ([Nexus Mods 7](https://www.nexusmods.com/masseffectlegendaryedition/mods/7), mod page).
Oblivion Remastered is a full rebuild in Unreal Engine 5 with "completely new models" and new beards and moustaches ([Push Square](https://www.pushsquare.com/guides/oblivion-remastered-everything-new-and-all-changes-compared-to-the-original-oblivion), press). It is a remake of the look, not a precedent for keeping paint.
Diablo II: Resurrected's team followed a "70/30 rule", where 70 percent of the art "has to hit nostalgia" and matches the original silhouettes, shapes and colours ([PCGamesN](https://www.pcgamesn.com/diablo-2-resurrected/fix-diablo-2), press).
I found no remaster that writes about mixing geometry facial hair with low-poly painted heads.

### What the sources say about mixing the two

A Polycount user notes that in Red Dead Redemption 2 the beard gets no hair shading, because "it's all lumped in with the skin" ([Polycount](https://polycount.com/discussion/comment/2761198), forum post, read through a search summary). So even a recent high-end game paints short beards into the skin.
BG3 shows the cost of geometry beards on a moving face. They clip through the face on Gale and on custom characters ([PC Gamer](https://www.pcgamer.com/games/rpg/theres-a-mod-to-fix-gales-beard-making-baldurs-gate-3-playable-at-last/), press).
MetaHuman shows the cost of switching between kinds of hair. Eyebrows that change from strands to cards between LODs can vanish up close ([Unreal forum](https://forums.unrealengine.com/t/eyebrows-disappear-close-up-to-metahumans/681472), forum post).
No source compares a card beard on a 250-vertex head with a painted one. My judgement is that a card or strand beard on a KOTOR head would look like a modern object stuck onto an old face, as the high-poly head mods do next to vanilla bodies.

## Recommendation for kjk

### The approach

Keep every kind of facial hair as paint drawn by the head's shader.
Each kind is a shape made from a few numbers, placed on the head's own landmarks, filled with hair stamps that follow a direction field, and lit with a small normal and a shadow.

- **Eyebrows.** Keep the seven numbers, the ends, the colour and the grooming settings that the prototype already has. Use the band only as a mask. Fill it with texture-bombed hair stamps that follow a direction field. The field points up near the inner end and along the curve toward the outer end, and one grooming number blends between the two. Add a soft shadow just under the band, as Reallusion does, and a per-hair normal.
- **Lashes and eyeliner.** Keep the four lash numbers along the lid edge. Add eyeliner as a separate layer on the same lid curve, with a width, a darkness, a colour and a wing length at the outer corner. This also fixes the known fault that eyeliner cannot be told from a strong lash line, because the two get separate settings.
- **Stubble.** Paint stubble from a density mask made of five regions. They are the upper lip, the chin, the jaw line, the cheeks and the neck. Each region has a density number, and one overall "stubble" slider scales them all, as Elden Ring does. The hair dots come from texture bombing with one dot per cell, kept only where a random number is below the density. A colour, a darkness and a tiny normal noise finish it.
- **Beards and moustaches.** Use the same regions as stubble, with a length setting. Long hair changes three things. The stamps become longer strokes that follow a downward direction field. The edges of the regions are drawn as curves with their own numbers, for example the cheek line height and the lower edge under the jaw. The colour darkens toward the middle of the beard. A full beard keeps the painted look of KOTOR's bearded heads.
- **Baking.** When the player confirms a character, bake the hair layers into a per-character texture at 1024x1024 or 2048x2048. At runtime the head then costs one texture read, and the shader math runs only in the character creator.

### Why this approach

- It matches the source. KOTOR paints every eyebrow, every lash line and every beard but two. KOTOR modders add beards and stubble by painting.
- It follows any face shape. The curves sit on the head's own vertices, as the brow ridge already does, so a slider that moves the brow or the jaw moves the hair with it.
- It avoids the stretching problem of plain textures. A painted texture stretches when a face slider stretches the UVs, as the Starfield eyebrow modder and the BG3 stubble modder describe. Stamps placed in millimetres on the surface keep their size.
- It needs no new geometry, no sorting of transparent cards and no hair LODs, so the faults seen in BG3 and MetaHuman do not appear.
- It fits "everything through sliders". Every feature is a number.

### Trade-offs

- A painted beard has no silhouette. In profile, a full beard is flat against the jaw. KOTOR's own painted beards have the same limit, so this matches the style, but a player who wants a thick beard will not get volume. If that is ever wanted, two or three shells over the beard region are the cheapest addition. They should be an option, not the default.
- Lashes have no silhouette either. KOTOR has no lash geometry, so this matches the style. A thin card strip along the lid edge could be added later for close-up dialogue shots.
- Stamps cost shader work. Each pixel in a hair region checks its own cell and its neighbours, typically 4 to 9 cells. Baking at confirmation removes this cost outside the character creator.
- Baking ties the hair to a texture resolution. At 1024x1024 over the whole head, one millimetre of face is roughly 3 to 4 texels, which is enough for brow hairs but not for single stubble dots. The stubble dots may need to stay procedural, or the face area may need its own higher-resolution region.
- Real photos are only partly usable. The photo fit can read an eyebrow's band and colour, as `brows.py` does now. It cannot read single hairs, so the direction field and the stamp shape stay hand-set defaults.
- The research code is not reusable. EMS is non-commercial, and no other paper ships parametric texture code. kjk has to write this itself, but the pieces (distance functions, texture bombing, Kajiya-Kay highlights) are all well documented.

### What to try first

1. Replace the solid eyebrow band by texture-bombed hair stamps with a direction field, and compare it against the hand-painted eyebrows of the HD KOTOR retextures.
2. Add the shadow band and the per-hair normal, and check the brows under a moving light in the renderer.
3. Build stubble as five region densities and one overall slider, and compare it against the stubble of Carth and the bearded player heads.
4. Only then decide whether full beards need shells.
