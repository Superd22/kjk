# What the slowest frames are made of

This document was researched on 2026-11-18 and 2026-11-19.
It answers one question.
What makes the slowest 1% of frames in the three big bench fights, which set the 1% low fps?

The fights are `anchorhead_brawl` (Anchorhead on Tatooine), `refugee_brawl` (the Refugee Quad on Nar Shaddaa) and `crowd_brawl` (the Jekk'Jekk Tarr on Nar Shaddaa).
The build was `origin/repeatable-fight` at `1037d9c1`, with no change.
The step thread was on (`KJK_STEP_BATCH=on`).

## Summary

- On the owner's PC, the main thread limits all three fights. The slowest frames there have two main causes.
- The first cause is a check of the network code that the solo game makes on the main thread once every 32 ticks, which is 4 times a second. That frame costs about 3.5 to 5 ms more than a normal frame in the traces. It is in 51% to 72% of the slowest 5% of frames, and in 36% to 39% of the slowest 1% in the Nar Shaddaa fights.
- The second cause is the main thread waiting for the step thread. This happens when a simulation tick takes two to three times its usual time. Almost all of these slow ticks are in the scripts, and they come back every 3 seconds, which is the length of a combat round. The main thread waits more than 2 ms in 33% to 57% of the slowest 1% of frames.
- Without both causes, the p99 frame time of the traced runs would be 4% lower in Anchorhead, 15% lower in the Refugee Quad and 13.5% lower in the Jekk'Jekk Tarr.
- In Anchorhead on the PC, a third of the slowest 1% of frames wait for the render thread.
- On the Mac, the GPU limits Anchorhead and the Jekk'Jekk Tarr. There, 96% to 97% of the slowest 1% of frames are the main thread waiting for the render thread, which waits for the GPU.
- The bench's own measurement adds a little to the slowest frames, and it reports the worst 5-second window's p99, not the p99 of the whole run.
- One fix is cheap and safe: move the network check off the main thread, or split it across frames. It does not change what the simulation does. No pull request was opened for it, because moving the work touches the shared server state and needs its own review and benchmark.

## How it was measured

### The tool

A new script reads a trace from `kjk bench --trace` and writes one row per frame.
It is kept outside the repository, in the session's scratchpad, as `slow-frames/frames.py`.
A second script, `slow-frames/clean.py`, reads the rows again without the trace's own stalls.
A third script, `slow-frames/inside.py`, lists the spans inside long spans of one name.

A frame is one `update` span of the main thread.
Its length is the time from its start to the start of the next frame.
Each row holds these columns:

- The frame's length, and the time the main thread spent outside `update`.
- The time of `sim_host::advance` on the main thread.
- The time the main thread waited in `advance` for the step thread. This is the end of the last tick of the job, minus the start of `advance`.
- The number of ticks the step thread stepped for that job, their total time, and the slowest tick with the system that took the most time in it.
- The time the main thread waited for the render thread before extracting, and the time of the extract.
- The render thread's time for this frame and for the frame before it.
- The time of pool threads working for the main thread, of asset loads, and of pipeline compiles.
- The self time of the 20 main-thread systems with the most self time.

The report then compares the slowest 1% and the slowest 5% of frames with the middle frames, which are those between the 40th and the 60th percentile.
It does this for every column, for every main-thread system, and for every render-thread system.
It also says how many slow frames come in a row and how far apart they are.

The window of each trace is its last 28 seconds, ending 1.5 seconds before its last frame.
That last frame is the bench's final dump, so the window holds only the fight.

### The runs

Each fight ran once with `--trace`, with 15 seconds of warm-up and 30 seconds of measurement.

On the Mac (M4 Max, Metal), the runs were headless and muted.
The traces were 14 to 22 GB.
Other owners compiled on the Mac at the same time.
The Anchorhead run had no compile, and the Jekk'Jekk Tarr run had a compile in 1 of its 12 samples taken every 5 seconds.
The Refugee Quad ran twice, and each run had compiles in 6 and 8 of its 11 samples.
The Refugee Quad's numbers on the Mac are therefore not used below, except to show what a busy CPU does.

On the PC (RTX 4080, i7-8700K, windowed), the main session ran one plain run of the three fights and one traced run of each.
The traces were 7.3 to 9.5 GB and were summarised on the PC, then deleted.

### Tracing changes the times

Tracing makes every frame slower.
On the PC, the median frame took 14.8, 13.8 and 12.6 ms in the traced runs, against 9.2, 8.4 and 8.3 ms in the plain run.
So the comparisons below use the shapes and the ratios, not the absolute times.

The PC traces also hold stalls that the plain runs do not have.
In the Refugee Quad and the Jekk'Jekk Tarr, a few frames took 0.15 to 4.6 seconds inside `console::run_queue`, in bursts of two or three.
In Anchorhead, two frames took 0.7 and 3.3 seconds.
The plain runs' p99 was 13.7 to 14.2 ms, so these stalls are not in a normal run. They most likely come from the traces being written to disk. That is an inference; the traces do not show the disk.
`clean.py` drops each of these frames and the 10 frames after it.
That drops 7 to 29 frames of 1000 to 1500 per fight.

## The PC

### The plain run

| Fight | Mean fps | Mean ms | p50 ms | p99 ms | 1% low fps | Main thread ms | Render thread ms |
|---|---|---|---|---|---|---|---|
| anchorhead_brawl | 106.6 | 9.38 | 9.20 | 14.20 | 70.4 | 8.71 | 7.77 |
| refugee_brawl | 115.8 | 8.63 | 8.37 | 13.66 | 73.2 | 7.95 | 6.82 |
| crowd_brawl | 117.0 | 8.55 | 8.31 | 13.67 | 73.2 | 7.84 | 5.66 |

The slowest 1% of frames take about 1.5 to 1.6 times the median frame.

### The traced runs, without the trace's stalls

All times in this table are traced times, in milliseconds.

| Fight | Frames | p50 | p99 | Slowest 1% mean | Middle frames mean |
|---|---|---|---|---|---|
| anchorhead_brawl | 980 | 14.79 | 20.35 | 24.85 | 14.78 |
| refugee_brawl | 1330 | 13.75 | 20.12 | 22.70 | 13.75 |
| crowd_brawl | 1450 | 12.55 | 17.68 | 22.15 | 12.55 |

What the slowest 1% of frames have that the middle frames do not, on average:

| Fight | Main thread waits for the step thread | Network check in `net.pump` | Main thread waits for the render thread | Slowest tick of the job |
|---|---|---|---|---|
| anchorhead_brawl | +1.48 ms | none in the slowest 1% | +3.92 ms | 4.53 ms against 3.60 ms |
| refugee_brawl | +2.40 ms | +2.45 ms | none | 5.26 ms against 4.97 ms |
| crowd_brawl | +5.59 ms | +1.48 ms | none | 10.22 ms against 4.32 ms |

Share of frames with each feature:

| Fight | Group | Main thread waits over 2 ms for the step thread | Network check in the frame | Main thread waits over 2 ms for the render thread |
|---|---|---|---|---|
| anchorhead_brawl | slowest 1% | 33% | 0% | 33% |
| anchorhead_brawl | slowest 5% | 8% | 51% | 8% |
| anchorhead_brawl | middle | 0% | 0% | 0% |
| refugee_brawl | slowest 1% | 39% | 39% | 0% |
| refugee_brawl | slowest 5% | 20% | 71% | 0% |
| refugee_brawl | middle | 0% | 0% | 0% |
| crowd_brawl | slowest 1% | 57% | 36% | 0% |
| crowd_brawl | slowest 5% | 17% | 72% | 0% |
| crowd_brawl | middle | 0% | 0% | 0% |

The other main-thread systems add little in the slow frames.
`play_view::draw` adds 0.35 to 0.61 ms, and `script_fx::draw` adds 0.07 to 0.53 ms.
No asset load ran during the fights.
The extract and the time outside `update` were the same in slow and middle frames.

### Cause 1: the network check every 32 ticks

The solo game runs its own server and client in the same process.
Once every 32 ticks, the server sends its own client a snapshot, and the client rebuilds it and checks its hash.
This only proves that the network code works. It changes nothing in the game, because the solo game draws its own world's snapshots.
The interval is the constant `CHECKED_EVERY` in `crates/game/src/solo.rs:64`. Its comment says that a rebuild took 1.5 ms in Ahto Central.

All of this runs on the main thread, inside `net.pump`:

1. The server splits the snapshot into parts and hashes the whole world (`Stepped::done`, `crates/net/src/codec/frame.rs:145`).
2. The server encodes a delta covering 32 ticks of change and deflates it (`encode_frame` and `pack`, `crates/net/src/codec/frame.rs:181` and `:209`).
3. The client inflates and decodes it, hashes the parts again, and joins them into a full new snapshot (`rebuild`, `crates/net/src/codec/frame.rs:272`).

In the PC traces, this frame's `net.pump` took 3.5 ms in Anchorhead, 5.1 ms in the Refugee Quad and 3.9 ms in the Jekk'Jekk Tarr, against 0.02 ms in other frames.
In the Mac traces it took 1.45 ms in Anchorhead, exactly every 0.256 seconds.
The traced runs ran at 52 to 75 fps, so the check fell in 5% to 6% of frames.
In a plain run at 107 to 117 fps it falls in about 3.5% of frames.

### Cause 2: slow ticks every combat round

The main thread hands a frame's ticks to the step thread and picks up the result at the start of the next frame.
It waits there when the step thread has not finished.
In the middle frames it never waits.
It waits when the job holds a tick that takes two to three times the usual time.

In all three fights on the PC, every tick that took more than twice the median had `scripts::scripts` as its largest system.
On the Mac the same was true for 77 of 77 such ticks in Anchorhead and 44 of 54 in the Jekk'Jekk Tarr.
These ticks come back every 3 seconds.
In the Mac's Anchorhead trace, they fell at 0.14, 3.14, 6.14, 9.14, 12.14 and 15.15 seconds of the window, each followed by others 0.03, 0.44 and 0.60 seconds later.
In the PC's Anchorhead trace, they fell at 10.28, 13.41, 16.27, 19.27 and 22.28 seconds.
3 seconds is the length of a combat round, so many characters' combat-round scripts run in the same few ticks.

A frame that starts after a long frame steps more ticks, up to 8, which can make it wait again.
In the PC traces, 67% of the slowest 1% of Anchorhead frames had a job of 3 ticks or more, against 2% of the middle frames.

### What removing each cause would do

`clean.py` was used to take the extra time of each cause out of each frame and compute the p99 again.
These are traced times.

| Fight | p99 | Without the network check | Without the waits for the step thread | Without both |
|---|---|---|---|---|
| anchorhead_brawl | 20.35 | 20.35 (0%) | 19.92 (-2.1%) | 19.57 (-3.8%) |
| refugee_brawl | 20.12 | 18.56 (-7.8%) | 19.65 (-2.3%) | 17.00 (-15.5%) |
| crowd_brawl | 17.68 | 16.13 (-8.8%) | 17.05 (-3.6%) | 15.29 (-13.5%) |

The two causes together do more than the sum of each one, because many slow frames hold both.
Taking out the waits for the render thread lowers Anchorhead's p99 by 2%.

## The Mac

On the Mac, the render thread and the GPU limit Anchorhead and the Jekk'Jekk Tarr.

| Fight | Frames | p50 ms | p99 ms | Slowest 1%: main thread waits for the render thread over 2 ms | Extra time of the render thread's `queue_submit` in the frame the slow frame waited for |
|---|---|---|---|---|---|
| anchorhead_brawl | 4038 | 6.58 | 13.78 | 97.5% | +6.0 ms |
| crowd_brawl | 5168 | 5.04 | 11.02 | 96.1% | +3.5 ms |

`queue_submit` waits when the GPU has too much work queued, so these frames are slow GPU frames.
Without the waits for the render thread, Anchorhead's p99 would be 8.10 ms instead of 13.78 ms.
The simulation never made a frame wait more than 2 ms in Anchorhead.
The network check cost 1.45 ms, but on the Mac it does not make frames slower, because the main thread has time to spare while it waits for the render thread.

In the Jekk'Jekk Tarr, one pipeline was compiled in the middle of the fight.
It took 138 ms on the render thread, inside `PipelineCache::process_pipeline_queue_system`, and made a frame of 148 ms.
The game compiles its pipelines before play (`crates/game/src/warmup.rs`), so this pipeline was missed by that warm-up.
The trace does not say which pipeline it was.

### What a busy CPU does

Both Refugee Quad runs on the Mac overlapped other owners' compiles.
In the second run, the frames got slower as the number of running compilers grew from 0 to 3.
In its 5-second stretches, the mean frame went from 10.2 ms to 29.3 ms.
In the slowest 1% of those frames, the main thread spent 32 to 41 ms in `propagate_parent_transforms`, against 0.5 to 1.1 ms in the middle frames.
That system splits its work across the pool threads, and the main thread waits until every part has finished.
When one pool thread is paused by the system, the whole frame waits for it.
This matters for a player whose CPU is also busy with something else, and probably for the weak machine, whose container shares its CPUs. That last point is an inference; the weak machine was not traced for this document.

## The bench's own part

Every 5 seconds the bench runs `dump`, which writes the frame times of the last 5 seconds.
In the PC traces, the dump's frame took 9 to 14 ms more than the median.
About 4 ms of that is the dump itself. The rest is the main thread waiting 5 to 8 ms for the step thread, because the dump reads the simulation's facts and so brings the world home before the step thread has finished.
Without the dump's frame and the 2 frames after it, the p99 is 0.4% to 1.6% lower.

The bench reports the highest of the six 5-second windows' p99 values, not the p99 of the whole run (`Measured::worst`, `crates/launcher/src/bench.rs:430`).
In the PC traces, the worst window's p99 was up to 7% higher than the p99 of the whole window.
So the reported 1% low is a little lower than the run's true 1% low.
This does not change comparisons between builds, but it should be known when the 1% low is compared with other games' numbers.

## The fixes, ranked

The expected effects are estimates from the traced shapes. Each one should be confirmed with a plain bench on the PC.

1. **Move the network check off the main thread.** The server's split and hash can be made on the step thread, as a networked host already does (`Prepare { split }` in `crates/game/src/sim_host.rs:2585`). The client's rebuild can run on a worker thread, as the rebaser already does. The check can also be split so that the server's half and the client's half fall in different frames. This changes nothing the simulation does. Expected effect: about 8% to 9% lower p99 in the Refugee Quad and the Jekk'Jekk Tarr, so about 6 fps more on their 1% low of 73 fps. Little effect in Anchorhead. Raising `CHECKED_EVERY` would not help much, because a check once a second would still fall in about 1% of frames, which is exactly where the 1% low is measured.
2. **Stop waiting for a late step thread.** The note `docs/research/2026-11-18-present-last-sim-snapshot.md` recommends a queue of a few ticks with a cap. The main thread would draw the newest finished tick and would wait only when the queue is full. Expected effect: 2% to 4% lower p99 alone. Together with fix 1, 4% to 15.5% lower p99, so about 3 fps more on the 1% low in Anchorhead and 11 to 13 fps more in the Nar Shaddaa fights. This is probably the largest fix for the weak machine, where one frame in ten waited 6 to 12 ms for the simulation.
3. **Make the combat-round scripts cheaper.** The slow ticks are the scripts that run at each 3-second combat round. Making those scripts cheaper keeps the simulation the same. Spreading them across ticks would change what the simulation does, so it would need the owner's approval. Expected effect: part of fix 2's gain, without a change to how the view is drawn.
4. **Less GPU work in Anchorhead.** A third of Anchorhead's slowest 1% on the PC, and almost all slow frames on the Mac, wait for the render thread and the GPU. The other lines that cut render-thread and GPU time help here. Expected effect: about 2% lower p99 in Anchorhead on the PC.
5. **Find the pipeline that the warm-up misses.** A log line for each pipeline compiled after the warm-up would name it. Expected effect: one 140 ms hitch less in the Jekk'Jekk Tarr on the Mac. It does not change the p99.
6. **Measure the 1% low over the whole run, and keep the dump from bringing the world home.** These are fixes to the bench, not to the game. Expected effect: a reported 1% low up to 7% higher, closer to what a player sees.

## What could not be done

- The Refugee Quad could not be traced on the Mac without other compiles running.
- The weak machine was not traced for this document. Its earlier traces hold only means and percentiles.
- The traces cannot name the pipeline that was compiled during the fight, nor the scripts that make the slow ticks. The scripts' own spans do not go below `scripts::scripts`.
- The PC traces were written to the same disk the game ran from, which made a few frames of 0.15 to 4.6 seconds. Those frames were dropped from the analysis.

## Files

All files are in the session's scratchpad, under `slow-frames/`.

- `frames.py` makes the per-frame rows and the report from a trace.
- `clean.py` drops the trace's stalls, gives the bench-style p99, and compares the slow frames again.
- `inside.py` lists what runs inside long spans of one name.
- `run-mac.sh` and `run-pc.sh` are the run scripts.
- `mac/` and `pc/` hold each fight's `frames-*.csv` and `frames-*.txt`, the run logs, and on the Mac the compile samples.
