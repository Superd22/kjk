# The skull refitted under KOTOR's hair

This note was written on the night of 2026-11-21 by the conversation briefed by the head HQ (brief: `/private/tmp/claude-501/-Users-david-perso-kjk/52003e36-b8ca-4208-a733-a6d51b417e4e/scratchpad/jaw/brief-skull-refit.md`). The work serves B2 for the skull, and A2. Nothing went live. Nothing was committed. No judges were run. Nothing in the main kjk checkout changed except the roadmap note.

## Verdict

The skull refit works as a prototype. On the 21 heads with modelled hair, the new skull lies on the bald reference instead of on KOTOR's hair. The face, the jaw and the neck do not move. It is in viewer 2 behind a URL option, and the default page is unchanged.

What is still wrong, first:
- **Only 4 of the 21 heads pass every skull row of the guard outright.** They are pfhb02, Keanu, the Handmaiden and pmha06. Most of the 17 others miss by 0.4 of KOTOR's spread or less beyond the pass mark. The worst misses are these:
  - pfha01 misses on length by 1.46 spreads and on the forehead slope by 2.05 spreads. Its fringe lies inside the reference's face box, so the reference keeps hair on the brow. This is a fault of the reference, not of the refit.
  - tcs_bastila and the Exile miss on where the top lies, front to back, by about 0.8 to 1.0 spreads.
  - pfhc02 misses on the forehead slope by 0.63 spreads, and Terena by 0.73 spreads.
  - pmhc04's top stays 4 mm above the reference (102 mm against 98 mm), which is 0.21 spreads beyond the mark.
- **A faint shoulder remains where the held forehead meets the narrower temples.** It shows on pmhc04 and pfha06 in viewer 2's light, in the front and three-quarter views. The fold check does not see it.
- **The ears move with the side of the head now.** Their shape is untouched, but the skin around them was carried. Fault triangles remain on 20 heads, 0 to 13 per head, and nearly all of them are at the ear: 74 of the 79 marked triangles.
- **On pmhc04, KOTOR's hair in viewer 2 is only its bangs part.** The rest of its hair is painted on its face mesh and is not shown on our head. On the new, narrower skull, the side strands of the bangs stand clear of the temples.
- **The Exile's hair cards float further from the head.** The median gap at their edges grows from 8.5 mm to 12.7 mm. Their edge gaps were already large before.

What is better:
- The skull width above the ears was a median 14.5 mm wider than the reference. It is now 2.0 mm narrower. The back of the head was 3.8 mm off (median of the absolute gap), and it is now 2.0 mm off. On the top, the median absolute gap went from 2.0 mm to 1.0 mm.
- pmhc04's skull is 123 mm wide against 125 mm in the reference, where it was 155 mm. Bastila's is 152 mm against 152 mm, where it was 171 mm.
- Before, the skull stood outside KOTOR's hair at up to 203 places per head (Terena), by up to 15.6 mm. Now it stands outside at none, on all 21 heads.

## The references (step 1)

- **pfha02 and pfhc02 were moved to the haired set.** pfha02 has a bun at the back with a step under it. pfhc02 has a raised crown of layered hair. Both were seen on their colour sheets. `SKULL_SHOWS` in `bald.rs` now has 36 heads.
- **The Exile and the Handmaiden carry hair volumes in their face mesh.** The Exile has blond hair under its hair cards. The Handmaiden has white hair under her hood. The first references of both were 190 mm and 146 mm wide, with tops above the hair. Both now have their face-mesh hair cut.
- **The cut.** Every head's hair parts are cut. On 12 heads the face mesh carries the hair too (`FACE_HAIR` in `bald.rs`): the Exile, pfha01, Keanu, Daraala, Terena, the Handmaiden, Kreia, pfha02, pfha06, pfhc02, pmha06 and pmhc03. There, `painted_hair` cuts what the skin test (`kotor_skin`) calls not skin. It never cuts inside the face (between the brows, the eyes' outer corners and the chin), at the temples in front of the ears, in the beard, or on the neck in front of the ears below 45 mm under the eyes. Behind the ears the cut reaches down to 100 mm under the eyes, for long hair. Kreia's hood and the Handmaiden's white hair are found by their grey colour, and Terena's crest by its dark brown. Kreia's hood and pfha06's bob hang beside the face, so the temple and neck limits do not apply to them. pmhc04 keeps its own rule from the last conversation.
- **How good the fill is.** Each head's hole was cut into the 36 skull-showing heads and filled from the other 35. The median of the root-mean-square gap is 2.7 to 4.2 mm per head, and the worst head is 4.9 to 7.3 mm.
- **What was seen.** The sheets of all 21 references were looked at. Some faults were fixed and some remain:
  - The cut fixes were for Keanu's and pfha01's hair at the nape, Kreia's hood, Terena's crest, pfha06's bob sides, and the temple patches of pfha02 and pfhc02.
  - These faults remain, and all of them lie where the refit holds the head:
    - pfha01's fringe and the Handmaiden's fringe, inside the face box;
    - Kreia's tan drape bands and her collar;
    - the bottom of Keanu's long hair, below the nape.

## The refit (step 2)

`sliders skull refit` moves each merged vertex along its normal by a smooth amount, solved by conjugate gradients. The amount is pulled toward the reference: with 1 on KOTOR's real skin and 0.6 on the filled skull (`SKULL_FILLED`). The pull fades in over 45 mm from any held vertex (`SKULL_FADE`), and bending is penalised with a stiffness of 60 (`SKULL_STIFFNESS`). These are held still:
- a face zone's vertex inside the face box or on real skin;
- the hidden insides;
- the neck's rim (`keep` outside the ear area);
- everything below the neck's cut;
- everything below 75 mm under the eyes;
- everything in front of 25 mm behind the eyes and below 40 mm above them (the lids' linings).

Nothing below 35 mm under the eyes is pulled. The ear flap (the ear area below 32 mm above the eyes) is not pulled. It takes the move of the skin around it, spread smoothly. A last step pushes any filled skull vertex still outside KOTOR's head 1 mm inside, and spreads the push.

It does not rerun `kotor_fit.py`. FLAME's 300 shapes each move the whole head, so a FLAME refit would move the face, which the brief forbids. The refit starts from viewer 2's live head (the sliders on their start shapes), because that is what viewer 2 shows. Its skull is the FLAME fit's skull.

## The guard (step 3), on the final run

The guard reads the absolute skull rows: the top, the back, the height of the back, the length, the skull width above the ears, the forehead slope, and where the top lies front to back. Each row passes when the new skull lies within its mark of the reference. The mark is half of KOTOR's spread over the 36 skull-showing heads, or the row's 95th-percentile wobble when that is larger. The ratio rows divide by the face's height, which the refit holds, so they are shown but not guarded.

| Head | Skull width, old / new / reference (mm) | Top above the eyes | Back behind the eyes | Rows beyond the mark (in KOTOR's spreads) | Fault triangles (at the ear) | Places outside KOTOR's hair, old / new | Hair parts' edge gap, median, old / new (mm) |
|---|---|---|---|---|---|---|---|
| mod_exile_canon | 161 / 142 / 141 | 119 / 118 / 118 | 172 / 176 / 179 | back_height by 0.20, crown_front by 0.78 | 4 (4) | 12 / 0 | 8.5 / 12.7 |
| mod_hp_pfha01 | 155 / 142 / 141 | 119 / 118 / 117 | 178 / 179 / 180 | length by 1.46, forehead_slope by 2.05 | 1 (1) | 10 / 0 | none |
| mod_hp_pfhb02 | 154 / 138 / 141 | 116 / 118 / 117 | 175 / 174 / 175 | passes | 2 (2) | 0 / 0 | none |
| mod_improved_bastila | 172 / 156 / 157 | 117 / 118 / 117 | 179 / 179 / 181 | back_height by 0.05 | 6 (6) | 0 / 0 | 6.1 / 5.9 |
| mod_jaesa | 150 / 142 / 146 | 112 / 113 / 113 | 175 / 173 / 173 | skull_width by 0.00 | 4 (4) | 0 / 0 | 18.3 / 18.6 |
| mod_keanu | 154 / 140 / 141 | 113 / 111 / 110 | 172 / 165 / 165 | passes | 0 (0) | 32 / 0 | 18.3 / 19.3 |
| mod_meetra | 150 / 141 / 146 | 111 / 112 / 112 | 175 / 173 / 173 | skull_width by 0.03 | 4 (4) | 0 / 0 | 12.8 / 12.3 |
| mod_tcs_bastila | 166 / 149 / 153 | 113 / 117 / 116 | 178 / 174 / 176 | back_height by 0.14, length by 0.34, crown_front by 0.99 | 2 (2) | 0 / 0 | 15.7 / 15.9 |
| n_daraalah | 150 / 145 / 147 | 109 / 113 / 111 | 170 / 164 / 167 | length by 0.12 | 7 (5) | 7 / 0 | 12.0 / 9.7 |
| n_dodonnah | 153 / 150 / 149 | 114 / 121 / 120 | 181 / 176 / 177 | forehead_slope by 0.36 | 3 (3) | 0 / 0 | 4.8 / 4.7 |
| n_terenaadareh | 156 / 134 / 138 | 114 / 97 / 97 | 182 / 168 / 175 | back by 0.30, length by 0.19, forehead_slope by 0.73, crown_front by 0.27 | 4 (4) | 203 / 0 | 2.0 / 2.3 |
| p_bastilah | 171 / 152 / 152 | 117 / 118 / 116 | 179 / 179 / 183 | back_height by 0.11, length by 0.10 | 1 (1) | 1 / 0 | 4.1 / 4.3 |
| p_handmaidenbb | 164 / 144 / 146 | 114 / 116 / 117 | 179 / 172 / 174 | passes | 1 (1) | 0 / 0 | 11.3 / 11.3 |
| p_kreiah | 167 / 146 / 150 | 112 / 111 / 110 | 183 / 191 / 193 | back_height by 0.23, forehead_slope by 0.14, skull_width by 0.02 | 0 (0) | 0 / 0 | none |
| pfha02 | 155 / 137 / 138 | 111 / 112 / 112 | 187 / 184 / 186 | forehead_slope by 0.15 | 6 (4) | 127 / 0 | none |
| pfha06 | 164 / 139 / 143 | 118 / 116 / 114 | 184 / 177 / 179 | crown_front by 0.08 | 13 (13) | 14 / 0 | none |
| pfhc02 | 157 / 136 / 137 | 119 / 120 / 120 | 179 / 175 / 177 | forehead_slope by 0.63 | 3 (3) | 81 / 0 | none |
| pmba01 | 152 / 133 / 135 | 103 / 105 / 103 | 177 / 162 / 165 | forehead_slope by 0.29 | 4 (4) | 149 / 0 | 4.2 / 4.5 |
| pmha06 | 154 / 149 / 149 | 109 / 105 / 103 | 181 / 165 / 168 | passes | 6 (6) | 88 / 0 | 6.8 / 6.4 |
| pmhc03 | 143 / 127 / 135 | 114 / 106 / 104 | 173 / 161 / 164 | skull_width by 0.37 | 1 (1) | 173 / 0 | none |
| pmhc04 | 155 / 123 / 125 | 110 / 102 / 98 | 170 / 160 / 164 | top by 0.21, back by 0.02, back_height by 0.20 | 3 (3) | 166 / 0 | none |

The rest of the guard holds on all 21 heads:
- **The face.** The held face moves 0.0 mm on every head. No face zone ends further from the reference than before. That was read as each zone's root-mean-square gap to the reference along the normal.
- **The jaw and throat.** The under-jaw measure's 12 rows (`under_jaw::measure`) change by at most 9 × 10⁻¹³ of a unit.
- **The 36 skull-showing heads.** Their files are the live head unchanged, with a largest move of 0.0 mm.
- **Fault triangles** (turned over, or folded against the old head, over the whole head) are listed in the table. Nearly all of them are at the ear.

A sweep that was tried and not kept: with the filled skull pulling as hard as real skin (1.0), pmhc04's top lands within 0.03 spreads of its mark. The brief asks for a weaker pull on the filled skull, so the default is 0.6. A version that does not pull the forehead zone's freed part (`SKULL_ZONE_FREE=1`) is a little smoother at the temples. The guard showed it was worse on the forehead slope, for example pfha01 misses by 3.59 spreads against 2.05, so it was not kept.

## Viewer 2 (step 4)

The default page is unchanged. The links:
- http://127.0.0.1:8765/viewer2/index.html?shapes=data/skullrefit/shapes.json&shape=skull_refit&head=pmhc04&hair=none
- http://127.0.0.1:8765/viewer2/index.html?shapes=data/skullrefit/shapes.json&shape=skull_refit&head=p_bastilah&hair=none

Change `head=` for any of the 21 haired heads. `shape=skull_live` shows the live head rebuilt from the same files, for comparing. The other 36 heads show the live head in both. These shapes keep the rig from the file, not the live rig, so the face animates as on the fit.

Whether KOTOR's hair sits over the new skull:
- **Bastila: yes.** Her hair parts cover the new skull with no gap seen in the front, side or three-quarter views. The median gap at the parts' edges is 4.3 mm, where it was 4.1 mm.
- **pmhc04: only its bangs are shown, and they float.** Its hair is painted on its own face mesh, not on a part that viewer 2 can put on our head. On the new skull the side strands of the bangs stand clear of the temples.

Pictures:
- The overlays: `/private/tmp/claude-501/-Users-david-perso-kjk/98d9a4ee-4674-4971-b5a2-972896cdc30f/scratchpad/skull/look/overlay_pmhc04.png` and `/private/tmp/claude-501/-Users-david-perso-kjk/98d9a4ee-4674-4971-b5a2-972896cdc30f/scratchpad/skull/look/overlay_p_bastilah.png`. They show the old head in yellow and the new head in green, over the reference: real skin in clay, the filled skull in orange, and KOTOR's hair in blue on the hair-on tiles. They are drawn from the side and the front, with the hair off and on.
- The viewer 2 sheets: `/private/tmp/claude-501/-Users-david-perso-kjk/98d9a4ee-4674-4971-b5a2-972896cdc30f/scratchpad/skull/v2/sheet_pmhc04.png` and `sheet_p_bastilah.png` in the same folder. Their rows are KOTOR, the old head and the new head, with the hair off. `hair_pmhc04.png` and `hair_p_bastilah.png` are the same with KOTOR's hair on.
- The references: `/private/tmp/claude-501/-Users-david-perso-kjk/98d9a4ee-4674-4971-b5a2-972896cdc30f/scratchpad/skull/look/ref_0.png` to `ref_5.png`, with `ref_pfha06.png` and `ref_fix.png` for the fixed heads. The cuts in colour are `look/cut_0.png` to `cut_5.png` and `look/fix_0.png` to `fix_2.png`.
- `look/parts.png` shows the held and pulled parts on the base head. It was drawn before the face-box rule and the ear rule, so it is out of date.

## Not checked

- No judges were run, as the brief asked.
- David has not looked yet.
- The close-ups in viewer 2's light covered only pmhc04 and pfha06, from the front and at three-quarter. The other 19 heads were seen only in clay.
- The head was not checked animated. These shapes keep the rig from the file.
- The reference's fill uncertainty (3 to 6 mm) is not in the guard. A skull row that passes against the reference can still be up to that much off a real skull.

## What going live would need

- **The presets.** The refit is a free movement of the vertices, which no slider can make. Going live needs skull sliders (width, length, height and where the top lies), ranged from the 36 skull-showing heads. Their presets would then be solved toward these refit skulls or the references. The alternative is a change to the start shapes (`starts_all.bin`). Decision 47 says a head is only its slider values, so a start-shape change would go against it.
- **The calibration (decision 62).** The animation calibration records the fit it learned from, and it must be rerun once the skull changes. Its bones near the skull, the ears and the brows read these vertices.
- **The hair caps and cards of part D.** They sit on the scalp and are placed on today's skull. They must be placed again on the new scalp, which is up to 16 mm lower at the sides on pmhc04.
- **The paint layout.** The scalp's texels stretch with the skull. The scalp gets 4% of the density, so this matters little, but the hair paint's edge (the hairline) should be checked.
- **The ears.** They now move with the side of the head. Their own work (later) should start from the new places.

## What was built, and how to undo it

All of it is in the head-paint worktree, `/Users/david/perso/kjk/.claude/worktrees/head-paint`, crate `head_sliders`. It is built with `CARGO_TARGET_DIR=<worktree>/target-bald`. Nothing the viewer uses by default reads it.
- `src/skull.rs` (new) holds the commands `sliders skull colour|parts|refit|overlay|show|widest <data> ...`. They need `BALD_HAIR=/private/tmp/claude-501/-Users-david-perso-kjk/6938bbd2-56c1-41ef-8954-a9e7823dfa58/scratchpad/bald/hair`. `widest` is a debugging print.
- `src/lib.rs` has two new lines (`pub mod skull;` and its `cfg`). `src/bin/sliders.rs` has the `skull` key.
- `src/bald.rs` was changed:
  - `SKULL_SHOWS` has 36 heads, without pfha02 and pfhc02;
  - `hair_faces` also cuts `painted_hair` on the heads of `FACE_HAIR`;
  - `FACE_HAIR`, `painted_hair`, `NECK_Y` and `NAPE_Y` are new.
- The originals of the three changed files are in `/private/tmp/claude-501/-Users-david-perso-kjk/98d9a4ee-4674-4971-b5a2-972896cdc30f/scratchpad/skull/orig/`. To undo, copy them back and delete `src/skull.rs`.
- The references are in `/private/tmp/claude-501/-Users-david-perso-kjk/98d9a4ee-4674-4971-b5a2-972896cdc30f/scratchpad/skull/ref/`. They were made by `sliders bald fill <data> ref 6 <21 heads>`. The last conversation's folder was not touched.
- The final run is in `.../scratchpad/skull/final/`: `measure.json`, `guard.json`, `guard.txt`, `table.md`, `bins/` (in kit vertex order) and `viewer/`. The guard's script is `.../scratchpad/skull/scripts/guard.py`.
- Viewer: delete `/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data/skullrefit/`. Nothing else in viewer 2 changed.
- Roadmap: the line under "Running now".

## Traps

- **The width row of `bald measure` reads the top of our ear area.** It takes the widest level cut from 30 mm above the eyes, and our ear area reaches 43 mm. `skull_width` in `skull.rs` reads from 45 to 90 mm. The old row made the width look stuck while the skull moved.
- **`mesh.keep` holds the ears and the skin around them, up to 43 mm above the eyes, as well as the neck's rim.** Holding `keep` froze the sides of the skull.
- **The forehead zone covers the temples and the front of the scalp under the hair.** Holding whole face zones blocks the skull. The refit holds a zone only inside the face box or on real skin.
- **Ear flaps fold when moved along their own normals.** Carry them with the skin around them.
- **A hard end to the held face makes a crease across the forehead.** The pull must fade in from the held parts. 45 mm with a stiffness of 60 is the current balance. A faint shoulder remains at the temples.
- **`viewer_map.bin` does not cover every kit vertex.** Rebuilding kit positions from viewer files leaves the missing ones at 0. Keep a copy in kit order (`bins/`).
- **The colour test misses hair volumes in both ways.** The Exile's blond hair and the Handmaiden's white hair read as skin. Look at each head's colour sheet (`skull colour`, with `SKULL_FACE_ONLY=1` to hide the hair parts).
- **zsh does not split an unquoted variable into words.** `set -- $cfg` in a loop silently passed both settings as one value, and the defaults ran.
- **A test of "how far the hair stands above the skull" from a fixed hole does not work.** The fill is anchored at the hole's edge, and on pmhc04 the hair reaches below that edge. It was dropped.

## Next

David looks at pmhc04 and Bastila in viewer 2, then the other haired heads. If he accepts the skull, the next step is skull sliders, ranged from the 36 skull-showing heads and solved toward these refits, before anything goes live. The forehead-to-temple shoulder and pfha01's fringe in its reference are the first things to fix.
