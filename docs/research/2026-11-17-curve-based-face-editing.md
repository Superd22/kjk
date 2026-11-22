# Curve-based face editing

This note asks one question. Would curves be the missing piece for copying a real person's likeness in kjk's character creator? In a **curve-based** editor, the player does not only move sliders. The player also edits a small number of control points on a line that follows one facial feature, such as the outline of the lips or the edge of the upper eyelid.

The user who asked for this note says plainly that they are not an artist. They want the hypothesis checked, not confirmed. This note therefore looks as hard for evidence against curves as for evidence in favour, and it reports both.

It was written on 2026-11-17 for kjk. Three earlier notes in this folder cover the neighbouring subjects, and this note does not repeat them.

- The technology note (`docs/research/2026-11-16-character-creation-tech.md`) explains morph targets, FaceGen, MetaHuman's DNA files and Dragon's Dogma 2's scanned heads.
- The sculpting note (`docs/research/2026-11-16-character-sculpting.md`) explains how a mouse drag becomes a parameter change. It covers Lewis and Anjyo's direct manipulation solve, local region models, and the patents on direct manipulation.
- The likeness note (`docs/research/2026-11-16-likeness-recreation.md`) explains what people use to recognise a face, and how many numbers a face needs.

Every claim links to its source. Where only a press article, a fan wiki, a vendor's own magazine or a tutorial was available, the text says so with the words "secondary source". Where a statement is my own reasoning rather than a source's, the text says "inference".

The research for this note ran into hard limits, and the reader should know them. The web search budget for the session ran out before every item was checked. Google Patents refused most requests after the first ten. Several papers were too large to open or sat behind a login. Section 10 lists everything that could not be verified.

## What has been learned since the earlier notes

The earlier notes were written before any prototype existed. Since then, the prototype has taught five things that shape this note.

1. The prototype head is based on the MakeHuman mesh. It has 194 sliders, and each slider is a morph target. The sliders and the direct drag ("clay") share the same parameters.
2. Fitting the whole face automatically from a photo failed. Landmark fits, a regressor and 3DDFA all produced faces that looked like nobody. Face recognition networks were not a usable judge of likeness.
3. Fitting one zone at a time worked. The user traced an outline on the photo, such as the eyebrow or the eye opening, and a script turned that outline into slider values. The user then judged the result by eye. When many outline sliders were stacked at large values, the face became grotesque.
4. A test against a 3D scan of Abraham Lincoln's life mask showed where the limit lies. With every vertex free to move, the head fits the scan to within 0.21 mm, so the mesh itself is not the limit. The average head lies 2.34 mm from the scan, and the best slider fit lies 1.7 mm from it. Tripling the slider range only reaches 1.5 mm. The sliders therefore lack shapes, not range. The slider fit recovered 64% of the nose, 48% of the brows, 30% of the eyes and 1% of the cheeks. It lost the cheek hollows, the brow ridge and the folds beside the nose.
5. Some decisions are already taken. There will be no preset-blending creator and no photo projection onto the texture. Eyebrows are painted into the texture, not modelled as geometry.

Point 4 matters most for this note. Any new input method, curves included, can only reach faces that the shapes underneath can make. Section 9 comes back to this.

## 1. Terms used in this note

A **control point** is a point that the player can grab and move. A **curve** in this note is a smooth line defined by a few control points. A **spline** is a curve built from several pieces joined smoothly. Two common kinds appear below. A **Bézier curve** passes through its end points and is pulled toward its middle control points without passing through them. A **Catmull-Rom spline** passes through every one of its control points.

An **outline** (also called a **contour**) is a curve that follows the edge of a feature, such as the line where the lips meet the skin.

An **occluding contour** (also called a **silhouette**) is the line where a surface turns away from the viewer. The jawline seen from the front is an occluding contour. Which points of the 3D surface lie on it depends on the viewing angle and on the shape itself.

A **profile curve** is the outline of the face seen exactly from the side. It runs down the middle of the face, over the forehead, nose, lips and chin.

A **landmark** is a named point on a face, such as the left corner of the mouth. A **landmark scheme** is a fixed list of landmarks that a dataset or a tracker uses.

A **prior** is a statistical model of which faces are likely. A system with a prior can fill in what the player did not specify by choosing the most likely face that matches the player's input.

A **wire deformer** is a tool that binds a curve to a mesh. When the curve moves, the nearby vertices move with it, and the effect fades with distance from the curve. Section 3.1 gives the details.

**Laplacian deformation** moves a few chosen vertices to target positions and then moves all the other vertices so that the surface keeps its local shape, such as its bumps and creases, as well as possible. It needs one sparse linear solve per edit.

A **morph target** (also called a **blend shape**) is a copy of the mesh with some vertices moved. A slider sets how far the mesh blends toward it. The sculpting note explains this in more detail.

## 2. Shipped character creators

### No shipped game exposes a true curve

I found no shipped game or avatar editor that lets the player edit a curve, a spline or an outline with connected control points on a facial feature. This holds for shape features such as the lips and the jaw. It also holds for painted features such as the eyebrows, the eyeliner and the lipstick. Every product that could be checked uses one of five other kinds of control.

- **Bone transforms.** A slider or a handle on the face moves, rotates or scales one face bone.
- **Morph targets.** A slider, a drag zone or a small 2D pad blends between sculpted shapes.
- **Preset picks.** The player chooses a shape or a texture from a list.
- **Decals.** The player moves, rotates and scales a ready-made image on the face.
- **Freehand painting.** The player paints on the texture with a brush. Only VRoid Studio and Tomodachi Life: Living the Dream offer this.

The only real control-point curves found in a shipped creator are VRoid Studio's hair guides. VRoid's help page says "You can also adjust the shape of the guidelines by moving the green dots around" ([VRoid help, hairstyles](https://vroid.pixiv.help/hc/en-us/articles/900005678786-How-do-I-customize-hairstyles), read through an archived copy). These curves shape hair only, not the face.

This is a negative result, and it has a limit. The search budget ran out before every game was covered in depth. Nioh 2, Street Fighter 6, Soulcalibur VI, Monster Hunter Wilds, WWE 2K, NBA 2K, Saints Row, Ready Player Me, inZOI and Once Human were only checked through search snippets. None of those snippets mentions a curve.

### What each creator uses underneath

The table summarises what each creator offers. The column "Underneath" says what the control really is, where a source says so.

| Product | What the player sees | Underneath | Numbers | Source |
|---|---|---|---|---|
| Justice Online, PC (逆水寒, NetEase) | Sliders grouped by facial part, plus style pickers | Bone transforms. Each parameter sets "the position, orientation and scale" of a facial component. | 208 continuous parameters. Eyebrow 24, eye 51, nose 30, mouth 42, face 61. | [Shi et al., ICCV 2019](https://arxiv.org/pdf/1909.01064) |
| Revelation (天谕, NetEase, mobile) | Sliders and a "coordinate control" that changes several values at once | Bone-driven, with "basically the same configuration" as Justice | 174 parameters | [Shi et al., AAAI 2020](https://arxiv.org/pdf/2008.07132); [official guide](https://tym.163.com/guide/mxbk/nl/20210108/34152_926160.html) |
| Moonlight Blade, PC (天涯明月刀, Tencent) | Sliders, plus clicking a face part and pressing Q, W or E to move, rotate or scale it | Bone transforms. Makeup is preset decals. | 48 bones carry more than 200 parameters. More than 80 makeup decals. | [Official news, 2015](https://wuxia.qq.com/webplat/info/news_version3/5012/5013/5014/6549/m3486/201504/334426.shtml) |
| Black Desert Online (Pearl Abyss) | The player clicks a highlighted face region and moves, rotates or resizes it. Eyebrows, eyeliner and lips are presets with sliders. | A transform per region. Korean press calls it a "bone controller" (본 컨트롤러). Pearl Abyss has not published a bone count. | About 20 or more face regions (secondary source). 15 eyebrow shapes in the 2014 beta (secondary source). | [Official notice](https://blackdesert.pearlabyss.com/Asia/en-US/News/Notice/Detail?_boardNo=9); secondary source [Inven](https://m.inven.co.kr/webzine/wznews.php?site=black&p=8&idx=123689) |
| Throne and Liberty (NCSOFT) | Presets plus sliders. The eyebrow has a shape preset, density, thickness, and inner and outer height, width and protrusion. | One shared face frame with continuous parameters. NCSOFT does not say whether they are bones or morphs. | Eyebrow 1 preset and 8 sliders. Lips 8, nose 11, eye 9, jaw 7, cheekbones 7. | [NCSOFT research blog](https://ncsoft.github.io/ncresearch/f00b54e34d364846ce3dc534cb7c7817d368dd5e) |
| AION 2 (NCSOFT, 2025) | Presets, then sliders | Bone-based, according to a developer statement reported by the press | Eyebrow 7 controls, eyeliner 3, lip makeup 6, 31 face presets | [Gametoc](https://www.gametoc.co.kr/news/articleView.html?idxno=103714) (secondary source) |
| MetaHuman Creator (Epic) | Markers on the face that the user drags | Neither bones nor a curve. The tool "finds the closest combination of real human facial features scanned into the MetaHuman database". | No total marker count. One example shows 6 markers around the eye. | [Epic, Head Sculpt tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-head-sculpt-tool-in-unreal-engine) |
| EVE Online (CCP) | The player clicks a zone and drags | A 2D field of triangles that interpolates morph weights | 92 face morph targets | [EVE dev blog](https://www.eveonline.com/news/view/character-creation-3) |
| Phantasy Star Online 2 (Sega) | A 4-corner blend pad. In detailed mode each region has 4 on-screen handles that move in linked pairs. | Morph blending. Sega calls it "モーフィングによる直感的な操作" (intuitive operation by morphing). | 4 handles per region | [Official PSO2 page](https://pso2.jp/players/about/concept/character/) |
| Second Life | Sliders | Mostly morphs | 55 face sliders. 48 are morph-only and 7 move bones. | [Second Life wiki](https://wiki.secondlife.com/wiki/Appearance_Editor_and_affeced_bones) |
| VRoid Studio (pixiv) | Face sliders. Eyebrows, eyeliner and lips are preset textures with position and size. A texture editor allows freehand painting. | Raster textures for painted features | 17 face categories | [VRoid help, faces](https://vroid.pixiv.help/hc/en-us/articles/4406164546969-How-to-edit-faces) |
| Nintendo Mii | A preset pick per part, then step buttons for size, thickness, rotation, spacing and height | 2D part sprites stored as small integers | Eyebrow rotation 12 steps, spacing 13, size 9, thickness 7 (3DS format) | [3dbrew](https://www.3dbrew.org/wiki/Mii) (secondary source) |
| Illusion games (AI Shoujo, Koikatsu) | Sliders. Eyebrows and paint are decals. | Each slider samples a keyframed pose of 89 bones with 25 keys. That internal curve is not shown to the player. | 59 face sliders in AI Shoujo, 52 in Koikatsu | [Decompiled code mirror](https://github.com/request-time-out/A-Scripts/blob/master/AI_Assembly-CSharp/AIChara/ChaFileFace.cs) (secondary source) |
| Elden Ring | 63 sliders and 17 eyebrow presets | Not verified | Eyeliner and lipstick have only strength and colour | [Fan wiki](https://eldenring.wiki.gg/wiki/Character_Creator) (secondary source) |
| Dragon's Dogma 2 | Sliders from −100 to 100 | Blend shapes and joints (see the technology note) | The brow has 7 sliders, one of them "bend". 14 eyebrow shapes. | [Game8](https://game8.co/games/Dragons-Dogma-2/archives/446290) (secondary source) |
| Cyberpunk 2077 | Preset indices only | Not verified | Eyebrows 1 to 11, eyes, nose, mouth, jaw and ears 1 to 22 | [Modding wiki](https://wiki.redmodding.org/cyberpunk-2077-modding/for-mod-creators-theory/references-lists-and-overviews/cheat-sheet-character-creator) (secondary source) |
| Lost Ark | 25 face presets plus sliders. The eyebrow has only angle and height. | Not verified | 25 presets | [Fextralife wiki](https://lostark.wiki.fextralife.com/Character+Creation) (secondary source) |
| APB Reloaded | A symbol designer that builds decals from preset shapes, with up to 100 layers | Decals | The guide says "any image can only be constructed of these basic shapes" | [Ten Ton Hammer](https://www.tentonhammer.com/guides/apb/symbol-designer) (secondary source) |

The sculpting note already describes The Sims 4, Fallout 4 and inZOI. None of them uses a curve. ArcheAge, Aion, Blade and Soul, Naraka Bladepoint, Where Winds Meet, Code Vein, Starfield and Baldur's Gate 3 were checked through secondary sources only. All of them use sliders, presets or decals.

### The designs that come closest to a curve

Three shipped designs place several control groups along one feature. None of them joins the groups into a curve, and each group is edited on its own.

- **Justice splits each feature into named parts.** The eyebrow has a head, a body and a tail, with 8 controllers each. The eye has a whole-eye group of 6 controllers, then the outer upper eyelid, the inner upper eyelid, the lower eyelid, the inner corner and the outer corner, with 9 controllers each. The mouth has a whole-mouth group of 3, then the middle of the upper lip (6), the outer upper lip (9), the middle of the lower lip (6), the outer lower lip (9) and the corner (9) ([Shi et al., ICCV 2019, Table 5](https://arxiv.org/pdf/1909.01064)). Each part is a bone with a position, a rotation and a scale. In effect, the eyebrow has three points along its length, and each point has three axes of movement and three of rotation.
- **Throne and Liberty treats the eyebrow as two anchors.** The inner end and the outer end each have a height, a width and a protrusion slider ([NCSOFT research blog](https://ncsoft.github.io/ncresearch/f00b54e34d364846ce3dc534cb7c7817d368dd5e)). Calling them anchors is my own reading, because they are separate sliders on screen.
- **PSO2 gives each region 4 linked handles** that drive morphs ([official PSO2 page](https://pso2.jp/players/about/concept/character/)).

A NetEase developer article explains how such per-part controls are built. Artists author two extreme bone poses for each region, and the game interpolates between them from the player's input. One region needed more than 10 bones ([NetEase Game Academy](https://api.academy.163.com/course/careerArticle?course=446)).

The lesson is that the largest Chinese and Korean creators already give the player several independent points along a feature, each with full movement. They stop short of joining those points into a smooth curve. I found no developer statement that explains why.

### Eyebrow, eyeliner and lip makeup editors

No product found gives the player control points for a painted feature. Painted features are handled in three ways.

- **A preset texture with a few sliders.** This is the most common design. Black Desert offers eyebrow shapes with density, height, shine and area sliders (secondary source, [Inven](https://www.inven.co.kr/webzine/news/?news=108740&site=black)). Justice offers 36 eyebrow styles and 19 lipstick styles for the female model ([Shi et al., ICCV 2019](https://arxiv.org/pdf/1909.01064)). Elden Ring's eyeliner has only strength and colour (secondary source, [fan wiki](https://eldenring.wiki.gg/wiki/Character_Creator)).
- **A preset decal with position, rotation and scale.** Mii eyebrows, Illusion's eyebrows and paint slots, Moonlight Blade's makeup decals and Lost Ark's patterns all work this way.
- **Freehand painting.** VRoid Studio lets the player paint on the face texture with a brush ([VRoid help](https://vroid.pixiv.help/hc/en-us/articles/4406164546969-How-to-edit-faces)). Nintendo says of Tomodachi Life: Living the Dream, "You can also draw face paint however you like" ([Nintendo, Ask the Developer](https://www.nintendo.com/us/whatsnew/ask-the-developer-vol-21-tomodachi-life-living-the-dream-part-2/)).

Where an eyebrow's shape changes in a shipped game, it changes through the face rig, not through the paint. Justice uses its 24 eyebrow controllers. Dragon's Dogma 2 uses one "bend" slider (secondary source, [Game8](https://game8.co/games/Dragons-Dogma-2/archives/446290)).

## 3. Film, DCC and rigging tools

Curves are common in professional tools, unlike in games. DCC stands for digital content creation, which means tools such as Maya, Blender and ZBrush. This section describes how each curve-driven tool turns a curve into a mesh change. Four patterns appear, and section 3.6 groups them.

### 3.1 Wires (Singh and Fiume, SIGGRAPH 1998)

Karan Singh and Eugene Fiume's "Wires" is the founding paper on curve-driven deformation ([Singh and Fiume, SIGGRAPH 1998, PDF](https://www.dgp.toronto.edu/%7ekaran/pdf/ksinghpaperwire.pdf)). A wire has five parts.

- The **wire curve** W is the curve the artist moves.
- The **reference curve** R is a copy of W made when the wire is bound to the mesh. It records where the curve started.
- The **radius** r sets how far from the curve the effect reaches.
- The **scale** s lets the wire push the surface away from the curve or pull it in.
- The **falloff** f sets how the effect fades with distance. The paper's implementation uses f(x) = (x² − 1)² for x between 0 and 1, and zero beyond.

Binding happens once. Each vertex P finds the closest point on R and stores that point's curve parameter. It also stores its influence F = f(distance ÷ r). Vertices farther than r from the curve get no influence.

Each time the wire moves, every bound vertex goes through three steps ([same paper](https://www.dgp.toronto.edu/%7ekaran/pdf/ksinghpaperwire.pdf)).

1. It is scaled away from or toward its closest point on R, by the scale s weighted by F.
2. It is rotated by the angle between the tangent of W and the tangent of R at its parameter, weighted by F.
3. It is moved by the difference W − R at its parameter, weighted by F.

A vertex that lies on R follows W exactly. A vertex at distance r or more does not move. When several wires overlap, their offsets are blended by a power mean. An exponent of 0 gives a plain average, and a large exponent picks the largest offset. A variant weights each wire by how close it is, which lets a wire that has not moved act as an anchor.

The paper adds **domain curves**. A domain curve limits a wire's effect to one side of a line. Figure 9 of the paper is titled "Using domain curves to animate a facial crease". In that figure, a domain curve under the eye limits the wire's effect to the cheek ([same paper](https://www.dgp.toronto.edu/%7ekaran/pdf/ksinghpaperwire.pdf)). This is directly relevant to the folds beside the nose, as section 6 explains.

Everything in Wires is three-dimensional. The curves are 3D space curves, and the tube of influence around them is 3D. A wire therefore does not have a depth problem of its own. The depth problem appears only when the player has to place the curve from a 2D view.

The cost at run time is small. The closest-point search is done once at binding, after which each vertex needs "a few vector operations" per change ([same paper](https://www.dgp.toronto.edu/%7ekaran/pdf/ksinghpaperwire.pdf)).

### 3.2 Maya's wire deformer and curve-based eyelid rigs

Maya's wire deformer is the paper's algorithm under Maya's names. The "Dropoff Distance" is the radius r and defaults to 1.0. "Scale" is s. "Crossing Effect" and "Local Influence" control how overlapping wires combine ([Autodesk, wire deformer node](https://download.autodesk.com/us/Maya/2010help/files/Deformer_Nodes_wire.htm)). The "base wire" is the reference curve. A "holder" is a curve that limits the deformed region, which is the paper's domain curve ([Autodesk, creating a wire deformer](https://help.autodesk.com/cloudhelp/2026/ENU/Maya-CharacterAnimation/files/GUID-6EE5B6BD-6947-44EB-BF6F-B3F47BA006B1.htm)). Autodesk's node documentation says the twist setting can give "subtle lip and eyebrow effects in character setup" ([Autodesk](https://download.autodesk.com/us/Maya/2010help/files/Deformer_Nodes_wire.htm)).

Professional riggers build eyelids from curves in a standard way. The open-source tool facify shows it in code (secondary source, [facify, wy_facify.py](https://github.com/wayneyip/facify)).

- Each vertex on the edge of the eyelid gets a joint at the centre of the eyeball, aimed at that vertex.
- A dense curve runs through all the eyelid edge vertices. A locator sits on the curve for each vertex, and each joint aims at its locator.
- A low-resolution curve with 5 control points drives the dense curve through a wire deformer. The tool uses 5 controls per lid.
- An optional blink blends the upper and lower low-resolution curves together.

Because the joints sit at the centre of the eyeball and rotate, the eyelid edge stays on the eyeball whatever the animator does with the curve. This is the cleanest answer I found to the depth problem for eyelids. The animator moves 5 points, and the eyeball's sphere supplies the depth.

### 3.3 Pixar's curvenets

Pixar has made curves the core of its recent rigs. Its 2022 paper "Character Articulation through Profile Curves" describes "detail-preserving deformations fully controlled by 3D curves that profile the deforming surface" ([SIGGRAPH archive, abstract](https://history.siggraph.org/?p=117913)). The full paper could not be opened, because its old Pixar address now redirects.

Pixar's talks describe how it is used.

- Artists draw a network of Bézier curves, called a **curvenet**, near the surface. A tool called the **Profile Mover** binds the curves to the mesh by cutting the mesh along them. It then runs a mesh optimisation "that reconstructs surface details conforming to the posed curvenets" ([Pixar, Elemental talk, 2023](https://research.pixar.com/docs/2023.SiggraphTalks.NTSHFG.pdf)).
- Curvenets were first used on Turning Red (2022) for the body and face of the panda characters. On Elemental, every fire, water and air character had a curvenet rig. The lips and eyes used "partially connected and floating curves" ([same talk](https://research.pixar.com/docs/2023.SiggraphTalks.NTSHFG.pdf)).
- The talk says curvenets need "orders of magnitude fewer authored points" than a conventional rig ([same talk](https://research.pixar.com/docs/2023.SiggraphTalks.NTSHFG.pdf)).
- On Inside Out 2, Anxiety's lips had controls with "nine sections on each lip" ([Pixar, Inside Out 2 talk, 2024](https://research.pixar.com/docs/2024.SiggraphTalks.HNSZ.pdf)).
- Pixar's 2026 face rigging talk for Hoppers explains how curvenets carry weights onto the face. The curves are centripetal Catmull-Rom splines, so that every point of a curve lies exactly on the surface. Only a few points on the curves are weighted by hand. The rest of the curve is filled in by a Laplacian solve along the curve, and the surface weights are then spread from the curves by a second Laplacian solve. The mouth uses four weight maps. One of them is a depth coordinate that is "zero at the face surface and one at the back of the throat" ([Pixar, Hoppers face talk, 2026](https://research.pixar.com/docs/2026.SiggraphTalks.TSG.pdf)).

Pixar's curvenets are the strongest evidence in favour of curves as a way to shape a face. They are also used by expert riggers and animators, not by players, and the curves are 3D curves placed in a 3D viewport.

### 3.4 Other rigging tools

- **Softimage control splines.** Artists draw curves over the cheeks, forehead, eyelids and mouth "where you want to simulate skin sliding". A "Divisions" setting decides how many controls each spline gets. Joints are placed along the curve, and the mesh is skinned to them ([Autodesk Softimage 2014 guide](https://download.autodesk.com/global/docs/softimage2014/en_us/userguide/files/char_rig_CreatingControlSplinesforFacialRigs.htm)).
- **Blender bendy bones.** Blender "treats the bone as a section of a Bézier curve passing through the bones' joints", divided into 1 to 32 rigid segments ([Blender manual, Bendy Bones](https://docs.blender.org/manual/en/latest/animation/armatures/bones/properties/bendy_bones.html)).
- **Rigify's new face.** Rigify's skin system builds the face from "chains of one or more B-Bones connecting control points" ([Blender manual, Rigify skin rigs](https://docs.blender.org/manual/en/dev/addons/rigify/rig_types/skin.html)). Its eye rig needs one top and one bottom eyelid chain, and "the chains are rigged to follow the surface of the eye" ([Blender manual, Rigify face rigs](https://docs.blender.org/manual/en/dev/addons/rigify/rig_types/face.html)). The older default face has 8 lip bones and 16 eyelid bones, 4 per lid ([Rigify source, human.py](https://raw.githubusercontent.com/blender/blender/main/scripts/addons_core/rigify/metarigs/human.py)).
- **Blender hooks.** A hook ties curve control points or vertices to an object, with a falloff radius ([Blender manual, Hook modifier](https://docs.blender.org/manual/en/latest/modeling/modifiers/deform/hooks.html)).
- **Houdini Wire Deform.** Houdini has its own wire deformer, with geometry "captured to a curve via the Wire Capture node" ([SideFX, Wire Deform](https://www.sidefx.com/docs/houdini/nodes/sop/wiredeform.html)).
- **ZBrush curves.** ZBrush's Curve mode applies a brush along a curve that the artist can bend, and it can snap the curve to the surface ([Maxon, ZBrush Curve](https://help.maxon.net/zbr/en-us/Content/html/reference-guide/stroke/curve/curve.html)). This is a path for a brush stroke. I found no ZBrush tool that binds a curve to a face and deforms it as a wire does.
- **Cages and lattices.** Free-form deformation (Sederberg and Parry, 1986), mean value coordinates, harmonic coordinates (Joshi et al., 2007, used at Pixar) and Green coordinates all deform a mesh by moving the points of an enclosing volume ([Sederberg and Parry, abstract](https://history.siggraph.org/?p=116034); [Joshi et al., slides](https://www.cs.jhu.edu/~misha/Fall07/Notes/Joshi07.pdf); [Lipman et al., abstract](https://history.siggraph.org/?p=112883)). They handle depth because the cage is 3D. They are volumes, not curves, so they are an alternative to curves rather than an example of them.

### 3.5 Photo-fitting tools

Three tools fit a head to photographs, and one of them uses curves.

- **Reallusion Headshot 3 has a curve tool for likeness.** Its product page lists a "Spline-Based Reshaper" that "Supports portrait and profile views for precise alignment of features such as double eyelids and nasolabial folds" ([Reallusion, Headshot](https://www.reallusion.com/character-creator/headshot/)). Reallusion's own magazine describes the workflow (secondary source, vendor case study). In the front view, the user adjusts Bézier curves for the eye, the double eyelid and the side of the face. In the profile view, the user matches the nose, lips, chin, forehead, ear and skull depth. A preview then "converts the vector corrections into a 3D result" ([Reallusion Magazine, August 2026](https://magazine.reallusion.com/2026/08/11/digital-double-from-photos-phibixs-photorealistic-workflow-with-cc5-and-headshot-3-1/)). The same product also has 58 facial presets, each with a slider ([Reallusion, Headshot](https://www.reallusion.com/character-creator/headshot/)). This is the closest commercial product to the user's hypothesis. It is a professional tool, not a game. It takes depth from a profile view, and it uses the curves as input to a fit, not as a free deformation.
- **KeenTools FaceBuilder uses pins, not curves.** Each pin ties a point on the 3D model to a spot on a photo. The tool "uses camera settings and pin positions to change the shape", and the shape comes from a statistical model ([KeenTools, FaceBuilder](https://keentools.io/facebuilder)). Its source code shows that each pin drag calls a solver, and that the user can set a "shape rigidity" from 0 (most flexible) to 10 (most rigid) ([KeenTools add-on source, settings.py](https://github.com/KeenTools/keentools-blender)). Depth comes from several photos taken from different angles.
- **FaceGen PhotoFit uses points.** The user places crosshairs on about a dozen points, as the likeness note describes ([FaceGen, Create from photo](https://facegen.com/man_create_photo.htm)).
- **Daz Face Transfer** shows no landmark or contour editing on its product page ([Daz 3D, Face Transfer](https://www.daz3d.com/face-transfer)).

Performance capture also uses curves as constraints. ILM's 2016 talk says "roto spline contours are created to follow the inner lip lines and eye lids". These contours, together with tracked dots, feed a solve for the rig's controls ([ILM, SIGGRAPH 2016 talk](https://history.siggraph.org/wp-content/uploads/2022/09/2016-Talks-Cantwell_ILM-Facial-Performance-Capture.pdf)). Depth comes from 2 or 4 cameras on a helmet. I could not open Epic's documentation on the contours that MetaHuman's identity solve uses.

### 3.6 The four ways a curve drives a mesh

The tools above use four patterns.

1. **Distance falloff around the curve.** This is Wires, Maya's wire deformer and Houdini's Wire Deform. Each vertex is bound once to its closest point on the curve and receives that point's movement, scaled by a falloff weight.
2. **Joints along the curve.** This is the Maya eyelid rig, Softimage control splines and Rigify. A few control points shape a curve, joints follow the curve, and the mesh is skinned to the joints.
3. **A surface optimisation with the curve as a constraint.** This is Pixar's Profile Mover. The mesh is solved so that it follows the curves while keeping its detail.
4. **A fit of a parametric model to the curve.** This is Headshot 3's reshaper and ILM's contours. The curve does not move vertices directly. It becomes a target that a model's parameters are solved to match.

Only the fourth pattern keeps the result inside a model of plausible faces. The first three can make any shape, including shapes no human has.

## 4. Research on curves and faces

### 4.1 Sketch-based face editing

Research has built many systems in which the user draws curves to make or change a 3D face. Every serious system pairs a few curves with a strong prior. The table lists the systems found, with their numbers.

| System | What the user draws | How the face follows | Data and numbers | Source |
|---|---|---|---|---|
| FiberMesh (Nealen et al., SIGGRAPH 2007) | Curves on the surface that stay as handles | Curve deformation, then the new curve positions become constraints in a surface optimisation | 10 to 15 frames per second on meshes of 600 to 2,000 vertices on a 1 GHz laptop | [Paper](https://www.cs.princeton.edu/courses/archive/fall07/cos597B/papers/nealen-fibermesh.pdf) |
| Face Poser (Lau et al., SCA 2007) | Point constraints and pairs of source and target strokes, from any view | A MAP solve in a 15- or 20-dimension PCA space, with a learned prior | About 0.4 seconds for an eyebrow-sized stroke. Meshes of about 17,000 vertices. Expressions, not identity. | [SCA 2007 paper](https://www.cs.cmu.edu/~mlau/projects/face_poser/3dface_sca07_final.pdf) |
| Gunnarsson and Maddock (TPCG 2007) | Exactly 5 strokes for the nose, in a fixed order, on a template head | Feature points on the strokes are projected into a PCA model of 138 scanned faces | No recognition rate. "A subtle deviation when tracing the outline can result in a failed recognition." | [Paper](https://staffwww.dcs.shef.ac.uk/people/S.Maddock/publications/GunnarssonMaddock2007_TPCG.pdf) |
| Gunnarsson and Maddock (TPCG 2010) | Front-view strokes, matched to 46 feature points | The depth of each point is replaced by the most likely value from a Gaussian mixture | 36 training poses. 14 of the 46 points can never be sketched. | [Paper](https://staffwww.dcs.shef.ac.uk/people/S.Maddock/publications/GunnarssonMaddock2010_TPCG.pdf) |
| DeepSketch2Face (Han et al., SIGGRAPH 2017) | A silhouette line, feature lines for the brows, eyes, mouth, ears and nose, and "wrinkle lines" for everything else | A network regresses 50 identity and 16 expression coefficients of a bilinear face model. Later edits use Laplacian deformation. | Mean error 2.04 mm on a face scaled to 200 mm, against 6.06 mm for a plain morphable model fit | [Paper](https://arxiv.org/html/1706.02042v1) |
| CaricatureShop (Han et al., 2018) | 2D feature lines on the silhouette, mouth, nose, eyes and eyebrows, plus a side view for depth | A network turns edited lines into per-vertex exaggeration factors | About 145 ms per edit | [Paper](https://ar5iv.labs.arxiv.org/html/1807.09064) |
| SketchMetaFace (Luo et al., TVCG 2024) | Front contour strokes, a side profile sketch, and ridge and valley strokes with an adjustable depth | Neural implicit surfaces. The profile sketch is a Laplacian handle for depth. | 4,528 training models. 16-person user study. | [Paper](https://arxiv.org/html/2307.00804v2) |
| SimpModeling (Luo et al., UIST 2021) | 3D curve handles, then 2D strokes on the mesh | An implicit field guides the deformation | 1,955 animal heads | [Project page](https://zhongjinluo.github.io/SimpModeling) |
| DragGAN (Pan et al., SIGGRAPH 2023) | Points on a 2D image | A generator's features are pulled toward the targets | 2D only. With 68 handle points, the mean landmark error fell from 16.02 to 4.73 pixels. | [Paper](https://ar5iv.labs.arxiv.org/html/2305.10973) |

Chang and Jenkins's "Sketching Articulation and Pose for Facial Animation" (SCA 2006) draws a reference curve and a target curve, then searches a rig's pose space for the best match. I could only read its abstract, through a later paper that describes it ([Gunnarsson and Maddock, 2010](https://staffwww.dcs.shef.ac.uk/people/S.Maddock/publications/GunnarssonMaddock2010_TPCG.pdf)).

Three findings from these systems matter for kjk.

- **Raw handle dragging is hard for amateurs.** In DeepSketch2Face's study, 12 graduate students copied a reference portrait. With a handle-only deformation tool, nobody finished within the 15-minute limit, and about 80% of their time went into refinement. With the learned sketch system, they finished in about 10 minutes and spent about 10% of their time on refinement. In a separate rating, 38 raters preferred the sketch system's results in 374 of 456 votes ([Han et al., 2017](https://arxiv.org/pdf/1706.02042)).
- **A strong prior trades reach for speed.** In SketchMetaFace's study, 16 amateurs copied a reference model with four tools, and 50 raters scored how faithful each result was on a scale of 1 to 10. DeepSketch2Face was the fastest tool, at 2 to 5 minutes per model, but scored only 1.96. SketchMetaFace took 5 to 9 minutes and scored 6.28. ZBrush took 10 to 18 minutes and scored 5.82 ([Luo et al., 2024](https://arxiv.org/html/2307.00804v2)). The most constrained system could not reach the targets. This is the same lesson as the Lincoln test.
- **Curves describe outlines, not depth.** DeepSketch2Face's follow-up edits keep the existing depth of every handle vertex unchanged ([Han et al., 2017](https://arxiv.org/html/1706.02042v1)). Its authors also say a morphable model "cannot create geometric details such as wrinkles at novel locations". To raise or sink a region, it adds separate gestures, "up-roll" and "down-roll", at 3 levels each.

### 4.2 Fitting a face model to contours

A curve that a player draws on a front view behaves like an image edge that a fitting program has to match. The fitting literature has studied this problem in detail.

Bas, Smith, Bolkart and Wuhrer state the core difficulty. "Edges, for example caused by occluding boundaries, do not have a fixed correspondence to model vertices" ([Bas et al., 2016](https://ar5iv.labs.arxiv.org/html/1602.01125)). The jawline seen from the front is made of whichever vertices happen to face sideways at that angle. When the head turns, or when the jaw gets wider, a different set of vertices forms the outline. Their method finds the contour vertices again at every step, matches each one to the nearest image edge, and refits.

The method lowered the error measurably. On 10 faces in 9 poses, with automatically found landmarks, fitting to landmarks alone left a mean error of 6.43 mm. Fitting to edges with a soft distance field left 3.99 mm. Fitting with their re-matched edges left 3.35 mm ([Bas et al., 2016](https://ar5iv.labs.arxiv.org/html/1602.01125)).

3DDFA handles the same problem by "landmark marching". It moves the jaw landmarks along the surface to the silhouette as the pose changes ([Zhu et al., 3DDFA](https://ar5iv.labs.arxiv.org/html/1511.07212)). Romdhani and Vetter's 2005 method uses a soft cost based on the distance to image edges ([described by Bas et al.](https://ar5iv.labs.arxiv.org/html/1602.01125)). I could not open Romdhani and Vetter's own paper.

This matters for a curve editor. A jawline curve that is bound to a fixed set of vertices behaves wrongly once the player turns the camera or widens the face. The curve then pulls on the wrong part of the surface (inference from Bas et al.).

### 4.3 How many points describe each feature

Landmark schemes give a practical answer to how many points describe a feature's outline. The table lists the three schemes that could be verified.

| Feature | iBUG 68 (300-W) | WFLW 98 | MediaPipe Face Mesh |
|---|---|---|---|
| Jaw and face contour | 17 | 33 | 36, in a ring that also crosses the forehead |
| Each eyebrow | 5, along the upper edge only | 9, as a closed outline | 10, in two rows of 5 |
| Nose | 9 (4 on the bridge, 5 along the base) | 9 | No separate set |
| Each eye | 6 | 8 | 16, in a closed ring |
| Outer lip outline | 12 | 12 | 20 |
| Line between the lips | 8 | 8 | 20 |

The iBUG and WFLW index ranges come from FreeEnricher's Table 10 ([FreeEnricher](https://arxiv.org/html/2212.09525v1)). The 300-W page confirms its 68 points and says its benchmark uses the 51 points without the jaw ([iBUG, 300-W](https://ibug.doc.ic.ac.uk/resources/300-W/)). The WFLW total of 98 is confirmed by its official code ([LAB README](https://raw.githubusercontent.com/wywu/LAB/master/README.md)). The MediaPipe counts were taken from the index lists in MediaPipe's own source ([face_mesh_connections.py](https://raw.githubusercontent.com/google-ai-edge/mediapipe/master/mediapipe/python/solutions/face_mesh_connections.py)). Helen uses 194 points in total ([RPI list of landmark databases](https://sites.ecse.rpi.edu/~cvrl/database/Facial_Landmark_Databases.html)), but its split by feature could not be verified.

These are tracking points, and trackers use more points than a shape needs. Models built to describe one feature use far fewer numbers.

- **Outer lip outline. 6 key points.** Eveno, Caplier and Coulon describe the outer lip with 6 key points. These are the 2 mouth corners, the 2 peaks of the Cupid's bow, the dip between the peaks, and the lowest point of the lower lip. Four cubic curves join them ([described in Stillittano and Caplier, VISAPP 2008](https://www.scitepress.org/Papers/2008/10774/10774.pdf)).
- **Line between the lips. 4 or 5 key points.** For a closed mouth, Stillittano and Caplier use 5 key points with two cubic curves. For an open mouth, they use 4 key points with four cubic curves. They write that "a parabola is not accurate enough to represent the inner lip contour" ([same paper](https://www.scitepress.org/Papers/2008/10774/10774.pdf)).
- **Eyelids. 3 numbers each.** Iris recognition usually models each eyelid as a parabola with 3 parameters, which are the position of its top and its curvature. A survey states that "Most of eyelid localization methods presented in the literature model eyelids as parabolic arcs" ([Adam et al., Radioengineering, 2008](https://www.radioeng.cz/fulltexts/2008/08_04b_082_085.pdf)). A parabola is symmetric, so it cannot describe an eyelid whose highest point sits toward the nose or toward the ear (inference).
- **Eye as a whole. About 9 numbers.** Yuille, Hallinan and Cohen's 1992 eye template is a circle for the iris between two parabolas for the lids. A secondary source counts 9 parameters ([cnblogs summary](https://cnblogs.com/luofeiju/p/11611425.html), secondary source). I could not open the original paper.
- **Eyebrow. 4 to 5 points.** The iBUG scheme uses 5 points along the upper edge. A Bézier eyebrow with 4 points was mentioned in a patent snippet that I could not open. I found no primary source from the cosmetic or plastic-surgery literature on the "ideal brow".
- **Nose profile. About 5 points.** Soft-tissue profile analysis uses midline landmarks. The nose runs from glabella and nasion through the tip (pronasale) and the columella to the base (subnasale). DTX Studio's list of 3D soft-tissue landmarks has 14 midline points from the glabella to the chin ([DTX Studio help](https://helpfiles.dtxstudio.com/Help/d3d046fb-52e8-4b33-9f51-d3fd98ca3ed4/2.1/EN/3D_cephalometric_soft_tissue_landmarks.htm)).
- **Whole face profile. 18 points, 20 numbers.** Davidenko built profile silhouettes from 18 key points joined by splines. A principal component analysis with 20 dimensions explained more than 99% of the variation in these profiles ([Davidenko, Journal of Vision, 2007](https://bpb-us-e1.wpmucdn.com/sites.ucsc.edu/dist/6/210/files/2015/06/Davidenko-2007-jov-7-4-6.pdf)).

Classic statistical shape models agree that a face's landmarks are highly redundant. Lanitis, Taylor and Cootes placed 152 points on 160 faces, and 16 shape parameters described every face in their training set ([Lanitis, Taylor and Cootes, BMVC 1994](https://www.bmva-archive.org.uk/bmvc/1994/bmvc-94-006.pdf)). I found no paper that builds a separate shape model for each feature and reports its number of modes.

### 4.4 Facial curves in 3D recognition

3D face recognition has used curves on the surface as its main description. Drira and colleagues slice the face with planes through the nose tip at equal angles, and compare the resulting radial curves. On the GavabDB database (61 people, 9 scans each), they recognised 100% of neutral faces and 94.54% of faces with an expression ([Drira et al., 2010](https://proc.3dbody.tech/papers/2010/10229_48drira.pdf)). Samir, Daoudi and Srivastava use curves of equal depth instead, and reached about 98.9% on 240 scans of 40 people when they combined shape with texture ([Samir et al., 2007](https://www.scitepress.org/PublishedPapers/2007/20506/pdf/index.html)).

These curves are dense and cut through the whole face, including the cheeks. They are taken from a 3D scan, not drawn by a person. They show that a set of curves can describe a face's identity in 3D. They do not show that a player can draw those curves.

Harmon's profile recognition work from 1977 to 1981 is the classic source on recognising people from profile curves ([citations at VisionBib](https://visionbib.com/bibliography/people893.html)). I could not open any of those papers.

## 5. Evidence on likeness

This section collects what the perception and forensic literature says about outlines and likeness. Section 5.5 sorts it into evidence for and against.

### 5.1 Outlines alone carry identity poorly

The likeness note already reports one key result. People recognised 47% of famous faces from line drawings that held only contours, against 90% from photographs, in a study by Davies, Ellis and Shepherd ([Sinha et al., 2006, result 3](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)). Sinha's review adds that drawings which do work, such as an artist's sketches, carry light and dark areas, not just edges.

Benson and Perrett concluded that accurate line drawings of even very familiar faces are poor at conveying identity ([Benson and Perrett, St Andrews research portal](https://research-portal.st-andrews.ac.uk/en/publications/visual-processing-of-facial-distinctiveness/)). Line drawings need exaggeration to work. In Rhodes, Brennan and Carey's study, drawings of 37 lines through 169 points were judged most like the person when they were caricatured ([Rhodes, Brennan and Carey, abstract](https://researchconnect.stonybrook.edu/en/publications/identification-and-ratings-of-caricatures-implications-for-mental/)). For photographs, the best likeness needed only about 4.4% exaggeration ([Benson and Perrett, 1991](https://research-portal.st-andrews.ac.uk/en/publications/perception-and-recognition-of-photographic-quality-facial-caricat/)). My reading is that line drawings need distortion to make up for the information they lack.

Bruce and colleagues reported in 1992 that adding dark areas ("mass") to line drawings substantially improves identification. I could not open that paper and have no numbers from it.

### 5.2 Outlines do carry some identity

A contour alone is not useless. Davidenko showed 17 people a profile silhouette and asked them to pick the same person's front photo out of 4. They were right 70.1% of the time, where chance is 25%. They were right 80.9% of the time for male faces and 59.3% for female faces ([Davidenko, 2007](https://bpb-us-e1.wpmucdn.com/sites.ucsc.edu/dist/6/210/files/2015/06/Davidenko-2007-jov-7-4-6.pdf)). This was a matching task with both pictures in view, not naming a person from memory.

Feature shape matters too, not only spacing. Rhodes reported in 1988 that the shapes of the eyes, eyebrows and mouth mattered about as much as their positions in how people judged 41 faces. Cabeza and Kato's 2000 paper is titled "Features are also important". I saw both only through search abstracts. The likeness note's ranking from Abudarham and Yovel puts lip thickness, eye shape and eyebrow thickness among the most noticed features ([Abudarham and Yovel, 2016](https://people.socsci.tau.ac.il/mu/galityovel/files/2024/11/Reverseengineetingthefacespace2016.pdf)). Those three features are all outline features.

### 5.3 Forensic composites

Police build faces of suspects from witness descriptions. They have tried feature kits, artists' sketches and whole-face systems for 50 years, and they have measured the results. This is the largest body of evidence on how construction methods affect likeness.

**The systems.** Feature systems such as E-FIT and PRO-fit let the operator pick each feature from a library, then resize and move it. Shape changes beyond that were made by hand in a paint program ([Frowd et al., 2005](https://knowledge.lancashire.ac.uk/625/1/Frowd%20et%20al.%20(2005).%20A%20forensically%20valid%20comparison%20of%20facial%20composite%20systems.pdf)). No composite system found offers control points on a feature's outline. A sketch artist draws outlines freehand, which is the closest forensic practice to curve editing. EvoFIT is a holistic system. The witness repeatedly picks the best of several whole faces, and the system breeds new faces from those picks.

**The direct comparison.** Frowd and colleagues had ten celebrity faces built with five methods, after a delay of 3 to 4 hours. The naming rates were 19.0% for E-FIT, 17.0% for PRO-fit, 9.2% for an experienced police sketch artist, 6.2% for Photofit, and 1.5% for the 2005 version of EvoFIT ([Frowd et al., 2005](https://knowledge.lancashire.ac.uk/625/1/Frowd%20et%20al.%20(2005).%20A%20forensically%20valid%20comparison%20of%20facial%20composite%20systems.pdf)). The sketch artist had 15 years of police experience. In a separate sorting test, the sketched features were if anything more accurate than the feature-system features. The authors explain the poor naming by missing shading. The sketches had detailed outlines of the face, hair, eyes, eyebrows and mouth, but little shading on the forehead, cheeks, chin and nose ([same paper](https://knowledge.lancashire.ac.uk/625/1/Frowd%20et%20al.%20(2005).%20A%20forensically%20valid%20comparison%20of%20facial%20composite%20systems.pdf)).

**The meta-analysis.** Frowd and colleagues later combined 23 studies with 1,069 participants. Sketches did somewhat better than feature systems overall, with an odds ratio of 1.6. Modern EvoFIT composites were "over four times more identifiable" than feature or sketch composites. Feature and sketch systems reach about 20% naming after a delay of a few hours, but only about 5% after a delay of 1 to 2 days. EvoFIT reached 25% after 2 days, 45% when the inner features were built first, and 74% when combined with a holistic interview and a caricature view ([Frowd et al., Journal of Forensic Practice, 2015](https://knowledge.lancashire.ac.uk/id/eprint/18190/1/18190%20Frowd%20et%20al.%20(in%20press).%20A%20decade%20of%20evolving%20composite%20techniques_%20regression-%20and%20meta-analysis%20(final).pdf)).

**A whole-face step helps feature tools.** Frowd's 2012 chapter reports that asking the witness for seven whole-face judgements before construction raised PRO-fit naming from 9% to 41%. It also reports that naming of E-FIT and PRO-fit composites halved when the operator's freehand paint work was removed. That paint work is shading, wrinkles and marks ([Frowd, 2012](https://knowledge.lancashire.ac.uk/id/eprint/38381/1/38381%20Frowd%20(2012).%20Facial%20recall%20and%20computer%20composites.pdf)).

**The target in view changes everything.** Frowd's 2005 paper cites Davies and colleagues, who found that E-FIT composites built with the target face in view were named 49% of the time, against 17% from memory ([Frowd et al., 2005](https://knowledge.lancashire.ac.uk/625/1/Frowd%20et%20al.%20(2005).%20A%20forensically%20valid%20comparison%20of%20facial%20composite%20systems.pdf)). A kjk player copying a celebrity works with photos in view. The player is therefore in the easy case, where feature-by-feature construction does reasonably well. Most of the forensic advantage of holistic systems comes from the fact that people cannot recall single features from memory.

**People do not store faces as parts.** Tanaka and Farah found that a single feature was recognised 65% of the time on its own, against 77% inside the whole face. Houses showed no such effect, at 81% against 79% (reported in [Behrmann et al., 2015](https://newiipdm.haifa.ac.il/wp-content/uploads/2015/05/Behrmann-Richler-Avidan-Kimchi_2015_face_perception_chapterOHB.pdf)). A player who adjusts one feature's outline in isolation can therefore misjudge it. The prototype's experience fits this. Each zone fit looked right on its own, and the stack of them looked grotesque.

### 5.4 Studies that compare input methods

I found no user study that compares curve, sketch or point input against sliders for reaching a target face's identity. The nearest studies are these.

- **DeepSketch2Face and SketchMetaFace** compared sketch tools with each other, with a handle-only tool and with ZBrush (section 4.1). Neither study had a slider condition, and neither measured whether viewers recognised the person.
- **Face Poser** compared its stroke interface with Poser 7, a commercial tool for posing expressions. The comparison concerned expressions, not identity, and I could not open the paper that reports it ([TOG abstract](https://eprints.lancs.ac.uk/id/eprint/49897/)).
- **GANSlider** asked 138 people to reconstruct target images with sliders over a StyleGAN2 model. More slider dimensions made the task harder and needed more actions ([GANSlider, arXiv 2202.00965](https://arxiv.org/abs/2202.00965)). This is a warning for any interface that adds many control points.
- **Lewis and Anjyo** gave a mathematical argument, not a user study. Their project page says that a simple mathematical argument shows direct manipulation to be sometimes less efficient than editing the parameters. It adds that "the converse is equally true in other cases" ([Lewis and Anjyo, project page](https://scribblethink.org/Work/DirectManipBlendshapes/directmanip.html)). The sculpting note calls this a "study". That word overstates it, and this note corrects it.

### 5.5 Evidence for and against

The evidence for the hypothesis is this.

- Lip thickness, eye shape and eyebrow thickness are among the features people notice most, and all three are outlines ([Abudarham and Yovel, 2016](https://people.socsci.tau.ac.il/mu/galityovel/files/2024/11/Reverseengineetingthefacespace2016.pdf)).
- A profile outline alone lets people match a person at 70.1% where chance is 25% ([Davidenko, 2007](https://bpb-us-e1.wpmucdn.com/sites.ucsc.edu/dist/6/210/files/2015/06/Davidenko-2007-jov-7-4-6.pdf)).
- Freehand outline drawing beat fixed feature kits in the meta-analysis, with an odds ratio of 1.6 ([Frowd et al., 2015](https://knowledge.lancashire.ac.uk/id/eprint/18190/1/18190%20Frowd%20et%20al.%20(in%20press).%20A%20decade%20of%20evolving%20composite%20techniques_%20regression-%20and%20meta-analysis%20(final).pdf)).
- The prototype's own outline fits per zone worked where whole-face fits failed.
- Professionals use curves to shape faces. Pixar's curvenets and Reallusion's Spline-Based Reshaper are two examples, and the Reallusion tool targets likeness from photos directly.

The evidence against the hypothesis is this.

- Outlines without shading are recognised far less well than photos, at 47% against 90% ([Sinha et al., 2006](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)).
- In the most-cited direct comparison, the outline-based sketch was named less often than the feature kits, and the authors blamed missing shading ([Frowd et al., 2005](https://knowledge.lancashire.ac.uk/625/1/Frowd%20et%20al.%20(2005).%20A%20forensically%20valid%20comparison%20of%20facial%20composite%20systems.pdf)).
- Whole-face construction beats part-by-part construction by about four times when working from memory ([Frowd et al., 2015](https://knowledge.lancashire.ac.uk/id/eprint/18190/1/18190%20Frowd%20et%20al.%20(in%20press).%20A%20decade%20of%20evolving%20composite%20techniques_%20regression-%20and%20meta-analysis%20(final).pdf)).
- A curve drawn on a front view says nothing about depth, and the features the prototype lacks most are depth features (section 6).
- The Lincoln test shows that the slider basis, not the input method, is the ceiling. A curve that is solved into the same 194 sliders cannot pass the 1.7 mm limit.
- No study shows that curves beat sliders for reaching a target identity.

## 6. The depth problem

### 6.1 A front-view curve fixes only two of three coordinates

A curve drawn on a front view fixes where each point appears on the screen. It says nothing about how far forward or back that point sits. Gunnarsson and Maddock put it plainly: "The strokes are sketched in 2D which means the values along the depth axis … are unknown" ([Gunnarsson and Maddock, 2010](https://staffwww.dcs.shef.ac.uk/people/S.Maddock/publications/GunnarssonMaddock2010_TPCG.pdf)).

The systems found handle this in six ways.

1. **The curve lies on the current surface.** The curve's points are found by casting rays onto the mesh, and drags move them parallel to the screen. FiberMesh and Gunnarsson and Maddock's 2007 system work this way ([FiberMesh](https://www.cs.princeton.edu/courses/archive/fall07/cos597B/papers/nealen-fibermesh.pdf); [Gunnarsson and Maddock, 2007](https://staffwww.dcs.shef.ac.uk/people/S.Maddock/publications/GunnarssonMaddock2007_TPCG.pdf)).
2. **The depth is left unchanged.** DeepSketch2Face's follow-up edits keep each handle vertex's existing depth ([Han et al., 2017](https://arxiv.org/html/1706.02042v1)).
3. **The prior supplies the depth.** Face Poser solves for the most likely face that meets the screen constraints ([Lau et al., 2007](https://www.cs.cmu.edu/~mlau/projects/face_poser/3dface_sca07_final.pdf)). Gunnarsson and Maddock replace each point's depth with the most likely value given its screen position, which mostly corrected the mouth corners in their example ([Gunnarsson and Maddock, 2010](https://staffwww.dcs.shef.ac.uk/people/S.Maddock/publications/GunnarssonMaddock2010_TPCG.pdf)).
4. **A second curve in profile supplies the depth.** SketchMetaFace uses a side-view profile sketch as a Laplacian handle ([Luo et al., 2024](https://arxiv.org/html/2307.00804v2)). CaricatureShop has a side-view mode with its own network ([Han et al., 2018](https://ar5iv.labs.arxiv.org/html/1807.09064)). Reallusion's Headshot 3 uses a profile view for the nose, lips, chin, forehead and skull (secondary source, [Reallusion Magazine](https://magazine.reallusion.com/2026/08/11/digital-double-from-photos-phibixs-photorealistic-workflow-with-cc5-and-headshot-3-1/)).
5. **The curve sits on a known 3D shape.** Eyelid rigs rotate joints about the centre of the eyeball, so the eyelid edge always lies on the eyeball's sphere ([facify](https://github.com/wayneyip/facify), secondary source).
6. **A separate depth control.** DeepSketch2Face adds "up-roll" and "down-roll" gestures at 3 levels each. SketchMetaFace's ridge and valley strokes each carry an adjustable depth ([Han et al., 2017](https://arxiv.org/html/1706.02042v1); [Luo et al., 2024](https://arxiv.org/html/2307.00804v2)).

### 6.2 Silhouette curves slide on the surface

The jawline and the outer edge of the cheek, seen from the front, are occluding contours. Section 4.2 explains why they cannot be tied to fixed vertices. A jawline curve in a front view is therefore unsafe as a direct deformer. It is safer to describe the jaw by curves that lie on the surface, such as the lower edge of the jawbone seen in a three-quarter view. It is also safer to use the front outline only as a fitting target that is re-matched at every step, as Bas and colleagues do (inference from [Bas et al., 2016](https://ar5iv.labs.arxiv.org/html/1602.01125)).

### 6.3 Features with no outline from the front

The Lincoln test showed that the sliders fail worst on the cheek hollows, the brow ridge and the folds beside the nose. None of these has a clear outline in a front view. Each needs its own answer.

**The brow ridge** has an outline in profile. The profile line from the forehead through glabella to the nasion (the dip at the top of the nose) shows how far the brow ridge projects. A profile curve of about 3 points can set that projection at the midline. The ridge also runs sideways above each eye, and a single profile curve says nothing about its width or how it fades toward the temples (inference). A curve can therefore set the height of the ridge, but the shape of the ridge across the brow needs a region shape, such as a new morph target or a sculpt region.

**The fold beside the nose** (the nasolabial fold) is a crease, which is a line on the surface. Its path can be drawn as a curve from the side of the nose to beyond the mouth corner. Reallusion lists "nasolabial folds" among the features its spline reshaper aligns ([Reallusion, Headshot](https://www.reallusion.com/character-creator/headshot/)). The Wires paper animates a facial crease with a wire and a domain curve that limits the effect to one side ([Singh and Fiume, 1998](https://www.dgp.toronto.edu/%7ekaran/pdf/ksinghpaperwire.pdf)). The path alone is not enough. The crease also needs a depth and a width, and the cheek above the fold needs to bulge over it. SketchMetaFace's valley strokes with an adjustable depth are the nearest research equivalent ([Luo et al., 2024](https://arxiv.org/html/2307.00804v2)). A fold is therefore a curve plus at least two numbers. In my reading it suits a curve better than any other depth feature.

**The cheek hollow** is an area, not a line. It shows up only as shading under light, or as a slight dip in the outline when seen from a three-quarter view. No sketch system found has a stroke type for it. DeepSketch2Face handles bulges and dips with region gestures, not curves ([Han et al., 2017](https://arxiv.org/html/1706.02042v1)). The suggestive contours of DeCarlo and colleagues show that such shapes can be drawn as lines in theory ([Princeton, suggestive contours](https://gfx.cs.princeton.edu/proj/sugcon/)). I found no face modeller that uses suggestive contours as input. A curve does not suit the cheek hollow. A region control does, such as a morph target, a soft sculpt region, or a local statistical model fitted from scans.

The forensic findings point the same way. Frowd's sketch artist drew good outlines and still lost, because the forehead, cheeks, chin and nose lacked shading ([Frowd et al., 2005](https://knowledge.lancashire.ac.uk/625/1/Frowd%20et%20al.%20(2005).%20A%20forensically%20valid%20comparison%20of%20facial%20composite%20systems.pdf)). In kjk, photo projection is ruled out. The cheek hollows and the brow ridge must therefore come from the mesh, under the game's lighting. That makes them a geometry problem that outlines cannot solve (inference).

## 7. What a game must handle

### 7.1 Keeping the face plausible

A curve can describe any shape, including shapes no person has. Every system that ships to non-experts keeps a curve inside a model of real faces.

- MetaHuman projects every sculpt onto "the closest combination of real human facial features" in its scan database ([Epic, Head Sculpt tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-head-sculpt-tool-in-unreal-engine)).
- FaceBuilder lets the user set how rigid its statistical model is, from 0 to 10 ([KeenTools source](https://github.com/KeenTools/keentools-blender)).
- Face Poser shows what happens without enough freedom. Constraints that no face in the prior can meet "fight" each other and are not all satisfied ([Lau et al., 2007](https://www.cs.cmu.edu/~mlau/projects/face_poser/3dface_sca07_final.pdf)).

The prototype's grotesque stacked face is the same failure seen from the other side. Each zone fit was plausible alone, and their sum was not. A curve editor needs the same guards that the sculpting note lists for drags. These are limits per control, limits that depend on other controls, and a prior that pulls the face back toward real faces.

### 7.2 Symmetry

Symmetry is easy for curves. A curve on the left side mirrors to the right side by negating the sideways coordinate of each control point. Curves on the midline, such as the profile and the centre of the lips, must keep their middle point on the midline. MetaHuman makes edits symmetric by default ([Epic, Head Sculpt tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-head-sculpt-tool-in-unreal-engine)). The lip outline needs special care. Its two Cupid's bow peaks mirror each other, and its dip and lowest point stay on the midline.

### 7.3 Facial animation and lip sync

A curve edit changes the rest shape of the face. The expressions then have to work on top of it. There are two cases, and the sculpting note already covers both in general.

- **If the curve is solved into slider values**, nothing changes for animation. The result is a slider vector like any other.
- **If the curve deforms the mesh directly**, the deformation must be baked into the rest shape before the expression shapes are added. Expression morph targets authored on the average face will then be slightly wrong on a strongly edited face. The lips are the worst case, because a reshaped lip outline changes where every mouth shape should put the lip edge. Deformation transfer can rebuild the expression shapes for each edited face ([Sumner and Popović, 2004](https://people.csail.mit.edu/sumner/research/deftransfer/)). That is a heavy step for a game. Small limits on the lip curve are the cheaper answer (inference).

Pixar's rigs avoid this problem by making the curves the animation controls as well ([Pixar, Elemental talk](https://research.pixar.com/docs/2023.SiggraphTalks.NTSHFG.pdf)). That needs a full curve-based rig, which is far beyond a character creator.

### 7.4 Storage and sharing

A curve stores very little. Each control point can be stored as an offset from the default curve, in one byte per axis. Section 9 counts the bytes. Like the sculpting note's sliders, the curves should be stored as parameters, never as a mesh. If a curve drives a free deformation, the deformation rules must carry a version number. A later change to the falloff would otherwise change every saved face.

### 7.5 Controller input

A controller has no mouse. A curve editor on a gamepad needs a way to pick a point and move it. One workable scheme uses the shoulder buttons to pick a curve, the D-pad to step from one control point to the next, and the left stick to move the point (inference). Sliders work on a controller without any of this. A curve editor should therefore keep sliders for every feature, as the sculpting note recommends for drags.

## 8. Patents

This section describes patents as a caution, not as legal advice. A patent lawyer should check any design that kjk ships. Google Patents refused most requests during this research. Only six patent pages could be opened. The other patents below are known from search snippets only, and their claims and status were not verified.

**Patents whose pages were opened.**

| Patent | Owner | Status on Google Patents | What it covers |
|---|---|---|---|
| [CN 120789661 A](https://patents.google.com/patent/CN120789661A/en) | NetEase (Hangzhou) | Pending. Priority date 18 July 2025. | Control points shown on a 3D face, one per facial region. Selecting a point shows a mesh overlay of the region, and dragging the point changes the region by direction and distance. It claims drag handles per region, not curves along a feature. |
| [CN 110111418 B](https://patents.google.com/patent/CN110111418B/en) | SenseTime | Active. Expected to expire 15 May 2039. | Key points detected on a face photo become features, matched against a database to find bone parameters for a 3D face. The description says the features can be curves fitted through the eyelid and lip key points. Family members include US 11,836,943 B2. |
| [CN 118552703 A](https://patents.google.com/patent/CN118552703A/en) | Perfect World | Pending | A network turns a photo into head-model coefficients. No user-edited curves. |
| [CN 111991808 A](https://patents.google.com/patent/CN111991808A/en) | Perfect World | Listed as pending, with a rejection recorded in 2025 | A model turns a photo into 46 bone and texture values. No curves. |
| [CN 117422800 A](https://patents.google.com/patent/CN117422800A/en) | NetEase | Pending | Text descriptions turned into face parameters. No curves. |
| [US 10,628,984 B2](https://patents.google.com/patent/US10628984B2/en) | Tencent | Active. Expected to expire 9 March 2037. | Picking a face region with a colour-coded mask, then editing it. The claims could not be read. |

**Patents known only from search snippets.**

- **Pixar, "Character articulation through profile curves"** (US 2024/0203056 A1, and probably the grant US 12,450,835). It covers curvenets that cut the mesh and spread the curves' deformation over the surface. This is the main studio patent on curve-driven faces. Its claims and grant were not verified.
- **Karan Singh, "Method and apparatus for geometric model deformation using wires"** (US 6,204,860 B1, filed 1998). This is the wire deformer patent. Its filing date suggests it has expired, but I did not verify its status.
- **"Method for customizing avatars and heightening online safety"** (US 8,555,164 B2, filed 2004). Its description lets the user customise an avatar's face by moving the control points of curves tied to virtual muscles. It has probably expired. I did not verify its status or owner.
- **Snap, "Automated avatar generation"** (US 10,339,365 B2). It describes the eyebrow's edges with polynomial curves fitted from a photo, then picks a matching template. The user does not edit the curve.
- **NetEase, "Method and device for editing face model in game"** (CN 110624244 B). Its content is unknown.

I found no patent that claims a chain of connected control points on a feature's outline inside a game character creator. I also found no patent on editing a painted eyebrow, eyeliner or lipstick shape through control points in a game. These were keyword searches in English, Chinese and Korean, and only the first page of results was reviewed. They did not cover the Korean (KIPRIS) or Japanese (J-PlatPat) databases.

Two cautions follow for kjk.

- SenseTime's family covers fitting game face parameters from photo key points, including curves through the eyelid and lip points. It is active until about 2039. The prototype's outline-to-slider fitting tool does something similar. If kjk ever ships a photo-to-slider feature, that family needs a lawyer's reading.
- NetEase's pending application covers drag handles per facial region with a live overlay. A curve editor with connected points has a different structure, but whether that is enough is a legal question.

## 9. Summary of the findings

- **No shipped game exposes a true curve on a facial feature.** The deepest creators give several independent bone controls along a feature instead. Justice gives the eyebrow 3 parts with 8 controllers each, and each eyelid part 9 controllers ([Shi et al., 2019](https://arxiv.org/pdf/1909.01064)). No game gives control points for painted eyebrows, eyeliner or lipstick.
- **Professional tools use curves widely.** Wires, Maya, Houdini, Softimage, Blender and Pixar all drive faces with curves. Pixar's curvenets shape whole characters with "orders of magnitude fewer authored points" than conventional rigs ([Pixar, 2023](https://research.pixar.com/docs/2023.SiggraphTalks.NTSHFG.pdf)).
- **One commercial tool uses curves for likeness.** Reallusion's Headshot 3 aligns Bézier curves on front and profile photos, including double eyelids and nasolabial folds, and fits a head to them ([Reallusion](https://www.reallusion.com/character-creator/headshot/)).
- **Research systems always pair curves with a prior.** Raw handle dragging defeated amateurs in DeepSketch2Face's study. The most constrained system in SketchMetaFace's study was the fastest and the least faithful.
- **Few points describe an outline.** The outer lip needs about 6 key points and the line between the lips 4 or 5. An eyelid needs a 3-number parabola at minimum. A brow needs 4 or 5 points, and the nose profile about 5.
- **Front-view curves do not set depth.** Systems take depth from the surface, from a prior, from a profile view, from a known shape such as the eyeball, or from a separate depth control.
- **Silhouette curves slide.** The jawline in a front view is not a fixed set of vertices, and fitting methods must re-match it at every step ([Bas et al., 2016](https://ar5iv.labs.arxiv.org/html/1602.01125)).
- **Outlines alone carry identity poorly.** Line drawings were recognised at 47% against 90% for photos. An expert sketch artist's composites lost to feature kits because they lacked shading ([Frowd et al., 2005](https://knowledge.lancashire.ac.uk/625/1/Frowd%20et%20al.%20(2005).%20A%20forensically%20valid%20comparison%20of%20facial%20composite%20systems.pdf)).
- **Whole-face methods beat part-by-part methods** by about four times when working from memory. With the target in view, feature methods do much better ([Frowd et al., 2015](https://knowledge.lancashire.ac.uk/id/eprint/18190/1/18190%20Frowd%20et%20al.%20(in%20press).%20A%20decade%20of%20evolving%20composite%20techniques_%20regression-%20and%20meta-analysis%20(final).pdf)).
- **No study compares curves with sliders** for reaching a target identity.

## 10. What could not be verified

- Black Desert's number of points per feature, and whether its regions are bones. Only Korean press calls them a "bone controller".
- The underlying technique (bones or morphs) for Elden Ring, Lost Ark, ArcheAge, Code Vein and Where Winds Meet.
- inZOI, Nioh 2, Street Fighter 6, Soulcalibur VI, Monster Hunter Wilds, WWE 2K, NBA 2K, Saints Row, Ready Player Me and Once Human. These were checked only through search snippets, and none mentions a curve.
- MetaHuman's total number of sculpt markers, and the contours used by MetaHuman's identity solve. Epic's documentation pages did not load.
- The full text of Pixar's 2022 Profile Mover paper, and its patent's claims and grant.
- Reallusion Headshot 3's spline reshaper beyond the product page and the vendor's magazine. I found no technical description of how the curves become a 3D head.
- Romdhani and Vetter's 2005 paper, Chang and Jenkins's 2006 paper, the Face Poser user study against Poser 7, and the details of SketchFaceNeRF.
- Bruce and colleagues' 1992 study on "mass" in line drawings. I have no numbers from it.
- Rhodes 1988, Cabeza and Kato 2000, and Wells and Hasel 2007. I saw abstracts only.
- The point counts of Helen's 194 landmarks by feature, and of JD-landmark's 106.
- A primary cosmetic or surgical source on eyebrow shape, and the eyebrow patent with a 4-point Bézier curve.
- Harmon's profile recognition papers from 1977 to 1981.
- Every patent outside the six opened ones, including the status of the Wires patent.

## 11. What this means for kjk

### The verdict

The hypothesis is partly right and mostly misplaced.

It is right about the eyelids, the lips and the eyebrow. These features are outlines, people notice them strongly, and a few points describe each one. The prototype's own success with outline fits per zone confirms this. A curve is a more direct way for the player to say "the lid is higher here" than a set of sliders.

It is misplaced as "the missing piece". The Lincoln test measured where the prototype loses the face. The losses are the cheeks at 1% recovered, the brow ridge and the folds beside the nose. None of these has an outline in the front view, which is where a player would draw. The perception and forensic studies agree that what outlines lack is shading, which on kjk's heads must come from geometry. An expert artist's outlines lost to feature kits for exactly this reason.

The second reason is more basic. A curve is an input method. What the face can become depends on the shapes underneath. If a curve is solved into the existing 194 sliders, it cannot pass the 1.7 mm ceiling that the Lincoln test found, however well the player draws. A curve only adds new faces if it drives a new deformation, and then it loses the plausibility that the sliders provide.

So the missing piece is mostly missing shapes, not missing curves. Curves are a good input for three or four features, on top of better shapes.

### Feature by feature

| Feature | Does a curve suit it? | Which view | Suggested control points | Notes |
|---|---|---|---|---|
| Upper eyelid | Yes | Front | 5, including both corners | Keep the edge on the eyeball by rotating about the eye's centre, as eyelid rigs do. A plain parabola is symmetric, so it cannot place the lid's highest point toward the nose or the ear (inference). 5 points are worth the extra cost. |
| Lower eyelid | Yes | Front | 4, including both corners | Shares its corners with the upper lid. |
| Lips, outer outline | Yes | Front, plus profile for how far each lip projects | 6 per mouth with symmetry, from the 2 corners, 2 bow peaks, 1 dip and 1 lowest point | The corners are shared with the line between the lips. Projection needs 2 more numbers or a short profile curve. |
| Line between the lips | Yes | Front | 5 with the corners | Affects every mouth shape in lip sync, so its limits should be small. |
| Eyebrow, painted shape | Yes, as a mask on the texture | Front | 4 or 5 along the upper edge, plus a thickness at each point | This is a texture edit. It has no depth problem and no animation problem. No game offers it, so it would also be new. |
| Eyebrow, position on the face | Partly | Front | 3 (head, arch, tail), as Justice does with bones | Moves the brow's skin, not its paint. |
| Nose profile | Yes | Profile | 5, from nasion through the dorsum, tip and columella to subnasale | The sliders already recover 64% of the nose, so the gain is smaller here. |
| Brow ridge | Partly | Profile | 3 at the midline | A profile curve sets the ridge's projection. Its width across the brow needs region shapes. |
| Fold beside the nose | Yes, as a crease | Three-quarter or front | 3 or 4 along its path, plus a depth and a width | Wires and Reallusion both treat it as a curve. It needs a depth number that no outline gives. |
| Jawline | Weakly | Three-quarter, as a curve on the surface | 4 or 5 per side | A front-view jawline is a silhouette that slides on the surface. Bind it to the jawbone edge instead, or use it only as a fitting target. |
| Cheek hollow and cheekbone | No | None | None | An area seen only in shading. It needs region shapes, such as new morph targets or a local statistical model from scans. |

With symmetry on, this comes to about 35 to 40 control points for the whole face. Each point needs 2 or 3 numbers.

### Workable designs

**Design 1. Curves as targets for the existing sliders.** The player edits a curve, and a least-squares solve finds the slider values whose mesh best matches the curve's sample points. It is the Lewis and Anjyo solve from the sculpting note, with curve samples in place of a dragged vertex. It is also what the prototype's outline fitting already does with traced photo outlines.

- It keeps sliders as the single source of truth, so storage, sharing, symmetry and animation do not change.
- It needs a regularisation term toward the current values, which also limits the grotesque stacking seen in the prototype.
- It cannot pass the slider ceiling, which is 1.7 mm on the Lincoln scan.
- I estimate a few days of work, because the prototype's fitting code already does the hard part (inference).

**Design 2. A wire deformer baked into the rest shape.** Each curve is a Singh and Fiume wire with a falloff radius. The curve's change from its default moves nearby vertices, and the result is baked into the head before expressions are added.

- It can make shapes that no slider makes, so it can pass the ceiling.
- It needs hard limits on each control point, because nothing keeps the result human.
- It works on the lips and eyelids only if the expression shapes still look right on the edited face. Small limits are the cheap way to keep that true.
- Domain curves can limit a fold's wire to the cheek side, as the Wires paper does for a crease.
- I estimate one to two weeks of work, most of it tuning radii and limits (inference).

**Design 3. Laplacian deformation with the curve as a soft constraint.** The curve's sample points become targets in a solve that keeps the surface's local shape. This is FiberMesh's and DeepSketch2Face's method.

- It gives smoother results than a wire and keeps existing detail.
- It needs a sparse solve per edit. With a factorisation computed once, that is fast enough for the prototype's mesh size. FiberMesh ran at 10 to 15 frames per second on 600 to 2,000 vertices on a 2007 laptop ([FiberMesh](https://www.cs.princeton.edu/courses/archive/fall07/cos597B/papers/nealen-fibermesh.pdf)).
- It has the same plausibility problem as design 2.
- I estimate two to three weeks of work (inference).

**Design 4. Joints along the curve for the eyelids.** A short chain of joints rotates about each eye's centre, aimed at points on the lid curve. This is the standard film eyelid rig.

- It solves the eyelid's depth problem by construction.
- It adds joints to the head's skeleton, and those joints must also drive blinks.
- It suits only the eyelids.

**Design 5. Curves as targets for new local shapes learned from scans.** This is design 1 with a better shape basis. New morph targets or local statistical models are built for the regions where the Lincoln test failed, such as the cheeks, the brow ridge and the folds. Curves and sliders then both drive them.

- It is the only design that addresses the measured gap and keeps faces plausible.
- It needs head scans, which the Lincoln test found hard to obtain, and the work of fitting local models.
- It is the most expensive design by a wide margin.

### The cheapest workable plan

1. **Build design 1 first, for the eyelids, the lips and the nose profile.** It reuses the prototype's outline fitting and changes nothing downstream. It will show whether players find curves easier than sliders, which no published study has tested.
2. **Add the painted eyebrow curve.** It is a texture mask with 4 or 5 points and a thickness at each point. It has no depth problem and no animation problem, and no shipped game offers it.
3. **Add new shapes for the cheeks, the brow ridge and the folds.** The Lincoln test says this is where likeness is lost. These shapes can be sliders or clay regions. They do not need curves, except for the fold, which can take a crease curve with a depth.
4. **Add design 2 only if design 1 hits the slider ceiling on real targets.** Keep its limits small, and keep sliders next to every curve.

### What to store

| Item | Count | Size |
|---|---|---|
| Eyelids, upper and lower, with symmetry | 7 unique points × 2 numbers, because the lids share their corners | 14 bytes |
| Lips, outer and inner | 9 unique points × 2 numbers, plus 2 projection values | 20 bytes |
| Painted eyebrow | 5 points × 3 numbers (2 for position, 1 for thickness) | 15 bytes |
| Nose and brow profile | 8 points × 2 numbers | 16 bytes |
| Folds beside the nose | 4 points × 2 numbers, plus depth and width | 10 bytes |
| Jaw, if used | 5 points × 2 numbers | 10 bytes |

With symmetry and one byte per number, a fully edited face adds under 100 bytes to the look block. An asymmetric face doubles the points that are not on the midline. Untouched curves need not be stored. As for sliders, a loader must clamp every value to the current limits, and the curve rules need a version number.
