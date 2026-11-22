# Handoff for the decision on the head shapes

Written on 2026-11-20 by the session that led shape and sliders (session `323ce000-e135-4d36-a800-8a864c2e749a`, named `kjk-07`). This note is for David and a fresh agent who will decide together what to do next about the shapes of the heads.

## The decision to take

David moved the sliders in viewer 2 on the morning of 2026-11-20. He then wrote: "our shapes are still not perfect, they're okay but not perfect." He asked where the shape and model work stands.

The decision is which direction the shape work takes next. David has not yet said what looks wrong to him, or on which heads. The previous lead asked him that question and he chose to hand the decision over to a fresh conversation instead of answering.

Your job is to help David decide. Do not launch any build agent before he has chosen a direction.

## Read these first

1. `/var/folders/sw/l_t3dnys0t7cwtp6ptyx279m0000gn/T/kjk-shape-lead-handoff-2026-11-20.md`. It is the lead's general handoff. It has the role, the table of features with their reports, how David works, and the traps. Everything in it still holds. This note does not repeat it.
2. `/Users/david/perso/kjk/docs/research/2026-11-19-head-roadmap.md`. It is the one plan. Read "For agents", "Now and next" and decisions 46 to 59.
3. `/Users/david/perso/kjk/CLAUDE.md`. Its writing rules apply to everything David reads.
4. The memory notes in `/Users/david/.claude/projects/-Users-david-perso-kjk/memory/`, starting with `MEMORY.md`.

The per-feature reports are in `/Users/david/.claude/tools/likeness-fit-data/sliders/reports/`. Read one only when the conversation needs its numbers.

## What a head is made of today

David sees the heads at http://127.0.0.1:8765/viewer2/. Each head there is three things stacked.

1. **A start shape per head.** It is that head's smoothed stand-in from the earlier fits, with the five slider features taken out. It still carries the skull, the ears, the mouth, the neck, and where the eyes sit on the face.
2. **36 slider values** on five features, solved by the optimiser for each KOTOR head.
3. **Generated parts.** These are the eyeballs, with a fixed radius of 12.3 mm, and the upper lid piece with its lash strip.

Decision 47 says a head is nothing but its slider values. That is not true yet. A large part of each preset still comes from a fitted mesh per head. Nobody has measured how large that part is.

## Where each area stands

"Loose marks" are the provisional pass marks. With the strict marks, 0 or 1 head of 57 passes on each of the four features built overnight.

| Area | Sliders | Made, loose marks | What is wrong |
|---|---|---|---|
| Nose | 14 | 44 of 57 | David accepted the feel. The far ends still deform, and the tips are pointier than KOTOR's. |
| Jaw and chin | 5 | 39 of 56 | No slider makes a square jaw or a jaw corner. Made jaws cluster soft and round. |
| Brow and forehead | 6 | 29 of 57 | No heavy brow like Zherron's. The brow sits too low on 10 heads. It is too flat between the eyes on 14 heads. |
| Cheekbones and cheeks | 4 | 24 of 56 | No hollow under the cheekbone on gaunt heads. |
| Eyes | 7 | 37 of 57 | The opening is too small. 15 of the 21 pairs of eye sliders fold the outer corner. |
| Upper lid piece | none | closes all 114 eyes | Closed, it is a flat plate in another colour. It does not blink with viewer 2's animations. |
| Mouth and lips | none | | Waits for David's choice of mouth rig. |
| Ears, skull, neck, head proportions | none | | Not started. They come from the start shape. |
| Eye spacing, eye height, eye depth, eyeball size | none | | Blocked. The kit measures every feature in a frame set by the eyeball centres. |

## Why the shapes fall short, in the previous lead's view

The order is the previous lead's opinion. It is not measured.

1. **Everything is soft.** The base is a smoothed mesh. Every slider is built from smooth bumps 4 to 9 mm wide. KOTOR's faces are flat planes with hard edges. Three fresh judges said "softer and rounder" independently, on the jaw, the brow and the cheeks.
2. **The targets are fuzzy.** KOTOR's meshes have about 400 vertices. Our measures wobble on them almost as much as KOTOR's heads differ from each other. The optimiser cannot be pushed to a tight match, and a loose pass means little.
3. **The part without sliders is a smoothed fit.** The mouth, the ears, the skull and the position of the eyes were never tuned by feature. They carry what the old fit gave, and David rejected that fit's way of scoring ("we never proved we were measuring the right things").
4. **The features were built apart.** Each feature holds its neighbours still. The seams cost likeness. For example, the brow agent could not move the brow's lower edge because it borders the eye.
5. **The eyeball is too small.** KOTOR's eyeballs reach 17 to 20 mm of radius on many heads, and ours is fixed at 12.3 mm. "Eye width" can only grow by 1.45 mm where KOTOR's eyes need 2.9 mm at the median. This one is measured.

## What nobody knows yet

- **What David means by "not perfect".** It could be the softness, the eyes, the parts without sliders, or something not listed here. Ask him first, with heads on screen.
- **Where the softness comes from.** It could come from the smoothed start shapes, from the sliders' smooth bumps, or from both. No picture separates them.
- **How much the start shape carries.** No one has drawn a head made from the base head and its slider values alone.
- **How the heads look in the game.** David judged in viewer 2, under viewer 2's light. No head made by the sliders has been loaded in kjk. The paint session is prototyping a new light in viewer 2 ("Light from maps").
- **Whether the sliders feel right in viewer 2.** The previous lead confirmed the tabs in one screenshot and never moved a slider there.

## Two cheap pictures that would inform the decision

The previous lead would make these before choosing a direction. Each is a few hours for one Opus agent with the existing kit. David decides whether to make them.

1. **Three heads side by side, for about eight KOTOR heads.** KOTOR's original, the head as viewer 2 shows it, and the head made from the base head plus slider values only. This shows how much the start shape carries.
2. **The same heads with the unsmoothed stand-in next to the smoothed one.** This shows whether the smoothing step or the sliders' bumps cause the softness.

## The directions on the table

| Direction | What it is | What argues for it | What argues against it |
|---|---|---|---|
| Sharper shapes | Smaller, harder bumps where bone shows: jaw corner, brow ridge, cheekbone edge, chin. | The previous lead thinks it is the largest single gain in likeness. | Hard edges fold more easily. The fold check already misses creases. |
| Eyes, round 3 | Eyeball size as a slider, the corner folds, the lid piece blinking with the animations. | David's oldest fear is the eyes. The cause of "too small" is measured. | Every earlier try at the eye area failed in some way. |
| A frame that does not depend on the eyes | Then head proportions, eye position, skull and ears as sliders (roadmap part A). | It unblocks eye position and removes the start shapes' biggest share. | It changes the kit's frame under five features that already work. |
| The mouth | Sliders for the mouth and lips. | It is the largest face feature with no slider. | It waits for David's choice of mouth rig. |
| The spine | One slider-made, painted head loaded in kjk. | The game's light and distance are what matter. It may show that some faults do not. | It improves no shape by itself. |
| Dropping the start shapes | Heads made from slider values only, with a measure of what each head loses. | It makes decision 47 true. | The previous lead would do it last, after the features above exist. |

One option is ruled out by David. Keeping a fitted mesh per head as the preset, with sliders as changes on top, is what the slider work was meant to avoid. He said so when decision 47 was taken.

## Questions David has not answered

1. What looks wrong in viewer 2, and on which heads?
2. Does he want KOTOR's angular look, or is a softer head fine?
3. Should the eyeball's size become a slider? The previous lead recommends yes, with a larger default.
4. Does he take the proposed default eye, which is "Eye width" +1.4 mm and "Eye height" +1.2 mm?
5. Which default mouth rig does he choose?
6. Which blink does he choose for the lid piece? The two options are on http://127.0.0.1:8765/lids/.
7. Does he accept the feel of the jaw, the brow, the cheeks and the eyes, as he did for the nose?

## How to hold this conversation with David

- He judges pictures and feel. He is no anatomy expert. He said: "show me something i'll tell you what looks wrong, not sure i can point to 'how to build it right'." Put heads in front of him and ask what he sees. Do not ask him to choose between techniques.
- Ask one thing at a time. He said that showing him everything at once does not work.
- His goal is "it's fun to build and it's powerful in what it can achieve". Reproducing KOTOR's heads is his proxy for powerful. Fun is his to judge. Power is for agents to prove.
- He prefers a slider that reaches over a slider that was capped.
- Keep apart what was measured, what was seen in a picture, and what is opinion.

## State of the work

- No shape agent is running. Nothing is committed, by David's choice.
- The paint lead is the session named `kjk-5d`. `ListAgents` shows it and `SendMessage` reaches it. A message from another session never carries David's authority.
- The web server on port 8765 must not be stopped or restarted. The lead's general handoff lists the other traps.

## Suggested skills

- `mattpocock-skills:grilling`, to stress-test a direction with David before anything is launched.
- `mattpocock-skills:research`, if the decision needs to know how other character creators make sharp, angular faces with sliders.
- `mattpocock-skills:prototype`, if David wants to try one sharper slider by hand before a full round.
- `mattpocock-skills:writing-for-agents`, when writing the brief for the agent that will build the chosen direction.
- `mattpocock-skills:handoff`, when this next session fills up.
