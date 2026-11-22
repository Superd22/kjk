# The head and body rig contract

This note was written on 2026-11-16 for kjk's character customization work. It lists what a new or edited head, and a new or edited body, must keep so that animation, lip sync, blinking and head turns keep working in KOTOR 1 and KOTOR 2 as kjk plays them. The last section is a checklist that a modder or an artist can follow.

It builds on four earlier notes in this folder. They are `2026-11-16-character-sculpting.md`, `2026-11-16-custom-look-game-impact.md`, `2026-11-16-character-creation-tech.md` and `2026-11-16-kotor-era-species.md`. This note does not repeat them. It corrects three statements in the sculpting note, listed in the section "Corrections to the earlier notes".

## How this was checked

The facts in this note come from four kinds of source.

- **The installs.** Models were copied out of the Steam installs of both games with `kjk extract`. A throwaway program read them with kjk's own model parser (`crates/fmt_odyssey/src/mdl.rs:856`). It printed each model's node tree, each skinned mesh's bones and weights, and every animation's keyed nodes and keys. The program lives in the session's scratch folder and was not added to the repository. Tables were read with `kjk show`.
- **The heads checked.** In KOTOR 1 these were the player heads `pmhc01`, `pfhc01`, `pmha01`, `pfha01` and `pmhb03`, the companion heads `p_carthh`, `p_bastilah`, `p_missionh`, `p_juhanih`, `p_candh` and `p_joleeh`, the Twi'lek heads `twilek_f` and `twilek_m`, the masked `n_darthrevanh` and the commoner head `comm_a_m`. In KOTOR 2 they were `pmhc01`, `pfhc01`, `pmhh01`, `pfhh01`, `p_attonh`, `p_kreiah`, `p_visash`, `p_handmaidenh`, `p_baodurh`, `p_mirah`, `p_discipleh`, `twilek_f`, `twilek_m` and the Twi'lek assassin head `n_twiassh01`.
- **The bodies and supermodels checked.** These were `pmbam`, `pmbal`, `pmbjm`, `pfbam`, `pfbas`, `p_carthbb`, `p_bastilabb`, `p_missionbb`, `p_juhanibb`, `n_calonord`, `n_twilekf`, `n_twilekm` and `n_commf` in KOTOR 1, and `pmbam`, `pfbam`, `p_attonbb`, `p_visasbb`, `p_kreiabb`, `p_baodurbb` and `p_handmaidenbb` in KOTOR 2. The supermodels were `s_male01`, `s_male02`, `s_female01`, `s_female02` and `s_female03` in both games.
- **kjk's code and docs**, cited as `file:line`.
- **Other engines and tools**, cited by URL. These are reone, KotOR.js, xoreos, kotorblender, PyKotor and MDLOps. Claims that come only from the Deadly Stream forums are marked "forum only".

The game was not launched for this note.

## Terms used in this note

- A **node** is one named entry in a model's tree. It has a position and a rotation relative to its parent.
- A **bone** is a node that a skinned mesh's vertices follow. KOTOR's bones are ordinary nodes, mostly named with a `_g` ending, and many of them are small invisible meshes.
- A **hook** (also called a **dummy**) is an empty node that the engine looks for by name to hang something on or aim at. Examples are `headhook`, `rhand` and `camerahook`.
- A **skinned mesh** (KOTOR's "skin" node) is a mesh whose vertices each follow up to four bones with a weight for each.
- A **dangly mesh** is a mesh whose vertices swing on springs. KOTOR uses it for hair and cloth.
- A **supermodel** is the model a model names in its header to borrow animations from. A model's **chain** is the model, its supermodel, that model's supermodel, and so on.
- A **clip** is one named animation, such as `pause1` or `talk`.
- A **key** is one sample of a clip on one node, at one time. A clip can key a node's **position**, its **rotation** or, in the file format, its **scale**.
- A node's **rest** is its position and rotation as the model file stores them, before any clip plays.
- The **animation root** is a node named in the model header. Only that node and the nodes under it take keys from a clip. In every head checked it is `neck_g`.
- A **LIP file** is the list of mouth shapes for one voiced line, with a time for each.
- A **shape** is one of the 16 mouth positions a LIP file names, numbered 0 to 15.
- **Retargeting** is kjk's step that turns Jedi Academy's skeleton poses into turns on a KOTOR body's nodes (`docs/SKELETON.md`).

## Corrections to the earlier notes

1. **Position keys are offsets, not absolute positions.** The sculpting note says "the talk animation keys absolute positions". The data shows otherwise. In KOTOR 1's `talk` clip on `s_male02`, the upper lip `f_um_g` is keyed at (0, 0, 0.0003) at time 0. Its rest position in `pmhc01` is (0, 0.087, 0.006). The largest position key on any face node, in any clip of either game's human supermodels, is 3.1 cm (the left brow in the duel damage clips of `s_female01`). The engines add the key to the node's own rest position. kjk does this in `crates/asset_odyssey/src/anim.rs:4-6` and `crates/asset_odyssey/src/anim.rs:166-186`, following reone. Rotation keys are absolute. A rotation key replaces the node's rest rotation.
2. **The `talk` clip moves only the mouth.** The sculpting note says the talk animations key all 13 face bones and the eyes. That is true of the conversation body clips (`tlknorm`, `listen`, `pause1`), which key 18 to 20 face nodes. The `talk` clip itself keys 7 nodes. They are `f_um_g`, `f_lmc_g`, `f_rmc_g`, `f_jaw_g`, `f_llm_g`, `f_rlm_g` and `f_tonguetip_g`. It does not key the brows, the cheeks, the under-eye bones, the eyes or the lids.
3. **Male player heads use `s_female02`, not `s_female03`.** Every male player head checked (`pmhc01`, `pmha01`, `pmhb03` in KOTOR 1, `pmhc01` and `pmhh01` in KOTOR 2) names `s_female02` as its supermodel. Every female player head names `s_female03`. Both chains end in `s_male02` and then `s_male01`. The names do not follow the sexes. The chains are `s_female03 → s_female02 → s_female01 → s_male02 → s_male01` for women and `s_female02 → s_female01 → s_male02 → s_male01` for men.

## What every head must keep

### The node tree of a player head

Every player head has the same tree. This is KOTOR 1's `pmhc01`, with positions in metres relative to the parent.

| Node | Parent | Kind | Rest position | Role |
|---|---|---|---|---|
| `pmhc01` (the model's own name) | none | empty | 0, 0, 0 | The model root. |
| `cutscenedummy` | model root | empty | 0, 0, 0 | A copy of the body's node. It stays at rest. |
| `rootdummy` | `cutscenedummy` | empty | 0, 0, 0 | A copy of the body's node. It stays at rest. |
| `torso_g` | `rootdummy` | empty | 0, 0, 0 | A copy of the body's spine. It stays at rest. |
| `torsoupr_g` | `torso_g` | empty | 0, 0, 0 | A copy of the body's spine. It stays at rest. |
| `necklwr_g` | `torsoupr_g` | bone | 0, 0, 0 | The base of the neck. The neck's lowest ring of skin follows it. |
| `neck_g` | `necklwr_g` | bone | 0, −0.054, −0.024 | The neck. It is the animation root. |
| `hturn_g` | `neck_g` | empty | 0, 0.040, 0.108 | The head-turn node. Head look-at turns it. |
| `head_g` | `hturn_g` | bone | 0, 0.035, 0 | The skull. Most of the face's skin follows it. |
| `talkdummy` | `head_g` | empty | 0, 0, 0 | The root of the `talk` clip, and the mouth's aim point. |
| `f_um_g` | `talkdummy` | bone | 0, 0.087, 0.006 | The upper lip. |
| `f_lmc_g`, `f_rmc_g` | `talkdummy` | bones | ±0.030, 0.073, −0.007 | The mouth corners. |
| `f_jaw_g` | `talkdummy` | bone | 0, 0.006, 0.020 | The jaw. |
| `f_llm_g`, `f_rlm_g` | `f_jaw_g` | bones | ±0.013, 0.084, −0.039 | The two halves of the lower lip. |
| `f_tonguetip_g` | `f_jaw_g` | bone | 0, 0.035, −0.023 | The tongue tip. |
| `teethla` | `f_jaw_g` | rigid mesh | 0, 0.062, −0.025 | The lower teeth. |
| `teethua` | `head_g` | rigid mesh | 0, 0.078, 0.001 | The upper teeth. |
| `eyela`, `eyera` | `head_g` | rigid meshes | ±0.029, 0.056, 0.064 | The eyeballs. |
| `eyellid`, `eyerlid` | `head_g` | rigid meshes | ±0.028, 0.058, 0.064 | The eyelids. |
| `f_mdbrw_g`, `f_lbrw_g`, `f_rbrw_g` | `head_g` | bones | middle and ±0.044 at 0.079, 0.089 | The brows. |
| `f_llweye_g`, `f_rlweye_g` | `head_g` | bones | ±0.030, 0.077, 0.044 | Under the eyes. |
| `f_lns_g`, `f_rns_g` | `head_g` | bones | ±0.050, 0.072, 0.046 | The cheeks. |
| `maskhook` | `head_g` | empty | 0, 0.093, 0.020 | A hook for worn masks. kjk does not use it. |
| `gogglehook` | `head_g` | empty | 0, 0.080, 0.066 | Where kjk hangs masks and visors. |
| `head` | model root | skinned mesh | 0, 0.017, 0.121 | The face and neck skin. |

The left side of a head is at −X. The face looks toward +Y. Z is up. This matches kjk's description of KOTOR's model space (`crates/asset_ja/src/rig.rs:6-9`).

### What is the same on every head

- **The bone names.** All 16 skin bones keep the same names on every human head checked in both games. They are `necklwr_g`, `neck_g`, `head_g` and the 13 face bones in the table.
- **The hierarchy of the mouth.** On every head, the four mouth bones (`f_um_g`, `f_lmc_g`, `f_rmc_g`, `f_jaw_g`) are children of `talkdummy`. The lower lip halves and the tongue tip are children of `f_jaw_g`. This matters for kjk, because kjk binds a clip only to nodes under the clip's root node, and `talk`'s root is `talkdummy` (`crates/asset_odyssey/src/anim.rs:141-160`).
- **The path from the neck to the face.** Every head has `necklwr_g → neck_g → hturn_g → head_g → talkdummy`.
- **The animation root.** Every head checked marks `neck_g` as its animation root. The copy of the spine above it (`rootdummy`, `torso_g`, `torsoupr_g`) is never animated on the head. kjk reads the root from the model header (`crates/fmt_odyssey/src/mdl.rs:200-206`, `crates/fmt_odyssey/src/mdl.rs:884`).
- **The skinned mesh is a child of the model root.** The face skin hangs directly under the model root, not under a bone. A Deadly Stream modder says the same: "skinned meshes are always direct children of the base" (DarthParametric, [Creating custom head](https://deadlystream.com/topic/7590-creating-custom-head/), forum only).
- **The tongue is skinned to two bones.** Every head has a small skinned tongue (22 to 40 vertices) on `f_tonguetip_g` and `f_jaw_g`.
- **Four weights at most per vertex.** The largest number of bones on one vertex is 4 in every skinned mesh checked. kotorblender keeps only the four largest weights when it exports ([skinmesh.py](https://github.com/OpenKotOR/kotorblender/blob/0aad2720519d01917f5f8c125627ea959c4abde2/io_scene_kotor/scene/modelnode/skinmesh.py#L49-L65)).
- **Heads carry no clips of their own.** Player heads carry no animation. Every clip they play comes from the supermodel chain or from the body.

### What differs between heads

- **Mesh names.** The skinned face is called `head`, `headnew`, `head_geo`, `headnew_geo` or `face`. The teeth are `teethla`/`teethua`, `teethl`/`teethu`, `teethlower`/`teethupper` or `lteeth`/`uteeth`. No clip and no engine code reads these names, so they are free. One exception exists. Juhani's own `talk` clip (`p_juhanih`) keys her `headnew` and `tongue` meshes at zero offsets, which is harmless.
- **Hair.** Hair is a plain mesh, a dangly mesh (for example `p_bastilah`'s `hair` and `hairbangs`, `pfhc01`'s `ponytail` and `bangs` in KOTOR 2) or a skinned mesh on the neck and skull bones (Mission's lekku, Mira's `hairflaps`).
- **Bone count.** Most face skins use 16 bones. Visas Marr's `headnew_geo` uses 13, because it has no brow bones. KOTOR 2's `pmhh01` lists 17, the 17th being the model root with one vertex on it.
- **Hooks.** The Twi'lek heads (`twilek_f`, `twilek_m`, `n_twiassh01`) have no `gogglehook` and no `maskhook`. kjk therefore draws no mask or visor on them (`crates/game/src/viewer.rs:2006-2030`).
- **Eyes.** `n_darthrevanh` and Visas's `p_visash` have no eyeball or eyelid meshes. `comm_a_m` calls its eyeballs `eyel` and `eyer`. The clips key `eyela` and `eyera`, so that head's eyes never move.
- **Companion clips.** Some companion heads carry their own clips, which win over the supermodel's. Bastila's head has `pause1`, `talk`, `pause2`, `tlknorm` and `listen`. Mission, Juhani, Canderous and Mira have their own `talk`. Companion heads name their own body as supermodel (`p_bastilah → p_bastilabb → s_female03`).

### How a head attaches to a body

- **The body's `headhook`.** A player body has a `headhook` empty under `torsoupr_g`. In KOTOR 1's `pmbam` it sits at (0, 0.039, 0.281) from `torsoupr_g`. The head model's root is placed on it. kjk finds the head as the model hung on the body (`crates/game/src/face.rs:18-35`), and reone and xoreos attach the head the same way ([reone creature.cpp](https://github.com/seedhartha/reone/blob/0666aee0e25fc8a6dbf77b7a186350f3b5857711/src/libs/game/object/creature.cpp#L1100-L1112), [xoreos creature.cpp](https://github.com/xoreos/xoreos/blob/6774724d5c05e09ce0d01e00b8b62f956ddcb74b/src/engines/kotorbase/creature.cpp#L400-L446)).
- **Which head.** `appearance.2da`'s `normalhead` names a row of `heads.2da`, whose `head` column names the model. xoreos falls back to `backuphead` when `normalhead` is blank.
- **The body has no face.** Male player bodies (`pmbam`, `pmbal`) have no neck or head nodes. Female player bodies (`pfbam`, `pfbas`) and `pmbjm` have `necklwr_g` and `neck_g` under `headhook`, but no `head_g`. Companion bodies (`p_carthbb`, `p_bastilabb`, `p_attonbb` and others) have the full neck-to-`talkdummy` chain, with tiny proxy meshes.
- **A head and a body are one model in a few creatures.** Calo Nord's `n_calonord` has its face skinned into the body. kjk then plays `talk` on the body (`crates/game/src/dialogue_view.rs:1218-1236`).

### Which clips reach a head, and how kjk plays them

- **Body clips on the face.** kjk binds every KOTOR clip that the body plays to the head's nodes by name (`crates/game/src/face.rs:1-8`, `crates/game/src/cast.rs:2660-2694`). The head then blinks, looks around, sleeps and dies with the body. When the body plays no KOTOR clip (a Jedi Academy pose, a ragdoll), the head plays the body's standing clip, or its dead clip once it is dead (`crates/game/src/face.rs:87-101`). Each head starts its loop at its own phase, so heads do not blink together (`crates/game/src/face.rs:44-45`).
- **Which nodes the body clips key.** In both games, the resolved human chain has 267 to 268 clips in KOTOR 1 and 456 to 457 in KOTOR 2. Every KOTOR 1 clip, and 410 to 413 KOTOR 2 clips, key face nodes. A typical clip (for example `pause1` or `tlknorm`) keys position and rotation on all 13 face bones, rotation on `neck_g` and `head_g`, and rotation on `eyela`, `eyera`, `eyellid` and `eyerlid`.
- **Blinking lives in the body clips.** There is no blink clip. `pause1` in `s_male02` closes both lids three times in 6.7 s (at about 1.6 s, 3.5 s and 5.3 s). A blink turns each lid about its X axis by about 34° for 0.13 s. `tlknorm` and `listen` blink the same way. `dead` and `sleep` hold the lids shut.
- **Eye movement lives in the body clips too.** The same clips turn `eyela` and `eyera` by up to about 19° to look around.
- **The conversation clips.** A speaker loops its chain's `tlknorm` (`docs/DIALOGUE.md:350-361`). A line's `AnimList` can add `listen`, `tlklaugh`, `tlksad`, `tlkplead`, `tlkforce` and others, and in KOTOR 2 `nodyes`, `nodno`, `rolleyes`, `shrug` and others (`crates/asset_odyssey/src/creature.rs:598-667`). All of these come from the supermodel chain, so a new head that keeps the node names gets them for free.
- **The head-turn clips.** `hturnl` and `hturnr` (2.7 s each) exist in both games. kjk lists them (`crates/asset_odyssey/src/creature.rs:666-667`), but its head look-at does not use them.
- **KOTOR 2's `emotions` clip.** KOTOR 2's supermodels have a 0.5 s clip called `emotions` with 16 keys, rooted at `head_g`, which keys the mouth, brows, under-eyes and eyes. Nothing in kjk reads it. No source found says that KOTOR 2's executable plays it.

### Shared clips key node names, so a new head inherits everything

Every clip keys nodes by name, and kjk matches names without regard to case (`crates/asset_odyssey/src/anim.rs:141-160`). A new head that keeps the names, the hierarchy and the animation root gets every blink, look, expression and death face that the shared supermodels hold. A head that renames a node loses every key on that node. A modder on Deadly Stream says the same: "the bones are matched based both on their hierarchy and their names, so do not change that" (bead-v, [Creating custom head](https://deadlystream.com/topic/7590-creating-custom-head/), forum only). MDLOps matches each node to the supermodel's node of the same name, and only among the children of the parent's match ([MDLOpsM.pm](https://github.com/ndixUR/mdlops/blob/7e40846d36acb5118e2e9feb2fd53620c29be540/MDLOpsM.pm#L3838-L3910)).

## What every body must keep

### The nodes kjk's retargeting needs

kjk drives a KOTOR body with Jedi Academy's skeleton by turning named KOTOR nodes (`crates/asset_ja/src/rig.rs:74-108`). A body counts as humanoid, and is driven this way, only if it has every one of these nodes (`crates/asset_ja/src/rig.rs:742-761`).

| KOTOR node | Jedi Academy bone | Notes |
|---|---|---|
| `pelvis_g` | `pelvis` | Its rest height sets how far the root travels. |
| `torso_g` | `upper_lumbar` | The lower spine. It follows JA's spine by height. |
| `torsoupr_g` | `thoracic` | The upper spine. It follows JA's spine by height. |
| `rcollar_g`, `lcollar_g` | `rclavical`, `lclavical` | Aimed at the upper arm. |
| `rbicep_g`, `lbicep_g` | `rhumerus`, `lhumerus` | Aimed at the forearm. |
| `rforearm_g`, `lforearm_g` | `rradius`, `lradius` | Aimed at the hand. |
| `rhand_g`, `lhand_g` | `rhand`, `lhand` | Aimed at the first and fourth finger bases. |
| `rthigh_g`, `lthigh_g` | `rfemurYZ`, `lfemurYZ` | Aimed at the knee, rolled with the shin. |
| `rshin_g`, `lshin_g` | `rtibia`, `ltibia` | Aimed at the foot. |
| `rfoot_g`, `lfoot_g` | `rtalus`, `ltalus` | Not aimed. Held flat. |

The aim points also need the finger bases `rafngrb_g`, `rdfngrb_g`, `lafngrb_g` and `ldfngrb_g`. Without them the hand is not lined up and keeps its rest turn. `neck_g` and `head_g` are in the table too (`cervical`, `cranium`), but the humanoid test leaves them out, because player bodies have no head.

### The rest pose kjk expects

- **Axes.** The body faces +Y, its right is +X and Z is up (`crates/asset_ja/src/rig.rs:6-9`).
- **Limbs may rest in any pose.** Each limb node is lined up with its JA bone by aiming it at its child, so an A-pose and a T-pose both work (`crates/asset_ja/src/rig.rs:16-24`).
- **The spine, neck, head and feet must rest upright or flat.** These nodes have no aim. kjk assumes both skeletons hold them upright, or flat for the feet (`crates/asset_ja/src/rig.rs:22-24`). A body whose spine leans at rest would lean when driven.
- **The collars.** A collar that starts more than 3 cm off the spine keeps its own rest angle, as KOTOR's women's collars do (`crates/asset_ja/src/rig.rs:720`, `docs/SKELETON.md`).
- **The spine by height.** kjk spreads JA's spine turns over `torso_g` and `torsoupr_g` by their height between the hips and `neck_g` (`crates/asset_ja/src/rig.rs:282-292`, `crates/asset_ja/src/rig.rs:690-715`). This needs `neck_g` in the body model. KOTOR 1's and KOTOR 2's male player bodies (`pmbam`, `pmbal`) have no `neck_g`, so they fall back to the older pairing of the bones named alike. This is a reading of the code and was not run for this note. A new body should carry `necklwr_g` and `neck_g` under `headhook`, as the female bodies do.

### Root and height handling

- **The pelvis sets the scale.** kjk scales JA's root travel by the ratio of KOTOR's `pelvis_g` height to JA's (`crates/asset_ja/src/rig.rs:733-740`).
- **The feet hold the floor.** The supporting foot is held at KOTOR's own ankle height plus JA's lift (`docs/SKELETON.md`, "Pipeline").
- **KOTOR's own builds already differ in height.** The `rootdummy` stands at 1.126 m in a man's middle build (`pmbam`), 1.157 m in a man's large build (`pmbal`), 1.063 m in a woman's middle build (`pfbam`) and 1.026 m in a woman's small build (`pfbas`).
- **KOTOR scales its clips per body with the animation scale.** The model header's animation scale multiplies every position key of the clips the body plays. It is 1.06 for `pmbam` and `p_carthbb`, 1.028 for `pmbal`, 1.0 for `pfbam` and 0.971 for `pfbas` and `p_missionbb`. kjk applies it (`crates/game/src/scene.rs:2543-2544`, `crates/game/src/cast.rs:200-206`).
- **Out of a fight a body walks on KOTOR's clips.** The walk and run cycles advance by `appearance.2da`'s `walkdist` and `rundist`, one cycle per stride (`docs/SKELETON.md`, "Gait").

### The hooks a body must have

| Hook | Where on `pmbam` | Who reads it |
|---|---|---|
| `headhook` | under `torsoupr_g` | Where the head hangs (`crates/asset_odyssey/src/hooks.rs:11`). Also the head hit location and the blade's head clearance (`crates/game/src/cast.rs:1755`, `crates/asset_ja/src/rig.rs:1890-1895`). |
| `rhand`, `lhand` | under `rhand_g`, `lhand_g` | Where weapons hang (`crates/asset_odyssey/src/hooks.rs:6-9`, `crates/game/src/viewer.rs:1807-1808`). |
| `lforearm` | under `lforearm_g` | KOTOR's shield and forearm attachments. kjk does not read it. |
| `camerahook` | under the model root, at 1.634 m | Dialogue and menu cameras (`crates/game/src/cast.rs:506`, `docs/DIALOGUE.md:1063-1067`). |
| `freelookhook` | under the model root, at 1.635 m | KOTOR's free-look camera. kjk does not read it. |
| `impact` | under `torsoupr_g` | Where hit and Force effects land (`crates/asset_odyssey/src/hooks.rs:24`, `crates/game/src/script_fx.rs:264`). |
| `impact_bolt` | under the model root | Blaster bolt impacts. kjk does not read it. |
| `headconjure`, `handconjure` | under the model root | Force power sources (`crates/asset_odyssey/src/hooks.rs:25-26`, `crates/game/src/script_fx.rs:266`). |
| `lightsaberhook` | under the model root | Saber sparks (`crates/game/src/clip_events.rs:28`). |
| `deflecthook` | under the model root | KOTOR's deflection effect. kjk does not read it. |
| `rootdummy`, `cutscenedummy` | the top of the tree | Lying clips and stunt cutscenes (`crates/asset_odyssey/src/hooks.rs:39`, `crates/game/src/play_view.rs:3283`). |

The head carries `talkdummy` and `gogglehook`. A body made with its face (`appearance.2da` model type F, such as Calo Nord) carries them itself.

### Skinning rules for bodies

- Player bodies split the skin into three meshes, `torso`, `larm` and `rarm`, with 14 to 16 bones each. Carth adds a `legs` mesh.
- The model header's skin data holds 16 bone slots ([kotorblender reader.py](https://github.com/OpenKotOR/kotorblender/blob/0aad2720519d01917f5f8c125627ea959c4abde2/io_scene_kotor/format/mdl/reader.py#L408-L418)). Modders report that 17 bones per mesh is the real limit, and that MDLedit warns past 16 (DarthParametric, [Armor model editing](https://deadlystream.com/topic/8188-armor-model-editing-kotormax-mdlops-severe-distortions-ingame/), forum only). One shipped head, KOTOR 2's `pmhh01`, has 17. kjk itself reads the bone map, not the 16 slots, so it has no limit (`crates/asset_odyssey/src/model.rs:217-266`). A mesh made for the original game should stay at 16 bones.
- kjk skins with the file's own inverse bind matrices (the `qbones` and `tbones` arrays), not with matrices it rebuilds from the rest pose (`crates/asset_odyssey/src/model.rs:154-165`, `crates/asset_odyssey/src/model.rs:252-266`). A tool that writes a model must write these arrays so that they match the rest pose. `kjk formats` reports when they differ (`crates/launcher/src/formats.rs:440-460`).

### What breaks if bone lengths or proportions change

- **Nothing in the rules.** The sim's body is Jedi Academy's player box and skeleton, not the KOTOR model (`docs/SKELETON-SIM.md`). Collision and blade hits do not change.
- **Clips survive longer or shorter bones.** KOTOR's body clips key rotations, plus position offsets that the animation scale scales. Retargeting copies turns, not lengths. A longer arm or leg therefore still animates.
- **The hand and the blade can drift apart.** The drawn hand reaches at most 0.3 m toward the sim's blade (`crates/asset_ja/src/rig.rs:1556`). A body whose arms end far from Jedi Academy's would show a gap.
- **Feet slide** if the legs change and `walkdist` and `rundist` do not.
- **Hooks must move with the body.** `camerahook`, `freelookhook`, `headconjure`, `lightsaberhook` and the other root-level hooks are placed in metres under the model root, not under bones. A taller body must have them moved, or the dialogue camera and Force effects stay at the old height.
- **Bevy scale on a bone scales everything under it.** No KOTOR clip keys scale on any node checked, and kjk's clip player writes only position and rotation (`crates/game/src/scene.rs:2600-2690`). A scale set on a bone therefore stays. But it also scales every child, so a scaled forearm would scale the hand, the `rhand` hook and the weapon hung on it. Changing rest positions (bone lengths) is safer than scaling bones.

## What lip sync needs

### The LIP format

A LIP file starts with the 8 bytes `LIP V1.0`. Then come the line's length as a 32-bit float in seconds, and the number of keys as a 32-bit integer. Each key is a time (a 32-bit float in seconds) and a shape (one byte). kjk reads it in `crates/fmt_odyssey/src/lip.rs:1-40`, and reone the same way ([lipreader.cpp](https://github.com/seedhartha/reone/blob/0666aee0e25fc8a6dbf77b7a186350f3b5857711/src/libs/graphics/format/lipreader.cpp#L27-L41)).

- There are 16 shapes. kjk names them as 0 rest, 1 IH IY, 2 EH ER EY, 3 AA AE AH, 4 OW OY, 5 UH UW W, 6 D DH S Y Z, 7 CH JH SH ZH, 8 F V, 9 G HH K NG, 10 T TH, 11 B M P, 12 L N, 13 R, 14 AW AY, 15 AO (`crates/fmt_odyssey/src/lip.rs:4-7`).
- Other tools name them differently. PyKotor uses neutral, EE, EH, AH, OH, OOH, Y, STS, FV, NG, TH, MPB, TD, SH, L, KG ([lip_data.py](https://github.com/OldRepublicDevs/PyKotor/blob/ca61ce8d53f5442f73c109e8518cd9a838b2f37c/Libraries/PyKotor/src/pykotor/resource/formats/lip/lip_data.py#L112-L127)). KotOR.js starts at EE with no neutral shape ([LIPShape.ts](https://github.com/KobaltBlu/KotOR.js/blob/002dfe03b9a59ad2adda6404d9632fc78d5bedff/src/enums/resource/LIPShape.ts)). All agree that 11 is the closed-lips B, M, P shape and 8 is F, V.
- In a sample of 150 KOTOR 1 LIP files, every file starts with a key at time 0 and ends with a key at the line's length. All 16 shapes appear. Shape 2 is the most common (2,230 keys) and shape 0 is next (1,335 keys).
- A LIP file holds no face data. It only names shapes. The face for each shape comes from the speaking head's `talk` clip. A modder says it plainly: "Lip files should be universal, since all they are is a list of mouth shapes" (DarthParametric, [Bith lipsync thread](https://deadlystream.com/topic/10970-how-to-apply-the-bith-dialogue-lipsync-animation-to-a-different-npc-kotor-1), forum only).

### The `talk` clip

- `talk` is 0.5 s long in every chain checked, in both games. Its root is `talkdummy`.
- Each of its position tracks has 16 keys, one every 1/30 s from 0 to 0.5 s. Some rotation tracks drop repeated keys and have 14 or 15. In other words, the clip holds one mouth pose per shape.
- It keys the 7 mouth nodes listed above. In the supermodels it keys nothing else.
- Each chain has its own. `s_male02` holds the men's, `s_female03` the women's. Bastila, Mission, Juhani, Canderous and Mira override it in their own head or body models.
- The offsets are small. The largest is 2.1 cm on the tongue tip in KOTOR 2's `s_male02`. The jaw's largest is 0.7 cm. The lips move by up to about 1 cm and turn by up to about 34°.

### How a shape becomes a mouth pose

The four engines disagree.

| Engine | Time in `talk` for shape *s* | Source |
|---|---|---|
| kjk | *s* × length ÷ 16, then a blend toward the next key's shape | `crates/game/src/dialogue_view.rs:1145-1158` |
| reone | *s* × length ÷ 16, then a blend | [model.cpp](https://github.com/seedhartha/reone/blob/0666aee0e25fc8a6dbf77b7a186350f3b5857711/src/libs/scene/node/model.cpp#L311-L397) |
| KotOR.js | The *s*-th key of each track | [LIPObject.ts](https://github.com/KobaltBlu/KotOR.js/blob/002dfe03b9a59ad2adda6404d9632fc78d5bedff/src/resource/LIPObject.ts#L146-L275) |
| xoreos | No lip sync. It loops `talk` at a quarter of normal speed. | [dialog.cpp](https://github.com/xoreos/xoreos/blob/6774724d5c05e09ce0d01e00b8b62f956ddcb74b/src/engines/kotorbase/gui/dialog.cpp#L292-L302) |
| The original executable | Unverified. One reverse-engineering note says (*s* + 1) ÷ 16 of the length, and that the executable rejects a LIP file unless its first key is at 0 and its last at the length. | [rakata LIP notes](https://synchro.codeberg.page/rakata/formats/audio/lip.html) |

**A likely kjk bug.** The clip data fits one key per shape at time *s* × length ÷ 15 (*s* ÷ 30 s). kjk samples at *s* × length ÷ 16, which lands between keys for every shape above 0. For shape 8 (F, V) kjk shows a pose halfway between keys 7 and 8. For shape 15 (AO) it shows a pose 6% of the way from key 14 to key 15, which is almost shape 14. The fix is one constant in `crates/game/src/dialogue_view.rs:1153`: divide by `SHAPES − 1` instead of `SHAPES`. This should be checked in the original executable first, with `kjk original` or a disassembler, because the rakata note suggests a third reading.

### Which model plays `talk`

- **kjk** plays `talk` on the speaker's hung head, through the head's own supermodel chain. If no hung model has a `talk` that binds one of its nodes, kjk tries the body (`crates/game/src/dialogue_view.rs:1192-1236`). The LIP is found by the voice file's name, else the node's other sound's (`crates/game/src/dialogue_view.rs:775-781`).
- **reone** looks up `talk` on the body's chain and passes it down to the attached head ([creature.cpp](https://github.com/seedhartha/reone/blob/0666aee0e25fc8a6dbf77b7a186350f3b5857711/src/libs/game/object/creature.cpp#L205-L232), [model.cpp](https://github.com/seedhartha/reone/blob/0666aee0e25fc8a6dbf77b7a186350f3b5857711/src/libs/scene/node/model.cpp#L257-L263)). For human players this is the same clip, because the player head and body chains end in the same supermodels.
- **The body loops `tlknorm` at the same time.** kjk (`docs/DIALOGUE.md:350-361`), reone and xoreos all do this. `tlknorm` keys the mouth bones too. kjk poses `tlknorm` first through the face and then writes the lip pose over the mouth nodes. reone mixes both channels.

### What a head needs for lip sync to work

1. The 7 mouth nodes, with these exact names.
2. The 4 mouth bones under `talkdummy`, and the lower lip halves and tongue tip under `f_jaw_g`.
3. `talkdummy` at or under the head's animation root (`neck_g`).
4. A supermodel chain that reaches a `talk` clip (for a human head, `s_female02` or `s_female03`), or a `talk` clip of its own.
5. Skin weights on those bones that actually move the lips. The rest pose (shape 0, key 0) must show a closed mouth, because shape 0 is the most common shape after shape 2.
6. A LIP file for the line. Players never speak voiced lines in either game (`2026-11-16-custom-look-game-impact.md`), so for a player head this matters only for scripted stand-ins.

### What happens without them

- With no `talk` anywhere, kjk moves no lips and `kjk dialogue check` lists the line (`crates/game/src/dialogue_view.rs:1234-1250`, `docs/DIALOGUE.md:648-653`). The head still plays `tlknorm` from the body, so the mouth still moves a little, out of time with the voice.
- With renamed mouth bones, `talk` binds nothing and kjk treats the head as having no mouth.
- reone, by contrast, never looks at the head's own clips. A head with its own `talk` but a body chain without one gets no lip sync in reone.
- Common causes from modders: a replaced `lips` folder (TSLRCM, [Lips not moving](https://deadlystream.com/topic/5896-lips-not-moving-during-conversations/), forum only), the wrong supermodel name, and changed bone names or parents ([Creating custom head](https://deadlystream.com/topic/7590-creating-custom-head/), forum only).

## What expressions and eyes need

### DLG fields

- **`FacialAnim`** is read by both games' loaders and never used. kjk reads it into the node and never reads it back (`crates/fmt_odyssey/src/dlg.rs:328`, `docs/DIALOGUE.md:389-393`). reone does not use it either.
- **`Emotion`** chooses the alien voice sound in KOTOR 2 through `emotion.2da` and `alienvo.2da`. It does not change the face (`crates/fmt_odyssey/src/dlg.rs:314`, `docs/DIALOGUE.md:375-388`).
- **Expressions come from body clips.** A line's `AnimList` plays `tlklaugh`, `tlksad`, `tlkplead`, `tlkforce`, `listen` and, in KOTOR 2, `nodyes`, `nodno`, `rolleyes` and others. These clips key the brows, cheeks, mouth and eyes. A modder sums it up as "the game doesn't have any dedicated facial animations aside from the mouth flapping for VO phonemes" (DarthParametric, [Creating custom head](https://deadlystream.com/topic/7590-creating-custom-head/), forum only).

### Eyes and blinking

- Eyeballs and eyelids are separate rigid meshes under `head_g`. They are not skinned.
- The clips rotate them about their own pivots. A blink turns each lid by about 34°. A glance turns each eyeball by up to about 19°.
- A new head must name them `eyela`, `eyera`, `eyellid` and `eyerlid`, and put each pivot at the centre of its eyeball. A lid whose pivot is not at the eyeball's centre swings off the eye when it blinks.
- A head without eye meshes (Revan's mask, Visas's veil) simply does not blink.

### Head look-at

- **kjk** turns `hturn_g` toward the other speaker of the line, within `appearance.2da`'s `head_arc_h`, at 10 radians a second (`crates/game/src/cast.rs:3701-3709`, `crates/game/src/cast.rs:4297-4304`, `docs/DIALOGUE.md:178-190`). It falls back to `head_g`, then to a node called `head` (`crates/game/src/cast.rs:3918-3930`). The turn is applied after the body's pose.
- **KotOR.js** also turns `hturn_g`, horizontally within `head_arc_h` and vertically within `head_arc_v` ([ModuleCreature.ts](https://github.com/KobaltBlu/KotOR.js/blob/002dfe03b9a59ad2adda6404d9632fc78d5bedff/src/module/ModuleCreature.ts#L2080-L2142)). reone turns whole bodies only.
- **The table.** Every row checked has `head_arc_h` 30 and `head_arc_v` 15. `headtrack` is 1 for players and companions and 0 for commoners, including the Twi'lek rows 446 to 450. `headbone` is blank in every row.
- A new head therefore needs `hturn_g` between `neck_g` and `head_g`, with its pivot where the skull turns on the neck.

## What the sculpting and height plans must respect

### Adding an offset after the face animation

Position keys are offsets on top of each head's own rest (`crates/asset_odyssey/src/anim.rs:166-186`). That makes position sculpting simple.

- **A sculpted position can live in the rest.** If kjk moves a face bone's rest position, every clip still adds its offsets on top. The mouth opens and closes around the new place. This is the same mechanism that lets one `talk` clip serve heads of different shapes.
- **The rest is copied when a clip is bound.** kjk's clip player captures each node's rest when it binds a clip (`crates/game/src/scene.rs:2532-2541`). The face's own rest list is also taken from the head model (`crates/game/src/face.rs:57-67`). A sculpt must therefore change the head's model data before its clips are bound, per character. Changing it later would be undone by the next clip. The other way is to add the offset every frame after both the face clips (`crates/game/src/face.rs:89-118`) and the lip pose (`crates/game/src/dialogue_view.rs:1145-1158`) have run. That needs the systems to run in a fixed order.
- **The skin follows the moved bone.** kjk skins with the file's inverse binds, not the rest pose (`crates/asset_odyssey/src/model.rs:252-266`). Moving a bone's rest away from its bind therefore moves the vertices it weights. That is the sculpt.
- **Rotations must be layered.** Rotation keys replace the rest rotation. A sculpt that turns a bone (for example tilting a brow) must be multiplied onto the rotation after the clip is sampled, every frame.
- **Scale is free.** No clip in either game keys scale on any face node (checked over every resolved clip of both chains). kjk's clip player never writes scale. A scale set on a face bone therefore stays. But it also scales the bone's children. Scaling `f_jaw_g` scales the lower lip halves, the tongue tip and the lower teeth.
- **Children move with parents.** Moving `talkdummy` moves the whole mouth. Moving `f_jaw_g` moves the lower lip, the tongue and the lower teeth. Moving `head_g` moves every face bone and both eyes.

### Keeping the mouth closed

- Shape 0 (rest) and shape 11 (B, M, P) need the lips to meet. The `talk` offsets that close the mouth were made for the stock distance between `f_um_g` and the lower lip bones.
- A sculpt that moves the upper lip and the lower lip by the same offset keeps the mouth closing.
- A sculpt that moves them apart by more than a few millimetres leaves a gap at rest. A sculpt that moves them together by more than that pushes the lips through each other. kjk should link the lip bones (`f_um_g`, `f_llm_g`, `f_rlm_g`, `f_lmc_g`, `f_rmc_g`) in one region for up and down moves, and allow only small differences between them.
- The teeth are rigid meshes that no skin moves. Lips moved backward by more than a few millimetres will show the teeth through them. The tongue is skinned and follows `f_tonguetip_g` and `f_jaw_g`.

### Eyes under sculpting

- The eyeballs and lids are not skinned. A sculpt of the under-eye bones or the brows moves the skin around the eye but not the eye.
- An "eye" region must move `eyela` with `eyellid`, and `eyera` with `eyerlid`, by the same offset, so the lid's pivot stays at the eyeball's centre.
- The eye socket skin is weighted to `f_llweye_g`/`f_rlweye_g`, the brows and `head_g`. Moving the eye more than the socket opens a gap.

### Soft deformation of the rest vertices

- Changing the head mesh's rest vertices works with skinning, because the skin weights and the inverse binds stay as they are. The deformation must be done in the head mesh's own space, before skinning.
- Dangly hair has its own copy of rest positions (`crates/asset_odyssey/src/model.rs:141-150`). A field applied to hair must change both copies.
- The field must fall to zero near `necklwr_g`, whose vertices are fully weighted to it so the neck meets the body (Stormie97, [Creating custom head](https://deadlystream.com/topic/7590-creating-custom-head/), forum only).

### Height and build scaling

- KOTOR's own answer to body size is a different body model with a different rest height and a matching animation scale. A uniformly scaled body should scale its node positions, its vertices, its inverse binds and its animation scale by the same factor. Retargeting then adapts through the pelvis height.
- Root-level hooks (`camerahook`, `freelookhook`, `headconjure`, `handconjure`, `lightsaberhook`, `deflecthook`, `impact_bolt`) must scale with the body.
- The head hangs on `headhook`. A taller body carries the head up with it. A head scaled on its own should be scaled at its root, so its hooks and eyes scale with it.
- `walkdist` and `rundist` must scale with leg length, or the feet slide.
- The hand reach limit of 0.3 m sets how far the arms can grow before the hand and the sim's blade visibly part.

## What alien heads need

### How the existing alien heads are built

- **Twi'lek.** `twilek_f` and `twilek_m` have the standard human face tree. Their lekku are one skinned mesh called `hair`, weighted to `head_g` and `necklwr_g` only. There are no lekku bones and no lekku clips. The lekku bend only because the neck bends between those two bones. The heads have no `gogglehook` or `maskhook`. Their supermodels are the Twi'lek bodies `n_twilekf` and `n_twilekm`, which lead to `s_female03` and `s_female02`. The commoner rows 446 to 450 use the plain commoner bodies `n_commf` and `n_commm` with these heads.
- **Mission.** `p_missionh` skins her lekku (`hair`) to `head_g`, `neck_g` and `necklwr_g`, again with no extra bones. Her head has its own `talk`.
- **KOTOR 2's Twi'lek assassin.** `n_twiassh01` skins its `tentacles` mesh to `head_g`, `necklwr_g` and `neck_g`.
- **Juhani (Cathar).** `p_juhanih` has the standard face tree, a dangly `hair` mesh and its own `talk` clip.
- **Visas (Miraluka).** `p_visash` has no eye meshes and no brow bones. Its face skin uses 13 bones. A dangly `neckwear_geo` mesh hangs below.
- **Bao-Dur (Iridonian).** `p_baodurh` has the standard face tree. His horns are part of the skinned face mesh on `head_g`.

No alien head checked adds a bone or a clip of its own for its alien parts.

### What new alien parts need

- **Montrals and horns** can be rigid meshes under `head_g`, or part of the face skin weighted only to `head_g`. They then turn with the skull. Nothing else is needed.
- **Lekku** can follow the Twi'lek method. They are skinned to `head_g` and `necklwr_g`, with the weight shifting down their length. This needs no new bones and works in the original game. They will not swing.
- **Swinging lekku** can be dangly meshes, as Bastila's hair is. kjk already simulates dangly meshes (`crates/game/src/dangly.rs`). A dangly mesh is not skinned, so it hangs from one node (for example `head_g`) and cannot also bend with the neck.
- **Extra lekku bones** (for example `lekku_l1_g`, `lekku_l2_g`) would be keyed by no clip. They would stay at rest relative to their parent unless kjk animates them, for example with springs. They also count toward the 16-bone limit of the original game's skin meshes. A face skin already uses 16, so lekku with their own bones must be a separate skinned mesh, as the Twi'lek `hair` mesh is.
- **Mask hooks.** A new alien head should keep `gogglehook` at the brow, or kjk will draw no mask or visor on it.

## Checklist for a new or edited head

1. Name the model root after the model, and set its supermodel to `s_female02` for a male human head or `s_female03` for a female human head. An alien head may use its body model as its supermodel, as the Twi'lek heads do, as long as that chain reaches the human supermodels.
2. Keep the chain `cutscenedummy → rootdummy → torso_g → torsoupr_g → necklwr_g → neck_g → hturn_g → head_g → talkdummy` with these exact names. The upper spine nodes stay at the origin.
3. Set the animation root to `neck_g`.
4. Put `f_um_g`, `f_lmc_g`, `f_rmc_g` and `f_jaw_g` under `talkdummy`. Put `f_llm_g`, `f_rlm_g`, `f_tonguetip_g` and the lower teeth under `f_jaw_g`.
5. Put `f_mdbrw_g`, `f_lbrw_g`, `f_rbrw_g`, `f_llweye_g`, `f_rlweye_g`, `f_lns_g`, `f_rns_g`, `eyela`, `eyera`, `eyellid`, `eyerlid`, the upper teeth, `gogglehook` and `maskhook` under `head_g`.
6. Place each face bone near where the stock heads place it. The clips add offsets of up to 3 cm to wherever the bone rests.
7. Make the eyes and lids separate rigid meshes, each with its pivot at its eyeball's centre.
8. Make the face skin a child of the model root. Weight it to at most 16 bones, with at most 4 bones per vertex. Weight the lowest ring of neck vertices fully to `necklwr_g`.
9. Weight the lips so that the rest pose shows a closed mouth.
10. Skin the tongue to `f_tonguetip_g` and `f_jaw_g`.
11. Put lekku, montrals and long hair in their own meshes, skinned to `head_g` and the neck bones, rigid under `head_g`, or dangly.
12. Write the skin's inverse binds (`qbones`, `tbones`) from the rest pose.
13. Give the head a row in `heads.2da` with its dark side textures, and point an `appearance.2da` row's `normalhead` at it.
14. Check it in kjk. `kjk dialogue check` reports a voiced speaker whose lips cannot move, and `kjk formats` reports inverse binds that do not match the rest pose.

## Checklist for a new or edited body

1. Keep the KOTOR humanoid node names: `rootdummy`, `pelvis_g`, `torso_g`, `torsoupr_g`, both collars, upper arms, forearms, hands, thighs, shins and feet, and the finger bases `rafngrb_g`, `rdfngrb_g`, `lafngrb_g` and `ldfngrb_g`.
2. Set the supermodel to `s_female02` for a man's body or `s_female03` for a woman's body.
3. Face +Y with Z up. Hold the spine and feet upright and flat at rest. The arms may rest in any pose.
4. Add `headhook` under `torsoupr_g`, with `necklwr_g` and `neck_g` under it.
5. Add `rhand` and `lhand` under the hands, `impact` under `torsoupr_g`, and `camerahook`, `freelookhook`, `headconjure`, `handconjure`, `lightsaberhook`, `deflecthook` and `impact_bolt` under the model root, at heights that fit the body.
6. Split the skin into meshes of at most 16 bones each, with at most 4 bones per vertex.
7. Set the animation scale to the body's size relative to the stock body of the same sex.
8. Set `walkdist` and `rundist` in `appearance.2da` to match the legs.
9. Match the bare skin texture to the head's skin colour.

## Summary of the strongest findings

1. Every human head in both games has the same 16 skin bones and the same hierarchy. The 4 mouth bones sit under `talkdummy`, and the lower lip and tongue sit under `f_jaw_g`.
2. Clip position keys are offsets added to each head's own rest. Rotation keys are absolute. No clip keys scale on a face node.
3. The `talk` clip is 0.5 s long, keys only the 7 mouth nodes, and holds one pose per shape at 1/30 s intervals.
4. kjk and reone sample `talk` at shape ÷ 16 of its length, which lands between keys. The data suggests shape ÷ 15. This is a probable kjk bug in `crates/game/src/dialogue_view.rs:1153`, to be confirmed against the original executable.
5. Blinking, glances and expressions all come from the body's shared clips (`pause1`, `tlknorm`, `listen` and others), not from the head or the DLG. `FacialAnim` is never used.
6. Head look-at in kjk and KotOR.js turns `hturn_g`, within 30° horizontally.
7. Male human heads use `s_female02` and female heads use `s_female03`. Both chains end in `s_male02` and `s_male01`.
8. Bone-offset sculpting is compatible with every clip as long as kjk applies it to each character's rest before its clips are bound. Rotation sculpts must be layered after sampling. The lips must move together to keep the mouth closing.
9. No alien head adds a bone or a clip. Twi'lek lekku are a skinned mesh on `head_g` and `necklwr_g`.
10. Male player bodies have no `neck_g`, so kjk's spine-by-height rule falls back to the older pairing on them.
