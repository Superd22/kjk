# Level transitions in KOTOR 1 and KOTOR 2

Date: 2026-11-24.

This note lists every level (module) transition found in the data of both games. David wants to judge how hard it would be to make level transitions seamless, which means the next level streams in while the player keeps walking. For that, each transition needs a start position and a landing position. The two CSV files hold the full list. This note explains how they were made, counts the transitions, and analyses whether the two ends of a transition share one coordinate space.

Files:

- `docs/research/2026-11-24-level-transitions-k1.csv` holds the KOTOR 1 transitions.
- `docs/research/2026-11-24-level-transitions-k2.csv` holds the KOTOR 2 transitions.

## Method

The data was read from the Steam installs on this Mac. KOTOR 1 was read from the `Contents/Assets` folder of `swkotor`. KOTOR 2 was read from the `Contents/GameData` folder of `Knights of the Old Republic II`. The Workshop items were turned off, so the numbers describe the unmodified games.

The reading was done with kjk's own code. A small throwaway program in the scratchpad used the `vfs` crate to find the installs and stack the layers (the `override` folder, the module archives and the BIFs), the `fmt_odyssey` crate to read GFF, 2DA and NCS files, and the `script_ncs` crate to read `nwscript.nss` so that routine numbers could be turned into names. Nothing in the repository was changed apart from the new files in `docs/research/`.

The steps were these.

1. The module list came from `Resolver::modules`, which lists every `.rim` and `.mod` file in the `modules` folder. For each module the program opened the module's own archives on top of the shared layers. It read the module IFO to get the area list and the entry point, and it read the GIT of every area in that list. The GIT is named after the area, not after the module, so the area list is required.
2. For every door in the GIT door list and every trigger in the trigger list, the program read `LinkedToModule`, `LinkedTo` and `LinkedToFlags`. If the GIT instance did not hold a field, the program fell back to the UTD or UTT template. Every entry whose `LinkedToModule` is not empty was kept. No door or trigger had a `LinkedTo` without a `LinkedToModule`, so there are no transitions inside a module in either game.
3. A trigger's position is the centre of its geometry, which is its position plus the average of its geometry points. The CSV also gives the width and depth of the geometry's bounding box.
4. The landing position was read from the target module's GIT. When `LinkedToFlags` has bit 1 set and not bit 2, the tag names a door in the target module, and the landing is that door's position and bearing. Otherwise the tag names a waypoint, and the landing is the waypoint's position. The facing of a waypoint is the angle of its orientation vector, counted in degrees from the positive X axis. If the tag is empty, the landing is the module entry point from the IFO, and the CSV says so in the `target_kind` column (`entry`).
5. For scripts, the program scanned every compiled script (NCS) in every layer of each install. These were 12,893 scripts in KOTOR 1 and 5,120 in KOTOR 2. It looked for calls to the routine `StartNewModule`. KOTOR 2 has no other routine that starts a module, so no similar function was needed. When the module name and the waypoint tag were pushed as plain string constants, they were read directly. When a script chose them at run time, the program recorded every module name among the script's strings as a candidate and marked the row as a dynamic candidate in the notes.
6. To find what fires a script, the program searched every DLG, UTT, UTD, UTP, UTC, UTE, UTW, ARE, IFO and GIT file, and every NCS file, for the script's name. The `fired_by` column lists the first referrers it found, for example a dialog node or a trigger event.
7. For paired doors and triggers, the program looked in the target module for a door or trigger that links back to the source module. If there were several, it took the one whose start position is nearest to the landing position. It then measured the distances described below.

The CSV columns are these. `kind` is `door`, `trigger`, `script` or `galaxy`. `galaxy` marks the scripts of the galaxy map and the Ebon Hawk landing. `target_kind` is `waypoint`, `door`, `entry` or `unresolved`. `dist_src_to_tgt` is the 3D distance between the start position and the landing position of the same transition. `dist_tgt_to_back_src` is the distance from the landing position to the start of the transition back. `dist_src_to_back_tgt` is the distance from the start position to the landing of the transition back. Positions are in the units of the game, which are meters.

## Module count check

Both installs have only `.rim` files, `_s.rim` files and (in KOTOR 2) `_dlg.erf` files in the `modules` folder. There are no `.mod` files in either install, so no module is overridden by a `.mod` file in this data. The program still handles `.mod` files, because the resolver reads them first when they exist.

| Game | Files in `modules` | Modules found | Modules with a GIT for every area in the IFO |
|---|---|---|---|
| KOTOR 1 | 234 (117 `.rim` and 117 `_s.rim`) | 117 | 117 |
| KOTOR 2 | 246 (82 `.rim`, 82 `_s.rim` and 82 `_dlg.erf`) | 82 | 82 |

The count of distinct module names in each folder matches the module count the program found, so no module was skipped.

## Counts

| Kind | KOTOR 1 | KOTOR 2 |
|---|---|---|
| Doors with a module link | 116 | 39 |
| Triggers with a module link | 40 | 85 |
| Door and trigger transitions in total | 156 | 124 |
| Script transitions (rows of kind `script`) | 225 | 126 |
| Galaxy map and Ebon Hawk landing rows (kind `galaxy`) | 52 | 21 |
| Compiled scripts that call `StartNewModule` | 163 | 105 |

Notes on these counts:

- A script row is one pair of a script and a target. Several scripts of the same name exist in different module archives, and rows with the same script, target and condition were merged into one row, so the count does not depend on how many copies exist.
- Of the KOTOR 1 doors, 66 link to a waypoint, 48 link to a door of the target module, and 2 have `LinkedToFlags` 0. Of the KOTOR 2 doors, 19 link to a waypoint, 18 link to a door, and 2 have flags 0. All triggers link to a waypoint. The four doors with flags 0 still name a target module, so they are in the CSV with a note that the link may be inactive.
- The 52 KOTOR 1 galaxy rows include 12 rows that are dynamic candidates, and the 7 KOTOR 1 script rows and 18 KOTOR 2 galaxy rows also come from dynamic candidates (see the unresolved section).

## Galaxy map and ship travel

The planets 2DA is `planetary.2da` in both games. It has no module names. It lists only the display data of each planet (name string, description string, icon and model). The module names come from scripts.

KOTOR 1 `planetary.2da` has 17 rows. These are Endar Spire, Taris, Ebon Hawk, Dantooine, Tatooine, Kashyyyk, Manaan, Korriban, Leviathan, Unknown World, Star Forge and six unused rows named Live_Planet_01 to Live_Planet_06.

KOTOR 2 `planetary.2da` has 16 rows. These are Dantooine, Dxun, Ebon Hawk, Korriban, M4-78, Malachor V, Nar Shaddaa, Onderon, Peragus, Telos, Citadel Station, Peragus Tutorial and four unused Live_Planet rows.

How travel works in KOTOR 1:

1. The script `k_pebn_galaxy` calls `ShowGalaxyMap`. It sets which planets are available, then the engine shows the map and stores the chosen planet in the global number `K_FUTURE_PLANET`.
2. The script `k_sup_galaxymap` then starts one of the stunt modules (`stunt_12`, `stunt_14`, `stunt_16`, `stunt_18`, `stunt_42` or `stunt_44`) depending on the planet. These modules are the ship flight cutscenes. The strings such as `05_2C` in these calls are movie names, not waypoints.
3. After the stunt, the scripts `k_ren_nextplanet` and `k_ren_visionland` start `ebo_m12aa`, which is the Ebon Hawk interior.
4. When the main character leaves the Ebon Hawk, the script `k_pebn_exithawk` starts the planet module. It picks the module and waypoint from the planet number. For example, number 15 starts `danm13` at the waypoint `K_DAN_EBON_HAWK_TRANSITION` at (187.2, 75.2, 12.6), and number 20 starts `kas_m22aa` at (57.1, 51.0, 64.4).

How travel works in KOTOR 2:

1. The script `a_galaxymap` (in module `003ebo`) and the script `a_galaxy_map` (in module `001ebo`) call `ShowGalaxyMap`.
2. The scripts `check_party_gui` (in `003ebo`) and `tr_leave_ehawk` (in `004ebo`) then start the planet module. The module name is built at run time from the planet, so the CSV lists each candidate module from the script's strings. The candidates are `106per`, `201tel`, `262tel`, `301nar`, `401dxn`, `601dan`, `701kor` and `901mal`. Where the waypoint `WP_from_ebonhawk` exists in the candidate module, the CSV gives its position. For example, `201tel` lands at (-13.7, 11.6, 13.1) and `301nar` lands at (-109.0, 26.3, 9.7). For `106per`, `262tel` and `901mal` the module has no such waypoint, so the CSV gives the module entry point.

The galaxy rows are in the same CSVs as the other transitions, with kind `galaxy`.

## Transitions that could not be resolved

KOTOR 1 has 40 transition rows that are unresolved, 27 of kind `script` and 13 of kind `galaxy`. KOTOR 2 has 20 unresolved rows, which are 4 doors or triggers and 16 script rows. The reasons are these.

- A target module is not in the install. KOTOR 1 scripts name `lev_40ac`, `liv_m99ab` to `liv_m99af`, `korr_m33`, `ebo_m44aa` and `sta_45aa`. These look like typos or cut content. KOTOR 2 scripts name `000TEST` (5 rows), `106PERm`, `211TELm`, `tat_m17ae` and the door `d_hk` in `232tel` targets the missing `298tel`.
- A waypoint tag does not exist in the target module. Examples in KOTOR 1 are `K_TARIS_DESTROYED` in `ebo_m12aa` (8 rows), the swoop track points `K_DAN_SWOOP_POINT_02` and similar (9 rows), `k_tar_ebon_hawk_transition` in `tar_m02aa`, and `tar02_swplayerapt` in `tar_m02aa`. Examples in KOTOR 2 are the triggers `TO_302NAR` in `301nar`, `To_411` in `410dxn` and `To_504` in `511ond`, and the script targets `From_300NAR` in `304nar` and `305nar`. For these, the program cannot say where the engine would place the main character. The most likely fallback is the module entry point, but this was not checked in the engine.
- The script passes an empty module name. In KOTOR 1 the scripts `k_pebn_exithawk` and `k_pebn_swoop` do this for the planet numbers 5 and 50, which are most likely the Endar Spire and the Star Forge. The target is a placeholder.
- A script builds the module name from variables and has no module name among its strings. These rows are reported as unresolved with that note.

Dynamic candidate rows are guesses. The module is one of the module names in the script, and the waypoint tag is the first string in the script that exists as a waypoint of that module. They show which modules a script can reach, but they do not say which one a given call reaches.

The `fired_by` column is empty for 29 of the KOTOR 1 rows of kind script and 14 of the KOTOR 2 rows of kind script. The scripts are started from places that the search did not cover, such as the dialogs of other scripts through `ExecuteScript` with a variable name, or heartbeat and user-defined events.

## Analysis

### How many are paired

Doors and triggers give 156 transitions in KOTOR 1 and 124 in KOTOR 2.

In KOTOR 1, 140 of the 156 have a matching door or trigger back (109 doors and 31 triggers). Another 13 have a script as the way back, most of them the triggers and doors that go to the Ebon Hawk (`ebo_m12aa`), whose exit is decided by the script `k_pebn_exithawk`. Three have no way back at all. These are the door `end_door10` in `end_m01aa`, the door `lev40_hangar` in `lev_m40ac` and the door `tar10_elevator` in `tar_m10ab`.

In KOTOR 2, 109 of the 124 have a matching door or trigger back (33 doors and 76 triggers). One door has a script as the way back. Ten have no way back (for example the triggers `To_602`, `To_602_2` and `To_602_3` in `603dan`, and `TO_904MAL` in `903mal`), and four are unknown because their landing cannot be resolved.

The 156 KOTOR 1 transitions connect 79 pairs of modules. The 124 KOTOR 2 transitions connect 56 pairs of modules.

The scripts are the one way transitions. KOTOR 1 has 225 script rows and KOTOR 2 has 126. Most are fired from dialogs. Of the 277 KOTOR 1 rows of kind script or galaxy, 97 are listed as fired by a dialog node. Of the 147 KOTOR 2 rows, 99 are. Another 44 KOTOR 1 rows and 15 KOTOR 2 rows are fired by a trigger event. The rest are fired from area or module events, from other scripts, or from sources the search did not find. Typical script transitions are cutscenes (the `a_` scripts of KOTOR 2 such as `a_sion_cs` and `a_kreiaend`), the swoop race entries, the stunt flights between planets, and the party handover to the Ebon Hawk.

Thirty-two KOTOR 1 modules and 17 KOTOR 2 modules have no door or trigger link at all. They are reached only by script. They are mostly stunt cutscene modules, mini games and the Ebon Hawk modules.

### Do paired modules share coordinates

In general, no. Each module uses its own origin.

The test is the distance between the start of a transition (in the source module) and its landing (in the target module). If two modules shared one world space, a door at the edge of one module would land next to the same spot in the other module, and the distance would be near zero.

- The median distance is 168.5 meters in KOTOR 1 and 86.6 meters in KOTOR 2. The largest distances are 555 and 645 meters.
- Only 7 KOTOR 1 transitions and 23 KOTOR 2 transitions have a distance of 5 meters or less.
- The landing is always close to the start of the transition back, because the landing waypoint stands next to the return door. The median distance from the landing to the start of the transition back is 3.5 meters in KOTOR 1 and 3.9 meters in KOTOR 2, and the largest is 50.4 and 21.0 meters. This is local to the target module and says nothing about the source module.

The exceptions are module pairs that split one location into parts. There are 4 such pairs in KOTOR 1 and 11 in KOTOR 2, where every link between the two modules has a start and a landing less than 15 meters apart. In KOTOR 1 they are `end_m01aa` and `end_m01ab`, `sta_m45ab` and `sta_m45ac`, `tar_m02aa` and `tar_m02af`, and `unk_m41aa` and `unk_m41ad`. In KOTOR 2 they are `151har`, `152har` and `154har`, the Telos cantina and shop interiors (`202tel` and `207tel`, `203tel` with `204tel`, `208tel` and `209tel`, and `220tel` with `221tel`), `304nar` and `305nar`, `502ond` and `512ond`, and `903mal` with `905mal` and `906mal`. These are interiors or sub areas entered through a door. Their start and landing are near because the door and the waypoint sit at the same place in a shared frame, so these pairs do share a coordinate space.

A second weaker signal is a consistent shift. Two pairs have two links each with a shift that agrees within 1 meter but is not zero, so the modules share a frame up to a translation. These are `sta_m45aa` and `sta_m45ab` in KOTOR 1 (a shift of 27.4 meters along Y) and `153har` to `152har` in KOTOR 2 (a shift of about 48.4 meters along X and 43.6 meters along negative Y). With two links, this is weak evidence.

### What this means for streaming

- For most transitions, the target module cannot be placed beside the source module by simply keeping its coordinates. The two have unrelated origins and unrelated headings. A streaming design must compute a placement offset (and a rotation) for each pair, which the data does not provide. The transition's start and landing give one anchor point, and the facing of the waypoint and of the door give the heading. A rotation is needed because the door bearing and the waypoint facing differ.
- Only the 15 pairs listed above can load beside each other with their own coordinates, and these are small interiors.
- The one way script transitions (cutscenes, dialog hand-offs, the galaxy map) do not have a start position in the world, so they cannot be seamless in the walking sense. They can still be preloaded.
- Each module is split into rooms and areas, and the data here covers only the module level. The room layouts (LYT files) were not read, so this note does not say whether a seam would fit the geometry at the door.

## Reproducing

The program is in the scratchpad: `/private/tmp/claude-501/-Users-david-perso-kjk/c28e6014-c9df-4c72-af08-bb4c9f8c5ae1/scratchpad/lt/src/main.rs` reads the installs and writes TSV files. `/private/tmp/claude-501/-Users-david-perso-kjk/c28e6014-c9df-4c72-af08-bb4c9f8c5ae1/scratchpad/build.py` merges them into the two CSV files. Both are throwaway and are not part of the repository.
