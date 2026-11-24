# Character creation technology in games

This document surveys how current games build character creators, from the face and body shapes down to the bytes a game saves and sends over the network. The last section turns the survey into options for kjk.

Each claim carries a link to its source. Where the only source is a press article, a fan wiki or a forum, the text says so.

## Terms used in this document

A **mesh** is the set of points (vertices) and triangles that make up a 3D model.

A **morph target**, also called a **blend shape** or **shape key**, is a copy of a mesh in which some vertices have moved. The engine stores the difference (the "delta") between each moved vertex and its rest position. A weight between 0 and 1 says how much of that difference to add. The glTF 2.0 file format stores morph targets as per-vertex position, normal and tangent differences, and a mesh's default weights in a `weights` array ([Khronos glTF 2.0 specification, Morph Targets](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#morph-targets)).

**Skinning** moves a mesh's vertices with a skeleton. Each vertex is tied to up to four bones (also called **joints**) with a weight for each. When a bone moves, the vertices tied to it follow.

A **bone-driven face** is a face whose shape and expressions come from moving many small bones in the head, rather than from morph targets.

A **corrective shape** is a morph target that is not chosen by the player or the animator directly. The rig turns it on by itself when the skeleton reaches a given pose, to fix a shape that skinning alone gets wrong (for example, a collapsing elbow).

**PCA** (principal component analysis) is a statistical method. Given many scanned faces with the same vertex layout, it finds the directions in which faces most often differ from the average face. Each direction becomes one morph target, sorted from the most important to the least important.

A **draw call** is one request from the engine to the graphics card to draw one mesh with one material. Every separate mesh part and every separate material on a character adds at least one draw call.

## Face shaping

### Plain morph targets

Most character creators start with morph targets. An artist sculpts a version of the head with, for example, a longer nose. A slider then sets that morph target's weight. Unity exposes this as `SkinnedMeshRenderer.SetBlendShapeWeight`, which sets one blend shape's weight by index on one renderer ([Unity Scripting API](https://docs.unity3d.com/2023.2/Documentation/ScriptReference/SkinnedMeshRenderer.SetBlendShapeWeight.html)).

Morph targets require every target to share the base mesh's vertex count and vertex order. Unity's documentation describes a blend shape as a morph from the original mesh "to a target BlendShape" with the same topology ([Unity Scripting API](https://docs.unity3d.com/2023.2/Documentation/ScriptReference/SkinnedMeshRenderer.SetBlendShapeWeight.html)). This rule has a direct effect on kjk, as the last section explains.

EVE Online's 2010 character creator is a well-documented example of a morph-based face. CCP's developer blog says each face has 92 morph targets, and the body uses 20 more ([EVE Online dev blog, "Character Creation", CCP Ph00ze, December 2010](https://www.eveonline.com/news/view/character-creation-3)). The same blog says each head has about 4,500 vertices at its highest detail level, and that all faces share the same vertex counts so that they stay compatible with each other ([EVE Online dev blog](https://www.eveonline.com/news/view/character-creation-3)). EVE ships 24 base models, one per bloodline, race and gender combination ([EVE Online, "Introducing our new character creator", October 2010](https://www.eveonline.com/article/introducing-our-new-character-creator)).

### Statistical face spaces (FaceGen)

FaceGen, made by Singular Inversions, builds its morph targets with PCA from scanned faces. In FaceGen, a face is a "face coordinate" of 130 signed floating point numbers ([FaceGen SDK manual, Overview](https://facegen.com/dl/sdk/doc/manual/overview.html)). Each number is measured in standard deviations from the average face and usually lies between -5 and 5 ([FaceGen SDK manual, Overview](https://facegen.com/dl/sdk/doc/manual/overview.html)). The 130 numbers split as follows ([FaceGen SDK manual, Overview](https://facegen.com/dl/sdk/doc/manual/overview.html)):

- 50 numbers describe the symmetric shape of the face.
- 30 numbers describe how the face is asymmetric.
- 50 numbers describe the skin colour, which is always symmetric.

Each shape mode is a per-vertex displacement. The manual says "the modes are one standard deviation 3D displacements for each vertex" ([FaceGen SDK manual, In Depth](https://facegen.com/dl/sdk/doc/manual/indepth.html)). The final vertex position is the average face's position plus the sum of every mode's displacement times its coordinate ([FaceGen SDK manual, In Depth](https://facegen.com/dl/sdk/doc/manual/indepth.html)). In other words, a FaceGen face is 80 morph targets with weights chosen by statistics.

The sliders a player sees are not the 130 numbers. FaceGen defines each slider as a "linear control", which is a fixed direction through the 130-number space ([FaceGen SDK manual, In Depth](https://facegen.com/dl/sdk/doc/manual/indepth.html)). The slider's value is the face coordinate projected onto that direction. Moving the slider adds the direction, scaled by the change, to the face coordinate. Because these directions are not independent, the manual warns that changing one control "will affect all values" ([FaceGen SDK manual, In Depth](https://facegen.com/dl/sdk/doc/manual/indepth.html)). Age is measured in years. Gender runs from -1 (male) to +1 (female). Race controls are offsets between the average faces of different groups ([FaceGen SDK manual, In Depth](https://facegen.com/dl/sdk/doc/manual/indepth.html)).

FaceGen stores the modes "in decreasing order of their magnitudes". A game with less memory can drop modes from the end and still keep the best approximation for the modes it keeps ([FaceGen SDK manual, In Depth](https://facegen.com/dl/sdk/doc/manual/indepth.html)).

FaceGen fits hair, beards and other accessories by giving each accessory its own statistical shape model. The same coefficients are applied to the head and to the accessory, so they deform together ([FaceGen SDK manual, In Depth](https://facegen.com/dl/sdk/doc/manual/indepth.html)). Skin colour is a colour map built from the colour modes and then multiplied by a "detail texture" stored in a cylindrical layout. The detail texture covers only the face area ([FaceGen SDK manual, In Depth](https://facegen.com/dl/sdk/doc/manual/indepth.html)).

The Elder Scrolls IV: Oblivion used the FaceGen SDK. The fan wiki UESP states this ([UESP, Bethesda Softworks](https://en.uesp.net/wiki/General:Bethesda_Softworks)). I found no Bethesda or Singular Inversions page that lists the games using FaceGen, so the claim that Fallout 3 and Fallout: New Vegas used it rests on secondary sources only.

### Bone-driven and hybrid faces (MetaHuman, DNA and RigLogic)

Epic's MetaHuman system stores each character in a DNA file. Epic says DNA "encodes all information necessary to assemble and configure a full MetaHuman head, body and its rigs" ([Epic, MetaHuman DNA](https://dev.epicgames.com/documentation/metahuman/metahuman-dna-rig-definition-and-rig-operation)). A DNA file holds the following ([Epic, MetaHuman DNA](https://dev.epicgames.com/documentation/metahuman/metahuman-dna-rig-definition-and-rig-operation)):

- The head and body meshes and the skeleton, in their neutral pose.
- Eight levels of detail, from LOD 0 (the most detailed) to LOD 7.
- Skin weights for each level of detail.
- One joint animation for each facial expression.
- Per-vertex displacements, used only at LOD 0.

RigLogic is the runtime that evaluates a DNA file. Epic's documentation says it takes "hundreds of semantically meaningful channels" (for example, "raise the inner left brow") and turns them into joint transforms, per-vertex displacements at LOD 0, and extra parameters such as wrinkle map triggers ([Epic, MetaHuman DNA](https://dev.epicgames.com/documentation/metahuman/metahuman-dna-rig-definition-and-rig-operation)). The Unreal Engine API reference describes the RigLogic module's outputs as bone transforms, blend shape weights and "animated map" multipliers ([Unreal Engine 4.27 API, RigLogic module](https://docs.unrealengine.com/4.27/API/Plugins/RigLogicModule/index.html)).

MetaHuman is therefore a hybrid. Joints carry most of the motion, and morph targets add detail on top. Epic explains why the joints come first. Its documentation says "per-vertex shape interpolation is linear and therefore looks artificial when moving large masses" ([Epic, MetaHuman DNA](https://dev.epicgames.com/documentation/metahuman/metahuman-dna-rig-definition-and-rig-operation)). Joints rotate, so they can move a jaw along an arc. A morph target can only move each vertex along a straight line.

One joint set drives all eight levels of detail, and the lower levels leave some joints out. The vertex displacement layer is "completely ignored in lower LODs" ([Epic, MetaHuman DNA](https://dev.epicgames.com/documentation/metahuman/metahuman-dna-rig-definition-and-rig-operation)). This is a common way to scale cost with distance. Morph targets are only evaluated on the closest, most detailed version of the head.

Epic released RigLogic and the DNA library as open source. The MetaHuman 5.8 release notes say the OpenRigLogic repository contains "the RigLogic and DNA libraries released under an MIT license" ([Epic, MetaHuman 5.8 release notes](https://dev.epicgames.com/documentation/metahuman/metahuman-5-8-release-notes-in-unreal-engine); repository at [github.com/EpicGames/OpenRigLogic](https://github.com/EpicGames/OpenRigLogic)). These are C++ libraries. I found no Rust binding.

MetaHuman Creator shapes a face by blending presets rather than by moving dozens of sliders. Its Blend tool mixes up to three selected presets. The blend can apply to the whole face or to one region ([Epic, MetaHuman Creator Blend tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-blend-tool-in-unreal-engine)). The user drags a control point inside a circle, and each preset's influence grows as the point gets closer to it ([Epic, MetaHuman Creator Blend tool](https://dev.epicgames.com/documentation/metahuman/metahuman-creator-blend-tool-in-unreal-engine)).

### Corrective shapes

The founding paper for corrective shapes is "Pose Space Deformation" by J. P. Lewis, Matt Cordner and Nickson Fong, published at SIGGRAPH 2000 ([Lewis, Cordner and Fong, PSD paper](https://www.scribblethink.org/Work/PSD/PSD.pdf)). An artist sculpts the correct shape at a few key poses. The method then interpolates the corrections between those poses with radial basis functions ([Lewis, Cordner and Fong, PSD paper](https://www.scribblethink.org/Work/PSD/PSD.pdf)). Modern rigs use the same idea. MetaHuman's neck, for example, uses an RBF solver driven by three neck joints ([Epic, updating the neck RBF setup](https://dev.epicgames.com/documentation/metahuman/updating-neck-rbf-setup-tool-drivers-for-expression-editor)).

Corrective shapes matter for a character creator in one specific case. When two creation sliders both move the same vertices (for example, "wide jaw" and "strong chin"), their sum can look wrong. A corrective shape that turns on when both sliders are high can fix that sum.

### Scanned base heads and joint offsets (Dragon's Dogma 2 on RE Engine)

Capcom's Dragon's Dogma 2 combines blend shapes, joint edits and scanned heads. An Autodesk article with Capcom's technical art staff gives these facts ([Autodesk, "Behind Capcom's In-House RE ENGINE Tools For Dragon's Dogma 2", April 2025](https://blogs.autodesk.com/media-and-entertainment/2025/04/10/ideas-and-ingenuity-behind-capcoms-in-house-re-engine-tools-for-dragons-dogma-2/)):

- The team scanned about 80 people's heads to make variations on a base head.
- All variations share one topology, so the blend shapes are built as one set.
- Players can also move joints in each part of the face.
- The body sliders and joints are set up in Maya. A calibrated editor exports the slider setup from Maya to RE Engine, so developers use the same sliders in both.
- Because players can move the sliders freely, the team "set up dependencies and limits between the sliders" to prevent unrealistic builds.
- The game has over 1,000 characters, and all of them were built with the same character editing tools as the player.

### Presets only (Baldur's Gate 3)

Larian chose not to give Baldur's Gate 3 face sliders. Lead character artist Alena Dubrovina said "there is a risk of faces being a bit blank and characterless if the character creator is heavily dependent on the sliders" ([Game Developer, "Building an inclusive character creator for the fantasy world of Baldur's Gate 3"](https://gamedeveloper.com/art/building-an-inclusive-character-creator-for-the-fantasy-world-of-baldur-s-gate-3)). The faces come from a scanning pipeline, and she noted that "small millimeter changes can make or break a face completely" ([Game Developer](https://gamedeveloper.com/art/building-an-inclusive-character-creator-for-the-fantasy-world-of-baldur-s-gate-3)). The player picks a premade head, then picks colours and features such as hair, makeup and tattoos.

### Direct sculpting

Some creators let the player pull on the face itself instead of moving sliders. EVE Online's creator has a sculpt mode in which the player "grab[s] onto the nose and move[s] the mouse to make it bigger" ([EVE Online, "Introducing our new character creator"](https://www.eveonline.com/article/introducing-our-new-character-creator)). The technical blog explains how it works ([EVE Online dev blog, "Character Creation"](https://www.eveonline.com/news/view/character-creation-3)):

- The engine finds where the player clicked from the mesh's UV coordinates.
- Each face region holds a small 2D field of triangles. Each corner of the field stores a combination of morph weights.
- The mouse drag moves a point inside that field, and the engine interpolates the morph weights at that point.

Direct sculpting is therefore still morph targets under the hood. Only the input method changes. The Sims 4 also uses direct manipulation, in which players pull and push the model ([GDC Vault, "Innovations in The Sims 4 Character Creator", Sri Nair, GDC 2015](https://www.gdcvault.com/play/1022085/Innovations-in-The-Sims-4)).

Black Desert Online is often named as the best example of a detailed sculpting creator. I found no technical source from Pearl Abyss on how it works. Its official wiki only documents saving and sharing appearance files (see "Data representation" below).

## Bodies and clothing

### Body types as presets

Many games offer a few premade body types instead of continuous body sliders. Baldur's Gate 3 offers four body types per race, and press coverage notes there is no way to make a heavy character ([PC Gamer](https://www.pcgamer.com/baldurs-gate-3-character-creation-deep-dive-reveals-tailed-dragonborn-flashy-class-intros-and-bespoke-genitals); secondary source). Cyberpunk 2077 offers two body types and no weight or frame sliders ([PC Gamer](https://pcgamer.com/cyberpunk-2077-character-creator-drops-male-and-female-options-in-favor-of-body-type); secondary source). I found no CD Projekt Red technical source on how Cyberpunk 2077's clothing handles the two bodies.

Presets keep clothing cheap. Every armour piece needs one version per body type, and an artist can check and fix each one by hand.

### Separate armour per body type

Dragon's Dogma 2 shows the cost of this approach at scale. Capcom first wanted one universal equipment mesh, but it did not work because of positional conflicts ([Autodesk article on Dragon's Dogma 2](https://blogs.autodesk.com/media-and-entertainment/2025/04/10/ideas-and-ingenuity-behind-capcoms-in-house-re-engine-tools-for-dragons-dogma-2/)). The team made separate meshes for beastren and humans, and for male and female bodies. Beastren armour hides the fur with a mask shader. The team prepared "close to 300" equipment variations ([Autodesk article](https://blogs.autodesk.com/media-and-entertainment/2025/04/10/ideas-and-ingenuity-behind-capcoms-in-house-re-engine-tools-for-dragons-dogma-2/)).

### Shape transfer to clothing (wrap deformers)

When the body can take any shape, clothing has to follow it. The usual tool is a **wrap deformer**. Each clothing vertex is bound to the nearest face of the base body. When the body changes shape, each clothing vertex moves with the face it is bound to.

- Dragon's Dogma 2 "implemented Maya's wrap deformer into the engine" for beards, eyebrows and helmets. The wrap deformer's falloff map becomes a vertex colour in the engine. Different vertex colours mark rigid parts (helmets) and soft parts (some hair) ([Autodesk article](https://blogs.autodesk.com/media-and-entertainment/2025/04/10/ideas-and-ingenuity-behind-capcoms-in-house-re-engine-tools-for-dragons-dogma-2/)).
- Unreal's Mutable plugin has a "Skeletal Mesh Reshape" node. It binds each vertex of a mesh to the faces of a base shape by proximity, then moves it to the matching faces of a target shape. It can also move the skeleton's joints and the physics bodies the same way. A vertex colour channel can mark "rigid clusters" that move as one piece ([Mutable documentation, Skeletal Mesh Reshape](https://github.com/anticto/Mutable-Documentation/wiki/Skeletal-Mesh-Reshape)).
- EVE Online's glasses "inherit the morphs of the faces without distorting them" ([EVE Online dev blog](https://www.eveonline.com/news/view/character-creation-3)).

### Deformation maps (The Sims 4)

The Sims 4 adds a third way to change a body's shape, next to morph targets and bones. Its GDC 2015 talk describes "deformation maps" and explains when they work better than morph geometry or bone changes ([GDC Vault, "Innovations in The Sims 4 Character Creator"](https://www.gdcvault.com/play/1022085/Innovations-in-The-Sims-4)). A deformation map is a texture that stores a displacement for each point of the body's surface. Any clothing mesh that has a second UV set mapping it onto that surface can read the same texture and deform with the body. The talk abstract confirms the method exists. The details of the texture and UV setup come from the modding community only ([Sims 4 Studio forum, UV mapping thread](https://sims4studio.com/thread/19464/uv-mapping); secondary source). The main benefit is that one body slider can reshape thousands of clothing items without a morph target authored for each item.

The same talk describes "asset pairing". Items that occupy the same place on the body are paired, with a rule saying which one is drawn over the other. This lets players mix parts without the meshes poking through each other ([GDC Vault](https://www.gdcvault.com/play/1022085/Innovations-in-The-Sims-4)).

### Height and the skeleton

Changing a character's height with morph targets breaks skinning. The Mutable documentation explains that bones are not affected by morphs, so after a morph the vertices can sit where the bones no longer match, and the mesh deforms badly. It says that using morph targets to change height "is discouraged" ([Mutable documentation, FAQ](https://github.com/anticto/Mutable-Documentation/wiki/FAQ)). It recommends reshaping the skeleton with the mesh, or scaling bones after the character is built ([Mutable documentation, FAQ](https://github.com/anticto/Mutable-Documentation/wiki/FAQ)).

Dragon's Dogma 2 uses both sliders and joints for its bodies, and its height and weight also change how the character plays ([GamesRadar](https://gamesradar.com/why-is-dragons-dogma-2s-character-creator-so-good-its-director-says-the-rpgs-team-scanned-nearly-80-faces-for-base-head-variations-for-starters); secondary source).

The simplest form of this is to scale or offset a few bones (spine, legs, arms, neck) and keep the animations unchanged. Animations that store bone rotations, and not bone positions, keep working on the scaled skeleton. Foot placement and hand contacts then need correction by inverse kinematics.

## Modular meshes and draw calls

A modular character is made of separate skinned meshes (head, torso, legs, hands, hair) that all follow one skeleton. Epic documents three ways to do this in Unreal, with this comparison ([Epic, Working with Modular Characters](https://dev.epicgames.com/documentation/en-us/unreal-engine/working-with-modular-characters-in-unreal-engine)):

| Method | Setup cost | Game thread cost | Render thread cost | Morph targets |
|---|---|---|---|---|
| Leader Pose Component | Minimal | Minimal | High | Yes |
| Copy Pose from Mesh | Medium | High | High | Yes |
| Skeletal Mesh Merge | High | Medium | Low | No |

- With Leader Pose, each part reuses the leader's bone transforms. Each part still costs its own draw calls ([Epic, Working with Modular Characters](https://dev.epicgames.com/documentation/en-us/unreal-engine/working-with-modular-characters-in-unreal-engine)).
- With Copy Pose from Mesh, each part runs its own animation graph. It copies only the bones that match by name, and the other bones keep their reference pose ([Epic](https://dev.epicgames.com/documentation/en-us/unreal-engine/working-with-modular-characters-in-unreal-engine)).
- With Skeletal Mesh Merge, the parts become one mesh. Epic's example is 50 characters of 3 parts each. Unmerged, they cost 150 draw calls. Merged, they cost 50 ([Epic](https://dev.epicgames.com/documentation/en-us/unreal-engine/working-with-modular-characters-in-unreal-engine)). The merged mesh cannot carry the parts' morph targets ([Epic](https://dev.epicgames.com/documentation/en-us/unreal-engine/working-with-modular-characters-in-unreal-engine)).

Unreal's Mutable plugin automates this whole pipeline. It "generates Skeletal Meshes, Materials, and Textures dynamically in the editor or at runtime" ([Mutable documentation, Home](https://github.com/anticto/Mutable-Documentation/wiki/)). It can do the following:

- It can merge parts and remove hidden geometry, for example the skin under a shirt. Its node list includes "Skeletal Mesh Merge" and "Remove Mesh Blocks" ([Mutable documentation, Home](https://github.com/anticto/Mutable-Documentation/wiki/)).
- It can pack the textures of several parts into one texture at runtime and rewrite the mesh UVs to match. It can reorder blocks, remove unused blocks and add new ones. The documentation warns that texture packing "is costly" and should be used only when it is needed, for example to reduce draw calls or texture size ([Mutable documentation, Texture Packing](https://github.com/anticto/Mutable-Documentation/wiki/Texture-Packing)).
- It bakes morphs into the generated mesh by default. Morph targets that must stay live at runtime are a separate option, disabled by default ([Mutable documentation, Real-Time Morphs](https://github.com/anticto/Mutable-Documentation/wiki/Real%E2%80%90Time-Morphs)).
- Its work runs on worker threads. Its working memory peaks at roughly twice the size of the largest texture (uncompressed) or skeletal mesh it generates ([Mutable documentation, Resource Usage](https://github.com/anticto/Mutable-Documentation/wiki/Resource-Usage)).

## Materials

### Skin colour, makeup and tattoos

Most creators change skin colour, makeup and tattoos with textures and shader parameters, not with new meshes. There are two ways to do it:

1. **Layering in the shader.** The material reads a base texture and several masks, and blends tints and decals every frame. This costs a few texture reads per pixel, every frame, for every character.
2. **Baking.** The game composites the layers once into a new texture when the character is created or loaded, and the shader reads only that texture. This costs memory for one texture per character, and nothing extra per frame.

MetaHuman's export pipeline bakes. Its baked face material has one base colour texture that holds "skin texture, skin accents, make-up, freckles, etc." ([Epic, MetaHuman materials and textures](https://dev.epicgames.com/documentation/metahuman/metahuman-materials-and-textures?lang=en-US)). The material still exposes global adjustments to base colour, specular, roughness, scatter and normal. Its wrinkle maps ("animated maps") are stored as differences from the base map ([Epic, MetaHuman materials and textures](https://dev.epicgames.com/documentation/metahuman/metahuman-materials-and-textures?lang=en-US)). The same page lists its texture sizes and its material levels of detail ([Epic](https://dev.epicgames.com/documentation/metahuman/metahuman-materials-and-textures?lang=en-US)):

- The "UE Optimized" export uses 2K textures at most. The "UE Cine" export bakes normal and specular maps up to 8K.
- LOD 1 turns off micro skin detail normals.
- LOD 2 turns off the wrinkle maps.
- LOD 4 and lower switch to a simple shading model.
- Subsurface scattering is off at the Medium and Low material quality settings.

Bungie's Destiny is the standard example of layered dyes. Bungie's GDC 2017 talk "Destiny Shader Pipeline" covers the shader system behind its gear customisation ([Advances in Real-Time Rendering, Destiny Shader Pipeline, GDC 2017](https://advances.realtimerendering.com/destiny/gdc_2017/index.html)). Press reports of an earlier GDC 2014 talk say each piece of gear has a dye for its armour, cloth and suit areas, and that a dye sets colour, detail maps and specular properties ([GamingBolt](https://gamingbolt.com/bungies-destiny-character-customization-system-detailed-features-decal-system); secondary source).

### Skin shading

Realistic skin needs subsurface scattering, which is light that enters the skin and comes out nearby. The common real-time method is "Separable Subsurface Scattering" by Jorge Jimenez and colleagues. It blurs the lit image in screen space with two passes and seven samples per pixel, instead of the twelve passes of the older sum-of-Gaussians method ([Jimenez et al., Separable Subsurface Scattering, Computer Graphics Forum 2015](https://graphics.unizar.es/papers/Separable-Subsurface-Scattering.pdf)).

### Eyes

Eyes are their own material. MetaHuman's eye materials offer customisation of the sclera, cornea, iris and pupil at runtime, with separate colour and normal textures for the sclera and the iris ([Epic, MetaHuman materials and textures](https://dev.epicgames.com/documentation/metahuman/metahuman-materials-and-textures?lang=en-US)). For a creator, this means eye colour is usually a shader parameter (an iris tint), not a texture swap.

### Hair

Games draw hair in one of two ways:

- **Hair cards** are flat strips of triangles with a hair texture on them. They are cheap. EA's Frostbite team notes that reaching high quality with cards "can be extremely time consuming for the content creators" ([EA Frostbite, "Frostbite Hair Rendering and Simulation, Part 2"](https://www.ea.com/frostbite/news/frostbite-hair-rendering-and-simulation-2)).
- **Strands** are thousands of individual curves, simulated and drawn one by one.

EA's strand hair in Dragon Age: The Veilguard uses 50,000 strands per character over more than 100 hairstyles ([EA, "Strand hair in Dragon Age: The Veilguard"](https://www.ea.com/technology/news/strand-hair-dragon-age-the-veilguard)). The team raised the maximum strand length from 63 points to 255 points. It draws the hair with "a bespoke compute software rasterizer" and composites it into the frame ([EA](https://www.ea.com/technology/news/strand-hair-dragon-age-the-veilguard)). With eight hair assets on screen, its console budget is 6.5 ms per frame at 30 frames per second and 3 ms at 60 frames per second ([EA](https://www.ea.com/technology/news/strand-hair-dragon-age-the-veilguard)). On low PC settings and on Xbox Series S, the game swaps strand hair for card hair ([EA](https://www.ea.com/technology/news/strand-hair-dragon-age-the-veilguard)).

Unreal's Groom system supports both. Each level of detail of a groom can be strands, cards or a mesh, chosen by screen size ([Epic, Setting up level of detail for grooms](https://dev.epicgames.com/documentation/unreal-engine/setting-up-level-of-detail-for-grooms-in-unreal-engine)). Unreal also has a Hair Card Generator plugin that builds cards from a strand groom ([Epic, Hair Card Generator](https://dev.epicgames.com/documentation/en-us/unreal-engine/hair-card-generator-for-grooms-in-unreal-engine)).

Larian outsourced many Baldur's Gate 3 hairstyles to external partners, and noted that coiled hair "requires more volume" than straight hair ([Game Developer](https://gamedeveloper.com/art/building-an-inclusive-character-creator-for-the-fantasy-world-of-baldur-s-gate-3)). Hair is often the most expensive item in a creator to produce.

## Data representation, sharing and versioning

Every system above stores a character the same way. It stores the choices, not the meshes. The game rebuilds the meshes and textures from the choices when it loads the character.

- FaceGen stores a face as 130 floating point numbers ([FaceGen SDK manual, Overview](https://facegen.com/dl/sdk/doc/manual/overview.html)).
- Mutable stores a character as a "Customizable Object Instance", which "contains only a reference to a Customizable Object and the value for each of its parameters" ([Mutable documentation, Replication](https://github.com/anticto/Mutable-Documentation/wiki/Replication)). Its parameter types include float, colour, enum (a choice from a list), texture, projector (a decal placed on the body) and skeletal mesh ([Mutable documentation, Home](https://github.com/anticto/Mutable-Documentation/wiki/)).
- MetaHuman Creator stores a DNA file, which holds the full rig and meshes, not only parameters ([Epic, MetaHuman DNA](https://dev.epicgames.com/documentation/metahuman/metahuman-dna-rig-definition-and-rig-operation)). It is the exception, because MetaHuman characters are exported as assets for a game, not made by players at runtime.

### Sharing files and codes

- Black Desert Online saves an appearance to a file from its customisation screen and can load another player's file. Its official wiki says the player can "save the appearance applied to the character in a file format, or share it with other Adventurers" ([Black Desert official wiki](https://www.naeu.playblackdesert.com/Wiki?wikiNo=5)). The game also has a "Beauty Album" where players upload and download appearances in game ([Black Desert Mobile official wiki](https://www.world.blackdesertm.com/Ocean/Wiki?wikiNo=34)).
- Capcom released a free "Character Creator & Storage" tool for Dragon's Dogma 2 before launch, on Steam, PlayStation 5 and Xbox Series X|S ([Steam store page](https://store.steampowered.com/app/2674810)). Players could store up to five Arisen and five Pawns and import one of each into the full game ([Video Games Chronicle](https://videogameschronicle.com/news/capcom-has-released-a-dragons-dogma-2-character-creator-tool); secondary source).
- Monster Hunter Wilds let players carry their beta character into the full game through a "Beta Test Data" option in the character creator, with the data held on Capcom's servers ([OpenCritic news](https://opencritic.com/news/13132/monster-hunter-wilds-reveals-how-to-transfer-open-beta-character-data-to-full-game); secondary source).
- The Sims 4 shares Sims through "tray" files in the Gallery. I found no EA technical source on the tray file format.

All of these work only because the file holds parameters. A parameter file is small and stays valid as long as the game still has the assets it names.

### Versioning

None of the sources above publish their versioning rules in detail. These are the methods that the documented designs make possible:

- FaceGen's ordered modes allow a game to keep fewer modes and still read the same face coordinates ([FaceGen SDK manual, In Depth](https://facegen.com/dl/sdk/doc/manual/indepth.html)).
- Mutable names its parameters, and its descriptor can be saved in a "compact" form ([Mutable documentation, Replication](https://github.com/anticto/Mutable-Documentation/wiki/Replication)). The compact form is smaller, and the full form keeps names, which survive reordering.

In general, a saved character should carry a format version and should name its sliders by a stable identifier, not by position. A loader should give a default value to any slider the file does not mention, and ignore any slider the game no longer has. A preset should be referenced by a stable identifier, not by its row in a list that may grow.

### Network replication

Mutable's documentation gives the general rule: "The simplest way to replicate a customization is to send the set of parameter values plus which Customizable Object they should be applied to" ([Mutable documentation, Replication](https://github.com/anticto/Mutable-Documentation/wiki/Replication)). It provides `SaveDescriptor` and `LoadDescriptor` functions for this. Each client then builds the meshes and textures itself.

A character's look changes rarely. It is sent once when a player joins, and again only when the player edits it at a mirror or a barber. It does not need to be in every frame's snapshot.

## Performance

### Where morph targets are evaluated

There are two places to apply morph targets:

1. **Every frame, on the GPU.** The vertex shader adds each morph target's delta times its weight. This is needed for morphs that animate, such as facial expressions.
2. **Once, at load.** The game adds the deltas to the vertices on the CPU and keeps the result as a plain mesh. This suits creation sliders, which do not change during play.

Unreal evaluates morph targets on the GPU by default. A community reference of Unreal's console variables lists `r.MorphTarget.Mode` with "1: Enable GPU method (default)" and "0: Use original CPU method" ([UE console variable reference](https://indxzero.github.io/ue544cvarwiki/articles/r.morphtarget.mode); secondary source). Mutable bakes creation morphs into the generated mesh by default ([Mutable documentation, Real-Time Morphs](https://github.com/anticto/Mutable-Documentation/wiki/Real%E2%80%90Time-Morphs)).

MetaHuman applies its vertex displacements only at LOD 0 ([Epic, MetaHuman DNA](https://dev.epicgames.com/documentation/metahuman/metahuman-dna-rig-definition-and-rig-operation)). This keeps the per-frame morph cost to the one or two characters close to the camera.

### Memory

A morph target costs memory in proportion to the number of vertices it moves. Engines that store morph targets densely (one delta for every vertex of the mesh, even unmoved ones) pay for every vertex. Bevy stores them densely. Each vertex of each target takes 48 bytes there, as the next section explains. A dense 2,000-vertex head with 40 targets would take 2,000 × 40 × 48 bytes, which is about 3.8 MB. This is an example figure, not a measurement of any KOTOR head.

Baked textures cost memory per character. EVE Online's highest setting uses an uncompressed 4096 × 2048 texture per character ([EVE Online dev blog](https://www.eveonline.com/news/view/character-creation-3)).

## Bevy and Rust specifics

kjk uses Bevy 0.19.1 (`Cargo.toml` in the kjk repository). The facts below come from the Bevy 0.19.1 source code.

### Morph targets in Bevy

- A `Mesh` holds morph targets as a list of `MorphAttributes` values, set with `Mesh::set_morph_targets` ([bevy_mesh mesh.rs](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_mesh/src/mesh.rs#L2383)). Each `MorphAttributes` holds a position delta, a normal delta and a tangent delta, each padded to 16 bytes, so each one takes 48 bytes ([bevy_mesh morph.rs](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_mesh/src/morph.rs)).
- The `MorphWeights` component holds the weights. It is "the source of truth" for the meshes under it. A mesh can also point to another entity's `MorphWeights` with `MeshMorphWeights::Reference`, so several meshes can share one set of weights ([bevy_mesh morph.rs](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_mesh/src/morph.rs)).
- A mesh can have at most 256 morph targets (`MAX_MORPH_WEIGHTS`) ([bevy_mesh morph.rs, line 15](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_mesh/src/morph.rs#L15)). Older Bevy versions allowed only 64 ([docs.rs, Bevy 0.14 MorphWeights](https://docs.rs/bevy/0.14.2/bevy/render/prelude/struct.MorphWeights.html)).
- On platforms with storage buffers, Bevy reads the deltas from a storage buffer. On platforms without them, it reads them from a 3D texture of at most 2048 × 2048 texels per layer ([bevy_pbr morph.wgsl](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/morph.wgsl); [bevy_mesh morph.rs](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_mesh/src/morph.rs)).
- The vertex shader loops over every morph target for every vertex, every frame. It skips a target whose weight is exactly zero ([bevy_pbr mesh.wgsl, `morph_vertex`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/mesh.wgsl#L13)). The same loop runs again in the depth prepass and the shadow passes, because those passes also need the moved vertices.
- The glTF loader reads morph targets and their names. Bevy's `morph_targets` example plays a glTF animation with morph targets and reads the target names ([Bevy example, morph_targets.rs](https://github.com/bevyengine/bevy/blob/v0.19.1/examples/animation/morph_targets.rs)).

### Skinning in Bevy

- A `SkinnedMesh` component holds a list of joint entities and a handle to the inverse bind poses ([bevy_mesh skinning.rs](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_mesh/src/skinning.rs)). Because the joints are ordinary entities, several meshes can list the same joint entities. That is the Bevy equivalent of Unreal's Leader Pose. Every part follows one skeleton, and every part is still its own draw call.
- A skin can have at most 256 joints (`MAX_JOINTS`) ([bevy_pbr skin.rs, line 28](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/skin.rs#L28)).
- Bevy's `custom_skinned_mesh` example builds a skinned mesh in code, without glTF ([Bevy example, custom_skinned_mesh.rs](https://github.com/bevyengine/bevy/blob/v0.19.1/examples/animation/custom_skinned_mesh.rs)).
- `Mesh::merge` appends one mesh's vertices and indices to another's ([bevy_mesh mesh.rs](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_mesh/src/mesh.rs#L1796)). To merge skinned parts, the code must also remap each part's joint indices to one shared joint list. Bevy does not do this for you.

### Custom materials

Bevy's `ExtendedMaterial` adds custom shader code on top of an existing material ([bevy_pbr extended_material.rs](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/extended_material.rs#L145)). A full custom `Material`, as kjk already uses, gives the same freedom.

### Existing crates

I found no maintained Bevy crate for character customisation, modular clothing or mesh merging. The closest project is `bevy_daz`, an early, unpublished loader for Daz 3D figures, whose repository was last updated in April 2024 ([bevy_daz on GitHub](https://github.com/dannymcgee/bevy_daz)). Epic's OpenRigLogic is C++ under the MIT license and has no Rust binding ([OpenRigLogic](https://github.com/EpicGames/OpenRigLogic)).

## What this means for kjk

### What kjk has today

A character's look is the `Persona` struct in `crates/sim/src/persona.rs:23`. It holds four values plus the seat:

- `appearance`, a row of `appearance.2da`. That row picks a whole premade body and head.
- `gender`, a row of `gender.2da`.
- `portrait`, a row of `portraits.2da`.
- `name`, the character's name.

`Persona::encode` (`crates/sim/src/persona.rs:41`) writes the seat (1 byte), the appearance (2 bytes), the gender (1 byte), the portrait (2 bytes) and the name. The personas ride in the snapshot's tail, so the host sends them to every guest and a save keeps them.

KOTOR already works the way Baldur's Gate 3 and Dragon's Dogma 2 handle armour. There is no shape transfer. Each player appearance row names one build, and each armour class has its own whole body model for that build. kjk's look rules are written out in `crates/asset_odyssey/src/creature.rs:1`. The body model comes from the row's `model<letter>` column, where the letter comes from the worn armour's base item. The head comes from `heads.2da` and is hung on the body's `headhook` node (`crates/asset_odyssey/src/hooks.rs:11`). For example, a male player's medium body is `pmbam` without armour and `pmbbm` in the Endar Spire clothes, as an approved test records (`crates/approved_tests/tests/approved.rs:4683`).

kjk already draws bodies as skinned meshes with Bevy's `SkinnedMesh` (`crates/game/src/scene.rs:1496`). It already draws them with its own material, `OdysseyMaterial` (`crates/game/src/material.rs:1505`).

### Two limits from the original assets

1. **The heads do not share a topology.** Each KOTOR head is a separate model. Morph targets only blend meshes with the same vertex count and order, so kjk cannot blend one KOTOR head into another. Face sliders on the original heads would need morph targets authored for each head separately. Alternatively, one new base head would need to be made, onto which the original heads are re-fitted.
2. **Armour is a whole body.** A body slider (for example, "heavier") would need to reshape every body model of every armour letter. With the original assets, the cheap ways to do that are bone scaling and a wrap deformer from one reference body.

### Option A. More presets, plus colour (low cost, original assets only)

This is the Baldur's Gate 3 approach. It works with the original assets and changes no meshes.

- Separate the head from the body. Today one `appearance.2da` row picks both. The player would pick a build (small, medium or large body), and separately pick a head from all player heads of that gender in `heads.2da`. This needs no new assets. It multiplies the visible combinations.
- Add skin tint, hair tint and eye tint as shader parameters in `OdysseyMaterial`. KOTOR's head textures are painted with their colours, so a tint needs a mask that says which pixels are skin, hair or eyes. The masks can be painted once per head texture and shipped as new files. A tint without a mask would also tint the clothes and the eyes.
- Add tattoos, scars and face paint as decal textures. Draw them over the head texture in the shader, or composite them once into a new head texture at load. Compositing once is cheaper per frame. KOTOR's 2003-era head textures are small next to the 2K to 8K textures of current games, so one extra baked head texture per player costs little memory.
- Keep the dark side texture steps. kjk already swaps head textures by alignment (`headtexe` to `headtexvvve`, `crates/asset_odyssey/src/creature.rs:113`). Tints and decals must then apply to each step's texture, which baking at load handles naturally.

### Option B. Bone-driven sliders on the original meshes (medium cost)

This is the Dragon's Dogma 2 joint approach applied to KOTOR's skeleton.

- **Height and build.** Scale bones of the shared player skeleton (spine, thighs, shins, upper arms, forearms, neck) by a small amount, for example plus or minus 8%. Animations that store rotations keep working. Every armour model follows, because all armour models are skinned to the same skeleton. Mutable's documentation recommends this over morph targets for height ([Mutable FAQ](https://github.com/anticto/Mutable-Documentation/wiki/FAQ)).
- **Gameplay effect.** In kjk, height must stay a visual change unless the rules say otherwise. The sim's collision capsule, the saber reach and the lip-sync camera should not change with it, or co-op results would depend on looks.
- **Head shape.** Scale the whole head and move or scale a few head nodes, if the head models have them. This gives a few cheap sliders (head size, neck length). It is unlikely to give real face shaping, because the head's nodes exist to pose the 16 mouth shapes of its `talk` animation, not to shape the face. How many nodes each head has, and where they sit, needs checking against the actual head models before planning on it.
- **Check needed.** kjk would apply the scale after animation and before skinning. KOTOR's animations include position keys on some bones, and those would need to be scaled with the bone.

### Option C. Morph-target face sliders (higher cost, new art)

This is the EVE Online and FaceGen approach. It needs new art.

- Pick one or a few base heads per gender, either original KOTOR heads or new ones in a similar art style.
- Sculpt 15 to 30 morph targets for each base head (jaw width, nose length, eye spacing and so on), plus a few corrective shapes for combinations that look wrong.
- Bake the creation sliders into the mesh at load, on the CPU. Bevy's vertex shader evaluates every morph target every frame in every pass that draws the mesh ([bevy_pbr mesh.wgsl](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/mesh.wgsl#L13)). Creation sliders do not move during play, so that work would be wasted. kjk builds its meshes itself from MDL files, so it can add the deltas to the vertex positions before it uploads the mesh. Keep Bevy's `MorphWeights` only for the character creator screen, where the sliders do move.
- Keep KOTOR's lip sync working. A KOTOR LIP file indexes 16 mouth poses of the head model's own `talk` animation (`crates/fmt_odyssey/src/lip.rs:1`), so speech moves the head's nodes. Baked creation morphs only change rest positions, so the speech animation runs as before, as long as the morphs do not move vertices away from the nodes they are skinned to.
- Re-fit hair and helmets with a wrap deformer, as Dragon's Dogma 2 does. KOTOR draws headgear as a separate model on the head's `gogglehook` (`crates/asset_odyssey/src/creature.rs:66`), which kjk would deform by the same deltas near each of its vertices.

A statistical (FaceGen-style) model is not practical here. It needs a set of scanned faces with one shared topology. A hand-sculpted set of named sliders is the realistic scope.

### Option D. New modular characters (highest cost)

This replaces the KOTOR player bodies with new modular bodies: a head, a torso, arms, legs and hands as separate parts on one skeleton, with armour made for each part. It is the Unreal Leader Pose approach. In Bevy, each part is a mesh entity whose `SkinnedMesh` lists the same joint entities.

The cost is all the armour. KOTOR has armour models for every body letter and every build, and a modular system needs every armour redrawn for the new parts. This option only makes sense with a large art budget. It also breaks the "replays the original" goal for players who want the original look, unless kjk keeps both paths.

If kjk does go modular, it should merge the parts into one mesh per character when the character is built. This means remapping each part's joint indices to the shared skeleton and joining the vertex buffers. It cuts the draw calls from one per part to one per material, as Unreal's mesh merge does ([Epic, Working with Modular Characters](https://dev.epicgames.com/documentation/en-us/unreal-engine/working-with-modular-characters-in-unreal-engine)).

### Recommended order

1. Start with Option A. It uses only the original assets, it touches the persona format once, and it gives the most visible variety per hour of work.
2. Add the body part of Option B (height and build by bone scale) next, after checking how KOTOR's animations key bone positions.
3. Treat Option C as a separate art project. Prototype it on one head first, with five sliders, to measure how well KOTOR's low-poly heads take morphs.

### What to store and sync in co-op

The look should stay a small set of parameters, as every system above stores it. kjk should never send meshes or textures. Every client already has the game's assets and rebuilds the look from the parameters.

A proposed `Look` block, added to `Persona`:

| Field | Size | Meaning |
|---|---|---|
| version | 1 byte | The look format version. |
| build | 1 byte | The body build (small, medium or large). This replaces the build part of today's `appearance` row. |
| head | 2 bytes | The `heads.2da` row. |
| skin tint | 1 byte | An index into a fixed palette of skin colours. |
| hair tint | 1 byte | An index into a fixed palette of hair colours. |
| eye tint | 1 byte | An index into a fixed palette of eye colours. |
| decals | 1 byte count, then 3 bytes each | Each decal is a decal id (2 bytes) and a colour index (1 byte). |
| sliders | 1 byte count, then 2 bytes each | Each slider is a slider id (1 byte) and a value (1 byte, 0 to 255, where 128 is neutral). |

With 4 decals and 24 sliders, the block is about 80 bytes per player. With four players, that is about 320 bytes, sent when a player joins and again when a player changes their look.

These rules keep it safe across versions and across players:

- **Palette indices, not raw colours.** An index into a fixed palette is one byte. It also keeps players from choosing colours that break the art.
- **Slider ids, not slider positions.** A file names each slider by id. A loader gives the neutral value (128) to any slider the file does not name, and skips any id it does not know. Adding a slider later then needs no migration.
- **Quantised values.** One byte per slider is enough for a character creator. The client converts it to a float weight on load. Every client then computes the same mesh from the same bytes.
- **Keep `appearance` for scripts.** KOTOR's scripts read `GetAppearanceType`. kjk should keep filling `appearance` with the row that matches the chosen build and gender, so scripts and dialogue conditions behave as in the original.
- **Keep the look out of the frame snapshot.** Today the personas ride in every snapshot's tail. The look block changes only at creation and at rare edits. The host could send it in its own message when it changes, and the snapshot could carry only a 1-byte look revision per seat. This keeps per-frame snapshots small as the look grows.
- **Look is not simulation state.** The sim should never read sliders, tints or decals. Then a mismatch in how two clients draw a character cannot cause a desync.
- **Share codes.** The same bytes, encoded in base64, make a share code of about 110 characters for 80 bytes. Players could paste it to copy a look, like a Black Desert appearance file but shorter.
