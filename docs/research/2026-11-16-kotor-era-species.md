# Alien species for a KOTOR-era character creator

This note was written on 2026-11-16. It asks which Star Wars species could be offered as playable choices in a kjk character creator for KOTOR 1 and KOTOR 2. Both games take place between about 3,956 BBY and 3,951 BBY. That is the Old Republic era, just after the Mandalorian Wars and during and after the Jedi Civil War.

The note looks at three things for each species. The first is what the two games already hold for it: appearance rows, head models and skeletons. The second is whether the lore places the species in the galaxy at that time, with Jedi or Sith of that species. The third is how much new art the species would need, and how the story and the companions would react to it.

Other notes in this folder cover the rest of the character creation work. `2026-11-16-custom-look-game-impact.md` covers what a custom look breaks in the games. `2026-11-16-custom-look-mod-impact.md` covers mods. `2026-11-16-character-creation-design.md` and `2026-11-16-character-creation-tech.md` cover the design and the technology.

## How this was checked

The game data comes from the Steam installs of both games, read with kjk's own tools.

- `kjk show k1 appearance.2da` and `kjk show tsl appearance.2da` gave the appearance rows. KOTOR 1's table comes from `rims/global.rim` and KOTOR 2's from `chitin.key`.
- `kjk show <game> heads.2da`, `portraits.2da`, `racialtypes.2da`, `subrace.2da` and `classes.2da` gave the other tables.
- `kjk extract` copied creature and head models into a scratch folder outside the repo. A short script read each model's header for its supermodel name and its animation scale. The script also listed the node names in each model.
- `kjk clips k1` and `kjk clips tsl` reported which animations each creature model finds through its supermodel chain.
- A short script read each game's `dialog.tlk` and counted lines that call the player "human".

The lore comes from these sources.

- Wookieepedia pages for each species and character. Fandom refuses automated page fetches, so the pages were read as raw wikitext through Wookieepedia's MediaWiki API (`starwars.fandom.com/api.php?action=parse&prop=wikitext`). Each claim below names the primary work that the Wookieepedia page cites for it. This note could not read the comics, novels or the Campaign Guide themselves. Claims from those works are cited through Wookieepedia, and the note says so each time.
- The official Star Wars: The Old Republic (SWTOR) species pages at `swtor.com/info/holonet/inhabitants/<species>`. Seven of the twelve species pages returned text. The Cathar, Cyborg, Human, Nautolan and Togruta pages returned only the site's footer.

## Terms used in this note

- **BBY** means "before the Battle of Yavin", the in-universe calendar. KOTOR 1 takes place in 3,956 BBY. KOTOR 2 takes place in 3,951 BBY.
- **Legends** is the name for the Star Wars stories told before 2014, when Lucasfilm reset its canon. KOTOR 1, KOTOR 2, the KOTOR comics and SWTOR are all Legends. Lore in this note is Legends lore unless it says otherwise.
- A **near-human** species is one whose members look almost like humans. They differ in skin colour, eyes, horns, tattoos or a few other features. Wookieepedia uses the word as a category.
- An **appearance row** is a row of `appearance.2da`. It names a body model, and for a split model it also names a head.
- A **split model** is an appearance row whose `modeltype` is `B`. It draws a body model and a separate head model from `heads.2da`. All player rows are split models.
- A **full model** is an appearance row whose `modeltype` is `F`. The head is part of the body mesh and cannot be swapped.
- A **creature model** is an appearance row whose `modeltype` is `S` or `L`. These are beasts, droids and large aliens with their own animations.
- A **supermodel** is the model that a creature model takes its skeleton and its animations from. Two models with the same supermodel share the same bones and the same clips.
- An **armour letter** is the body variant a worn item asks for, from `a` (no armour) to `j` in KOTOR 1 and to `n` in KOTOR 2. Each appearance row names one body model for each letter.
- **SWTOR** is the online game Star Wars: The Old Republic. It is set from 3,643 BBY, about 300 years after KOTOR 2 ([Wookieepedia, Star Wars: The Old Republic](https://starwars.fandom.com/wiki/Star_Wars:_The_Old_Republic), infobox `timeline`, citing the game's own dates).
- The **Campaign Guide** is the *Knights of the Old Republic Campaign Guide* for the Star Wars Roleplaying Game Saga Edition. Wizards of the Coast published it on 19 August 2008. Rodney Thompson, Sterling Hershey, Abel G. Peña and John Jackson Miller wrote it ([Wookieepedia, Knights of the Old Republic Campaign Guide](https://starwars.fandom.com/wiki/Knights_of_the_Old_Republic_Campaign_Guide)).

## The player is human in both stories

Both games treat the player as a human. A non-human player would contradict the games in four places.

The tables say so. `racialtypes.2da` in both games has only two named rows. Row 5 is `Droid` (`RACIAL_TYPE_DROID`) and row 6 is `Human` (`RACIAL_TYPE_HUMAN`). Rows 0 to 4 are empty. Neither row gives any ability bonus. A player portrait is a `portraits.2da` row with `forpc` set to 1. KOTOR 1 has 30 player portraits (`po_pfha1` to `po_pmhb5`) and KOTOR 2 has 33. Every one of them has `race` 6, which is Human, and every one shows a human face.

The lore says so. Wookieepedia gives Revan's species as Human and cites the Campaign Guide ([Revan/Legends](https://starwars.fandom.com/wiki/Revan/Legends)). It gives Meetra Surik, the Exile of KOTOR 2, as Human and cites the novel *The Old Republic: Revan* and the Campaign Guide ([Meetra Surik](https://starwars.fandom.com/wiki/Meetra_Surik)).

The dialogue says so. KOTOR 1's `dialog.tlk` has 533 lines with the word "human". In 368 of them "human" stands alone between punctuation, which is how a character addresses someone ("Keep your little secrets, human"). KOTOR 2's `dialog.tlk` has 301 lines with the word and 110 of the addressing kind. Not every one of them is spoken to the player, but many are. These are some that are.

- Juhani says "You are human... you could not possibly know." (KOTOR 1, line 4204). She also says "you have not had to flee the destruction of your homeworld either, human" (line 4180).
- Yuthura Ban on Korriban calls the player "human" in at least nine lines (lines 13515 to 13591).
- A shop owner in Taris's Upper City says "a human like you shouldn't run into any problems" (line 21945).
- A character says "So you're human. You learned the words from somewhere, though." (line 34156) and another says "You're human, but you speak like a Twi'lek..." (line 43847).
- The Red Eclipse slavers on Nar Shaddaa in KOTOR 2 address the player as "human" (lines 75845 and 80374 to 80382).

The story setting says so. Taris's Upper City restricts aliens. A citizen says "Alien species have restricted access to the Upper City" (line 22060). Mission says "They allow a few of us Twi'leks up here, but for the most part it's humans only" (line 35813). Gorton Colu of the Anti-Alien League confronts a player who walks with alien companions (lines 21873 and 21874). A non-human player in the Upper City would walk past all of this without a reaction.

The two scenes that copy the player's face also assume a human body. KOTOR 1's Revan reveal puts the player's head on Darth Revan's robe body (appearance rows 371 and 372). KOTOR 2's dark Exile vision in the Korriban cave puts the player's head on a dark robe body (rows 128 and 173). The custom-look note covers both scenes in detail. A head that fits a human body still works in both. A Wookiee or a Rodian does not have a separate head and would break both scenes.

kjk could still offer other species. It would then have to accept that some lines call the player "human". The rest of this note sorts the species by how well they fit.

## Species in the games' own data

This table lists every alien species that has a humanoid model in either game. The row numbers are `appearance.2da` rows. "Head" is the `heads.2da` row and its model, for split models only.

| Species | Game | Example rows | Model type | Head | Body and supermodel |
|---|---|---|---|---|---|
| Twi'lek | both | 47, 48, 269, 270, 365, 366, 387 (Zaerdra) | split | 61 `twilek_f`, 62 `twilek_m`, 63, 64, 65, 67, 71 | `N_TwilekF` on `S_Female03`, `N_TwilekM` on `S_Female02` |
| Twi'lek on human bodies | K1 | 446 to 450 (commoners), 376 (Sith), 380 (Jedi) | split | 61, 62, 63, 64, 65 | `N_CommF`, `N_CommM`, `N_SithComM`, `N_JediCounM` |
| Twi'lek, Mission Vao | both | 10 | split | 6 `p_MissionH` | own bare body `P_MissionBA`, player armour `PFBCS` to `PFBJS` |
| Twi'lek, Yuthura Ban | K1 | 280 | split | 93 `N_UthuraH` | human Sith body `N_SithComF` |
| Twi'lek, Lala Xu and assassins | TSL | 498, 652, 653 | split | 104, 184, 185 | `N_TwilekF`, `N_TwiAssasin` on `S_Female03` |
| Cathar, Juhani | both | 8 | split | 5 `P_JuhaniH` | own bare body `P_JuhaniBA` on `S_Female03`, player armour `PFBCM` to `PFBJM` |
| Miraluka, Visas Marr | TSL | 457 | split | 98 `P_VisasH` | player body `PFBAM`, player armour |
| Zabrak, Bao-Dur | TSL | 451 | split | 100 `P_BaoDurH` | own body `P_BaoDurBB` on `S_Female02`, own armour `BAODUR_BC` to `BAODUR_BF` |
| Echani, the Handmaiden | TSL | 454, 638 | split (638) | 171 `P_HandmaidenH` | `P_HandmaidenBA` on `S_Female03`, player armour |
| Echani, the Handmaiden's sisters | TSL | several | split | 191 `N_HandSis` | human bodies |
| Devaronian | TSL | 570 | split | 129 `N_DevH` | human commoner body `N_CommM`, player armour `PMBCM` to `PMBNM` |
| Wookiee | both | 9 (Zaalbar), 50, 51, 511 (Hanharr) | full | none | `N_WookieM` on `S_Female02` at scale 1.225 |
| Trandoshan | both | 452, 453 (K1), 464, 465 (TSL) | full | none | `N_Trandoshan` on `S_Female02` at scale 1.028 |
| Rodian | both | 39, 256 to 258 | full | none | `N_Rodian` on `S_Female02` at scale 0.947 |
| Duros | both | 24, 373, 374 | full | none | `N_Duros` on `S_Female02` at scale 1.06 |
| Bith | both | 12 | full | none | `N_Bith` on `S_Male02` at scale 0.881 |
| Aqualish ("Walrusman") | both | 49, 418 to 420 | full | none | `N_Walrusman` on `S_Male02` at scale 0.936 |
| Rakata | K1 | 36, 254, 255 | full | none | `N_Rakata` on `S_Female02` at scale 1.11 |
| Tusken Raider | both | 46, 90, 266 to 268 | full | none | `N_Tusken` on `S_Female02` |
| Quarren, Gran, Weequay, Sullustan, Gand | TSL | 537, 565, 599, 601, 569 | full | none | each on `S_Female02` (Gand at scale 0.75) |
| Vandar's species | K1 | 52 | full | none | `N_Yoda` on `S_Male02` at scale 0.278 |
| Selkath | K1 | 40, 259, 260 | creature (`L`) | none | `N_Selkath`, no supermodel |
| Toydarian | TSL | 661 | split, no head | none | `C_Toydarian`, no supermodel |
| Gamorrean, Ithorian, Jawa, Hutt | both | 68, 72, 73, 70 | creature (`S`) | none | own creature models |
| Droids | both | 2, 3 (T3-M4, HK-47), 56 to 66 | full or creature | none | own models |

Three facts from this table matter most.

First, the games already put alien heads on human bodies. KOTOR 1 rows 446 to 450 are Twi'lek commoners built from the human commoner bodies `N_CommF` and `N_CommM` with Twi'lek heads. They wear the player's own armour models (`PFBCM` to `PFBIM` for women, `PMBCM` to `PMBIM` for men). Row 380 is a male Twi'lek Jedi built from the human Jedi body `N_JediCounM`. KOTOR 2's row 570 is a Devaronian built from the human commoner body with head 129.

Second, the alien companions with split models wear the player's armour. Juhani (row 8) and Mission (row 10) wear the female player armour bodies for letters `c` to `j`. Visas Marr (row 457) wears the female player body `PFBAM` for letter `a` and player armour for most other letters. Only Bao-Dur has armour bodies of his own, because his right arm is an energy prosthesis.

Third, `appearance.2da`'s `racialtype` column does not mark species. It holds 20 on every row of both tables. The species is known only from the row's label.

## Which models share the player's skeleton

The male player body `PMBAM` uses the supermodel `S_Female02`. The female player body `PFBAM` uses `S_Female03`. The names are a quirk of the original data: `S_Female02` is the male skeleton.

These alien models ride the same male skeleton as the player, `S_Female02`. They are the Rodian, Duros, Trandoshan, Wookiee, Rakata, Tusken, male Twi'lek, Quarren, Gran, Weequay, Sullustan and Gand models, and Bao-Dur's body. These ride the female player skeleton, `S_Female03`. They are the female Twi'lek, Juhani's body, Mission's body, Visas Marr's body and the Handmaiden's body. These ride a third, smaller humanoid skeleton, `S_Male02`. They are the Bith, the Aqualish and Vandar's species. The Selkath and the Toydarian have no supermodel and use their own skeletons.

A model on the player's skeleton plays the player's animations. Its armour does not follow, though. A full model has one mesh for every armour letter. Zaalbar's row names `P_Zaalbar` for every letter, so his look never changes when he equips armour. To dress a Rodian in the player's armour, an artist would have to cut a Rodian head out of the full model. The artist would also have to make Rodian hands and feet for the armour bodies, or accept human ones.

`kjk clips` reports the animation scale of each model. The player bodies and the Twi'lek bodies play at 1.06 (398 creature uses in KOTOR 1). The Wookiee plays at 1.2 for females and 1.225 for males. The Rodian plays at 0.947 and the Bith at 0.881. A scale other than 1.06 means the body is taller or shorter than a player body. kjk's custom-look note explains that a taller or shorter body changes only what is drawn, because the sim uses Jedi Academy's player box and skeleton.

## What the alien heads carry

Every alien head checked has the same neck nodes as a player head (`head_g`, `neck_g` and `necklwr_g`). These heads were checked: `P_JuhaniH`, `p_MissionH`, `twilek_m`, `P_BaoDurH`, `N_DevH` and `P_VisasH`. None has separate bones for lekku, horns or a tail. The lekku of a Twi'lek are part of the head mesh. Juhani's body model has no tail node.

Dark-side textures differ between the two games.

- In KOTOR 1, the Twi'lek heads (61 to 71), Juhani's head (5) and Mission's head (6) have no dark-side textures. Player head 26 (`PFHA01`) has four (`PFHA01D` to `PFHA01D1`).
- In KOTOR 2, the companion heads have two dark-side textures each. Visas has `P_VisasHD01` and `P_VisasHD02`. Bao-Dur has `P_Bao_DurHD1` and `P_Bao_DurHD2`. The Handmaiden has `P_HandmaidenHD1` and `P_HandmaidenHD2`. The Twi'lek NPC heads have one alternate texture each (`twilek_f01`, `twilek_m01`) and no dark textures.

Any head offered to the player needs the dark textures of the game it is used in. That means four in KOTOR 1 and two in KOTOR 2.

KOTOR 2's `subrace.2da` adds a row 3, `Zabrak`, beside `None`, `Wookiee` and `Beast`. KOTOR 1's table has only the first three rows.

## Species attested in the Old Republic era

This section takes each candidate species in turn. It says whether the species existed and was known in the galaxy around 3,956 to 3,951 BBY. It names Jedi or Sith of that species from that time, if the sources name any.

### Human

Humans are the default. Revan, the Exile, Bastila, Carth, Atton, Kreia and most Jedi and Sith in both games are human ([Kreia](https://starwars.fandom.com/wiki/Kreia) and [Atton Rand](https://starwars.fandom.com/wiki/Atton_Rand) cite the Campaign Guide and KOTOR 2).

### Twi'lek

Twi'leks are present in large numbers in both games. Mission Vao is a Rutian Twi'lek companion in KOTOR 1 ([Mission Vao](https://starwars.fandom.com/wiki/Mission_Vao), citing KOTOR 1). Zhar Lestin is a male Lethan Twi'lek Jedi Master on the Dantooine Enclave Council during the Mandalorian Wars and the Jedi Civil War ([Zhar Lestin](https://starwars.fandom.com/wiki/Zhar_Lestin)). Yuthura Ban is a Twi'lek Sith at the Korriban academy ([Yuthura Ban](https://starwars.fandom.com/wiki/Yuthura_Ban), citing KOTOR 1). So both Jedi and Sith Twi'leks are attested in this era.

Wookieepedia says the Republic discovered Ryloth around 10,000 BBY and that Hutt cartels then enslaved many Twi'leks ([Twi'lek/Legends](https://starwars.fandom.com/wiki/Twi%27lek/Legends)). The official SWTOR page tells the same story of Ryloth's ryll mines and Hutt slavery ([swtor.com, Twi'lek](https://www.swtor.com/info/holonet/inhabitants/twilek)). Twi'leks speak Basic in both games.

### Zabrak

Bao-Dur is a Zabrak from Iridonia in KOTOR 2. Wookieepedia gives his species from *Star Wars: The Ultimate Visual Guide* and calls him a Zabrak Jedi and engineer, because the Exile can train him ([Bao-Dur](https://starwars.fandom.com/wiki/Bao-Dur)). The Campaign Guide includes the Zabrak among its species ([Wookieepedia, Campaign Guide](https://starwars.fandom.com/wiki/Knights_of_the_Old_Republic_Campaign_Guide), species list). The official SWTOR page describes vestigial horns, facial tattoos received in a rite of passage, and Iridonia as the homeworld ([swtor.com, Zabrak](https://www.swtor.com/info/holonet/inhabitants/zabrak)). Wookieepedia notes that Zabrak colonies were already established before the Republic's first contact ([Zabrak/Legends](https://starwars.fandom.com/wiki/Zabrak/Legends)).

### Cathar

Cathar are attested, but they were nearly wiped out a few years before KOTOR 1. Wookieepedia dates the Battle of Cathar to 3,973 BBY, citing *The Essential Guide to Warfare* ([Battle of Cathar](https://starwars.fandom.com/wiki/Battle_of_Cathar)). It says more than 90 percent of the species was killed by the Mandalorians in that battle, citing *The Essential Atlas* ([Cathar/Legends](https://starwars.fandom.com/wiki/Cathar/Legends)). The survivors fled off-world, and many were enslaved. The Cathar began to resettle their world after the Mandalorian Wars.

Cathar Jedi are attested both before and during this era. Crado and Sylvar were Cathar Jedi in the Great Sith War, about 4,000 BBY ([Crado](https://starwars.fandom.com/wiki/Crado), [Sylvar](https://starwars.fandom.com/wiki/Sylvar), citing the comic *Tales of the Jedi: Dark Lords of the Sith*). Juhani is a Cathar Jedi Guardian in KOTOR 1 ([Juhani](https://starwars.fandom.com/wiki/Juhani), citing KOTOR 1). She grew up on Taris after her family escaped Cathar. She belongs to the Juhani subspecies, one of the two Cathar subspecies.

A Cathar player would be one of very few survivors. That fits a player character who stands apart, but Juhani's own story then loses its weight. She tells the player that no other members of her species were on Taris (KOTOR 1, line 4802 is the player's question).

### Miraluka

Miraluka are near-humans with no eyes. They see through the Force, so every Miraluka is Force-sensitive. Wookieepedia says they have only vestigial eye sockets and perceive the world through Force sight, and that most cover their eye sockets with headbands, visors or masks ([Miraluka/Legends](https://starwars.fandom.com/wiki/Miraluka/Legends)). The official SWTOR page says the same and adds that Miraluka "wear decorative veils" ([swtor.com, Miraluka](https://www.swtor.com/info/holonet/inhabitants/miraluka)).

Darth Nihilus destroyed the Miraluka colony on Katarr in about 3,952 BBY, during a Jedi convocation ([Miraluka/Legends](https://starwars.fandom.com/wiki/Miraluka/Legends)). The SWTOR page names Visas Marr as the only survivor. Visas is a Miraluka companion in KOTOR 2 ([Visas Marr](https://starwars.fandom.com/wiki/Visas_Marr), citing the Campaign Guide). Q'Anilia is a Miraluka Jedi Master who served the Jedi Covenant during the Mandalorian Wars ([Q'Anilia](https://starwars.fandom.com/wiki/Q%27Anilia), citing the comics *Knights of the Old Republic* 6 and 9). So Miraluka Jedi and a Miraluka Sith (Visas) are both attested.

A Miraluka player raises a rule question. The player never takes off the headband, and the games' cutscenes and conversation cameras would show a covered face. The game has no "blind" state, so a Miraluka would play exactly like a human. Visas's line about seeing through the Force refers to her own species, so a Miraluka player would not contradict her.

### Echani

The Echani are a near-human culture from the Inner Rim world of Eshan. They have light skin, white hair and silver eyes ([Echani/Legends](https://starwars.fandom.com/wiki/Echani/Legends), citing KOTOR 2). The Handmaiden, Brianna, is a Human and Echani hybrid in KOTOR 2 ([Brianna](https://starwars.fandom.com/wiki/Brianna), citing KOTOR 2). She and her sisters serve Atris on Telos. Wookieepedia says the Echani fought on the Republic's side in the Mandalorian Wars and fought against Revan in the Jedi Civil War. Revan killed the Handmaiden's father, Yusanis, in a duel.

The Echani are attested and close to the story of KOTOR 2. They fit near-perfectly on a human body. A player could be an Echani with a recoloured human head. The Handmaiden's dialogue about her sisters and her mother assumes the player is an outsider, which an Echani player would still be.

### Mirialan

Mirialans are near-humans from Mirial. They have green to yellow skin and geometric facial tattoos that record their achievements ([Mirialan/Legends](https://starwars.fandom.com/wiki/Mirialan/Legends), [swtor.com, Mirialan](https://www.swtor.com/info/holonet/inhabitants/mirialan)). The official SWTOR page says "During the Jedi Civil War, Mirial sided with Darth Malak against the Jedi Order". That places Mirialans in this exact era. Wookieepedia lists the comic *Knights of the Old Republic* 6 and KOTOR 2 among the Mirialan page's appearances. This note did not find which KOTOR 2 character it refers to.

### Togruta

Togruta are attested in this era through the comics. Raana Tey is a Togruta Jedi Master of the Jedi Covenant who took part in the Padawan Massacre on Taris early in the Mandalorian Wars ([Togruta/Legends](https://starwars.fandom.com/wiki/Togruta/Legends), citing the comic arc *Knights of the Old Republic: Commencement*). Togruta have montrals, two large hollow horns, and lekku that run along the montrals. Wookieepedia says Togruta were discovered by 25,000 BBY, citing *The Essential Atlas*.

### Nautolan

Nautolans are amphibious humanoids from Glee Anselm with sixteen head-tentacles ([Nautolan/Legends](https://starwars.fandom.com/wiki/Nautolan/Legends)). Wookieepedia lists the comics *Knights of the Old Republic* 2, 18, 33 and 47 among their appearances. Most Nautolans speak Basic when they travel off-world. This note found no named Nautolan Jedi or Sith of this era.

### Chiss

Chiss are blue-skinned near-humans from Csilla in the Unknown Regions. The Chiss Ascendancy kept itself apart from the rest of the galaxy ([Chiss/Legends](https://starwars.fandom.com/wiki/Chiss/Legends)). Wookieepedia says Chiss technology existed in the galaxy by 3,951 BBY, because Chiss charric weapons are sold on Onderon in KOTOR 2. The species itself was not known to the Republic. The official SWTOR page says that, 300 years after KOTOR, "the species' very existence is unknown except in high level political" circles of the Republic ([swtor.com, Chiss](https://www.swtor.com/info/holonet/inhabitants/chiss)). Wookieepedia calls a Chiss meeting with Corellian traders in 27 BBY "the first modern contact between the Ascendancy and the Old Republic".

A Chiss player in KOTOR's era contradicts the lore. A Jedi Chiss in the Republic of 3,956 BBY would be an unexplained exception.

### Rattataki

The Rattataki are pale, hairless near-humans from Rattatak, a world of gladiator pits. The official SWTOR page says they lived underground in isolation for thousands of years before a warlord united them and they made contact with the galaxy ([swtor.com, Rattataki](https://www.swtor.com/info/holonet/inhabitants/rattataki)). The page does not date that contact. Their first notable role comes "shortly after the Empire returned", when Darth Vich recruited them. That is about 3,681 BBY or later. Wookieepedia's Rattataki page lists no KOTOR-era appearance ([Rattataki/Legends](https://starwars.fandom.com/wiki/Rattataki/Legends)).

A Rattataki player is not ruled out, but nothing places one in KOTOR's era.

### Sith Pureblood

The Sith species are red-skinned humanoids from Korriban. Sith Purebloods are Sith with a high share of Sith blood, after millennia of mixing with exiled human Dark Jedi ([Sith (species)/Legends](https://starwars.fandom.com/wiki/Sith_(species)/Legends), [swtor.com, Sith Pureblood](https://www.swtor.com/info/holonet/inhabitants/sith-pureblood)). After the Great Hyperspace War, about 5,000 BBY, the surviving Sith Empire fled into the Unknown Regions and settled on Dromund Kaas. Wookieepedia says the Sith Emperor kept building that Empire in secret "throughout the events of the Great Sith War, Jedi Civil War, and afterwards". It also says that in KOTOR's time "it was largely thought that the species had died out centuries prior", citing KOTOR 1 and KOTOR 2.

A Sith Pureblood player contradicts the story. The species is thought extinct, and the hidden Empire is a secret that neither game reveals. Revan's journey into the Unknown Regions after KOTOR 2 is where the novel *The Old Republic: Revan* begins to touch that Empire.

### Cyborg

SWTOR's Cyborg is a human with visible cybernetic implants. Cyborgs are common in this era. Bao-Dur has an energy arm ([Bao-Dur](https://starwars.fandom.com/wiki/Bao-Dur), infobox `cyber`). Wookieepedia lists KOTOR 1 and KOTOR 2 among the appearances of cyborgs ([Cyborg/Legends](https://starwars.fandom.com/wiki/Cyborg/Legends)). Both games sell cybernetic implants that the player equips. A cyborg is a human with an added piece, not a species, so it raises no lore issue.

### Devaronian

Devaronians are horned near-humans. KOTOR 2 already builds one from a human body with a Devaronian head (row 570, head 129 `N_DevH`). This note did not look for a Devaronian Jedi of this era.

### Arkanian and Arkanian Offshoot

Jarael, the deuteragonist of the KOTOR comics, is described as an Arkanian Offshoot and later as part Sephi ([Jarael](https://starwars.fandom.com/wiki/Jarael), citing *Knights of the Old Republic* 16 and 48). The Campaign Guide lists Arkanians among its species. Arkanians are near-humans with white eyes. They fit a human body with a new head.

### Species that are not candidates as near-humans

Several species are attested in this era but have very different bodies.

- **Wookiee.** Zaalbar (KOTOR 1) and Hanharr (KOTOR 2) are Wookiee companions. Wookieepedia says the construction of the Wookiee vocal apparatus prevents them from speaking Basic ([Wookiee/Legends](https://starwars.fandom.com/wiki/Wookiee/Legends), [Shyriiwook/Legends](https://starwars.fandom.com/wiki/Shyriiwook/Legends)). It also reports that George Lucas ruled out more Wookiee Jedi in the Expanded Universe, and that Obsidian Entertainment was forbidden to make Hanharr a Dark Jedi for that reason. The Wookiees of Kashyyyk were enslaved by Czerka until 3,956 BBY, when Revan helped them overthrow it.
- **Rodian, Duros, Bith, Aqualish, Trandoshan, Quarren, Gran, Weequay, Sullustan, Gand.** These species appear in the games as NPCs. They are attested in this era. Their models are full models with no separate head.
- **Rakata.** The Rakata of KOTOR 1 live on the Unknown World. Their Infinite Empire fell long before. A Rakata player would contradict the KOTOR 1 plot, which reveals the Rakata to the player as a lost people.
- **Selkath.** The Selkath of Manaan are aquatic, and their model has its own skeleton.

## Lore constraints

These are the lore rules that matter for a character creator.

- **Miraluka and Force sight.** Every Miraluka is Force-sensitive and has no eyes. A Miraluka head should always have its eyes covered. A Miraluka soldier or scoundrel would still be Force-sensitive in the lore. That fits both games, because both players become Jedi.
- **Wookiees cannot speak Basic.** Every player line in both games is written in Basic. A Wookiee player would need every line to be read as Shyriiwook with a translator, the way Zaalbar and Hanharr speak. Lucasfilm also limited Wookiee Jedi in the Expanded Universe.
- **Sith Purebloods are hidden.** The Sith Empire of Purebloods is hidden in the Unknown Regions throughout both games, and the galaxy believes the species extinct.
- **Chiss live in the Unknown Regions.** The Republic does not know the species exists, even 300 years later.
- **Mandalorians are a culture, not a species.** Wookieepedia quotes "We have no species or bloodline... We're a culture." ([Mandalorian/Legends](https://starwars.fandom.com/wiki/Mandalorian/Legends)). The Mandalorians began as the Taung species and took in people of many species. Canderous Ordo is human. Rohlan Dyre, a Mandalorian in the comics, is human ([Rohlan Dyre](https://starwars.fandom.com/wiki/Rohlan_Dyre), citing the Campaign Guide). A Mandalorian background belongs in a background or origin option, not in a species list. KOTOR 1's `Alien_Mandalorian_01` row (35) is armour, not a species.
- **Echani are a near-human culture tied to KOTOR 2's Handmaidens.** An Echani player would be a relative or countryman of Atris's handmaidens. Their lore expects white hair and silver eyes.
- **Cathar were nearly exterminated.** The Mandalorians killed more than 90 percent of the Cathar in 3,973 BBY, 17 years before KOTOR 1. Cathar survivors are rare, often former slaves, and often face prejudice, as Juhani's story on Taris shows.
- **Droids cannot be the player.** HK-47 and T3-M4 are droids. `classes.2da` gives droids two classes of their own, Combat Droid (row 6) and Expert Droid (row 7). A droid cannot take a Jedi class. The plots of both games turn on the player being a Jedi with a lost past: Revan's erased memory in KOTOR 1, and the Exile's severed bond with the Force in KOTOR 2. A droid player cannot become a Jedi, cannot wear the player's armour bodies, and has no head to copy in the two face scenes.

## Practical fit and the art each species needs

This section sorts the species by the art they would need. The baseline is a human player body from the games, with a new or reused head.

### Head only

These species need only a new head model, with its textures and its dark-side textures. They wear the human player bodies and all the player's armour.

- **Echani.** A human head with white hair, pale skin and silver eyes. A recolour of a stock player head may be enough.
- **Mirialan.** A human head with green or yellow skin and facial tattoos. The bare body's skin also needs the new colour, because the bare body's texture must match the head.
- **Cyborg.** A human head with a cybernetic piece.
- **Miraluka.** A human head with a headband over the eyes. KOTOR 2's `P_VisasH` exists, but it is a female head only and belongs to a companion.
- **Zabrak.** A human head with horns and tattoos. KOTOR 2's `P_BaoDurH` exists, but it is a male head only and belongs to a companion.
- **Twi'lek.** The games already hold seven Twi'lek heads in KOTOR 1 (61 to 71), with four female and three male. KOTOR 1 already puts them on human bodies (rows 446 to 450). These heads lack the four dark-side textures a player head needs. The bare body needs a skin texture to match the head's colour.
- **Devaronian and Arkanian.** A new head each. KOTOR 2 already puts the Devaronian head on a human body (row 570).
- **Chiss and Rattataki.** A blue or pale head with no other change. The art is small, but the lore does not place them in this era.
- **Sith Pureblood.** A red head with cheek tendrils. The art is moderate, but the lore rules it out.

Hair is part of the head mesh in all these heads, as in the stock player heads. Masks and visors hang from the head's `MaskHook` and `GoggleHook` nodes, so every new head needs those nodes.

### Head plus extra parts

These species need more than a head.

- **Cathar.** Juhani's head (`P_JuhaniH`) already sits on player armour bodies. Her bare body is her own furred model (`P_JuhaniBA`). A Cathar player needs a furred bare body texture for each build, or the hands and arms show human skin whenever armour leaves them bare. The original game accepts human skin on Juhani's hands inside armour. Juhani's model has no tail. Legends Cathar do not need one.
- **Togruta.** The head mesh would include large montrals and lekku that rest on the shoulders. Robes, hoods and armour collars would need checking for clipping. The montrals also collide with helmets and masks.
- **Nautolan.** The head mesh would include sixteen head-tentacles that hang down the back. The clipping risk is the same as for Togruta.
- **Rodian, Duros, Trandoshan and the other full-model species.** Each needs its head cut out of the full model and turned into a head model. Each also needs alien hands, and for the Trandoshan alien feet, on every armour body. The skeleton is already the player's male skeleton, so the animations would work. A female version would have to be made from scratch, because none of these species has a female model in either game.

### Does not fit

These species need a different body or break the story.

- **Wookiee.** The body is a full model at 1.225 times the animation scale, and it ignores armour. The species cannot speak Basic, and Lucasfilm restricted Wookiee Jedi.
- **Bith, Aqualish and Vandar's species.** These ride the smaller `S_Male02` skeleton, not the player's.
- **Selkath and Toydarian.** These use their own skeletons.
- **Gamorrean, Ithorian, Jawa and Hutt.** These are creature models.
- **Droids.** Droids have their own classes and cannot be Jedi.
- **Rakata.** The plot of KOTOR 1 depends on the Rakata being a lost people the player discovers.

## Voice, dialogue and companion reactions

The player does not speak voiced lines in either game. The player's sound set gives only grunts and battle cries. kjk picks it from the portrait's `sex` column, as `Player_Female_W` or `Player_Male_W` in KOTOR 1's `soundset.2da`. These grunts are human. They suit every near-human species. A Wookiee or a Trandoshan would need new grunts.

The written player lines are in Basic. All near-human species in the first tier speak Basic in the lore, so the player's lines need no change for them.

Many NPC lines call the player "human" (see the first section). A non-human player would hear them unchanged. kjk has three options for this.

1. Accept the lines as they are. Some players already accept this in mods that add alien heads.
2. Replace the word "human" in addressing lines with a species word, at the moment the line is shown. This needs a list of the lines, because "human" in other lines refers to other people.
3. Offer only species whose lore and looks pass as human in conversation. These are the Echani, the Cyborg and possibly the Miraluka with a headband.

The companions react to the player's species in a few known places.

- Juhani's conversation in KOTOR 1 compares her life as a Cathar with the player's life as a human (lines 4180 and 4204). Her story about being the only Cathar on Taris would conflict with a Cathar player.
- Mission's lines about Twi'leks in the Upper City assume she is the only Twi'lek in the party.
- The Taris Upper City restricts aliens. A non-human player would walk there without any reaction from the Sith patrols or the Anti-Alien League.
- Visas Marr's lines about Katarr would conflict with a Miraluka player only if the player claimed to come from Katarr. A Miraluka from Alpheridies does not conflict.
- The Handmaiden's lines about Echani duels assume the player is not Echani. An Echani player would still be an outsider to her sisters.

## Tiers

### Tier 1, fits with a new head only

These species are attested in KOTOR's era, can wear the human player bodies and all armour, and speak Basic.

| Species | Era evidence | Art needed |
|---|---|---|
| Twi'lek | Mission, Zhar Lestin (Jedi), Yuthura Ban (Sith) in KOTOR 1 | Reuse KOTOR 1 heads 61 to 71, add dark textures and bare-body skin |
| Zabrak | Bao-Dur in KOTOR 2, the Campaign Guide | New head; Bao-Dur's head is a model to follow |
| Miraluka | Visas Marr (Sith) in KOTOR 2, Q'Anilia (Jedi) in the comics | New head with a covered face; Visas's head is a model to follow |
| Echani | The Handmaiden and her sisters in KOTOR 2 | Recoloured human head |
| Mirialan | Mirial sided with Malak (SWTOR); the comics and KOTOR 2 | New head and skin colour |
| Cyborg | Implants in both games, Bao-Dur's arm | Human head with an added piece |
| Devaronian | KOTOR 2 row 570 | Reuse KOTOR 2 head 129, add dark textures |
| Arkanian | Jarael in the comics, the Campaign Guide | New head |

### Tier 2, needs extra parts or a lore exception

| Species | Why it is tier 2 |
|---|---|
| Cathar | Attested (Juhani, Crado, Sylvar), but needs a furred bare body and conflicts with Juhani's story |
| Togruta | Attested (Raana Tey in the comics), but the montrals and lekku need clipping work |
| Nautolan | Appears in the comics, but the head-tentacles need clipping work |
| Rodian, Duros, Trandoshan and other full-model aliens | Attested in the games, but each needs a head cut from a full model and alien hands; no female models exist |
| Chiss | Head-only art, but the Republic does not know the species exists |
| Rattataki | Head-only art, but nothing places the species in this era |

### Tier 3, does not fit

| Species | Why it does not fit |
|---|---|
| Sith Pureblood | Thought extinct in this era; the Sith Empire is hidden in the Unknown Regions |
| Wookiee | Cannot speak Basic, full model at a larger scale, no armour, Lucasfilm limit on Wookiee Jedi |
| Bith, Aqualish, Vandar's species | Smaller skeleton |
| Selkath, Toydarian | Own skeletons |
| Gamorrean, Ithorian, Jawa, Hutt | Creature models |
| Rakata | The KOTOR 1 plot reveals them as a lost people |
| Droids | Droid classes only, cannot be Jedi, no head |
| Mandalorian | A culture, not a species; belongs in a background option |

## Summary of the strongest findings

1. Both games treat the player as human. `racialtypes.2da` has only Droid and Human rows. Every player portrait (30 in KOTOR 1, 33 in KOTOR 2) has `race` 6 (Human). Revan and the Exile are human in Legends.
2. Many NPC lines call the player "human". KOTOR 1 has 368 lines that address someone as "human" and KOTOR 2 has 110. Juhani, Yuthura Ban and the Taris Upper City all assume a human player.
3. The games already put alien heads on human bodies. KOTOR 1 rows 446 to 450 and 380 are Twi'leks on human bodies. KOTOR 2 row 570 is a Devaronian on a human body.
4. The split-model alien companions wear the player's armour. Juhani, Mission and Visas use the female player armour bodies. Only Bao-Dur has armour of his own.
5. The full-model aliens share the player's male skeleton (`S_Female02`). They have no separate head, so each would need a head cut from its model.
6. Twi'lek, Zabrak, Miraluka, Echani, Mirialan, Cyborg, Devaronian and Arkanian fit with a new head only, and all are attested in or near this era.
7. Cathar, Togruta and Nautolan are attested but need extra art. Cathar also conflicts with Juhani's story as the rare survivor.
8. Chiss, Rattataki and Sith Purebloods are SWTOR species that do not belong to KOTOR's era. The Chiss are unknown to the Republic, nothing places the Rattataki here, and the Sith Purebloods are thought extinct.
9. Wookiees, droids and Mandalorians do not fit. Wookiees cannot speak Basic, droids cannot be Jedi, and Mandalorians are a culture, not a species.
10. Every alien head the player could use lacks the player's dark-side textures in KOTOR 1. A playable head needs four dark textures in KOTOR 1 and two in KOTOR 2.

## Gaps

- This note did not read the Campaign Guide, the KOTOR comics or the novels. It cites them through Wookieepedia.
- The official SWTOR pages for the Cathar, Cyborg, Human, Nautolan and Togruta returned no text.
- The counts of "human" lines include lines spoken to other characters. A full list of lines spoken to the player needs each conversation's speaker and listener.
- This note did not look for Jedi or Sith of the Rodian, Duros, Devaronian, Nautolan or Arkanian species in this era.
- This note did not test any alien head on a player body in kjk.
