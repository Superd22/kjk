# Lid piece: handoff (2026-11-20)

The task: a generated upper lid piece per eye with a lash strip, turned rigidly by KOTOR's lid bone, in `head_eyes` (new modules `piece.rs`, `piece_check.rs`, `web.rs`, tool `src/bin/lid_piece.rs`), and a page at http://127.0.0.1:8765/lids/ (source `~/.claude/tools/likeness-fit/hq/lids/`, data `~/.claude/tools/likeness-fit-data/lids/`). Scratch work is in `/private/tmp/claude-501/-Users-david-perso-kjk/323ce000-e135-4d36-a800-8a864c2e749a/scratchpad/lids/`.

## Facts found (about 01:00)
- The 28-vertex edge loop is a mesh boundary: the top of the lid's lining, lying on the eyeball (r 12.2 to 12.9 mm mid-lid, 15 mm at the inner corner). Each edge vertex has a coincident copy (`edge_copies`) that starts a separate socket sheet going back behind the eyeball.
- From the edge, the front skin goes down 1 to 2 rings to the lid margin (lowest point, r about 12.1 to 12.2, touching the eyeball), then out and up. On the base head the margin is ring 3 of the upper lid at x = -2.7 mm.
- Viewer data: `heads/<name>.json` `ours.pos` = `fit_smooth[viewer_map]` exactly; KOTOR's head (`kotor.materials`, unindexed) is in the same fit frame, skinned by `kskin/<name>.json`; KOTOR's lids are bones 21 (eyellid, our +x eye) and 22 (eyerlid).
- Clip lid turn (s_male02): rest about 1 degree, blink peaks 35.3 (tlknorm), 35.1 (tlksad); closing is a turn about our +x by a positive angle.
- Bastila's KOTOR lid pivot is 5.3 mm in front of her eyeball node.

## Where I am (about 01:50)
- Built: `head_eyes/src/piece.rs` (generator), `src/piece_check.rs` (checks, `build_and_check`, `EyeCheck::passes`), `src/web.rs` (LidMaker), `src/bin/lid_piece.rs` (`export`, `report [open] [match]`, `debug <head> <eye>`; `LID_DEBUG=1` prints paths and columns, `LID_SHOWN=1` prints uncovered eyeball points). Reports: `likeness-fit-data/lids/report.json` and `report_clip.json`.
- Page: http://127.0.0.1:8765/lids/ (`?debug` red piece, `?nopiece` the head without it, `?nomask`). Scripts in the scratchpad `lids/`: `shoot.mjs <dir> <heads> <cams> <amounts> [settings json]` (env Q adds URL flags), `sheet.py`, `bench.mjs`, `ui.mjs`.
- Two judging rounds done (fresh Sonnet agents). Left: the closed lid reads as a plate of another colour; the lash strip pokes through the lower lid when closed; eyeball shows when closed on pfha06 (a hole in the fitted lower lid), old_w_f, pfhc02 and pfha05.
- Roadmap: entry under "Running now" and decision 59.
