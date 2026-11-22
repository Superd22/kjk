# Designing head measures and sliders

This guide was written on 2026-11-21. It holds the method, the rules and the traps for building a measure or a slider for kjk's head customization. Every rule comes from something that happened while the nose, the jaw and chin, the brow, the cheeks, the eyes, the lids and the paint were built between 2026-11-17 and 2026-11-21. Each rule gives its reason and, where one exists, a real example.

Read it before you build a measure, a slider, a construction or a solve for any part of the head. The plan itself is the roadmap, `2026-11-19-head-roadmap.md`. Its decisions win if this guide ever disagrees with them. The last section is a checklist to follow when you start a new feature.

Some words used throughout:
- **David** is the user. He judges likeness and fun.
- **KOTOR's heads** are the 57 human head meshes of KOTOR 1 and 2 (and a few mods) that serve as the test set.
- **Live** is what viewer 2 shows today.
- **A stand-in** is a head made only from slider values, solved to look like one KOTOR head.
- **A construction** is the code that builds a part of the mesh from slider values, such as the eye opening built from a curve on the eyeball.
- **A judge** is a fresh sub-agent that only looks at pictures and scores them.

## 1. Measures

A measure turns a head into numbers that say whether a feature looks like its target. Everything later depends on it. The solver matches whatever the measure reads, and nothing else. A wrong measure gives good numbers on heads that look wrong.

### Measure features and shapes, not millimetres

Score each feature by its proportions and its shapes. A proportion is where the feature sits and how big it is, as a ratio or a named length. A shape is an outline seen from the front, the side or below, compared after its position and size are taken out. An average surface distance in millimetres is not a likeness measure.

Why: an average distance does not prove that a nose reads as the same nose. A small miss on a cheek or the forehead goes unnoticed, while a small miss on a lid edge changes the face.

Example: on 2026-11-19, six eye rounds improved their millimetre numbers each time while the pictures did not improve. David said that earlier agents "gave me mm differences but we never proved we were measuring the right things". This became decision 49. Later, on the eyes, he said "it's not just about a number, it's mm^2 but of the right SHAPE, like that's what the human sees, shape". The profile area of the eyeball was then replaced by the outline of that area, which passed the four checks.

### Measure exactly what the person sees

Read the feature with every part that muddies it switched off, on KOTOR's head and on ours alike. For the eyes, that means the lid pieces on both sides, the lash strip, the caruncle and the iris paint are all off. The measure reads the opening's outline, where the eyeball comes through the skin, and the skin round it.

Why: a part that sits over the feature changes the reading without changing the shape. The solver then fits that part instead of the feature.

Example: a whole day of eye rounds was steered by lid pieces, the caruncle and the way the iris was drawn. On 2026-11-19 the lid edges had been fitted to the outer edge of KOTOR's eye paint, which made our bare eyes stare. On 2026-11-21 David said that the lid piece "only muddies the waters and can be copied independently after we have the eye shape nailed". The eye shape measure of that day leaves out the lid pieces, the lash strip and the caruncle on both heads, and it was the first eye measure that worked.

### Read the measure on KOTOR's own meshes

Read KOTOR's side of every measure on KOTOR's own mesh, with its own eyeball and its own skin. Do not read it on our fit of that head. When hair or cloth covers the feature on KOTOR's mesh, hide those rows and say which heads lost them.

Why: a fit is already an answer. Measuring the target on our own fit hides exactly the misses the measure exists to find.

Example: the jaw measure hides KOTOR vertices that are not skin and lie more than 6 mm outside our fit. That hid parts of the jaw on Kreia, pfha06, pmha06 and four mod heads. Malak's metal jaw was refused outright and left out of the scorecard and the ranges.

### Check any claim about KOTOR on all 57 meshes first

Before you propose or brief a change to a construction, measure the change's premise on KOTOR's 57 meshes. Put that measurement in the brief as "step 0". Look at its numbers before building anything.

Why: a large change built on a wrong premise costs a whole conversation. A step 0 costs minutes.

Example: the night diagnosis of 2026-11-21 said, among its traps, that KOTOR's eyeballs sit 5 to 10 mm further back than ours. The HQ briefed a rebuild around a larger eyeball set further back. KOTOR's own eyeball meshes showed that pmhc04's eyeball sits 1.5 mm in front of ours, Bastila's sits in the same place, and the median over all heads is only 0.8 mm further back. The claim had come from the "ball" measure, which reads KOTOR's eyeball front as a radius about our centre. It was never KOTOR's real eyeball. The next brief, corners on the eyeball's equator, ran a step 0 first. It found that most of our outer corners already sit there and that KOTOR's inner corners do not, and it stopped cheaply (decisions 64 and 65).

### Prove a measure with the four checks

A measure is proven when it passes four checks (decision 49). Prove it before anyone fits to it, and say in the report how it was proven.
1. **It ignores what nobody notices.** The same head measured twice, or a head's left side against its right, must differ far less than two different heads do.
2. **It sees what David notices.** When David names a fault, the measure must move with it. The smallest step David notices becomes the pass mark.
3. **It tells KOTOR's heads apart.** For most heads, a head's nearest match must be itself (for example, its right eye matched against every head's left eye).
4. **Its extremes look right.** Draw the heads it calls most alike and most different, and the best and worst fits, and look at them.

Why: an unproven measure gives numbers with no meaning. A solver fitted to it is a waste.

Example: the eye shape measure of 2026-11-21 was proven this way. A KOTOR head's left eye against its own right eye differed by a median of 0.23 mm on the curve. Two different heads differed by 3.22 mm, so the curve is 14 times more sensitive to the head than to the side. For 48 of 54 heads, the right eye's nearest match was the head's own left eye. Its overview sheet showed the curve tracing KOTOR's bare eyeball on every head except three mods, which were left out of the proof. The jaw shows the other side of this check. Its "chin point shape" was dropped because KOTOR's spread on it was 2.7 mm, which is smaller than the measure's own wobble.

### Measure relations, and add measures when the solve cheats

Measure the relations between features as well as the features, such as the skin beside the nose against the eyeball's front. When a fit matches every measure and still looks wrong, the measures are blind to the fault. Add the measure that reads the fault.

Why: the solver is faithful to the measures it is given. A fault that no measure reads is free for the solver.

Example: on pmhc04 the eye solve matched 27 proportions almost perfectly. It matched the width (28.2 mm against KOTOR's 26.6), the opening height (7.0 against 7.4) and the rim depth (12.1 against 12.1). But none of the 27 read the eyeball's front against the skin beside the nose, the eyeball seen in profile, the outer corner's depth or the lid piece's reach. To match them, the solve set Eye depth to -4.83 and Eyeball size to +2.70. That pushed the whole eye 4.8 mm back into the head, and the inner corner looked buried. David saw it at once in the pictures.

### Traps in measures

- **Lining shapes up on the wrong centre.** A shape measure lined up on the eyeball's centre is right for comparing shapes, but wrong for a fit. Our eyeball centres sit 1 to 8 mm from KOTOR's. Version 4 of the eye handles was fitted to KOTOR's outline moved onto our centre, so its openings sat in the wrong place in the face. On pmha02 the eye sat 3.5 mm too high while the brow, the nose and the ears lined up. The refit measured in the head's own frame, and the median gap to KOTOR's opening over 57 heads fell from 2.57 mm (version 4) to 1.00 mm. Overlay pictures have the same trap: draw KOTOR's curve moved to our centre when the measure scores it that way, or the picture shows a gap the measure does not score.
- **Pictures that draw a part in the wrong place.** The old sheets drew every iris, KOTOR's included, at 0.42 radians round the eyeball's forward axis. Viewer 2 paints its own iris, 11.8 mm wide by default and centred in the opening. So the iris measures matched the sheets the judges saw, not viewer 2. Before you measure a painted or drawn part, check which drawing of it the measure reads.
- **KOTOR's low-poly meshes.** KOTOR's eyeballs have only 16 to 33 triangles. Their vertices lie on a sphere, but their faces lie well inside it. A corner at 0.89 radii to the side lies 0.15 radii in front of the centre, not 0.46 as on a sphere. Read the mesh, not the sphere fitted to it.
- **Masks that remove the feature.** The `kotor_skin` mask drops pmhc04's lower lid. The eye shape measure uses all of KOTOR's face triangles for that reason.
- **Broken target heads.** KOTOR's outline of mod_hp_pfha01 cannot be read, because slits above the eye send it into the brow. mod_narve_canderous reads as nearly closed, and mod_tcs_bastila's upper arch wanders. Name such heads, and do not trust their scores.
- **Pass marks inside the noise.** David asked "you keep giving me mm, i've never stopped to ask, what's good enough?". Every measure needs its wobble (how much it moves on the same head) next to KOTOR's spread. A pass mark below the wobble means nothing. The jaw's provisional marks passed 54 of 56 heads because KOTOR's jaw meshes wobble. The strict marks, a quarter of KOTOR's spread, passed 1.

## 2. Sliders and constructions

### One name, one main effect

Each slider sets one named measure and holds the others. Its name says in plain words what it does. A natural second change is allowed when the slider's hint says so. Drop a slider whose name you cannot write in plain words, or whose measure barely answers it, and report that you dropped it.

Why: a player cannot use a slider whose effect they cannot predict. A solver cannot use a slider that moves everything at once.

Example: the nose's round 1 had 12 sliders learned by regression. They reproduced 54 of 57 heads, but they were tangled and had names like "Shape 2" and "Shape 3". David found that "width at the wings changes the curve", and their extreme values turned the nostril's outer wall inside out. Round 2 had 14 named sliders in five groups (root, bridge, tip, wings and base). Each was tied to one measure and built from smooth mirrored bumps on its own part of the nose. David called it "the best system we've had on any feature so far". On the eyes he rejected a tilt that moved the corners: "tilt changes the ends rather than just tilting. That's a big issue for me." He also found that Eye width dragged the eyeball along and left a gap in the corner.

### A head is only its slider values

The same slider value gives the same result on any head (decision 47). A starting head is a saved list of slider values with no shape of its own underneath. The build does not depend on the order of the edits.

Why: the goal is a creator that can make any head. A result that depends on a hidden starting shape proves nothing about the sliders.

Example: when a lead described sliders as changes relative to a starting head, David asked "do we agree it will give you the same exact nose whatever the starting point was?". That question became decision 47.

### Everything through sliders

When a KOTOR head cannot be made, a slider is missing. Add or change a slider rather than build an effect that only a fit can produce. This holds for shape, paint and hair alike, and for the creases, whose generator is driven by a few sliders that the fitter guesses (decision 55).

Why: a result that only a fit can produce does not prove that a player can make it.

Example: on 2026-11-18 David asked that "everything we build can be done via sliders (ie: if we need to tweak sliders we do it)". On the eyes, sliders for the socket's depth and tilt, the inner corner's shape and the points of each lid arch were missing, so Bastila could not be made. In the paint work, the facial hair grid was not accepted until it had drawn a real goatee. David said: "We've built the grid but we haven't proven it works to create shapes, like a goatee."

### Test the power of the existing sliders before building new ones

Before you build a new slider or construction, solve a few heads with only the existing sliders and the new measure. Then look at what is still out of reach. That gap names the control that is missing.

Why: this costs an hour and shows exactly what to build. Building first and testing after costs a day when the new slider was not the gap.

Example: on 2026-11-21 the new eye shape measure was given to the existing solver on 8 heads. The mean gap to KOTOR's curve fell from 1.5 to 6.8 mm (live) to 0.4 to 0.9 mm on seven heads. One part stayed out of reach on every head: the far outer corner seen from the temple side. Kreia was short by 5.2 mm, Bastila by 3.1 mm and pmhc04 by 1.6 mm. That told the next step what to build.

### When patching stops working, rebuild the construction

When round after round fixes one symptom and breaks something nearby, the construction itself is wrong. Stop tuning and rebuild the part from a different idea. Before the rebuild, say how confident you are that it will work and why.

Why: patches stack on a construction that cannot express the target. Each patch fights the ones before it.

Example: the eye opening was first built by pressing the lid edges onto the eyeball and pushing them around with bump sliders, a corner carve, a free stretch and an eye paste. Seven rounds on pmhc04 on 2026-11-21 each moved one number and broke another. Round 1 brought the skin beside the nose to 3.1 mm in front of the eyeball (KOTOR 1.8), and round 2 sent it back to 5.7 mm. Round 5's new slider notched the outer corner from a value of about 1.5. Round 6 reached both targets, but only by moving the whole eye 2.5 mm forward. David said: "i think we need to stop doing small tweaks that don't work, and go for the bigger change". He also asked "Do we have any confidence THAT will work? we keep trying stufff that don't". The rebuild that worked builds the opening from a closed curve on the eyeball. It lays the visible rim on that curve and moves the skin round it as one smooth field. It reproduced the curve it was fed within 0.08 mm, and its ten handles replaced 23 old eye sliders.

### Build from what the feature really is

Look at how the feature is really built before you design its construction. Count the mesh's layers, slice it, and draw it from the side.

Why: a construction aimed at the wrong edge or the wrong layer cannot be tuned into the right one.

Example: our eye's lid edge loop (`lid_edges.json`, 28 vertices per eye) is not where the skin meets the eyeball. It sits under the lid, against the eyeball, and the skin rolls over it. The visible rim is the third layer out from that loop on every head checked. Earlier work had moved the loop and wondered why the visible edge did not follow. A 3 mm slice through the eye of KOTOR and of ours, seen from the side, showed the order of the lid piece, the lid skin and the eyeball directly.

### Test every slider at both ends, open and closed

Draw every slider at both ends on the base head and on all 57 stand-ins. Draw every pair of sliders at its four corners. For anything near the eyes, draw it with the eyes open and with the eyes shut. Look at these sheets before you believe the bench.

Why: the middle of a range is usually clean. The faults live at the ends and at the corners of pairs, where a player will go.

Example: on the jaw, a judge looking at the ends found that the high end of chin projection made a beak-like shelf. Chin width creased at both ends, and jaw width folded at the jaw angle. "Lower jaw width" was broken at both ends, with a gash at the low end and a shelf at the high end, and it was taken out. On the eyes, the closed lid turns about the eyeball's centre. Any part of the opening behind that centre stays bare when the eye shuts, which only the closed-eye pictures showed.

### Repair the far ends before you cap them

When a slider's end looks wrong, fix the construction so the end is clean. Cap the range only when the fix fails, and then cap it just past the values the KOTOR fits use.

Why: David prefers a range that reaches over a range cut short. Capping loses variety that players would use.

Example: of the nose, David asked to "re-tweak them… rather than just cap them and lose variety". On the eye curve, the ends of Outer corner height (5.43) and Outer corner place (26.74) still made a fold and a spot after the ring was rebuilt. Only then were they set to 4.6 and 25.8, just above every head's fit (decision 2).

### Keep the skin convex and smooth

Check the skin between the feature and its neighbours. KOTOR's skin round the eyes is mostly convex and smooth. A shape can match the feature's outline and still sink the skin round it.

Why: a hollow or a step reads at once as a deformity, even when the feature itself is right.

Example: David found that "there's a concave cavity for the eyes now in male C4, in kotor models the curves are mostly convex and smooth". Male A2 and A4 showed sunken holes under the brows. The first ring round the eye curve capped the skin's slope, and the cap let go 9 to 15 mm out, which made a step and an alcove. The rebuilt ring moves the whole region within 22 mm as one smooth field that fades with no step.

### Cutting skin is allowed

A slider may cut or open skin when that is the way to reach a shape without folds. It must do so procedurally, with the same result for the same value on any head, and it must hold up on all 57 heads, open and closed.

Why: pushing skin around to reach a shape it was not built for makes folds.

Example: our lids met in a sharp, wrapping point at the outer corner, where KOTOR's opening ends blunt and open. Pushing the skin toward KOTOR's shape made folds in round 2 of the pmhc04 work. David then said he was "happy to start cutting some skin if we need/when we need, if it's done procedurally via sliders and leads to good results consistently".

### Change only what was asked

When David says a part is good except for one fault, name what must stay in the brief, and make the smallest change that fixes the fault.

Why: a rebuild aimed at one fault breaks what already worked.

Example: David asked for a narrower caruncle with a bit of shading and depth. An agent rebuilt its reach and sank it, and it no longer met the lids and the eyeball. David said: "before it was perfect just too wide, now it's not connected at all."

### Folds, and the fold guard's blind spots

The fold check counts triangles that turn over. It is a guard, not a verdict. Treat every flagged fold as real until a close-up proves otherwise, and treat a clean count as unproven until the close-ups are clean too.

Why: the count misses some faults and counts others that no one can see.

Examples of what it misses and what it miscounts:
- On 2026-11-20, an eye agent called the fold check's 57 of 57 flagged heads a false alarm, because its distant clay sheets showed nothing. David then found torn skin flaps under the lower lid and a rolled tube at the outer corner, in a three-quarter close-up in viewer 2.
- It misses creases and notches. Round 5's notches at the outer corner passed the guard. They were found in close-ups.
- A crease from a height floor under the eye showed only in viewer 2's light, not in the clay sheets.
- It counts triangles folded under the eyeball, where no one sees them. Count only those wholly outside it.
- It counts against the live head's lid. A construction with a new lid edge counts 70 to 160 folded triangles per head where the pictures are clean.

When David asked how to see folds, the answer was a fold viewer that shows which sliders cause each fold.

## 3. Solving and fitting

The solver finds the slider values that make one KOTOR head. Its results are the presets, the proof of power, and the source of every slider's range.

### KOTOR's heads first, then the ranges

Test on the base head and the KOTOR fits only, never on random slider settings. The values the 57 fits use set each slider's range.

Why: random heads are faces nobody makes. Fixing them before the KOTOR heads work is wasted work.

Example: on 2026-11-19 an agent tested the blink on random FLAME heads, and one random head was broken before the blink even started. David said: "prove to me we can do all KOTOR heads with our flame. Then when we can, we have a set of range."

### Stop the solver from cheating

A solver reaches a target by the cheapest route, which is often to move the whole feature. Hold what must not move, such as the eye's place in the face and the opening's height, or give it a cost.

Why: a solve that moves the whole eye matches the shape numbers and breaks the face round it.

Examples:
- Round 6 reached both eye targets on pmhc04 by moving the eye 2.5 mm forward. The space between the eyes grew from 61.7 to 67.2 mm (KOTOR 60.0), and the judges saw a hollow under the eye.
- Round 7 held the eye in place and reached pmhc04's outline. But its eye was taller and rounder (8.5 mm high, KOTOR 7.35), and both judges preferred live. A good outline overlap can come with the wrong height, so the opening's height must be held too.
- The refit lets the eyeball's centre move only along the line from its live place to KOTOR's centre, give or take 1 mm. Holding it at KOTOR's centre alone would have been wrong, because KOTOR's centres are not where its openings are. pfha05's centre sits 8 mm behind ours.

### Make sure the solver can move every slider

A slider that acts like a threshold never moves under a solver that takes small steps. A noisy measure stalls the solver early. Test such sliders by hand, and add a coarse search after the fine one.

Why: a slider that the solver never moves looks dead in the bench when it is not.

Example: Outer corner reach acts like a threshold, so a finite-difference solver never moves it from 0. The eye refit stalled on Jolee, pmha02 and pmha04 until three passes of a coarse search were added. Jolee's cost then fell to 9.9, against 30.9 without it.

### Refit after a construction changes

When the construction under the sliders changes, every preset and every learned value built on the old one is stale. Refit them from the original data.

Why: the old values were fitted to a different mesh. They put features in the wrong place on the new one.

Examples:
- The eye refit after the curve construction moved the median gap to KOTOR's opening from 2.01 mm (live) to 1.00 mm, and fixed the twisted openings that version 4 had left on two mod heads.
- Field sliders are baked into the data. A code change to a curve does nothing until the fields are remade. Remake them from the original data, never from a folder that was already remade, or the amounts are fitted to the new fields.
- The animation calibration records which fit it was learned from, and must be rerun when the mesh changes (decision 62).

### Guard every other head

A new construction or preset ships to all 57 heads, not only to the head you worked on. A head takes a new preset only when it passes its guard. The guard covers folds, stray eyeball when shut, and no head worse than its wobble on three or more measures. Then check the pictures of the heads that changed.

Why: a fix aimed at pmhc04 often breaks Bastila or Kreia. David said: "Let's try and NOT destroy every other heads while we're at it".

Examples:
- In round 1, pmhc04 improved, but two judges found Bastila and Kreia worse, so they kept their old presets.
- In round 3, 14 heads took new presets and 43 kept round 1's.
- The refit's guard tries 1, 0.75, 0.5 and 0.25 of each head's move and keeps the largest share that passes. 47 heads kept the full move, 4 kept three quarters and 6 kept half.
- Always solve and judge Bastila next to pmhc04. Fixes for one have failed on the other three times.

### Fit the system, not the heads one by one

When something must be learned from the fits, such as the rig's weights, learn rules that work from any slider-made head. Do not store one answer per KOTOR head.

Why: the KOTOR heads are only test targets. Everything ships to a creator, and a player's head has no stored answer.

Example: of the brow rig, David said: "our goal is to have a procedural system so I don't understand why we're fitting on the HEADS rather than on our sliders/the output." The calibration now learns rules from the fitted heads that compute any head's rig from its own vertices (decision 62).

## 4. Showing and judging

### Pictures before numbers

Look at pictures of a result before you believe its numbers, and before you write any measure or fitter. In a new feature, draw KOTOR's 57 heads on the area first, then set sliders by hand on a few heads, and look.

Why: every fault found so far was in the target or the method, not in the sliders. Pictures find those faults in minutes.

Example: on 2026-11-19 a crease agent spent 90 minutes on a measure, a placement study and a fitter before any good picture existed. Its fitter matched a wrong measure. Hair edges read as creases, and KOTOR's nose-to-mouth fold is a border of tone, not a line. David asked "are we sure we're targeting the rights things? It's been 1h30."

### David judges, in viewer 2, next to KOTOR

A step is not done until David can see its result in viewer 2, on our head next to the KOTOR original, for every head. Say exactly what was checked and how. Separate what was measured, what was seen, and what was not checked. Never say "it works" from numbers alone.

Why: David judges likeness and fun by eye. He says he is "very very bad at copying", and he is not an anatomy expert. Show him something, and he says what looks wrong.

Example: David asked "wait what am i looking at, they look completely broken?" of a sheet that came with good scores, and later "can i see it in viewer2 first?". When a base tone fit came as numbers and a swatch sheet, he said "I need to be able to see dude otherwise i can't help. for EVERY step we do i need to be able to compare to base." Ask him one small question at a time. He said "you're gonna need another way to ask me than showing me everything all at once", and the slider page moved to single cards marked "Same" or "Not the same".

### Close-ups from every side, in viewer 2's light, with the hair off

Every check of a shape includes close-ups in three-quarter view, from above, from the temple side, from the nose side and in profile. Render them in viewer 2's "In game" light with its paint, as well as in clay. Side views are rendered with the hair off, on our head and on KOTOR's. Look at the eyes closed as well as open.

Why: distant front sheets in clay hide folds, creases and hollows that the game's light shows.

Example: torn flaps under the lower lid were found in a three-quarter close-up in viewer 2 after clay sheets showed nothing. Bastila's hair hid her side view until David said "Let's tell the judges to toggle the hair off."

### Draw overlays correctly

Check that every line drawn on a picture lines up with what it marks before anyone sees the picture. A dark line means before and a light line means after, and both run from our face to KOTOR's.

Why: a wrong overlay makes a good result look bad, or the reverse, and it costs David's trust in every later picture.

Example: David said "dude your lines dn't even align with THEMSELVES" of an early overlay sheet.

### Fresh blind judges, and their bias toward one row

Judges are fresh sub-agents that only look at pictures. Run two, on Fable. Give each one only what to judge and the paths of the comparison pictures. Never give a judge the history, the code, the brief or the answer you expect. Each judge scores and proposes fixes.

Judges on a stacked sheet prefer one row, often the second, whatever it shows. Work round that bias this way:
- Put KOTOR in the middle row, with the two candidates above and below it, drawn at random per head.
- Show the second judge the same sheets with the rows swapped.
- When the two judges split, each picking the same row, the result is only the bias. Then show enlarged crops of the one view in question side by side, with the sides swapped, to two more judges.
- Look at the pictures yourself as well, and say what you see.

Why: an unbalanced sheet gives a confident verdict that means nothing.

Example: in pmhc04 round 2, both judges picked the second row on every head, in both orders. In rounds 3 to 5, the profile on stacked sheets split by row each time. Enlarged profiles side by side with the sides swapped gave an agreed answer in round 3 (3 of 5 against 2 of 5, in both orders).

### Honest reports name what is broken first

Start every report with the verdict. List what still looks broken before what is better. Compare the before and after pictures for anything that got worse, and say so first. Name the heads that miss and why.

Why: David decides from the report. A report that leads with the gains hides the regressions that cost the next day.

Example: the eye ring report of 2026-11-21 opens with "What still looks broken, first". It names the fold at Outer corner height's high end on Kreia, the spot at Outer corner place's high end, a thin pale edge along both lids, and a ring measure that did not improve. Only then does it list what is fixed. The refit could go live because the faults were all on the table.

## 5. How the work is run

### Read the roadmap, and say which item you serve

Read `2026-11-19-head-roadmap.md` first, with its "For agents" rules and its decisions. Say which item your work serves. Keep "Now and next" current.

Why: the roadmap is the one plan for David and every agent. An HQ once confused tooling with goals, and a close fit on zones where all KOTOR heads are alike proved nothing.

### One conversation per step, with notes between them

Break the work into steps, such as a diagnosis, then fix 1, then fix 2. Each step runs in its own full conversation, started by the HQ from a standalone brief. Each conversation writes a note that the next one reads: its verdict, what changed, how to undo it, its traps, and what comes next. A conversation whose context passes about 750,000 tokens writes a handoff, and the HQ starts a fresh one with the same brief and that note.

Why: context is expensive, and a fresh conversation with a good note is faster than an old one carrying a day of dead ends. David said: "one convo for diagnosis, one for fix #1, one for fix #2, ... let's try and save on context".

Example: the eyes went through about fifteen conversations on 2026-11-20 and 2026-11-21. Each left a note in `$TMPDIR/kjk-*.md`, from `kjk-pmhc04-eyes-diagnosis-2026-11-21.md` to `kjk-eye-refit-2026-11-21.md`. Their "Traps" sections are where most of this guide's traps come from.

### Conversations build, sub-agents only judge

A full conversation does all building itself: code, builds, tests, solves, benches, renders and browser checks. Sub-agents are only judges of pictures, and the research for a guide like this one.

Why: a sub-agent loses its cache after 5 minutes, and most of these commands take longer. A sub-agent that builds pays its whole context again after every long command. David said: "make sure the conversations use sub-agent only to actually judge results, not to 'make sure slider works' or wtv".

### Small steps, a bounded number of loops, and a stop

Work in small steps. Diagnose and measure first, then plan, then run 2 to 3 loops of fixes at most, and stop for David. When there is no good plan, stop and say so rather than try another guess.

Why: open-ended loops spend a day on tweaks that do not converge. David said: "I'm fine with the implementations after the brainstorming/measurement having 2 to 3 loops maximum… if we don't have a good plan or can't figure it out, happy with you calling it a day."

### A prototype gate before a full build

A large change goes through gates:
1. **Step 0.** Measure the change's premise on KOTOR's 57 meshes.
2. **Step 1.** Build a prototype on code that viewer 2 does not use. Set its values by hand on pmhc04, Bastila and a few other heads. Have two blind judges compare it with KOTOR and live.
3. **Step 2.** Build it fully only when the prototype is a clear win on both pmhc04 and Bastila.

Change nothing live before step 2 passes, and say how to undo each step.

Why: a failed prototype costs a conversation, while a failed full build costs a day and a live regression.

Example: on 2026-11-21 two eye rebuilds stopped at step 1. The larger eyeball set further back failed its premise. The corners on the equator split the judges on Bastila. Neither touched live. The curve construction passed step 1, so its handles, its ring and its refit followed as separate conversations.

### Start the task without asking

A conversation started to do a task does it without asking for a go-ahead. It stops for David only when something goes against its brief, or when a choice would change what is being built.

Why: David said: "if a convo is launched to do x, no need to ask 'shall i do x now?'".

### Rust for heavy work

Write measures, slider making, solves, benches and renders in Rust. Use Python only for light work, such as reading KOTOR's files once (decision 53).

Why: David asked "why is solving SO slow? is it python?". The Rust solver that shares its smoothing between the 57 heads solves the eyes' presets in about 13 seconds.

### Aim at KOTOR's style, not realism

The target is KOTOR's stylised, low-detail look. A slider's range must reach it.

Why: a realistic result can still miss the look the game needs.

Example: on the beard paint, David found that "even on max fickness it still looks a litlte like felt/too fuzy", where KOTOR's hairs are straight, thick and stylised. He rejected drawing hairs at render time because "the face is PLENTY sharp enough for what we want to do". He also found that the first facial hair "absolutely does not register on black faces", so every paint check covers every skin tone.

## 6. Checklist for a new feature

Follow these steps in order. Each step ends where it says it ends.

1. **Read.** Read the roadmap's "For agents", "Now and next" and decisions, `CLAUDE.md`'s writing rules, and this guide. Write down which roadmap item you serve.
2. **Look at KOTOR.** Draw the feature on all 57 KOTOR heads, front, three-quarter and profile, with the hair off and every muddying part switched off. You are done when you can say in plain words how the feature varies across KOTOR.
3. **Check your premise.** If your plan rests on a claim about KOTOR, measure it on the 57 meshes. You are done when the numbers support the claim. If they do not, stop and report.
4. **Build the measure.** Name each proportion and outline. Read KOTOR's side on KOTOR's own meshes. You are done when the measure passes the four checks, with its wobble, KOTOR's spread and the extremes sheet written down.
5. **Test the existing sliders.** Solve a few heads, including pmhc04 and Bastila, with only the existing sliders and the new measure. You are done when you can name what is out of reach.
6. **Prototype.** Build the new slider or construction on code that viewer 2 does not use. Set values by hand on a few heads. Draw both ends of every slider, open and closed, in close-ups in viewer 2's light. You are done when two blind judges, with the rows swapped, find a clear win on both pmhc04 and Bastila. If they do not, stop and report.
7. **Build and solve.** Build the sliders fully. Solve all 57 heads with the positions held. Guard every head. You are done when the ends and pair corners are clean in the pictures and every head that takes a preset passes its guard.
8. **Judge and fix, twice at most.** Run two Fable judges on KOTOR next to ours, then fix once. You are done after the second loop, whatever the result.
9. **Show David.** Put the result in viewer 2 next to KOTOR for every head. Write the report with what is broken first, and keep what was measured, what was seen and what was not checked apart. Write the note for the next conversation, with its traps.

## 7. Added on 2026-11-22, from the jaw, skull, nose and lips work

These rules come from the night of 2026-11-21 to 2026-11-22. The roadmap's decisions 68 to 73 hold the decisions behind them.

### Measure against a reference, and build one when KOTOR has none

A diagnosis measures against a reference. It runs no judges (decision 70). When KOTOR's mesh hides the part, build the reference and mark what is filled in.

Why: without a reference, a measure of a hidden part means nothing, and judges only add opinions.

Example: KOTOR models no scalp under modelled hair. Our skulls had been fitted onto the outside of the hair on 21 heads, so pmhc04's bald skull was 155 mm wide against 125 mm. Bald references made of KOTOR's real skin, with a skull filled in from the 36 heads whose skull shows, found it (decision 71).

### Check what the hair test calls skin

The colour test in `kotor_hairless.py` fails both ways. On about 35 heads it calls painted short hair "not skin", although that surface is the real skull. On pmhc04 it called the hair volume "skin". Choose skull-showing heads by their shape, and look at every cut.

### A measure must read the part, not its neighbour

The first lip measure took the fold under the nose for the line between the lips on 21 of our heads, among them every female head, and on 6 of KOTOR's. The solve then made pfha05's upper lip taller where KOTOR's is thin. Search for a feature near where it must be, for example the lip paint's opening rows, and draw the reading on every head before solving on it.

### The fold guard counts every triangle

A guard that skips hidden triangles misses folds. The lips guard left out the slit between the lips, and 15 heads had 1 to 9 turned triangles there. Count every triangle against the head without the feature, not against an earlier version of the feature.

### Moves after the smoothing

A move applied before the build's last smoothing step is spread out by the smoothing. Mouth width could not widen the mouth, because the smoothing spread the corners' move along the lips. Moves that must stay sharp, such as mouth width and Cupid's bow, are a slider kind "after" (`set.rs`, `solve.rs`), applied after the smoothing.

### Vertices the zone holds at zero

Hidden vertices that a slider zone gives a weight of 0 never move, and they make notches. This happened under the jaw and at the mouth's corners (344 lining vertices). Move such vertices with their nearest skin vertices.

### Scoring judges one picture at a time

Fable judges chose by position on every stacked sheet in two conversations, even with the sides swapped and with enlarged side-by-side crops. When judges are used, show each attempt alone against KOTOR, with the pictures shuffled, and ask for a score.

### Fold clearly better results into viewer 2, and delete the prototype

A clearly better result goes straight into viewer 2's default, with no URL option (decision 72). Delete the prototype it replaces, and write how to undo it. Clearly better means that no head gets worse on any measure, nothing folds at the solved values, and the close-ups are clean.

### Pair corners need repair, not caps

At the far corners of two sliders, the solved presets are often far away, and folds hide there. Tonight's caps either barely helped or moved presets by up to 9 mm. Report the corners for David, and repair the slider ends as their own step.
