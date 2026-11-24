# GPU paint status

Written by the GPU paint coordinator on 2026-11-20. It replaces a message the coordinator could not send.

## Forehead lines on the page (about 00:40 UTC)

- The forehead lines are on the live page with the GPU painter.
- They pass the check against the processor painter at their hand-set values on Kreia and `old_w_m`. On each head, 100% of the 3,039,835 painted pixels are within 1 step of 255, with the lines alone and in the whole stack.
- I saw them in headless Chrome without the hair. Both heads show their wavy lines above the brows, and the lines disappear when the amount is set to 0.
- A repaint at 2,048 pixels takes about 22 ms in Chrome, with the read back.
- The wrinkle sets and the fine lines were not yet in the page's GPU painter at that time. The page named those sliders as not painted and pointed to `?painter=cpu`. They are being put in next, with the rule that turns off the old forehead set while the lines' amount is above zero.

## Every layer on the page (about 00:55 UTC)

- All five layers and the forehead lines now paint on the GPU, in the live page and natively. The page lists no slider as unpainted.
- The check passes on the 6 heads, each layer alone and all together, natively and in headless Chrome. The details, the timings and the memory are in `~/.claude/tools/likeness-fit/WASM-BUILD.md`, section "head_paint_gpu, the paint on the GPU".
- A slider move repaints the full 2,048 pixels in 50 to 98 ms in Chrome on this loaded Mac. The fine lines take about 45 ms of it.
