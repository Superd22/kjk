# pmhc04's eyes: diagnosis and plan (2026-11-21, night)

Written by the conversation kjk-5b for the lead kjk-48. This is phase 1 only. Nothing live was changed. No file in the worktree was edited. Everything below was made in the scratchpad `/private/tmp/claude-501/-Users-david-perso-kjk/29f40f34-3353-42e8-b2a3-84ee288b3c26/scratchpad/` (called `$S` below).

## Where I am

- Phase 1 is done. The diagnosis, the measures and the plan are in this note. The lead changed the scope to phase 1 only, so round 1 is for a fresh conversation.
- The roadmap's "Now and next" eye line was not edited, because the work has not ended.

## The verdict in short

Our eyes miss KOTOR's in two different ways.

1. **Every head shares a built-in difference.** In KOTOR, a large eyeball pokes about 4.5 mm out through a flat, recessed opening, the outer corner sits about 13 mm behind the eyeball's front, and the lid piece juts forward like a visor. Our construction presses the lid edges onto the eyeball and lets the lid skin wrap around it. No slider value can change that. This causes faults 3 and 4 on all 57 heads, and most of fault 2.
2. **pmhc04's solve makes it worse.** The solver matches the eye's existing 27 measures almost perfectly on pmhc04, but those measures do not read what David sees. To match them, the solve sets Eye depth to -4.83 and Eyeball size to +2.70. This pushes the whole eye 4.8 mm back into the head and inflates the eyeball, so the skin beside the nose ends up far in front of the eye. This causes fault 1.

## The pictures

All paths are under `$S/pics/`. Each sheet's left half is clay and its right half is viewer 2's "In game" light. Files whose names start with `s` are the same sheets shrunk for reading.

- `pmhc04_layers.png` (lit half `pmhc04_layers_lit.png`, shrunk `s1.png`). These are pmhc04's layers, one row each. The rows are KOTOR; LIVE (viewer 2's shown head, all five features); STAND-IN (live start with every value at 0); BASE+VALUES (the base head with pmhc04's live values); NO PRESS (live with the lid press switched off); and NO EYE SLIDERS (live with the eye values at 0). The columns are the front, three-quarter from the temple side, three-quarter from the nose side, and profile.
- `kl_8.png` (lit `kl_8_lit.png`, shrunk `s2.png`). This sheet shows KOTOR against LIVE on 8 heads: pmhc04, p_bastilah, p_carthh, p_kreiah, pfha05, pmha01, comm_a_m and n_zherronh.
- `slab.png` (shrunk `s5.png`). It shows a vertical slice 3 mm wide through the middle of the eye, seen from the side, for KOTOR and LIVE on the same 8 heads. It shows the order of the lid piece, the lid skin and the eyeball directly.
- `trial1.png` (`s3.png`). It shows hand-set trials of existing sliders on pmhc04.
- `trial2.png` (`s4.png`). It shows hand-set trials of a crude prototype of a new "eyeball forward" control.
- `extremes.png` (lit `extremes_lit.png`, `s6.png`). It shows KOTOR's heads at the low and high ends of the new measures, to check that the extremes look right.

## The four faults and their causes

The new measures in the "Measures" section below give the numbers.

**1. The inner corner looks buried, with the nose bridge bulging beside it.** David's reading is right in what it shows, but the nose is not the cause. On pmhc04, the skin 2 to 6 mm on the nose side of the inner corner stands 1.8 mm in front of the eyeball's front in KOTOR. In ours, it stands 13.4 mm in front. In absolute terms, KOTOR's eyeball front sits at 14.8 mm and ours at 11.6 mm, so ours is 3.3 mm too far back. The skin beside the corner sits at 16.6 mm in KOTOR and 25.0 mm in ours, so ours is 8.4 mm too far forward. Setting each nose, brow and cheek slider to 0 in turn moves this skin by at most 0.75 mm, so the nose sliders are not the cause. The eye sliders are. Eye depth at -4.83 pushes the eye back, and its fade carries part of the nearby skin with it. Eyeball size at +2.70 inflates the eyeball, and the press wraps the lids around it. Inner corner position at -4.98 moves the corner 4.5 mm toward the temple along the eyeball. That leaves a long hollow, and the caruncle fills it as a large pink wedge. That wedge is what makes the corner look "creepy". The stand-in adds about 3 mm to this skin on top of the base head. Bastila, Carth, Kreia and Zherron show the same pattern, with smaller gaps.

**2. The skin around the outer corner bulges like a cushion.** Our opening ends 7.5 mm behind the eyeball's front, but KOTOR's ends 12.8 mm behind it. So our lid skin closes around the side of the eyeball as a rounded pad, where KOTOR's eyeball keeps going back into the head. The bulge measure gives 4.5 mm for ours and 3.2 mm for KOTOR. Over KOTOR's 57 heads this bulge varies with a spread of 3.6 mm, but ours varies by only 0.87 mm. So no slider controls it today. Pushing Outer corner position further makes the bulge worse, because the lid then wraps further round the eyeball (trial T3 in `trial1.png`).

**3. In profile, the lids wrap the eyeball.** This fault is global. In KOTOR's profile, a median of 43 mm² of eyeball shows, against 15 mm² on our heads and 13 mm² on the base head. pmhc04 shows 53 mm² against our 12 mm². The eyeball stands 4.5 mm in front of the opening's rim in KOTOR (median), and 3.2 mm in ours. The cause is the construction. The press keeps every lid edge within 0.3 mm of the eyeball, and the lid skin fades smoothly from the edge to the socket. A bigger or more forward eyeball just pushes the lids out with it. Trials T1 to T5 in `trial1.png` set Eye depth, Eyeball size, Inner corner depth, Outer corner position and Eye width by hand. None of them shows KOTOR's wedge of eyeball in profile.

**4. The lid order looks reversed.** The slices in `slab.png` show it clearly on pmhc04, pmha01, comm_a_m and Carth. KOTOR's lid piece is a wedge that juts forward like a visor in front of the top of the eyeball, with the skin above it meeting the piece's top. Ours is a thin shell that lies on the eyeball under the skin's lid edge, with the skin and the piece concentric. In the front view, our piece does show as the front-most surface just above the opening, over 82% of that band on pmhc04 against 85% in KOTOR, so the order is not literally reversed there. The difference is the piece's angle and how far it reaches forward. Over KOTOR's heads, the piece reaches 2.9 mm in front of the eyeball's front (median), with a spread of 3.3 mm. Ours reaches 1.8 mm with a spread of 0.3 mm, so our piece is the same on every head. On pmhc04 alone, the reach is close (2.4 mm in KOTOR, 2.1 mm in ours). What differs there is that our lid skin is as far forward as the piece.

## Why the fit lands so badly

- **The solve is faithful to measures that are blind to these faults.** On pmhc04, the live head matches KOTOR on width (28.2 against 26.6 mm), opening height (7.0 against 7.4), depth of the rim (12.1 against 12.1), eyeball size (14.8 against 14.7) and nearly every other proportion. The audit file is `~/.claude/tools/likeness-fit-data/sliders/eyes/audit.json` (columns built, built_no_press, kotor, standin). None of the 27 proportions reads the eyeball's front against the skin beside the nose, the eyeball seen in profile, the outer corner's depth, or the lid piece's reach.
- **Two measures pull against each other.** The "ball" measure reads KOTOR's eyeball front as a radius about our eye's centre. KOTOR's eyeballs are large and sit back, so the measure reads a large radius (14.7 mm on pmhc04). The solver answers with Eyeball size +2.70. The "depth" measure reads the rim, and the solver answers with Eye depth -4.83. Eye depth moves the eyeball, the lids and part of the skin around them together, so the solve cannot bring the eyeball forward while keeping the rim back. KOTOR's look needs exactly that.
- **The sliders that could help are not solved.** Inner corner depth, Lower lid flatness and Caruncle width have no measure, so the presets leave them at 0 (round 6's bench change).
- **KOTOR's low-poly mesh.** The new measures read KOTOR's mesh, its own eyeball and its violet lid piece from `eyes/kotor_eyes.json` through a z-buffer. Their left and right eyes agree within 0.1 to 1.1 mm on the depth measures, and within 4 mm² on profile exposure. That is well under KOTOR's spread between heads, so they read shape and not mesh noise. I did not check how the old "depth" measure reads KOTOR's rim where the lid piece covers it.

## Measures

Code: `$S/diag/src/main.rs`, function `probe` (front and side z-buffers of a scene, each pixel tagged skin, eyeball, lid piece or caruncle). Each value is the mean of the two eyes, with the -x eye mirrored. Units are mm in the head frame, and mm² for areas. The data come from `$S/m_before.json` (all 57 heads; K is KOTOR, L is live, E is live with the eye values at 0, N is no press, S is stand-in, B is the base head). The summary is `$S/summary.txt`, and `$S/summ.py <file>` prints it.

| Measure | What it reads | KOTOR median | KOTOR spread | Wobble (left against right) | Ours median | Ours spread | KOTOR against ours (correlation) |
|---|---|---|---|---|---|---|---|
| nose_skin_z | Skin 2 to 6 mm on the nose side of the inner corner, in front of the eyeball's front | 5.0 | 2.7 | 1.1 | 10.1 | 4.0 | 0.46 |
| inner_sink | The inner corner's depth against that skin | -12.2 | 3.3 | 0.9 | -14.9 | 2.8 | 0.34 |
| profile_ball | The eyeball's area seen in profile | 43.3 | 15.9 | 3.9 | 15.3 | 8.5 | 0.38 |
| outer_back | The outer corner's depth behind the eyeball's front | -13.0 | 2.5 | 0.65 | -8.6 | 1.8 | 0.60 |
| ball_proud | The eyeball's front in front of the opening's rim (mean of the rim) | 4.5 | 1.0 | 0.07 | 3.2 | 0.55 | 0.31 |
| outer_bulge | The highest skin 1 to 5 mm beyond the outer corner, in front of the corner | 5.4 | 3.6 | 0.67 | 5.3 | 0.87 | 0.42 |
| piece_reach | The lid piece's front in front of the eyeball's front, middle 40% of the eye | 2.9 | 3.3 | 0.10 | 1.8 | 0.30 | -0.23 |
| ball_front_abs, nose_skin_abs | The same depths in the head frame, not relative to the eyeball | 14.7, 20.5 | 2.6, 3.3 | 1.2, 0.3 | 12.8, 23.3 | 2.9, 3.0 | 0.82, 0.52 |

The four checks:
- **It ignores what nobody notices.** The wobble between the left and right eye is 4 to 30 times smaller than KOTOR's spread on every measure.
- **It sees what David notices.** pmhc04 is an outlier exactly where David complained: nose_skin_z 1.8 against 13.4, and profile_ball 53 against 12. Bastila shows the same faults, with the same signs.
- **It tells KOTOR's heads apart.** Yes. KOTOR's spread is large next to the wobble, and our spread is much smaller than KOTOR's on outer_bulge, ball_proud and piece_reach. That shows our sliders cannot make the variety.
- **Its extremes look right.** I checked this by eye in `extremes.png` for profile_ball (sith_app and old_w_f are high, and the eyeball wedge is obvious), outer_back (old_b_f and pfhb04 reach far back) and nose_skin_z (Terena Adare and mod_keanu have flush inner corners). mod_hp_pfha01's high nose_skin_z may come from its heavy lid piece. I did not check outer_bulge's or piece_reach's extremes.

Measures that failed: `lid_order` (median of the piece's depth minus the skin's depth per pixel) picks up the skin inside the socket and is unusable. `lid_proud` (the piece's front minus the skin's front over the band 0 to 4 mm above the opening) does not separate KOTOR from ours. `thick_up` (how far the skin stands in front of the eyeball 0.5 to 2 mm outside the opening) has a correlation of 0.00. nose_skin_z's window moves with the inner corner, so a slider that moves the corner also moves the window. Read nose_skin_abs next to it.

Per-head numbers, as KOTOR / ours:

| Head | nose_skin_z | profile_ball | outer_back | ball_proud | outer_bulge | piece_reach | ball_front_abs | nose_skin_abs |
|---|---|---|---|---|---|---|---|---|
| pmhc04 | 1.8 / 13.4 | 53.4 / 11.9 | -12.8 / -7.5 | 4.1 / 2.5 | 3.2 / 4.5 | 2.4 / 2.1 | 14.8 / 11.6 | 16.6 / 25.0 |
| p_bastilah | 1.9 / 5.8 | 47.7 / 21.0 | -15.9 / -8.2 | 3.9 / 3.8 | 6.6 / 4.1 | 8.6 / 1.9 | 11.4 / 12.2 | 13.3 / 18.0 |
| p_carthh | 5.1 / 13.2 | 33.1 / 9.7 | -11.3 / -7.7 | 3.5 / 2.6 | 5.9 / 5.1 | 3.2 / 2.0 | 11.9 / 8.7 | 17.1 / 21.9 |
| p_kreiah | 1.7 / 7.1 | 60.1 / 12.2 | -9.9 / -6.6 | 3.1 / 3.2 | 1.4 / 4.0 | 6.1 / 1.2 | 11.3 / 11.7 | 13.0 / 18.8 |
| pfha05 | 3.2 / 4.6 | 39.2 / 19.7 | -14.6 / -10.2 | 5.8 / 3.9 | 5.1 / 6.0 | 8.9 / 1.6 | 18.8 / 17.4 | 22.0 / 22.0 |
| pmha01 | 4.1 / 7.2 | 48.9 / 15.8 | -11.6 / -8.4 | 4.5 / 3.1 | 3.4 / 4.5 | 0.8 / 1.8 | 18.5 / 17.4 | 22.6 / 24.7 |
| comm_a_m | 3.9 / 8.4 | 49.8 / 15.2 | -11.7 / -7.9 | 4.5 / 2.9 | 3.0 / 4.4 | 0.7 / 1.6 | 17.1 / 15.5 | 21.0 / 23.9 |
| n_zherronh | 0.5 / 10.1 | 28.6 / 8.0 | -7.8 / -6.0 | 2.8 / 2.5 | 3.7 / 4.4 | 1.6 / 2.0 | 15.5 / 14.1 | 16.1 / 24.2 |

## What each slider does to pmhc04

Run `pmhc_diag blame` (see the commands below). It sets each slider to 0 in turn. Only these sliders move the new measures by more than 0.5:
- Eye depth (-4.83). At 0, nose_skin_z falls from 13.4 to 11.1.
- Eyeball size (+2.70). At 0, profile_ball falls from 11.9 to 6.1 and the opening's area from 133 to 79 mm². The eye then looks small and the caruncle huge (`trial1.png`, T1).
- Inner corner position (-4.98). At 0, the inner corner moves from 17.2 to 12.6 mm from the middle of the face. KOTOR's is at 19.1 mm.
- Outer corner position (+3.00). At 0, outer_back goes from -7.5 to -5.9.
- Eye width (-2.48). At 0, outer_back goes to -9.5 and profile_ball to 17.5.
- Upper lid cover (+1.21).
- No nose, jaw, brow or cheek slider moves any of them by more than 0.75 mm.

## The prototype (not a plan to ship as is)

`prominence()` in `$S/diag/src/main.rs` moves every vertex inside an oval around each eye forward after the build, fading out over the socket. Its parameters are forward mm, half width, half height, fade and the share kept at the outer corner. In `trial2.png`, "P5 OUTER BACK" (5 mm forward, the outer corner held back) gives a KOTOR-like wedge of eyeball in profile, with profile_ball 26 mm² against 12, and outer_bulge drops from 4.5 to 1.1. But the eyeball pokes through the cheek below the lower lid, because the move comes after the press and the eyeball is a whole sphere. The real control must be a slider field applied before the press, with the lower lid skin carried along.

## The plan

The order is chosen so that each round is useful on its own. Each round ends with the before and after sheets, the measure table on all 57 heads, the fold check and a blind judge, as the original brief says.

### Round 1: measure what David sees, and add the missing "eye forward" slider

1. **Add the proven measures to the eye feature** (`head_sliders/src/eyes.rs`, PROPORTIONS and `measure_with`). Add nose_skin_z (or inner_sink), outer_back, ball_proud and profile_ball. Port `probe`'s z-buffer reads onto `Surface` plus `lids`, so the solver reads them on the shown head as it reads the others. Fixes faults 1 and 3. Risk: every preset is re-solved, so every head moves. Compare all 57 with the table.
2. **A new slider "Eye prominence" (working name).** It moves the eyeball's centre ring, the lid edges and the lid skin up to about 4 mm outside the opening forward along `Frame::head`'s forward axis. It fades to 0 over about 8 mm, so the skin beside the nose, the brow and the cheek stay. The lower lid and the skin under it must move with the eyeball so that the eyeball never shows through the cheek. Build it as a field before the press, like Inner corner depth (`eye_curves.rs`). Its measure is ball_proud, or profile_ball. Fixes faults 1 and 3. Risk: the eyeball showing outside the opening. Add a check that counts eyeball pixels outside the opening in the front view. It must be 0.
3. **Give Inner corner depth a measure** (nose_skin_z or inner_sink), so the solver can use it. Today the presets leave it at 0.
4. **Stop the solve from inflating the eyeball to fake a forward eyeball.** Lower the "ball" proportion's weight, or read it against the new slider. Then re-solve the presets (`sliders eyes presets <data>`).
5. The target on pmhc04: nose_skin_z under 6 mm (now 13.4), profile_ball over 30 mm² (now 12), and outer_back nearer -12 (now -7.5). No head may get a new fold. No other head may move further from KOTOR on these measures by more than its wobble, unless a judge prefers its after picture.

### Round 2: the outer corner reaches back, and the cushion goes

1. **A new slider "Outer corner depth"**, the mirror of Inner corner depth. It sets the outer corner back along the head axis, and the lid shortens instead of folding. The press must let the outer lid's end sit behind the eyeball's silhouette, where today it hands over to the radial press. Its measure is outer_back. Fixes faults 2 and 3.
2. **Use outer_bulge as a guard**, so that Outer corner position can no longer buy width by wrapping the lid round the eyeball.
3. Risk: a gap between the lid's end and the eyeball, seen from the temple side. Check it in the three-quarter close-ups.

### Round 3, only if David still sees fault 4 after rounds 1 and 2: the lid piece as a visor

1. **Add a setting to the open lid piece** (`head_eyes/src/piece.rs`), exposed as a slider ("Upper lid angle" or "Lid piece reach"). It turns the open piece forward about its hinge, so that its front edge stands in front of the eyeball. The skin's lid edge then tucks just behind the piece's top. The measure is piece_reach, plus a slice picture. Fixes fault 4.
2. Risk: the blink, because the closed angle is computed from the open piece. Check closed lids on the base head and the 57 heads, as round 4 did.
3. Possibly also: shrink the caruncle when Inner corner position is strongly negative, so that a long hollow is not filled by a large pink wedge. Today pmhc04's caruncle is the large wedge.

If round 1's slider cannot bring pmhc04 near its targets without the eyeball poking through the cheek on other heads, stop and report. The construction would then need the eyeball to be allowed partly behind skin, like KOTOR's. That is a larger change for David to decide.

## Commands

```
export PATH="$HOME/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin:$PATH"
S=/private/tmp/claude-501/-Users-david-perso-kjk/29f40f34-3353-42e8-b2a3-84ee288b3c26/scratchpad
D=~/.claude/tools/likeness-fit-data/sliders        # the live data folder; use a round's copy for "after"
cd $S/diag && cargo build --release                 # its own workspace and target; links head_sliders and head_eyes from the worktree
B=$S/diag/target/release/pmhc_diag                  # $S/pmhc_diag_before = the binary built on today's crate code, for "before"
SLIDERS_PX=8 $B sheet $D out.png pmhc04             # layers sheet; DIAG_ROWS=K,L picks rows (K L S B N E)
SLIDERS_PX=6 DIAG_ROWS=K,L $B sheet $D out.png pmhc04 p_bastilah p_carthh p_kreiah pfha05 pmha01 comm_a_m n_zherronh
SLIDERS_PX=8 $B slab $D out.png <heads...>          # vertical slices; SLAB_X sets the slice's distance from the middle (31.5 mm)
$B measure $D all > m.json; python3 $S/summ.py m.json   # the measures on all 57 heads; K, L, E, N, S, B per head
$B blame $D pmhc04                                  # each eye slider at 0 in turn; BLAME_ALL=1 for every feature's sliders
TRIALS="name|Eye depth=0,Eyeball size=0;;name2|prom=5:14:7:1.8:0.2" SLIDERS_PX=6 $B trial $D out.png pmhc04
```

Before and after sheets: run the same `sheet` and `slab` commands with `$S/pmhc_diag_before` on the live data folder, and with a freshly built `pmhc_diag` on the round's data copy. Stack them with the likeness-fit venv's Python (`~/.claude/tools/likeness-fit/.venv/bin/python -I`). The system Python has no PIL. Compare `measure` outputs with `summ.py`. The diag reads viewer 2's five feature sets from `<data>/web/` exactly as eye_audit's `shown_heads` does, so its LIVE row is viewer 2's head.

Every sheet uses the +x eye. The columns are the front (yaw 0), the temple three-quarter (yaw -40), the nose three-quarter (yaw +35) and profile (yaw -90).

## Traps

- The diag links the crate from the worktree, so a change in `head_sliders` changes the diag after a rebuild. Keep `$S/pmhc_diag_before` for the before pictures.
- The eye page has 18 sliders and `web/eyes/sliders.json` has 24. Sliders without a measure (Inner corner depth, Lower lid flatness, Caruncle width) are left at 0 by the presets.
- Inner corner position's negative values move the corner toward the temple, not toward the nose.
- KOTOR has no caruncle. Its inner corner shows white eyeball up to the tip.
- The "ball" proportion is a front match about our eye's centre, not KOTOR's real radius. KOTOR's eyeballs are larger and sit 5 to 10 mm further back.
- The prototype `prominence()` runs after the press. It is only for pictures. It pokes the eyeball through the cheek.
- In the slab pictures, clay renders the back faces dark. Read the shapes, not the shading.
- Another conversation (the brow edge work, kjk-be) may build in `head_sliders`. Run ListAgents and tell it before you build there.
