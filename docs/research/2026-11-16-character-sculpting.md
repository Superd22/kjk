# Direct sculpting in character creators

This note looks at one kind of character creator control in depth. In a **sculpting** creator, the player grabs a point on the character's face or body with the mouse and drags it. The face changes where the player pulled. This is different from a list of **sliders**, where each slider changes one named feature, such as "nose length".

It was written on 2026-11-16 for kjk. Two other notes in this folder cover the wider subject. The design survey is in `docs/research/2026-11-16-character-creation-design.md`, and the general technology is in `docs/research/2026-11-16-character-creation-tech.md`. This note does not repeat them. In particular, the technology note already explains morph targets, FaceGen, MetaHuman's DNA and RigLogic, corrective shapes, deformation maps, wrap deformers and EVE Online's sculpt mode. This note goes deeper on sculpting only.

Every claim links to its source. Where only a press article, a fan wiki or a modding tool was available, the text says so with the words "secondary source". Section 9 reports measurements that were taken for this note from KOTOR's own head models.

## 1. Terms used in this note

A **morph target** (also called a **blend shape** or **shape key**) is a copy of a mesh with some vertices moved. A weight between 0 and 1 blends the mesh toward that copy.

A **bone** (also called a **joint**) is a point with a position, a rotation and a scale. Vertices that are **skinned** to a bone follow it when it moves. Each vertex has a weight for each bone that moves it.

A **region** is the part of a face or body that one drag changes, such as "the nose" or "the left cheek".

A **hotspot** is the spot on the screen that the player can grab to change a region. Games usually highlight it when the mouse passes over it.

A **parameter vector** is the list of numbers that describes a character's shape, for example 200 slider values. A creator that stores a parameter vector can rebuild the same mesh on any computer from those numbers.

A **morphable model** is a statistical face model built from many scanned faces. Every face it produces is the average face plus a weighted sum of a few dozen to a few hundred "principal components". Principal components are the main directions in which the scanned faces differ from the average.

## 2. Which games let the player sculpt, and how it feels

### The Sims 4

The Sims 4's Create a Sim is the best documented sculpting creator. Sri Nair of Electronic Arts presented it at GDC 2015 in the talk "Innovations in The Sims 4 Character Creator". The talk abstract says that players "pull and push" the character's shape directly. It says the system is built from data-driven "selectors and modifiers", which include morph targets, bone-driven changes and deformation maps ([GDC Vault](https://www.gdcvault.com/play/1022085/Innovations-in-The-Sims-4)).

EA's patents describe the behaviour in more detail. US patent 10,275,947 says that a side-to-side drag on the nose widens or narrows it in the front view, but lengthens or shortens it in the profile view. A vertical drag moves a feature up or down. A diagonal drag changes two things at once. Holding a key limits the change to one direction. The mouse cursor changes shape to show whether the hotspot moves, rotates or scales ([Google Patents, US10275947B2](https://patents.google.com/patent/US10275947B2/en)).

The same patent describes how the game keeps faces plausible. Each change has a maximum displacement. A teen's nose, for example, can be lengthened to only 80% of the maximum allowed for an adult. When the player drags close to the limit, the hotspot turns yellow and the cursor slows down. At the limit, the hotspot turns red, and the game can play a sound or a controller vibration ([Google Patents, US10275947B2](https://patents.google.com/patent/US10275947B2/en)).

Community reverse engineering confirms how the shipped game stores this. The open-source Sims4Tools package editor reads a resource called `HotSpotControl`. Each hotspot has a zoom level (`Macro`, `Micro` or `Special`), a cursor type, a body region and a list of entries. Each entry names a view angle, a "flip horizontal" flag, and four modifiers called `SimModifierLeft`, `SimModifierRight`, `SimModifierUp` and `SimModifierDown` ([Sims4Tools, HotSpotControl.cs, lines 160 to 166](https://github.com/s4ptacle/Sims4Tools/blob/b5db166dd4b935abc9f47c4c998fce98c61bd4de/s4pi%20Wrappers/CASPartResource/HotSpotControl.cs#L160-L166)). The view angles are front, three-quarter right, three-quarter left, profile right, profile left and back. The cursor types are horizontal and vertical arrows, horizontal arrows only, vertical arrows only, diagonal, and rotation ([Sims4Tools, HotSpotControl.cs, lines 263 to 299](https://github.com/s4ptacle/Sims4Tools/blob/b5db166dd4b935abc9f47c4c998fce98c61bd4de/s4pi%20Wrappers/CASPartResource/HotSpotControl.cs#L263-L299)). In other words, a drag to the left raises the weight of one modifier, and a drag to the right raises the weight of a different modifier. Each direction is a separate one-sided shape, and the mapping changes with the camera angle.

Each `SimModifier` in turn points to a bone pose, a deformation map for the shape, a deformation map for the normals, and a list of bones with a multiplier for each ([Sims4Tools, SimModifierResource.cs, lines 41 to 48](https://github.com/s4ptacle/Sims4Tools/blob/b5db166dd4b935abc9f47c4c998fce98c61bd4de/s4pi%20Wrappers/CASPartResource/SimModifierResource.cs#L41-L48)). So one drag direction can move bones and displace vertices at the same time.

### Fallout 4

Fallout 4 lets the player select a face region and change its position, rotation and scale (secondary source, [SiliconANGLE player guide](https://siliconangle.com/blog/2015/11/17/how-to-make-a-fallout-4-character-that-isnt-butt-ugly/)). The nose, for example, has separate sculpt regions for the bridge, the tip and the nostrils (secondary source, same guide). A Nexus modder writes that the face shaping is done with "a bunch of bones", and that each grabbed region moves bones (secondary source, [Nexus Mods](https://www.nexusmods.com/fallout4/mods/25087)).

The game's save data confirms the region model. The xEdit record definitions for Fallout 4 show that each character stores a list of "Face Morphs". Each entry is a region index followed by seven numbers. The numbers are a position in x, y and z, a rotation about x, y and z, and one uniform scale ([xEdit, wbDefinitionsFO4.pas, lines 10408 to 10422](https://github.com/TES5Edit/TES5Edit/blob/e60cd5e755f2bf3715575bd3b3e38e59a323d656/Core/wbDefinitionsFO4.pas#L10408-L10422)). The same record also stores a list of morph keys with a value for each, five body values (head, upper torso, arms, lower torso and legs), and one "Facial Morph Intensity" value ([xEdit, wbDefinitionsFO4.pas, lines 10383 to 10424](https://github.com/TES5Edit/TES5Edit/blob/e60cd5e755f2bf3715575bd3b3e38e59a323d656/Core/wbDefinitionsFO4.pas#L10383-L10424)). Fallout 4's sculpting is therefore a set of rigid transforms, one per face region, on top of a set of morph weights. It is the closest commercial example to what kjk could do with KOTOR's bones (see section 9).

### Black Desert Online

Pearl Abyss's official customization guide describes Black Desert's face editing in these steps. The player clicks the face area to change. The player then picks a controller type. The controller types are move, rotate and size. The player then drags the part directly, or uses a control bar for length, width and depth ([Black Desert, Adventurer's Guide, Customization](https://blackdesert.pearlabyss.com/asia/en-US/Game/Wiki?_masterWikiNo=3)). The guide says that "each direction is applied based on the screen view regardless of the direction of the character" ([same guide](https://blackdesert.pearlabyss.com/asia/en-US/Game/Wiki?_masterWikiNo=3)). A drag to the right therefore always moves the part to the right on the screen, whichever way the head is turned. This is simpler than The Sims 4, where the meaning of a drag depends on the view.

The body uses the same controllers. The guide says the player can "directly move the part you want to adjust by clicking on it" ([Black Desert, GM Notes, Customization Guide](https://blackdesert.pearlabyss.com/ASIA/en-us/News/Notice/Detail?_boardNo=9)).

Black Desert keeps an undo history for each part, such as the hair or the face. The player steps through it with "Previous" and "Next" buttons. It also keeps up to 10 temporary saves ([Black Desert, Adventurer's Guide](https://blackdesert.pearlabyss.com/asia/en-US/Game/Wiki?_masterWikiNo=3)). Neither official page mentions a symmetry option. I found no technical talk or patent from Pearl Abyss about how the editor works.

### MetaHuman Creator

MetaHuman Creator is Epic's character tool, not a game, but it is the most advanced public sculpting creator. It has three ways of shaping a head.

- The **Blend** tool shapes a face by mixing up to three presets. The player drops presets onto an "Influence Circle" and drags a control point toward one of them. The blend can apply to the whole head or to one local region ([Epic, Blend tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-blend-tool-in-unreal-engine)).
- The **Transform** tool moves broad regions. Each marker controls a group of underlying markers, so one drag produces a broad change ([Epic, Head Transform tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-head-transform-tool-in-unreal-engine)).
- The **Sculpt** tool makes small local changes. It places markers on the face. The area around the eyes, for example, has two markers under the brow ridge and four more for the lids and corners. The player can add a marker with Ctrl and a click, and remove one the same way ([Epic, Head Sculpt tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-head-sculpt-tool-in-unreal-engine)).

Edits are symmetric by default. The player turns off "Symmetric Manipulation" to make one side different from the other. Epic warns that turning symmetry back on keeps "some essence of the asymmetry" in regions edited later ([Epic, Head Sculpt tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-head-sculpt-tool-in-unreal-engine)).

The most important detail is how MetaHuman keeps faces human. The sculpt does not move vertices freely. The system "finds the closest combination of real human facial features scanned into the MetaHuman database" and shows that instead ([Epic, Head Sculpt tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-head-sculpt-tool-in-unreal-engine)). A drag is a request, and the result is the nearest real face. Section 3.3 explains the mathematics behind this kind of projection.

### inZOI

inZOI is KRAFTON's life simulation game. Its Character Studio was released on Steam on 16 August 2024 ([KRAFTON press release](https://press.krafton.com/KRAFTON-Launches-inZOI-Character-Studio-on-Steam)). I found no official KRAFTON description of its sculpting controls. Fan sources say the face can be edited with precise sliders or with a sculpt mode in which the player grabs the face "like clay". They say that symmetric editing is the default, and that an individual mode allows asymmetric changes. They also say the body can be pulled outward to enlarge a part and pushed inward to shrink it (secondary source, [The Games Wiki, Create a Zoi](https://thegameswiki.com/inzoi/wiki/create-a-zoi/history/892b1585-d0a7-4bab-a069-3649ceca8e25) and [The Games Wiki, body type customization](https://thegameswiki.com/inzoi/wiki/body-type-customization)). PC Gamer reports that the creator has a basic edit mode and a detailed edit mode that do different things (secondary source, [PC Gamer](https://www.pcgamer.com/games/life-sim/inzoi-character-studio-tips)).

### Games that use sliders rather than sculpting

The request for this note named several more games. For most of them I found no source that shows direct dragging on the model.

- **Dragon's Dogma 2** uses sliders, with 12 sliders for a human nose alone according to an IGN-sourced report (secondary source, [esports.net](https://www.esports.net/news/dragons-dogma-2-character-creation/)). Its developers scanned about 80 heads as base faces ([Autodesk Area Japan](https://area.autodesk.jp/case/game/dragonsdogma-2/)). The creator's technology is covered in the technology note.
- **Monster Hunter Wilds** offers contour, nose, eye and skin settings that press guides describe as slider menus (secondary source, [Game8](https://game8.co/games/Monster-Hunter-Wilds/archives/482570)).
- **Dragon Age: The Veilguard** uses sliders and a triangle "Body Morpher", as the design note describes.
- **Phantasy Star Online 2 New Genesis** uses sliders. A press report lists new sliders for the outer corner of the eyes and for eyelid height (secondary source, [Bumped.org](https://www.bumped.org/psublog/whats-new-in-pso2-new-genesis-character-creator/)). SEGA gave a CEDEC 2022 talk on how New Genesis carried old avatar data onto its new renderer ([CEDEC 2022 session page](https://cedec.cesa.or.jp/2022/session/detail/25.html)). Its slides require a CEDiL login ([CEDiL](https://cedil.cesa.or.jp/cedil_sessions/view/2558)), so I could not read them.
- **Soulcalibur VI** uses sliders for height, body parts and muscle definition (secondary source, [PC Invasion](https://www.pcinvasion.com/make-mega-man-chun-li-soulcalibur-6/)).
- **Saints Row (2022)** introduced asymmetric faces, where the left side of the face can differ from the right side (secondary source, [TechRadar](https://techradar.com/features/weve-fiddled-with-saints-rows-free-character-creator-and-its-a-powerful-tool-for-representation)). I found no source that says its face editor uses dragging.
- **Lost Ark** offers face types, presets and makeup (secondary source, [Fextralife wiki](https://lostark.wiki.fextralife.com/Character+Creation)). I found no source for direct dragging.

In short, the well-documented sculpting creators are The Sims 4, Fallout 4, Black Desert Online, EVE Online (in the technology note), MetaHuman Creator, and probably inZOI. Many highly praised creators, including Dragon's Dogma 2, still use sliders.

### Design lessons from these creators

1. **The drag has to mean something predictable.** The Sims 4 makes a drag mean different things in front and profile views. Black Desert always moves the part along the screen. Both work, but the player must be able to guess the result before letting go.
2. **Show what can be grabbed.** The Sims 4 highlights hotspots and changes the cursor to show move, rotate or scale. MetaHuman shows markers. Black Desert asks the player to pick a controller type first.
3. **Symmetry is on by default.** MetaHuman and inZOI (secondary source) both default to symmetric edits. Asymmetry is a deliberate choice.
4. **Limits are felt, not hidden.** The Sims 4 slows the cursor and colours the hotspot as the player nears a limit. A slider shows its end stop. A drag needs an equivalent.
5. **Undo is per part.** Black Desert keeps an undo history per part, which matches how players work on one feature at a time.
6. **Sliders stay available.** Black Desert keeps length, width and depth bars next to the drag. inZOI keeps numeric sliders next to its sculpt mode. Lewis and Anjyo's study (section 3.2) found that a drag is sometimes faster than sliders and sometimes slower, which supports offering both.

## 3. How a drag becomes a shape change

A sculpting creator has to solve four problems. It has to find what the player grabbed. It has to turn a 2D mouse movement into a change of one or more parameters. It has to keep the result plausible. It has to mirror the edit to the other side when symmetry is on.

### 3.1 Finding the region under the cursor

There are three common methods.

**Region ID images.** EA's patent US 9,940,757 stores each selectable region as a unique colour in an image that is mapped onto the character's mesh. The game finds where the cursor touches the mesh, reads the colour there, and looks up the region ([Google Patents, US9940757B2](https://patents.google.com/patent/US9940757B2/en)). The patent uses one colour channel for each zoom level. In US 10,275,947, the blue channel holds the top-level regions, green holds the "macro" regions and red holds the "micro" regions ([Google Patents, US10275947B2](https://patents.google.com/patent/US10275947B2/en)). Zooming in on the face therefore makes smaller regions selectable without changing the mesh. EVE Online also finds the clicked point through the mesh's texture coordinates, as the technology note describes.

**Ray casting against the mesh.** The engine shoots a ray from the camera through the mouse position and finds the first triangle it hits. Bevy has this built in. In Bevy 0.19.1, a ray cast hit returns the hit point, the triangle index, the barycentric coordinates within the triangle and the texture coordinate ([docs.rs, Bevy 0.19.1, RayMeshHit](https://docs.rs/bevy/0.19.1/bevy/picking/mesh_picking/ray_cast/struct.RayMeshHit.html)). From the triangle and the barycentric coordinates, the creator can read any per-vertex data, such as a region ID or the vertex's bone weights. One caution applies. Bevy's mesh ray cast tests the mesh's stored vertex positions and its transform. It does not apply skinning or morph targets ([Bevy 0.19.1 source, intersections.rs](https://docs.rs/crate/bevy_picking/0.19.1/source/src/mesh_picking/ray_cast/intersections.rs)). A creator that picks on a posed or morphed head must either pick on a CPU copy of the deformed mesh, or keep the head in its rest pose while sculpting.

**Handles.** MetaHuman places visible markers on the face, and the player grabs a marker rather than the skin ([Epic, Head Sculpt tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-head-sculpt-tool-in-unreal-engine)). Handles are easier to see and easier to hit with a controller cursor. They also make the set of regions obvious. Their cost is clutter on a small face.

### 3.2 Turning a 2D drag into parameter changes

**One shape per drag direction.** The Sims 4 gives every hotspot four modifiers, one per direction, for each view angle (section 2). This is the simplest mapping. A drag right of 50 pixels sets the "right" modifier to some weight, and the "left" modifier stays at zero. Vertical and horizontal movement are independent, so a diagonal drag changes two modifiers at once.

**A pair of opposite shapes per axis.** MakeHuman and its Blender successor MPFB use the same idea for sliders. Each MPFB modifier has a negative and a positive target. For example, `nose-trans-down-up` blends toward a "down" shape on one side of zero and toward an "up" shape on the other. MPFB's target list has 234 modifiers, and 66 of them have separate left and right versions ([MPFB, target.json](https://github.com/makehumancommunity/mpfb2/blob/9de0707fdc88438d7c53a1a2ac7937d9221e2431/src/mpfb/data/targets/target.json)). A drag-to-sculpt interface on such a model only needs to map each screen axis to one of these signed modifiers.

**Bone transforms.** Fallout 4 and Black Desert move, rotate or scale a region. Fallout 4 stores a position, a rotation and a scale per region (section 2). A drag becomes a translation of the region's bone, in the plane of the screen or along a chosen axis.

**Points inside a 2D field.** EVE Online's regions each hold a small 2D field whose corners store mixes of morph weights. The drag moves a point inside the field, as described in the technology note. MetaHuman's Blend tool works the same way, with presets in place of the corners ([Epic, Blend tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-blend-tool-in-unreal-engine)).

**Solving for weights.** The most general method treats the drag as an inverse problem. The player says where one vertex should go, and the creator finds the parameter values that put it there. J. P. Lewis and Ken Anjyo published this for blend-shape faces in 2010 as "Direct Manipulation Blendshapes" ([preprint](https://www.scribblethink.org/Work/DirectManipBlendshapes/DMBpreprint.pdf); [project page](https://scribblethink.org/Work/DirectManipBlendshapes/directmanip.html)). Their key idea is that the slider values are already a good description of the face. So the solver looks for the smallest change in slider values that moves the grabbed vertex where the player dragged it. Their objective is

`minimise |B w − m|² + α |w − w0|² + μ |w|²`

In this formula, `w` is the new vector of blend-shape weights and `w0` is the vector before the drag. `m` holds the target positions of the dragged and pinned vertices, and `B` holds the rows of the blend-shape matrix for those vertices. The paper sets `α` to about 0.1 and `μ` to 0.001, and it clamps every weight to the range 0 to 1 ([preprint](https://www.scribblethink.org/Work/DirectManipBlendshapes/DMBpreprint.pdf)). Each pinned vertex adds three rows to `B`. A pinned vertex is one the player has fixed in place so that it does not move with the drag. The paper uses soft constraints instead of exact ones on purpose, because they let the face move in directions that are almost, but not quite, blocked by the pins. The `μ` term stops the weights from drifting after many back-and-forth drags ([preprint](https://www.scribblethink.org/Work/DirectManipBlendshapes/DMBpreprint.pdf)).

The system to solve is small. With 40 blend shapes, it is a 40 by 40 linear system per mouse movement. This makes the method practical in a game creator. Its result is still a slider vector, so the sliders and the drag stay in step.

### 3.3 Local regions in statistical models

A whole-face morphable model has a weakness for sculpting. Each principal component changes the whole face a little. Dragging the nose would also shift the jaw and the eyes.

Blanz and Vetter's 1999 paper, which introduced the 3D morphable face model, already addressed this. It splits the face into four regions (the eyes, the nose, the mouth and the surrounding area). Each region is blended independently, and the results are blended together at the borders. The model was built from 200 scanned heads with about 70,000 vertices each ([Blanz and Vetter, SIGGRAPH 1999](https://www.cs.princeton.edu/courses/archive/spring11/cos598A/pdfs/Blanz99.pdf)). The paper also keeps edited faces plausible by placing a probability on the coefficients. A face's likelihood falls as its coefficients move away from the average, measured in units of each component's spread ([same paper](https://www.cs.princeton.edu/courses/archive/spring11/cos598A/pdfs/Blanz99.pdf)). Its successors are summarised in the 2019 survey "3D Morphable Face Models, Past, Present and Future" ([Egger et al., arXiv 1909.01815](https://arxiv.org/abs/1909.01815)).

Later research made the regions finer and made them work with dragging.

- **Tena, De la Torre and Matthews (2011)** built a face from many small PCA models, one per region. PCA (principal component analysis) is the method that finds the principal components defined in section 1. The regions are found automatically from data. Neighbouring regions share their border vertices, and agreement at the borders is enforced in a least-squares sense, together with the player's drag constraints. The model is posed by click and drag ([Disney Research](https://la.disneyresearch.com/publication/interactive-region-based-linear-3d-face-models/); [paper PDF](https://la.disneyresearch.com/wp-content/uploads/tena-siggraph2011_11.pdf)).
- **Neumann and others (2013)** published "Sparse Localized Deformation Components". It finds deformation components that each affect only a small area, which makes them suitable for local edits ([SIGGRAPH history archive](https://history.siggraph.org/?p=208344)). A reference implementation is on GitHub ([tneumann/splocs](https://github.com/tneumann/splocs)).
- **Gruber and others (2020)** built an "anatomical local face model" from several hundred scans. The player can drag the skin directly, change age, sex, ethnicity and body mass index, or reshape the skull and jaw. The skin follows the bones ([ETH Research Collection](https://research-collection.ethz.ch/handle/20.500.11850/432184)). The matching patent says the face is split into patches. Each patch has its own compressed shape space and its own sliding over the bone. A drag becomes a constraint in an optimisation that solves for the patch weights ([Google Patents, US11645813B2](https://patents.google.com/patent/US11645813B2/en)).

MetaHuman's "closest combination of real human facial features" is very likely a commercial version of this idea. Epic has not published the method, so this is an inference.

### 3.4 Keeping the face plausible

Every creator above uses one or more of these guards.

- **Hard ranges per parameter.** The Sims 4 stores a maximum displacement per change, scaled by age group ([US10275947B2](https://patents.google.com/patent/US10275947B2/en)). Lewis and Anjyo clamp every weight to 0 to 1 ([preprint](https://www.scribblethink.org/Work/DirectManipBlendshapes/DMBpreprint.pdf)). The NetEase games studied in section 5 normalise every parameter to the range 0 to 1 ([Shi et al., ICCV 2019](https://ar5iv.labs.arxiv.org/html/1909.01064)).
- **Ranges that depend on other values.** Dragon's Dogma 2 makes some sliders depend on others. This prevents, for example, a very large head on a very thin body ([Autodesk Area Japan](https://area.autodesk.jp/case/game/dragonsdogma-2/)).
- **A statistical prior.** Blanz and Vetter penalise unlikely coefficients. MetaHuman projects every sculpt onto its scan database.
- **Soft feedback near the limit.** The Sims 4 slows the cursor and colours the hotspot yellow, then red ([US10275947B2](https://patents.google.com/patent/US10275947B2/en)).

### 3.5 Symmetric edits

Symmetry is simple when the parameters already come in left and right pairs. MPFB's modifiers name both sides, for example `l-eye-corner1-up` and `r-eye-corner1-up` ([MPFB, target.json](https://github.com/makehumancommunity/mpfb2/blob/9de0707fdc88438d7c53a1a2ac7937d9221e2431/src/mpfb/data/targets/target.json)). A symmetric drag writes the same value to both. For a translation, the mirrored side receives the same movement with the left-right component negated. Each Sims 4 hotspot entry has a "flip horizontal" flag, which suggests that one hotspot definition serves both sides by mirroring ([Sims4Tools, HotSpotControl.cs, line 235](https://github.com/s4ptacle/Sims4Tools/blob/b5db166dd4b935abc9f47c4c998fce98c61bd4de/s4pi%20Wrappers/CASPartResource/HotSpotControl.cs#L235)). This reading of the flag is an inference from its name.

A statistical model needs more care. Its components are usually not symmetric. Epic's warning that MetaHuman keeps "some essence of the asymmetry" after symmetry is turned back on shows the problem ([Epic, Head Sculpt tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-head-sculpt-tool-in-unreal-engine)).

### 3.6 The result is still a parameter vector

None of the creators above store a sculpted mesh for the player's character, with one exception. The Sims 4 stores modifier weights. Fallout 4 stores region transforms and morph weights. MPFB stores modifier values. MetaHuman stores a DNA file that describes the rig and geometry (see the technology note).

The exception is Mass Effect. Its save files store a `HeadMorph` with morph feature weights and bone offsets. They also store the vertex positions of all four levels of detail of the head, in fields called `lod0_vertices` to `lod3_vertices` ([Trilogy Save Editor, appearance.rs](https://github.com/KarlitosVII/trilogy-save-editor/blob/6451f23d1fba62358328dce4e87733970f36ebf1/src/save_data/shared/appearance.rs)). One Mass Effect base head has 2,232 vertices at its highest level of detail ([LegendaryExplorer wiki](https://github.com/ME3Tweaks/LegendaryExplorer/wiki/Working-with-MorphTargets-and-Morphable-Heads)). At 12 bytes per vertex, that one level is about 27 KB, which is far more than the slider values. This is my own estimate from those two sources. Storing the mesh makes the save independent of later changes to the morph data, but it costs space and makes sharing harder.

## 4. How the sculpted shape reaches everything else

### Other levels of detail

A blend shape authored on the highest level of detail must also exist on each lower level, or the face changes shape when the camera moves away. Mass Effect's morph targets carry vertex data for two levels of detail, while their bone offsets apply to every level because all levels share one skeleton ([LegendaryExplorer wiki](https://github.com/ME3Tweaks/LegendaryExplorer/wiki/Working-with-MorphTargets-and-Morphable-Heads)). This is a general rule. A bone-based sculpt reaches every level of detail for free. A vertex-based sculpt has to be authored or transferred for each level.

### Hair, beards, eyebrows, helmets and eyelashes

Three methods appear in the sources.

1. **Bind each vertex of the attachment to the face.** MakeHuman's clothes, hair and eyelash files (`.mhclo`) tie every vertex of the attachment to three vertices of the base body. The file stores three base vertex indices, three weights that sum to 1, and an offset ([MPFB, mhclo format](https://github.com/makehumancommunity/mpfb2/blob/9de0707fdc88438d7c53a1a2ac7937d9221e2431/docs/fileformats/mhclo.md)). When the base mesh is sculpted, the attachment's vertices are rebuilt from the moved base vertices. The same file can also scale the offsets by the distance between two chosen base vertices on each axis ([same document](https://github.com/makehumancommunity/mpfb2/blob/9de0707fdc88438d7c53a1a2ac7937d9221e2431/docs/fileformats/mhclo.md)).
2. **A wrap deformer run once at edit time.** Dragon's Dogma 2 places beards, eyebrows and helmets on the edited face and deforms them with an in-engine wrap deformer modelled on Maya's. The result is then frozen and skinned, so the wrap does not run during play ([Autodesk Area Japan](https://area.autodesk.jp/case/game/dragonsdogma-2/)).
3. **A deformation map shared by all attachments.** EA's patent US 9,892,540 stores the body's displacement for each point of its surface in an image. Any clothing item that maps its vertices into that image can read the same displacement. Two maps represent tight and loose fits, and a vertex colour chooses between them ([Google Patents, US9892540B2](https://patents.google.com/patent/US9892540B2/en)).

A fourth method follows from section 9 and is cheap for kjk. If the sculpt is an analytic field, meaning a formula that gives a displacement for any point in head space, then any attachment can be deformed by evaluating the same formula at its vertices. No binding data is needed.

### Expressions and lip sync on a sculpted face

There are two cases.

**Bone-based faces.** If expressions and sculpting both use bones, the sculpt must be layered under the animation, not replaced by it. Dragon's Dogma 2 applies posture variations as offsets on top of base motions, in the way a Maya animation layer works. It also sets limits per body part that switch the effect off when rotations become too extreme ([Autodesk Area Japan](https://area.autodesk.jp/case/game/dragonsdogma-2/)). MetaHuman keeps animations working on every face because all MetaHumans "can run on the same facial rig" (secondary source, [80.lv](https://80.lv/articles/epic-revealed-the-tech-side-of-metahuman-creator-face-rigs)). Its rig definition is "static", and expressions are stored per face as joint animations plus a per-vertex correction used only at the highest level of detail ([Epic, MetaHuman DNA](https://dev.epicgames.com/documentation/metahuman/metahuman-dna-rig-definition-and-rig-operation)).

**Blend-shape faces.** If the expressions are blend shapes, an expression authored on the average face is not correct on a sculpted face. A smile shape adds the same offsets to every face, even though a wide mouth should stretch further than a narrow one. Deformation transfer solves this by copying the deformation of each triangle from one mesh to another ([Sumner and Popović, SIGGRAPH 2004](https://people.csail.mit.edu/sumner/research/deftransfer/)). Corrective shapes, covered in the technology note, fix the remaining errors.

### Armour on sculpted bodies

The technology note covers the general methods. One sculpting-specific point is worth adding. Body sculpting by bone scale reaches every armour model skinned to the same skeleton. Body sculpting by vertex displacement needs a transfer method, such as the deformation maps above. The Sims 4 uses both kinds, because each of its modifiers can carry a bone list and two deformation maps ([Sims4Tools, SimModifierResource.cs](https://github.com/s4ptacle/Sims4Tools/blob/b5db166dd4b935abc9f47c4c998fce98c61bd4de/s4pi%20Wrappers/CASPartResource/SimModifierResource.cs#L41-L48)).

## 5. Data size, presets and sharing

The table lists what a sculpted character stores in each documented system.

| System | What one character stores | Source |
|---|---|---|
| The Sims 4 | Weights for modifiers, which the game names by 64-bit instance IDs. I found no public description of how a save stores the weights. | [Sims4Tools](https://github.com/s4ptacle/Sims4Tools/blob/b5db166dd4b935abc9f47c4c998fce98c61bd4de/s4pi%20Wrappers/CASPartResource/HotSpotControl.cs#L163-L166) (the IDs only) |
| Fallout 4 | 7 numbers per edited face region, a list of morph keys and values, 5 body values and 1 intensity value. | [xEdit](https://github.com/TES5Edit/TES5Edit/blob/e60cd5e755f2bf3715575bd3b3e38e59a323d656/Core/wbDefinitionsFO4.pas#L10383-L10424) |
| NetEase "Justice" | 208 continuous face values, plus 56 discrete values for male characters or 102 for female characters. | [Shi et al., ICCV 2019](https://ar5iv.labs.arxiv.org/html/1909.01064) |
| NetEase "Revelation" (mobile) | 174 adjustable values. | [Shi et al., AAAI 2020](https://ar5iv.labs.arxiv.org/html/2008.07132) |
| MakeHuman / MPFB | One line per non-zero modifier, such as `modifier nose/nose-trans-down-up 0.300000`, out of 234 modifiers. | [MPFB, mhm format](https://github.com/makehumancommunity/mpfb2/blob/9de0707fdc88438d7c53a1a2ac7937d9221e2431/docs/fileformats/mhm.md) |
| Mass Effect | Feature weights, bone offsets, and the head's vertex positions for 4 levels of detail. | [Trilogy Save Editor](https://github.com/KarlitosVII/trilogy-save-editor/blob/6451f23d1fba62358328dce4e87733970f36ebf1/src/save_data/shared/appearance.rs) |

The NetEase papers add a useful fact about what these values mean. In the "Justice" face editor, each "controller panel" holds several continuous values for one facial part, and those values represent the part's "shift, orientation and scale" ([Shi et al., AAAI 2020](https://ar5iv.labs.arxiv.org/html/2008.07132)). That is the same design as Fallout 4's per-region transforms. A large Chinese MMO therefore uses about 200 bone-like transform values per face.

Sharing follows from the storage. Black Desert saves customization files that players can rename, swap and share with others ([Black Desert, GM Notes](https://blackdesert.pearlabyss.com/ASIA/en-us/News/Notice/Detail?_boardNo=9)). Dragon's Dogma 2 shares whole player-made pawns online, which is one reason its editor has so many options ([Autodesk Area Japan](https://area.autodesk.jp/case/game/dragonsdogma-2/)).

### Versioning

Two cautions come from the sources.

- **Formats grow.** The Sims 4's `SimModifier` resource added a sub-region field at version 144, and readers must check the version before reading it ([Sims4Tools, SimModifierResource.cs, line 86](https://github.com/s4ptacle/Sims4Tools/blob/b5db166dd4b935abc9f47c4c998fce98c61bd4de/s4pi%20Wrappers/CASPartResource/SimModifierResource.cs#L86)). MPFB's preset format adds optional fields, such as `expressions`, that older presets omit and still load ([MPFB, human preset format](https://github.com/makehumancommunity/mpfb2/blob/9de0707fdc88438d7c53a1a2ac7937d9221e2431/docs/fileformats/human_preset.md)).
- **Parameters mean only what the assets say.** A stored parameter vector rebuilds the face from the game's current morph data and bones. If an update changes a morph target, every saved face that uses it changes too. Mass Effect's choice to store vertex positions avoids this, at the cost described in section 3.6.

## 6. Patents

This section describes patents as technical sources and as a legal caution. It is not legal advice. Whether any design infringes a patent depends on the full claims and on the law of each country, and a patent lawyer should check any design that kjk ships. "Anticipated expiration" dates come from Google Patents, which says they are an assumption, not a legal conclusion.

| Patent | Owner | Status | What it covers |
|---|---|---|---|
| [US 9,940,757 B2](https://patents.google.com/patent/US9940757B2/en), "Modifying a simulated character by direct manipulation" | Electronic Arts | Granted 10 April 2018. Active. Anticipated expiration 17 March 2036. Priority date 1 August 2014. | Claim 1 picks a colour channel by zoom level, reads a region ID from that channel of an image at the cursor's location, and modifies the matching part of the character from the input. |
| [US 10,275,947 B2](https://patents.google.com/patent/US10275947B2/en), same title, a continuation of the patent above | Electronic Arts | Granted 30 April 2019. Active. Anticipated expiration 31 March 2035. | Claim 1 selects a modifiable part from the cursor location and the current zoom level, so different zoom levels give different parts at the same spot. It then modifies the part in at least two directions from the cursor's movement. Claims 9 and 15 add the viewing angle. |
| [US 9,626,808 B2](https://patents.google.com/patent/US9626808) and [US 9,892,540 B2](https://patents.google.com/patent/US9892540B2/en), "Image-based deformation of simulated characters of varied topology" | Electronic Arts | Active. Anticipated expiration 15 January 2035 for the continuation. | Displacement stored in images that all clothing items share, with two images for different fits. |
| [US 11,645,813 B2](https://patents.google.com/patent/US11645813B2/en), "Techniques for sculpting digital faces based on anatomical modeling" | Disney Enterprises and ETH Zurich | Granted 9 May 2023. Active. Anticipated expiration 6 July 2040. | A face split into patches with local shape spaces and bone sliding. Sculpting input becomes constraints in an optimisation. |
| [US 2005/0026685 A1](https://patents.google.com/patent/US20050026685A1/en), "Customizing players in a video game using morphing from morph targets" | Electronic Arts | Abandoned in December 2007. Never granted. | A slider that blends between two morph targets, including values beyond the ends ("overdriven" shapes). Its publication is prior art for slider creators. |
| [CN 109785449 A](https://patents.google.com/patent/CN109785449A/en), "Character facial editor and decal method based on FFD" | Suzhou Snail Digital Technology | Application withdrawn in May 2022. | Free-form deformation of a face inside a control lattice, with each control point's movement limited to the space of the facial bone of its region. |

I found no patent from Pearl Abyss, Bandai Namco, Capcom or SEGA on sculpting controls in my searches. These searches were keyword searches on Google Patents and the web. They did not include the Korean (KIPRIS) or Japanese (J-PlatPat) databases, where those companies would most likely file first.

What this means as a caution for kjk:

- EA's two direct-manipulation patents are active until about 2035 to 2036. Their independent claims are narrow and concrete. Claim 1 of US 9,940,757 needs region IDs stored in a colour channel chosen by zoom level. Claim 1 of US 10,275,947 needs the selected part to depend on the zoom level.
- A design that picks regions by ray cast and bone weights, uses one fixed set of regions at every zoom level, and moves parts along the screen in the same way from every view angle, as Black Desert does, has a different structure from those claims. Whether that is enough is a legal question.
- Disney and ETH's patent covers a specific anatomical optimisation. A bone-offset or simple falloff deformer does not use patches, local subspaces or bone sliding.

## 7. Open-source prior art

- **MakeHuman and MPFB.** MPFB is the Blender add-on that continues MakeHuman. It ships 1,258 morph targets ([MPFB, target format](https://github.com/makehumancommunity/mpfb2/blob/9de0707fdc88438d7c53a1a2ac7937d9221e2431/docs/fileformats/target.md)) grouped into 234 signed modifiers ([target.json](https://github.com/makehumancommunity/mpfb2/blob/9de0707fdc88438d7c53a1a2ac7937d9221e2431/src/mpfb/data/targets/target.json)). A target file lists only the vertices it moves, one line per vertex with an index and an offset ([MPFB, target format](https://github.com/makehumancommunity/mpfb2/blob/9de0707fdc88438d7c53a1a2ac7937d9221e2431/docs/fileformats/target.md)). Its `.mhclo` files show how attachments follow a reshaped body (section 4). MPFB has sliders but no drag-to-sculpt tool.
- **CharMorph** is an open-source Blender character creator that uses the MB-Lab base meshes and morphs (secondary source, [LinuxLinks](https://www.linuxlinks.com/charmorph-character-creation-tool-blender/)).
- **SPLOCS** is the reference code for sparse localised deformation components ([tneumann/splocs](https://github.com/tneumann/splocs)).
- **OpenRigLogic** is Epic's open-source library for evaluating MetaHuman rigs and reading DNA files ([EpicGames/openriglogic](https://github.com/EpicGames/openriglogic)).
- **Drag-to-sculpt on a morphable model.** I found no maintained open-source game-style implementation. The closest is "Moldable Face", a student C++ and OpenGL project that applies a blend-shape solve to a dragged vertex (secondary source, [itch.io](https://yidazou.itch.io/moldable-face)). Lewis and Anjyo's formula in section 3.2 is short enough to implement directly. Later papers refine it, for example a transpose-based variant that reduces unexpected movement ([Cetinaslan, Orvalho and Lewis, 2017](https://www.scitepress.org/Papers/2017/61319/)).

## 8. Reading list of the key papers

| Paper | Why it matters for sculpting |
|---|---|
| Blanz and Vetter, "A Morphable Model for the Synthesis of 3D Faces", SIGGRAPH 1999 ([PDF](https://www.cs.princeton.edu/courses/archive/spring11/cos598A/pdfs/Blanz99.pdf)) | Face regions blended independently, attribute directions, and a probability prior that keeps faces plausible. |
| Lewis and Anjyo, "Direct Manipulation Blendshapes", IEEE CG&A 2010 ([preprint](https://www.scribblethink.org/Work/DirectManipBlendshapes/DMBpreprint.pdf)) | The drag-to-weights solve with regularisation and pins. |
| Tena, De la Torre and Matthews, "Interactive Region-Based Linear 3D Face Models", SIGGRAPH 2011 ([Disney Research](https://la.disneyresearch.com/publication/interactive-region-based-linear-3d-face-models/)) | Local PCA regions with shared borders, posed by click and drag. |
| Neumann et al., "Sparse Localized Deformation Components", SIGGRAPH Asia 2013 ([archive](https://history.siggraph.org/?p=208344)) | Automatically found local components for local edits. |
| Gruber et al., "Interactive Sculpting of Digital Faces Using an Anatomical Modeling Paradigm", SGP 2020 ([ETH](https://research-collection.ethz.ch/handle/20.500.11850/432184)) | Sculpting through skin, attributes or bones, with anatomical constraints. |
| Shi et al., "Face-to-Parameter Translation for Game Character Auto-Creation", ICCV 2019 ([ar5iv](https://ar5iv.labs.arxiv.org/html/1909.01064)) | The parameter counts of a shipped Chinese MMO face editor. |
| Sumner and Popović, "Deformation Transfer for Triangle Meshes", SIGGRAPH 2004 ([project](https://people.csail.mit.edu/sumner/research/deftransfer/)) | Moving expressions onto sculpted faces. |

## 9. What this means for kjk

### What KOTOR's heads contain

The technology note left one question open. It asked how many bones KOTOR's heads have and where they sit. For this note I read the heads directly from the KOTOR 1 and KOTOR 2 Steam installs with a throwaway program that uses kjk's own model parser (`crates/fmt_odyssey/src/mdl.rs:188`). The program was not added to the repository.

These are the results.

- **Triangle counts.** The skinned face mesh of a player head has 733 to 893 triangles. The whole head model, with the eyes, eyelids, teeth, tongue and any hair, has 876 to 1,418 triangles. For example, the K1 head `pmhc01` has 958 triangles in total, of which 872 are in the skinned `head` mesh. Bastila's head `p_bastilah` has 1,418 triangles, of which 476 are hair.
- **The same 16 bones on every head.** Every head checked (`pmhc01`, `pfhc01`, `pmha02`, `pfhb03`, `p_carthh` and `p_bastilah` in KOTOR 1, and `pmhc01`, `pfhc01` and `pmhh01` in KOTOR 2) skins its face mesh to the same 16 named bones. The KOTOR 2 head `pmhh01` also lists its own root node as a bone. Three of the 16 are neck and head bones (`necklwr_g`, `neck_g` and `head_g`). The other 13 are face bones.
- **The face bones and their likely roles.** The names and positions suggest these roles. The roles are my reading of the names and positions, not documented facts.
  - `f_mdbrw_g`, `f_lbrw_g` and `f_rbrw_g` sit on the brow line, in the middle, on the left and on the right.
  - `f_llweye_g` and `f_rlweye_g` sit just below the eyes.
  - `f_lns_g` and `f_rns_g` sit about 5 cm to each side of the centre at cheek height. They are probably cheek bones, not nostril bones.
  - `f_um_g` is the upper lip. `f_llm_g` and `f_rlm_g` are the two halves of the lower lip. `f_lmc_g` and `f_rmc_g` are the mouth corners.
  - `f_jaw_g` is the jaw, and `f_tonguetip_g` is the tongue.
- **Separate eye meshes.** The eyeballs (`eyela`, `eyera`) and eyelids (`eyellid`, `eyerlid`) are separate rigid meshes, not part of the skinned face.
- **Symmetry.** In `pmhc01`, the left and right bones mirror each other across the head's centre plane to within about 1 mm. The head mesh is about 16.6 cm wide, 23 cm deep and 28 cm tall.
- **No nose bone.** No bone sits on the nose, the forehead or the skull.
- **Speech keys absolute positions.** The talk animations live in the shared supermodel (`s_female03` for the K1 heads checked). They key both a position and a rotation on all 13 face bones and a rotation on the eyes and eyelids. KOTOR's lip sync picks one of 16 poses of this animation for each mouth shape (`crates/fmt_odyssey/src/lip.rs:1`).

### Design A. Bone-offset sculpting on the original heads

This is the cheapest design that is still real sculpting. It is the Fallout 4 model (section 2) applied to KOTOR's existing bones.

- **Regions.** The player can grab 9 regions with symmetry turned on, or 15 with symmetry turned off. They are the middle of the brow, the brows, the under-eye area, the cheeks, the upper lip, the lower lip, the mouth corners, the jaw, and the eyes. Moving an eye moves the eyeball and eyelid meshes together. The whole head adds one more scale.
- **Picking.** A Bevy ray cast on the head mesh returns the hit triangle and its barycentric coordinates. The creator reads the three vertices' bone weights and picks the face bone with the largest weight. The head stays in its rest pose while sculpting, so the ray cast's lack of skinning does not matter.
- **Drag mapping.** A drag moves the bone in the plane of the screen, as in Black Desert. A held key or the mouse wheel moves it toward or away from the camera. A second mode scales the bone, which widens or narrows the skin around it. For example, scaling `f_jaw_g` sideways widens the chin.
- **Layering under speech.** Because the talk animation keys absolute bone positions, the sculpt cannot change the bones' rest positions. Instead, kjk adds the sculpt offset after it samples the animation, every frame. The mouth then still opens and closes, around the moved position.
- **Limits.** Each region gets a small box of allowed movement, for example 6 mm in each direction for the brows and lips, and a scale range of 85% to 115%. The creator slows the drag near the edge of the box and tints the region. The tint idea comes from The Sims 4, so the exact behaviour needs the legal check in section 6.
- **Symmetry.** Left and right bones are paired by name (`f_l…` and `f_r…`). The mirrored side receives the same offset with the left-right component negated.
- **What it cannot do.** It cannot reshape the nose, the forehead or the skull, because no bone sits there. The bones' skin weights were painted to make the mouth speak, not to shape the face. Some regions will therefore move as soft, broad patches. That may look fine on a 900-triangle head, but this has to be tested on real heads before committing.
- **Cost.** I estimate about one to two weeks of work. This covers the picking, the drag, the offsets layered after animation, limits, symmetry, undo, and storage. No new art is needed.

### Design B. A soft deformation field baked into the head

This design adds the missing regions without new art. Each region is a centre, a radius and a smooth falloff in head space. A drag moves or scales the vertices inside the radius, with the full effect at the centre and none at the edge. This is the "soft selection" or "cluster" idea that the animators in Lewis and Anjyo's study compared their tool with ([preprint](https://www.scribblethink.org/Work/DirectManipBlendshapes/DMBpreprint.pdf)).

- **Regions on every head without per-head work.** All heads share the 16 named bones. Each region's centre can therefore be placed relative to the bones. The nose tip, for example, is the most forward vertex on the centre plane between the eye bones and `f_um_g`. The forehead sits above `f_mdbrw_g`. The radius scales with the head's width. The same rules then work on every one of KOTOR's heads.
- **Baking.** When the look changes, kjk moves the head's rest vertices on the CPU and recomputes the normals. A head has about 700 to 900 vertices, so this costs well under a millisecond. The skin weights stay as they are, so speech keeps working.
- **Attachments for free.** The field is a formula in head space. kjk can apply the same formula to hair meshes and to anything hung on the `gogglehook` or `maskhook` nodes (`crates/asset_odyssey/src/hooks.rs:11`). This replaces a wrap deformer.
- **Neck seam.** The field must be zero near the neck, or the head would separate from the body at the seam. The neck bones mark where that is.
- **Risks.** KOTOR's skin textures have shading painted in. Moving a cheek moves its painted highlight with it, which usually helps, but stretching it too far blurs it. The noses have few vertices, so a strong nose edit will look faceted. Both risks argue for small limits.
- **Cost.** I estimate about two to three more weeks on top of design A. Most of that time goes into tuning the region rules and limits across all the heads.

### Design C. New heads with blend shapes

This is the route to a modern sculpting creator. One new base head per gender would share one topology. The KOTOR heads would be refitted to it as presets. Then 30 to 60 signed blend shapes could be authored in the MPFB style, and a Lewis and Anjyo solve could map drags onto them. This needs a 3D artist for each base head, its shapes, its levels of detail and its 16 talk poses, plus refitted hair and headgear. It is the most expensive option by a wide margin, as the technology note already concludes.

### The cheapest workable plan

1. Build design A first. It proves the picking, the drag, the limits and the storage, and it gives players nine real sculpt regions.
2. Add design B's field for the nose, forehead, cheekbones and chin only if design A's results look too limited on real heads.
3. Keep sliders next to the drag for every region, as Black Desert and inZOI do. A slider is also the only good input for a controller.

### What to store and sync in co-op

Sculpting fits the `Look` block that the technology note proposes for `Persona` (`crates/sim/src/persona.rs:22`). Each region becomes a few numbered sliders, one for each axis and one for scale. Each slider is one byte with 128 as neutral.

| Item | Count | Size |
|---|---|---|
| Design A regions (9 with symmetry, 15 without) | 15 regions × 4 values (x, y, z and scale) | 60 sliders, 120 bytes as slider id and value pairs |
| Design B regions | About 6 more regions × 4 values | 24 sliders, 48 bytes |
| Symmetry flag | 1 | 1 byte |

A fully sculpted asymmetric head therefore needs about 170 bytes. A symmetric one needs about half that, because the mirrored side is not stored. Untouched regions are not stored at all. With four players, the total stays under 1 KB. As the technology note proposes, the host sends it only when a look changes, not in every frame.

Three rules from the sections above apply.

- **Store parameters, never meshes.** Every client rebuilds the same head from the same bytes and the same game files. This keeps share codes short. It also means a change to the region rules in a later kjk version changes old faces, so the region rules need a version number in the look block.
- **Clamp on load.** A loader clamps every value to the current limits. A share code from another version or a hand-edited file then cannot produce a broken face.
- **Keep the sim blind to the look.** The sculpt changes only drawing. The sim must not read it, so two clients that draw a face slightly differently can never desync.
