# State of the art in capes, robes and secondary motion

This document was researched on 2026-11-19.
It belongs to the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts").
It answers one question.
Which techniques do the best-looking games and engines use for capes, robes and other loose parts, and which of them would suit kjk with Jedi Academy style acrobatics?

Every claim carries the URL of the page it was read on.
The words "not verified" mark claims that could not be confirmed on a primary source.
Many studios have never published how their cloth works, so several entries below are thin on purpose.
This was a web pass of one session.
Nothing was measured in kjk, and no cost below is a kjk measurement.

## Summary

- Almost every well-reviewed game cape or robe is a hybrid. A small simulated mesh or a chain of simulated bones sits on top of the skeleton, and it drives the bones or vertices of a much denser visual mesh. Full-resolution cloth on the whole garment is rare.
- Four stability tools appear in every published setup. Each simulated point may only stray a limited distance from where the animation would put it. The cloth collides with a handful of capsules or ellipsoids that follow the body, not with the body mesh. The simulation runs on a fixed small time step, often split into several substeps. The cloth only partly follows the character's motion in world space, so a fast spin or a teleport does not throw it away.
- The most useful published numbers are these. Remedy's jacket in Alan Wake limits each vertex to about 40 cm from its skinned place and mixes in only 10 to 30 percent of world-space motion ([Game Developer](https://www.gamedeveloper.com/programming/the-secrets-of-cloth-simulation-in-i-alan-wake-i-)). NvCloth allows at most 32 collision capsules per cloth ([NvCloth user guide](https://docs.nvidia.com/gameworks/content/gameworkslibrary/physx/nvCloth/UserGuide/Index.html)). Ubisoft says a compute shader rewrite made its cloth 10 times faster ([GDC Vault](https://gdcvault.com/play/1020857/Efficient-Usage-of-Compute-Shaders)).
- Machine learning cloth is not an option for capes. Epic's own documentation says its machine learning cloth assumes quasi-static cloth and is "not recommended for loose garments like dresses or capes" ([Epic](https://dev.epicgames.com/documentation/unreal-engine/machine-learning-cloth-simulation-overview)).
- The best fit for kjk is a position-based (XPBD) cloth on a low-resolution proxy made from the model's own danglymesh vertices, with the skinned pose as a distance limit and the ragdoll capsules as colliders. It would be sub-stepped on the CPU. Its output would drive the existing dangly vertices, so the render path stays as it is.
- A cheaper first step is a chain of simulated bones (a spring bone chain) for capes and hoods. It has the least authoring need, and it can be built from the danglymesh constraints and the skin weights.
- The owner's rule that effects must come from the games' data is met by all the shortlisted options. Each of them needs only the mesh, the skin weights, the per-vertex danglymesh constraint and the ragdoll capsules. None needs a hand-painted asset per model, but the setup rules must be generic, and a few tuning values per garment type will still be needed.

## How to read the per-game entries

For each game the entry names the technique, how it handles collision, how it keeps the cloth stable, the reported cost and how it is authored.
Where a studio has not said, the entry says so.
It does not fill the gap with a guess.

## Games

### Alan Wake (Remedy), the best published recipe

Remedy described its jacket cloth in detail ([Game Developer, "The Secrets Of Cloth Simulation In Alan Wake"](https://www.gamedeveloper.com/programming/the-secrets-of-cloth-simulation-in-i-alan-wake-i-)).
This is an older game, but it is the clearest account of a production cape-like setup, and its ideas still appear in current engines.

- **Technique.** The lower jacket is a low-resolution cloth mesh solved with Verlet integration on the CPU. Its result drives bones of the high-resolution visual jacket. Artists can therefore model the jacket freely, and normal maps carry the wrinkles. The hood is one extra bone moved by a Verlet pendulum.
- **Constraints.** Stretch constraints link neighbouring particles. Shear and bend constraints link diagonal and farther particles, with separate solver settings. Vertical stretch is solved top-down in one pass so that the length stays exact. Horizontal stretch is left loose because players notice it less.
- **Collision.** Particles are tested against ellipsoids that follow the legs and body. Long ellipsoids are split into more even ones.
- **Stability.** The physics and game logic run on a fixed time step. Each vertex may stray at most about 40 cm from its skinned rest position. The team found this limit worked better than collision planes at preventing knots and clipping.
- **Motion tuning.** The simulation runs mostly in the character's local space, and only 10 to 30 percent of world-space motion is added. Vertices that touch the body get more damping, which acts like friction. Edge vertices get less damping, more wind and a larger distance limit. The jacket is calm during a walk and lively during a dodge or a jump.
- **Cost.** The article gives no milliseconds. It says the low-resolution simulation mesh keeps the cost down, and that driving bones avoids rewriting a large vertex buffer.
- **Authoring.** No animator keys the jacket. The same system was reused for a down jacket and a veil by changing parameters. The article lists clipping on the low-resolution mesh as a known gap.

### Ghost of Tsushima and Ghost of Yotei (Sucker Punch)

- **Technique.** Sucker Punch runs a GPU compute cloth system with "a set of carefully tuned heuristics to simulate large numbers of individual cloth simulations efficiently." Ghost of Yotei added "multiple layers of simulated cloth" and cloth collisions ([PlayStation Blog, Ghost of Yotei tech deep dive](https://blog.playstation.com/?p=411201)).
- **Wind.** The cloth, ropes, tassels and foliage all read the same wind field, so everything on screen seems to move in the same gust ([GDC Vault, "Blowing from the West"](https://gdcvault.com/play/1027350/Blowing-from-the-West-Simulating)). The wind simulation favoured volume over accuracy, according to a [Game Developer write-up of the talk](https://gamedeveloper.com/design/using-vorticles-to-simulate-wind-in-i-ghost-of-tsushima-i-).
- **Not verified.** The solver type, the collision shapes, the stability rules and the cost are not public in the pages that could be read. The GDC Vault pages need a login.
- **Lesson for kjk.** The praise for this game comes largely from the shared wind. kjk already has area wind for danglymeshes, so the cape should read the same wind.

### Assassin's Creed Unity and later (Ubisoft Anvil)

- **Technique.** Ubisoft's system is called Motion Cloth. It started on the CPU. For Unity the team rewrote it for compute shaders and reports a 10 times speed gain ([GDC Vault, "Efficient Usage of Compute Shaders on Xbox One and PS4"](https://gdcvault.com/play/1020857/Efficient-Usage-of-Compute-Shaders)).
- **Batching.** Third-party copies of the slides say several cloth items are packed into one compute dispatch, up to 32 items, instead of 50 or more separate dispatches ([SlideShare copy](https://www.slideshare.net/slideshow/efficient-usage-of-compute-shaders-on-xbox-one-and-ps4/269666732)). This is not verified against the original deck.
- **Not verified.** Collision shapes, stability rules and per-character cost are not in the readable pages.
- **Lesson for kjk.** Cost per character falls sharply when many cloths are solved in one batch. This matters only if kjk ever puts cloth on crowds.

### Elden Ring and other FromSoftware games

- **Technique.** FromSoftware's cloaks are praised for moving procedurally in a physics engine, with settings tuned for each garment, according to [Polygon's video essay](https://sonicgame.netlify.app/host-https-www.polygon.com/videos/2024/6/20/24182415/elden-ring-cloth-shadow-of-the-erdtree).
- **Middleware.** [Havok lists Elden Ring](https://www.havok.com/havok-cloth/) among the games that use Havok Cloth. Havok does not say how FromSoftware set it up.
- **Collision.** A Dark Souls cape passing through the body is named as an example of clipping that players accept ([80.lv](https://80.lv/articles/cloth-simulation-for-games-difficulties-and-current-solutions)). This suggests the games accept some clipping and rely on motion and silhouette.
- **Not verified.** Everything about stability, cost and authoring. No talk by FromSoftware was found.

### Batman Arkham City and Arkham Knight (Rocksteady)

- **Technique.** The one technical account found is about Arkham City. The cape mixes a real-time cloth simulation driving a skeletal rig, hand-keyed animation, and offline cloth simulation baked to bones. Most gameplay uses the real-time simulation, and keyed animation takes over for iconic moves. Level artists placed wind volumes ([search summary of a Game Informer era article](https://gamesradar.com/batmans-cape-how-cloak-drew-players-cowl-gothams-protector)). This is not verified for Arkham Knight.
- **Reception.** GamesRadar praises how the cape billows, how its edges flick when Batman runs and how it swings with weight ([GamesRadar](https://gamesradar.com/batmans-cape-how-cloak-drew-players-cowl-gothams-protector)).
- **Lesson for kjk.** A cape that is simulated by default and replaced by a keyed animation for signature moves is a known pattern. kjk has no keyed cape animation, so it would use the simulation only.

### Hitman (IO Interactive)

- IO Interactive licensed the Shroud cloth middleware from CloakWorks for Hitman: Absolution in 2012 to simulate Agent 47's suit in real time ([MCV](https://mcvuk.com/development-news/hitman-studio-chooses-cloakworks-cloth-tech/)).
- No later talk about capes or coats was found, so the current technique is not verified.

### Uncharted 4 and The Last of Us (Naughty Dog)

- The only readable source says Uncharted 4 blends in normal-map wrinkles as the player moves, which works only for cloth close to the body, and that The Last of Us uses simple cloth for ropes and long one-piece garments ([80.lv](https://80.lv/articles/cloth-simulation-for-games-difficulties-and-current-solutions)).
- No Naughty Dog talk about cloth solvers was found. Their technique is not verified.

### Star Wars Jedi: Fallen Order and Survivor (Respawn)

- No technical source was found. Player reports on the EA forums say the Jedi robes in Survivor show no cloth movement and stay attached to Cal's legs, while other outfits do move ([EA forums](https://forums.ea.com/discussions/star-wars-jedi-survivor-en/jedi-survivor-jedi-robes-broken-physics/10712139)). Respawn's patch notes mention fixes to dynamic cloth, which shows the games do have a cloth system ([Game Skinny](https://www.gameskinny.com/news/star-wars-jedi-survivor-week-one-patch-notes-pc-playstation-xbox-fixes/)).
- The engine used here is Unreal, and its cloth is described in the Chaos entry below. Respawn's own setup is not verified.
- **Lesson for kjk.** The closest game to kjk's subject leaves the Jedi robe skirt almost rigid. Even a modest working skirt would stand out against it.

### The Witcher 3, Horizon Zero Dawn, Assassin's Creed Odyssey, Dark Souls 3, Spider-Man, Cyberpunk 2077, Final Fantasy XVI, Black Myth: Wukong, Destiny

- The only readable source names The Witcher 3, Horizon Zero Dawn, Dark Souls 3 and Assassin's Creed Odyssey as games with simple cloth on ropes and long one-piece garments ([80.lv](https://80.lv/articles/cloth-simulation-for-games-difficulties-and-current-solutions)).
- For Black Myth: Wukong, a search found only that the team moved from Unreal Engine 4 to Unreal Engine 5 in August 2021 and that clothing changes dynamically in battle. It gave no technique ([21jingji](https://m.21jingji.com/article/20240908/herald/ef6d0bc54aa528204297d7bd0ec91d88.html)).
- No source was found for Spider-Man, Cyberpunk 2077, Final Fantasy XVI, Final Fantasy VII Remake or Destiny. Their techniques are not verified, and none is described here.

## Engines and middleware

### Unreal Engine Chaos Cloth

- **Technique.** Chaos Cloth is a position-based cloth solver on a simulation mesh. Each particle has a Max Distance, the furthest it may be from its animated skinned position. A particle with a Max Distance of 0 is kinematic and follows the animation exactly ([Epic, Max Distance](https://dev.epicgames.com/documentation/unreal-engine/node-reference/Dataflow/SimulationMaxDistanceConfig)).
- **Backstop.** Each particle also has a backstop sphere placed behind its skinned position along the normal. The particle cannot enter that sphere, which keeps cloth off the body without testing the body mesh ([Epic, Backstop](https://dev.epicgames.com/documentation/unreal-engine/node-reference/Dataflow/SimulationBackstopConfig)).
- **Anim drive.** A spring pulls particles toward the animated pose, with a stiffness and a damping value ([Epic, Anim Drive](https://dev.epicgames.com/documentation/unreal-engine/node-reference/Dataflow/SimulationAnimDriveConfig)).
- **Teleport.** The engine offers "Force Next Update Teleport", which moves the particles to the new reference bone place and keeps the pose and velocities, and a reset, which drops the old state ([Epic, Force Next Update Teleport](https://dev.epicgames.com/documentation/unreal-engine/BlueprintAPI/ClothComponent/ForceNextUpdateTeleport)).
- **Authoring.** Artists paint Max Distance and backstop values on the simulation mesh, in the clothing tool or in the newer Dataflow cloth asset ([Epic, clothing tool](https://dev.epicgames.com/documentation/unreal-engine/clothing-tool-in-unreal-engine)).
- **Not verified.** Cost per character.

### Unreal machine learning cloth and ML Deformer

- **Technique.** A small neural network predicts cloth shape from the skeleton pose, and a nearest-neighbour search adds fine detail. The network needs about 5,000 simulated training poses for a humanoid and 50 to 100 key poses for the fine detail ([Epic](https://dev.epicgames.com/documentation/unreal-engine/machine-learning-cloth-simulation-overview)).
- **Limits.** The same page says the cloth is assumed to be quasi-static. It captures wrinkles but not swinging or draping. It names pants and T-shirts as the good cases and does not recommend dresses or capes.
- **Related work.** EA's Swish uses a small network and a normal-map library for tight clothing in Madden NFL 21 ([EA SEED](https://www.ea.com/seed/amp/news/swish-neural-network-cloth-sim-in-madden-nfl-21)). The page gives no runtime cost.
- **Fit for kjk.** Poor. It needs training data per garment, it breaks the rule of using only the games' data, and it cannot do a flying cape.

### Unity Cloth

- Unity's built-in cloth works only on skinned meshes. It has a per-particle Max Distance. It collides only with sphere, capsule and conical capsule colliders that the user lists. The restriction is stated as a performance choice. The simulation is one-way, so cloth never pushes back on a collider ([Unity manual](https://docs.unity3d.com/Manual/class-Cloth.html)).
- Self-collision and inter-collision can take a large share of the simulation time, according to the same manual.

### NVIDIA APEX, PhysX cloth and NvCloth

- NvCloth is a solver library for cloth. It uses tethers, which are long-range distance constraints from each particle to a fixed point, created when the cloth is cooked. They reduce stretch ([NvCloth user guide](https://docs.nvidia.com/gameworks/content/gameworkslibrary/physx/nvCloth/UserGuide/Index.html)).
- Collision uses capsules made of pairs of spheres, with a limit of 32 capsules per cloth, plus convex shapes and planes.
- The older PhysX cloth API has linear, angular and centrifugal inertia scales. These reduce how much a change in the character's global pose moves the cloth. This is the standard tool for taming fast moves ([PxCloth reference](https://docs.nvidia.com/gameworks/content/gameworkslibrary/physx/apireference/files/classPxCloth.html)).
- The solver frequency can be lowered to avoid oscillation or raised for stiffness and better collision detection.
- A forum user reports that its self-collision and inter-collision are unstable and need much tuning ([NVIDIA forum](https://forums.developer.nvidia.com/t/physx-3-4-cloth-missing-many-documented-features/50561)).

### Havok Cloth

- Havok Cloth offers layered cloth without full cloth-to-cloth collision, rigid attachments, selective collision, detection and resolution of pinched cloth, popping prevention, and variable frame rate and slow-motion support ([Havok](https://www.havok.com/havok-cloth/)).
- Authoring can be tuned down to single vertices, with setup tools for Maya and 3ds Max.
- Listed games include Elden Ring, Demon's Souls, The Legend of Zelda: Breath of the Wild and Tears of the Kingdom, Starfield, Helldivers 2, Doom: The Dark Ages and Street Fighter 6.
- No cost figures are published on the page.

### Spring bones in VRM and in Unity add-ons

- The VRM spring bone model is a chain of bones. Each chain has a stiffness (how strongly it returns to its rest pose), a drag force (how much motion is damped), a gravity power and direction, and a hit radius for collision. Colliders are spheres and capsules grouped by index ([three-vrm docs](https://pixiv.github.io/three-vrm/docs/interfaces/types-vrm-0.0.SecondaryAnimationSpring.html), [Bevy VRM source](https://docs.rs/crate/bevy_vrm/0.3.0/source/src/spring_bones.rs)).
- Each step keeps the bone length fixed by renormalising the tail position. This is a Verlet step with a length constraint.
- Magica Cloth for Unity offers both bone chains and mesh cloth with penetration limits and presets. Forum posts show collider limits on bone chains ([Unity forum](https://discussions.unity.com/t/released-magica-cloth/773425?page=37)). No performance numbers were found.
- This family is cheap and simple. It is also the weakest at keeping the shape of a wide cape, because the chains do not know about their neighbours unless extra constraints join them.

## Research on stability

### Position based dynamics and XPBD

- Position based dynamics solves constraints directly on positions, one after another (Müller and others, 2007). XPBD adds compliance, so stiffness no longer depends on the iteration count or the time step, and it approximates implicit Euler integration ([Macklin, Müller, Chentanez, XPBD](https://matthias-research.github.io/pages/publications/XPBD.pdf)).

### Small steps

- Macklin and others report that splitting a frame into n substeps with one solver iteration each is better than one big step with n iterations. Constraint error and damping drop, and stiffness settings stay stable over a wider range ([Eurographics page](https://diglib.eg.org/handle/10.1145/3309486-3340247), [PDF](https://diglib.eg.org/bitstreams/5ccd0aa4-edd9-48a1-8ed5-1c50d227a37a/download)).
- This is the right way to survive fast spins. A character that turns quickly moves the pinned cloth points far in one frame, and short substeps stop the cloth from being flung.
- A later paper notes that XPBD can still become unstable on complex scenes with a large time step ([Vertex Block Descent](https://arxiv.org/pdf/2403.06321)).

## What keeps cloth stable in extreme moves

These tools come from the sources above.

| Problem | Tool | Source |
|---|---|---|
| Cloth is flung by a spin or flip | Mix only 10 to 30 percent of world-space motion into the simulation | Alan Wake |
| Cloth is flung by a spin or flip | Linear, angular and centrifugal inertia scales | PhysX cloth |
| Cloth stretches or knots | Maximum distance from the skinned position, about 40 cm for a jacket | Alan Wake, Chaos, Unity |
| Cloth stretches | Tethers, or a top-down vertical length pass | NvCloth, Alan Wake |
| Cloth passes through legs or torso | Few capsules or ellipsoids, and a backstop sphere behind each point | Alan Wake, Chaos, NvCloth, Unity |
| Cloth jitters or explodes | Fixed time step, substeps with one iteration each | Alan Wake, Small Steps |
| Cloth explodes after a teleport or a cut | Move the particles with the character and keep or reset velocities | Chaos, PhysX cloth |
| Cloth looks dead when walking, wild when dodging | Extra damping where the cloth touches the body | Alan Wake |

## Cost

Almost no studio publishes a per-character figure, so none can be quoted with confidence.
The reports that exist are these.

- Ubisoft: a compute shader rewrite gave a 10 times gain, with up to 32 cloth items per dispatch.
- NvCloth: at most 32 capsules per cloth.
- Remedy: a low-resolution simulation mesh driving bones, with no vertex buffer rewrite.
- Unity: self-collision and inter-collision can take a large share of the total time.

A kjk benchmark is the only reliable source of numbers.
The roadmap issue already notes that the present sway rewrites each moving mesh on the CPU every frame.
A bone-driven cape would avoid that rewrite, but kjk skins from the games' data, so this needs a check in the code.

## Ranked shortlist for kjk with Jedi Academy style acrobatics

The ranking weighs how good the result looks in a spin, a flip and a wall jump, then the stability, then the cost, then the authoring need.

### 1. Low-resolution XPBD cloth with substeps on the existing dangly vertices

- **What it is.** Treat the danglymesh vertices as particles, and join each one to its neighbours with stretch, shear and bend constraints taken from the mesh edges. Pin or limit each particle to the skinned pose, using the danglymesh constraint value as the pin strength and the maximum distance. Solve it with XPBD in 4 to 8 substeps per frame with one iteration each. Collide it with the 14 ragdoll capsules plus a backstop along the skinned normal.
- **Why it looks best.** It gives a cape that spreads, folds and wraps, which a bone chain cannot do. It is the family of techniques behind the Alan Wake jacket and Chaos Cloth.
- **Stability in acrobatics.** Substeps, a maximum distance, an inertia scale of about 0.1 to 0.3 for world motion, tethers to the shoulders, and a teleport rule that moves the particles with the character. All of these are published tools.
- **Cost.** Not measured. Roughly a few hundred particles per garment with a handful of capsule tests, so the cost should be small per character, but this must be benchmarked.
- **Authoring.** None per model if the particle set, the edges and the pin strengths come from the mesh and from the danglymesh constraints. A few tuning values per garment class (cape, skirt, hood) are needed. A model whose cape is not a danglymesh needs the skin-weight step described in the next entry.

### 2. Spring bone chains for capes, hoods and robe skirts

- **What it is.** Each strand of vertices becomes a chain of bones with a fixed length, stiffness, drag, gravity and wind, with capsule colliders. This is the VRM and FromSoftware style.
- **Why it ranks second.** It is the cheapest and the easiest to keep stable. It looks good for hoods, ponytails and narrow cape tails. It looks weaker on a wide cape, because neighbouring chains do not hold the cloth together.
- **Stability in acrobatics.** The fixed bone length prevents stretching. Adding a maximum angle from the animated pose, the 10 to 30 percent world-motion mix and substeps keeps it calm.
- **Cost.** Lowest of the shortlist, because each chain needs a few vector operations per bone.
- **Authoring.** Chains can be found automatically from the skin weights, by following vertices whose weights are on a bone that hangs below a root. Models without such bones need the same skin-weight step as option 1.

### 3. Bones driven by a low-resolution simulation (Alan Wake method)

- **What it is.** Option 1 run on a coarse proxy of about 50 to 100 particles, with the result driving bones that skin the full mesh.
- **Why it ranks third.** It looks as good as option 1 on the whole, and it is the best match for a renderer that wants skinned meshes and clean motion vectors for TAA and DLSS. It needs a proxy and a bone set per garment, which the games' data does not contain. That breaks the owner's rule unless the proxy is built automatically.
- **Cost and authoring.** Low cost. The authoring need is a generated proxy and bone set per model.

### 4. Extra settings on the existing damped spring

- **What it is.** Raise the stray limit, add a gravity term, and add a limited capsule push-out, while keeping the present model.
- **Why it ranks fourth.** It costs almost nothing and it is a safe default. It cannot make a cape fly during a spin, because it has no inter-vertex constraints.
- **Authoring.** None.

### Not recommended

- **Machine learning cloth and ML Deformer.** Epic says it does not suit capes. It needs 5,000 simulated training poses per garment.
- **Full-resolution GPU cloth with self-collision.** The reports show it is expensive and tricky to tune. It would need a compute pass that kjk does not yet have for skinned characters.
- **Hand-keyed cape animation as in Arkham City.** It needs a hand-made asset for each move, which kjk cannot supply.

## Suggested next steps

1. Count which character models in each game already have danglymeshes, and on which parts. The roadmap issue lists this as unchecked, and options 1 and 2 depend on it.
2. Prototype option 4 first as the setting's "stronger" level. It is the safest default.
3. Prototype option 1 on one cape model, such as a Jedi robe or Darth Malak's cloak, in a test scene that spins, flips and wall jumps. Compare it with the original sway in the viewer.
4. Benchmark options 1 and 2 with the 1% low frame rate next to the mean, as the project's benchmark rule requires, and make each a setting if it costs frame time.
5. Share one wind field between the foliage sway and the cape, as Ghost of Tsushima does.

## Source gaps

- No talk or article was readable for the cloth in Jedi: Fallen Order and Survivor, Spider-Man, Cyberpunk 2077, Final Fantasy XVI, Final Fantasy VII Remake, Black Myth: Wukong, Destiny, Uncharted 4 and The Last of Us Part II.
- The Ghost of Tsushima and Ubisoft talks are behind the GDC Vault login. Only their abstracts were read.
- The NVIDIA GDC 2017 slides on animation technologies in Unreal Engine could not be read as text, so their cost numbers are missing.
- AMD TressFX and FidelityFX hair were not researched in this pass. They are hair simulations and matter only for the hair part of the issue.
