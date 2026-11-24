# Seamless loading prior art

Date: 2026-11-24.

David wants to judge how hard it would be to remove the loading screens between levels (modules) in kjk. This note collects how other games, remakes, source ports and mods have handled the same problem. It ends with lessons for kjk. The context is in `docs/research/2026-11-24-module-loading.md` and `docs/research/2026-11-24-level-transitions.md`.

This was a quick pass. Many primary pages blocked the fetch tool (Nexus Mods and the Valve Developer Wiki returned HTTP 403), so some claims come from search summaries or press articles. Every claim carries a link, and each one that is secondary, from a forum or unverified says so.

## Short answer

Every engine that avoids loading screens does one of three things.

1. It keeps one continuous world and streams pieces of it in and out (Unreal World Partition, Spider-Man, Horizon).
2. It hides the load behind something the player must wait for anyway, such as a door, an elevator or a tunnel (Metroid Prime, Mass Effect 1).
3. It keeps two separate spaces alive for a short time and makes the player cross between them at one agreed point (Half-Life and Source landmarks, Portal style portals).

kjk's idea of a second world placed far away behind a portal belongs to the third group. It is the same idea as the Source engine's landmark transition, with a portal added so the player can see the other side before crossing.

## Modern games

### World Partition and level streaming in Unreal Engine

Unreal Engine 5 keeps the whole world in one persistent level and splits it into grid cells. Streaming sources, such as the player controller, decide which cells load. The cell size, loading range and priority are settings. The default is a 2D runtime hash grid, and the documentation warns that using several grids can hurt performance. See the [World Partition documentation](https://dev.epicgames.com/documentation/en-us/unreal-engine/world-partition-in-unreal-engine).

The design cost is that the whole world must share one coordinate space. Epic calls the older system, World Composition, legacy and recommends World Partition for new projects, according to the [World Composition documentation](https://dev.epicgames.com/documentation/unreal-engine/world-composition-in-unreal-engine). The older system used world origin shifting so that worlds could be bigger than the engine's maximum world size, per the [UE 4.26 World Browser documentation](https://docs.unrealengine.com/4.26/BuildingWorlds/LevelStreaming/WorldBrowser). Actors that reference each other load as one bundle, and actors referenced from the level blueprint are always loaded, per the World Partition page above.

Unreal also supports volume-based level streaming. A level loads when the viewpoint enters a volume and unloads when it leaves. See [Level Streaming Overview](https://dev.epicgames.com/documentation/unreal-engine/level-streaming-overview-in-unreal-engine). This is the closest match to loading a neighbouring module when the character nears a door. The levels still have to be authored to line up in one shared space.

### Open world streaming in Spider-Man and Horizon Zero Dawn

Insomniac's Marvel's Spider-Man had a GDC 2019 technical postmortem by Elan Ruskin. The session description says it covered adapting rendering, streaming and lighting to fit a New York City on a Blu-ray disc ([session summary on 80.lv](https://80.lv/articles/marvels-spider-man-a-technical-postmortem)). The slides are on the members-only [GDC Vault](https://gdcvault.com/play/1026496), so no memory numbers were read for this note.

A press report says the swing speed was reduced so the PS4 could keep up with streaming ([GamingBolt](https://gamingbolt.com/10-things-video-games-do-secretly/6)). This is a secondary source, but it shows the usual cost. The speed at which the player can move is limited by the speed at which the disk can deliver data.

Guerrilla Games gave a talk called [Streaming the World of Horizon Zero Dawn](https://www.guerrilla-games.com/read/Streaming-the-World-of-Horizon-Zero-Dawn). The page says it covers the asset pipeline, low-level and high-level streaming, memory management and scheduling. The page itself has no numbers, and the slides were not read.

### Corridors, doors and elevators

Metroid Prime used doors as hidden loading screens. A former Retro Studios engineer, Zoid Kirsch, said the next room starts loading when the player touches an invisible trigger. The door opens only after the room has finished loading. If the room is not ready, the door stays shut and the player waits. See the [Siliconera interview summary](https://www.siliconera.com/former-retro-studios-developer-shared-metroid-prime-trivia/). The cost is a design constraint. Every room connection must be a door or a similar choke point.

Mass Effect 1 used elevator rides to avoid showing loading screens. Writer Thomas Roy said the elevators were made "so we didn't have to show boring loading screens". Casey Hudson said they would be removed in Mass Effect 2 and replaced by a loading screen. See [Kotaku on the transition system](https://kotaku.com/completely-new-transition-system-in-mass-effect-2-is-5419999) and [Giant Bomb on elevators that mask loading](https://giantbomb.com/wiki/Concepts/Elevators_That_Mask_Loading_Times). The search results also say the Mass Effect 3 Normandy elevator was cut because its load time kept growing. That detail came from a six-page developer interview, and the page was not opened. The lesson is that a hiding trick fails when the load takes longer than the trick lasts.

God of War (2018) has no camera cuts. Cory Barlog said he wanted "no cinematics, no camera cuts" and that the player would never lose camera control ([GamesBeat interview](https://gamesbeat.com/god-of-war-full-interview-the-definitive-story-behind-the-crafting-of-god-of-war/5)). The interviews found do not describe how streaming is hidden, and the GDC 2019 talk found is a design talk. No primary source for the loading technique was found, so this note makes no claim about it. God of War Ragnarok was not researched.

### Portals between separate spaces

Portal (2007) draws the view through a portal by rendering the scene again for each portal and using the stencil buffer to limit where each pass draws. A Valve postmortem in Game Developer says the team used a stencil method that gave flexibility with recursion depth, per [a student write-up quoting it](https://torinmr.github.io/cs148). A quote attributed to Valve's David Kircher gives a maximum of nine recursive views. That quote comes from a [Chinese blog](https://cnblogs.com/LovelyTheKnife/p/5115389.html), so treat it with caution.

The cost is rendering the scene again for each visible portal. A portal that shows only one other space at one door needs one extra pass, which is much cheaper than Portal's recursive case.

Ratchet and Clank: Rift Apart uses rifts that swap the visible world. Insomniac's creative director Marcus Smith said the dimension hopping would not have been possible without the PS5 solid state drive ([GamesRadar](https://www.gamesradar.com/ratchet-and-clank-rift-apart-would-not-have-been-possible-without-ps5s-faster-loading-times/)). The technology director said the game moved to the streaming system used in Spider-Man ([GamingBolt](https://gamingbolt.com/ratchet-and-clank-rift-aparts-streaming-tech-allows-for-more-density-content-and-quality-in-every-corner-insomniac)). A competing claim from Traveller's Tales founder Jon Burton, that the rifts could have been done on the PS3, is only an opinion ([VGC](https://www.videogameschronicle.com/news/ratchet-clanks-rifts-could-easily-have-been-done-on-ps3-claims-travellers-tales-founder/)). The practical point is that the rift swap needs the second world to be loadable quickly, which is a storage speed requirement.

Prey (2017) and It Takes Two were searched for and no primary or reliable source was found about their transition technique. The Prey search results were about its long load times, and they say nothing about hidden loading. This note makes no claim about either game.

## Older games, remakes and mods

### Half-Life, Source and Black Mesa

Half-Life is the closest match to KOTOR's doors. A level change needs a `trigger_changelevel` in each map and an `info_landmark` in each map. The two landmarks share a name, and that name is set in the trigger. The landmark is the reference point used to place the player and carried entities in the new map. See [info_landmark (GoldSrc)](https://developer.valvesoftware.com/wiki/Info_landmark_(GoldSrc)) and [trigger_changelevel (GoldSrc)](https://developer.valvesoftware.com/wiki/Trigger_changelevel_(GoldSrc)), both read through search summaries because the pages returned HTTP 403.

Only entities inside the area of a matching `trigger_transition` are taken to the new level, per [TWHL's tutorial](https://twhl.info/wiki/page/Tutorial%3A_Changing_Levels/6303), also read through a summary. Valve's guidance for a seamless look is to put the same brushes and entities in the visible area on both sides of the change, per the same summary.

This matters for kjk because Source solved the origin problem with a named reference point in each level. Each level keeps its own origin. The engine computes the offset between the two landmarks at transition time. Half-Life still shows a short load at the change, so it is a seamless look, not seamless streaming.

Black Mesa kept this system. A PC Gamer review of the early access version says the loading points between maps are frequent and that the opening tram ride is split into three parts because of loading pauses ([PC Gamer](https://www.pcgamer.com/black-mesa-review-early-access)). The review is from early access, so the current state is unknown. No source was found on whether Xen uses seamless transitions.

### Starfield mods

David remembered a Starfield mod that removed loading. The search found five different ones, and no source says which one he saw. The best match for "removes loading between levels" is Seamless City Interiors.

| Mod | Author | What it does | Source |
|---|---|---|---|
| Seamless City Interiors | PanConKeso | Tells the game that interior locations are exterior locations, so entering a building shows no loading screen. | [PCGamesN, 4 November 2024](https://www.pcgamesn.com/starfield/seamless-city-travel-mod) |
| Neon core relocation | Same Nexus page, 17340 | Moves the Neon core into the exterior world space. The author notes Neon still has loading screens. | [Nexus 17340](https://www.nexusmods.com/starfield/mods/17340), read through a search summary because the page returned HTTP 403 |
| Seamless elevators | EdgiestSwing23 | Replaces 18 elevators at 15 locations with movable elevators that need no loading. | [Wccftech](https://wccftech.com/starfield-mod-adds-real-elevators-with-no-loading-screen/) |
| Seamless Planet Takeoffs | 0xBobby | Removes the loading screen when leaving a planet. It is a beta. | [PC Gamer](https://www.pcgamer.com/games/rpg/a-modder-has-brought-no-mans-sky-style-seamless-takeoffs-to-starfield-and-even-in-beta-they-look-pretty-great/) |
| Seamless Landings | Frederik Wessberg | Removes the loading screen when landing. The headline says it took over a year. | [GamesRadar](https://www.gamesradar.com/games/rpg/starfield-modder-finally-brings-seamless-planetary-landing-to-bethesdas-rpg-without-any-loading-screens-after-over-a-year-of-work/) |

The Seamless City Interiors trick is the part that matters. The mod does not stream anything new. It changes how the game classifies the interior, so the game treats it as part of the outdoor world space. That only works because Bethesda's engine can already hold both in one world space. The coverage says it works for only a small number of locations in Neon, and mods that change those interiors may break because the game now treats them as exteriors ([PCGamesN](https://www.pcgamesn.com/starfield/seamless-city-travel-mod)). The mod was described as early and possibly incompatible with some NPC interactions in a search summary, which was not checked against the Nexus page.

The relevant lesson is that this approach needs the interior to share the exterior's coordinates. KOTOR modules do not, so the same trick would need every module to be moved into a shared space first.

### Skyrim and Fallout mods

Open Cities for Skyrim moves walled cities into the open world so players walk in without a loading screen. Its page says it aims to bring back the feel of Morrowind, where most cities were part of the world. It uses occlusion planes so that nothing behind the city walls is drawn. Patches for other mods exist, and users of the mod will not see added interiors in their city. See [Open Cities on Nexus](https://nexusmods.com/skyrimspecialedition/mods/87707), read through a search summary. A 2022 forum post says the Special Edition version was removed from Nexus and is hosted on the author's own site, so check its status before relying on it.

Interiors in the Creation Engine are separate cells joined by load doors. A forum answer says that for interior cells you must use load doors, which can be invisible, to switch between cells ([Nexus forum thread](https://forums.nexusmods.com/topic/6142703-need-some-help-ripping-an-exterior-area-and-placing-it-in-a-new-worldspace)). This is a forum post, not a Bethesda source. No Skyrim or Fallout mod that removes load doors for ordinary interiors was found in this pass.

### Elden Ring Seamless Co-op

Seamless Co-op by LukeYui replaces the game's summon and session system with a persistent session across the whole world for up to four players ([PC Gamer](https://www.pcgamer.com/playing-elden-ring-with-friends-the-seamless-co-op-mod-is-the-way-to-do-it)). It does not remove loading screens, since Elden Ring has few. It is a contrast case. It changes who owns the world state, not how the world loads. It is mentioned only because the request asked for it. No technical source on its internals was found.

### OpenMW and Morrowind

Morrowind and OpenMW move the player between cells by doors that set a destination cell and position. OpenMW's documentation describes this as the common method ([Doors and Connecting Cells](https://openmw-improved-docs.readthedocs.io/en/latest/reference/modding/doors-and-teleports.html)).

A 2012 forum discussion proposed Skyrim style seamless interiors. A contributor said it would need predictive background loading, changes to the AI, and "basically implementing portals". Another thread noted that many interiors are larger than their exteriors, which makes alignment hard. Interior cell preloading was said to be implemented by Scrawl. These are forum posts found through search summaries ([OpenMW forum](https://forum.openmw.org/viewtopic.php?p=2286)). The page was blocked when fetched, so none of this was checked directly. No evidence was found that portals were ever built.

Tamriel Rebuilt was searched for. Only bug reports about misplaced doors and wrong teleport positions came up, such as [this one](https://tamriel-rebuilt.org/node/13369). They show that door destinations are a common source of mistakes even without seamless loading.

### KOTOR fan engines

reone is a C++ engine for KOTOR 1 and 2 ([repository](https://github.com/seedhartha/reone)). KotOR.js is a TypeScript engine built on THREE.js ([repository](https://www.github.com/KobaltBlu/KotOR.js)). xoreos has KOTOR code under `src/engines/` and its module code handled screen fades, according to a [2017 commit log](https://xoreos.org/pipermail/xoreos-git/2017-December/000405.html). No source says that any of these three engines streams modules seamlessly. The search found nothing on how they implement transitions. Reading their module code would be the next step if this matters.

### Gothic and Oblivion remakes

These were not found in this pass. No claim is made about them.

## Lessons for kjk

These lessons are the author's reading of the sources above. They are not claims from the sources.

1. The Source landmark model fits KOTOR best. Each module keeps its own origin. A pair of doors or triggers acts as the landmark pair, and the offset between them is computed at the moment of transition. The transitions note says most KOTOR transitions are paired doors and edge triggers, so most pairs already exist in the data.
2. A portal view can show the second world before the crossing. The second world is placed far away only to keep the two worlds apart. The cost is one extra render pass for the portal, which is far cheaper than Portal's recursive case. Insomniac's Rift Apart shows that a swap between two worlds needs fast storage, so loading must start early. Metroid Prime shows a safe fallback. If the target is not ready when the character arrives, the door stays shut.
3. Teleporting at the crossing keeps only one world as the active one. The Source rule that only entities inside a transition volume cross over gives a clear answer for party members. Party members within a set distance of the door cross with the character, and the others are placed at the landing point.
4. Scripts that assume a fresh load are the biggest risk. In Source, the engine saves the old level's state and runs a normal level load for the new one. A second world that is live at the same time would run its heartbeat, spawn and on-enter scripts early, before the character arrives. The safe design is to load the target module as visuals and collision only, with no scripts, and run the normal enter sequence at the crossing.
5. Coordinates must be handled once, at the crossing. Starfield's Seamless City Interiors works only because the engine already holds interiors and exteriors in one world space. KOTOR modules have no shared space, so the first step would be to keep the second world at a fixed large offset, as in the proposed idea. Unreal's history of origin shifting shows that far offsets lose floating point precision, so the offset should stay small, such as a few hundred meters.
6. Expect the mod record to show partial coverage. Seamless City Interiors covers a few locations in one city. The seamless elevators mod covers 18 elevators. The seamless landing mod took over a year. The realistic first target is a short list of simple door pairs, not all 156 KOTOR 1 door and trigger transitions.
7. Hiding tricks fail when load times grow. Mass Effect 3 cut an elevator for this reason. kjk should measure the load time of each target module and have a fallback such as a held door or a short fade.

## Open questions

- No primary source was read for how God of War, Prey, It Takes Two, the Gothic remake or the Oblivion remaster hide loading.
- How reone, KotOR.js and xoreos handle transitions is unknown. Reading their code would be a short follow-up.
- The Nexus pages for the Starfield mods and the Valve Developer Wiki pages could not be fetched. Their content came through search summaries.
- David should confirm which Starfield mod he remembered. Seamless City Interiors is the best guess.
