# What the original engine does with loose parts

This note was written on 2026-11-19. It is one axis of the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts"). It asks what the original games do with hair, capes, robes and other loose parts, and whether kjk's sway (merged in https://github.com/kjk-engine/kjk/pull/98) leaves out anything the originals have.

The sources are three reference projects, which were only read and not copied. The first is the Odyssey reconstruction of the Knights of the Old Republic 1 (K1) PC executable, version 1.03, at https://github.com/TeamGizka/Odyssey. The second is K12, a port of both games to Android, at https://github.com/shinyquagsire23/K12. The third is reone, a clean-room engine, at https://github.com/modawan/reone. Odyssey and K12 contain readable sway code that agrees with each other. reone contains a simplified version.

## Summary

- The original engine has exactly one mechanism for loose parts, and it is the danglymesh sway that kjk already copies. It has no cloth solver, no collision with the body, no ground or wall test, and no special case for hair, capes or any named model.
- kjk uses every field of the danglymesh data in the same way as the original. The fields are the displacement, the tightness, the period, the constraint of each vertex, and the rest place of each vertex. The formulas, the constants and the order of the steps match the K12 code line by line, and they also match the Odyssey code for K1. I found no difference between K1 and K2 in the formulas.
- A danglymesh is its own node type. A node is either a skinned mesh or a danglymesh, never both. A skinned mesh therefore never sways, and a danglymesh never follows bone weights. A robe that is modelled as a skinned mesh only follows the bones in the original.
- The original does not collide the sway with anything. The only limit is the box around each vertex's rest place. The only protection against fast movement is the same box, a 5 m jump check, and a check that resets a vertex that has flown 10 billion units away.
- The original steps the sway once for each frame in which a mesh passes the view test. It uses the time of the rendered frame, held between 12.5 ms and 35 ms. The sway is therefore tied to the frame rate. At 144 frames per second the spring runs 1.8 times too fast, because the time step cannot go below 12.5 ms. kjk copies this clamp, so kjk has the same dependence.
- The two open questions in the description of PR 98 can now be answered. The original steps the springs when the mesh is drawn, and not when the simulation advances. The original does not step a mesh that is outside the view.
- The original also has other loose-part mechanisms, but none of them is cloth. Particle emitters collide with the world and bounce. Grass blades are small springs pushed by the wind. The lightsaber keeps a short trail of its earlier positions. Some meshes carry keyframed vertex animation that is authored, not simulated. kjk does not play that vertex animation.
- The Android ports of both games removed the sway completely, so in those ports every danglymesh stays at rest.

## 1. Where the sway code lives

In the originals, a danglymesh is a triangle mesh that has an extra header with five kinds of data. In kjk the reader is `crates/fmt_odyssey/src/mdl.rs` and the data is stored in `DanglyIr` in `crates/asset_odyssey/src/model.rs`. The spring is in `crates/game/src/dangly.rs`.

The original steps each danglymesh at the moment the scene collects the meshes that it is about to draw. A mesh that is outside the view planes is not collected, so it is not stepped. A mesh that is collected is stepped before it is drawn. This is done once for every time the scene is drawn. The sway is therefore driven by drawing, and not by the game simulation.

The step receives the time of the last rendered frame. The step holds that time to at least 12.5 ms and at most 35 ms. After it has moved the vertices, it writes them into the model's vertex buffer, so that the drawn mesh and the shadow of the mesh use the swayed places. It then passes the held time on to the node's children.

A developer switch in the original console turns the sway off. When it is off, danglymeshes are not stepped and are not even drawn. Nothing in the shipped data uses this switch.

## 2. The data fields, one by one

### Displacement

The displacement is one number for the whole mesh. It is the largest distance, in metres, that a free vertex may stray from its rest place along each axis.

The original multiplies it by `1 - constraint / 255` for each vertex. A vertex with constraint 0 therefore has the full displacement, but a vertex with constraint 0 is never moved at all, as described below. A vertex with constraint 255 has a limit of 0.

kjk reads it and uses it the same way, in the constant `CONSTRAINT_FULL` and in `Dangly::step`. kjk also uses the displacement as the size of the wind's push, as the original does.

### Tightness

The tightness is one number for the whole mesh. It sets how hard the spring pulls a vertex back to its rest place. The pull for one vertex is the tightness, times 0.5, times the vertex's constraint, times the time step, times the distance from the rest place.

kjk uses it the same way, with `PULL` as the 0.5.

### Period

The period is one number for the whole mesh. It sets how much of the vertex's velocity is lost. In each step the vertex loses its velocity times the period, times 1.5, times the time step.

kjk uses it the same way, with `DAMPING` as the 1.5. The name "period" is misleading. The value does not set a time between swings. It is the damping.

### Constraints

Each vertex has one constraint from 0 to 255, stored as a decimal number. A vertex with constraint 0 is skipped by the step and never moves. The data uses constraint 0 to pin the root of a flap or a hair strand to its bone.

A larger constraint has three effects at once. The spring pulls harder. The box around the rest place becomes smaller. The wind pushes with less reach. At 255 the box has size 0, so the vertex cannot move.

The data can hold a constraint above 255. The limit would then be negative, and the box would flip. kjk's function `reach` allows for this, so it measures the box with the absolute value. The original has no such case. I did not check whether any shipped model has a constraint above 255.

kjk uses the constraints the same way.

### Rest places

Each vertex has a rest place, in the node's own space. The spring pulls the vertex back to it, and the box is centred on it.

The original keeps the rest places in a copy that stays untouched. The swayed places are kept in a second copy. kjk keeps the same two copies in `Dangly::rest` and `Dangly::at`.

The original does not change the normals, the tangents or the texture coordinates of the swayed vertices. Only the positions move. kjk does the same. The shader in `crates/game/src/dangly.wgsl` replaces only the position and keeps the normal from the rest pose.

### Velocity

The velocity of each vertex is not stored in the model. The original keeps it in a separate array for each mesh instance, which starts at zero. kjk keeps it in `Dangly::velocity`, which also starts at zero.

The velocity is not limited. Only the position is held in the box. When a vertex rests against a side of the box, its velocity keeps changing, but it cannot move the vertex past the box. The original and kjk behave in the same way here.

## 3. The order of one step

Both engines do the following for each danglymesh in each frame.

1. They find the node's place and turn in the world.
2. They compare the place with the place from the last step. If it is more than 5 m away, they put every vertex back at its rest place, and they do not clear the velocities. The original does not clear them, and neither does kjk.
3. Otherwise, they compute the travel since the last step, expressed in the node's space of the last step, and the turn since the last step.
4. They compute the wind at the node's place, bring it into the node's space and multiply it by 20.
5. For each vertex with a constraint above 0, they move the vertex and its velocity by the travel and the turn, so that the vertex keeps its place in the world. They then add the spring's pull and the damping to the velocity, and move the vertex by the velocity times the time step.
6. They add the wind times the vertex's own limit.
7. If the vertex is more than 10 billion units away from its rest place along any axis, they put it back at rest. This protects against numbers that have blown up.
8. Otherwise, they clamp each axis separately to the limit around the rest place. The limit is a box and not a ball, so a vertex can be `sqrt(3)` times the displacement away from its rest place along a diagonal.

kjk follows these eight steps in this order. The probes at the end of `crates/game/src/dangly.rs` test steps 2, 3, 5, 6 and 8.

## 4. What the original has that kjk does not, or does differently

### Fast turns, flips and teleports

The original has only two protections. The first is the 5 m jump check, which resets every vertex to its rest place. The second is the 10 billion check, which resets one vertex.

A fast turn needs no extra protection. The step moves each vertex by the turn, so a vertex that is 0.5 m from the node's origin and turns by 90 degrees would be about 0.7 m from its rest place. The box then clamps it to a few centimetres. A fast turn therefore pins the free vertices against the side of their box in the first frame. The same happens in kjk.

A character that flips or jumps quickly gets the same result. The moves are never larger than the box, so the sway looks stiff and does not look wild. This is the reason why the sway in the original is subtle, and it is also the reason why kjk's acrobatics will not make capes fly while only this mechanism is used.

I did not find any limit on the number of frames or on the speed of the node. The jump check measures the distance moved in one step and nothing else.

### Frame-rate dependence

The original holds the time step to between 12.5 ms and 35 ms. This was meant to keep a short frame from exploding and a long frame from freezing the sway. The two ends have different effects.

- At 60 frames per second a frame lasts 16.7 ms, which is inside the range, so the sway runs in real time.
- At 144 frames per second a frame lasts 6.9 ms, but the step uses 12.5 ms. The spring and the damping therefore run 1.8 times faster than real time. At 240 frames per second the factor is 3.
- At 20 frames per second a frame lasts 50 ms, but the step uses 35 ms. The spring then runs at 0.7 times real time.

The travel and the turn are applied exactly, once for each frame. Only the spring and the damping use the held time. At high frame rates the spring is therefore too stiff and too quick compared with the way the vertices are carried.

K12 describes this as a bug, and its fix removes the 12.5 ms lower limit, so that the time step can go as low as the real frame time. K12 also passes the held time to the children of the node, which made them run too fast in the original. kjk copies the clamp in the constants `DT_MIN` and `DT_MAX` and in the line `dt.clamp(DT_MIN, DT_MAX)` in `sway`.

kjk therefore shows the same frame-rate dependence as the original. This is a difference from the original only for a player who uses a high refresh rate, because the original game was normally played at 60 frames per second or less. The explicit step is also stable only while the spring is not too strong. I worked out, as my own estimate, that the pull for each step must stay below about 4 divided by the square of the time step. I did not check this against the values of the shipped models.

### Time held while paused

kjk steps the springs by the game's clock. When the clock is held, as in the `anim` and `record` modes, kjk steps them by the amount the clock moved, and not at all if the clock is frozen. This was needed so that recorded frames repeat.

The original uses the time of the rendered frame, and does not look at whether the simulation is paused. As far as the code shows, a paused game keeps swaying its hair and keeps blowing its leaves. This answers the first open question in the description of PR 98. I could not test this against the running game.

### Meshes outside the view

kjk skips a mesh that the camera did not see in the last frame. The original skips a mesh that is outside the view planes in the current frame. The two are close. A mesh that comes into view in kjk uses the place from the frame before, so that it does not jump. In the original, a mesh that was outside the view for a long time has an old place from its last step. When it comes back into view and is more than 5 m from that place, it is reset. When it is closer, it lurches by the whole travel.

This answers the second open question in the description of PR 98. The original does not animate parts that it does not draw.

### Node scale

The original takes the node's place and turn from the world transform and ignores its scale. kjk divides the travel and the wind by the scale in `Dangly::step`. The two are the same for a node with a scale of 1. I did not check whether any shipped danglymesh has a scale other than 1.

### The first frame and an identity turn

The original decides that it is on its first step when the stored turn of the last frame has zero in its x, y and z parts. It then takes the current place and turn as the "last" ones, so that the first step has no travel. This is also true in every later frame in which the stored turn is exactly the identity turn, which has zero in those three parts.

A node whose world turn is exactly the identity turn would therefore never see any travel. This is almost certainly never the case for a character, because the bones turn it. It could be the case for a static object with a tree leaf, but a static object has no travel either way.

kjk uses an empty value for the first frame and does not have this quirk. I do not think kjk needs to copy it.

### Several characters sharing one model

Both reconstructions write the swayed vertices of every instance into the vertex buffer of the model, at the node's own offset. If two characters share the same model, they may share one swayed shape in the original. I could not confirm which buffer the drawing reads from. kjk gives every instance its own slots in `SwaySlots`, so kjk does not have this problem, and it should not copy it. This is marked as unverified.

## 5. Things I looked for and did not find

- No collision of a danglymesh with the character's body, the floor, a wall or another character. The sway code never asks the world anything except the wind.
- No special case for hair, capes, robes or any named model. The code receives only the five kinds of data listed above.
- No skin and dangly combination. The node types are separate and the engine only accepts the exact combinations of node flags that it knows. A skinned mesh cannot hold danglymesh data, and a danglymesh cannot hold bone weights.
- No difference between K1 and K2 in the formulas or in the constants. The Odyssey code for K1 and the K12 code for K2 use the same time limits, the same factors 0.5, 1.5 and 20, the same 5 m jump check and the same 10 billion check.
- No use of the animation of the mesh's own bones other than through the node's world transform. The sway reacts to a bone's animation because the node moves with the bone.
- No cloth flag on a model. K12 contains a comment that calls the danglymesh a "cloth-like mesh". That is a description by the people who wrote it, and not a separate feature.

## 6. Other loose-part mechanisms in the originals

### Particle emitters

An emitter makes particles that have gravity, drag, mass, a random velocity and a wind push. A particle can test its movement against the world with a line test. When the line hits a surface, the particle bounces. The bounce uses the emitter's own bounce coefficient. The engine has two bounce modes, a damped one and an elastic one, and a particle comes to rest when its speed or its angle is small enough.

This is the only place where the original asks the world about a loose part. It is used for sparks and debris, and not for cloth. I did not find any emitter attached to a cape or a robe in the code. The models' data might still have one, but I did not check any model files.

### Grass

Grass blades are small springs. Each blade is pulled back to its base and pushed by the wind and by wind from point sources. The step time for the grass is held to at most 0.1 s, and the wind is capped at a magnitude of 5. This is separate from the danglymesh sway. It lives in the terrain code.

### Lightsaber trail

A lightsaber blade keeps a ring of its earlier positions, with a life of 12 ms for each one, to draw the swing trail. The ring starts again when the blade moves more than 1 m between two steps. This is like the 5 m jump check of the danglymesh, with a smaller distance.

### Keyframed vertex animation

A different kind of mesh, the animated mesh, stores vertex positions and texture coordinates as keyframes with a sample period. The engine blends between two keyframes. This is authored by hand, and does not react to movement or wind.

kjk's MDL reader steps over the header of this kind of mesh and does not play the keyframes. I did not check which shipped models use it.

### Mechanisms that do not exist

None of the mechanisms above is a ragdoll, a cloth solver, soft-body physics or a verlet integrator. A search of all three reference projects for these terms found nothing.

## 7. The other reference projects

K12's Android ports removed the sway at build time. The code for it exists, but it is switched off in the shipped mobile build. Danglymeshes in those builds always stay at their rest places, and the vertex array that holds the velocities is allocated and never used. This is a performance choice by the porters and not an original design.

reone, the clean-room engine, uses the same two factors and the same time limits, but it simplifies the rest. It handles only the translation of the node and not its turn. It uses a made-up wind, the absolute value of a sine wave along one axis, with a strength of 0.01. It limits the mesh to 768 vertices because of the size of its uniform buffer. It is therefore less faithful than kjk, and it gives nothing that kjk lacks.

## 8. What this means for kjk

The original gives kjk no further mechanism to copy for loose parts. Everything in the original is the danglymesh data, and kjk already uses all of it.

Any behaviour beyond the sway would therefore not come from the games' own data. This includes collision of loose parts with the body, a ground test, a larger box for acrobatics, a sway for skinned robes, and a cloth solver. The owner rejects effects that are not built from the games' own data, so each of these needs an explicit decision, and the other axes of this issue cover how they could be built.

Two small items could be taken from this note without any new feature.

1. Decide whether the 12.5 ms lower limit on the sway's time step should stay. It makes the springs run too fast at high refresh rates. K12 removes it. Removing it would not change anything at 60 frames per second or below, because those frames last at least 16.7 ms. It would only change the sway at frame rates above 80.
2. Decide whether a paused or frozen game should keep swaying. The original does, because it is driven by drawing. kjk stops the sway when its clock is frozen, which is needed for recorded frames, but could keep it for a normal pause.

## 9. What I could not verify

- Whether the running game really steps the sway while the simulation is paused. This follows from the code, but I did not run the game.
- Which vertex buffer the original reads when it draws a danglymesh that several characters share.
- Whether any shipped model has a constraint above 255, a danglymesh node with a scale other than 1, or an emitter attached to a cape or robe. This needs the game's model files, and this note was written from source code only.
- Which shipped models use keyframed vertex animation.
- Whether the first frame of a mesh in the original starts from the origin or from the node's place. This would only matter for the 5 m reset check on the first frame, and the effect is only a reset to rest.
