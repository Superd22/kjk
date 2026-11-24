# What a simulation tick costs in the biggest fights

This document was researched on 2026-11-18.
It answers two questions.
Where does kjk's simulation spend its time in each tick of the biggest bench fights?
How can that time be made smaller?

The fights are the three bench scenes with 40 added foes: `refugee_brawl` (the Refugee Quad on Nar Shaddaa), `crowd_brawl` (the Jekk'Jekk Tarr on Nar Shaddaa) and `anchorhead_brawl` (Anchorhead on Tatooine).
The work was done on `origin/repeatable-fight` at `0600adc3`.
Every timing in this document was taken on the M4 Max Mac, headless and muted, unless it says otherwise.

The document has five parts.
The first part says how the measurements were made.
The second part gives where each tick's time goes, before any change.
The third part lists the changes that keep every result bit for bit the same, with what each one saves.
The fourth part lists the changes that would alter what the simulation computes, for the owner to decide on.
The fifth part says what could not be measured.

## Summary

- In `refugee_brawl`, most of each tick went to three or four of the 40 bench foes. The bench makes all 40 foes at one spot in front of the player, and the placement spreads them out from there. In the Refugee Quad, three or four of them end up inside a wall. Each step, a character whose box is inside the level searches 27 spots near it, then up to 576 spots out to 48 units, for a place where it fits. None of these foes ever finds one, so they search again in every step of the whole fight, twice per step. This search took 4.1 ms of the 7.2 ms that a tick cost on the Mac, and 86% of all level traces.
- In `crowd_brawl` and `anchorhead_brawl`, no foe stays stuck after the first two seconds. A tick there cost 2.1 ms and 1.3 ms on the Mac. Movement took about half of it, and the scripts about a quarter.
- Five changes that keep every result bit for bit the same were built as prototypes. Together they make a `refugee_brawl` tick cost 1.15 ms instead of 6.14 ms on the Mac, 5.3 times less. The whole replay of `crowd_brawl` takes 6.7 s instead of 11.7 s, and that of `anchorhead_brawl` 4.9 s instead of 6.8 s. These times are from builds without any timers. Each prototype replayed all three recorded fights and matched every one of their recorded checkpoints, 51 to 53 of them per fight, and their end hashes.
- The largest of the five makes the level remember its traces. A level never changes once it is built, apart from the doors' states, and a trace's answer depends only on the level, the trace's start and end, its shape, its mask and the doors' states. Characters who stand still ask the same traces in every tick, so most traces in a crowd are answered from memory. In `crowd_brawl` the level traces actually computed fall from 899 to 72 per tick.
- The changes that would alter the simulation are listed for the owner. The most useful one is to make the arena's foes appear only where they fit, so that the bench fights measure a fight and not foes stuck in a wall.

## How it was measured

### Timers and counters

A temporary set of timers and counters was added to the simulation, and removed before the end.
Each timer measured the time spent inside one part of the step, such as one system, the walker's step or the search for a free spot.
Each counter counted one kind of work, such as level traces, brushes clipped, bodies tested or script instructions run.
The values were summed per tick and written to a file for a window of ticks.
Ticks 410 to 3,400 or to 5,100 were measured, which starts just after the 40 foes arrive at tick 401.

### Sampling

The macOS `sample` tool took call stacks of the running game, one per millisecond, for 6 to 15 seconds.
The stacks were summed per function, inside the simulation's step only.
The sampled profile and the timers agreed on where the time went.

### Replays of the recorded fights

A live run of a bench fight is a poor A/B test on the Mac.
In one live run of `crowd_brawl`, every timing halved from tick 1,460 on, while every counter stayed the same.
The thread had moved to another core, or other work on the machine had stopped.

So each fight was recorded once in a live run, and then replayed with `kjk sim replay`.
A replay steps the recorded ticks as fast as the machine can, with no window and no rendering.
It also compares the world with the recording's checkpoint hashes, one every 100 ticks, and with the end hash.
A replay that matches every checkpoint is proof that the build computes the same fight, bit for bit.

`kjk sim replay` could not replay a recording made on an arena stage, because the arena's level differs from the plain module's.
The local commit "Replays of arena stage recordings" adds two environment variables for this.
`KJK_REPLAY_STAGE` names the stage, and `KJK_REPLAY_HOSTILE` names the creatures that `--hostile` turned.

The recordings hold 5,106 ticks of `refugee_brawl`, 5,337 ticks of `crowd_brawl` and 5,367 ticks of `anchorhead_brawl`, about 41 to 43 seconds each.
Ticks 0 to 400 are the arrival before the foes are called in.

## Where each tick's time goes, before any change

The table gives the mean time per tick on the Mac, with the timers in place, over ticks 410 to 5,000 or 5,100 (37 seconds of fight).
The counts below it are from ticks 410 to 3,400 of a live run.
The timers themselves add a little time, mostly through the counters inside the brush clipping.

| Part of the tick | `refugee_brawl` | `crowd_brawl` | `anchorhead_brawl` |
|---|---|---|---|
| Characters | 108 | 122 | 82 |
| Whole step | 7.21 ms | 2.08 ms | 1.27 ms |
| Movement (`move_actors`) | 6.10 ms | 1.02 ms | 0.52 ms |
| of which the free-spot search of stuck characters | 4.14 ms | 0.08 ms | 0 ms |
| of which the walker's step (`kotor_step`) | 0.55 ms | 0.01 ms | 0.04 ms |
| Scripts | 0.56 ms | 0.51 ms | 0.34 ms |
| Minds (`think`) | 0.14 ms | 0.25 ms | 0.15 ms |
| Sabers | 0.13 ms | 0.14 ms | 0.09 ms |
| Force powers | 0.11 ms | 0.20 ms | 0.09 ms |
| Firing and bolts | 0.10 ms | 0.18 ms | 0.08 ms |
| Snapshot | 0.10 ms | 0.10 ms | 0.07 ms |

All three columns come from the replays.
A live run of `crowd_brawl` measured 3.7 ms per tick over the same ticks, and the difference is the machine, as the counters were the same.

### Counts per tick in `refugee_brawl`

- All 108 characters were alive in every measured tick. Nobody died in the first 40 seconds of the fight.
- 86 characters moved by JA's movement (`move_body`) and 22 were carried by KOTOR's walker. 20 of those took a walker's step (`kotor_step`).
- The ground check (`find_ground`) ran 172 times, twice per character moved by JA's movement.
- The level was traced 3,668 times, and 49,432 brushes were clipped.
- 46,977 body boxes were weighed against traces, and 620 of them were clipped.
- The scripts ran 9.4 times and executed 6,880 instructions.

### The stuck foes in the Refugee Quad

A trace that started inside the level, the sign of a stuck character, came from 2 to 4 characters in each tick.
They stood at x from -1,291 to -1,401 and y from 7,155 to 7,183 (units), at z 654, in a grid 27.6 units apart.
That grid is where the bench's `arena spawn` order puts the 40 foes.
Each of these foes stood inside the level's solid, not inside another body.

For each of them, every step ran the ground check twice.
Each ground check traced once below the box, searched the 27 spots within a unit of the origin twice, and then searched up to 576 spots out to 48 units.
None of the spots fit, so the search ran to the end every time.
That made 1,262 traces per stuck character per step.
In the first 750 ticks, 7.4 such checks ran per tick, and from then on 4.0.

### The first seconds after the foes arrive

In each fight, the first 150 ticks after the foes arrive cost more.
In `crowd_brawl` the scripts took 3.4 ms per tick in those ticks, and 0.5 ms afterwards.
These are the 40 new creatures' first scripts.
Two of the foes were also stuck for about the first 2.5 seconds, until their free-spot search found them a place.

### How the cost grows with the number of characters

All characters stayed alive, so the number of characters did not change within a fight.
Across the three fights, the step cost about 10 to 17 µs per character on the Mac, once the stuck foes are left out.
Two kinds of work grow with the square of the number of characters.
Each trace of a character's body weighs every other body's box: 26,000 boxes per tick for 82 characters, 47,000 for 108 and 58,000 for 122.
Each move also builds lists of all the other characters.
Both are still small next to the level traces.

## Changes that keep every result the same

Each of these changes keeps the simulation's results bit for bit the same.
The sim hash does not move, so none of them needs a bumps file.
Each prototype was checked by replaying the three recorded fights, which must match every checkpoint hash and the end hash.

The times below are the mean per tick over ticks 410 to 5,000 or 5,100, from the replays with the timers in place, taking the changes in the order listed.

| Change | `refugee_brawl` | `crowd_brawl` | `anchorhead_brawl` |
|---|---|---|---|
| None | 7.21 ms | 2.08 ms | 1.27 ms |
| 1. Cheaper checks for stuck characters | 2.72 ms | | |
| 2. Remembered fits checks | 2.22 ms | | |
| 3. One walk of the level for the doors | 1.74 ms | 1.89 ms (1 to 3) | 1.18 ms (1 to 3) |
| 4 and 5. Remembered traces and reputation constants | 1.23 ms | 1.04 ms | 0.87 ms |

### Times without timers

The table gives the wall time of each replay, with builds that have no timers in them.
Each replay ran three times, and the builds took turns.
The three times of each cell were within 0.4 s of each other, apart from one run of 18.6 s.
Every run matched all of its recording's checkpoints.
A replay's time includes building the level and the 400 ticks before the foes arrive, about 4 seconds in all.

| Build | `refugee_brawl` (5,106 ticks) | `crowd_brawl` (5,337 ticks) | `anchorhead_brawl` (5,367 ticks) |
|---|---|---|---|
| `0600adc3`, no change | 35.7 s | 11.7 s | 6.8 s |
| Change 1 | 17.5 s | 11.6 s | 6.9 s |
| Changes 1, 2 and 4 | 12.7 s | 7.5 s | 5.3 s |
| Changes 1 to 4 | 10.1 s | 7.6 s | 5.2 s |
| Changes 1 to 5 | 9.6 s | 6.7 s | 4.9 s |

A shorter recording of the same `refugee_brawl` fight, of 1,194 ticks, took 11.6 s with no change and 5.1 s with all five changes.
The difference between the two recordings is 3,912 ticks of the fight after the first seconds.
Those ticks cost 6.14 ms each with no change and 1.15 ms each with all five changes.

For `crowd_brawl` and `anchorhead_brawl` only the whole replays were timed.
If their level takes as long to build as the Refugee Quad's, a tick of `crowd_brawl` costs about 1.5 ms with no change and 0.6 ms with all five, and a tick of `anchorhead_brawl` about 0.6 ms and 0.2 ms.
These two are estimates.

A live run of `refugee_brawl` held at tick 1,500 wrote the same world hashes in its dump with no change and with all five changes.

### 1. Cheaper checks for stuck characters

This is the earlier line kept on the branch `stuck-ground-checks` at `e042224d`.
It applied to the current code without conflicts.
It tests each spot of the free-spot search with a new collision query, `fits`, which stops at the first brush that the box stands inside.
It also reuses the first search's answer for the second ground check of the same step, when the origin and the box are the same.

In `refugee_brawl` it cut the brushes clipped from 49,432 to 4,324 per tick, and the step from 7.21 ms to 2.72 ms.
In the two other fights it saves almost nothing, because no character is stuck there after the first seconds.

### 2. Remembered fits checks

A stuck foe searches the same spots in every step, and the level does not change between steps.
So the level now remembers the answer of each `fits` check, with everything it depends on: the spot, the box, the kind of shape, the mask and the doors' states.
The doors' states are kept whole beside the answers and compared, so a remembered answer is never used for other doors.
A body only makes a spot more solid, and `fits` asks the level first, so a spot the level fills is answered from memory without looking at the bodies.

This took the free-spot search from 0.53 ms to 0.11 ms per tick in `refugee_brawl`, and the step from 2.72 ms to 2.22 ms.
In the prototypes it was then folded into change 4.

### 3. One walk of the level for the doors a walker meets

Each step of KOTOR's walker checks whether a closed door stands across the step.
It tested every door of the area in turn, and each test walked the level's tree of brushes for the same small box.
The Refugee Quad has many doors and 20 walkers, so this took 0.54 ms per tick.

Now the doors that the box touches are found in one walk of the tree, and only those doors are tested further.
A door is in that list exactly when its own test would have found it.
The check now takes 0.04 ms per tick there.

### 4. Remembered level traces

Change 2 is made general.
The level remembers the answer of every trace, with its start and end, its shape and kind, its mask and the doors' states.
A character standing still asks the same traces in every step: its two ground probes, the check for placeables around it (`walker_world`), the check for furniture it may stand in (`tucked`), and the line of sight between two characters who have not moved.
In these fights most characters are bystanders who stand still.

The level traces actually computed fell from 899 to 72 per tick in `crowd_brawl`, from 585 to 134 in `refugee_brawl` and from 707 to 93 in `anchorhead_brawl`.
A level keeps 65,536 answers of each kind and forgets them all when it has more.
That is at most about 20 MB per level.
A remembered answer costs a lock and a hash lookup, about 0.1 µs, where a trace costs 1 to 3 µs.
In a fight where every character moves all the time, fewer traces repeat, and the gain is smaller.

### 5. Reputation constants read once

`GetNearestCreature` with a reputation criterion looked up the three `REPUTATION_TYPE` constants by name for every creature it weighed.
Each lookup searched the script catalog's constants one by one, comparing names.
In `crowd_brawl` this took 8% of the sim's time.
The library now reads the three values once, when it is made.

### Further changes not built

These keep results the same too, but were not built.
The savings are estimates from the sampled profiles of the prototypes.

- **Copies of each character in the sabers and Force powers systems.** Both systems build a list with a copy of every character's sheet, fighter state and motion state in each tick. Copying memory took 8% of `crowd_brawl`'s sim time with changes 1 to 3, mostly here. Borrowing instead of copying would save an estimated 0.05 to 0.1 ms per tick.
- **The best known row of a Force power.** `force::best` walks all rows of the power table for every character and power in each tick, instead of the rows the character knows. It took 4% of `crowd_brawl`'s sim time with changes 1 to 3. Walking only the known rows, in the same order, would save an estimated 0.03 to 0.05 ms per tick.
- **Factions compared per pair.** `module::relation` took 4% of `crowd_brawl`'s sim time with changes 1 to 3. A table of relations made once per tick would save an estimated 0.03 ms per tick.
- **Bodies near a trace.** Each trace weighs every other body's box, 44,000 to 58,000 boxes per tick. A grid of the bodies, visited in the same order as the list, would save an estimated 0.05 to 0.1 ms per tick. The order must stay the list's order, because the first body found wins a tie.

## Changes that would alter the simulation

These change what the simulation computes.
Each needs a bumps file and the owner's approval.

### Foes appear only where they fit

The arena's `spawn` order makes each foe at one spot in front of the player, `rules::world::arena::SPAWN_AHEAD_M` metres ahead, and the creature placement spreads them out from there.
In the Refugee Quad, three or four of the 40 bench foes end up inside a wall, where they stand for the whole fight.
The arena could instead make each foe on the nearest spot of the walkmesh where its box fits.

What the player would see: no foe would stand inside a wall in the arena.
The bench's `refugee_brawl` would then measure 40 fighting foes, not 36 fighters and some wedged ones.
The sim hash of the determinism run would not move, because that run uses no arena orders, but the bench fights would play differently, so their recorded runs would no longer compare with older builds.

With changes 1 to 4, a stuck foe costs little time, so this matters more for what the bench measures than for speed.
If scripts in the real game also place creatures inside the level, the same fix would apply to `CreateObject`, and that would need its own check against the original game.

### Characters far from every player step less often

The authority moves every character in every tick.
A prediction already skips the creatures farther than `PREDICT_NEAR` from every player.
The authority could step characters far from every player, out of combat and with nothing queued, at a lower rate, for example once every 4 ticks with four times the time.

What the player would see: nothing near them.
A far character's walk would be computed in larger steps, so it could round a corner or stop at a door slightly differently.
The saving could be large in big areas with many idle characters, up to three quarters of their movement cost, but with change 4 their movement is already cheap.
This was not measured.

### Perception and minds at KOTOR's own rate

The scripts' perception and the minds run in every tick, 125 times a second.
They took 0.03 to 0.21 ms and 0.12 to 0.25 ms per tick in these fights.
If the original game updates perception and its AI less often, kjk could match it and save most of that time.
Whether it does, and at what rate, would need a check against the original engine first.
What the player would see: foes would notice the player and react up to one update later.

## Measured impact of the behaviour-changing options

This part was added later on 2026-11-18.
It measures the last two options of the part above: characters far from every player stepping less often, and perception and minds at the original game's rate.
Both were built as prototypes on top of the five speedups, so the numbers are what would ship.
Every number in this part was measured on the M4 Max Mac, unless it says "estimated".

### How it was measured

Each option was built as a prototype that is off unless an environment variable turns it on.
With the variables unset, the build replays all three recorded fights and matches every checkpoint and the end hash.

A replay of a recording stops matching as soon as an option changes anything.
So the replay tool was given a mode that steps on past the first mismatch.
Every tick still gets the recorded inputs: the arena's 40 foes at tick 401, god mode for the player, and no key presses.
This is a live run of each fight for a fixed number of ticks, with the same inputs every time, stepped as fast as the machine can.
The fights last 5,106, 5,337 and 5,367 ticks, about 41 to 43 seconds.

During each run a tracker wrote down every character's place every 25 ticks (0.2 s), every hit, block and death, and every change of target.
It also timed each tick, and the movement, the minds and the perception inside each tick.

These fights are chaotic.
To know how much of a difference is the option and how much is chaos, one foe was moved by 1 millimetre at tick 600, with no option on.
This is called the nudged run below.
Any difference that the nudged run also shows cannot be blamed on an option.

### Determinism

Each option gives the same result on two runs.
The base fights, the far stepping with N = 4 and 20 m, and the original perception rate were each run twice, and each pair ended on the same hash.

Neither option moves `bumps/SIM_HASH`.
The pinned determinism run ended on the same hash with every option on as with none.
That run has four players and 32 wanderers, but no creature with a mind or a perception, so the options never apply in it.
Both options do change any fight with creatures in it, so recordings made before them would no longer replay.
Neither option changes a save, snapshot or replay format.

### Option 1. Characters far from every player step less often

#### What the prototype does

A creature farther than a set distance from every player moves only once every N ticks.
When it moves, its step covers N ticks of time.
Each creature takes its turn on a different tick, by its id, so the work is spread evenly over the ticks.
Players are never slowed, and a prediction is not changed.
N = 2 and N = 4 were tried, each with 20 m and with 40 m.

#### Performance

The table gives the time that movement took per tick, from tick 560 to the end of each fight.
Each cell is the median of three replays.
Ticks 401 to 560 were left out, because the first scripts of the 40 new foes take most of the time there.

| Setting | `refugee_brawl` | `crowd_brawl` | `anchorhead_brawl` |
|---|---|---|---|
| Base | 0.453 ms | 0.286 ms | 0.216 ms |
| Nudged run (no option) | 0.443 ms | 0.327 ms | 0.163 ms |
| N = 2, 20 m | 0.377 ms | 0.319 ms | 0.155 ms |
| N = 4, 20 m | 0.379 ms | 0.295 ms | 0.155 ms |
| N = 2, 40 m | 0.399 ms | 0.261 ms | 0.157 ms |
| N = 4, 40 m | 0.383 ms | 0.256 ms | 0.162 ms |
| Steps skipped per tick, N = 4, 20 m | 49 | 49 | 30 |

Only `refugee_brawl` shows a gain larger than the nudged run's change.
There movement fell by about 0.07 ms per tick, from 0.45 ms to 0.38 ms.
That is about 6% of the whole tick.
In `crowd_brawl` and `anchorhead_brawl`, the nudged run alone changed movement by as much as any setting of the option, so no gain could be told apart from chaos.

A skipped step saves about 1.5 to 2 µs (estimated from `refugee_brawl`: 0.07 ms over 33 to 49 skipped steps).
The far characters are mostly bystanders standing still, and change 4 already answers their traces from memory.
That is why skipping them saves so little.
N = 4 saves no more than N = 2, because the steps left over are the costly ones.

The whole tick, with the snapshot, took the following times from tick 560 on (median of five replays).
The worst 1% is the mean of the slowest 1% of those ticks.

| Setting | `refugee_brawl` mean | worst 1% | `crowd_brawl` mean | worst 1% | `anchorhead_brawl` mean | worst 1% |
|---|---|---|---|---|---|---|
| Base | 1.19 ms | 3.20 ms | 1.06 ms | 2.60 ms | 0.83 ms | 2.61 ms |
| N = 2, 20 m | 1.11 ms | 3.09 ms | 1.14 ms | 2.79 ms | 0.74 ms | 2.49 ms |
| N = 4, 20 m | 1.11 ms | 2.93 ms | 1.12 ms | 2.71 ms | 0.75 ms | 2.52 ms |

The nudged run, with no option, took 1.18 ms, 1.12 ms and 0.72 ms per tick.
So a different course of the same fight moves the whole tick by 6% to 12%, as much as the option does.
From tick 410 on, the worst 1% is 8.8 ms, 4.3 ms and 8.6 ms with every setting, because the foes' first scripts make those ticks.
The option does not touch them.

#### How the fights play

No one died in any run, with or without the option, because the player stands in god mode and does not attack.
The 40 foes went for the player 21 to 22 ticks after they arrived in every run, because the arena orders them to.
The first hit landed at the same tick in every run of this option: 170 ticks after the foes arrived in `refugee_brawl`, 22 in `crowd_brawl` and 127 in `anchorhead_brawl`.

The number of hits and blocks changed, but no more than in the nudged run.
For example, `anchorhead_brawl` had 395 blocks in the base run, 228 in the nudged run, and 277 to 316 with the option.

Where the far characters end up also changed by no more than in the nudged run.
After 40 seconds in `refugee_brawl`, the characters farther than 20 m stood 3.6 to 4.3 m from their base places on average with the option, and 3.8 m in the nudged run.
In `crowd_brawl` the figures were 0.4 to 0.8 m with the option and 0.5 m in the nudged run.
In `anchorhead_brawl` they were 0.8 to 0.9 m with the option and 0.8 m in the nudged run.

The option's own effect shows best in the first 400 ticks, before the foes arrive, when nothing else differs.
In `crowd_brawl` every character stood at the same place, to the millimetre, with and without the option.
In `anchorhead_brawl` no character moved by more than 5 cm.
In `refugee_brawl` one bystander, about 120 m from the player, walked off in another direction from its first far step on.
By tick 400 it stood 3.8 m (N = 4) or 4.7 m (N = 2) from its base place.
With N = 4 it walked into a second bystander and pushed it 1 m, and that one stayed there.
This is the visible case.
The player could not see it, as both stood far beyond 20 m.
No run showed a character taking another door or corner because of the option, but the tracker did not look for door crossings, only for places.

### Option 2. Perception and minds at the original game's rate

#### What the original game does

This comes from reading Odyssey, the byte-identical reconstruction of `swkotor.exe` 1.03, and was checked against reone.

- The server updates every creature's AI once per rendered frame. It has a budget of 10 ms per frame, and goes on from where it stopped in the next frame when the budget runs out. A creature's actions (its walk, its attack) run in every one of these updates.
- A creature never updates less often because it is far from the player. Only a creature in an area with no player in it updates its heartbeat and its perception far less often.
- A creature that is not in the party looks at everything around it once every 4 seconds. It also checks the party alone in every update, so in every frame.
- The creature that the player controls looks around every 200 ms.
- A combat round lasts 3 seconds, and the AI scripts choose their next attack at its end. The heartbeat script runs every 3.0 to 4.2 seconds.

So the original does not update perception less often than kjk for what matters most.
A creature notices a party member in the next frame after it comes into view.
kjk's creatures look around every 300 to 700 ms (`stealth.perception`), for everyone.
A creature in kjk therefore notices the player 0 to 700 ms after the player comes into view, 250 ms on average (estimated from the rate, 0 to 87 ticks).
In the original it notices the player in the next frame, and anyone else 0 to 4 seconds later.

kjk's minds already pick their foe on the original's pace where the game's scripts run, at the end of each 3-second round.
Where no scripts run, as on the arena stage of these fights, a mind looks for the nearest foe every 200 ms (`minds.looking.look_every_ms`, kjk's own value).
The original has no such mind, because its creatures always have scripts.

#### What the prototypes do

Two prototypes were built anyway, to measure them.
The first follows the original's perception: a creature outside the party looks at everyone every 4.0 to 4.4 seconds, and at the party alone in every tick.
The second makes the minds without scripts look for a foe every 3 seconds instead of every 200 ms, the length of a combat round.

#### Performance

The table gives the time per tick from tick 560 on, as the median of three replays for each system and of five replays for the whole tick.

| Setting | `refugee_brawl` | `crowd_brawl` | `anchorhead_brawl` |
|---|---|---|---|
| Perception, base | 0.051 ms | 0.059 ms | 0.032 ms |
| Perception at the original's rate | 0.122 ms | 0.143 ms | 0.088 ms |
| Minds, base | 0.118 ms | 0.125 ms | 0.105 ms |
| Minds looking every 3 s | 0.111 ms | 0.128 ms | 0.105 ms |
| Whole tick, base (worst 1%) | 1.19 ms (3.20 ms) | 1.06 ms (2.60 ms) | 0.83 ms (2.61 ms) |
| Whole tick, original perception (worst 1%) | 1.32 ms (3.36 ms) | 1.29 ms (3.58 ms) | 0.80 ms (2.31 ms) |
| Whole tick, minds every 3 s (worst 1%) | 1.08 ms (2.82 ms) | 1.16 ms (3.24 ms) | 0.78 ms (2.31 ms) |

The original's perception costs more than kjk's, not less.
It took 0.06 to 0.08 ms more per tick, more than twice as much.
The check of the party in every tick traces a line of sight from each of the 80 to 120 creatures to the player.

Looking for a foe every 3 seconds saved at most 0.007 ms per tick in the minds.
The whole-tick differences in the table are within what the nudged run shows (6% to 12%), so they come from the fight going another way.

#### How the fights play

The 40 foes went for the player 19 to 22 ticks after they arrived with both prototypes, as in the base run, because the arena orders them to.
No one died in any run.
The number of hits and blocks changed no more than in the nudged run.

The visible differences are in `crowd_brawl`, where some of the bar's thugs are hostile and stand near the player when the foes arrive.
- In the base run, a thug hit the player 22 ticks (0.18 s) after the foes arrived.
- With the original's perception, the first hit on the player came 242 ticks (1.9 s) after they arrived, and it was a foe's.
- With minds looking every 3 seconds, the first hit came 125 ticks (1.0 s) after they arrived. The first thug to go for the player did so 1,543 ticks (12.3 s) after the foes arrived, instead of 20 ticks.
- The nudged run kept the first hit at 22 ticks, so these changes come from the prototypes.

The far characters' places drifted by no more than in the nudged run with either prototype.

### Recommendations

- **Characters far from every player step less often.** Do not ship it now. It saves about 0.07 ms per tick in `refugee_brawl`, about 6% of the tick, and nothing that could be measured in the two other fights. The five speedups already made far, idle characters cheap. In exchange, far walkers take other ways, as the Refugee Quad's bystander did, and every recording made before it stops replaying. It could be worth trying again in an area with many walking characters far from the player, which these fights do not have.
- **Perception and minds at the original game's rate.** Do not ship it. The original updates perception of the party in every frame, not less often, and its full look every 4 seconds costs kjk more than its current rate, not less. Matching the original would make foes notice the player sooner, and would cost 0.06 to 0.08 ms more per tick. Slower minds save almost nothing and make hostile bystanders slow to join a fight. Whether kjk's 300 to 700 ms rate should become the original's per-frame check of the party is a question of faithfulness, not of speed, and is left to the owner.

### The prototypes

They are local commits on the branch `sim-behaviour`, in the worktree `.claude/worktrees/sim-behaviour`, on top of `sim-cost`.
None of them is pushed, and none is meant to be merged as it is.

- `KJK_FAR_N` and `KJK_FAR_M` turn on the far stepping, every N ticks beyond M metres.
- `KJK_PERC_ORIG` turns on the original's perception rate, and `KJK_LOOK_MS` sets how often a mind without scripts looks for a foe.
- `KJK_REPLAY_ONWARD` makes `kjk sim replay` step on past a mismatch, `KJK_TRACK` names the tracker's file, and `KJK_NUDGE` moves one creature by 1 mm (`id@tick`).
- The replay also prints the time spent in movement, minds and perception from tick 560 on.

The scratch data, the scripts and the analysis are in the session's scratchpad, under `simcost/beh/`.

## What could not be measured

- The Windows PC and the weak Xeon machine were not used. The savings there are expected to be in the same proportion as on the Mac, because the changes remove work rather than make it faster, but this is an inference. On the Xeon, `refugee_brawl` ran 26 to 37 ticks per second, about 30 ms per tick. If the gain holds there, a tick would cost about 6 ms, and the simulation would keep up with 125 ticks per second.
- The fights in the bench do not kill anyone in their first 40 seconds, so the cost of deaths, corpses and the change in the number of characters during a fight was not measured.
- The cost of the scripts was measured as a whole, with its instruction count. Which scripts run, and which heartbeats, was not broken down.
- The first seconds after the foes arrive cost more, mostly in scripts. They were measured but not studied.
- The memory used by the remembered traces was estimated from their size, not measured.
- Change 4 was measured in fights where most characters stand still. A fight where every character moves was not measured.

## The prototypes

They are local commits on the branch `sim-cost`, in the worktree `.claude/worktrees/sim-cost`, on top of `0600adc3`.
None of them is pushed.
On the branch, `cargo fmt --check` and the workspace's clippy pass, and the approved tests pass (161 tests, `sim_determinism` among them, with the sim hash unchanged).

- `843c77c7` "Cheaper ground checks for characters stuck in the level" and `7187ed03` "Review fixes for cheaper ground checks" are the `stuck-ground-checks` line, applied without changes.
- `320dc68c` "Replays of arena stage recordings" is the replay tool used for this document.
- `3e7544ff` "Remembered level traces" is changes 2 and 4.
- `7d5e0241` "One walk of the level for the doors a walker meets" is change 3.
- `a212feb5` "Reputation constants read once" is change 5.
