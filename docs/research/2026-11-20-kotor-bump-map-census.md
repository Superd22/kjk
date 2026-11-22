# Bump map census of KOTOR 1 and KOTOR 2

This note was researched on 2026-11-20.
David asked how many textures of the original games name a bump map, so that he knows how large a gap it is that kjk does not draw bump maps.
The question comes from `2026-11-20-head-detail-maps.md`, which found that kjk reads the `bumpmaptexture` and `bumpyshinytexture` lines of a texture's TXI but never draws a bump map.

Terms used below.
- A **TXI** is the small text block of settings that belongs to a texture. It sits at the end of a `.tpc` texture file, or in a loose `.txi` file next to a `.tga`.
- A **bump map** is a second texture that a first texture names with `bumpmaptexture`. It tells the renderer how the surface is tilted at each texel, so that light can shade small dents and ridges that the colour texture does not show.
- A **bumpy shiny** texture has a `bumpyshinytexture` line. That line names an environment map, which is a cube picture of the surroundings. The original engine reflects that picture through the bump map. kjk treats the name as an ordinary environment map (`crates/asset_odyssey/src/texture.rs:107`).
- An **environment map** is named by `envmaptexture`. kjk already draws it.
- A **normal map** stores the surface direction in its red, green and blue channels, so a flat surface is the colour (128, 128, 255). A **height map** is a grey picture where brighter means higher.

## Summary

- The count was made on the Steam installs of both games on this Mac (the Aspyr ports), by reading every texture of every archive with kjk's own `fmt_odyssey` readers. I did not use the Windows PC, which did not answer on its address.
- In KOTOR 1, 120 of the 3,318 textures that are not lightmaps and not interface pictures name a `bumpmaptexture`. That is 3.6%. In KOTOR 2 the number is 76 of 3,344, which is 2.3%.
- A `bumpyshinytexture` is named by 107 textures in KOTOR 1 and by 80 in KOTOR 2. Most of them also name a bump map. Only 2 textures in KOTOR 1 and 12 in KOTOR 2 name a bumpy shiny texture without a bump map.
- Altogether 122 textures in KOTOR 1 and 88 in KOTOR 2 name a bump map or a bumpy shiny texture. An `envmaptexture` is named by 338 textures in KOTOR 1 and by 477 in KOTOR 2. kjk draws all of those, and it also draws the cube map of the bumpy shiny textures, so it draws an environment map on 445 textures in KOTOR 1 and 557 in KOTOR 2.
- No lightmap and no interface texture names any of the three settings.
- Most bump-mapped textures are surfaces of areas. In KOTOR 1 that is 87 of the 120 bump-mapped textures. In KOTOR 2 it is 36 of the 76. The others belong to creatures, droids, non-human heads, doors, placeable objects, minigame models and test textures. No weapon texture names a bump map in either game.
- No human head texture names a bump map in either game. None of the 151 (KOTOR 1) and 140 (KOTOR 2) head textures whose names start with `pmh` or `pfh` names a bump map or an environment map. The only head textures with a bump map are the Twi'lek heads (`twilek_m01` to `twilek_m05`) and three NPC heads built on the same model family (`n_komadh`, `n_xorh` and `n_zharh`). All of them share the bump map `Twilek_M01b`.
- The bump map files exist. In KOTOR 1, 86 different files are named, and 85 of them exist. One file, `LDA_flr04bmp`, is named by `lda_flr03` and is not in any archive. In KOTOR 2 all 52 named files exist. Most are normal maps. The water bump maps are grey height maps.
- The three texture packs `swpc_tex_tpa`, `swpc_tex_tpb` and `swpc_tex_tpc` do not differ in this. They hold the same names with the same TXI text. Only the sizes of the pictures differ, and the bump maps shrink with the rest.
- My judgement is that the gap is small for characters and larger for areas. Human heads are not affected, and a few creatures, droids and Twi'lek heads are. In KOTOR 1 about a third of the room models of the game use a bump-mapped texture, and that is where a player could notice the gap. In KOTOR 2 it is about 6% of the room models. Section 7 gives the reasons, and the last section says what I could not check.

## 1. Counts of textures

The three groups below are separate resources in the archives.
- **World textures** are the textures that models use. In KOTOR 1 they are the 3,294 textures of the texture pack, 9 more from `Patch.erf` whose names are not in the pack, and 15 loose pictures in `textures.bif` and `legacy.bif`. In KOTOR 2 they are the 3,286 textures of the pack, 10 loose pictures in `textures.bif` and `legacy.bif`, and 48 pictures in the `override` folder.
- **Lightmaps** are the baked light pictures of the rooms.
- **Interface textures** are the pictures in `swpc_tex_gui.erf`.

A texture's TXI was chosen as the game chooses it. A loose `.txi` of the texture's name comes first, and otherwise the TXI stored at the end of the `.tpc` is used.

| Group | KOTOR 1 textures | KOTOR 2 textures |
|---|---|---|
| World textures | 3,318 | 3,344 |
| Lightmaps | 5,586 | 3,801 |
| Interface textures | 1,570 | 1,912 |
| All textures | 10,474 | 9,057 |

| Setting named by a world texture | KOTOR 1 | KOTOR 2 |
|---|---|---|
| `bumpmaptexture` | 120 | 76 |
| `bumpyshinytexture` | 107 | 80 |
| `envmaptexture` | 338 | 477 |
| `bumpmaptexture` and `bumpyshinytexture` together | 105 | 68 |
| `bumpyshinytexture` without `bumpmaptexture` | 2 | 12 |
| `bumpmaptexture` or `bumpyshinytexture` | 122 | 88 |
| `bumpmaptexture` with no environment map of any kind | 14 | 7 |
| Environment map of any kind (`envmaptexture` or `bumpyshinytexture`) | 445 | 557 |
| Different bump map files named | 86 | 52 |

The lightmaps and the interface textures name none of the three settings in either game.

A world texture counts only once even where it is stored twice.
In KOTOR 1 the `Patch.erf` file holds 84 textures. 75 of them replace a texture of the pack under the same name, and none of the 84 names a bump map or an environment map.

## 2. Lists of textures

The columns hold the name that each setting points to, as the TXI spells it. A dash means that the texture has no such line.
The column "Assigned by" says whether I found a model that uses the texture (model) or had only the name to go by (name). Section 3 explains the groups.

### KOTOR 1, 122 textures

| Texture | bumpmaptexture | bumpyshinytexture | envmaptexture | Group | Assigned by |
|---|---|---|---|---|---|
| lda_flr03 | LDA_flr04bmp | CM_Dantne | - | area surface | name |
| lda_flr05 | LDA_flr01B | CM_Dantne | - | area surface | model |
| lda_flr06 | LDA_nobump | CM_Dantne | - | area surface | name |
| lda_flr07 | LDA_flr01B | CM_dantii | - | area surface | model |
| lda_flr08 | LDA_nobump | CM_dantii | - | area surface | model |
| lda_flr09 | LDA_flr09b | CM_Dantne | - | area surface | model |
| lda_flr10 | LDA_flr10b | CM_danint | - | area surface | name |
| lda_orock | LDA_flr01B | CM_HawkBaym | - | area surface | name |
| lda_water01 | LMA_Water01B | CM_danint | - | area surface | model |
| lda_water02 | LDA_water02B | CM_HawkBaym | - | area surface | name |
| lda_water03 | LDA_water03b | CM_dsk | - | area surface | model |
| lda_water04 | LDA_water01b | CM_SaberCrys | - | area surface | model |
| lda_water2 | LDA_Water2B | CM_jedcom | - | area surface | name |
| leh_floor06 | LEH_floor06b | CM_eh | - | area surface | name |
| lka_mud02 | lka_mud02b | CM_kashyyyk | - | area surface | model |
| lka_wall01 | LKA_wall01B | CM_kashyyyk | - | area surface | model |
| lko_water01 | LKO_Water01B | CM_Caves | - | area surface | model |
| lkoa_acid01 | Lkoa_acid01B | CM_sithtomb | - | area surface | model |
| lma_floor03 | LMA_floor03BMP | CM_UPMAN | - | area surface | model |
| lma_ocean | LMA_oceanBMP | - | CM_mansea | area surface | model |
| lma_wall16 | LMA_wall16BMP | CM_UPMAN | - | area surface | model |
| lma_water01 | LMA_Water01B | CM_Manaan0 | - | area surface | model |
| lma_water02 | LMA_Water01B | CM_MFOUNT | - | area surface | model |
| lma_water03 | LMA_Water01B | CM_MRPOOL | - | area surface | model |
| lma_waterfall | LMA_waterfallB | CM_MFOUNT | - | area surface | model |
| lmg_water01 | LMG_Water01B | CM_M26mg | - | area surface | model |
| lqa_water | LQA_waterBMP | CM_QATEST | - | area surface | model |
| lrk_wall01 | LRK_wall01B | CM_rakata | - | area surface | name |
| lrk_wall03 | LRK_wall03b | CM_rakata | - | area surface | name |
| lrk_water02 | LRK_water02B | CM_BlackRakata | - | area surface | model |
| lrk_water03 | LRK_water03B | CM_BlackRakata | - | area surface | model |
| lsf_trim02 | LSF_trim02BMP | CM_LSF | - | area surface | model |
| lsf_trim21 | LSF_trim21BMP | CM_LSFAD | - | area surface | model |
| lsf_trim22 | LSF_trim22BMP | CM_LSFAD | - | area surface | model |
| lsf_wall13 | LSF_wall13BMP | CM_LSF | - | area surface | model |
| lsf_wall20 | LSF_wall20BMP | CM_LSFAD | - | area surface | model |
| lsf_wall23 | LSF_wall23BMP | CM_LSFAD | - | area surface | model |
| lsi_win01 | LSI_win01bmp | CM_asith | - | area surface | model |
| lta_flr01 | LTA_flr01aB | CM_rakata | - | area surface | model |
| lta_grate01_00a | LTA_grate01_00b | CM_m17ac | - | area surface | name |
| lta_grate01_01a | LTA_grate01_01b | CM_m17ab_00a | - | area surface | model |
| lta_metal01_00a | LTA_metal01_00b | CM_m17ac | - | area surface | name |
| lta_metal02 | LTA_Metal02b | CM_anchorhead | - | area surface | model |
| lta_metal02_00a | LTA_Metal02_00b | CM_m17ac | - | area surface | model |
| lta_metal11_00a | LTA_Metal11_00b | CM_m17ac | - | area surface | name |
| lta_metal11_01a | LTA_Metal11_01b | CM_m17ab_00a | - | area surface | model |
| lta_metal12 | LTA_Metal12b | CM_anchorhead | - | area surface | name |
| lta_metal20_00a | LTA_metal20_00b | CM_m17ac | - | area surface | model |
| lta_metal20_01a | LTA_metal20_01b | CM_m17ab_00a | - | area surface | name |
| lta_metal26_00a | LTA_metal26_00b | CM_m17ac | - | area surface | name |
| lta_metal26_01a | LTA_metal26_01b | CM_m17ab_00a | - | area surface | name |
| lta_trim02_00a | LTA_trim02_00b | CM_m17ac | - | area surface | model |
| lta_trm02_01a | LTA_trm02_01b | CM_m17ab_00a | - | area surface | model |
| lta_trm03_00a | LTA_trm03_00b | CM_m17ac | - | area surface | name |
| lta_trm03_01a | LTA_trm03_01b | CM_m17ab_00a | - | area surface | model |
| lts_bwall02 | LTS_Bwall04B | CM_m02abB | - | area surface | model |
| lts_bwall02i | LTS_Bwall02iB | CM_m02aa | - | area surface | model |
| lts_bwall04 | LTS_Bwall04B | CM_m02abB | - | area surface | model |
| lts_bwall04i | LTS_Bwall04B | CM_m02aa | - | area surface | model |
| lts_floor02 | LTS_floor01B | CM_m02aa | - | area surface | model |
| lts_nwall04 | LTS_Bwall04B | CM_m02ab | - | area surface | model |
| lts_nwall04i | LTS_Bwall04B | CM_m02aa | - | area surface | model |
| lts_pwall01i | LTS_Bwall04B | CM_m02aa | - | area surface | model |
| lts_untrim01 | LTS_untrim01b | CM_Baremetal | - | area surface | model |
| lts_water01 | LTS_Water01B | CM_awintaris | - | area surface | model |
| lts_ywall01 | LTS_Bwall04B | CM_m02abB | - | area surface | name |
| lun_gwing01 | LEH_antibump | CM_raktempb | - | area surface | name |
| lun_noise | LUN_noiseb | CM_unwbeach | - | area surface | name |
| lun_redwall01 | LUN_redwall01b | CM_unwsand | - | area surface | model |
| lun_sand04 | LUN_sand04b | CM_unwbeach | - | area surface | model |
| lun_sand05 | LUN_sand05b | CM_unwbeach | - | area surface | model |
| lun_tempflr01 | LUN_wll05b | CM_raktempb | - | area surface | name |
| lun_water01 | LUN_Water01B | CM_unwbeach | - | area surface | model |
| lun_water02 | LUN_Water01B | CM_unwsea | - | area surface | model |
| lun_water03 | LUN_Water01B | CM_unwsunset | - | area surface | model |
| lun_wll01 | LUN_wll01b | CM_unwsand | - | area surface | model |
| lun_wll02 | LUN_wll02b | CM_unwsand | - | area surface | model |
| lun_wll05 | LUN_wll05b | CM_raktempb | - | area surface | model |
| luz_water02 | LUZ_Water01B | CM_unwsea | - | area surface | model |
| luz_wll01 | LUZ_wll01b | CM_unwsand | - | area surface | model |
| luz_wll02 | LUZ_wll02b | CM_unwsand | - | area surface | model |
| lyv_trim03 | LYV_trim03b | CM_Baremetal | - | area surface | model |
| lza_flr04 | LZA_flr04BMP | CM_Hawkbaym | - | area surface | name |
| lza_flr06 | LZA_flr06BMP | CM_LZA03a | - | area surface | model |
| lza_flr07 | LZA_flr06BMP | CM_LZA03a | - | area surface | model |
| lza_flr12 | LZA_flr06BMP | CM_m33aa | - | area surface | name |
| lza_flr13 | LZA_flr06BMP | CM_m33aa | - | area surface | name |
| lza_win01 | - | CM_Czerka1 | - | area surface | model |
| v_dshanty01 | - | CM_tat | - | area surface | model |
| c_hutt01 | C_Hutt01b | - | - | creature or character body | model |
| c_hutt02 | C_Hutt01b | - | - | creature or character body | name |
| c_hutt03 | C_Hutt01b | - | - | creature or character body | name |
| c_hutt04 | C_Hutt01b | - | - | creature or character body | name |
| c_rakghoul01 | C_Rakghoul01b | CM_SpecMap | - | creature or character body | model |
| c_rancor01 | C_Rancor01b | - | - | creature or character body | model |
| c_terantanak01 | C_Terantanak01b | CM_SpecMap | - | creature or character body | model |
| lma_watersuit01 | LMA_WaterSuitBMP | CM_Manaan | - | creature or character body | name |
| lqa_dewback | LQA_dewbackBMP | CM_QATEST | - | creature or character body | model |
| lqa_mykal | LQA_mykalBMP | - | - | creature or character body | model |
| n_rakata01 | N_Rakata01b | - | - | creature or character body | model |
| n_rakata02 | N_Rakata01b | - | - | creature or character body | name |
| n_rakata03 | N_Rakata01b | - | - | creature or character body | name |
| n_selkath01 | N_Selkath01b | CM_SpecMap | - | creature or character body | model |
| n_selkath02 | N_Selkath01b | CM_SpecMap | - | creature or character body | name |
| n_selkath03 | N_Selkath01b | CM_SpecMap | - | creature or character body | name |
| n_komadh | Twilek_M01b | CM_SpecMap | - | head (non-human) | model |
| n_xorh | Twilek_M01b | CM_SpecMap | - | head (non-human) | model |
| n_zharh | Twilek_M01b | CM_SpecMap | - | head (non-human) | model |
| twilek_m01 | Twilek_M01b | CM_SpecMap | - | head (non-human) | model |
| twilek_m02 | Twilek_M01b | CM_SpecMap | - | head (non-human) | model |
| twilek_m03 | Twilek_M01b | CM_SpecMap | - | head (non-human) | model |
| plc_acidice | PLC_Acidice2B | CM_sithtomb | - | placeable | model |
| plc_water01 | PLC_water01B | CM_danint | - | placeable | model |
| lma_rfighter | LMA_RFighterBMP | CM_UPMAN | - | minigame or vehicle | name |
| mgf_turret01 | MGF_turretB01 | - | - | minigame or vehicle | model |
| mgg_ebonhawk01 | MGG_ebonhawkB01 | - | - | minigame or vehicle | model |
| v_ehawk01 | V_EHawk01b | CM_Hawkbaym | - | minigame or vehicle | model |
| lte_bmlmdiffuse | LTE_bump | - | - | test texture | name |
| lte_bmlmspec | LTE_bump | - | - | test texture | name |
| lte_bmspecdiff | LTE_bump | - | - | test texture | model |
| lte_embmcube | LTE_bump | LTE_cubemap | - | test texture | model |
| lte_embmsphere | LTE_bump | LTE_cubemap | - | test texture | model |

### KOTOR 2, 88 textures

| Texture | bumpmaptexture | bumpyshinytexture | envmaptexture | Group | Assigned by |
|---|---|---|---|---|---|
| 002_ebo_main1 | 002_EBO_Main1B | CM_Asith | - | area surface | model |
| 002_ebo_main2 | 002_EBO_Main2B | CM_Asith | - | area surface | name |
| dan_flr05 | DAN_flr01B | CM_Dantne | - | area surface | model |
| dan_flr06 | DAN_flr06B | CM_602dan | - | area surface | model |
| dan_flr09 | DAN_flr09b | CM_602dan | - | area surface | model |
| dan_mwfl | DAN_MWFlB | CM_610M | - | area surface | model |
| dan_mwtr | DAN_MWtrB | CM_610M | - | area surface | model |
| dan_newt | DAN_NEWtB | CM_jedcom | - | area surface | model |
| dan_water03 | DAN_water03b | CM_dsk | - | area surface | name |
| dan_water03n | DAN_water03Nb | CM_dskN | - | area surface | model |
| dan_water04 | DAN_water03b | CM_cryscave | - | area surface | name |
| dan_water05 | DAN_water03b | CM_Crys | - | area surface | model |
| dxn_eh1 | DXN_EH1b | CM_401dxn | - | area surface | name |
| dxn_engin01 | dxn_engin01b | - | CM_401dxn | area surface | name |
| dxn_freightl02 | dxn_FreightL02b | CM_401dxn | - | area surface | name |
| dxn_mt02 | - | CM_nadtomb | - | area surface | model |
| dxn_mt03 | - | CM_nadtomb | - | area surface | model |
| dxn_tek3 | - | CM_nadtomb | - | area surface | model |
| dxn_tmp1 | DXN_Tmp1b | CM_401dxn | - | area surface | name |
| dxn_tmp2 | DXN_Tmp2b | CM_401dxn | - | area surface | model |
| dxn_tx02 | - | CM_nadtomb | - | area surface | model |
| dxn_water03 | dxn_water03b | CM_401dxn | - | area surface | model |
| dxn_water04 | dxn_water03b | CM_nadtomb | - | area surface | model |
| dxn_water05 | dxn_water03b | CM_401dxn | - | area surface | name |
| dxn_water07 | dxn_water03b | CM_401dxn | - | area surface | model |
| dxn_water09 | dxn_water03b | CM_401dxn | - | area surface | model |
| dxn_wtr1 | DXN_Wtr1B | CM_401dxn | - | area surface | name |
| kor_water01 | KOR_Water01B | CM_Caves | - | area surface | model |
| leh_floor06 | LEH_floor06b | CM_eh | - | area surface | name |
| nar_d_flr01 | NAR_D_flr01b | CM_DRDYT | - | area surface | model |
| nar_d_flr03 | NAR_D_flr03b | CM_DRDYT | - | area surface | model |
| nar_d_wl01 | NAR_D_WL01b | CM_DRDYT | - | area surface | model |
| nar_d_wl03 | NAR_D_WL01b | CM_DRDYT | - | area surface | model |
| ond_tr04 | - | CM_ond | - | area surface | model |
| ond_water01 | OND_water01b | CM_ond | - | area surface | name |
| ond_water02 | OND_water01b | CM_510OND | - | area surface | model |
| tel_frtr | TEL_Frtrb | CM_201TEL | - | area surface | name |
| tel_roc | - | CM_261tel | - | area surface | name |
| tel_rock04 | TEL_rockb | CM_unwbeach | - | area surface | name |
| tel_rwfl | TEL_RWFlB | CM_RedAlt | - | area surface | model |
| tel_rwtr | TEL_RWtrB | CM_RedAlt | - | area surface | model |
| tel_water01 | TEL_Water01B | CM_unwbeach | - | area surface | model |
| v_dshanty01 | - | CM_tat | - | area surface | name |
| c_boma_d | C_Boma_b | - | - | creature or character body | model |
| c_boma_v_d | C_Boma_b | - | - | creature or character body | model |
| c_laigrek_d | C_Laigrek_B | CM_610M | - | creature or character body | model |
| c_malbeast01 | c_malbeast01b | - | - | creature or character body | model |
| c_rakghoul01 | C_Rakghoul01b | CM_SpecMap | - | creature or character body | model |
| c_rancor01 | C_Rancor01b | - | - | creature or character body | model |
| n_gand_d | N_gandB | CM_baremetal | - | creature or character body | model |
| n_gran | N_granB | CM_baremetal | - | creature or character body | model |
| n_qarrenb01 | - | CM_baremetal | - | creature or character body | model |
| n_rakata01 | N_Rakata01b | - | - | creature or character body | model |
| n_rakata02 | N_Rakata01b | - | - | creature or character body | name |
| n_rakata03 | N_Rakata01b | - | - | creature or character body | name |
| n_selkath01 | N_Selkath01b | CM_SpecMap | - | creature or character body | model |
| n_selkath02 | N_Selkath01b | CM_SpecMap | - | creature or character body | name |
| n_selkath03 | N_Selkath01b | CM_SpecMap | - | creature or character body | name |
| n_sullustan | - | CM_baremetal | - | creature or character body | model |
| c_condrdboss | C_CONDRDBOSSB | CM_baremetal | - | droid | model |
| c_condrdl | C_CONDRDLB | CM_baremetal | - | droid | model |
| c_drdmouse01 | C_DrdMouse01B | CM_baremetal | - | droid | model |
| c_drdmouse02 | C_DrdMouse02B | CM_baremetal | - | droid | model |
| c_drdmouse03d | C_DrdMouse03B | CM_baremetal | - | droid | model |
| c_drdmover01 | C_DrdMover01b | CM_baremetal | - | droid | model |
| c_drdmover02 | C_DrdMover02b | CM_baremetal | - | droid | model |
| c_mk1_drd | C_MK1_DRDb | CM_baremetal | - | droid | model |
| c_mk1_drd01 | C_MK1_DRDb | CM_baremetal | - | droid | name |
| c_mk1_drd02 | C_MK1_DRDb | CM_baremetal | - | droid | name |
| c_mk2_drd | C_MK2_DRDb | CM_baremetal | - | droid | model |
| c_mk2_drd01 | C_MK2_DRDb | CM_baremetal | - | droid | name |
| c_mk2_drd02 | C_MK2_DRDb | CM_baremetal | - | droid | name |
| c_tankdroid | C_TankDroidB | CM_baremetal | - | droid | model |
| p_gt | P_GtB | CM_goto | - | droid | model |
| n_komadh | Twilek_M01b | CM_SpecMap | - | head (non-human) | model |
| n_xorh | Twilek_M01b | CM_SpecMap | - | head (non-human) | model |
| n_zharh | Twilek_M01b | CM_SpecMap | - | head (non-human) | model |
| twilek_m01 | Twilek_M01b | CM_SpecMap | - | head (non-human) | model |
| twilek_m02 | Twilek_M01b | CM_SpecMap | - | head (non-human) | model |
| twilek_m03 | Twilek_M01b | CM_SpecMap | - | head (non-human) | model |
| twilek_m04 | Twilek_M01b | CM_SpecMap | - | head (non-human) | name |
| twilek_m05 | Twilek_M01b | CM_SpecMap | - | head (non-human) | name |
| dxn_dr02 | - | CM_nadtomb | - | door | model |
| dxn_dr03 | - | CM_401dxn | - | door | model |
| plc_acidice | PLC_Acidice2B | CM_sithtomb | - | placeable | model |
| plc_fncom | - | CM_nadtomb | - | placeable | model |
| plc_water01 | PLC_water01B | CM_danint | - | placeable | model |
| v_ehawk01 | V_EHawk01b | CM_Hawkbaym | - | minigame or vehicle | model |

## 3. What the groups hold

The next table counts the textures of the lists above by group. The numbers in brackets are the textures that I assigned by their name alone.

| Group | KOTOR 1 | KOTOR 2 |
|---|---|---|
| Area surfaces (walls, floors, water, acid) | 89 (24) | 43 (15) |
| Creatures and character bodies | 16 (8) | 16 (4) |
| Droids | 0 | 15 (4) |
| Heads of non-human characters | 6 | 8 (2) |
| Doors | 0 | 2 |
| Placeable objects | 2 | 3 |
| Minigame and vehicle models | 4 (1) | 1 |
| Test textures (`lte_`) | 5 (2) | 0 |
| Weapons | 0 | 0 |
| Total | 122 | 88 |

What I can say about each group.
- **Area surfaces.** 21 of the 89 in KOTOR 1 and 18 of the 43 in KOTOR 2 have `water`, `wtr`, `ocean`, `acid` or `waterfall` in their names. The others are walls, floors, trims and grates. In KOTOR 2 the prefixes show Dantooine (`dan_`), Dxun (`dxn_`), Telos (`tel_`), Onderon (`ond_`), Korriban (`kor_`), Nar Shaddaa (`nar_`) and the Ebon Hawk (`002_`). In KOTOR 1 the prefixes are three-letter codes such as `lda_`, `lka_`, `lma_`, `lta_` and `lts_`, and I did not check which place each code stands for.
- **Creatures and bodies.** In KOTOR 1 these are the Hutt (`c_hutt01` to `c_hutt04`), the Rakghoul, the Rancor, the Terentatek, the Rakata, the Selkath, a dewback, `lqa_mykal` and the Manaan water suit `lma_watersuit01`. In KOTOR 2 they are the Boma beast, the Mal beast, the Laigrek, the Rakghoul, the Rancor, the Gand, the Gran, the Quarren, the Sullustan, the Rakata and the Selkath.
- **Droids.** All 15 are in KOTOR 2. They are the textures of `c_mk1_drd` and `c_mk2_drd` (three each), `c_drdmouse01` to `c_drdmouse03d`, `c_drdmover01` and `c_drdmover02`, `c_condrdboss`, `c_condrdl`, `c_tankdroid` and `p_gt`. No KOTOR 1 droid names a bump map.
- **Heads.** All the heads belong to the Twi'lek family. They are `twilek_m01` to `twilek_m05`, `n_komadh`, `n_xorh` and `n_zharh`. All of them point to the same bump map `Twilek_M01b`, and in KOTOR 1 they point to the environment map `CM_SpecMap`.
- **Doors and placeable objects.** The two doors are `dxn_dr02` and `dxn_dr03` of KOTOR 2. The placeable objects are `plc_acidice` and `plc_water01` in both games, and `plc_fncom` in KOTOR 2.
- **Minigame and vehicle models.** In KOTOR 1 these are `mgf_turret01`, `mgg_ebonhawk01`, `v_ehawk01` and `lma_rfighter`. In KOTOR 2 it is `v_ehawk01`. I put `v_dshanty01` with the area surfaces, because in KOTOR 1 the room model `m18aa_03a` uses it.
- **Test textures.** Five textures named `lte_bm...` and `lte_embm...` are test pieces for diffuse bump, specular bump, and bump with a cube or a sphere map. They are in the game data of KOTOR 1. The models `c_bmspecdiff`, `c_embmcube` and `c_embmsphere` use three of them, and I did not look for other users.

### How I assigned the groups

1. I read every model in the game's archives and wrote down which textures each mesh names (the diffuse and the lightmap slot).
2. A texture was an area surface when a model that uses it is a room of a module's layout (`.lyt` file) or has a room name such as `m22aa_01a` or `601danj`.
3. A texture was a head when a model that uses it appears in the `head` column of `heads.2da`.
4. A texture was a droid when its name or its model's name contains `drd`, `droid` or `g0t0`. I used the name because `appearance.2da` has no column that says a model is a droid (its `racialtype` column holds 20 in every row, so it says nothing).
5. A texture was a door when its model starts with `dor_`.
6. A texture was a creature or body when its model is in the `race` column of `appearance.2da`, or when the model's class byte says it is a character (value 4).
7. A texture was a placeable when its model is in `placeables.2da`.
8. A texture with no model was assigned by its prefix: `l` plus two letters, `dan_`, `dxn_`, `tel_`, `ond_`, `kor_`, `nar_` and `002_` for areas, `c_`, `n_` and `p_` for creatures, `mgf`, `mgg` and `v_` for minigames and vehicles, and `lte_` for tests.

I am sure of the assignments that rest on a model (KOTOR 1: 65 area surfaces, 8 creatures, 6 heads, 3 minigame models, 3 tests and 2 placeables. KOTOR 2: 28 area surfaces, 12 creatures, 11 droids, 6 heads, 3 placeables, 2 doors and 1 minigame model). Those are tied to a concrete mesh and to a table of the game.
I am fairly sure of the assignments by name (KOTOR 1: 24 area surfaces, 8 creatures, 2 tests and 1 minigame model. KOTOR 2: 15 area surfaces, 4 droids, 4 creatures and 2 heads). The prefixes are consistent in the game, and each such texture points to a bump map that carries the same stem as its siblings that I could tie to a model. A few may belong to a different kind of model than I said. I corrected three by hand after reading the model lists: `lma_watersuit01` is a body, `lma_rfighter` is a ship, and `v_dshanty01` is an area surface although its prefix says vehicle.
Some creature textures are also used by a placeable object that shows the creature lying down (`c_malbeast01`, `c_mk1_drd` and `c_mk2_drd` in KOTOR 2, `n_rakata01` and `n_selkath01` in both games). I counted such a texture as a creature, a droid or a body.

## 4. Heads

The question was whether any human head texture names a bump map or an environment map.

- The stock games have 107 head models in `heads.2da` of KOTOR 1 and 151 in KOTOR 2. I took the textures that those models use, the texture names in the `headtex...` columns, and every texture whose name starts with `pmh`, `pfh` or follows the pattern `p_..h`.
- In KOTOR 1 none of the 151 textures named `pmh...` or `pfh...` names a bump map, an environment map or a bumpy shiny texture. Of the 230 head textures I found among the world textures, only 6 name any of the settings. They are `twilek_m01`, `twilek_m02`, `twilek_m03`, `n_komadh`, `n_xorh` and `n_zharh`. All six name `Twilek_M01b` as a bump map and `CM_SpecMap` as an environment map.
- In KOTOR 2 none of the 140 textures named `pmh...` or `pfh...` names any of them. Among the other head textures of the head tables, 8 name something. `twilek_m01` names the bump map `Twilek_M01b` and the environment map `CM_SpecMap`. `n_darkjh01` and `n_dkj2h01` name the environment map `CM_Baremetal`. `n_lala_xuh`, `n_sithassnh01`, `p_visash01`, `p_visashd01` and `p_visashd02` name the environment map `mycube`. I did not check which of these characters are human. Visas Marr, whose texture is `p_visash01`, is a Miraluka and not a human. The names of the others suggest Dark Jedi, a Sith assassin and Lala Xu. The other Twi'lek heads of KOTOR 2 (`twilek_m02` to `twilek_m05`, `n_komadh`, `n_xorh` and `n_zharh`) also name `Twilek_M01b`. KOTOR 2's `heads.2da` does not list them, so they were not in this scan.
- So the answer is no for the human heads of the player and of the stock characters that I could tie to a `pmh` or `pfh` name. The heads that use an environment map in the stock game are few, and the only head bump map is the Twi'lek one.


## 5. The bump map files

The 86 bump map files that KOTOR 1 names and the 52 that KOTOR 2 names are of these kinds.

| Kind of file | KOTOR 1 | KOTOR 2 |
|---|---|---|
| Normal map, stored as an uncompressed RGBA picture in the texture pack | 69 | 42 |
| Grey height map in the texture pack | 4 | 1 |
| Grey height map as a loose `.tga` in `textures.bif` (all the water bump maps checked) | 12 | 9 |
| Named but not found in any archive | 1 | 0 |

I opened every named file. The 69 and 42 normal maps all have a mean blue value of at least 220 out of 255, and a mean red and green value near 128. That is the colour of a flat tangent-space normal map. The grey ones have equal red, green and blue in every texel.

A sample of five, from the high quality pack.

| File | Game | Size | Kind | What I measured |
|---|---|---|---|---|
| `C_Rancor01b` | KOTOR 1 | 512 by 512 | Normal map | Mean colour (128, 128, 248). Red runs from 38 to 217, so the surface is tilted noticeably. |
| `LTA_Metal02b` | KOTOR 1 | 256 by 256 | Normal map | Mean colour (128, 128, 223). Red runs from 3 to 252, so this is a strong relief. |
| `Twilek_M01b` | Both | 256 by 256 | Normal map | Mean colour (128, 128, 254). Red runs from 38 to 217. |
| `002_EBO_Main1B` | KOTOR 2 | 1024 by 1024 | Normal map | Mean colour (128, 128, 254). Red runs from 71 to 184, a gentle relief. |
| `LDA_water01b` | KOTOR 1 | 256 by 256 | Grey height map | Every texel has equal red, green and blue, mean 122. It is a `.tga` in `textures.bif`. |

I also measured how strong the 111 normal maps are. For each map I took the mean distance of the red and green values from 128. The median is 6.1 for KOTOR 1 and 7.3 for KOTOR 2, on a scale where 127 would be the steepest tilt. So the typical bump map is gentle. 6 of the 69 in KOTOR 1 and 1 of the 42 in KOTOR 2 are flat or nearly flat (a distance below 1), among them `lda_nobump`, which is a flat 8 by 8 picture.
The strongest are `LUN_wll05b` in KOTOR 1 (59.5) and `PLC_Acidice2B` in KOTOR 2 (44.2).

## 6. The texture packs

The packs `swpc_tex_tpa.erf`, `swpc_tex_tpb.erf` and `swpc_tex_tpc.erf` are the high, medium and low quality packs (kjk's `TextureQuality` picks them in that order, `crates/vfs/src/resolver.rs:99-110`).

| Pack | KOTOR 1 size | KOTOR 2 size | Textures K1 and K2 | Bump maps K1 and K2 | Bumpy shiny K1 and K2 | Environment maps K1 and K2 |
|---|---|---|---|---|---|---|
| `tpa` (high) | 400 MB | 463 MB | 3,294 and 3,286 | 120 and 76 | 107 and 80 | 338 and 477 |
| `tpb` (medium) | 116 MB | 127 MB | 3,294 and 3,286 | 120 and 76 | 107 and 80 | 338 and 477 |
| `tpc` (low) | 45 MB | 43 MB | 3,294 and 3,286 | 120 and 76 | 107 and 80 | 338 and 477 |

- The three packs of each game hold exactly the same names. I compared the TXI text of every texture between `tpa` and `tpb`, and between `tpa` and `tpc`, and found no difference in either game.
- The bump maps are in the packs too, at the pack's size. `C_Rancor01b` is 512 by 512 in `tpa`, 256 by 256 in `tpb` and 128 by 128 in `tpc`. `Twilek_M01b` goes from 256 to 128 to 64. The low pack's bump maps are also flatter. The red channel of `C_Rancor01b` runs from 38 to 217 in `tpa` and from 100 to 157 in `tpc`.
- The water bump maps are not in the packs. They are the loose pictures in `textures.bif`, and they are the same at every quality.

## 7. How visible the gap is

This is my estimate from numbers. I have not looked at the original game with and without its bump pass, so none of it is shown by a picture.

- **Characters and heads.** In KOTOR 1, 16 creature and body textures and 6 Twi'lek head textures name a bump map. In KOTOR 2 it is 16 creature and body textures, 15 droid textures and 8 Twi'lek head textures. No human head and no weapon names one. A player sees these in the Rancor, the Rakghoul, the Selkath, the Mark droids and the Twi'leks. The 8 Twi'lek heads of KOTOR 2 share one bump map.
- **Areas.** In KOTOR 1, 410 of the 1,219 room models (34%) that I could read use a bump-mapped texture, and they are spread over 66 of the 123 module layouts. In KOTOR 2 it is 71 of 1,236 (6%) in 20 of 86 layouts. In KOTOR 1 the module codes include `m02`, `m17`, `m22`, `m26`, `m41` and `m45`. I did not check the places that these codes stand for. Walls and floors there carry a bump map together with an environment map, and kjk draws the environment map already.
- **Water.** 21 KOTOR 1 and 18 KOTOR 2 surfaces have water, ocean, waterfall or acid in their names. Their bump map is a grey height map. I did not check what the original does with it. kjk does not use these maps at all. Whether kjk's water already looks right is a separate question that this note does not answer.
- **Lighting.** The previous note found that kjk lights a surface with a flat ambient light under the original lighting, and that a normal map shows nothing under flat light. It also found that the original draws its bump pass only when the graphics hardware supports it. So the visible effect of a bump map depends on how many directional lights fall on the surface. I did not measure that.

My judgement is that the gap is small in the number of textures (3.6% in KOTOR 1 and 2.3% in KOTOR 2) and small for characters. It is moderate for KOTOR 1, where a third of the room models carry one. It is small for KOTOR 2, where the bump-mapped rooms are in 20 of the 86 layouts, among them the Ebon Hawk, Telos, Dxun, Nar Shaddaa, Onderon, Korriban and Dantooine.

## How I counted

1. The games are installed on this Mac under Steam. KOTOR 1 is in `~/Library/Application Support/Steam/steamapps/common/swkotor/Knights of the Old Republic.app/Contents/Assets` (Steam build 4143502), and KOTOR 2 is in `.../Knights of the Old Republic II/KOTOR2.app/Contents/GameData`. I only read from these folders.
2. I wrote a small Rust program outside the repository, in `/private/tmp/claude-501/-Users-david-perso-kjk/c3910254-b661-450a-9003-645d2158b8cb/scratchpad/bump-census/tool`. It depends on `crates/fmt_odyssey` by path and uses its `Key`, `Bif`, `Erf`, `Rim`, `Texture::parse_tpc`, `Txi`, `Model::parse`, `Layout::parse` and `TwoDa` readers. It does not use `vfs`.
3. The program reads `chitin.key` and every BIF it names, every ERF and RIM in `TexturePacks`, `modules` and `rims`, `Patch.erf` and the `override` folder. It parses every `.tpc` fully and reads its TXI, reads every loose `.txi`, parses every model for its texture names, every layout for its rooms, and every `.2da`.
4. The program's output is in `.../bump-census/out/k1` and `.../bump-census/out/k2` (`res.tsv`, `txi.tsv`, `mdl.tsv`, `lyt.tsv`, `2da/`), and the Python scripts that make the counts and the groups are in `.../bump-census/scripts`. Game files were not copied, since the program reads them in place.
5. I checked the reader with a plain byte scan. I searched the whole `swpc_tex_tpa.erf` file of each game for the words `bumpmaptexture`, `bumpyshinytexture` and `envmaptexture`, case-insensitive, without parsing anything. The scan found 120, 107 and 338 hits in KOTOR 1 and 76, 80 and 477 in KOTOR 2. Those are exactly the numbers that the reader found in the pack's textures, so each such line is counted once and none was missed.
6. The counts of "bump" use the effective TXI, a loose `.txi` first and otherwise the TXI at the end of the `.tpc`. In KOTOR 1 three loose TXI files in `textures.bif` (`lsf_glow...`) name a bumpy shiny texture. The pack's own TPC has the same line, so the counts do not change.
7. To read a bump map I decoded the first mip level of each named file as RGBA and measured the mean colour, the share of texels with equal red, green and blue, and the mean distance from 128.

## What I could not find out

- I did not check the Windows PC. It did not answer on its address, and the files were here. The installs on this Mac are the Aspyr ports. Their pack names (`swpc_tex_...`) and their TXI text look like the PC data, but I did not compare them with the PC retail files, for example by file size or hash. The Mac KOTOR 1 has extra BIF files (`items`, `party`, `player` and `_newbif`) and a `Patch.erf`, so it may differ from the PC in small ways. The counts could differ by a few textures.
- I did not read the texture archives of the Xbox or the mobile ports.
- I counted textures, not what is drawn. How many bump-mapped surfaces a player sees in an hour of play depends on which areas they visit, and I did not tally it with the scene tally near `crates/game/src/scene.rs:1371`. That tally needs the game running.
- The room count (410 of 1,219 and 71 of 1,236) comes from the room models listed in the layout files. A bump-mapped texture on a model of a room is counted even if only a small mesh of the room uses it.
- Droids are assigned by name, since no table marks them.
- The species of five KOTOR 2 head textures that name an environment map (`n_darkjh01`, `n_dkj2h01`, `n_lala_xuh`, `n_sithassnh01` and `p_visash01`) is not checked.
- I did not check the direction of the green channel of the normal maps, so I cannot say whether they follow the DirectX or the OpenGL convention.
- I did not check how the Aspyr Mac build or the Windows build draws bump maps. The statement that the original draws a bump pass only on special hardware is quoted from `2026-11-20-head-detail-maps.md` and is not re-checked here.
- Models that kjk would draw from the Jade Empire or other sources are not part of this.
