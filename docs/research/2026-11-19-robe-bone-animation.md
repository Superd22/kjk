# Does kjk play the robe and cape bone tracks

This note was written on 2026-11-19 for the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts"). It answers one question from `docs/research/2026-11-19-loose-parts-census.md`: does kjk's animation player play the robe bone tracks and the cape bone tracks today, so that these robes and capes move as in the original game? The answer was measured on the installed KOTOR 1 and KOTOR 2 with a test, and the code was read to explain the results.

## Summary

- Yes for the KOTOR 2 robes, outside of combat. All 11 robe bodies (`pmbim`, `pmbmm`, `pmbnm`, `pfbim`, `pfbmm`, `pfbnm`, `pfbl`, `n_jedirobem`, `n_darkjhim`, `n_darkjhif` and `n_talia`) bind their robe bone tracks to the robe bones, and the skinned skirt vertices move with them. The tracks come from the body's own clips first and then from `s_male02`, `s_female03` and the other models of its supermodel chain.
- The robe mesh of every one of these bodies is skinned to the robe bones. `pmbim` has 11 robe bones in the skin of its `robe_geo` mesh, `pmbmm` has 9 in `robe01`, and `pfbl` has 4 in `skirtthing`. No mesh has more than 17 joints, so the skinning limit is not a problem.
- Measured against the same pose with the robe tracks cut out, the skirt vertices of `pmbim` move up to 24 cm in `pause1`, 60 cm in `walk` and 94 cm in `run`. The other 10 bodies move by similar amounts. The results are in the table below.
- Nothing in the name matching, the node filter, the track types or the supermodel chain drops these tracks. The one place where robe tracks are not played is a fight. A player body or humanoid creature uses Jedi Academy clips for combat (`asset_ja::rig`), and those clips never key the robe bones. The robe then stays at its rest pose while the character is armed or readying a weapon. KOTOR 2 plays its own clips with robe keys in a fight, so this is a difference from the original. This was found by reading the code and was not measured.
- The capes are a different story, because the capes of the Darth Revan and Darth Malak models are not keyed by their own clips in the way the census note assumed.
  - KOTOR 2 `n_darthrevan` has a cape skinned to `cape01_g` to `cape05_g`. None of the clips in its chain keys a cape bone, so the cape stays at rest in kjk. The robe keys of `s_male02` target `frobe1_g` and similar names, which this model does not have.
  - KOTOR 2 `n_darthmalak02` and `n_darthmalak03` have a cape skinned to `cape01_g`, `cape02_g` and three more bones. Their supermodel `n_darthmalak` keys 27 cloth bones per clip, but under the names `cloth_1_g` to `cloth_8_g` and so on. These do not match the names of the cape bones, so no track binds and the cape stays at rest.
  - KOTOR 1 `n_darthmalak` has a cape skinned to `cloth_1_g` to `cloth_8_g`, and its own 66 clips key those bones. All 27 cloth tracks bind and move the cape.
  - KOTOR 1 `n_darthrevan` has a cape that none of its clips keys. The cutscene model `m44ac_c01_char01` keys `cape01_g` to `cape05_g` in 6 clips of its own, and when its first three clips are played on `n_darthrevan`, all 5 cape bones bind and move the cape by up to 66 cm.
- If kjk should move a cape that the original leaves at rest, or a robe during a fight, that needs new motion and not a fix of the player. Section "What a fix would touch" lists the places.

## Method

A test was added in a git worktree at `.claude/worktrees/robe-bones`. It is `crates/approved_tests/tests/robe_bones.rs` and it is not committed. It needs the installs named in `.env`, and it skips a game that is missing. Run it with `cargo test -p approved_tests --test robe_bones -- --nocapture`. It never opens a window.

For each body the test does the following.

1. It opens the game with kjk's own resolver (`vfs::resolver::Resolver`) and loads the body and each supermodel above it (`ModelIr::from_model`), following the `supermodel` name of each model until `null`, a missing model or a repeat. This is the same walk as `game::viewer::clip` in `crates/game/src/viewer.rs`.
2. It takes the clip named `pause1`, `walk` or `run` from the first model of the chain that has it.
3. It binds the clip to the body's nodes with `AnimationIr::bind` and poses every node with `anim::pose`. Both are in `crates/asset_odyssey/src/anim.rs` and are the functions the renderer's `Animator` uses.
4. It poses the body a second time with the same clip after removing every track of a node whose name contains `robe`, `cape` or `cloth`. The difference between the two poses is the motion that the robe or cape tracks add, apart from the walk of the whole body.
5. It finds the skinned vertices that have a weight above 0 on a robe, cape or cloth bone. It moves each by its skin matrices in both poses (the joint's world matrix times `rest_inverse_binds`) and records the largest distance between the two over the clip, sampled every 1/30 of a second.

The test asserts that every robe body has a robe bone in a skin, that at least one robe track binds in each of `pause1`, `walk` and `run`, and that the skirt vertices move by more than 5 mm. All three tests pass.

## How the code plays a track

- Supermodel clips are found by name through the chain (`game::viewer::clip`). A clip is the supermodel's own `AnimationIr`, and its tracks are matched to the body by node name without case (`AnimationIr::bind`). Because of that, a clip from `s_male02` moves the robe bones of `pmbim` as long as `pmbim` has nodes with the same names.
- `bind` skips a track in two cases. The first is that the body has no node of that name. The second is that the node is not marked `animated`, which means it is not at or below the animation root of the body (`fmt_odyssey::mdl`). Both robe bone sets are below the root in all 11 bodies, because the test found every robe track bound.
- `anim.rs` keeps position keys (controller 8) and orientation keys (controller 20, with 2 or 4 columns). It keeps no other controller as a track. The robe and cape tracks use only these two, so nothing is lost. The 2-column packed orientation form is handled.
- The skin of a mesh keeps every joint of the file's bone map (`asset_odyssey::model::skin`), up to 12 per robe mesh here. Bevy takes 256 joints per mesh, so the limit is not reached. Each vertex takes four joints and four weights, which is what the files hold.
- A skinned mesh follows its joint entities in the scene (`SkinnedMesh` in `crates/game/src/scene.rs`). Moving the robe bone entity therefore moves the skirt. The culling box of a skinned mesh is the rest box grown by the model's size (`fixed_bounds::skinned`), so a swinging skirt is not culled.
- Out of a fight, a humanoid body plays its own `pause1`, `walk` and `run` clips (`crates/game/src/gait.rs`). `pose_over` in `crates/game/src/scene.rs` lays each clip over the pose, and the robe bones are written like any other bound node. Script clips and talk clips play the same way.
- In a fight, the JA clips are moved onto the KOTOR skeleton by `asset_ja::rig::retarget`. Its documentation says "Unmapped nodes keep their rest rotation under their parent." The robe bones have no JA bone to follow, so they stay at rest. `at_rest` in `crates/game/src/cast.rs` puts every node of the rig back to rest before the clips are laid.

## Measured results for the robe bodies

Each row gives the clip, the model of the chain that holds it, the number of robe bones whose track bound, the number of skirt vertices weighted to the robe bones, and the largest distance one of those vertices moved from the same pose without the robe tracks. Distances are in metres.

| Body | Robe bones in the skin | Clip, from | Bound | Vertices | pause1 | walk | run |
|---|---|---|---|---|---|---|---|
| `pmbim` | 11 | `s_male02` | 11 | 234 | 0.24 | 0.60 | 0.94 |
| `pmbmm` | 9 | `s_male02` | 9 | 359 | 0.16 | 0.31 | 0.65 |
| `pmbnm` | 11 | `s_male02` | 11 | 238 | 0.24 | 0.60 | 0.95 |
| `pfbim` | 11 | `pfbim` for pause1 and walk, `s_female03` for run | 11 | 224 | 0.32 | 0.59 | 0.83 |
| `pfbmm` | 9 | `s_female03` | 9 | 359 | 0.22 | 0.30 | 0.50 |
| `pfbnm` | 11 | `pfbnm` for pause1 and walk, `s_female03` for run | 11 | 231 | 0.30 | 0.56 | 0.77 |
| `pfbl` | 4 | `s_female03` | 4 | 32 | 0.10 | 0.18 | 0.37 |
| `n_jedirobem` | 11 | `s_male02` | 11 | 195 | 0.23 | 0.52 | 0.87 |
| `n_darkjhim` | 9 | `s_male02` | 9 | 179 | 0.24 | 0.62 | 1.09 |
| `n_darkjhif` | 9 | `n_darkjhif` for pause1 and walk, `s_female03` for run | 9 | 174 | 0.31 | 0.63 | 0.82 |
| `n_talia` | 5 | `s_female03` | 5 | 140 | 0.13 | 0.20 | 0.58 |

Three facts follow from the table.

- The male bodies take their robe tracks from `s_male02`, and the female bodies from `s_female03`. A few bodies (`pfbim`, `pfbnm`, `n_darkjhif`) have their own `pause1` and `walk`, and these key the robe bones too.
- Some bodies lack some robe bones. `pmbmm`, `pfbmm`, `n_darkjhim` and `n_darkjhif` have 9 of the 11 (no `frobe1_g` and `frobe2_g`), `pfbl` has only `frobe1_g`, `frobe2_g`, `mrobe1_g` and `mrobe2_g`, and `n_talia` has 5. In each case every robe bone the body has is bound.
- The `run` distances of up to 1 m are large because the run clip swings the robe bones far. They show that the tracks move the skirt, and they are not a statement about how the original looks.

## Capes and cloth bones

| Body | Game | Cape bones in the skin | Clips in the chain that key them | Bound for `pause1`, `walk` and `run` |
|---|---|---|---|---|
| `n_darthrevan` | KOTOR 2 | `cape01_g` to `cape05_g` | none | 0 |
| `n_darthmalak02` and `n_darthmalak03` | KOTOR 2 | `cape01_g`, `cape02_g`, `cape02_g01` to `cape02_g03` | none under these names (the supermodel keys `cloth_*`) | 0 |
| `n_darthmalak` | KOTOR 1 | `cloth_1_g` to `cloth_8_g` and more | all 66 clips of the model key 27 cloth bones | 27 |
| `n_darthrevan` | KOTOR 1 | `cape01_g` to `cape05_g` | none | 0 |
| `m44ac_c01_char01` clips on KOTOR 1 `n_darthrevan` | KOTOR 1 | `cape01_g` to `cape05_g` | 6 cutscene clips, each with 5 tracks | 5 |

- The cutscene case is the only one where the clips and the body have matching names for a cape. For `cut002w`, `cut003w` and `cut004w` the cape vertices move by up to 14 cm, 23 cm and 66 cm. Whether the game plays this model's clips on the cape of Revan or of someone else was not checked. The cape of Bastila in `m44ac_c01_char01` is a chain of rigid meshes on the bones themselves, which `ModelIr` draws as ordinary nodes, so those move too.
- The K1 `n_darthmalak` cloth bones move by up to 1.0 m in `pause1` and 1.4 m in `run` against the same pose without them. The cloth tracks of that model are large, so a cape that is very wide would not look like a hanging cloth. This was not looked at on screen.
- The KOTOR 2 Malak cape has bones that its supermodel does not name. kjk, like reone as `anim.rs` cites it, matches tracks to nodes by name only. kjk therefore leaves this cape at rest. Whether the original game also leaves it at rest was not checked against the executable.

## What a fix would touch

Robes in a fight:

- `asset_ja::rig::retarget_with` in `crates/asset_ja/src/rig.rs` builds the clip that moves a KOTOR body to JA's motion. It would also have to add tracks for the robe bones. These could copy a KOTOR clip's robe tracks, such as `s_male02`'s `g1r1` or a walk or run clip matching JA's legs. The clips are found as in `crates/game/src/viewer.rs` (`scripted_clips`).
- The robe motion would then be laid in `crates/game/src/cast.rs` (the `Look::Retargeted` branch around `gait.shown()`), either by also laying the gait clip's robe tracks during a fight or by a robe-only clip. `pose_over` writes every bound node, so a robe-only `Animator` can be made with `Unbound` after removing all other tracks.
- A new simulation of the robe bones, as proposed in `docs/research/2026-11-19-loose-part-solvers.md`, would replace those tracks. It would write the same robe bone entities that `Animator` writes. The skin side needs no change.

Capes that no clip moves:

- The KOTOR 2 Revan and Malak capes have bones in the skin but no clip. Only a new motion can move them. A solver that writes `cape01_g` and the following bones would do it, again through the node entities of the spawned model (`crates/game/src/scene.rs`).
- A rename table from `cloth_N_g` to the `cape` names would let the existing Malak clips move the K2 cape. This is a guess at what the data intends. The clips key 27 bones, and the K2 body has 5 cape bones, so a mapping would have to be chosen by position.

## Limits of this check

- The test measures where the vertices go, not how they look. No window was opened and nothing was drawn.
- Whether a robe body plays its walk and run clips in the real game, with the gait weights of `gait.rs`, was read from the code and not run. The test uses the clip functions directly.
- The combat statement for robes comes from reading `asset_ja::rig::retarget` and `at_rest`. It was not measured.
- It was not checked which of the 11 bodies pass `asset_ja::rig::is_humanoid`. A body that does not pass plays only KOTOR clips and so has moving robes in a fight as well.
