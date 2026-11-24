# Building the head crates for the browser

This note is shared by every session that builds a head crate for the browser. Read it before you change `sliders/build_wasm.sh`, the tool in `sliders/bin-tools`, or the `wasm` and `nose` links in the served folder. Add to it when you change any of them. It was started on 2026-11-19 by the nose slider session.

## What is installed

- The target `wasm32-unknown-unknown` is installed for the pinned toolchain 1.98.1 (`rust-toolchain.toml` of the head-paint worktree), with `rustup target add wasm32-unknown-unknown --toolchain 1.98.1`.
- The tool `wasm-bindgen` (the command line program), version **0.2.129**, is in `~/.claude/tools/likeness-fit/sliders/bin-tools/bin/wasm-bindgen`. It was installed with `cargo install wasm-bindgen-cli --version 0.2.129 --locked --root ~/.claude/tools/likeness-fit/sliders/bin-tools`.
- The version must match the `wasm-bindgen` crate in the worktree's `Cargo.lock`, which is 0.2.129 (Bevy already pulls it in). A crate that uses it pins the same version in its own `Cargo.toml`, as a dependency for that target only: `[target.'cfg(target_arch = "wasm32")'.dependencies] wasm-bindgen = "0.2.129"`. Do not change the version or move the tool without writing it here first, because the paint build depends on both.

## The build command

```
~/.claude/tools/likeness-fit/sliders/build_wasm.sh <crate> [out dir] [worktree]
```

It works for any crate of the worktree. It builds the crate's library in release mode for `wasm32-unknown-unknown`, then runs `wasm-bindgen --target web --no-typescript`.

The worktree was a constant in this script until 2026-11-22, always `~/perso/kjk/.claude/worktrees/head-paint`. It is now the third argument, or the environment variable `KJK_WORKTREE`, and it still defaults to that same worktree, so every old command line keeps working. The script refuses a path with no `rust-toolchain.toml` and prints the crate, the worktree and the output folder before it builds. This was changed so that two conversations in two worktrees can each build their own module. The flow is the section "Working in parallel on the head" in `docs/research/2026-11-19-head-roadmap.md`, and `sliders/serve_feature.sh` sets a conversation's stack up for it. The crate needs `crate-type = ["rlib", "cdylib"]` under `[lib]`, and its browser functions marked with `#[wasm_bindgen]` in a module compiled only for that target (`#[cfg(target_arch = "wasm32")] pub mod web;`). See `head_sliders/src/web.rs` for an example.

The output goes by default to `~/.claude/tools/likeness-fit/hq/wasm/<crate>/`: `<crate>.js` (the glue) and `<crate>_bg.wasm` (the module).

### The libLLVM.dylib workaround

The pinned toolchain is installed with the minimal profile. Its linker for this target, `rust-lld`, looks for `libLLVM.dylib` next to itself (`lib/rustlib/aarch64-apple-darwin/lib/`) and does not find it, so linking fails with "Library not loaded: @rpath/libLLVM.dylib". The library is in the toolchain's own `lib/` folder. The script sets `DYLD_FALLBACK_LIBRARY_PATH` to that folder before calling cargo, which fixes it. The same message appears as a harmless warning ("stripping debug info with rust-objcopy failed") on every native release build.

## How a page loads a module

The local server (port 8765, never restart it) serves the folder `viewer-local` in the first session's scratchpad. Two links were added to that folder by hand, without rerunning `serve_local.sh`:
- `viewer-local/wasm` points to `~/.claude/tools/likeness-fit/hq/wasm`, so a module is at `/wasm/<crate>/`;
- `viewer-local/nose` points to `~/.claude/tools/likeness-fit/hq/nose`, the nose page.

Rerunning `serve_local.sh` deletes everything in `viewer-local`, these two links included. Recreate them afterwards:
```
S=/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-local
ln -sfn ~/.claude/tools/likeness-fit/hq/wasm $S/wasm
ln -sfn ~/.claude/tools/likeness-fit/hq/nose $S/nose
ln -sfn ~/.claude/tools/likeness-fit/hq/paint $S/paint
```

The link `viewer-local/paint` points to `~/.claude/tools/likeness-fit/hq/paint`, the live paint page at `/paint/`. It was added on 2026-11-19 by the paint session.

The server sends `.wasm` files as `application/wasm`, so streaming compilation works.

At about 22:20 UTC on 2026-11-19 the server on port 8765 was found stopped, by no one in the nose session. The nose session started it again with the same command on the same folder: `cd viewer-local && nohup /usr/bin/python3 -m http.server 8765 --bind 127.0.0.1 &`. That Python is 3.9 from Xcode, where the first server was 3.14; both send `application/wasm`.

A page imports the shared loader `/wasm/load.js` (source `hq/wasm/load.js`) as a module:
```js
import { loadCrate, fetchTyped, fromBase64, timed } from "/wasm/load.js";
const m = await loadCrate("head_sliders", "/wasm", "11");   // the crate's exports; the last argument is added to the
                                                             // files' addresses, so bump it after a rebuild or the browser
                                                             // keeps the old module
const base = await fetchTyped("data/base.bin", Float32Array);
const b = new m.Builder(base, triangles, keep, slidersJson);
const [positions, ms] = timed(() => b.build(start, values));
```

## How arrays cross

- A Rust parameter `&[f32]` takes a `Float32Array`, `&[f64]` a `Float64Array`, `&[u32]` a `Uint32Array`, `&[u8]` a `Uint8Array`. Each call copies the array into the module's memory.
- A Rust `Vec<f32>` comes back as a new `Float32Array`.
- `&str` and `String` cross as JavaScript strings; JSON is the easy way to pass structures.
- An error returned as `Result<_, JsError>` becomes a JavaScript exception.
- A struct marked `#[wasm_bindgen]` becomes a JavaScript class whose memory lives in the module. Keep one and call it many times, rather than rebuilding it per call.

## What the paint crate will hit

- `head_paint` starts threads, in `src/closest.rs` and elsewhere. `std::thread::spawn` panics on `wasm32-unknown-unknown`. Shared-memory threads in the browser need the page to be cross-origin isolated (the headers `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp`), and the local server sends no such headers, so threads will not work as served. Put the threaded loops behind a function that runs on one thread for this target, for example with `#[cfg(target_arch = "wasm32")]`.
- `std::time::Instant::now()` panics on this target. `head_hair` calls it in `src/field.rs` and `src/lib.rs`. Keep timing out of library code, or put it behind the same `cfg`.
- `std::fs` compiles but fails at run time. Pass bytes or strings in from JavaScript instead (the nose crate keeps its file reading in a module compiled only for native builds).
- Large arrays should be sent once and kept inside a `#[wasm_bindgen]` struct.

What the paint session found on 2026-11-19 when it built `head_paint`:
- The paint itself starts no thread, reads no clock and reads no file. Only the tools in `src/bin` do, and they are not built for the browser.
- `closest.rs` asks `available_parallelism()` for its number of threads. On this target that call returns an error, so the code already falls back to one thread and never spawns. It needed no change, and the native build is unchanged.
- The crate builds with `build_wasm.sh head_paint` as it is. Its browser functions are in `src/web.rs` (`controls_json`, `values_json` and the class `Painter`).

## Sizes and timings

The figures below were measured on 2026-11-19 on this Mac in Chrome, for `head_sliders`.
- Module: 278 KB (`head_sliders_bg.wasm`), glue 12 KB.
- Loading and starting the module: 9 ms when the browser has it cached.
- Preparing the head mesh (`new Builder`, which reads the slider set and prepares the smoothing): 16 to 53 ms.
- Building a head (`build`: the stand-in plus 12 slider movements, then the smoothing of 4,750 vertices): 6 to 7 ms.
- Measuring its nose (`measure`): 4 to 6 ms.
- The same measure natively takes 1.1 to 1.3 ms. The machine had a load average of 40 to 90 from other sessions while these were taken.

### head_paint, the live paint page

The paint session measured these figures on 2026-11-19 in headless Chrome 157, in a Web Worker, on one thread, for Kreia (`p_kreiah`). The machine was busy with other sessions' work, with a load average of 30 to 70 on its 16 cores. The figures are the best of 3 runs, but they are probably 1.5 to 2 times slower than on an idle machine. These figures were measured inside the worker, so they leave out the transfer of the texture to the page.

- Module: 358 KB (`head_paint_bg.wasm`) since the split repaint of 2026-11-20, glue 16 KB. It was 339 KB before.
- Memory: one painter of the whole texture holds 664 MB at 2,048 pixels, measured on 2026-11-20 after a slider of every layer had moved. An earlier estimate of 500 MB was too low. Most of it is the head's surface read at every pixel (64 bytes per pixel), the pixel map (32 bytes per pixel) and the kept results.

| Repaint, in milliseconds | 256 px | 512 px | 1,024 px | 2,048 px |
|---|---|---|---|---|
| Prepare a head: the pixel map and the surface | 315 | 322 | 620 | 783 |
| The whole stack, with nothing kept | 218 | 712 | 4,335 | 13,623 |
| Repaint with nothing changed (the last step only) | 23 | 90 | 566 | 1,990 |
| Base tone slider moved (melanin) | 40 | 160 | 1,064 | 2,822 |
| Shading slider moved (strength) | 41 | 164 | 1,069 | 2,604 |
| Sheen slider moved (cheekbones) | 40 | 180 | 1,081 | 2,542 |
| Wrinkle slider moved (age) | 62 | 156 | 773 | 1,681 |
| Fine lines' depth moved | 23 | 97 | 606 | 1,408 |
| Wrinkles' random pattern moved | 171 | 729 | 2,773 | 7,184 |
| Pores' darkness moved | 23 | 144 | 493 | 1,407 |
| Pores per cm² moved | 29 | 182 | 650 | 1,770 |
| Grain size moved | 38 | 252 | 802 | 2,103 |
| Pores' random pattern moved | 50 | 312 | 990 | 2,848 |
| Base tone alone, nothing kept | 4 | 29 | 109 | 450 |
| Shading alone, nothing kept | 19 | 126 | 493 | 1,461 |
| Sheen alone, nothing kept | 10 | 69 | 282 | 676 |
| Wrinkles and folds alone, nothing kept | 166 | 929 | 3,190 | 6,947 |
| Pores and grain alone, nothing kept | 49 | 331 | 1,136 | 3,467 |

What these mean for the page:
- The painter (`head_paint/src/live.rs`) keeps each part of the stack per pixel and computes a part again only when a setting it depends on changes. A slider that only scales a part, such as a depth or a strength, costs no more than the last step.
- The last step costs about 0.5 µs per pixel in WebAssembly. It runs `shift_lightness` and the sRGB encoding on every pixel, which is about 15 calls of `powf` per pixel. It sets the floor: about 2 s at 2,048 pixels and 90 ms at 512 pixels on this busy machine.
- The page paints at 256 pixels while a slider moves, which stays under 100 ms for every slider except the two random patterns. At 512 pixels most sliders take 140 to 250 ms.
- With one worker for the full size, as the page first had it, most sliders took 1.4 to 2.8 s on release. The random patterns take 3 to 7 s, because the fine lines are the most costly part (378 hashed strokes per pixel).
- With one worker, the first full repaint after a head is picked took about 10 to 14 s.
- Shared-memory threads would need the cross-origin headers, which the server does not send. Since 2026-11-20 the page splits the full size over several workers instead, which needs no such headers (see the next section).

### The full size split over several workers

Since 2026-11-20 the page paints the full size with several workers. Each worker paints every n-th pixel of the pixel map and keeps the surface and the kept results of its own pixels only. Each worker finds its pixels with `live::rasterise`, which gives the pixels of `PixelMap::new` in the same order without holding the whole map. A test checks that both give the same pixels. The page's own thread puts the shares together with `Assembler` (`head_paint/src/live.rs`, and the class `Assembler` in `src/web.rs`). The assembler fills the background, puts each share's pixels in their texels, and grows the paint past the patches' edges as `paint` does.

The number of workers is the core count minus 2, with a cap of 8 (`SHARES` in `hq/paint/paint.js`). The drag painter and the page's own thread keep a core each. On this Mac, which has 16 cores, the cap gives 8 workers. Going from 4 to 8 workers made every repaint 1.4 to 1.9 times faster, at a cost of about 70 MB more memory. `?shares=4` in the page's address sets another number.

The paint session measured these figures on 2026-11-20 in headless Chrome, at 2,048 pixels, for Kreia, with `paintPage.bench(2048, "p_kreiah", workers)`. Each one runs on the page's thread from the request until the whole texture is ready, so it includes the transfer and the assembly. Each figure is the best of 3 runs. The load average was 42 to 60 on the 16 cores during the runs with 1 and 8 workers, and 29 to 42 during the run with 4 workers.

| Repaint at 2,048 pixels, in milliseconds | 1 worker | 4 workers | 8 workers |
|---|---|---|---|
| Prepare a head (every worker) | 503 | 649 | 367 |
| The whole stack, with nothing kept | 12,829 | 2,818 | 1,496 |
| Repaint with nothing changed | 1,496 | 373 | 300 |
| Base tone slider moved (melanin) | 2,595 | 674 | 461 |
| Shading slider moved (strength) | 4,815 | 673 | 482 |
| Sheen slider moved (cheekbones) | 2,579 | 676 | 479 |
| Wrinkle slider moved (age) | 3,208 | 498 | 390 |
| Fine lines' depth moved | 2,137 | 380 | 239 |
| Wrinkles' random pattern moved | 7,124 | 2,269 | 1,522 |
| Pores' darkness moved | 1,408 | 378 | 238 |
| Pores per cm² moved | 1,768 | 489 | 297 |
| Grain size moved | 2,099 | 604 | 386 |
| Pores' random pattern moved | 3,503 | 765 | 456 |
| Base tone alone, nothing kept | 416 | 82 | 53 |
| Shading alone, nothing kept | 1,515 | 325 | 190 |
| Sheen alone, nothing kept | 765 | 180 | 117 |
| Wrinkles and folds alone, nothing kept | 6,925 | 2,041 | 1,390 |
| Pores and grain alone, nothing kept | 4,561 | 712 | 479 |
| The modules' memory after the run | 664 MB | 694 MB | 761 MB |

- With 8 workers, every slider repaints the full size in under 0.5 s, except the wrinkles' random pattern, which takes 1.5 s.
- The first full repaint after a head is picked takes about 1.9 s with 8 workers (preparing 0.4 s, then the whole stack 1.5 s). In the page, it took 2.6 s from loading the head to showing the texture.
- Memory as a person uses the page, after a slider of every layer had moved: with 8 workers, the full size workers hold 763 MB (95 MB each), the drag worker 28 MB and the page's own module 73 MB, which is 864 MB in all. With 1 worker the same figures are 664 MB, 28 MB and 1 MB, which is 694 MB in all. The split costs 170 MB more. About 100 MB of it is what each worker holds besides its share, such as its copy of the layout, the head's shape maps and the pixel slots it uses while finding its pixels. The other 73 MB is the assembler: the texel of every pixel, the growth past the patches' edges, and the texture it builds.

### Do the browser and native builds paint the same bytes?

On 2026-11-19 the module painted the same bytes as the native build for 6 heads: Kreia, Female B4, Canderous from the mod, Bastila, Elder W male and Male B1. It was checked with 8 textures per head: every layer on, each layer alone at 2,048 pixels, and every layer on at 512 and 1,024 pixels. All 48 textures were equal byte for byte. The scripts are `check/compare.mjs` (Node) and `paint_live paint` (native) in the paint session's scratchpad.

On 2026-11-20 the same 48 textures were painted in headless Chrome split over 8 workers and put together by the assembler. All 48 had the same SHA-256 as the native textures. A split over 5 workers, whose shares do not divide the pixels evenly, gave the same result. The script is `check/splitcheck.mjs`.

The maths functions themselves differ. The native build calls Apple's maths library, and the module calls Rust's own port of musl's. On 4 million inputs each, `powf`, `cbrt`, `exp`, `atan2`, `asin`, `sin` and `cos` gave a different last bit on 4% to 18% of the inputs. `powi` matched on every input. The textures stay equal because a difference of one last bit almost never moves a value across a rounding step. It can, though, so equal bytes are likely but not guaranteed. These yes-or-no choices of the paint depend on such a function:
- `surface.rs`: the horizon sector of a neighbour comes from `atan2`;
- `sheen.rs`: a sheen weight under 1% is cut to zero, after `exp`;
- `pores.rs`: the grid's cell size comes from `cbrt`, and `floor` then picks each point's cell; whether a pore is kept compares a hash with a T-zone weight made with `exp`;
- `pigment.rs`: the table cell comes from `floor` of a `cbrt`;
- `paint.rs`: each byte is the rounding of a `powf` result.

## head_sliders, round 2 (2026-11-20)

- The nose session rebuilt `head_sliders` for the browser at 00:11 on 2026-11-20 with the same `build_wasm.sh`, after its measure grew to 14 proportions. The module is 292 KB and the glue 12 KB. The nose page loads it with the version `13` (`VER` in `hq/nose/nose.js`).
- `head_sliders` gained the `image` crate for its clay pictures (`src/render.rs`). It is a dependency for native builds only (`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`), so the browser build does not compile it. The modules `heads.rs` and `render.rs` are native only too.
- Neither server was restarted.

## head_paint_gpu, the paint on the GPU

This section was written on 2026-11-20 by the GPU paint coordinator. The crate `head_paint_gpu` (next to `head_paint` in the worktree) paints the same texture as `head_paint::paint::paint`, with wgpu 29.0.4 (the version Bevy uses) and WGSL shaders. The same crate and the same shader files run natively and in the browser through WebGPU. The live paint page uses it by default since 2026-11-20, and `?painter=cpu` in the address brings back the processor's workers.

### Building a wgpu crate for the browser

- `build_wasm.sh head_paint_gpu` builds it with no change to the script. The module is 745 KB and the glue 41 KB.
- wgpu needs no `--cfg web_sys_unstable_apis` flag for WebGPU. The crate asks for wgpu with its default features minus `gles`: `std`, `parking_lot`, `wgsl`, `metal`, `vulkan`, `dx12` and `webgpu`. The `webgpu` feature is what the browser build uses.
- `head_paint`'s own `#[wasm_bindgen]` functions are exported by every module that depends on it. A crate that depends on `head_paint` must not export a function of the same name, or `wasm-bindgen` stops with "the name ... is exported by multiple crates". `head_paint_gpu` therefore uses `head_paint`'s `memory_bytes` and `controls_json` instead of its own.
- A `#[wasm_bindgen]` method cannot hold `&mut self` across an `await`. The browser class `GpuPainter` keeps the painter in an `Rc<RefCell<...>>` and each method that waits (`prepare`, `done`, `read`) clones what it needs before it waits. `GpuPainter.create(...)` is a static async function.
- Reading a buffer back is the same code natively and in the browser: `map_async` sends to a `futures_channel::oneshot`, natively `device.poll` waits, and in the browser the promise resolves when the browser calls back.
- Headless Chrome 157 on this Mac gives WebGPU an adapter (Apple, Metal 3) with `--headless=new --enable-unsafe-webgpu`, on a page served from `127.0.0.1`. `navigator.gpu` is missing on `about:blank`, because WebGPU needs a secure context. The driver is `puppeteer-core` in `~/.claude/tools/likeness-fit/paint/gpu/` (`browser.mjs`).
- `std::time::Instant` still panics in the browser. One layer's first version timed itself with it, and it was removed before it reached the page.

### Does the browser paint the same picture?

On 2026-11-20 the browser's textures were compared with the processor's textures (`paint/gpu/browser_check.mjs`, references written by `gpu_check --raw`). The heads were Kreia, Female B4, Canderous from the mod, Bastila, Elder W male and Male B1, at 2,048 pixels, each layer alone and all together, which is 36 textures. Every texel was counted, the background and the growth past the patches included. All 36 passed: at least 99.9997% of the 4,194,304 texels of each are within 1 step of 255. The only texels further off are 14 on Canderous and 1 on Male B1, at 2 steps, the same texels as in the native GPU run. The native GPU run and the browser give the same counts to within a few dozen texels off by 1.

### Timings and memory

Measured on 2026-11-20 on this Mac (Apple GPU), with a load average of 45 to 56 from other sessions. Each figure is the best of 5 runs. "GPU" is from the request until the GPU has finished. "Read back" also copies the texture's bytes to the page, which the page then gives to three.js as a `DataTexture`.

| Repaint at 2,048 pixels, in milliseconds | Chrome, Kreia | Chrome, Male B1 | Native, median of 6 heads |
|---|---|---|---|
| Prepare a head and the size (once per head) | 523 | 612 | 214 to 1,421 |
| The whole stack, GPU | 46 | 54 | 56 to 70, and 10 on Bastila |
| The whole stack, with the read back | 57 | 59 | |
| A slider moved, with the read back | 63 to 98 | 66 to 79 | |
| Base tone alone, GPU | 0.8 | 0.8 | 1.4 to 2.9 |
| Shading alone, GPU | 0.6 | 1.3 | 1.4 to 5.8 |
| Sheen alone, GPU | 0.9 | 1.1 | 1.3 to 6.1 |
| Wrinkles, fine lines and forehead lines alone, GPU | 54 | 45 | 48 to 58, and 5 on Bastila |
| Pores, grain and mottling alone, GPU | 9.3 | 8.8 | 9.9 to 14.2 |

- Today's painter took 1,496 ms for the whole stack with 8 workers, and 300 to 1,522 ms per slider. The GPU painter takes 57 to 98 ms with the read back, so it is 5 to 25 times faster, and it paints the full 2,048 pixels on every move of a slider.
- The fine lines cost about 45 ms of the 46 to 54 ms. Bastila's recipe has no fine lines, and her whole stack takes 10 ms natively. They are the 378 hashed strokes per pixel, ported as they are, and they are to be redesigned.
- In the live page, a repaint shown on screen took 50 to 71 ms on 5 heads.
- The browser module's memory was 112 MB after the benchmark, which made a second painter beside the page's own. Today's painter held 864 MB with 8 workers.
- The GPU holds about 50 MB per texture size: the painted texture (16 MB), the finished texels (16 MB) and the finish pass's table (16 MB), plus the wrinkle layer's lines (5 to 10 MB). Each read back makes a 16 MB buffer for the time of the read.

### The yes-or-no choices of the list above, on the GPU

- `surface.rs` (the horizon sector from `atan2`) and `sheen.rs` (the weight under 1% cut to zero) run per vertex on the processor, in f64, as before.
- `pigment.rs` (the table cell from a `cbrt`) runs on the processor, once per recipe, as before.
- `pores.rs`: the cell size from `cbrt` is computed on the processor and sent to the shader. The `floor` that picks a point's cell and the keep test (a hash against a T-zone weight made with `exp`) run in the shader in f32. The hash is exact. The pore agent estimated that a pore flips about once in 15,000 pores. No flip was seen in the checks.
- `paint.rs`: each byte is the rounding of a `pow` result in f32. This is where most texels off by 1 come from.

## head_eyes, the lid piece (2026-11-20)

- The lid piece session builds `head_eyes` with the same script: `build_wasm.sh head_eyes`. The crate gained `wasm-bindgen = "0.2.129"` for the browser target only, and `serde_json` for native builds only (its tool `lid_piece` reads `lid_edges.json`). The library itself has no dependency.
- The browser API is the class `LidMaker` in `head_eyes/src/web.rs`: `new LidMaker(triangles, basePositions)`, then `add_eye(...)` once per eye from `lid_edges.json`, then `build(positions, open, lashLength, lashThickness, lashTilt, matchClip, pivotBack)`, then getters per eye (`positions`, `triangles`, `colour_from`, `colour_weights`, `shade`, `column`, `lash_positions`, `lash_triangles`, `hidden_faces`, `info`).
- The module is 303 KB and the glue 13 KB. Making both pieces of a head takes about 1.0 ms in headless Chrome, measured with `lidPage.bench(name)` on a Mac with a load average of 150. Matching KOTOR's 34-degree clip by moving the pivot takes about 14 ms, because it searches the pivot's place 22 times.
- The page is `hq/lids/` (index.html, lids.js), served through the link `viewer-local/lids`. Its data folder `likeness-fit-data/lids/` holds the 58 heads' positions (`heads.bin`, written by `lid_piece export`) and links to `lid_edges.json`, the nose page's `layout.json` and viewer 2's data (`vd`). `VER` in `lids.js` is the module's version; bump it after a rebuild.
- A trap: the page's sidebar is `<div id="side">`, so no control may use the id `side`.

### The new pore layer (2026-11-20, pores and grain round)

- The GPU pore layer is a new design since about 02:00 UTC on 2026-11-20: a furrow net on two levels with pores where its cells meet (`head_paint_gpu/shaders/pores.wgsl`). The processor's painter keeps the old pores and Perlin grain, so the two no longer match.
- `gpu_check` leaves the pore layer out. A switch set with the pore digit at 1 is compared with the pores off on both sides, and `check.json` marks it with `pores_left_out`. On 6 heads, 36 textures at 2,048 pixels all passed for the other layers.
- `paint/gpu/browser_check.mjs` compares the browser with `gpu_check --raw` references. Its sets with the pore digit at 1 will now differ, because the references leave the pores out. It was not changed.
- The pore shader's helpers are checked by `head_paint_gpu/tests/pore_helpers.rs`: the net's grid points and hashes against a Rust twin, the region map against `pores::region_factors`, and the corner area against a count.
- Three controls only the GPU paints: `pore_gradient`, `pore_stretch` and `grain_direction` (`gpu_only` in the control list). The page names them when it runs with `?painter=cpu`.
- Every head's pore values in the page's `recipes.json` are now `controls::DEFAULT_PORES`, written by `paint_live data`. The old values are kept in `~/.claude/tools/likeness-fit-data/paint/pores/recipes-before-2026-11-20.json`.
- The page loads the GPU module with the version `g8`.
- Repaint of the pore layer alone at 2,048 pixels, native, best of 10, measured with `pore_pictures --time` while the machine had a load average of 58 to 140: 13 to 16 ms new, against 8 to 13 ms for the old shader in the same runs. The whole stack took 104 to 121 ms new and 100 to 128 ms old in those runs. The earlier 9 ms figure was measured on a less loaded machine.
- Update at about 02:10 UTC, after a round judged at face distance: the mottling's default fell from 8 to 2.5 L* and its size from 16 to 10 mm, because at 8 L* it drew streaks and blotches over the whole face. The grain's default fell from 6 to 4 L*. The furrows and the pores now fade out between 18 and 32 mm above the eyes, because the layout's pixels grow in steps there. The page loads the GPU module with the version `g9`.

### The eye paint layer (2026-11-20)

The eye paint agent of the paint session added a sixth layer, the eye paint (`head_paint_gpu/src/layers/eyes.rs`, `shaders/eyes.wgsl`, settings in `head_paint/src/eyes.rs`). It takes bindings 12 and 13. Only the GPU paints it: the processor's painter ignores `Recipe.eyes`, `gpu_check` turns the layer off on both sides, and every eye control is marked `gpu_only`. The paint pass always calls `eyes_apply` last, and the layer's own uniform says whether it is on.

- After the change, the browser modules are 804 KB (`head_paint_gpu_bg.wasm`) and 406 KB (`head_paint_bg.wasm`). The page loads the GPU module with the version `g11`.
- Measured natively on this Mac with a load average of about 40, at 2,048 pixels, from the request until the GPU has finished, as the median of 9 repaints: the eye layer alone takes 4.5 to 5.2 ms on Kreia, Bastila and pmha01.
- In headless Chrome the page showed the eye layer on Bastila, Carth and Kreia, with the whole texture repainted in 17 to 230 ms with the read back, on a loaded machine.

## head_sliders as a kit (2026-11-20)

- On 2026-11-20 the slider session reshaped `head_sliders` into a kit: each feature (the nose, the jaw and chin) is a small module, and
  the browser functions grew a `measure_feature(feature, positions, eyes)` method on `Builder` and a free function `merge_sets(sets)`,
  which merges several features' slider sets into one for a single `Builder`.
- The module was rebuilt with the same `build_wasm.sh` at 01:16. It is 410 KB (`head_sliders_bg.wasm`) and the glue 14 KB. The page
  loads it with the version `16` (`VER` in `hq/nose/nose.js`).
- The build script, `load.js` and the tool versions did not change. Neither server was restarted.

### The lip layer (2026-11-20)

The lip agent added the lip layer to `head_paint_gpu` at about 03:00 CET on 2026-11-20. It is the seventh layer, at bindings 14 and 15 (`layers/lips.rs`, `shaders/lips.wgsl`), and only the GPU painter paints it. Its 21 sliders are in `head_paint::lips::SLIDERS` and reach the page as the group "Lips". The page loads the GPU module with the version `g12`. After the change the module was 866 KB and the processor module 435 KB. In headless Chrome on Kreia and Bastila, the lip layer alone took 1.9 ms on the GPU at 2,048 pixels, and 7.6 to 8.1 ms with the read back. `gpu_check` keeps the lips off on both sides, because the processor painter does not paint them.

- On 2026-11-20 at 04:06 the upper face session rebuilt `head_sliders` with the brow and the cheeks: the module is 514 KB (525,761 bytes), the glue 13.7 KB, and the nose page loads them with the version `21`.

The lip polish round of 2026-11-20 (about 04:40 CET) changed only `head_paint/src/lips.rs`, `head_paint_gpu/shaders/lips.wgsl` and `head_paint_gpu/src/bin/lip_fit.rs`, and no binding. Both modules were rebuilt with `build_wasm.sh`. The GPU module is 900,774 bytes, and the page loads it with the version `g16`. In headless Chrome with a load average of about 11, the lip layer alone took 1.4 ms on Kreia and 1.8 ms on Bastila on the GPU at 2,048 pixels, and 5.6 to 5.9 ms with the read back. A lip slider moved on the page repainted the whole texture in 17 to 136 ms with the read back.

- The eye sliders' round 3 agent rebuilt `head_eyes` and `head_sliders` with the same `build_wasm.sh` at 13:46 on 2026-11-20. `head_eyes` now reads each eyeball's radius from its 8 eyeball-centre vertices (12.3 mm × their RMS distance from their mean / 6.907 mm), and `LidMaker.info(e)` reports it as `eyeball_radius_mm`. `head_sliders` has 18 eye measures. Viewer 2's `sliders.js` loads them as `?b=23` and `head_eyes` version `14`.

### Checking the browser's texture against the native one (2026-11-20, 14:55)

Run this after every rebuild of the browser module of `head_paint_gpu`:

```
~/.claude/tools/likeness-fit/paint/gpu/native_browser_check.sh <out dir> [switches] [head]...
```

- It builds the native tool `browser_ref` (in `head_paint_gpu/src/bin/`) from the worktree and paints the heads natively. Then it paints the same heads in headless Chrome with the module that the page at port 8765 serves, and compares the two colour textures texel by texel.
- The default switches are `111111111`, which turns on every layer and the light from maps. The default heads are `pmba01`, `p_kreiah`, `p_bastilah`, `p_carthh`, `pfhb04` and `mod_hp_pfhb02`.
- It prints every texel that differs by more than 2 steps of 255. Each line gives the texel's place, both colours, and the layers that paint there, which are the layers whose switch changes that texel natively.
- It writes `report.json` to the out dir. For each head that has such texels, it also writes `<head>_diff.png` and `<head>_crops.png`. It exits with 1 when any texel differs by more than 2 steps.
- The environment variables `PAGE`, `DATA` and `CRATES` point it at a private copy of the page, the data or the crates.

The module was rebuilt at 14:52, and the page loads it with the version `g30`. With that module, all six default heads pass. Before the fix, Chrome painted 3,430 texels black or green over 20 of the 57 heads. The cause and the fix are noted in `pore_glint` in `shaders/pores.wgsl`.

## The processor painter is gone from the browser (2026-11-20, about 18:40)

The CPU painter cleanup (session bde0c387) removed the processor's live painter from `head_paint`. The classes `Painter` and `Assembler` are no longer exported, so the module `hq/wasm/head_paint/` and the worker `hq/paint/worker.js` were deleted, and the live paint page no longer accepts `?painter=cpu` or `?shares=`. The sections above about several workers, `SHARES`, `paintPage.bench` and `hashOf` describe code that does not exist any more.
`head_paint` keeps `controls_json`, `values_json` and `memory_bytes` for the browser, and `head_paint_gpu` exports them. `paint/gpu/browser_check.mjs` and its `gpu_check` tool were deleted. The check of the browser against the native GPU is `paint/gpu/native_browser_check.sh`.
The module `head_paint_gpu` was rebuilt as `g45`. It is 1,363,283 bytes (it was 1,432,925) and its glue is 69,032 bytes (it was 77,365). On `p_carthh` and `pmba01` no texel differs from the native paint by more than 1 step.

## The rest of the processor painter is gone (2026-11-20, about 18:50)

The second CPU cleanup (session bde0c387) removed the processor painter from `head_paint` and the old live paint page.

- In `head_paint`, the functions `paint`, `paint_with_folds` and `paint_with_detail`, the types `DetailPaint` and `Texture`, `BaseTone::srgb8`, the tests of the processor painter, and the three fitters `fold_fit`, `fold_hand` and `line_hand` (with their shared file `shared/fold_paint.rs`) are deleted. In `head_paint_gpu`, the example `pore_probe` is deleted.
- `HeadSurface` stays, but only as the read of the shading and sheen maps at each pixel. The two fitters `fit_shading` and `fit_skin_detail` still read it, and they also use `Recipe::colour_at`, `Recipe::base_colour`, `Pores::lightness_shift` and `WrinkleField::lightness_shift`. `fold_field` uses `FoldField::lightness_shift`. Those items stay as long as the tools stay.
- The old page `hq/paint/index.html` and `hq/paint/paint.js` are deleted, and `index.html` is now a short page that redirects to viewer 2. Every subfolder of `hq/paint/` stays, because viewer 2 loads `gpu_version.js`, `live/` and `gpu/`, and the other folders hold picture sheets.
- Nine page-only check scripts in `paint/gpu/` (`page_check`, `pores_page_check`, `lips_page_check`, `lips_gloss_check`, `lines_shot`, `eyes_shot`, `bench`, `notice`, `craters_check`) and `paint/eyeball/ball_shot.mjs` are deleted, because they drove `window.paintPage` on `/paint/`. `paint/gpu/probe.mjs` now opens `paint/gpu/check.html`. Copies of all of them are in `~/.claude/tools/likeness-fit-data/backups/old-paint-page-2026-11-20/`, and a fresh archive of both crates is `head-paint-crates-before-cleanup2-2026-11-20.tgz` in the same `backups` folder.
- The module `head_paint_gpu` was rebuilt as `g46` (1,363,155 bytes). Only comments in the shaders changed. `native_browser_check.sh` on `p_carthh` and `pmba01` finds no texel off by more than 1 step.

## head_rig, the live rig of a slider head (2026-11-20)

- `build_wasm.sh head_rig` builds the module with no change to the script (290 KB, glue 15 KB). The crate has `crate-type = ["rlib", "cdylib"]`. Its `calibrate` module, `faer`, `head_paint`, `nalgebra` and `image` are compiled only for native builds, and its browser functions are in `src/web.rs` (`RigMaker` and `LiveRig`).
- Viewer 2 loads it with `loadCrate("head_rig", "/wasm", "1")` and the three files `data/calibration/weights_model.json`, `regressor.json` and `lips.json`, which `calibrate.sh` copies there. Bump the version string in `liveRigKit` of `viewer2.js` after a rebuild.
- One rig of one head takes about 1 ms in headless Chrome.

## head_eye_curve, the eye curve (2026-11-21)

- The live head builds its eye openings on the curve inside `head_sliders`, which depends on `head_eye_curve`. The live modules are in `hq/wasm/curve/` (`head_sliders`, `head_eyes` and `head_eye_curve`). Viewer 2 loads them with the version `EYE_CURVE_VER` in `sliders.js`. Raise it after a rebuild.
- `hq/wasm/head_eye_curve/data/` keeps only `group.bin`, `first.bin` and `hidden.bin`, the merged groups, the first vertex of each group and the hidden flags of the head mesh. Viewer 2 passes them to `Builder.init_curve`.
- The preview module of `head_eye_curve` alone (`CurveMaker`, the tab "Eye curve (prototype)" and `?eyecurve=1`) was removed on 2026-11-22. A copy is in `~/.claude/tools/likeness-fit-data/backups/eyes-legacy-2026-11-22/wasm/head_eye_curve/`.
- A trap: Rust's `hypot` is slow in the browser (it is software). A distance loop with 13 million calls took 155 ms per eye in Chrome against 11 ms natively. A plain square root brought it to under 20 ms.
- Timings in headless Chrome: a whole head with the preview on takes about 460 ms. A handle move takes about 33 ms, because the skin's height field depends only on the live head and is kept between builds.
