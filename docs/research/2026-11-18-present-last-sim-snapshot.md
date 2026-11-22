# Presenting the last finished sim snapshot

This document was researched on 2026-11-18.
It answers one question.
Should kjk's main thread draw the last snapshot the step thread has finished, instead of waiting for the step thread to finish the ticks it was handed?

The document has three parts.
The first part describes what kjk does today, read from its code.
The second part gives what shipped engines and games do, from their own documentation, source code and talks.
The third part is a recommendation for kjk.
The recommendation is kept apart from the findings, and every claim in it is marked as an inference unless it cites a source.

Every claim in the first two parts carries its source.
For a web page that is the URL it was read on.
For source code it is the repository, the path, and a commit or tag.
For a talk on video it is the video URL and a timestamp.
The video quotes come from YouTube's automatic captions, so they are close paraphrases, not exact words.
The section "Sources that could not be reached" lists what was not read.

## Summary

- kjk already decouples rendering from its 125 Hz simulation. The view blends the two newest snapshots, and the step thread steps the ticks while the main thread draws. The main thread waits only at the start of the next frame, in `Authority::settle`, for the ticks it handed over in the frame before.
- Every engine and game read for this document keeps the simulation on a fixed step and draws an interpolated state that is up to one step old. Unity and Fiedler's "Fix Your Timestep!" say so directly. Bevy's own fixed-timestep example does the same.
- The engines that put the simulation and the renderer on separate threads (Unreal, Bungie's Halo and Destiny, Bevy's pipelined rendering) let the renderer work on frame N while the game works on frame N+1. They still hand over at a fixed point each frame, and the game thread blocks there if the other side is late. None of the sources describes a renderer that skips a simulation result it is waiting for.
- The engines with client prediction (Source, Quake 3, Overwatch, Rocket League, Unity Netcode for Entities, Unreal's networked physics) tag every input with the tick it belongs to and keep a buffer of inputs. The buffer exists so that the simulation can consume each input at its tick, whenever it actually runs.
- kjk already tags each command with its tick (`sim_worker::Order`). The tick a key press lands on is decided by the wall clock in the frame that reads the keys, not by when the step thread runs. So presenting without waiting would not change which tick an input lands on. It would only change how soon the result is drawn in the frames where the step thread is late. This is an inference from kjk's code.
- The recommendation is to stop waiting, but to do it as a queue with a cap, not as a free-running thread. The step thread would keep the world, take orders from a queue of a few ticks, and send back each tick's snapshot as soon as it is stepped. The main thread would draw the newest ticks that have arrived, and would wait only when the queue is full.

## What kjk does today

This part was read on the branch `uncapped-headless` at commit `0600adc3`, which contains the commit that made the step thread the default (`3fece16e`, "Step thread on by default").
The `main` branch at `1c055edb` still has the step thread off by default.
The line numbers below are those of `crates/game/src/sim_host.rs` at `0600adc3`.

### The tick clock

The simulation runs at 125 ticks a second, so one tick is 8 ms (docs/SIM.md, "Pieces").
`sim::Accumulator` turns wall time into due ticks (`crates/sim/src/rate.rs:77`).
It keeps the time that is left over, and `alpha()` gives how far the clock is into the next tick, from 0 to 1 (`crates/sim/src/rate.rs:100`).
A frame steps at most `MAX_CATCH_UP` ticks, which is 8. Wall time beyond that is thrown away and counted in `dropped()` (`crates/sim/src/rate.rs:86`).

### When the input is read and how it reaches the sim

The system `advance` runs in Bevy's `PreUpdate` schedule, after the input systems, the pad sampling, picking, the action bar and the power wheel (`sim_host.rs:787` to `805`).
It reads the player's keys and mouse once per frame through `PlayerInput::read` (`sim_host.rs:1027`).
It then asks the accumulator for the ticks that are due (`sim_host.rs:2355`).
For each due tick it makes a command from the same keys (`sim_host.rs:2481` to `2580`).
So every tick in one frame gets the same keys, as docs/SIM.md says in "The step thread".

A frame hands all its ticks to the step thread when that is allowed (`batched`, `sim_host.rs:2468`).
It is not allowed when a pilot or a minigame bot drives, when a break holds, when a transition failed to plan, or when the game is a host that other players can join (`sim_host.rs:2448` to `2477`, the comment above `batched`).
In those cases the frame steps all but its last tick itself, and only the last tick goes to the step thread.

Each handed-over tick is an `Order` that carries its tick number, its input, its length in milliseconds and what to prepare beside the snapshot (`crates/game/src/sim_worker.rs`, `struct Order`).
`Stepper::send` moves the whole world into a job on the thread, together with the list of orders (`sim_worker.rs`, `fn send`).
The thread steps the orders in order. It stops after a tick that holds the world, which is a tick that calls a break or asks for a transition, and it returns the inputs it did not step (`sim_worker.rs`, `fn ends` and `Stepper::new`).

### Why the main thread waits

The world is a single value, and it is either at home on the main thread or away on the step thread (`sim_worker.rs`, `struct WorldSlot`).
The step thread returns the world and the stepped ticks together, in one message, when the whole job is done.
At the start of the next frame, `advance` calls `Authority::settle` (`sim_host.rs:2003`, defined at `1411`).
`settle` calls `Stepper::land`, which blocks on the channel with `recv()` until the job comes back (`sim_worker.rs:206` to `212`).

The main thread needs the job back for these reasons, all read in `advance` and `settle`:

- The world itself must be home before the next job can be sent, because there is only one world and a job takes it whole.
- Each landed tick is recorded, told to the session's server and client (`solo.stepped`), and its notices become screens, sounds and explosions (`sim_host.rs:1416` to `1433` and `2017` to `2050`).
- The next frame's commands read `authority.last`, the newest snapshot. One example is the shared Block button, which is decided by what the character holds (`InHand::of`, `sim_host.rs:2544`). Another is `commands_for`, which reads whether a break holds.
- The next frame's commands also read the world's level (`authority.world.level()`, `sim_host.rs:2480`), which needs the world at home.
- A tick that held the world cuts the job short. The clock then goes back to that tick, and the inputs not stepped go back to the session (`Solo::unstepped`).

### What the view shows

`PresentedSnapshot` holds the two newest snapshots and a blend factor (`sim_host.rs`, `struct PresentedSnapshot`).
The views draw from it, and they cannot reach the world itself (docs/SIM.md, "Render reads no authority state").
When a frame batched its ticks, it shows the ticks of the frame before, blended by the alpha that the clock had at the end of that frame (`authority.before`, `sim_host.rs:2748` to `2768`).
docs/SIM.md says that the player sees "the time of the frame before, less one tick". That is 8 ms plus one frame behind the clock.

Bevy's pipelined rendering is on, because kjk adds `DefaultPlugins` and does not disable `PipelinedRenderingPlugin` (`crates/game/src/runtime.rs:154` to `173`).
So the render thread draws frame N while the main thread runs frame N+1 (see "Bevy" below).

### The net code

A guest in LAN co-op predicts its own seat.
Its new ticks also go to a thread beside the frame, and land at the start of the next frame (docs/NET.md, "Processes").
That landing also blocks: `Beside::land` and `rebase::Rebaser::land` call `recv()` (`crates/net/src/session/rebase.rs:281` to `300`).
A rewind, however, does not block. `Rebaser::take` uses `try_recv()` and returns nothing when the rewind is not finished, so the rewind lands at a later frame (`crates/net/src/session/rebase.rs:385` to `400`).
The approved test `net_rewinds` checks that a client rewinding on a thread beside its frames never shows its own seat going back in time (docs/TESTS.md).
So kjk already has one case where a frame draws without waiting for a thread's result, and a test that checks the view stays monotonic.

### The measured cost of the wait

The task that asked for this document gives these numbers, measured under tracing.
The wait in `settle` costs 1.6 ms a frame on average in one big fight and 3.4 ms in another.
Most frames wait about 0.09 ms, but about one frame in ten waits 9 ms or more.
The step thread is busy 66% to 77% of the time in those fights.
These numbers were not measured again for this document.

## How engines decouple rendering from a fixed-step simulation

### Glenn Fiedler's "Fix Your Timestep!"

The article steps the physics with a fixed `dt` and carries the time left over to the next frame in an accumulator: "This left over time is passed on to the next frame via the accumulator variable and is not thrown away." (https://gafferongames.com/post/fix_your_timestep/)
Its goal is "a fixed delta time value for the simulation plus the ability to render at different framerates" (same page, "Free the physics").
It blends the previous and the current physics state by `alpha = accumulator / dt`, with `State state = currentState * alpha + previousState * ( 1.0 - alpha );` (same page, "The final touch").
Because the blend is between the previous and the current state, the drawn state is up to one step behind the newest one. The page does not say this in one sentence, but it follows from its code.
The page names the "spiral of death", which "is what happens when your physics simulation can't keep up with the steps it's asked to take". Its guard is to "clamp at a maximum # of steps per-frame and the simulation will appear to slow down under heavy load" (same page).

Fiedler's "Snapshot Interpolation" applies the same idea to the network.
It keeps snapshots "for a short amount of time in an interpolation buffer", with enough delay to survive two lost packets (https://gafferongames.com/post/snapshot_interpolation/).
It says that "we've traded a small amount of added latency for smoothness".
It calls extrapolation a way to hide that delay, but says that "extrapolation doesn't work very well for rigid bodies because their motion is non-linear and unpredictable" (same page).

His "Deterministic Lockstep" samples the input "at the beginning of each simulation frame" and simulates frame n only "when it has the input for that frame" (https://gafferongames.com/post/deterministic_lockstep/).
It uses a "playout delay buffer" to make inputs arrive at a steady rate (same page).

### Bevy

All Bevy quotes below were read in Bevy 0.19.1, the version kjk uses, at tag `v0.19.1` (commit `b56fc29d3016e641754765244b5ba3f9cc504671`, https://github.com/bevyengine/bevy/tree/v0.19.1).

`Time<Fixed>` runs the `FixedMain` schedules "a number of times between PreUpdate and Update according to the accumulated overstep() time divided by the timestep()", which "means the schedule may run 0, 1 or more times during a single update" (`crates/bevy_time/src/fixed.rs`, doc comment of `struct Fixed`).
`overstep_fraction()` gives the time left over as a fraction of a step, which is Fiedler's alpha (`crates/bevy_time/src/fixed.rs`).
`run_fixed_main_schedule` runs the fixed schedule on the main thread, in a loop, while the accumulator has a whole step (`crates/bevy_time/src/fixed.rs`, `fn run_fixed_main_schedule`).
The default step is 64 Hz and not 60 Hz. The doc comment explains that 60 Hz "has the potential for a pathological interaction with the monitor refresh rate where the game alternates between running two fixed timesteps and zero fixed timesteps per frame" (`crates/bevy_time/src/fixed.rs`).
Bevy's guard against the spiral of death is `Time<Virtual>::max_delta`, 250 ms by default. Its doc comment says that without it "the game would enter a 'death spiral' where computing each frame takes longer and longer", and that with it the game "will run slower than real time, but it will not freeze" (`crates/bevy_time/src/virt.rs`).

Bevy's example `examples/movement/physics_in_fixed_timestep.rs` shows the pattern Bevy recommends (https://github.com/bevyengine/bevy/blob/v0.19.1/examples/movement/physics_in_fixed_timestep.rs).
It stores "the player's inputs since the last physics update" in an `AccumulatedInput` component, read every frame before the fixed loop.
The fixed step consumes that accumulated input.
After the fixed loop, a system interpolates the drawn transform "between the player's previous and current position".
The example says that "every visual frame is now slightly lagging behind the actual physical frame, but in return, the player's movement will appear smooth".
It also says that the camera's rotation should update every frame before the fixed step, because in 3D games "the camera rotation often determines which direction the player moves".

Bevy's `PipelinedRenderingPlugin` "moves rendering into a different thread, so that the Nth frame's rendering can be run at the same time as the N + 1 frame's simulation" (`crates/bevy_render/src/pipelined_rendering.rs`, doc comment).
The hand-over between the two threads uses channels of capacity one in each direction (`async_channel::bounded::<SubApp>(1)`), and the main app's extract step waits until the render world comes back (`renderer_extract`, same file).
So the main thread blocks at extract when the render thread is late.

### Unity

Unity's fixed update runs "at a fixed time interval". When the frame rate is higher than the fixed rate, "each frame has one fixed update or none at all", and when it is lower, "each frame has one or more fixed updates to allow the physics simulation to catch up with the backlog" (https://docs.unity.com/en-us/engine/current/manual/scripting/managing-time-and-frame-rate/fixed-updates).
`Time.maximumDeltaTime` "Bounds the maximum number of times Unity executes MonoBehaviour.FixedUpdate() in a frame". The reason given is that "a long frame causes multiple executions of FixedUpdate phases in the following frame, which causes another long frame and so on" (https://docs.unity.com/en-us/engine/current/script-reference/unityengine/time/maximumdeltatime).
Rigidbody interpolation "delays the Rigidbody's pose by one physics update, so that it has two points to use for its calculation". The page says that "Interpolation is more accurate than extrapolation, but it has a time lag of one physics update", and that extrapolation "might visibly overshoot collision bounds" (https://docs.unity.com/en-us/engine/current/manual/physics-section/physics-overview/rigidbody/rigidbody-interpolation).
Unity's Input System can process input events either before every `Update` or before every `FixedUpdate`. In the first mode, "Querying input state in FixedUpdate will result in errors being logged" (https://docs.unity3d.com/Packages/com.unity.inputsystem@1.8/api/UnityEngine.InputSystem.InputSettings.UpdateMode.html).

Unity's Netcode for Entities names two timelines on the client.
The "prediction tick" is "the tick the client estimates the server will apply the commands on" (https://docs.unity3d.com/Packages/com.unity.netcode@1.5/manual/time-synchronization.html).
The "interpolation tick" shows other objects "in a state it has received data for" (same page).
The client adjusts its clock so that the server "receives commands just before it needs them", with a setting `TargetCommandSlack` in ticks (same page).
The interpolation buffer's size is `ClientTickRate.InterpolationTimeMS`, and the interpolated timeline "runs behind the server authoritative simulation timeline" (https://docs.unity3d.com/Packages/com.unity.netcode@1.8/manual/interpolation.html).
Each tick's input is stored in a command buffer, and a system will "Retrieve inputs from the command buffer for the current tick" (https://docs.unity3d.com/Packages/com.unity.netcode@1.5/manual/command-stream.html).

### Unreal Engine

"In Unreal Engine, the entire renderer operates in its own thread that is a frame or two behind the game thread" (https://dev.epicgames.com/documentation/unreal-engine/threaded-rendering-in-unreal-engine).
"The game thread blocks at the end of each Tick() until the rendering thread catches up", within that limit of one or two frames (same page).
"FlushRenderingCommands is the standard method of blocking the game thread until the rendering thread has caught up" (same page).
Epic describes Unreal Engine 4 as "normally configured as a 'single frame behind' renderer", where "the RenderThread is processing Frame N while the GameThread processes Frame N+1" (https://dev.epicgames.com/documentation/unreal-engine/parallel-rendering-overview-for-unreal-engine).

Unreal's physics settings offer "Tick Physics Async", which "Defines whether to tick physics simulation on an async thread", with an "Async Fixed Time Step Size". Epic marks this as experimental (https://dev.epicgames.com/documentation/unreal-engine/physics-settings-in-the-unreal-engine-project-settings).
Physics sub-stepping runs "in a separate physics thread", and "some calls to the physics engine have to be interpolated or maintained" (https://dev.epicgames.com/documentation/unreal-engine/physics-sub-stepping-in-unreal-engine).
In networked physics with resimulation, the client is "half a Round Trip Time (RTT) ahead of the server". It compares the server's state with "the cached physics state in its history for the corresponding physics frame", and after a correction it "interpolates the rendering of the object from its current state to the new state" (https://dev.epicgames.com/documentation/en-us/unreal-engine/networked-physics-overview).
No primary source was found that describes how Unreal passes input from the game thread to the async physics thread.

### Valve's Source engine

Source simulates 66.67 ticks a second by default, with a 15 ms step (https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking, read through a 2024 Wayback Machine copy).
"The client creates user commands from sampling input devices with the same tick rate that the server is running with", and sends several commands per packet (same page).
The client draws other entities 100 ms in the past by default: "The rendering time is the current client time 10.32 minus the view interpolation delay of 0.1 seconds" (same page).
The delay is `max( cl_interp, cl_interp_ratio / cl_updaterate )`, so that "even if one snapshot is lost, there are always two valid snapshots to interpolate between" (same page).
The local player is predicted: "the client runs exactly the same code and rules the server will use to process the user commands" (same page).
A prediction error is smoothed over `cl_smoothtime` (same page).

Yahn Bernier's paper on the earlier Half-Life engine gives an example of the command buffer: "If the client is running at 50 frames per second (fps) and has 100 milliseconds of latency (roundtrip), then the client will have stored up five user commands ahead of the last one acknowledged by the server" (https://developer.valvesoftware.com/wiki/Latency_Compensating_Methods_in_Client/Server_In-game_Protocol_Design_and_Optimization, read through the Wayback Machine).
The client "only plays effects if the user command is being run for the first time" (same page).

### id Tech 3 and OpenJK

In Quake III Arena, the client makes one command per frame. `CL_CreateNewCommands` says "Create a new usercmd_t structure for this frame" (id-Software/Quake-III-Arena, `code/client/cl_input.c`, commit `dbe4ddb10315479fc00086f08e25d968b4b43c49`).
It caps the frame's time at 200 ms, "to prevent unexpected moves after a hitch" (same file).
Each command carries the server time, "so the amount of movement can be determined without allowing cheating" (same file, `CL_FinishMove`).
The client's clock is kept so that "time will never flow backwards", and its ideal is "to have the adjusted time approach, but not pass, the very latest snapshot" (`code/client/cl_cgame.c`, `CL_SetCGameTime` and `CL_AdjustTimeDelta`).
The client predicts its own player by running "all unacknowledged commands each time, not just the new ones", and prediction errors are "decayed off over several frames" (`code/cgame/cg_predict.c`, `CG_PredictPlayerState`).
The server runs at `sv_fps` 20 with its own accumulator (`code/server/sv_init.c:596`, `code/server/sv_main.c:775` to `833`).

OpenJK, the source of Jedi Academy's engine, has `pmove_fixed` off and `pmove_msec` at 8 by default (JACoders/OpenJK, `codemp/game/g_xcvar.h:175` to `177`, commit `260c59c2907187af555a676fe0cc798893bf7757`).
With `pmove_fixed` on, the server rounds each command's time up to a multiple of `pmove_msec` (`codemp/game/g_active.c:2031` to `2040`).
`Pmove` chops a long command into steps "to prevent framerate dependent behavior", and each step is at most `pmove_msec` with `pmove_fixed` on (`codemp/game/bg_pmove.c:11172` to `11220`).
The server game runs a client's command when it arrives, not on the server's tick. `ClientThink_real` "will be called once for each client frame" (`codemp/game/g_active.c`).
kjk's 8 ms tick comes from this default (docs/SIM.md, "Pieces").

### Overwatch

Timothy Ford's GDC 2017 talk "Overwatch Gameplay Architecture and Netcode" was read from its upload on YouTube (https://www.youtube.com/watch?v=W3aieHjyNvw).
At 24:26 he says that time "is quantized into what we call command frames. Each command frame lasts a fixed 16 milliseconds."
At 24:57 he says that tournament servers use a fixed 7 ms, and that clock time becomes command frames through "an accumulator with rollover".
At 25:27 he says that "the client's clock is always ahead of the server by half RTT plus one buffered command frame".
At 26:28 he says "you want to keep this buffer as small as you possibly can".
At 29:00 he says that if the server's input buffer runs dry, the server repeats the last input.
At 30:00 he says that after a loss the client "start[s] to simulate slightly faster", treating a 16 ms step as 15.2 ms, and at 31:01 that it slows back down when the link is healthy.
At 51:07 he says that each player's input is kept in "a ring buffer of all of your inputs for the last two seconds".
At 57:23 he says that "it's a fixed simulation, you must run at 60. You don't have to render at 60", and that a client rendering at 30 Hz runs two simulation steps per frame.

### Rocket League

Jared Cone's GDC 2018 talk "It IS Rocket Science! The Physics of Rocket League Detailed" was read from its upload on YouTube (https://www.youtube.com/watch?v=ueEmiDM94IE).
At 5:07 he says that physics frames "are always working with the same delta time no matter how fast the game is rendering", at a "120 Hertz tick rate so each frame is approximately 8 milliseconds".
At 23:35 he says that adding input delay "is not an option" for them, so they predict.
At 32:48 he says that the client records its input for every physics frame and sends it to the server, which puts it into a buffer and takes one item per frame. At 33:50 he says that this buffer "increases average latency".
At 32:18 he says that clients "predict everything", their own car, the other cars and the ball.
At 37:56 he says that the client keeps each physics frame's input and frame number, and on a correction rewinds and runs the physics frames again.
At 46:44 he says "we don't use extrapolation".

### Bungie

David Aldridge's GDC 2011 talk "I Shot You First" on Halo: Reach is on GDC's YouTube channel (https://www.youtube.com/watch?v=h47zZrqjgLc).
At 6:55 he rejects lockstep because its input latency "includes the network round-trip time".

Natalya Tatarchuk's GDC 2015 slides on Destiny's renderer describe Halo's pipeline: "As we go through Game Tick N, simulation computes game tick for frame N, while we are rendering the previous frame. Once the simulation has finished we copy the gamestate to start rendering this frame during the next gametick." (https://advances.realtimerendering.com/destiny/gdc_2015/Tatarchuk_GDC_2015__Destiny_Renderer_web.pdf, pages 14 and 15).
Destiny extracts only what is visible into a "frame packet" each frame (page 41).
The slides say that "extract phase is required to complete before we can unblock the next frame's game tick, thus increasing game simulation latency", so "we have to run extract as quickly as possible" (pages 91 to 93).
So in Destiny the simulation waits for the renderer's extract, and the renderer works on a copy one frame behind.

### Fighting games and rollback

GGPO's README says that "Rollback networking uses input prediction and speculative execution to send player inputs to the game immediately" (https://github.com/pond3r/ggpo/blob/master/README.md, commit `7ddadef8546a7d99ff0b3530c6056bc8ee4b9c0a`).
Its developer guide asks for a game that can "load, save, and execute a single simulation frame without rendering the result of that frame" (https://github.com/pond3r/ggpo/blob/master/doc/DeveloperGuide.md).
It asks the game to "Make your game state advanced by a fixed time quanta per frame, even if your render loop does not", and to "Separate Updating Game State from Rendering" (same page).
It says that "GGPO uses both frame delay and speculative execution to hide latency", and that "any frame delay larger than 1 can be noticed by most intermediate players" of a fighting game (same page).
GGPO predicts at most 8 frames ahead (`GGPO_MAX_PREDICTION_FRAMES 8` in `src/include/ggponet.h`) and tells a client that runs too far ahead to slow down (`GGPO_EVENTCODE_TIMESYNC`, same header).

Michael Stallone's GDC 2018 talk on rollback in Mortal Kombat and Injustice 2 was read from its upload on YouTube (https://www.youtube.com/watch?v=7jb0FOcImdg).
At 3:11 he says the games run a hard 60 Hz, and that 16.66 ms "is the game tick, that is our render tick". Those games do not decouple rendering from the simulation.
At 13:03 he says they support "ten frames of latency", made of "three frames of static input latency plus the seven frames of rollback".
At 9:57 he says that the worst case is "eight total frames simulated in sixteen point six six milliseconds".

## What the sources say about the questions asked

### Rendering the last finished state

Every engine read here draws a state that is interpolated between the two newest simulation steps, so the drawn state is up to one step old (Fiedler, Unity, Bevy's example, Source, Quake 3).
Unity puts a number on it: interpolation "has a time lag of one physics update".
Engines that run the renderer on its own thread draw frame N while the game computes frame N+1 (Unreal, Bevy, Bungie).
In all three, the game side blocks at the hand-over when the other side is late. Unreal's game thread "blocks at the end of each Tick() until the rendering thread catches up". Bevy's extract waits for the render world. Destiny's next game tick waits for extract.
No source read here describes a renderer that draws an older simulation result rather than wait for a newer one that is due.
Source and Unity Netcode do draw other players from a buffer with a fixed delay (100 ms in Source), so the drawn state is often several ticks old by design. That delay is chosen to survive network loss, not to avoid a thread wait.

### Input latency and responsiveness

The engines with prediction keep the local player responsive by tagging each input with its tick and buffering it, so the simulation can consume it at that tick (Source, Quake 3, Overwatch, Rocket League, Unity Netcode).
The buffers are kept as small as possible. Overwatch adjusts the client's step length to keep the server's buffer near one frame, and Rocket League says the buffer "increases average latency".
Bevy's example accumulates input between fixed steps and lets the camera's rotation update every frame, outside the fixed step.
Fighting games add a fixed input delay of a few frames and hide the rest with rollback (GGPO, Mortal Kombat).

### Determinism and rollback beside a render thread

GGPO asks for a simulation that can run a step without rendering it, and for rendering to be separate from the game state.
Rocket League rewinds and runs physics frames again on a correction, and Mortal Kombat runs up to eight simulation frames per render frame.
In all of these the simulation consumes inputs by frame number, so when the renderer draws has no effect on the simulation's result.

### Failure modes

The spiral of death is the main failure named by Fiedler, Bevy and Unity. All three cap the work per frame and let the game slow down instead.
Bevy warns that a step rate close to the display rate can make frames alternate between two steps and zero steps.
Quake 3 caps a frame's time at 200 ms after a hitch, and keeps its clock from flowing backwards.
Fiedler warns that too short an interpolation buffer makes the game hitch each time a packet is lost.

## Recommendation for kjk

Everything in this part is an inference by the author of this document, unless it cites a source above.

### Presenting without waiting does not change which tick an input lands on

In kjk, the tick a key press lands on is decided in the frame that reads the keys. That frame asks the clock which ticks are due and gives them all the frame's keys (see "When the input is read").
Each order carries its tick number.
So the step thread could run the order late, and the key press would still land on the same tick, with the same result.
This is the buffering that Overwatch, Rocket League and Unity Netcode do. kjk already has it, in `sim_worker::Order`.
What it lacks is room for more than one job: today the world must come home before the next job can leave.
So input buffering in a new form is not needed. A queue of orders, deeper than one job, is.

### What not waiting would cost

Today, in a frame where the step thread is late, the main thread waits about 9 ms and then shows the newest ticks.
Without the wait, that frame would start on time and show the ticks it already has, which are one frame older.
The newest ticks would be drawn in the next frame, about 8 to 10 ms later.
So the result of an input would be drawn at about the same time either way in those frames. The difference is that the main thread does useful work during those 9 ms instead of idling.
In the nine frames out of ten where the step thread is already done, nothing changes.
At 125 Hz and a main thread of 8 to 10 ms, the added delay of the drawn picture is therefore zero in most frames and at most one frame, about 10 ms, in a late frame. Averaged over frames, it would be about 1 ms, which is close to the 1.6 to 3.4 ms the wait costs today.
The real cost is smoothness. A frame that shows no new tick while ticks were due looks like a small hitch, unless the view clock hides it (see below).

### A likely cause of the one-in-ten waits

At a frame rate below 125, some frames have two due ticks and the others one.
The share of frames with two ticks is (125 − f) / f for a frame rate f. At about 114 frames a second, that is one frame in ten.
A frame with two ticks hands the step thread twice the work, and with the thread 66% to 77% busy on average, two ticks can take longer than one frame.
This is the beat between step rate and frame rate that Bevy's doc comment warns about for 60 Hz.
This should be checked by logging, for each frame, the number of ticks in the job beside the time `settle` waited.
If the check holds, a queue fixes most of it, because the thread's average load is below 100% and it can catch up during the next frame.

### The proposed design

1. The step thread keeps the world. It no longer sends the world home with each job.
2. The main thread sends orders to the step thread through a queue. Each order carries its tick, its input and its milliseconds, as today.
3. The step thread sends back each tick's `Stepped` as soon as that tick is stepped, not at the end of the job. This lets a frame use the first tick of a two-tick job when the second is not finished.
4. At the start of a frame, `settle` drains the ticks that have arrived with `try_recv`, as `Rebaser::take` already does for rewinds. It records them, tells the session of them and raises their notices, in order, as today.
5. The main thread waits only when the queue holds more than a cap of unstepped ticks. A cap of 8 ticks, the same as `MAX_CATCH_UP`, keeps the spiral of death bounded, as Fiedler, Bevy and Unity do. When the cap is reached, the frame waits as it does today.
6. Reads that need the world itself, which are a save, a dump and the end of the session, send a request through the queue and wait for the answer. This is what `settle` already does for them.
7. The level the commands read (`authority.world.level()`) is kept on the main thread as a shared copy, since the world is no longer home.
8. A tick that holds the world (a break or a transition) makes the step thread drop the orders behind it and return their inputs, as `ends` does today. The main thread must then ignore any ticks it queued after that tick. A generation number on each order would do this, as the guest's rewinds already use (`stepped.generation`, `crates/net/src/session/client.rs`).

### The view clock

Today the alpha comes from the clock of the frame that handed the ticks over (`authority.before`).
Without the wait, the newest tick that arrived may be older than that.
The view should have its own clock, the wall clock minus one frame and one tick, as today.
Each frame it would blend the two arrived ticks around that time.
When the needed tick has not arrived, the view should hold at the newest tick (alpha 1) and not extrapolate, because Fiedler and Unity both say extrapolation overshoots.
To avoid repeated holds, the delay could grow a little when holds happen and shrink when they stop, the way Overwatch adjusts its input buffer. This should come only if the measurements below show repeated holds.
The view's time must never go backwards, which the `net_rewinds` test already checks for a guest's own seat.

### Interaction with LAN co-op and rollback

The simulation's result depends only on the inputs and their ticks (docs/SIM.md, "Rules every system keeps"). So presenting without waiting does not touch determinism, the recordings or the hashes, as long as the orders keep their order and their ticks.
Two inputs read the newest snapshot on the main thread: the shared Block button (`InHand::of`) and the walk to a target (`target_view::steer`). With a late step thread, they would read a snapshot one frame older. These inputs are recorded with the tick, so replays stay exact, but the choice made in play could differ in the rare frame where the held item changed in the frame before.
A host that other players can join does not batch today, because a guest's command must be taken when the tick's input is made (docs/SIM.md, "The step thread"). The proposal should leave the host as it is at first.
A guest's own new ticks land with a blocking `recv()` too (`Rebaser::land`). The same queue could serve them later, since the guest's rewinds already land without waiting.

### What to measure

- The time `settle` waits per frame, as traced today.
- The number of ticks in each job, beside that wait.
- The age of the drawn picture. This is the wall clock minus the time of the newest tick drawn.
- The number of frames that drew no new tick while ticks were due. These are the hitches the view clock must hide.
- The depth of the order queue, and how often the cap is hit.
- The step thread's busy share, as today.
- A blind play test like the one of 2026-11-18, in the same fight with the change on and off.

### Risks

- The order of the notices, the recording and the session's `stepped` calls must stay tick by tick. A tick that arrives late must still be told before the next one.
- A break or a transition must cut the queue cleanly, or ticks after a break could be stepped.
- A slow step thread could hide behind the queue and the cap for a long time. The game would then run behind the wall clock, and `dropped()` would no longer show it, unless the cap counts it.
- The co-op host and guest keep their blocking paths at first, so two code paths remain.

## Sources that could not be reached

- developer.valvesoftware.com refuses automated fetches (HTTP 403). Both Valve pages were read through Wayback Machine copies from 2024 of the same URLs.
- The Unreal Engine source code is on GitHub only for accounts linked to Epic, so it was not read. The console variable `r.OneFrameThreadLag` and the way Chaos passes input to its async physics thread are not confirmed by any primary source here.
- The Unreal API pages for `UAsyncPhysicsInputComponent` and `FNetworkPredictionSettings` returned no text.
- Unity's manual page "Handling variation in time" returned HTTP 404.
- The GDC Vault videos were not used. The talks were read from automatic captions of GDC's own YouTube uploads. The channel of the Overwatch, Rocket League and Mortal Kombat videos was not checked, because the transcript tool's quota ran out. The Halo: Reach video was confirmed to be on GDC's channel.
- Rocket League's input upsampling and downsampling was not found in the captions of the talk. It may be on a slide that is not spoken aloud.
- No later id Tech engine (id Tech 4 to 7) and no Bungie talk on Destiny's networking were read.
