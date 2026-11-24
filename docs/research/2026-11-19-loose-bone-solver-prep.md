# Preparing the loose bone solver

This note was written on 2026-11-19 for the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts"). It prepares the implementation of one XPBD solver that moves points, where a point is a bone joint or a mesh vertex. The first users are the KOTOR 2 robe bones and the cape bone chains of both games. The note answers four questions. Where the solver runs in the frame, how bones become points and points become bones again, which bones in the games are loose bones and how to find them from data, and which chains are neighbours. It builds on `docs/research/2026-11-19-loose-parts-census.md`, `docs/research/2026-11-19-robe-bone-animation.md`, `docs/research/2026-11-19-loose-part-solvers.md` and `docs/research/2026-11-19-loose-part-body-collision.md`. The code was read in this repository and the data was measured from the installed games. Nothing in the repository was changed, and no game window was opened.

## Summary

- **Where it runs.** The solver is one new system in `PostUpdate`. It runs after every system that writes the bone transforms of a character (`play_view::draw`, the face lips, the remote view and `own_fx`) and before `TransformSystems::Propagate`. At that point the final pose of the frame is in the local `Transform` of every bone. Bevy then propagates the solver's rotations, and every later reader sees them. The skinning extract, the shadow tiles, the acceleration structures and the previous-frame joint buffer of the motion vectors all work without a change, and nothing lags a frame. The solver cannot read `GlobalTransform` at that point, so it must walk the skeleton itself in the character's model space. That costs a few microseconds per character.
- **Two changes to the cast are needed.** The first is that the cast leaves a bone alone when its standing clip does not key it (`held_nodes`). A loose bone that `pause1` does not key would keep the solver's own output from the frame before, and the solver would read it back as the target pose. The loose bones must therefore be taken out of the held set. The second is that the cast must tell the solver which characters it posed this frame, because it does not pose characters that are far away or lying still.
- **Bones to points.** Each loose bone gives one point at its head, and each leaf bone gives one extra tip point. The head of a bone whose parent is not a loose bone is pinned to the animated parent. After the solve, each bone turns by the shortest arc that aims it at its solved child point. The twist comes from the animated pose. The rest world positions are in `NodeIr::world`, the hierarchy is in `NodeIr::parent`, and the skin gives the tip length. The file's inverse bind matrices agree with the node rest positions to 0.0000 m in every loose bone that was checked.
- **The loose bones.** In the humanoid models, 18 KOTOR 2 character models and 10 KOTOR 1 character models have loose bones that carry garment vertices. A three-part rule finds them without a name list. The rule finds 180 loose bone entries in KOTOR 2 and 109 in KOTOR 1, counted per model and leaving out the menu scene models, and every one of them is a garment bone. The rule misses the hoods, sleeves and a few thigh and shin flaps, and a second, looser rule adds them with a few false positives. It also misses the garments of skeletons that are not humanoid, such as the Rakata and the Ithorian.
- **The neighbours.** Chains are not all straight. The KOTOR 1 Malak cape is a tree with branches. The rule that works is to link any two strands that share a root, or that share at least 10 vertices with two bones of weight 0.2 or more, or that are close and lie less than 120 degrees apart around the spine. On `pmbim` this links the left side to the back and the back to the right side, and it leaves the front apron free. On `pfbl` it links nothing. On `n_jedirobem` it links all of the left, back, right and front pieces except front to back. On `n_darthmalak` it links each pair of branches that share a root, which gives four separate wings.

## Question 1, where the solver runs

### The frame, in order

All of this is in `crates/game` and runs in `PostUpdate`, unless it says otherwise.

1. The simulation snapshot is presented in an earlier stage. The view systems read it in `PostUpdate`.
2. `play_view::draw` (`crates/game/src/play_view.rs:1420`) is in the set `play_view::Drawn` (`play_view.rs:200`). The system is registered at `play_view.rs:625-646` as the end of a chain that is ordered `.before(TransformSystems::Propagate)`. It calls `Cast::draw` at `play_view.rs:1753`.
3. `Cast::draw` (`crates/game/src/cast.rs:3758`) poses every character that is in `view.close`. With fewer than 8 characters that are fully posed, the poses are made in a plain loop. From 8 moving characters (`PARALLEL_FROM`, `cast.rs:4022`) they are made on the compute task pool, each in a `Staged` copy of the transforms, and the changed transforms are applied afterwards (`cast.rs:3897-3921`). Characters that are lying still (`Placing::still`) are not posed at all. Characters that are not in `view.close` get only their root placed, because `place` returns early when `pose` is false (`cast.rs:5983`).
4. For one character, the function `place` (`cast.rs:5864`) works in this order for a body that uses Jedi Academy clips (`Look::Retargeted`).
   - It writes the root transform (`cast.rs:5979`).
   - It puts every node back at rest with `at_rest` (`cast.rs:5986`, defined at `cast.rs:4363`), except the nodes that are held.
   - It poses the face to rest (`cast.rs:5989`).
   - It chooses the gait clip for a character that is outside a fight (`cast.rs:5949-5962`, `crates/game/src/gait.rs`).
   - It runs the retargeted Jedi Academy pose and writes the rotations of the mapped bones only (`rig.retargeter.pose` at `cast.rs:6219`, writes at `cast.rs:6226-6232`). The retarget leaves the robe bones alone. Its documentation says that unmapped nodes keep their rest rotation under their parent (`crates/asset_ja/src/rig.rs`).
   - It lays the KOTOR clips over that pose with their weights (`lay` at `cast.rs:6317`). The gait clips `pause1`, `walk` and `run` are laid here, and they key the robe bones.
   - It then turns the head, re-grips guns and adds flinches (`cast.rs:6322-6335`).
5. For a body that plays only KOTOR clips (`Look::Clips`, which covers script clips, cutscene clips and the creatures), `place` runs `at_rest` at `cast.rs:6390` and `clips.pose` at `cast.rs:6393`.
6. At the end of `Cast::draw` the fallen bodies are written by the ragdolls (`cast.rs:4011`). `play_view::draw` then writes the cutscene stunts (`play_view.rs:1771`).
7. The rest of the chain in `play_view.rs:625-646` follows. These are `severing::apply`, `made::track`, `wardrobe::dress` and others. They run before or after `draw` in one chain, and none of them writes the rotation of a loose bone.
8. Three more systems write bone transforms, all with the ordering `.after(play_view::Drawn).before(Propagate)`. They are `dialogue_view::lips` (`crates/game/src/dialogue_view.rs:186-190`, the face bones), `remote_view::draw` (`crates/game/src/remote_view.rs:63-66`, droid bodies) and `own_fx`'s `Posed` and `Drawn` sets (`crates/game/src/own_fx.rs:91-106`, rooms and placeables).
9. `TransformSystems::Propagate` runs `mark_dirty_trees`, `propagate_parent_transforms` and `sync_simple_transforms` in that order (`bevy_transform-0.19.1/src/plugins.rs`).
10. Systems that read `GlobalTransform` run after Propagate. Some of them are the shadow tiles (`crates/game/src/shadow_tiles.rs:641-651`, which build the joint matrices at `shadow_tiles.rs:989-1038`), the dangly sway (`dangly.rs:88-90`), the blades (`scene.rs:376-383`), `fixed_bounds::follow_strays` (`fixed_bounds.rs:29-32`) and `play_view::cast_shadows` (`play_view.rs:612-617`).
11. Bevy's render extract runs once the main schedule has finished. `extract_skins` (`bevy_pbr-0.19.1/src/render/skin.rs:272`, registered at `render/mesh.rs:202`) copies the joint matrices of every changed joint into a staging buffer. The joint matrix is `GlobalTransform` times the inverse bind matrix (`skin.rs:410`). `prepare_skins` (`skin.rs:176-190`, registered at `mesh.rs:212`) swaps the current and the previous joint buffers each frame and writes the new one.

### Where the new system goes

The new system goes in `PostUpdate` with these three orderings.

- `.after(crate::own_fx::Drawn)`, which already comes after `play_view::Drawn` (`own_fx.rs:96-102`).
- `.after` the lips system and `remote_view::draw`, since both write bone transforms in the same stretch of the schedule.
- `.before(bevy::transform::TransformSystems::Propagate)`.

The system reads the local `Transform` of the body bones, which is the final animated pose of the frame, whatever mixed it (the gait layer, the retargeted fight clips, `at_rest` or a cutscene clip). It writes the new local rotations of the loose bones into the same transforms.

The four requirements are met as follows.

- **(a) Nothing overwrites the result.** After the system only Propagate runs, and it only reads `Transform`. The next frame starts with `at_rest` and the clips, so the solver must read the target pose after them, which it does.
- **(b) Skinning sees the result in the same frame.** Propagate turns the solver's rotations into `GlobalTransform` values. The extract copies them after `PostUpdate`. The kjk readers listed above run after Propagate, so they see the same values. No kjk system needs an ordering change.
- **(c) The previous-frame joint transforms stay right.** Bevy keeps them itself by swapping two buffers (`skin.rs:187-188`). This is correct as long as the main world holds the final value of each frame, which it does. A skipped frame (the character was not posed) leaves the joint unchanged, so its motion vector is zero for that frame. When the solver restarts after a skip it must snap its points to the target pose and start with zero velocity, as the sway does for a mesh that moved more than 5 m (`dangly.rs`).
- **(d) It does not lag a frame.** The solver reads this frame's pose and this frame's root transform, and Propagate runs right after it.

If the system ran after Propagate instead, it would have to write both `Transform` and `GlobalTransform` of every loose bone and every bone below it, and it would need an `.after(...)` and a `.before(...)` against about ten readers. Running before Propagate avoids both.

### What the solver reads without `GlobalTransform`

The solver works in the model space of the character, which is game space with the root motion included, because `SceneRoot` has a fixed rotation (`scene.rs:1891`) and the model root carries the sole position that the cast writes (`cast.rs:5979`). Every node entity is a child of its parent node or of the model root (`scene.rs:1146-1160`). The solver therefore needs only the local transforms. It walks the node list in order from the root and builds a matrix for each node, using `ModelIr::nodes[i].parent`. The supermodels measured here have 101 to 109 nodes, so this is about 100 matrix products per character.

The same walk gives the capsule poses. The 14 capsules of the ragdoll fit (`Body::doll`, `crates/game/src/ragdolls.rs:178`) are stored in the rest frame of their nodes, so each capsule is one rigid transform of two points.

### What must change in the cast

- **Held nodes.** `held_nodes` (`cast.rs:4351`) marks every node that the body's standing clip does not key. `at_rest` skips these nodes (`cast.rs:4368`). A loose bone that `pause1` does not key, such as the cape bones of `n_darthrevan`, would therefore never be reset, and it would carry the solver's output into the next frame. The solver would then read its own output as the target. The loose bones must be set to "not held" where `held` is built (`cast.rs:271-276` and `cast.rs:3177`). With that, `at_rest` puts every loose bone back to rest each frame, and a keyed clip writes its keys over the rest.
- **The jobs.** The cast does not pose every character every frame. The solver needs a list of the characters that were posed this frame, written by `Cast::draw` as a resource. The solver must run only for those characters. For a character that was not posed, the loose bones still hold the solver's last output, and reading it as a target would feed back.
- **Fallen bodies.** The ragdolls write only their own parts (`cast.rs:4011`). A fallen body should use the rest pose as its target, and not read the transform of its loose bones. The solver then lets a robe fall with the body.
- **The per clip flag.** The roadmap decision is one flag per clip. For a clip whose robe or cape keys only fake physics, the loose bones are left out of the clip's bindings when the clip is bound (`scene::animator`, called from `gait::Gait::bound` at `gait.rs:121-131` and from the script clips in `cast.rs`). The cast then never writes them, `at_rest` resets them, and the target is the rest pose. For a real flourish the bindings stay, and the target is the keyed pose that the cast wrote.

### Threads

- The solver is a normal system, not an exclusive one. Bevy's multithreaded executor (`multi_threaded` is on in `Cargo.toml`) runs it on a task pool thread. It is not tied to the main thread.
- It writes `Transform`, and so do `play_view::draw`, the lips, the remote view and `own_fx`. Those systems are ordered before it, so it cannot run beside them. It also cannot run beside Propagate. Systems that touch neither `Transform` nor `GlobalTransform` can run beside it, but few of them are left at that point in the frame.
- The work can be split inside the system in the same way as `Cast::draw` does it. The system gathers the inputs of each character, solves the characters on the compute task pool, and applies the new rotations in one pass. A crowd needs this and a single character does not. Below `PARALLEL_FROM` characters the thread wake-ups cost about as much as they save (`cast.rs:4018-4022`).
- Nothing in the solve can start before the pose is final, so the solve cannot overlap with the posing. The cost estimate in `docs/research/2026-11-19-loose-part-solvers.md` and the one in `docs/research/2026-11-19-loose-part-body-collision.md` are both estimates, and the prototype must measure them.

## Question 2, from bones to points and back

### What the data offers

| What | Where | Detail |
|---|---|---|
| Node hierarchy | `NodeIr` in `crates/asset_odyssey/src/model.rs:68` | Each node has `parent`, a local `translation` and `rotation` at rest, and `world`, its rest matrix in model space. |
| Rest pose per body | `BodyRig::rest` and `cast::rest_of` | The rest translation and rotation of every node. `at_rest` writes these. |
| Skin per mesh | `SkinIr` at `model.rs:159` | `joints[slot]` is the node index of bone slot `slot`. `inverse_binds[slot]` takes a vertex of the mesh node into that bone's space. Each vertex has four slots and four weights. |
| The entities | `SceneModel::nodes` (`scene.rs:86`) and `Body::nodes` | One entity per node, in node order. The skinned mesh follows these entities (`scene.rs:1549-1554`). |
| Which matrix is used | `scene.rs:1549` | The scene uses `inverse_binds`, the file's values. `rest_inverse_binds` is kept only for measuring. |

The file's inverse bind matrix agrees with the node. For every loose bone of `pmbim`, `n_jedirobem`, `pfbl` and the KOTOR 1 Malak cape, the bind position taken from `inverse_binds` and carried by the mesh node's world matrix is equal to the node's rest position, with a difference of 0.0000 m. A model that has a mesh node away from the origin (`robe_geo` of `pmbim` is 1.2 m above it) therefore needs the mesh node's matrix when a vertex is compared with a bone. Using `rest_inverse_binds` instead of `inverse_binds` gives a 1.2 m error on `pmbim`, so the solver must read `inverse_binds`.

### Points

For one body, the loose bones found by the rule of question 3 give one loose forest. A bone is a root of the forest when its parent is not a loose bone. The loose forest becomes points like this.

1. Every loose bone gets a **head point** at the world position of its node in the rest pose (`NodeIr::world`).
2. Every leaf bone (a loose bone without a loose child) gets one **tip point** after its head. The tip lies along the line from the parent's head to this bone's head, so the last segment continues in the same direction. The distance from the head is the tip length.
3. The **tip length** is the largest distance from the bone's head to a vertex whose weight on the bone is at least 0.5. The distance is measured in the bone's own space, `|inverse_binds[slot] * vertex|`. Where a bone sits in several meshes, the largest of the values is used. The length is clamped to between 0.05 m and 0.6 m.
4. A head point is **pinned** when the bone is a root. It follows the animated parent exactly, because a bone's head cannot move relative to its parent. Every other head point and every tip point is a free point.
5. Each bone is a **segment** from its head to its child head, or to its tip point. The segments get distance constraints at their rest length.
6. A bone with two or more children (the Malak cape) gets one more distance constraint between each pair of its child heads, at their rest distance. This keeps the branch rigid, so that one rotation of the parent bone can fit all of its children.

Checks of the tip length against the segment spacing show that it is a sound choice. On `pmbim` the joints of the side chains are 0.35 m and 0.36 m apart, and the tip lengths of the leaves are 0.38 to 0.41 m. On `pfbl` the front chain has a joint spacing of 0.15 m and a tip length of 0.17 m. A joint that carries no dominant vertex gives 0, as the roots of the Malak cape do, which is why the clamp is needed.

Counts of points are small. The `pmbim` robe has 11 heads and 4 tips, so 15 points, and 4 of them are pinned roots. The KOTOR 1 Malak cape has 27 heads and 8 tips, so 35 points, and 4 of them are pinned.

### Points back to bones

This is done top down, root first, for every loose bone `b`.

1. Let `Rp` be the new world rotation of the parent. For a root, the parent is the attach bone, whose world rotation comes from the animated pose.
2. Read the animated local rotation `qt` of the bone, and the animated local translation `tc` of its child, from the `Transform` components. These are the target pose.
3. The target world rotation of the bone is `Qt = Rp * qt`. The target direction of the segment is `dt = normalize(Qt * tc)`.
4. The solved direction is `ds = normalize(P_child - P_b)`, from the solved points.
5. The swing is the shortest-arc rotation `S` that takes `dt` to `ds`. If the two are nearly opposite, a fixed axis perpendicular to `dt` is used.
6. The new world rotation is `Q = S * Qt`. The twist about the bone's axis is the animated twist, because `S` has no component about `dt`.
7. The new local rotation is `Rp⁻¹ * Q`. This is written to the bone's `Transform`. The translation is not changed.
8. For a bone with several children, `S` is the normalized weighted sum of the swings to each child, with the segment lengths as weights and all quaternions put in one hemisphere first. The rigid branch constraint of step 6 above keeps the children's spread so small that the sum is a good fit. The Malak branches are 0.09 to 0.2 m apart.
9. For a leaf bone, the target direction of the tip is `Qt * tip_local`, where `tip_local` is the rest tip direction in the bone's own frame. This is stored at load time, as are the lengths.

Because the bone lengths are enforced as constraints, the rotation puts each child head where the solver put it, within the stretch the solver allows. The skinned vertices then follow. Vertices that are also weighted to the pelvis, the torso or a thigh keep those weights, and nothing in the skin needs a change. `robe_geo` of `pmbim` is weighted to `pelvis_g`, `torso_g` and `torsoupr_g` as well as to the robe bones.

### Targets

The target of a free point is its position in the animated pose. It is found by the same walk, using the animated local transforms of the loose bones and the new rotations of their parents. The solver pulls each point toward this target with a constraint that has a compliance, and it clamps the point to a maximum distance from it. This is the attachment constraint described in `docs/research/2026-11-19-loose-part-solvers.md`. In rest mode the target is the rest pose, and in keyed mode it is the pose written by the clip.

## Question 3, the loose bones

### Method

A throwaway tool built on `crates/fmt_odyssey` and `crates/asset_odyssey` read every model of both installs, from `chitin.key`, the override folder, `Patch.erf` and the module archives. It kept the models that have a skinned mesh, or a node whose name has a garment word in it (robe, cape, cloth, flap, skirt, sash, belt, apron, hood, sleeve, scarf, coat, shoulder, tie, mesh, object, jacket, shirt, pant). The tool is `/private/tmp/claude-501/-Users-david-perso-kjk/5fbe746b-0603-4318-a877-f1ce4292c9c8/scratchpad/loose/src/main.rs`, and the analysis scripts are in the same folder. This gave 394 KOTOR 1 models and 522 KOTOR 2 models. Of these, 162 and 207 pass the humanoid test (they have every node that `asset_ja::rig::is_humanoid` requires below the neck). The tool read the games read-only.

The humanoid skeleton map is the `HUMANOID` list in `crates/asset_ja/src/rig.rs:83-117`. It names 22 KOTOR nodes that the Jedi Academy retarget moves (`pelvis_g`, `torso_g`, `torsoupr_g`, `neck_g`, `head_g`, both collars, biceps, forearms, hands, the two finger-base nodes of each hand, both thighs, shins and feet).

### The rule

A node is a loose bone when all of these hold.

1. The model passes the humanoid test.
2. The node is not one of the 22 mapped nodes, and it carries garment vertices. This means it is a joint of a skin with at least one vertex of weight 0.25 or more on it, or it is a rigid, non-dangly mesh node that has a loose parent or a loose child. A danglymesh node does not count. The sway code already handles those.
3. Walking up through unmapped nodes, the first mapped ancestor is `pelvis_g`, `torso_g` or `torsoupr_g`. No mapped node lies below the node. The node's name appears in fewer than 25 percent of the humanoid models of the same game.

Each part removes a kind of false positive.

- Part 1 keeps out creatures and droids with their own bone names, where "below the pelvis" means something else.
- The "first mapped ancestor" test and the "no mapped node below" test keep out fingers, toes, the face, ears and the jaw (their first mapped ancestor is a hand, a foot or `head_g`), and also bones such as `necklwr_g` and `rbicepl_g` that sit in the middle of the body chain (they have a mapped node below them).
- The name frequency test keeps out `breastbone`. This is a skinned bone below `torsoupr_g` that has no mapped node below it, and it is in 37 percent of the KOTOR 1 humanoids and 39 percent of the KOTOR 2 humanoids. Without the frequency test the rule would wrongly include it in 10 KOTOR 1 models and 31 KOTOR 2 models. The robe bones are in at most 7.7 percent of the KOTOR 2 humanoids, and the cape bones in at most 9.3 percent of the KOTOR 1 humanoids. The threshold of 25 percent sits in the wide gap between these values and 37 percent. A threshold of 10 percent gives the same result.

### Result of the rule

The counts below exclude the menu scene models (`3dgui`, `mainmenu`, `mainmenu_model` and the numbered `mainmenu` variants). Those are scenes that hold copies of the same characters, and the rule finds the same bones in them.

| Game | Model | Loose bones | Names |
|---|---|---|---|
| KOTOR 2 | `pmbim`, `pfbim`, `pmbnm`, `pfbnm`, `n_jedirobem` | 11 each | `frobe1_g`, `frobe2_g`, `mrobe1_g` to `mrobe3_g`, `lrobe1_g` to `lrobe3_g`, `rrobe1_g` to `rrobe3_g` |
| KOTOR 2 | `pmbmm`, `pfbmm`, `n_darkjhim`, `n_darkjhif` | 9 each | the same without `frobe1_g` and `frobe2_g` |
| KOTOR 2 | `n_talia` | 6 | `frobe1_g`, `frobe2_g`, `mrobe1_g` to `mrobe3_g`, `frontflappiece` |
| KOTOR 2 | `pfbl` | 4 | `frobe1_g`, `frobe2_g`, `mrobe1_g`, `mrobe2_g` |
| KOTOR 2 | `n_darthnihilus` | 12 | `lcape01` to `lcape04`, `mcape01` to `mcape04`, `rcape01` to `rcape04` |
| KOTOR 2 and KOTOR 1 | `n_darthrevan`, `n_darthrevanm`, `n_darthrevanf` | 10 each | `cape01_g` to `cape05_g` and `belt01_g` to `belt05_g` |
| KOTOR 2 and KOTOR 1 | `n_darthmalak` | 27 | `cloth_1_g` to `cloth_27_g` |
| KOTOR 2 and KOTOR 1 | `n_darthmalak02`, `n_darthmalak03` | 5 each | `cape01_g`, `cape02_g`, `cape02_g01` to `cape02_g03` |
| KOTOR 1 | `m40ab_c03_char01`, `m40ad_c01_char01` | 5 each | `cape01_g`, `cape02_g`, `cape02_g01` to `cape02_g03` (rigid pieces) |
| KOTOR 1 | `m44ac_c01_char01`, `p_bastilabb02` | 16 each | `cape01_g` to `cape05_g`, `bflap01_g` to `bflap05_g`, `lshldr01_g` to `lshldr03_g`, `rshldr01_g` to `rshldr03_g` |

This is 18 KOTOR 2 models and 10 KOTOR 1 models. The census already named the 11 KOTOR 2 robe bodies, the Revan and Malak capes and Bastila's cutscene body. Three finds are new. KOTOR 2's `n_darthnihilus` has a 12 bone cape in three chains of four. Both games' Revan models have five `belt` bones next to the capes. KOTOR 1 and KOTOR 2 both hold the 27 bone `n_darthmalak` cape.

The KOTOR 2 supermodels `s_male01`, `s_male02`, `s_female01` and `s_female03` already contain the 11 robe bones and `breastbone` as nodes. The robe bones are therefore part of the skeleton that the animation clips key. Which meshes use them is decided by the body model.

### What the rule misses

| Missed | Models | Why |
|---|---|---|
| Hood bones `mhood_g`, `mfhood_g`, `lhood_g`, `rhood_g` | 9 KOTOR 2 models (`p_handmaidenbb`, `p_handmaidenbd`, `p_kreiabb` and six more Kreia bodies) | The first mapped ancestor is `head_g`. |
| Hood bones `hood_g`, `hoodl_g`, `hoodr_g` | `n_darthrevan` in both games | The same. |
| Sleeve bones `lsleeve1_g`, `lsleeve2_g`, `rsleeve1_g`, `rsleeve2_g` | 13 KOTOR 2 models (`pmbim`, `pfbim`, `pmbnm`, `pfbnm`, `n_jedirobem`, `p_atrisbb` and others) | The first mapped ancestor is a forearm. |
| Robe bones `l_robe_g`, `r_robe_g` | the three Revan models in both games | The first mapped ancestor is a shin. |
| Thigh flaps `lflap01_g` to `lflap05_g`, `rflap01_g` to `rflap05_g` | `p_bastilabb02`, `m44ac_c01_char01` | The first mapped ancestor is a thigh. |
| Shoulder plates `lshldrplate_g`, `rshldrplate_g` | `n_mandalorian` | The first mapped ancestor is a bicep. These are armour, not cloth. |
| Garment bones of other skeletons | `n_rakata` and `c_holorakata` (`flap_g` under `pelvis_g`), `c_ithorian` (`rclothflap_g`, `lclothflap_g`, `flap_g`), `n_selkath` and `c_selkath` (four flap bones under `head_g`), `c_hutt` shoulders, `c_mykal` wing flaps | These models fail the humanoid test. The first two are real garments. The Selkath flaps and the wing flaps are body parts. |
| The skirts that have no bones | the KOTOR 1 Jedi robes `pmbj*`, `pfbj*` and every other skinned skirt in the census | There is nothing to find. This is the case that generated bones would fix later. |

A second rule picks up most of these. It is the same as the first, except that the first mapped ancestor may be any mapped node, and the node must be a skin joint, not a rigid mesh. It finds all the hood, sleeve, robe, thigh flap and shoulder plate bones in the table, and it adds false positives from three groups. They are the face bones of two head-only models (`c_female` and KOTOR 1's `p_juhani`), the Rodian antennae (`lanttop`, `lantlwr`, `ranttop`, `rantlwr`) with the Trandoshan eyelids (`eyellid_t`, `eyerlid_t`), and the Jawa `toque_g`. The antennae and the toque are loose parts, but they are not garments.

### What the rule wrongly includes

- **Nothing in the models of the table was found to be wrong.** Each name matches its mesh. The checks were made by hand on the meshes that carry them (`cape` and `belt` for Revan, `robe_geo` and `apron_geo` for `pmbim`, `backcape` for Malak, `backflaps` for Bastila).
- **Rigid single nodes were included before the chain test was added.** `lflap`, `rflap` and `object02` of `n_darthmalak02`, `object01` of the `spacesuit01` and `watersuit` models, and the rigid `hood` of `m41ad_c01_char15` are rigid meshes attached to the pelvis or the torso. They have no loose parent or child and no clip moves them. The chain test removes them. `frontflappiece` of `n_talia` stays, since it hangs from `frobe1_g`.
- **Models that are scenes of characters.** `3dgui`, `mainmenu` and its variants pass every test, since they hold copies of Revan, Malak, Nihilus and Bastila. They are not extra loose bones.
- **Bastila's `lshldr` and `rshldr` bones.** The rule includes them as cloth, because they are skinned to the `backflaps` mesh. Their name says shoulder, and a hand check could still call them a flap.

### Not measured

- The rule was tested only on models whose skeleton uses the KOTOR humanoid names. A mod that adds its own robe bones would pass the rule only if the bone names are rare in the humanoid models of the install.
- The frequency count includes the supermodel copies and the menu models. This does not change the result, but it means the percentages are not exact shares of the player-visible bodies.
- Whether each bone is animated by a clip was not used in the rule. The census and the animation note give that fact per model.

## Question 4, which chains are neighbours

### The strands

The loose forest is cut into strands. A strand is the path from one root to one leaf. A bone that has several children belongs to several strands, one for each leaf below it. The KOTOR 2 robes have four strands, the front one of two bones and the others of three. The KOTOR 1 Malak cape has eight strands in four trees.

### The rule

Two strands are neighbours when at least one of these holds.

1. **They share a root.** They are two branches of one tree, such as `cloth_15_g` with its two children in the Malak cape.
2. **The skin joins them.** At least 10 vertices of the body's meshes have a weight of 0.2 or more on a bone of each strand. The artists already weighted the mesh across the two strands, so the vertices move together.
3. **They are close and on one side.** The mean distance between the two strands, measured at the points of 20, 40, 60, 80 and 100 percent of their length, is at most 1.5 times the median nearest distance of all strands of the model. The mean angles of the two strands around the vertical axis through the pelvis differ by less than 120 degrees. The angle is `atan2(x - x_pelvis, y - y_pelvis)` and its zero is the front.

The strand length is that of the polyline of its heads and its tip. The rule uses model space, and nothing in it depends on bone names.

### Results

Distances are mean distances in metres. Angles are in degrees, with 0 at the front, 90 to the character's right (the `r` bones are at positive x) and 180 at the back.

**`pmbim`** has four strands. The median nearest distance is 0.26 m, so the limit of the third part is 0.39 m.

| Pair | Distance | Angle gap | Shared vertices | Result |
|---|---|---|---|---|
| left (`lrobe`) and back (`mrobe`) | 0.26 | 64 | 39 | linked |
| back (`mrobe`) and right (`rrobe`) | 0.25 | 63 | 46 | linked |
| front (`frobe`) and left | 0.50 | 114 | 0 | not linked |
| front and right | 0.49 | 119 | 0 | not linked |
| front and back | 0.50 | 178 | 0 | not linked |
| left and right | 0.48 | 127 | 0 | not linked |

The front apron is its own mesh (`apron_geo`) and hangs free. The robe is a chain of three links around the back, from the left side to the right side.

**`pfbl`** has two strands, a front one at 0 degrees and a back one at 180 degrees, 0.31 m apart. The rule links nothing, because the angle gap of 180 degrees is over the limit and no vertex is shared. This is right for a front flap and a back flap.

**`n_jedirobem`** has the same four strands as `pmbim`. The median nearest distance is 0.30 m.

| Pair | Distance | Angle gap | Shared vertices | Result |
|---|---|---|---|---|
| left and back | 0.30 | 67 | 29 | linked |
| back and right | 0.30 | 66 | 36 | linked |
| front and left | 0.58 | 111 | 24 | linked by the skin |
| front and right | 0.59 | 117 | 21 | linked by the skin |
| front and back | 0.58 | 178 | 0 | not linked |
| left and right | 0.55 | 133 | 0 | not linked |

Here the robe is a closed ring of four links (front, left, back, right) with the front opening at the front-back pair. The skin joins the front strand even though its distance (0.58 m) is over the geometric limit. This is why the skin is a part of the rule. The geometric part alone gives only the left, back and right links.

**KOTOR 1 `n_darthmalak`** has eight strands in four trees. Each tree is a wing.

| Tree (root bone) | Side and height | Strands | Shared vertices | Result |
|---|---|---|---|---|
| `cloth_15_g` | front of the hips | 2 | 52 | linked by the shared root |
| `cloth_9_g` | right hip, reaching x = 0.71 m | 2 | 30 | linked by the shared root |
| `cloth_12_g` | left hip, reaching x = -0.85 m | 2 | 25 | linked by the shared root |
| `cloth_1_g` | right shoulder, reaching x = 0.92 m | 2 | 22 | linked by the shared root |

No pair of strands from different trees shares a vertex. The right shoulder tree and the right hip tree are 0.41 to 0.62 m apart and 19 to 53 degrees apart, which is farther than the limit of 0.17 m (1.5 times the median nearest distance of 0.11 m). The skin, the roots and the geometry therefore agree. The Malak cape is four separate wings, each a pair of branches. The solver must treat a branch as a tree of points and must not assume a chain.

### Why three parts

- The geometric part alone cannot tell a robe from two flaps. `pfbl` is the example, since its two strands are only 0.31 m apart on opposite sides.
- The skin alone misses the apron of `pmbim`, which is a separate mesh. It does link the front of `n_jedirobem`, which the geometry alone would not.
- The shared root is needed for the branches of one tree, since the strands of the right shoulder tree are 0.20 m apart and over the limit of the third part.

The links become distance and bending constraints between points of the two strands at the same fraction of their length. Links from the skin and from a shared root should be stiffer than links from the geometry alone.

## What remains unsure

- The cost of the solver. All figures for the cost of the walk and the solve are estimates from counts of operations. A prototype must measure them with the bench scenes.
- Whether the solver runs well in `Look::Clips` bodies (cutscene bodies and the creatures). The ordering is the same, but the held nodes of those bodies come from `clips.held` (`cast.rs:3177`), which must also drop the loose bones.
- Whether a node scale is used anywhere on a body. The walk assumes a rigid transform with scale 1, as `NodeIr` has no scale. `severing` and the death code might scale nodes, and this was not checked.
- Whether the base `Transform` of a loose bone that is changed by `at_rest` and then changed back by the solver costs anything noticeable. Each frame writes the rest rotation and then the solved one, so the bone is marked changed twice. It is likely cheap, and it was not measured.
- Whether the strongly branching K1 Malak tree needs the weighted-swing step or just the first child. The data has only one such model.
- How the capsules look under the robe bodies. The note on body collision already warned that the leg capsules are fitted to skin that includes the robe's own vertices on K1 Jedi robes. For the K2 robes the leg skin is separate, so this is a smaller risk, but it was not measured.
- Whether the threshold of 25 percent is stable. The gap in the data (7.7 percent and 9.3 percent against 37 percent) is large, but a mod could close it.
- The Selkath and wing flaps, and the Rakata and Ithorian garments, need either a skeleton map for those bodies or a data flag, and neither exists.
