# Loose parts in the KOTOR 1 and KOTOR 2 character models

This note was written on 2026-11-19 for the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts"). It answers one question: which character models of the two games have loose parts, what kind of parts they are, and what data kjk could use to move them. Everything here was measured from the installed games with kjk's own readers.

## Summary

- KOTOR 1 has 51 character models with at least one danglymesh, and KOTOR 2 has 81. These counts are model files. They include size variants such as `pmbjl`, `pmbjm` and `pmbjs`, which share one hood.
- Most danglymeshes are hair. KOTOR 1 has 42 distinct hair meshes on 32 models, and KOTOR 2 has 83 on 56 models. Both games together have fewer than 30 danglymeshes on bodies. Capes, hoods, skirts and flaps are therefore rare in the data.
- The cloth danglymeshes are small. A body cloth mesh has between 10 and 57 vertices, a hood has 76 to 104, and a cape has 30 or 33. The two biggest cloth meshes are Juhani's cloth (48 vertices, 12.5 cm limit) and the 5 cm and 7.5 cm capes on `n_jedimalek`.
- In both games about 30 to 64 percent of the vertices of a danglymesh have constraint 0, so they sway freely. The rest have constraints between 64 and 253. No vertex anywhere has constraint 255. This means the hem of a cloth mesh is free and its top edge is held, which is what a flap needs.
- The real robes are not danglymeshes at all. KOTOR 1's Jedi robes (`pmbj*`, `pfbj*`) put the whole robe in the `legs` mesh, and only the hood sways. In KOTOR 2, 11 body models have a robe or skirt mesh that is skinned to a chain of 11 dedicated robe bones (`frobe1_g`, `frobe2_g`, `mrobe1_g` to `mrobe3_g`, `lrobe1_g` to `lrobe3_g` and `rrobe1_g` to `rrobe3_g`). The clips of the supermodels keyframe those bones for KOTOR 2's own animations. KOTOR 1 has no robe bone tracks in its supermodels.
- This changes the answer to the roadmap question "should a robe skirt sway where the data has no danglymesh". For the KOTOR 2 robes, the skirt vertices can be found exactly, because they are the vertices weighted to the robe bones. For KOTOR 1's Jedi robes, skin weights alone do not find the skirt cleanly, as section "Skirts without a danglymesh" shows.
- The danglymesh data cannot carry a wide robe. A cape or robe that flies in a spin or a flip needs motion far larger than the 2.5 to 12.5 cm box of the data, so any new motion for robes must come from the skin weights, the robe bones or a new simulation, not from the existing danglymesh values.
- Outside characters, the data has many danglymeshes. KOTOR 1 has 2,444 leaf meshes in 22 area models (Kashyyyk), plus tarps, tents, hides, shrubs, a palm and a flag. KOTOR 2 has 567 leaf meshes in 5 Dantooine area models, plus 16 flag meshes in 6 models and a few shrubs.

## Method

A small Rust program built on `crates/fmt_odyssey` and `crates/asset_odyssey` (`Key`, `Bif`, `Erf`, `Rim`, `Model::parse` and `ModelIr::from_model`) opened both installs read only. The same program design was used by the head study in `docs/research/2026-11-18-kotor-head-meshes.md`. The program was kept outside the repository and nothing in the repository was changed.

- It read every `.mdl` listed in `chitin.key`, every model in the override folder and every model in `Patch.erf`, `modules`, `rims` and `lips`. KOTOR 1 gave 2,984 model entries and KOTOR 2 gave 3,239. After removing repeats of the same name there are 2,832 and 3,237 different models.
- All danglymeshes of both games live in models from `chitin.key`. No module archive or override file adds a danglymesh.
- 88 entries in KOTOR 1 and 1 entry in KOTOR 2 could not be parsed. The 88 are almost all `pfha01` to `pfha05`-style player head models in `chargen.rim`. The same names also load from `chitin.key`, so no model is missing from the counts. The one KOTOR 2 failure is `w_rockpoisn`.
- A danglymesh counts as belonging to a character when its parent node is a body or head bone (`head_g`, a face bone `f_*`, `pelvis_g`, `torso_g`, `torsoupr_g`, a thigh or a forearm). All other danglymeshes are reported as non-character.
- The part type (hair, cape, hood and so on) was chosen from the node name and the parent bone. A few names are ambiguous, for example `object` under `pelvis_g`. Those were put under "belt, skirt or leg flap".
- Model units are metres. The skinned vertices of a mesh are stored in the space of the mesh node, so the program applied the node's rest matrix before comparing heights and positions.
- The constraint values are the 0 to 255 numbers of the file. The bins used below are 0, 1 to 63, 64 to 127, 128 to 254 and 255.

## Totals per game

The tables count model files, meshes and vertices. "Models" counts copies of a model, such as the three size variants of one robe, as separate models. The "distinct meshes" column of the later tables removes copies that have the same node, vertex count, texture and constraints.

### KOTOR 1

KOTOR 1 has 51 character models with a danglymesh. 32 have danglymeshes only on the head, and 19 only on the body. No model has both. Three of the models are cutscene copies named like `m15aa_c01_char02`.

| Part type | Models | Meshes | Vertices | Vertices per mesh | Displacement | Tightness | Period |
|---|---|---|---|---|---|---|---|
| Hair and head parts | 32 | 42 | 4,953 | 3 to 428 | 0.015 to 0.15 | 0.5 to 3 | 1 to 8 |
| Hood | 7 | 7 | 560 | 76 to 104 | 0.025 | 3 | 1 to 8 |
| Belt, skirt or leg flap | 7 | 11 | 397 | 12 to 57 | 0.025 to 0.03 | 0.8 to 4 | 3 to 8 |
| Fur flaps (Wookiee) | 3 | 4 | 76 | 14 to 24 | 0.02 to 0.025 | 0.9 to 1 | 4 to 8 |
| Cape | 2 | 4 | 126 | 30 to 33 | 0.05 to 0.075 | 1.5 | 8 |
| Juhani's cloth | 2 | 2 | 96 | 48 | 0.125 | 1.75 | 7 |
| Shoulder flaps | 1 | 1 | 44 | 44 | 0.025 | 0.85 | 3 |

### KOTOR 2

KOTOR 2 has 81 character models with a danglymesh. 62 have danglymeshes only on the head, and 19 only on the body. No model has both. There are no cutscene copies.

| Part type | Models | Meshes | Vertices | Vertices per mesh | Displacement | Tightness | Period |
|---|---|---|---|---|---|---|---|
| Hair and head parts | 56 | 83 | 7,078 | 4 to 428 | 0.015 to 0.15 | 0.1 to 3 | 1 to 10 |
| Belt, skirt or leg flap | 7 | 11 | 395 | 12 to 57 | 0.025 to 0.03 | 0.8 to 4 | 3 to 8 |
| Hood | 6 | 6 | 456 | 76 | 0.025 | 3 | 8 |
| Fur flaps (Wookiee) | 4 | 6 | 104 | 14 to 24 | 0.02 to 0.025 | 0.9 to 1 | 4 to 8 |
| Ears (Gran) | 2 | 3 | 142 | 22 to 96 | 0.025 | 0.5 to 1.75 | 5 to 7 |
| Earrings | 2 | 2 | 86 | 18 to 68 | 0.025 | 0.9 to 1 | 1 to 4.5 |
| Sash or arm streamer | 2 | 4 | 99 | 10 to 30 | 0.025 | 1 to 1.2 | 3 to 4.5 |
| Jacket or shirt flap | 1 | 2 | 47 | 11 to 36 | 0.025 | 0.75 to 1.2 | 2 to 3.5 |
| Head tentacles (Quarren) | 1 | 4 | 168 | 38 to 46 | 0.045 | 0.65 | 5.5 |
| Cape | 1 | 2 | 63 | 30 to 33 | 0.05 to 0.075 | 1.5 | 8 |
| Breathing gear (Gand) | 1 | 1 | 101 | 101 | 0.025 | 0.2 | 5.5 |
| Beard (a hologram) | 1 | 1 | 22 | 22 | 0.025 | 1.75 | 2.12 |
| Shoulder flaps | 1 | 1 | 44 | 44 | 0.025 | 0.85 | 3 |

KOTOR 2 reuses all but a few of KOTOR 1's loose parts unchanged. The Jedi hood, the Wookiee flaps, the `n_jedimalek` cape and Bastila's flaps appear in both games with the same numbers.

### Constraint distribution

Each cell is the share of the vertices of the distinct meshes of that part type, in percent. No vertex has constraint 255 in either game, so that bin is left out. The "largest" column is the highest constraint seen.

| Game | Part type | Distinct meshes | Vertices | 0 | 1 to 63 | 64 to 127 | 128 to 254 | Largest |
|---|---|---|---|---|---|---|---|---|
| KOTOR 1 | Hair and head parts | 42 | 4,953 | 50 | 0 | 0 | 49 | 253 |
| KOTOR 1 | Hood | 2 | 180 | 51 | 0 | 0 | 49 | 238 |
| KOTOR 1 | Belt, skirt or leg flap | 11 | 397 | 31 | 3 | 9 | 57 | 240 |
| KOTOR 1 | Cape | 2 | 63 | 43 | 0 | 6 | 51 | 226 |
| KOTOR 1 | Juhani's cloth | 1 | 48 | 38 | 0 | 0 | 62 | 220 |
| KOTOR 1 | Shoulder flaps | 1 | 44 | 64 | 0 | 0 | 36 | 195 |
| KOTOR 1 | Fur flaps | 4 | 76 | 38 | 0 | 3 | 59 | 226 |
| KOTOR 2 | Hair and head parts | 83 | 7,078 | 42 | 0 | 0 | 58 | 253 |
| KOTOR 2 | Hood | 1 | 76 | 38 | 0 | 0 | 62 | 238 |
| KOTOR 2 | Belt, skirt or leg flap | 11 | 395 | 30 | 3 | 9 | 57 | 240 |
| KOTOR 2 | Cape | 2 | 63 | 43 | 0 | 6 | 51 | 226 |
| KOTOR 2 | Jacket or shirt flap | 2 | 47 | 60 | 0 | 0 | 40 | 238 |
| KOTOR 2 | Sash or arm streamer | 4 | 99 | 43 | 0 | 4 | 53 | 243 |
| KOTOR 2 | Head tentacles | 4 | 168 | 19 | 0 | 19 | 62 | 240 |
| KOTOR 2 | Ears, earrings, beard, breathing gear | 7 | 351 | 14 to 36 | 0 | 0 | 64 to 86 | 216 to 245 |

The constraints are nearly always either 0 or above 128. Only the belt and leg flaps of Bastila's armour and the Quarren tentacles have a noticeable share between 64 and 127.

## The danglymeshes on bodies

These are all the body danglymeshes, so this is the complete list of cloth that the games let sway. A "free" vertex has constraint 0. Values are displacement, tightness and period. The vertex count and the share of free vertices follow in the last two columns. When a model appears in both games with the same data, it is listed once.

| Model | Games | Node (parent bone) | Part | Displacement, tightness, period | Vertices | Free vertices |
|---|---|---|---|---|---|---|
| `p_bastilabb` (Bastila's armour) | both | `object` (`pelvis_g`) | belt flap | 0.03, 3, 8 | 57 | 22 |
| `p_bastilabb` | both | `frntflap` (`pelvis_g`) | front flap | 0.03, 3, 8 | 51 | 6 |
| `p_bastilabb` | both | `rlegflap`, `llegflap` (thighs) | leg flaps | 0.03, 3, 8 | 26 each | 10 each |
| `n_jedimalek` | both | `cape`, `cape01` (`pelvis_g`) | cape | 0.075, 1.5, 8 and 0.05, 1.5, 8 | 30 and 33 | 10 and 17 |
| `n_jedimemf` (Jedi Council member) | both | `bumflap`, `frontflap` (`pelvis_g`) | rear and front flap | 0.025, 0.85, 3 | 29 and 12 | 9 and 4 |
| `n_jedimemf` | both | `shoulderflaps` (`torsoupr_g`) | shoulder flaps | 0.025, 0.85, 3 | 44 | 28 |
| `n_jedimemm` | both | `object` (`pelvis_g`) | belt flap | 0.025, 0.8, 5 | 30 | 10 |
| `n_jedicounf` | both | `flaps` (`pelvis_g`) | flaps | 0.03, 4, 6 | 36 | 12 |
| `n_jedicounm` | both | `mesh` (`pelvis_g`) | flaps | 0.025, 0.8, 3.5 | 50 | 14 |
| `p_joleebb` (Jolee) | KOTOR 1 | `frontflap`, `hood` | front flap, hood | 0.025, 1, 3 and 0.025, 3, 1 | 41 and 104 | 13 and 63 |
| `pmbjl`, `pmbjm`, `pmbjs`, `pfbjl`, `pfbjm`, `pfbjs` (Jedi robe) | both | `hood` (`torsoupr_g`) | hood | 0.025, 3, 8 | 76 | 29 |
| `p_juhanibb` | KOTOR 1 | `cloth` (`pelvis_g`) | cloth | 0.125, 1.75, 7 | 48 | 18 |
| `n_wookiem`, `n_wookief`, `l_wookief`, `p_hanharr` | both | flaps and forearm fur | fur | 0.02 to 0.03, 0.9 to 3, 4 to 8 | 14 to 39 | 6 to 12 |
| `p_attonbb` (Atton) | KOTOR 2 | `jacketflapnew`, `shirtflapnew` (`torsoupr_g`) | jacket and shirt flap | 0.025, 0.75 to 1.2, 2 to 3.5 | 36 and 11 | 20 and 8 |
| `p_mirabb` (Mira) | KOTOR 2 | `shirttie` (`torsoupr_g`) | shirt tie | 0.025, 1, 3 | 10 | 3 |
| `p_visasbb` (Visas Marr) | KOTOR 2 | `sash_geo` (`pelvis_g`) and two forearm streamers | sash, arm streamers | 0.025, 1.15 to 1.2, 4 to 4.5 | 29 and 30 each | 12 and 14 each |

Three points stand out.

- The KOTOR 1 Jedi robe (`pmbj*`, `pfbj*`) has a swaying hood and nothing else. The most common robe of both games therefore does not sway below the neck.
- The two capes belong to `n_jedimalek`, which is a Jedi model that both games use. The cape is only 30 and 33 vertices.
- Juhani's cloth and Juhani's hair (`p_juhanih`) are the only parts with the 12.5 cm limit. Hair on the head of `pfhc03` uses 15 cm.

## Skirts without a danglymesh

### How robes are built in the data

The games build every loose garment in one of four ways.

1. A danglymesh, as listed above. This covers hoods, belt flaps, shoulder flaps and short capes.
2. A skinned mesh that follows ordinary body bones. KOTOR 1's Jedi robes (`pmbj*`, `pfbj*`) and the `tailflap` of `pfbil`, `pfbim` and `pfbis` are built this way. `pmbim`'s `apron_geo` and Bastila's `backflaps` and `frntflap` (in the skinned copy `p_bastilabb02`) too.
3. A skinned mesh that follows dedicated cloth bones. This exists in KOTOR 2 for robes and in both games for long capes.
4. A chain of single bones with a small rigid mesh on each, animated only by cutscene clips. The cutscene models `m44ac_c01_char01` (Bastila with `cape01_g` to `cape05_g`, `lflap01_g` to `lflap05_g`, `rflap01_g` to `rflap05_g` and `bflap01_g` to `bflap05_g`) and `m40ab_c03_char01` (Malak, 24-vertex meshes `cape01_g` and `cape02_g`) do this. Each of these has 6 clips that keyframe the cloth bones.

### Models with dedicated robe bones

The robe bone chain of KOTOR 2 has 11 bones. They are `frobe1_g` and `frobe2_g` at the front, `mrobe1_g` to `mrobe3_g` in the middle, `lrobe1_g` to `lrobe3_g` on the left and `rrobe1_g` to `rrobe3_g` on the right. KOTOR 2's `s_male02` supermodel keyframes all 11 bones in 226 of its 230 clips (the walk clip included), and `s_female03` keyframes them in 66 of its 68 clips. KOTOR 1's supermodels have no track for any robe or cape bone.

| Model | Game | Robe or skirt mesh | Vertices | Cloth bones |
|---|---|---|---|---|
| `pmbim`, `pfbim` | KOTOR 2 | `robe_geo` and `apron_geo`, or `robeback` and `robefront` | 223 and 60, or 207 and 56 | robe bones |
| `pmbmm`, `pfbmm` | KOTOR 2 | `robe01`, `skirt_geo` | 394 | robe bones |
| `pmbnm`, `pfbnm` | KOTOR 2 | `robe` and `flap` | 215 or 217 and 62 or 60 | robe bones |
| `pfbl` | KOTOR 2 | `skirtthing` | 40 | `frobe` and `mrobe` bones |
| `n_jedirobem` | KOTOR 2 | `robe` | 224 | robe bones |
| `n_darkjhim`, `n_darkjhif` | KOTOR 2 | `skirt` | 204 and 206 | robe bones plus the pelvis and thighs |
| `n_talia` | KOTOR 2 | `pantflaps` | 170 | `frobe`, `mrobe` and the pelvis |
| `n_darthrevan`, `n_darthrevanm`, `n_darthrevanf` | both | `cape` (110 vertices) | 110 | `cape01_g` to `cape05_g`, plus `l_robe_g` and `r_robe_g` |
| `n_darthmalak02`, `n_darthmalak03` | both | `backcape` (106 vertices) | 106 | `cape01_g`, `cape02_g` and three more |
| `p_bastilabb02` (cutscene body) | KOTOR 1 | `backflaps` | 112 | the torso bone only |

For these models, the skirt vertices of a robe are exactly the vertices of the mesh named in the table. A new simulation can therefore drive the robe by its robe bones without any guesswork. In the robe meshes, between 3 and 34 percent of the vertices have a single bone with weight 1, so the weights are also sharp enough to use as a constraint (a vertex weighted to `frobe1_g` hangs from the front, one weighted to `mrobe3_g` is the free hem at the back).

The K1 `n_darthrevan` models have `l_robe_g` and `r_robe_g` bones and `cape` bones, but none of the K1 clips keys them. A kjk simulation would be the first thing to move them.

### Models whose skirt is only a skinned mesh on the body bones

These are the cases that the roadmap issue worried about. To test whether the skirt can be found from the skin weights, the program computed the following for each skinned body model.

- It took every vertex below the thigh joints (above 0.25 m, to leave the feet out) whose dominant bone is the pelvis, a thigh, a shin or `torso_g`.
- It measured each vertex's distance to the straight segments from thigh to shin to foot, on both legs. A vertex more than 15 cm from both legs is called a skirt vertex. This is a geometric test, not a weights test.
- For the skirt vertices it then looked at which bones carry them, whether the bone side matches the side of the vertex, and how many are blended between the left and the right thigh.

Trousers give a baseline. `pmbal`, `pmbam` and `pmbcm` (KOTOR 1 trousers) have 3 to 4 percent of such vertices, all of them pelvis vertices at the hips (height 0.89 to 0.98 m) and none on a thigh or shin. A skirt therefore shows up as vertices that are carried by thigh or shin bones but far from the legs.

| Model | Mesh holding the skirt | Skirt vertices of leg-weighted vertices | On pelvis | On thighs | On shins | Hem height (m) | Right side bone | Wrong side bone | Blend of both thighs |
|---|---|---|---|---|---|---|---|---|---|
| `pmbjm` (KOTOR 1 Jedi robe) | `legs` | 56 of 200 | 6 | 28 | 22 | 0.41 | 35 | 15 | 25 |
| `pfbjm` | `legs` | 49 of 222 | 3 | 32 | 14 | 0.38 | 25 | 21 | 29 |
| `pfbim` (KOTOR 1) | `tailflap` and `torso` | 60 of 426 | 26 | 34 | 0 | 0.52 | 34 | 0 | 32 |
| `pmbim` (KOTOR 1) | `torso` | 65 of 411 | 18 | 47 | 0 | 0.53 | 36 | 11 | 37 |
| `n_darthrevan` (KOTOR 1) | `legs` and `belt` | 90 of 442 | 30 | 46 | 14 | 0.41 | 42 | 18 | 20 |
| `n_sithappren` | `torso` | 55 of 274 | 10 | 15 | 30 | 0.43 | 42 | 3 | 3 |
| `p_kreiabb` (KOTOR 2) | `torso_geo` | 37 of 79 | 1 | 17 | 19 | 0.27 | 28 | 8 | 11 |
| `p_visasbb` (KOTOR 2) | `torsonew_geo` | 58 of 148 | 6 | 16 | 36 | 0.30 | 28 | 24 | 37 |
| `p_atrisbb` (KOTOR 2) | `torso_geo` | 90 of 208 | 5 | 39 | 46 | 0.34 | 74 | 11 | 61 |
| `n_darthtraya` (KOTOR 2) | `torso_geo` | 27 of 135 | 1 | 16 | 10 | 0.26 | 16 | 10 | 17 |
| `n_quarren` (KOTOR 2) | `torso` | 30 of 72 | 0 | 15 | 15 | 0.31 | 29 | 1 | 19 |

(The `pmbim` and `pfbim` rows are KOTOR 1's models. The KOTOR 2 models of the same names are different robes, which are in the previous table.)

The test gives these results.

- The hem of these skirts is low. It is between 0.26 and 0.53 m above the floor, which is knee height or lower. This is the part that moves most in a leg swing.
- The skirt vertices lie on thigh and shin bones, not on the pelvis. For the KOTOR 1 Jedi robe 50 of 56 vertices are on thighs and shins. For Visas and Atris it is 52 of 58 and 85 of 90.
- The split between left and right is not clean. For `pmbjm`, 35 vertices are weighted to the bone on their own side, but 15 are weighted to the bone of the opposite side, and 25 of the 56 carry a second weight of at least 0.1 on the opposite thigh. For Visas 28 vertices have the right side and 24 the wrong side. A vertex in the middle of a robe's front or back is weighted to both thighs, so the bone side cannot say whether the vertex is "front" or "back" or "left" or "right".
- The plain skinned skirt is also not a separate mesh. In `pmbjm` and `pfbjm` it is the lower part of the single `legs` mesh, with 438 vertices in all. In `p_kreiabb`, `p_visasbb` and `p_atrisbb` it is the lower part of the `torso` mesh. A mesh name therefore cannot find the skirt in these models, and the test needs the position of each vertex as well as its weights.
- For one robe, the `legs` mesh of `pmbjm`, 330 of 438 vertices are weighted to a thigh or a shin. The legs themselves are weighted the same way, so a weight-only rule ("heavily on thigh or shin, below the hip") cannot tell skirt from leg. The rule needs a distance test against the leg capsules, like the one used here. Those capsules are what the ragdoll code already fits to each body.
- With that distance test the skirt is found with the false positives of the baseline. Trousers give about 3 percent, and those are all hip vertices. A 15 cm limit also leaves out the hem of a robe that hugs the leg closely (a narrow robe such as `n_sithappren` only has 55 skirt vertices of 274).

In summary, the skin weights alone find the skirt poorly, but the weights plus a distance from the leg segments find a usable set. For KOTOR 2's 11 robe models and for the long capes, the dedicated bones find it exactly.

## Capes that are plain skinned meshes

A cape or long skirt with no danglymesh and no cloth bone is rare. Three are found.

- `n_quarren` (KOTOR 2) has a 57-vertex `cape` mesh skinned to `torso_g` and `torsoupr_g` only.
- `p_bastilabb`'s `backflaps` (112 vertices) is weighted to `torsoupr_g` on 106 of its vertices. It is a short back flap that follows the spine.
- `n_darthrevan`, `n_darthmalak02` and `n_darthmalak03` have long capes (110 and 106 vertices) skinned to the `cape01_g` to `cape05_g` chain, so they are the cloth-bone case, not the plain case.

## Non-character danglymeshes

These come from area and effect models, not from characters. Most are relevant only if the engine's sway should handle them. Counts are meshes, with vertices in brackets.

| Game | Texture | Part | Models | Meshes (vertices) |
|---|---|---|---|---|
| KOTOR 1 | `lda_leaf02` | Kashyyyk tree leaves (`branchie*`, 10 vertices each, displacement 0.1, tightness 0.5, period 0.3) | 22 area models (`m14aa_*`, `m14ab_*`, `m14ac_*`, `m14ad_*`) | 2,444 (24,440) |
| KOTOR 1 | `lta_hide01`, `lta_hide02` | Tatooine hide flaps (`sidetarp`, displacement 0.15) | 9 area models (`m20aa_*`) | 50 (3,662) |
| KOTOR 1 | `lta_tent01`, `lta_tent02` | Tatooine tent tarps (`tarp_`, 126 to 370 vertices, displacement 0.1 to 0.4) | 13 area models (`m17aa_*`, `m17ab_00a`) | 22 (4,224) |
| KOTOR 1 | `lta_metal01`, `lta_metal20` | Tatooine wires (`wire`) and a pole | 3 area models (`m17ac_00a`, `m20aa_00b`) | 15 (9,130) |
| KOTOR 1 | `m36_shrub` | Manaan shrubs (`box`, 160 to 1154 vertices, displacement 0.1) | 7 area models (`m33ab_01`, `m36aa_*`) | 9 (5,378) |
| KOTOR 1 | `luz_palm01` | Palm (`cylinder`, 620 to 632 vertices) | 1 (`m41az_03b`) | 5 (3,148) |
| KOTOR 1 | `lqa_flag` | A flag (`plane`, 32 vertices, displacement 0.3) | 1 (`mqa_01a`) | 1 (32) |
| KOTOR 2 | `dan_leaf02` | Dantooine leaves | 5 area models (`601dana`, `601danb`, `601danc`, `601danf`, `601dang`) | 567 (18,896) |
| KOTOR 2 | `dan_flag2` | Dantooine banners (`box`, 38 vertices, displacement 0.08, tightness 1, period 3, one 152-vertex banner of displacement 0.1) | 6 area models (`602danb`, `602danc`, `602danj`, `602dank`, `602danp`, `602danr`) | 16 (760) |
| KOTOR 2 | `m36_shrub` | Korriban shrubs | 3 area models (`701korb`, `701korc`, `701kore`) | 4 (1,612) |
| both | `fx_expparts` | Explosion debris (`hullplate`, 12 vertices) | 1 (`fx_part_04`) | 1 (12) |

Two surprises came up.

- The banners of KOTOR 2's Dantooine ruins (`dan_flag2`, 38 vertices) are the only cloth in the data that is made to hang from a pole and wave. They are small, and each has constraint 0 on only about 8 percent of its vertices.
- `p_juhani` (KOTOR 1) is a model with no supermodel and no parent bone for its danglymesh. It is one 275-vertex mesh named `cylinder` and it has no texture name. Its constraints are 58 free vertices and 217 above 128. It is not in a part-type table because it is not clear what it is, and it was not examined further.

## What the census means for the roadmap issue

- The first targets should not be the danglymeshes. The cloth danglymeshes have 10 to 57 vertices and a free hem of 2.5 to 12.5 cm. They are too small to show a flying robe, and only about 19 body models in each game have them.
- KOTOR 2 has 11 robe models with 11 robe bones and animated clips. A simulation could drive those bones directly, for the player's robes (`pmbim`, `pmbmm`, `pmbnm` and the female versions), the Jedi robes of NPCs (`n_jedirobem`), the Sith (`n_darkjhim`, `n_darkjhif`) and Talia and `pfbl`. Nothing has to be guessed about which vertices form the robe.
- KOTOR 1 has no robe bones except the Darth Revan models. Its Jedi robes (`pmbj*`, `pfbj*`) and the `tailflap` meshes need skirt vertices found from weights plus a distance test against the leg capsules, or a one-time vertex mark per model. About 12 KOTOR 1 body models fit this case in the sample, and the exact count would need the item and appearance tables to name every robe-class body.
- The ragdoll code already fits 14 capsules to each humanoid body. The distance test used here to find the skirt vertices is the same idea as testing a cloth vertex against a leg capsule, so the capsules should work as colliders.
- The hair danglymeshes are the biggest group (125 distinct hair meshes in both games) but they are mostly very small, so the cost of moving them is low per mesh and high only in a large crowd.

## Limits of this census

- The part type is guessed from node names, and the robe classification of the plain skinned meshes is geometric. A few body models were put into a type by judgment.
- The skirt test used the rest pose of the skeleton. The legs in this pose are splayed (the thigh joints are at x = 0.10 m, and the feet at x = 0.20 m), so a 15 cm limit was chosen after checking trouser bodies.
- The list of robe-class bodies in KOTOR 1 is a sample, not a full list. A full list needs `appearance.2da`, `baseitems.2da` and the item tables to find every body variant that is a robe or a coat.
- The animation check looked for tracks named `robe`, `cape` or `flap`. It did not check whether kjk's animation player already plays them for the KOTOR 2 robes.
