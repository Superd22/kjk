# What the danglymesh sway costs per frame

This document was researched on 2026-11-19.
It answers one question for the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts").
How much time does the current danglymesh sway (`crates/game/src/dangly.rs`) take each frame, and how much budget is left for a richer cloth solver?

## Summary

- The sway is cheap. In the busiest scene measured, the Jekk'Jekk Tarr fight with about 120 characters, it moved 33 meshes and 3,700 vertices per frame and took 0.06 ms per frame in the quiet run. That is 1.7% of a 3.58 ms main thread frame and 1.2% of the 5.43 ms frame.
- The time per vertex is about 15 to 23 nanoseconds in the five scenes with more than 100 vertices moved per frame, and it grows in step with the number of vertices stepped. The one scene with fewer than 60 vertices per frame, Taris's Upper City Cantina, took 33 nanoseconds per vertex, which is the fixed cost of the system showing on a tiny load.
- The sway runs on the main thread, as a Bevy system in `PostUpdate` after transform propagation. It never runs on the render thread. The render thread only uploads the changed vertex places to the GPU and the vertex shader applies them.
- The runs with the sway turned off do not show a difference in fps. The machine was busy with other work during the runs, so the frame rates moved by more than 10% between repeated runs of the same build. The sway's own timer is the reliable number, and the on and off frame rates are given only for completeness.
- A cloth solver that does 10 times the vertex work would cost about 0.6 ms per frame in the Jekk'Jekk Tarr, and about 1.1 ms if the machine is as busy as it was in the second run. That fits inside a 60 fps budget of 16.7 ms, and inside the 240 fps budget of 4.2 ms only in scenes with fewer vertices than the Jekk'Jekk Tarr.
- The current sway touches few vertices, because only the meshes the games mark as danglymeshes sway, and only the ones the camera saw in the last frame. The real extra cost of a better cloth will come from more vertices, collision and substeps. Those multiply the work of every vertex, so the measured nanoseconds per vertex are the number to scale from.

## How it was measured

### The build

The build was `main` at `af1379f3`, in the worktree `/Users/david/perso/kjk/.claude/worktrees/dangly-bench` on the branch `dangly-bench`.
The worktree holds a small patch that is not committed and is not meant to be merged.

- `crates/game/src/dangly.rs` reads the environment variable `KJK_DANGLY`. When it is `off`, the `sway` system returns at once and no mesh sways.
- The same system times itself with `std::time::Instant` and counts how many frames it ran, how many danglymeshes it stepped, and how many vertices those meshes hold. The totals go into the existing `dangly` fact of each dump, as `sway_ns_sum`, `sway_frames`, `stepped_sum` and `vertices_sum`, next to the existing `meshes` count. The new field `total_vertices` is the vertex count of all danglymeshes in the world.
- `crates/launcher/src/bench.rs` copies that fact of the first and the last dump into each scene's line of `results.jsonl`.

No span existed for the sway, and a full trace is 14 to 22 GB per scene, so the timer is used instead of `--trace`.

### The runs

Each run was `kjk bench --headless` with the default preset, a 15 second warm-up and 30 seconds measured, in the default 1280x720 headless frame, on the Mac (M4 Max, Metal).
Headless means that no window opens, so nothing showed on the user's workspace.
The bench ran six scenes: `crowd_brawl`, `refugee_brawl`, `anchorhead_brawl`, `trayus_academy`, `proving_grounds` and `upper_cantina`.
The scenes were run in four whole passes, in this order: sway on, sway off, sway on again, sway off again.

The other owners were using the Mac at the same time.
The load average was 3.9 when the work began and 8.2 at the end, with a 15 minute average of 13.7.
This is why the second pass of some scenes is much slower than the first.

### What the numbers mean

- Mean fps is the frames the run counted divided by its seconds.
- p99 is the bench's worst p99 frame time among its 5 second windows. The 1% low fps is 1000 divided by that p99. It is therefore the worst window's value, which is stricter than the p99 of the whole run.
- Sway ms is the sway system's wall time per frame on the main thread, from the timer above.
- Vertices per frame counts the vertices of the danglymeshes that were stepped, which are the ones the camera saw in the last frame.

## Results

### The sway's own cost

Pass 1 was taken when the machine was quieter. Pass 2 was taken when it was busier.

| Scene | Danglymeshes in the world | Vertices in the world | Meshes stepped per frame (pass 1) | Vertices stepped per frame (pass 1) | Sway ms per frame, pass 1 | Sway ms per frame, pass 2 | Nanoseconds per vertex, pass 1 |
|---|---|---|---|---|---|---|---|
| crowd_brawl | 115 | 11,702 | 33.1 | 3,701 | 0.0625 | 0.1077 | 16.9 |
| refugee_brawl | 77 | 3,043 | 27.9 | 1,003 | 0.0187 | 0.0270 | 18.6 |
| anchorhead_brawl | 13 | 2,267 | 12.0 | 1,901 | 0.0293 | 0.0317 | 15.4 |
| trayus_academy | 9 | 253 | 5.0 | 144 | 0.0033 | 0.0038 | 22.9 |
| proving_grounds | 5 | 145 | 3.9 | 115 | 0.0028 | 0.0028 | 24.3 |
| upper_cantina | 12 | 878 | 2.0 | 54 | 0.0018 | 0.0018 | 33.3 |

Three things stand out.

- Only 28% to 100% of the danglymeshes in a scene are stepped in a frame. In the Jekk'Jekk Tarr, 33 of 115 are stepped, because the camera sees from above only part of the crowd. The sway skips every mesh the camera did not see in the last frame.
- The Malachor fights have almost no danglymeshes (9 and 5), even though they hold 70 and 59 robed Sith. Their robes are therefore skinned meshes with no sway data, which agrees with the open question in the issue.
- The cost grows with the vertices stepped. Across the first five scenes the time per vertex stays between 15 and 24 nanoseconds. It does not grow faster than linearly with the number of characters. It does not depend on the number of meshes, only on their vertices.

### Frame rates with the sway on and off

The sway is a main thread cost of at most 0.11 ms, so the frame rate differences below come from the busy machine and not from the sway.
Each pair of passes was run in the order on, off, on, off, and the frame rates swing more between two passes of the same setting than between on and off.

| Scene | Pass | Mean fps, on | 1% low fps, on | Mean fps, off | 1% low fps, off |
|---|---|---|---|---|---|
| crowd_brawl | 1 | 184.0 | 85.8 | 181.7 | 84.3 |
| crowd_brawl | 2 | 57.2 | 11.0 | 188.7 | 88.3 |
| refugee_brawl | 1 | 95.2 | 45.5 | 104.0 | 33.3 |
| refugee_brawl | 2 | 84.3 | 32.7 | 87.6 | 32.0 |
| anchorhead_brawl | 1 | 148.6 | 118.2 | 155.4 | 134.4 |
| anchorhead_brawl | 2 | 114.4 | 17.5 | 51.1 | 10.6 |
| trayus_academy | 1 | 227.5 | 244.8 | 201.1 | 188.0 |
| trayus_academy | 2 | 219.8 | 191.1 | 225.5 | 124.0 |
| proving_grounds | 1 | 334.2 | 251.7 | 346.8 | 234.4 |
| proving_grounds | 2 | 335.2 | 247.0 | 328.9 | 213.2 |
| upper_cantina | 1 | 442.3 | 333.7 | 356.4 | 153.0 |
| upper_cantina | 2 | 439.7 | 268.7 | 290.0 | 172.6 |

The sway-on run of `crowd_brawl` in pass 2 (57.2 fps, p99 91 ms, main thread 17 ms) and the sway-off run of `anchorhead_brawl` in pass 2 (51.1 fps, p99 94.8 ms) are both runs that overlapped heavy work by other owners. They are not caused by the sway.
In the same way, the sway is not the reason that the sway-on run of a scene is sometimes faster than its sway-off run.
A comparison that can resolve 0.06 ms out of 5 ms would need a quiet machine and many repeats, and the timer already gives that number directly.

The main thread mean (`main ms` of the bench) of the quieter runs is 3.58 ms in `crowd_brawl`, 4.46 ms in `refugee_brawl` and 3.96 ms in `anchorhead_brawl`.
Against those, the sway's 0.06, 0.02 and 0.03 ms are 1.7%, 0.4% and 0.7%.

## Where the sway runs

- The spring is stepped on the CPU, in the system `sway` in `PostUpdate`, after transform propagation. It is on the main app's schedule, so it shares the main thread's frame with the simulation handoff, posing and drawing.
- The step thread (`KJK_STEP_BATCH`) is not involved, because the sway is view-only and does not touch the simulation.
- Each stepped mesh writes its vertices into `SwaySlots`. The extract step copies the changed range into a copy in the render world, and `write_sway` on the render thread sends it to the GPU with one `queue.write_buffer`. Each slot is 24 bytes, so the 3,700 vertices of the Jekk'Jekk Tarr are about 89 KB per frame.
- The shader (`sway.wgsl`) moves the vertices on the GPU, so the mesh itself is not rewritten on the CPU. The issue's description that the sway "rewrites each moving mesh" is true of the slot data and not of the mesh asset.
- The timer measures only the `sway` system. It does not include the copy in the extract step or the GPU upload. The upload size is already in the existing `uploaded_bytes` fact, which this study did not read.

## Budget for a richer cloth solver

Take the sway's cost as 17 nanoseconds per vertex in a quiet run and about 30 nanoseconds in a busy one, which are the Jekk'Jekk Tarr's numbers for pass 1 and pass 2.

| Work | Vertices per frame | Time in a quiet run | Time in a busy run |
|---|---|---|---|
| Today's sway, Jekk'Jekk Tarr | 3,700 | 0.06 ms | 0.11 ms |
| 10 times the vertex work | 37,000 | 0.63 ms | 1.1 ms |
| 10 times the vertex work and 4 substeps | 37,000 steps, 148,000 vertex updates | 2.5 ms | 4.4 ms |

How this compares to a frame:

- At 60 fps a frame is 16.7 ms, so even the last row is below 27% of it.
- At 144 fps a frame is 6.9 ms. The 10 times row takes 9% to 16% of it, and the substep row takes 36% to 64%.
- The Jekk'Jekk Tarr already runs at about 184 fps (5.4 ms) with a main thread of 3.6 ms, so the substep row would become the main thread's largest single system in that scene.
- The estimate for collision is not measured. A collision against the 14 capsules of each character would add a per-vertex cost that these numbers do not show.

The three cheapest ways to keep the cost low are these.

- Keep skipping meshes the camera did not see, which today removes 28% to 72% of the vertices in the crowd scenes.
- Run the solver on the GPU in a compute pass, as the skinning of shadow tiles (`KJK_SKIN_ONCE`) already does. The slot buffer and the vertex shader already exist.
- Keep a setting that turns the richer solver off, with the original sway as the default, as the issue suggests.

## Limits of this study

- The machine was shared and busy, so the frame rates carry noise of more than 10%. The sway's timer was taken inside the system and is less affected, but it still reads higher in the second pass (0.108 ms against 0.063 ms in the Jekk'Jekk Tarr).
- The bench runs headless at 1280x720. The simulation and the sway do not depend on the frame size, but the GPU times do.
- No scene was measured in the Dantooine Jedi Enclave, because no bench scene exists for it. The scenes with the most robed characters (the two Malachor fights) hold almost no danglymesh data, so they do not stress the sway.
- The study did not measure the PC (RTX 4080, i7-8700K). A slower CPU would scale the sway's milliseconds roughly with its single thread speed.
- The timer sums wall time, which includes any time the main thread spends descheduled during the system.
- The counts of vertices stepped use the whole mesh each time a mesh is stepped. They do not count only the vertices whose constraint is above 0.
