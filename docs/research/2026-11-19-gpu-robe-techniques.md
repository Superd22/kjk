# Cheap GPU techniques for robes and capes

This note was researched on 2026-11-19.
It belongs to the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts").
It answers one question.
Is there a cheap GPU approach for robe and cape motion that complements or replaces the CPU bone solver of the plan (`docs/research/2026-11-19-loose-parts-plan.md`)?

The older notes cover the CPU solvers and what shipped games do with bones and cloth.
They are `docs/research/2026-11-19-cloth-state-of-the-art.md` and `docs/research/2026-11-19-cloth-algorithms-sota.md`.
This note does not repeat them.
It looks only at work done on the GPU, in the vertex shader or in a compute pass, and at how games hand over between a near and a far version of cloth.

Every claim carries a link.
The words "read in full", "read in part", "abstract only" and "snippet only" say how much of a source was read.
"Snippet only" means that only a search result or a page summary was seen.
Anything marked "my estimate" is my own arithmetic and was not measured in kjk or taken from a source.
Nothing in this note was run or benchmarked.
Three research agents on Opus found and read most of the sources.
I read the kjk and Bevy source files in section "How it fits kjk and Bevy 0.19" myself.

## Summary

- There is one published method that matches the idea of a stateless vertex shader almost exactly. It is Velocity Skinning (Rohmer and others, Eurographics 2021). A vertex is skinned normally, and then it is bent by each bone's linear and angular velocity. The per-vertex weights come from the existing skin weights and the bone hierarchy, with no hand authoring. It keeps no state between frames, so it cannot get stuck. Its authors measured about twice the cost of plain skinning on a laptop Quadro P3200. Its code is MIT licensed GLSL. Source: https://arxiv.org/abs/2104.04934 (read in full).
- Shipped games use a simpler version of the same pattern for hair, shirts and plants. The CPU passes a few numbers, such as the player's speed or the wind, and a painted vertex colour scales the motion. Bungie, Naughty Dog and Crytek published this. In every case a person painted the weights, which kjk's owner does not allow. Velocity Skinning is the only source that removes the painting.
- I found no first-party source for pushing skinned vertices out of body capsules in a vertex shader. The VRM spring bone specification publishes the push-out maths, but for bones on the CPU. The idea is simple and its cost is small, but it is my own design and not a shipped technique.
- Per-vertex Verlet with a maximum distance from the skinned position is an established industry pattern. NVIDIA calls it a motion constraint, and Unreal Chaos Cloth and Unity Cloth call it max distance. NvCloth ships it as a DirectX 11 compute shader with one workgroup for each cloth and one dispatch for all cloths. Ubisoft's cloth for Assassin's Creed Unity also went from more than 50 dispatches to one. This is the proven GPU layout.
- The known weak point of per-vertex Verlet is fast motion. NVIDIA's own guide lists tunnelling, self-intersection, instability and stretching, and its advice is to pass on only part of the character's motion to the cloth.
- No geometric fix after skinning stops a skirt panel from stretching between two thigh bones. Dual quaternion skinning is likely worse than linear blend skinning for such a panel. Delta Mush and Direct Delta Mush smooth the creases at the hip and the knee, but they cannot make cloth hang, swing or keep its length. The Direct Delta Mush paper says so itself.
- Direct Delta Mush is cheap on a GPU. EA measured 1.13 to 1.45 nanoseconds per vertex on an RTX 3090. My estimate for a GTX 1650 is about 0.25 to 0.8 ms for 50 characters of 500 to 1,500 skirt vertices.
- Games switch cloth off with distance in two documented ways. Unreal blends between the simulated and the skinned pose with one weight (`cloth_blend_weight`). NVIDIA APEX scales every vertex's maximum distance toward zero, which pins the cloth to the skinned pose. No source gives a fade length. Unreal and APEX both snap the cloth back to the skinned pose when it is switched on again.
- My ranked recommendation is at the end. In short, try first a stateless vertex-shader layer that runs after the CPU bone solver. It pushes skirt vertices out of the body capsules with a per-vertex radius, and it adds a Velocity Skinning style drag for KOTOR 1 skirts that have no bones. Try second Direct Delta Mush variant 0, only if bulging remains after the generated skirt bones. Keep the per-vertex Verlet compute pass in reserve for the player's character. Ignore plain Delta Mush, dual quaternion skinning, learned deformers and multi-dispatch graph-coloured solvers.

## What kjk has today

These facts come from the repository, not from outside sources.

- The CPU robe solver moves robe and cape bones and costs about 2.4 microseconds per character (`docs/research/2026-11-19-loose-parts-plan.md`).
- The danglymesh sway moves vertices on the GPU. The CPU writes each vertex's current and previous position into the storage buffer `SwaySlots` (`crates/game/src/sway.wgsl`, `crates/game/src/dangly.wgsl` on the branch `loose-parts-l5`). The vertex shader `dangly.wgsl` reads `sway[slot]` and places the vertex with the mesh's world matrix. In the prepass it also places the previous position with the previous world matrix, so TAA and DLSS motion vectors stay correct. It is written for an unskinned mesh.
- Bevy 0.19.1 skins in the vertex shader. `skinning::skin_model` in `bevy_pbr-0.19.1/src/render/skinning.wgsl` blends up to four joint matrices from a storage buffer (`joint_matrices`), and `skin_prev_model` does the same with `prev_joint_matrices` for motion vectors. I searched the `bevy_pbr`, `bevy_render` and `bevy_mesh` 0.19.1 sources for a skin cache, a compute skinning pass or a mesh deformer and found none.
- kjk has its own compute pass, `crates/game/src/skin_once.rs` and `skin_once.wgsl`. It reads Bevy's vertex buffers and the joint matrices, places every vertex of a character's meshes in the world, and writes them to one buffer (`skinned`, four floats per vertex). Only the shadow tiles and the ray-traced acceleration structures read it. The main pass still skins with Bevy's shader. The pass is off unless `KJK_SKIN_ONCE=on` is set or shadows are at Ultra. The comment in the file says that on an RTX 4070 Ti SUPER it costs more than it saves.
- kjk already overrides vertex shaders. `crates/game/src/glow_shell.wgsl` reproduces Bevy's skinning (`skinning::skin_model` and `skin_normals`) inside a custom vertex function. `grass.rs` overrides both `vertex_shader` and `prepass_vertex_shader`. The main robe material, `odyssey.wgsl`, uses Bevy's default vertex shader.
- Bevy has a per-mesh number `MeshTag(pub u32)` (https://docs.rs/bevy_mesh/0.19.1, source `bevy_mesh-0.19.1/src/components.rs:156`), which reaches the shader as `mesh[instance_index].tag` (`bevy_pbr-0.19.1/src/render/mesh_types.wgsl:25`). kjk does not use it yet. It is a natural way to give each character a slot in a per-character storage buffer.
- The robe data is as the census note describes (`docs/research/2026-11-19-loose-parts-census.md`). KOTOR 2 robe skirts are the vertices weighted to 11 robe bones. KOTOR 1 Jedi robe skirts are part of the legs mesh and are skinned only to the thigh and shin bones.

## Technique 1, stateless vertex-shader secondary motion

### How it works

The CPU keeps a few smoothed numbers per character or per bone, such as linear velocity, acceleration and angular velocity.
The vertex shader skins the vertex as usual and then moves it by an amount that depends on those numbers and on a weight.
The weight is larger for vertices further down the garment, so the hem moves most and the waist does not move.
The shader stores nothing from the previous frame, so it has no state that can drift or get stuck.

### Who uses it

**Velocity Skinning (Rohmer, Tarini, Kalyanasundaram, Moshfeghifar, Cani and Zordan, Eurographics 2021).**
- Source: https://arxiv.org/abs/2104.04934 (read in full). Code: https://github.com/drohmer/velocity_skinning_replicability (MIT licence, GLSL; the GPU vertex shader was read in full).
- The vertex is skinned with linear blend skinning. Then, for each bone, the vertex is rotated about the axis of that bone's angular velocity, which passes through the bone's origin. The angle is minus a floppiness factor times the length of the angular velocity crossed with the vertex's offset from the bone. The angle therefore grows with the distance from the bone, so a shape curves and does not shear. For linear motion the vertex is moved by minus the floppiness factor times the bone's linear velocity.
- The bone velocities come from finite differences between the previous and the current frame. The paper says they can be smoothed by time averaging.
- The weight of bone i on a vertex is the sum of the vertex's ordinary skin weights over bone i and all of its descendants. The paper calls these upward-propagated weights. A skirt vertex skinned to a thigh therefore also feels the drag of the pelvis and the root. The paper also shows an automatic per-vertex scale from the local shape diameter, and per-bone switches that turn a whole branch off.
- Its authors say the method "requires no book-keeping of previous states, resulting in a single-pass method".

**Bungie, Destiny hair.**
- Source: Natalie Burke, "Hair in Destiny", SIGGRAPH 2014, https://advances.realtimerendering.com/destiny/siggraph2014/heads/siggraph_2014_destiny_hair_pipeline.pdf (read in part; the deck is mostly images and the method appears only as slide titles).
- Player hair is moved by "Vertex Animation as a Hair Sim", driven by "Player Linear Speed". The red vertex colour channel is the strength of the movement and the blue channel is its timing. Additive noise is added. The slides list jumping as future work.
- The talk "Animating With Math" (GDC 2016, https://www.gdcvault.com/play/1022965/Animating-With) was read as an abstract only.

**Naughty Dog, Uncharted 4.**
- Source: "Technical Art of Uncharted 4", https://advances.realtimerendering.com/other/2016/naughty_dog/NaughtyDog_TechArt_Final.pdf (read in part; the vertex shader section was read in full).
- For foliage, the shader rotates each part about a stored pivot, and a time offset that grows with the distance to the pivot makes outer parts lag. For wet hair, they "pushed the verts that are furthest from the pivot down in world-space", and say the stretching "wasn't noticeable". For a shirt, they scroll a 2D wave down the mesh, masked by a skinned direction, and blend a matching normal map because the normal update "was a very big part of the effect". Their closing advice is that "doing joint rotation looks much better" than layering sine functions.
- The slides give no millisecond numbers.

**Crytek, Crysis vegetation, also used on cloth and hair.**
- Source: Tiago Sousa, "Vegetation Procedural Animation and Shading in Crysis", GPU Gems 3, https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-16-vegetation-procedural-animation-and-shading-crysis (read in full through a fetch summary; I did not compare the code line by line).
- The CPU sums the wind sources into one vector per instance. The vertex shader bends vertices along the wind, scaled by height, and then renormalises each vertex to its original distance from the pivot. That keeps its length. Red, green and blue vertex colours hold edge stiffness, a per-leaf phase and overall stiffness. Cloth and hair skip the main bending and use only the detail bending.

**Rare, Sea of Thieves ropes.**
- Source: https://history.siggraph.org/wp-content/uploads/2022/09/2018-Talks-Ang_The-Technical-Art-of-Sea-of-Thieves.pdf (read in full; it is two pages).
- The CPU solves the catenary equation for each rope and passes the curve's parameters to the vertex shader, which bends a tube along it. This is the same split between a few numbers on the CPU and an analytic bend on the GPU.

**Sucker Punch, Ghost of Tsushima.**
- The PlayStation Blog says the cloth and ropes are simulated and use "the same wind inputs as the foliage and particles" (https://blog.playstation.com/?p=345372, read in full through a fetch summary). The GDC talk on wind (https://gdcvault.com/play/1027350/Blowing-from-the-West-Simulating) is behind a login and was read as an abstract only. This is not a vertex-shader method.

**What I could not find.**
- No first-party description of a cape or skirt driven by the vertex shader for Overwatch, Valorant, League of Legends, Halo, Fortnite, Zelda, Genshin Impact, Guilty Gear, Frostbite, Decima, Anvil crowds or Source 2.
- No Japanese conference material on swaying parts in the vertex shader.
- No first-party source for pushing skinned vertices out of capsules or spheres in a vertex shader.
- The closest sources are the VRM spring bone specification (https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_springBone-1.0/README.md, read in part), which gives the sphere and capsule push-out maths for bones, and a DreamWorks patent for stateless skin contact deformation by ray casting (https://patents.google.com/patent/US10803660B2/en, read in part, and patented).

### How the weights are chosen

- Velocity Skinning derives them from the skin weights and the bone hierarchy, as described above. This is the only source that needs no painting.
- Bungie, Naughty Dog and Crytek paint them as vertex colours. Unity's own blog says "in most cases a vertex colour mask would be easier and more flexible" than a UV or height mask (https://unity.com/blog/engine-platform/creating-animated-materials-with-shader-graph, read in full through a fetch summary).
- For a skirt or a cape, the downward weight can be computed. Two choices are the height of the vertex below the hip bone divided by the length of the garment, and the distance along the surface from the nearest vertex that is rigidly skinned. Naughty Dog's distance to the pivot and Crytek's height are the same idea. The sources describe these values as authored, but nothing stops a program from computing them at load time. This is my judgement.
- A phase for flutter can come from a hash of the vertex position. Crytek and Bungie paint it, and I found no source that hashes it.

### Cost

- Velocity Skinning, Table 1, on an NVIDIA Quadro P3200 laptop GPU: for 150,000 vertices, plain linear blend skinning took 3.9 ms and velocity skinning took 8.0 ms with one moving bone and 8.7 ms with twelve. For 750,000 vertices the times were 18.7 ms, 34.8 ms and 36.7 ms. The method costs about twice plain skinning in the authors' shader.
- The extra cost comes from dense weights. The paper says the method cannot use the usual limit of 2 to 8 bone links per vertex. The shader loops over every joint for every vertex.
- No shipped game publishes the cost of its vertex-shader secondary motion.
- My estimate for kjk is far smaller than the paper's. kjk would not loop over every joint. It would use the pelvis, the two thighs and perhaps the torso, with a handful of smoothed numbers per character. That is a few dozen arithmetic operations per skirt vertex. For 50 characters of 1,500 skirt vertices (75,000 vertices), I estimate 0.1 to 0.2 ms on a GTX 1650. If the paper's own shader were used unchanged, scaling its 4.1 ms of extra time for 150,000 vertices gives about 2 ms for 75,000 vertices, which is too much.
- The extra work lands in three vertex passes, which are the main pass, the prepass and the shadow pass, unless the shadow pass is left unchanged.

### Fast moves and getting stuck

- The method cannot get stuck, because it has no state. It recovers on the frame the motion stops.
- It over-bends at high speed. The paper's fix is a maximum bending angle of 45 degrees (its Figure 9). The shader in the repository does not apply that cap, so kjk must add one.
- It has no collision. Highly floppy parts self-intersect, and the paper says so for a cow's ears.
- It has no overshoot and no swing. It is a drag toward the direction opposite to the motion, and it stops as soon as the motion stops. The paper says that adding acceleration terms would narrow the gap to a simulation, but the authors did not do it.
- Engines that keep state document the same fixes for fast moves. Unreal Chaos Cloth passes on only part of the reference bone's velocity, with defaults of 0.75 for both the linear and the angular scale (https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/ChaosClothingInteractor, read in part). The VRM specification evaluates inertia in a "center" space, so walking does not shake the bones. The Unreal Spring Controller resets the bone when the error grows too large, for example on a teleport (https://dev.epicgames.com/documentation/en-us/unreal-engine/animation-blueprint-spring-controller-in-unreal-engine, read in full).
- A shader offset after skinning must also be computed for the previous frame, or the motion vectors are wrong. Epic's issue tracker has a bug of this kind for world position offset (https://issues.unrealengine.com/issue/UE-229684, snippet only). kjk's dangly shader already handles this by storing a previous position. A stateless shader would instead keep the previous frame's smoothed numbers and run the same function twice.

### Per-vertex push-out from body capsules in the shader

- This is my design and not a shipped technique. For each skirt vertex, the shader tests the skinned position against the character's 14 ragdoll capsules and moves the vertex out along the direction from the capsule's axis to the vertex.
- The plan already says each point needs its own radius, because 15% to 30% of garment vertices start inside a capsule in the rest pose. That radius is the smaller of the capsule's radius and the vertex's rest distance from the capsule's axis, minus 0.5 cm. It can be computed at load time and stored per vertex.
- Cost, my estimate. For 75,000 vertices and 14 capsules, that is about one million capsule tests, with a few dozen operations each. On a GTX 1650 this is well below 0.1 ms for each vertex pass. The 14 capsule poses per character must be uploaded each frame, which is 14 times about 32 bytes per character, or about 22 KB for 50 characters.
- It can never get stuck, because it works from the skinned position every frame. It does not know which side of the body a vertex was on in the previous frame, so a robe that the skinning puts behind a leg is pushed to the nearest surface, which can be the wrong one. A vertex shader cannot remember. This matters most for vertices that are skinned right through a thigh in a split. For these, the nearest surface is usually still the correct one, because the skinned position is where the animation wants the cloth to be.
- It cannot fix a robe that the CPU solver has left stuck behind the legs, because the shader sees only the solver's bones. It would catch the skin between two bones that cuts through a thigh, which the plan lists as risk 3.

### Data from KOTOR's models

- Everything needed can be computed from the data the games give: skin weights, bone heights, the skeleton for capsules, and mesh topology for the connected components. Nothing needs hand authoring.
- For KOTOR 1 skirts, the skin weights alone do not find the skirt cleanly (`docs/research/2026-11-19-loose-parts-census.md`, section "Skirts without a danglymesh"). A height-based weight below the hip is a safer rule, and the census note's finder of which vertices are skirt must be reused.

### Code

- GLSL: https://github.com/drohmer/velocity_skinning_replicability (MIT).
- I found no Rust or WGSL code for any of these methods.

## Technique 2, per-vertex Verlet in a compute pass

### How it works

Each skirt vertex is a free point with a previous and a current position.
Each frame the shader integrates it with gravity and damping.
Then it pulls the point back into a sphere around its skinned position (the maximum distance), pushes it out of the capsules, and moves it toward the average of its neighbours for one or two passes.
The skinned position is where the animation wants the vertex, so the point can never be far from it.

### Who uses it

**NVIDIA NvCloth, PhysX cloth and APEX Clothing.**
- Sources: the NvCloth repository https://github.com/NVIDIAGameWorks/NvCloth, its solver documentation (`NvCloth/docs/documentation/Solver/Index.html`, read in full) and its user guide (read in part), and the DirectX 11 kernel https://github.com/NVIDIAGameWorks/NvCloth/blob/master/NvCloth/src/dx/DxSolverKernel.hlsl (read in part). The licence is BSD-3.
- The solver is Verlet. Per iteration, it integrates, applies wind, applies the motion constraints, then tethers, edge constraints, separation constraints, collision and self collision.
- A motion constraint is a sphere for each particle. Its centre is the skinned position, and its radius is the maximum distance. A particle outside the sphere is pulled back. A radius of 0 pins the particle. The user guide says that motion constraints "can also be used to make particles stay within a set distance from the skinned mesh of a character", and that shrinking the radius gradually "can be used to blend between skinned and physical meshes".
- A separation constraint is the opposite. It keeps a particle outside a sphere, so it works as a cheap backstop.
- A tether limits the distance to the nearest fixed particle.
- Local-space simulation has separate linear, angular and centrifugal inertia multipliers between 0 and 1, which set how much of the character's motion reaches the cloth. There are `teleport()` and `clearInertia()` calls for cuts.
- GDC 2017 slides confirm that NvCloth is the same solver as APEX and PhysX cloth and that it has a DirectX 11 compute path: https://developer.download.nvidia.com/assets/gameworks/downloads/regular/GDC17/NVIDIA_GameWorks_Animation_Technologies_in_Unreal_Engine_4_GDC17_FINAL.pdf (read in part; no numbers).

**Unreal Engine Chaos Cloth and Unity Cloth.**
- Unreal defines Max Distance as "the maximum distance any point on the cloth can move from its animated position", and a Backstop Distance and Backstop Radius that stop painted points from entering a region. An Anim Drive Multiplier drives springs toward the animated position. Source: https://dev.epicgames.com/documentation/en-us/unreal-engine/clothing-tool-in-unreal-engine (read in part). I found no Epic statement that Chaos Cloth runs on the GPU, and forum posts report a few milliseconds of CPU per clothed character (anecdotal, snippet only).
- Unity Cloth, which is PhysX cloth underneath, has Max Distance and Surface Penetration constraints, tethers, and World Velocity Scale and World Acceleration Scale. Its collision is limited to spheres, capsules and conic capsules "to help boost performance". Self collision "can take a significant amount of the overall simulation time". Source: https://docs.unity.com/en-us/engine/current/manual/physics-section/physics-overview/class-cloth (read in full).

**Long Range Attachments (Kim, Chentanez and Muller, SCA 2012).**
- Source: https://matthias-research.github.io/pages/publications/sca2012cloth.pdf (read in part).
- Each free particle gets a one-sided distance constraint to up to four particles that follow the animation. The rest distance is the distance along the cloth surface, because the straight distance is too short when the bind pose is bent. The authors also use a motion constraint that keeps each particle within a small radius of the skinned position.

**AMD TressFX 4.1 (hair).**
- Source: https://github.com/GPUOpen-Effects/TressFX, `src/Shaders/TressFXSimulation.hlsl` (read in part).
- The integration kernel skins the rest position, integrates with Verlet, and then moves each vertex a fraction of the way back to its skinned position. If a reset flag is set, it sets the current and previous positions to the skinned positions. This is a teleport reset. A separate kernel called velocity shock propagation applies a fraction of the root's motion to every vertex of the strand in one step, so that a fast head turn does not lag.

**Ubisoft Motion Cloth on compute shaders (Assassin's Creed Unity, Far Cry 4).**
- Source: Alexis Vaisse, https://www.slideshare.net/slideshow/efficient-usage-of-compute-shaders-on-xbox-one-and-ps4/269666732 (read in part). The GDC sessions are at https://gdcvault.com/play/1020857/contactUs and https://www.gdcvault.com/play/1022350/contactUs (snippets only).
- The compute version was more than 10 times faster. The first version used a dispatch per step and was limited by the CPU. The final version used one dispatch instead of more than 50, and simulated up to 32 cloths in it. A thread group handles 64 vertices. Keeping the vertices in local shared memory was 1.9 times faster than video memory. Packing positions to 64 bits and normals to 32 bits made it 2.3 times faster. Within a group, the constraints are split into four sets separated by barriers.
- The slides give counts of dancers at a 5 ms CPU budget, and say that "the brute-force approach is almost always the fastest one". They do not mention a maximum distance.

**Other GPU cloth that I could only read about.**
- EA Frostbite "Warp" for EA Sports FC 24, a compute shader cloth for jerseys: https://schedule.gdconf.com/session/warp-high-fidelity-cloth-simulation-in-ea-sport-fc-24/898847 (snippet only, no numbers).
- The Ghost of Yotei deep dive describes a "high-performance GPU compute cloth system" with several layers, and gives no numbers: https://blog.playstation.com/?p=411201 (read in full).
- EA's Frostbite self-collision talk gives CPU numbers: 0.4 ms for a full character with self-colliding cloth per 30 Hz step on one thread, 0.2 ms for a layered skirt, and 0.1 ms for two flaps against a leg: https://media.contentapi.ea.com/content/dam/eacom/frostbite/files/gdc2018-chrislewin-clothselfcollisionwithpredictivecontacts.pdf (read in full).

### Cost

- Long Range Attachments paper: a dress of about 1,500 vertices takes 0.91 ms per frame on one core of an Intel i7-930. On an NVIDIA GTX 470, 14 characters with about 20,000 vertices in total take 0.8 ms in a single GPU pass, one streaming multiprocessor for each cloth. The attachment constraints cost less than 2% of the simulation. These have full stretch and bend constraints, so they are heavier than the version in this section.
- Flex (Macklin and others, SIGGRAPH 2014) on a GTX 680: 10,000 cloth particles with 6 iterations per step took 5.6 ms, at 2 or 3 steps per frame. Source: https://matthias-research.github.io/pages/publications/flex.pdf (read in part). It is a general solver with collision and is not comparable.
- Vivace (Fratarcangeli and others, 2016) on a GTX 970: self-colliding cloth with 108,000 triangles took 5 ms per frame. Source: https://publications.lib.chalmers.se/records/fulltext/245496/local_245496.pdf (read in part).
- I found no number for a GTX 1050 Ti or a GTX 1650, and no measured cost of one extra dispatch in a wgpu compute pass. A WebGPU preprint reported 24 to 36 microseconds of overhead per dispatch on Vulkan and 32 to 71 on Metal, but it is marked withdrawn, it measures one operation per submit, and it overstates the cost inside an open pass (https://arxiv.org/abs/2604.02344, abstract only). The kjk Vast.ai bench skill can measure the real value.
- My estimate for a GTX 1650 with 50 characters of 500 to 1,500 skirt vertices, so 25,000 to 75,000 vertices, using the layout of NvCloth and Ubisoft (one dispatch, one workgroup for each character, all iterations inside the shader): 0.3 to 1.0 ms of GPU time. This takes the 0.8 ms for 20,000 vertices with full constraints on a GTX 470, scales for a GTX 1650's greater arithmetic throughput, and then scales for 3.7 times the vertices with lighter constraints. A skirt of 1,500 vertices does not fit in one workgroup of 1,024 threads, so each thread would handle two vertices, which is how NvCloth runs larger cloths.
- The extra buffers are small. A vertex needs a current and a previous position, which is 24 bytes, as the sway slots do today. For 75,000 vertices that is 1.8 MB.

### Fast moves, artefacts and getting stuck

- NvCloth's user guide says: "Fast movements can cause problems like tunneling through collision shapes, self-intersection, instabilities and stretchy cloth. Use smaller inertia multipliers if these issues are noticeable."
- The Long Range Attachments paper says: "We typically use 3-5 iterations per frame. Using less iteration results in collision detection failure for typical (fast) game character motion."
- Plain position-based dynamics with few iterations stretches. The papers fix it with tethers or with a single-pass chain solver (Muller, Kim and Chentanez, "Fast Simulation of Inextensible Hair and Fur", https://matthias-research.github.io/pages/publications/FTLHairFur.pdf, read in part). Macklin's "Small Steps in Physics Simulation" (https://matthias-research.github.io/pages/publications/smallsteps.pdf, abstract only) says many small steps with one iteration each beat one large step with many iterations.
- Jacobi iteration, which a GPU does without graph colouring, can oscillate for redundant constraints, and Flex fixes it by averaging each particle's corrections. It also gives jitter in contacts. A Laplacian smoothing pass is a Jacobi pass and needs no colouring.
- Graph-coloured Gauss-Seidel costs one dispatch per colour, and the Vivace paper's colourings used 10 to 22 colours. NvCloth and Ubisoft avoid this by keeping one cloth in one workgroup, with barriers between the colour sets inside the shader.
- A particle can end on the wrong side of a capsule. Chaos Cloth answers it with a backstop, and NvCloth with separation spheres. I found no primary source that measures how often cloth gets trapped.
- Because the point is held within the maximum distance of its skinned position, it cannot stay far from the animation. A robe stuck behind the legs by the CPU solver would be pulled back to its skinned place only to the extent that the maximum distance says. For skirt vertices that are bound to bones the solver has bent away, the skinned position is already the solver's pose, so this compute pass does not repair the solver's mistake. It only adds detail around it.
- A teleport must reset the previous positions to the skinned positions, as TressFX does.

### Data from KOTOR's models

- The maximum distance is one number per vertex. NvCloth, Unity and Chaos all take exactly this. kjk can compute it from the distance of the vertex below the hip, or from the distance along the surface to the nearest rigidly skinned vertex.
- Tether anchors, tether lengths and the neighbour lists come from the mesh topology. The Long Range Attachments paper stores them in 16 bytes per particle.
- None of the primary sources needs hand-painted data beyond these scalar masks.
- KOTOR meshes are often made of several pieces that share positions but not vertices. The neighbour lists must connect close vertices of the same bone, as the Direct Delta Mush paper does for its multi-piece meshes.

### Code

- NvCloth, with a 1,861 line DirectX 11 kernel: https://github.com/NVIDIAGameWorks/NvCloth (BSD-3).
- TressFX 4.1 in HLSL: https://github.com/GPUOpen-Effects/TressFX/blob/master/src/Shaders/TressFXSimulation.hlsl.
- I found no Rust or WGSL code that combines skinned maximum distance with capsule push-out. Leads that I did not read are a Rust compute Verlet cloth in the Nightshade engine (https://docs.rs/crate/nightshade/0.32.0/source/src/ecs/cloth.rs, snippet only), a three.js WebGPU gist (https://gist.github.com/bandinopla/0ff5e437fb1a6a3dce5d355b93e22b68, snippet only), and a Metal compute spring bone implementation (https://awesome.ecosyste.ms/projects/github.com%2Farkavo-org%2Fvrmmetalkit, snippet only).

## Technique 3, post-skinning corrections

### Delta Mush

- How it works. The mesh is first skinned crudely. Then it is smoothed by Laplacian smoothing, which gives the "mush". At the rest pose, each vertex stores its offset from the smoothed rest mesh in a local frame. Each frame, the method skins, smooths with the same number of iterations, and adds the stored offsets back. This description comes from the introduction of the Direct Delta Mush paper (https://media.contentapi.ea.com/content/dam/ea/seed/presentations/direct-delta-mush-and-variants.pdf, read in full). The original paper (Mancewicz and others, DigiPro 2014, https://doi.org/10.1145/2633374.2633376) could not be read, because the page returned an HTML wall.
- Who uses it. Rhythm and Hues has used it in all its character rigs since 2010 (https://history.siggraph.org/?p=138877, snippet only), and Maya has a Delta Mush deformer (https://help.autodesk.com/cloudhelp/2016/ENU/Maya/files/GUID-139B703C-28E7-4787-8FD4-C2991BD6C990.htm, snippet only). The Direct Delta Mush paper says that games do not run it live: "In the games industry, a common practice is to 'bake' the DM to a traditional skinning model".
- Cost. Each iteration of smoothing needs the results of the previous one, so a GPU needs a barrier between iterations. The EA SEED slides say the implicit solve is hard to parallelise on a GPU (https://media.contentapi.ea.com/content/dam/ea/seed/presentations/le2019-siggraph2019-direct-delta-mush-skinning-and-variants.pdf, read in full). A Unity project found it with 10 iterations "costlier than any" Direct Delta Mush variant, but the timings were in images (https://github.com/PacosLelouch/MeshDeformUnity, read in part). My estimate for a GTX 1650 and 25,000 to 75,000 vertices with 10 iterations of about 6 neighbours each is 1 to 3 ms. That is too much.

### Direct Delta Mush

- Source: Le and Lewis, SIGGRAPH 2019, https://media.contentapi.ea.com/content/dam/ea/seed/presentations/direct-delta-mush-and-variants.pdf (read in full), with slides at the second link above.
- How it works. The smoothing is moved to a precomputation on the rest pose. It gives each vertex a symmetric 4 by 4 matrix for each bone that influences it. Each frame, a vertex sums its bones' transforms times those matrices, and takes a 3 by 3 decomposition to get its best-fit rotation. Variant 0 uses a singular value decomposition and is the robust one. Variant 1 uses an inverse and inverts the deformation in extreme poses (its Figure 5). Variant 5 is linear blend skinning. Three global numbers control the look, and the paper says the results "required no manual weight tuning".
- Data. Each vertex stores 10 floats for each bone, because the matrices are symmetric, and the paper found "factors of between 2-10x more weights" than linear blend skinning. The precomputation took 2.2 s for a 20,604 vertex model and 9.7 s for a 55,886 vertex model on a CPU, using a sparse solver. I estimate well under a second for a 1,500 vertex legs mesh. All of it is computed from the mesh and the skin weights at load time, with no authoring. Meshes of separate pieces need extra edges between close vertices of the same bone.
- Cost, measured. On an RTX 3090, DirectX 12 HLSL, a batch of 300 characters, the 2021 follow-up measured 1.13 to 1.45 nanoseconds per vertex for the full method and 0.48 to 0.81 for sparse linear blend skinning (https://media.contentapi.ea.com/content/dam/ea/seed/presentations/ddm-compression-with-continuous-examples.pdf, read in part). Storage was 7.1 to 21.7 MB for each of its large characters.
- Cost, my estimate. Skinning is limited by memory bandwidth. An RTX 3090 has about 936 GB/s and a GDDR5 GTX 1650 about 128 GB/s, so I scaled by about 7 times, giving about 10 nanoseconds per vertex. For 25,000 vertices this is 0.25 ms. For 75,000 vertices it is 0.8 ms. Small dispatches lose efficiency, so plan for up to 1 ms. Only the legs and skirt meshes need it, so the true cost is lower than for whole characters.
- Does it fix bulging between skirt bones? It softens the hard seams and the pinching where thigh weights meet shin weights, and the paper shows it fixing the cleft at the hip (its Figure 10). It does not stop a skirt panel from stretching between the legs when they split, because it computes each vertex from the pose alone. The paper's limitations section says: "the inability to handle self-collision, the lack of secondary effects, and insufficient volume preservation".
- Fast moves. It is stateless, so it cannot lag, overshoot or pop.
- Code. Unity HLSL compute shaders under BSD-2: https://github.com/PacosLelouch/MeshDeformUnity (file list and README read in part) and a fork at https://github.com/V-Sekai/unity-mesh-deform-direct-delta-mush. I found no Rust or WGSL version. An extension that handles bones with scale is in https://arxiv.org/pdf/2101.02798 (read in part), and KOTOR's bones are rigid.

### Dual quaternion skinning and centres of rotation

- Dual quaternion skinning (Kavan and others, 2008, https://users.cs.utah.edu/~ladislav/kavan08geometric/kavan08geometric.pdf, read in part) costs about the same as linear blend skinning. The paper itself uses a two-bone cloth to show that it can tear neighbouring vertices apart when their rotation centres differ (its Figures 7 and 15). The two thighs of a skirt are sibling bones, so dual quaternion skinning is likely worse than linear blend skinning for the panel between them.
- Optimised centres of rotation (Le and Hodgins, SIGGRAPH 2016, https://la.disneyresearch.com/wp-content/uploads/Real-time-Skeletal-Skinning-with-Optimized-Centers-of-Rotation-Paper.pdf, read in part) gives each vertex one precomputed point. It then blends quaternions for the rotation and uses linear blend skinning of that point for the translation. On a flat cloth skinned to joints that are not in one chain, which is the skirt's case (its Figure 12), it behaves like linear blend skinning. It helps twist at the hip and knee, not stretching. It has no official code.

### Helper bones, learned deformers and proxies

- Helper bones with learned dynamics (Mukai and Kuriyama, SIGGRAPH 2016, https://history.siggraph.org/?p=102759, snippet only) have no public code.
- Unreal's ML Deformer (https://dev.epicgames.com/documentation/en-us/unreal-engine/ml-deformer-framework-in-unreal-engine, snippet only) needs offline simulation data for training.
- EA's neural cloth for Madden NFL 21 predicts about 32 shape weights from the bone rotations, and costs per player 20 microseconds of CPU for the network, 451 microseconds of CPU for the dispatch, and 115 microseconds of GPU time. It needed 6 to 48 hours of training data generation per asset (https://media.contentapi.ea.com/content/dam/ea/seed/presentations/seed-siggraph21-swish-cloth-simulation-madden-nfl-slides.pdf, read in full). It breaks the rule that effects come from the games' data.
- A simulated proxy mesh that drives the full mesh by linear blend skinning is the standard industry practice (LightSpeed Studios, SIGGRAPH 2024, https://kuiwuchn.github.io/proxycloth/proxycloth.pdf, read in part). Its skirt example uses a 128 vertex proxy. This is the same idea as the plan's generated skirt bones.
- I found no GDC or SIGGRAPH talk specifically about game skirt rigs. The community advice is to weight a skirt to both the leg bones and its own skirt bones (https://polycount.com/discussion/134601/animation-for-games-how-to-rig-a-skirt, snippet only, and not a primary source).

## How games hand over between near and far

- **Unreal.** `cloth_blend_weight` is "the weight to blend between simulated results and key-framed positions". A value of 0 is the skinned pose and 1 is the simulation. `disable_cloth_simulation` plays the original animation. `resume_clothing_simulation` resumes it and teleports the cloth to the skinned pose on the next tick. Source: https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/SkeletalMeshComponent?application_version=5.4 (read in part). I found no Epic page that says the engine fades the weight over a number of frames. If kjk wants a fade, it must ramp the weight itself.
- **Unreal clothing LODs.** Each cloth LOD can store data for a smooth transition from the skinned mesh to the clothed mesh (`bSmoothTransition`), and the engine builds data that skins the incoming mesh to the outgoing one (https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/ClothingSystemRuntimeCommon/FClothLODDataCommon and the page for `BuildLodTransitionData`, both snippet only). The Clothing Tool lets a generated LOD ignore a cloth section for lower quality LODs (https://dev.epicgames.com/documentation/unreal-engine/clothing-tool-in-unreal-engine---properties-reference, read in part).
- **Unreal Animation Budget Allocator.** It throttles skeletal mesh updates to a budget of 1.0 ms of game thread time by default, with a maximum tick rate of 10 and a maximum of 16 interpolated components. It does not mention cloth (https://dev.epicgames.com/documentation/en-us/unreal-engine/animation-budget-allocator-in-unreal-engine, read in part).
- **NVIDIA APEX Clothing.** Each cloth actor gets a benefit score from its screen size, a weight and a bias. The actors with a lower score are switched off first, and the documentation suggests a lower weight for hair than for a cape. A scale between 0 and 1 multiplies every vertex's maximum distance. Reducing it toward 0 pins the cloth to its skinned position. The documentation warns against setting the maximum distance to 0 for invisible characters, because switching the cloth back on is slow, and suggests halving it. A non-continuous frame resets the cloth to its skinned state, and doing it for several frames in a row "will look very awkward". Source: https://archive.docs.nvidia.com/gameworks/content/gameworkslibrary/physx/apexsdk/APEX_Clothing/Clothing_PG_Runtime_Interfaces.html (read in full). Using the scale as a fade is my inference, not a statement from the page.
- **Others.** An Intel article says that with APEX cloth in Unreal 4, cloth was tied to LOD 0 and stopped when a mesh left it (snippet only). I found no primary source with cloth LOD distances, vertex counts per LOD or fade lengths for Naughty Dog, Ghost of Tsushima, Horizon, Cyberpunk 2077, The Witcher 3, Total War, Fortnite, Anvil crowds or Hitman crowds. The GDC Vault talks are behind logins.

## How it fits kjk and Bevy 0.19

I read these files myself and did not build anything.

### Running a custom vertex shader after skinning

- Bevy 0.19.1 has no compute skinning, so skinning happens in each vertex shader. A material that overrides `vertex_shader` can call `skinning::skin_model` (as `glow_shell.wgsl` does), then change the skinned world position before it writes `VertexOutput`. The material needs `prepass_vertex_shader` too, so the depth prepass matches. For the previous position, the prepass calls `skin_prev_model` and applies the same displacement function to the previous frame's smoothed numbers.
- The main robe material, `odyssey.wgsl`, uses Bevy's default vertex shader today. The robe meshes would need a vertex function that does the same as Bevy's plus the displacement. The shadow pass (`shadow_tiles.wgsl`) skins on its own and would need the same function, or it would cast a shadow of the undisplaced skirt. Which of the three passes must match is a choice. A mismatch in the shadow is small. A mismatch in the prepass would give a depth and motion vector error, so the prepass must match.
- Per-character data can go in a storage buffer indexed by `MeshTag`. The tag arrives as `mesh[instance_index].tag`. This also keeps instancing, because the tag is part of each instance's data.
- Per-vertex data, which are the downward weight and the capsule radius, can go in a custom vertex attribute as `dangly.rs` does for the slot, or in a storage buffer indexed by `vertex_index`. The attribute costs no extra bind group.
- The sway buffer cannot be reused as is. It stores final positions for unskinned meshes. A vertex-shader displacement of a skinned mesh needs no per-vertex store. It needs only the character's numbers and the per-vertex weights.

### Running a compute pass after skinning

- `skin_once` already places skinned vertices in the world in a compute pass, so a correction pass (Direct Delta Mush, or per-vertex Verlet) has a place to run. It would read the joint matrices and write a corrected position into `skinned`.
- The main pass does not read `skinned`. To use the compute result for the main pass, a robe material would need a vertex shader that reads the buffer, as `dangly.wgsl` reads `sway`, and ignores its own skin. That works, and it makes the compute pass the owner of the final position, so the shadow tiles and the acceleration structures see the same result.
- The comment in `skin_once.rs` says the pass costs more than it saves on an RTX 4070 Ti SUPER. It would stay off by default, and a robe-only job list would be small. The jobs are one mesh each, so only the robed characters' skirt meshes would be in it.
- Per-vertex Verlet needs its own state buffer with the current and previous position for each vertex, and a second pass over it. It can write its result into the same buffer layout that `sway.wgsl` reads, so the prepass motion vectors keep working with the existing `now` and `was` fields. This part of the existing path fits well.

### The risk to the existing design

- The plan says the CPU solver runs before `TransformSystems::Propagate` and moves bones, so skinning and shadows see it in the same frame. A vertex-shader layer works on top of that and does not change it. A compute Verlet on the vertices would conflict with the bone solver if both bent the same skirt, because the compute pass would start from the solver's pose and then add its own motion. The two must be told apart by size, as in the recommendation.

## Ranked recommendation

1. **Try first, a stateless vertex-shader layer after the bone solver.** It has two parts.
   - A per-vertex capsule push-out with the per-vertex radius that the plan already defines. This targets the risk that skin between two bones cuts through a thigh. It costs, by my estimate, well below 0.1 ms on a GTX 1650 for 50 characters, and it cannot get stuck. It does not repair a robe that the CPU solver has left behind the legs.
   - A Velocity Skinning style drag. For KOTOR 1 skirts, which have no bones, it gives a cheap sway from the pelvis and thigh velocity, weighted by height below the hip. A maximum bending angle of 45 degrees and a partial pass of the body's velocity, as in Unreal's 0.75 scale, keep it stable in spins. For KOTOR 2 robes and capes it adds only fine flutter on top of the solver's bones, with a small weight. It costs, by my estimate, 0.1 to 0.2 ms in total.
   - This needs a custom vertex function in the robe material, the prepass and (optionally) the shadow pass, a `MeshTag` for the character slot, and the previous frame's numbers for motion vectors.
2. **Try second, Direct Delta Mush variant 0, only if bulging remains.** First add the generated skirt bones, because plan step C may remove the bulge. If a bulge remains at the hip and knee, run Direct Delta Mush on the legs and skirt meshes of the near characters in the `skin_once` compute pass. My estimate is 0.25 to 1 ms for 50 characters on a GTX 1650. Its weights and matrices are computed at load time from the skin weights and the topology. It does not add swing and it does not fix stretching. Its largest cost is the storage of 10 floats for each bone link, which must be capped for the skirt meshes.
3. **Keep in reserve, a per-vertex Verlet compute pass for the player's character only.** This is the plan's option D on the GPU. Use the NvCloth and Ubisoft layout, which is one dispatch, one workgroup for each character, all iterations inside the shader, a maximum distance per vertex, a tether, capsule push-out and one Laplacian pass. My estimate is 0.3 to 1.0 ms for 25,000 to 75,000 vertices, which for one character is only a few hundredths of a millisecond. For a single character a CPU vertex cloth could be just as cheap, and the plan must compare the two. The GPU pass gains most when more than a few characters use it.
4. **Ignore.** Plain Delta Mush, dual quaternion skinning, optimised centres of rotation, and the learned deformers (Madden's neural cloth, Unreal's ML Deformer, learned helper bones). Also ignore graph-coloured Gauss-Seidel with one dispatch per colour, and TressFX's per-strand kernels with 5 plus N dispatches, because each costs more dispatches than one character's skirt is worth.

### How it combines with the CPU bone solver

- **Near the camera.** The CPU bone solver gives the big shape, which means the swing, the lag and the fall of the hem. The vertex layer adds detail that bones cannot give, which means the push-out from the legs and a little flutter. Direct Delta Mush, if it is used, smooths the creases.
- **Far from the camera.** Switch the CPU solver off. The stateless drag in the vertex shader keeps a robe from looking frozen, and it costs no CPU. Plain skinning and the push-out remain. The drag does not need the bones at all, because it uses only the body's velocity.
- **Blending.** Ramp a weight from 0 to 1 over a length of our choice, like Unreal's `cloth_blend_weight` or APEX's maximum distance scale, and keep the solver's points reset to the animated pose while the weight is 0. Unreal and APEX both reset the cloth when it is switched on again. Pick the characters by an APEX style score from screen size and importance. No source gives a fade length, so it must be found by watching the stress scene.
- **Big shape against detail.** The bone solver owns every motion longer than about a tenth of the garment. The shader layer must stay small, because the plan's goal is stable acrobatics, and a large stateless offset on top of a moving solver can double-count the motion. A way to avoid this is to feed the shader layer the body's velocity minus the part that the solver already follows. This is a design choice that the stress scene must settle.

## What no source could tell

- Nobody published the cost of one more dispatch inside a wgpu compute pass, and nobody published numbers on a GTX 1050 Ti or a GTX 1650 for these techniques. Every GTX 1650 figure in this note is my estimate. The Vast.ai bench (`kjk-vast-bench`) can measure the real values.
- No source says how often cloth is trapped behind a body when it is held by a maximum distance and capsules.
- No source gives a cloth LOD distance, vertex count per LOD or fade length.
- Several talks are behind logins, and the original Delta Mush paper could not be read.
- I did not compile or run any shader. The statement that the three vertex passes can all call the same displacement function is based on reading `glow_shell.wgsl`, `dangly.wgsl`, `grass.rs` and Bevy's `prepass.wgsl`, and it has not been tried.

## Sources

- Velocity Skinning: https://arxiv.org/abs/2104.04934, code https://github.com/drohmer/velocity_skinning_replicability
- Destiny hair: https://advances.realtimerendering.com/destiny/siggraph2014/heads/siggraph_2014_destiny_hair_pipeline.pdf
- Uncharted 4: https://advances.realtimerendering.com/other/2016/naughty_dog/NaughtyDog_TechArt_Final.pdf
- Crysis vegetation: https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-16-vegetation-procedural-animation-and-shading-crysis
- Sea of Thieves: https://history.siggraph.org/wp-content/uploads/2022/09/2018-Talks-Ang_The-Technical-Art-of-Sea-of-Thieves.pdf
- VRM spring bone: https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_springBone-1.0/README.md
- NvCloth: https://github.com/NVIDIAGameWorks/NvCloth
- Long Range Attachments: https://matthias-research.github.io/pages/publications/sca2012cloth.pdf
- TressFX: https://github.com/GPUOpen-Effects/TressFX
- Ubisoft compute cloth: https://www.slideshare.net/slideshow/efficient-usage-of-compute-shaders-on-xbox-one-and-ps4/269666732
- Flex: https://matthias-research.github.io/pages/publications/flex.pdf
- Vivace: https://publications.lib.chalmers.se/records/fulltext/245496/local_245496.pdf
- Direct Delta Mush: https://media.contentapi.ea.com/content/dam/ea/seed/presentations/direct-delta-mush-and-variants.pdf
- Direct Delta Mush compression: https://media.contentapi.ea.com/content/dam/ea/seed/presentations/ddm-compression-with-continuous-examples.pdf
- Dual quaternion skinning: https://users.cs.utah.edu/~ladislav/kavan08geometric/kavan08geometric.pdf
- Centres of rotation: https://la.disneyresearch.com/wp-content/uploads/Real-time-Skeletal-Skinning-with-Optimized-Centers-of-Rotation-Paper.pdf
- Proxy cloth for games: https://kuiwuchn.github.io/proxycloth/proxycloth.pdf
- Madden neural cloth: https://media.contentapi.ea.com/content/dam/ea/seed/presentations/seed-siggraph21-swish-cloth-simulation-madden-nfl-slides.pdf
- APEX Clothing runtime: https://archive.docs.nvidia.com/gameworks/content/gameworkslibrary/physx/apexsdk/APEX_Clothing/Clothing_PG_Runtime_Interfaces.html
- Unreal skeletal mesh cloth controls: https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/SkeletalMeshComponent?application_version=5.4
- Unity Cloth: https://docs.unity.com/en-us/engine/current/manual/physics-section/physics-overview/class-cloth
- Frostbite cloth self collision: https://media.contentapi.ea.com/content/dam/eacom/frostbite/files/gdc2018-chrislewin-clothselfcollisionwithpredictivecontacts.pdf
- Ghost of Yotei deep dive: https://blog.playstation.com/?p=411201
