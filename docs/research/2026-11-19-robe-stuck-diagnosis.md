# Why the robe stays flared and bunches into a lump

This note was written on 2026-11-19.
It belongs to the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts").
It explains two failures of the robe solver that the user saw in the L5 recording `game-pmbim-on-side.mp4`, and what else keeps the robe from looking right.
It diagnoses only. Nothing in the solver was fixed.

The work is on the branch `loose-parts-diag`, in the worktree `.claude/worktrees/loose-parts-diag`, made from `loose-parts-l5`.
Its one commit adds a debugging aid: switches, a per-substep trace and an overlay in play (see "How it was measured").
The videos, contact sheets, traces and scripts are in `kjk-artifacts/loose-parts-diag/` in that worktree.

## Summary

- **Failure 1, the robe stays flared after the flip.** The cause is a fault in the solver's bone-segment collision. That collision keeps the line between two points out of a capsule. When the closest point of that line lies right next to a pinned root, and the root itself sits inside the capsule, the free end of the bone is pushed by the depth divided by its share of the bone. A 7 cm depth at 0.2% of the bone's length throws the point 35 m away in one substep. The solver's step limit then lets the point come back only 8 cm per substep, which is 19 m/s. Meanwhile the bone above it aims at the faraway point, so that panel of the robe sticks out stiffly until the point is back.
- **Failure 2, the robe bunches into a lump at the hip and thigh during the spin.** Three things add up. First, the spin while running is a Jedi Academy animation, so the turn lives in the skeleton's bones and not in the model's root. The solver therefore feels all of it, and the robe is dragged around at its 0.45 m leash. Second, when the KOTOR run clip takes over at the end of the spin, the cast moves the whole facing from the bones to the model's root in one frame, 160 to 165 degrees at once. The solver carries the points 75% of that turn around the body, although the body did not turn. Third, the robe swings into the legs, and the same bone-segment fault throws the front piece's point 8 to 97 m away. The front piece then points away from the body for 0.4 to 5 seconds and shows as the stuck lump.
- **What was ruled out.** The movement and turn caps (cause B) almost never act: the spin is not a root turn. Raising them changed nothing. Points tunnelling through a leg capsule (cause A) happen 0 to 24 times in a whole run and are not the cause. Neighbouring back chains never fold against each other (cause C). They stay within 3 to 21 degrees of each other. Turning off the drive, the leg capsules or the inertia did not remove either failure (cause D).
- **The recording made it worse.** The user's video was recorded at 10 frames per second of game time. The solver then gets 0.1 s per frame and blends the body linearly over 24 substeps. In a fast flip this shrinks the body's shape inside the frame, so pinned roots end up deeper inside capsules. At 10 fps the worst throw in a run was 36 to 130 m. At 60 fps it was 2 to 28 m. Both failures still happen at 60 fps, but shorter.
- **The planned fixes do not reach the main fault.** None of A, B or C fixes the bone-segment push or the step limit. Both are small changes to the solver, and they should come first. The turn handoff also needs its own fix, which the plan does not have.
- **The robe is "not perfect" even when nothing breaks.** Its target pose hangs about 31 degrees back from vertical, so the robe always flares in a stiff A-line. Its spring is very lightly damped and swings for about a second. It is held at its 0.45 m leash through every fast move. The hem points never test the shins and feet. The front piece is one chain that nothing links to the back.

## How it was measured

The L5 agent's script for the recording is in its session scratchpad (`bin/game_all.sh` and `bin/rec.sh`). The command was this:

```
kjk play --stage arena@903mal --class jedi --body pmbim --head pmha01 --camera side \
  --headless --size 960x540 \
  --moves "300ms attack; 1s; 800ms w; 600ms w jump; 2s; 150ms d jump attack; 2.4s; 600ms s jump; 2s; 900ms w; 300ms w crouch; 2s; 1.2s turn 720 w; 1s" \
  --cmds "tick 10; arena clear; wait 10f; tick run; record game-pmbim-on-side 2.5 16 10; dump; quit"
```

The same command, run again, does not give the same timing.
In my runs every move came about 0.7 s later than in the user's video.
Two of my runs also stopped advancing game time, at 9.9 s and 12.8 s.
Each run was therefore read with its own trace, and the moments below are given in each run's own game time.
In my runs the flip lands at about 6.6 s and the spin runs from about 14.2 s to 15.6 s.
In the user's video they are at about 5.9 s and 13.5 s.

The debugging aid on `loose-parts-diag` adds these switches.
- `KJK_LOOSE_TRACE=file` writes one line per frame (game time, step, reset, the model's place and yaw, the root node's turn) and one line per substep. A substep line holds the root's real and felt movement and turn, every capsule, and for each point its place, target, speed, the drive's pull, how much the maximum distance and the step limit moved it, every capsule it is near or inside with its depth and whether it crossed the capsule's axis, every bone-segment push with the share along the bone, and the stretch of its bone.
- `KJK_LOOSE_OVERLAY=1` draws, in play, the 14 capsules (legs in green, the rest in blue), the solver's points (yellow, or larger and red inside a capsule), the bones between them (orange), the links (grey) and a cyan line from each point to its target. `KJK_LOOSE_GHOST=0.5` fades the body.
- `KJK_LOOSE_NO=segments,legs,backstop,drive,collide` turns one part off. `KJK_LOOSE_TURN`, `KJK_LOOSE_TRAVEL`, `KJK_LOOSE_MAXSTEP`, `KJK_LOOSE_RATE` and `KJK_LOOSE_INERTIA` change a setting or a value.
- `KJK_SIDE_CAM="yaw,distance,height,pitch,fov"` places the side camera, which follows the player.
- The log prints each point's values and the capsules it collides with.

I recorded 21 runs: the baseline at 10 and 60 fps from the side and from behind, with a solid body, with the overlay, and with a faded body, then one run for each switch.
I looked at the frames of every moment named below before drawing a conclusion.

## The robe's points

The `pmbim` robe has 15 points.
- Points 0 to 2 are the front piece: its root `frobe1_g`, then `frobe2_g`, then a tip.
- Points 3 to 6 are the right back chain (`rrobe1_g` to `rrobe3_g` and a tip).
- Points 7 to 10 are the left back chain (`lrobe1_g` to `lrobe3_g` and a tip).
- Points 11 to 14 are the middle back chain (`mrobe1_g` to `mrobe3_g` and a tip).
- Points 0, 3, 7 and 11 are pinned roots.
- Six links join the middle chain to the right and the left chains. Nothing links the front piece to the back.

Each free point collides with the 6 capsules nearest to it at rest, within two steps of the pelvis and the belly.
For the knee-height points 5, 6, 9, 10, 13 and 14 those are the pelvis, the belly, the chest, the two upper arms and the two thighs.
The shins and the feet are not among them.

## Failure 1: the robe stays flared after the flip

### What happens, step by step

The best run to watch is `s10-close-solid`, side by side with `s10-close-solidov` in `failure1-flare-10fps-side-solid-and-overlay.mp4`.

1. From 5.8 s to 6.2 s the player is upside down in the flip. The robe's targets point up, 110 to 150 degrees from straight down, and the robe follows them.
2. At 6.3 s to 6.5 s the player lands bent forward. The robe flares out behind, almost horizontal. So far it follows its target.
3. At 6.67 s, in one substep, the bone-segment collision pushes a point of the back by 15.5 m. In the overlay run the same push at 6.64 s is 31.4 m. The overlay shows orange bones running out of the picture behind the player.
4. From 6.7 s to 7.3 s the player stands in his stance. The back panel of the robe stays flared out behind. The bone above the thrown point aims at it, wherever it is.
5. By 7.4 s the point is back within reach and the robe falls.

### The cause

The bone-segment collision is in `segments` in `crates/loose/src/solver.rs`.
For each bone and each capsule it finds the bone's closest point to the capsule's axis, at a share `s` of the way from the parent to the child.
It moves the two ends by these amounts.

```
near_parent = w_parent * (1 - s)
near_point  = w_point * s
sum         = near_parent * (1 - s) + near_point * s
push        = way * (radius - distance) / sum
parent     += push * near_parent
point      += push * near_point
```

When the parent is a pinned root, its weight is 0.
The point then moves by the depth divided by `s`.
The rule is right in theory, because moving the far end by depth / s does move the closest point out by the depth.
In practice it fails when the closest point is the pinned root itself or lies right next to it.
That happens when the animation moves a capsule over the robe's root.
The robe's roots hang on the pelvis, and the belly capsule hangs on the torso.
When the body bends in a flip or a landing, the belly capsule comes closer to the roots than it was at rest.
Then no movement of the free end can clear the contact, and the solver throws the free end far away.

The trace shows this in every run.
- There were 2,782 pushes over 0.3 m in the 21 runs.
- 2,697 of them (97%) had the contact within the first 5% of the bone from its root.
- Every one moved a first free point under a pinned root: point 1 (2,332 pushes), point 4 (284), point 12 (103) or point 8 (63).
- The capsules were the belly (1,056), the pelvis (671), the left thigh (559), the right thigh (215), the right upper arm (177) and the shins (102).
- Examples: a 7.3 cm depth at 0.2% of the bone gave a 35.6 m push. A 10.5 cm depth at 1% gave 10.7 m. A 0.9 cm depth at 0.8% gave 1.1 m.

The thrown point then stays away because of the step limit (`max_step`, 0.08 m per substep).
In `limit`, the solver first clamps each point to its maximum distance from its target, then clamps its movement in this substep to 0.08 m from where it started.
The tether and the maximum distance pull the point straight back, but the step limit undoes them.
The point therefore crawls back at 0.08 m × 240 substeps per second, which is 19.2 m/s.
The trace shows a returning point at exactly 19.09 m/s.
The bone-segment push itself comes after the step limit, so the step limit does not limit the throw.

The 10 fps recording makes this much worse.
At 10 fps the solver gets 0.1 s per frame and blends the capsules and the targets linearly over 24 substeps.
In a flip the body turns about 60 degrees in 0.1 s, and a linear blend of two turned poses shrinks the body by up to 13% inside the frame.
The pinned roots then sit deeper inside the trunk capsules.

| Run | Pushes over 0.3 m | Worst push | Time with a point over 0.95 m from its target |
|---|---|---|---|
| 10 fps, as shipped (4 runs) | 21 to 425 | 36 to 130 m | 2.5 to 13.2 s of 13.5 s |
| 60 fps, as shipped (5 runs) | 6 to 21 | 1.9 to 27.8 m | 0.3 to 1.9 s |
| 10 fps, bone-segment collision off | 0 | none | 0.02 s and 0.00 s (2 runs; one stopped advancing game time at 9.9 s) |
| 60 fps, bone-segment collision off | 0 | none | 0.27 s |
| 10 fps, step limit off (`KJK_LOOSE_MAXSTEP=100`) | 51 and 53 | 11.8 and 16.3 m | 0.07 and 0.08 s |
| 10 fps, movement and turn caps raised | 453 | 82 m | 12.5 s |
| 10 fps, inertia 1 | 482 | 95 m | 7.8 s |
| 10 fps, leg capsules off | 26 | 49 m | 1.7 s |
| 10 fps, drive off | 378 | 98 m | 12.5 s |

`timeline-point-offset-and-segment-pushes.png` shows the largest distance of any point from its target over the run, with each large push as a red dot.
With the bone-segment collision off, no point goes past 0.6 m at 10 fps, and the flare after the flip is gone.
With the step limit off, the pushes still happen, but each one lasts one substep, because the maximum distance pulls the point back at once.

At 60 fps the same flip lands without a stuck flare (`failure1-flare-60fps-side-solid-and-overlay-4x-slow.mp4`).
The robe follows its target within about 0.1 s.
It still swings behind for about a second, which is the lightly damped spring described under "Not perfect".

### Other causes checked

- Tunnelling through a capsule in one substep (cause A) is rare. The trace counts a crossing when a point ends a substep on the other side of a capsule's axis after passing within its radius. There were 0 to 8 crossings in each baseline run. None of them came before a flare.
- The movement and turn caps (cause B) never acted during the flip. The flip turns the body inside the skeleton, and the root node does not turn.
- The drive (cause D) is not the cause. With the drive off, the failure was the same.
- The backstop and the floor did not act during the flare.

### Which planned fixes solve it

- **A, collision that remembers its side.** It does not touch this fault. The point is not on the wrong side of a capsule. It is thrown by the bone-segment rule.
- **A, a backstop from the body surface.** It does not touch this fault either.
- **A, an OGC-style bound on how far a point may move per substep.** The solver already has such a bound, the step limit. It is what keeps the thrown point away. An OGC bound helps only if it also bounds the bone-segment push, and if it does not undo the maximum distance and the tether.
- **A, more substeps.** More substeps per frame reduce the blending error, as 60 fps does. They do not remove the fault: 60 fps still threw points up to 27.8 m.
- **B, angle limits per skirt bone.** A cone would stop the bone from aiming far out, so the panel would not show the flare. The point would still be metres away, and the next frames would still be wrong.
- **C, generated extra skirt bones.** They do not touch this fault.

### What the plan misses

- The bone-segment push must be bounded. It must not move a free point by more than the depth when the other end is pinned. A contact at a pinned root, or within a few centimetres of it, should be skipped or handled by the point's own capsule test.
- The step limit must not override the maximum distance and the tether. It could limit only the free motion before the constraints, or come before them.
- The maximum distance could be applied after the collisions, as a last hard limit.

## Failure 2: the robe bunches into a lump during the spin

### What happens, step by step

The best videos are `failure2-spin-10fps-side-solid-and-overlay.mp4` (run `s10-close-solid` beside `a10-close-overlay`) and, at 60 fps, `failure2-spin-60fps-behind-overlay-4x-slow.mp4`.

1. From 14.2 s the player runs in a circle and spins. The hips turn about 90 degrees every 0.1 s, which is about 900 degrees per second. The model's own yaw stays at 0 the whole time.
2. The robe's points stay roughly where they were while the body turns under them. All four tips sit at their 0.45 m leash from their targets (the trace shows 0.45 for points 6, 10 and 14 through most of the spin). The robe wraps around the body. At 14.8 s in `s10-close-solid` the back panel flies out flat.
3. At 14.96 s the KOTOR run clip takes over. The model's yaw jumps from 0 to minus 160 or minus 165 degrees in one frame, while the hips turn by only about 18 degrees in the world.
4. The solver carries the points 75% of the model's turn around with the root. At 60 fps the jump is 41 degrees per substep, the turn cap lets the points feel 15 of them, and the tips jump to 0.73 to 0.87 m from their targets.
5. In the same frame the robe hits the legs and the belly. The bone-segment fault throws point 1, the front piece's free point, 8.3 m (60 fps) or 96.6 m (10 fps) away, and point 8 up to 15.9 m.
6. From 15.0 s on, the front piece's bone aims at its faraway point. The front piece points 90 to 113 degrees away from its target. It shows as a lump at the front of the hip and thigh. In `s10-close-solid` the lump stays past 15.9 s. At 60 fps it lasts about 0.4 s. From behind, the overlay shows an orange bone from the hip to a point far out of the picture from 14.4 s to past 15.9 s.

### The causes

**The turn of a Jedi Academy move lives in the bones.**
`crates/game/src/cast.rs:6019` places a retargeted body.
While Jedi Academy animations play, it sets the model root's yaw to 0 and puts the facing into the pose of the bones.
Only when a KOTOR clip shows (here the gait's run clip) does it turn the model root to the facing.
The solver treats the model root as the character's movement through the world, and it treats everything inside the skeleton as animation that the points feel in full.
So every flip, cartwheel and spin is felt in full, at any speed.
The movement and turn caps never act during the spin: the trace shows a root turn of 0 degrees per substep from 14.2 s to 14.9 s.

**The facing jumps from the bones to the root in one frame.**
At the switch the model root turns by 160 to 165 degrees, and the bones turn back by about the same amount, so the body stays put.
The solver cannot tell this from a real turn.
With inertia 0.25, it carries each point 75% of the root's turn.
The robe therefore swings about 120 degrees around the body in one frame.
At 60 fps the turn cap of 15 degrees per substep also acts, which adds to the error.
This jump was found in every run, at 14.96 to 15.05 s.

**The bone-segment fault makes the lump stay.**
The swing puts the robe into the legs and the belly, and the fault from failure 1 throws the first free points metres away.
The step limit keeps them away for 0.4 to 5 seconds.

**The front piece is one chain.**
Nothing links it to the back chains, and it is the point the fault throws most (84% of all large pushes).
During the spin it goes 50 to 110 degrees away from its target even when nothing is thrown.
The mesh at the sides of the hips is skinned between the front piece and the back chains, so it folds when they go different ways.

### Evidence from the switches

- With the bone-segment collision off at 60 fps (`ablation-spin-60fps-segment-collision-on-vs-off-4x-slow.mp4`), the robe still wraps around the legs during the spin, but it settles by 15.4 s and makes no lasting lump.
- Raising the movement and turn caps (`KJK_LOOSE_TURN=180 KJK_LOOSE_TRAVEL=10`) changed nothing at 10 fps: 453 large pushes against 21 to 425 in the baseline.
- Inertia 1 keeps the points in the world during the handoff, but it does not stop the spin inside the skeleton or the bone-segment fault. It gave 482 large pushes.
- The two back chains next to each other stay within 3 to 21 degrees of each other during the spin, while their targets are 5 to 6 degrees apart. Cause C does not apply to the back chains.

### Which planned fixes solve it

- **A, more substeps in fast spins.** They do not help. The spin is not a root turn, and the caps do not act during it.
- **A, side memory, the backstop and an OGC bound.** They keep points out of the legs more cleanly. Only a bound that also limits the bone-segment push would stop the throw.
- **B, generous angle limits per skirt bone that move with the thigh.** These would keep the front piece from pointing 90 degrees away from the thigh, so the lump would not show. They do not remove the cause.
- **C, generated extra skirt bones.** Bones between the front piece and the back chains would spread the fold at the hip sides over more bones. They do not remove the cause either.

### What the plan misses

- The solver should feel the character's real turn in the world, not the model root's. For example, it could take the pelvis's world frame as the frame whose movement it partly feels. Then the handoff between bones and root would not show, and a Jedi Academy spin would get the same inertia and limits as a turn of the root.
- Or the cast should keep the facing in one place, so the root never jumps by 160 degrees.
- The bone-segment fault and the step limit, as in failure 1.
- The front piece needs a link to the side chains, or its own collision with the thighs.

## Not perfect, even when nothing breaks

These come from the 60 fps runs, where neither failure happened most of the time.

- **The target pose flares.** The target is the rest pose under the animated pelvis. Standing, the back chains' targets hang 31 to 33 degrees back from straight down, and the front piece's 6 degrees. The robe therefore always stands out behind in a stiff A-line and never falls straight. In a flip its target points up, so it flies up too.
- **The spring is lightly damped.** A tip's drive has a compliance of 0.01 m/N, which is a spring of 100 N/m on a point of 1 kg. That swings at about 10 radians per second. The damping of 1.5 per second gives a damping ratio of about 0.075. After the flip the tips swing 0.1 to 0.4 m around their targets for about a second.
- **The leash holds the robe in every fast move.** The maximum distance of 0.45 m is reached in every flip and spin. While it holds, the hem moves rigidly with the body.
- **The step limit adds lag.** In a flip the hem's target moves faster than 19.2 m/s. The step limit keeps the hem up to 0.9 m behind its target even with the bone-segment collision off.
- **The hem never meets the shins or the feet.** Each point meets only the 6 capsules nearest to it at rest, and for the knee-height points the upper arms win over the shins. In a 60 fps run a point sat inside a leg capsule in about 3% of substeps, up to 5.4 to 6.1 cm deep. Most of these were hem points inside a shin or a foot. The front tip also sits inside a forearm or the chest at times.
- **The front piece swings on its own.** It is a single chain of one bone and a tip, with no links. It is the part that looks wrong most often.
- **The solver depends on the frame rate.** Each frame's body is blended linearly between two poses. At low frame rates, or after a hitch, a fast move is shrunk inside the frame.

## Files

All paths are in `.claude/worktrees/loose-parts-diag/kjk-artifacts/loose-parts-diag/`.

- `failure1-flare-10fps-side-solid-and-overlay.mp4`: the flare after the flip at 10 fps, solid body beside the overlay, played at half speed.
- `failure1-flare-60fps-side-solid-and-overlay-4x-slow.mp4`: the same moment at 60 fps, with no stuck flare.
- `failure2-spin-10fps-side-solid-and-overlay.mp4`: the spin and the lump at 10 fps, solid body beside the overlay on a faded body.
- `failure2-spin-10fps-behind-solid.mp4`: the spin from behind at 10 fps.
- `failure2-spin-60fps-side-solid-and-overlay-4x-slow.mp4` and `failure2-spin-60fps-behind-overlay-4x-slow.mp4`: the spin at 60 fps, played 4 times slower.
- `ablation-spin-60fps-segment-collision-on-vs-off-4x-slow.mp4`: the spin with and without the bone-segment collision.
- `ablation-whole-run-10fps-on-vs-segments-off-vs-maxstep-off.mp4`: the whole run as shipped, without the bone-segment collision, and without the step limit.
- `ablation-whole-run-10fps-caps-inertia-legs-drive.mp4`: the whole run with the caps raised, inertia 1, no leg capsules and no drive.
- `timeline-point-offset-and-segment-pushes.png`: the largest distance of a point from its target over four runs.
- Each video has a `-sheet.png` contact sheet next to it.
- `traces/`: the per-substep traces of every run. `tools/`: the scripts that recorded and read them (`run.sh`, `summary.py`, `runaway.py`, `facing.py`, `flare.py`, `detail.py`, `plot.py`, `video.py`).
- `frames/` and `logs/`: every run's frames and logs.
