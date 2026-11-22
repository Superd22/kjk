# Robe and cape physics plan

This note was written on 2026-11-19. It is the plan for the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts"). It brings together the ten research notes listed at the end, and the decisions the user made while reading them.

## For agents

Read this whole note before working on robes, capes or other loose parts. Then follow these rules.
- Say which step of this plan your work serves (L0 to L7).
- Remember that kjk's movement copies Jedi Academy. Characters spin, flip, cartwheel and roll, so capes and robes will fly around. Every choice must hold up under those moves.
- Step the solver by game time, in fixed substeps, never by the wall time of a frame. Otherwise two recordings of the same scene do not match.
- Benchmark every change as soon as it exists. Give mean fps and 1% low fps for every run. The 1% low is 1000 divided by the run's p99 frame time.
- Keep the solver in the view. It must never touch the deterministic simulation (`docs/SKELETON-SIM.md`).
- Do not copy class or function names from the original executables into code or text.
- Write every text for a person by the rules of `CLAUDE.md` at the root of the kjk repo.
- When your work changes what is done or what comes next, update "Now and next" below.

## The goal

Robes, capes, hoods and other loose parts should move far better than in the original games. They should look good and stay stable during Jedi Academy acrobatics, and they should not pass through the character's legs and body.

## What the research found

- The original games have no physics for robes or capes. The danglymesh sway that kjk already copies is their only physics, and it lets a vertex move only a few centimetres. Every large movement of a robe or a cape in the games is hand-keyed by an artist.
- Most robes and capes are moved by bones, not by danglymeshes. KOTOR 2's robes are skinned to 11 robe bones. The long capes of Revan, Malak and Nihilus are skinned to chains or trees of cape bones. KOTOR 1's Jedi robe skirts are part of the legs mesh and have no bones of their own.
- Danglymeshes are mostly hair. The cloth danglymeshes are small: 12 to 104 vertices each.
- Outside a fight, kjk plays the KOTOR 2 robe keys correctly. During a fight, kjk uses Jedi Academy animations, and the robe bones stay in their rest pose. No animation moves the capes of KOTOR 2 Revan or KOTOR 2 Malak.
- Of the 920 animations that key a loose bone, 894 only imitate physics. 14 are real flourishes, and all of them pull a hood up or down. 12 are unclear and need a person to watch them.
- The current sway costs 0.06 to 0.11 ms per frame in the busiest crowd, for 3,700 vertices. A bone solver moves a few dozen points per character, so it should cost much less than vertex cloth.

## Decisions

1. New behaviour whose inputs all come from the games' data counts as "built from the games' own data". Collision with the body, swaying skinned robes and larger limits are allowed.
2. One system moves every loose part. There are no separate quick fixes, such as playing the KOTOR 2 robe keys during fights.
3. The system is one XPBD solver that moves points. A point is either a bone joint or a mesh vertex. It starts with bone points.
4. KOTOR 1 robe skirts may later get generated bones. That needs no change to the solver.
5. Hand-keyed motion of loose bones that only imitates physics is dropped, because the solver replaces it. Real flourishes stay. In a flourish animation, the keyed pose becomes the solver's target, and the physics works around it.
6. If the solver is cheap, it is on by default with no setting. If it costs performance, it becomes a setting.
7. Bones carry no physics values. Where the data gives nothing to compute from, each kind of garment gets one sane default. Where the data does give something, such as chain length or how much mesh a bone carries, it adjusts that default.

## The design

### Finding loose bones

A loose bone is a bone that the humanoid skeleton map does not know, that carries garment vertices, and that hangs from the pelvis or the torso. The model must pass the humanoid test. This rule finds 18 KOTOR 2 and 10 KOTOR 1 character models, and every hit checked was a garment (`2026-11-19-loose-bone-solver-prep.md`, section 3).

Two problems remain.
- The rule drops `breastbone` with a test on how often a bone's name appears. That test is fragile and must be replaced by a cleaner one, or by a short reviewed list of exceptions.
- The rule misses hood bones under the head, sleeve bones under the forearms, Revan's robe sides, Bastila's thigh flaps and Mandalorian shoulder plates. A looser version catches most of them, but it also catches some face bones, Rodian antennae, Trandoshan eyelids and the Jawa's hood.

For a closed set of models, a rule from the data plus a short reviewed list of exceptions is acceptable.

### Points

- Each loose bone gives one point at its joint. Each last bone of a chain also gives a tip point. The tip's length is the farthest vertex skinned to that bone with a weight of at least 0.5, held between 0.05 and 0.6 m.
- The first loose bone of each chain is pinned to its animated parent.
- Chains can branch, as on KOTOR 1 Malak's cape, so the solver handles trees.
- Neighbouring chains are linked, so a robe does not split apart. Two chains are linked if they share a root, if at least 10 vertices carry a weight of 0.2 or more on both, or if they hang close together on the same side of the pelvis.
- After the solve, each bone is rotated, from the top of the tree down, to point at its child's new position. It keeps the twist that the animation gave it. Its translation does not change.
- The model's `inverse_binds` must be used, never `rest_inverse_binds`, which is wrong by 1.2 m on `pmbim`.

### The solver

- XPBD at a fixed step taken from game time, with substeps. The solver note suggests 8 substeps of 1/240 s. The real number is set by the stress scene and the bench.
- The solve happens relative to the character's root. The cloth feels only part of the character's movement through the world, and each substep limits how far the root can move or turn.
- Each point has a maximum distance from its target pose.
- Each point has a tether to its pinned root.
- A teleport, or a jump of more than 5 m, resets the points to their target pose.

### Collision

- The 14 ragdoll capsules are posed every frame from the animated skeleton. Each capsule moves with its bone.
- The fit returns capsules on every robed and caped body tested, and their radii are close to those of plain bodies (`2026-11-19-robe-capsules-and-stress-scenes.md`). The user watched the capsules drawn over 17 bodies through rolls, flips and cartwheels on 2026-11-19. The capsules follow their limbs and are about the size of the limbs, also on slim bodies such as Talia and `pfbl`. The foot capsules are sometimes a little too high or tilted up, but not by much. Few robe vertices start inside a capsule, and most of those are where the robe is tied to the body at the back and the lower back.
- Many garment vertices start inside a capsule in the rest pose: 15% to 24% on KOTOR 1 Jedi robes, and up to 30% on some KOTOR 2 robes. Each point therefore gets its own collision radius. It is the smaller of the capsule's radius and the point's rest distance from the capsule's axis, minus 0.5 cm.
- The pelvis, belly and chest get a backstop. A backstop keeps each point on the outside of the skinned body.
- The floor is one plane at the walkmesh height. Walls can wait.

### Where it runs

- The solver is one new system in `PostUpdate`. It runs after every system that writes bone transforms and before `TransformSystems::Propagate`. At that point every bone holds its final animated pose.
- It walks the skeleton itself, because world transforms are not ready yet.
- Nothing writes the loose bones after it, so the skinning and the shadows see the result in the same frame. Bevy keeps the previous frame's joints, so TAA and DLSS motion vectors stay correct.
- It runs on a worker thread, and each character can be solved in parallel.
- The cast must reset loose bones to their target every frame instead of holding them (`held_nodes` in `crates/game/src/cast.rs`).
- The cast must publish which characters it posed this frame. The solver leaves the others alone.
- When an animation is loaded, its loose-bone keys are dropped, unless the animation is a flourish.

## Steps

**L0. Bench and stress tools.** Land the sway timer and mesh counts from the `dangly-bench` worktree. Land the `robe_acrobatics` bench scene from the `robe-stress` worktree. That scene has a `pmbim` player doing flips, cartwheels, rolls and spins among 44 robed characters. Record the baseline again three times on a quiet machine.

**L1. Loose bone finder.** Build the rule and its reviewed exceptions. Add an approved test that lists, for every model in both games, its loose trees, their roots, their neighbour links and the kind of garment.

**L2. Physics values.** Set one default per kind of garment: cape, robe skirt, hood, flap and sleeve. Choose which data adjusts it, such as chain length, distance from the root and mesh carried.

**L3. Solver core.** Write the XPBD solver on points, with trees, tethers, the distance limit, the partial world motion, fixed substeps and the teleport reset. Add approved tests for five hard cases: a spin, a flip, hanging upside down, a hard stop and a teleport.

**L4. Collision.** Pose the capsules every frame. Add the per-point radius, the backstop and the floor plane.

**L5. Engine hook.** Add the new system, the change to held bones, the list of posed characters and the per-animation flag. Start with the 11 KOTOR 2 robe bodies and the cape models.

**L6. Look and cost.** Record `robe_acrobatics` before and after, and watch it. Benchmark it with mean and 1% low fps. Decide whether it stays on by default. The deciding benchmark runs at the very end on a weak machine, such as a rented GTX 1650 or GTX 1050 Ti. Until then, Mac runs only need to be good enough to catch large costs, and the solver's own timer is the main measure.

**L7. Extend.** Add hoods, with their 14 flourish animations as targets. Add sleeves and flaps. Move the danglymesh vertices into the same solver, which also ends the sway's dependence on the frame rate. Later, give KOTOR 1 robe skirts generated bones.

## Now and next

- Done: the research and the decisions above.
- Done: the viewer flags `--capsules`, `--marks`, `--bones`, `--ghost` and `--clip` for `kjk view` and `kjk rig`. They are committed on `loose-parts-l0`.
- The overview page of every recording is `kjk-artifacts/loose-parts/index.html` in the main checkout. It is built by a script in the session scratchpad and is not committed.
- Done: the user watched the clips and decided on 2026-11-19 that all 11 Malak cutscene clips and Darth Nihilus's lightsaber throw (`throwsablp`) are fake physics. The odd look of the Malak clips came from the viewer (`2026-11-19-malak-cutscene-cloth.md`). The only flourishes are the hood animations.
- L0 is done on the branch `loose-parts-l0`, in three commits. They hold the sway timer and counts, the `KJK_DANGLY=off` switch, the `robe_acrobatics` bench scene and the viewer options. On a quiet Mac (load average 3.7 to 4.4), three runs of `robe_acrobatics` gave 71.1, 80.9 and 66.0 mean fps, with 1% lows of 23.6, 22.3 and 19.1 fps. The scene itself varies by about 10% between runs. The sway took 0.011 ms per frame in every run.
- L1 and L2 are done on the branch `loose-parts-l12`, in five commits, reviewed and fixed. The finder and the values are in `crates/asset_ja/src/loose.rs` and `crates/asset_ja/src/loose/physics.rs`. It finds 11 KOTOR 1 and 29 KOTOR 2 character models. Its approved test `loose_bones` awaits approval.
- L3 and L4 are done on the branch `loose-parts-l34`, in three commits, reviewed and fixed. The solver is the new crate `crates/loose`. It runs only in the viewer for now, with `--sim`, on a temporary choice of bones and values. Its approved test `loose_parts` awaits approval.
- Both branches start from `loose-parts-l0`.
- The user accepted the change to the tip length rule in L1.
- L5 is done on the branch `loose-parts-l5`, made from `loose-parts-l0` with both branches merged in, in five commits after the merge, reviewed and fixed. The game moves the robe skirts and capes of every posed body with the solver (`crates/game/src/loose_parts.rs`). Hoods, sleeves, flaps and armour still play their keys. The flourishes are found by a rule on the data: a clip that turns a hood bone 90 degrees or more. `KJK_LOOSE=off` turns the solver off. The viewers' `--ghost` now fades KOTOR 1 Revan and Malak.
- L5 bench on a Mac (load average 4 to 6): `robe_acrobatics` gave 74.4, 54.4 and 58.9 mean fps with the solver off and 61.0, 56.0 and 78.5 with it on, with 1% lows of 15.1, 14.0 and 12.0 off and 15.9, 13.9 and 27.2 on. The scene's spread hides the solver. The solver took 0.092 to 0.097 ms per frame for 38 bodies and about 575 points. `crowd_brawl` has no robe the solver moves.
- Next, L6: the look. In a handstand or a cartwheel the robe and the cape still point up, away from the floor. Their target is the rest pose under the upside-down pelvis, and L2 holds a hem at most 0.45 m from it, while the hem would have to move about 2 m to fall toward the head. The KOTOR 1 Malak cloth strands splay wide.
- The user finds the solver nice but not perfect. The look gets refined after L5.
- The user watched the L5 game recordings on 2026-11-19. In the KOTOR 2 robe side view, the robe stays flared out behind for about a second after a roll (video time 00:04). During a spin it bunches into a lump at the hip and stays stuck (00:12).
- Decided on 2026-11-19 for the refinement: do three things together. A makes the solver robust: collision that remembers which side of a capsule a point was on, a backstop from the body's surface, and more substeps instead of lag in fast spins. B adds angle limits: each skirt bone swings only within a cone that moves with the thigh. The cones must be generous, as a safety net next to the collision. C adds generated extra skirt bones between the existing chains, with the skirt reskinned to them. This also gives KOTOR 1 robes their bones.
- Vertex cloth on the skirt mesh (option D) stays in reserve, perhaps only for the player's character.
- The research note on newer cloth algorithms (`2026-11-19-cloth-algorithms-sota.md`) found nothing that beats XPBD with small substeps for a few dozen bone points per character. Its one idea worth borrowing is from Offset Geometric Contact (OGC, 2025). Each point gets a bound on how far it may move per substep, so it cannot cross a capsule, and a point that ends inside goes back to its last place outside. This fits option A. The bound's formula must be taken from the full paper first.
- The research note on cheap GPU techniques (`2026-11-19-gpu-robe-techniques.md`) recommends a stateless layer in the vertex shader after the CPU bone solver. It pushes each skinned skirt vertex out of the 14 body capsules, and it adds a Velocity Skinning style drag (Rohmer et al., 2021) whose weights come from the skin weights. The drag also gives KOTOR 1 skirts some sway. Far from the camera, the solver could stop and leave only this layer. Direct Delta Mush is the second choice, only if bulging remains after the generated skirt bones. None of this is decided yet.
- The diagnosis (`2026-11-19-robe-stuck-diagnosis.md`) found that both failures come from faults in the solver, not from too few bones. First, the collision of bones with capsules throws a point metres away when the contact is near a pinned root, because it divides the push by a tiny share of the bone. Second, the 8 cm limit per substep then overrides the maximum distance and the tether, so the thrown point takes seconds to come back. Third, during Jedi Academy moves the cast keeps the model root's facing at 0 and turns the bones instead, so the solver feels a whole spin at about 900 degrees per second. When a KOTOR clip takes over, the root's facing jumps by 160 to 165 degrees in one frame, and the robe swings about 120 degrees. The robe's front piece has no link to the back chains. The recordings at 10 fps of game time made both failures far worse than at 60 fps. Extra bones (C) and cones (B) would hide these failures but not cure them.
- The diagnosis also lists why the robe looks imperfect when nothing breaks. The back chains' target hangs 31 to 33 degrees back from vertical, so the robe stands out in a stiff A-line. The damping ratio is about 0.075, so it swings for about a second. The 0.45 m maximum distance is reached in every flip and spin. The hem tests only the 6 capsules nearest at rest, so it misses the shins and sits up to 6 cm inside them.
- The fixes from the diagnosis are done on the branch `loose-parts-fix`, made from `loose-parts-l5`, in its worktree `.claude/worktrees/loose-parts-fix`. Each fix is one commit. Neither end of a bone now moves farther than the depth of a capsule contact, and a point under a pinned root keeps its bone's length. The step limit comes before the tethers and the maximum distance. The solver works in the pelvis's space, so the facing handoff between the bones and the placed node no longer swings the robe. The finder links a robe's front piece to its side chains (a new "around" rule, which changes pfbim, pfbnm, pmbim and pmbnm). A robe skirt's target hangs 60% of the way toward straight down and a cape's 50%. The damping is 6 per second on a robe and 4 on a cape. The maximum distance is 1.4 times a point's length along the chain. A point meets every part within two steps of its skin and the rest of each leg below.
- In the diagnosis runs, after the fix, there are no bone-segment pushes over 0.3 m (the largest is 0.16 m), at 10 fps and at 60 fps. Before, there were 286 and 306 at 10 fps and 21 and 6 at 60 fps, up to 129 m. The largest distance of a point from its target fell from 8 to 171 m to 1.57 m. A point now stays past its maximum distance at most 0.1 s at a time, and only where a capsule pushes it out of the body.
- The trace, the overlay in play and the side camera switch are kept as debugging aids (`KJK_LOOSE_TRACE`, `KJK_LOOSE_OVERLAY`, `KJK_LOOSE_GHOST`, `KJK_SIDE_CAM`, in `docs/RUN.md`). The other switches are removed. Every recording from now on uses 60 fps of game time.
- The recordings and the before and after chart are in `kjk-artifacts/loose-parts-fix/` in that worktree. After a flip the robe now hangs along the back leg instead of standing out in an A-line, and the spin leaves no lump. KOTOR 2 Revan's cape and KOTOR 1 Malak's strands hang instead of splaying flat.
- The fix's bench was taken while another session kept the Mac at a load average of 10 to 124, so its frame rates mean little. In an interleaved run at a load of about 10, the solver took 0.252 ms per frame for 38 bodies against 0.125 ms for L5. That is about 2.0 µs per body and substep against 1.0 µs, because each point now tests 12 capsules instead of 6. The mean fps was 34.2 with the fix and 48.8 with L5, but the two solver-off runs of the same round gave 40.6 and 34.3, so the scene's spread hides the solver. The tables are in `kjk-artifacts/loose-parts-fix/bench/`.
- Next: the user watches the recordings and refines the look. A hem reaches its 1.5 m maximum distance in the hardest flips. The ideas A (side memory, backstop from the surface), B (cones) and C (generated skirt bones) are no longer needed for the two failures, and can be weighed again for the look and for KOTOR 1 robes.

- The user watched the `loose-parts-fix` recordings on 2026-11-20. It is better, but not "wow". During the first front flip, the back leg passes through the robe, and upside down the robe passes through itself. Two tracks now run in parallel from `loose-parts-fix`. Track 1, on `loose-parts-bones2`, adds generated skirt chains, chains that keep apart and a GPU push-out of skin from the legs. Track 2, on `loose-parts-cloth`, prototypes real cloth on the skirt mesh for the player only, behind `KJK_LOOSE_CLOTH=on`. The user will compare both and choose.
- Track 2 is done on the branch `loose-parts-cloth`, in its worktree `.claude/worktrees/loose-parts-cloth`, in five commits, reviewed and fixed. The cloth solver is `crates/loose/src/cloth.rs` and the game hook is `crates/game/src/loose_cloth.rs`. On `pmbim` it finds 206 particles, of which 168 are free, and 378 triangles. The bone solver still moves the robe bones, and the cloth uses that skirt as its skinned places. The cloth is drawn through the sway list, with the previous places for the motion vectors, turned normals and the shadow tiles. It is not drawn in the shadows at Ultra. KOTOR 1 robes are not done.
- What track 2 shows (`kjk-artifacts/loose-parts-cloth/`, with the close-ups `flip-cloth-vs-fix-close-quarter.mp4` and `spin-cloth-vs-fix-close-quarter.mp4`). Through the first front flip the cloth crosses itself less than the bones' skirt (48 to 107 edge crossings a frame against 94 to 191), and it puts 2 to 3 times fewer samples into the legs' capsules. Upside down it bunches around the upper body instead of standing out as a stiff panel. After the cartwheel it tangles. From then on it crosses itself 130 to 600 times a frame, against 13 to 25 for the bones' skirt, and its back panel hangs short and bunched at the knee while standing and in the spin. The mesh is coarse (about 8 cm between vertices), so most crossings are tight accordion folds of one panel. A stronger pull toward the bones' skirt (`KJK_CLOTH_TUNE=drive=1e-4`) removes the tangles but makes the cloth little more than the bones' skirt with collision on every vertex.
- Track 2's cost. Under a load of 18 to 55 the cloth took 0.21 to 0.34 ms per substep, three quarters of it keeping out of itself, so about 1 to 1.3 ms per frame at 60 fps. The probes' skirt of 112 particles costs 0.16 ms per frame. The frame rates of the bench (7 to 16 fps in both modes) hide it.
- Next for track 2, if it is chosen: a cheaper and stronger self-collision (edge against edge, or a finer mesh for the solver only), a pull that undoes tangles without fighting gravity, and the KOTOR 1 rule. KOTOR 1 needs a second rule that finds the skirt in the legs mesh: vertices that the legs carry and that rest farther from their leg's capsule than its radius, and vertices carried by both legs. It has no robe bones, so its skinned places spread with the legs, and it needs a larger maximum distance and no pull.

- Track 1 is done on the branch `loose-parts-bones2`, made from `loose-parts-fix`, in its worktree `.claude/worktrees/loose-parts-bones2`, in six commits, reviewed and fixed. It has three parts.
  - Generated skirt chains. A skirt gets up to 10 chains of 3 bones at even angles around the pelvis, made when the model is read, while the solver runs. A chain is kept only where the skirt has cloth. A KOTOR 2 robe's skirt is the cloth of its robe bones, and it gets 7 to 10 chains. A KOTOR 1 Jedi robe's skirt is the part of the legs mesh that lies far from the legs and hangs across the middle between them. It gets 8 chains. Trousers, bulky bodies and suits get none. Every vertex rests where it rested before. The approved test `loose_bones` lists the chains of every humanoid model.
  - Chains that keep apart. Each free point keeps at least half its rest distance from the bones of every other chain of its garment, once a frame. Upside down, the hem stays a ring around the body instead of gathering into a point. Three new probes check it.
  - A push on the GPU. The vertex shader pushes each robe and cape vertex out of the six leg capsules, in the main pass, the prepass and the shadow tiles. `KJK_LOOSE_PUSH=off` turns it off.
- The recordings are in `kjk-artifacts/loose-parts-bones2/` in that worktree. In the first flip and the spin, no leg passes through the robe in the frames I looked at, and the robe no longer folds through itself upside down. The fix build shows a leg through the robe's side in the flip and a boot through the hem in the spin. The KOTOR 1 Jedi robe now flies in a flip, where it used to stay on the legs like trousers.
- The cost went up. On `robe_acrobatics` the solver took 2.8 to 4.4 ms a frame for 56 to 58 bodies and about 1,850 points, against 1.5 to 2.0 ms for 37 bodies and about 550 points with `loose-parts-fix`, in alternating runs at a load average of 20 to 130. More bodies are solved, because more models have skirts now, and each robe has about 33 points instead of 15. The GPU time of the push could not be measured, because `--gpu` lost the device in all four runs on this Mac.
- Next: the user compares track 1 with track 2 (`loose-parts-cloth`). Open points for track 1 are the solver's cost, a KOTOR 1 skirt that stretches long in a flip, the skin-once pass under Ultra (it does not push), and a few models whose generated skirt needs a look (`l_wookie`, `l_alien02`, `l_alien03`, `n_gand`, `n_quarren`, `n_dansalvager`, the Onderon commoners).

- The lead session looked at the close-up sheets of both tracks on 2026-11-20. In track 1 the legs stay inside the robe in the flip and the spin. The robe now hangs longer and straighter, almost like a heavy tube, and it trails a long tail on the flip's landing. A dark patch shaped like a knee shows on the robe's back at 13.92 s. The KOTOR 1 skirt reaches below the feet at 3.40 s. In track 2 the flip looks livelier, but after the cartwheel the robe's back is visibly shorter in the spin. Both tracks are on the overview page, track 1 first.

- On 2026-11-20 the user found the robe still lets the legs through at the very start of the track 1 recordings, and that it feels too light. Both were looked at on `loose-parts-bones2`. The results are in `kjk-artifacts/loose-parts-bones2/weight-and-start.html` in that worktree.
- The weight. The switch `KJK_LOOSE_TUNE` changes the robes' and capes' default values for one run (commit d0e81164, documented in `docs/RUN.md`). Four variants were recorded on `pmbim` and the KOTOR 1 Jedi robe, at the close and side cameras: A is the current values, B doubles the damping (12 per second on robes, 8 on capes), C adds a 4 times stiffer bend, and D is heavy (damping 14 and 10, the stiffer bend, a 3 times stronger pull toward the target pose, inertia 0.15 and 0.2, and a reach share of 1.1). On the flip's landing, A trails the robe far behind, B a little less, C falls back sooner and D hangs nearly straight down. The solver's own time does not change between them. The defaults are unchanged until the user picks. These runs were made before any foot change.
- The start. The first frames of the earlier recordings came after a jump of 1 to 2 s of game time, which put the robe back at its target pose in mid-stride, with the back leg already kicked back. Recordings now start at game time 0, so that jump is gone.
- The clip is not fixed yet. In every running stride the back boot and calf come out behind the robe for about 5 to 7 frames. The ragdoll's foot capsule sits at the ankle, above the boot, with a radius of at most 5 cm. A foot capsule longer by 7 cm over the heel (bca49b64) did not help and was reverted (5d1c3649). A foot capsule fitted to the boot's vertices, together with a push on the GPU that moves each robe vertex out along the robe's outer side, hides most of the boot. The heel still shows in about 4 frames of the first stride. The overlay shows why: at push-off the foot passes under the robe's lowest points, then rises behind the robe, and the shin crosses the back chains. A rule in the solver that puts a point found deep inside a leg back on the robe's outer side did not change this. These changes are not committed. Their patches are in `kjk-artifacts/loose-parts-bones2/start-clip/patches/`.
- The clip in the run is fixed on `loose-parts-bones2`, in two commits. The results, the videos and the zoomed frames are at the top of `kjk-artifacts/loose-parts-bones2/weight-and-start.html` in that worktree.
- The measure (85bd5c0c). Two game runs of the same moves never pose the body the same way, so the painted count could not weigh a small change. `KJK_LOOSE_RECORD` now makes the game write the skeleton that the solver reads. `kjk loose-replay` steps the solver again on such a record, without the game, and gives the same count every time. It counts the boot vertices that lie behind the robe's trailing side while the robe is in front of them and not above them. A frame counts when 3 or more boot vertices are through. Both are documented in `docs/RUN.md`.
- The cause. A robe's back hem hangs at the height of the boots, a few centimetres above the axis of the foot's capsule. At push-off the foot moves back under the hem. The solver pushed each hem point straight out from the foot's axis, so the point went up over the heel and the foot passed under the robe. The chain tips already sit at the hem's real height, so extra hem points would not help.
- The fix (863d59e7). A shin or a foot now pushes a robe point, and the bone above it, out along the robe's way out: level and away from the node the chain hangs on. This holds only while the point lies on the outer half of the capsule, and never for a bone that hangs from a pinned root. A thigh still pushes straight out, because a chain pushed along the way out was lifted over a capsule that came down on it. Each foot capsule is also fitted to the boot's vertices, with a radius of up to 10 cm. Without the fitted feet the change does little.
- The numbers. On 17 records of `pmbim` and 16 of the KOTOR 1 Jedi robe, the run frames with the boot through fall from a mean of 83.5 to 9.9 on `pmbim`, and from 72.2 to 30.5 on the KOTOR 1 robe. In 3 painted game runs of each, the run frames fall from 90 to 46 and from 128 to 104. The painted count still counts a boot under the hem's back corner, where the hem lies over the heel. The zoomed frames show that this is what remains. The flips, the cartwheel and the roll count about the same as before.
- The cost. The solver takes about 22% more time per step: 9.5 to 9.9 µs against 7.8 to 8.1 µs on `pmbim`, and 7.8 against 6.4 µs on the KOTOR 1 robe, at a load average of 8 to 17. Every probe of the approved test `loose_parts` passes, unchanged.
- Measured as no help on the replay: the GPU push along the robe's way out (dropped), a larger nine-in-ten foot fit, and the way-out push on thighs, which breaks the probe of a capsule over a pinned root.
- Next: the KOTOR 1 robe flares nearly flat behind in the run, so its back boot often shows under the hem. That is the weight question the user is to decide. On the KOTOR 1 robe the fix also adds some flicker: points that move more than 4 cm and straight back between frames, in 30 run frames of one record against 14 before. Watch the videos for it.

- The user watched the heel fix on 2026-11-20. It is better and shows no glitch, but it is still not amazing. The robe in those videos has the current weight (A).
- Weights A, C and D were recorded again with the heel fix (`kjk-artifacts/loose-parts-bones2/weight-fixed/`). The user found C broken in the jump. The robe passed through the body. This is a collision fault that C's stiffer bend brings out, not a matter of how heavy the robe looks. D has the same bend, so it may only hide the fault.
- Considered and dropped on 2026-11-20: a target pose that follows the legs, as in Unreal's and Unity's cloth, where each point may stray only a "max distance" from its skinned place. It suits a stride, but the user pointed out that in a roll or a flip the legs fold up, and the robe must then follow inertia and gravity. A target tied to the legs would wrap the robe around the folded legs like trousers. The current model stays: the robe aims for its hanging pose, and the legs only push it.
- The user's idea, built on 2026-11-20: the robe's real skin is kept out of the body, and the push goes back to the bones. It is not committed yet. It sits on top of 863d59e7 in the worktree.
  - The solver's input has samples of the skin (`loose::Sample`). They are the vertices of the skinned meshes that the solved bones carry by half or more. A vertex shared by several meshes, or that rests within a millimetre of another, is taken once (`samples` in `crates/game/src/loose_parts.rs`).
  - In every substep, after the points' own collision, each sample is placed by its bones. Each loose bone carries the sample from its animated place, turned as far as the bone has turned from its animated way. A sample inside a body capsule moves the points at both ends of its bones, by their share of the sample, as one XPBD contact. No point moves farther than the depth of the contact. A shin or a foot pushes along the robe's way out, as in the heel fix. The pass runs twice per substep, and then the bone lengths are restored (`Solver::skin_out` in `crates/loose/src/solver.rs`).
  - Each sample keeps 1 cm of cloth thickness from every capsule. A sample that rests closer than that keeps its rest distance less 0.5 cm, as the points do.
  - `KJK_LOOSE_SKIN=off` turns it off.
  - The doing on the GPU was weighed and set aside. Reading results back from the GPU arrives one or two frames late or stalls the frame.
- A second count in `kjk loose-replay` measures it. A corner of a robe triangle counts as sunk in the body when it lies more than 3 cm inside a leg capsule, or more than half the radius inside any other capsule. A frame counts when 10 or more corners are sunk. On the same records, the sunk frames fall from 120 to 35 with C on `pmbim` record 1, from 59 to 37 with A, from 68 to 36 with D on record 2, and from 124 to 98 with A on the KOTOR 1 robe. What remains on `pmbim` is in the two flips: about 5 to 8 vertices sink into the thighs while the body is upside down.
- The cost. The solver takes about 48 µs a step on `pmbim` instead of about 9.6 µs, and about 26 µs instead of 8 µs on the KOTOR 1 robe, at a load average of 17 to 45. Nothing culls the samples yet. If the cost matters, three cuts are planned. The first tests a capsule around each bone before its samples. The second tests the samples only in the last substeps. The third turns the samples off for characters far from the camera. The approved tests `loose_parts` and `loose_bones` pass. The probes have no samples.
- The user watched C without and with the samples (`kjk-artifacts/loose-parts-bones2/skin/pmbim-close-skin.mp4`) on 2026-11-20. Their verdict: the samples are "definitely better", but still not perfect.
- New recordings, all with the samples, on the page `kjk-artifacts/loose-parts-bones2/weight-and-start.html`:
  - A, B, C and D with a wide camera, `wide/pmbim-wide-ABCD.mp4`. The close camera cut the robe off at the top of the flips. The wide camera is `KJK_SIDE_CAM=0,3.9,1.5,2,50` at 960×720, and `tools/rec3.sh` records with it (view `wide`, with `CAM` and `MOVES` to override the camera and the moves).
  - A, C and D crouching and by a wall, `props/pmbim-crouch-wall-ACD.mp4`, from `tools/props.sh`. The character crouches, walks crouched, turns a full circle crouched, runs sideways into a corridor, crouches by its wall and walks along it. The corridor is dark, so the video is brightened 1.6 times. In the arena, the red ring next to the start is an open doorway, not a wall.
- The robe solver collides only with the body and the floor. A robe pressed against a wall or a prop passes through it. Collision with the level would be a separate step. It could stay cheap if only the hem points test the walls.
- Next: the user picks a weight from the wide video, the samples are committed, the thighs in the flips are looked at, and collision with walls is decided.

## Before the PR is ready for review

- Remind the user that the maintainer must say whether the approved test `loose_bones` may list model and bone names in its source. The user chose to settle this at the end.

## Risks

1. **Physics values.** Bones carry none, so the look depends on good defaults (decision 7).
2. **Look and stability during acrobatics.** The values can only be found by watching the stress scene.
3. **Robes clipping through spread legs.** A KOTOR 2 robe has only 11 bones. During a kick or a cartwheel, the skin between two bones can cut through a thigh even when the bone points stay outside the capsules. Extra generated points between chains, or a vertex backstop on the skirt, may be needed.
4. **Finding loose bones.** This is contained, because the set of models is closed and mistakes are easy to see.

## Open questions

- kjk has no wall jump or wall run. Only their animation names exist. Robes cannot be tested in those moves until the simulation has them.
- kjk's model reader skips animmesh nodes, so kjk never plays their hand-keyed vertex animation. That should be its own issue.
- Garments on non-humanoid skeletons, such as Rakata and Ithorian flaps, need a skeleton map or a data flag.
- Nobody checked how death and dismemberment scale the nodes.

## Research notes

- `2026-11-19-loose-parts-census.md`: which models have danglymeshes, robe bones and cape bones.
- `2026-11-19-original-loose-part-behaviour.md`: what the original engine does.
- `2026-11-19-robes-in-similar-games.md`: games of the same era.
- `2026-11-19-cloth-state-of-the-art.md`: recent games and engines.
- `2026-11-19-loose-part-solvers.md`: the choice of solver.
- `2026-11-19-loose-part-body-collision.md`: collision with the body.
- `2026-11-19-dangly-sway-cost.md`: the cost of the current sway.
- `2026-11-19-robe-bone-animation.md`: whether kjk plays the robe keys.
- `2026-11-19-cape-flourishes.md`: which keyed animations are flourishes.
- `2026-11-19-loose-bone-solver-prep.md`: where the solver runs, bones to points, finding loose bones and linking chains.
- `2026-11-19-robe-capsules-and-stress-scenes.md`: the capsules on robed bodies and the stress scene.
