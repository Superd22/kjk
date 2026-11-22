# Which solver should move capes and robes

This note was written on 2026-11-19. It is one research axis for the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts"). It asks which solver kjk should use to move capes and robes, given how the engine is built and what the owner requires.

## Summary

The recommended solver is XPBD with small steps, run on the CPU in the view. The solver works in the character's own frame, so the cloth feels only a chosen fraction of the character's world motion. It has four kinds of constraints. They are a distance constraint on every mesh edge, a bending constraint across every pair of triangles that share an edge, a long range tether from each vertex to its nearest pinned vertex, and a hard maximum distance from the skinned position. Collision uses the 14 body capsules that the ragdolls already fit to each humanoid.

The reasons are these.

- The current per-vertex spring has no link between neighbouring vertices. If its limit grows from 3 cm to 30 cm, neighbours can drift apart and the cloth tears or crosses itself. It also limits each vertex to an axis-aligned box in the mesh's local space, so the limit changes with the character's facing.
- XPBD is stable under any acceleration, because it moves positions instead of integrating forces. That is the property the Jedi Academy style moves need. Spins, flips and wall jumps will produce accelerations far larger than anything in the original games.
- Every parameter can come from the model data. The danglymesh constraint becomes the vertex's freedom, the tightness becomes a stiffness, the period becomes the damping, and the rest lengths, tether lengths and bending angles come from the mesh topology. A robe skirt without danglymesh data can get its freedom from its skin weights.
- The original spring is one special case of the solver. It is the "attachment to the rest place" constraint with every other constraint switched off. This means the setting "original sway" and the setting "stronger movement" can share one code path, and the owner's rule about using only the games' data is kept.
- The cost is small. Section "Cost" estimates 0.2 to 0.5 ms per frame for one robe of 1,000 vertices with 8 steps on one CPU core. These numbers are estimates and the prototype must measure them.
- A CPU solver can write its result through the existing slot buffer, which already carries the current and the previous position of every vertex. The TAA and DLSS motion vectors then keep working without any renderer change.

A GPU compute version is not recommended as the first step. It would help only if a crowd of cloth characters makes the CPU cost visible in the benchmarks.

Spring bones generated from the mesh are a good second choice for hair, ponytails and tails. They are cheaper and very stable, but they deform a wide cape or a skirt badly, and they need new bones added to the skinning.

## What the engine already has

The facts below come from reading the code in this repository on 2026-11-19.

- The sway is in `crates/game/src/dangly.rs`. Each danglymesh keeps its vertices in the mesh node's space. Every frame, `Dangly::step` carries each free vertex back by the mesh's travel since the last frame, turns it with the mesh's turn, adds a pull towards the rest place and a damping, and then clamps the vertex into a box around its rest place.
- The data for each vertex is a constraint from 0 to 255, plus three numbers for the whole mesh. These are the displacement (the box size), the tightness and the period. They are stored in `DanglyIr` in `crates/asset_odyssey/src/model.rs`, together with the rest position of every vertex.
- In the code, the pull per second squared is `constraint × tightness × 0.5`. The damping per second is `period × 1.5`. The box half size along each axis is `displacement × (1 − constraint / 255)`. A vertex with constraint 0 is never moved.
- The frame time is clamped to between 12.5 ms and 35 ms. A mesh that moves more than 5 m in one frame snaps back to rest. A mesh that the camera did not see in the last frame is skipped.
- The wind in `crates/game/src/wind.rs` adds a push to every free vertex.
- The result goes to the renderer through a storage buffer. `SwaySlots` gives each mesh a run of slots, and each slot holds two positions of three floats: the position now and the position in the previous frame (`SLOT_BYTES` is 24). The shader `crates/game/src/sway.wgsl` and `crates/game/src/dangly.wgsl` read these. The previous position is what gives the sway its motion vector for TAA and DLSS.
- The engine is built on Bevy 0.19.1, which uses wgpu. `crates/game/Cargo.toml` names wgpu 29.0.3 for the room occlusion queries.
- `crates/ragdoll` uses Rapier 0.36 and `ragdoll::humanoid::PLAN`, which fits 14 capsules to a humanoid body. `docs/RAGDOLLS.md` describes this.
- `docs/SKELETON-SIM.md` says the simulation must be deterministic. The cloth solver must therefore never write into the simulation. It reads poses from the view only.

One consequence matters for design. The sway code works on a mesh in its node space, and a danglymesh is a rigid node mesh that hangs on a bone. A plain skinned robe skirt is different. Its vertices are moved by several bones in the vertex shader. A solver for such a skirt must give its result in a form the skinning shader can add to the skinned position. I did not read the skinning shader for this note, so how best to inject the result is the first thing the prototype must check.

## What each solver would do

### The current spring with a larger limit

This is the cheapest option. Each free vertex is an independent damped spring around its rest place. The cost per vertex is about 40 floating point operations per frame with no substeps.

The mapping from the data is already done. The constraint scales the pull and the reach, the tightness scales the pull, and the period scales the damping.

There are four problems when the limit is enlarged to 20 or 40 cm.

- A vertex does not know its neighbours, so adjacent vertices can move apart by more than the cloth's true length. A cape would stretch and show gaps in its texture.
- The limit is a box in node space. When the character turns, the box turns with it, so a cape hangs differently when the character faces north and when it faces east.
- Nothing prevents a vertex from passing into the body.
- The explicit step is stable only while the spring's pull times the square of the frame time stays below 4. The current constants stay inside this, but a larger tightness for a stronger look would not.

The current spring is still the right choice for the default mode, for leaves in the wind, and for any mesh whose displacement is a few centimetres. It is the "original sway" the owner wants as the default.

### Verlet or PBD cloth

Position based dynamics (PBD) moves the vertices by their velocity and gravity, and then repeatedly moves them again so that constraints hold. Müller and co-authors describe it in "Position Based Dynamics" (https://matthias-research.github.io/pages/publications/posBasedDyn.pdf). Matthias Müller's "Ten Minute Physics" series gives short working code for cloth, including the dihedral bending constraint (https://matthias-research.github.io/pages/tenMinutePhysics/index.html).

The constraints come from the mesh.

- A distance constraint for each edge of each triangle, with the rest length taken from the rest pose.
- A bending constraint for each pair of triangles that share an edge. It can be a distance constraint across the two opposite corners, which is cheap, or a dihedral angle constraint, which is more accurate.

The mapping from the danglymesh data is the following.

- The inverse mass of a vertex is its freedom. The freedom is 0 when the constraint is 0, so that the vertex is pinned. Otherwise it is `1 − constraint / 255`, the same factor the original uses for the reach. A vertex with constraint 255 would have freedom 0 and also be pinned, as the original data says it cannot stray.
- The distance stiffness between two vertices is derived from the tightness and the constraints at both ends.
- The damping is the original's own number. Each step multiplies the velocity by `1 − period × 1.5 × dt`.

The weakness of plain PBD is that its stiffness depends on the number of iterations and the step size. A cape that looks right at 60 frames per second looks stiffer at 144. That makes the setting hard to tune and is the reason XPBD exists.

The cost per vertex per iteration is about 100 to 150 floating point operations. A triangle mesh has about three edges per vertex, and each distance constraint costs about 35 operations. The bending constraints add another 1.5 per vertex at about 60 operations each, and the integration step adds about 20.

### XPBD with small steps and long range tethers

Extended PBD (XPBD) adds a compliance to every constraint. The compliance is the inverse of a stiffness in real units, so the cloth looks the same at any step size. The paper is Macklin, Müller and Chentanez, "XPBD: Position-Based Simulation of Compliant Constrained Dynamics" (https://matthias-research.github.io/pages/publications/XPBD.pdf).

The "small steps" paper by Macklin and co-authors, "Small Steps in Physics Simulation" (https://mmacklin.com/smallsteps.pdf), shows that one solver iteration per substep with many small substeps converges better than many iterations per large step. For cloth this means 6 to 10 substeps per frame with one pass over the constraints in each. It also removes most of the stretching under fast motion, which is the situation the Jedi Academy style moves create.

The long range attachment method is from Kim, Chentanez and Müller, "Long Range Attachments - A Method to Simulate Inextensible Clothing in Computer Games" (listed on https://matthias-research.github.io/pages/publications/publications.html). Each free vertex gets one extra constraint. The constraint says that the vertex may not be farther from its nearest pinned vertex than the rest length of the shortest path along the mesh edges. This removes the "rubber band" look of a long cape that stretches under a fast fall, and it needs no extra iterations to do so.

The mapping from the data works as follows.

- The pinned set is the set of vertices with constraint 0. The tether length of every other vertex is the geodesic distance, measured along mesh edges in the rest pose, to its nearest pinned vertex, multiplied by 1.02 to 1.05.
- The original spring becomes one more XPBD constraint, which attaches each vertex to its own rest place. Its compliance is the inverse of the pull, `1 / (constraint × tightness × 0.5)`, with every vertex given a mass of 1. In the original-sway mode this is the only constraint switched on, plus the box clamp. This is the exact special case that shows the solver extends the games' rule instead of replacing it.
- The maximum distance from the skinned position is `freedom × limit`. The limit depends on the mode. In the original mode it is the mesh's `displacement`. In the stronger mode it is a larger value taken from the mesh, for example 0.5 times the shortest geodesic distance between the pinned vertices and the tips.
- The period becomes the velocity damping, as above.

Stronger movement therefore needs no hand-authored asset. It needs the original numbers, the mesh edges, and one scale value that is a setting.

The cost per vertex per substep is about 150 operations with distance, bending and tether constraints, and about 30 more for collision. With 8 substeps, a 1,000 vertex robe costs about 1.4 million operations per frame.

### Spring bones or chain bones generated from the mesh

VRM spring bones (https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_springBone-1.0/README.md) move a chain of bones. Each bone is a point with an inertia, a stiffness back to its animated direction, a drag, a gravity and a collision sphere. The bone length is kept fixed. UniVRM (https://github.com/vrm-c/UniVRM) is the reference implementation. Kawaii Physics for Unreal (https://github.com/pafuhana1213/KawaiiPhysics) is a similar solver with a smaller feature set. Unreal's AnimDynamics node is the same idea with a spring between chain links. Magica Cloth 2 for Unity offers both cloth and spring bone modes. It is a commercial asset and I found no public source repository for it, so I did not read it.

The way to generate chains from a mesh is to cluster free vertices by their geodesic distance and angle around the pinned ring. Each column of vertices below one pinned root becomes one chain. A bone would then be added to the skin for each chain link.

The mapping is simple. The chain root is a pinned vertex. The stiffness is the tightness. The drag is the period. The freedom of a link is the mean freedom of its vertices.

The cost is very low. One bone costs about 50 operations per substep, and a skirt needs 20 to 60 bones.

There are three problems for capes and robes.

- A wide cape is a surface and not a set of independent strings. Neighbouring chains need extra constraints that link them side to side, and that makes the solver a cloth solver again.
- New bones must be added to the skinned mesh. The vertices have at most four influences in Bevy's skinning, and the original vertices already use them for the leg bones.
- Danglymesh vertices are not skinned at all. They would need a rig that the games never had.

Spring bones suit hair, ponytails, hood tips and tails very well. They should be the fallback for those parts, not the first target for capes and robes.

### Limits borrowed from skinned cloth

Game cloth systems limit the cloth by the animated pose. NVIDIA NvCloth (https://github.com/NVIDIAGameWorks/NvCloth) and Unreal's clothing tool (https://dev.epicgames.com/documentation/en-us/unreal-engine/clothing-tool-in-unreal-engine) use three limits. These limits fit XPBD directly and are the main answer to the acrobatics problem.

- **Maximum distance.** Each vertex has a radius around its skinned position. The cloth can never leave that sphere. This is a hard guarantee. Even if the solver fails, the cloth stays near the character. In the danglymesh data, this radius is `displacement × freedom` in the original mode.
- **Backstop.** The cloth may not move behind a surface offset from the skinned position, so it cannot sink into the body. For kjk, the body capsules from the ragdoll fit do this job without extra authored data.
- **Inertia scaling.** The solver runs in the character's root frame. Each frame, the root's change in position and orientation is applied to the particles only by a fraction. A fraction of 1 means the cloth is fully carried with the character and feels no world acceleration. A fraction of 0 means the cloth stays in the world and drags behind. The current spring is effectively at 0 for both travel and turn. Magica Cloth and Unreal both expose a similar value.

The inertia value matters most for the Jedi Academy style moves, and these rules are recommended.

- Clamp the root's translation per substep to about 0.5 m and its rotation per substep to about 60 degrees, so a single frame of a spin or a teleport cannot throw the cloth.
- Apply 0.3 to 0.6 of the root's linear motion and 0.2 to 0.4 of its angular motion. These starting values are estimates and must be tuned by watching the cloth.
- Keep the existing rule that a mesh moving more than 5 m in one frame resets to its skinned pose.

### GPU compute against CPU

kjk uses Bevy with wgpu, so compute shaders are available. A GPU cloth solver would run one thread per vertex. Gauss-Seidel order, which PBD relies on, does not map well to a GPU. A GPU version must either colour the constraint graph and run one dispatch per colour, or use Jacobi updates and about two to three times more iterations. Each substep needs at least one dispatch, so 8 substeps need at least 8 dispatches and barriers per cloth per frame.

For this game the CPU is the better start for these reasons.

- A scene shows a handful of cloth characters. Ten robes of 1,000 vertices with 8 substeps cost about 14 million operations, which is a small fraction of one core.
- The result has to go to the renderer anyway, and the existing slot buffer upload is already written for CPU data.
- Collision data, tethers and the root's motion are all easier to prepare on the CPU, because the poses live there.
- Determinism is not a concern for the view, so a GPU version could be added later without a rule change.

The GPU version becomes attractive only if the benchmarks show the CPU time of a crowd, for example thirty cloth characters in a cantina. In that case the solver should be a separate setting, and the same constraint data can be uploaded to storage buffers. The previous position buffer would be swapped between frames on the GPU, which is what TAA needs.

## Giving a skinned robe skirt its data when it has no danglymesh

Some robe skirts are plain skinned meshes. They have no constraint values, so the solver must derive them. The only inputs are the mesh and its skin weights, which are both the games' own data. This is what I recommend.

1. Pinned vertices are those whose skin weights sit entirely on bones at or above the pelvis, for example the pelvis, the belly and the chest. These vertices follow the body and need no cloth.
2. Free vertices are those with weight on the thigh or calf bones. A vertex that has more than 0.2 weight on a left leg bone and more than 0.2 on a right leg bone is a "bridge" vertex. It sits in the gap between the legs. These vertices are the strongest sign of a skirt, because a body surface does not stretch across that gap.
3. The freedom of a free vertex is its geodesic distance from the pinned region, divided by the largest such distance in the mesh. The result is in the range 0 to 1. The hem has freedom 1.
4. The tightness and the period are taken from the nearest danglymesh in the same game with the same role if one exists. If none exists, they take the median of all danglymeshes in the game. This is a data-driven default and not an authored value.
5. The skirt's vertices are collided against the leg capsules, and the maximum distance is derived as for danglymeshes.

The risk is false positives. A tight robe whose lower half follows the legs should not move as a free skirt. The bridge rule helps, because tight clothes have no bridge vertices. The prototype should print, for every body model in both games, the number of pinned, free and bridge vertices. A human can then check the list for models that look wrong.

I did not count how many models have danglymesh data or which have robe skirts. The issue says this is not known, and it is the first step of the plan below.

## Stability under Jedi Academy style movement

The movement in kjk copies Jedi Academy. Characters spin, flip, jump high, wall jump and hang upside down. Cloth will see accelerations and angular speeds that the games never had.

A solver is stable here when it passes five tests.

1. **Spin.** The character turns at 1,440 degrees per second. The cloth must stay within the maximum distance of the skinned pose and must not produce a NaN.
2. **Flip.** The character turns end over end while moving 10 m/s. The cloth must fall towards world gravity and not flip with the body unless the inertia scale says so.
3. **Hanging upside down.** The cloth falls away from the body, and the body capsules must still keep it out of the legs.
4. **A hard stop on a wall jump.** The root stops in one frame. The cloth must not stretch more than 5 percent of its tether lengths.
5. **Teleport.** A jump of more than 5 m resets the cloth.

The reasons XPBD with small steps and a maximum distance pass these tests are the following.

- XPBD constraint solves move positions to satisfy a limit, so a large acceleration cannot produce a large force. The solve never adds energy through a stiff spring. This is the property that makes it unconditionally stable for the constraint part.
- Small steps cut the distance a particle can move in one substep. At 240 substeps per second, a particle moving at 20 m/s moves 8 cm per substep, which is less than the thickness of the body capsules.
- The tether removes the long stretch that builds up when the root accelerates away from a long cape.
- The maximum distance is a hard clamp. It holds even if every other constraint fails.
- The solver should clamp each particle's speed per substep and reset any particle that is not a finite number to its skinned position.

The explicit spring, by contrast, is safe only because the games' numbers are small. Enlarging the limit or the tightness would make large frame times and fast spins visible as popping.

One remaining risk is that capsules move fast too. The capsule poses should be interpolated between the previous and the current frame for each substep, so that fast limbs do not pass through the cloth.

## Fit with TAA and DLSS motion vectors

TAA and DLSS need the position of every vertex in the previous frame. The current sway already writes both positions in each slot, so a CPU XPBD solver can use the same path with one rule. The previous position is the position at the end of the previous frame, not the position at the previous substep. A motion vector that covers one frame is what the temporal filter expects.

The solver must also keep the mesh that follows the skeleton consistent with the cloth. For skinned meshes, the cloth offset must be applied in the same space in both frames. The simplest rule is to solve in the character's root frame, write positions in the node's space, and let the shader transform them with the current and the previous bone matrices. The prototype must check that Bevy's skinned motion vectors then work with the offset, because I did not read the skinning shader.

When the sway is frozen or the held clock is zero, the solver must write the same position for now and was, as `dangly.rs` does today. A reset must also write the same value for both.

## Cost

These figures are estimates from counting operations. They are not measurements. The prototype must measure them with the benchmark method the project uses, which reports 1% low frames per second next to the mean.

| Solver | Operations per vertex per substep | Substeps | Operations per frame for 1,000 vertices | Likely time on one core |
|---|---|---|---|---|
| Current spring | 40 | 1 | 40,000 | under 0.02 ms |
| PBD, distance and bending | 130 | 4 iterations | 520,000 | 0.1 to 0.2 ms |
| XPBD, small steps, tethers, collision | 180 | 8 | 1,440,000 | 0.2 to 0.5 ms |
| Spring bones, 40 bones | 50 per bone | 8 | 16,000 | under 0.02 ms |

Three measures keep the cost down. A mesh that the camera did not see in the last frame is skipped, as today. A cloth far from the camera drops to 2 or 4 substeps. A character that has not moved for a second, and is not in wind, is put to sleep until it moves.

## Rust crates and reference code

- Rapier 0.36 is already used for the ragdolls (https://github.com/dimforge/rapier). It has no cloth solver, so it cannot be used for this directly.
- Parry (https://github.com/dimforge/parry, version 0.31.1 on crates.io) offers capsule and shape queries that can serve as the body colliders. A hand-written point against capsule push out is only about 25 operations and may be simpler than the crate.
- `bevy_silk` (https://github.com/ManevilleF/bevy_silk, version 0.10.0 published 2025-12-08) is a Bevy cloth plugin. I did not check whether it supports Bevy 0.19.1. It does not have tethers, inertia scaling or skinned limits, so it is only a reference.
- Avian (https://github.com/avianphysics/avian) is an ECS physics engine for Bevy with XPBD style solving for rigid bodies. It is not a cloth solver, but its solver code is a good reference for the substep structure.
- Matthias Müller's "Ten Minute Physics" cloth and bending code (https://matthias-research.github.io/pages/tenMinutePhysics/index.html) is the shortest correct reference.
- The PositionBasedDynamics library (https://github.com/InteractiveComputerGraphics/PositionBasedDynamics) is a C++ reference with distance, dihedral bending and long range constraints.
- NvCloth (https://github.com/NVIDIAGameWorks/NvCloth) is the reference for the skinned cloth limits, including the tethers and the sphere colliders.
- Rayon (https://github.com/rayon-rs/rayon) can spread several characters over several cores. Glam is already the engine's math library.

I recommend writing the solver in Rust inside a new small crate and not adding a dependency. The solver is about 500 lines, and none of the crates above matches the data model the engine needs.

## Recommended solver

The recommendation is XPBD with 8 substeps per frame, the character-frame inertia scale, the maximum distance and the long range tethers, with the original spring as the default setting.

In detail, it has these parts.

- Each vertex has an inverse mass equal to its freedom. This comes from the constraint value, or from the skin weights if there is no danglymesh.
- Each vertex has a damping equal to `period × 1.5`.
- Each edge has a distance constraint, and each pair of triangles that share an edge has a bending constraint.
- Each free vertex has a tether to its nearest pinned vertex along the mesh.
- Each free vertex has a maximum distance from its skinned position.
- Each free vertex collides with the capsules of the body parts it is skinned to, and with the two or three nearest capsules.
- The solver runs on the CPU in the view only, and writes positions to the existing slot buffer.
- The settings have three levels. "Original" keeps the current spring unchanged and is the default. "Enhanced" runs the XPBD solver with the original displacements as limits. "Stronger" runs it with limits derived from the mesh.

## First prototype plan

1. **Inventory.** With `kjk extract` and a short script, list for each model in both games how many danglymeshes it has, how many vertices each has, and the histogram of constraint values. Also list the skinned body meshes that have bridge vertices between the left and right leg bones. This answers the roadmap issue's open question about which models have the data. It also shows what constraint 255 looks like in the real data. Neither the issue nor the code settles this, because roots could be 0 or 255.
2. **Cloth data from a mesh.** In a new crate, for example `crates/cloth`, written in Rust and without Bevy, build edges, rest lengths, bending pairs, pinned set, geodesic tether lengths and freedom values from a `MeshIr`. Test it on synthetic meshes first, then on Juhani's cloth, which has the largest displacement of 12.5 cm.
3. **XPBD solver.** Write the substep loop with a fixed substep time of 1/240 s and an accumulator. A fixed substep keeps recorded frames identical between runs when the held clock is used. Add the inertia scale, the clamps and the NaN guard.
4. **Probes.** Add approved probes in the style of `dangly_spring` for the five stability tests in this note. Each probe checks that no vertex leaves its maximum distance, no value is not a number, and no vertex ends more than 1 cm inside a capsule.
5. **Hook to the slot buffer.** Replace `Dangly::step` with the solver for meshes in the "Enhanced" setting only, and keep the slot buffer write unchanged. Leave "Original" as the current code path.
6. **Look at it next to the original.** Show the cloth on our model next to the original sway in the viewer, in a spin and a jump. A step is done only when it can be seen this way.
7. **Benchmark.** Use the scenes of the benchmark method the project already uses. Report the mean and the 1% low frames per second with the setting off and on, with one cloth character and with a crowd. If the cost is visible, make the setting off by default and consider the GPU version.
8. **Skinned skirts.** Add the skin weight rules for robe skirts without danglymesh. Check the list from step 1 by eye before turning any model on.
9. **Hair and tails.** Add generated spring bones for the parts where the cloth solver is a poor fit.

## Open questions

- Which space should the cloth offset be added in for skinned meshes? The prototype must read the skinning shader to answer.
- What does a constraint of 255 mean in the real data? The step 1 histogram answers this.
- Do robe models have the leg bridge vertices described above? The same inventory answers this.
- Do Jedi Academy or Movie Duels have cloth worth copying? I did not check, and the roadmap issue lists this as an open question.

## Sources

- Position Based Dynamics, Müller et al. 2007: https://matthias-research.github.io/pages/publications/posBasedDyn.pdf
- XPBD, Macklin, Müller and Chentanez 2016: https://matthias-research.github.io/pages/publications/XPBD.pdf
- Small Steps in Physics Simulation, Macklin et al. 2019: https://mmacklin.com/smallsteps.pdf
- Matthias Müller's publication list, which includes the long range attachments paper: https://matthias-research.github.io/pages/publications/publications.html
- Ten Minute Physics: https://matthias-research.github.io/pages/tenMinutePhysics/index.html
- VRM spring bone specification: https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_springBone-1.0/README.md
- UniVRM: https://github.com/vrm-c/UniVRM
- Kawaii Physics: https://github.com/pafuhana1213/KawaiiPhysics
- NvCloth: https://github.com/NVIDIAGameWorks/NvCloth
- Unreal Engine clothing tool: https://dev.epicgames.com/documentation/en-us/unreal-engine/clothing-tool-in-unreal-engine
- bevy_silk: https://github.com/ManevilleF/bevy_silk
- Rapier: https://github.com/dimforge/rapier
- Parry: https://github.com/dimforge/parry
- Avian: https://github.com/avianphysics/avian
- PositionBasedDynamics library: https://github.com/InteractiveComputerGraphics/PositionBasedDynamics
- The roadmap issue: https://github.com/kjk-engine/roadmap/issues/5
