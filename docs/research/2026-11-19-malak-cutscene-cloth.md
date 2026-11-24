# Darth Malak's cloth in the Bastila torture cutscene

This note was written on 2026-11-19 for the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts"), step L1 of `docs/research/2026-11-19-loose-parts-plan.md`. It follows up the 11 KOTOR 1 clips that `docs/research/2026-11-19-cape-flourishes.md` left unclear. All 11 belong to the stunt model `m44aa_c02_char02`, which is Darth Malak in the cutscene where he tortures Bastila (module `STUNT_18`, dialog `m44aa_c02`). The user had watched recordings of these clips and found that Malak's cloth looked "very weird, almost buggy".

## Summary

- The clips mean the same pose on `n_darthmalak` as on the stunt model. The two models have the same 99 animated nodes, with the same names, parents, node numbers and rest positions. Their rest rotations differ by at most 5.4 degrees, except on two finger bones of the left hand, which differ by 18 and 24 degrees.
- The original game plays the cutscene the same way: it draws Malak with `n_darthmalak` and plays the stunt model's clips on it. The viewer was already doing the same.
- The buggy look came from the viewer, not from the data. It had two causes.
- The first cause is the faint body (`--ghost`). Its surfaces are blended without being sorted by depth, so the cape behind Malak was drawn over his chest. This made the cape look as if it was draped diagonally across his chest.
- The second cause is the camera. Every clip of this scene turns Malak by about 172 degrees through its `cutscenedummy` node and moves him about 118 metres away. The viewer measured its camera angle from the model's own front, so the "front" view showed his back and the "rear" view showed his front.
- The viewer now measures the camera angle from the way the clip turns the body. The 11 clips were recorded again with a solid body.
- Seen correctly, 10 of the 11 clips fake physics. Malak's cape and hip flaps hang down, or swing out behind him when he turns, leans or walks, and fall back afterwards.
- `cut001w` is a special case. Each of its cloth bones has only two keys. The first key is the cloth's bind pose, with the hip flaps spread out to the sides, and the second is a hanging pose. It is not a flourish. A solver can replace it like the others.
- None of the 11 clips is a flourish. The "cloth held at fixed angles" in the flourish note is real data, but it is the cloth hanging along a bent or turned body, not a gesture.

## How the two models compare

A small program outside the repository (see "Tool") read both models from the KOTOR 1 install with kjk's model reader and printed every node.

- `m44aa_c02_char02` names `n_darthmalak` as its supermodel. `n_darthmalak` has no supermodel.
- The stunt model has 99 nodes. Every one of them also exists in `n_darthmalak` under the same parent. `n_darthmalak` has 12 more nodes, which are hooks and mesh nodes such as `head`, `torso`, `lightsaberhook` and `backcape`.
- Every shared node has the same node number in both files, and the same rest position to within 1 millimetre.
- The rest rotations differ by 0 to 5.4 degrees on the body and cloth bones. They differ by 24 degrees on `ldfngrb_g` and 18 degrees on `ldfngrt_g`, two finger bones of the left hand. These small differences do not matter, because the clips key the rotation of every bone they move, and the game uses the keyed rotation in place of the rest rotation.
- The clips key all 27 cloth bones, both arms, the legs, the spine, the head and the face. Their position keys are offsets from the rest position, and they are 0 on every bone except `cutscenedummy` and `rootdummy`. `cutscenedummy` is placed at about (110.9, 41.2, 0.4) and turned by about 172 degrees about the vertical axis.

The keys therefore put each bone of `n_darthmalak` exactly where the stunt model's skeleton would put it. The suspicion that different rest orientations or names would put the cloth in the wrong place is false.

## How the original game plays the cutscene

- The dialog `m44aa_c02` lists two stunts. The participant `darthmalak` uses the stunt model `m44aa_c02_char02`, and `bastila` uses `m44aa_c02_char01`. The dialog is an animated cutscene with the camera model `m44aa_c02_cam`.
- The creature with the tag `DarthMalak` in `STUNT_18` comes from the template `n_darthmalak001`. Its appearance is row 21 of `appearance.2da`, `unique_darth_malak`, whose model is `n_darthmalak`. The game therefore draws Malak with `n_darthmalak`. It never draws the stunt model, which only holds clips and two eye meshes.
- The reference engines Odyssey and K12 agree on what happens when the cutscene starts. For each stunt, the game finds the participant's creature and loads the stunt model as an extra source of clips for that creature's own body model, and for its head model if it has one. When a line asks for a clip, the game looks for it in that extra model first and then in the body model.
- The game applies a clip by walking the body model's node tree and the clip's node tree together from the root. At each level, it matches a child of the body to a child of the clip by node number. Because the stunt model was built on `n_darthmalak`, its node numbers are those of `n_darthmalak`. Matched this way, all 99 nodes of the clip land on the node of the same name.
- When the cutscene ends, the game removes the extra model and returns the creature to its combat idle.
- reone does the same. It plays the stunt model's clip on the participant's own model, matching nodes by name.

The order of the lines and their clips is: `cut001w`, `cut010w`, `cut002w`, `cut011w`, `cut003w`, `cut016w` ("You are strong, child. But I will break you."), `cut012w` ("I'll never fall to the dark side!"), `cut004w`, `cut005w`, `cut006w`, `cut007w`, `cut013w` ("Never!"), `cut014w`, `cut008w` ("Become my apprentice, and all its power can be yours!"), `cut015w` and `cut009w`. The first five lines and the last two have no text.

### Frames from the original game

None could be taken. The original game was started headless on `STUNT_18` with the user's tool. The Mac port stopped at once on an alert that says "This game needs Steam. Please make sure that Steam client is running", because Steam was not running. Starting Steam would have shown a window on the user's screen, so the attempt was stopped there.

## How kjk plays the cutscene

kjk plays this scene the same way as the game. When a staged conversation shows a line, kjk takes the line's clip from the participant's stunt model and poses the participant's drawn body with it, matching nodes by name (`stage_stunts` in `crates/game/src/play_view.rs`). The body stands where the clip puts `cutscenedummy`. There are two small differences from the viewer. kjk plays stunt clips with a position scale of 1, as reone does, and the viewer used the model's scale of 1.16. Only `cutscenedummy` and `rootdummy` have position keys in these clips, so this changes nothing visible on the cloth.

## Why the recordings looked buggy

The recordings were made with `--ghost 0.55`, which draws every surface of the body blended at 55% opacity. Blended surfaces do not write depth, and the engine orders them mesh by mesh, not triangle by triangle. Malak's body and cape are separate meshes, so the cape behind him was drawn after his chest and over it. In Malak's own clips, such as `c2a1`, the flaw did not show. The likely reason is that the body was not turned round there, so the mesh order happened to be right.

The viewer's orbit camera started at an angle measured from the model's own front (+Y). The stunt clips turn Malak by about 172 degrees through `cutscenedummy`, so the left video, meant to be a front three-quarter view, showed his back.

A still of `cut013w` drawn solid shows Malak standing upright with his arms at his sides, the cape hanging behind his right shoulder and the flaps hanging over his legs. This is the same look as his own idle clip `pause1`.

## What changed in the viewer

These changes are in the `loose-viewer` worktree.

- `crates/game/src/loose_view.rs`: the orbit camera's angle is now measured from the way the clip turns the model's `cutscenedummy` at the clip's start. A model without that node, or without a clip, still uses its own front.
- `crates/launcher/src/loose_args.rs`: the help of `--ghost` now says that the faint surfaces are not sorted by depth, so a cape behind the body can show over the chest, and that the flag should be left out to judge how a garment hangs.

The 11 clips, and the 5 slow versions at 40% speed, were recorded again into `kjk-artifacts/loose-parts-recordings` with a solid body, with the same file names. The page `index.html` there was regenerated with new descriptions.

## The 11 clips judged again

Each verdict comes from 8 frames of the front view and 8 frames of the rear view, spread evenly over the clip.

| Clip | Line | What the frames show | Verdict |
|---|---|---|---|
| `cut001w` | first line, no text | Malak stands still. The hip flaps start spread out to both sides, as in the bind pose, and swing slowly down to hang. One flap swings out sideways again at the end. Every cloth bone has two keys, and the first is exactly the bind pose. | Fake physics. It is a straight blend from the bind pose to a hanging pose, not a gesture. |
| `cut005w` | "You think torture will turn me, Malak?" | Malak stands and moves his arms a little. The cape and flaps hang still. | Fake physics, nearly static. |
| `cut006w` | "Torture? No, dear Bastila." | Malak leans far forward with one arm out, then straightens. The cape hangs straight down along his back. | Fake physics. |
| `cut007w` | "When you finally swear loyalty to me..." | Malak stands, then turns round quickly. The cape flies out behind him, close to horizontal, during the turn and falls back. | Fake physics. The swing is the follow-through of the turn. |
| `cut008w` | "Become my apprentice..." | Malak gestures and raises both hands. The cape and flaps hang. | Fake physics, nearly static. |
| `cut009w` | last line, no text | Malak crouches in a wide stance with his arms forward. The flap behind his back leg is held out along that leg for the whole clip. | Fake physics. The flap lies along the leg so that it does not cut through it. A solver with leg collision gives the same result. |
| `cut012w` | "I'll never fall to the dark side!" | Malak is hunched, then straightens and turns away. The cape swings out wide during the turn. | Fake physics. |
| `cut013w` | "Never!" | Malak stands still. The cape hangs behind him and sways a little. | Fake physics. It is not the cloak flare the flourish note suspected. |
| `cut014w` | "Such resolve in your words..." | Malak turns and walks while he gestures. The cape swings with the turns and steps. | Fake physics. |
| `cut015w` | no text | Malak holds his hands to his face, then throws his arms out. The cape hangs and sways a little. | Fake physics. |
| `cut016w` | "You are strong, child. But I will break you." | Malak leans forward and swings an arm. The cape swings out behind him and falls back. | Fake physics. |

## How sure this is

- That the two models share one skeleton and that the keys mean the same pose: certain. It was read from the files.
- That the game draws `n_darthmalak` and plays the stunt clips on it: very likely. The creature's appearance was read from the module, and two independent reconstructions of the executable and reone agree on how stunt clips are loaded and applied.
- That the buggy look was the viewer's: certain. The same clip at the same time is clean when drawn solid.
- The verdicts: fairly sure for 10 clips, because in each the cloth only moves when the body moves or stays hanging. `cut001w` is less sure. Its cloth moves while the body stands still, but its keys are a plain blend out of the bind pose. Nobody has checked whether the cutscene's camera shows Malak's hips during that first line.
- Nothing was compared with the real game on screen, because the original game could not start without Steam.

## Tool

The comparison program is outside the repository, in `/private/tmp/claude-501/-Users-david-perso-kjk/5fbe746b-0603-4318-a877-f1ce4292c9c8/scratchpad/mk/src/main.rs`. Its commands `tree`, `anim`, `bind`, `keys` and `gff` print a model's nodes, a clip's nodes, the result of matching a clip to a model by node number, a clip's keys, and a template's fields. The time strips used for the verdicts are in the `stills` folder next to it.
