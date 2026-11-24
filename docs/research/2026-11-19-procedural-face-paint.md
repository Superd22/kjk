# Procedural face paint

This note was researched on 2026-11-19 for kjk's head paint track, which is part C of the roadmap (`2026-11-19-head-roadmap.md`).
It asks how games and research make a whole face's colour from a few settings, which colour models are learned from data, and how shipped character creators build their skin layers.
It then says which of those techniques suit kjk's setup: a mesh with per-vertex positions, procedural layers computed on the 3D surface, and one baked texture, all in Rust.

Terms used in this note.
- **Albedo** is the colour of a surface with no lighting in it. A photo has lighting in it. An albedo map does not.
- **Diffuse albedo** is the colour of light that enters the skin and comes back out. **Specular albedo** is the strength of the shiny reflection off the skin surface.
- **Melanin** is the brown pigment of the skin. It has two kinds. **Eumelanin** is dark brown and black. **Pheomelanin** is yellow and red-brown.
- **Haemoglobin** (also spelled hemoglobin) is the red pigment of blood. **Chromophores** are the pigments that give skin its colour, which are melanin and haemoglobin.
- **A tint mask** is a greyscale image, laid over the face, that says where a colour is applied and how strongly.
- **A morphable model** is a statistical model made by running principal component analysis (PCA) on many aligned scans. It describes a face as a mean plus a weighted sum of components. The weights are the model's parameters.
- **A UV layout** is the map from the 3D surface to a flat texture.
- **FLAME** is the head model our customiser is built on.
- **CIELAB** is a colour space where L\* is lightness, a\* is redness (green to red) and b\* is yellowness (blue to yellow). Distances in it roughly follow what the eye sees.

Every claim carries a link.
Where I could not open a page myself and only saw a search result's summary of it, I say so.
Modding wikis, mod pages and forum posts are marked as such.
A sentence with "I infer" or "probably" is my own reasoning and not a fact from a source.

## Summary

- **Two ways to get a face's colour from a few settings.** One way is a stack of masked colour layers, which is what the Bethesda games, Cyberpunk 2077, The Sims 4, Reallusion SkinGen and most others do. The other way is to compute the skin colour from two pigment amounts, melanin and haemoglobin, which is what Baldur's Gate 3 does, and what the research models of Jimenez and of Alotaibi and Smith do. Many games use both. The pigment way gives the base tone and the blush. The mask way gives makeup, tattoos and freckles.
- **The pigment model is small enough to implement.** Jimenez and colleagues compute a skin colour from four numbers, bake all colours into a 2D lookup table, and then store only two maps (melanin and haemoglobin) per face ([Jimenez et al., SIGGRAPH Asia 2010](https://www.iryoku.com/skincolor/downloads/A-Practical-Appearance-Model-for-Dynamic-Facial-Color.pdf)). Alotaibi and Smith print the whole spectral formula, with its constants, in one paper ([Alotaibi and Smith, ICCV Workshops 2017](https://openaccess.thecvf.com/content_ICCV_2017_workshops/papers/w16/Alotaibi_A_Biophysical_3D_ICCV_2017_paper.pdf)).
- **No learned colour model can ship in the mod, and none suits KOTOR's look.** AlbedoMM is academic-only. The Basel Face Model and the FLAME texture spaces are non-commercial. FFHQ-UV has an MIT-licensed code repository, but its texture maps come from photographs of real people, and I could not confirm the terms for the dataset itself. All of them describe real photographic skin, not KOTOR's hand-painted textures. They are useful as analysis tools and as a source of numbers, not as shipped data.
- **Real skin-colour numbers exist.** One measured study gives the mean lightness, redness and yellowness of the forehead and cheek for four ethnic groups, with standard deviations ([Xiao et al., 2017](https://eprints.whiterose.ac.uk/id/eprint/100718/7/Xiao_et_al-2017-Skin_Research_and_Technology.pdf)). The study covers no dark African skin, so it sets only the light and medium end.
- **Shipped creators almost never publish how they blend.** Epic, EA and Reallusion say what the player controls. Only Baldur's Gate 3 and the Bethesda games have the layer structure documented, and only by modding wikis and mod pages. I found no talk or paper for the Sims 4, Black Desert, Cyberpunk 2077, Dragon's Dogma 2 or Dragon Age: The Veilguard texture pipelines.
- **Recommendation.** Build the base tone from two pigment numbers per vertex through a lookup table. Put regional tints on the pigment maps as multipliers. Put lips, makeup and decals on top as masked colour layers. Put pores and age detail on top as a separate noise layer. The last section gives the reasons.

## How a whole face's colour is made from a few settings

### The base skin tone

**A colour swatch with two coordinates.** MetaHuman Creator sets skin tone from a colour swatch, and two sliders (U for horizontal, V for vertical) place a picker on it ([Epic, MetaHuman skin material tools](https://dev.epicgames.com/documentation/metahuman/skin-material-tools?lang=en-US)). Epic's texture synthesis class can turn a colour into a "Skin UV" value, and it clamps the value to the range from 0 to 1 when the colour falls outside what the model can show ([Epic API page for FMetaHumanFaceTextureSynthesizer](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/MetaHumanCoreTechLib/FMetaHumanFaceTextureSynthesizer); I could not open this page, so this comes from a search result's summary of it). The swatch therefore looks like a 2D parameter of a learned colour space. The texture synthesis runs on a cloud service, so it cannot be copied ([Epic, MetaHuman 5.6 release notes, as quoted in a search result](https://forums.unrealengine.com/t/metahuman-5-6-release/2539165)).

**Hue, undertone and melanin.** Dragon Age: The Veilguard offers a cool, neutral or warm skin hue, separate undertones, a melanin slider, a vitiligo slider and 40 complexions ([Game Informer](https://gameinformer.com/exclusive/2024/06/13/a-deep-dive-into-dragon-age-the-veilguards-expansive-character-creator)). The game director said the cool, neutral and warm split copies how makeup companies sort shades (reported in a [search result summary of an interview](https://www.gamesradar.com/games/dragon-age/dragon-age-the-veilguard-director-wants-the-rpgs-character-creator-to-be-a-make-good-full-disclosure-dragon-age-has-traditionally-not-done-skin-tones-well); I did not open the interview). I found no source for how these controls become a texture.

**Two pigment maps through a lookup table.** Jimenez and colleagues store a melanin map and a haemoglobin map. They bake the skin colour for every pair of pigment amounts into one 2D lookup table and read the colour from it ([Jimenez et al. 2010](https://www.iryoku.com/skincolor/downloads/A-Practical-Appearance-Model-for-Dynamic-Facial-Color.pdf), section 6). The method is written out in "The pigment-to-colour mapping" below.

**Baldur's Gate 3 blends four maps.** Larian's lead character artist describes a skin shader that blends four maps, named Hemoglobin, Melanin, Vein Map and Yellowing Map. She says this makes fair and deeper skin tones easier to adjust than the older multiply approach ([80.lv, "Creating Characters for Baldur's Gate 3"](https://80.lv/articles/creating-characters-for-baldur-s-gate-3/)). The modding wiki says the four maps are the red, green, blue and alpha channels of one texture called HMVY, stored as BC3 ([bg3.wiki, modding wiki](https://bg3.wiki/wiki/Modding:Face_and_body_textures)). The same wiki calls the green channel "melanin (freckles)" on one page and "pigment amount" on another, so the channel's meaning is not settled ([bg3.wiki, modding wiki](https://bg3.wiki/wiki/Modding:Face_and_body_textures)).

**What real faces measure.** Xiao and colleagues measured skin with a spectrophotometer on 960 people (187 Caucasian, 202 Chinese, 145 Kurdish and 426 Thai) and gave CIELAB means with standard deviations in brackets ([Xiao et al. 2017](https://eprints.whiterose.ac.uk/id/eprint/100718/7/Xiao_et_al-2017-Skin_Research_and_Technology.pdf), Tables 1 and 2).

| Group, spot | L\* | a\* | b\* |
|---|---|---|---|
| Caucasian, forehead | 59.2 (5.1) | 11.6 (2.8) | 15.1 (2.3) |
| Caucasian, cheek | 59.6 (5.5) | 11.8 (3.1) | 14.6 (2.6) |
| Chinese, forehead | 56.4 (3.2) | 11.7 (2.1) | 16.3 (1.4) |
| Chinese, cheek | 58.9 (3.1) | 11.4 (2.1) | 14.2 (1.5) |
| Kurdish, forehead | 56.1 (4.5) | 11.3 (2.1) | 16.4 (2.2) |
| Kurdish, cheek | 58.0 (4.4) | 11.7 (2.3) | 15.8 (2.1) |
| Thai, forehead | 56.8 (4.1) | 11.6 (2.2) | 17.7 (1.8) |
| Thai, cheek | 60.7 (4.0) | 10.5 (2.3) | 17.2 (2.1) |

The Caucasian group spans a\* from about 2 to 20 and b\* from about 6 to 25. The Chinese group spans a\* from about 3 to 18 and b\* from about 9 to 21 (same paper). The study found that differences in lightness and yellowness come mostly from ethnicity, and differences in redness come mostly from the body location (same paper). The four groups do not include dark African skin, so these numbers do not set the dark end of a slider.

### Regional tints

**Redness differs by place, in measured data.** In the same study the forehead and cheek have a\* of about 11 to 12 in every group, while the back of the hand has 7 to 9 and the inner arm has 5.6 to 7.1 (Table 2 of [Xiao et al. 2017](https://eprints.whiterose.ac.uk/id/eprint/100718/7/Xiao_et_al-2017-Skin_Research_and_Technology.pdf)). The facial skin is also darker than the inner arm in every group, which the authors link to sun exposure (same paper). I infer that a face that is painted from one base colour needs an extra redness and darkness step on the face's own regions to look like a face.

**MetaHuman has eight symmetric accent regions.** Its Accents tab applies redness and saturation to named regions of the face, and the changes are always symmetric between left and right ([Epic, skin material controls](https://dev.epicgames.com/documentation/metahuman/skin-material-tools?lang=en-US)). Epic's enum for the regions is Scalp, Forehead, Nose, UnderEye, Cheeks, Lips, Chin and Ears ([Epic API page for EMetaHumanCharacterAccentRegion, as quoted in a search result](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/MetaHumanCharacterEditor/EMetaHumanCharacterAccentRegion)). The doc says accents give effects such as blushing, a tan or burn, and dark circles under the eyes ([Epic](https://dev.epicgames.com/documentation/metahuman/skin-material-tools?lang=en-US)). A lightness control was named in my summary of the doc, but a second search did not confirm it.

**Skyrim has 14 numbered tint types.** The race record lists a mask type for each tint: 1 Lip Color, 2 Cheek Color, 3 Eyeliner, 4 EyeSocket Upper, 5 EyeSocket Lower, 6 Skin Tone, 7 Paint, 8 Laugh Lines, 9 Cheek Color Lower, 10 Nose, 11 Chin, 12 Neck, 13 Forehead and 14 Dirt. Each tint has a path to a mask file, a list of colour presets, and a default value from 0 to 1 ([UESP, Skyrim race record format, modding wiki](https://en.uesp.net/wiki/Skyrim_Mod:Mod_File_Format/RACE)). This list is the closest thing I found to a list of the regional tints that a shipped creator uses.

**The same pattern exists in Fallout 4.** The Creation Kit lets each face tint layer take a colour (a preset or an RGB value) and an interpolation value ([CK wiki, Character Gen Parts tab, as quoted in a search result](https://ck.uesp.net/wiki/Character_Gen_Parts_Tab); I could not open this page). The LooksMenu tint API, a mod, lists four blend modes: Multiply, Overlay, SoftLight and HardLight ([LooksMenu tint API page, mod page](https://www.nexusmods.com/fallout4/mods/30163); I could not open it, so this comes from a search result's summary). A mod guide says tint masks are BC1 images where white marks the affected area ([mod guide, quoted in a search result](https://www.nexusmods.com/fallout4/mods/30163?tab=posts)).

**Haemoglobin can be edited by a scale and a bias.** Jimenez and colleagues treat the blood distribution of a region as a Gaussian with a mean and a standard deviation. They move one region's distribution onto another with a scale and a bias (equation 3 of [Jimenez et al. 2010](https://www.iryoku.com/skincolor/downloads/A-Practical-Appearance-Model-for-Dynamic-Facial-Color.pdf), written out below). Their maps stay detailed because only the low-frequency scale and bias change. Their scale and bias maps could be reduced to 256 by 256 pixels without visible loss (same paper). They scaled the whole face's haemoglobin by 127% for an angry face and by 81% for a frightened face (same paper).

**Blue-grey beard shadow and eye socket darkening.** I found no primary source that measures or models these as colour. The Skyrim list above has EyeSocket Upper and Lower tints, and the BG3 wiki lists "lip lines and tear lines" in its MSK texture ([bg3.wiki, modding wiki](https://bg3.wiki/wiki/Modding:Face_and_body_textures)). The painted-facial-hair note covers stubble already (`2026-11-18-painted-facial-hair.md`).

### Lips

- **Lips are a mask channel.** The Baldur's Gate 3 CLEA texture keeps the lips in its blue channel, next to cavities and pores in red, eyebrows in green and ambient occlusion in alpha ([bg3.wiki, modding wiki](https://bg3.wiki/wiki/Modding:Face_and_body_textures)). The game's lip colour is a tint on that mask. This is my reading of the wiki and not a stated fact.
- **Lip colour is a tint with a preset list.** Skyrim has one tint type for lips, and the LooksMenu doc's example entries use Overlay for the lip masks ([UESP](https://en.uesp.net/wiki/Skyrim_Mod:Mod_File_Format/RACE); [LooksMenu tint API page, mod page](https://www.nexusmods.com/fallout4/mods/30163), from a search result's summary).
- **Lipstick is a layer.** Reallusion SkinGen has a "Lip Makeup" layer on top of the skin layers ([Reallusion manual](https://manual.reallusion.com/character-creator-4/content/enu/4.0/15_skingen/Basic-Concepts/Skin-and-Makeup-Layers-Concepts.htm)). Veilguard has lips among its 30 or more makeup options ([Game Informer](https://gameinformer.com/exclusive/2024/06/13/a-deep-dive-into-dragon-age-the-veilguards-expansive-character-creator)).
- **I found no source on how any game softens or sharpens the lip border.** The roadmap's plan to use the lip border vertices of our own mesh has no published equivalent that I found.

### Fine detail such as pores, freckles, moles and blemishes

- **MetaHuman picks a scanned detail texture by index.** The Face Texture Index selects one of a flipbook of discrete textures that set wrinkle density and apparent age. A filter can narrow the choice by wrinkles, stubble, freckles and moles ([Epic](https://dev.epicgames.com/documentation/metahuman/skin-material-tools?lang=en-US)). The freckle pattern is chosen first. Then four sliders set it: Density, Strength (opacity), Saturation and Tone Shift ([Epic](https://dev.epicgames.com/documentation/metahuman/skin-material-tools?lang=en-US)). The Density slider changes how many freckles show and not the area they cover. I infer that a freckle pattern is a fixed mask, and the sliders threshold and tint it.
- **Baldur's Gate 3 keeps pores in a cavity channel.** The red channel of CLEA holds cavities and pores, and the NORMAL map holds wrinkles and scars ([bg3.wiki, modding wiki](https://bg3.wiki/wiki/Modding:Face_and_body_textures)). Larian's character shader has overlays for bruises, blood, sweat, dirt, makeup, tattoos and scars ([80.lv](https://80.lv/articles/creating-characters-for-baldur-s-gate-3/)).
- **Reallusion splits detail into layers.** The skin stack is Skin Base, Normal Effects (wrinkles, bumps, fine lines), Skin Details (capillaries, noise, coloration), Blemish (freckles, moles, acne, suntan), Acquired (wounds, scars, tattoos, liquid, dirt), Body Hair and Nails. Layers apply from the bottom to the top ([Reallusion manual](https://manual.reallusion.com/character-creator-4/content/enu/4.0/15_skingen/Basic-Concepts/Skin-and-Makeup-Layers-Concepts.htm)). The freckle layer of the head skin loads default patterns or a custom mask ([Reallusion manual](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_Substance/Head_Skin_Texture_Layers.htm)).
- **Procedural pore research exists, but I could not read it.** Haro, Guenter and Essa grew skin normal maps across the face with texture synthesis ([Haro et al., EGWR 2001](https://diglib.eg.org/items/1bfc433a-8dc1-4c05-b14b-7251363cfd25)). A Computer Graphics Forum paper makes micro-wrinkles from a graph whose nodes are pores and whose edges are candidate wrinkles ([CGF vol. 42 issue 5, from a search result's summary](https://diglib.eg.org/bitstream/handle/10.1111/cgf14904/v42i5_08_14904.pdf)). Both are about geometry and normals, not colour. I could not open either full text, so I have no formula to give. The kjk paint can lean on its own 3D noise here.

### Ageing and weathering

- **Evenness of colour tracks perceived age.** Matts and colleagues took cheek photographs of 170 women and girls aged 11 to 76 and had 353 people rate them. Colour evenness correlated with estimated age at r = -0.45. Evenness of the melanin map correlated at -0.32 and of the haemoglobin map at -0.38 ([Matts et al. 2007, abstract](https://www.em-consulte.com/en/article/438141)). A later study on men found melanin evenness was the stronger cue for age and haemoglobin distribution was more tied to health and attractiveness ([Fink et al. 2012](https://pubmed.ncbi.nlm.nih.gov/22044626/)). I infer that an age slider can raise the variance of the melanin and haemoglobin maps and lower their smoothness, and that this would match what viewers read as age.
- **Games mostly use authored textures for age.** MetaHuman's face texture index runs from young to old on a flipbook, and the 5.6 release dropped the age order, which users complained about ([Epic doc](https://dev.epicgames.com/documentation/metahuman/skin-material-tools?lang=en-US); [forum post](https://forums.unrealengine.com/t/metahuman-5-6-release/2539165)). Reallusion's Surface Normals layer sets the apparent age ([Reallusion manual](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_Substance/Head_Skin_Texture_Layers.htm)). Reallusion's Tan layer applies a suntan or sunburn ([same page](https://manual.reallusion.com/Character-Creator-4/Content/ENU/4.0/15_Substance/Head_Skin_Texture_Layers.htm)). The Sims 4 has a burn mask that adds pale areas around the eyes and neck ([Skininator, mod page](https://modthesims.info/d/568474)).
- **Older research applied pre-made wrinkle masks.** Boissieux and colleagues applied generic precomputed wrinkle masks as textures. A patent describes this approach as limited, because a population mean cannot predict one person's own features ([US patent 8,290,257, as quoted in a search result](https://patents.google.com/patent/US8290257)). The patent also describes age spots as hyperpigmented spots that get darker with age.

## Colour models learned from data

### Table of the models

| Model | Trained on | Parameters | Texture and layout | Licence | FLAME |
|---|---|---|---|---|---|
| Basel Face Model colour (2009, reused in 2017) | 200 scans from a structured light scanner ([Gerig et al. via AlbedoMM paper](https://arxiv.org/abs/2004.02711)) | Not stated in the pages I opened | One RGB colour per vertex, on a mesh of 53,490 vertices ([Alotaibi and Smith](https://openaccess.thecvf.com/content_ICCV_2017_workshops/papers/w16/Alotaibi_A_Biophysical_3D_ICCV_2017_paper.pdf)). No UV layout | Non-commercial research only. Commercial use through Unitectra ([Basel](https://faces.dmi.unibas.ch/bfm/bfm2017.html)) | Converted by script, see below |
| FLAME texture space converted from BFM | The BFM colour model | Not stated on the repo page | A UV texture in FLAME's UV layout. Resolution not stated | Must follow the BFM licence ([BFM_to_FLAME](https://github.com/TimoBolkart/BFM_to_FLAME)) | Yes, made for it |
| FLAME texture space from MPI | Not stated | Not stated | Not stated | A separate "Texture License", terms not shown ([FLAME site](https://flame.is.tue.mpg.de/)) | Yes |
| AlbedoMM | 73 scans (50 captured, 23 from 3DRFE), mirrored to 146 samples ([Smith et al. 2020](https://arxiv.org/abs/2004.02711)) | Not stated. I infer at most 145 | Per vertex, linear sRGB, diffuse and specular. No UV layout | "Academic research purposes" only ([README](https://github.com/waps101/AlbedoMM)) | Yes, a FLAME version exists |
| FFHQ-UV | 54,165 images made from FFHQ photographs ([Bai et al. 2023](https://arxiv.org/abs/2211.13874)) | A StyleGAN2 latent vector. The size is not stated in the text I read | 1024 by 1024 UV maps in the layout of HiFi3D++, a head model | Code is MIT ([LICENSE](https://github.com/csbhr/FFHQ-UV/blob/main/LICENSE)). The dataset terms are not stated | A README gives a way to use it on a FLAME mesh |
| UV-IDM | A new "BFM-UV" dataset | Latent diffusion model | BFM-based UV textures | Not checked | No, BFM |
| Relightify | A UV reflectance dataset rendered under varied light | Diffusion model | UV maps of diffuse, specular and normals | Not checked | Not stated |

### Basel Face Model colour

The Basel Face Model of 2009 was built from 200 scans, with texture from three synchronised cameras with diffused flashes ([Smith et al. 2020](https://arxiv.org/abs/2004.02711)). The 2017 version "used largely the same data" (same paper). The colour is a linear PCA on per-vertex RGB ([Alotaibi and Smith](https://openaccess.thecvf.com/content_ICCV_2017_workshops/papers/w16/Alotaibi_A_Biophysical_3D_ICCV_2017_paper.pdf)). The AlbedoMM authors say its albedo is not real diffuse albedo, because lighting is baked in, and that BFM and similar models are biased toward Caucasian faces ([Smith et al. 2020](https://arxiv.org/abs/2004.02711)). The licence is non-commercial, and a commercial licence needs a request to Unitectra ([Basel](https://faces.dmi.unibas.ch/bfm/bfm2017.html)).

DECA, a FLAME-based face reconstruction, uses the BFM-converted texture as an optional file named `FLAME_albedo_from_BFM.npz`, which the user must build with the BFM_to_FLAME script ([DECA](https://github.com/yfeng95/DECA)). A search result of a third-party copy of DECA's loader shows keys `MU` and `PC` and 200 components ([search result, secondary source](https://huggingface.co/spaces/RipleysNest/PSHuman2/blob/main/lib/pixielib/models/FLAME.py)). I did not open that copy.

### AlbedoMM

AlbedoMM is the only model I found that is built to give real diffuse and specular albedo and not lit colour ([Smith et al., CVPR 2020](https://arxiv.org/abs/2004.02711)). The details I found follow.
- **Data.** 50 people captured in a light stage, aged 18 to 67 and covering Fitzpatrick skin types I to V, plus 23 scans from the 3DRFE database. The capture was repeated three times, which gave 24 effective views.
- **Preparation.** The authors undid the gamma of 2.2, divided every map by the camera's ISO setting, and fitted one 3 by 3 colour matrix to match the 3DRFE scans. They masked 5% of the vertices as artefacts and filled them by statistical inpainting and Poisson blending.
- **Model.** A linear PCA, `x(b) = P b + mean`, for diffuse and for specular. The specular components are built from the same combinations of training samples as the diffuse ones, so one parameter vector drives both.
- **Drawing it.** The colour at a vertex is `[i_diff (P_diff b + mean_diff) + i_spec (P_spec b + mean_spec)] ^ (1/2.2)`. In this formula `i_diff` and `i_spec` are the diffuse and specular shading from the lighting.
- **Layout.** Per vertex on the BFM 2017 template, with a cropped inner-face version. A FLAME-topology version exists ([paper, figure 7](https://arxiv.org/abs/2004.02711); [README](https://github.com/waps101/AlbedoMM)). Missing data on the full head is extrapolated with zero gradient (same paper).
- **Licence.** "We give permission to use the model and code only for academic research purposes" ([README](https://github.com/waps101/AlbedoMM)).
- **Fit for kjk.** The licence rules it out for shipping. Its per-vertex colour is too coarse for a texture at mod resolution, as the roadmap's finding on per-vertex paint also showed. I infer that it could still serve as a test of our pigment fit, since it has real albedo and a FLAME version.

### FFHQ-UV and the FLAME texture spaces

FFHQ-UV starts from the 70,000 FFHQ portraits. It edits each face with StyleGAN to a neutral expression and even light, extracts a UV texture from several views, and fills errors and missing parts (eyeballs, mouth interior, nostrils, ears, neck) from a template texture with Poisson editing ([Bai et al. 2023](https://arxiv.org/abs/2211.13874)). After manual filtering 54,165 maps remain, at 1024 by 1024. The authors train a StyleGAN2 texture decoder on them, and fit its latent code to a photograph in a three-stage optimisation ([same paper](https://arxiv.org/abs/2211.13874)). The paper says the decoder expresses more skin tones than earlier models.

The UV layout belongs to HiFi3D++ and not to FLAME. The README only says a solution for FLAME exists ([FFHQ-UV README](https://github.com/csbhr/FFHQ-UV)). The FFHQ photographs carry five per-image licences, and several are non-commercial ([FFHQ dataset page](https://github.com/NVlabs/ffhq-dataset)). I infer that the UV maps inherit those limits, but I found no statement from the authors on it.

The MPI FLAME site offers a "FLAME texture space" for download and says AlbedoMM is also available as a FLAME texture model ([FLAME site](https://flame.is.tue.mpg.de/)). It gives no component count or resolution on that page. A search result shows that the TF_FLAME fitting code loads `texture_data_512.npy` and has files for 256, 1024 and 2048 ([search result for TF_FLAME](https://github.com/TauGames/TF_FLAME); I did not open the code).

### Newer neural and diffusion models

- **UV-IDM (CVPR 2024)** is an identity-conditioned latent diffusion model that makes UV textures for BFM-based reconstruction. It was trained on a new BFM-UV dataset made from FFHQ-UV's multi-view data ([Li et al. 2024](https://openaccess.thecvf.com/content/CVPR2024/papers/Li_UV-IDM_Identity-Conditioned_Latent_Diffusion_Model_for_Face_UV-Texture_Generation_CVPR_2024_paper.pdf)).
- **Relightify (ICCV 2023)** fits a 3D morphable model to a photograph, takes the visible part of the texture, and fills the rest with a diffusion model trained on UV reflectance data. It outputs diffuse and specular albedo and high-frequency normals ([Papantoniou et al. 2023](https://arxiv.org/abs/2305.06077v1), from a search result's summary).
- **A semantic GAN trained on 44,000 diffusion-made faces** gives geometry and albedo from semantic attributes, with continuous editing in latent space. The abstract says code will be released with the paper, and no link is on the page ([Cai et al., arXiv 2504.15259](https://arxiv.org/abs/2504.15259)).
- **FaceRiff (SIGGRAPH Asia 2026)** adapts SDXL with low-rank adapters, using 2,049 scans from about 350 identities, and gives a mesh, albedo and displacement in one UV space from text. The paper is under CC BY-NC-ND 4.0 and no code is linked ([arXiv 2611.21867](https://arxiv.org/abs/2611.21867)).
- **FreeUV (CVPR 2025)** recovers a UV texture from one photo with a pre-trained Stable Diffusion model and no ground-truth textures. A search result says its figures use FLAME-based renderings ([FreeUV](https://arxiv.org/pdf/2503.17197)).

None of these gives a few settings a player could understand. They take a photograph, text or a latent code. I infer that none would reproduce KOTOR's painted look without retraining on KOTOR-style data.

### MetaHuman's texture synthesis

MetaHuman's face albedo comes from a texture synthesizer that takes a skin UV and a high-frequency map index. It writes an albedo texture into a pre-allocated image, and normal and cavity maps are assigned directly ([Epic API pages, from search results' summaries](https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/MetaHumanCoreTechLib/FMetaHumanFaceTextureSynthesizer?lang=en-US)). The creator then bakes skin, accents, makeup and freckles into one base colour texture. Its detail maps hold only differences from that base ([Epic, materials and textures](https://dev.epicgames.com/documentation/metahuman/metahuman-materials-and-textures?lang=en-US)). The synthesis needs a cloud connection ([forum post quoting the release notes](https://forums.unrealengine.com/t/metahuman-5-6-release/2539165)). Epic does not say what data trained it.

## Physically based skin colour from pigments

### The pigment-to-colour mapping

All the models below use the same idea. Skin has a thin top layer, the epidermis, that holds melanin. Under it is the dermis, which holds blood. Light is absorbed in the epidermis, scattered in the dermis, and comes back out. The colour you see depends on how much melanin and blood there are.

**The Donner and Jensen model.** Donner and Jensen's spectral model has four controls, which are the amounts of oil, melanin and haemoglobin, and it matched African, Asian and Caucasian skin ([Donner and Jensen 2006](https://cseweb.ucsd.edu/~henrik/papers/skin_bssrdf/)). Jimenez and colleagues use the later layered version by Donner and colleagues ([Donner et al. 2008](https://gfx.cs.princeton.edu/pubs/Donner_2008_ALH/donner08hskin.pdf)). I opened only the abstract-level pages of those, not the formulas.

**Jimenez's parameters and ranges.** Table 1 of their paper gives four parameters ([Jimenez et al. 2010](https://www.iryoku.com/skincolor/downloads/A-Practical-Appearance-Model-for-Dynamic-Facial-Color.pdf)).

| Parameter | Meaning | Range |
|---|---|---|
| Cm | Melanin fraction | 0 to 0.5 |
| βm | Melanin type blend (eumelanin to pheomelanin) | 0 to 1 |
| Che | Haemoglobin fraction in the epidermis | 0 to 0.1 |
| Chd | Haemoglobin fraction in the dermis | 0 to 0.32 |

Their lookup table sets Che to 0.25 of Chd. It uses 61% eumelanin and 39% pheomelanin. It is indexed by `u = cbrt(Cm)` and `v = cbrt(Chd)`, which spreads the samples evenly in perception, because the spectral model is strongly non-linear. The ticks on their figure 9 are 0, 0.002, 0.0135, 0.0425, 0.1, 0.185, 0.32 and 0.5 for melanin, and 0, 0.003, 0.02, 0.07, 0.16 and 0.32 for haemoglobin ([same paper](https://www.iryoku.com/skincolor/downloads/A-Practical-Appearance-Model-for-Dynamic-Facial-Color.pdf)). Each table entry is the total diffuse reflectance of their two-layer model, which they precompute ([same paper](https://www.iryoku.com/skincolor/downloads/A-Practical-Appearance-Model-for-Dynamic-Facial-Color.pdf)). The authors also released their neutral melanin and haemoglobin maps and the lookup table through the ACM portal ([same paper](https://www.iryoku.com/skincolor/)). I did not download them.

**Alotaibi and Smith's full formula.** Their paper prints every constant ([Alotaibi and Smith 2017](https://openaccess.thecvf.com/content_ICCV_2017_workshops/papers/w16/Alotaibi_A_Biophysical_3D_ICCV_2017_paper.pdf)). The method is as follows.
1. The only free parameters are the blood volume fraction `f_blood` (2% to 7%) and the melanosome volume fraction `f_mel` (1% to 43%).
2. Fixed values are eumelanin concentration 80 g/L, pheomelanin concentration 12 g/L, eumelanin blend 61%, haemoglobin concentration 150 g/L, oxygenated fraction 75%, epidermis thickness 0.021 cm and papillary dermis thickness 0.2 cm.
3. For each wavelength λ, the epidermis transmits `T = exp(-mu_a_epidermis)`. Here `mu_a_epidermis = f_mel * mu_a_mel + (1 - f_mel) * mu_baseline`, and `mu_a_mel` mixes eumelanin and pheomelanin by the blend. Eumelanin's extinction is `6.6e11 * λ^-3.33` and pheomelanin's is `2.9e10 * λ^-4.75`. The baseline absorption is `0.244 + 85.3 * exp(-(λ - 154) / 66.2)`.
4. The dermis reflects light by the Kubelka-Munk formula. Its absorption is `f_blood * mu_a_blood + (1 - f_blood) * mu_baseline`. Its scattering is a Mie term `2e5 * λ^-1.5` plus a Rayleigh term `2e12 * λ^-4`. Blood absorption mixes oxygenated and deoxygenated haemoglobin by the 75% ratio, using measured extinction tables.
5. The spectrum becomes a colour by integrating over the light source and the camera's sensitivity, as `i_C = integral of E(λ) S_C(λ) R(λ) dλ`.
Their figure shows total reflectance for melanin of 2%, 6% and 10% at 5% blood, and for blood of 2%, 3.9% and 5% at 2% melanin. The measured haemoglobin extinction tables are cited from another source, and I did not download them, so I cannot say if they are free to use.

**A hybrid model that stays on the skin colour curve.** The same paper notes that skin colour lies on a curved surface in RGB, so a linear model produces impossible colours. They run PCA on the pigment parameters instead. The vector per vertex is `[f_blood, f_mel, dR, dG, dB]`, where the last three are RGB offsets for things that are not skin. The albedo is `A = biophysical(f_mel, f_blood) + [dR, dG, dB]` ([same paper](https://openaccess.thecvf.com/content_ICCV_2017_workshops/papers/w16/Alotaibi_A_Biophysical_3D_ICCV_2017_paper.pdf)). Scaling the haemoglobin map by 2 gave a flushed face. Scaling the melanin map by 1.5 gave a suntanned face (same paper).

**Separating the pigments from a photograph.** Tsumura and colleagues state three assumptions. Two pigments cause the colour variation. Their amounts are spatially independent. The colour signals and the pigment amounts are linearly related in the optical density domain (the negative logarithm of the reflectance). They separate the two with independent component analysis, and they re-synthesise faces by changing the two amounts ([Tsumura et al., JOSA A 1999, abstract](https://www.mi.tj.chiba-u.jp/~tsumura/Tsumura/papers/JOSA99_ICA.htm)). I read only the abstract page.

**Games that use it.** Baldur's Gate 3 does, by its haemoglobin, melanin, vein and yellowing maps ([80.lv](https://80.lv/articles/creating-characters-for-baldur-s-gate-3/)). The Veilguard has a melanin slider ([Game Informer](https://gameinformer.com/exclusive/2024/06/13/a-deep-dive-into-dragon-age-the-veilguards-expansive-character-creator)), but I found no source that says it is a biophysical model. I found no game that documents a spectral lookup table like Jimenez's. I also found no Unreal or Unity sample that ships one.

## How shipped creators build their skin

For each game, the table says what the player controls, how that becomes texture, and how good my source is.

| Game or tool | The player controls | How it becomes texture | Source quality |
|---|---|---|---|
| MetaHuman Creator (Unreal 5.6 and later) | Skin tone swatch (U, V), face texture index or a filter, roughness, freckle pattern with 4 sliders, accent regions with redness and saturation | Cloud texture synthesis gives an albedo from the skin UV and a detail index. Everything is baked into one base colour texture on export. Wrinkle maps are stored as differences | Epic's own docs ([skin tools](https://dev.epicgames.com/documentation/metahuman/skin-material-tools?lang=en-US), [materials](https://dev.epicgames.com/documentation/metahuman/metahuman-materials-and-textures?lang=en-US)). The synthesis internals are not published |
| Reallusion Character Creator 4 with SkinGen | A stack of skin layers (Base, Normal Effects, Skin Details, Blemish, Acquired, Body Hair, Nails) and makeup layers (Foundation, Eye, Eyelash, Lip, Eyebrow, Misc) | Layers overlap from the bottom to the top. "Bake All Skin Textures" gives a texture from 256 to 4096 pixels | Official manual ([layers](https://manual.reallusion.com/character-creator-4/content/enu/4.0/15_skingen/Basic-Concepts/Skin-and-Makeup-Layers-Concepts.htm), [baking](https://manual.reallusion.com/character-creator-4/content/enu/4.0/15_SkinGen/Basic-Concepts/Baking-All-Skin-Textures.htm), from a search result's summary). The blend maths is not documented |
| Baldur's Gate 3 | Presets for faces plus a skin colour, freckles, makeup, tattoos, scars and similar overlays | A shader blends haemoglobin, melanin, vein and yellowing maps. Masks multiply custom colours (an ID mask holds 11 colour parameters per ID colour, an RGB mask holds 3). Overlays go on top | Larian's artist in [80.lv](https://80.lv/articles/creating-characters-for-baldur-s-gate-3/). The channel layout comes from a [modding wiki](https://bg3.wiki/wiki/Modding:Face_and_body_textures) |
| Skyrim and Fallout 4 | Per tint type a colour from a preset list and a value from 0 to 1, plus RaceMenu and LooksMenu extras | A tint mask file per type, a colour, and a blend (Multiply, Overlay, SoftLight or HardLight in LooksMenu) | [Modding wiki](https://en.uesp.net/wiki/Skyrim_Mod:Mod_File_Format/RACE), [mod page](https://www.nexusmods.com/fallout4/mods/30163). The engine's own formula is not documented anywhere I found |
| The Sims 4 | A skin tone, with overlays for details and makeup | A TONE file points to one base colour texture shared by all ages and genders, a body texture of shading and detail per age, gender and body type, and optional overlays. A burn mask adds pale areas | One [mod page](https://modthesims.info/d/568474). I found no EA source |
| EA Sports FC 25 (Cranium) | Complexion, scars and other texture controls, and FB Morph sliders for the shape | EA says the texture features "are generated in the runtime" by a "Cryo Texture Compositor" | EA's own [announcement](https://www.ea.com/news/cranium-technology-ea-sports-fc-25). It gives no blend details |
| Dragon Age: The Veilguard | 40 complexions, hue (cool, neutral, warm), undertone, melanin, vitiligo, 30 or more makeup options, scars, tattoos and paint with intensity | Not published | [Game Informer](https://gameinformer.com/exclusive/2024/06/13/a-deep-dive-into-dragon-age-the-veilguards-expansive-character-creator) |
| Cyberpunk 2077 | Skin tone (12) and skin type (5), blemishes, makeup, freckles, tattoos | Each complexion is its own set of albedo textures, numbered `_d01` to `_d05`. Makeup, freckles, blemishes, tattoos and scars are separate head meshes with materials. Eye makeup is a greyscale mask with a material | A [modding wiki](https://wiki.redmodding.org/cyberpunk-2077-modding/for-mod-creators/references-lists-and-overviews/cheat-sheet-head) and a [game guide](https://www.pcgamesn.com/cyberpunk-2077/character-creator) |
| Black Desert Online | Skin colour and shine, makeup, blush, lips, eyebrows, tattoos with placement and opacity | Not published | The [official wiki](https://www.naeu.playblackdesert.com/Wiki?wikiNo=5) states the controls. The mobile wiki gives a luster range of 0 to 100 |
| Dragon's Dogma 2 | About 40 skin types and 59 skin colours, wrinkle and sheen sliders | Not published | A [game guide, search result](https://samurai-gamers.com/dragons-dogma-2/dd2-character-creator-recipes-list/), secondary. The director said about 80 faces were scanned ([GamesRadar](https://gamesradar.com/why-is-dragons-dogma-2s-character-creator-so-good-its-director-says-the-rpgs-team-scanned-nearly-80-faces-for-base-head-variations-for-starters)) |
| Mass Effect, Unreal and Unity skin samples, 2K and Bethesda's own docs | Not searched in depth | Not found | See the gaps |

Three patterns stand out.
- **Almost every creator keeps the base skin separate from the paint.** The Sims 4 has a base colour texture and overlays. Reallusion has a base layer that cannot be removed. Bethesda has a Skin Tone tint and other tints. Baldur's Gate 3 has pigment maps and overlays.
- **Almost every creator bakes in the end.** MetaHuman bakes on export. Reallusion bakes on request. EA says it composites at runtime, probably into a texture (I infer this from the name "Texture Compositor"). The earlier note says the same about shader layering against baking.
- **Authored scanned detail is the norm.** MetaHuman and Veilguard both use pre-made complexions (40 in Veilguard) and not procedural pores. The only procedural pore or noise work I found was in research.

## Blend maths that a programmer can use

### Layer compositing

The Bethesda tint system and Reallusion use the standard image blend modes. The W3C compositing standard gives the formulas, with colours in the range 0 to 1 ([W3C Compositing and Blending Level 1](https://www.w3.org/TR/compositing-1/)). Here `Cb` is the colour below, `Cs` is the layer colour, and the result is clamped.

- Multiply is `Cb * Cs`.
- Screen is `Cb + Cs - Cb * Cs`.
- Hard light is `Multiply(Cb, 2 * Cs)` if `Cs <= 0.5`, and otherwise `Screen(Cb, 2 * Cs - 1)`.
- Overlay is hard light with `Cb` and `Cs` swapped.
- Soft light is `Cb - (1 - 2 * Cs) * Cb * (1 - Cb)` if `Cs <= 0.5`. Otherwise it is `Cb + (2 * Cs - 1) * (D(Cb) - Cb)`. Here `D(Cb)` is `((16 * Cb - 12) * Cb + 4) * Cb` if `Cb <= 0.25`, and otherwise `sqrt(Cb)`.

The standard mixes with opacity as `result = (1 - a) * Cb + a * Blend(Cb, Cs)`, where `a` is the layer's opacity times its mask value. This is the form I would use. I found no source that says the Bethesda engines use these exact formulas, so that match is my assumption.

### Pigment lookup

1. Choose the table grid. Use `u = cbrt(Cm)` and `v = cbrt(Chd)`, with Cm from 0 to 0.5 and Chd from 0 to 0.32, as in Jimenez.
2. For each grid point, run the spectral model of Alotaibi and Smith, or Donner's, to get a linear RGB colour. Store the table in linear colour.
3. For each surface point, read melanin and haemoglobin from the point's two values, and look up the colour by bilinear filtering.
4. Edit by multiplying the two values. A multiplier of 2 on haemoglobin gave a flushed face, and 1.5 on melanin gave a tan ([Alotaibi and Smith](https://openaccess.thecvf.com/content_ICCV_2017_workshops/papers/w16/Alotaibi_A_Biophysical_3D_ICCV_2017_paper.pdf)).

### Moving one blood distribution onto another

For each pixel, take the local mean and standard deviation of a haemoglobin map `H` over a disc of radius `r`.
`mu(x) = (H * f)(x)` and `sigma(x) = sqrt((H^2 * f)(x) - mu(x)^2)`, where `f` is the disc's averaging kernel.
To move a neutral map `Hn` with statistics `(mu_n, sigma_n)` onto target statistics `(mu, sigma)`, use `H = mu + (sigma / sigma_n) * (Hn - mu_n)`.
To mix several targets with weights `w_i` that sum to 1, use `H = sum(w_i * bias_i) + H0 * sum(w_i * scale_i)`.
Here `H0 = Hn - mu_n`, `bias_0 = mu_n`, `scale_0 = 1`, `bias_i = mu_i` and `scale_i = sigma_i / sigma_n` ([Jimenez et al. 2010](https://www.iryoku.com/skincolor/downloads/A-Practical-Appearance-Model-for-Dynamic-Facial-Color.pdf), equations 1 to 6).
In kjk this could drive a blush or a flush from a few weights, with the bias and scale as low-resolution maps on the face.

### Setting a base tone from measured numbers

A tone setting in CIELAB can be taken straight from the Xiao table above. A programmer would sample L\*, a\* and b\* around those means, convert to linear sRGB with the standard CIELAB to XYZ to sRGB formulas, and use the result as the target of the lookup table's fit. The conversion is textbook and I did not take it from a source. A face spot adds a regional offset. For example, the study's forehead has about 5 more a\* units than the inner arm.

## Which techniques fit kjk's paint track

This section is my own judgement.
Our setup is a mesh with per-vertex positions, procedural layers computed on the 3D surface, and a bake to one texture in Rust.

1. **Base tone and regional tints from two pigment values.** The pigment lookup in Jimenez's paper needs only two numbers per point. It can be computed per vertex or per texel from the 3D surface and baked. It also makes the blush and tan edits of the roadmap (redder cheeks, darker eye sockets) into multiplications of the haemoglobin and melanin values in a region. I recommend it because Baldur's Gate 3 shipped a version of it and Larian's artist says it beat the older multiply approach for fair and deep tones ([80.lv](https://80.lv/articles/creating-characters-for-baldur-s-gate-3/)). KOTOR's textures have lighting painted into them, so I infer the fit should be done on the low-frequency colour only, and the roadmap's open question on painted-in shading applies.
2. **Masked colour layers for lips, makeup and decals.** All the creators I could read use masks with blend modes here, and it is the easy part. Each mask would come from a distance on the 3D surface (to a landmark or a lip border vertex), with a fading edge. This is the roadmap's "built from the head" rule. I found no published way to make the lip border, so that design is ours.
3. **A separate fine detail layer with its own noise.** No source gives a colour recipe for pores, freckles or moles. MetaHuman uses scanned textures chosen by index, and the research on procedural pores and wrinkles is about geometry. I infer our layer must be made from 3D noise sampled on the surface, with a density that varies by region, and baked in the same pass.
4. **Age as less even colour.** The best evidence I found says that less even melanin and haemoglobin go with a higher perceived age ([Matts et al. 2007](https://www.em-consulte.com/en/article/438141)). I infer a first age slider could raise the variance of the two pigment maps and add a few darker spots. This is a guess to test against the KOTOR heads, not a published method.
5. **Learned colour models for analysis only.** AlbedoMM and the BFM and FLAME texture spaces cannot ship. They are trained on photographed adult faces, with few dark skin tones (BFM is biased by its authors' own statement, and AlbedoMM stops at Fitzpatrick type V). I would use them, if at all, to check what a plausible pigment range looks like, and not as the paint's base.
6. **Do not copy MetaHuman's synthesis.** It is a cloud service, its internals are not published, and it needs a model trained on large scanned sets.

## What I could not find

- **Formulas for the engines' tint blends.** I found no document from Bethesda for the exact tint formula in Skyrim or Fallout 4. Creation Kit and UESP pages returned errors for most of my requests, so the Bethesda facts come from the one UESP page I could read and from search result summaries.
- **The MetaHuman texture synthesis internals.** Epic's API pages were blocked for direct reading. I know the inputs (skin UV, a high-frequency index) only through search summaries. I also could not see the fields of the skin settings struct or its ranges.
- **How BioWare, CD Projekt, Capcom, Pearl Abyss and Maxis build their skin textures.** I found creator feature lists and modding pages, and no talk or paper. I did not find the GDC 2024 Larian talk's slides or recording.
- **Sources on Unity's and Unreal's skin samples,** such as Unity's digital human demos, and on Mass Effect creators. I did not search them in depth.
- **Colour recipes for pores, freckles, moles, blemishes and the blue-grey beard shadow.** I found none. Haro et al. and the 2023 micro-wrinkle paper might help, but I could not read their full text.
- **The size of the learned models.** I could not find the number of components of AlbedoMM, the BFM colour model, the FLAME texture spaces or FFHQ-UV's latent code, nor the resolution of the FLAME texture spaces.
- **Licences of the FFHQ-UV dataset, UV-IDM, Relightify and the FLAME "Texture License".** The terms were not readable from the pages I opened.
- **The data tables of the spectral model.** Alotaibi and Smith cite measured haemoglobin extinction tables that I did not download. Jimenez's released melanin and haemoglobin maps and lookup table are behind the ACM portal, and I did not download them either.
- **Dark skin numbers.** The Xiao study has no dark African skin. I did not search for a measured source of CIELAB values for it.
- **Whether any game ships Jimenez's lookup table or an equivalent spectral table.** I found none that documents it.
