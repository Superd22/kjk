# Which cape and robe keys are flourishes and which fake physics

This note was written on 2026-11-19 for the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts"). The owner's rule is that keys which only fake physics can be dropped once a solver moves the loose bones, and that keys which are real flourishes must stay. This note lists every clip of both games that keys a loose bone, measures the keys, sorts each clip by a written rule, and names the cutscenes. It follows `docs/research/2026-11-19-loose-parts-census.md` and `docs/research/2026-11-19-robe-bone-animation.md`.

## Summary

- 920 clips key at least one loose bone. KOTOR 1 has 156 of them and KOTOR 2 has 764. Some models exist in both games with the same clips (`n_darthmalak`, `n_darthrevanm`, `n_darthrevanf`, `c_ithorian`, `c_holorakata`, `n_rakata`, `mainmenu`, `mainmenu_model` and `3dgui`), so they are counted once per game.
- 894 clips fake physics (145 in KOTOR 1 and 749 in KOTOR 2). Of these, 55 are keyed but nearly static, because the bone never moves more than 15 degrees from its hang pose.
- 14 clips are flourishes, all in KOTOR 2, and all of them pull a hood up or down. They are listed one by one below.
- 12 clips are unclear: 11 clips of Malak in the KOTOR 1 cutscene `m44aa_c02`, and the lightsaber throw loop `throwsablp` of Darth Nihilus in KOTOR 2. They are listed one by one below.
- No clip of any body model, in either game, holds a cape or a robe hem above the horizontal for as long as one second. No clip throws a cape back and holds it. The keys that artists made for capes and robes in this data are therefore almost all follow-through of the body, and the owner's rule lets a solver replace them.
- The classification is firm for the 749 + 145 fakes and the 14 hood clips. It is weak for the 12 unclear clips, which need a person to watch them.

## Method

### Finding the loose bones

A small Rust program (kept outside the repository, see "Tool") read every model of both installs with `Model::parse` and `ModelIr::from_model`, taken from `chitin.key`, the override folder, `Patch.erf`, `modules`, `rims` and `lips`. It dumped every node (with its parent and the meshes skinned to it) and every track of every clip. Two things were then done with that dump.

- All animated node names were listed. KOTOR 1 has 2,633 different animated names and KOTOR 2 has 3,384. Every name that contains `robe`, `cape`, `cloth`, `flap`, `belt`, `hood`, `sleeve`, `sash`, `shirttie` or `shldr` was read together with the model it is on.
- All joints that a mesh is skinned to, in models whose supermodel chain reaches `s_male01`, `s_male02`, `s_female01`, `s_female02` or `s_female03`, were compared with the joints of the plain skeleton (the nodes of `s_male01` and `s_female01` that are not robe bones). Every extra joint was read, to find garment bones with names outside the list above.

The bones that were counted as loose are these.

- Robe bones: `frobe1_g`, `frobe2_g`, `mrobe1_g` to `mrobe3_g`, `lrobe1_g` to `lrobe3_g`, `rrobe1_g` to `rrobe3_g`, and `l_robe_g` and `r_robe_g` on the Darth Revan models.
- Cape bones: `cape01_g` to `cape05_g`, `cape02_g01` to `cape02_g03`, the KOTOR 2 Nihilus bones `lcape01` to `lcape04`, `mcape01` to `mcape04` and `rcape01` to `rcape04`, and the two nodes `cape` and `cape01` of `m15aa_c01_char02`.
- Cloth bones: `cloth_1_g` to `cloth_27_g` on Malak, `cloth` on `p_juhani`, and `lclothflap_g` and `rclothflap_g` on `c_ithorian`.
- Flap bones: `flap_g` on the Rakata and Ithorian models, `lflap` and `rflap` on `m40ad_c01_char01`, `bflap01_g` to `bflap05_g`, `lflap01_g` to `lflap05_g`, `rflap01_g` to `rflap05_g` and `fflap01_g` to `fflap04_g` on Bastila's dress, and `frontflap` on `p_joleebb`.
- Other garment bones: `belt01_g` to `belt05_g` on Revan, `lshldr01_g` to `lshldr03_g` and `rshldr01_g` to `rshldr03_g` on Bastila, the hood bones (`hood`, `hood_g`, `hoodl_g`, `hoodr_g`, `mhood_g`, `mfhood_g`, `lhood_g`, `rhood_g`), the KOTOR 2 sleeve bones (`lsleeve1_g`, `lsleeve2_g`, `rsleeve1_g`, `rsleeve2_g`), `sash_geo` on Visas and `shirttie` on Mira.

The census note did not count the hood bones, the sleeve bones, the flaps, the belts and the shoulder bones. They are included here because they only exist to move garments.

These animated bones were left out on purpose. They are wing flaps (`lwingflap_g`, `rwingflap_g`), ship engine flaps (`g_l_engflap*`), door flaps (`dflap*`, `doorflap02`), the flaps on the head of the Selkath (`lflapupr_g` and similar, which are skinned to the head mesh and are skin, not cloth), the flaps of the droid `c_drdmktwo`, hair flaps, Wookiee fur, the Ithorian backpack, the Rodian antennae, creature tails, probe droid arms and the `wiggledummy` effect nodes.

### Measuring a clip

Each clip was played on the model that holds it, in steps of 1/30 second, with `anim::pose` from `crates/asset_odyssey/src/anim.rs`. For a clip of a supermodel such as `s_male02`, the clip is posed on the supermodel itself.

The hang pose of a bone is the first key of the clip `pause1` of the same model. A model without its own `pause1` uses the `pause1` of `s_male02` for robe bones and the `pause1` of `n_darthmalak` for cloth bones. A bone that neither provides stays at its rest pose.

For every loose bone the program measured these values.

- Local angle: the angle between the bone's keyed rotation and its hang rotation, relative to the bone's parent.
- Angular speed: how fast that rotation changes, in degrees per second.
- World lift: the angle between the direction of the bone in the model's space and straight down. For a bone that ends a chain (a leaf, meaning no other loose bone hangs below it), the direction runs from its parent to it. Leaf bones were used because the first bones of a chain sit at the shoulder or the hip and legitimately point sideways or forward.
- Body motion: the speed of the pelvis, torso, head and both hands in the model's space, and the turning speed of the pelvis. The body counts as still when the speed is below 0.15 metres per second and the turning is below 15 degrees per second for at least 7 frames in a row. It counts as slow below 0.5 metres per second.
- Standing: the pelvis is higher than 85 percent of its rest height. Sitting, kneeling and lying bodies are left out of the lift measures, because a robe on a seated body lies along the thighs and is not lifted.

### The rule

A clip is a flourish if any of these holds.

1. A hood bone turns 90 degrees or more from its hang pose. This is a hood pulled up or down.
2. A leaf bone of a cape or robe stays 90 degrees or more from straight down (at or above the horizontal) for one second or longer while the body is standing and slow, and its angular speed stays below 15 degrees per second. This is a cape or robe held up against gravity.
3. A leaf bone is 120 degrees or more from straight down while the body is still. This is a large motion that the body does not explain.

A clip is unclear if it is not a flourish and any of these holds.

1. A cape, robe or cloth bone stays 60 degrees or more from its hang direction in the world for one second or longer while the body is slow and standing.
2. A bone of that kind is 90 degrees or more from its hang direction while the body is still.
3. A leaf bone is 70 degrees or more from straight down while the body is still.

Every other clip fakes physics. These clips show angles that follow the body's own motion: the front robe lifts with the thigh, the side robes swing out when the body turns, and the bones lag behind and fall back toward the hang pose after the body stops.

Two clips matched the unclear rule but were moved to "fakes physics" after reading their angle curves. They are `c2a6` of `s_female02` and `f2p3a` of `s_male01`. In both, the robe swings up to about 80 degrees only in the moment after the body stopped moving, and then falls back to about 35 degrees within a second. This is follow-through of the body, which is what the owner's rule says to drop.

Sleeve bones were measured but are not used by the rule, except that they never decide a flourish. They are skinned to the forearm, and their angle follows the arm. The `s_female01` sleeves, for example, sit about 180 degrees from the `s_male02` hang pose in every clip, which is a difference of rest convention and not a gesture.

## Totals

| Game | Clips | Fake physics | Flourish | Unclear |
|---|---|---|---|---|
| KOTOR 1 | 156 | 145 | 0 | 11 |
| KOTOR 2 | 764 | 749 | 14 | 1 |
| Both | 920 | 894 | 14 | 12 |

## Clips per model

"Fake" means fakes physics. All counts are clips that key at least one loose bone.

### KOTOR 2

| Model | Loose bones | Clips | Fake | Flourish | Unclear |
|---|---|---|---|---|---|
| `s_male02` | 11 robe bones and 4 sleeve bones | 226 | 226 | 0 | 0 |
| `s_male01` | same | 65 | 65 | 0 | 0 |
| `s_female01` | same | 76 | 76 | 0 | 0 |
| `s_female02` | same | 84 | 84 | 0 | 0 |
| `s_female03` | same | 66 | 66 | 0 | 0 |
| `n_darthnihilus` | `lcape01` to `rcape04` | 92 | 91 | 0 | 1 |
| `n_darthmalak` | 27 cloth bones | 66 | 66 | 0 | 0 |
| `pfbim`, `pfbnm` | robe and sleeve bones | 5 each | 10 | 0 | 0 |
| `n_darkjhif` | 9 robe bones | 5 | 5 | 0 | 0 |
| `p_kreiabb`, `p_kreia1hbb`, `p_kreiaevilbb`, `p_kreiaevil0hbb` | hood and sleeve bones | 9, 5, 5 and 6 | 19 | 6 | 0 |
| `p_kreiaevil02` | hood and sleeve bones | 5 | 1 | 4 | 0 |
| `p_kreia1hstunt01`, `p_kreiastunt` | hood and sleeve bones | 1 and 2 | 3 | 0 | 0 |
| `p_handmaidenbb`, `p_handmaidenbd` | hood bones | 2 each | 0 | 4 | 0 |
| `p_visasbb`, `p_mirabb` | `sash_geo`, `shirttie` | 3 each | 6 | 0 | 0 |
| `c_ithorian` | 3 cloth flap bones | 16 | 16 | 0 | 0 |
| `c_holorakata`, `n_rakata` | `flap_g` | 6 and 4 | 10 | 0 | 0 |
| `n_darthrevanm`, `n_darthrevanf` | cape, belt and robe bones | 1 each | 2 | 0 | 0 |
| `mainmenu`, `mainmenu_model`, `3dgui` | cape and flap bones | 1 each | 3 | 0 | 0 |

The 517 clips of the five body supermodels are the ones that the census counted for `s_male02` and `s_female03`. The other supermodels `s_male01`, `s_female01` and `s_female02` carry robe tracks as well.

### KOTOR 1

| Model | Loose bones | Clips | Fake | Unclear |
|---|---|---|---|---|
| `n_darthmalak` | 27 cloth bones | 66 | 66 | 0 |
| `m44aa_c02_char02` | 27 cloth bones | 16 | 5 | 11 |
| `m15aa_c01_char04` | cape, belt, hood and robe bones | 7 | 7 | 0 |
| `m15aa_c01_char02` | `cape`, `cape01` | 1 | 1 | 0 |
| `m40ab_c03_char01` | `cape01_g` to `cape02_g03` | 6 | 6 | 0 |
| `m40ad_c01_char01` | cape bones and `lflap`, `rflap` | 10 | 10 | 0 |
| `m45ad_c01_char01` | cape bones | 7 | 7 | 0 |
| `m44ac_c01_char01` | flaps and shoulder bones of Bastila's dress | 6 | 6 | 0 |
| `m44ac_c02_char01` | cape, belt and robe bones | 1 | 1 | 0 |
| `m41ad_c01_char15` | `hood` | 2 | 2 | 0 |
| `c_ithorian`, `c_holorakata`, `n_rakata` | cloth flaps and `flap_g` | 16, 6 and 4 | 26 | 0 |
| `p_joleebb`, `p_juhani` | `frontflap`, `cloth` | 1 and 2 | 3 | 0 |
| `n_darthrevanm`, `n_darthrevanf` | cape, belt and robe bones | 1 each | 2 | 0 |
| `mainmenu`, `mainmenu_model`, `3dgui` | cape and flap bones | 1 each | 3 | 0 |

KOTOR 1 has no robe tracks in its body supermodels, so every KOTOR 1 clip above belongs to a creature, an NPC model or a cutscene model.

## Every flourish

All 14 flourishes are hood clips of KOTOR 2. The hood bones move from 140 to 155 degrees from the hang pose, which is a hood pushed back off the head or pulled up over it. A solver cannot create that, so these keys must stay.

| Model | Clip | Length in seconds | Hood angle | What it is |
|---|---|---|---|---|
| `p_kreiabb` | `onhood` | 1.5 | 140 | Kreia pulls her hood up |
| `p_kreiabb` | `offhood` | 1.5 | 140 | Kreia pushes her hood back |
| `p_kreia1hbb` | `onhood` | 1.5 | 140 | the same for the one-handed variant |
| `p_kreia1hbb` | `offhood` | 1.5 | 140 | the same |
| `p_kreiaevilbb` | `onhood` | 1.5 | 140 | the same for the evil variant |
| `p_kreiaevilbb` | `offhood` | 1.5 | 140 | the same |
| `p_handmaidenbb` | `onhood` | 1.5 | 146 | a handmaiden pulls her hood up |
| `p_handmaidenbb` | `offhood` | 1.5 | 140 | a handmaiden pushes her hood back |
| `p_handmaidenbd` | `onhood` | 1.5 | 146 | the same for the other handmaiden model |
| `p_handmaidenbd` | `offhood` | 1.5 | 140 | the same |
| `p_kreiaevil02` | `cut001w` | 6.7 | 155 | cutscene, hood held down while Kreia speaks |
| `p_kreiaevil02` | `cut002wl` | 1.7 | 155 | cutscene, looped, hood held down |
| `p_kreiaevil02` | `cut003w` | 4.7 | 155 | cutscene, hood held down |
| `p_kreiaevil02` | `cut004w` | 17.7 | 155 | cutscene, hood held down |

The ten `onhood` and `offhood` clips are the same two gestures copied onto five models. The four `p_kreiaevil02` clips hold the hood in one pose for the whole scene. That is a pose, and a solver would drop the hood onto the shoulders or let it hang, so the keys must stay.

The cutscene is the dialog `kreiaend` of the module `907MAL` (Malachor V). It is Kreia's final monologue ("There are dark places in the galaxy where few tread..."). Its stunt list names the participant `kreiaevil` on the model `p_kreiaevil02`, a stunt lightsaber and `n_darthsion03`. The dialog has no video link found online (see "Videos").

## Every unclear clip

### Malak in the KOTOR 1 cutscene `m44aa_c02` (11 clips)

These clips are all on the model `m44aa_c02_char02`, which is Darth Malak in the stunt module `STUNT_18` (dialog `m44aa_c02`, see "Cutscenes"). Malak's cloak is made of 27 cloth bones. In these clips some cloth bones are held at a fixed angle from the hang pose, between 60 and 122 degrees, for the whole clip, while Malak barely moves. Their directions in the world still point mostly downward (about 15 to 45 degrees from vertical), so the cloak is not held up against gravity. Nothing swings or oscillates. This looks like a hand-posed arrangement of the cloak that suits Malak's raised-arm poses, and it may be a correction and not a gesture.

| Clip | Length in seconds | What the numbers show |
|---|---|---|
| `cut001w` | 2.3 | Body fully still. `cloth_1_g` and others settle from the hang pose to 98 degrees away over the clip. |
| `cut005w` | 3.8 | `cloth_15_g` held 122 degrees from hang for the whole clip, `cloth_10_g` 79 and `cloth_23_g` 81. All still hang downward. |
| `cut006w` | 12.1 | The same fixed offsets for 12 seconds. `cloth_1_g`, the collar bone, points upward (world direction 0.47 up) while Malak stands almost still. |
| `cut007w` | 6.1 | Malak turns (up to 440 degrees per second) and the collar bone swings 132 degrees from straight down. This is probably follow-through, but it is long. |
| `cut008w` | 5.5 | Fixed offsets for 2.8 seconds while the body is still for 43 percent of the clip. |
| `cut009w` | 2.6 | `cloth_15_g` 122 degrees from hang, body still for 62 percent of the clip. |
| `cut012w` | 2.6 | The collar bone is 115 degrees from its hang direction while the body is mostly moving. |
| `cut013w` | 1.3 | Body still. The collar bone starts 168 degrees from its hang direction (pointing up and forward) and falls to 125 degrees over 1.3 seconds. This is the clip most likely to be a real flare of the cloak. |
| `cut014w` | 14.5 | Fixed offsets for 2.5 seconds, collar bone up to 130 degrees. |
| `cut015w` | 2.7 | Collar bone 92 degrees from its hang direction during fast body motion. |
| `cut016w` | 5.8 | Collar bone 118 degrees from its hang direction, body still for 34 percent of the clip. |

The five clips of this model that were not flagged (`cut002w`, `cut003w`, `cut004w`, `cut010w` and `cut011w`) stay within 20 to 88 degrees of the hang pose and move with Malak, so they fake physics.

### Darth Nihilus in KOTOR 2 (1 clip)

- `n_darthnihilus`, `throwsablp`, 0.9 seconds. This is the loop that Nihilus plays while holding a thrown lightsaber. The body is still (below 0.2 metres per second). The 12 cape bones are held 35 to 60 degrees from the hang pose for the whole clip, and the world direction of the hem bones is 55 to 80 degrees from straight down, with only small jitter. The cape is therefore held away from its hang direction, possibly because his arms are out. It is a held pose with no swing, so a person should watch it.

## Cutscenes

These are the cutscene models whose clips key loose bones. All are in KOTOR 1 except the Kreia scene. The KOTOR 1 cutscenes are stunt modules (`modules/STUNT_*.rim`). Each stunt module holds a dialog whose `StuntList` names the stunt models and whose lines play the clips `cut001w`, `cut002w` and so on in order. The tables below came from those dialogs, which were read with kjk's own dialog reader. The names of the dialogs and the lines they quote were read from the files.

| Model | Cutscene | Module and dialog | Who is in it | Clips |
|---|---|---|---|---|
| `m44aa_c02_char02` | Malak tortures Bastila | `STUNT_18`, dialog `m44aa_c02` | Malak (`darthmalak`) and Bastila (`m44aa_c02_char01`). Malak says "You are strong, child. But I will break you." and "Become my apprentice, and all its power can be yours!" | `cut001w` to `cut016w`, 16 clips |
| `m44ac_c01_char01` | Dark side ending, Sith ceremony | `STUNT_55a`, dialog `end_55a` | Bastila (`bastila`) and the player (`m44ac_c01_char02`). Bastila says "Malak is dead - all hail the return of Darth Revan, the true Lord of the Sith!" | `cut001w` to `cut006w`, 6 clips (flap and shoulder bones of her dress) |
| `m44ac_c02_char01` | Darth Revan appears | `STUNT_31b`, dialog `31b_convers` | Darth Revan (`darthrevan`), one clip of 23 seconds | `cut001w` |
| `m45ad_c01_char01` | Malak and Admiral Karath on the Star Forge | `STUNT_19`, dialog `19` | Malak, Admiral Saul Karath (`m45ad_c01_char02`) and a probe droid. Malak says "Patience, Commander. My new apprentice is nearly ready." | `cut001w` to `cut007w`, 7 clips |
| `m40ab_c03_char01` | Taris bombardment, first scene | `STUNT_06`, dialog `scene_06` | Malak and Karath. Karath says "Taris is defenseless against our assault, Lord Malak." and Malak says "Resume the bombardment, Commander." | `cut001w` to `cut006w`, 6 clips |
| `m40ad_c01_char01` | Malak orders the destruction of Taris | `STUNT_03a`, dialog `m40ab_c01` | Malak and Karath. Malak says "The search for Bastila is taking too long." Karath asks "Th... the entire planet, Lord Malak?" | `cut001w` to `cut010w`, 10 clips |
| `m41ad_c01_char15` | Medal ceremony after Malak's defeat | `STUNT_57`, dialog `m41ad_c01` | Yoda, Dodonna, the whole party, with Jolee on `m41ad_c01_char15`. Dodonna says "You have defeated Malak, destroyed the Star Forge and broken the spirit of the Sith!" | `cut018w`, `cut021w` (the `hood` node of Jolee) |
| `m15aa_c01_char04`, `m15aa_c01_char02` | Dantooine ancient temple, star map vision | module `danm15` | Darth Revan (belt, cape, hood and robe bones) and a second figure. No dialog names these stunt models, so a script must start them. This is probably the star map vision in which Bastila says "We saw Revan and Malak speaking of the Star Forge". | `char04`: `cut001w` to `cut007w`, 7 clips. `char02`: `cut001w`, 1 clip |
| `p_kreiaevil02` | Kreia's last monologue | KOTOR 2 module `907MAL`, dialog `kreiaend` | Kreia and Darth Sion | `cut001w` to `cut005wl`, 5 clips |

Notes about the matching.

- The Dantooine row is the least certain. The module name `danm15` holds the model name, and the Dantooine temple dialog `dan15_bastila` has Bastila describe the vision. No dialog refers to `m15aa_c01_char04`, so the link rests on the module and the story, and was not run in the game.
- The ending, Taris and Star Forge identifications rest on the dialog text and the participant tags, which are exact. Which of these plays as an in-engine stunt and which as a prerendered movie was not checked. The folder `movies` of the KOTOR 1 install has numbered Bink files (`55.bik`, `31a.bik`, `06a.bik` and others) whose numbers match some stunt modules, but the match was not verified.
- The cutscene clips of `m40ab_c03_char01`, `m40ad_c01_char01`, `m45ad_c01_char01` (Malak at rest on his bridge) and `m44ac_c01_char01` (Bastila) show no loose motion beyond 60 degrees. The capes hang downward all the time, with small angles while the body is still, so they fake physics or are static.

### Videos

Two web searches for the Malak and Bastila torture scene and for the dark side ending did not return any video link. No video is named in this note. Searching YouTube for "KOTOR Bastila Malak torture cutscene" and "KOTOR dark side ending" will probably find the two scenes.

## What this means for the roadmap

- The solver can replace the keys of all 894 clips that fake physics. This covers all robe bones of the five KOTOR 2 body supermodels, all cloth bones of Malak and the Ithorian and Rakata flaps. The 55 near-static clips would simply leave the bone at its rest pose.
- The keys of the 14 hood clips must stay. Hood bones should therefore not be handed to the solver while a hood clip plays, or the solver should start from the keyed pose.
- The 11 Malak cutscene clips of `m44aa_c02` and the Nihilus clip `throwsablp` need someone to watch them. If the solver is used there, the safe choice is to keep the keyed bones of the cloak in those cutscene clips, since the fixed offsets look hand-posed.
- The flourishes that the owner described (throwing a cape back, gripping it, wrapping it around the body) are absent from this data. The rule found none, and the cutscene capes were all hanging down or only following the body. If such a gesture exists, it is on a bone that is not named like a garment, so this note cannot rule it out.

## Limits

- The classification is by numbers. Nothing was drawn on screen and no clip was watched.
- The hang pose of a model without `pause1` is the rest pose, or the `pause1` of `s_male02` or `n_darthmalak` for matching bone names. The cutscene models `m44aa_c02_char02`, `m15aa_c01_char04` and the Revan models have no `pause1` of their own. Their results depend on that choice, and the Malak cloth bones changed between rules when this choice was changed. That is why those clips are unclear and not flourishes.
- "Body still" is judged from the pelvis, torso, head and hands. A clip with a still body can still have a moving arm that lifts a cape.
- The thresholds (45, 60, 70, 90 and 120 degrees, 0.15 and 0.5 metres per second, one second) are judgment values. They were chosen after looking at the distributions of the 920 clips. The biggest hold of any cape or robe leaf above the horizontal in a body clip was 0.07 second. The rule's result therefore does not change if the hold time is lowered to 0.5 second, but it would change for the 12 unclear clips.
- Two clips were moved by hand, as described under "The rule".
- Where a model exists in several archives, only the first copy was measured.
- The cutscene identification came from dialogs. Stunt models that no dialog names (the Dantooine ones) were matched by module only.

## Tool

The program and its outputs are in the scratchpad, outside the repository. Nothing in the repository was changed.

- Source: `/private/tmp/claude-501/-Users-david-perso-kjk/5fbe746b-0603-4318-a877-f1ce4292c9c8/scratchpad/cen3/src/main.rs` (commands `rows`, `bones`, `curve`, `dlg`, `arch` and `tda`).
- Per-clip numbers: `q1.tsv` (KOTOR 1) and `q2.tsv` (KOTOR 2) in the same folder. The rule is in `classify.py` there.
- Dump of every node and track: `w1.tsv` and `w2.tsv` in the same folder.
