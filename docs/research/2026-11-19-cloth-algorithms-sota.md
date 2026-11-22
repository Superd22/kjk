# New cloth and secondary motion algorithms since 2022

This note was researched on 2026-11-19.
It belongs to the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts").
It answers one question.
Is there a new and better algorithm for real-time game cloth or secondary motion than XPBD with small substeps, and is there a "new kid on the block" that kjk should try?

The older notes cover the classic solvers and what shipped games do.
They are `docs/research/2026-11-19-loose-part-solvers.md` and `docs/research/2026-11-19-cloth-state-of-the-art.md`.
This note does not repeat them.
It looks only at work from 2022 to 2026 and at what the engines added recently.

Every claim carries a link.
The words "abstract only" or "project page only" mark claims where the full paper could not be read.
Nothing here was measured in kjk.
All timings are the authors' own, on their own hardware, and they are not comparable with kjk's 2.4 microseconds per character.

## Summary

- No recent algorithm beats XPBD with small substeps for kjk's bone points. A few dozen points per character at 1/240 s substeps is far below the size where any newer solver pays off. The newer solvers are all aimed at tens of thousands of vertices on a GPU, and none of them has a Rust version.
- The newest serious solver is Vertex Block Descent (VBD) from SIGGRAPH 2024, and its extension Augmented VBD from SIGGRAPH 2025. VBD solves implicit Euler one vertex at a time. The authors say it is unconditionally stable and faster than XPBD at the same settings on a 230,000 vertex test. The authors also say it is "not ideal for high-resolution stiff systems". The only cloth in the VBD paper is a tearing cloth of 2,500 vertices that takes 11.2 ms per step on a CPU. That is too slow for kjk's CPU budget.
- The most interesting new idea for kjk is not a solver. It is the collision model called Offset Geometric Contact (OGC), published in ACM Transactions on Graphics in 2025. OGC gives every vertex a displacement bound for each step, so that no vertex can cross a surface. It needs no continuous collision detection. This is the same kind of guarantee kjk lacks when a point tunnels through a leg capsule. The paper's code is in C++ and CUDA, and its authors tie it to VBD. Only the idea can be borrowed, and a point against capsule version of it is small. I have not read the full paper, so the exact formula must be taken from the PDF before implementing it.
- Learned cloth that runs in games has moved on to bone-driven networks in 2026 (PhySkin and HyperBones from Meta authors, and UNIC). They are fast, at about 1 ms per frame on a consumer GPU for HyperBones. They need training per garment or per character, and the one that states its physics assumption (PhySkin) is quasi-static. This means they model draping and not swinging. None of them fits a robe that must follow a roll or a spin.
- Spring bone methods have not produced a new algorithm that beats the bone chains in kjk. The closest research is Spring Decomposed Skinning (2025), which simulates bones with spring forces using position based dynamics. This confirms kjk's approach. Its code is Python.
- Shipped engines have added little to the solver itself. Unreal 5.6 and 5.7 added editor tools, outfit assets and cloth to cloth constraints. The collision features that Unreal documents are a collision thickness, an experimental inner thickness that runs continuous collision only where a particle has already gone deep, and a per-particle backstop. Unreal does not name a new solver in the pages I could read.
- The answer for kjk is to keep XPBD and to fix the collision. The ranked list at the end gives the order.

## What is meant by "better than XPBD with small substeps"

The reference method is the one kjk already runs.
It is XPBD, which means position based dynamics with compliance, with many small substeps and one solver iteration per substep.
Its source is the Small Steps paper of Macklin and others, at https://diglib.eg.org/handle/10.1145/3309486-3340247 (abstract page, and the earlier note cites the PDF).
A newer method has to beat it on at least one of these points.

- Stability under fast moves, which for kjk means spins, flips, cartwheels and hard stops.
- Collision that does not tunnel and does not trap cloth on the wrong side of the body.
- Cost for the sizes kjk uses.

The sections below test each candidate against these three points.

## Vertex Block Descent (VBD)

**Source.** Chen, Liu, Yang and Yuksel, "Vertex Block Descent", SIGGRAPH 2024. The arXiv version is https://arxiv.org/abs/2403.06321, and the project page is https://graphics.cs.utah.edu/research/projects/vbd/. I read the arXiv HTML version up to its reference list, which is about 100,000 of 120,000 characters (https://arxiv.org/html/2403.06321v3). I could not watch the supplemental video.

**What it is.** VBD is "a block coordinate descent solution for the variational form of implicit Euler". Each vertex is moved in turn to the position that lowers the total energy, using only its own 3 by 3 system. Vertices that share no element are given the same colour and are updated at the same time.

**What it improves over XPBD.**
- The authors say it is unconditionally stable and that its iteration count can be cut to fit a budget.
- It handles large mass ratios. In a 1 to 2000 mass ratio cube test, the paper says XPBD "cannot overcome the mass ratio" and that VBD has "no such difficulties".
- On a squishy ball with tentacles (230,000 vertices and 700,000 tetrahedra), VBD at h = 1/120 s with 120 iterations took 0.031 s per frame. XPBD with the same settings took 0.32 s. XPBD with h = 1/3000 s and 5 iterations took 0.35 s when collisions were checked every 125 iterations, and 3.1 s when they were checked every 5 iterations.

**Cost.**
- The only cloth test is "Tearing Cloth", with 2,500 vertices and 4,800 triangles. The table gives 11.2 ms average and 11.5 ms maximum per step, with h = 1/300 s and 20 iterations, and marks it as CPU.
- The test machine was an AMD Ryzen 5950X and an NVIDIA RTX 4090. The paper says collisions run on the CPU through Embree and the two parallel phases run on the GPU through CUDA.
- If those 11.2 ms per step were also kjk's cost, five steps per frame would take about 56 ms. This is only a rough scaling and it was not measured. It shows that VBD is not a cheap CPU method at this size. It is a GPU method.

**Stability under fast moves.** The paper tests recovery from bad initial states, such as a teapot with scattered vertices. It does not test spinning characters. The authors state that "VBD is not ideal for high-resolution stiff systems, as it may require too many iterations". They also state that its penetration-based collisions "don't guarantee penetration-free results". Cloth is stiff against stretch, so the first limit applies to garments.

**Collision.** Collision is a quadratic penetration energy. Edge to edge pairs use continuous collision detection. Vertex to triangle pairs use continuous or discrete detection. Friction follows the model of IPC. There is no body proxy of capsules in the paper, since the tests use triangle meshes.

**Fit for kjk.** It suits a few thousand vertices on a GPU, and it does not suit a few dozen bone points. For a few dozen points, a per-vertex 3 by 3 solve gives nothing that a distance and bend projection does not already give.

**Code.**
- The authors' C++ code is in the Gaia engine, at https://github.com/AnkaChan/Gaia, under the Apache 2.0 licence. Its README lists a VBD simulator and a PBD simulator, CPU libraries (OneTBB, Eigen, Embree) and a Windows and Visual Studio test only. The README does not mention cloth. I read the repository front page only.
- NVIDIA's open Newton physics engine has a VBD cloth solver in Python (https://github.com/newton-physics/newton/tree/main/newton/_src/solvers/vbd, named on the OGC page below). I could not open its documentation page, so its features are not verified.
- A search for a Rust version found none. The only other ports found were for Houdini (C++, VEX and OpenCL, https://github.com/mysterypancake/houdini-vbd) and a third-party CUDA project (below).

**Worth trying in kjk.** Not now. It would only matter for the later real vertex cloth on the main character's skirt, and only if XPBD fails there.

## Augmented Vertex Block Descent (AVBD)

**Source.** Giles, Diaz and Yuksel, "Augmented Vertex Block Descent", ACM Transactions on Graphics 44, 2025, DOI 10.1145/3731195. The project page is https://graphics.cs.utah.edu/research/projects/avbd/. I read the project page and the Crossref and OpenAlex abstract through a search result, and I did not read the paper. The PDF of the two-page Real-Time Live summary (https://graphics.cs.utah.edu/research/projects/avbd/Augmented_VBD-SIGGRAPH25_RTL.pdf) could not be read as text.

**What it improves over VBD.** It adds an augmented Lagrangian formulation. The project page says this lets it "handle infinitely stiff hard constraints without numerical instability" and converge better at high stiffness ratios. These were the two weak points of VBD.

**Cost.** The project page gives no numbers. A search summary of the Real-Time Live abstract says 3.5 ms per frame on an RTX 4090 without collision detection and 9.8 ms with it. I could not confirm this on the original, so treat it as unverified. The scenes are rigid and soft bodies, and cloth is not mentioned on the project page.

**Collision.** The project page says AVBD maintains contacts, stacking and friction, and shows chainmail. It does not describe the collision method.

**Fit for kjk.** It targets rigid and soft body piles on a GPU. It has nothing for a few dozen bone points. A hard constraint solver could help the tether and max-distance constraints in principle, but kjk already gets those from XPBD's small steps.

**Code.** Two MIT licensed C++ demos are by the first author, at https://github.com/savant117/avbd-demo2d and https://github.com/savant117/avbd-demo3d. The 3D README calls it "an easy to understand demonstration" and not an optimised implementation. No Rust version was found.

**Worth trying in kjk.** No.

## Offset Geometric Contact (OGC)

**Source.** Chen, Hsu, Liu, Macklin, Yang and Yuksel, "Offset Geometric Contact", ACM Transactions on Graphics 44(4), 2025, DOI 10.1145/3731205. The project page is https://graphics.cs.utah.edu/research/projects/ogc/ and the author's page is https://ankachan.github.io/Projects/OGC. I read both pages, which give the abstract and a few timings. I did not read the full paper, because the PDF could not be read as text. One search result calls the venue SIGGRAPH and another calls it SIGGRAPH Asia, so the venue is unverified.

**What it improves over XPBD.**
- Each face is offset along its normal to make a volumetric contact shape, which lets the contact radius be large without artefacts.
- Each vertex gets "vertex-specific displacement bounds". These bounds guarantee that the simulation stays free of penetration, "avoid CCD" and improve convergence.
- The authors say this is more than two orders of magnitude faster than prior methods. The page does not name the baseline.

**Cost.** The teaser drops 50 layers of cloth (246,000 vertices and 475,000 triangles) on a cylinder. The average and maximum time per step is 0.21 and 0.55 s on a CPU and 6.3 and 11.5 ms on an RTX 4090. The CPU model is not named on the page. A twisted cloth test has 40,000 vertices and a fixed 5 mm contact radius, and IPC shows artefacts there where OGC does not.

**Stability and collision.** This is the part that matches kjk's problem. A displacement bound per vertex means that a vertex cannot move farther in one step than the distance to the nearest surface it could cross. A point that cannot cross a leg capsule cannot be pushed out on the wrong side. The guarantee holds only if the cloth starts without penetration. The authors do not claim to recover cloth that is already inside the body.

**Fit for kjk.** The full method is made for triangle meshes with many vertices. For bone points, the same idea reduces to a small rule. Before each substep, find the distance from the point to each capsule surface, and clamp the point's move so that it cannot reach a surface. This is my own adaptation and it was not tested. It is cheap, because kjk has 14 capsules and a few dozen points. It is also a form of continuous collision against capsules, and a simple swept test of the point's last position against the capsule would give a similar result.

**Code.**
- The authors point to Gaia (https://github.com/AnkaChan/Gaia) and to Newton's VBD solver (https://github.com/newton-physics/newton/tree/main/newton/_src/solvers/vbd). I did not open the OGC code inside either one. The author's page says the Newton link "suggests" the VBD solver is where OGC lives, which is an inference from the URL.
- A third-party CUDA engine for OGC exists at https://github.com/mmtmn/OGC. It is not official, its licence is not stated, and I did not verify its quality.
- No Rust version was found.

**Worth trying in kjk.** Yes, as an idea. It is the only recent result aimed at the collision failure that kjk shows. Read the paper's displacement bound formula before using it.

## Other implicit and penetration-free solvers

### Barrier-free penetration-free elastodynamics (2025 to 2026)

- Zheng, Luo and Li, "Robust and Efficient Penetration-Free Elastodynamics without Barriers", ACM Transactions on Graphics 2026, https://arxiv.org/abs/2512.12151. I read the abstract only.
- It replaces the barrier functions of IPC with an augmented Lagrangian and adds all contact pairs found by continuous collision detection at once. The authors report "up to 103x speedup over GIPC" on contact-rich benchmarks. The abstract does not mention cloth. The authors say code and data are open at https://simulation-intelligence.github.io/barrier-free, which I did not open.
- This is a general GPU method that targets scenes with heavy contact. It does not fit a garment on a character in a game.
- Worth trying in kjk: no.

### GPU cloth with non-distance barriers and subspace reuse (2024)

- Lan and others, https://arxiv.org/abs/2403.19272 (arXiv only, abstract read). It extends projective dynamics with a barrier weight based on the "virtual life span of a collision event" and reuses a low-frequency subspace.
- The authors claim it "outperforms existing fast cloth simulators by at least one order". The abstract gives no numbers and no code link.
- It targets high-resolution garments on a GPU. Worth trying in kjk: no.

### Two-way continuous collision handling (2022)

- Wang and others, https://arxiv.org/abs/2211.04045 (arXiv only, abstract read). It runs optimisation both from the integrated state and from the previous intersection-free state, and it alternates between them until the result converges.
- The authors say it handles volumes, cloth, hair and sand, and that it is robust for large time steps. The abstract gives no numbers and no code link.
- The idea of keeping the last intersection-free state is relevant. For a point, it means that when a point ends a substep inside a capsule, the solver can return to the last substep position and slide it along the surface. This is a restatement of the abstract in kjk's terms, and not a claim from the paper.
- Worth trying in kjk: as an idea only.

### Generalised PBD (2025)

- Chaudhary, Pokhariya and Narain, https://arxiv.org/abs/2511.23131 (search result abstract only). It extends PBD to constraints that are not linearly compliant, such as data-driven cloth models and a neo-Hookean model with an inversion barrier, and it reports speedups over a Newton baseline at high mesh resolutions.
- It improves material accuracy for vertex cloth and does nothing for collision or spins. Worth trying in kjk: no.

### Primal and dual descent methods (2020)

- Macklin and others, Computer Graphics Forum 39(8), 2020, https://diglib.eg.org:443/handle/10.1111/cgf14104. It compares force-based and constraint-based formulations and proposes a preconditioned frictional contact solver. This is before the 2022 window and it is for contact, not cloth, so it is listed only to explain that the "stable constrained dynamics" line did not produce a cloth solver for games. Abstract only.

### Search result for XPBD follow-ups

- I found no 2024 to 2026 paper that addresses XPBD tunnelling or trapped cloth directly. One search showed XPBI (Yu and others, 2024), which adds plasticity for snow and sand, and it does not apply here. The search was one query and it is not proof that no such paper exists.

## Learned cloth that runs in games

**Epic machine learning cloth.** Epic says it assumes quasi-static cloth and does not recommend it for dresses or capes (https://dev.epicgames.com/documentation/unreal-engine/machine-learning-cloth-simulation-overview). This was in the earlier note.

**PhySkin (2026).** Srivastava and others, https://arxiv.org/abs/2603.27013 (abstract only). It is a reduced-space and quasi-static neural simulation that maps garment degrees of freedom to bone handles. It trains without simulated data, and it reports microseconds per frame on a single CPU thread. The abstract targets loose-fitting garments, and the quasi-static label means it predicts a draped rest shape for a pose and not momentum. No code is linked.

**HyperBones (2026).** Srivastava and others, https://arxiv.org/abs/2605.20460 (abstract only). It uses virtual bones and a network that predicts corrections to linear blend skinning, plus a wrinkle network. It reports about 1 ms per frame on a consumer GPU and 30 times faster than earlier autoregressive neural simulators. No code is linked.

**UNIC (2026).** Zhao and others, https://arxiv.org/abs/2603.25580 (abstract only). It learns an instance-specific neural deformation field for each garment, so every garment needs its own training. The abstract does not say whether loose garments are covered. A project page exists at https://igl-hkust.github.io/UNIC/, which I did not open.

**Limits for kjk.** All three need training, and kjk has no source of poses and simulated or self-supervised training for every robe in two games. They also do not show what happens after a Jedi Academy roll, which is far outside normal training motion. They break the rule that effects must come from the games' data. Worth trying in kjk: no.

## Spring bones and secondary motion

**Spring Decomposed Skinning (2025).** Akyürek and Sahillioğlu, Computer Graphics Forum 44(5), 2025, https://bartuakyurek.github.io/publications/SDS (project page and abstract only, and the publisher page returned an error). It simulates the skinning bones as spring bones with position based dynamics, with no volumetric mesh. The authors say it applies to rigid bodies, elastic bodies, soft tissue and cloth. The code is Python under the MIT licence at https://github.com/bartuakyurek/Spring-Decomposed-Skinning, and it is marked as a work in progress. This is the same idea as kjk's loose bones, and it gives kjk nothing to copy beyond confirmation.

**VRM 1.0 extended colliders.** The VRM spring bone extension is version 1.0 and complete. It adds an inside sphere and an inside capsule, which keep spring bones inside the shape instead of outside, and a plane, which keeps them on one side (https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_springBone_extended_collider-1.0/README.md). This is not new research, but it is a cheap and standard tool. A skirt chain could be kept inside a loose capsule drawn around the legs, so that a point inside the leg can never be pushed through to the outside. This would address the "wrong side" trap by making the allowed side explicit.

**Magica Cloth 2.** The Unity asset has bone cloth with an overall angle limit and a "Clamp Position" setting for the farthest distance from the origin. These are from forum replies by its developer and not first-party documentation (https://discussions.unity.com/t/released-magica-cloth/773425?page=37). One reply says per-axis angle limits were not available then. Its performance numbers were not found. This supports the "angle (cone) limit per bone" option as an accepted practice.

## What shipped engines added

**Unreal Chaos Cloth.**
- Unreal 5.6 added a beta cloth panel editor in the unified Dataflow editor, an outfit asset system with refitting, cloth to cloth constraints and simulation morph targets. A forum reply says cloth to cloth did not work as expected in 5.6 (https://forums.unrealengine.com/t/tutorial-chaos-cloth-updates-5-6/2555686). Unreal 5.7 improved the editor timeline and painting tools (https://forums.unrealengine.com/t/tutorial-chaos-cloth-updates-5-7/2674293). The official 5.7 release notes page returned an empty table of contents, so I could not read Epic's own list. These are forum tutorials and not first-party documents.
- The collision settings are documented first-party. The cloth collision thickness is added to the cloth when it hits collision shapes. An experimental inner collision thickness "runs the CCD code path, but only where particles penetration exceeds the given inner thickness value", and the page says it is faster than full continuous collision and does not resolve fast moving objects ([SimulationCollisionConfig](https://dev.epicgames.com/documentation/unreal-engine/node-reference/Dataflow/SimulationCollisionConfig)). The self collision page notes that self intersection resolution is a separate pass that "can be expensive" ([SimulationSelfCollisionConfig](https://dev.epicgames.com/documentation/unreal-engine/node-reference/Dataflow/SimulationSelfCollisionConfig)).
- The idea of a two-tier collision, with a cheap discrete check and an expensive continuous check only for deep points, is what kjk could copy for capsules.
- The kinematic collider was deprecated in favour of skinned triangle mesh bodies in the physics asset. This is from the same search result and I did not read the page it came from.
- Forum users report self collision problems in 5.5 (https://forums.unrealengine.com/t/simple-chaos-cloth-self-collision-causes-freak-out-on-skm-in-ue-5-5/2331953). This is an anecdote.

**Unity, EA, Ubisoft, NVIDIA and AMD.** I found no first-party 2024 to 2026 talk or document about a new cloth solver from these companies. A search for GDC 2024 and 2025 cloth talks returned only talks from 2017 and 2018, including EA Frostbite's predictive contacts talk for self collision (https://media.contentapi.ea.com/content/dam/eacom/frostbite/files/gdc2018-chrislewin-clothselfcollisionwithpredictivecontacts.pdf). I could not read that PDF as text. NVIDIA's Newton engine (above) is the newest NVIDIA-linked source, and its VBD solver is GPU Python code.

## Why the fast-move problems do not need a new solver

Three of kjk's observed problems have known fixes that are not new solvers.

- **Cloth stays flared after a roll, and lags in a spin.** This is about how much of the character's world motion the cloth feels. The tools are the inertia scales and the world-space mix of 10 to 30 percent from Alan Wake. They are in the earlier note, and kjk already has inertia scaling in its solver. A newer solver would not change this.
- **Cloth bunches at the hip.** This is a collision trap. The fixes found in this research are the per-vertex displacement bound of OGC, the two-way idea of returning to the last collision-free state, the inside colliders of VRM, and the two-tier thickness and CCD of Chaos Cloth.
- **Folding and bulging of the skinned mesh.** This comes from coarse bones, and it is not a solver problem. The options already under consideration are generated skirt bones, angle limits and later vertex cloth.

## Ranked list

1. **Try the OGC idea in a small form, together with an inside collider.** Give each point a per-substep displacement bound against the 14 capsules, so that a point cannot cross a capsule surface in one substep. Also remember the last substep position that was outside, and move a point that ends inside back to it and slide it along the surface. Optionally add a loose inside capsule around the legs for the skirt. This directly targets the lump at the hip. It costs a few capsule distance tests per point. I have not read the full OGC paper, so confirm the bound formula first. No Rust code exists, and none is needed.
2. **Add angle limits per bone and generated skirt bones.** These are the existing options. They are supported as practice by Magica Cloth 2 and by the spring decomposed skinning idea, and they target the folding and bulging. They are not new algorithms.
3. **Keep XPBD for later vertex cloth on the main character.** Test XPBD first at a few hundred vertices on the skirt, with the same collision fix as item 1. Only if that fails with stretch or stiffness, look at VBD, as a CPU or GPU port written in Rust from Gaia and the paper. No Rust version exists. The VBD paper's own cloth timing (11.2 ms for 2,500 vertices per step) shows it is a GPU method.
4. **Ignore for now.** AVBD, the barrier-free penetration-free solver, the non-distance barrier projective dynamics, generalised PBD, and every learned garment method (PhySkin, HyperBones, UNIC and Epic's machine learning cloth). They target thousands of vertices on a GPU or they need training for each garment.

Nothing beats XPBD with small substeps for kjk's few dozen bone points per character.
The one recent paper that matters for kjk is OGC, and what it gives is a collision idea and not a replacement solver.

## Sources

- Vertex Block Descent (arXiv): https://arxiv.org/abs/2403.06321, HTML: https://arxiv.org/html/2403.06321v3, project page: https://graphics.cs.utah.edu/research/projects/vbd/
- Gaia, the authors' C++ code: https://github.com/AnkaChan/Gaia
- Augmented VBD project page: https://graphics.cs.utah.edu/research/projects/avbd/
- AVBD demos: https://github.com/savant117/avbd-demo2d and https://github.com/savant117/avbd-demo3d
- Offset Geometric Contact project page: https://graphics.cs.utah.edu/research/projects/ogc/, author's page: https://ankachan.github.io/Projects/OGC
- Newton VBD solver: https://github.com/newton-physics/newton/tree/main/newton/_src/solvers/vbd
- Third-party CUDA OGC: https://github.com/mmtmn/OGC
- Barrier-free penetration-free elastodynamics: https://arxiv.org/abs/2512.12151
- GPU cloth with non-distance barriers: https://arxiv.org/abs/2403.19272
- Two-way continuous collision handling: https://arxiv.org/abs/2211.04045
- Generalised PBD: https://arxiv.org/abs/2511.23131
- Primal and dual descent methods: https://diglib.eg.org:443/handle/10.1111/cgf14104
- PhySkin: https://arxiv.org/abs/2603.27013
- HyperBones: https://arxiv.org/abs/2605.20460
- UNIC: https://arxiv.org/abs/2603.25580
- Spring Decomposed Skinning: https://bartuakyurek.github.io/publications/SDS and https://github.com/bartuakyurek/Spring-Decomposed-Skinning
- VRM extended collider specification: https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_springBone_extended_collider-1.0/README.md
- Magica Cloth forum thread: https://discussions.unity.com/t/released-magica-cloth/773425?page=37
- Unreal Chaos Cloth collision configuration: https://dev.epicgames.com/documentation/unreal-engine/node-reference/Dataflow/SimulationCollisionConfig
- Unreal Chaos Cloth self collision configuration: https://dev.epicgames.com/documentation/unreal-engine/node-reference/Dataflow/SimulationSelfCollisionConfig
- Unreal Chaos Cloth tutorials for 5.6 and 5.7: https://forums.unrealengine.com/t/tutorial-chaos-cloth-updates-5-6/2555686 and https://forums.unrealengine.com/t/tutorial-chaos-cloth-updates-5-7/2674293
- The roadmap issue: https://github.com/kjk-engine/roadmap/issues/5
