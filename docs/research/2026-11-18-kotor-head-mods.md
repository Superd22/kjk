# KOTOR head mods with new 3D meshes

This document looks for KOTOR 1 and KOTOR 2 mods that add or replace human heads with new or reshaped 3D meshes. The goal is to find faces that kjk's head slider system can try to reproduce. A mod that only repaints a head keeps the original mesh, so it does not help a shape test. Such mods are listed only so the reader knows they were checked and set aside.

It extends `docs/research/2026-11-16-custom-look-mod-impact.md`. That document covers how head mods install and what a custom look would break. This one covers which head mods exist, how popular they are, and which ones ship real geometry.

All numbers were read on 2026-11-18. The sections before "What the archives contain" were written before any mod file was downloaded. In those sections, every claim about a mod's contents comes from its public page, and the "How we know" column says what on the page supports it. The last section was written after the archives were downloaded and opened.

## Sources and method

- Deadly Stream file pages were read for 46 head mods. The site's title search for "head", "heads", "high poly" and "remesh" found most of them. The K1 and TSL skin categories, sorted by downloads, found the most popular ones.
- Nexus Mods was read through the user's own Chrome, because Nexus refuses automated page reads. Its search for "head", "face", "revan", "anakin", "poly", "remodel" and "TCS" was sorted by endorsements in both games. Twelve Nexus mod pages were then opened.
- The KOTOR 2 Steam Workshop was searched for "head" and "face", sorted by most subscribed. The browse page shows no subscriber numbers, so only one item page was opened for its count.
- Snigaroo's KOTOR 1 and KOTOR 2 full builds on kotor.neocities.org were read for head mods ([K1 full build](https://kotor.neocities.org/modding/mod_builds/k1/full), [K2 full build](https://kotor.neocities.org/modding/mod_builds/k2/full)).

## The main findings

1. Most popular head mods are textures. The ten most downloaded head mods on Deadly Stream are all reskins, eye fixes or small mesh fixes. Revan's Head (17,258 downloads), HD Darth Malak (68,249), Canonical Jedi Exile (46,142) and Bearded PMHC01 (14.3k downloads and 608 endorsements on Nexus) ship only textures.
2. Mods with real new geometry are much less popular. Most have between 50 and 5,000 downloads. The best-known group is DarthParametric's "TOR Ports", which bring heads from Star Wars: The Old Republic (SWTOR) into both games. Each has between 745 and 4,754 downloads on Deadly Stream.
3. A new wave of high-poly remeshes started in late 2025. Dark Hope has published a high-poly version of 15 of KOTOR 1's female player heads between August and October 2026. Elayerfawkes45 has remeshed Carth, Mission, Canderous, Juhani, Zhar and several NPCs. Narve built a new Canderous head modelled on the actor Clancy Brown. These mods have few downloads because they are new, not because they are disliked.
4. The community's reference builds use no mod that adds a new human head mesh. They use texture reskins and a few mesh fixes, such as redrob41's TSL Head Model Fixes and Ashton Scorpius's Twi'lek ears.
5. Several heads are meant to look like a real person or a known character. The real people are Clancy Brown (Canderous), Keanu Reeves, Ewan McGregor as Obi-Wan Kenobi, Hayden Christensen as Anakin Skywalker, and Layne Staley of Alice in Chains. The known characters are Meetra Surik, Jaesa Willsaam, Kira Carsen, Kyle Katarn, Revan, and Bastila as drawn on the game's cover.

## Terms used in this document

- A **new mesh** is a head model built from scratch or ported from another game. Its vertices and UV map differ from every vanilla KOTOR head.
- A **reshaped mesh** is a vanilla KOTOR head model whose vertices the author moved, added or removed. Kenobi's head, for example, is "a heavily remodeled pmhc06" in the author's words.
- A **mesh fix** is a vanilla head with a small repair, such as a closed neck gap or a fixed eyelid. Its face shape stays the same.
- A **texture-only mod** ships TGA, TPC or DDS images and no MDL or MDX file. It keeps the vanilla mesh.
- A **port** moves a vanilla head from one KOTOR game to the other without changing its shape. A port is useful to kjk only as a copy of a vanilla head, which the other research agent already extracts.

## How login works on each site

- **Nexus Mods** requires a free account to download any file. This is the site's general policy. It was not tested here, because the browser used was already signed in.
- **Deadly Stream** shows a guest the button "Download this file", with no sign-in notice beside it. Whether the button then asks for a sign-in was not tested, because clicking it would start a download.
- **Steam Workshop** items are delivered only through a Steam account that owns KOTOR 2. They cannot be downloaded from a web page.

## All candidates

Downloads are the site's own count. On Nexus the "Likes" column gives endorsements. On Deadly Stream it gives the number of reactions, such as "Like" and "Thanks", plus the number of reviews. The "Mesh" column says whether the mod ships a new mesh, a reshaped mesh, a mesh fix, a port or textures only.

| # | Mod | Game | Author | Site | Downloads | Likes | Size | What it changes | Mesh | How we know | Heads | Meant to look like | Reuse terms on the page |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | [TOR Ports: Meetra Surik Female Player Head for TSL](https://deadlystream.com/files/file/1300-tor-ports-meetra-surik-aka-jedi-exile-female-player-head-for-tsl/) | K2 | DarthParametric | Deadly Stream | 4,754 | 11 reactions, 1 review | 1.47 MB | Adds a new female player head | New (SWTOR port) | Page says "Original models and textures ported from The Old Republic MMO" and describes dropped bones | 1 | Meetra Surik, the Jedi Exile, as drawn in SWTOR | None stated |
| 2 | [TOR Ports: Meetra Surik Female Player Head for K1](https://deadlystream.com/files/file/1302-tor-ports-meetra-surik-aka-jedi-exile-female-player-head-for-k1/) | K1 | DarthParametric | Deadly Stream | 1,357 | not read | not read | Same head for KOTOR 1 | New (SWTOR port) | Same series and wording as row 1 | 1 | Meetra Surik | Not read |
| 3 | [TOR Ports: Jaesa Willsaam Female Player Head for K1](https://deadlystream.com/files/file/1307-tor-ports-jaesa-willsaam-female-player-head-for-k1/) | K1 | DarthParametric | Deadly Stream | 4,035 | 5 reactions, 1 review | 2.03 MB | Adds a new female player head | New (SWTOR port) | Page explains that SWTOR's eyelids raised the bone count to 18 and two nose bones were dropped | 1 | Jaesa Willsaam, a SWTOR companion | None stated |
| 4 | [TOR Ports: Jaesa Willsaam Female Player Head for TSL](https://deadlystream.com/files/file/1306-tor-ports-jaesa-willsaam-female-player-head-for-tsl/) | K2 | DarthParametric | Deadly Stream | 3,348 | 7 reactions, 1 review | 1.97 MB | Same head for KOTOR 2 | New (SWTOR port) | Same wording as row 3 | 1 | Jaesa Willsaam | None stated |
| 5 | [TOR Ports: Kira Carsen Female Player Head for K1](https://deadlystream.com/files/file/1303-tor-ports-kira-carsen-female-player-head-for-k1/) | K1 | DarthParametric | Deadly Stream | 3,012 | not read | not read | Adds a new female player head | New (SWTOR port) | Same series as row 6 | 1 | Kira Carsen, a SWTOR companion | Not read |
| 6 | [TOR Ports: Kira Carsen Female Player Head for TSL](https://deadlystream.com/files/file/1301-tor-ports-kira-carsen-female-player-head-for-tsl/) | K2 | DarthParametric | Deadly Stream | 2,648 | 8 reactions, 2 reviews | 1.59 MB | Adds a new female player head | New (SWTOR port) | Page says it "adds the head model" of Kira and that SWTOR uses one base head mesh for Kira and Meetra | 1 | Kira Carsen | None stated |
| 7 | [TOR Ports: Pureblood Sith Male Player Head for K1](https://deadlystream.com/files/file/1309-tor-ports-pureblood-sith-male-player-head-for-k1/) | K1 | DarthParametric | Deadly Stream | 1,243 | 7 reactions, 2 reviews | 2.56 MB | Adds a Sith Pureblood head and underwear | New (SWTOR port) | Page describes porting the head and underwear models | 1 | A generic Sith Pureblood, not a human | None stated |
| 8 | [TOR Ports: Pureblood Sith Male and Female heads for TSL](https://deadlystream.com/files/file/1310-tor-ports-pureblood-sith-male-player-head-for-tsl/) | K2 | DarthParametric | Deadly Stream | 1,989 male, 1,614 [female](https://deadlystream.com/files/file/1795-tor-ports-pureblood-sith-female-player-head-for-tsl/) | not read | not read | Adds Sith Pureblood heads | New (SWTOR port) | Same series | 2 | Not human | Not read |
| 9 | [Narve's New High Poly Canderous Head](https://deadlystream.com/files/file/3092-narves-new-high-poly-canderous-head/) | K1 | Narve | Deadly Stream and [Nexus](https://www.nexusmods.com/games/kotor/mods?keyword=poly) | 60 on DS, 56 on Nexus | 3 reactions, 1 endorsement | 11.54 MB | Replaces Canderous's head | New | Page says "This is a new model that is significantly different from the vanilla version" and lists `p_candh.mdl` and `p_candh.mdx` | 1 | The actor Clancy Brown | None stated |
| 10 | [Canderous's head higher poly and retextured](https://deadlystream.com/files/file/2875-canderouss-head-higher-poly-and-retextured/) | K1 | Elayerfawkes45 | Deadly Stream | 160 | 3 reactions, 1 review | 17.86 MB | Replaces Canderous's head | Reshaped | Author writes "I remeshed Canderous" | 1 | An unnamed fan artwork of Canderous | None stated |
| 11 | [Carth head remesh higher poly and retextured (v2)](https://deadlystream.com/files/file/2881-carth-head-remesh-higher-poly-and-retextured-v2/) | K1 | Elayerfawkes45 | Deadly Stream | 150 (plus 95 for [v1](https://deadlystream.com/files/file/2880-carth-head-remesh-higher-poly-and-retextured/)) | 1 reaction | 2.29 MB | Replaces Carth's head | Reshaped | Title says "remesh higher poly". The page lists no files. | 1 | Carth, from a reference image the page does not show | None stated |
| 12 | [Higher poly and retextured Mission's head](https://deadlystream.com/files/file/2879-higher-poly-and-retextured-missions-head/) | K1 | Elayerfawkes45 | Deadly Stream | 217 | 1 reaction | 38.4 MB | Replaces Mission's head | Reshaped | Author writes "I remade Mission's textures and mesh" | 1 | Mission, a Twi'lek, so not fully human | None stated |
| 13 | [Zhar remesh and retextured](https://deadlystream.com/files/file/2853-zhar-remesh-and-retextured/) and [Master Ulthar Remesh](https://deadlystream.com/files/file/2852-master-ulthar-remesh/) | K1 | Elayerfawkes45 | Deadly Stream | 226 and 584 | not read | not read | Replace two Jedi Masters' heads | Reshaped | Titles say "remesh". The pages were not opened. | 2 | Zhar is a Twi'lek. Ulthar is human. | Not read |
| 14 | [High poly PFHA01 to PFHC04 series](https://deadlystream.com/search/?q=%22high%20poly%22&type=downloads_file&search_in=titles&sortby=newest) | K1 | Dark Hope | Deadly Stream | 78 to 180 each | 0 to 4 reactions each | 19.5 to 25 MB each | Replaces 15 vanilla female player heads | Reshaped (method not stated) | Each page says "The mod replaces the PFHxxx model and textures" | 15 (PFHA01, A02, A04, A05, B01 to B05, C01 to C04, plus [PFHA03 DP](https://deadlystream.com/files/file/3068-pfha03-dp/)) | The same women as the vanilla heads | None stated |
| 15 | [Improved Bastila Head](https://deadlystream.com/files/file/2090-improved-bastila-head/) | K1 | Darth_Tempust (Tempust85) | Deadly Stream | 2,806 | 13 reactions, 0 reviews | 1.14 MB | Replaces Bastila's head | New or reshaped (not stated which) | Page lists `p_bastilah.mdl` and `p_bastilah.mdx` and says the head is "fully weighted for talking and facial expressions" | 1 | Bastila as painted on the game's cover | "This mod may not be used without the explicit permission of the author" |
| 16 | [TCS Bastila V7](https://www.nexusmods.com/kotor/mods/1745) | K1 | TheChadSurvivor | Nexus | 4,305 (3,250 unique) | 123 endorsements | 87.8 MB | Remodels Bastila | Ships model files. Whether the head itself is new was not confirmed. | The terms say "Do NOT reupload my model files". The file preview did not load. | 1 | Bastila | Model files must never be reuploaded. Retextures are allowed if they list this mod as required. |
| 17 | [Detran's UHD Bastila](https://www.nexusmods.com/kotor/mods/1538) | K1 | detran | Nexus | 4,811 (2,819 unique) | 115 endorsements | not read | Retextures Bastila on TheChadSurvivor's model | Includes TheChadSurvivor's model files | Page says "This mod includes model files made by TheChadSurvivor" | 1 | Bastila | Not read |
| 18 | [Keanu Reeves Inspired Player Head (K1)](https://deadlystream.com/files/file/3041-keanu-reeves-inspired-player-head-k1/) | K1 | ShaolinGhost | Deadly Stream | 162 | 1 reaction | 36.7 MB | Adds a new male player head | New | Page describes a "Custom high-detail head model" with its own UVs. The author says to assume AI assistance. | 1 | The actor Keanu Reeves | "Do not re-upload, redistribute, or include this mod in another public release" without permission |
| 19 | [Kenobi (Episode III)-Inspired Head TSL](https://deadlystream.com/files/file/654-kenobi-episode-iii-inspired-head-tsl/) | K2 | 1Leonard | Deadly Stream | 2,556 | 9 reviews | 7.32 MB | Adds a new male player head | Reshaped | Page says "The head is a heavily remodeled pmhc06". The uninstall steps name .mdl and .mdx files. | 1 | Obi-Wan Kenobi as Ewan McGregor played him in Episode III | No reupload "without my knowledge" |
| 20 | [Kyle Katarn Head](https://deadlystream.com/files/file/762-kyle-katarn-head/) | K2 | 1Leonard | Deadly Stream | 2,572 | 1 reaction, 9 reviews | 18.53 MB | Adds a new male player head | Reshaped | Page says "a heavily modified pmhc07". The uninstall steps name .mdl and .mdx files. | 1 | Kyle Katarn from the Jedi Knight games | No reupload "without my knowledge" |
| 21 | [Female Exile Head ("Canon")](https://deadlystream.com/files/file/1230-female-exile-head-canon/) | K2 | bagthesnoot (model by Schizo) | Deadly Stream | 3,371 | 5 reviews | 1.17 MB | Replaces PFHC07 | Reshaped | Page says the new head "is a modified version of PFHC05" and that DarthParametric fixed the model | 1 | The Exile in a Wookieepedia Legends illustration | None stated |
| 22 | [Anakin Skywalker Player Head](https://www.nexusmods.com/kotor2/mods/1324) | K2 | Its The MuMu | Nexus | 487 (361 unique) | 9 endorsements | 8.6 MB | Replaces PMHC01 | Probably reshaped, not confirmed | Page thanks DarthParametric for help "on how to better use Blender" while "making this head". It lists no files. | 1 | Anakin Skywalker as Hayden Christensen played him in Episode III | Not read |
| 23 | [Twin Mullet Men v2](https://deadlystream.com/files/file/780-twin-mullet-men) | K2 | redrob41 | Deadly Stream | 1,677 | 6 reviews | 3.55 MB | Adds two male player heads | Reshaped | Changelog says "The only files that have changed are RMHC06.mdl & RMHC06.mdx". The author "modified ... mostly PMHH01 model and texture". | 2 | Variants of KOTOR 1's "Mullet Man" head | None stated |
| 24 | [Layne Staley PC Head](https://deadlystream.com/files/file/54-layne-staley-pc-head/) | K2 | VarsityPuppet | Deadly Stream | 590 | 2 reviews | 2.99 MB | Adds a new male player head | Not confirmed | The v2 changelog says "Remapped head", which suggests a new UV map and so a model file | 1 | The singer Layne Staley | "You may use this mod as you see fit" with credit, but "DO NOT release it with any of your mods without my permission" |
| 25 | [VP modder's source compilation](https://deadlystream.com/files/file/213-vp-modders-source-compilation/) | K2 | VarsityPuppet | Deadly Stream | 4,008 | 3 reviews | 3.31 MB | Modder's resource with two heads | Not confirmed | Page lists a "Meetra Surik Canon Head" and a "Revan Canon Head". The Nexus retexture [VP's Canon Revan](https://www.nexusmods.com/kotor/mods/77) (177 endorsements) calls it "the Canon Revan Model by Varsity Puppet". | 2 | Canon Revan and canon Meetra Surik | None stated |
| 26 | [Darth Maul Head for Kotor 1](https://deadlystream.com/files/file/3042-darth-maul-head-for-kotor-1/) | K1 | EwokKotor | Deadly Stream and [Nexus](https://www.nexusmods.com/games/kotor/mods?keyword=head) | 50 on DS, 110 on Nexus | 1 endorsement | 7.72 MB | Replaces PMHB03 | Reused from other mods | Page says "There are two head .mdl's included", built on Effixian's Zabrak heads | 2 | Darth Maul, a Zabrak | Thanks DarthParametric for permission to use his asset |
| 27 | [Shaleena Original Head Restoration](https://deadlystream.com/files/file/1896-shaleena-original-head-restoration/) | K1 | StellarExile | Deadly Stream | 3,778 | 4 reactions | 387 kB | Restores a cut vanilla head and fixes its teeth | Mesh fix of a vanilla head | Page says it "adjusts the model so that her upper teeth are no longer hidden" | 1 | Not a likeness | Ask before redistributing or reusing |
| 28 | [Party Model fixes and HD Bastila](https://deadlystream.com/files/file/1273-party-model-fixes-and-hd-bastila/) | K1 | redrob41 | Deadly Stream | 33,409 | 12 reactions, 6 reviews | 9.96 MB | Fixes UV errors on party models | Mesh fix | Page lists "head models for Bastila and Carth" among UV fixes | 2 | Not a likeness | None stated |
| 29 | [TSL Head Model Fixes](https://deadlystream.com/files/file/489-tsl-head-model-fixes/) | K2 | redrob41 | Deadly Stream | 53,903 | 8 reactions, 14 reviews | 178 kB | Fixes six vanilla heads | Mesh fix | Page lists the tongue, neck and eyelid fixes per head | 6 | Not a likeness | None stated |
| 30 | [Male Head Pack by Inyri Forge](https://deadlystream.com/files/file/617-male-head-pack-by-inyri-forge/) | K1 | inyriforge | Deadly Stream | 5,421 | 9 reviews | 6.95 MB | Adds 10 male player heads | Not confirmed | Page mentions many .2da files and horn items. It does not mention models. | 10 (3 not human) | One Zabrak is Darth Maul | "Uploaded by permission of the author" |
| 31 | [6 Player Heads](https://deadlystream.com/files/file/1418-6-player-heads) | K2 | InSidious | Deadly Stream | 814 | 2 reactions | 8.92 MB | Adds 6 player heads | Not confirmed | Page mentions "a defect in the model" but not whether the model is new | 6 | Carth, Trask, Shaardan, Gadon Thek, Dhagon Ghent and the Peragus medical officer | None stated |
| 32 | [K1 Heads Port](https://deadlystream.com/files/file/1407-k1-heads-port/) | K2 | Ferc Kast | Deadly Stream | 2,258 | 2 reviews | 8.57 MB | Brings all 30 KOTOR 1 player heads into KOTOR 2 | Port | Page says it "ports the original player heads from KOTOR I" | 30 | Vanilla faces | Ask before reusing |
| 33 | [Effixian's K1 to K2 Heads](https://deadlystream.com/files/file/1392-effixians-k1-to-k2-heads/) | K2 | Effix | Deadly Stream | 1,436 | 2 reviews | 3.31 MB | Brings PFHC01 and PMHA05 into KOTOR 2 | Port | Page says the heads are "from KotOR 1" | 2 | Vanilla faces | Reuse "if you simply ask me", with credit |
| 34 | [mjpb3's Bastila PC Head](https://www.nexusmods.com/kotor2/mods/994) | K2 | mjpb3 | Nexus | 4,143 (3,010 unique) | 169 endorsements | 3.6 MB | Adds Bastila as a KOTOR 2 player head | Port, probably | Page says it "adds a new Bastila PC head" with TSLPatcher. It does not say the mesh changed. | 1 | Bastila | Not read |
| 35 | [Play as Bastila](https://steamcommunity.com/sharedfiles/filedetails/?id=1144989479) | K2 | Aizawa | Steam Workshop | 11,010 current subscribers | 562 ratings | 2.96 MB | Replaces one female head with Bastila's | Not confirmed | Page describes replacing "one of the female heads". It says the item was removed for breaking Steam guidelines. | 1 | Bastila | None stated |
| 36 | [Canon Male Inspired Head for PC](https://www.nexusmods.com/kotor/mods/1601) | K1 | LuKaSai, from RefrigeratorDry495 | Nexus | 893 | 34 endorsements | 2.2 MB | Replaces a male player head | Port plus textures | Page says it combines "Effixian Kotor's 2 Head model" with new textures | 1 | Canon Revan from SWTOR | Original author's "open permissions for making bug fixes" |
| 37 | [SWTOR Canon Inspired head for Male PC](https://www.nexusmods.com/kotor/mods/1589) | K1 | RefrigeratorDry495 | Nexus | 586 | 11 endorsements | 1.7 MB | Replaces a male player head | Reshaped, slightly | Page says "Using a KOTOR 2 head model with a few edits" | 1 | Canon Revan from SWTOR | Not read |
| 38 | [Anakin EP2 Head (PMHA05)](https://www.nexusmods.com/kotor/mods/1327) | K1 | Ergonpandilus | Nexus | 1,024 | 52 endorsements | 3.6 MB | Replaces PMHA05 | Not confirmed | Page gives no file list | 1 | Anakin Skywalker as Hayden Christensen played him in Episode II | Not read |
| 39 | [anakin head](https://www.nexusmods.com/kotor/mods/1263) | K1 | Ellimar, reuploaded by sirran | Nexus | 1,012 | 41 endorsements | 815 KB | Anakin head | Not confirmed | Page gives no file list | 1 | Anakin Skywalker | Reupload of a lost mod without the author's permission |
| 40 | [Long Hair for revan](https://www.nexusmods.com/kotor/mods/20) | K1 | valten7 | Nexus | 11,046 | 454 endorsements | 243 KB | Gives PMHC01 and PMHC05 long hair | Hair added to vanilla meshes, probably | Author says the dark side bug comes from not knowing how to "apply textures to the models" | 2 | Canon Revan | Others may "feel free to mod the files" |
| 41 | [Revan's Head](https://deadlystream.com/files/file/939-revans-head) | K1 | Darth Vhail | Deadly Stream | 17,258 | not read | not read | Reskins PMHC04 as SWTOR's Revan | Texture only | Covered in the 2026-11-16 document | 1 | Revan | Not read |
| 42 | [Bearded PMHC01 Head Replacement](https://deadlystream.com/files/file/485-bearded-pmhc01-head-replacement/) | K1 | inyriforge (vesaldi on Nexus) | Deadly Stream and [Nexus](https://www.nexusmods.com/games/kotor/mods?keyword=head) | 2,853 on DS, 14.3k on Nexus | 608 endorsements | 1.7 MB | Adds a beard or goatee to PMHC01 | Texture only | File list names only .tga files | 1 | Not a likeness | None stated |
| 43 | [Ajunta Pall Unique Appearance](https://deadlystream.com/files/file/824-ajunta-pall-unique-appearance/) | K1 | Silveredge9 | Deadly Stream | 69,721 | 4 reactions, 8 reviews | 1.58 MB | Gives Ajunta Pall a unique look | Texture only | File list has textures, 2DA files and scripts, no models | 1 | Not a likeness | Reuploaded with permission |
| 44 | [HD Darth Malak](https://deadlystream.com/files/file/980-hd-darth-malak/) | K1 | Dark Hope | Deadly Stream | 68,249 | 21 reactions, 19 reviews | 37.64 MB | Retextures Malak | Texture only | Page says "New HD texture for Malek" | 1 | Malak | None stated |
| 45 | [Darth Sion remake](https://deadlystream.com/files/file/1244-darth-sion-remake/) | K2 | FF97 | Deadly Stream | 34,366 | 6 reactions, 4 reviews | 28.1 MB | Repaints Sion from film photos | Texture only, per the K2 build | The K2 build says it improves "Sion's base textures" | 1 | Mixes photos of Two-Face, Kroenen and Snoke | None stated |
| 46 | [Kenobi, Kyle Katarn and other reskins by Effix, Dark Hope and others](https://deadlystream.com/files/category/8-skins/?sortby=file_downloads&sortdirection=desc) | Both | Many | All three sites | up to 46,142 | various | small | Canonical Jedi Exile, Atton with scruff, Effixian's reskins on the Workshop, Male Head 2k Re-skin (277 endorsements), Handmaiden Head Retexture (246 endorsements), Better PMHC04, Count Dooku PMHC01 | Texture only | Each page or build note describes textures only | many | Some aim at canon looks | Mixed |

Row 46 groups texture-only mods so that the table stays readable. None of them changes a face's shape.

## How many ship real meshes

The table holds 46 rows, and row 46 stands for at least 10 more texture mods. Counted by row, the results are as follows.

- 18 rows ship a new or reshaped human head mesh that the page confirms. These are rows 1 to 6, 9 to 11, 13 to 15, 18 to 21, 23 and 37. Row 13 counts for Ulthar only, because Zhar is a Twi'lek. Row 14 alone holds 15 heads.
- 2 rows ship model files whose head part is not confirmed. These are TCS Bastila V7 and Detran's UHD Bastila (rows 16 and 17), which share one model.
- 4 rows ship new meshes that are not human. These are the Sith Pureblood heads (rows 7 and 8), Mission (row 12) and Darth Maul (row 26).
- 3 rows ship mesh fixes that keep the vanilla face shape. These are rows 27, 28 and 29.
- 5 rows are ports of vanilla heads between the two games. These are rows 32 to 34 and 36, and probably 35.
- 8 rows could not be confirmed either way from the page. These are rows 22, 24, 25, 30, 31, 38, 39 and 40.
- The rest ship textures only.

## Shortlist

These ten picks ship real new or reshaped human head meshes, as far as the page shows. Each one has a face that differs clearly from the vanilla heads, or that is meant to look like someone a tester can check against photos.

| Pick | Mod | Size | Why it is a good target |
|---|---|---|---|
| 1 | [TOR Ports: Meetra Surik for TSL](https://deadlystream.com/files/file/1300-tor-ports-meetra-surik-aka-jedi-exile-female-player-head-for-tsl/) | 1.47 MB | It is the most downloaded head with a new mesh (4,754 downloads). The face is a canon character with plenty of reference art. It was made by a respected modder and already fits KOTOR's 16-bone face rig. |
| 2 | [TOR Ports: Jaesa Willsaam for K1](https://deadlystream.com/files/file/1307-tor-ports-jaesa-willsaam-female-player-head-for-k1/) | 2.03 MB | It is the second most downloaded new mesh (4,035). DarthParametric says SWTOR builds Kira and Meetra on one base head, so Kira would add little shape. Jaesa is not named as sharing that head, so she is the better second female face. This was not checked against the models. |
| 3 | [Narve's New High Poly Canderous Head](https://deadlystream.com/files/file/3092-narves-new-high-poly-canderous-head/) | 11.54 MB | It is the only page that confirms a mesh built far from the vanilla one and names its model files. The face is modelled on Clancy Brown, so a tester can compare the slider fit against a real person. It is a heavy, older male face, which the female TOR ports do not cover. |
| 4 | [Kenobi (Episode III)-Inspired Head TSL](https://deadlystream.com/files/file/654-kenobi-episode-iii-inspired-head-tsl/) | 7.32 MB | It reshapes a vanilla KOTOR 2 head into a famous face, Ewan McGregor's Obi-Wan. Because it starts from PMHC06, it shows how far a vanilla head can move while it keeps KOTOR's style. That is exactly the range kjk's sliders need. |
| 5 | [Kyle Katarn Head](https://deadlystream.com/files/file/762-kyle-katarn-head/) | 18.53 MB | It is by the same author as pick 4 and uses the same method on PMHC07. It adds a second well-known bearded male face with game art to compare against. |
| 6 | [Improved Bastila Head](https://deadlystream.com/files/file/2090-improved-bastila-head/) | 1.14 MB | It has the most downloads (2,806) of any companion head with a new model file. It aims at the game's cover art, so the target face is fixed and well known. Its page restricts use without permission, so it should stay a private test target. |
| 7 | [Keanu Reeves Inspired Player Head (K1)](https://deadlystream.com/files/file/3041-keanu-reeves-inspired-player-head-k1/) | 36.7 MB | It is a new male mesh meant to look like a very well-known real person. It tests whether sliders can reach a narrow, long modern face. The author says to assume AI assistance, and the page forbids redistribution. |
| 8 | [Dark Hope's high-poly female player heads](https://deadlystream.com/search/?q=%22high%20poly%22&type=downloads_file&search_in=titles&sortby=newest) | about 20 to 25 MB each | These are 15 vanilla KOTOR 1 female faces rebuilt at higher detail by one author. They match the vanilla faces one to one, so they test whether the sliders can reach finer detail of the very faces kjk must reproduce. Start with [PFHA01](https://deadlystream.com/files/file/3097-high-poly-pfha01/), the most downloaded (180). |
| 9 | [Elayerfawkes45's Carth remesh](https://deadlystream.com/files/file/2881-carth-head-remesh-higher-poly-and-retextured-v2/) | 2.29 MB | It is a higher-poly Carth, the main male companion of KOTOR 1. Together with his [Canderous remesh](https://deadlystream.com/files/file/2875-canderouss-head-higher-poly-and-retextured/) (17.86 MB), it gives two companion faces to compare with Narve's Canderous and the vanilla Carth. |
| 10 | [Female Exile Head ("Canon")](https://deadlystream.com/files/file/1230-female-exile-head-canon/) | 1.17 MB | It reshapes PFHC05 into the Exile from a Legends illustration. It gives a second, independent attempt at Meetra Surik to compare with pick 1, made from a vanilla head instead of a SWTOR one. |

Two more mods would join the shortlist if their pages confirmed a new head mesh. [Anakin Skywalker Player Head](https://www.nexusmods.com/kotor2/mods/1324) (8.6 MB) appears to be a head reshaped in Blender toward Hayden Christensen. [TCS Bastila V7](https://www.nexusmods.com/kotor/mods/1745) (87.8 MB, 123 endorsements) ships model files, but the page does not say whether the head is among them.

## Permission concerns

- None of these mods should ship inside kjk. The plan here is only to use them as private shape targets on a developer's machine. Any use beyond that needs the author's written permission.
- The TOR Ports heads (picks 1 and 2) are models taken from Star Wars: The Old Republic, a commercial game. DarthParametric states no terms. The models belong to the SWTOR rights holders, so kjk could never redistribute them, even with the modder's consent.
- Improved Bastila Head says "This mod may not be used without the explicit permission of the author." Its page adds that the author, now named Tempust85, gave permission for its publication in 2026.
- TCS Bastila V7 says its model files must never be reuploaded "under any circumstances".
- The Keanu Reeves head forbids redistribution without permission. Its author also says to assume AI assistance.
- Kenobi and Kyle Katarn forbid reuploading "without my knowledge".
- Layne Staley allows free use with credit but forbids release inside another mod without permission.
- Five heads are meant to look like real people. These are Clancy Brown, Keanu Reeves, Ewan McGregor, Hayden Christensen and Layne Staley. Fitting sliders to them in private is fine, but kjk should not ship a preset named after or built to match a real person.
- The anakin head on Nexus (row 39) is a reupload of a lost mod by someone who is not its author. Play as Bastila (row 35) was removed from the Workshop for breaking Steam's guidelines.

## What could not be confirmed

- No page lists its archive's full contents, except Narve's Canderous, Improved Bastila, Bearded PMHC01 and Ajunta Pall. For the other mods, "new" or "reshaped" rests on the author's words, the uninstall steps or the changelog. Opening each archive would settle this, but that needs the user's approval to download.
- Dark Hope's 15 high-poly pages say only that they replace "the model and textures". They give no polygon count and do not say whether the heads were subdivided, resculpted or rebuilt.
- Whether the Anakin TSL head, Layne Staley head, VarsityPuppet's canon heads, 6 Player Heads, Male Head Pack and both KOTOR 1 Anakin heads ship new meshes was not confirmed.
- Whether TCS Bastila V7's model files include a new head was not confirmed. Nexus's file preview failed to load.
- Deadly Stream's guest download rule was not tested. Nexus's login rule was taken from its known policy and not tested.
- Steam Workshop subscriber counts were read for one item only, Play as Bastila. The browse page shows no counts.
- Likes and file sizes for some Deadly Stream pages, such as the K1 versions of the TOR ports, were not read. Their download counts come from search results.
- Nexus Mods' internal search may miss mods whose titles do not contain the words searched. Mods titled only with a character name, such as "Fens - Carth", were checked only when a search found them.

## What the archives contain

The 10 shortlisted mods, the two Nexus mods that would join the shortlist, and six mods from the "could not confirm" list were downloaded on 2026-11-18. That makes 25 archives, because some mods offer more than one file. No file was larger than 1 GB, so none was skipped for size. The largest was TCS Bastila V7 at 92 MB. Dark Hope's high-poly series was sampled with three of its 15 heads. These are PFHA01 and the two heads whose vanilla face shapes differ most from PFHA01, which are PFHB05 and PFHB02.

The archives were opened as data only. No installer, TSLPatcher or script was run. Each head model was read with kjk's own MDL reader and compared with the vanilla model of the same name, or with the vanilla model the author says the head started from. The heads were exported as OBJ files with UVs and PNG textures, for private use as test targets on one machine. kjk must never ship them or anything made from them.

### How the shape was measured

- The **face mesh** is the head's largest mesh that is skinned to the face bones. In many heads it also holds the hair.
- **Vertices** below count distinct vertex positions. The game splits a vertex in two along a UV seam, and those copies are counted once.
- Each mod head and its vanilla head are placed so that the point between the two eyes lines up. Both are then scaled by the same factor, which gives the vanilla head a 63 mm eye distance. The mod head keeps its own size.
- The **front of the face** is the area up to 50 mm either side of the centre, from 35 mm above the eyes to 90 mm below them, and no more than 15 mm behind the eyes.
- The **gap** is the distance from each point on the front of one face to the nearest point on the surface of the other face, measured both ways. The table gives the average gap, the largest gap and the share of points whose gap is more than 2 mm.
- A head counts as **vanilla** if its largest gap is under 1 mm. It counts as **reshaped** if it starts from a vanilla mesh and moves its points. It counts as **new** if no vanilla head shares its mesh.

### Heads with a new or reshaped face

| # | Mod | Game | Head model | Face vertices | Face triangles | Compared with | Its vertices and triangles | Average gap | Largest gap | Share over 2 mm | Verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | TOR Ports: Meetra Surik for TSL | K2 | `PFH_TOR_Meetra` | 1,090 | 2,152 | none | | | | | New |
| 2 | TOR Ports: Jaesa Willsaam for K1 | K1 | `PFH_TOR_JaesaK1` | 1,092 | 2,156 | none | | | | | New |
| 3 | Narve's New High Poly Canderous Head | K1 | `p_candh` | 6,115 | 12,160 | K1 `p_candh` | 388 and 760 | 2.55 mm | 10.33 mm | 56% | Reshaped strongly |
| 4 | Elayerfawkes45's Canderous | K1 | `p_candh` | 6,115 | 12,160 | K1 `p_candh` | 388 and 760 | 0.75 mm | 5.52 mm | 7% | Reshaped lightly |
| 5 | Elayerfawkes45's Carth v2 | K1 | `p_carthh` | 5,987 | 11,904 | K1 `p_carthh` | 380 and 744 | 0.99 mm | 4.05 mm | 7% | Reshaped lightly |
| 6 | Kenobi (Episode III)-Inspired Head TSL | K2 | `pmhobi01` | 392 | 769 | K2 `pmhc06` | 392 and 769 | 1.54 mm | 7.41 mm | 31% | Reshaped |
| 7 | Kyle Katarn Head | K2 | `pmhk01` | 398 | 780 | K2 `pmhc07` | 398 and 780 | 2.12 mm | 9.00 mm | 35% | Reshaped |
| 8 | Improved Bastila Head | K1 | `p_bastilah` | 414 | 800 | K1 `p_bastilah` | 408 and 784 | 2.37 mm | 7.33 mm | 59% | Reshaped |
| 9 | Keanu Reeves Inspired Player Head | K1 | `P_KNEOH` | 2,130 | 3,161 | none | | | | | New |
| 10 | Dark Hope's High Poly PFHA01 | K1 | `pfha01` | 4,920 | 9,714 | K1 `pfha01` | 405 and 794 | 2.80 mm | 10.21 mm | 66% | Reshaped strongly |
| 11 | Dark Hope's High Poly PFHB05 | K1 | `pfhb05` | 3,033 | 6,036 | K1 `pfhb05` | 510 and 1,006 | 1.11 mm | 7.15 mm | 5% | Reshaped lightly |
| 12 | Dark Hope's High Poly PFHB02 | K1 | `pfhb02` | 2,112 | 4,170 | K1 `pfhb02` | 439 and 866 | 2.26 mm | 7.74 mm | 60% | Reshaped |
| 13 | Female Exile Head ("Canon") | K2 | `PFHC07` | 438 | 860 | K2 `pfhc05` | 438 and 860 | 2.37 mm | 6.91 mm | 63% | Reshaped |
| 14 | Anakin Skywalker Player Head v1.2 | K2 | `PMHC01` | 451 | 886 | K2 `pmhc01` | 451 and 886 | 0.43 mm | 3.73 mm | 8% | Reshaped lightly |
| 15 | TCS Bastila V7.1 | K1 | `p_bastilah` | 6,328 | 12,544 | K1 `p_bastilah` | 408 and 784 | 1.77 mm | 7.88 mm | 44% | Reshaped |
| 16 | Layne Staley PC Head v2 | K2 | `P_LayneH` | 408 | 757 | nearest is K2 `p_handmaidenbd` | 419 and 820 | 1.69 mm | 7.53 mm | 32% | New, or reshaped from a head that could not be found |
| 17 | VP modder's source compilation, Revan | K2 | `PMHC01` | 466 | 924 | K2 `pmhc01` | 451 and 886 | 2.14 mm | 6.68 mm | 51% | Reshaped |
| 18 | VP modder's source compilation, Meetra Surik | K2 | `PFHC06` | 446 | 838 | K2 `pfhc06` | 422 and 832 | 0.32 mm | 2.92 mm | 3% | Reshaped slightly |

What the table shows, head by head:

- Narve's Canderous is not built from scratch, although its page says it is a new model. It has exactly 16 times the vanilla head's 760 triangles. That is what two rounds of subdivision produce. Narve then moved the face a long way. More than half of the front of the face moved by more than 2 mm.
- Elayerfawkes45's Canderous has the same mesh as Narve's, with the same 6,115 vertices and 12,160 triangles. Its face moved much less. Only 7% of the front of the face moved by more than 2 mm. Elayerfawkes45's Carth was made the same way, with 16 times the vanilla Carth's 744 triangles. Subdivision alone rounds off the vanilla head's flat faces by about 1 mm, so these two heads are mostly smoother copies of the vanilla faces.
- TCS Bastila V7.1 also started as the vanilla Bastila head subdivided twice, which gives 16 times 784 triangles. The whole head model has 61,965 vertices, and most of them are in separate hair strands. Its archive confirms that the model files include the head.
- Kenobi, Kyle Katarn, the Female Exile and the Anakin head keep the vanilla mesh exactly, with the same vertices and triangles, and only move points. The Female Exile replaces `PFHC07`, but its mesh is the vanilla `PFHC05` mesh, as its page says.
- The Anakin head moves the face only a little. Its largest gap is 3.73 mm, and 8% of the front of the face moved by more than 2 mm. VarsityPuppet's Meetra Surik moves even less, with a largest gap of 2.92 mm.
- Improved Bastila and VarsityPuppet's Revan add a few vertices and triangles to the vanilla mesh and then reshape it.
- Dark Hope's three heads are rebuilt meshes with between 4 and 12 times the vanilla triangle count. Their triangle counts are not a whole multiple of the vanilla counts, so they are not plain subdivisions. PFHA01 and PFHB02 moved a lot. PFHB05 stays close to the vanilla face. In PFHA01 and PFHB05 the eyes also sit about 3 mm closer together, with an eye distance of 60.2 mm and 60.6 mm.
- The Layne Staley head matches no vanilla head. The nearest vanilla head is a female one, the Handmaiden's, at an average of 1.69 mm. It may be a heavily reshaped vanilla head whose source could not be found.
- Improved Bastila's download holds three versions of the head. The table uses the one named "Edited". The Layne Staley download holds two versions, and the table uses version 2.

### Mods checked and set aside

- Male Head Pack by Inyri Forge holds 10 head models, `pmhi01` to `pmhi10`. Every one is an exact copy of a vanilla KOTOR 1 head. Three are copies of `pmha03`, two of `pmhc04`, and one each of `pmha05`, `pmhc01`, `pmhc02`, `pmhc03` and `pmhc05`. Its Zabrak horns are separate mask models.
- 6 Player Heads holds six head models. Each is an exact copy of a vanilla NPC head. These are the Peragus medical officer, Shaardan, Gadon Thek, Carth, Dhagon Ghent and Trask.
- Anakin EP2 Head (PMHA05) ships only textures for the vanilla `PMHA05` head.
- The Nexus "anakin head" by Ellimar ships only textures and a lightsaber item. Its readme says that it repaints `PMHH01` in KOTOR 2, even though it is listed under KOTOR 1 on Nexus.

### Where the files are

- The archives, the unpacked files and the exported heads are in `~/.claude/tools/likeness-fit-data/kotor-heads/mods/`. Each mod has its own folder, and each head's OBJ, MTL and PNG files are in that folder's `extracted` subfolder.
- A contact sheet of all 18 heads is at `~/.claude/tools/likeness-fit-data/kotor-heads/mods/mod_heads.png`. It shows each head untextured from the front and the side, next to the vanilla head it was compared with.
- The scripts that downloaded, read and measured the heads are in `~/.claude/tools/kotor-heads/mods/`.

## High-resolution face textures

This section looks for mods that repaint or upscale the face textures of the vanilla human heads. kjk's paint sliders (eyebrows, lip line, lip colour, eyelid crease and skin colour) need a sharp painted face to compare against. The vanilla face textures are small. Most KOTOR 1 heads use a 256x256 texture, and most KOTOR 2 heads use a 512x512 one. At that size the lip line is only a few texels wide.

A retexture helps only if it keeps the vanilla UV layout, so that it still fits the vanilla mesh. Every file listed here was checked against the vanilla texture of the same name. These files are private test targets on a developer's machine. kjk never ships them.

All numbers were read on 2026-11-18. The files in this section were downloaded and opened, as the archives in the section above were.

### Where the files are

- Each download sits in its own folder under `~/.claude/tools/likeness-fit-data/kotor-heads/hd-textures/<short-name>/archives/`.
- The face textures taken out of each archive are converted to PNG in `<short-name>/png/`. The file `<short-name>/textures.json` lists each one with its source path inside the archive.
- `hd-textures/index.json` maps each vanilla head mesh to its best HD texture. It gives the resolution, the mod, its link, how the texture was made, and the other HD textures that also fit that mesh.
- `hd-textures/fit_results.json` holds the fit check for every texture.
- `hd-textures/hd_face_sheet.png` shows 12 heads, with the vanilla face area beside the HD one at the same display size.
- The scripts are in `~/.claude/tools/kotor-heads/hd/`. `dsget.py` downloads from Deadly Stream, `hdconv.py` extracts and converts, `hdfit.py` checks the fit and writes the index, and `hdsheet.py` draws the sheet.

### How the fit was checked

- Each archive was opened as data only. Only textures whose file name matches a vanilla head texture of the same game were extracted. Nothing in an archive was run.
- Each HD texture was scaled down to the vanilla size and compared with the vanilla texture. The script blurs both images slightly and compares their brightness. It reports the correlation over the whole texture and over the face area. The face area is the UV box of the face mesh's front vertices.
- A phase correlation measures how far the HD image is shifted from the vanilla one. A texture counts as fitting the vanilla mesh when the shift is at most 2 vanilla texels and either correlation is at least 0.3.
- A texture whose face-area correlation is below 0.6 is marked as a repaint. A repaint still sits on the vanilla UVs, but it paints a different face, for example with a new beard, new make-up or a different skin colour. This was confirmed by eye for the borderline cases.
- A texture counts as high resolution only if it is larger than the vanilla one. Effixian's TSL player head reskins, for example, are 512x512 like the vanilla KOTOR 2 heads, so they do not count.
- For each mesh, the index picks the texture that is most faithful to the vanilla face first, then the one made by hand, then the largest. The score is 4 times the face-area correlation, plus 1.0 for a hand-made texture, 0.8 for a hand-guided upscale and 0.3 for a raw AI upscale, plus 0.25 for each doubling of size above 512 pixels. A texture that is the mesh's own vanilla texture gets 0.5 more.

### Results

- 314 face textures were extracted and checked. 308 of them fit their vanilla mesh. The 6 that do not fit are hologram head textures from Ultimate Character Overhaul. Their upscales do not line up with the vanilla holograms.
- 190 of the 198 vanilla head meshes now have at least one HD texture that fits. These are all 93 KOTOR 1 meshes and 97 of the 105 KOTOR 2 meshes. One of the KOTOR 1 meshes, the hologram of Admiral Dodonna, gains only a 64x64 eye texture, so 189 meshes gain a real face texture.
- 45 KOTOR 1 meshes and 21 KOTOR 2 meshes have a texture made by hand or upscaled with hand touch-ups. The others have only a raw AI upscale from Ultimate Character Overhaul.
- The best textures are 1024x1024 or 2048x2048. One KOTOR 2 head, PFHC05, has a 3896x3896 hand-edited texture.
- No downloaded texture uses a different UV layout. Five of Dark Hope's textures are shifted by one vanilla texel, which is within the tolerance. No texture is flipped.

| Source of the best texture | KOTOR 1 meshes | KOTOR 2 meshes | Resolution | How it was made |
|---|---|---|---|---|
| Dark Hope's player head HD series | 17 | 0 | 1024 | Hand-edited |
| redrob41's 4x Upscale+ packs and Party Model fixes | 15 | 17 | 1024 and 2048 | AI upscales blended and touched up by hand |
| Laast's Characters Revisited (updates part) | 3 | 0 | 2048 | Upscaled, then reworked by hand |
| Better PMHC04 Male head | 1 | 0 | 2048 | Repainted |
| anakinnzz' Complexion Tweaks | 0 | 3 | 2048 | Upscaled, with hair, skin and eyebrows painted by hand |
| Female Player Character 5 Face | 0 | 1 | 3896 | Hand-edited in Photoshop |
| Ultimate Character Overhaul LITE | 57 | 76 | 512 (K1) and 512 or 1024 (K2) | Raw ESRGAN AI upscale |

### Packs found and what was downloaded

"Likes" gives Nexus endorsements, or Deadly Stream reviews where no endorsements exist. "UVs" says whether the textures fit the vanilla mesh. It is "yes, checked" when a downloaded file was compared, and "not checked" otherwise.

| Pack | Game | Site | Heads covered | Face texture size | UVs | Downloads | Likes | Size | How it was made | Downloaded |
|---|---|---|---|---|---|---|---|---|---|---|
| [Ultimate Character Overhaul -REDUX-](https://www.nexusmods.com/kotor/mods/1282) by ShiningRedHD | K1 | Nexus | Every character in the game, 104 of the checked head textures | 512 in LITE (2x), 1024 in FULL (4x) | Yes, checked | 171,139 unique | 5,737 | 314.5 MB LITE, 1.2 GB FULL | Raw ESRGAN AI upscale | LITE TPC part only. FULL is over 1 GB. |
| [Ultimate Character Overhaul -REDUX-](https://www.nexusmods.com/kotor2/mods/1060) by ShiningRedHD | K2 | Nexus | Every character in the game, 95 of the checked head textures | 512 or 1024 in LITE (2x) | Yes, checked | 85,112 unique | 2,877 | 504.8 MB LITE, 1.9 GB FULL TPC | Raw ESRGAN AI upscale | LITE TPC part only. FULL is over 1 GB. |
| [4x Upscale+ Character Textures & Model Fixes](https://deadlystream.com/files/file/2659-4x-upscale-character-textures-model-fixes/) by redrob41 | K1 | Deadly Stream | Companions and about a dozen NPC heads. Version 0.52 has no player heads. | 1024 and 2048 | Yes, checked | 8,011 | 1 review | 643 MB (4x TPC part) | Several AI upscales blended by hand, then touched up by hand | 4x TPC part. The 2 GB TGA part was skipped. |
| [4x Upscale+ Character Textures & Model Fixes (TSL)](https://deadlystream.com/files/file/2660-4x-upscale-character-textures-model-fixes-tsl/) by redrob41 | K2 | Deadly Stream | Companions and NPC heads, 18 checked head textures | 1024 and 2048 | Yes, checked | 4,930 | 1 review | 838 MB (4x TPC part) | Same as the KOTOR 1 version | 4x TPC part. The 2.48 GB TGA part was skipped. |
| [Party Model fixes and HD Bastila](https://deadlystream.com/files/file/1273-party-model-fixes-and-hd-bastila/) by redrob41 | K1 | Deadly Stream | Bastila and Jolee | 2048 for Bastila, 512 for Jolee | Yes, checked | 33,409 | 6 reviews | 10 MB | Hand-made | Yes |
| [Characters Revisited](https://www.nexusmods.com/kotor/mods/1639) by Laast | K1 | Nexus | Every character | Mostly 2048 | Yes, checked for the 3 heads in the updates part | 3,607 unique | 134 | 3.1 GB main, 90 MB updates | Upscaled, then every texture reworked by hand | Updates part only (PMHC01, Canderous, Czerka commander). The main part is over 1 GB. |
| [KOTOR II Revisited](https://www.nexusmods.com/kotor2/mods/1256) by Laast | K2 | Nexus | Every character in its Characters part | Mostly 2048 | Not checked | 2,465 unique | 99 | 4.4 GB Characters, 211 MB update | Hand-painted, partly from reworked vanilla upscales | Update part only. It holds no head textures. The Characters part is over 1 GB. |
| [Vanilla Characters HD Optimized](https://deadlystream.com/files/file/2987-vanilla-characters-hd/) | K2 | Deadly Stream | Every character | About 2048 | Not checked | 395 | 0 reviews | 1.54 GB main, 142 MB and 40 MB patches | Topaz Gigapixel AI 4x upscale | No. The main file is over 1 GB. |
| [Selphadur's Kotor Texture Redux](https://www.nexusmods.com/kotor/mods/1302) | K1 | Nexus | Almost every texture in the game | Not read | Not checked | 11.5k unique | 724 | 9.0 GB | 4x AI upscale with hand-made improvements | No. The only full file is over 1 GB. |
| [Optimized and Upscaled Textures - K1](https://deadlystream.com/files/file/2833-optimized-and-upscaled-textures-k1/) | K1 | Deadly Stream | Every texture in the game | 2x vanilla, so 512 for most heads | Not checked | 1,757 | 1 review | 680 MB | Raw 2x upscale | No. It is one archive of the whole game at only 2x, the same size as Ultimate Character Overhaul LITE. |
| Dark Hope's player head HD series, for example [PFHA01 HD](https://deadlystream.com/files/file/1739-pfha01-hd/) | K1 | Deadly Stream | 27 of the 30 player heads. There are pages for every PFH and PMH head except PFHC01, PFHC04 and PMHB02. | 1024 | Yes, checked | 2,406 to 3,185 for the three pages read | 0 to 9 reviews | 22 to 25 MB each | Hand-edited retextures | 23 heads. PMHC01, PMHC02, PMHC04 and PMHC05 failed because Deadly Stream stopped answering. |
| Dark Hope's NPC HD series, for example [HD Darth Malak](https://deadlystream.com/files/file/980-hd-darth-malak/) and [HD Carth Onasi](https://deadlystream.com/files/file/1133-hd-carth-onasi/) | K1 | Deadly Stream | About 30 NPCs and companions, including Malak, Carth, Canderous, Bastila, Jolee, Trask, Vrook, Uthar, Bandon, Davik, Calo Nord and commoners | Not read | Not checked | 68,249 for Malak, 61,189 for Carth | 14 to 19 reviews | 3 to 43 MB each | Hand-edited | No. Deadly Stream stopped answering. |
| [Fens series](https://www.nexusmods.com/kotor/mods/1192) by Fenharel | K1 | Nexus | Bastila, Mission, Jolee, Carth, Canderous, Juhani, PMHC04 and PFHC01 | 1024, and 2048 for Canderous | Yes, checked | 111,599 unique for Jolee | 2.3k to 4.9k each | 8 to 17 MB each | Hand-made retextures that change the look a little, for example Carth gets more stubble | Yes, all 8 |
| [Better PMHC04 Male head](https://deadlystream.com/files/file/393-better-pmhc04-male-head/) | K1 | Deadly Stream | PMHC04 | 2048 | Yes, checked | 2,928 | 7 reviews | 32.9 MB | Repainted | Yes |
| [Male Head 2k Re-skin - Mullet Man](https://www.nexusmods.com/kotor/mods/1323) by elucidAce | K1 | Nexus | PMHC04 | 2048 | Yes, checked, but a repaint | 5,115 unique | 277 | 41.4 MB | Hand-edited. It adds a beard. | Yes |
| [Revan's Face (Blue Eyes)](https://www.nexusmods.com/kotor/mods/1404) by Alakzar | K1 | Nexus | PMHC04 | 2048 | Yes, checked, but a repaint | 1,637 unique | 80 | 7.2 MB | Blue-eyed edit of elucidAce's texture | Yes |
| [Bastila High Resolution Retexture](https://www.nexusmods.com/kotor/mods/1152) by Curtis1973 | K1 | Nexus | Bastila | 512 | Yes, checked | 8,783 unique | 440 | 10.3 MB | Hand-made, with new hair and eyes | Yes |
| [anakinnzz' Party and Player Head Complexion Tweaks](https://deadlystream.com/files/file/2876-anakinnzz-party-and-player-head-complexion-tweaks/) | K2 | Deadly Stream and [Nexus](https://www.nexusmods.com/kotor2/mods/1319) | Atton, Handmaiden, female Exile, Bao-Dur, Visas, Mira, Disciple and Kreia | 2048 | Yes, checked | 1,720 | 3 reviews, 50 endorsements | 1.01 GB in all, split by character | Upscaled from 512, with hair, skin and eyebrows painted by hand | Exile, both Atton options, Visas and Bao-Dur. The other four parts failed when Deadly Stream stopped answering. |
| [Female Player Character 5 Face and Portrait](https://www.nexusmods.com/kotor2/mods/1227) by SolaLuka | K2 | Nexus | PFHC05 | 3896 | Yes, checked | 511 unique | 32 | 15.2 MB | Hand-edited in Photoshop | Yes |
| [Effixian's Player Head Reskins](https://deadlystream.com/files/file/1096-effixians-player-head-reskins/) | K2 | Deadly Stream | 16 player head textures | 512, the same as vanilla | Yes, checked | 3,930 | 5 reviews | 15.4 MB | Hand-painted reskins | Yes, but not high resolution |
| [Handmaiden Head Retexture](https://www.nexusmods.com/kotor2/mods/943) by corpsecotillion | K2 | Nexus | Handmaiden | 512, the same as vanilla | Yes, checked | 4,758 unique | 247 | 1.5 MB | Hand-made | Yes, but not high resolution |
| [Quanon's reskins](https://deadlystream.com/files/file/941-quanons-canderous-ordo-reskin/) of Canderous, [Carth](https://deadlystream.com/files/file/973-quanons-carth-onasi-reskin/) and [Jolee](https://deadlystream.com/files/file/1113-quanons-jolee-reskin/) | K1 | Deadly Stream | 3 companions | Larger than vanilla, size not read | Not checked | 50,655, 2,749 and 2,536 | 8 to 14 reviews | 19, 37 and 43 MB | Hand-made | No. Deadly Stream stopped answering. |
| [HD 2K Visas Marr](https://deadlystream.com/files/file/519-darth-sapiens-presents-hd-2k-visas-marr/) by Darth_Sapiens | K2 | Deadly Stream | Visas | 2048 | Not checked | 36,736 | 31 reviews | 6.2 MB | Hand-made, keeps the vanilla look | No. Deadly Stream stopped answering. |
| [4k Atton](https://deadlystream.com/files/file/441-4k-atton/) by Constar | K2 | Deadly Stream | Atton | 4096 | Not checked | 38,577 | 19 reviews | 27.6 MB TPC part | Reworked vanilla textures. The author says the face is still like vanilla. | No. Deadly Stream stopped answering. |
| [Canonical Jedi Exile](https://deadlystream.com/files/file/170-canonical-jedi-exile/) | K2 | Deadly Stream | One female player head | Not read | Not checked | 46,142 | 14 reviews | 565 kB | Photoshop repaint | No. Deadly Stream stopped answering. |
| Smaller TSL reskins: [Atton with scruff](https://deadlystream.com/files/file/528-atton-rand-with-scruff/), [Leilukin's Kreia](https://deadlystream.com/files/file/1003-leilukins-kreia-reskin/), [Higher Detail Handmaiden](https://deadlystream.com/files/file/858-higher-detail-handmaiden-skin-retexture-fit-an-athletic-version/), [Darth Sion remake](https://deadlystream.com/files/file/1244-darth-sion-remake/) | K2 | Deadly Stream | One character each | Not read | Not checked | 432 to 34,366 | 3 to 10 reviews | 0.8 to 28 MB | Hand-made | No. Deadly Stream stopped answering. |
| [Jedi Council HD](https://www.nexusmods.com/kotor/mods/1278) and the other "4X Upscaled Texture" packs by Red11BY | K1 | Nexus | Zhar, Vandar, Dorak and Vrook | 2048 | Not checked | 4,443 unique | 192 | 13.6 MB | Raw ESRGAN upscale | No. Ultimate Character Overhaul replaces them. |
| Effixian's Workshop reskins, [Ultimate Character Overhaul (official Workshop version)](https://steamcommunity.com/sharedfiles/filedetails/?id=2282938060) and [Complete Character Overhaul - Ultimate HD Pack](https://steamcommunity.com/sharedfiles/filedetails/?id=1834523847) | K2 | Steam Workshop | Many | Not read | Not checked | Not shown | Not shown | Not read | Mixed | No. Workshop items download only through the Steam client. |

The community mod builds on kotor.neocities.org use Ultimate Character Overhaul at 2x for both games. On top of it they add Dark Hope's HD heads, Fens - Jolee and Quanon's Canderous in KOTOR 1, and Canonical Jedi Exile, HD 2K Visas Marr and 4k Atton in KOTOR 2.

### Files skipped for size

These single files are over 1 GB, so they were not downloaded. The user can decide whether to fetch them.

| File | Size | Page |
|---|---|---|
| Ultimate Character Overhaul -REDUX- (FULL), TPC, KOTOR 1 | 1.2 GB | https://www.nexusmods.com/kotor/mods/1282?tab=files |
| Ultimate Character Overhaul -REDUX- (FULL), TPC, KOTOR 2 | 1.9 GB | https://www.nexusmods.com/kotor2/mods/1060?tab=files |
| Characters Revisited, main file | 3.1 GB | https://www.nexusmods.com/kotor/mods/1639?tab=files |
| KOTOR II Revisited, 02_K2R_Characters | 4.4 GB | https://www.nexusmods.com/kotor2/mods/1256?tab=files |
| Vanilla Characters HD 3.1 TPC | 1.54 GB | https://deadlystream.com/files/file/2987-vanilla-characters-hd/ |
| Selphadur's Kotor Texture Redux v1.1 Standalone | 9.0 GB | https://www.nexusmods.com/kotor/mods/1302?tab=files |
| 4x Upscale+ Character Fixes KotOR, 4x TGA part | 2 GB | https://deadlystream.com/files/file/2659-4x-upscale-character-textures-model-fixes/ |
| 4x Upscale+ Character Fixes TSL, 4x TGA part | 2.48 GB | https://deadlystream.com/files/file/2660-4x-upscale-character-textures-model-fixes-tsl/ |

The two 4x TGA parts hold the same textures as the 4x TPC parts that were downloaded, only uncompressed. They add nothing for kjk. Characters Revisited and KOTOR II Revisited are the most promising of the skipped files. They are reworked by hand at 2048x2048 for every character, and only three of their heads could be checked here.

### What could not be done

- Deadly Stream stopped answering during the downloads, at about 19:55 on 2026-11-18. It was still down 30 minutes later, when this section was written. WebFetch from another network was refused too, so the site itself may have been down. Our segmented downloader opened up to 48 connections per file at first, so a block on our address is also possible. The downloader now opens at most 4.
- The missing Deadly Stream files are listed in `~/.claude/tools/kotor-heads/hd/batch3.txt`. Running `waitds.sh` in the same folder fetches them one page at a time once the site answers. After that, `hdconv.py` and `hdfit.py` must be run again to add them to the index.
- Steam Workshop items were not downloaded, because they need the Steam client.
- The Dark Hope player head pages were read for three heads only. Their download counts and review counts for the other heads were not read.
