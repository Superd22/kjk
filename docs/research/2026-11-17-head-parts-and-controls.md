# Head parts and their controls

> Since 2026-11-19 the order of work is set by `2026-11-19-head-roadmap.md`. This note stays the reference for each head part. Its order of work is out of date.

This note documents the head customization prototype. It was written at the end of 2026-11-17 and brought up to date on 2026-11-18. It lists every part of the head, every control of each part, how each control is set from a photo, how well it scored, and what is still wrong.

**The goal.** The work must show that our sliders, which make up the character designer, can reproduce any face. Reading a face from a photo is a separate problem and comes second. So a slider is judged by setting it from a true 3D head, with no photo involved, and comparing the result with that head. The photo tests show how well the photo reading works.

**Decision of 2026-11-18.** From now on, the sliders are tested only against 3D head meshes. Photo tests kept leaving one question open: when a face came out wrong, was the photo reading at fault, or the sliders? The new goal is a slider system that can reproduce every human head in KOTOR 1 and KOTOR 2, so that our heads match the style of the original games. Well-liked head mods with recognisable faces are a second source of target meshes. The photo tools below stay, but they no longer measure success.

The plan that led here is in `docs/research/2026-11-17-head-sub-systems-plan.md`. This note replaces the plan's list of parts with what was built.

Nothing in this note is in the game or in the Likeness Lab page yet. All of it lives in test scripts outside the repository.

## Terms

A **part** is one region of the head with its own controls and its own rules. The parts are the base layer, the jawline, the lips, the eyes, the lashes and the eyebrows.

A **shape** control moves the skin. A **paint** control changes colour on the skin and moves nothing. Most parts have both.

The **base head** is the head with every control at zero.

The **detector** is MediaPipe's face landmarker. It marks 478 points on a face in a photo.

All measurements on photos are counted in **pupil distances** and shown in millimetres, with the distance between the pupils taken as 63 mm. A photo does not say how large a head is.

**FLAME** is the head model FLAME 2023 Open from the Max Planck Institute. It has 300 identity shapes learned from about 3,800 scanned heads. Its licence is CC BY 4.0.

A person in a test sheet is named by a number, such as **#005**. The number is the start of the photo's file name.

## The order in which a head is built

Each step works on the result of the step before.

1. The base head is loaded.
2. The base layer sets the overall shape from the 300 FLAME shapes.
3. The jawline part moves the edge of the lower face.
4. The eye part sets the size, the place and the lid curves of each eye opening.
5. The paint is drawn: eyebrows, lashes, iris, caruncle and lips.

## The base head

The base head is our own mesh, which comes from MakeHuman and has 4,750 vertices, carried onto FLAME's average head. The script is `flame_transfer.py`.

The transfer does more than copy FLAME. Each item below fixed a fault that showed on every person.

| Step | What it does | Value | Fault it fixed |
|---|---|---|---|
| Surface snap | Moves our skin vertices onto FLAME's surface after the wrap | 0.28 mm on average | A V-shaped point under the lower lip, which came from the MakeHuman mesh |
| Own lids | Keeps our own lids within 15.5 mm of each eye's centre. The skin is FLAME's from 30 mm outward, and a smooth blend lies between | `EYE_OWN`, `EYE_FADE` | FLAME's lids are wide open and its skin runs behind the eyeballs, so eyes came out round and staring |
| Eye place | Puts each eye at the centre of FLAME's own eyeball, in every shape too | 59.0 mm between the pupils | The eyes sat 55.5 mm apart and stayed behind when the face moved |
| Lid thinning | Scales the height of the lid skin above the eyeball | 0.35 (`LID_THIN`) | Our lids were thick rolls about 4 mm in front of FLAME's, so every eye bulged out of the face |
| Opening size | Makes the eye opening wider and taller | 1.27 and 1.33 (`EYE_OPEN`) | The opening was smaller than real eyes on average |
| Opening place | Moves the eye opening toward the nose | 2.9 mm (`EYE_TO_NOSE`) | The opening sat off the eyeball's centre, so the iris could not be centred |

## The base layer

**What it is.** 300 numbers, one weight per FLAME shape. Each weight is counted in standard deviations and may go from -3 to 3. A prior holds the weights near zero.

**What it sets.** The overall shape of the head: how wide and long the face is, the cheeks, the forehead, the nose as a whole, and where the mouth sits.

**How a photo sets it.** The function is `fit_posed` in `lips_layers.py`. It uses four kinds of evidence.

| Evidence | Detail |
|---|---|
| Landmarks | The nose bridge, nose tip, nose base and the two nostril wings, the two mouth corners, the middle of the upper lip, the bottom of the lower lip, and 6 points of the line between the lips |
| Face outline | How far the face reaches in 57 directions from the point between the pupils, from one temple down around the chin to the other |
| Profile photo | Eight places on the profile's outline, when a profile photo exists. They are the root of the nose, the nose tip, under the nose, the upper lip, between the lips, the lower lip, the hollow under the lip and the chin |
| Pose | The head's tilt and turn are estimated first, from all the face landmarks |

**Rules it follows.**

- The distance between the pupils is held at the base head's. Left free, the fit moved the eyes together by 11% to make a face look wide.
- The eye corners, the lids and the eyebrows are not used as landmarks. The eye landmarks squeezed the face, and the eyebrow landmarks turned low eyebrows into a heavy brow bone.
- The facing points of the two lips are held together, so the mouth stays closed.
- The camera is taken to stand 1.5 m away, so the jaw looks slightly smaller than the cheeks.

**Scores.**

| Measure | Value |
|---|---|
| Face outline, average head | 6.18 mm |
| Face outline, with the base layer | 2.83 mm |
| Line between the lips, front photo only | about 1.0 mm |
| Profile outline from nose root to chin, front photo only | 1.50 mm |
| Profile outline from nose root to chin, with the profile photo | 0.87 mm |
| Outline around the mouth in profile, with the profile photo | 0.59 mm |

**Known faults.**

- The tilt estimate from one front photo is uncertain. A head tilted forward looks much like a short face. Over 105 studio photos the fitted tilt averages about 4.5 degrees with the chin down, which is most likely not real. Where the tilt is large, our head drawn level has its mouth too low. On #006 (8.3 degrees) the line between the lips is several millimetres too low, and on #010 (6.5 degrees) too. Holding the head level in the fit (`LEVEL_HEAD=1`) puts the mouth right on #010 but makes the nose too short on #006, #007, #004 and #012. Neither setting fits everyone, and this is parked.
- The skull above the eyes is the same dome on everyone.
- The nose's width at the wings is now read from the photo itself. Its tip and the groove between the tip and the wings are still the same on everyone. See the nose section.
- How deep the eyes sit is the average for everyone. A front photo holds no depth.

## The jawline part

**Why it exists.** The base layer's shapes are smooth. They round off a square jaw and a pointed chin. After the base layer, the outline was 3.07 mm off around the jaw's angle and 2.21 mm off at the chin, and only 1.54 mm off at the cheeks.

**What it is.** Five shape controls, the same on both sides of the face. The function is `jawline` in `lips_layers.py`.

| Control | What it moves | Range | Average over 105 photos | Differs between people by |
|---|---|---|---|---|
| Jaw width at the angle | The edge of the face below the ears, straight out or in | -8 to 8 mm | -0.4 mm | 2.1 mm |
| Jaw angle | The corner of the jaw, from soft to sharp | -8 to 8 mm | -0.1 mm | 2.6 mm |
| Chin width | Stretches the chin sideways | -8 to 8 mm | +6.4 mm | 3.1 mm |
| Chin point | Draws the middle of the chin down | 0 to 8 mm | 0.0 mm | 0.2 mm |
| Chin height | Moves the chin's lower edge down or up | -8 to 8 mm | 0.0 mm | 1.6 mm |

A sixth control, how far the chin comes forward, is not in this part. The base layer sets it from the profile photo.

**How it moves the skin.** Each control acts around its own place on the jawline. The place is given as an angle from straight down, seen from the point between the pupils. The corner of the jaw lies about 33 degrees from straight down. The edge of the face moves in full. The skin inside the edge follows less and less toward the middle of the face. The neck follows in full. Nothing behind the depth of the jaw's angle moves.

**How a photo sets it.** The five controls are solved so the head's outline reaches as far as the photo's in every direction below the ears. Before each solve, 1 mm of each control is tried on the head to measure what it really does to the outline.

**Scores, over 105 photos.**

| Region | Base layer | With the jawline part |
|---|---|---|
| Jaw around its angle | 3.07 mm | 1.73 mm |
| Chin | 2.21 mm | 1.95 mm |
| Cheeks | 1.54 mm | 1.61 mm |
| Whole outline | 2.83 mm | 2.40 mm |

**Known faults.**

- Chin width is at its limit on 57% of people. The base head's chin is too narrow, or the control is too weak per millimetre.
- Pointed chins are not found. #005 is the test case.
- The detector draws a round line at every chin. On #901 the line on the photo itself is round, where the chin is flat.
- A beard confuses the outline, as on #004.

### The test against true 3D heads

The three Star Wars heads are 3D models, so their true shape is known. Our mesh was wrapped onto each. The test then set the controls from the true shape, with no photo involved, and compared that with what the photo fit makes. The script is `jaw_truth.py`.

| Head | Distance to the true head in the jawline's zone | Over the face |
|---|---|---|
| Base layer set from the true 3D shape | 1.06 mm | 1.28 mm |
| Base layer and jawline controls set from the true 3D shape | 1.08 mm | 1.28 mm |
| Photo fit without the jawline part | 4.33 mm | 3.48 mm |
| Photo fit with the jawline part | 5.32 mm | 3.72 mm |

Three things follow.

- The base layer alone can make these jaws to about 1 mm when it knows the true shape. The jaws then look right, with a defined edge.
- The jawline controls add nothing on these heads. Set from the truth, they stay within 1.5 mm of zero.
- The photo fit is four times further off, and the jawline part makes it worse. From the photos the fit asks for 7 to 8 mm of chin width, where the truth asks for about -0.7 mm.

So the fat, undefined jaws come from how a photo is read, and not from what the controls can do. The jawline part is switched off in the comparison (`JAWLINE = False` in `eyes_layers.py`). The sheet shows the photo and one head.

### The repair of the photo fit

The same test found three faults in how a photo is read. All three are fixed in `scan/lips_layers.py`.

1. **The pupil frame.** A photo is measured from the pupils. The base layer's shapes move the eyes as well as the face, and the fit did not count that. Each shape now counts with what it does to a vertex, less what it does to the point between the pupils, less what it does to the distance between them.
2. **The head's outline.** The head's outline counted skin under the chin and on the neck, which no photo shows as the chin's edge. The outline now leaves out skin whose normal points away from the camera (`FACING = 0.1`) or down (`DOWN = 0.45`). Beside the cheeks and the temples, the detector's outline lies 4 to 6 mm inside the true face on all three heads. Those directions now count 0.3 of the directions around the jaw and the chin (`SIDE_TRUST`).
3. **The profile photo's heights.** The heights read from a profile photo fought the front photo. They now count 0.3, and the depths count in full (`SIDE_HEIGHTS` in `eyes_layers.py`).

The fit also has three new controls for where the eyes sit against the face. They are the eye height, the eye spacing and the eye depth. One unit moves the eyes by 3 mm, and the range is 9 mm either way. Each eyeball moves as one piece with its lids, and the skin around it follows less and less, out to 32 mm from the eye's centre. The function is `free_eyes(rig)`. A front photo sets the height and the spacing. The depth stays at the average unless a 3D shape sets it.

The table gives the mean over the three true heads. The outline column is the gap between our head's outline and the true head's outline as a front photo shows them, around the jaw and the chin.

| Head | Jawline's zone, in 3D | Face, in 3D | Outline in the front view |
|---|---|---|---|
| Average head, nothing fitted | 2.72 mm | 2.49 mm | 7.0 mm |
| Photo fit before the repair | 4.33 mm | 3.48 mm | not measured |
| Photo fit after the repair | 2.56 mm | 2.42 mm | 2.4 mm |
| Fit from the true front outline and landmarks | 2.93 mm | 2.57 mm | 0.9 mm |
| Base layer set from the true 3D shape | 1.06 mm | 1.28 mm | 0.7 mm |

Two things follow.

- The distance in 3D is mostly depth, which a front photo does not show. Even perfect front evidence does not beat the average head in 3D. The front view is what the sheets show, and there the fit is three times closer than the average head.
- The limit is now the detector's outline, which lies about 2 mm from the true edge of the jaw. With the true outline the fit reaches 0.9 mm.

One result is unexplained. When the test reads the profile outline off the true head, the fit gets worse. The real profile photos make it better.

Two corrections were made after this table was written.

- The rule that left out skin facing down was wrong and is off (`DOWN = 2.0`). It let the chin end about 9 mm below the photo's chin.
- The true head of #902 was damaged. The game meshes have about 4,200 vertices each, which is too few to stand for their surface, and the laying-on step went wrong on #902. The three heads were laid on again with `wrap.py --finer 2`. On the clean heads the average head scores 3.81 mm around the jaw, 3.21 mm over the face and 6.06 mm for the front outline. The photo fit scores 3.41 mm, 3.12 mm and 3.94 mm. The fit from the true front outline and landmarks scores 3.29 mm, 3.12 mm and 0.70 mm. The base layer set from the true 3D shape scores 1.35 mm and 1.73 mm. The conclusions above are unchanged.

## The nose

The nose is not a finished part. This section records what was tried on 2026-11-18 and what was found.

### What the base layer does

The base layer sets the nose's size and place. Over 105 photos, the nose's length is off by 1.0 mm and the height of the wings by 0.4 mm. The width at the wings was read from the detector's points, which follow the average nose. It is now read from the photo itself, as the section on the width from a photo explains.

### The nostrils of the base head

The average head of the base layer has shallow pits for nostrils. Our mesh folded into them and got a bright hooked rim on every head. The base head now keeps our own mesh's shape for the 38 vertices inside the nostrils and the 102 skin vertices within 6 mm of them (`NOSTRIL_OWN` in `flame_transfer.py`). The skin around carries them smoothly, in the base head and in all 300 shapes. The inside of the mouth lies within 6 mm behind the nostrils and is left out, or the lips break.

### What was tested against the true 3D heads

The script is `scan/nose_truth.py`. It sets controls from the true 3D shape of the three Star Wars heads and writes `sw3d/nose_truth.png`. Each figure is the mean distance to the true nose.

| Head | Distance |
|---|---|
| Average head | 1.19 mm |
| Base layer | 0.65 mm |
| Plus eight hand-made controls (`scan/nose.py`) | 0.64 mm |
| Plus the page's 25 nose sliders, held firmly | 0.59 mm |
| Plus the page's 25 nose sliders, held loosely | 0.56 mm |
| Plus 32 nose-only shapes cut from the base layer's shapes | 0.52 mm |

- The eight hand-made controls are a failed experiment. Only `nose_zone` from that file is used.
- The page's nose sliders are the 21 sliders of the group "nose" and the 4 sliders named "asym-nose". Held loosely they seemed to draw new creases under the wings. Held firmly they change little.
- The creases were mostly a fault of the test. It gave each end of a slider its own weight, so a fit could push both ends of one slider at once. The page cannot do that. With one value per slider, the creases are gone on #901 and #902, and a faint line stays on #903's right wing. The distance is then 0.61, 0.56 and 0.58 mm, against 0.68, 0.61 and 0.67 mm for the base layer alone. The noses still look like the base layer.
- The user confirmed that the true 3D head in our renderer reads as the person's nose. So shape is enough, and nostril paint matters little.

### What was learned from 15 true heads

On 2026-11-18 the nose was tested on 15 true 3D heads. They are the three Star Wars heads, Lincoln's life mask, the Lee Perry-Smith head and ten people from Meta's Multiface set. The twelve scans were wrapped onto our mesh again with the finer split. Their wraps are in `likeness-fit-data/nose15`.

Each head was scored with what was learned from the other 14 heads only. The table gives the mean distance to the true surface, first on the whole nose and then on the underside of the tip and the joins between the wings and the cheeks.

| What is used | Whole nose | Underside and wing joins |
|---|---|---|
| Base layer | 0.61 mm | 0.67 mm |
| The page's 25 nose sliders, one value each | 0.55 mm | 0.59 mm |
| A fixed correction alone | 0.56 mm | 0.56 mm |
| The fixed correction and the page's sliders | 0.49 mm | 0.50 mm |
| The fixed correction, the page's sliders and 2 learned controls | 0.49 mm | 0.47 mm |

- **The fixed correction** is the average of what the base layer misses on the true noses. It is the same for everyone, and it is now part of the base head (`--nose` in `flame_transfer.py`). It moves the nose by 1.34 mm at most. It removes most of the round lumps that every head of ours had at the nostril corners, and the tip now dips a little below the nostrils in the middle. The user confirmed that the noses look closer to the true ones.
- **The two learned controls** are the two main ways in which the heads' misses differ. One unit of a control is one standard deviation across the 15 heads. They are stored in `likeness-fit-data/nose15/nose_learned.npz` but nothing uses them yet, because no step reads them from a photo.
- **The heads differ in many small ways.** The first three directions hold only 19%, 15% and 11% of the spread between heads. Five learned controls draw ragged lines.
- **The tip's shape is not reproduced.** The user noted that Lincoln's tip is very flat, #902's tip is pointier, and #901's tip is pointier with a bulge just above it. One or two learned controls kept to the tip gain 0.01 mm and change nothing to the eye.
- **Multiface may only be used for non-commercial purposes**, and 10 of the 15 heads come from it. The fixed correction and the learned controls carry that limit.

### The width of the nose from a photo

The face detector's points on the nose wings follow the average nose more than the person's. Over the 15 people of the face sheet, its width varies by 1.4 mm between people, while the width a person can see varies by 3.3 mm. It read #003's nose, about 30 mm wide, as 38 mm.

So the photo fit now reads the width from the picture itself (`scan/nose_width.py`). Beside each wing the skin folds into the cheek, and the fold is a narrow dark line. The finder looks for a line that is darker than the skin within about 1.3 mm of it. It averages that darkness down the wing's height, so skin pores drop out. Of the strong lines it takes the outermost, so the edge of a dark nostril is not taken.

On our head, vertices 1765 (right) and 4101 (left) stay within 0.5 mm of our own visible wing edge on 25 different heads. The fit aims them at the photo's wing edges, sideways only, at three times the weight of a landmark. The detector's wing points keep their height only.

| Visible nose width, our head against the photo, 15 people | Before | After |
|---|---|---|
| Off by, either way | 2.0 mm | 0.7 mm |
| Correlation with the photo | 0.61 | 0.99 |
| How much our widths differ between people (photos: 2.7 mm) | 1.2 mm | 2.6 mm |

The rest of the face did not change. On a photo where the person has turned their head, our head drawn straight on cannot match both wing edges at once, even when the width is right. A nose tip sits about 22 mm in front of the eyes, so a turn of 5 degrees moves it about 2 mm sideways. So the face sheet now draws our head turned left or right as the photo's head is, and not tilted. Drawn that way, our wing edges land within 0.9 mm of the photo's on the three most turned people.

### Can the sliders make the nose's width

This is the test of the goal. For each of the 15 true heads, the base layer, the page's 25 nose sliders (one value each) and the two learned controls were set from the true 3D shape. The nose's width was then compared slice by slice, from 6 to 20 mm above the bottom of the nose. In each horizontal slice the width runs between the two folds where the cheeks turn into the sides of the nose. A fold is where the skin's depth across the face bends most sharply inward.

- On the 10 heads where the true curve is clean, our sliders follow the true width within about 1 mm from the wings up to 16 mm.
- Near the bridge, 18 to 20 mm up, ours stays wider on #902 (5 to 6 mm), #903 (3 to 4 mm) and one scan (2 to 3 mm). On another scan ours is 5 mm too narrow just above the wings.
- On five scans the true width jumps by 10 to 20 mm from one slice to the next, which no nose does. The fold finder fails on those heads, and their numbers are not used.
- The two learned controls were learned from these same 15 heads, so the result is slightly better than it would be on a new head.

A slider for the upper sides of the nose was built to close the gap near the bridge (`scan/nose_upper.py`). It pulls the sides toward the middle, in full from 13 to 23 mm above the bottom of the nose, and fades out toward the wings, the bridge and the cheeks. Set from the true heads it stayed near zero (+0.41, -0.07 and -0.42), and it changed the distance to the true nose by 0.01 mm. The user compared the renders and saw the same widths. So the gap near the bridge is in the fold measure, not in the surface, and the slider was dropped.

What still differs to the eye is the tip. Our tips are broader and rounder, and our noses have no groove between the tip and the wings. That is the next gap for the sliders.

### What failed on the nose

| Attempt | What went wrong |
|---|---|
| Fixing a few of the page's nose sliders | No single slider closes more than 2% of the miss under the tip or at the wing joins |
| Giving the 300 shapes their own movement back around the nostrils | It changed nothing |
| Controls cut from the 300 shapes, kept on the nose | They barely move the underside of the tip. The 300 shapes carry almost no variety there |
| Controls kept to the tip | They gain 0.01 mm |
| A measure of bending between faces, to find creases | The base layer's smooth shapes score as high as the sliders. Creases must be judged on a rendered picture |
| Finding the soft edge of the nose's sides above the wings on photos | The edge is shading, not a crease, and the finder followed it on some photos and not on others |
| Comparing widths in pupil distances on the true 3D heads | The true heads have no eyes of ours. Placed by the eyes of our base layer fitted to them, they came out about 15% too small. Compare true heads in 3D, in millimetres |
| An upper nose width slider | Set from the true heads it stayed near zero, and the renders looked the same without it |

### Two rules for any test against a true head

- A game mesh has too few vertices to stand for its surface. It must be split finer before our mesh is laid on it (`wrap.py --finer 2`).
- Our vertices slide when our mesh is laid on a head. A test must not tie each vertex to the same vertex of the true head. It must let each vertex slide along the true surface (`onto_surface` in `nose_truth.py`). The base layer test in `jaw_truth.py` does not do this yet.

### Lip vertices

The lip paint used to find its vertices by running the face detector on a picture of our own head. Any change to the renderer or the base head moved them and put a kink in the painted lip line. They are now stored in `~/.claude/tools/likeness-fit/lip_vertices.json`.

## The lips

The lips have a shape layer and a paint layer. The paint is set by 14 sliders: six for the upper lip's outline, four for the lower lip's outline and four for the lip colour. A photo sets them through `lips_read.py`, and the main sheet uses them.

**Shape.** There is no lips shape control yet. The base layer places the mouth, and the profile photo sets how far each lip stands forward.

### The upper lip sliders

The upper lip's top edge is a height above the line between the lips, from one mouth corner to the other. The outline is the same on both sides of the middle. The sliders are in `scan/lips_upper.py`.

| Slider | What it sets | Range |
|---|---|---|
| Height | The height at the two peaks of the bow, in mm | 2 to 18 |
| Bow depth | How much lower the dip between the peaks is than the peaks, in mm | 0 to 4 |
| Bow width | The distance between the two peaks, in mm | 6 to 20 |
| Bow sharpness | 0 makes the dip a soft curve, 1 a sharp V | 0 to 1 |
| Sides | The share of the height the lip keeps 45% of the way from a peak to the corner | 0.3 to 1 |
| Corners | The share of the height the lip keeps 80% of the way from a peak to the corner | 0 to 0.6 |

The bow used to look the same on everyone. The paint took the upper lip's edge from the face detector, whose outline follows the average mouth. On 42 photos the detector's bow was 1.2 to 1.9 mm deep, while the bows traced from the lip colour were 0.5 to 2.9 mm deep. The traced upper lips of 103 London photos spread mostly in height (92% of the spread), then in sides (5%), then in corners (2%), then in the bow.

### The lower lip sliders

The lower lip's bottom edge is a depth below the line between the lips. The sliders are in `scan/lips_lower.py`.

| Slider | What it sets | Range |
|---|---|---|
| Height | The height in the middle, in mm | 2 to 22 |
| Bottom width | How far either side of the middle the lip keeps its full height, as a share of the way to the corner. It does not go below 0.1, so the bottom never comes to a point | 0.1 to 0.5 |
| Sides | The share of the height kept 45% of the way from the end of the bottom to the corner | 0.3 to 1 |
| Corners | The share of the height kept 80% of the way from the end of the bottom to the corner | 0 to 0.7 |

The lip thins out into each corner. Halfway between the corners slider's place and the corner, it keeps 35% of the corners slider's height. The lower lips of 103 photos spread in height (92%), sides (5%) and corners (2%). Bottom width barely matters on real people, but it lets a designer make a square lower lip.

### The lip colour sliders

Each colour slider is a difference from the person's own skin in CIE Lab, where 1 step is about the smallest difference a person can see. So the lips follow the skin when the skin changes. The sliders are in `scan/lip_colour.py`.

| Slider | What it sets |
|---|---|
| Lightness | The lower lip's lightness minus the skin's |
| Redness | The lower lip's red against green minus the skin's |
| Warmth | The lower lip's yellow against blue minus the skin's. Positive is coral or brown, negative is pink or mauve |
| Upper lip shade | The upper lip's lightness minus the lower lip's |

The upper lip's redness is the lower lip's minus 3.9, and its warmth is the same. On 103 photos the lip colours spread from pale to deep (72%), then warm against cool (12%), then in the upper lip's shade (8%). The upper lip is on average 10 steps darker than the lower lip, because it faces down. The skin colour is the cheek below the eyes, which has no stubble.

### How a photo sets them

1. The upper lip's top edge is traced from the photo's redness. The edge is the path across the mouth that follows the strongest rise of redness while bending little (`lip_edge.py`).
2. The lower lip's bottom edge is soft. It is traced from redness plus pink against yellow, only where the colour is about halfway between lip and skin. Bends cost more, and every mm away from the detector's lower edge costs 0.3.
3. The shape sliders are fitted to the traced edges, out to 85% of the way to each corner.
4. Our head is drawn once with those sliders. The renderer's paint mask (`lipMask`) shows exactly where the paint lies. Our head draws the lower lip's edge 0.45 to 0.6 mm lower than asked, at any thickness, and a thin upper lip about 0.3 mm short. The cause is not found. The sliders are fitted again with the difference taken off.
5. Our skin is set so that our cheek measures as the photo's cheek. Then the lip colours are set so that each lip's difference from our cheek measures as on the photo. Light scales each colour channel, so each step is a division, repeated three times.

### Scores

On the 15 sheet people, with our heads' edges read from the paint mask:

| | The old paint of 22 heights | The sliders |
|---|---|---|
| Upper lip outline | 0.89 mm | 0.52 mm |
| Lower lip outline | 1.07 mm | 0.33 mm |
| Lower lip colour against the skin | 13.8 Lab steps | 0 |
| Upper lip colour against the skin | 21.5 Lab steps | 2.8 |

On the photos alone, against the traced edges of 103 people, the upper lip sliders miss by 0.34 mm against 0.84 mm for the old paint, and the lower lip sliders by 0.29 mm against 0.79 mm. With a profile photo, how far each lip stands forward is off by about 0.5 mm. From a front photo alone it was off by 1.5 to 2.1 mm.

### Known faults

- The lips are flat and matte. Photos show shine and texture.
- Lopsided bows, such as #901's and #020's, can only get a symmetrical one.
- #003's upper lip trace is wrong on one side, so its upper lip does not improve.
- #006's lower lip is fuller in 3D than her upper lip. A front outline cannot show how far a lip pushes forward or how round it is.
- On tilted photos the mouth sits too low on our level head. See the base layer's known faults.
- The renderer's 0.5 mm offset on the lower lip is corrected, not explained.

### What failed

- Moving the skin until a fixed painted lip matched the photo's outline gave a raised muzzle and dark smears. The paint, not the shape, carries most of a lip's character.
- Five upper lip sliders without a corners slider put "bow width" on the lip's shoulders and left a kink at each peak.
- Tracing our own renders by colour. On our heads the tracer follows the lip's 3D shading as well as the paint, and it read the lower lip 0.7 mm too tall. A correction built on that reading made thin lips thinner. Our renders are measured from the paint mask.
- Matching lip colour with the same peach skin on every head. The same difference from the skin turned #002's pink lips red-orange and #006's mauve lips pale pink.

## The eyes

The script is `eyes_layers.py`. The eye part has shape controls for the opening and paint for the iris and the caruncle.

### Shape of the eye opening

| Control | Count per eye | What it does | Range |
|---|---|---|---|
| Opening width | 1 | Makes the opening wider or narrower | 0.75 to 1.3 times the base head's |
| Opening height | 1 | Makes the opening taller or shorter | 0.75 to 1.3 times the base head's |
| Opening place | 1 | Moves the opening sideways around the eyeball | -4 to 4 mm |
| Lid curves | 14 | Move the lid edge up or down at 7 places per lid, which are the two corners and five places between them | -2 to 2 mm each |

**How it moves the skin.** The edge of the lids stays on the eyeball where the eyeball reaches. Near the two corners the lids leave the eyeball, because an eye opening is wider than the eyeball. The skin within 24 mm of the eye follows smoothly. The functions are `stretch_opening` in `eye_targets.py` and `lid_curves` in `eyes_layers.py`.

**How a photo sets it.** The detector's 16 points per eye give the photo's outline. Our own opening is measured on the rendered picture, as the area where the eyeball shows. The size and the place are set first, and the lid curves last, over 10 rounds.

**Scores, over about 105 photos.**

| Measure | Value |
|---|---|
| Lid outline against the photo's | 0.30 mm |
| Width between the lid corners | 0.00 mm |
| Opening height | about 0.3 mm |
| Lid curve numbers at their limit, of 28 per person | about 6 |

### Iris and gaze

| Control | How a photo sets it |
|---|---|
| Iris size | The detector's four points on the rim of the iris. The photos average 12.0 mm |
| Pupil size | The pupil's width as a share of the iris's, where the brightness is halfway between the pupil's and the iris's. The London photos give 0.35 (0.30 to 0.47). Where the pupil cannot be told from a dark iris, it is 0.35. It used to be 0.47 for everyone |
| Iris colour | The iris's colour against the brightest white of the same eye, which stands in the same light, so the photo's exposure and tint cancel out |
| Iris rim | How dark the ring at the iris's edge is drawn. It is 0.6 for everyone, because at the photos' size, about 19 pixels from the centre to the rim, blur from the white hides it |
| Gaze | Not a control. The iris always sits in front of the eyeball's centre, so the eye looks at the camera |

Each eye is read in rings from the iris's centre, using only the pixels inside the eye opening and leaving out the brightest 10% of each ring, which is the lamp's reflection (`scan/iris.py`). The detector's iris outline can be too large, as on #901, where the real iris ends at about 0.65 of it. So the iris's edge is taken from the brightness, halfway between the iris and the white, and the pupil and the colour are read against that edge. Our iris's size still follows the detector, so #901's irises are drawn too large. Light falls differently on our iris than on our white, so the colour and the pupil are set by drawing our head, measuring it the same way, and correcting three times. Before this, #005's iris was 3 times too bright against the white. Now our irises' brightness against the white is within 0.002 of the photos' and the pupils within 0.03 of the iris's width. `OLD_IRIS=1` brings back the old reading.

### Caruncle

The caruncle is the pink flesh in the inner corner of the eye. It is painted on the eyeball. The white of the eye ends in a round end 2.8 mm short of the inner corner, and the caruncle fills the corner around that end. Its colour is the skin colour with green and blue at about 85%.

The eye opening is wider than the eyeball, so near the inner corner the lids leave the eyeball and their inner faces show. They used to show as a light wedge of skin, as on #003's left eye. The lids' inner faces within 6 mm of the inner corner, behind it, now get the same pink (`canthus` in the renderer; `CANTHUS=2` shows them in green).

### Inner fold

The inner fold is a fold of skin over the inner corner of the eye, as on #005. It is a slider from 0 to 1 (`set_inner_fold`, `INNER_FOLD`). At 0 the pink corner shows. At 1 the fold hides it: the white runs to the corner, and a soft shadow carries the upper lid's line 3.5 mm on toward the nose and 1.5 mm down.

A photo does not set it yet. Measuring how much pink the inner end of the opening shows failed: it ranked #005 among the pinkest, because on a narrow eye the measured area takes in lid skin.

### Upper lid fullness and eye depth

Two 3D sliders, the same on both sides of the face (`scan/upper_lid.py`, `UPPER_FULL` and `EYE_DEPTH`).

| Slider | What it does | Range |
|---|---|---|
| Upper lid fullness | At 1 the skin between the upper lid's edge and the brow comes 3 mm forward, most about 9 mm above the eye's centre. At -0.5 it goes 1.5 mm back and deepens the hollow under the brow. The painted socket shade fades as the lid fills | -0.5 to 1 |
| Eye depth | At 1 the eyeball, the lids and the skin within 15 mm of the eye's centre come 2 mm forward, fading to nothing 25 mm away | -1 to 1 |

Each slider's strength is set by place and then blended with the neighbouring vertices' strength 60 times, so it has no edge, and the skin moves along its own surface direction. The eyeball and the lid's edge do not move with fullness. Below -0.5 the skin where the lid folds into the eye collapses into a sharp crease. The sliders show best in profile; from the front, under studio light, depth barely shows. A photo does not set them yet. The London profile photos show the lid's depth behind the brow.

### What failed on the upper lid

- Learning upper lid sliders from the 15 true heads. Our 300 shape sliders were fitted to each scan with and without the upper lid area, and the main patterns of the difference were learned. On heads they had not learned from they did not help: 2.19 mm before, 2.07 mm after. In this band the Multiface scans record eyebrow hair and lashes as surface, and a head's left and right eyes differ by 2 to 3 mm. A first result of 0.80 mm, from fitting all 300 sliders to the same scans, measured how well they follow that noise, and is not to be used.
- Moving the skin straight forward with a strength that stops abruptly, at a line behind the eye's centre, at the eyeball and just above the lid's edge. It left a ridge under the brow and a notch in the profile.

### White of the eye

The white is slightly grey, and it darkens under the upper lid. It has no control.

### Known faults of the eyes

- Dark irises look lighter than on the photos, although they measure the same. On the photos, dark lid edges, lashes and shadow frame the iris, and ours have none.
- There is no crease above the eye. #005's smooth lid and #004's deep crease look the same.
- In profile the upper lid's edge has a small pointed hook, most on #004. It comes from how the lid edge is fitted to the opening.
- The inner fold and the upper lid sliders are not read from photos yet.
- How deep the eyes sit is the average for everyone.
- The fold of the upper lid is the same on everyone.
- The detector's outline is smooth near the corners, so our corners follow the detector and not quite the person.

### What failed on the eyes

| Attempt | What went wrong |
|---|---|
| The four lid sliders of the Likeness Lab page | They turn a wide patch of skin, and long pushes made the lids and the skin beside the eye bulge. They are switched off |
| Running the detector on a picture of our own head | It does not follow our lids. Sliders set from it ran to their limits |
| Moving the iris sideways to match the photo | An iris off the eyeball's centre is a turned eye. The eyes looked past the viewer |
| Keeping the lid corners on the eyeball | The corners could never be further apart than the eyeball is wide |
| Scaling all skin near the eye by distance | It swung skin across the nose and wrecked the face |

## The lashes

The lashes are paint, drawn from the lid edge of the head. They follow the eye's size and the lid curves. The renderer function is `set_lashes`.

| Control | What it does | Range |
|---|---|---|
| Upper lash line | How dark the line on the upper lid edge is | 0 to 1 |
| Upper lashes | How far the fringe reaches above the lid | 0 to 1 |
| Lower lash line | How dark the line on the lower lid edge is | 0 to 1 |
| Lower lashes | How far the fringe reaches below the lid | 0 to 1 |
| Lash colour | Planned. It is one dark brown for everyone now | |

Both eyes share the values. The fringe is longer toward the outer corner, and the line stops before the caruncle.

**How a photo sets it.** The script measures how dark the skin is along each lid edge, and how far out the dark band reaches. Our render's upper line follows the photos with a correlation of 0.85, and the upper reach with 0.72.

**Known faults.** The lower lid measure counts the shadow under the eye as lashes and is not trusted. The lash colour is not set. Eyeliner cannot be told from a strong natural lash line.

The lashes were left at this state on purpose.

## The eyebrows

The eyebrow is hair on a ridge. The script is `brows.py`.

**The ridge.** The brow ridge is a line on the bone above each eye. It is a fixed set of the head's own vertices, found once on the base head as the most forward line of the skin 11 to 32 mm above the eye's centre. It moves when the head's shape changes. The two sides use mirrored places.

**The hair.** Each eyebrow is seven numbers counted from the ridge, plus its two ends, its colour and two grooming settings. All seven at zero give the average eyebrow of the test photos.

| Group | Control | Meaning | Differs between people by |
|---|---|---|---|
| Line | Height | How far above or below the ridge the hair runs | 2.5 mm |
| Line | Tilt | How much higher the outer half is than the inner half | 1.7 mm |
| Line | Arch | How much the hair rises in the middle | 1.8 mm |
| Line | Peak | Whether the arch peaks toward the nose or toward the temple | 1.9 mm |
| Thickness | Overall | How thick the eyebrow is | 1.2 mm |
| Thickness | Taper | How much thinner the outer half is than the inner half | 1.2 mm |
| Thickness | Belly | How much thicker the middle is than the ends | 1.8 mm |
| Ends | Inner end and outer end | Where the hair starts and stops, from the middle of the face | 1.2 mm and 1.9 mm |
| Colour | Colour | A share of the skin's colour | |
| Grooming | Lower edge trim | How cleanly the underside is cut, from 0 to 1 | |
| Grooming | Inner end fullness | Whether the inner end is as full as the middle, from 0 to 1 | |

The left and the right eyebrow have their own numbers, so uneven brows stay uneven. An eyebrow ends before the skin turns to the side of the head.

**How a photo sets it.** The detector only says roughly where the eyebrow is. The script reads the photo's pixels at 11 places along each eyebrow and finds the dark band against the forehead. The band gives the middle height, the thickness and the colour. The seven numbers are then fitted to the 11 places. They leave 0.19 mm of the height unexplained.

**Scores.** Our render's thickness follows the photos with a correlation of about 0.8 to 0.9, and its darkness the same.

**Eyebrows that cannot be read.** A very thick, very high or very faint band is refused, because it is hair over the forehead. So is a pair whose heights differ by more than 4.5 mm. A refused eyebrow copies the other side, or becomes the average eyebrow. #003, who has a fringe, is the test case.

**What a study of 204 eyebrows showed.**

- Men's eyebrows sit 3.5 mm lower than women's, arch 1.1 mm less and are 0.6 mm thicker.
- An eyebrow's line needs 4 numbers and its thickness 2 or 3.
- From a front photo, our ridge is close to the average ridge and does not predict where the hair sits. So a photo is read from the pupils and then converted into ridge settings.

**Not built.** An expression setting that separates a raised brow from a high one. Storing the hair on the skin, so it is right on a turned head. A ridge set from real depth.

## Test settings of the renderer

The test renderer is `glrender.py`. It copies the Likeness Lab page's paint. Every addition is off by default, so older tools render as before.

| Setting | Page value | Test value | Reason |
|---|---|---|---|
| Light | From above | Even, from the camera (`set_flash(1)`) | The photos are studio portraits |
| Camera distance | 0.62 m | 1.5 m | A portrait camera stands further away |
| Eye socket shade | 0.5 | 0.15 | The page's shade hid the lids |
| Lid liner band | 1.0 | Replaced by the lashes | The band ran into the iris and the eye looked half closed |
| Lips | One fixed shape | Painted from an outline of up to 65 rows (`set_lips`), with their own upper lip colour and full cover (`set_lip_colour`) | The bow needs finer rows than 33, and deep or red lips cannot be reached when the lip colour is mixed at 44% into the skin |
| Paint mask | None | `lipMask` draws only where the lip paint lies and where its line between the lips runs | To measure our lips exactly |
| Skin colour | One fixed colour | Set to the photo's cheek on the face sheet | Lip colour is a difference from the skin |
| Iris place | Set by the page | In front of the eyeball's centre (`gl.iris_shift = 1e-9`) | Without it the drawn pupils sit about 1.3 mm inside the eye centres |
| View on the face sheet | Straight on | Turned left or right as the photo's head is, not tilted | A nose tip sits about 22 mm in front of the eyes, so a turned photo cannot match a straight head |
| Eyebrows | One stroke with three points | Painted from the ridge model (`set_brow_paint`) | |

## Test data

| Set | Content | Licence | Place |
|---|---|---|---|
| Face Research Lab London Set | 102 people, each with a neutral front photo, a smiling front photo and a left profile | CC BY 4.0 | `~/.claude/tools/likeness-fit-data/london/` |
| Star Wars heads | Han Solo (#901), Luke Skywalker (#902) and Leia Organa (#903), game heads photographed in the London set's format | Test use only | `~/.claude/tools/likeness-fit-data/london-sw/` |
| Scans | The Lincoln life mask, the Lee Perry-Smith head and ten people from Multiface | Mixed, test use only | `~/.claude/tools/likeness-fit-data/scans/` |
| The 15 true heads for the nose | The twelve scans laid onto our mesh again with the finer split, the nose's fixed correction and its two learned controls | As above | `~/.claude/tools/likeness-fit-data/nose15/`, with the Star Wars heads in `sw3d/` |
| FLAME 2023 Open | 300 identity shapes | CC BY 4.0 | `~/.claude/tools/likeness-fit-data/flame/` |

The London set is at https://figshare.com/articles/dataset/Face_Research_Lab_London_Set/5047666.

Shapes derived from FLAME 2023 Open may ship with attribution. The licence bans pornographic use and fake or defamatory content. Nothing from Multiface or from the Star Wars heads may ship.

## Scripts

All scripts are in `~/.claude/tools/likeness-fit/`. They run as `cd ~/.claude/tools/likeness-fit && .venv/bin/python -I <script>`.

| Script | Role |
|---|---|
| `scan/flame_transfer.py` | Builds the base head and the 300 shapes on our mesh |
| `scan/lips_layers.py` | The base layer fit, the face outline, the jawline part and the old lip paint |
| `scan/lips_upper.py`, `scan/lips_lower.py`, `scan/lip_colour.py` | The upper lip, lower lip and lip colour sliders |
| `scan/lip_edge.py` | Traces the lips' outer edges on a photo from their colour |
| `scan/lips_read.py` | Reads the mouth sliders from a photo and sets them on our head, with the skin colour |
| `scan/lips_profile.py` | Finds the outline and the eight places of a profile photo |
| `scan/eyes_layers.py` | The eyes, the lashes, the eyebrow reading, and the comparison sheets |
| `scan/brows.py` | The eyebrow model: the ridge and the seven numbers |
| `scan/brow_ridge.py` | The study of how eyebrows sit against the ridge |
| `scan/jaw_truth.py` | The test of the base layer and the jawline part against heads whose true 3D shape is known |
| `scan/nose_truth.py` | The nose test against the three Star Wars heads |
| `scan/nose_learned.py` | Learns the nose's fixed correction and two nose controls from the 15 true heads |
| `scan/nose_width.py` | Finds the visible edges of the nose wings in a picture |
| `scan/nose_upper.py` | The dropped upper nose width slider, kept for later |
| `scan/trials-2026-11-18/` | The tests of 2026-11-18, with their data: the slider capacity test, the lined-up comparison sheets and the mouth and width measures |
| `eye_targets.py` | `stretch_opening`, the size and place of an eye opening |
| `glrender.py` | The test renderer |

The main comparison is one command.

```
D=~/.claude/tools/likeness-fit-data
.venv/bin/python -I scan/eyes_layers.py $D/flame/targets.npz $D/lips london_faces \
  $D/london-sw/neutral/neutral_front $D/london/neutral/neutral_front --face
```

It writes three files into `~/.claude/tools/likeness-fit-data/lips/`.

- `london_faces.png` shows 15 faces, each as the photo beside our head. The three Star Wars heads come first.
- `london_faces_side.png` shows the same people in profile.
- `london_faces.json` holds every number for every photo.

Adding `--overlay` draws the detector's face outline in green on both pictures. Without `--face`, the sheet shows only the eyes and eyebrows.

Switches set as environment variables change the sheet without changing the code.

| Switch | Effect |
|---|---|
| `SHOW_STRAIGHT=1` | Draws our head straight on instead of turned like the photo |
| `SHOW_POSED=1` | Draws our head with the photo's tilt as well as its turn |
| `LEVEL_HEAD=1` | Holds the head level in the fit and fits its turn only |
| `DETECTOR_WINGS=1` | Reads the nose's width from the detector's wing points, as before 2026-11-18 |
| `SAVE_HEADS=<folder>` | Saves each fitted head |
| `SAVE_SHOTS=<folder>`, `GL_SIZE=2048` | Saves the full picture of each of our heads, drawn larger |
| `OLD_IRIS=1` | Reads the iris colour and pupil size as before 2026-11-18 |
| `INNER_FOLD`, `UPPER_FULL`, `EYE_DEPTH` | Set the inner fold (0 to 1), the upper lid fullness (-0.5 to 1) and the eye depth (-1 to 1) by hand |
| `CANTHUS=0`, `CANTHUS=2`, `EYE_DEBUG=1` | Leave the lids' inner faces as skin, or show them in green; draw the eyeball in green |
| `OLD_LIPS=1` | Paints the lips from the old 22 heights and the page's lip and skin colours |
| `LOWER_PULL`, `LOWER_BEND`, `LOWER_TINT` | How the lower lip's edge is traced: the pull toward the detector (0.3), the cost of bending (0.5), and how much pink against yellow counts (2) |
| `MOUTH_DEBUG=1`, `WING_DEBUG=1`, `OUTLINE_DEBUG=1` | Print or draw how far the fitted head misses the photo |

The sheet's own line "nose width at the wings" still compares the detector's points with fixed vertices, and its numbers no longer mean anything.

The base head is rebuilt in two steps, in this order. Without `--nose` it loses the nose's fixed correction.

```
.venv/bin/python -I scan/nose_learned.py $D/flame/targets.npz $D/nose15/nose_learned.npz $D/sw3d $D/nose15
.venv/bin/python -I scan/flame_transfer.py $D/flame/download/flame2023_Open.pkl $D/flame --nose $D/nose15/nose_learned.npz
```

## Lessons that apply to every part

1. **Paint and shape are separate layers.** A photo's lip edge, eyebrow and lash line are colour. Pushing skin to match them breaks the face.
2. **Never calibrate the detector on a picture of our own head.** It does not follow our lids, and it draws our jaw 7 mm outside the head. This mistake was made twice.
3. **Check geometry without paint and from the side.** The eye bulge was invisible in painted front views and obvious in a plain side view.
4. **A good score can look wrong.** The jawline version with the best chin score gave chins a dent. Every change is checked on the sheet.
5. **Check the sheet's timestamp.** A naming clash once left a stale sheet in place for several runs.
6. **A steady error across all people is a fault in the base head.** Sliders running to one side, or a correction that averages far from zero, should be built into the base head.
7. **The outline of a face is its furthest point in each direction.** A corner that moves out also becomes the edge for the directions beside it. What a control does to the outline must be measured, not assumed.
8. **The detector does not follow our nose either.** On our renders it read our noses as narrower than the photos when they were wider. Measure our renders from the picture or from the geometry.
9. **Draw every measure before trusting its numbers.** Edge finders on photos failed several times in ways only a picture showed: on cheek shading, on pores, on a nostril, on glossy lips. A sign error in the picture code once put every height 6 mm off.
10. **Show the user straight, continuous lines.** Pictures are scaled and levelled so the pupils sit in the same place. Each head shows only its own lines, and the lines run on over the photo. Wing edges and other small features are judged on full-size crops, not on the small sheet tiles.
11. **Test the sliders against 3D, not photos.** Whether a slider can make a face is a question for the true 3D heads. A face that looks wrong on the photo sheet may be a photo reading fault.
12. **Everything goes through sliders.** A feature that only a photo fit can produce does not prove that the character designer can make the face. When the sliders cannot make something, the sliders are added to or changed.
13. **Measure our own renders from what was drawn, not by tracing them.** A colour tracer tuned on photos also follows our heads' 3D shading. The renderer can draw a mask of its own paint, and that is exact.
14. **Compare colours against the person's own skin, and match the skin too.** A lip colour copied as a difference from the skin looks wrong on a skin of a different colour.

## Open work

| Item | State |
|---|---|
| Nose tip and the groove between the tip and the wings | The same on everyone. Lincoln's flat tip and the pointier tips of #901 and #902 are not reproduced. Next for the sliders |
| Nose sliders read from a photo | Only the width at the wings is read. The page's nose sliders and the two learned controls are not set from photos |
| Mouth height on tilted photos | Parked. Neither the fitted tilt nor a level head fits everyone |
| Lips | Outline and colour are set by sliders. Shine, 3D depth and fullness are next |
| Skin colour | Set to the photo's cheek on the face sheet, as three numbers. No skin sliders yet |
| Hair | Not started in this work |
| Chin width in the base head | The average of +6.4 mm should be built in |
| Pointed chins | Not found by the fit |
| Skull above the eyes | The same on everyone |
| Eye depth and upper lid fullness | Sliders exist. Not read from photos yet; the profile photos show them |
| Side sheet | The photo and our head are not at the same scale |
| Pupils of the true 3D heads | Unknown, so the true heads cannot be compared in pupil distances |
| Base layer test in `jaw_truth.py` | Still ties each vertex to one vertex of the true head instead of letting it slide |
| Scan tests on FLAME | Not rerun since the base head changed. The old results were 0.72 to 0.82 mm |
| Eyebrow and lip paint on a turned head | Drawn for the front view only |
| Likeness Lab page and the game | Unchanged |

## Deferred until the KOTOR shape match is proven

On 2026-11-18 the user decided to get as close as possible to the KOTOR heads first, and to defer two pieces of work until then. The first is reducing the head's vertex count. The second is naming the sliders. Both plans are recorded here so they are not lost.

### Reducing the vertex count

A test reduced our mesh and scored FLAME's fit on the 57 KOTOR heads (`kotor_vertices.py`, results in `likeness-fit-data/kotor-heads/kotor_vertices.json`). The skull was not held in this test.

| Vertices | Lost by reducing the mesh alone | FLAME's fit over the face |
|---|---|---|
| 4,553 (full) | 0 mm | 0.66 mm |
| about 2,900 | 0.17 mm | 0.66 mm |
| about 1,450 | 0.33 mm | 0.69 mm |
| about 690 | 0.45 mm | 0.75 mm |

The candidates for the shipped head are our current mesh reduced, FLAME's own mesh reduced, and a new head of KOTOR's size (600 to 1,000 vertices) with the sliders carried onto it. The texture layout follows the mesh. The choice is made with an in-engine talking-head test of the one or two candidates left.

### Editing FLAME with named controls

FLAME's 300 weights each change the whole face in a way nobody can name, so a player cannot use them. The plan is a layer of named controls, each one a fixed direction in FLAME's weight space.

- **Measured directions.** A direction that changes one measure people understand, such as nose length, eye spacing or jaw width, and changes the other measures as little as possible.
- **Local directions learned from data.** Sparse localised components (Neumann and others, 2013) learned from the KOTOR heads. Each one moves one small region, such as a cheekbone or the brow ridge, and is named by hand.
- **Our own part controls** where they already work: the nose sliders, the lid curves and the lip outlines.

The controls would be grouped from coarse to fine. The whole head comes first (size, face length and width, how far the face sits forward). The regions come next (forehead, brow ridge, cheeks, jaw and chin). The features come last (eyes, nose, lips and ears). Asymmetry between the left and right sides is an extra layer that is off by default. About 60 to 80 controls in all.

FLAME can tell how likely a face is. The editor could use that to soften a control as the face nears the edge of what FLAME can make, so stacked controls do not produce a grotesque face. Dragging the face directly (the "clay" input) works on the same weights: the tool solves for the weights that move the dragged point there and change the rest as little as possible.

The proof has two steps. First, the named controls alone, learned on some KOTOR heads and tested on the others, must come close to what FLAME's raw weights reach (0.68 mm on 2026-11-18, against 1.92 mm for today's named controls). Second, the user rebuilds a few KOTOR characters with the controls in a prototype, and the result and the time it took are measured.

One question is open. A player could start from the average face, or pick one of the KOTOR heads as a starting point and edit from there. Picking a starting head is not the preset blending that was rejected, but it has not been decided.
