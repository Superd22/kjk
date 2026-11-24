# Robe capsules and stress scenes for the loose-part solver

This note was written on 2026-11-19 for the roadmap issue https://github.com/kjk-engine/roadmap/issues/5 ("Better physics for robes and other loose parts"). It makes two measurements that come before any solver is written. The first asks whether the ragdoll body capsules can serve as colliders on robed and caped bodies. The second asks how robes and capes will be driven through hard moves, watched and benchmarked in kjk. It follows `docs/research/2026-11-19-loose-part-body-collision.md`, `docs/research/2026-11-19-loose-parts-census.md`, `docs/research/2026-11-19-robe-bone-animation.md` and `docs/research/2026-11-19-dangly-sway-cost.md`.

## Summary

### Measurement 1: the capsules

- The humanoid capsule fit works on every robed and caped body that was tried, and `asset_ja::rig::is_humanoid` is true for all of them. The models were the 11 KOTOR 2 robe bodies, the six KOTOR 1 Jedi robe bodies, the capes of Darth Revan and Darth Malak in both games, and 20 plain or other clothed bodies. `Template::fit` returned a template for each one.
- Robe and cape vertices are counted in the fit. The fit uses every skin vertex of every mesh. Each vertex goes to the plan part that owns the bone that moves it most.
- In KOTOR 2 the robe bones hang from `pelvis_g` and `torso_g`, so the skirt vertices that a robe bone moves most are counted in the pelvis and belly capsules and not in the leg capsules. For example, `pmbim` has 234 skirt vertices. Of these, 180 are counted in the belly, 43 in the pelvis and 11 in the two thighs.
- The thigh and shin capsules of the KOTOR 2 robe bodies are not made larger by the robe. The unclamped thigh radius is 0.093 to 0.115 m on the robe bodies, against 0.102 to 0.103 m on the plain KOTOR 2 body `pmbhm`. Taking the robe vertices out of the fit changes a thigh radius by at most 0.9 cm.
- The belly and pelvis radii are made much larger by the skirt, but the plan's limits stop it. The unclamped belly radius is 0.33 to 0.87 m on the robe bodies, and the capsule keeps its limit of 0.14 m. Every belly capsule on every body is a sphere with no length, because the two belly nodes are closer than 14 cm.
- The KOTOR 1 Jedi robe (`pmbj*`, `pfbj*`) is the one case where the robe does inflate the legs. The whole robe is the `legs` mesh, skinned to the leg bones. The unclamped thigh radius is 0.135 to 0.152 m, against 0.089 to 0.096 m on plain KOTOR 1 bodies. The unclamped shin radius is 0.102 to 0.161 m, against 0.071 to 0.087 m. The plan's limits of 0.095 m and 0.070 m hold the capsules at the maximum, which is about what plain bodies get.
- The limits hide the inflation but leave a different problem. At rest, 15% to 24% of the KOTOR 1 Jedi robe vertices lie inside a thigh or shin capsule, up to 3.5 cm deep. In KOTOR 2, up to 8% of the robe vertices lie inside a thigh or shin capsule (19 of 231 on `pfbnm`), and up to 30% lie inside some capsule when all 14 are counted.
- The proposed fix is to leave the fit alone and give each cloth vertex its own collision radius. The radius is the capsule's radius, or the vertex's rest distance from the capsule's axis minus 0.5 cm, whichever is smaller. Excluding loose-bone vertices from the fit is not needed.
- Most capsules sit at the plan's maximum even on plain bodies. The 65th percentile distance is larger than the limit in most cases, so the fit is mostly a fixed size. This matters for the solver, because the capsules are not tight.

### Measurement 2: stress scenes

- kjk already has a repeatable way to drive any KOTOR body through hard moves. `kjk play --body BODY --head HEAD --moves SCRIPT` plays a scripted input that is a pure function of the sim's time. `--cmds` then holds the game's time at a chosen second (`anim T`), takes screenshots, or renders frames with `record`. It works headless, so no window opens.
- The sim has forward, back and side flips, aerial cartwheels, rolls and a kata. The player can also spin by turning the view. All of these were triggered and checked by reading the legs animation name from a dump. The sim has no wall jump and no wall run. Only their animation names exist in the tables.
- No stock KOTOR 2 scene has more than 4 characters with robe bones. `trayus_academy` has 70 Sith, but only 4 of them wear robe bodies (`pmbnm` and `pfbnm`). The other 65 wear `n_darkjlowm`, `n_darkjmidm`, `n_darkjmidf` and `n_darthsion`, which have no robe bones. A robe-bone solver will therefore barely show up in `trayus_academy`.
- A bench scene was added in the worktree `/Users/david/perso/kjk/.claude/worktrees/robe-stress`, not committed. `robe_acrobatics` is a robed player (`pmbim`) doing flips, a cartwheel and a roll among the 70 Sith and 40 added robed Sith. Its baseline on the current build is 94.9 fps mean and 46.7 fps 1% low in one run, and 55.4 fps mean and 18.1 fps 1% low in an earlier run with a different move script.
- A calm capture scene, with one robed character in the grey-box arena, was recorded as a before set of 105 frames.

## Measurement 1: are the capsules usable on robed bodies

### Method

A test was added in the worktree `/Users/david/perso/kjk/.claude/worktrees/robe-bones`, which already held the data-loading test. It is `crates/approved_tests/tests/robe_capsules.rs`. It is not committed. It needed `asset_ja` and `glam` added to `crates/approved_tests/Cargo.toml` in that worktree, which is also not committed. It is marked `#[ignore]`. Run it with `cargo test -p approved_tests --test robe_capsules -- --nocapture --ignored`. It never opens a window.

For each model the test does the following.

1. It loads the model with the resolver, as `robe_bones.rs` does.
2. It builds the skeleton and the skin vertices the way `game::ragdolls::skeleton` does (`crates/game/src/ragdolls.rs:265`), and calls `Template::fit` with `ragdoll::humanoid::PLAN`.
3. It calls `asset_ja::rig::is_humanoid` on the model.
4. For each capsule it reports the radius, the length, the plan's range and whether the radius sits at the range's maximum or minimum. It also reports how many skin vertices the part received, how many of them are weighted to a robe, cape or cloth bone, and the radius before clamping. It computes that radius again with the loose-bone vertices left out.
5. For the skirt or cape vertices it reports how far each one lies from the nearest capsule surface at rest. The skirt vertices are the ones weighted to a bone whose name contains `robe`, `cape` or `cloth`, plus, for the KOTOR 1 Jedi robes, the vertices of the `legs` mesh, which holds the whole robe.

The radius is clamped to the plan's range. The ranges are 0.060 to 0.095 m for a thigh, 0.045 to 0.070 m for a shin, 0.080 to 0.130 m for the pelvis, 0.090 to 0.140 m for the belly, 0.090 to 0.130 m for the chest and 0.035 to 0.065 m for an upper arm. A part with fewer than 8 vertices gets the maximum. No capsule on any body hit the minimum.

### Is the fit humanoid, and are the robe vertices counted

`is_humanoid` was true and `Template::fit` succeeded for all 50 models that were tried. This includes the capes and the KOTOR 1 cutscene model `m44ac_c01_char01`.

The fit counts every skin vertex. It does not look at bone names. `skin_by_part` (`crates/ragdoll/src/body.rs:517`) takes the bone with the largest weight on each vertex, walks up from that bone to the nearest bone that is a plan part, and gives the vertex to that part. This has three results.

- A KOTOR 2 robe vertex whose largest weight is on a robe bone goes to the pelvis or the belly. The first robe bone of each chain (`frobe1_g`, `mrobe1_g`, `rrobe1_g`) hangs from `pelvis_g` or `torso_g`. No plan part lies below those two bones, so the leg parts never receive these vertices.
- A robe vertex whose largest weight is on a leg bone goes to a leg part even if it also has a robe weight. This is what puts 5 to 16 vertices into each thigh of `pmbim`, `pmbnm`, `pfbnm`, `n_darkjhim`, `n_darkjhif` and `n_talia`.
- A cape that hangs from `torsoupr_g` (Darth Revan's `cape01_g`, and the Malak capes) goes to the chest. A cape that hangs from `pelvis_g` (`n_jedimalek`, `m44ac_c01_char01`) goes to the pelvis.

### Results for the KOTOR 2 robe bodies

Each cell gives the clamped radius in metres, then the radius before clamping. A star after the radius means the radius sits at the plan's maximum. "Skirt vertices" counts the vertices weighted to a robe bone. The last column says where they were counted.

| Body | Humanoid | Robe bones | Skirt vertices | Thigh left, right | Shin left, right | Where the skirt was counted |
|---|---|---|---|---|---|---|
| `pmbim` | yes | 12 | 234 | 0.095* / 0.107, 0.095* / 0.108 | 0.070* / 0.099, 0.070* / 0.100 | belly 180, pelvis 43, thighs 11 |
| `pmbmm` | yes | 10 | 359 | 0.095* / 0.103, 0.095* / 0.097 | 0.070* / 0.078, 0.070* / 0.079 | belly 351, pelvis 8 |
| `pmbnm` | yes | 12 | 238 | 0.095* / 0.109, 0.095* / 0.110 | 0.070* / 0.093, 0.070* / 0.094 | belly 170, pelvis 36, thighs 32 |
| `pfbim` | yes | 13 | 224 | 0.088 / 0.093, 0.092 / 0.092 | 0.070* / 0.079, 0.070* / 0.081 | belly 181, pelvis 41, thighs 2 |
| `pfbmm` | yes | 9 | 359 | 0.088 / 0.088, 0.094 / 0.096 | 0.057 / 0.060, 0.066 / 0.066 | belly 288, pelvis 71 |
| `pfbnm` | yes | 12 | 231 | 0.095* / 0.100, 0.095* / 0.106 | 0.070* / 0.089, 0.070* / 0.081 | belly 179, pelvis 23, thighs 29 |
| `pfbl` | yes | 4 | 32 | 0.074 / 0.076, 0.076 / 0.081 | 0.046 / 0.049, 0.050 / 0.050 | belly 16, pelvis 16 |
| `n_jedirobem` | yes | 13 | 195 | 0.095* / 0.107, 0.095* / 0.111 | 0.070* / 0.097, 0.070* / 0.102 | pelvis 195 |
| `n_darkjhim` | yes | 9 | 179 | 0.095* / 0.108, 0.095* / 0.111 | 0.070* / 0.086, 0.070* / 0.085 | belly 115, pelvis 42, thighs 22 |
| `n_darkjhif` | yes | 9 | 174 | 0.095 / 0.095, 0.095* / 0.104 | 0.070* / 0.080, 0.068 / 0.070 | belly 118, pelvis 34, thighs 22 |
| `n_talia` | yes | 5 | 140 | 0.095* / 0.115, 0.095* / 0.111 | 0.070* / 0.087, 0.070* / 0.085 | belly 57, pelvis 68, thighs 15 |
| `pmbhm` (plain, for comparison) | yes | 0 | 0 | 0.095* / 0.102, 0.095* / 0.103 | 0.070* / 0.092, 0.070* / 0.097 | not applicable |
| `pfbhm` (plain, for comparison) | yes | 0 | 0 | 0.084 / 0.087, 0.088 / 0.091 | 0.070* / 0.086, 0.070* / 0.083 | not applicable |

The thigh and shin capsules of the robe bodies are the same size as the plain bodies' capsules. At the maximum, a thigh capsule is 19 cm across and a shin capsule is 14 cm across. The small female body `pfbl` and the medium robes `pfbmm` and `pfbim` get smaller capsules from their own data, which is the fit working as designed.

Leaving the robe vertices out of the fit changes the unclamped thigh radius by at most 0.9 cm (`n_darkjhif`: 0.104 to 0.096). It does not move any thigh or shin capsule off its limit except in `pfbim` (0.093 to 0.094, which is no change), so the clamped result is the same.

The pelvis and belly are different. With the robe counted, the unclamped belly radius is 0.44 to 0.87 m on the robes, and without it is 0.09 to 0.19 m. Only the plan's maximum of 0.14 m keeps the belly sphere sane. The pelvis radius reaches 0.221 m on `n_talia` and 0.239 m on `n_jedirobem`, and is clamped to 0.13 m. `n_talia`'s pelvis stays at 0.344 m unclamped even with the robe left out, because her dress has more skirt vertices that no robe bone moves.

### Results for the KOTOR 1 Jedi robes

These bodies have no robe bones. The robe is the `legs` mesh, 438 vertices on the male bodies and 460 on the female bodies. It is skinned to the leg bones.

| Body | Thigh left, right | Shin left, right | Robe vertices inside a thigh or shin capsule at rest | Inside any capsule |
|---|---|---|---|---|
| `pmbjl` | 0.095* / 0.148, 0.095* / 0.148 | 0.070* / 0.116, 0.070* / 0.161 | 66 of 438 | 96 |
| `pmbjm` | 0.095* / 0.144, 0.095* / 0.144 | 0.070* / 0.113, 0.070* / 0.157 | 69 of 438 | 107 |
| `pmbjs` | 0.095* / 0.140, 0.095* / 0.140 | 0.070* / 0.111, 0.070* / 0.152 | 79 of 438 | 113 |
| `pfbjl` | 0.095* / 0.152, 0.095* / 0.143 | 0.070* / 0.109, 0.070* / 0.146 | 89 of 460 | 116 |
| `pfbjm` | 0.095* / 0.147, 0.095* / 0.139 | 0.070* / 0.106, 0.070* / 0.142 | 98 of 460 | 130 |
| `pfbjs` | 0.095* / 0.142, 0.095* / 0.135 | 0.070* / 0.102, 0.070* / 0.137 | 109 of 460 | 147 |

For comparison, the plain KOTOR 1 bodies `pmbal`, `pmbam`, `pfbam`, `pmbbm` and `pfbbm` have unclamped thigh radii of 0.089 to 0.103 m and unclamped shin radii of 0.071 to 0.087 m. So the robe adds about 5 cm to the unclamped thigh and up to 8 cm to the unclamped shin. The plan's limit cuts both back to 0.095 m and 0.070 m. The final capsule is therefore 0 to 0.7 cm larger than the capsule of a plain body in a thigh and 0 to 0.2 cm in a shin.

The Jedi hood is a danglymesh that is not skinned and is not counted in the fit.

### Results for the capes

| Body | Humanoid | Loose bones | Skirt or cape vertices | Where they were counted | Thigh radius before clamp | Shin radius before clamp |
|---|---|---|---|---|---|---|
| `n_darthrevan` (both games) | yes | 8 | 272 | chest 226, thighs 18, shins 26 | 0.127, 0.124 | 0.130, 0.140 |
| `n_darthmalak` (both games) | yes | 28 | 220 | pelvis 130, chest 86 | 0.102, 0.101 | 0.087, 0.088 |
| `n_darthmalak02`, `n_darthmalak03` | yes | 6 | 86 | chest 86 | 0.102, 0.101 | 0.087, 0.088 |
| `n_jedimalek` (both games) | yes | 2 | 63 | pelvis 63 | 0.105, 0.103 | 0.109, 0.107 |
| `m44ac_c01_char01` (KOTOR 1 cutscene Revan) | yes | 5 | 120 | pelvis 120 | 0.093, 0.092 | no shin vertices |

All of these are clamped at 0.095 m and 0.070 m except `m44ac_c01_char01`, which has a thigh radius of 0.093 and 0.092 m and a default shin radius of 0.070 m, because its mesh has no shin vertices.

Darth Revan's cape has the one clear case of cape vertices that reach the legs. 18 cape vertices are counted in the thighs and 26 in the shins, and the Revan shin radius before clamping is 0.130 to 0.140 m, against 0.113 to 0.125 m when those vertices are left out. The clamp still holds the capsule at 0.070 m.

### Where the cloth already is, relative to the capsules

For each skirt or cape vertex, the clearance is its distance to the nearest capsule's surface at rest. A negative number means the vertex starts inside a capsule.

| Body group | Skirt vertices | Inside any capsule at rest | Inside a thigh or shin capsule at rest | Deepest, any capsule | Median clearance, any capsule |
|---|---|---|---|---|---|
| K2 robes (11 bodies) | 32 to 359 | 4 to 106 per body | 0 to 19 per body | 1.8 to 5.9 cm | 2.6 to 5.3 cm |
| K1 Jedi robes (6 bodies) | 438 or 460 | 96 to 147 | 66 to 109 | 3.3 to 5.8 cm | 2.2 to 2.8 cm |
| Revan's cape | 272 | 76 | 0 | 4.5 cm | 4.1 cm |
| Malak's cape and cloth | 86 to 220 | 2 to 4 | 0 | 0.4 to 3.5 cm | 9.2 to 22.6 cm |
| `n_jedimalek` cape | 63 | 4 | 0 | 0.1 cm | 5.7 cm |

Most of the vertices that start inside a capsule are in the pelvis, belly and chest capsules. A skirt or a cape lies close to the body there, and the 14 cm belly sphere is the biggest cause. Only the KOTOR 1 Jedi robes and a few KOTOR 2 robes (`pfbnm` 19, `n_darkjhif` 11, `pfbim` 4, `n_darkjhim` 4) start inside a leg capsule.

### What this means for the solver

- The 14 capsules can be used as they are. Their radii are bounded by the plan, so a robe cannot make one grow past 9.5 cm for a thigh.
- A cloth vertex that starts inside a capsule must not be pushed out at the first step. That would make the robe jump outward when the sim starts and on every frame at rest. Each cloth vertex should keep its own collision radius, set at load as the smaller of the capsule's radius and the vertex's rest distance from the capsule's axis minus 0.5 cm. A vertex that starts 2 cm inside then stays at its rest distance from the axis and cannot go deeper.
- A solver that uses the skinned backstop from `2026-11-19-loose-part-body-collision.md` handles the same case in the pelvis, belly and chest, and costs nothing per capsule.
- The belly capsule is a sphere of 14 cm radius on every body. If the solver needs a belly that follows the torso, the plan needs a belly with two ends. That is a change to `crates/ragdoll/src/humanoid.rs`, and the ragdolls use it too, so it should be checked against the ragdoll tests.
- Leaving loose-bone vertices out of the fit was tested and gives no gain. If a fit change is wanted anyway, the better one for the KOTOR 1 Jedi robe is to fit the leg parts without the `legs` mesh, which gives them no vertices and puts them at the plan's maximum. That is what the clamp already does.
- The fit is mostly a fixed size. The 65th percentile distance is above the plan's maximum for the thighs on about half of the plain bodies and for the shins on most of them. A solver that needs tight capsules would need the plan's maxima lowered, or a second fit at a lower percentile. That is outside the question asked here.

## Measurement 2: a way to stress and watch robes and capes

### What kjk already has

- **Bench scenes** (`kjk bench`, `crates/launcher/src/bench.rs`). The scenes are fixed starts with a script. `trayus_academy` is Malachor's Trayus Academy (`arena@903mal`) with a Jedi player, 70 Sith and the world held at tick 125 and 500. A bench scene can also set the body and the moves through `play`'s own arguments. The bench measures mean frame time, p50, p95 and p99, the CPU of the main and render schedules, and the posing time. `--headless` runs it with no window.
- **Which bodies the `trayus_academy` Sith use.** A dump of the scene lists 69 of the 70 characters. 56 are `n_darkjlowm`, 4 are `n_darkjmidf`, 3 are `n_darkjmidm`, 1 is `n_sithappren002` on `n_darkjmidm`, 1 is `n_darthsion`, 2 are `pfbnm` and 2 are `pmbnm`. Only the last 4 have robe bones. The 65 others have no loose bones, so their robes sit in their torso and leg skins and cannot be moved by a robe-bone solver. Other TSL scenes are no better. `arena@906mal` (the proving grounds) has 3 robe-bone bodies in 59, `905mal` has 2 in 34, `702kor` has 1 in 20, and the other TSL areas that were scanned have none.
- **Spawning.** `arena spawn TEMPLATE` calls in a foe from a console command. In `arena@903mal` the templates `g_dark_jed01m001` and `g_dark_jed01f001` are Sith in `pmbnm` and `pfbnm`. A run that spawned 20 of them showed 90 characters, 31 of them `darkjedi`, which is 11 plus 20.
- **Scripted input.** `kjk play --moves SCRIPT` takes segments such as `600ms w jump`. The input is a pure function of the sim's time, so a run repeats exactly. `docs/RUN.md` and `crates/sim/src/moves.rs` document it. The words are `w a s d`, `jump`, `crouch`, `walk`, `attack`, `alt`, `kick`, `block`, `yaw`, `turn` and others. `--body BODY --head HEAD` puts any KOTOR body on the player. `--camera side` films the character from 3 m. `--stage arena` is a grey-box room with nothing else in it.
- **Held-time capture.** `--cmds 'anim T; screenshot NAME'` holds the game's time at second T and takes a picture. `record NAME T0 T1 FPS` renders a frame at each moment from T0 to T1 in turn, and makes an mp4 if `ffmpeg` is found. `clip` and `film` take frames from the running game. All run headless with `--headless --size WxH`.
- **A viewer with an animation.** `kjk rig a --body pmbim --head pmha01 --moves acrobatics --headless --cmds 'anim 3; screenshot x; quit'` plays JA's acrobatics timeline (crouch, rolls, force jump, flips) on a KOTOR body with no sim around it. `kjk view k2 pmbim` shows a model. For KOTOR models, `view` only loops the model's own clip, and its `--clip` option is for Jedi Academy models.
- **`kjk playthrough`.** It plays a module headless, goal by goal (`talk`, `travel`, `fight` and so on), at the authority's top speed with no rendering. It has no acrobatic goals and draws no frames. Its `inputs.kjkr` cannot be drawn from a module yet. It is no use for watching robes. `--run` is only a script of the game's own scripting for the first module, not a move script.
- **Acrobatics in the sim.** Checked by playing the moves and reading the legs animation from a dump of the player:

| Move | Keys in `--moves` | Result seen |
|---|---|---|
| Forward flip | `600ms w jump` | `BOTH_FLIP_F`, 4.8 m up at the top of the jump |
| Back flip | `600ms s jump` | `BOTH_FLIP_B` |
| Side flip | `600ms d jump` while already moving | `BOTH_FLIP_R` |
| Aerial cartwheel right | `150ms d jump attack`, from a stand with the saber lit | `BOTH_ARIAL_RIGHT`, about 3 m to the side |
| Roll | `900ms w` then `300ms w crouch` from a stand | `BOTH_ROLL_F`, about 8 m |
| Kata | `300ms attack alt` from a stand with the saber lit | `BOTH_A2_SPECIAL` |
| Spin | `1.2s turn 720 w` | The view and the body turn two full turns in 1.2 s |
| Kick | `kick` | Needs the duel rules (`docs/COMBAT.md`), not checked here |
| Wall jump, wall run | none | Not in the sim. `anim.rs` lists `BOTH_WALL_RUN_*` names and nothing starts them. |

The saber is lit by the first `attack` press (`300ms attack`). A cartwheel or a roll started during or soon after a flip becomes a side flip or a crouched walk, so each of these moves needs a standing pause of about 2 seconds before it.

### How the robes are posed in these moves

From `2026-11-19-robe-bone-animation.md`, in a fight the sim's JA clips never key the robe bones, so the robe skirt stays at its rest pose. Out of a fight, the body's own `walk` and `pause1` clips move the robe bones. The scenes below put the player in a fight state (the saber is lit and the moves are JA moves), so the baseline robes are rigid. This is the behaviour a solver must replace.

### The proposed repeatable stress scenario

**Characters.**

- A K2 robe body, `pmbim` with head `pmha01`. It has 12 robe bones and 234 skirt vertices. Also `pfbim` with `pfha01` and `n_jedirobem`, as other shapes.
- A KOTOR 1 Jedi robe, `pmbjm` with head `pmhc01`. It has no robe bones, and is the case where the robe is skinned to the legs.
- A cape, `n_darthrevan` with head `n_darthrevanh`. It has 8 cape bones, and none of its clips key them.

**Moves.** One round, about 15 seconds of sim time: run, forward flip, cartwheel right, back flip, run into a roll, a double spin. Each move has a standing pause of 2 seconds before it, as the table above says.

```text
300ms attack; 1s;
800ms w; 600ms w jump; 2s; 150ms d jump attack; 2.4s; 600ms s jump; 2s;
900ms w; 300ms w crouch; 2s; 1.2s turn 720 w; 1s
```

**Calm capture for a side by side.** One character alone in the grey-box arena, filmed from the side, with every frame rendered at a held time.

```bash
kjk play --stage arena --class jedi --body pmbim --head pmha01 --camera side \
  --headless --size 1280x720 \
  --moves '300ms attack; 1s; 800ms w; 600ms w jump; 2s; 150ms d jump attack; 2.4s; 600ms s jump; 2s; 900ms w; 300ms w crouch; 2s; 1.2s turn 720 w; 1s' \
  --cmds 'wait 30f; record robe_before 2.5 13 10; quit'
```

Run the same command on the build with the solver and name the record `robe_after`. Frame N of the two runs is the same game time, so the two sets can be put side by side. `kjk compare-pictures` measures how far two frames differ. The same command with `--body pmbjm --head pmhc01` and with `--body n_darthrevan --head n_darthrevanh` covers the K1 robe and the cape. For one picture, use `anim T; screenshot NAME`. The times in this move script, as seen in a dump, are 3.5 s to 5 s for the forward flip, 6 s to 6.5 s for the cartwheel, 8.5 s to 9.5 s for the back flip and 12 s to 12.5 s for the roll.

One condition belongs to the solver and not to this scenario. For the two runs to match frame by frame, the solver must step by the game's held time and not by the wall time of each frame. If it uses the frame's real delta, two runs of `record` give different cloth. The solver should therefore take its step from the game's time, in fixed substeps.

**Benchmark.** The bench scene `robe_acrobatics`, described below, plays the same round in the Trayus Academy among 110 characters, 44 of which have robe bones, so that the cost of many robes shows. It reports mean fps and the 1% low fps (1000 divided by p99 frame time). A scene with a solver that costs 0.1 ms for each robed character should show a loss of about 4 ms in `main ms` for these 44.

### What was missing and what was built

A bench scene with a robed player and many robed foes did not exist. It was small to add, so it was built. It is in the worktree `/Users/david/perso/kjk/.claude/worktrees/robe-stress`, at commit `af1379f3` (current `main`), with one uncommitted change to `crates/launcher/src/bench.rs` that adds 56 lines. It is not committed, and the checked-out source of the main checkout was not touched.

- The scene is `robe_acrobatics`. It is held out, so it runs only when named: `kjk bench --headless --scenes robe_acrobatics`.
- Its arguments are `play --stage arena@903mal --class jedi --body pmbim --head pmha01 --camera side --moves <5 rounds>`.
- Its setup is `tick 125; arena god; <40 spawns of g_dark_jed01m001 and g_dark_jed01f001>; wait 10f; tick run`. The player is immortal, as in the other fights.
- The two macros are `robed_foes!` and `acrobat_round!`.

Because the scene is a fight, the Sith swarm the player, and the robes in the picture are the rigid ones the original sim leaves. A bench figure from this scene therefore includes the pose time of 110 characters and the effect of the crowd on the camera. This makes it a good scene for the cost of many robes, and a poor one for judging a single robe's look. The calm capture serves that.

### Baseline, headless

Both runs were `kjk bench --headless` with the default preset, 5 seconds of warm-up and 20 seconds measured, at 1280x720, on the Mac (M4 Max, Metal), on the build `af1379f3`. The 1% low is 1000 divided by the run's worst p99 window, as in `2026-11-19-dangly-sway-cost.md`. The machine was in use by others. The load average was 13 during run A and 6 during run B. That is why the same scene differs between runs.

| Run | Scene | Mean fps | 1% low fps | p99 ms | Main ms | Render ms | Pose ms | Posed per frame |
|---|---|---|---|---|---|---|---|---|
| A | `trayus_academy` | 181.6 | 202 | 4.95 | 4.46 | 2.87 | 0.218 | 17.1 |
| A | `robe_acrobatics`, first move script | 55.4 | 18.1 | 55.23 | 9.69 | 12.67 | 0.512 | 60.1 |
| B | `trayus_academy` | 199.0 | 262 | 3.81 | 4.06 | 2.57 | 0.198 | 15.5 |
| B | `robe_acrobatics`, final move script | 94.9 | 46.7 | 21.41 | 6.15 | 7.78 | 0.476 | 57.5 |

`robe_acrobatics` in run A used the first draft of the move script, which had no pauses and a kata. Run B is the final script, and the one to compare against. The scene's frame is dominated by the GPU, as the camera sits close to 40 robed Sith with swords. The 1% low is therefore much worse than the mean. For a solver, the numbers to watch are `main ms` and the 1% low, and the baseline run should be repeated 3 times on a quiet machine before a comparison is made.

The picture of the final frame is kept at `/Users/david/perso/kjk/.claude/worktrees/robe-stress/kjk-artifacts/bench/baseline-b/robe_acrobatics-default.png`, and the results are in `results.jsonl` beside it.

### Baseline frames of the calm capture

105 frames of the calm capture on `pmbim` were rendered at 640x360 and 10 frames a second, game time 2.5 s to 13 s. They are in `/Users/david/perso/kjk/.claude/worktrees/robe-stress/kjk-artifacts/runs/20261119-174039-47228/` as `robe_before-NNNN.png` (12 MB). The `kjk rig a` viewer was also tried and gives a picture, so it is available for a timeline of JA's own acrobatics.

### What the capture does not yet cover

- Wall jumps and wall runs do not exist in the sim. If the roadmap needs robes in those, the moves must first be built, and that is a sim feature and not a view one.
- Only the robed player is moved by `--moves`. The foes in the bench scene fight on their own. A fixed, repeatable look at several robed characters at once needs the calm arena and a second character, which `kjk play` does not offer apart from a joined seat (`kjk host` and `kjk join`). One robed character at a time is enough for the first solver steps.
- A recording from `record` has the HUD in it, as `--camera side` does not hide it. A step that compares pictures should crop the picture to the character.

## Files

- `/Users/david/perso/kjk/.claude/worktrees/robe-bones/crates/approved_tests/tests/robe_capsules.rs` and the two dependency lines in `/Users/david/perso/kjk/.claude/worktrees/robe-bones/crates/approved_tests/Cargo.toml`. Neither is committed.
- `/Users/david/perso/kjk/.claude/worktrees/robe-stress/crates/launcher/src/bench.rs`, the new scene. Not committed.
- The code read: `crates/ragdoll/src/body.rs`, `crates/ragdoll/src/humanoid.rs`, `crates/game/src/ragdolls.rs`, `crates/asset_ja/src/rig.rs`, `crates/sim/src/moves.rs`, `crates/sim/src/motion/force_jump.rs`, `crates/launcher/src/saber_verbs.rs`, `crates/launcher/src/bench.rs`, `docs/RUN.md`, `docs/SKELETON.md`.
