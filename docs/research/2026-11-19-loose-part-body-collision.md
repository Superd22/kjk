# Collision of loose parts with the character's own body

This note was written on 2026-11-19. It is one axis of the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts"). It asks how robes, capes and hoods can be kept from passing through the character's own body, and whether the ragdoll capsules can serve as the colliders.

## Summary

- The 14 capsules are fitted once per body model, from the rest-pose skeleton and the skin vertices. A capsule's radius is the distance that keeps 65% of the part's skin vertices inside. Nothing in the fit depends on the character dying, so the same capsules can be posed every frame from the animated skeleton. Each capsule only needs its node's current frame.
- The 14 capsules fit humanoids only. For other skeletons the ragdoll code uses `Template::auto`, which makes up to 28 capsules for flesh and 16 boxes for metal. Those are fitted from the mesh alone and are not guaranteed to be sensible as cloth colliders. A robe or cape is worn by a humanoid, so the 14 capsules cover the case that matters.
- The 14 capsules are fitted to the naked skin of the body model. A robe is a separate layer that sits outside the skin by several centimetres. The cloth therefore needs the capsule radius plus a margin, and for a hanging robe skirt the capsules do not describe the shape of the garment. They only stop legs and arms from poking through the cloth.
- The recommended design has three layers. The first layer is a skinned backstop, which keeps each loose vertex on the outer side of the skinned surface and costs nothing per collider. The second layer is a small set of capsules (about 8 to 10: two thighs, two shins, pelvis, chest, and optionally two upper arms), tested with a hand-written closest-point function inside the cloth solver. The third layer is a ground plane, taken from the walkmesh height under the character.
- Rapier queries are not recommended for this job. A hand-written capsule test costs a few dozen floating point operations per vertex and capsule. A Rapier query adds a broad phase traversal and a call boundary for each vertex.
- Signed distance fields are not recommended. They must be baked per pose or rebuilt every frame for a character that flips and kicks, and they need hand-tuned grids.
- Tunnelling during fast moves is handled by substepping the cloth, by moving the capsules through their motion in small increments, and by clamping each vertex's speed. The skinned backstop is the safety net when all of these fail.
- Estimated cost per character: about 0.1 to 0.25 milliseconds for a 600-vertex robe with 10 capsules and 3 substeps on one core. These numbers are estimates from operation counts and have not been measured. They must be measured before the choice is final.

## What the code does today

### The sway

The danglymesh sway is in `crates/game/src/dangly.rs`. Each vertex with a constraint above 0 is a damped spring. The vertex is clamped to a box around its rest place. The box is `displacement × (1 − constraint/255)` along each axis. For most hair, flaps and hoods that is 2.5 to 3.5 cm. The sway runs on the CPU, writes the new vertex places into a shader buffer, and is view-only. The step function (`fn step` at `crates/game/src/dangly.rs:371`) works on the mesh node's transform, not on the skeleton, so it has no knowledge of the legs.

A vertex's rest place is stored in the node's space (`DanglyIr::rest` in `crates/asset_odyssey/src/model.rs:134`). The box limit is already a very weak "skinned backstop in reverse". It keeps the vertex near the skinned position, but in all directions, including into the body.

### The ragdoll capsules

`docs/RAGDOLLS.md` and `crates/ragdoll/src/body.rs` describe the fit. The facts that matter here are these.

1. The humanoid plan is in `crates/ragdoll/src/humanoid.rs` (`PLAN`). It names 14 parts: pelvis, belly, chest, head, two upper arms, two forearms (with the hand), two thighs, two shins and two feet.
2. `Template::fit` (`crates/ragdoll/src/body.rs:341`) takes the plan and a `Skeleton`. A `Skeleton` is the node list in the rest pose plus the skin vertices with their bone weights (`Vertex` at `crates/ragdoll/src/body.rs:20`).
3. Each skin vertex is given to the part of the nearest plan node at or above the node that moves it most (`skin_by_part`, `crates/ragdoll/src/body.rs:517`).
4. A limb part's capsule runs from its node to the next node (for example thigh to shin), plus a few centimetres past it for some parts. The ends are then drawn in by half the radius (`TUCK`, 0.5) so neighbours overlap less at their joints.
5. The radius is found by `fit_radius` (`crates/ragdoll/src/body.rs:530`). It sorts the distances of the part's vertices to the capsule axis and takes the distance at the 65th percentile (`OUTSIDE` is 0.35). The result is clamped to the plan's thickness range. The thigh range is 6 to 9.5 cm, the upper arm range is 3.5 to 6.5 cm, and the forearm range is 3 to 5 cm. If a part has fewer than 8 vertices, the maximum of the range is used.
6. Each capsule is stored in its node's rest frame (`Solid::Capsule { ends, radius }`). Posing it later is therefore one rigid transform of two points per capsule, from the node's current frame.

The data used is entirely the skeleton and the skin, so nothing is placed by hand per model. The plan's names and thickness ranges are written once for the shared KOTOR humanoid skeleton. `docs/RAGDOLLS.md` reports that 1073 of the 1074 humanoids in KOTOR 1 fit, as counted by `kjk clips k1`.

### Can the capsules be posed every frame?

Yes. The death code already poses bodies from the animation. The living body is drawn from its animated skeleton, so each capsule's world frame is the node's animated frame multiplied by the capsule's stored local ends. That is 14 matrix-vector products. The ragdoll world is not needed for this. A cloth solver only needs the 28 end points and 14 radii, which can be recomputed every frame (and every substep, by interpolating between the previous and the current frame).

Two caveats follow from the code.

- `Template::fit` can fail. Its error cases include a skeleton with no hips node, a bone of zero length, and a plan part whose parent comes later. The fit also needs the skin, which the game gets from `cast::rest_of`. A loose-part system must run the fit at model load and cache the result per body model, as the ragdolls already do.
- A body whose head is its own model (appearance model type B, which is every commoner, every Republic soldier and every player body) has no `neck_g` or `head_g`. The plan substitutes `headhook` for both (`Plan::stand_ins`). This is already handled.

### What fails for non-humanoid or odd skeletons

- Creatures and droids use `Template::auto`. Every node that carries real mesh becomes a body. The number of bodies is up to 28 for flesh and 16 for metal, and the metal ones are boxes. The krayt dragon, the bantha and the rancor needed special handling for the root and the joints (`docs/RAGDOLLS.md`, issue 558). None of those problems matter for a collider, because a collider has no joints. The capsules would still be sensible shapes, but the largest radius is 1.98 m on the krayt dragon's chest, and the shapes are not tuned for cloth.
- Almost no non-humanoid in KOTOR wears a robe or cape, so a first version can switch the whole feature off for any body that does not fit the humanoid plan. `doll_fits` (`crates/game/src/ragdolls.rs:399`) already reports this per model.
- The one humanoid that does not fit is not named in the docs. A loose-part feature must handle the `Err` from `Template::fit` by turning collision off for that model.
- A capsule fitted to the skin of a body model describes the body under the robe. Many KOTOR robes are part of the body model itself. For those, the skin includes the robe's own skirt vertices. Those vertices are bound to the leg bones, so they enlarge the leg capsules, and the 65th percentile can pull the thigh radius to the top of its range (9.5 cm). The capsule would then be too large for the true leg and too small for the skirt. This was not measured. It must be checked on Juhani and on the Jedi robe bodies before relying on the radii.

## Collider options

### Option A: all 14 capsules

- Number of tests: up to 14 per vertex per substep.
- Strength: it already exists, and it covers the arms, which matter for a cape whipping around in a spin.
- Weakness: a robe skirt rarely touches the arms, belly or head, so most of those tests always fail. The pelvis, belly and chest capsules overlap each other (they are in group `Core`, which the ragdoll lets overlap), so a vertex can be pushed out of one into another.
- Mitigation: assign each cloth vertex a short list of capsules from its skin weights at rest, for example the 3 capsules whose bones weigh most on it plus their neighbours. A hem vertex bound to the legs tests only the two thighs, the two shins and the pelvis. This reduces the work to about 5 tests per vertex.

### Option B: fewer capsules (thighs, shins, pelvis, chest)

- Number of tests: 6 to 8 per vertex.
- This is the choice that Unreal's cloth documentation points toward. Unreal takes collision from the character's Physics Asset, and the Chaos cloth tool's text says "If this cloth asset is for a character, use its Physics Asset here to get proper collision for the cloth simulation" ([Unreal Engine, Clothing Tool](https://dev.epicgames.com/documentation/en-us/unreal-engine/clothing-tool-in-unreal-engine)). A Physics Asset is normally a rough set of capsules, so Unreal's own practice is close to the 14 capsules here.
- Arms are left out. A cape or a wide sleeve can then pass through an arm. That matters little for a hood and a robe skirt, and more for a cape in a flip.

### Option C: spheres and sphere pairs (tapered capsules)

- NvCloth, the cloth library used by many games, supports collision spheres, and two spheres can be joined into a capsule that tapers between two radii ([NvCloth documentation](https://nvidia-omniverse.github.io/PhysX/nvcloth/1.1.6/index.html)). To my recollection the limit is 32 spheres, with capsules made from sphere index pairs, and 32 planes. I did not re-read the page for this note, because the direct fetch of the documentation file failed, so these limits must be checked before they are quoted elsewhere.
- Strength: a thigh is wider at the hip than at the knee. A tapered capsule fits that better than the ragdoll's straight capsule, so the robe sits closer to the leg.
- Cost: a sphere pair test is the same as a capsule test plus one linear interpolation of the radius.
- The fit would need a second radius at each end. The existing `fit_radius` could be run twice, once for each half of the bone, using the same skin vertices. This is a small change to the fit and still uses only the skeleton and mesh.

### Option D: signed distance fields

- A signed distance field stores the distance to the body on a voxel grid. A vertex test is one trilinear lookup, so the cost does not depend on the number of capsules.
- The grid must follow the body. For a character that is skinned and animated, the field must be rebuilt or deformed every frame, or stored per bone and blended. Both cost far more than 10 capsule tests per vertex for a body with a few hundred cloth vertices.
- A field baked in the rest pose is only valid in the rest pose.
- It is not recommended.

### Option E: Rapier queries

- Rapier 0.36 is already used for the ragdolls. A collider per capsule could be placed in a query-only world, and each vertex could ask for the closest point or a shape cast.
- Each query goes through the broad phase and a dynamic dispatch. That costs far more than a hand-written capsule test, and the ragdoll world is built for rigid bodies and joints, not for hundreds of point queries per frame.
- The ragdoll world also runs at 120 steps a second only for bodies that exist. A living character has no bodies in it.
- It is not recommended for the per-vertex test. Rapier could still be used for rare large tests, such as whether a whole cape bounds a wall.

### Option F: hand-written closest-point test in the solver

- The test for a point `p` against a capsule with ends `a` and `b` and radius `r` is the one in `segment_distance` (`crates/ragdoll/src/body.rs:540`): clamp the projection of `p` on the segment, then compare the distance to `r`. If the point is inside, push it out along the line from the closest point to `p`, to a distance of `r` plus a margin.
- About 25 floating point operations plus one square root per test, and the code already exists in the ragdoll crate.
- This is the recommended way to test.

## Stopping tunnelling in fast moves

kjk's movement copies Jedi Academy, so a character can flip, spin and kick quickly. The legs of a 1.8 m character rotate about the hip at up to about 10 radians a second in a kick, which moves a shin's end by about 4 m per second, or 6.7 cm in one 60 frames-a-second frame. A thigh capsule has a radius of about 6 to 9.5 cm. A single step per frame can therefore move a capsule almost its whole radius, and a cloth vertex can end up on the wrong side of the capsule.

Measures, from cheapest to most costly.

1. **Substepping.** Split each frame into 2 or 3 solver substeps, and interpolate the capsules' end points between the previous and the current frame. Position-based dynamics papers advise small substeps over many iterations for this reason ([Macklin et al., "Small Steps in Physics Simulation", 2019](https://matthias-research.github.io/pages/publications/smallsteps.pdf)). At 3 substeps the capsule moves at most about 2.2 cm per substep in the kick above, which is well under a thigh's radius.
2. **Clamp each vertex's travel.** A vertex may not move more than a fraction of the smallest capsule radius in one substep. This is a cheap safety net.
3. **Swept test.** Test the segment from the vertex's previous place to its new place against each capsule, and stop at the first hit. This is continuous collision for a point. It costs about three times a point test and is needed only for the fastest cases.
4. **Skinned backstop.** See the next section. It does not prevent tunnelling in the physical sense, but it brings any vertex that has gone through back to the outer side.
5. **Snap back.** The existing sway snaps all vertices back to rest if the mesh jumps more than 5 m (`JUMP_M`). A similar rule for the case where a vertex is farther than a set distance from its skinned place is simple and prevents long-lasting glitches.

Rapier's own continuous collision detection is not available for cloth vertices, since they are not Rapier bodies.

## The skinned backstop

The skinned backstop keeps each loose vertex on the outer side of the skinned surface. Unreal's cloth tool uses the same idea. Its Max Distance is "the maximum distance any point on the cloth can move from its animated position", its Backstop Distance is "the offset used from any point on the cloth to limit movement within the Max Distance", and its Backstop Radius "uses a sphere radius that when intersected with the Max Distance will prevent any painted point on the cloth from moving into that region" ([Unreal Engine, Clothing Tool](https://dev.epicgames.com/documentation/en-us/unreal-engine/clothing-tool-in-unreal-engine)). In short, the cloth point is allowed to move inside a sphere around its animated position, and the part of that sphere that lies behind the animated surface is cut away.

For kjk this would work as follows.

- For each loose vertex, the sway already has the skinned (animated) place, which is its rest place carried by the node. The vertex's skinned normal is the animated normal.
- After the spring step, compute `d = dot(position − skinned_place, skinned_normal)`. If `d` is below a small negative number (the backstop distance), move the vertex back along the normal until `d` equals it.
- The cost is one dot product and, rarely, one correction per vertex. No colliders are tested.
- The danglymesh data already gives each vertex a constraint, which can scale the sphere, in the way Unreal's Max Distance paint does. The data has a "displacement × (1 − constraint/255)" reach already (`reach` at `crates/game/src/dangly.rs:422`).

Limits of the backstop:

- It only works while the skinned surface is on the correct side of the body. A robe skirt bound to the leg bones is, and it follows the leg when the leg kicks. A freely swinging cape end that is bound to the spine is not: when the spine is bent forward during a flip, the cape's skinned place is itself inside the legs.
- It is a one-sided limit, so the cloth can still enter a leg that is not part of the vertex's own bone chain, for example the opposite leg kicking across.
- A cloth much more free than the 2.5 to 12.5 cm of today's data (a real cape) needs the colliders as well.

The two methods fit together. The backstop handles the thin garment close to the body, and the capsules handle large departures from the skinned place.

## Floor and walls

The floor matters for robe hems, cape ends and hair in three cases: a character that crouches or sits, a landing after a flip, and a character lying down.

- The sim has the walkmesh height under each character. A simple ground plane through that point, with the walkmesh's face normal, is enough for a standing character on level ground. NvCloth's own collision shapes include planes, and Unreal's cloth supports collision planes as well, so the idea is standard. This was not re-checked in the NvCloth documentation for this note.
- A hem vertex pushed up by the plane would stay about 1 cm above the walkmesh. The plane should be placed at the current height under the vertex's own horizontal position, not under the character's centre. Otherwise a robe on a staircase or a slope would hang through the next step. That needs one walkmesh height lookup per hem vertex, or per group of hem vertices.
- During a wall jump the character's body is close to a wall, and a cape or robe skirt would be pushed through the wall. KOTOR walls are single faces (`docs/RAGDOLLS.md` says "a KOTOR wall is a single face"). The ragdoll code already gathers the level's faces around the pelvis in a box (`GATHER`, 6 m below and 3 m above, with at least 5 m to each side). A cloth system could reuse that list, but testing every cloth vertex against every nearby face is far too expensive. A cheaper rule is to test only the cloth's few outermost vertices (the hem and the cape's tail), and only against faces within 1 m.
- Falling robes are not the only problem. A landing from a high flip compresses the character's body into the floor for a few frames. The plane test then pushes the hem up, and the robe jumps. A damped, one-sided push, limited per substep, avoids this.
- Conclusion: a ground plane from the walkmesh is enough for the first version. Wall collision is optional, and a cape that passes into a wall for a few frames during a wall jump is acceptable for a first release. A hem-only wall test with the ragdoll's face list can come later.

## Cost per character

The estimates below use these assumptions, which are mine and have not been measured: a robe or cape has 400 free vertices (kjk's sway rewrites each moving mesh on the CPU, and the issue says nobody has counted), 3 substeps per frame, and about 30 floating point operations per capsule test. One core does roughly 2 to 4 billion simple floating point operations a second, so 100 000 operations take about 0.03 milliseconds.

| Option | Tests per vertex per substep | Tests per frame (400 vertices, 3 substeps) | Estimated time |
| --- | --- | --- | --- |
| Skinned backstop only | 0 (one dot product) | 0 | under 0.01 ms |
| 5 capsules chosen per vertex from the skin weights | 5 | 6 000 | about 0.05 ms |
| 8 capsules (thighs, shins, pelvis, chest, upper arms) | 8 | 9 600 | about 0.08 ms |
| 14 capsules | 14 | 16 800 | about 0.14 ms |
| 14 capsules with swept tests | 14 swept (about 3 times as much) | 16 800 swept | about 0.4 ms |
| Rapier point query per vertex | 1 query (with broad phase, about 50 to 200 times a capsule test) | 1 200 | about 0.2 to 0.6 ms, plus building a query world |
| Signed distance field | 1 lookup, but a per-frame rebuild | 1 200 | rebuild cost dominates |

Notes on the table:

- A crowd of 20 characters, each with the 8-capsule option, would cost about 1.6 ms a frame on one core. With the existing sway skipping off-screen meshes (`crates/game/src/dangly.rs`, "a mesh the camera did not see in the last frame does not move"), the same rule can skip collision.
- The numbers do not include the cost of reading the animated skeleton, which the game already computes for drawing.
- The ragdoll capsule posing is 14 transforms per character, which is well under 0.01 ms.
- Per the project rule, this must be benchmarked, and the feature becomes a setting if it costs frames. The natural setting levels are: original sway only, sway with backstop, and sway with capsules and floor.

## Recommendation

1. Start with the skinned backstop on the existing danglymesh sway. It is almost free, it uses only data the games already provide (the skinned place, the normal and the constraint), and it stops the most visible case, a robe vertex passing into its own leg.
2. Add capsules next. Use a reduced humanoid set (thighs, shins, pelvis and chest at least, upper arms for capes), fitted by the existing `Template::fit` and posed from the animated skeleton each frame. Choose per vertex, at load, the 4 to 6 capsules that its skin weights reach, so the per-vertex cost stays near 5 tests.
3. Test with the hand-written closest-point function from the ragdoll crate, inside the cloth step. Do not use Rapier queries and do not build signed distance fields.
4. Substep 2 or 3 times and interpolate the capsules between frames. Add the per-vertex travel clamp. Add a swept test only if the kick and spin cases still tunnel after measuring.
5. Add a ground plane from the walkmesh height under each hem vertex. Leave walls for later.
6. Before building it, measure three things on real models: the vertex count of each danglymesh in the robes and capes, the true radius of the thigh capsule on a body that has a robe in its skin (see the caveat above), and the cost of the current CPU sway for a crowd. Without those three numbers the table above stays a guess.
7. If a tapered fit is wanted later, give each capsule two radii by running `fit_radius` on each half of the bone. This stays derived from the skeleton and mesh.

## Sources

- Unreal Engine, Clothing Tool: https://dev.epicgames.com/documentation/en-us/unreal-engine/clothing-tool-in-unreal-engine (read for this note: Physics Asset collision, Max Distance, Backstop Distance, Backstop Radius).
- NvCloth documentation: https://nvidia-omniverse.github.io/PhysX/nvcloth/1.1.6/index.html (not re-read for this note; the sphere, capsule and plane limits are from memory).
- Macklin, Storey, Lu, Terdiman, Chentanez, Jeschke and Müller, "Small Steps in Physics Simulation", 2019: https://matthias-research.github.io/pages/publications/smallsteps.pdf (cited for substepping; not re-read for this note).
- Code read: `docs/RAGDOLLS.md`, `crates/ragdoll/src/body.rs`, `crates/ragdoll/src/humanoid.rs`, `crates/game/src/dangly.rs`, `crates/game/src/ragdolls.rs`, `crates/asset_odyssey/src/model.rs`.
