# Custom aliens and non-human species in character creators

This document surveys how games let a player make a non-human character. It covers Star Wars games with alien species, games where the player builds a creature from nothing, and fantasy or science fiction games with horned, tailed, eared or furred races. It then lists the techniques these games use, and what those techniques cost. The last section turns the survey into advice for kjk.

The general technology of character creators (morph targets, bone sliders, modular meshes, materials, saved data) is covered in `docs/research/2026-11-16-character-creation-tech.md`. This document does not repeat it.

Each claim carries a link to its source. Where the only source is a press article, a fan wiki or a forum, the text says so. Some facts were read directly from game files or from released source code. Those facts name the file they come from.

## Terms used in this document

A **species** is a playable people with its own look, such as a Twi'lek or a Wookiee. Fantasy games call it a **race**. This document uses both words as each game does.

An **attachment** is a separate mesh fixed to one bone of the character. A horn, a tail, a pair of ears or a helmet can be an attachment. The attachment moves with that bone.

**Lekku** are the two head-tails of a Twi'lek. **Montrals** are the two hollow horns on a Togruta's head.

A **plantigrade** leg stands on the whole sole of the foot, like a human leg. A **digitigrade** leg stands on the toes, with the heel raised high, like a cat's or a dog's hind leg. A digitigrade leg has one more bend than a human leg.

**Retargeting** is playing an animation made for one skeleton on another skeleton with different proportions or different bones.

**Headgear** is anything worn in the head slot, from a pair of goggles to a full helmet.

**Clipping** is when one mesh passes visibly through another, such as a horn poking through a hat.

A **body type** is one of a fixed set of body shapes that every outfit is made for. The Old Republic, for example, calls them Body Type 1 to Body Type 4.

A **danglymesh** is KOTOR's name for a mesh whose vertices sway on springs, such as hair, hoods and capes. kjk already draws them (`crates/game/src/dangly.rs:1`).

## Star Wars games

### Star Wars Galaxies

Star Wars Galaxies (SWG, Sony Online Entertainment, 2003) had ten playable species by 2004. The original game had eight: Human, Twi'lek, Zabrak, Wookiee, Rodian, Trandoshan, Mon Calamari and Bothan. The Jump to Lightspeed expansion added Ithorian and Sullustan ([MassivelyOP, Hyperspace Beacon](https://massivelyop.com/2019/03/27/hyperspace-beacon-this-is-the-toughest-choice-in-star-wars-galaxies-legends/), press).

The facts below come from the game's own data files, as published in the SWG-Source project on GitHub. SWG-Source is a community release of the leaked SWG server and client source. It is not an official Sony or Daybreak publication, but the files are the game's real data.

**Every species shares one animation set.** Each player species has a template file that names its animation map. All ten male templates, and the female Twi'lek template, name the same file, `all_male.map` ([shared_twilek_male.tpf](https://github.com/SWG-Source/dsrc/blob/master/sku.0/sys.shared/compiled/game/object/creature/player/shared_twilek_male.tpf), [shared_wookiee_male.tpf](https://github.com/SWG-Source/dsrc/blob/master/sku.0/sys.shared/compiled/game/object/creature/player/shared_wookiee_male.tpf), [shared_ithorian_male.tpf](https://github.com/SWG-Source/dsrc/blob/master/sku.0/sys.shared/compiled/game/object/creature/player/shared_ithorian_male.tpf)). This means a Wookiee, an Ithorian and a human all play the same walk, run and combat animations.

**Each species has its own mesh and its own height range.** Each template names a different skeletal appearance file, such as `appearance/twk_m.sat` for the male Twi'lek and `appearance/wke_m.sat` for the male Wookiee. Each template also gives a scale range. The male ranges are:

| Species | Scale range |
|---|---|
| Bothan | 0.75 to 0.83 |
| Rodian | 0.81 to 0.94 |
| Mon Calamari | 0.89 to 1.00 |
| Human | 0.89 to 1.11 |
| Twi'lek | 0.92 to 1.11 |
| Zabrak, Ithorian, Sullustan | 0.92 to 1.06 |
| Trandoshan | 1.03 to 1.25 |
| Wookiee | 1.11 to 1.28 |

The values are read from each species' player template in [SWG-Source/dsrc](https://github.com/SWG-Source/dsrc/tree/master/sku.0/sys.shared/compiled/game/object/creature/player).

**Each slider is a pair of blend shapes.** The table `customization_data.tab` lists every slider for every species and sex ([customization_data.tab](https://github.com/SWG-Source/dsrc/blob/master/sku.0/sys.shared/compiled/game/datatables/customization/customization_data.tab)). A slider such as "nose length" drives two blend shapes, `blend_noselength_0` and `blend_noselength_1`. One shape moves the nose one way and the other moves it the other way. A template stores each value as a whole number from 0 to 255 ([shared_hoth_collector.tpf](https://github.com/SWG-Source/dsrc/blob/master/sku.0/sys.shared/compiled/game/object/mobile/shared_hoth_collector.tpf)).

**The species reuse the same slider names for different features.** The male Ithorian's "trunk height" slider drives `blend_noselength`. Its "hump size" slider drives `blend_jaw`, and its "head shape" slider drives `blend_cheeks`. The male Mon Calamari's "head" slider drives `blend_ears`, and his "gullet" slider drives `blend_chinsize`. The Rodian has a slider of its own, "sensors", on `blend_sensor`. The engine therefore needs only one list of blend names. Each species' artist sculpts whatever that species needs into each name.

**The number of sliders differs by species.** Counting the male rows of the same table gives these numbers:

| Species | Shape sliders | Colour choices |
|---|---|---|
| Human | 20 | 5 |
| Sullustan | 20 | 7 |
| Twi'lek | 19 | 7 |
| Zabrak | 19 | 7 |
| Ithorian | 18 | 7 |
| Bothan | 15 | 5 |
| Mon Calamari | 15 | 7 |
| Rodian | 15 | 6 |
| Wookiee | 14 | 7 |
| Trandoshan | 11 | 4 |

**Markings are a pattern index and a colour.** The Twi'lek lekku group has a "pattern" slider on `index_texture_1`, a skin colour and a markings colour. The Wookiee's "fur" group has the same three controls. The Zabrak has a "tattoo style" index and a tattoo colour.

**Lekku and horns are hair.** SWG keeps lekku and Zabrak horns as hair objects, chosen like a hairstyle. The Twi'lek hair base template is named "lekku" ([shared_hair_twilek_base.tpf](https://github.com/SWG-Source/dsrc/blob/master/sku.0/sys.shared/compiled/game/object/tangible/hair/twilek/base/shared_hair_twilek_base.tpf)). There are 8 Twi'lek styles per sex, and each is its own skeletal appearance, such as `twk_f_hair_s01.sat` ([hair/twilek folder](https://github.com/SWG-Source/dsrc/tree/master/sku.0/sys.shared/compiled/game/object/tangible/hair/twilek)). The Zabrak's colour group for the same slot is called `horns_hair`, with one colour for the horns and one for the hair.

**Some species could not wear most clothes.** When SOE added the Ithorian, the team wrote that "our current set of player clothing was simply not going to fit the Ithorians too well", "particularly because of the enormous size difference in the neck". The team decided "to make an amendment to our wearable system" rather than remake every item ([GameBanshee quoting an IGN developer feature, 2004-09-16](https://www.gamebanshee.com/news/73782-star-wars-galaxies-jump-to-lightspeed-feature.html)). Wookiees had their own armour sets, such as Kashyyykian Black Mountain and Kashyyykian Hunting armour, which only Wookiees could wear ([SWGEmu archive of the official site](https://www.swgemu.com/archive/scrapbookv51/data/20070213112516)). The SWG client also ships Wookiee-only clothing objects, such as `object/tangible/wearables/wookiee/shared_wke_lifeday_robe_m.iff` ([client-tools patch list](https://github.com/SWG-Source/client-tools/blob/master/publish/patch_05.txt)). Players on a fan server still complain that the best battle armour does not fit Wookiees or Ithorians ([SWG Restoration forum](https://swgr.org/post/bh-and-ris-armor-wearable-by-wookie-ithorian-races.1507), forum).

### Star Wars The Old Republic

Star Wars: The Old Republic (SWTOR, BioWare, 2011) builds each character from a species, a body type and a head. Its patch notes name body types per species. Patch 2.7.1 says "The ears on Male Twi'lek Body Types 2, 3, and 4 are now visibly humanoid". The same patch fixes gear that did not "render the Cathar female Body Type 2 correctly" ([SWTOR 2.7.1 patch notes](https://www.swtor.com/patchnotes/4292014/2.7.1-patch-notes)). The species therefore share the numbered body types, and armour is made per body type, not per species.

Lekku clip through headgear, and BioWare fixes the worst cases one item at a time. Game Update 7.3.1a says "The Mandalorian Commander Helmet is no longer clipping with the Twi'lek's lekku" ([SWTOR 7.3.1a patch notes](https://www.swtor.com/patchnotes/91323/game-update-7.3.1a)). Players on the official forum describe their choices as no helmet, no lekku, or lekku through the hat ([SWTOR forum](https://forums.swtor.com/topic/528577-hats-for-twileks/), forum).

The Togruta arrived in Game Update 3.3 on 21 July 2015 ([SWTOR developer blog](https://www.swtor.com/info/news/blog/20150722)). Togruta never show headgear. Players on the official forum quote community manager Eric Musco saying that "due to their Montrals, any item worn in your head slot will be automatically hidden" ([SWTOR forum](https://forums.swtor.com/topic/689864-togruta-officially-confirmed-as-next-playable-species-more-or-less/), forum). No official page with that quote was found. Players say that Togruta and Twi'lek also hide hoods automatically ([SWTOR forum](https://forums.swtor.com/topic/909635-headpiece-that-disables-hoods), forum).

New species were sold, not given. The Cathar cost 600 Cartel Coins per account ([Engadget, 2013-04-26](https://www.engadget.com/2013-04-26-the-cathar-arrive-in-swtors-next-update.html), press). Producer Bruce Maclean called the Togruta "a Cartel Market feature", not an expansion feature ([Engadget interview, 2014-10-24](https://www.engadget.com/2014-10-24-bioware-on-swtors-revan-expansion-upcoming-species-and-class.html), press).

### Jedi Academy

Jedi Academy (Raven Software, 2003) is the source of kjk's combat, so its own species system matters most here. The facts below were read from `assets1.pk3` of a Steam install of the game, and from the OpenJK source code.

Jedi Academy has six playable species folders. They are `jedi_hf` (human female), `jedi_hm` (human male), `jedi_kdm` (Kel Dor male), `jedi_rm` (Rodian male), `jedi_tf` (Twi'lek female) and `jedi_zf` (Zabrak female). Each folder holds a `playerchoice.txt`. The game's menu builds the species list by looking for that file in every folder under `models/players` ([OpenJK ui_main.cpp](https://github.com/JACoders/OpenJK/blob/master/code/ui/ui_main.cpp)).

**All six species use one skeleton.** Each species' `model.glm` names `models/players/_humanoid/_humanoid` as its skeleton. That skeleton has 53 bones. They include face bones (`ceyebrow`, `jaw`, four lip bones, two eye bones) and two bones named `ltail` and `rtail`.

**A species is one mesh with many parts that can be switched off.** The Twi'lek model has 207 surfaces, the Zabrak 209, the Rodian 202 and the Kel Dor 198. A "skin" file turns surfaces on or off and gives each a texture. For example, `head_a1.skin` of the Twi'lek turns on `heada`, `heada_face`, `heada_teeth` and `heada_tentacles`, and turns off every `headb` surface. The menu lists every skin file whose name starts with `head_`, `torso_` or `lower_` as a choice for that body part ([OpenJK ui_main.cpp](https://github.com/JACoders/OpenJK/blob/master/code/ui/ui_main.cpp)).

**The Twi'lek's lekku do not use the tail bones.** The lekku surface `heada_tentacles` has 76 vertices and 110 triangles. It is skinned to five spine and head bones (`lower_lumbar`, `upper_lumbar`, `thoracic`, `cervical`, `cranium`). The lekku therefore follow the back as it bends, but they do not swing on their own.

**The Kel Dor's mask is a separate surface on one bone.** The surface `heada_mask` has 237 vertices and is skinned only to `cranium`. The Rodian's head uses the jaw and lip bones but no eye bones.

**Skin colour is a tint chosen from a short list.** The Twi'lek's `playerchoice.txt` lists colour choices such as "twilek_green1" (red 255, green 235, blue 100) and "twilek_blue1" (red 62, green 155, blue 255). Each choice sets three console variables, one per colour channel.

### KOTOR itself

KOTOR's alien heads are separate models, like its human heads. kjk already has a test that hangs KOTOR 2's Twi'lek woman head (`heads.2da` row 61, texture `twilek_f01`) on a player body (`crates/game/src/alignment_look/probes.rs:23`). The KOTOR 1 Community Patch adds a male Twi'lek row to `appearance.2da`, as the mod impact note records (`docs/research/2026-11-16-custom-look-mod-impact.md:122`).

## Games where the player builds the creature

### Spore

Spore (Maxis, 2008) lets the player build a creature with any number of legs, arms, mouths and eyes. Its technique has three parts: a skin that is rebuilt as the player edits, parts that the player stretches with handles, and animation that works on bodies nobody has seen.

**The skin is an implicit surface.** Chris Hecker, who wrote the first version, says Spore "needs to regenerate the skin in real-time as the player deforms the torso and attaches and detaches limbs". He "chose a blobby implicit surface (sometimes called metaballs)" because such surfaces are "very topologically robust", even though they "lack local control" ([Hecker, My Liner Notes for Spore, archived](http://web.archive.org/web/20261006092736/http://www.chrishecker.com/My_Liner_Notes_for_Spore)). The surface uses only spherical metaballs, spaced along the limbs and torso. Each vertex's bone weights come from the body part whose metaballs made it. Hecker writes that this works for limbs, but "the torso on fat creatures can shear". The skin is "one big implicit surface", so limbs that move close together grow webbing between them. Players used that bug to make bat wings, and Maxis kept it.

**Textures are laid out in 10 milliseconds.** A normal game lays out a mesh's texture once, by hand. Spore must do it every time the player changes the creature. Hecker wrote "a very simple but optimized charter" that "does the job in 10 milliseconds on our minspec platform" ([Hecker, liner notes](http://web.archive.org/web/20261006092736/http://www.chrishecker.com/My_Liner_Notes_for_Spore)).

**Parts are rigged blocks with handles.** The parts are "Rigblocks", which are Maya models whose shape the player changes with handles. A Maxis presentation by Lydia Choy, Ryan Ingram, Ocean Quigley, Brian Sharp and Andrew Willmott says "Animations driven by player-controlled handles". The base block is the body mesh, generated from metaballs, and the rigblocks attach to it ([Rigblocks presentation, mirror](https://sambuz.com/doc/rigblocks-player-deformable-objects-ppt-presentation-723005)). Before play, the game bakes each creature into one mesh with "a single texture page, a single material, and generated LODs" (same source).

**Paint is procedural, with masks painted by artists.** The player picks three colours. Each part's texture carries 8 to 9 extra channels. Masks in those channels say how much of the procedural paint shows through the hand-painted texture ([Jane Ng, Spore Rigblocks and SkinPaint](https://janeng.com/?p=143), a Spore part artist's own blog). Maxis described the system in a SIGGRAPH 2007 talk, "Player-driven procedural texturing" ([SIGGRAPH history archive](https://history.siggraph.org/learning/player-driven-procedural-texturing-by-debry-goffin-hecker-quigley-shodhan-et-al/)).

**Animators tag what matters, and the game solves the rest.** Hecker's GDC 2007 lecture, "How To Animate a Character You've Never Seen Before", states the problem: "my creature might have two arms and one leg, yours might have no arms and seven legs, and two mouths" ([Hecker's page for the talk, archived](http://web.archive.org/web/20261006214734/https://www.chrishecker.com/How_To_Animate_a_Character_You%27ve_Never_Seen_Before)). The SIGGRAPH 2008 paper by Hecker, Raabe, Enslow, DeWeese, Maynard and van Prooijen describes the answer ([ACM DOI 10.1145/1360612.1360626](https://doi.org/10.1145/1360612.1360626); text read from a [course copy of the paper](https://www.cse.chalmers.se/edu/year/2011/course/TDA361/Advanced%20Computer%20Graphics/027-hecker%20copy.pdf)):

- A creature is made of 20 to 80 "bodies". A body is what other engines call a bone. Each body carries tags called capabilities, such as "grasper" for a hand.
- In the animation tool, Spasm, an animator keys a motion on a sample creature. For each channel, the animator says which parts to select (for example, "the front graspers") and which aspects of the motion matter (for example, "relative to the rest pose" or "relative to a target").
- The tool stores the motion in a form that does not depend on any one body. At runtime, the game turns it back into goals for the actual creature, and an inverse kinematics solver poses the creature to meet the goals.
- Legs get their own gait system. Legs are grouped by length, and the groups' step rates follow simple ratios of their lengths. Animators wrote gait settings for 1 to 6 feet, and the game generates settings for 7 or more feet. A creature with no feet crawls like an inchworm on parts of its spine.
- Parts that the animation does not select sway on a damped spring system called "Jiggles". The sway never feeds back into the animated parts.
- A creature with 25 bodies took about 0.2 ms per frame on a 1.7 GHz Pentium M. About 35% of that time was the inverse kinematics solver.
- Testers ran "several hundred creatures and a thousand animations". About 90% passed.

The paper lists its own limits. The system cannot express "hug each other" or "rub your chin", because it has no idea of the body's volume. It ignores collisions between a creature's own limbs. Animators "create complete animations on a single character within a few hours", but "it takes weeks to build up an intuition about which kinds of motions generalize" ([Hecker et al. 2008](https://doi.org/10.1145/1360612.1360626)).

Spore saves a creature inside a PNG picture of it. The data hides in the low bits of the pixels, not in a PNG text chunk ([Ned Batchelder, 2008](https://nedbatchelder.com/blog/200806/spore_creature_creator_and_steganography), blog). A community format document says the data starts with the word "spore" and a version number, followed by the creature as XML ([Spore Community, Pollen Metadata](https://spore-community.github.io/docs/pollination/pollen_metadata), community).

### No Man's Sky

No Man's Sky (Hello Games, 2016) generates its animals rather than letting the player build them. The best description of the method comes from outside the studio. A blogger's analysis, summarised by Kotaku, found that many creatures start "from the same base model", which holds every possible body part. Code picks parts from it by name ([Kotaku, 2016-10-18](https://kotaku.com/a-look-at-how-no-mans-skys-procedural-generation-works-1787928446), press). Art director Grant Duncan's GDC 2015 talk on procedural art is free on the GDC channel ([No Man's Sky blog](https://www.nomanssky.com/2015/02/no-mans-sky-at-gdc/)).

For the player's own character, No Man's Sky offers a fixed list of species (Gek, Korvax, Vy'keen, Anomaly and others) at an "Appearance Modifier" terminal. The player then picks heads, helmets and other parts per species ([PC Gamer](https://www.pcgamer.com/see-the-weird-heads-helmets-and-beaks-of-no-mans-sky-nexts-player-customization), press). No developer source was found on whether the species share a skeleton.

### Stellaris and portraits

Stellaris (Paradox, 2016) shows each species as an animated portrait. The art team's diary says the portraits look 2D in game but are "technically" made "in a 3D software", with each character "split into layers" before rendering ([Stellaris Dev Diary 188](https://forum.paradoxplaza.com/forum/developer-diary/stellaris-dev-diary-188-necroid-characters-the-art-process.1436338), official forum). A portrait is a fixed piece of art per species, so its cost does not multiply with clothing. Dev Diary 342 says that reactive portraits, which change with the empire's state, needed "programmers, animators and multiple artists" because of "the randomly generated nature of stellaris species" ([Stellaris Dev Diary 342](https://admin-forum.paradoxplaza.com/forum/developer-diary/stellaris-dev-diary-342-the-art-of-the-machine-age-part-i.1666149), official forum).

## Races with horns, ears, tails and snouts

### Final Fantasy XIV

Final Fantasy XIV added the Viera (rabbit-eared people) and the Hrothgar (lion-like people) in the Shadowbringers expansion in 2019. Most headgear did not show on them. Square Enix's own forum answer said that "the unique design of Viera and Hrothgar just prevents us from allowing these pieces, if we want to follow our specifications" ([search summary of the official forum thread](https://forum.square-enix.com/ffxiv/threads/423457-New-Races-Unable-to-Wear-Puppet-Bunker-Raid-Headgear?p=5405362), official forum).

A fan wiki counts which hats display on each race. On its last edit, 2,609 of 2,998 hats displayed on Viera, and 2,593 of 2,998 displayed on Hrothgar ([FFXIV wiki, Hrothgar and Viera Hat Compatibility](https://ffxiv.consolegameswiki.com/wiki/Hrothgar_%26_Viera_Hat_Compatibility), fan wiki). Patch 7.3 was the big step. Its notes say that "a large number of headgear items will now display when worn by Viera and Hrothgar" ([Patch 7.3 notes](https://na.finalfantasyxiv.com/lodestone/topics/detail/c04405c6cbe8519a0b6c8aa5e4d88a5d447419c9/)).

The same patch made the ears a per-item choice. A Viera player can switch "Show ears through headgear (select gear only)" in the Character window, next to the visor setting ([official UI guide](https://na.finalfantasyxiv.com/uiguide/equipment/equipment-my/character_vears.html)). A text command, `/vieraears`, does the same ([Eorzea Database](https://na.finalfantasyxiv.com/lodestone/playguide/db/text_command/190dc7f8137/?patch=latest)). Press reports say the setting is saved in gear sets, and that helmets will follow later ([Siliconera, 2025-06-20](https://www.siliconera.com/ffxiv-hrothgar-and-viera-headgear-support-coming-for-more-hats/), press).

The male Viera and female Hrothgar came years after the first sexes. Press reports say artists built the male Viera in free time between normal patch work ([Siliconera](https://www.siliconera.com/ffxiv-male-viera-made-in-free-time-between-normal-work-on-patches/), press). No primary source was found for the often-quoted figure of ten thousand items that must be refitted.

### World of Warcraft

World of Warcraft makes each helmet as a separate model for each race and sex. Modders describe a folder of helmet models per race and sex, and say a helmet with no model for a race shows as a white box ([modding forum](https://elijahketchersid.com/forums/thread-86034.html), forum). Blizzard has not published a count.

Blizzard's artists now use machine learning to refit helmets to all races. Lead world artist Kristy Moret said at BlizzCon 2023 that the team has "embarked upon a big project that uses machine learning to refit helmets to all character races". She called the manual work very tedious for a character artist ([Blizzard Watch, 2023-11-06](https://blizzardwatch.com/2023/11/06/hair-under-helmets-ai/), press). Franchise director John Hight said in 2024 that the team uses machine learning to fit armour onto character models instead of doing it by hand for each one ([PC Gamer](https://pcgamer.com/games/world-of-warcraft/were-not-using-generative-ai-within-wow-says-franchise-director-john-hight-though-the-teams-been-using-machine-learning-to-cut-out-busywork), press).

Most "allied races" reuse an existing race's skeleton, so existing armour and animation fit them. A fan wiki says they "mostly use the skeleton of an existing race but altered and with a different model", and that Kul Tirans are the only allied race with "a completely unique animation skeleton" ([Warcraft Wiki, Allied race](https://warcraft.wiki.gg/wiki/Allied_race), fan wiki). Art director Chris Robinson put the animation cost of the 2014 model update at "roughly 180 animations per character, times 10 races in both male and female variants, equaling approximately 3,600 animations" ([Wowhead, quoting Robinson's ArtCraft post](https://www.wowhead.com/news/warlords-of-draenor-female-human-model-226814), press).

The Dracthyr (a dragon race added in 2022) wear no normal armour in dragon form. They get a humanoid "visage" form for regular armour. A fan source says the dragon form's armour is chosen only at creation and at the barber ([MeinMMO](https://mein-mmo.de/en/wow-players-are-bothered-by-the-appearance-of-the-new-evokers-i-was-100-sure-i-would-play-dracthyr-but,911985), press). No developer statement on the reason was found.

### Guild Wars 2

Guild Wars 2 (ArenaNet, 2012) has five races, including the Charr, who are tall cat-like people with tails. Lead designer Eric Flannum said that in the first Guild Wars, each profession had its own armour and animations. With several races, "multiplying the number of professions with the number of races in the game made it next to impossible". The team "went to a much simpler armor system where professions use light, medium, or heavy armor" ([GameBanshee quoting an MMORPG.com interview, 2009-12-09](https://www.gamebanshee.com/99r), press).

### The Elder Scrolls

In Morrowind (2002), Argonians and Khajiit cannot wear boots, shoes or closed helmets ([UESP, Morrowind Argonian](https://en.uesp.net/wiki/Morrowind:Argonian), fan wiki). In Oblivion, Skyrim and The Elder Scrolls Online, they can. In those games their legs are human legs. UESP's page on The Elder Scrolls Online's races explains that the beast race player models "reuse the humanoid models that the other races use, and thus lack digitigrade legs" ([UESP, Online Races](https://en.uesp.net/wiki/Online:Races), fan wiki). No Bethesda statement on the change was found. Players and modders believe the reason is armour, because boots made for human feet would not fit a digitigrade foot ([Elder Scrolls Online forum](https://forums.elderscrollsonline.com/en/discussion/70027/one-model-skeleton-for-every-race), forum).

In Skyrim, helmets still need separate meshes for the beast heads. Mods that add beast versions of helmets switch the mesh by the wearer's race ([Nexus Mods](https://www.nexusmods.com/skyrim/mods/15927), mod page).

### Dragon Age

The Qunari of Dragon Age have horns. They had none in Dragon Age: Origins (2009). Creative director Mike Laidlaw said the blocker was "technical issues around horns and having a horned character in the player's party", because "you need to worry about helmet slots" ([GamesRadar interview before Dragon Age II's launch](https://gamesradar.com/our-final-dragon-age-2-interview-before-launch), press).

Dragon Age: Inquisition (2014) made the Qunari playable with horns. A Qunari wears "vitaar", a kind of war paint, in the helmet slot, instead of a helmet ([Dragon Age Wiki, Vitaar](https://dragonage.fandom.com/wiki/Vitaar), fan wiki). The horn choice in the creator replaces the hairstyle choice ([PC Gamer](https://pcgamer.com/dragon-age-inquisition-will-let-you-hug-your-companions-customise-your-horns), press). In Dragon Age: The Veilguard (2024), players report horns poking through helmets ([EA forum](https://forums.ea.com/discussions/dragon-age-the-veilguard-discussion-en/dragon-age-the-veilguard-qunari-helmet/7386027), forum).

### Baldur's Gate 3

Baldur's Gate 3 (Larian, 2023) has horned tieflings and tailed Dragonborn. Larian chose presets rather than sliders for height, body and face. Lead character artist Alena Dubrovina said that "small millimeter changes can make or break a face completely" ([Game Developer](https://gamedeveloper.com/art/building-an-inclusive-character-creator-for-the-fantasy-world-of-baldur-s-gate-3), press). Press reports say Dragonborn helmets do not always fit their head shape ([Dexerto](https://dexerto.com/baldurs-gate/baldurs-gate-3-never-quite-got-dragonborn-helmets-right-2881523), press). No Larian statement on horns or tails under armour was found.

### Mass Effect

No Mass Effect game lets the player be an alien in the main story. BioWare's Dusty Everman said "all party members needed to use a humanoid skeleton", and "if we'd ever tried to use a non-humanoid, the cost would have been huge". Casey Hudson said alien squad members "had to function like the human characters" and "be able to carry a gun" ([Shacknews](https://www.shacknews.com/article/79126/the-huge-cost-of-non-humanoid-aliens-in-mass-effect), press). For Mass Effect: Andromeda, general manager Aaryn Flynn gave a story reason. "The story is very much a human one, and we wanted to focus on that so it's relatable" ([Aaryn Flynn on Twitter, 2015-11-04](https://twitter.com/AarynFlynn/status/661705574426517504), quoted by [Gameranx](https://www.gameranx.com/updates/id/31493/article/why-mass-effect-andromeda-s-protagonist-is-human)).

Mass Effect 3's multiplayer did let players be turians, krogan, asari and other aliens. Each kit has fixed armour, so there is no armour-per-species problem. Press reports say BioWare left out the Elcor and Hanar because they would need a lot of expensive animation ([Softpedia](https://games.softpedia.com/blog/No-Elcor-or-Hanar-for-Mass-Effect-3-s-Multiplayer-309193.shtml), press).

Andromeda's developers planned "five or six" new species. Former developers told TheGamer that most were cut for budget and because some designs could not be built by cosplayers ([Game Informer, 2021-02-05](https://www.gameinformer.com/2021/02/05/mass-effect-andromeda-was-supposed-to-have-a-lot-more-new-aliens), press).

### Other games

- **Warframe** has one attachment rule for every body. Its official guidelines say a syandana (a back cape) is "intended to fit all Warframes", and its "attachment peg" must be long enough for "the most convex and the most concave backs". Chest, shoulder and leg pieces follow the same rule. The guidelines give triangle budgets: 8,000 for an alternative helmet, 7,500 for a syandana, 2,500 for chest armour, and 5,000 for both shoulders or both legs ([Warframe TennoGen technical guidelines](https://www.warframe.com/en/steamworkshop/help)).
- **Halo Infinite** split armour into "cores" in 2021, and pieces fitted only their own core. During the Season 2 livestream, 343 Industries said it would "move away" from cores and admitted "we didn't realise it'd be as bad as it was". Fracture armour stayed limited because of its unusual shape ([Pure Xbox](https://www.purexbox.com/news/2022/04/halo-infinite-is-officially-ditching-its-controversial-armour-core-system), press).
- **Black Desert** ties outfits to the class. The official mobile wiki says outfit stats and appearance apply "only for the class you are currently playing" ([Black Desert Mobile wiki](https://www.world.blackdesertm.com/Ocean/Wiki?wikiNo=54)). Each class has a fixed sex and body. No Pearl Abyss statement on the reason was found.
- **Saints Row** (2022) offers prosthetic limbs, elf ears and robot eyes as parts ([GamesRadar](https://www.gamesradar.com/saints-row-character-customization), press).
- **Fortnite** outfits play one shared animation set. A third-party manual lists the UEFN skeleton at 88 bones ([Reallusion manual](https://manual.reallusion.com/Metahuman-Interoperability/Content/MetaHuman-Interoperability/17-Metahuman-Interoperability/Skeleton-between-CC-and-UE-Characters.htm), third party). Epic's own guide shows how to retarget animations onto the Fortnite mannequin ([Epic, Transfer character animations in UEFN](https://dev.epicgames.com/documentation/en-us/fortnite/transfer-character-animations-in-unreal-editor-for-fortnite)).
- **Starfield** has only human player characters. No developer statement on why was found.

## Techniques

### Attachments on bones

The cheapest alien feature is a rigid part on one bone. Jedi Academy's Kel Dor mask is skinned only to the `cranium` bone. SWG's Zabrak horns and Twi'lek lekku are hair objects, picked like a hairstyle. Dragon Age: Inquisition's horn menu replaces the hair menu. In each case, the part needs no new animation and costs one extra mesh.

A part that should move on its own, such as lekku or a tail, needs either bones or a simulation. Spore's "Jiggles" is a damped spring per unselected part, and it never feeds back into the animation ([Hecker et al. 2008](https://doi.org/10.1145/1360612.1360626)). KOTOR's danglymesh is a damped spring per vertex, with a small maximum stray of 2.5 to 3.5 cm for most hair and hoods (`crates/game/src/dangly.rs:1`). Jedi Academy's skeleton has `ltail` and `rtail` bones, but its playable Twi'lek's lekku are skinned to the spine and head instead.

### Species heads that share names but not shapes

SWG gives every species' head the same list of blend shape names. Each species' artist sculpts that species' features into those names. The Ithorian's trunk lives in the "nose length" blend, and the Mon Calamari's head dome in the "ears" blend ([customization_data.tab](https://github.com/SWG-Source/dsrc/blob/master/sku.0/sys.shared/compiled/game/datatables/customization/customization_data.tab)). The engine and the save format see one fixed set of sliders. The menu shows a different label per species.

### One shared skeleton or one skeleton per species

Most games that support many species share one skeleton, and add species-specific parts on top of it.

- SWG uses one animation map for ten species, with a scale range per species ([SWG-Source player templates](https://github.com/SWG-Source/dsrc/tree/master/sku.0/sys.shared/compiled/game/object/creature/player)).
- Jedi Academy uses one 53-bone skeleton for six species.
- SWTOR's species share numbered body types ([SWTOR 2.7.1 patch notes](https://www.swtor.com/patchnotes/4292014/2.7.1-patch-notes)).
- Most of WoW's allied races reuse an existing race's skeleton ([Warcraft Wiki](https://warcraft.wiki.gg/wiki/Allied_race), fan wiki).
- Mass Effect required "a humanoid skeleton" for every party member ([Shacknews](https://www.shacknews.com/article/79126/the-huge-cost-of-non-humanoid-aliens-in-mass-effect), press).

A species with its own skeleton, such as WoW's Kul Tirans, needs its own animations. The cost of that is close to WoW's estimate of about 180 animations per character ([Wowhead](https://www.wowhead.com/news/warlords-of-draenor-female-human-model-226814), press).

### Digitigrade legs

No game in this survey gives a playable humanoid species digitigrade legs together with a full wardrobe of shared armour. The Elder Scrolls dropped digitigrade beast legs after Morrowind, where those races could not wear boots ([UESP](https://en.uesp.net/wiki/Morrowind:Argonian), fan wiki). Guild Wars 2's Charr keep their animal legs, but ArenaNet simplified the whole armour system because of the number of races ([GameBanshee](https://www.gamebanshee.com/99r), press).

Retargeting an animation onto a digitigrade leg is a solved problem in tools. Unreal's IK Retargeter maps "chains" of bones, not single bones, so "you are defining the entire arm chain, no matter the number of bones". Optional IK goals then keep the feet where they should be ([Epic, IK Rig Animation Retargeting](https://dev.epicgames.com/documentation/en-us/unreal-engine/ik-rig-animation-retargeting-in-unreal-engine)). Spore goes further and synthesises the gait from leg lengths ([Hecker et al. 2008](https://doi.org/10.1145/1360612.1360626)). The cost of digitigrade legs is not the animation. The cost is every pair of trousers, boots and leg armour.

### Armour and helmets over horns, ears, tails and snouts

Games use these strategies, often together.

| Strategy | Example | Cost |
|---|---|---|
| Hide the item on that species | SWTOR's Togruta hide everything in the head slot ([forum report](https://forums.swtor.com/topic/689864-togruta-officially-confirmed-as-next-playable-species-more-or-less/)). FFXIV hid most hats on Viera and Hrothgar from 2019. | Free in art. Players dislike it. |
| Hide the alien part under the item | FFXIV 7.3 lets a Viera fold her ears under chosen hats ([official UI guide](https://na.finalfantasyxiv.com/uiguide/equipment/equipment-my/character_vears.html)). Players hide lekku in SWTOR ([forum](https://forums.swtor.com/topic/528577-hats-for-twileks/)). | One switch per item and per part. |
| Let it clip, and fix the worst items | SWTOR's patch notes fix lekku clipping one helmet at a time ([7.3.1a](https://www.swtor.com/patchnotes/91323/game-update-7.3.1a)). | Small and ongoing. |
| A separate item for the slot | Dragon Age: Inquisition's Qunari wear vitaar paint instead of helmets ([Dragon Age Wiki](https://dragonage.fandom.com/wiki/Vitaar)). Morrowind's beast races wear no boots or closed helmets ([UESP](https://en.uesp.net/wiki/Morrowind:Argonian)). | A few species items. No variants of existing items. |
| Species-only items | SWG's Wookiee armour and Wookiee robes ([SWGEmu archive](https://www.swgemu.com/archive/scrapbookv51/data/20070213112516)). | A whole wardrobe per species, usually small. |
| A variant of every item per species | WoW's helmet per race and sex ([modding forum](https://elijahketchersid.com/forums/thread-86034.html)). Skyrim's beast helmets ([Nexus Mods](https://www.nexusmods.com/skyrim/mods/15927)). | Items times species times sexes. |
| One attachment rule for every body | Warframe's attachment pegs ([TennoGen guidelines](https://www.warframe.com/en/steamworkshop/help)). | Free per body. The items must stay simple. |
| Automatic refit | WoW's machine learning helmet refit ([Blizzard Watch](https://blizzardwatch.com/2023/11/06/hair-under-helmets-ai/)). | A tool project. Results still need review. |

### Skin patterns and markings

Every game here stores markings as a choice from a list plus a colour. SWG stores a pattern index (`index_texture_1`) and a pattern colour (`index_color_pattern`) for the Twi'lek, the Wookiee, the Mon Calamari and the Ithorian ([customization_data.tab](https://github.com/SWG-Source/dsrc/blob/master/sku.0/sys.shared/compiled/game/datatables/customization/customization_data.tab)). Jedi Academy tints the whole skin with one red, green and blue value picked from a short list. Spore blends three player colours through masks that artists paint into each part's texture ([Jane Ng](https://janeng.com/?p=143)).

### Voices and faces for non-human mouths

FaceFX, the lip sync tool used by many games, maps the phonemes it hears to poses that an artist makes for each character. Epic's guide describes the default set as nineteen bone poses, fifteen for the lips and four for the eyes and brows ([Epic UDN, Introduction to FaceFX](https://udn.epicgames.com/Three/IntroductionToFaceFX.html)). A face with a different mouth needs its own set of poses, but the speech pipeline stays the same. A paper from the USC Institute for Creative Technologies says an artist-driven method suits "non-human characters" whose mouths do not match a human's ([ICT paper](https://ict.usc.edu/pubs/A%20Practical%20and%20Configurable%20Lip%20Sync%20Method%20for%20Games.pdf)).

Jedi Academy's Rodian head uses the jaw and lip bones of the shared skeleton, the same bones as the human heads. Its alien mouth is a matter of how the mesh is weighted to those bones, not of new bones.

### Procedural animation for any body plan

Spore is the only game in this survey that animates bodies nobody designed. Its method needs an animation tool of its own, a custom IK solver and a test team running animations on hundreds of creatures ([Hecker et al. 2008](https://doi.org/10.1145/1360612.1360626)). The paper's authors still list hugs, chin rubs and self-collision as things it cannot do.

## Costs

| Game | What the cost was | Source |
|---|---|---|
| Star Wars Galaxies | Every wearable would not fit the Ithorian neck, so the wearable system was changed instead of remaking each item. | [GameBanshee, quoting IGN](https://www.gamebanshee.com/news/73782-star-wars-galaxies-jump-to-lightspeed-feature.html) |
| Final Fantasy XIV | 389 of 2,998 hats still do not show on Viera, and 405 do not show on Hrothgar, six years after release. | [FFXIV wiki](https://ffxiv.consolegameswiki.com/wiki/Hrothgar_%26_Viera_Hat_Compatibility), fan wiki |
| World of Warcraft | About 3,600 animations for 10 races of 2 sexes. Helmet refits for every race are now a machine learning project. | [Wowhead](https://www.wowhead.com/news/warlords-of-draenor-female-human-model-226814), [Blizzard Watch](https://blizzardwatch.com/2023/11/06/hair-under-helmets-ai/), press |
| Guild Wars 2 | Professions times races made per-profession armour "next to impossible", so armour became three weight classes. | [GameBanshee](https://www.gamebanshee.com/99r), press |
| Mass Effect | A non-humanoid party member would have had a "huge" cost. Most of Andromeda's five or six planned species were cut for budget. | [Shacknews](https://www.shacknews.com/article/79126/the-huge-cost-of-non-humanoid-aliens-in-mass-effect), [Game Informer](https://www.gameinformer.com/2021/02/05/mass-effect-andromeda-was-supposed-to-have-a-lot-more-new-aliens), press |
| Dragon Age: Origins | Horns were dropped because of helmet slots. | [GamesRadar](https://gamesradar.com/our-final-dragon-age-2-interview-before-launch), press |
| Halo Infinite | Pieces built for one core did not fit others, and the core system was dropped. | [Pure Xbox](https://www.purexbox.com/news/2022/04/halo-infinite-is-officially-ditching-its-controversial-armour-core-system), press |
| Spore | Weeks of learning per animator. About 90% of animations passed on test creatures. | [Hecker et al. 2008](https://doi.org/10.1145/1360612.1360626) |

## Gaps in the sources

- No primary source was found for Star Wars Outlaws, Monster Hunter's Palico creator, Palia, Destiny's Awoken and Exo, or Starfield on this topic. They are left out.
- The often-repeated claim that FFXIV must refit about ten thousand items for a new race has no primary source that this survey could find.
- No Bethesda statement explains why Oblivion dropped digitigrade beast legs. The armour explanation is a fan explanation.
- WoW's per-race helmet models are described by modders, not by Blizzard.
- The SWTOR quote about Togruta montrals hiding the head slot is reported by players. The official page was not found.

## What this means for kjk

### Which approach fits

kjk should follow Star Wars Galaxies and Jedi Academy, not Spore. Both games keep one humanoid skeleton and one animation set, and make each species a different head and skin on that skeleton. This fits kjk exactly. kjk already plays Jedi Academy's animations on KOTOR's humanoid skeleton (`crates/asset_ja/src/rig.rs:1`). The bone pairing covers the pelvis, spine, neck, head, arms, hands and legs (`crates/asset_ja/src/rig.rs:73`). Any species built on that skeleton gets all of kjk's combat for free.

Spore's method answers a different question. It animates bodies with any number of limbs. kjk's species all hold a lightsaber in two hands and walk on two legs, so the cost of a Spore-style system would buy nothing.

### What KOTOR's assets already give

- KOTOR heads are separate models hung on the body's `headhook`, and kjk already draws any body with any head (`docs/research/2026-11-16-custom-look-mod-impact.md:170`). An alien head is therefore just another head.
- KOTOR 2's `heads.2da` already has a Twi'lek woman head (row 61), and a kjk test already hangs it on a player body (`crates/game/src/alignment_look/probes.rs:23`).
- KOTOR's headgear is small. kjk draws the head slot item, a mask or a visor, on the head's `gogglehook` (`crates/game/src/headgear.rs:1`). Armour that changes the whole look, such as KOTOR 1's Mandalorian battle armour, does it with a disguise row and not with a helmet (`docs/research/2026-11-16-custom-look-game-impact.md:188`). This is the biggest reason alien heads are cheaper in kjk than in an MMO. A Twi'lek's lekku or a Togruta's montrals rarely meet a hat at all.
- kjk already lets a player hide their headgear, as a render-only setting (`crates/game/src/headgear.rs:1`). That is the SWTOR and FFXIV fallback, already built.
- kjk already sways danglymeshes the way KOTOR does (`crates/game/src/dangly.rs:1`). Lekku made as danglymeshes would sway a little with no new system.
- KOTOR head meshes allow at most 16 bones per mesh, as a head mod author notes (`docs/research/2026-11-16-custom-look-mod-impact.md:103`). New alien heads must keep to that limit, or kjk must lift it.

### Cheap alien features

These features fit on the existing skeleton and the existing armour with little or no new art per item.

- **Alien heads.** A Twi'lek, Zabrak, Cathar or Rodian head is a head model on the `headhook`. Some original NPC heads, such as KOTOR 2's Twi'lek woman head, may serve as they are. New heads are new models, each with its own dark side textures.
- **Skin colour per species.** A palette of skin tones per species, as SWG and Jedi Academy do. The same tint must reach the bare body texture (body letter A), because KOTOR paints the underwear body with the head's skin tone (`docs/research/2026-11-16-custom-look-mod-impact.md:38`).
- **Markings.** A pattern index and a pattern colour, as SWG stores them. Zabrak tattoos, Twi'lek lekku patterns and Mirialan face marks all fit this.
- **Montrals and horns.** A rigid part on the head bone, like Jedi Academy's Kel Dor mask. Montrals and Zabrak horns need no animation.
- **Lekku that follow the body.** Jedi Academy's player Twi'lek simply skins its lekku to the spine and head. kjk can do the same, and add KOTOR's danglymesh sway on top.

### Expensive alien features

- **Lekku with real physics.** Lekku that swing, collide with the shoulders and react to combat need a bone chain and a solver. Jedi Academy has `ltail` and `rtail` bones, but its own playable Twi'lek does not use them. KOTOR's skeleton has no such bones (`crates/asset_ja/src/rig.rs:73`).
- **Tails.** A tail needs bones the KOTOR skeleton lacks, and a hole or a cover in every body model that covers the hips. In KOTOR every armour is a whole body model per body letter and per build, so a tail touches all of them.
- **Digitigrade legs.** The animation can be retargeted by bone chains, as Unreal does. The armour cannot. Every body letter's legs and boots would need a new version. Bethesda's beast races lost their digitigrade legs after Morrowind, and Morrowind's could not wear boots.
- **Snouts and large heads under masks.** A Rodian, Trandoshan or Ithorian head is much larger than a human head where masks and visors sit. Each mask would need a per-species fit, or the species would hide masks, as SWTOR's Togruta hide all headgear.
- **Wookiees and other large bodies.** A Wookiee is a whole new body with fur. SWG gave Wookiees their own armour instead of fitting the shared items. In kjk this means new body models for every armour letter, or a Wookiee that never shows armour.
- **Alien voices and lip sync.** A KOTOR head speaks through its own `talk` animation, which the LIP file indexes. Each new alien head needs those mouth poses made for it, as FaceFX needs per-character poses. Whether KOTOR's NPC alien heads already have player-quality talk animations needs checking per head.

### What the stored look would need

The `Look` block proposed in the tech note (`docs/research/2026-11-16-character-creation-tech.md:344`) needs a few additions for species. The species choice decides which heads, palettes, patterns and parts the other fields can name.

| Field | Size | Meaning |
|---|---|---|
| species | 1 byte | An id from a kjk species table. 0 means human. |
| skin tint | 1 byte | An index into that species' skin palette, not a shared one. |
| pattern | 1 byte | An index into that species' marking patterns. 0 means none. |
| pattern tint | 1 byte | An index into that species' marking palette. |
| parts | 1 byte count, then 2 bytes each | Each part is a slot id (1 byte, such as horns or lekku) and a style id (1 byte). |

The sliders block can stay as it is, if kjk copies SWG's idea. Each slider id has one meaning for the engine, and each species' head is sculpted into those ids in its own way. A species that has no use for a slider ignores it.

Three rules should go with it.

- **Species is a look, not a rule.** As in the tech note, the sim should never read the species. KOTOR's scripts and dialogue may still ask for the player's racial type, so kjk should keep reporting the original value to scripts until the game impact of an alien player is studied.
- **Headgear rules live in data, per item and per species.** Each mask or visor should say, per species, whether it shows, whether it hides a part (such as folding lekku), or whether it hides itself. This is FFXIV's 7.3 design. A missing rule should default to hiding the item on that species, which is the SWTOR Togruta rule and never shows clipping.
- **Only the parameters travel.** As in the tech note, clients rebuild the look from these few bytes and their own assets. A species block adds 5 bytes plus 2 bytes per part to each seat.
