# What a custom head and body would change in KOTOR 1, KOTOR 2 and kjk

This note was written on 2026-11-16. It asks one question. Suppose kjk let the player build a head and a body of their own, instead of picking one premade look. What in the two games and in kjk would break, what would need a rule, and what would keep working?

Today a player's look is one `Persona` (`crates/sim/src/persona.rs:23-34`). A persona holds four things. It holds one `appearance.2da` row, which picks the whole body and the head together. It holds one `gender.2da` row, one `portraits.2da` row and a name. The persona travels in saves and in co-op snapshots.

Two companion notes cover the design side and the technology side. They are `docs/research/2026-11-16-character-creation-design.md` and `docs/research/2026-11-16-character-creation-tech.md`.

## How this was checked

Every claim below comes from one of four kinds of source.

- kjk's own code and docs, cited as `file:line`.
- The games' own tables, read with `kjk show k1 <table>.2da` and `kjk show tsl <table>.2da` on the Steam installs of both games.
- The games' compiled scripts. A throwaway scanner read every `.ncs` file in `chitin.key`'s BIF archives, in every module RIM and MOD, in `Patch.erf` and in `override`. It listed each script that calls a given routine. Each hit was then read with `kjk script dis`. The scanner numbered the routines from each game's own `nwscript.nss`. It agrees with kjk's listing, which puts `GetAppearanceType` at 524 in both games.
- reone's source on GitHub, where it shows how another engine reads the same data.

The scanner also read every item template (`.uti`) in both games for the Disguise item property (property 59).

## Terms used in this note

- An **appearance row** is a row of `appearance.2da`. A player row names a body model for each armour letter (`modela` to `modelj` in KOTOR 1, `modela` to `modeln` in KOTOR 2). It names a texture for each letter (`texa` to `texj` or `texn`). It names a head (`normalhead`, a row of `heads.2da`).
- An **armour letter** is the body variation a worn item asks for. It comes from `baseitems.2da`'s `bodyvar` column for the item's base item. A character wearing no body armour uses letter `a`, the **bare body**.
- A **build** is one of the three body sizes a player portrait offers. `portraits.2da` names a small row (`appearance_s`), a middle row (`appearancenumber`) and a large row (`appearance_l`).
- A **skin group** is the letter A, B or C in a player head's name (`PFHA01`, `PFHB01`, `PFHC01`). KOTOR 2 adds a fourth group, H.
- A **dark step** is how far a character has fallen to the dark side, from 0 to 4. kjk takes the steps at an alignment of 40, 30, 20 and 10 (`docs/UI.md:153-166`).
- A **stand-in** is a creature a script makes to stand for the player in a scene, such as a vision or a hologram.
- A **custom look** in this note means a head and a body that the player picks or shapes separately. It may use the games' own head and body models, or new ones.

## How the games build a player today

KOTOR 1 has 30 player portraits, 15 for women and 15 for men. Each portrait names three appearance rows, one for each build. That makes 90 player rows (`portraits.2da` rows 1 to 32 with `forpc` 1).

In KOTOR 1 every player row of one sex and one build names the same ten body models. A woman's small build always uses `PFBAS` to `PFBJS`. A man's large build always uses `PMBAL` to `PMBJL`. Only two things differ between the 15 rows of one sex and build. The first is the head (`normalhead`, `heads.2da` rows 26 to 55). The second is the bare body's texture (`texa`), which follows the head's skin group. Row 91 uses `PFBASA` for skin group A. Row 106 uses `PFBASB` for skin group B. Row 121 uses `PFBAS` for skin group C.

So KOTOR 1 already builds a player from separate parts. It takes a body set by sex and build, and adds a head and a matching bare skin. A custom look that chooses the head and the body set separately follows the same structure.

KOTOR 2 has 34 player portraits, with 98 player rows. Every KOTOR 2 player row uses the middle build, whatever the class. The three columns of a portrait name rows whose bodies are all `PFBxM` or `PMBxM`. KOTOR 2 still ships the small and large body models (`pfbas.mdl`, `pmbjl.mdl` and so on, for letters A to J). Its table never uses them.

The player's movement numbers in `appearance.2da` depend only on sex and game. A woman's walk cycle covers 1.6 m and a man's covers 1.813 m (`walkdist`). A run cycle covers 3.96 m in KOTOR 1 and 3.66 m in KOTOR 2 (`rundist`). Every player row has a `height` of 1.6, a `hitradius` of 0.25 and a personal space of 0.35 m (`perspace`).

kjk picks the row the same way. `crates/game/src/persona.rs:18-30` maps a soldier and a Jedi Guardian to the large build, a scoundrel and a Jedi Consular to the small build, and any other class to the middle build. `Portraits::persona` (`crates/game/src/persona.rs:69-82`) turns a portrait and a class into a persona.

## Things that would visibly break

### The Exile's stand-ins in KOTOR 2 copy only the appearance row

KOTOR 2 makes two stand-ins of the player by copying the player's appearance row. Both scripts call `GetAppearanceType` on the player and give the result to `EffectDisguise` on the stand-in.

- The prologue on the Ebon Hawk (`001EBO`, script `a_bet3m4`) creates `c_medbaypc`, the unconscious Exile lying in the medical bay. Its template has appearance row 14. The script then disguises it as the player's row (`kjk script dis tsl a_bet3m4 --module 001EBO`, instructions `0x285` to `0x2eb`).
- The last scenes on Malachor V (`907MAL`, script `k_907_enter`) create `c_hologrampc`, a hologram of the Exile (template field `Hologram` 1). The same pair of calls disguises it as the player's row.

A disguise effect carries one number, an appearance row. It cannot carry a separate head or a separate body set. With a custom look, these two stand-ins would show the premade row that the persona keeps, not the player's custom face. kjk's disguise path draws the row's own head (`crates/game/src/made.rs:1058-1089`).

A fix needs a rule. One option is that an `EffectDisguise` whose row equals the player's own row copies the player's whole custom look. kjk's `GetAppearanceType` already answers with the persona's row (`crates/sim/src/script/engine.rs:2830-2856`), so the sim could tag such a disguise when it is applied.

### The head-copy scenes would show the premade face

Two scenes give a story character the player's face with `DuplicateHeadAppearance`. Each game uses it in exactly one script.

- KOTOR 1's Revan reveal (`STUNT_31b`, script `k_scene_start`) creates Darth Revan in the vision. It uses `n_darthrevan001` (appearance row 371) for a male player and `n_darthrevan003` (row 372) for a female player, chosen with `GetGender`. Both rows have Revan's own head, `heads.2da` row 66. The script then calls `DuplicateHeadAppearance(Revan, player)`, so Revan wears the player's face.
- KOTOR 2's last vision in the Korriban cave (`711KOR`, script `tr_last_vision`) creates the dark Exile. It uses `g_darkpc` (row 128) for a female player and `g_darkpc_male` (row 173) for a male player. Both wear a dark robe (`a_robe_09`). The script then copies the player's head onto it.

kjk implements `DuplicateHeadAppearance` by reading the other creature's appearance row and taking that row's `normalhead` (`crates/sim/src/script/presentation.rs:453-470`, `crates/sim/src/script/mod.rs:2113-2120` and `588-598`). The result is kept as one byte, a `heads.2da` row (`crates/sim/src/combat/fighter.rs:454-461`). With a custom look, kjk would copy the premade row's head, not the player's chosen one. A head that is not a `heads.2da` row cannot be kept in that byte at all.

These two scenes are the strongest story moments that depend on the player's face. They must copy the whole custom head, with its textures and its dark step.

### A custom face has no portrait

Both games show the player's portrait as a premade picture from `portraits.2da`. A custom face has no such picture. These places in kjk read the persona's portrait row.

- The HUD's portrait, at the persona's dark step (`crates/game/src/hud.rs:618`).
- The character sheet and the party pages (`crates/game/src/character.rs:777`, `crates/ui/src/screens/tabs.rs:133-145`).
- The co-op lobby's hero slot. The host turns the persona's row into a texture name (`crates/game/src/lobby_host.rs:263-267`). The lobby then sends that name to the guests as a string (`crates/session/src/lobby.rs:40` and `59`). A guest would need a way to draw a portrait that is not a file in its install.
- Character creation's portrait label (`crates/ui/src/screens/chargen.rs:1044` and `1359`).

Each player portrait has darker versions for the dark steps (`portraits.2da` columns `baseresrefe` to `baseresrefvvve`, read by `crates/asset_odyssey/src/creature.rs:116-141`). KOTOR 1 has four darker pictures for each portrait. KOTOR 2 has two distinct ones (`d1` for steps 1 to 3, `d2` for step 4). A custom portrait needs the same darkening.

kjk already renders the character being made, with its head on its own stage, during character creation (`crates/game/src/gui_scene.rs:691-710`). That code could render a portrait from the custom head. reone shows another engine's fallback. When a creature has no portrait row, reone looks for a portrait whose `appearance_s`, `appearancenumber` or `appearance_l` names its appearance row ([reone `portraits.cpp`](https://github.com/seedhartha/reone/blob/master/src/libs/game/portraits.cpp#L62-L66), [reone `creature.cpp`](https://github.com/seedhartha/reone/blob/master/src/libs/game/object/creature.cpp#L109-L112)).

kjk's save screen shows no portrait today. It hides the three party portrait slots (`LBL_PM1` to `LBL_PM3`) and shows the module's loading image instead of a screenshot (`crates/ui/src/screens/save_load.rs:56-57` and `153-160`). So saves are not affected yet.

### The player's voice is found through the portrait

kjk finds the player's sound set, the grunts and battle cries, through the portrait row. The view names a player's sound line `pc<portrait row>` (`crates/game/src/ambience.rs:1091-1094`). It then reads that portrait's `sex` column and picks the `soundset.2da` row labelled `Player_Female_W` or `Player_Male_W` (`crates/asset_odyssey/src/creature.rs:846-866`, `crates/game/src/ambience.rs:298-310`). A custom look without a portrait row would lose the player's grunts in KOTOR 1. The sound set should be chosen from the persona's gender instead. KOTOR 2 has no player sound set rows, so its player has none in either case.

### The bare body's skin must match the head

The bare body's texture follows the head's skin group, as shown above. Armour textures do not. In KOTOR 1 every row of one sex and build uses the same textures for letters `b` to `j` (`PFBBS`, `PFBC`, `PFBD` and so on). Only `texa` differs by skin group. KOTOR 2 has two textures that follow the skin group. They are `texa` (`PFBAMA`, `PFBAMB`, `PFBAMC`) and the slave outfit's `texl` (`PFBLA`, `PFBLB`, `PFBLC`).

If the player paired a head of one skin group with a bare body of another, the arms, legs and neck would not match the face. This shows whenever the player wears no body armour. A custom look must either tie the bare body's texture to the head's skin group, or give the body its own skin colour and recolour the head to match.

The dark side body texture is a known exception in the original data. KOTOR 1 uses one dark bare texture for each build, whatever the skin group (`texaevil` is `PFBASD`, `PFBAMD` or `PFBALD`). KOTOR 2 uses `PFBAMD` and `PMBAMD` for every player row.

### The face must darken with the dark side

Each player head has its own dark textures. `heads.2da` gives one texture for each dark step in the columns `headtexe`, `headtexve`, `headtexvve` and `headtexvvve`. KOTOR 1's head 26 uses `PFHA01D1`, `PFHA01D2`, `PFHA01D3` and `PFHA01D`. KOTOR 2's head 26 uses `PFHA01D1` for steps 1 to 3 and `PFHA01D2` for step 4. KOTOR 2 also draws a head's `alttexture` when it has no dark texture for the step. Some KOTOR 2 heads use a different texture than the model's own (head 54 uses `PMHC04A`).

kjk applies these in `creature::align` (`crates/asset_odyssey/src/creature.rs:162-197`) for every look (`crates/game/src/alignment_look.rs:19-25`). A custom head that reuses a stock head keeps working. A recoloured head, or a new head model, needs its own four dark textures in KOTOR 1 and two in KOTOR 2. Without them the face stays light while the portrait darkens. The approved test `alignment_looks` pins the current steps (`crates/approved_tests/tests/approved.rs:471-490`).

## Things that need a rule decided

### Which appearance row the persona keeps

The scripts read the player's look through `GetAppearanceType`. kjk answers with the persona's row, unless a disguise or a script's change holds the player (`crates/sim/src/script/engine.rs:2830-2856`). A custom look still needs a real player row behind it, for three reasons.

- The stand-ins and head copies above take this row.
- KOTOR 1's `k_sp1_generic` (the Force powers' shared script) compares `GetAppearanceType` of a target with droid and turret rows: 59, 60, 61, 65, 182 and 183. KOTOR 2's copy adds rows 640 and 657. No script in either game compares the result with a player row. A player row is therefore safe for every script.
- Walk and run strides, the bare body's texture and the dark bare body all come from the row (`crates/game/src/persona.rs:253-273`).

One possible rule is that the persona keeps the nearest premade row. That is the row of the same sex and build, whose head has the same skin group. The custom look then sits on top of that row.

### How gender relates to the body and the head

The scripts read the player's gender far more than the look. `GetGender` appears in 874 compiled script copies in KOTOR 1 and 396 in KOTOR 2. The Revan reveal picks Revan's body by gender. The Korriban vision picks the dark Exile's body by gender. KOTOR 2's prologue stores the gender in the global `000_PLAYER_GENDER` (`a_bet3m4`).

The animation also follows the body's sex. KOTOR 1 has one skeleton for every male body and another for every female body. kjk fits JA's motion to each in its own way (`docs/SKELETON.md:67-80`). Character creation frames the head from `camerahookm` or `camerahookf` by sex (`crates/game/src/gui_scene.rs:703-704`).

The simplest rule is that the body's sex is the persona's gender. A player who wants a male head on a female body, or the reverse, is a separate decision. Both games' heads carry the same face nodes (`f_jaw_g`, `eyeLlid`, `eyeRlid`, `talkdummy`, `GoggleHook`, `MaskHook` and the brow and lip nodes, read from `pfha01.mdl` and `pmhc05.mdl` in both games). No premade row ever puts a head of one sex on a body of the other, so the fit at the neck is untested.

### Whether builds stay tied to class

KOTOR 1 ties the build to the class. KOTOR 2 uses the middle build for every class. No script reads the build. A custom look could let the player choose the build freely.

KOTOR 2's small and large bodies cover letters A to J only. KOTOR 2 has no small or large body for the flight suit (K), Zeison Sha armour (M) or the Jedi Knight and Jedi Master robes (N). It ships only `pfbkm`, `pfbmm`, `pfbnm`, `pmbkm`, `pmbmm` and `pmbnm`. A small or large player in KOTOR 2 would need a rule for those three letters. The player could fall back to the middle body for them, or fall back to the bare body as kjk does for rows that name no model (`QUIRKS.md` Q-0137, `crates/asset_odyssey/src/creature.rs:923-930`).

KOTOR 2 also has no male slave outfit. Male player rows leave `modell` empty, so a man in the slave outfit is drawn in his bare body (Q-0137).

### What a different height or shape would do

Every premade player row has the same `height` (1.6). A custom body that is taller, shorter or broader would change only what is drawn.

- The sim's body is JA's player box, not the KOTOR model. Its personal space comes from the row's `perspace` (`crates/sim/src/world.rs:697-714`). Collision would not change.
- The sim's blades come from JA's own skeleton (`models/players/kyle`), not from the KOTOR body (`docs/SKELETON-SIM.md:9-17`). The drawn hand reaches for the sim's blade by at most 0.3 m (`docs/SKELETON.md:93-98`). It stops when the blade would pass within 0.2 m of the head's middle. A body whose arms end far from Kyle's would show a gap between the hand and the blade.
- The conversation camera reads each body's `camerahook` and head as drawn (`docs/DIALOGUE.md:1063-1067`). The camera would follow a taller or shorter head without change.
- The walk and run cycles cover the row's `walkdist` and `rundist`. A longer or shorter leg would slide its feet unless those distances change with it.

### What new head models must carry

A custom head that reuses one of the games' head models needs nothing new. A new head model must carry the nodes the engine and the clips use.

- `GoggleHook` and `MaskHook`, where masks and visors hang (`crates/game/src/viewer.rs:2006-2030`).
- `talkdummy`, which reone uses to aim at a head ([reone `creature.cpp`](https://github.com/seedhartha/reone/blob/master/src/libs/game/object/creature.cpp#L424-L427)) and kjk uses for head hits (`crates/game/src/script_fx.rs:265`).
- The face nodes the body's clips move. kjk plays the body's clips on the head's nodes, so the head blinks, sleeps and dies with the body (`crates/game/src/face.rs:1-8`). A conversation speaker moves its lips with its head's `talk` clip (`crates/game/src/dialogue_view.rs:1190-1230`).

The player does not speak voiced lines. In Trask's conversation on the Endar Spire (`end_trask01.dlg`), every voice file named is Trask's. So the lips matter only for the stand-ins, and only if a scene gives them a voice.

### Saves, co-op and the format version

A custom look adds data to the persona. These places carry it today.

- The snapshot's tail encodes each persona as a seat byte, a 16-bit row, a gender byte, a 16-bit portrait row and a name (`crates/sim/src/persona.rs:41-82`). New fields need a new snapshot version (`crates/sim/src/snapshot.rs:357`). The approved test `old_saves_load` then needs a new fixture (`crates/approved_tests/tests/approved.rs:5191-5215`).
- A save's `story.json` keeps each player's name, appearance, gender and portrait (`crates/saves/src/story.rs:113-117` and `444-453`). It rebuilds the persona from them (`crates/saves/src/story/restore.rs:340-359`). The story's schema only grows, so the new fields would be added beside the old ones.
- A save's `meta.json` keeps the character made in character creation as a `CharacterDesc` (`crates/saves/src/lib.rs:5`, `crates/ui/src/intent.rs:20-30`). It names the portrait by its texture.
- The playtest builds name a portrait (`data/routes/tsl/exile.build:15`, `data/routes/tsl/exile_fighter.build:11`, read by `crates/launcher/src/playthrough/build.rs:184`).
- The co-op handshake compares the files the resolvers read for world data (`docs/VFS.md:166-194`, `docs/COOP.md:200-210`). If a custom look used new models or textures that kjk ships, every machine in a session would need the same ones. The snapshot carries only names and numbers.

The probe `player_persona_scripts` pins what the scripts read of a persona (`crates/approved_tests/tests/approved.rs:4670-4677`). The probe `body_dressed_by_armour` pins how K1's rows 137 and 92 dress (`crates/approved_tests/tests/approved.rs:4679-4700`). Both would grow to cover the custom look.

### The main menu's Exile and the original game

After a dark side ending of KOTOR 2, kjk shows the player's own character on the main menu. It keeps that figure as an appearance row, an armour and an alignment (`crates/game/src/menu_figure.rs:60-64`, drawn by `crates/game/src/gui_scene.rs:418`). The figure would need the custom look too.

`kjk original` starts the original game to compare a bug with kjk. The original game knows only premade rows. It cannot show a custom look, so a comparison would use the nearest premade row.

## Things that just work

### kjk already draws a body and a head separately

kjk's render side already treats the body and the head as two models. A seat's `Wear` names a body, a head, a head texture, a body texture, the headgear, the strides and the env map (`crates/game/src/play.rs:212-226`). `play::dress` loads the body, hangs the head on the body's `headhook` and hangs the headgear on the head (`crates/game/src/play.rs:1247-1297`). `kjk play --body <model> --head <model>` already draws any pair (`crates/game/src/play.rs:183-206`). Only `persona::wear` ties the pair to one row (`crates/game/src/persona.rs:253-273`). The wardrobe dresses a player again whenever its look changes (`crates/game/src/wardrobe.rs:103-119` and `146-237`).

### Armour picks its model by letter, not by face

Body armour chooses a model by its base item's letter, inside the body set of the player's sex and build. KOTOR 1 has these letters.

- Basic clothing and disguise items use B.
- Armour classes 4 to 9 use C to H.
- Every Jedi robe uses I.
- Revan's armour uses J.

KOTOR 2 keeps these and adds four letters. The armoured flight suit uses K. The slave outfit uses L. Zeison Sha armour uses M. The Jedi Knight and Jedi Master robes use N. The miner's uniform uses C. (All from `baseitems.2da`'s `bodyvar` column.) Both games ship all ten KOTOR 1 letters for every sex and build (`pfbas.mdl` to `pmbjl.mdl`, 60 models). The head is never involved. A custom look that keeps the games' body sets keeps every armour.

### Full-body disguises hide the head anyway

Every disguise that the games put on the player replaces the whole body and head with a model of type F (a full figure with its own head). These rows are listed below.

- KOTOR 1's Sith armour on Taris (`ptar_sitharmor`, Disguise property to row 28, `Sith_Soldier_01`).
- KOTOR 1's Mandalorian battle armour (`g_a_class9010`, Disguise property to row 35, `Alien_Mandalorian_01`).
- KOTOR 1's Sand People disguise on Tatooine (`tat17_sandperdis`, row 46, and `EffectDisguise(46)` in `k_ptat_sandper`).
- The environment suit (row 181) on Manaan (`k_pman_airlock02`, `k_pman_airlock11`) and in KOTOR 2 (`a_don_envsuit`).
- The space suit (row 279) on the Leviathan (`k_plev_airouopen`, module `lev_m40ad`) and in KOTOR 2 on Peragus and Nar Shaddaa (`a_toggle_suit`, `a_104per_enter`, `a_fadeto_304`, `a_don_spacesuit`).

KOTOR 2's own items carry no Disguise property. kjk draws a disguised player in the disguise's body (`crates/game/src/made.rs:1058-1089`). When the disguise ends, the wardrobe draws the player's own look again. A custom look changes nothing here.

### Scripts never change the player's body for good

KOTOR 1 has no `ChangeObjectAppearance`. KOTOR 2 calls it in three scripts, and none targets the player. `a_kreia_cut` (`107PER`) and `a_sion_cut2` (`153HAR`) set row 627, Kreia without her hand. `a_kreia_fight` (`904MAL`) sets row 651, Kreia's dark form. Neither game has a routine that reads or sets a portrait. Neither `nwscript.nss` declares `GetPortraitId` or `SetPortraitId`. The persona's comment that scripts read `GetPortraitId` (`crates/sim/src/persona.rs:3-4`) is wrong on that point.

### Cameras and screens read the look as drawn

- The conversation camera frames each body by its `camerahook` and head as drawn (`docs/DIALOGUE.md:1063-1067`).
- The character sheet's 3D view shows the body and what hangs from it as the scene drew them (`crates/game/src/gui_scene.rs:12-19`).
- The party selection's 3D view shows companions only, never the player.
- First person finds the eyes from the head hung on `headhook` (`crates/game/src/first_person.rs:121-165`).

### Movies cannot show the player's face

Both games ship their movies as fixed Bink files. KOTOR 1 ships 62 movie files and KOTOR 2 ships 67. No file name varies by portrait or by sex. A movie that showed the player's face would need one copy for each of the 30 or more premade faces, and none exists. In-engine cutscene modules, such as KOTOR 1's `STUNT_*` modules, draw the player's own creature. They follow the wardrobe.

## Summary of the strongest findings

1. KOTOR 1 already composes a player from a body set (by sex and build) and a head, with a bare skin texture tied to the head's skin group. A custom head on the existing body sets matches the games' own structure.
2. kjk's renderer already draws any body and head pair. Only `persona::wear` and the persona's single row tie them together.
3. Two story scenes copy the player's head with `DuplicateHeadAppearance`. They are KOTOR 1's Revan reveal (`STUNT_31b`) and KOTOR 2's Korriban cave vision (`711KOR`). kjk copies the row's premade head today, so both scenes would show the wrong face.
4. Two KOTOR 2 stand-ins copy the player's appearance row with `EffectDisguise(GetAppearanceType(player))`. They are the Exile in the Ebon Hawk's medical bay (`001EBO`) and the Exile's hologram on Malachor V (`907MAL`). They would show the premade face.
5. A custom face has no portrait. The HUD, the character sheet, the party pages and the co-op lobby need a rendered portrait, with its dark versions.
6. The player's grunts are found through the portrait row's sex in KOTOR 1. They should come from the gender.
7. The bare body's texture must follow the head's skin group, or the arms and legs will not match the face.
8. A recoloured or new head needs its own dark side textures, four in KOTOR 1 and two in KOTOR 2.
9. Scripts compare `GetAppearanceType` only with droid and turret rows, and read `GetGender` hundreds of times. A real player row and a gender that matches the body keep every script working.
10. Full-body disguises, armour letters, cameras and movies need no change.
