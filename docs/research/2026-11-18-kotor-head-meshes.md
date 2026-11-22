# The human head meshes of KOTOR 1 and KOTOR 2

This note was written on 2026-11-18 for kjk's head slider work. From now on the sliders are tested against 3D meshes instead of photos, and the goal is a slider system that can rebuild every human face of KOTOR 1 and KOTOR 2. The worry was that many game heads share one mesh and differ only by their texture, which would make them poor 3D targets. This note lists every human head mesh of both games and measures how different their shapes really are.

## Summary

- KOTOR 1 ships 101 human head models and KOTOR 2 ships 183. Together they hold 194 different head meshes. KOTOR 2 reuses 77 of KOTOR 1's meshes unchanged.
- Texture-only variants are fewer than feared. Only 16 meshes carry two or more different face textures. They are the commoner heads (`comm_*`, `old_*`), KOTOR 2's republic and TSF officers (`rep_off`, `rep_soldier`, `rep_soldier_f`), and the child heads. Every player head and every companion head has its own mesh.
- But many of the different meshes are near copies. Half of the heads have another head within 0.56 mm of their face shape. 82 heads have a twin closer than 0.5 mm, which is below what our slider fits can tell apart. Named NPCs were often made by moving a few vertices of a commoner head.
- The heads differ from their average face by a median of 1.7 mm RMS. Real faces differ from their average by about 2 mm. So the spread of the game faces is close to the spread of real faces, a little smaller.
- When heads that differ by less than 1.5 mm are merged, 48 distinct face shapes remain. When heads that differ by less than 2 mm are merged, 15 remain, and the three largest groups are roughly "women", "light-skinned men" and "dark-skinned and Asian men".
- The verdict is that the meshes are usable but limited test material. About 40 to 48 of them are worth fitting, one per group. A face mesh has about 500 vertices and the face itself about 250, so the meshes carry the proportions of a face and nothing finer. Eyebrows, the lip line, eyelid creases, wrinkles, stubble, beards, most hair and most of the shading are painted into the texture.

## Method

### Finding the heads

A small Rust program built on kjk's own readers (`crates/fmt_odyssey/src/mdl.rs`, `crates/fmt_odyssey/src/key.rs`, `crates/fmt_odyssey/src/bif.rs`, `crates/asset_odyssey/src/model.rs`) opened both Steam installs read only. It parsed every model listed in `chitin.key`, in the override folder, in `Patch.erf` and in every module archive. A model counted as a head when it has a `head_g` node and a mesh skinned to a mouth bone (`f_um_g`, `f_jaw_g` and the others), or a mesh of more than 150 vertices under `head_g`. This found 171 models in KOTOR 1 and 237 in KOTOR 2. The module archives held no head model.

The program also wrote `heads.2da`, `appearance.2da` and `portraits.2da` of both games as text. Each head model was joined to the `heads.2da` rows that name it, and through `appearance.2da`'s `normalhead` and `backuphead` columns to the appearance rows that use it. A model that is a whole body with its face in it, like Calo Nord's `n_calonord` or Kreia's `p_kreiabb`, was joined through the `modela` and `modelb` columns.

The program does not tell humans from aliens. That list was made by hand from the names and the contact sheets. These models were left out as not human: the Twi'leks (`twilek_*`, Mission's `p_missionh`, Yuthura Ban's `n_uthurah`, `n_komadh`, `n_xorh`, `n_zharh`, the assassins `n_twiassh01` and `n_twiassh02`), Juhani (`p_juhanih`), the Wookiees, the Bith, Duros, Rodian, Selkath, Trandoshan, Rakata, Devaronian, Gand, Gran, Quarren, Sullustan, Weequay and Aqualish models, Vandar, Nihilus, the swoop gang head, the supermodels and the main menu models. The stunt models of cutscenes (`m01aa_c01_char01` and the like) were also left out. They are copies of player and companion heads posed for one cutscene, and their face meshes are not readable as head meshes.

### The face mesh of each head

The face mesh of a head is its largest mesh that is skinned to a mouth bone, leaving out the tongue and the teeth. Its node is called `head`, `headnew`, `head_geo`, `headnew_geo`, `head02` or `object01`, depending on the head. Two meshes are "the same mesh" when their face lists are identical and their vertex positions agree to 0.1 mm. Two meshes have "the same topology" when their vertex counts and face lists are identical but their positions differ.

### Units

All distances are in millimetres. The heads keep their own sizes relative to each other. One factor turns model units into millimetres for all heads at once. It was chosen so that the mean distance between the eye centres is 63 mm over the 276 head models that have eyeball meshes. That mean is 57.5 mm in the game's own units, so one game metre counts as 1,096 mm here. The eye centre is the centre of the bounding box of the eyeball mesh (`eyela` and `eyera`). Nine heads have no eyeballs. For those the eye centres were placed 21 mm below and 20 mm behind the under-eye bones `f_llweye_g` and `f_rlweye_g`, which is where the eyeballs sit in the player heads.

### The face region

Every head was first put in a frame of its eyes. The origin is the midpoint between the eye centres. X runs from the left eye to the right eye, Z stays the model's up, and Y points out of the face.

The face region is the part of the average face that the distances are measured over. It was cut from K1's `pmhc01` and then replaced, point for point, by the average of all heads (see below). A point is part of the face region when all of these hold:

- It is no more than 62 mm to the side of the midline. This keeps the cheeks and leaves out the ears.
- It is no higher than 35 mm above the eye centres. This keeps the brows and the lower forehead and leaves out most of the hairline, the scalp and the caps.
- It is no lower than the bottom of the chin, which is 129 mm below the eye centres on the average face. This leaves out the neck.
- It is no more than 22 mm behind the eye centres, and its surface does not face backwards. This leaves out the back of the skull, the sides of the head and the underside of the jaw.
- It is more than 13 mm from either eye centre, measured in the front view. This leaves out the eye openings. The eyelids are separate meshes in KOTOR, and the eyeball shows through a hole in the face mesh, so the eye opening is not part of the face surface.

The face region of the average face has 3,197 points, about one every 3 mm.

### Alignment and the average face

Each head was aligned to the face region with a similarity transform (rotation, translation and one uniform scale). The alignment is iterative closest point. For every point of the face region it finds the nearest point on the head's surface, fits the best similarity transform to those pairs, and repeats. The worst 10% of the pairs are ignored in each fit, so that a cap or a beard does not pull the head out of place. The surfaces were sampled densely, one point every 0.6 mm, and a distance is measured along the surface normal when the nearest sample is close.

The average face was built by repeating this five times. After each round, every point of the face region was moved to the mean of its nearest points on all heads. The average then moved by 0.16 mm or less per round. Eleven heads were measured but kept out of the average: Malak with his metal jaw, the three Darth Sion heads, Visas Marr behind her veil, Calo Nord in goggles and helmet, the three child heads and the two hologram heads. Eight more heads have their face covered by a mask or are not human under a hood, and they were left out of the study entirely: the two masked Dark Jedi of KOTOR 1 (`n_darkjedif`, `n_darkjedim`), KOTOR 2's masked Dark Jedi (`n_dkjh02`) and Onderon soldier (`n_ondsoldierh`), and the four non-humans named above whose models carry human bones. That leaves 190 meshes in the study, 179 of them in the average.

### The distances

- **Distance to the average.** This is the root mean square (RMS) of the distances between the head's face and the average face after the similarity alignment. It is taken both ways and the two are averaged. From the average to the head, every point of the face region is used. From the head to the average, a head point is used only when it lies over the face region, faces the same way as the average there, and is no more than 10 mm behind it. The last rule removes the back wall of the mouth, which lies 30 to 50 mm behind the lips and faces forward.
- **Scale-free and size.** The distance to the average is scale-free, because the alignment may scale the head. The size difference is the scale the alignment used, given as a percentage of the average face. A rigid alignment (no scale) was also run, and its RMS is given in the statistics.
- **Distance between two heads.** For every pair of heads, the points of head A that correspond to the face region were aligned to head B's surface with the same method, and the RMS was taken. Then the same was done from B to A, and the two values were averaged. This is a surface-to-surface distance. It does not need the two meshes to share vertices.
- **Vertex by vertex.** Heads that share a topology were also compared vertex by vertex. The face vertices of one head were aligned to the same vertices of the other with a similarity transform, and the RMS and the largest vertex distance were taken.
- **Groups.** The heads were grouped by average-linkage clustering of the pairwise distances, at several thresholds.

Identical meshes measure between 0.03 and 0.07 mm apart with this method. That is the noise floor of the measurement.

## The heads

### How many there are

| | KOTOR 1 | KOTOR 2 | Both games |
|---|---|---|---|
| Human head models | 101 | 183 | 284 |
| Different face meshes | 89 | 168 | 194 |
| Different topologies | 76 | 152 | 174 |
| Player head models | 30 | 46 | 76 |
| Models with a separate hair mesh | 20 | 46 | |
| Vertices of the face mesh | 339 to 1,144 | 329 to 1,144 | |
| Triangles of the face mesh | 512 to 1,064 | 512 to 1,322 | |

KOTOR 1 has 30 player heads. They are `pfha01` to `pfhc05` for women and `pmha01` to `pmhc05` for men, five each for the groups A, B and C. KOTOR 2's `heads.2da` names 40 player head models in 41 rows. KOTOR 2 adds a fourth group, H, with `pfhh01`, `pfhh02`, `pmhh01` and `pmhh02`, and new heads such as `pfha06`, `pfhc06`, `pfhc07`, `pmha06`, `pmha07` and `pmhb06` to `pmhb10`. KOTOR 2's files hold 6 more player head models that its `heads.2da` does not name. They are `pfha02`, `pmha02`, `pmha04`, `pmhb02`, `pmhb04` and `pmhb05`. The groups are named by the `appearance.2da` labels. Group A is labelled Asian, B Black and C White. KOTOR 2's group H is not labelled. Its heads look Hispanic or mixed, and the commoner labels of KOTOR 2 do use the word Hispanic.

KOTOR 2 rebuilt several of KOTOR 1's player heads under the same names, for example `pfhc01`, `pfha01`, `pmhc01`, `pmha05` and `pfhb05`. The rebuilt heads have new topology. Their face shapes stay close to KOTOR 1's, mostly within 1 mm.

### Hair, eyes, lids and bones

- **Hair.** Most heads have no hair mesh of their own. Their hair is the top of the face mesh, shaped a little and painted with the texture. In KOTOR 1, 20 of 101 models add a separate hair mesh, such as `hairalpha`, `ponytail`, `bangs`, `lhair` or `rhair`. In KOTOR 2, 46 of 183 do. Some hair meshes hang in front of the face. Examples are `pfha05`'s long strands and the bangs of `pfha01`.
- **Eyes and lids.** Every head has the eyeball meshes `eyela` and `eyera` and the lid meshes `eyellid` and `eyerlid`, with a few exceptions. The five Peragus hologram officers of KOTOR 2 (`c_hadmnoffh`, `c_hdockoffh`, `c_hmainoffh`, `c_hmedoffh`, `c_hsecoffh`), `n_talia`, `n_forcezombie`, Visas Marr and Calo Nord have no eyeballs. Their eye openings are empty or covered.
- **Lip sync bones.** 98 of KOTOR 1's 101 models and 178 of KOTOR 2's 183 models skin the face to all six mouth bones (`f_um_g`, `f_jaw_g`, `f_lmc_g`, `f_rmc_g`, `f_llm_g`, `f_rlm_g`). The masked Dark Jedi, the masked Onderon soldier and Malak's jawless head lack some of them. 266 of the 284 face meshes use 16 skin bones, 12 use 17, and a few masked or odd heads use 4 to 15. The full table below gives the number for each model.
- **Brow bones.** Every face except four is also skinned to the brow bones `f_lbrw_g`, `f_rbrw_g` and `f_mdbrw_g`. The exceptions are Calo Nord in both games, Visas Marr and KOTOR 2's masked Dark Jedi `n_dkjh02`.
- **Model errors.** Two models carry small faults. `comm_a_f2` has two nodes named `eyera`, one of them on the left. `p_carthbbh` is an unused copy of Carth's head with no skinned face.

### Meshes with several textures

Only 16 meshes are shared by models with different face textures. These are the true texture-only variants.

| # | Mesh used by | Face textures |
|---|---|---|
| 44 | K1 and K2 `n_child_m`, `n_child_md` | `n_child_m`, `n_child_m02` |
| 76 | K1 and K2 `comm_a_m`, `comm_a_m2` | `n_comamh01`, `n_comamh02` |
| 77 | K1 and K2 `comm_b_f`, `comm_b_f2` | `comm_b_f01`, `comm_b_f02` |
| 78 | K1 and K2 `comm_b_m`, `comm_b_m2` | `n_comm_b_m`, `n_comm_b_m02` |
| 79 | K1 and K2 `comm_w_f`, `comm_w_f2` | `comm_w_f01`, `comm_w_f02` |
| 80 | K1 and K2 `comm_w_m`, `comm_w_m2` | `n_comm_w_m`, `n_comm_w_m02` |
| 81 | K1 `old_a_f`, `old_a_f2`, K2 `old_a_f2` | `n_oldafh01`, `n_oldafh02` |
| 82 | K1 and K2 `old_a_m`, `old_a_m2` | `n_oldamh01`, `n_oldamh02` |
| 83 | K1 and K2 `old_b_f`, `old_b_f2` | `n_oldbfh01`, `n_oldbfh02` |
| 84 | K1 and K2 `old_b_m`, `old_b_m2` | `old_b_m01`, `old_b_m02` |
| 85 | K1 `old_w_f`, `old_w_f2`, K2 `old_w_f2` | `old_w_f01`, `old_w_f02` |
| 86 | K1 and K2 `old_w_m`, `old_w_m2` | `n_oldcmh01`, `n_oldcmh02` |
| 87 | K1 and K2 `rep_off`, K2 `n_tsfcmdrh` | `n_repoffh`, `n_tsfcmdrh` |
| 89 | K1 and K2 `rep_soldier`, K2 `n_tsfoffh` | `n_repsoldh`, `n_tsfoffh` |
| 90 | K1 and K2 `rep_soldier_f`, K2 `n_tsfoff_fh` | `n_repsold_fh01`, `n_tsfoff_fh` |
| 65 | K1 and K2 `n_swoopgang` (not human) | `n_swoopgang01`, none in KOTOR 2 |

There is a second kind of texture variant. `heads.2da`'s `headtex*` columns give the player heads and some companions three or four more textures for the dark side, such as `pmhc01d1` to `pmhc01d3`. These darken and scar the same mesh.

Many more models share a mesh with a model of the other game under the same name, with the same texture. KOTOR 2 took 77 of its human head models unchanged from KOTOR 1.

The third kind of reuse is the most common. A named NPC was made from a commoner head by moving vertices, and the topology stayed the same. Examples are Shaardan and Tanis from the Asian commoner man (`n_shaardanh`, `n_tatcoma_m`, `comm_a_m`), Mekel from the white commoner man (`n_mekelh`, `comm_w_m`), Jorak from the old Asian man (`n_jorakh`, `old_a_m`), Davik and Vrook from one base (`n_davikh`, `n_vrookh`), and Gendar from the black player head `pmhb01`. These heads are 0.5 to 1.7 mm apart over the face. Some of the edits slide vertices along the surface without changing the shape much. Gendar is 2.2 mm from `pmhb01` vertex by vertex but only 1.2 mm surface to surface.

| Pair with the same topology | Vertex RMS (mm) | Largest vertex distance (mm) | Surface RMS (mm) |
|---|---|---|---|
| `n_davikh` and `n_vrookh` | 3.06 | 7.5 | 1.74 |
| `n_jorakh` and `old_a_m` | 2.29 | 9.7 | 1.37 |
| `pmhb01` and `n_gendarh` | 2.23 | 8.1 | 1.24 |
| `n_dorakh` and `old_b_m` | 2.10 | 6.7 | 1.27 |
| `n_mekelh` and `comm_w_m` | 2.00 | 7.6 | 0.86 |
| `n_shaardanh` and `n_tatcoma_m` | 1.50 | 5.6 | 0.55 |
| `n_tatcomw_f` and `comm_a_f` | 1.45 | 12.3 | 0.54 |
| `n_shaardanh` and `comm_a_m` | 0.65 | 5.1 | 0.58 |
| `n_lashoweh` and `n_shaleenah` | 0.27 | 2.0 | 0.12 |
| K1 `pfha02` and K2 `pfha02` | 0.21 | 0.8 | 0.07 |
| `c_holododonna` and `n_dodonnah` | 0.00 | 0.0 | 0.06 |
| `n_sithoffmh` and `sith_off` | 0.00 | 0.0 | 0.06 |

The large vertex distances of `n_tatcomw_f` and `comm_a_f` (12 mm) are in the hair above the face region, not in the face.

## How different the shapes are

### The numbers

These figures are over the 179 heads of the average. The distances are RMS over the face region in millimetres.

| Measure | Median | 10% of heads below | 90% of heads below | Smallest | Largest |
|---|---|---|---|---|---|
| Distance to the average, scale-free | 1.70 | 1.35 | 2.49 | 1.12 | 5.13 |
| The same, rigid (with size) | 1.91 | 1.52 | 2.86 | 1.23 | 4.97 |
| Distance to the nearest other head | 0.56 | 0.05 | 1.32 | 0.03 | 3.26 |
| Size against the average | +0.2% | −7.1% | +7.1% | −12.3% (leaving out the failed fit of `p_kreia1hbb`) | +15.1% |
| Eye distance | 63.9 | 59.0 | 66.5 | 54.3 | 73.3 |

- The mean of the absolute distance to the average, as opposed to the RMS, has a median of 1.2 mm.
- Over the upper face (the brows and the lower forehead, more than 10 mm above the eyes) the median distance to the average is 1.83 mm. Over the middle face (the eyes, the nose and the cheeks) it is 1.71 mm. Over the lower face (the mouth and the chin, more than 45 mm below the eyes) it is 1.36 mm. The upper face is pushed up by caps, helmets and hair that reach down to the brows.
- The sizes spread by about ±7% around the average. On a face 129 mm from the eye line to the chin, 7% is 9 mm. The rigid distances are only 0.2 mm larger than the scale-free ones, so most of each head's difference is shape, not size.
- The eye distances spread more than the face sizes do. With all heads at their own size, the distance between the eye centres runs from 54 to 56 mm (`c_holododonna`, `n_lala_xuh`, `old_b_f`, `pfha01`) to 73 mm (`n_vrookh`, `n_holog0t0`). This is a real feature of the meshes, since the eyes sit in holes of the face mesh. The face region leaves the eye openings out, so this spread is not inside the distances above.
- The women's player heads are 1.83 mm apart on average. The men's player heads are 2.10 mm apart. A woman's player head and a man's player head are 2.19 mm apart. Sex moves the face shape only a little more than the differences inside one sex do.

### Comparison with real faces and with our fits

Real faces differ from their average by about 2 mm. KOTOR's human faces differ from their average by a median of 1.7 mm. The game's cast is therefore almost as varied in shape as real people are, measured over the face.

Our slider fits against real 3D scans miss by about 0.5 to 2 mm. Half of the game heads have another head within 0.56 mm, and 126 of the 179 have one within 1 mm. A slider fit good enough for scans would therefore not be able to tell many of these heads from their nearest twin. The useful test is to fit one head of each group and check that the fit lands closer to that head than to the neighbouring group.

### Groups of truly different faces

| Merge heads closer than | Groups | Groups of one head |
|---|---|---|
| 0.25 mm | 153 | 125 |
| 0.5 mm | 137 | 102 |
| 1.0 mm | 106 | 60 |
| 1.5 mm | 48 | 22 |
| 2.0 mm | 15 | 8 |
| 3.0 mm | 6 | 4 |

At 1.5 mm there are 48 groups. These are the representative heads of each group, with the group's size in brackets. The representative is the head with the smallest total distance to the rest of its group.

- Men: `pmhc01` (14, the light-skinned player and officer heads, Atton, Dopak), `comm_w_m` (14, the white commoners, the albino Sith, Saul, the Czerka commander, Brejik, Tobin), `n_admoffh` (11, most of the black and some Asian player heads, Gadon), `n_traskh` (8, Trask, Canderous in both games, the Disciple, Azkul), `old_b_m` (5), `sith_app` (5, the Sith apprentice, Bandon, Uthar, Dhagon Ghent), `n_hussef` (5, Davik, Deadeye, Vrook, Dol Grenn, Hussef), `pmba01` (4, `pmhb01`, Gendar, `pmhb06`), `pmha01` (4, Jorak, the old Asian man), `p_carthh` (4, Carth, Dustil, Vaklu, Saedhe), `comm_a_m` (4, Shaardan, Tanis), `pmha04` (2, with `pmha07`), `pmha05` (2), `pmhc03` (2, Coorta), `n_dorakh` (2, the contest winner).
- Women: `n_dodonnah` (13, the light-skinned player heads `pfhc01` and `pfhc04`, the white commoners, the officers, the Handmaiden's sisters), `pfha02` (12, the Asian player heads, Jana Lorso, Vash, Lala), `comm_a_f` (12, the Asian commoners, Lashowe, Shaleena, `pfhb03`), `p_handmaidenbb` (11, the Handmaiden, `pfhb02`, `pfhc05`, KOTOR 2's new women's heads), `p_kreiah` (10, every Kreia head, Atris, Mira, Talia), `pfhb04` (8, the other black player heads), `pfhc02` (5, `pfhc03`), `pfha05` (3, Tae-Plae), `p_bastilah` (2, Jedi Master 4).
- Heads that stand alone at 1.5 mm: `pmha02`, Jolee (`p_joleeh`), Malak as a Jedi (`n_jedimalek`), the Republic soldier with his helmet (`rep_soldier`), KOTOR 2's `pfha06`, `pmha06` and `pmhc04`, Bao-Dur, Daraala, Jorran, the Peragus main officer, Master Kavar, the Republic captain, Terena Adare, Zez-Kai Ell, Zherron and Batono. Malak's jawless head, Visas, Calo Nord, the hologram of Goto, Kreia's crippled body model and the children and Darth Sion also stand alone, but they are odd for the reasons given above.

At 2 mm the 15 groups are mostly three large ones. One holds 54 heads, nearly all of them women. One holds 53 heads, the light-skinned men and a few light-skinned women. One holds 45 heads, most of the Asian and dark-skinned men. A fourth group of 14 holds older and heavier men (Carth, Davik, Vrook, Bao-Dur, Kavar). The rest are Kreia and her look-alikes, KOTOR 2's broad-faced `pmha04`, `pmha06` and `pmha07` with Terena Adare, Jolee with Malak, and single heads.

### The heads furthest from the average

The heads furthest from the average, leaving out the odd ones, are Jorran (5.1 mm, a modelled beard), Kreia's crippled body model `p_kreia1hbb` (4.9 mm, where the similarity fit failed and shrank the head to 71%, its rigid distance is 1.95 mm), Batono (4.6 mm, a modelled beard), the Republic officer `rep_off` (3.7 mm, the peak of the cap), the Peragus main officer (3.3 mm, goggles on the forehead), Terena Adare (3.3 mm), Zez-Kai Ell (3.3 mm), the Republic captain (3.2 mm, a cap), `pmha04` (3.2 mm) and the female officer `rep_off_f` (3.0 mm, a cap). Many of the largest distances come from things on the face, not from the face itself.

## What lives in the texture

The two contact sheets show the same heads with and without their textures. Seen without the texture, most heads look alike. The mesh holds:

- the outline of the face, the width of the cheeks and the jaw, and the length of the chin;
- the brow ridge, as one soft bulge;
- the length, the width and the bridge of the nose, and the nostrils as a small step;
- the mouth as two simple lips with a slit, and the lip volume;
- the eye sockets as holes, with the eyeballs and lids as separate meshes;
- the volume of the hair on the skull, as one smooth cap, and in some heads a few hair cards.

The texture holds:

- the eyebrows, every one of them, with their shape, thickness and colour;
- the lip line and the lip colour. The mesh has no edge where the red of the lip meets the skin, so the shape of the lip as people see it is painted;
- the eyelid crease, the eyeliner, the lashes and the shadow around the eyes;
- the shading, painted in. Each texture has darker paint under the brow, beside the nose, under the lower lip and under the jaw, and lighter paint on the nose and the cheekbones. This painted shading makes the faces look more sculpted than their meshes are;
- wrinkles, the nasolabial folds and the bags under the eyes of the old heads (`old_*`, Vrook, Jorran);
- stubble, goatees, moustaches and most beards. Carth's stubble, `pmhc01`'s goatee and the moustaches of `old_b_m` are paint. Jorran's and Batono's beards are the exceptions that are modelled;
- the hairline and the texture of the hair on the skull cap;
- scars, tattoos and make-up, such as Uthar's and Jana Lorso's markings and Bandon's scars.

So a KOTOR face is about half mesh and half paint. A slider system that matches the meshes perfectly will still not look like the game faces until it also matches the painted brows, lip line, eye shadow and shading. This agrees with the earlier finding in the likeness work that eyebrows, hair and the lip edge rank higher for recognition than proportions.

## Verdict

The game meshes are good material for testing the face proportions of the slider system, with limits.

- **Use them.** The meshes are real, different shapes in most cases, and their spread around their average (1.7 mm) is close to the spread of real faces (about 2 mm). They are also exactly the faces that players will try to rebuild.
- **Use one per group.** About 40 to 48 heads carry the real variety. A good test set is the 48 representatives at 1.5 mm listed above, minus Malak's jawless head, Darth Sion, Visas, Calo Nord, the hologram of Goto, Kreia's crippled body model and the children. Add the commoner bases (`comm_*`, `old_*`) because so many NPCs derive from them.
- **Measure the fit over the face region, and below the brows for heads with caps or helmets.** The forehead of a third of the NPC heads is covered by a cap, a helmet, goggles or hair.
- **Expect a ceiling of detail.** The face part of a game mesh has about 250 vertices, one every 6 to 8 mm. A slider fit can match the proportions, but there are no lid shapes, lip edges or folds in the mesh to match.
- **Match the paint separately.** Eyebrows, the lip line, the eye make-up and the painted shading come from the texture. They need the paint sliders, not the shape sliders, and they should be tested against the textures.
- **Leave out** the masked heads, the non-human heads that use human bones (Mission, Juhani, Yuthura, the swoop gang), Malak's jawless head and the Sion heads.

## What this study did not do

- The study did not check that every head in the list is used by the game. It lists the `heads.2da` and `appearance.2da` rows that name each head. Some heads, such as `p_kreiah`, `n_saedheh` and `n_handsis`, are named by no appearance row and may only be used by scripts or cutscenes.
- The eye centres of the nine heads without eyeballs were placed from the under-eye bones. Their eye distances in the table are therefore not measured.
- The face region is a fixed box. The top of the box sits 35 mm above the eyes, so heads with bangs or a low cap still put some hair or cloth into the measured region.
- The similarity fit failed on one head, Kreia's crippled body model `p_kreia1hbb`. It shrank the head to 71%. Its rigid distance (1.95 mm) is the better number for it.
- Closest-point correspondence lets points slide along the surface. The distances therefore measure how far apart the surfaces are, not how far a landmark such as the corner of the mouth has moved. Two faces whose surfaces lie on top of each other but whose features sit at different places would measure close.
- The textures were not measured, only looked at in the contact sheets.
- The original games were not run. The meshes were read with kjk's parser. The heads were drawn at rest, without any clip, so the jaw and the lids are in their rest pose.

## Files

- The research note: `docs/research/2026-11-18-kotor-head-meshes.md`.
- The meshes: `~/.claude/tools/likeness-fit-data/kotor-heads/k1/obj/` and `~/.claude/tools/likeness-fit-data/kotor-heads/k2/obj/`. There is one OBJ per different mesh, named after the first model that uses it. Each OBJ holds the face mesh and the meshes near the head (eyes, lids, teeth, tongue, hair) in model space at rest, in the model's units (metres), with UVs and a material file. Every face texture of every model that uses the mesh is next to it as a PNG. Three textures were not found in either game: `n_dansalvager_h`, `n_misch01` and the swoop gang's KOTOR 2 texture.
- The average face: `~/.claude/tools/likeness-fit-data/kotor-heads/average_face_mm.obj`, in millimetres, in the frame of the eye centres.
- The contact sheets: `~/.claude/tools/likeness-fit-data/kotor-heads/contact_shaded.png` (untextured and shaded) and `~/.claude/tools/likeness-fit-data/kotor-heads/contact_textured.png` (textured). Both show 198 meshes from the front and from the side, at one scale. These are the 194 human meshes and the four non-human heads that use human bones (Mission, Juhani, Yuthura Ban and the swoop gang). The numbers on the sheets are the numbers in the tables of this note.
- The data: `survey.json` (one entry per head model), `obj_index.json` (one entry per mesh), `shape_results.json` (every distance and every group), `pairwise_rms.npy` and `pairwise_names.json` (the distance matrix), in the same folder.
- The scripts: `~/.claude/tools/kotor-heads/`. `dump/` is the Rust reader. `survey.py`, `export.py`, `sheet.py`, `shape.py` and `tables.py` build the survey, the OBJ files, the contact sheets, the measurements and the tables of this note. Run the Python scripts with `~/.claude/tools/likeness-fit/.venv/bin/python -I`.

## Full tables

### Distances of each head

The distances are RMS in millimetres. "Upper face" is more than 10 mm above the eyes. "Lower face" is more than 45 mm below them. "Size" is the scale of the alignment against the average face. "Eye distance" is at the head's own size.

| # | Model | Game | To average, RMS (mm) | Upper face | Middle face | Lower face | Size vs average | Eye distance (mm) | Nearest other head | To nearest, RMS (mm) | Note |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `pfha01` | K1 | 1.53 | 1.25 | 2.04 | 1.20 | -7.2% | 55.5 | n_lala_xuh(k2) | 0.31 |  |
| 2 | `pfha02` | K1 | 1.64 | 1.31 | 2.54 | 1.15 | -2.5% | 66.0 | pfha02(k2) | 0.07 |  |
| 3 | `pfha03` | K1 | 1.78 | 2.50 | 1.93 | 1.15 | -2.1% | 66.7 | pfha03(k2) | 0.34 |  |
| 4 | `pfha04` | K1 | 1.69 | 2.08 | 2.50 | 0.88 | -10.7% | 60.1 | n_tatcoma_f(k1) | 0.10 |  |
| 5 | `pfha05` | K1 | 2.13 | 3.67 | 2.00 | 1.46 | -3.9% | 64.3 | n_taeplaeh(k2) | 0.23 |  |
| 6 | `pfhb01` | K1 | 2.22 | 0.67 | 1.69 | 2.29 | -5.9% | 60.9 | pfhb01(k2) | 0.16 |  |
| 7 | `pfhb02` | K1 | 1.36 | 1.66 | 1.23 | 1.16 | -2.3% | 61.0 | pfhb02(k2) | 0.06 |  |
| 8 | `pfhb03` | K1 | 1.13 | 0.77 | 0.92 | 1.23 | -7.1% | 60.9 | pfhb03(k2) | 0.21 |  |
| 9 | `pfhb04` | K1 | 2.24 | 1.34 | 1.59 | 2.22 | -5.0% | 62.6 | pfhb04(k2) | 0.56 |  |
| 10 | `pfhb05` | K1 | 1.86 | 1.03 | 1.20 | 2.20 | -1.9% | 64.3 | n_medoff(k2) | 0.48 |  |
| 11 | `pfhc01` | K1 | 2.07 | 1.43 | 2.37 | 1.58 | -7.4% | 55.8 | pfhc01(k2) | 0.97 |  |
| 12 | `pfhc02` | K1 | 1.91 | 2.62 | 1.95 | 1.35 | -5.5% | 57.4 | pfhc02(k2) | 0.51 |  |
| 13 | `pfhc03` | K1 | 1.44 | 1.13 | 1.42 | 1.19 | -4.8% | 59.4 | pfhc02(k2) | 1.10 |  |
| 14 | `pfhc04` | K1 | 1.99 | 1.01 | 1.75 | 2.00 | -4.3% | 59.7 | n_handsis(k2) | 0.05 |  |
| 15 | `pfhc05` | K1 | 1.65 | 1.92 | 2.14 | 0.84 | -9.2% | 62.5 | pfhc07(k2) | 1.00 |  |
| 16 | `pmba01` | K1 | 1.57 | 1.94 | 1.99 | 0.87 | +5.8% | 64.0 | pmhb01(k1) | 0.04 |  |
| 17 | `pmha01` | K1 | 1.51 | 1.95 | 1.76 | 1.28 | +4.2% | 62.3 | pmha01(k2) | 0.76 |  |
| 18 | `pmha02` | K1 | 2.00 | 4.56 | 2.29 | 1.17 | +2.0% | 65.6 | pmha01(k2) | 1.42 |  |
| 19 | `pmha03` | K1 | 1.41 | 1.85 | 1.94 | 0.91 | +6.8% | 66.7 | pmha03(k2) | 0.18 |  |
| 20 | `pmha04` | K1 | 3.16 | 4.07 | 3.52 | 1.77 | -2.7% | 70.6 | pmha07(k2) | 1.32 |  |
| 21 | `pmha05` | K1 | 1.46 | 1.13 | 2.05 | 1.32 | +0.0% | 63.1 | pmha05(k2) | 0.63 |  |
| 22 | `pmhb01` | K1 | 1.57 | 1.93 | 1.98 | 0.87 | +5.8% | 64.0 | pmba01(k1) | 0.04 |  |
| 23 | `pmhb02` | K1 | 2.47 | 1.25 | 2.16 | 2.38 | +3.8% | 60.2 | pmhb09(k2) | 1.24 |  |
| 24 | `pmhb03` | K1 | 1.57 | 2.40 | 1.75 | 1.12 | +10.4% | 63.2 | n_repsolbh(k2) | 0.89 |  |
| 25 | `pmhb04` | K1 | 1.40 | 1.00 | 1.36 | 1.44 | +11.0% | 65.9 | pmhb07(k2) | 1.10 |  |
| 26 | `pmhb05` | K1 | 1.51 | 2.42 | 1.67 | 1.11 | +14.6% | 65.8 | n_admoffh(k2) | 0.58 |  |
| 27 | `pmhc01` | K1 | 1.50 | 1.60 | 1.53 | 1.15 | +3.6% | 63.4 | n_brnmodulh(k2) | 0.04 |  |
| 28 | `pmhc02` | K1 | 1.54 | 2.96 | 1.45 | 1.13 | +5.3% | 58.8 | n_scrtyoffh(k2) | 0.05 |  |
| 29 | `pmhc03` | K1 | 2.31 | 2.13 | 1.77 | 2.18 | +4.2% | 64.7 | n_coortah(k2) | 0.37 |  |
| 30 | `pmhc04` | K1 | 2.03 | 3.88 | 1.99 | 1.01 | +6.2% | 66.0 | pmhh01(k2) | 0.91 |  |
| 31 | `pmhc05` | K1 | 2.00 | 1.97 | 2.53 | 1.18 | +1.5% | 57.7 | n_dockof(k2) | 0.72 |  |
| 32 | `p_bastilah` | K1 | 1.68 | 3.67 | 1.65 | 1.02 | -4.2% | 68.2 | n_jedmast4h(k2) | 1.05 |  |
| 33 | `p_candh` | K1 | 1.75 | 1.65 | 1.40 | 1.78 | +4.3% | 60.6 | p_candh03(k2) | 0.05 |  |
| 34 | `p_carthh` | K1 | 1.73 | 2.55 | 1.53 | 1.51 | -1.8% | 59.0 | n_saedheh(k2) | 0.18 |  |
| 35 | `p_joleeh` | K1 | 2.67 | 2.61 | 1.88 | 2.38 | +4.5% | 61.0 | p_baodurh(k2) | 1.84 |  |
| 38 | `ad_saul` | K1 | 2.03 | 1.59 | 1.13 | 1.60 | +2.2% | 65.0 | czerka_com_h(k1) | 0.92 | cap |
| 39 | `c_holododonna` | K1 | 1.64 | 1.43 | 1.78 | 1.35 | -22.7% | 54.3 | n_dodonnah(k1) | 0.06 | hologram; hologram, cap |
| 40 | `czerka_com_h` | K1 | 2.32 | 1.30 | 1.30 | 1.87 | +3.1% | 65.0 | ad_saul(k1) | 0.92 | cap |
| 41 | `darthband_h` | K1 | 1.59 | 1.44 | 1.46 | 1.46 | -2.1% | 61.4 | sith_app(k1) | 0.07 |  |
| 42 | `n_brejikh` | K1 | 1.50 | 1.45 | 1.71 | 1.05 | +5.2% | 65.9 | comm_w_m(k1) | 1.07 |  |
| 43 | `n_calonord` | K1 | 4.11 | 4.96 | 1.75 | 2.33 | +5.4% | 64.1 | n_jedimalek(k1) | 1.97 | goggles and helmet |
| 44 | `n_child_m` | K1 | 1.55 | 2.12 | 1.27 | 1.38 | -4.5% | 62.5 | n_childfhd(k1) | 0.77 | child |
| 45 | `n_childfh` | K1 | 1.61 | 1.38 | 1.50 | 1.62 | -6.7% | 60.9 | n_childfhd(k1) | 0.29 | child |
| 46 | `n_childfhd` | K1 | 1.60 | 1.41 | 1.49 | 1.56 | -6.7% | 60.9 | n_childfh(k1) | 0.29 | child |
| 49 | `n_darthmalak02` | K1 | 6.23 | 6.58 | 2.79 | 8.84 | -18.2% | 69.5 | pfha05(k2) | 4.38 | metal jaw |
| 50 | `n_davikh` | K1 | 2.10 | 3.89 | 1.71 | 1.70 | +13.7% | 65.0 | n_hussef(k2) | 0.22 |  |
| 51 | `n_deadeyeh` | K1 | 2.47 | 2.60 | 1.67 | 2.14 | +15.1% | 68.4 | n_vrookh(k1) | 1.16 |  |
| 52 | `n_dodonnah` | K1 | 1.64 | 1.44 | 1.77 | 1.35 | -4.7% | 59.5 | c_holododonna(k1) | 0.06 | cap |
| 53 | `n_dorakh` | K1 | 1.80 | 0.91 | 2.26 | 1.23 | +6.5% | 65.9 | old_b_m(k1) | 1.27 |  |
| 54 | `n_dustilh` | K1 | 1.68 | 2.65 | 1.58 | 1.37 | -2.6% | 59.0 | p_carthh(k1) | 0.39 |  |
| 55 | `n_gadonh` | K1 | 1.72 | 2.51 | 1.81 | 1.27 | +11.2% | 66.1 | pmhb10(k2) | 1.27 |  |
| 56 | `n_gendarh` | K1 | 2.01 | 2.29 | 2.32 | 1.19 | +10.0% | 64.0 | pmhb01(k1) | 1.24 |  |
| 57 | `n_jedimalek` | K1 | 2.46 | 3.10 | 1.99 | 2.24 | +12.1% | 69.7 | n_mastkavarh(k2) | 1.81 |  |
| 58 | `n_jorakh` | K1 | 1.45 | 1.95 | 1.79 | 1.13 | +3.3% | 62.3 | pmha01(k2) | 1.15 |  |
| 59 | `n_lashoweh` | K1 | 1.38 | 1.15 | 0.90 | 1.39 | -4.1% | 65.0 | n_shaleenah(k1) | 0.12 |  |
| 60 | `n_mekelh` | K1 | 1.55 | 1.46 | 1.52 | 1.47 | +4.0% | 65.0 | comm_w_m(k1) | 0.86 |  |
| 61 | `n_shaardanh` | K1 | 1.73 | 4.33 | 1.52 | 1.08 | +3.7% | 62.3 | n_tatcoma_m(k1) | 0.55 |  |
| 62 | `n_shaleenah` | K1 | 1.35 | 1.25 | 0.92 | 1.34 | -4.6% | 65.0 | n_lashoweh(k1) | 0.12 |  |
| 63 | `n_sithofffh` | K1 | 1.70 | 1.26 | 1.75 | 1.49 | -6.3% | 59.5 | sith_off_f(k1) | 0.15 | cap |
| 64 | `n_sithoffmh` | K1 | 1.33 | 2.25 | 1.10 | 1.25 | +5.5% | 65.0 | sith_off(k1) | 0.06 | cap |
| 66 | `n_tatcoma_f` | K1 | 1.69 | 2.05 | 2.49 | 0.88 | -10.6% | 60.1 | pfha04(k1) | 0.10 |  |
| 67 | `n_tatcoma_m` | K1 | 1.73 | 4.40 | 1.37 | 1.16 | +4.4% | 62.3 | n_shaardanh(k1) | 0.55 |  |
| 68 | `n_tatcomw_f` | K1 | 1.40 | 2.00 | 1.45 | 1.14 | -11.8% | 60.2 | comm_a_f(k1) | 0.54 |  |
| 69 | `n_tatcomw_m` | K1 | 1.17 | 2.25 | 1.02 | 0.96 | +5.8% | 65.0 | comm_w_m(k1) | 0.42 |  |
| 70 | `n_traskh` | K1 | 1.38 | 1.16 | 0.98 | 1.50 | +2.0% | 63.9 | n_traskh(k2) | 0.06 |  |
| 71 | `n_utharh` | K1 | 2.36 | 3.59 | 2.80 | 1.47 | +6.4% | 61.4 | pmhc03(k2) | 1.26 |  |
| 73 | `n_vrookh` | K1 | 2.83 | 2.53 | 1.58 | 2.64 | +13.7% | 72.8 | n_deadeyeh(k1) | 1.16 |  |
| 74 | `comm_a_f` | K1 | 1.43 | 1.43 | 1.40 | 1.31 | -10.1% | 60.2 | comm_a_f2(k1) | 0.07 |  |
| 75 | `comm_a_f2` | K1 | 1.44 | 1.41 | 1.40 | 1.31 | -10.1% | 60.2 | comm_a_f(k1) | 0.07 |  |
| 76 | `comm_a_m` | K1 | 1.56 | 2.63 | 1.71 | 1.06 | +2.1% | 62.3 | c_hdockoffh(k2) | 0.42 |  |
| 77 | `comm_b_f` | K1 | 1.80 | 1.34 | 1.44 | 1.81 | -4.4% | 65.0 | pfhb04(k1) | 0.65 |  |
| 78 | `comm_b_m` | K1 | 1.67 | 0.76 | 1.36 | 1.81 | +0.2% | 65.9 | old_b_m(k1) | 1.07 |  |
| 79 | `comm_w_f` | K1 | 1.58 | 0.98 | 1.70 | 1.42 | -6.3% | 57.0 | pfhc04(k1) | 0.97 |  |
| 80 | `comm_w_m` | K1 | 1.15 | 1.78 | 1.21 | 0.92 | +5.5% | 65.0 | c_hmainoffh(k2) | 0.25 |  |
| 81 | `old_a_f` | K1 | 1.79 | 2.02 | 1.92 | 1.43 | -10.0% | 61.6 | old_a_f(k2) | 0.05 |  |
| 82 | `old_a_m` | K1 | 1.93 | 1.31 | 2.49 | 1.40 | -1.3% | 62.3 | pmha01(k2) | 1.28 |  |
| 83 | `old_b_f` | K1 | 1.48 | 0.93 | 1.36 | 1.36 | -3.2% | 55.4 | n_shaleenah(k1) | 1.07 |  |
| 84 | `old_b_m` | K1 | 1.81 | 0.96 | 1.85 | 1.63 | +2.8% | 65.9 | c_hadmnoffh(k2) | 0.41 |  |
| 85 | `old_w_f` | K1 | 1.56 | 0.60 | 1.51 | 1.44 | -5.1% | 57.0 | old_w_f(k2) | 0.08 |  |
| 86 | `old_w_m` | K1 | 1.77 | 1.54 | 1.12 | 1.75 | +3.4% | 65.0 | c_hsecoffh(k2) | 0.27 |  |
| 87 | `rep_off` | K1 | 3.67 | 2.54 | 1.68 | 1.05 | +7.1% | 61.6 | pmhc02(k1) | 0.86 | cap |
| 88 | `rep_off_f` | K1 | 2.96 | 1.39 | 1.83 | 1.48 | -6.0% | 59.5 | rep_soldier_f(k1) | 0.33 | cap |
| 89 | `rep_soldier` | K1 | 2.44 | 3.52 | 1.44 | 2.19 | +6.8% | 65.0 | n_sithoffmh(k2) | 1.17 | helmet |
| 90 | `rep_soldier_f` | K1 | 1.76 | 1.83 | 1.82 | 1.49 | -6.2% | 59.5 | sith_off_f(k1) | 0.32 | helmet |
| 91 | `sith_app` | K1 | 1.59 | 1.43 | 1.47 | 1.46 | -2.1% | 61.4 | sith_app(k2) | 0.05 |  |
| 92 | `sith_off` | K1 | 1.34 | 2.22 | 1.11 | 1.24 | +5.4% | 65.0 | n_sithoffmh(k1) | 0.06 | cap |
| 93 | `sith_off_f` | K1 | 1.69 | 1.38 | 1.76 | 1.48 | -6.3% | 59.5 | n_sithofffh(k1) | 0.15 | cap |
| 94 | `pfha01` | K2 | 1.62 | 1.15 | 2.22 | 1.28 | -6.1% | 56.5 | n_lala_xuh(k2) | 0.15 |  |
| 95 | `pfha02` | K2 | 1.64 | 1.31 | 2.55 | 1.15 | -2.5% | 66.0 | pfha02(k1) | 0.07 |  |
| 96 | `pfha03` | K2 | 1.92 | 2.52 | 1.99 | 1.37 | -1.6% | 69.8 | pfha03(k1) | 0.34 |  |
| 97 | `pfha04` | K2 | 1.67 | 1.99 | 2.44 | 0.91 | -10.1% | 64.0 | pfha04(k1) | 0.22 |  |
| 98 | `pfha05` | K2 | 2.05 | 1.69 | 1.57 | 2.13 | +2.8% | 62.8 | pfha05(k1) | 1.18 |  |
| 99 | `pfha06` | K2 | 2.62 | 1.98 | 1.69 | 2.57 | -3.3% | 66.6 | pfha05(k2) | 1.23 |  |
| 100 | `pfhb01` | K2 | 2.24 | 0.72 | 1.72 | 2.29 | -5.7% | 60.9 | pfhb01(k1) | 0.16 |  |
| 101 | `pfhb02` | K2 | 1.36 | 1.64 | 1.24 | 1.16 | -2.3% | 63.5 | pfhb02(k1) | 0.06 |  |
| 102 | `pfhb03` | K2 | 1.13 | 0.82 | 0.95 | 1.22 | -7.1% | 61.2 | pfhb03(k1) | 0.21 |  |
| 103 | `pfhb04` | K2 | 2.37 | 1.54 | 1.57 | 2.40 | -4.9% | 64.0 | pfhb04(k1) | 0.56 |  |
| 104 | `pfhb05` | K2 | 1.36 | 1.11 | 1.11 | 1.53 | -3.2% | 64.3 | pfhb05(k1) | 0.86 |  |
| 105 | `pfhc01` | K2 | 1.66 | 1.59 | 1.91 | 1.16 | -6.7% | 55.8 | pfhc01(k1) | 0.97 |  |
| 106 | `pfhc02` | K2 | 1.77 | 2.28 | 1.82 | 1.28 | -6.4% | 55.9 | pfhc02(k1) | 0.51 |  |
| 107 | `pfhc05` | K2 | 1.31 | 1.80 | 1.06 | 1.23 | -2.4% | 61.0 | pfhb02(k1) | 0.94 |  |
| 108 | `pfhc06` | K2 | 1.31 | 1.22 | 1.22 | 1.31 | -6.9% | 59.4 | pfhh01(k2) | 0.68 |  |
| 109 | `pfhc07` | K2 | 1.50 | 0.89 | 1.98 | 0.93 | -0.2% | 63.9 | pfhc05(k1) | 1.00 |  |
| 110 | `pfhh01` | K2 | 1.60 | 1.00 | 1.01 | 1.83 | -6.3% | 59.4 | pfhc06(k2) | 0.68 |  |
| 111 | `pfhh02` | K2 | 1.54 | 2.79 | 1.25 | 1.19 | -7.3% | 63.2 | pfhc06(k2) | 0.98 |  |
| 112 | `pmha01` | K2 | 1.67 | 2.65 | 1.91 | 1.20 | +5.2% | 62.3 | pmha01(k1) | 0.76 |  |
| 113 | `pmha03` | K2 | 1.38 | 1.89 | 1.86 | 0.91 | +7.0% | 66.7 | pmha03(k1) | 0.18 |  |
| 114 | `pmha05` | K2 | 1.72 | 1.13 | 2.07 | 1.65 | +1.3% | 62.1 | pmha05(k1) | 0.63 |  |
| 115 | `pmha06` | K2 | 2.76 | 2.22 | 3.93 | 1.73 | +1.7% | 67.3 | pmha04(k1) | 1.56 |  |
| 116 | `pmha07` | K2 | 2.60 | 3.16 | 3.19 | 1.41 | -1.1% | 65.6 | pmha04(k1) | 1.32 |  |
| 117 | `pmhb06` | K2 | 1.40 | 1.45 | 1.79 | 0.87 | +5.1% | 64.0 | pmba01(k1) | 1.01 |  |
| 118 | `pmhb07` | K2 | 1.23 | 1.23 | 1.09 | 1.20 | +9.5% | 65.3 | pmhb04(k1) | 1.10 |  |
| 119 | `pmhb08` | K2 | 1.23 | 1.20 | 1.59 | 1.04 | +11.0% | 66.8 | n_admoffh(k2) | 1.25 |  |
| 120 | `pmhb09` | K2 | 1.79 | 1.09 | 2.00 | 1.60 | +1.8% | 60.2 | pmhb02(k1) | 1.24 |  |
| 121 | `pmhb10` | K2 | 1.66 | 1.43 | 1.81 | 1.36 | +11.6% | 63.2 | pmhb03(k1) | 0.98 |  |
| 122 | `pmhc01` | K2 | 1.56 | 2.78 | 1.51 | 1.10 | +3.2% | 63.4 | pmhc01(k1) | 0.40 |  |
| 123 | `pmhc03` | K2 | 1.90 | 2.57 | 2.17 | 1.24 | +8.5% | 65.8 | pmhc01(k1) | 1.05 |  |
| 124 | `pmhc04` | K2 | 2.34 | 3.58 | 1.57 | 1.96 | +3.0% | 65.5 | pmhc04(k1) | 1.40 |  |
| 125 | `pmhc06` | K2 | 1.59 | 1.61 | 1.73 | 1.14 | +3.1% | 57.7 | pmhc01(k1) | 0.91 |  |
| 126 | `pmhc07` | K2 | 1.76 | 3.43 | 1.76 | 1.22 | +4.3% | 64.1 | pmhh02(k2) | 0.81 |  |
| 127 | `pmhh01` | K2 | 1.92 | 3.89 | 1.76 | 1.17 | +4.9% | 64.1 | pmhc04(k1) | 0.91 |  |
| 128 | `pmhh02` | K2 | 1.69 | 3.23 | 1.88 | 0.91 | +4.8% | 64.1 | pmhc07(k2) | 0.81 |  |
| 129 | `p_atrisbb` | K2 | 2.03 | 2.52 | 1.99 | 1.31 | -6.2% | 59.2 | n_darthtraya(k2) | 0.95 |  |
| 130 | `p_attonh` | K2 | 1.23 | 1.56 | 1.31 | 0.97 | +6.1% | 65.3 | pmhc01(k1) | 1.27 |  |
| 131 | `p_baodurh` | K2 | 2.41 | 3.99 | 2.32 | 1.75 | +14.2% | 64.3 | n_genvakluh(k2) | 1.46 |  |
| 132 | `p_candh03` | K2 | 1.74 | 1.65 | 1.40 | 1.75 | +4.5% | 60.6 | p_candh(k1) | 0.05 |  |
| 133 | `p_discipleh` | K2 | 1.49 | 1.75 | 1.53 | 1.25 | +3.9% | 67.2 | n_misch01(k2) | 1.03 |  |
| 134 | `p_handmaidenbb` | K2 | 1.35 | 0.94 | 1.55 | 1.05 | -5.3% | 60.8 | p_handmaidenh(k2) | 0.03 |  |
| 135 | `p_handmaidenbd` | K2 | 1.35 | 0.94 | 1.55 | 1.05 | -5.3% | 60.8 | p_handmaidenh(k2) | 0.04 |  |
| 136 | `p_handmaidenh` | K2 | 1.35 | 0.93 | 1.56 | 1.05 | -5.3% | 60.8 | p_handmaidenbb(k2) | 0.03 |  |
| 137 | `p_kreia1hbb` | K2 | 4.89 | 6.33 | 2.97 | 4.85 | -28.7% | 66.1 | n_talia(k2) | 3.26 | similarity fit failed (scale 0.71), rigid fit 1.95 mm |
| 138 | `p_kreia1hstunt01` | K2 | 2.22 | 3.58 | 2.25 | 1.38 | +0.2% | 65.8 | p_kreiabb(k2) | 0.08 |  |
| 139 | `p_kreiabb` | K2 | 2.17 | 3.52 | 2.20 | 1.34 | -0.0% | 69.1 | p_kreiah(k2) | 0.04 |  |
| 140 | `p_kreiaevil02` | K2 | 2.27 | 3.59 | 2.29 | 1.41 | +0.1% | 66.1 | p_kreia1hstunt01(k2) | 0.17 |  |
| 141 | `p_kreiaevil0hbb` | K2 | 2.18 | 3.50 | 2.20 | 1.35 | -0.0% | 66.1 | p_kreiah(k2) | 0.05 |  |
| 142 | `p_kreiah` | K2 | 2.17 | 3.52 | 2.20 | 1.34 | -0.1% | 66.1 | p_kreiabb(k2) | 0.04 |  |
| 143 | `p_kreiastunt` | K2 | 2.14 | 3.51 | 2.17 | 1.32 | -0.2% | 66.1 | p_kreiah(k2) | 0.06 |  |
| 144 | `p_mirah` | K2 | 2.60 | 4.60 | 2.31 | 1.62 | -4.6% | 68.1 | n_talia(k2) | 0.56 |  |
| 145 | `p_visash` | K2 | 3.11 | 2.87 | 1.54 | 1.61 | -4.1% | 64.2 | pfhh02(k2) | 1.44 | veil over the eyes |
| 146 | `c_hadmnoffh` | K2 | 1.81 | 1.05 | 1.86 | 1.71 | +3.3% | 64.1 | old_b_m(k1) | 0.41 |  |
| 147 | `c_hdockoffh` | K2 | 1.60 | 2.53 | 1.81 | 1.19 | +1.9% | 64.1 | comm_a_m(k1) | 0.42 |  |
| 148 | `c_hmainoffh` | K2 | 1.12 | 1.81 | 1.24 | 0.94 | +5.5% | 64.1 | comm_w_m(k1) | 0.25 |  |
| 149 | `c_hmedoffh` | K2 | 1.58 | 1.54 | 1.69 | 1.73 | -10.1% | 64.1 | comm_a_f(k1) | 0.85 |  |
| 150 | `c_hsecoffh` | K2 | 1.76 | 1.56 | 1.15 | 1.77 | +3.4% | 64.1 | old_w_m(k1) | 0.27 |  |
| 151 | `n_admoffh` | K2 | 1.32 | 2.40 | 1.40 | 0.95 | +12.5% | 62.1 | pmhb05(k1) | 0.58 |  |
| 152 | `n_azkulh` | K2 | 1.47 | 1.48 | 0.98 | 1.50 | +4.0% | 63.9 | n_traskh(k1) | 0.53 |  |
| 153 | `n_brnmodulh` | K2 | 1.50 | 1.61 | 1.53 | 1.15 | +3.5% | 63.4 | pmhc01(k1) | 0.04 |  |
| 154 | `n_coltobinh` | K2 | 1.59 | 1.31 | 1.50 | 1.29 | +2.4% | 65.0 | n_correnfalth(k2) | 1.10 |  |
| 155 | `n_contestwin` | K2 | 1.96 | 1.00 | 2.64 | 1.31 | -9.4% | 59.7 | n_dorakh(k1) | 1.42 |  |
| 156 | `n_coortah` | K2 | 2.27 | 2.66 | 1.96 | 1.95 | +5.3% | 63.0 | pmhc03(k1) | 0.37 |  |
| 157 | `n_correnfalth` | K2 | 1.27 | 0.91 | 1.39 | 1.09 | +1.3% | 65.8 | n_coltobinh(k2) | 1.10 |  |
| 158 | `n_dansalvagerh` | K2 | 4.63 | 0.94 | 1.49 | 5.20 | +3.2% | 61.1 | n_jorranh(k2) | 2.53 | beard modelled |
| 159 | `n_daraalah` | K2 | 2.09 | 0.95 | 2.67 | 1.39 | -7.0% | 55.8 | n_contestwin(k2) | 1.60 |  |
| 160 | `n_darthsion` | K2 | 2.02 | 2.17 | 2.05 | 1.50 | +10.8% | 66.8 | n_darthsion02(k2) | 0.04 | disfigured |
| 161 | `n_darthsion02` | K2 | 2.43 | 2.20 | 2.05 | 1.49 | +10.8% | 66.8 | n_darthsion(k2) | 0.04 | disfigured |
| 162 | `n_darthsion03` | K2 | 2.43 | 2.21 | 2.05 | 1.49 | +10.7% | 66.8 | n_darthsion(k2) | 0.04 | disfigured |
| 163 | `n_darthtraya` | K2 | 2.39 | 3.92 | 2.20 | 1.40 | -4.1% | 68.1 | p_mirah(k2) | 0.73 |  |
| 164 | `n_dhagongh` | K2 | 1.43 | 2.12 | 1.48 | 1.21 | +1.1% | 61.4 | sith_app(k1) | 1.27 |  |
| 166 | `n_dockof` | K2 | 2.51 | 2.42 | 2.75 | 1.90 | +2.8% | 57.7 | pmhc05(k1) | 0.72 |  |
| 167 | `n_dolgrenh` | K2 | 2.18 | 2.35 | 1.35 | 1.99 | +13.2% | 65.0 | n_hussef(k2) | 1.17 |  |
| 168 | `n_dopakh` | K2 | 2.48 | 3.42 | 2.67 | 1.49 | +2.6% | 65.6 | pmhc04(k1) | 1.20 |  |
| 169 | `n_forcezombie` | K2 | 1.50 | 1.41 | 1.09 | 1.54 | +0.7% | 64.1 | n_traskh(k1) | 0.84 |  |
| 170 | `n_genvakluh` | K2 | 1.74 | 2.70 | 1.55 | 1.50 | -0.4% | 59.0 | n_saedheh(k2) | 0.80 |  |
| 171 | `n_handsis` | K2 | 2.00 | 1.01 | 1.75 | 2.01 | -4.3% | 59.7 | pfhc04(k1) | 0.05 |  |
| 172 | `n_holog0t0` | K2 | 3.56 | 1.99 | 1.93 | 3.88 | +18.5% | 73.3 | n_vrookh(k1) | 1.95 | hologram |
| 173 | `n_hussef` | K2 | 2.05 | 3.80 | 1.59 | 1.70 | +13.5% | 65.0 | n_davikh(k1) | 0.22 |  |
| 174 | `n_janalorsoh` | K2 | 1.60 | 1.32 | 2.50 | 1.08 | -2.5% | 66.0 | pfha02(k1) | 0.28 |  |
| 175 | `n_jedmast2h` | K2 | 1.71 | 2.19 | 2.53 | 0.89 | -11.1% | 60.1 | pfha04(k1) | 0.32 |  |
| 176 | `n_jedmast4h` | K2 | 1.90 | 3.52 | 1.84 | 1.45 | -6.5% | 64.9 | p_bastilah(k1) | 1.05 |  |
| 177 | `n_jorranh` | K2 | 5.13 | 1.26 | 1.50 | 6.36 | -1.0% | 63.1 | n_dansalvagerh(k2) | 2.53 | beard modelled |
| 178 | `n_lala_xuh` | K2 | 1.60 | 1.12 | 2.18 | 1.27 | -6.4% | 55.4 | pfha01(k2) | 0.15 |  |
| 179 | `n_mainof` | K2 | 3.34 | 2.05 | 2.42 | 3.79 | +0.3% | 63.7 | pmhc03(k1) | 2.05 | goggles on the forehead |
| 180 | `n_mastkavarh` | K2 | 1.80 | 2.07 | 1.06 | 1.92 | +3.1% | 60.0 | n_genvakluh(k2) | 1.52 |  |
| 181 | `n_medoff` | K2 | 1.90 | 1.13 | 1.21 | 2.26 | -2.5% | 66.5 | pfhb05(k1) | 0.48 |  |
| 182 | `n_misch01` | K2 | 1.14 | 1.32 | 1.15 | 1.08 | +4.9% | 66.5 | n_azkulh(k2) | 1.00 |  |
| 184 | `n_repcaph` | K2 | 3.16 | 1.36 | 1.48 | 2.83 | +0.7% | 65.0 | n_vrookh(k1) | 1.56 | cap |
| 185 | `n_repsolbh` | K2 | 1.67 | 1.89 | 1.57 | 1.40 | +7.3% | 63.2 | pmhb03(k1) | 0.89 | helmet |
| 186 | `n_saedheh` | K2 | 1.74 | 2.67 | 1.59 | 1.48 | -1.3% | 59.3 | p_carthh(k1) | 0.18 |  |
| 187 | `n_scrtyoffh` | K2 | 1.54 | 2.94 | 1.45 | 1.13 | +5.3% | 58.8 | pmhc02(k1) | 0.05 |  |
| 188 | `n_shaleenah` | K2 | 1.38 | 1.76 | 1.02 | 1.32 | -6.8% | 66.0 | n_shaleenah(k1) | 0.37 |  |
| 189 | `n_sithoffmh` | K2 | 1.41 | 2.41 | 1.19 | 1.22 | +5.9% | 65.0 | sith_off(k1) | 0.23 | cap |
| 190 | `n_taeplaeh` | K2 | 2.20 | 3.83 | 2.07 | 1.46 | -4.2% | 64.3 | pfha05(k1) | 0.23 |  |
| 191 | `n_talia` | K2 | 2.65 | 4.45 | 2.35 | 1.70 | -4.7% | 64.2 | p_mirah(k2) | 0.56 |  |
| 192 | `n_terenaadareh` | K2 | 3.30 | 5.16 | 4.07 | 1.72 | -12.3% | 61.4 | pmha04(k1) | 1.81 |  |
| 193 | `n_traskh` | K2 | 1.38 | 1.16 | 0.97 | 1.51 | +1.9% | 63.9 | n_traskh(k1) | 0.06 |  |
| 194 | `n_zezkaiellh` | K2 | 3.28 | 1.93 | 2.49 | 2.67 | +3.1% | 66.0 | pmhb06(k2) | 2.11 |  |
| 195 | `n_zherronh` | K2 | 1.56 | 2.05 | 1.50 | 1.29 | +1.1% | 67.5 | pmhc01(k1) | 1.41 |  |
| 196 | `old_a_f` | K2 | 1.79 | 1.99 | 1.91 | 1.43 | -9.8% | 61.6 | old_a_f(k1) | 0.05 |  |
| 197 | `old_w_f` | K2 | 1.56 | 0.60 | 1.51 | 1.45 | -5.2% | 57.0 | old_w_f(k1) | 0.08 |  |
| 198 | `sith_app` | K2 | 1.60 | 1.41 | 1.47 | 1.48 | -2.2% | 61.4 | sith_app(k1) | 0.05 |  |

### Every human head model

The # column is the number of the model's mesh on the contact sheets. "Group" is the ethnicity grouping that the game's labels give. "Skin bones" is the number of bones the face mesh is skinned to. "Lip sync bones" says whether the face is skinned to all six mouth bones.

#### K1

| # | Model | Face texture | Used by | Sex | Group | Vertices | Triangles | Separate hair meshes | Eyeballs | Lids | Skin bones | Lip sync bones | Brow bones |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `pfha01` | `pfha01` | Jedi_Asian_Female_01, P_FEM_A_LRG_01, P_FEM_A_MED_01 and 1 more | female | Asian | 668 | 794 | hairalpha | yes | yes | 16 | yes | yes |
| 2 | `pfha02` | `pfha02` | Jedi_Asian_Female_02, P_FEM_A_LRG_02, P_FEM_A_MED_02 and 1 more | female | Asian | 690 | 810 | none | yes | yes | 16 | yes | yes |
| 3 | `pfha03` | `pfha03` | Jedi_Asian_Female_03, P_FEM_A_LRG_03, P_FEM_A_MED_03 and 1 more | female | Asian | 991 | 982 | none | yes | yes | 16 | yes | yes |
| 4 | `pfha04` | `pfha04` | Commoner_Female_Fat, Jedi_Asian_Female_04, Jedi_Asian_Female_05 and 3 more | female | Asian | 704 | 815 | none | yes | yes | 16 | yes | yes |
| 5 | `pfha05` | `pfha05` | Commoner_Female_Fat, Jedi_Asian_Female_01, Jedi_Asian_Female_02 and 7 more | female | Asian | 685 | 813 | none | yes | yes | 16 | yes | yes |
| 6 | `pfhb01` | `pfhb01` | Jedi_Black_Female_01, P_FEM_B_LRG_01, P_FEM_B_MED_01 and 1 more | female | Black | 581 | 772 | none | yes | yes | 16 | yes | yes |
| 7 | `pfhb02` | `pfhb02` | Jedi_Black_Female_02, P_FEM_B_LRG_02, P_FEM_B_MED_02 and 1 more | female | Black | 908 | 866 | lhair, rhair, tophair | yes | yes | 16 | yes | yes |
| 8 | `pfhb03` | `pfhb03` | Jedi_Black_Female_03, P_FEM_B_LRG_03, P_FEM_B_MED_03 and 1 more | female | Black | 529 | 733 | none | yes | yes | 16 | yes | yes |
| 9 | `pfhb04` | `pfhb04` | Jedi_Black_Female_04, Jedi_Black_Female_05, P_FEM_B_LRG_04 and 2 more | female | Black | 678 | 800 | none | yes | yes | 16 | yes | yes |
| 10 | `pfhb05` | `pfhb05` | Jedi_Black_Female_01, Jedi_Black_Female_02, Jedi_Black_Female_03 and 5 more | female | Black | 770 | 1006 | none | yes | yes | 16 | yes | yes |
| 11 | `pfhc01` | `pfhc01` | Jedi_White_Female_01, P_FEM_C_LRG_01, P_FEM_C_MED_01 and 1 more | female | White | 576 | 794 | none | yes | yes | 16 | yes | yes |
| 12 | `pfhc02` | `pfhc02` | Jedi_Council_Member_Female, Jedi_White_Female_02, P_FEM_C_LRG_02 and 2 more | female | White | 586 | 798 | hairalpha, hair | yes | yes | 16 | yes | yes |
| 13 | `pfhc03` | `pfhc03` | Jedi_White_Female_03, P_FEM_C_LRG_03, P_FEM_C_MED_03 and 1 more | female | White | 632 | 851 | ponytail, hairalpha | yes | yes | 17 | yes | yes |
| 14 | `pfhc04` | `pfhc04` | Jedi_White_Female_04, Jedi_White_Female_05, P_FEM_C_LRG_04 and 2 more | female | White | 1144 | 1064 | none | yes | yes | 16 | yes | yes |
| 15 | `pfhc05` | `pfhc05` | Jedi_Council_Member_Female, Jedi_White_Female_01, Jedi_White_Female_02 and 6 more | female | White | 547 | 758 | none | yes | yes | 16 | yes | yes |
| 16 | `pmba01` | `pmhb01` | no appearance row |  |  | 366 | 512 | hair | yes | yes | 16 | yes | yes |
| 17 | `pmha01` | `pmha01` | Jedi_Asian_Male_01, P_MAL_A_LRG_01, P_MAL_A_MED_01 and 1 more | male | Asian | 623 | 756 | none | yes | yes | 16 | yes | yes |
| 18 | `pmha02` | `pmha02` | Jedi_Asian_Male_02, P_MAL_A_LRG_02, P_MAL_A_MED_02 and 1 more | male | Asian | 623 | 784 | none | yes | yes | 16 | yes | yes |
| 19 | `pmha03` | `pmha03` | Jedi_Asian_Male_03, P_MAL_A_LRG_03, P_MAL_A_MED_03 and 1 more | male | Asian | 664 | 882 | none | yes | yes | 16 | yes | yes |
| 20 | `pmha04` | `pmha04` | Jedi_Asian_Male_04, Jedi_Asian_Male_05, P_MAL_A_LRG_04 and 2 more | male | Asian | 591 | 718 | hairalpha01 | yes | yes | 16 | yes | yes |
| 21 | `pmha05` | `pmha05` | Jedi_Asian_Male_01, Jedi_Asian_Male_02, Jedi_Asian_Male_03 and 5 more | male | Asian | 649 | 806 | none | yes | yes | 16 | yes | yes |
| 22 | `pmhb01` | `pmhb01` | Jedi_Black_Male_01, P_MAL_B_LRG_01, P_MAL_B_MED_01 and 1 more | male | Black | 531 | 690 | none | yes | yes | 16 | yes | yes |
| 23 | `pmhb02` | `pmhb02` | Jedi_Black_Male_02, P_MAL_B_LRG_02, P_MAL_B_MED_02 and 1 more | male | Black | 537 | 712 | none | yes | yes | 16 | yes | yes |
| 24 | `pmhb03` | `pmhb03` | Jedi_Black_Male_03, P_MAL_B_LRG_03, P_MAL_B_MED_03 and 1 more | male | Black | 559 | 716 | none | yes | yes | 16 | yes | yes |
| 25 | `pmhb04` | `pmhb04` | Jedi_Black_Male_04, Jedi_Black_Male_05, P_MAL_B_LRG_04 and 2 more | male | Black | 884 | 904 | none | yes | yes | 16 | yes | yes |
| 26 | `pmhb05` | `pmhb05` | Jedi_Black_Male_01, Jedi_Black_Male_02, Jedi_Black_Male_03 and 5 more | male | Black | 560 | 736 | none | yes | yes | 16 | yes | yes |
| 27 | `pmhc01` | `pmhc01` | Commoner_Male_Fat, Jedi_White_Male_01, Lite_Commoner_Male_01 and 3 more | male | White | 636 | 872 | none | yes | yes | 16 | yes | yes |
| 28 | `pmhc02` | `pmhc02` | Commoner_Male_Fat, Jedi_Council_Member_Male, Jedi_White_Male_02 and 4 more | male | White | 555 | 768 | none | yes | yes | 16 | yes | yes |
| 29 | `pmhc03` | `pmhc03` | Jedi_Council_Member_Male, Jedi_White_Male_03, P_MAL_C_LRG_03 and 2 more | male | White | 661 | 820 | none | yes | yes | 16 | yes | yes |
| 30 | `pmhc04` | `pmhc04` | Jedi_White_Male_04, P_MAL_C_LRG_04, P_MAL_C_MED_04 and 1 more | male | White | 689 | 776 | none | yes | yes | 16 | yes | yes |
| 31 | `pmhc05` | `pmhc05` | Jedi_White_Male_01, Jedi_White_Male_02, Jedi_White_Male_03 and 5 more | male | White | 564 | 760 | hair_alpha | yes | yes | 16 | yes | yes |
| 32 | `p_bastilah` | `p_bastilah04` | Party_NPC_Bastila, Party_NPC_Bastila_CUTSCN, Party_NPC_Bastila_SLAVE |  |  | 554 | 784 | hairbangs, hair | yes | yes | 16 | yes | yes |
| 33 | `p_candh` | `p_candh01` | Party_NPC_Cand |  |  | 545 | 760 | none | yes | yes | 16 | yes | yes |
| 34 | `p_carthh` | `p_carthh01` | Party_NPC_Carth |  |  | 565 | 744 | hair | yes | yes | 16 | yes | yes |
| 35 | `p_joleeh` | `p_joleeh01` | Party_NPC_Jolee |  |  | 615 | 702 | none | yes | yes | 16 | yes | yes |
| 36 | `p_juhanih` | `p_juhanih01` | Party_NPC_Juhani |  |  | 488 | 754 | hair | yes | yes | 16 | yes | yes |
| 37 | `p_missionh` | `p_missionh01` | Party_NPC_Mission |  |  | 446 | 662 | hair, haircap | yes | yes | 16 | yes | yes |
| 38 | `ad_saul` | `n_saulh` | Unique_Admiral_Saul |  |  | 594 | 792 | none | yes | yes | 16 | yes | yes |
| 39 | `c_holododonna` | `c_holododonnah` | no appearance row |  |  | 339 | 546 | hair01 | yes | no | 16 | yes | yes |
| 40 | `czerka_com_h` | `czerka_com_h` | Czerka_Commander |  |  | 672 | 838 | none | yes | yes | 16 | yes | yes |
| 41 | `darthband_h` | `n_darthband01_h` | Unique_Darth_Bandon |  |  | 551 | 704 | none | yes | yes | 16 | yes | yes |
| 42 | `n_brejikh` | `n_brejikh01` | Unique_Brejik |  |  | 504 | 668 | none | yes | yes | 16 | yes | yes |
| 43 | `n_calonord` | `n_calonordh01` | Unique_Calo_Nord |  |  | 588 | 805 | none | no | no | 12 | yes | no |
| 44 | `n_child_m` | `n_child_m` | Child_Male | male |  | 353 | 546 | hair | yes | yes | 16 | yes | yes |
| 44 | `n_child_md` | `n_child_m02` | Child_Male_Dirty | male |  | 353 | 546 | hair | yes | yes | 16 | yes | yes |
| 45 | `n_childfh` | `n_childfh01` | Child_Female | female |  | 358 | 546 | hair | yes | yes | 16 | yes | yes |
| 46 | `n_childfhd` | `n_childfh02` | Child_Female_Dirty | female |  | 358 | 546 | hair | yes | yes | 16 | yes | yes |
| 47 | `n_darkjedif` | `n_darkjedif01` | Dark_Jedi_Female_01 | female |  | 410 | 558 | none | yes | yes | 11 | no | yes |
| 48 | `n_darkjedim` | `n_darkjedim01` | Dark_Jedi_Male_01, Dark_Jedi_Master_01, Dark_Jedi_Master_02 and 2 more | male |  | 398 | 520 | none | yes | yes | 11 | no | yes |
| 49 | `n_darthmalak02` | `n_jedimalekh02` | Unique_Darth_Malak_JAWLESS | male |  | 645 | 780 | lflap, rflap, backcape | yes | yes | 13 | no | yes |
| 50 | `n_davikh` | `n_davikh01` | Unique_Davik |  |  | 545 | 730 | none | yes | yes | 16 | yes | yes |
| 51 | `n_deadeyeh` | `n_deadeyeh01` | Unique_Deadeye |  |  | 545 | 730 | none | yes | yes | 16 | yes | yes |
| 52 | `n_dodonnah` | `n_dodonna_h2` | Unique_Admiral_Dodonna |  |  | 339 | 546 | hair01 | yes | yes | 16 | yes | yes |
| 53 | `n_dorakh` | `n_dorakh` | Unique_Dorak |  |  | 540 | 710 | none | yes | yes | 16 | yes | yes |
| 54 | `n_dustilh` | `n_dustilh01` | Unique_Dustil |  |  | 562 | 744 | none | yes | yes | 16 | yes | yes |
| 55 | `n_gadonh` | `n_gadonh01` | Unique_Gadon |  |  | 639 | 796 | none | yes | yes | 16 | yes | yes |
| 56 | `n_gendarh` | `n_gendarh01` | Unique_Gendar |  |  | 531 | 690 | none | yes | yes | 16 | yes | yes |
| 57 | `n_jedimalek` | `n_jedimalekh01` | Unique_Malek_Jedi | male |  | 526 | 778 | cape, cape01 | yes | yes | 16 | yes | yes |
| 58 | `n_jorakh` | `n_jorakh01` | Unique_Jorak |  |  | 547 | 713 | none | yes | yes | 16 | yes | yes |
| 59 | `n_lashoweh` | `n_lashoweh` | Unique_Lashowe |  |  | 518 | 726 | none | yes | yes | 16 | yes | yes |
| 60 | `n_mekelh` | `n_mekelh01` | Unique_MekelH |  |  | 499 | 668 | none | yes | yes | 16 | yes | yes |
| 61 | `n_shaardanh` | `n_shaardanh01` | Unique_Shaardan |  |  | 524 | 674 | none | yes | yes | 16 | yes | yes |
| 62 | `n_shaleenah` | `n_shaleenah` | Unique_Shaleena |  |  | 518 | 726 | none | yes | yes | 16 | yes | yes |
| 63 | `n_sithofffh` | `n_sithoff_f_h` | Sith_Female_Albino | female |  | 582 | 792 | none | yes | yes | 17 | yes | yes |
| 64 | `n_sithoffmh` | `n_sithoff_h` | Sith_Male_Albino | male |  | 351 | 512 | hair | yes | yes | 16 | yes | yes |
| 65 | `n_swoopgang` | `n_swoopgang01` | Lite_Swoop_Gang, Swoop_Gang_01, Swoop_Gang_02 and 3 more |  |  | 572 | 566 | none | no | no | 16 | yes | yes |
| 66 | `n_tatcoma_f` | `n_tatcoma_f` | Czerka_Protcol_OfficerF, Tatooine_comm_Asian_Female | female | Asian | 712 | 815 | none | yes | yes | 16 | yes | yes |
| 67 | `n_tatcoma_m` | `n_tatcoma_m` | Tatooine_comm_Asian_Male, Unique_Tanis | male | Asian | 524 | 674 | none | yes | yes | 16 | yes | yes |
| 68 | `n_tatcomw_f` | `n_tatcomw_f` | Tatooine_comm_White_Female | female | White | 521 | 704 | none | yes | yes | 16 | yes | yes |
| 69 | `n_tatcomw_m` | `n_tatcomw_m` | Tatooine_comm_White_Male | male | White | 472 | 668 | none | yes | yes | 16 | yes | yes |
| 70 | `n_traskh` | `n_traskh` | Unique_Trask |  |  | 528 | 674 | none | yes | yes | 16 | yes | yes |
| 71 | `n_utharh` | `n_utharh` | Unique_Uthar |  |  | 565 | 704 | none | yes | yes | 16 | yes | yes |
| 72 | `n_uthurah` | `n_uthurah` | Unique_Yuthura_Ban |  |  | 359 | 546 | hair | yes | yes | 16 | yes | yes |
| 73 | `n_vrookh` | `n_vrookh` | Unique_Vrook |  |  | 545 | 730 | none | yes | yes | 16 | yes | yes |
| 74 | `comm_a_f` | `comm_a_f` | Commoner_01_Fem_Asian, Commoner_Dirty_Fem_Asian, Sith_Fem_Asian | female | Asian | 521 | 704 | none | yes | yes | 16 | yes | yes |
| 75 | `comm_a_f2` | `comm_a_f02` | Commoner_02_Fem_Asian, Commoner_Dirty_02_Fem_Asian, Sith_Fem_Asian_02 | female | Asian | 521 | 704 | none | yes | yes | 16 | yes | yes |
| 76 | `comm_a_m` | `n_comamh01` | Commoner_01_Mal_Asian, Commoner_Dirty_Mal_Asian, Czerka_Officer_Asian and 2 more | male | Asian | 524 | 674 | none | yes | yes | 16 | yes | yes |
| 76 | `comm_a_m2` | `n_comamh02` | Commoner_02_Mal_Asian, Commoner_Dirty_02_Mal_Asian, Czerka_Officer_Asian_02 and 3 more | male | Asian | 524 | 674 | none | yes | yes | 16 | yes | yes |
| 77 | `comm_b_f` | `comm_b_f01` | Commoner_01_Fem_Black, Commoner_Dirty_Fem_Black, Republic_Officer_Fem_Black and 2 more | female | Black | 527 | 734 | none | yes | yes | 16 | yes | yes |
| 77 | `comm_b_f2` | `comm_b_f02` | Commoner_02_Fem_Black, Commoner_Dirty_02_Fem_Black, Sith_Fem_Black_02 | female | Black | 527 | 734 | none | yes | yes | 16 | yes | yes |
| 78 | `comm_b_m` | `n_comm_b_m` | Commoner_01_Mal_Black, Commoner_Dirty_Mal_Black, Czerka_Officer_Black and 4 more | male | Black | 536 | 674 | none | yes | yes | 16 | yes | yes |
| 78 | `comm_b_m2` | `n_comm_b_m02` | Commoner_02_Mal_Black, Commoner_Dirty_02_Mal_Black, Czerka_Officer_Black_02 and 3 more | male | Black | 536 | 674 | none | yes | yes | 16 | yes | yes |
| 79 | `comm_w_f` | `comm_w_f01` | Commoner_01_Fem_White, Commoner_Dirty_Fem_White, Sith_Fem_White | female | White | 535 | 712 | none | yes | yes | 16 | yes | yes |
| 79 | `comm_w_f2` | `comm_w_f02` | Commoner_02_Fem_White, Commoner_Dirty_02_Fem_White, Sith_Fem_White_02 | female | White | 535 | 712 | none | yes | yes | 16 | yes | yes |
| 80 | `comm_w_m` | `n_comm_w_m` | Commoner_01_Mal_White, Commoner_Dirty_Mal_White, Czerka_Officer_White and 5 more | male | White | 499 | 668 | none | yes | yes | 16 | yes | yes |
| 80 | `comm_w_m2` | `n_comm_w_m02` | Commoner_02_Mal_White, Commoner_Dirty_02_Mal_White, Czerka_Officer_White_02 and 3 more | male | White | 499 | 668 | none | yes | yes | 16 | yes | yes |
| 81 | `old_a_f` | `n_oldafh01` | Commoner_01_Fem_Old_Asian, Commoner_Dirty_Fem_Old_Asian, Jedi_Asian_Old_Fem and 3 more | female | Asian | 519 | 716 | none | yes | yes | 16 | yes | yes |
| 81 | `old_a_f2` | `n_oldafh02` | Commoner_02_Fem_Old_Asian, Commoner_Dirty_02_Fem_Old_Asian, Sith_Fem_Old_Asian_02 | female | Asian | 519 | 716 | none | yes | yes | 16 | yes | yes |
| 82 | `old_a_m` | `n_oldamh01` | Commoner_01_Mal_Old_Asian, Commoner_Dirty_Mal_Old_Asian, Czerka_Officer_Old_Asian and 4 more | male | Asian | 547 | 713 | none | yes | yes | 16 | yes | yes |
| 82 | `old_a_m2` | `n_oldamh02` | Commoner_02_Mal_Old_Asian, Commoner_Dirty_02_Mal_Old_Asian, Czerka_Officer_Old_Asian_02 and 3 more | male | Asian | 547 | 713 | none | yes | yes | 16 | yes | yes |
| 83 | `old_b_f` | `n_oldbfh01` | Commoner_01_Fem_Old_Black, Commoner_Dirty_Fem_Old_Black, Jedi_Black_Old_Fem and 3 more | female | Black | 514 | 726 | none | yes | yes | 16 | yes | yes |
| 83 | `old_b_f2` | `n_oldbfh02` | Commoner_02_Fem_Old_Black, Commoner_Dirty_02_Fem_Old_Black, Czerka_Officer_Old_Black_02 and 1 more | female | Black | 514 | 726 | none | yes | yes | 16 | yes | yes |
| 84 | `old_b_m` | `old_b_m01` | Commoner_01_Mal_Old_Black, Commoner_Dirty_Mal_Old_Black, Czerka_Officer_Old_Black and 4 more | male | Black | 540 | 710 | none | yes | yes | 16 | yes | yes |
| 84 | `old_b_m2` | `old_b_m02` | Commoner_02_Mal_Old_Black, Commoner_Dirty_02_Mal_Old_Black, Republic_Officer_Mal_Old_Black_02 and 2 more | male | Black | 540 | 710 | none | yes | yes | 16 | yes | yes |
| 85 | `old_w_f` | `old_w_f01` | Commoner_01_Fem_Old_White, Commoner_Dirty_Fem_Old_White, Jedi_White_Old_Fem and 3 more | female | White | 532 | 708 | none | yes | yes | 16 | yes | yes |
| 85 | `old_w_f2` | `old_w_f02` | Commoner_02_Fem_Old_White, Commoner_Dirty_02_Fem_Old_White, Sith_Fem_Old_White_02 | female | White | 532 | 708 | none | yes | yes | 16 | yes | yes |
| 86 | `old_w_m` | `n_oldcmh01` | Commoner_01_Mal_Old_White, Commoner_Dirty_Mal_Old_White, Czerka_Officer_Old_White and 4 more | male | White | 545 | 730 | none | yes | yes | 16 | yes | yes |
| 86 | `old_w_m2` | `n_oldcmh02` | Commoner_02_Mal_Old_White, Commoner_Dirty_02_Mal_Old_White, Czerka_Officer_Old_White_02 and 3 more | male | White | 545 | 730 | none | yes | yes | 16 | yes | yes |
| 87 | `rep_off` | `n_repoffh` | Republic_Officer_Mal_White | male | White | 599 | 776 | none | yes | yes | 16 | yes | yes |
| 88 | `rep_off_f` | `n_r_off_f01` | Republic_Officer_Fem_White | female | White | 599 | 810 | none | yes | yes | 17 | yes | yes |
| 89 | `rep_soldier` | `n_repsoldh` | Republic_Soldier_Mal_w_Helmet | male |  | 674 | 850 | none | yes | yes | 16 | yes | yes |
| 90 | `rep_soldier_f` | `n_repsold_fh01` | Republic_Soldier_Fem_w_Helmet | female |  | 633 | 880 | none | yes | yes | 17 | yes | yes |
| 91 | `sith_app` | `n_sithappren02_h` | Dark_Jedi_Sith_Apprentice |  |  | 552 | 704 | none | yes | yes | 16 | yes | yes |
| 92 | `sith_off` | `n_sithoff_h` | no appearance row |  |  | 351 | 512 | hair | yes | yes | 16 | yes | yes |
| 93 | `sith_off_f` | `n_sithoff_f_h` | no appearance row |  |  | 357 | 546 | hair | yes | yes | 16 | yes | yes |

#### K2

| # | Model | Face texture | Used by | Sex | Group | Vertices | Triangles | Separate hair meshes | Eyeballs | Lids | Skin bones | Lip sync bones | Brow bones |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 13 | `pfhc03` | `pfhc03` | Jedi_White_Female_03, P_FEM_C_LRG_04, P_FEM_C_MED_04 and 1 more | female | White | 632 | 851 | ponytail, hairalpha | yes | yes | 17 | yes | yes |
| 14 | `pfhc04` | `pfhc04` | Jedi_White_Female_04, Jedi_White_Female_05, P_FEM_C_LRG_03 and 8 more | female | White | 1144 | 1064 | none | yes | yes | 16 | yes | yes |
| 16 | `pmba01` | `pmhb01` | no appearance row |  |  | 366 | 512 | hair | yes | yes | 16 | yes | yes |
| 18 | `pmha02` | `pmha02` | no appearance row | male | Asian | 623 | 784 | none | yes | yes | 16 | yes | yes |
| 20 | `pmha04` | `pmha04` | no appearance row | male | Asian | 591 | 718 | hairalpha01 | yes | yes | 16 | yes | yes |
| 22 | `pmhb01` | `pmhb01` | P_MAL_B_LRG_01, P_MAL_B_MED_01, P_MAL_B_SML_01 | male | Black | 531 | 690 | none | yes | yes | 16 | yes | yes |
| 23 | `pmhb02` | `pmhb02` | no appearance row | male | Black | 537 | 712 | none | yes | yes | 16 | yes | yes |
| 24 | `pmhb03` | `pmhb03` | Unique_Gerevick | male | Black | 559 | 716 | none | yes | yes | 16 | yes | yes |
| 25 | `pmhb04` | `pmhb04` | no appearance row | male | Black | 884 | 904 | none | yes | yes | 16 | yes | yes |
| 26 | `pmhb05` | `pmhb05` | no appearance row | male | Black | 560 | 736 | none | yes | yes | 16 | yes | yes |
| 28 | `pmhc02` | `pmhc02` | Commoner_Male_Fat, Jedi_White_Male_05, Onderon_Commoner_Mal_06 and 5 more | male | White | 555 | 768 | none | yes | yes | 16 | yes | yes |
| 31 | `pmhc05` | `pmhc05` | Jedi_White_Male_01, Jedi_White_Male_02, Jedi_White_Male_03 and 8 more | male | White | 564 | 760 | hair_alpha | yes | yes | 16 | yes | yes |
| 32 | `p_bastilah` | `p_bastilah04` | Party_NPC_Bastila, Party_NPC_Bastila_CUTSCN, Party_NPC_Bastila_SLAVE |  |  | 554 | 784 | hairbangs, hair | yes | yes | 16 | yes | yes |
| 34 | `p_carthh` | `p_carthh01` | Party_NPC_Carth |  |  | 565 | 744 | hair | yes | yes | 16 | yes | yes |
| 38 | `ad_saul` | `n_saulh` | Unique_Admiral_Saul |  |  | 594 | 792 | none | yes | yes | 16 | yes | yes |
| 39 | `c_holododonna` | `c_holododonnah` | no appearance row |  |  | 339 | 546 | hair01 | yes | no | 16 | yes | yes |
| 40 | `czerka_com_h` | `czerka_com_h` | Czerka_Commander |  |  | 672 | 838 | none | yes | yes | 16 | yes | yes |
| 41 | `darthband_h` | `n_darthband01_h` | Unique_Darth_Bandon |  |  | 551 | 704 | none | yes | yes | 16 | yes | yes |
| 42 | `n_brejikh` | `n_brejikh01` | Unique_Brejik |  |  | 504 | 668 | none | yes | yes | 16 | yes | yes |
| 43 | `n_calonord` | `n_calonordh01` | Unique_Calo_Nord |  |  | 588 | 805 | none | no | no | 12 | yes | no |
| 44 | `n_child_m` | `n_child_m` | Child_Male | male |  | 353 | 546 | hair | yes | yes | 16 | yes | yes |
| 44 | `n_child_md` | `n_child_m02` | Child_Male_Dirty | male |  | 353 | 546 | hair | yes | yes | 16 | yes | yes |
| 45 | `n_childfh` | `n_childfh01` | Child_Female | female |  | 358 | 546 | hair | yes | yes | 16 | yes | yes |
| 46 | `n_childfhd` | `n_childfh02` | Child_Female_Dirty | female |  | 358 | 546 | hair | yes | yes | 16 | yes | yes |
| 47 | `n_darkjedif` | `n_darkjedif01` | Dark_Jedi_Female_01 | female |  | 410 | 558 | none | yes | yes | 11 | no | yes |
| 48 | `n_darkjedim` | `n_darkjedim01` | Dark_Jedi_Master_01, Dark_Jedi_Master_02, Dark_Jedi_Master_03 and 1 more |  |  | 398 | 520 | none | yes | yes | 11 | no | yes |
| 49 | `n_darthmalak02` | `n_jedimalekh02` | Unique_Darth_Malak_JAWLESS | male |  | 645 | 780 | lflap, rflap, backcape | yes | yes | 13 | no | yes |
| 50 | `n_davikh` | `n_davikh01` | no appearance row |  |  | 545 | 730 | none | yes | yes | 16 | yes | yes |
| 51 | `n_deadeyeh` | `n_deadeyeh01` | Onderon_Commoner_Mal_07, Unique_Deadeye | male |  | 545 | 730 | none | yes | yes | 16 | yes | yes |
| 52 | `n_dodonnah` | `n_dodonna_h2` | Unique_Admiral_Dodonna |  |  | 339 | 546 | hair01 | yes | yes | 16 | yes | yes |
| 53 | `n_dorakh` | `n_dorakh` | no appearance row |  |  | 540 | 710 | none | yes | yes | 16 | yes | yes |
| 54 | `n_dustilh` | `n_dustilh01` | Unique_Dustil |  |  | 562 | 744 | none | yes | yes | 16 | yes | yes |
| 55 | `n_gadonh` | `n_gadonh01` | Unique_Gadon |  |  | 639 | 796 | none | yes | yes | 16 | yes | yes |
| 56 | `n_gendarh` | `n_gendarh01` | Unique_Gendar |  |  | 531 | 690 | none | yes | yes | 16 | yes | yes |
| 57 | `n_jedimalek` | `n_jedimalekh01` | Unique_Malek_Jedi | male |  | 526 | 778 | cape, cape01 | yes | yes | 16 | yes | yes |
| 58 | `n_jorakh` | `n_jorakh01` | Unique_Jorak |  |  | 547 | 713 | none | yes | yes | 16 | yes | yes |
| 59 | `n_lashoweh` | `n_lashoweh` | Unique_Lashowe |  |  | 518 | 726 | none | yes | yes | 16 | yes | yes |
| 60 | `n_mekelh` | `n_mekelh01` | Unique_MekelH |  |  | 499 | 668 | none | yes | yes | 16 | yes | yes |
| 61 | `n_shaardanh` | `n_shaardanh01` | 00_Artist_Test_Body_Head_Change, Onderon_Soldier_Mal_01, Unique_Shaardan | male |  | 524 | 674 | none | yes | yes | 16 | yes | yes |
| 63 | `n_sithofffh` | `n_sithoff_f_h` | Sith_Female_Albino | female |  | 582 | 792 | none | yes | yes | 17 | yes | yes |
| 63 | `sith_off_f` | `n_sithoff_f_h` | no appearance row |  |  | 582 | 792 | none | yes | yes | 17 | yes | yes |
| 65 | `n_swoopgang` | `` | Lite_Swoop_Gang, Swoop_Gang_01, Swoop_Gang_02 and 3 more |  |  | 572 | 566 | none | no | no | 16 | yes | yes |
| 66 | `n_tatcoma_f` | `n_tatcoma_f` | Czerka_Protcol_OfficerF, Tatooine_comm_Asian_Female | female | Asian | 712 | 815 | none | yes | yes | 16 | yes | yes |
| 67 | `n_tatcoma_m` | `n_tatcoma_m` | Tatooine_comm_Asian_Male, Unique_Tanis | male | Asian | 524 | 674 | none | yes | yes | 16 | yes | yes |
| 68 | `n_tatcomw_f` | `n_tatcomw_f` | Tatooine_comm_White_Female | female | White | 521 | 704 | none | yes | yes | 16 | yes | yes |
| 69 | `n_tatcomw_m` | `n_tatcomw_m` | Onderon_Soldier_Mal_06, Tatooine_comm_White_Male | male | White | 472 | 668 | none | yes | yes | 16 | yes | yes |
| 71 | `n_utharh` | `n_utharh` | Unique_Uthar |  |  | 565 | 704 | none | yes | yes | 16 | yes | yes |
| 72 | `n_uthurah` | `n_uthurah` | no appearance row |  |  | 359 | 546 | hair | yes | yes | 16 | yes | yes |
| 73 | `n_vrookh` | `n_vrookh` | Unique_Vrook_Master |  |  | 545 | 730 | none | yes | yes | 16 | yes | yes |
| 74 | `comm_a_f` | `comm_a_f` | Commoner_Dirty_Fem_Asian, Commoner_Fem_Asian_01_A, Commoner_Fem_Asian_01_B and 5 more | female | Asian | 521 | 704 | none | yes | yes | 16 | yes | yes |
| 75 | `comm_a_f2` | `comm_a_f02` | no appearance row | female | Asian | 521 | 704 | none | yes | yes | 16 | yes | yes |
| 76 | `comm_a_m` | `n_comamh01` | Commoner_Dirty_Mal_Asian, Commoner_Mal_Asian_01_A, Commoner_Mal_Asian_01_B and 4 more | male | Asian | 524 | 674 | none | yes | yes | 16 | yes | yes |
| 76 | `comm_a_m2` | `n_comamh02` | no appearance row | male | Asian | 524 | 674 | none | yes | yes | 16 | yes | yes |
| 77 | `comm_b_f` | `comm_b_f01` | Commoner_Dirty_Fem_Black, Commoner_Fem_Black_01_A, Commoner_Fem_Black_01_B and 6 more | female | Black | 527 | 734 | none | yes | yes | 16 | yes | yes |
| 77 | `comm_b_f2` | `comm_b_f02` | no appearance row | female | Black | 527 | 734 | none | yes | yes | 16 | yes | yes |
| 78 | `comm_b_m` | `n_comm_b_m` | Commoner_Dirty_Mal_Black, Commoner_Mal_Asian_01_C, Commoner_Mal_Black_01_A and 9 more | male | Black | 536 | 674 | none | yes | yes | 16 | yes | yes |
| 78 | `comm_b_m2` | `n_comm_b_m02` | no appearance row | male | Black | 536 | 674 | none | yes | yes | 16 | yes | yes |
| 79 | `comm_w_f` | `comm_w_f01` | Commoner_Dirty_Fem_White, Commoner_Fem_Asian_01_C, Commoner_Fem_Black_01_D and 10 more | female | White | 535 | 712 | none | yes | yes | 16 | yes | yes |
| 79 | `comm_w_f2` | `comm_w_f02` | no appearance row | female | White | 535 | 712 | none | yes | yes | 16 | yes | yes |
| 80 | `comm_w_m` | `n_comm_w_m` | Commoner_Dirty_Mal_White, Commoner_Mal_White_01_A, Commoner_Mal_White_01_B and 8 more | male | White | 499 | 668 | none | yes | yes | 16 | yes | yes |
| 80 | `comm_w_m2` | `n_comm_w_m02` | no appearance row | male | White | 499 | 668 | none | yes | yes | 16 | yes | yes |
| 81 | `old_a_f2` | `n_oldafh02` | no appearance row | female | Asian | 519 | 716 | none | yes | yes | 16 | yes | yes |
| 82 | `old_a_m` | `n_oldamh01` | Commoner_Dirty_Mal_Old_Asian, Commoner_Mal_Old_Asian_01_A, Commoner_Mal_Old_Asian_01_B and 8 more | male | Asian | 547 | 713 | none | yes | yes | 16 | yes | yes |
| 82 | `old_a_m2` | `n_oldamh02` | no appearance row | male | Asian | 547 | 713 | none | yes | yes | 16 | yes | yes |
| 83 | `old_b_f` | `n_oldbfh01` | Commoner_Dirty_Fem_Old_Black, Commoner_Fem_Old_Black_01_A, Commoner_Fem_Old_Black_01_B and 4 more | female | Black | 514 | 726 | none | yes | yes | 16 | yes | yes |
| 83 | `old_b_f2` | `n_oldbfh02` | no appearance row | female | Black | 514 | 726 | none | yes | yes | 16 | yes | yes |
| 84 | `old_b_m` | `old_b_m01` | Commoner_Dirty_Mal_Old_Black, Commoner_Mal_Old_Black_01_A, Commoner_Mal_Old_Black_01_B and 9 more | male | Black | 540 | 710 | none | yes | yes | 16 | yes | yes |
| 84 | `old_b_m2` | `old_b_m02` | no appearance row | male | Black | 540 | 710 | none | yes | yes | 16 | yes | yes |
| 85 | `old_w_f2` | `old_w_f02` | no appearance row | female | White | 532 | 708 | none | yes | yes | 16 | yes | yes |
| 86 | `old_w_m` | `n_oldcmh01` | Commoner_Dirty_Mal_Old_White, Commoner_Mal_Old_White_01_A, Commoner_Mal_Old_White_01_B and 9 more | male | White | 545 | 730 | none | yes | yes | 16 | yes | yes |
| 86 | `old_w_m2` | `n_oldcmh02` | no appearance row | male | White | 545 | 730 | none | yes | yes | 16 | yes | yes |
| 87 | `n_tsfcmdrh` | `n_tsfcmdrh` | TSF_Commander_Mal_02 | male |  | 599 | 776 | none | yes | yes | 16 | yes | yes |
| 87 | `rep_off` | `n_repoffh` | Republic_Officer_Mal_White | male | White | 599 | 776 | none | yes | yes | 16 | yes | yes |
| 88 | `rep_off_f` | `n_r_off_f01` | Republic_Officer_Fem_White, TSF_Commander_Fem_02 | female | White | 599 | 810 | none | yes | yes | 17 | yes | yes |
| 89 | `n_tsfoffh` | `n_tsfoffh` | TSF_Officer_Mal_03 | male |  | 674 | 850 | none | yes | yes | 16 | yes | yes |
| 89 | `rep_soldier` | `n_repsoldh` | Republic_Soldier_Mal_w_Helmet | male |  | 674 | 850 | none | yes | yes | 16 | yes | yes |
| 90 | `n_tsfoff_fh` | `n_tsfoff_fh` | TSF_Officer_Fem_02 | female |  | 633 | 880 | none | yes | yes | 16 | yes | yes |
| 90 | `rep_soldier_f` | `n_repsold_fh01` | Republic_Soldier_Fem_w_Helmet | female |  | 633 | 880 | none | yes | yes | 16 | yes | yes |
| 92 | `sith_off` | `n_sithoff_h` | no appearance row |  |  | 351 | 512 | hair | yes | yes | 16 | yes | yes |
| 94 | `pfha01` | `pfha01` | Jedi_Asian_Female_01, Onderon_Commoner_Fem_01, P_FEM_A_LRG_01 and 5 more | female | Asian | 584 | 844 | pfha01_hair_strand, hair | yes | yes | 16 | yes | yes |
| 95 | `pfha02` | `pfha02` | no appearance row | female | Asian | 690 | 810 | none | yes | yes | 16 | yes | yes |
| 96 | `pfha03` | `pfha03` | Jedi_Asian_Female_03, Onderon_Commoner_Fem_03, P_FEM_A_LRG_03 and 5 more | female | Asian | 460 | 746 | none | yes | yes | 16 | yes | yes |
| 97 | `pfha04` | `pfha04` | Commoner_Female_Fat, Jedi_Asian_Female_04, Jedi_Asian_Female_05 and 11 more | female | Asian | 563 | 849 | none | yes | yes | 16 | yes | yes |
| 98 | `pfha05` | `pfha05` | Commoner_Female_Fat, Jedi_Asian_Female_01, Jedi_Asian_Female_02 and 12 more | female | Asian | 896 | 1055 | none | yes | yes | 16 | yes | yes |
| 99 | `pfha06` | `pfha06` | Jedi_Asian_Female_02, Onderon_Commoner_Fem_02, P_FEM_A_LRG_01 and 5 more | female | Asian | 554 | 848 | none | yes | yes | 16 | yes | yes |
| 100 | `pfhb01` | `pfhb01` | Jedi_Black_Female_01, P_FEM_B_LRG_01, P_FEM_B_LRG_02 and 4 more | female | Black | 474 | 763 | none | yes | yes | 16 | yes | yes |
| 101 | `pfhb02` | `pfhb02` | Jedi_Black_Female_02, P_FEM_B_LRG_01, P_FEM_B_LRG_02 and 4 more | female | Black | 725 | 824 | head_bangs, ponytail | yes | yes | 16 | yes | yes |
| 102 | `pfhb03` | `pfhb03` | Jedi_Black_Female_03, P_FEM_B_LRG_03, P_FEM_B_LRG_04 and 4 more | female | Black | 487 | 760 | hair_strands | yes | yes | 16 | yes | yes |
| 103 | `pfhb04` | `pfhb04` | Jedi_Black_Female_04, Jedi_Black_Female_05, P_FEM_B_LRG_03 and 8 more | female | Black | 636 | 935 | hair_strands | yes | yes | 16 | yes | yes |
| 104 | `pfhb05` | `pfhb05` | Jedi_Black_Female_01, Jedi_Black_Female_02, Jedi_Black_Female_03 and 5 more | female | Black | 1054 | 1322 | none | yes | yes | 16 | yes | yes |
| 105 | `pfhc01` | `pfhc01a` | Jedi_White_Female_01, P_FEM_C_LRG_01, P_FEM_C_LRG_02 and 4 more | female | White | 632 | 893 | ponytail, bangs | yes | yes | 16 | yes | yes |
| 106 | `pfhc02` | `pfhc02a` | Jedi_Council_Member_Female, Jedi_White_Female_02, P_FEM_C_LRG_01 and 5 more | female | White | 745 | 977 | bangs | yes | yes | 16 | yes | yes |
| 107 | `pfhc05` | `pfhc05` | Jedi_Council_Member_Female, Jedi_White_Female_01, Jedi_White_Female_02 and 6 more | female | White | 791 | 860 | lhair, rhair, tophair | yes | yes | 16 | yes | yes |
| 108 | `pfhc06` | `pfhc06` | P_FEM_C_LRG_06, P_FEM_C_MED_06, P_FEM_C_SML_06 | female | White | 627 | 832 | none | yes | yes | 16 | yes | yes |
| 109 | `pfhc07` | `pfhc07` | P_FEM_C_LRG_03, P_FEM_C_LRG_06, P_FEM_C_MED_03 and 3 more | female | White | 637 | 1004 | pony | yes | yes | 16 | yes | yes |
| 110 | `pfhh01` | `pfhh01` | P_FEM_H_LRG_01, P_FEM_H_LRG_02, P_FEM_H_MED_01 and 3 more | female | Hispanic | 589 | 832 | ponytail_dangly, bangs | yes | yes | 16 | yes | yes |
| 111 | `pfhh02` | `pfhh02` | P_FEM_H_LRG_01, P_FEM_H_LRG_02, P_FEM_H_MED_01 and 3 more | female | Hispanic | 637 | 867 | bangs | yes | yes | 16 | yes | yes |
| 112 | `pmha01` | `pmha01` | Jedi_Asian_Male_01, Onderon_Soldier_Mal_02, P_MAL_A_LRG_01 and 5 more | male | Asian | 519 | 761 | none | yes | yes | 16 | yes | yes |
| 113 | `pmha03` | `pmha03a` | Jedi_Asian_Male_01, Jedi_Asian_Male_02, Jedi_Asian_Male_03 and 14 more | male | Asian | 901 | 936 | none | yes | yes | 16 | yes | yes |
| 114 | `pmha05` | `pmha05a` | Jedi_Asian_Male_02, Onderon_Commoner_Mal_01, PER_Miner_Mal_01 and 6 more | male | Asian | 585 | 870 | none | yes | yes | 16 | yes | yes |
| 115 | `pmha06` | `pmha06` | Jedi_Asian_Male_04, Jedi_Asian_Male_05, Onderon_Commoner_Mal_02 and 4 more | male | Asian | 539 | 753 | hairstrands, hairflap | yes | yes | 16 | yes | yes |
| 116 | `pmha07` | `pmha07` | Jedi_Asian_Male_03, Onderon_Soldier_Mal_03, P_MAL_A_LRG_02 and 5 more | male | Asian | 597 | 830 | none | yes | yes | 16 | yes | yes |
| 117 | `pmhb06` | `pmhb06` | Jedi_Black_Male_01, Onderon_Commoner_Mal_04, PER_Miner_Mal_04 and 3 more | male | Black | 484 | 712 | hairalpha5, hairalpha6, hairalpha4, hairalpha3, hairalpha2, hairalpha1 | yes | yes | 16 | yes | yes |
| 118 | `pmhb07` | `pmhb07` | Jedi_Black_Male_04, Jedi_Black_Male_05, P_MAL_B_LRG_04 and 5 more | male | Black | 572 | 742 | none | yes | yes | 16 | yes | yes |
| 119 | `pmhb08` | `pmhb08` | Jedi_Black_Male_02, P_MAL_B_LRG_02, P_MAL_B_LRG_03 and 4 more | male | Black | 565 | 740 | none | yes | yes | 16 | yes | yes |
| 120 | `pmhb09` | `pmhb09` | Jedi_Black_Male_01, Jedi_Black_Male_02, Jedi_Black_Male_03 and 11 more | male | Black | 543 | 736 | none | yes | yes | 16 | yes | yes |
| 121 | `pmhb10` | `pmhb10` | Jedi_Black_Male_03, Onderon_Officer_Mal_03, P_MAL_B_LRG_02 and 5 more | male | Black | 559 | 716 | none | yes | yes | 16 | yes | yes |
| 122 | `pmhc01` | `pmhc01a` | Commoner_Male_Fat, Jedi_White_Male_01, Lite_Commoner_Male_01 and 3 more | male | White | 669 | 886 | none | yes | yes | 16 | yes | yes |
| 123 | `pmhc03` | `pmhc03` | Jedi_Council_Member_Male, Jedi_White_Male_03, Onderon_Officer_Mal_04 and 9 more | male | White | 563 | 818 | none | yes | yes | 16 | yes | yes |
| 124 | `pmhc04` | `pmhc04a` | Jedi_White_Male_04, Onderon_Commoner_Mal_06, P_MAL_C_LRG_03 and 5 more | male | White | 526 | 795 | bangs | yes | yes | 16 | yes | yes |
| 125 | `pmhc06` | `pmhc06` | Jedi_Council_Member_Male, Jedi_White_Male_02, Onderon_Commoner_Mal_05 and 3 more | male | White | 521 | 769 | hair_front | yes | yes | 16 | yes | yes |
| 126 | `pmhc07` | `pmhc07` | P_MAL_C_LRG_05, P_MAL_C_MED_05, P_MAL_C_SML_05 | male | White | 548 | 780 | hairfront | yes | yes | 16 | yes | yes |
| 127 | `pmhh01` | `pmhh01` | P_MAL_H_LRG_01, P_MAL_H_LRG_02, P_MAL_H_MED_01 and 3 more | male | Hispanic | 542 | 816 | none | yes | yes | 17 | yes | yes |
| 128 | `pmhh02` | `pmhh02` | P_MAL_H_LRG_01, P_MAL_H_LRG_02, P_MAL_H_MED_01 and 3 more | male | Hispanic | 513 | 750 | none | yes | yes | 16 | yes | yes |
| 129 | `p_atrisbb` | `p_atrish01` | Party_NPC_Atris |  |  | 689 | 915 | danglyhair | yes | yes | 16 | yes | yes |
| 130 | `p_attonh` | `p_attnh1` | Party_NPC_Atton |  |  | 546 | 733 | hairflaps_geo | yes | yes | 16 | yes | yes |
| 131 | `p_baodurh` | `p_bao_durh` | Party_NPC_BaoDur |  |  | 590 | 744 | none | yes | yes | 16 | yes | yes |
| 132 | `p_candh03` | `p_candh03` | Party_NPC_Mandalore_No_Helmet |  |  | 498 | 760 | none | yes | yes | 16 | yes | yes |
| 133 | `p_discipleh` | `p_discipleh` | Party_NPC_Discipile |  |  | 455 | 720 | none | yes | yes | 16 | yes | yes |
| 134 | `p_handmaidenbb` | `p_handmaidenh` | Party_NPC_Handmaiden |  |  | 580 | 826 | none | yes | yes | 16 | yes | yes |
| 135 | `p_handmaidenbd` | `p_handmaidenhd1` | Party_NPC_Handmaiden_Clothing_Evil_I |  |  | 558 | 820 | none | yes | yes | 16 | yes | yes |
| 136 | `p_handmaidenh` | `p_handmaidenh` | Party_NPC_Handmaiden_Head |  |  | 578 | 820 | hairdangle | yes | yes | 16 | yes | yes |
| 137 | `p_kreia1hbb` | `p_kreiah` | Party_NPC_Kreia_Gimped |  |  | 753 | 1086 | none | yes | yes | 16 | yes | yes |
| 138 | `p_kreia1hstunt01` | `p_kreiah` | no appearance row |  |  | 747 | 1084 | none | no | no | 16 | yes | yes |
| 139 | `p_kreiabb` | `p_kreiah` | Party_NPC_Kreia |  |  | 753 | 1086 | none | yes | yes | 16 | yes | yes |
| 140 | `p_kreiaevil02` | `p_kreiaevilh` | no appearance row |  |  | 747 | 1084 | none | yes | yes | 16 | yes | yes |
| 141 | `p_kreiaevil0hbb` | `p_kreiaevilh` | Party_NPC_Kreia_Evil_No_Hands |  |  | 747 | 1084 | none | yes | yes | 16 | yes | yes |
| 141 | `p_kreiaevilbb` | `p_kreiaevilh` | Party_NPC_Kreia_Evil |  |  | 747 | 1084 | none | yes | yes | 16 | yes | yes |
| 142 | `p_kreiah` | `p_kreiah` | no appearance row |  |  | 684 | 939 | none | yes | yes | 16 | yes | yes |
| 143 | `p_kreiastunt` | `p_kreiah` | Party_NPC_Kreia_Stunt |  |  | 704 | 970 | none | yes | yes | 16 | yes | yes |
| 144 | `p_mirah` | `p_mirah` | Party_NPC_Mira |  |  | 552 | 836 | hairflaps, hairflaps02, hairstrands | yes | yes | 16 | yes | yes |
| 145 | `p_visash` | `p_visash01` | Party_NPC_Visas |  |  | 505 | 744 | none | no | no | 13 | yes | no |
| 146 | `c_hadmnoffh` | `c_hadmnoffh01` | no appearance row |  |  | 499 | 672 | none | no | no | 16 | yes | yes |
| 147 | `c_hdockoffh` | `c_hdockoffh01` | no appearance row |  |  | 503 | 636 | none | no | no | 16 | yes | yes |
| 148 | `c_hmainoffh` | `c_hmainoffh01` | no appearance row |  |  | 479 | 626 | none | no | no | 16 | yes | yes |
| 149 | `c_hmedoffh` | `c_hmedoffh01` | no appearance row |  |  | 485 | 670 | ponytail | no | no | 16 | yes | yes |
| 150 | `c_hsecoffh` | `c_hsecoffh01` | no appearance row |  |  | 512 | 689 | none | no | no | 16 | yes | yes |
| 151 | `n_admoffh` | `n_admoffh01` | PER_Holo_Miner_Admin |  |  | 495 | 736 | none | yes | yes | 16 | yes | yes |
| 152 | `n_azkulh` | `n_azkulh01` | Unique_Azkul |  |  | 453 | 674 | none | yes | yes | 16 | yes | yes |
| 153 | `n_brnmodulh` | `n_brnmodrul_h` | Unique_Berun_Modrul |  |  | 559 | 872 | none | yes | yes | 16 | yes | yes |
| 154 | `n_coltobinh` | `n_coltobinh` | Unique_Colonel_Tobin |  |  | 458 | 676 | none | yes | yes | 16 | yes | yes |
| 155 | `n_contestwin` | `n_contestwinh` | Unique_ContestWinner |  |  | 492 | 754 | none | yes | yes | 16 | yes | yes |
| 156 | `n_coortah` | `n_coorta_h` | Unique_Coorta |  |  | 580 | 840 | none | yes | yes | 16 | yes | yes |
| 157 | `n_correnfalth` | `n_correnfalth` | Unique_CorrunFalt |  |  | 467 | 686 | none | yes | yes | 16 | yes | yes |
| 158 | `n_dansalvagerh` | `n_dansalvager_h` | Unique_Batono |  |  | 434 | 724 | none | yes | yes | 16 | yes | yes |
| 159 | `n_daraalah` | `n_daraalah01` | Unique_Daraala |  |  | 561 | 794 | hairdangly_geo | yes | yes | 17 | yes | yes |
| 160 | `n_darthsion` | `n_sionh` | Unique_Darth_Sion |  |  | 447 | 661 | none | yes | yes | 16 | yes | yes |
| 161 | `n_darthsion02` | `n_sionh` | no appearance row |  |  | 511 | 689 | none | yes | yes | 16 | yes | yes |
| 162 | `n_darthsion03` | `n_sionh` | no appearance row |  |  | 511 | 689 | none | yes | yes | 16 | yes | yes |
| 163 | `n_darthtraya` | `n_trayah` | no appearance row |  |  | 636 | 908 | none | yes | yes | 16 | yes | yes |
| 164 | `n_dhagongh` | `n_dhagongh` | Unique_Dhagon_Gent |  |  | 501 | 704 | none | yes | yes | 16 | yes | yes |
| 165 | `n_dkjh02` | `n_dkj2h01` | Dark_Jedi_Hi_Male_03, Dark_Jedi_Low_Male_03, Dark_Jedi_Mid_Male_03 | male |  | 495 | 688 | none | no | no | 4 | no | no |
| 166 | `n_dockof` | `n_dockofd01` | PER_Holo_Miner_Dock |  |  | 502 | 750 | none | yes | yes | 16 | yes | yes |
| 167 | `n_dolgrenh` | `n_dolgrenh` | Unique_DolGrenn |  |  | 490 | 730 | none | yes | yes | 16 | yes | yes |
| 168 | `n_dopakh` | `n_dopakh01` | Thug_01, Unique_Dopak |  |  | 579 | 810 | hairstrands | yes | yes | 16 | yes | yes |
| 169 | `n_forcezombie` | `n_forcezom02` | no appearance row |  |  | 422 | 674 | none | yes | yes | 16 | yes | yes |
| 170 | `n_genvakluh` | `n_genvakluh` | Unique_General_Vaklu |  |  | 510 | 744 | none | yes | yes | 16 | yes | yes |
| 171 | `n_handsis` | `n_handsis` | no appearance row |  |  | 1104 | 1064 | none | yes | yes | 16 | yes | yes |
| 172 | `n_holog0t0` | `n_holog0t0` | Unique_Hologram_Goto |  |  | 431 | 682 | none | yes | yes | 17 | yes | yes |
| 173 | `n_hussef` | `n_hussefd01` | Unique_Husef |  |  | 492 | 730 | none | yes | yes | 16 | yes | yes |
| 174 | `n_janalorsoh` | `n_janalorsoh` | Unique_JanaLorso |  |  | 581 | 808 | none | yes | yes | 16 | yes | yes |
| 175 | `n_jedmast2h` | `n_jedmast2h` | Unique_Vash_Jedi_Master |  |  | 634 | 828 | backhairbob, sidehairpieces | yes | yes | 16 | yes | yes |
| 176 | `n_jedmast4h` | `n_jedmast4h` | Jedi_Master_4H |  |  | 583 | 868 | hairstrands01, hairstrands02, hairstrands03, hairstrands04, hairstrands05 | yes | yes | 16 | yes | yes |
| 177 | `n_jorranh` | `n_jorranh01` | Unique_Jorran |  |  | 478 | 680 | hairback, hairstrands | yes | yes | 16 | yes | yes |
| 178 | `n_lala_xuh` | `n_lala_xuh` | Unique_Lala |  |  | 644 | 778 | hairalpha | yes | yes | 16 | yes | yes |
| 179 | `n_mainof` | `n_mainofd01` | PER_Holo_Miner_Main |  |  | 693 | 842 | hairlow, hairmid, hairtop | yes | yes | 16 | yes | yes |
| 180 | `n_mastkavarh` | `n_mastkavarh` | Unique_Master_Kavar |  |  | 511 | 770 | none | yes | yes | 16 | yes | yes |
| 181 | `n_medoff` | `n_medoff01` | PER_Holo_Miner_Med |  |  | 549 | 840 | none | yes | yes | 16 | yes | yes |
| 182 | `n_misch01` | `n_misch01` | no appearance row |  |  | 468 | 720 | none | yes | yes | 16 | yes | yes |
| 183 | `n_ondsoldierh` | `n_ondsoldmh1` | Onderon_Soldier_Mal_Masked | male |  | 359 | 572 | none | yes | yes | 11 | no | yes |
| 184 | `n_repcaph` | `n_repcaph01` | Republic_Captain_Mal_White_01 | male | White | 451 | 674 | none | yes | yes | 16 | yes | yes |
| 185 | `n_repsolbh` | `n_repsolbh01` | Republic_Soldier_Mal_Black_02 | male | Black | 611 | 854 | none | yes | yes | 17 | yes | yes |
| 186 | `n_saedheh` | `n_saedheh` | no appearance row |  |  | 534 | 756 | none | yes | yes | 16 | yes | yes |
| 187 | `n_scrtyoffh` | `n_scrtyoff_h` | PER_Holo_Miner_Security |  |  | 519 | 768 | none | yes | yes | 16 | yes | yes |
| 188 | `n_shaleenah` | `n_shaleenah` | Unique_Shaleena |  |  | 520 | 716 | none | yes | yes | 16 | yes | yes |
| 189 | `n_sithoffmh` | `n_sithoff_h` | Sith_Male_Albino | male |  | 329 | 512 | hair | yes | yes | 16 | yes | yes |
| 190 | `n_taeplaeh` | `n_taeplae_h` | Unique_TaePlae |  |  | 610 | 817 | hairponytail, hairsidedangly | yes | yes | 16 | yes | yes |
| 191 | `n_talia` | `n_taliah` | no appearance row |  |  | 609 | 876 | frontflappiece, pantflaps | no | yes | 15 | yes | yes |
| 192 | `n_terenaadareh` | `n_terenaadareh` | Unique_Terena_Adare |  |  | 663 | 1082 | none | yes | yes | 16 | yes | yes |
| 193 | `n_traskh` | `n_traskh` | Unique_Trask |  |  | 528 | 674 | none | yes | yes | 16 | yes | yes |
| 194 | `n_zezkaiellh` | `n_zezkaiellh` | Unique_ZezKaiEll |  |  | 609 | 904 | hairback, hairstrands | yes | yes | 16 | yes | yes |
| 195 | `n_zherronh` | `n_zherronh` | Unique_Zherron |  |  | 489 | 850 | none | yes | yes | 16 | yes | yes |
| 196 | `old_a_f` | `n_oldafh01` | Commoner_Dirty_Fem_Old_Asian, Commoner_Fem_Old_Asian_01_A, Commoner_Fem_Old_Asian_01_B and 5 more | female | Asian | 519 | 716 | none | yes | yes | 16 | yes | yes |
| 197 | `old_w_f` | `old_w_f01` | Commoner_Dirty_Fem_Old_White, Commoner_Fem_Old_White_01_A, Commoner_Fem_Old_White_01_B and 4 more | female | White | 532 | 708 | none | yes | yes | 16 | yes | yes |
| 198 | `sith_app` | `n_sithappren02_h` | Dark_Jedi_Hi_Male_02, Dark_Jedi_Low_Male_02, Dark_Jedi_Mid_Male_02 and 1 more | male |  | 501 | 704 | none | yes | yes | 16 | yes | yes |
