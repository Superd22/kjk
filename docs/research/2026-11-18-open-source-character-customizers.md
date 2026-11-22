# Open-source character customizers

This note was researched on 2026-11-18.
It answers one question.
Does an open-source character customizer already exist that kjk could reuse instead of its own head customizer?

A **character customizer** here means a tool that lets a player or an artist change a human character with sliders.
The sliders reshape the face and the body, and the tool also offers hair and skin colour or paint.

The search started with Bevy and Rust.
It then covered Godot, Unity, Unreal, the web, Blender add-ons, standalone tools and open-source games.

Every claim carries a link to the project's own repository, its documentation or its store page.
Repository dates were read from the GitHub API on 2026-11-18.
"Last push" means the date of the most recent push to any branch of the repository.

## What kjk already has

The head customizer is documented in `docs/research/2026-11-17-head-parts-and-controls.md`.
In short, kjk's base head is a MakeHuman mesh of 4,750 vertices, carried onto the average head of FLAME 2023 Open.
The overall shape comes from 300 FLAME identity shapes.
Separate parts then add a jawline with 5 controls, eye openings, lips, lashes and painted eyebrows.
Each part is scored against true 3D heads.

Two results from earlier notes matter for this comparison.

- On twelve scanned heads, the 194 MakeHuman sliders left a mean gap of 1.02 mm to the scan. The 300 FLAME shapes left 0.72 mm to 0.82 mm, depending on the prior (`docs/research/2026-11-17-head-sub-systems-plan.md`).
- The earlier sculpting note already listed MakeHuman, MPFB, CharMorph, SPLOCS and OpenRigLogic as prior art (`docs/research/2026-11-16-character-sculpting.md`, section 7). The earlier technology note found no maintained Bevy crate for character customisation (`docs/research/2026-11-16-character-creation-tech.md`, "Existing crates").

This note brings those findings up to date and adds the candidates they did not cover.

kjk is under the MIT licence (`Cargo.toml`, line 8, and `LICENSE`).
Code under the GPL or the AGPL cannot be copied into kjk without changing kjk's licence.
Assets under CC0 can be copied with no condition.

## Summary

- No open-source project does what kjk's head customizer does. None of them has a face model learned from scans with hundreds of identity shapes, plus separate jaw, eye and lip parts that are scored against real heads.
- No Bevy crate offers a character creator with face sliders, hair and skin paint. The closest Bevy project is `burn_human`, which draws NAVER's Anny body model in Bevy. Its face detail comes from MakeHuman targets, and it targets Bevy 0.20 while kjk uses Bevy 0.19.1.
- The most complete open-source runtime customizer in any engine is UMA, the Unity Multipurpose Avatar. It is MIT licensed and was pushed to on 2026-11-18. It is written in C# for Unity, so kjk can only borrow its ideas.
- Almost every open-source human creator is built on MakeHuman's CC0 assets. kjk already uses those assets. Its own tests show that MakeHuman's face sliders fit real heads less closely than the FLAME shapes kjk now uses.
- The verdict is to keep building kjk's own customizer. Three projects are worth borrowing ideas or data from. They are UMA for the runtime design, Humanizer for running MakeHuman assets inside a game engine, and Anny for future body sliders.

## Bevy and Rust

### Search method

The crates.io API was searched for "character creator", "character customization", "avatar customization", "makehuman", "morph target", "blendshape", "bevy character", "bevy avatar", "bevy morph", "bevy face", "vrm" and "smpl".
GitHub was searched for "bevy character creator", "bevy character customization", "bevy avatar", "bevy makehuman" and "bevy morph".
None of these searches found a crate that offers face sliders, hair and skin paint together.
The searches found the projects below.

### burn_human

`burn_human` is a Rust implementation of NAVER's Anny body model, with a Bevy plugin ([README](https://github.com/mosure/burn_human)).
Its README lists "parametric 3dmm forward", "bevy plugin" and "morphs, skeleton, skinning" as done.
It also runs motion generation and image-to-pose inference.
This release uses "Bevy 0.20, Burn 0.22 and WGPU 30" ([README](https://github.com/mosure/burn_human)).
kjk is on Bevy 0.19.1, so it would need an upgrade or a backport first.
The crate's own code is "mit or apache-2.0", and "anny stays under its original license" ([README](https://github.com/mosure/burn_human)).
The repository has 10 stars and was last pushed on 2026-10-09 (GitHub API).
The README says model loading uses "the existing authenticated CDN catalog" ([README](https://github.com/mosure/burn_human)). kjk would have to check that it can ship the model files itself.

What it gives kjk is a body model with phenotype sliders in Bevy.
What it does not give kjk is anything for the face that kjk lacks. The face detail is MakeHuman's.

### symbios-avatar and bevy_symbios_avatar

`symbios-avatar` builds "parametric humanoid and creature bodies, generated entirely from code — no base mesh and no third-party model licences" ([README](https://github.com/TheJanusStream/symbios-avatar)).
`bevy_symbios_avatar` binds it to Bevy.
A body is built from capsules, then a control cage, then Catmull-Clark subdivision ([README](https://github.com/TheJanusStream/symbios-avatar)).
Both crates are MIT licensed and were created on 2026-08-01. They have 1 star and 0 stars (GitHub API).
The bodies are stylised and procedural. They are not built to look like real people, so they do not fit kjk's goal of reproducing real faces.

### VRM loaders

`bevy_vrm` loads VRM avatars into Bevy. It is Apache-2.0 licensed and was last pushed on 2026-10-20 ([repository](https://github.com/unavi-xyz/bevy_vrm)).
`bevy_vrm1` loads VRM 1.0 and VRM animation files ([repository](https://github.com/not-elm/bevy_vrm1)).
These crates load finished avatars. They do not let a player change a face.
VRoid Studio, which makes VRM avatars, is free to use but its source is not published ([VRoid Studio](https://vroid.com/en/studio)).

### smpl-rs

Meshcapade's `smpl-rs` is a Rust library for the SMPL family of body models. Its code is MIT licensed and it was last pushed on 2025-12-18 ([repository](https://github.com/Meshcapade/smpl-rs)).
The SMPL models themselves are licensed separately by the Max Planck Institute.
They were ruled out for kjk in `docs/research/2026-11-17-morphable-model-list-survey.md`.

### bevy_daz

`bevy_daz` is the Daz 3D loader named in the earlier technology note. It has had no push since 2024-04-18 (GitHub API, [repository](https://github.com/dannymcgee/bevy_daz)).

## MakeHuman and its descendants

Most open-source human creators reuse the MakeHuman base mesh and its targets.
MakeHuman splits its licence in two.
The code is under the AGPL. The base mesh, the targets, the textures, the clothes and the poses "have been released under CC0 1.0 Universal" ([MakeHuman LICENSE.md](https://github.com/makehumancommunity/makehuman/blob/master/LICENSE.md)).
The project also states that it makes "no claim whatsoever" over exported models, renders or saved model files ([same file](https://github.com/makehumancommunity/makehuman/blob/master/LICENSE.md)).

### MakeHuman 1

MakeHuman 1 is a standalone Python application.
Its latest release is v1.3.0, published on 2024-05-15. The repository was last pushed on 2024-08-19 (GitHub API, [repository](https://github.com/makehumancommunity/makehuman)).
It is an authoring tool. It does not run inside a game.

### MakeHuman 2

MakeHuman 2 is meant to replace MakeHuman 1. Its README says it is in "Alpha III state" ([README](https://github.com/makehumancommunity/makehuman2)).
It plans "additional base meshes, animation and PBR shading" and glTF export ([README](https://github.com/makehumancommunity/makehuman2)).
The repository has no licence file at its root. Its bundled licence file uses the same split as MakeHuman 1, with AGPL code ([makehuman_license.txt](https://github.com/makehumancommunity/makehuman2/blob/main/data/licenses/makehuman_license.txt)).
The latest release is "Alpha2", published on 2026-05-05. The last commit is dated 2026-10-07 (GitHub API).

### MPFB 2

MPFB is the Blender add-on that continues MakeHuman.
Its code is under GPLv3 "in order to put it more in line with Blender's source code license" ([MPFB LICENSE.md](https://github.com/makehumancommunity/mpfb2/blob/master/LICENSE.md)). Its assets are under CC0 ([LICENSE.ASSETS.md](https://github.com/makehumancommunity/mpfb2/blob/master/LICENSE.ASSETS.md)).
The latest release is v2.0.18, published on 2026-10-11. The last commit is dated 2026-11-12 (GitHub API).
MPFB runs only inside Blender. The sculpting note already describes its 234 modifiers and its `.mhclo` fitting of hair and clothes.

### Humanizer for Godot

Humanizer is a "3D Character Creator for Godot 4.3" ([README](https://github.com/NitroxNova/humanizer)).
Its wiki says it is "based on the MakeHuman program and MPFB2 plugin" and has "over a thousand shape keys" ([wiki](https://github.com/NitroxNova/humanizer/wiki)).
The wiki lists weight, height, gender, race and age sliders, an overlay system for eye, hair and skin colours, clothing from MakeHuman, physical ragdolls and Mixamo animation support ([wiki](https://github.com/NitroxNova/humanizer/wiki)).
The author also published `Humanizer_Menu`, an MIT-licensed in-game menu ([author's repositories](https://github.com/NitroxNova)).
The plugin's code is AGPL-3.0 (GitHub API).
The latest release is v2.2.0, published on 2025-02-09. The master branch was last pushed on 2025-11-07 (GitHub API).
The wiki says version 3 is "coming soon" and splits into separate import and authoring plugins ([wiki](https://github.com/NitroxNova/humanizer/wiki)). The `humanizer_import` and `humanizer_author` repositories were last pushed in July 2025 ([author's repositories](https://github.com/NitroxNova)).

Humanizer is the closest match in another engine to "MakeHuman inside a game".
kjk cannot copy its code, because AGPL code cannot go into an MIT project.
kjk can read it to see how a game engine loads MakeHuman targets and fits `.mhclo` clothes at runtime.

### Go_MakeHuman_dot

Go_MakeHuman_dot is an "ingame character generator for godot (based on the project Makehuman)" ([README](https://github.com/Lexpartizan/Go_MakeHuman_dot)).
It targets Blender 2.79. Its README describes building each character's mesh on load instead of keeping every blend shape in video memory ([README](https://github.com/Lexpartizan/Go_MakeHuman_dot)).
It has no licence file and was last pushed on 2020-02-10 (GitHub API). It is abandoned.

### makehuman-js and makehuman.js

Two separate projects port MakeHuman to the browser.

- `makehuman-js` is "a library to build 3D human characters in the browser" and calls itself an alpha ([README](https://github.com/makehuman-js/makehuman-js)). Its `package.json` declares the AGPLv3. It was last pushed on 2024-08-27 (GitHub API).
- `makehuman.js` is "a MakeHuman variant with different features", written in TypeScript ([README](https://github.com/markandre13/makehuman.js)). It uses MakeHuman's split licence. Its author is working on face animation by mapping ARKit and ICT FaceKit shapes onto the MakeHuman mesh ([README](https://github.com/markandre13/makehuman.js)). It was last pushed on 2026-04-12 (GitHub API).

### Anny

Anny is NAVER Labs' "differentiable human body mesh model written in PyTorch" ([README](https://github.com/naver/anny)).
It covers "infants to elders, using a common topology and parameter space", and it has "full body and part-specific models for hands and faces" ([README](https://github.com/naver/anny)).
Its code is under Apache-2.0. Its MakeHuman assets, adapted from MPFB2, are under CC0. Its facial actions use the CC0 Face Units asset pack ([README, License section](https://github.com/naver/anny#license)).
An optional SMPL-X topology is for non-commercial use only ([README](https://github.com/naver/anny#license)). kjk would leave that topology out.
The latest version is v0.6.2, dated 2026-10-26 in the README. The repository was last pushed on 2026-11-04 (GitHub API).

Anny is a body model for research and machine learning, not a creator with a user interface.
Its interest for kjk is that it turns MakeHuman's many targets into a clean, permissively licensed set of body parameters. `burn_human` already runs it in Bevy.

## Blender add-ons outside MakeHuman

### MB-Lab

MB-Lab is "a character creation tool for Blender 4.0 and above, based off ManuelBastioniLAB" ([repository](https://github.com/animate1978/MB-Lab)).
The repository is archived. Its last push was on 2024-07-21 (GitHub API).

### CharMorph

CharMorph is "a character creation tool for Blender" that "uses base meshes and morphs from ManuelbastioniLAB/MB-Lab" ([README](https://github.com/Upliner/CharMorph)).
Its README lists hairstyles, direct setting of skin and eye colour, real-time fitting of assets, and Rigify support with a full face rig.
Its Python code is under GPLv3 ([license.txt](https://github.com/Upliner/CharMorph/blob/master/license.txt)).
Its character data is licensed per character. The MB-Lab male and female characters are AGPL3. The Antonia character is CC-BY 3.0 ([CharMorph-db, mb_male config](https://github.com/Upliner/CharMorph-db/blob/master/characters/mb_male/config.yaml), [antonia config](https://github.com/Upliner/CharMorph-db/blob/master/characters/antonia/config.yaml)).
The latest release is v0.4.0-3, published on 2025-03-27. The code repository was last pushed on 2026-11-16. The character database was last pushed on 2024-07-23 (GitHub API).
CharMorph runs only inside Blender.

## Godot

### Configura

Configura is an "Open-Source Character Creator Creation Tool in Godot" ([README](https://github.com/Team-Figoose/Configura)).
It is a framework, not a finished creator. The developer brings a 3D model, and Configura connects it to a user interface.
It supports modular hair, clothing and accessories, blend shape deformation, skeletal deformation, layered materials and colour customisation, and user interfaces generated from the mesh data ([README](https://github.com/Team-Figoose/Configura)).
The Godot asset store lists it as MIT licensed, for Godot 4.6, with v1.1.0 released on 2026-08-10 ([Godot asset store](https://store.godotengine.org/asset/configura-team/configura/)).
The GitHub repository has no licence file (GitHub API, [repository](https://github.com/Team-Figoose/Configura)).
Configura brings no human head, no face data and no shapes. It is a user interface layer over shapes that the developer supplies.

### godot-character-creator by Grumoth

This MIT-licensed project was made for Godot 3.2 ([README](https://github.com/Grumoth/godot-character-creator)). It was last pushed on 2020-03-08 (GitHub API). It is abandoned.

### Godot Character Creation Suite

This add-on is sold on itch.io for a minimum of $14.99 ([store page](https://thowsenmedia.itch.io/godot-character-creation-suite)). It is not open source.

## Unity

### UMA, the Unity Multipurpose Avatar

UMA is "an open-source character creation and customization system for Unity" ([documentation index](https://github.com/umasteeringgroup/UMA/blob/master/UMAProject/Assets/UMA/Docs/!README.md)).
It is MIT licensed ([LICENSE](https://github.com/umasteeringgroup/UMA/blob/master/LICENSE), [package LICENSE.md](https://github.com/umasteeringgroup/UMA/blob/master/UMAProject/Assets/UMA/LICENSE.md)). I found no separate asset licence in the repository.
The current release is 3.1.5, published on 2026-11-07. The repository was last pushed on 2026-11-18 (GitHub API). It supports "Unity 6.3 and newer" ([README](https://github.com/umasteeringgroup/UMA)).

UMA does at runtime what kjk wants to do.

- **Sliders.** A DNA slider "can change bones, apply a pose, drive a blendshape, reshape a mesh, move an overlay, or adjust colors and material properties". One slider can drive several of these at once ([DNA Creation Guide](https://github.com/umasteeringgroup/UMA/blob/master/UMAProject/Assets/UMA/Docs/DNACreationGuide.md)).
- **Skin paint.** Overlays are textures, colours and masks that the generator composites into one atlas. An Overlay Painter supports layered painting with symmetry and masks ([documentation index](https://github.com/umasteeringgroup/UMA/blob/master/UMAProject/Assets/UMA/Docs/!README.md)).
- **Hair and clothes.** Hair and clothes are slots combined into wardrobe recipes. The 3.1 release added hair that sways and body blend shapes transferred to clothing ([documentation index](https://github.com/umasteeringgroup/UMA/blob/master/UMAProject/Assets/UMA/Docs/!README.md)).
- **Fitting clothes to a reshaped body.** The Clothing Conformer binds each clothing vertex to the body by "triangle barycentrics, four-vertex fallback weights, normal offset" ([Clothing Conformer](https://github.com/umasteeringgroup/UMA/blob/master/UMAProject/Assets/UMA/Docs/ClothingConformer.md)).
- **Hiding parts under clothes.** A `MeshHideAsset` hides body triangles under a garment ([source file](https://github.com/umasteeringgroup/UMA/blob/master/UMAProject/Assets/UMA/Core/StandardAssets/UMA/Scripts/MeshHideAsset.cs)). The same idea could hide hair under kjk's 14 headgear items.

UMA's face is driven mostly by artist-made bones and blend shapes. It has no learned face space like FLAME.
Its code is C# and depends on Unity. kjk can only port its ideas.

## Unreal Engine

I found no open-source runtime character customizer for Unreal Engine.
The open-source morph projects found are tools for playing or editing morph targets.
`MorphsPlayer` is an MIT-licensed plugin that plays morph targets from animation tracks ([repository](https://github.com/JanSeliv/MorphsPlayer)).
The customizer plugins found on Fab, such as Morph Builder and EasyMorph, are commercial (not verified on their store pages).
Epic's MetaHuman Creator is not open source. Only its RigLogic and DNA libraries are, as the technology note records.

## The web

### Character Studio

Character Studio is "an open, collaborative, and evolving 3D avatar studio for making glTF / VRM avatars" built on Three.js ([README](https://github.com/M3-org/CharacterStudio)).
It is MIT licensed and was last pushed on 2026-08-11 (GitHub API).
Its features are assembling avatars from parts, a colour picker, mesh merging, texture atlasing and VRM export ([README, Features](https://github.com/M3-org/CharacterStudio#features)).
It does not list face shape sliders.

### Ready Player Me

Ready Player Me was a closed avatar service, not an open-source project.
Netflix bought it and announced on 2025-12-19 that the service would wind down on 2026-01-31 ([TechCrunch](https://techcrunch.com/2025/12/19/netflix-acquires-gaming-avatar-maker-ready-player-me/)).

## Open-source games and virtual worlds

### Second Life viewer

The Second Life viewer is open source under the LGPL 2.1 ([LICENSE](https://github.com/secondlife/viewer/blob/develop/LICENSE), [LICENSE-source.txt](https://github.com/secondlife/viewer/blob/develop/doc/LICENSE-source.txt)).
Its avatar definition, `avatar_lad.xml`, lists the shape sliders of the system avatar ([avatar_lad.xml](https://github.com/secondlife/viewer/blob/develop/indra/newview/character/avatar_lad.xml)).
I counted the edit groups in that file. 20 parameters belong to the head group, 29 to the eyes group, 13 to the mouth group, 11 to the nose group, 9 to the chin group and 4 to the ears group.
The licence of the system avatar's meshes is not stated apart from the source code licence, so their reuse is unclear (not verified).
The OpenSimulator community built its own mesh body, Ruth2, and released it under the AGPL ([Ruth2 README](https://github.com/RuthAndRoth/Ruth2)).
The viewer's slider list is a useful checklist of face controls that players of a long-running game expect.

### Ryzom Core

Ryzom Core is AGPL-3.0 and was last pushed on 2026-11-17 (GitHub API, [repository](https://github.com/ryzom/ryzomcore)).
A Ryzom character's face is stored as 8 morph target values with 8 steps each, plus an eye colour and a tattoo ([player_visual_properties.h, lines 147 to 156](https://github.com/ryzom/ryzomcore/blob/core4/ryzom/common/src/game_share/player_visual_properties.h)).
That is much coarser than kjk's controls.

### Veloren

Veloren is a Rust game under GPL-3.0 ([repository](https://github.com/veloren/veloren)).
A Veloren humanoid's look is a set of small numbers for species, body type, hair style, beard, eyes, accessory, hair colour, skin and eye colour ([humanoid.rs](https://gitlab.com/veloren/veloren/-/blob/master/common/src/comp/body/humanoid.rs)).
Each number picks a premade voxel part or colour. There are no shape sliders.

### OpenMW

OpenMW's race dialog lets the player pick a head and a hair style from lists and rotate the head ([race.cpp](https://gitlab.com/OpenMW/openmw/-/blob/master/apps/openmw/mwgui/race.cpp)). It has no shape sliders, like the original Morrowind.

## Comparison

"Runs in a game" means the tool works inside a shipped game, not only inside an editor.

| Project | Engine and language | Face shape | Hair | Skin paint | Runs in a game | Code licence | Asset licence | Last push | Use for kjk |
|---|---|---|---|---|---|---|---|---|---|
| kjk head customizer | Bevy 0.19.1, Rust and Python test scripts | 300 FLAME shapes plus jaw, eye and lip parts | Not started | Lips, brows, lashes, iris | Not yet | MIT | FLAME CC BY 4.0, MakeHuman CC0 | Active | Reference point |
| burn_human | Bevy 0.20, Rust | MakeHuman targets through Anny | No | No | Yes | MIT or Apache-2.0 | Anny Apache-2.0, assets CC0 | 2026-10-09 | Body sliders later, after a Bevy upgrade |
| symbios-avatar | Bevy, Rust | Procedural, stylised | Yes | Yes | Yes | MIT | Generated, no assets | 2026-11-15 | None. The style does not fit |
| UMA | Unity, C# | Bones and blend shapes made by artists | Yes | Yes, overlays and a painter | Yes | MIT | MIT | 2026-11-18 | Design reference for runtime parts, paint and hiding |
| Humanizer | Godot 4.3, GDScript | MakeHuman targets | MakeHuman hair | Overlays | Yes | AGPL-3.0 | MakeHuman CC0 | 2025-11-07 | Read for how MakeHuman runs in an engine |
| Configura | Godot 4.6, GDScript | Whatever the developer supplies | Modular parts | Colours and layers | Yes | MIT on the store, no licence file | None | 2026-08-10 | User interface ideas only |
| MPFB 2 | Blender, Python | 234 MakeHuman modifiers | MakeHuman hair | Skin materials | No | GPL-3.0 | CC0 | 2026-11-12 | Already used for data |
| MakeHuman 1 | Standalone, Python | MakeHuman modifiers | MakeHuman hair | Skin textures | No | AGPL-3.0 | CC0 | 2024-08-19 | Already used for data |
| MakeHuman 2 | Standalone, Python | MakeHuman modifiers | MakeHuman hair | PBR planned | No | AGPL-3.0 | CC0 | 2026-10-07 | Watch only. It is an alpha |
| Anny | PyTorch, Python | MakeHuman targets as a clean model | No | No | No | Apache-2.0 | CC0 | 2026-11-04 | Source of clean body parameters |
| CharMorph | Blender, Python | MB-Lab morphs | Yes | Skin and eye colour | No | GPL-3.0 | AGPL3 or CC-BY 3.0 per character | 2026-11-16 | None. The data licences do not fit |
| MB-Lab | Blender, Python | MB-Lab morphs | Yes | Yes | No | AGPL-3.0 | AGPL-3.0 | Archived, 2024-07-21 | None |
| makehuman.js | Browser, TypeScript | MakeHuman modifiers | No | No | Browser only | AGPL-3.0 | CC0 | 2026-04-12 | None |
| Character Studio | Browser, Three.js | None | Parts | Colours | Browser only | MIT | Separate packs | 2026-08-11 | None |
| Second Life viewer | C++ | About 80 face sliders | Yes | Yes | Yes | LGPL-2.1 | Unclear | 2026-11-18 | Checklist of expected face controls |
| Ryzom Core | C++ | 8 morph targets with 8 steps | Presets | Tattoos | Yes | AGPL-3.0 | Not checked | 2026-11-17 | None |
| Veloren | Rust | None | Presets | Presets | Yes | GPL-3.0 | Not checked | 2026-11-10 | None |

## Verdict

No open-source customizer exists that kjk could adopt in place of its own.

The reasons are these.

1. **Nothing reusable exists in Bevy.** The only Bevy human model, `burn_human`, is a body model on Bevy 0.20. It has no face parts, no hair and no skin paint.
2. **The complete runtime creators use other engines.** UMA is the most complete, and it is C# for Unity. Humanizer is GDScript for Godot and is under the AGPL, which kjk's MIT licence cannot take in.
3. **Their face data is weaker than kjk's.** Humanizer, MPFB, MakeHuman 2, Anny and `burn_human` all shape the face with MakeHuman targets. kjk already ships those assets and measured them. On the same twelve scans, the MakeHuman sliders left 1.02 mm and the FLAME shapes left 0.72 mm to 0.82 mm.
4. **None of them has kjk's parts.** None has separate jaw, eye-opening, lip and lash controls scored against true 3D heads. None reads a face from a photo.

The closest match depends on what is compared.

- For the runtime design, the closest match is UMA. It combines sliders that drive several effects, layered skin paint, hair and clothes as parts, clothes fitted to a reshaped body, and body triangles hidden under clothes.
- For the data, the closest match is MakeHuman and MPFB, under CC0. kjk already uses them.
- In Bevy, the closest match is `burn_human`, and only for bodies.

What kjk would gain by switching is small.
It would get a working user interface sooner and, from UMA or Humanizer, hair and clothes systems that already run.

What kjk would lose is large.
It would lose the FLAME face space and every part built on it, with their scores against true heads.
It would also take on a second engine, or an AGPL licence, or both.

The recommendation is to keep building kjk's own customizer and to borrow from three projects.

- **UMA**, as a design to copy in Rust. Its DNA slider can drive bones, blend shapes and paint at once. Its overlays are composited into one atlas. Its `MeshHideAsset` hides triangles under clothes, and the same idea can hide hair under the 14 headgear items.
- **Humanizer**, as a worked example of loading MakeHuman targets and `.mhclo` clothes and hair at runtime. kjk would read it and write its own code, not copy it.
- **Anny**, when kjk adds body sliders. Its Apache-2.0 code and CC0 assets give a clean body parameter space. `burn_human` already shows it running in Bevy.

## Limits of this search

- The Godot asset library API was not queried. Godot projects were found through GitHub, the Godot store and web search.
- Unreal Fab listings were found only through search snippets. Their prices and licences were not read on the store pages.
- No candidate was installed or run. Features come from each project's own README, wiki or documentation.
- Projects hosted only on GitLab, Codeberg or itch.io may have been missed, apart from those named above.
