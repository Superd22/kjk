# Custom character looks and the KOTOR mod scene

This document asks one question. If kjk replaced the KOTOR character look with a customizable head and body, what would that break or change for the mods that players of KOTOR 1 and KOTOR 2 use? It also asks what other mod scenes teach about extending a character creator without losing mods.

Today a kjk seat's look is `Persona` (`crates/sim/src/persona.rs`). A persona holds one `appearance.2da` row, one `gender.2da` row, one `portraits.2da` row and a name. The appearance row picks a whole premade body and head. kjk builds a persona from a player portrait in `crates/game/src/persona.rs`, which reads every `portraits.2da` row with `forpc` set to 1. kjk reads the player's install together with its mods. It reads the `override/` folder, `.mod` modules and KOTOR 2's Steam Workshop items (`docs/VFS.md`).

The research was done on 2026-11-16. Every claim carries a link. A claim that rests on a forum post alone says so.

## Terms used in this document

- An **appearance row** is one row of `appearance.2da`. It names a body model and texture for each body letter, a head (a `heads.2da` row in its `normalhead` column), a walking and running speed, and many other values.
- A **body letter** is one of the columns `modela` to `modelj` (KOTOR 1) or `modela` to `modeln` (KOTOR 2) of an appearance row. Each piece of body armour names one letter through its base item (`baseitems.2da`'s `bodyvar`). Letter A is the character's underwear.
- A **heads row** is one row of `heads.2da`. It names a head model and up to four darker textures that the game uses as the character falls to the dark side.
- A **portrait row** is one row of `portraits.2da`. A portrait row with `forpc` set to 1 is a choice in the original character creation.
- A **build** is one of the three body sizes a player portrait offers. KOTOR 1 calls them small, medium and large. Each player portrait names one appearance row per build.
- A **loose-file mod** is a mod whose files the player copies into `override/` by hand. A file there replaces the game's file of the same name.
- **TSLPatcher** is the mod installer the KOTOR scene has used since about 2006. **HoloPatcher** is a newer, open-source installer that reads the same instruction file. That file is called `changes.ini`.
- A **2DAMEMORY token** is a number that TSLPatcher remembers while it installs. A mod adds a row, stores the index the row received in a token, and writes that index into another table or into a creature file.
- A **Workshop item** is a mod that a KOTOR 2 player subscribes to on Steam. The game reads each item's own folder. Nothing merges two items' copies of the same file.
- A **custom look**, in this document, is any look that is not one appearance row. It could be a head chosen apart from the body, a skin tone, hair, or body shape.

## How the games tie a look together today

### What the tables hold

These numbers come from the unmodded Steam installs on this machine, read with `kjk extract` and a small 2DA reader on 2026-11-16.

| Table | KOTOR 1 | KOTOR 2 |
|---|---|---|
| `appearance.2da` rows | 509 | 671 |
| `appearance.2da` columns | 80 | 94 |
| Body letters | 10 (A to J) | 14 (A to N) |
| `heads.2da` rows | 107 | 192 |
| `portraits.2da` rows | 41 | 63 |
| Player portraits (`forpc` = 1) | 30 (15 male, 15 female) | 33 (16 male, 17 female) |
| Appearance rows used by player portraits | 90 (rows 91 to 180) | 98 (rows 91 to 180, then 543 to 668) |

In KOTOR 1 each player portrait names three different appearance rows, one per build. The three rows share one head. Across all player rows, each body letter uses 6 body models, one for each of 2 sexes and 3 builds. The underwear texture (letter A) has 18 versions, because it also follows the head's skin tone. The other letters have 6, 2 or 1 textures, so armour does not depend on skin tone.

In KOTOR 2 the three builds of a portrait point at rows with the same body models. Each body letter uses 2 body models, one per sex. The underwear texture has 6 versions, one for each of 2 sexes and 3 skin tones.

This means a player's skin tone lives in the appearance row and not in the head. A new head that is put on another head's row keeps that row's underwear colour. InSidious lists exactly this as a known issue of the [6 Player Heads](https://deadlystream.com/files/file/1418-6-player-heads) mod. The skin of its female head does not match the skin of the female underwear.

### How character creation picks a look

The original character creation lists the portrait rows that have `forpc` set to 1 and the chosen sex. The class then picks one of the portrait's three appearance columns. A KOTOR 1 soldier takes `appearance_l`, a scoundrel takes `appearance_s`, and a scout takes `appearancenumber`. KOTOR 2 maps the Jedi Guardian, Consular and Sentinel the same way. Rtas Vadum's walkthrough of the two tables documents the columns ([LucasForums archive, forum post](https://lucasforumsarchive.com/thread/196778-headsportraits2da-walkthrough)). reone implements the same rule in `PortraitSelection::getAppearanceFromCurrentPortrait` ([source](https://github.com/modawan/reone/blob/master/src/libs/game/gui/chargen/portraitselect.cpp)). kjk implements it in `crates/game/src/persona.rs`.

Because the menu is built from the table, a mod adds a playable head by adding rows. The menu shows the new portrait without any code change. This is the single most important hook the mod scene relies on.

### How armour picks a body

A piece of body armour names a body letter through its base item's `bodyvar`. The game then draws the appearance row's model for that letter, such as `PMBBM` for a medium male in letter B. kjk's probe `armour_body_from_base_item` checks this rule (`docs/TESTS.md`). A mod that makes new armour therefore ships one model per body that can wear it. In KOTOR 1 that is 6 models per letter. In KOTOR 2 it is 2.

There are only 10 or 14 letters. LoneWanderer's blog post explains how modders work around this limit by reusing rare letters such as L (the dancer outfit) and K (the flight suit) ([Deadly Stream blog, 2025-01-18](https://deadlystream.com/blogs/entry/424-workaround-for-the-limited-number-of-body-model-slots-in-kotor-and-tsl/)). The workaround costs about 20 new appearance rows per letter, one per player head, and about 40 disguise items per armour. In KOTOR 1 it needs three disguise items per head, because each class has its own body height.

### How scripts read and change a look

The game's own scripts read and change appearance rows. These calls come from the decompiled vanilla scripts kept by the community patch team ([Vanilla_KOTOR_Script_Source](https://github.com/KOTORCommunityPatches/Vanilla_KOTOR_Script_Source)).

- `EffectDisguise(row)` makes a creature look like another appearance row until the effect ends. KOTOR 1 uses it for the Taris Sith armour (`k_ptar_sitharm.nss`) and the space suits on the Leviathan (`k_plev_space_en.nss`). KOTOR 2 uses it for the space suits on Peragus (`a_toggle_suit.nss`).
- KOTOR 2 copies the player's look onto a stand-in creature twice. In the prologue on the Ebon Hawk, `a_bet3m4.nss` reads `GetAppearanceType` of the player and applies `EffectDisguise` with that row to the body lying in the medical bay. On Malachor, `k_907_enter.nss` does the same for the player's hologram.
- `ChangeObjectAppearance(object, row)` exists only in KOTOR 2. Its own scripts use it for Kreia (`a_kreia_cut.nss`, `a_kreia_fight.nss`) and Darth Sion (`a_sion_cut2.nss`).

A custom look breaks the stand-in scenes unless "the player's appearance" can still be copied whole. If `GetAppearanceType` returns a row and `EffectDisguise` applies only that row, the body in the medical bay will wear the premade head and not the player's chosen one.

### How a save holds the look

A save stores the player creature's appearance row and portrait row. Mod authors tell players to use the KOTOR Savegame Editor (KSE) to change a save's look after installing a head. An example is the [SWTOR Heads Customization Pack](https://deadlystream.com/topic/10763-modswtor-heads-customization-pack). If a mod removes or reorders rows, the save points at the wrong body. Row labels do not matter to the game. JCarter426 explains that the game uses the line number, not the label ([Deadly Stream thread, forum post, 2019-06-23](https://deadlystream.com/topic/7182-appearance2da-limit/)).

## How mods install

### TSLPatcher and HoloPatcher

A `changes.ini` lists the tables it edits under `[2DAList]`. For each table it can change a row, add a row, copy a row, or add a column (`ChangeRow`, `AddRow`, `CopyRow`, `AddColumn`). A change targets a row by its index (`RowIndex`) or its label (`RowLabel`). An added row can carry `ExclusiveColumn`, which makes the installer reuse a row that already holds the same value instead of adding a duplicate. A cell can be set to `high()`, the highest value in that column. The PyKotor wiki documents this syntax in full ([TSLPatcher 2DAList syntax](https://github.com/OpenKotOR/PyKotor/blob/master/wiki/TSLPatcher-2DAList-Syntax.md)).

The installer processes the tables before the creature and item files (`[GFFList]`) and before scripts. So a token set while adding an appearance row can be written into a creature's `Appearance_Type` field or compiled into a script.

The usual new-head install adds one `heads.2da` row, stores its index in a token, adds appearance rows whose `normalhead` is that token, stores their indexes, and adds a portrait row whose three appearance columns are those tokens. Ghost Studio, a head-building tool, generates exactly this chain. It adds the `heads.2da` row with `ExclusiveColumn=head`. It adds the `appearance.2da` row with `normalhead=2DAMEMORY0` and `ExclusiveColumn=label`. It adds the `portraits.2da` row with `ExclusiveColumn=baseresref` ([head_builder_package.py](https://github.com/CrispyW0nton/Ghost-Studio/blob/8fac98cb12c6e3253f536b8621dccd1801ce926e/src/io/head_builder_package.py)). The ModDB tutorial teaches the same steps by hand ([New Playable Character Tutorial](https://www.moddb.com/games/star-wars-knights-of-the-old-republic/tutorials/new-playable-character-tutorial)).

The result differs on every install. The new head's row index depends on which mods ran before it. Only the row's label and its column values are stable.

### Loose files

Many look mods are textures or models with the game's own file names. The player copies them into `override/`, and the file of that name wins. These mods use no table at all. A head texture mod named `PMHC04.tga` changes every character whose row uses head `PMHC04`.

### Steam Workshop items

A Workshop item cannot run an installer. It ships whole tables. Snigaroo wrote that "having two mods with the same .2da file means that one's _will_ inevitably win out" ([Why NOT to use the Steam Workshop, forum post, 2019-08-20](https://deadlystream.com/topic/7321-why-not-to-use-the-steam-workshop/)). The post adds that TSLRCM and M4-78 had to be combined into one Workshop item for this reason. DarthParametric answered a player whose head mod showed an invisible head after M4-78 was installed. He wrote, "Because M4-78 nuked your heads.2da, probably" ([forum post, 2020-01-29](https://deadlystream.com/topic/6406-modtor-ports-meetra-surik-aka-jedi-exile-female-player-head-for-tsl)). Head mod authors tell Workshop TSLRCM users to point the installer at TSLRCM's Workshop folder instead of the game folder ([Effixian's K1 to K2 Heads](https://deadlystream.com/files/file/1392-effixians-k1-to-k2-heads/)).

## The mods that touch looks

### New playable heads

These mods add rows so that a new portrait appears in character creation.

| Mod | Game | Downloads | Install |
|---|---|---|---|
| [TOR Ports: Jaesa Willsaam Female Player Head](https://deadlystream.com/files/file/1307-tor-ports-jaesa-willsaam-female-player-head-for-k1) (DarthParametric) | KOTOR 1 | 4,032 | Installer |
| [Effixian's K1 to K2 Heads](https://deadlystream.com/files/file/1392-effixians-k1-to-k2-heads/) (Effix) | KOTOR 2 | 1,436 | TSLPatcher, one per head |
| [6 Player Heads](https://deadlystream.com/files/file/1418-6-player-heads) (InSidious) | KOTOR 2 | 812 | TSLPatcher |
| [Keanu Reeves Inspired Player Head](https://deadlystream.com/files/file/3041-keanu-reeves-inspired-player-head-k1/) | KOTOR 1 | 156 | TSLPatcher |

Effixian's K1 to K2 Heads tells players to restore `appearance.2da`, `heads.2da` and `portraits.2da` to uninstall it, so it edits all three. The Keanu Reeves head describes itself as "merge-safe" for the same three tables and asks to be installed after large restoration packages. The Jaesa Willsaam head notes that KOTOR allows at most 16 bones per mesh. Its author removed two nose bones to fit, and the neck may show gaps on some bodies.

Some head mods add more than one head. 6 Player Heads adds five male heads and one female head, each with its own dark-side textures. Twin Mullet Men adds two heads based on the KOTOR 1 head `PMHC04` ([Twin Mullet Men](https://deadlystream.com/files/file/451-twin-mullet-men)).

Hair is not a separate part in these mods. DarthParametric's SWTOR ports merge the face, eye and hair textures into one texture so the dark-side textures still work. The [SWTOR Heads Customization Pack](https://deadlystream.com/topic/10763-modswtor-heads-customization-pack) offers four hair colours for three heads, but each colour is a separate download of replacement textures.

### Head and portrait replacements

These mods replace an existing head's files by name. They add no rows.

- [Revan's Head](https://deadlystream.com/files/file/939-revans-head) (Darth Vhail) changes the fourth Caucasian male head of KOTOR 1 to look like Revan from SWTOR. It ships head textures, dark-side textures and portraits as loose files. It has 17,221 downloads, more than all four head-adding mods above together.
- Snigaroo's KOTOR 1 full build lists Dark Hope's HD versions of the heads `PMHA05`, `PMHA02`, `PMHA01` and `PFHC05`, a fix for `PFHB02`'s dark-side eyes, Detran's Darth Revan, ndix UR's HD PC Portraits and Tinman888's Stylized Portraits. All of them are loose files ([KOTOR 1 full build](https://kotor.neocities.org/modding/mod_builds/k1/full)).
- Snigaroo's KOTOR 2 full build lists HD PC Portraits, PFHC03 Restoration, PFHC06 Fix, Redrob41's Head Model Fixes and Canonical Jedi Exile, which reskins one player head ([KOTOR 2 full build](https://kotor.neocities.org/modding/mod_builds/k2/full)). Only PFHC03 Restoration uses TSLPatcher.
- Ultimate Character Overhaul and its REDUX version (ShiningRedHD, Red_BY) upscale the textures of every character, the player heads and bodies included. The player drops the files into `override/` ([Nexus Mods, KOTOR 2 REDUX](https://www.nexusmods.com/kotor2/mods/1105)). Both builds include it. The Workshop has an official REDUX item too.

The full builds are the community's reference install lists. Neither build includes a mod that adds a new playable head. Every player-head mod in them replaces a head or portrait by name.

### The community patches

The KOTOR 1 Community Patch (K1CP) version 1.10.0 changes 47 rows of `appearance.2da` by index and adds 1 row ([changes.ini](https://github.com/KOTORCommunityPatches/K1_Community_Patch/blob/master/tslpatchdata/changes.ini)). None of the 47 rows is a player row. They fix NPCs, such as row 261 (a Sith soldier's texture) and rows 315 to 411 (commoners, droids and aliens). The added row is a male Twi'lek, with `ExclusiveColumn=label`. The build tells players to install K1CP before everything else.

The KOTOR 2 Community Patch (TSLCP) changes 90 player rows, rows 91 to 180 ([changes.ini](https://github.com/KOTORCommunityPatches/TSL_Community_Patch/blob/master/tslpatchdata/changes.ini)). For each row it sets `walkdist` and `driveanimwalk` to 2.6, and `rundist` and `driveanimrun_pc` to 3.96. This fixes how fast the player's feet move against the ground. TSLCP does not change the 14 player rows past 180. Five of KOTOR 2's 33 player portraits use those rows, so their walk stays at 1.6 or 1.813 and their run at 3.66.

This is a real case of behaviour stored per appearance row. A custom look that drops the row also drops TSLCP's fix, unless kjk keeps reading these columns.

### Bodies, robes and armour

- [JC's Fashion Line I: Cloaked Jedi Robes for K1](https://deadlystream.com/files/file/1378-jcs-fashion-line-i-cloaked-jedi-robes-for-k1/) (JCarter426) replaces the Jedi robe models of the player and many NPCs. It also replaces the `S_Male` and `S_Female` supermodels, the skeletons that every human body shares. Its gloveless options patch `appearance.2da`. The author says the gloveless options work only for the player's original appearance rows ([mod thread](https://deadlystream.com/topic/6701-modjcs-fashion-line-i-cloaked-jedi-robes-for-k1)). The K1 build pairs it with JC's Jedi Tailor, Cloaks with Shadows and Qel-Droma Reskin.
- Sith Uniform Reformation Revised and Korriban Back in Black offer a "Community Patch compatible" option in their installers. The K1 build tells players to pick it.
- The K1 build has a step that removes TGA files that a TPC of the same name would hide. It says a duplicate TPC takes priority and can crash the game.
- [Jedi from the Start](https://deadlystream.com/files/file/174-jedi-from-the-start/) (darthbdaman) has 22,350 downloads. It lets the player become a Jedi on the Endar Spire and gives robes from the starting locker. Its page names no table changes for looks. It is listed here because it changes what the player wears early, so it relies on robe models existing for the player's body.
- Bao-Dur Can Wear Heavy Armor (Effix, TSLPatcher) changes which armour a companion may wear.

### Scripts, disguises and appearance items

- [KK's Disguise Armband](https://deadlystream.com/files/file/808-kks-disguise-armband) (KOTOR 1) is an item the player equips and uses. It opens a dialogue in which the player picks a head, a body or both. The page warns that reordering `appearance.2da` points every choice at the wrong entry. The mod therefore depends on fixed row numbers. It has four versions for different sets of other installed mods.
- LoneWanderer's armour workaround replaced `ChangeObjectAppearance` calls in cut-scene scripts with disguise items, one per player head ([blog post](https://deadlystream.com/blogs/entry/424-workaround-for-the-limited-number-of-body-model-slots-in-kotor-and-tsl/)).
- Players ask on the forums how a script can tell the player's own appearance row apart ([LucasForums archive, forum post](https://lucasforumsarchive.com/thread/205830-can-a-script-recognize-the-pcs-line-from-appearance2da)). Mod scripts copy the player's row the same way the vanilla medical bay scene does.

### The Steam Workshop for KOTOR 2

The 30 most subscribed KOTOR 2 Workshop items on 2026-11-16 include TSLRCM (first), the TSLRCM and M4-78 combined item (sixth), Visas HD Re-Skin (fifth), Ultimate Character Overhaul REDUX (twelfth), Fix Head Model Clipping (22nd), Complete Character Overhaul Ultimate HD Pack (27th) and Canonical Jedi Exile 1.2 (28th) ([Workshop, most subscribed](https://steamcommunity.com/workshop/browse/?appid=208580&browsesort=totaluniquesubscribers&section=readytouseitems&actualsort=totaluniquesubscribers&p=1)). The page shows titles only. The look items among them are texture and model replacements. No item in the top 30 adds a playable head by its title.

Snigaroo's KOTOR 2 build says "the Workshop version of this or any other mod is NOT to be used!" It asks Steam players to check that they are not subscribed to anything. The machine used for this research has 10 Workshop items installed, TSLRCM (485537937) among them (`kjk games`).

## Prior art

### Other KOTOR engines and tools

- **reone** reads `portraits.2da` and lists the rows with `forpc` set to 1 and the chosen sex ([portraitselect.cpp](https://github.com/modawan/reone/blob/master/src/libs/game/gui/chargen/portraitselect.cpp), [portraits.cpp](https://github.com/modawan/reone/blob/master/src/libs/game/portraits.cpp)). It then picks the class's appearance column. A mod's new head appears in reone's creator. reone's look stays one appearance row.
- **xoreos** builds the player's file names from parts. A portrait is `po_p` plus `mh` or `fh`, a skin letter `a`, `b` or `c`, and a face number. The head and body models are built the same way ([chargeninfo.cpp](https://github.com/xoreos/xoreos/blob/master/src/engines/kotor/gui/chargen/chargeninfo.cpp), [creature.cpp](https://github.com/xoreos/xoreos/blob/master/src/engines/kotor/creature.cpp)). Its random characters pick a skin from 3 and a face from 5. xoreos never reads `forpc`. A head that a mod adds as new rows does not appear. A head that replaces files by name still shows, because xoreos asks for the same names.
- **KotOR.js** reads `forpc` into its portrait records ([SWPortrait.ts](https://github.com/KobaltBlu/KotOR.js/blob/master/src/engine/rules/SWPortrait.ts)). Its character creator steps through a fixed list of appearance rows per class written in its code, such as rows 136, 139, 142 and so on for the male scout ([CharGenClasses.ts](https://github.com/KobaltBlu/KotOR.js/blob/master/src/game/CharGenClasses.ts)). A mod's new head does not appear in its creator.
- **Holocron Toolset** edits creature files with drop-down lists filled from the install's `appearance.2da`, `heads.2da` and `portraits.2da`. It shows the dark-side portrait columns too ([utc.py](https://github.com/OpenKotOR/HolocronToolset/blob/master/src/toolset/gui/editors/utc.py)). The tool treats the tables as the list of choices, as the games do.

None of these projects has a head chosen apart from the body. No community project that replaces KOTOR's character creator with a free head and body was found. The closest the scene has come is KK's Disguise Armband, which lets a player pick a head and a body inside the game through dialogue.

### OpenMW (Morrowind)

OpenMW's race menu builds its head and hair lists from every loaded body part record. It keeps a record if the record is not flagged as not playable, is a skin part, is a head or hair part, matches the chosen sex, and belongs to the chosen race ([race.cpp, `RaceDialog::getBodyParts`](https://github.com/OpenMW/openmw/blob/master/apps/openmw/mwgui/race.cpp)). A mod adds a head by adding one record. No central table holds the list, so two head mods never fight over one file. Head packs such as MacKom's Humanoid Heads and Westly's head packs install this way ([modding-openmw.com list of head and body mods](https://modding-openmw.com/lists/total-overhaul-bodies-and-heads)).

### RaceMenu (Skyrim)

RaceMenu is an SKSE plugin that extends Skyrim's creator. Its code lists every head part that is flagged playable, is valid for the player's race and matches the player's sex ([SKEEHooks.cpp](https://github.com/expired6978/SKSE64Plugins/blob/479ecabf8e1208a31dae1e492dae3b851ec26fa2/skee64/SKEEHooks.cpp)). So a hair or eye mod appears in RaceMenu with no patch. RaceMenu's presets store each head part together with the name of the plugin that defines it ([PresetInterface.cpp](https://github.com/expired6978/SKSE64Plugins/blob/479ecabf8e1208a31dae1e492dae3b851ec26fa2/skee64/PresetInterface.cpp)). A preset then still finds the right part when the load order changes. Overlay and slider mods tell players to raise slot counts in `NiOverride.ini` or tint resolution in `SKSE.ini` (mod pages, for example [Nexus Mods 60769](https://www.nexusmods.com/skyrim/mods/60769?tab=docs)).

### LooksMenu (Fallout 4)

LooksMenu (F4EE) loads extra creator data per mod from folders named after each plugin. It reads `F4SE\Plugins\F4EE\Tints\<plugin>\templates.json` for skin tints and `F4SE\Plugins\F4EE\LUTs\<plugin>\haircolors.json` for hair colours ([CharGenInterface.cpp](https://github.com/expired6978/F4SEPlugins/blob/master/f4ee/CharGenInterface.cpp)). A form is named as `Plugin.esp|FormID`, and the plugin's current load index is looked up when the name is read ([Utilities.cpp](https://github.com/expired6978/F4SEPlugins/blob/master/f4ee/Utilities.cpp)). A mod extends the creator by shipping a file. It never edits a shared file.

### Baldur's Gate Enhanced Edition

The Enhanced Editions read custom portraits from a `portraits` folder in the player's documents. A portrait comes in up to three sizes whose names end in L, M and S. This rests on forum posts only ([Beamdog forum](https://forums.beamdog.com/discussion/78788/solved-can-i-still-use-small-medium-and-large-custom-portrait-sizes), [Sorcerers.net](https://sorcerers.net/community/threads/problems-with-custom-portraits.78494)). Players also report that newer patches resize a lone L file. The lesson is that a folder of files that the game discovers by name is easy for players and never conflicts.

### What the prior art agrees on

Every system that kept its mod scene does three things. It builds the creator's lists from data, not from code. It lets a mod add a choice without editing a shared list. It names a choice by something stable, such as a plugin and record, and not by its position in a list.

KOTOR's own system meets the first point and fails the other two. The scene built TSLPatcher, `ExclusiveColumn` and tokens to work around those two failures.

## What a custom look would break or keep, by category

### New playable head mods

This category breaks if kjk's creator stops listing portrait rows. The heads these mods add would then vanish from character creation. The mods change nothing else, so they would do nothing at all.

This category can be kept whole. kjk can read every portrait row with `forpc` set to 1, as it does now. From that row it can take the head (the `normalhead` of its appearance rows), the sex, the portrait and its four dark-side portraits. That head becomes one choice in the new creator. The mod's dark-side head textures still apply, because they live in the heads row.

One thing changes. The mod's three appearance rows carry a skin tone through their underwear texture. If the player picks this head with another skin tone, the head and body will not match. kjk should take the skin tone of a mod head from its own appearance row, or show the head with its own row's body only.

### Head and portrait replacement mods

Nothing in this category breaks, as long as kjk draws heads by their model and texture names. These are the most downloaded look mods and the ones the full builds use.

One thing changes if kjk adds new parts such as separate hair. A head that a mod replaced may then show kjk's hair over the mod's own hair. The mod's head was made with its hair merged in. kjk should not add separate hair to a head that a mod changed, or should let the player turn it off.

### Body, robe and armour model mods

A body shape that is not one of the existing ones breaks this category. KOTOR 1 armour exists as 6 models per letter and KOTOR 2 armour as 2. A new build, a height slider or a weight slider would need every armour mod to ship models for it. None do. Supermodel mods such as Cloaked Jedi Robes replace the shared skeleton, so a body that needs its own skeleton would also lose them.

This category can be kept whole if the custom body stays one of the original bodies, chosen by sex and build. Every armour model mod then keeps working. Texture mods for armour keep working too, because armour textures do not depend on skin tone.

### The community patches

TSLCP's speed fix lives in 90 appearance rows. A look that no longer has an appearance row would lose it.

The fix can be kept. kjk can keep one appearance row per seat as the "body row", chosen from the body's sex and build, and read its speeds, sizes and sounds from there. TSLCP's fix then applies as it does today.

### Workshop items and whole-table mods

Nothing new breaks here. A Workshop item that ships a whole `heads.2da` already hides other items' heads. That is how the games behave too.

One thing can improve. kjk could tell the player when a portrait row points at a head row or appearance row that does not exist. The invisible head after M4-78 is the example to catch.

### Disguise and appearance-changing scripts

Scripts that copy the player's look with `GetAppearanceType` and `EffectDisguise` break. KOTOR 2's medical bay and Malachor scenes do this. Mods such as KK's Disguise Armband also pick fixed rows.

Both kinds of disguise can be kept. A disguise to a row that is not a player's, such as a space suit, Sith armour or a droid, should replace the whole look as it does now. When it ends, the custom look should come back. A disguise to the row that `GetAppearanceType` returned for the player should copy the player's custom look whole.

### Saves

A custom look saved as kjk data has no meaning to KSE or to the original games. A save that holds only a row number breaks when a mod adds or removes rows before it.

Saves can still survive mods. kjk can save the custom look by stable names, such as the head's model name, the portrait's name and a mod row's label. It can keep a row number only as a fallback.

## What kjk should promise mods

1. kjk will build the creator's head list from `portraits.2da`. Every row with `forpc` set to 1 will stay a choice, with its sex, its head and its portrait. A head that a mod adds with TSLPatcher, HoloPatcher or a Workshop item will appear without a patch.
2. kjk will draw every head, body and portrait by the model and texture names the tables give. A loose-file mod that replaces `PMHC04`, `po_pmhc4` or `PMBBM` by name will change what kjk draws, as it does in the games.
3. kjk will read a head's dark-side textures from `heads.2da` and a portrait's dark-side images from `portraits.2da`. A head mod's alignment steps will keep working.
4. kjk will keep the original bodies. A custom body will be one of the 6 KOTOR 1 bodies or 2 KOTOR 2 bodies, with any extra choice limited to textures. Every armour, robe and supermodel mod will keep working. A new body shape will not ship until armour for it exists, and it will fall back to the nearest original body for armour that lacks it.
5. kjk will keep one appearance row behind every look, chosen by sex and build. It will read movement speeds, size, sounds and every other non-visual column from that row. TSLCP's speed fix and any similar mod change will keep applying.
6. kjk will match the body's skin tone to the head's own appearance row by default. A mod head whose rows name a skin tone will not appear on the wrong underwear unless the player chooses it.
7. kjk will treat `EffectDisguise` and `ChangeObjectAppearance` to any row as a full replacement of the look for as long as the effect lasts. It will then restore the custom look.
8. kjk will make `GetAppearanceType` of a player return its body row. A disguise to that same row, as KOTOR 2's medical bay scene applies, will copy the player's whole custom look onto the target.
9. kjk will save a custom look by stable names. These are the head's model name, the portrait's resource name and, for a row a mod added, the row's label. A save will still load if a mod is installed or removed and rows move. A part that is gone will fall back to the body row's own head and be named in the log.
10. kjk will not read new look data from a shared file that mods must edit. If kjk adds new kinds of choice, such as hair or skin tones, a mod will add them by shipping its own file with its own name, as LooksMenu and OpenMW let mods do.
11. kjk will say in the log when a portrait row points at a missing head row, a missing appearance row or a missing model. The player will learn which table lost the head instead of seeing an invisible one.
12. kjk will include the custom look's parts in the co-op handshake by name. A guest that lacks a mod's head will be told which head it lacks, as it is told today for missing stance sets (`docs/VFS.md`).

## Gaps

- No download ranking for player-head mods on Nexus Mods was found. Nexus pages refused automated reads. The download counts here are Deadly Stream's.
- The exact `changes.ini` of a typical head-adding mod was not read. The mods' archives are on Deadly Stream behind its download page. The token chain described above comes from Ghost Studio's generator, the ModDB tutorial and the mods' uninstall notes.
- The Workshop's most-subscribed page lists titles only. Whether any Workshop item adds a playable head was judged from titles.
- KOTOR Tool's source is not public, so it is not covered.
