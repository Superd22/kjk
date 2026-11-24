# Module loading in KOTOR 1, KOTOR 2 and kjk

This note describes, step by step, what happens when a module such as `danm13` or `tar_m02aa` is loaded. It covers the original games, the reone reimplementation and kjk. Its purpose is to help judge how hard seamless level transitions would be in kjk. A seamless transition means that the next level streams in while the player keeps walking, with no loading screen.

The last section lists every step and says whether it would block a seamless transition, could run in the background, or would need redesign.

## Short answer

The original engine tears the old module down completely before it reads the first file of the new one. It keeps exactly one module in memory, and it resets the object table, the faction table, the event queue and the temporary `CURRENTGAME:` folder when it unloads. A seamless transition is therefore impossible in the original design.

kjk already breaks this rule in one important way. It reads and plans the next module on a separate thread while the old world is frozen, and it only swaps the simulation world in one call at the end. The planning work (reading the files, building collision and navigation, collecting scripts, decoding room models and textures) does not touch the running world. This is the part that could run in the background while the player keeps walking.

The parts that cannot simply move into the background are the hand-over itself. The old module must be saved into the stash, the party must be placed in the new module, the entry scripts must run, the render scene must be replaced, and the shader pipelines of the new area must be compiled. Those are the steps that need either a short hitch or a redesign.

## Sources and how far to trust them

The sources are listed in the order of trust.

1. kjk's own code, read in this repository. File and line references below are relative to the repository root.
2. Odyssey, a reconstruction of `swkotor.exe` (KOTOR 1) that builds byte-identical to the retail executable. It was cloned from https://github.com/TeamGizka/Odyssey at commit `7ac4b01`. Its paths are relative to `dev/game/` in that clone. It covers KOTOR 1 only.
3. K12, a byte-matching decompilation of the Android ports of both games. It was cloned from https://github.com/shinyquagsire23/K12 at commit `ef0e18d`. It builds both games from one source tree, and the macro `KOTOR1` separates the two. This is the only decompilation here that shows KOTOR 2 behaviour. Its paths are relative to `src/` in that clone.
4. reone, a reimplementation, cloned from https://github.com/modawan/reone at commit `533f3f6`. It is not the original engine, so it only shows one correct way to do things. Its paths are relative to `src/libs/game/` in that clone.
5. Community documentation. I did not fetch any forum or wiki page for this note. The community knowledge that appears here comes from comments in kjk's own source, which cite xoreos and reone.

Two cautions apply to the decompilations. First, Odyssey marks some functions with the comment "Dead code: not in the PC executable". At least one such comment looks misplaced. `ForceRunAreaOnExitScripts` carries it, yet `StartNewModule` calls it. Second, I traced the server side of the original in detail and the client side only for the area and the loading screen. I did not trace the texture and model caches of the original resource manager.

## The original engine in KOTOR 1

All file names in this section are Odyssey paths. The original runs a server and a client inside one process. They exchange messages through an in-process network layer, and the server code runs in small steps while the client draws the loading screen.

### The trigger

A module change starts in one of four ways.

**A door or trigger with a link.** The GFF of a door or a trigger carries three fields: `LinkedToModule` (the module), `LinkedTo` (the tag of a waypoint) and `LinkedToFlags`. The trigger handler is `CSWSTrigger::EventHandler` in `servercore/nwstrigger.cpp:523`. The door handler is the `ON_CLICKED` case in `servercore/nwsdoor.cpp:1383`. Both follow the same sequence.

1. A creature enters the trigger, or the player clicks the door.
2. The engine checks that the global dialog state is zero, that the creature is the party leader or the player, and that `CSWParty::IsPartyTogether()` is true. If the party is not together, it runs the script `k_trg_transfail` instead.
3. The engine starts a global fade to black over 0.5 seconds. It then queues an event for the same object with a delay of `TRANSITION_FADE_DELAY`, which is 500 milliseconds (`servercore/include/nwstrigger.h:42`). It also marks the area as having a transition pending and gives it a numbered id.
4. When the delayed event fires, the engine checks that the pending id still matches and that the creature is not dead. It then calls `SetMoveToModulePending(1)`, `SetMoveToModuleString(LinkedToModule)` and `SetMoveToModuleStartWaypoint(LinkedTo)`.

The waypoint tag is only stored at this point. The server keeps it in the string `m_sMoveToModuleStartWaypoint` until the new area exists.

**A script calling `StartNewModule`.** `CSWVirtualMachineCommands::ExecuteCommandStartNewModule` (`servercore/nwvmachinecommands.cpp:15821`) pops the module name, the waypoint tag and up to six movie names. It sets the same three "move to module" values and adds the movies to a list. It skips the party-together check and the 500 millisecond delay. If a dialog is on screen, it starts an immediate fade instead.

**The galaxy map and similar screens.** These run a script that calls `StartNewModule`. I did not trace the galaxy map screen separately.

**A save load.** The client sends `MODULE_LOAD_GAME`. `HandlePlayerToServerModuleMessage` (`servercore/nwshandleplayermessage.cpp:1860`) calls `LoadGame`, which ends in `LoadModule` for the module stored in the save.

The pending values are consumed in `CServerExoAppInternal::MainLoop` (`servercore/serverexoappinternal.cpp`, the block that tests `m_bMoveToModulePending`). On the PC it records the world time and the paused time of day, calls `m_partyTable.UpdateMembers(0)`, and then calls `StartNewModule`.

### Leaving the old module

`StartNewModule` (`servercore/serverexoappinternal.cpp:3303`) does the following in order. It returns at once if a module is already loading.

1. It chooses the next load screen hint, clears the dialog state, closes any dialog, stops all one-shot sounds, and starts the movies given to the script (`PlayLoadModuleMovies`).
2. It checks that the module exists as a `.mod` or a `.rim` in `MODULES:`. If not, it gives up.
3. It sets `m_bLoadingModule` and `m_bDoingStartNewModule`.
4. It calls `ForceRunAreaOnExitScripts` (`:1423`). For each player it runs the area's `OnExit` script with the player as the exiting object.
5. It calls `StorePlayerCharacters` (`:331`). This clears all actions of each player creature, writes each player creature into a GFF file named `TEMP:pifo`, and then calls `m_partyTable.UpdateMembers(1)` and `UpdateInventory(1)`. These save the nine companion slots and the shared inventory.
6. It calls `StoreCurrentModule` (`:447`). If `modulesave.2da` says the module is included in saves, the module writes itself into `GAMEINPROGRESS:` as an ERF file. `CSWSModule::SaveModuleStart` (`servercore/nwsmodule.cpp:1158`) starts the IFO. `SaveModuleInProgress` (`:643`) calls `CSWSArea::SaveGIT`, which writes one GIT with the creatures, items, doors, triggers, encounters, waypoints, sounds, placeables, stores and area effects. `SaveModuleFinish` (`:2537`) copies the unmodified ARE files (`SaveStatic`) and finishes the IFO. The IFO written here holds the module's script variables, its custom tokens, the player list, the limbo creatures and the whole server event queue, which means every pending `DelayCommand` and similar event (`SaveModuleIFOFinish`, `:1403`, calls `SaveEventQueue`). A `REPUTE.fac` file with the faction table is written as well.
7. It reads `modulesave.2da`. `DoModuleEnterSaveCleanup` deletes the stored state of every module in the same save group as the one being entered. `AutoSaveOnEnteringModule` (`:1473`) decides whether an autosave is written, and `DoPCAutosave` writes it from the state the game is leaving.
8. It calls `ExecuteLoadModule` (`:3277`). This pauses the client and server world timers, disables input, and calls `LoadModule`.

At the end of this phase the old module has been written to disk and the old world is still alive.

### Tearing down the old module

`CServerExoAppInternal::LoadModule` (`servercore/serverexoappinternal.cpp:2891`) is called with the new module's name. Its first act is to call `UnloadModule` (`:2289`) if a module exists. `UnloadModule` does the following.

- It leaves the combat and pause states and replaces both exclusion lists.
- For every player it removes the player from the object table, sends a "dump player" message, removes the player creature from its area and deletes it, and clears the player's area load state.
- It calls `ClearParty` on the client party and `ClearInventory` on the party table. It clears the custom tokens.
- It deletes the `CSWSModule`, which deletes its area and every object in it.
- It replaces the faction manager and the whole game object array with new empty ones. The new object array starts counting ids again.
- It clears the server event queue. Every pending delayed command that was not saved into the IFO is gone.
- It deletes everything in the `CURRENTGAME:` folder.

So the old module's creatures, effects, timers, delayed commands and sounds are all freed here. Companions are not carried across as objects. Their state sits in the party table, which `StorePlayerCharacters` filled before. Music and the module's ambient sounds are stopped by the sound mode change and by the loading screen (see below).

### Loading the new module on the server

After the unload, `LoadModule` works out where the module file comes from. It looks in this order: a saved game in `GAMEINPROGRESS:` (`.rsv`, then `.sav`), then `.nwm` in `NWMFILES:`, then `.mod` and then `.rim` in `MODULES:`, then the same two types in the live content folders. It checks the free disk space, and it starts a "stall event" of the kind load module (or load game). The stall event is the reason the loading screen exists. It makes `MainLoop` take a different branch in which the server does one small piece of work per frame while the client draws the loading bar (`serverexoappinternal.cpp`, the `else` branch with `OPERATION_LOAD_MODULE`).

The work is done in three kinds of step.

1. **Step zero.** `CopyModuleToCurrentGame` copies the module file into `CURRENTGAME:`. The engine then constructs the `CSWSModule` and calls `CSWSModule::LoadModuleStart` (`servercore/nwsmodule.cpp:2199`). This function mounts the module's resources (`AddModuleResources`, see below). It then reads the module IFO. It reads the module id, the name, the tag, the entry area, the entry position and direction, the hour length and the dawn and dusk hours. It sets the world time from the "move to world time" values, so the clock continues across modules. It reads the expansion and cut scene lists, and then the module scripts: `Mod_OnHeartbeat`, `Mod_OnUsrDefined`, `Mod_OnClientEntr`, `Mod_OnClientLeav`, `Mod_OnActvtItem`, `Mod_OnAcquirItem`, `Mod_OnUnAqreItem`, `Mod_OnModLoad`, `Mod_OnModStart`, `Mod_OnPlrDeath`, `Mod_OnPlrDying`, `Mod_OnSpawnBtnDn`, `Mod_OnPlrRest`, `Mod_OnPlrLvlUp` and `Mod_OnEquipItem`. It reads the area list, which holds exactly one area (the code asserts this). For a module that is a save, it also reads the saved object id counters, the player list, the tokens, the script variables and the event queue. Last, it loads the factions: from `REPUTE.fac` in the game in progress if one exists, else from the default factions.
2. **One step per area.** `CSWSModule::LoadModuleInProgress` (`:834`) constructs the `CSWSArea` and calls `CSWSArea::LoadArea` (`servercore/nwsarea.cpp:3590`). In this order, `LoadArea` reads the ARE header (`LoadAreaHeader`), then the layout and rooms (`LoadRoomInfo` reads the LYT and, for each room, `LoadWalkMesh` reads the room's WOK and welds neighbouring rooms), then the GIT (`LoadGIT`, `:1617`), then the path points (`LoadPathPoints`, the PTH file). The GIT lists are read in this fixed order: creatures, items, doors, triggers, encounters, waypoints, sounds, placeables, stores, area effects, properties, maps and placeable cameras. A creature is built from its UTC template when the GIT says `UseTemplates`, or from the full saved record otherwise. It is moved to the nearest safe location within a radius, and added to the area. The area registers its tag in the module's lookup table.
3. **The finish step.** `CSWSModule::LoadModuleFinish` (`:874`) queues a script event of type 17 with a delay of zero for the module. That event runs `Mod_OnModLoad` on the first AI update. It then releases the module's resource handle.

The PTH file, the UTC, UTP, UTD, UTT, UTW, UTS, UTE and UTM templates are all read during step two, through the resource manager, from the module capsule and the game archives.

### The module's resource capsule

`CSWSModule::AddModuleResources` calls `CExoResMan::AddResourceImageFilesAsync`, which starts a thread that runs `CExoResMan::AsyncLoad` while the loading bar ticks. In K1, `AsyncLoad` (K12's `libsource/exoresman/exoresman.cpp`, the `KOTOR1` branch) mounts these layers in this order: `<module>_a.rim` if it exists, `<module>_adx.rim` if it exists, then `<module>.mod` if it exists, else `<module>_s.rim`, and then the copy of the module in `CURRENTGAME:`. The `.rim` itself is the module's main resource file, and it is copied to `CURRENTGAME:` in step zero. The `_s.rim` holds the module's scripts, and a `.mod` replaces both. The lip-sync archives `lips/<module>_loc.mod` and `lips/localization.mod` are also mounted. In K1 the `players` archive is mounted here as well.

kjk mirrors this order in `crates/vfs/src/resolver.rs:659` (`Resolver::with_module`).

### The client builds the area

When the server is ready, it sends the client an area message. The client handles it in `CSWCArea::LoadArea` (`clientcore/nwcarea.cpp:967`), after `CSWCMessage::HandleServerToPlayerArea` (`clientcore/nwchandlemessage.cpp:3539`) has shown the loading screen image and unloaded the previous client area. The client reads the area's lighting, fog, grass, weather and ambient sound settings from the message. It then loads the LYT again and spawns one room object for each room (`CSWCRoom::SpawnRoom`). This loads each room's MDL and MDX and its textures and lightmaps. After every fifth room it updates the loading bar and draws a frame. It attaches ambient sounds to room parts, loads the VIS file (`LoadVisibility`), reads the map pins, the sound objects, and the lights and static placeables. Creatures, doors and other placeables are not in this message. The client creates their models when the server sends object updates once it is running (`UpdateClientGameObjects`).

### Placing the party

Placement happens in a handshake between client and server.

1. The client tells the server the module is loaded (`MODULE_LOADED`). The server runs `InitiateModuleForPlayer` (`servercore/serverexoappinternal.cpp:702`). For a player who is new to the module it sets the disappear position to the IFO entry position and direction. It sends the client area message with that position and facing.
2. The client tells the server the area is loaded (area message 3). The server runs `MovePlayerToArea` (`:776`). If a waypoint tag is stored in `m_sMoveToModuleStartWaypoint`, the server looks the waypoint up in the module by tag and copies its position and orientation to the player creature. It then calls `ComputeSafeLocation` with a radius of 20 units, adds the creature to the area, and calls `m_partyTable.PositionParty()`. If the tag is empty or unknown, the IFO entry position is used.
3. `CSWPartyTable::PositionParty` (`servercore/partytables.cpp:380`) resets the party follow data at the player's position and facing. For each companion slot it computes a position from the formation offset, finds a safe location within 10 units, and sets the position and the facing of the leader.
4. `RunModule` (`serverexoappinternal.cpp:1910`) switches the server from loading to running mode, calls `MovePlayerToArea` for players whose area is loaded, and ends the "start new module" state.

The party's companions are spawned by `CSWPartyTable::CreateParty` (`partytables.cpp:263`), which calls `SpawnNPC` for each member and gives them the party inventory.

### Scripts that run on load

The order in the original is the following.

1. During `LoadArea`, each creature is created. A creature does not run its `OnSpawn` script at creation. The script runs in `CSWSCreature::SpawnInHeartbeatPerception` (`servercore/nwscreature.cpp:743`), the first time the creature's AI update runs, and the flag `m_bCreationScriptFired` stops it from running again. So spawn scripts run after the load, on the first AI updates of a running world.
2. `Mod_OnModLoad` (event type 17) is queued with zero delay by `LoadModuleFinish`.
3. `LoadCharacterFinish` (`serverexoappinternal.cpp:1737`) queues the `ON_CLIENT_ENTER` event (type 14) for the module with the player's creature as the argument. This runs `Mod_OnClientEntr`.
4. `CSWSArea::AddObjectToArea` (`nwsarea.cpp:3537`) queues an `ON_OBJECT_ENTER` event with the object's id for the area, whenever a creature is added to it when not loading a saved game. The event handler (`nwsarea.cpp:578`) runs the area's `OnEnter` script. Because it is raised for every creature that is added, the area's `OnEnter` also runs for each companion.
5. The module's heartbeat starts counting from the module's last update time. Creature heartbeats start as each creature is first updated.
6. After the world is running, the global fade lifts (`StartGlobalFade(1, ...)`, a half second fade in) unless a dialog holds it (`m_bHoldWorldFadeInForDialog`).

Several scripts assume the module is fresh, and a few more assume the old one is gone. Creature `OnSpawn` and the encounter spawns run once per module first visit in kjk, and the original saves the creatures in the stored GIT so they do not run again on a revisit. The area's `OnEnter` runs on every entry. The enter scripts of many modules start fades, cutscenes and dialogs, and they call `GetFirstPC` and global variable checks. They expect the player to be placed already and the world clock to be paused until the fade in.

### The loading screen

The loading screen is only a presentation of the stall event. The functions are in `clientcore/clientexoappinternal.cpp`.

- `SetLoadScreenByModuleName` (`:7181`) looks the module up in the 2DA `areatransition` (column `BMPResRef`). If there is no row, it uses the image `load_<module>` if one exists, else the `DEFAULT` row.
- `GetLoadMusicByModuleName` (`:7232`) does the same for the music (column `MusicResRef`, else `load_<module>` as an MP3, else `DEFAULT`).
- `GetNextLoadScreenHintSTRREF` (`:7971`) reads `loadscreenhints.2da`. It chooses the story column if the module was visited before (`GetModulePreviouslyVisited`, `serverexoappinternal.cpp:1509`, which looks for a stored save of the module) and the gameplay column if not. It moves one row down that column on each call and wraps to row zero at the end.
- `ShowLoadScreen` (`:5556`) pauses all sounds but music, adds the load screen panel, disables input, hides the mouse and starts the load music. `HideLoadScreen` (`:5603`) removes the panel, restores input and stops the load music.
- `SetLoadBarProgress` is called from the server's steps and from the client's room loop.

Movies passed to `StartNewModule` play before and over the load screen (`PlayLoadModuleMovies`).

### What persists across modules

The original keeps these things across a module change.

- Global variables: `m_globalVariableTable`, saved with the game and not with the module.
- The party table: the nine companion slots, the members, gold, experience pool, the played time and the shared inventory (`partytables.cpp`, `LoadTableInfo` at `:445`). The party table is not reset by `UnloadModule`, apart from `ClearInventory`.
- The journal, which belongs to the party and the globals.
- The player creature, through `TEMP:pifo`.
- The world time and the paused time of day.
- Each module's stored state in `GAMEINPROGRESS:` (the `.sav` or `.rsv`), which holds the GIT, the IFO with its variables and event queue, and the faction table. Coming back to a module loads this stored state instead of the original template GIT, so a revisited module is exactly as it was left. The saved event queue means a delayed command that was pending when the player left comes back when the player returns.
- The load screen hint counters.

## KOTOR 2

K12 compiles both games from one tree. The macro `KOTOR1` marks the lines that differ. These are the differences that matter for module loading.

- **Override folders.** In K2, `StartNewModule` and `LoadModule` call `AddModuleOverrideResourceDirectories` and `RemoveModuleOverrideResourceDirectories` around the existence checks (`src/game/servercore/serverexoappinternal.cpp:3258`). The K2 `AddModuleResources` mounts `lips/<module>_loc` and `lips/localization` from every override path (`src/game/servercore/nwsmodule.cpp:1772`). K2's `AsyncLoad` also probes `<override>/modules/<module>` and mounts a `_dlg` archive. K1 has no override probing and no `_dlg` archive. kjk's `Resolver::with_module` opens `_dlg.erf` for both games (`crates/vfs/src/resolver.rs:686`).
- **Reputation adjustments.** K2's `LoadModuleFinish` calls `LoadReputationAdjustments` after queuing the module load event (`src/game/servercore/nwsmodule.cpp:2350`). K1 has no such step.
- **The loading bar.** K1 advances the bar in sevenths of the second step and mounts the `players` archive. K2 advances it in fifteenths.
- **The party table.** `partytables.cpp` has about forty `KOTOR1` blocks. K2 adds puppets (the extra party members of the cutscenes) and the companion "influence" state. kjk's notes say K2's executable keeps the whole creature in its party table across a module change, so a companion is made again in a module that does not ship its template (`crates/sim/src/install.rs:292`).
- **The area's enter event.** kjk records that K2's `CSWSArea::AddObjectToArea` signals the entered event for every creature, so the area's `OnEnter` hears each companion and also the main character (`crates/sim/src/world/scripts.rs:2328`). The module's `OnClientEnter` is raised for each player only.
- **The load screen hints.** kjk's `Installer::hint` (`crates/game/src/lifecycle.rs:557`) documents that K1 keeps one story counter, taken from `loadscreenhints.2da`, and that K2 keeps one counter for each planet, taken from `loadstoryhints.2da`. The planet is the first digit of the module name. I read this only in kjk's comment and did not check it in K12.
- **The player enter event.** K12 names the event `ON_PLAYER_ENTER` where Odyssey calls it `ON_CLIENT_ENTER`. It is the same event type.

I did not find a difference in the order of the IFO, ARE, GIT, LYT, VIS and PTH reads between the two games.

## reone

reone is a reimplementation, and it shows a clean split between preparing and committing a module. The functions are in `src/libs/game/game.cpp` unless stated otherwise.

- A door or trigger or a script calls `scheduleModuleTransition(module, entry)` (`:5851`). It only stores the name and the entry tag. The next `update` calls `loadNextModule` (`:5752`).
- `loadModule` (`:1930`) first calls `prepareDestinationModule` (`:1841`). This asks the resource director to read and validate the destination's IFO, ARE and GIT, and to build the resource view of its capsule. Nothing in the running game changes in this call.
- `loadPreparedModule` then runs the old module's area `OnExit` script, clears actions, builds a snapshot of the old module's working state, captures the party's pending actions and the travelling companions, and may write an entry autosave. Only then does it show the loading screen and commit.
- Inside the loading screen, the commit does these things in order. It starts a fade out. `endSourceSession` ends minigames, conversations and station screens, and `retireActiveModuleRuntime` retires the old area. It commits the new resources. It loads the in-game menus. It publishes the new module object, calls `Module::load` (`object/module.cpp:253`, which reads the module info, builds the area from ARE and GIT, announces each creature to the area and sets up the cameras), makes the companions again from their records, restores saved effects and actions, signals the module as loaded (which queues `OnModLoad` and the enter events), forms the party, and calls `Module::loadParty` (`object/module.cpp:406`).
- `Module::getEntryPoint` (`object/module.cpp:489`) looks the waypoint tag up (lowercased) in the area. If the tag is empty or not found, it uses the IFO entry position and facing.
- The creature creation scripts run on the area's first update, after the load's queued events.
- reone resets its frame clock after a load, since "none of it was time the game world lived through".

## kjk

All paths in this section are relative to the repository root. `docs/LIFECYCLE.md` is the existing short description of this flow. This section adds the order of the steps and the threads.

### The trigger in kjk

1. A door or trigger that leads out, found by the simulation. Entering an open exit doorway or a trigger with a link sets `Transition` in that step's snapshot. The sim asks once (`crates/sim/src/world/uses.rs:230`, `crates/sim/src/world/interaction.rs:186`, `crates/sim/src/world/party.rs:609`). The ask first goes to the ready check, which asks that the whole party is ready. I found no 0.5 second fade and no 500 millisecond delay in kjk's ask path. The host stops stepping the world as soon as the ask lands in a snapshot.
2. A script calling `StartNewModule`. The engine call (`crates/sim/src/script/engine.rs:5503`) sets `out.new_module`. It queues the movie notices for the host, and the step's apply stage turns it into `area.pending` (`crates/sim/src/world/scripts/apply.rs:79`). A script's `StartNewModule` skips the ready check. In the same step, `leaving` (`crates/sim/src/script/mod.rs:4691`) queues the area `OnExit` and the module `Mod_OnClientLeav` for every main character, and runs them before the step ends.
3. The galaxy map. The planet pick runs a travel script, which calls `StartNewModule`.
4. A save load. `reload_start` (`crates/game/src/sim_host.rs:2043`) builds a request for the saved module and starts it as a job that is marked as a reload.
5. The `module` console command (`crates/game/src/sim_host.rs:2221`).

The waypoint tag is lowercased in `Request::new` (`crates/session/src/lib.rs:90`) and travels as `Request::arrival`.

### The host starts a job

When a frame lands a snapshot with a `transition`, `advance` (`crates/game/src/sim_host.rs:2130`) builds a request with `Installer::request_from` and calls `Authority::start` (`:1581`). `start` returns false when this transition failed before, so a failing transition is not retried. If it starts, the frame sets the state `loading` to true and the state `world` to false, and sets `held_up`, which stops the simulation clock (`:2615` to `:2630`). So the old world is frozen from the moment the transition is asked for until the commit.

`Job::spawn` (`:584`) starts one thread. That thread first reads the loading screen's image, bar and hint (`Installer::loading_screen`), and sends them to the main thread. It then calls `Installer::prepare_with`.

### Planning on the thread

`Installer::prepare_with` (`crates/game/src/lifecycle.rs:417`) does these steps.

1. It opens the module's resolver, `Resolver::with_module`. The install's base layers are shared and opened once per process. Only the module's own archives are new (`.rim`, `_s.rim`, `.mod`, `_dlg.erf`, the lips archives). Progress reaches 0.25.
2. It then runs two things in parallel. A scene thread builds the scene (`Installer::scene`, `crates/game/src/lifecycle.rs:493`). The main plan thread calls `session::plan_with`.
3. `session::plan_with` (`crates/session/src/lib.rs:463`) reads, in this order, the IFO, then for each area the ARE, GIT, LYT and VIS (`ModuleIr::build`, `crates/asset_odyssey/src/module.rs:866` and `:904`). It reads the instances of each kind in a fixed order (creatures, doors, placeables, triggers, waypoints, sounds, encounters, stores). It checks that each room's MDL and WOK exist, but does not parse them yet. It then builds the simulation's object facts (`objects::module_data`), fills in the door facts, reads the creature templates (the whole install's, or only the module's own with the import step's products), builds the object sheets, the force rooms and the encounters. It then builds the area: collision and the navigation mesh (`build_area`, `:797`), from the in-memory `AreaCache`, else from the on-disk store's bake, else from scratch. It then gathers the script library for the module (the compiled scripts of every hook, dialogue and the scripts these name), the item catalog, the placeable, waypoint and store templates, the quests, experience tables, classes and workshop. Finally it builds the `Level` and checks that every seat can arrive, by installing into a throwaway world (`:762` to `:775`).
4. The scene thread (`Installer::scene` and `viewer::module_scene`, `crates/game/src/viewer.rs:1197`) reads each room's MDL and MDX and its walkmesh and textures into intermediate data, builds grass, collects the room clips and effects, and reads the area sound in a third thread (`ambience::preread`). Cutscene camera models are read after the plan.

Nothing in this phase touches the running world. The result is a `Prepared` holding the `Plan`, the optional scene and the timings. Progress reaches 0.9.

### Committing on the main thread

`advance` polls the job each frame (`:2203`). When it finishes, the commit happens in one frame.

1. For a replay, it checks the plan against the recording.
2. For a save reload, it builds the world from the save (`crate::play::from_save`). Otherwise it calls `Plan::commit` (`:2312`), which calls `SimWorld::install` (`crates/sim/src/install.rs:190`).
3. `SimWorld::install` does the following. It settles the party switch and takes a snapshot of the old world. It reads the party table and keeps the templates of companions the new module lacks. It calls `assemble` (`:399`), a pure function of the old snapshot and the new level, which builds the next snapshot. In `assemble`, the old module's objects go into the stash (replacing an older stay of the same module). The new module's objects come from the stash if it was visited before, else fresh from the module's facts with new ids. The party's companions are made anew as objects from their templates. Each seat's player carries over with its id. The arrival is found by tag (`arrival_spot`, `:913`): the first door or waypoint with the tag, else the IFO entry. The players are dropped to the floor there, and the next seats and the companions go to free spots around it. The facing is that of the waypoint or door. A trigger that leads out, standing where the party arrives, counts the party as inside already, so it does not fire again.
4. `SimWorld::from_snapshot` builds the whole new ECS world with the new level. The scripts' state is taken from the old world (`take_scripts`), and `State::leave_module` (`crates/sim/src/script/state.rs:2221`) moves everything that waits for the old module's objects (pending script runs, delayed commands, action queues, uncommandable flags, corpses, encounter timers) into `State::left`, keyed by module. `State::return_to` (`:2262`) puts them back when the module is entered again, with the delayed commands' due times moved forward by the time spent away.
5. `SimWorld::enter_module` (`crates/sim/src/world/scripts.rs:2190`) clears what only belongs to one module: pending events, delayed commands, action queues, heartbeat phases, perception, goals, strikes, kept corpses, conversations, trades and the player restrict mode. It then runs the entry events. On a module's first visit, it makes every placement come alive. It stocks the starting items, and queues each creature's spawn script in the module's order (`come_alive`, `:2347`). It then queues, in order: the module's load script (`Mod_OnModLoad`); for each main character, the module's `OnClientEnter` and the area's `OnEnter`; and then the area's `OnEnter` for each companion, in party order. These queued runs execute in the first step after the install. A revisited module does not run its creature spawn scripts again, but it still runs the module and area enter scripts.
6. The host then swaps the render scene. It inserts `SceneContent` (`crates/game/src/sim_host.rs:2420`) and sets `SceneSwap` with the new generation, level and body parts (`:2422`). `spawn_scene` (`crates/game/src/scene.rs:1929`) runs when `SceneContent` changes. It despawns every old `SceneEntity` and spawns the new rooms, lights, grass and room effects. It makes the meshes, materials and images from the data the scene thread built. The old meshes, materials and images are dropped with their last handles.
7. The ambience system sees a new `SceneSwap::generation` (`crates/game/src/ambience.rs:773`). It stops the last area's music, clears its sounds from the bank, and starts the new area's music after the music wait.
8. The loading screen lifts only when the new area can be drawn whole. `Picture` (`crates/game/src/lifecycle_view.rs:120`) waits at least `HOLD` (0.35 seconds) after the commit and then until the pipeline warm-up has finished (`crates/game/src/warmup.rs`). While the warm-up compiles the pipelines, the scene cameras see only the warm-up layer (`hold_scene`). The wait is capped at `LONGEST_WAIT` (30 seconds). The screen then fades out over `FADE` (0.35 seconds). The first tick of the new area comes as the fade begins.

The warm-up is there because, on an empty shader cache, drawing an area while 200 to 340 pipelines were still compiling lost the graphics device in 5 of 30 loads.

### The loading screen in kjk

The loading screen image comes from `loadscreen.gui` and the image `load_<module>`, with the bar fill `bluefill`. The hint comes from `loadscreenhints.2da` (K1) or `loadstoryhints.2da` (K2), and `HintAsk` carries where the hints stand. The original's `GetModulePreviouslyVisited` is answered in kjk by `Snapshot::visited(module)`. The screen is drawn by `crates/game/src/lifecycle_view.rs` (`game_screen`, `settle`, `loading_screen`).

### What persists in kjk

- The script state (`crates/sim/src/script/state.rs`) carries across: the globals, the party table with its companions' creature templates, gold, items, the journal (`journal` at `:222`), `spawned` (the modules whose placements came alive), and `left`.
- The snapshot's `stash` (`crates/sim/src/snapshot.rs:1102`) holds each module left: its place and data digest, its objects by id, the objects made in play, and its roster of placements. Coming back finds the module as it was left. If the module's data changed since, `Restamp` and `reconcile` bring the saved objects onto the new data.
- Tick, time and random streams continue.

### What is cached across modules in kjk

- The install's base resolver, opened once per process (`crates/game/src/lifecycle.rs`, `Installer::install`).
- The built areas, in the in-memory `AreaCache` (`crates/session/src/lib.rs:221`), and, when baking is enabled, on disk in the store. The cache key is the module, the collision mode and the area.
- The import step's products: the creature and item templates of the whole install, when the install was imported.
- The pipelines the warm-up remembers (`warmup.rs`, `learn` and `prewarm`).
- The `Scripts` state, the stash, and `State::left`, as above.

Nothing else is cached. Scenes (room models, textures) are built again on every visit. The repository has no recorded load times. The host logs `prepare_ms`, `open_ms`, `plan_ms`, `scene_ms`, `commit_ms` and `area_cached` for every install (`crates/game/src/sim_host.rs:2395` and `:2437`), so numbers can be taken from a run.

## Comparing the three

| Step | Original (K1 and K2) | reone | kjk |
|---|---|---|---|
| Fade and delay before the load | 0.5 s fade, 500 ms event, party must be together | A global fade out inside the load | Ready check, no fade delay |
| Exit scripts | Area `OnExit` forced for each player | Area `OnExit` run first | Area `OnExit` and module `OnClientLeav` run in the step that asks |
| Old module saved | ERF in `GAMEINPROGRESS:` (GIT, IFO, event queue, factions) | Working state snapshot | Stash in the snapshot, and `State::left` |
| Old module freed | Before any file of the new one is read | After the destination is validated, at commit | At the commit, after planning |
| Reading the new module | In steps, one per area, with the world stopped | Prepared first, then committed | Planned on a thread, then committed |
| Entry waypoint | Looked up after the area exists, safe location radius 20 | Looked up by tag, else the IFO entry | Looked up in the plan, else the IFO entry |
| Spawn scripts | On each creature's first AI update | On the area's first update | Queued at install, run in the first step |
| Module load script | Event queued with delay zero | Queued at `signalLoaded` | Queued first at install |

## Step by step, and what a seamless transition would need

The list below follows the order of the steps in kjk. For each step it says whether the step would block a seamless transition, could run in the background, or would need redesign.

1. **Noticing that a transition is coming.** Could run in the background. Every door and trigger with a link is known from the GIT when the area is planned (`Facts::exits`, `crates/session/src/lib.rs`), so the destinations of walked transitions could be prefetched as the player approaches. A script's `StartNewModule` destination is only known when the script runs, although the script library holds the module names as strings. Scripted transitions (cutscenes, the galaxy map, story events) usually come with movies and should keep their loading screen.
2. **The ready check and the party-together rule.** Does not block. It runs in the old world, which keeps being stepped. The original's 0.5 second fade is far too short to hide a load, so prefetch must begin earlier than the trigger.
3. **Today's freeze of the old world.** Needs redesign. `advance` sets `held_up` and the state `world` to false the moment a transition appears. A seamless version must keep stepping the old world while the plan runs, and must only hold it for the commit tick.
4. **Running the exit scripts and the old module's last tick.** Blocks, but briefly. They must run against the old world, and they may change globals that the new module's scripts read. They would have to run at the commit, not at the start of prefetch.
5. **Opening the module's archives (`with_module`).** Could run in the background. It is file input, and the base layers are shared.
6. **Reading the IFO, ARE, GIT, LYT and VIS.** Could run in the background. `ModuleIr::build` needs only the resolver.
7. **Reading templates and building the object facts and sheets.** Could run in the background. It needs the request's player count, which is known when the prefetch starts and can be rechecked at the commit.
8. **Building collision and the navigation mesh.** Could run in the background. It is already cached in memory and in the store, and it is the heaviest part of a first visit.
9. **Gathering the script library.** Could run in the background. It reads compiled scripts only.
10. **The trial arrival.** Could run in the background. It uses a throwaway world.
11. **Building the render scene data (rooms, textures, grass, area sound).** Could run in the background. It already runs on its own thread and produces data only.
12. **Saving the old module into the stash.** Blocks, but briefly. It is a copy of the snapshot's object states. It needs a tick boundary of the old world. Its cost grows with the number of objects, and it has to happen at the commit.
13. **Autosave on entering the module.** The original writes it before the unload and blocks on it. I did not trace whether kjk writes an autosave on a module change. kjk has an autosave system that a script's `DoSinglePlayerAutoSave` triggers (`crates/game/src/sim_host.rs:737`). A seamless version could write any autosave from a snapshot on a background thread.
14. **Placing the party and making the companions.** Blocks, but briefly, and cannot run earlier. It needs the old party's positions, health, effects and templates at the commit tick. The arrival spot itself can be computed in advance, because it depends only on the new module's data.
15. **`SimWorld::from_snapshot` and the world swap.** Blocks and needs redesign. It builds the whole simulation world in one call on the main thread. A seamless transition would want its cost to be small, or to build most of the next world in advance and only patch in the party at the commit. This is also the point where the simulation holds one level at a time, which affects physics, collision, navigation and perception. The two modules are separate coordinate spaces, so there is no walkable seam between them. The design problem is to hide the cut, for example behind a door opening animation, a corridor or a short fade, and not to scroll one area into the other.
16. **Clearing the module-bound script state (`enter_module`).** Blocks, but briefly. It is cheap and has to happen at the commit.
17. **Running the entry scripts (module load, client enter, area enter, creature spawns).** Blocks and needs care. They run in the first step after the commit and mutate the world and the globals. They cannot run in the background. Many of them assume that the party was just placed, that the world fade is still down, and that no old-module events are running. They would have to run after the cut is hidden, and some of them (cutscene starters) would break a transition without a fade.
18. **Replacing the render scene (`spawn_scene`).** Needs redesign. `SceneContent` is one resource. `spawn_scene` despawns every old scene entity and builds the new ones in a single frame on the main thread, including the mesh, material and image creation. Culling, room queries, light probes, weather, wind, fog and the clear colour all read the single scene. Spreading the spawn over several frames, or keeping the old scene visible until the new one is ready, requires changes in all of them.
19. **Pipeline warm-up.** Needs redesign. The warm-up draws stand-ins for every layout the new area uses, and hides the scene until it is done. A cold shader cache can lose the graphics device when pipelines compile while the area draws. For a seamless load, the warm-up would have to run for the next area while the current area keeps drawing, with a frame budget, before the commit.
20. **Music and ambient sounds.** Could run in the background for reading, and needs a small change for the switch. `ambience::preread` already reads the area sound on the scene thread. The switch itself is fast. The original stops all one-shot sounds and plays load music, so a cross fade would be new behaviour.
21. **The loading screen, the hints and the movies.** Does not exist in a seamless transition. They would stay for scripted transitions, saved-game loads and movie transitions. The hint counters and the visit check would no longer advance on seamless transitions.
22. **Co-op.** Needs redesign. The host announces the install (`Announcement`) and each client plans the same module from it, checks its digests and commits at the same tick. A prefetch would need to be announced early, and the commit would need to land on the same tick for every player.
23. **Memory.** Needs a decision. A prefetch keeps the current level, its scene and the next plan and scene in memory at once. The stash holds only object states, so it adds little.
24. **Persistence across modules.** Does not block. Globals, the party table, the journal and the stash are independent of the loading screen.
25. **A saved-game load.** Keeps its loading screen. It rebuilds the world from the save and does not pass through the stash.

In short, the two largest pieces of work are the commit-time hand-over of the simulation world (step 15) and the replacement of the single render scene (steps 18 and 19). The reading, planning and building steps (5 to 11) are already separate from the running world and could be started earlier with little change.
