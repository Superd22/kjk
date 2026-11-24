# How games of the same family and era handle robes and capes

This document was researched on 2026-11-19.
It serves the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts").
It answers one question.
How did Jedi Academy, the Aurora engine games, The Force Unleashed and KOTOR modders make robes and capes move, and what can kjk borrow?

Every claim carries the URL of the page it was read on.
The words "not verified" mark every claim that could not be confirmed on a primary source.
Several forum sites refused automated reads (JKHub, Deadly Stream and the Star Wars Jedi: Survivor EA forum returned errors or blocks).
For those sites, only the short excerpts shown by search results were available, and they are marked as such.
No game was run, no source code was read, and no video was watched.
This was a web-only pass, so the findings about Jedi Academy mods are thinner than they should be.

## Summary

- None of the games of the same family and era found here shipped real cloth simulation on a robe or cape. Jedi Academy, KOTOR, Neverwinter Nights and The Force Unleashed all used skinned bones, baked animation or a vertex sway like kjk's. The Force Unleashed team said they deliberately avoided flowing Jedi robes.
- Jedi Academy has no cloth. Players asked for it for years. The OpenJK team called it very low priority. Modders who wanted cape motion added extra "cloth bones" to the skeleton and baked a simulation onto them in 3ds Max. This was judged too much work because about 1,000 animations would need the new bones.
- Neverwinter Nights has the same danglymesh as KOTOR. Its players call it "not true physics" and say it does not stop clipping. Capes there are skinned like robes, and the usual complaint is clipping through armour.
- KOTOR modders did not change the danglymesh data to get more motion. They added skirt bones to the player's skeleton and animated the cloak by hand. Their stated limit is that the engine has no cloth support. Clipping through the legs is accepted as normal.
- The one clearly documented game-grade method found is Remedy's Alan Wake jacket. It simulates a low-resolution mesh with Verlet integration, drives bones from it, collides with a few ellipsoids on the legs and body, and clamps every vertex to a maximum distance of about 40 cm from its skinned rest position. That last clamp is what kept the cloth stable in fast moves.
- The best fit for kjk is a mix. Keep the games' danglymesh data as the source of which vertices move. Add a Verlet pass with a larger, setting-controlled travel limit. Use the capsules the ragdolls already fit as colliders. Always clamp each vertex to a distance from its skinned position.

## Star Wars Jedi Knight: Jedi Academy and Jedi Outcast

### What the engine does

Characters in both games use the Ghoul2 system, which skins a mesh on a bone skeleton.
No cloth simulation exists in it.
A player model page on JKHub says long gowns and clothing follow the existing skeletal animations rather than any cloth physics (found through a search result for https://jkhub.org/files/category/5-player-models/, not read in full, not verified).
A JKHub comment explains that a cape is a separate mesh surface, and that it can be turned off by editing the model's .skin file so the surface is hidden (index of search results for https://jkhub.org/topic/2394-cloth-sim-experiments, not verified).
This means a Jedi Academy cape is a normal skinned surface and moves only as far as the bones move it.

### OpenJK

OpenJK is the community fork of the Jedi Academy and Jedi Outcast engines (https://jkhub.org/files/file/3856-openjk-for-jedi-academy/).
A JKHub thread titled "Physics for OpenJK" records a maintainer's answer that cloth physics is "very low priority" and would not be considered for "a good few weeks or possibly months" while stability work continued (https://jkhub.org/forums/topic/4754-physics-for-openjk/, quoted from a search result, the page itself could not be read).
The same thread proposes a method.
A mapper would write a "cloth" keyword in a texture shader, and the engine would then simulate every surface that uses that shader.
No evidence was found that this was ever built.
The OpenJK fork "Jedi Academy: Enhanced" adds RGB sabers and new Force powers.
It was not found to add cloth (https://www.moddb.com/mods/jedi-academy-enhanced).

### Movie Duels

Movie Duels is a Jedi Academy mod and remaster of the 2009 Movie Duels 2 that runs on a heavily modified OpenJK, according to one readme (https://www.nexusmods.com/starwarsjediknightjediacademy/mods/45).
The sixth major update arrived in January 2026 and the 6.5.2 readme is dated 2024-05-04 (https://www.moddb.com/mods/movie-duels).
Its update notes mention "updated Kit Fisto's cloth textures", which changes a texture and not a simulation (https://www.moddb.com/mods/movie-duels/downloads/md-update6-part1).
No cape or cloth physics was found for Movie Duels.
The claim that Movie Duels has none is therefore "not found", not "verified absent".

### The cloth bone experiments

A JKHub thread named "Cloth sim experiments" is the best record of what modders tried (https://jkhub.org/forums/topic/2394-cloth-sim-experiments/page/3, read through search result excerpts only).
One modder wrote that they were building their own cape cloth simulation with "fairly decent results".
The method was to convert the compiled GLA skeleton file to FBX with Noesis, add cape bones in 3ds Max, simulate the cape there, and bake the result onto the bones.
A later post suggested that about six baked bones would keep most of the cloth motion in the game.
Another poster pointed out that the skeleton file has just over 1,000 animations, and every one would need the new cloth bones animated.
A third said the engine would likely reject the larger skeleton without code changes.
A related thread, "Flowing robes", discusses the same bone approach (https://jkhub.org/topic/10600-flowing-robes, not read).
A commenter in the thread "Please make cloth physics!" said that bone animation cannot make the cloth feel heavier or lighter, and expected real cloth only if someone added a physics library (https://jkhub.org/topic/2242-please-make-cloth-physics/page-2, search excerpt only).
The planned Jedi Knight Galaxies skeleton lists cape bones meant to drive a cloth mesh, and they were still a goal and not a shipped feature (https://www.moddb.com/mods/jkgalaxies/images/start-of-jkg-skeleton).

### Behaviour during acrobatics

No source describing how a Jedi Academy cape behaves during flips, spins or wall jumps was found.
Because the cape is a skinned surface, it follows the skeleton.
It cannot flutter, trail or react to speed.
It clips through the body when the bones bend far.
One cape mod mentions "some slight clipping issues" (search result excerpt of https://jkhub.org/files/category/5-player-models/, not verified).
Whether that clipping is also seen in fast moves is not verified.

## Neverwinter Nights and Neverwinter Nights 2

The Aurora engine of Neverwinter Nights shares the danglymesh feature with KOTOR.
A Beamdog forum thread "New Capes and Cloaks: animations or physics? Clipping?" describes it well (https://forums.beamdog.com/discussion/comment/964679).
One member says danglymesh "isn't true physics".
It compares the model's previous and current position and adds a wind value.
Another member says it moves vertices within a set space.
Members agree it does not "deform around other objects", so it does nothing about clipping with armour or robes.
Capes are made like robes, by modelling the cape and skinning it.
Each body type and race needs its own cape model.
One member counted that a cape for every body would need about 10 times the number of body types as models.
One member preferred the older animations to the stiffer ones that cloaks forced onto characters.
Another member made invisible cloak versions for a persistent world.
A forum post claims clipping and lack of time were the reasons given for leaving some cloaks and flowing robes out of the original game (https://rpgcodex.net/forums/threads/15035/unread, a forum claim, not verified).

For Neverwinter Nights 2, no source describing how its cloaks and robes work was found.
The only claim available is from memory and is not verified.
It is that Neverwinter Nights 2 cloaks are skinned meshes with no cloth simulation.
A flag mod on Nexus Mods was simulated in 3ds Max and baked into an animation, which hints at a baked workflow, but it says nothing about the game's characters (https://www.nexusmods.com/neverwinter2/mods/904).

## The Witcher 1, Jade Empire and Mass Effect

- The Witcher uses a heavily modified Aurora engine, and one wiki says about 80% of it was rewritten for DirectX 9 (https://witcher.gamepedia.com/Technical_information). Its rag dolls are documented. No source on cloth for Geralt's clothing was found. Everything about cloaks in the Witcher results was about The Witcher 3 and is not relevant here.
- Jade Empire: no source was found at all. Nothing is claimed.
- Mass Effect 1: no source was found on its robes. Unreal Engine 3 includes a PhysX cloth system that runs on the CPU, needs CPU skinning and is off by default (https://udn.epicgames.com/Three/PhysXClothReference.html). Whether Mass Effect used it is not verified.

## Star Wars: The Force Unleashed

The game uses three physics systems.
Havok moves rigid objects, Digital Molecular Matter decides what materials are made of, and Euphoria drives how enemies react (https://www.engadget.com/2008/01/26/the-three-physics-engines-of-the-force-unleashed/).
That article does not mention cloth, capes or robes.
A GDC 2007 report says LucasArts partnered with Industrial Light and Magic for cloth and fur (https://slashdot.org/story/07/03/08/187225/gdc-lucasarts-and-the-force-unleashed, search excerpt).
An Animation World Network feature says most characters did not get cloth simulation, for example flowing robes on Jedi, and quotes a team member: "We wanted to stay away from those as much as possible" (https://www.awn.com/vfxworld/star-wars-force-unleashed-lucasarts-goes-next-gen, search excerpt, the page could not be read in full).
This is a studio with large budgets choosing to avoid the problem.
Its lesson for kjk is that a Jedi robe is hard to make look good over acrobatic play.

## Newer Star Wars games, for the player's view

Players of Star Wars Jedi: Survivor posted that the Jedi robes had no cloth movement and stayed fixed to Cal's legs.
Another player called the cloth on some jackets and costumes "stiff and glued" and named the Jedi attire the worst offender (https://forums.ea.com/discussions/star-wars-jedi-survivor-en/jedi-survivor-jedi-robes-broken-physics/10712139, search excerpt).
Other players reported capes that stretched across the screen on enemies, and one said a cape blocked the view from several rooms away.
These are anecdotal reports from 2023 and 2024.
They show two failure modes players notice at once.
Stiff cloth that looks glued is disliked.
Stretched cloth that fills the screen is disliked more.

## KOTOR mods

### Cloaks and robes

JCarter426's "JC's Fashion Line I" mods add cloaks to the Jedi robes the player and many NPCs wear (https://deadlystream.com/files/file/1378-jcs-fashion-line-i-cloaked-jedi-robes-for-k1/, search excerpts only).
The mod adds skirt bones to the player supermodels.
It does not add them to the cutscene models, so the skirt flaps may look stiff in scenes that use those models.
A commenter on its thread said there is clipping, but no more than in the usual KOTOR 2 robes.
The same commenter said there is little room to improve it because the engine has no cloth physics.
The same commenter said the skirt flaps on Bastila's and Jolee's models are danglymeshes, not animations, and move only by wind and very limited physics (https://deadlystream.com/topic/8216-modjcs-fashion-line-i-cloaked-hybrid-robes-for-k1/).
The "Revan's hoodless/maskless flowing robes" mod depends on cloak and belt animations from another modder's pack, and without that pack there is no cloak animation at all (https://deadlystream.com/files/file/1099-revans-hoodlessmaskless-flowing-robes-for-k1/).
The Yavin IV mask mod chose mesh rigging and hand-made animation over any cloth simulation, and offers an animated cloak as an experimental extra (https://deadlystream.com/topic/12167-modyavin-iv-by-jenkeee/).

### Danglymesh tuning

A Deadly Stream thread titled "problem with danglymeshes working only on some meshes" shows that modders know the feature and fight it (https://deadlystream.com/topic/8989-problem-with-danglymeshes-working-only-on-some-meshes, title only, the page could not be read).
No mod was found that changed the danglymesh values (constraint, displacement, tightness or period) to get larger motion.
This is "not found", not "does not exist".
The pattern is clear, though.
When KOTOR modders wanted visible cloth motion, they went to bones and animation, not to the danglymesh data.
No player opinion about the danglymesh sway itself was found, other than the remark that it is very limited.

## The one detailed method: Alan Wake

Alan Wake is not of the same family or era, but it is the only game-grade writeup found that matches kjk's needs (https://www.gamedeveloper.com/programming/the-secrets-of-cloth-simulation-in-i-alan-wake-i-).
It describes a character's jacket and veil, and the details are these.

- The simulation uses Verlet integration. Each vertex velocity comes from its current and previous position, is damped, and is pushed by gravity and wind.
- A relaxation pass pulls connected vertices back to their rest lengths. Stiffness scales the bend and shear corrections.
- Vertical stretch is removed in one pass sorted from top to bottom. Horizontal stretch is left alone because it is less visible.
- A low-resolution mesh is simulated and drives a higher-resolution render mesh through bones.
- The legs and body are approximated by ellipsoids. Each ellipsoid is scaled into a sphere so a cheap sphere test can be used, then scaled back.
- Every vertex has a maximum distance from its skinned rest position. If the simulation exceeds it, the vertex is pulled back. The limit is larger at the hem than near the shoulders. About 40 cm was the largest value before knots and clipping appeared.
- The author found the distance limit simpler and more effective than collision planes.
- The simulation runs in local space with only 10% to 30% of the character's world-space motion added. That looked better than full world space.
- Vertices that touch the body get more damping. The jacket looks calm when the character walks and livelier when the character dodges or jumps.
- A fixed time step keeps stiffness consistent.

Other engines document the same collider idea.
Baldur's Gate 3 authors cloth colliders as spheres attached to bones such as hips, knees and ankles, each with a position offset and a radius (https://bg3.wiki/wiki/Modding:Cloth_colliders).
Unreal Engine 3 keeps cloth off by default because most meshes do not need it (https://udn.epicgames.com/Three/PhysXClothReference.html).
These two pages do not discuss fast moves.

## What kjk could borrow

1. Keep the danglymesh data as the authority on which vertices are free. The constraint value of a vertex can scale how far it may stray. This keeps the effect built from the games' own data, which the owner requires. It also matches what Alan Wake did with a per-vertex distance limit.
2. Add a maximum distance from the skinned position as the main safety rule. In kjk this would be the existing box, made larger by a setting. Alan Wake reached 40 cm before artifacts appeared. The original games allow 2.5 to 3.5 cm for most meshes and 12.5 cm for Juhani's cloth. A setting between those two values is a reasonable first range to test.
3. Use the capsules the ragdolls already fit to each humanoid body as colliders. This follows the Alan Wake ellipsoids and the Baldur's Gate 3 bone-attached spheres. Collision then only has to push vertices out of 14 capsules.
4. Simulate in the character's local space and add only 10% to 30% of the world-space motion. This keeps a spinning or flipping character from throwing the cloth about. Alan Wake used this for the same reason.
5. Give vertices that touch the body more damping. This keeps a walking character calm and lets a flipping character look livelier.
6. Use a fixed time step. kjk's ragdolls already use 120 steps a second, so the same step can be reused.
7. Keep the effect out of the simulation. This is also how the sway works today. Co-op players do not have to agree on it, and the deterministic skeleton stays untouched.
8. Offer stronger motion as a setting, with the original sway as the default. KOTOR modders' choice of bones over danglymesh data suggests players do want more motion than the data allows.
9. Check what the skinned skirt does first. The KOTOR modders' note that skirt flaps on some models are danglymeshes, while others follow skirt bones, matches the roadmap's open question about which models have danglymesh data.

## What kjk should avoid

1. Do not add cloth bones and baked animation. The Jedi Academy modders found that every one of the 1,000 animations needed new work. kjk has far more movement than the originals, so baked motion cannot cover it.
2. Do not rely on a skinned cape with no motion of its own. It clips and looks "glued", which is what Jedi: Survivor players complained about.
3. Do not let the vertices follow the character's world velocity without a limit. Stretched capes filling the screen are the most visible failure in the player reports.
4. Do not simulate every vertex of a high-resolution mesh. Alan Wake simulated a low-resolution mesh and drove the render mesh from it. kjk's CPU cost for crowds is not yet measured.
5. Do not add a new physics library for cloth. Rapier and the capsules are already in the engine. The Jedi Academy thread said real cloth would need a physics library, but kjk has one.
6. Do not promise flowing Jedi robes as a first target without testing. The Force Unleashed team avoided them on purpose, and Jedi: Survivor shipped them stiff.
7. Do not take the "cloth" shader keyword idea from the OpenJK thread. It adds a new authored tag that the games' data does not have.

## Gaps that remain

- No source on Jedi Academy cape behaviour in flips and wall jumps was found.
- No source on Jade Empire, Mass Effect 1 or Neverwinter Nights 2 character cloth was found.
- No KOTOR mod that changed danglymesh values was found, and no player opinion on the sway itself.
- JKHub and Deadly Stream pages should be read in a browser to confirm the excerpts used above.
- The Jedi Academy source on GitHub (OpenJK) was not read, so it is not verified that the engine has no cloth code.
