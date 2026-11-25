# GPUI and component previews for a kjk UI storybook

Research date: 2026-11-25.
Zed source read: a shallow clone of https://github.com/zed-industries/zed at commit 8aeb64f7444d25e91e3369bf7f4985d03bf336de (2026-11-25).
Paths below are relative to that repository at that commit.
No Zed code was built or run.

## Summary and recommendation

**Do not use GPUI in kjk. Build the storybook as a generated static HTML gallery of headless PNGs with JSON sidecars, made by the tools kjk already has (`kjk gui render`, `kjk gui screen --step`, `kjk gui sweep`, `kjk gui sheet`). Borrow GPUI's and Zed's ideas, not their code.**

The reasons are these.

1. GPUI is a toolkit for building its own widget trees, and kjk already has its own GUI runtime that reads KOTOR's `.gui` files. A GPUI storybook would mean writing a second renderer for kjk's `DrawList`, its bitmap fonts and its textures. That renderer would then have to match `ui::raster` and the Bevy `ui.wgsl` material pixel for pixel, or the storybook would show something the game does not draw.
2. GPUI is a dependency on a moving target. Its README says it is "still pre-1.0" and that "there will often be breaking changes between versions". The gpui.rs site says GPUI "is tied to Zed". Zed pins nightly-adjacent wgpu and taffy revisions by git hash. kjk already depends on Bevy 0.19.1, and adding a second GPU stack to the workspace has a real cost for no gain.
3. Zed's own component preview is a tool for Zed's own widgets. It renders live GPUI elements inside Zed's workspace, with a sidebar. It is not an image gallery, and it cannot be read by an agent without a screen. Zed's separate screenshot tooling (the visual test runner) is macOS only, and it works only for tests written in Rust against live Zed windows.
4. The useful part of Zed's tooling is its model, which kjk can copy in a small amount of code. A component registers itself with a name, a scope, a status and a description. It supplies a `preview` function that returns named examples grouped under titles. A tool then renders every example. kjk can do the same with a manifest of screens and states.

The rest of this document gives the evidence, the comparison with alternatives, and the principles to borrow.

## 1. What GPUI is

### What the project says about itself

- The gpui.rs home page calls GPUI "A fast, hybrid immediate and retained mode, GPU-accelerated UI framework for Rust." It also says "For the near future, GPUI is tied to Zed, so contributions will need to be made there and kept in sync with it." (https://www.gpui.rs/)
- The crate README says the same thing in other words: "GPUI is still in active development as we work on the Zed code editor, and is still pre-1.0. There will often be breaking changes between versions." (`crates/gpui/README.md`)
- The crate is `gpui` version 0.2.2, licensed Apache-2.0, with `publish = true` (`crates/gpui/Cargo.toml`). On crates.io the newest release is 0.2.2, created on 2025-10-22, with about 569,000 total downloads (https://crates.io/api/v1/crates/gpui). The workspace in the repository is much newer than the last release, so a user who wants the current code, including the Windows, web and Linux crates described below, depends on the git repository and not on crates.io.
- Not every crate around it is Apache-2.0. The `component` and `component_preview` crates carry a `LICENSE-GPL` file (`crates/component/LICENSE-GPL`, `crates/component_preview/LICENSE-GPL`). Their code could not be copied into kjk without taking on that licence. Their ideas can be used freely.

### How it renders

The Zed team's own post "Leveraging Rust and the GPU to render user interfaces at 120 FPS" describes the design (https://zed.dev/blog/videogame). Its main points are these.

- A UI is reduced to a few kinds of primitive. The post says "most 2D graphical interfaces break down into a few basic elements: rectangles, shadows, text, icons, and images". Each kind has its own shader, and the CPU describes each one as data.
- Rectangles are drawn with a signed distance function in the fragment shader, which gives rounded corners and borders. Shadows use an approximation of a Gaussian blur. Text is shaped by the operating system, rasterised into alpha masks in a GPU texture atlas, and drawn as tinted instanced sprites.
- Painting pushes primitives into a platform-neutral `Scene`, organised in layers. Within a layer the renderer draws primitives in a fixed order: shadows, rectangles, glyphs, then icons and images.

The source confirms this model today.

- `crates/gpui/src/scene.rs` defines `enum Primitive` with the variants `Shadow`, `Quad`, `Path`, `Underline`, `MonochromeSprite`, `SubpixelSprite`, `PolychromeSprite`, `Surface` and `NativeView`.
- A `Quad` holds a draw order, bounds, a content mask (a clip), a background, a border colour, corner radii and border widths (`crates/gpui/src/scene.rs`, `struct Quad`).
- Elements paint through `Window::paint_quad`, `paint_path`, `paint_svg`, `paint_image` and `paint_surface` (`crates/gpui/src/window.rs`). A `canvas(prepaint, paint)` element exposes the same low level calls inside a view (`crates/gpui/src/elements/canvas.rs`).

The layout engine is Taffy, a Rust implementation of CSS flexbox and grid (`crates/gpui/Cargo.toml` depends on `taffy`; `crates/gpui/src/taffy.rs`). Styling uses a Tailwind-like builder (`div().flex().gap_3().p_8().rounded_xl()`), which can be seen throughout `crates/component/src/component_layout.rs`. The 2020s blog post describes a Flutter-inspired constraint layout, but the repository now uses Taffy, so the post is out of date on this point.

### Platform backends

The platform crates in the repository are `gpui_macos` and `gpui_apple` (Metal), `gpui_windows` (DirectX, with HLSL shaders in `crates/gpui_windows/src/shaders.hlsl`), `gpui_linux` (Wayland and X11), `gpui_wgpu` (a wgpu renderer with WGSL shaders and the `cosmic-text` text system), `gpui_web` (WebAssembly, with a browser gallery), and `gpui_ios`. `gpui_platform::application()` picks the backend for the host system, and `gpui_platform::headless()` creates an application without a window (`crates/gpui_platform/src/gpui_platform.rs`).

The README states the macOS detail that matters for kjk: "Rendering uses Metal and is always available, but glyph rasterization needs `font-kit`. Without it, GPUI falls back to a placeholder text system that lays text out but renders no glyphs." (`crates/gpui/README.md`)

### Use outside Zed

- The best known user is `gpui-component`, a component library from Longbridge. Its README describes 75 or more documented components, an Apache-2.0 licence, version 0.7 on crates.io, 21.4 thousand GitHub stars, macOS, Windows and Linux support, WebAssembly support, and a `story` crate that is a gallery app showing every component. It says Longbridge Pro was built with it. (https://github.com/longbridge/gpui-component, as read on 2026-11-25)
- The gpui.rs page names no other users. It says it comes "From the creators of Zed & Delta" and does not say how Delta uses it.

## 2. Zed's component preview and visual testing

### The storybook crate is gone

Older Zed versions had a `storybook` crate. In the checkout at the commit above, no crate with that name exists and no `Cargo.toml` mentions "storybook" (checked with `ls crates` and a search of every `Cargo.toml`). Its role was taken by the `component` and `component_preview` crates. I could not read the history of the removal, because the clone is shallow, so I do not claim when or why it was removed.

### How a component declares its previews

The `component` crate defines the `Component` trait (`crates/component/src/component.rs`). A component implements it, then derives `RegisterComponent` (a macro in `crates/ui_macros/src/ui_macros.rs`). The trait has these members.

- `scope()` returns a `ComponentScope`, one of: Agent, Collaboration, Data Display, Editor, Images and Icons, Forms and Input, Layout and Structure, Loading and Progress, Navigation, Unsorted, Notification, Overlays and Layering, Onboarding, Status, Typography, Utilities, Version Control.
- `status()` returns a `ComponentStatus`: `WorkInProgress` ("still being designed or refined, they shouldn't be used in the app yet"), `EngineeringReady` ("design complete or partially implemented, and ready for an engineer to complete"), `Live` (the default), or `Deprecated`.
- `name()`, `sort_name()` (so related components sort together, for example Button, IconButton, ToggleButton), `id()`, and `description()`. The doc comment of the type can serve as the description through the `Documented` derive.
- `preview(window, cx)` returns any element. The documented helpers are `single_example` and `example_group_with_title`.

Registration is automatic. The macro uses the `inventory` crate, which collects every registration at link time, and `component::init()` iterates them into a global `ComponentRegistry` (`crates/component/src/component.rs`).

The real `Button` shows the style. Its `preview` returns groups such as "Button Styles" (Default, Filled, Subtle, Tinted, Transparent) and "Tint Styles" (Accent, Error, Warning, Success), each example a named `Button` built with one setting changed (`crates/ui/src/components/button/button.rs`, function `preview`). The `crates/ui/src/components` folder holds about 40 such components, such as avatar, banner, button, callout, modal, tab, toggle and tooltip.

An example is a `ComponentExample` with a variant name, an optional description and an optional width. It is drawn as a title, a muted description and then the element inside a bordered box with a diagonal-stripe background pattern (`crates/component/src/component_layout.rs`). The stripes make transparent and translucent parts easy to see.

### How the previews are browsed

`ComponentPreview` (`crates/component_preview/src/component_preview.rs`, 991 lines) is a workspace item opened by the action `workspace: open component preview`. It has a sidebar grouped by scope, a page for one component, and an "all components" view. It stores the active page in a database so it reopens where the user left it. There is also a standalone example, run with `cargo run -p component_preview --example component_preview`, but it still builds a full Zed workspace, with a project, a language registry, a client and settings (`crates/component_preview/examples/component_preview.rs`). It is a live view for a person at a screen. It has no export, no JSON listing and no image output.

### Screenshot and visual testing tooling

- GPUI can render a window to an image without showing it. `Window::render_to_image` renders the current frame's scene to a texture and returns an `RgbaImage`. It exists only with the `test-support` feature (`crates/gpui/src/window.rs`). Implementations exist for macOS (`crates/gpui_macos/src/window.rs`) and Windows (`crates/gpui_windows/src/window.rs`, `directx_renderer.rs`), and the test window has one too (`crates/gpui/src/platform/test/window.rs`).
- `VisualTestAppContext::capture_screenshot` calls it (`crates/gpui/src/app/visual_test_context.rs`). It pairs real rendering with a deterministic task scheduler and a controllable clock (`advance_clock`), so tooltips and animations can be tested at fixed times (`crates/gpui/src/platform/visual_test.rs`).
- Zed's `zed-visual-test-runner` binary captures screenshots of real Zed windows and compares them with baseline images. `UPDATE_BASELINE=1` rewrites the baselines, and `VISUAL_TEST_OUTPUT_DIR` sets where output goes. Its header says "This tool is macOS-only", and on other systems it prints an error and exits (`crates/zed/src/visual_test_runner.rs`). The test module explains that the tests are ignored by default and must run on the main thread (`crates/zed/src/zed/visual_tests.rs`).
- GPUI also has an inspector (`crates/gpui/src/inspector.rs`, `crates/inspector_ui`) and `crates/gpui/src/_accessibility.rs`, which I did not read in detail.
- `gpui_web` ships a browser gallery of examples, built with Trunk (`crates/gpui/examples/README.md`).

## 3. Could GPUI host a kjk storybook?

### What it would take

A GPUI app would have to draw a `DrawList`. GPUI offers the low level calls needed, so this is possible in principle.

- A kjk quad with a texture maps to `paint_image` or `paint_quad`. The GPUI scene has no equivalent of kjk's blend in gamma space (documented in `docs/UI.md`: "Blended as KOTOR's, in gamma space ... `screens + (1 - alpha) * world`"), nor of the Catmull-Rom enlargement and the one-pixel-wide shape edges (`raster::Pixels::shape`). GPUI's shaders would need changes, which means forking Zed.
- kjk's text is KOTOR's bitmap fonts (TXI fonts), drawn as textured quads. GPUI's text path is built around operating system shaped, vector fonts rasterised into an atlas. The bitmap fonts would be drawn as images, one glyph at a time, through `paint_image`, which bypasses most of GPUI's text system.
- The layout side of GPUI (Taffy, the Tailwind-style builder) would go unused, since kjk's layout comes from the `.gui` file.

So GPUI would act only as a window, an event loop and a thin quad drawer, while paying for its whole dependency tree. A second drawing path also risks showing something the game does not draw. The storybook's value is that it shows the real output.

### Comparison

| Option | Effort | macOS and Windows | Agent friendliness | Fidelity to the game |
|---|---|---|---|---|
| GPUI app | High. A new renderer for `DrawList`, fonts and textures, plus a pinned Zed git dependency and a second Rust toolchain (Zed pins Rust 1.100.0 in `rust-toolchain.toml`, kjk pins 1.98.1). | Yes, on both (Metal and DirectX). Headless screenshots only on macOS and Windows with `test-support`. | Poor. A live window is for a person. Agents would need the same PNG export that kjk already has. | Risky. A second renderer can drift from `raster` and `ui.wgsl`. |
| Bevy app | Medium. kjk already draws the UI on the GPU with Bevy (`game::ui_view`, `ui_material`, `ui.wgsl`). A storybook window could reuse that path. | Yes, on both. | Poor to medium. A window needs a person or a screenshot step, and the repository's memory notes say game windows must run headless. | Best for a live view, because it is the game's own path. |
| egui app | Medium. egui can show textures and lists, but it is a new dependency and a third renderer for the images. | Yes, on both. | Poor. Same limits as any live window. | Medium. It would show PNGs made by `raster`, so fidelity is the PNGs'. |
| Static HTML gallery of PNGs plus JSON sidecars | Low. The PNG tools exist. The new work is a manifest, a sidecar writer and a page generator. | Anywhere a browser runs, and generation uses the existing `kjk gui` commands on both systems. | Best. An agent reads files: a JSON index and a sidecar per image, and can diff them. A person opens one page. | Exact. The images come from `ui::raster`. |

For the third column of the Bevy row, see the project memory note on game windows (`game-windows-aerospace.md`), which says kjk must run headless and never take the user's workspace.

One caveat applies to the HTML option. `ui::raster` is a CPU drawer, and `docs/UI.md` says "`raster` samples and places the same way, so headless pictures match" the GPU path. The storybook inherits that claim, so any gap between `raster` and the GPU is a gap in the storybook. The existing probes (`kjk gui probes`) are the place to guard it.

## 4. Principles to borrow

These come from GPUI and from Zed's component tooling, and all of them can be done without GPUI.

1. **A component registers itself with metadata, not a central list.** Zed's `inventory`-based registration means a new component shows up in the preview without anyone editing an index (`crates/component/src/component.rs`). For kjk, each screen module (`crates/ui/src/screens/`) would declare its own entries: screen name, layout, the steps that reach each state, and a description. A test then fails if a screen in `screens/` has no entry.
2. **A preview is a list of named, grouped examples.** Zed's `single_example(name, element)` inside `example_group_with_title(title, ...)` is the unit. In kjk the unit would be one state of one screen, for example "Inventory, filter set to weapons, item selected", reached by the same `--step "press TAG"` steps that `kjk gui screen` already takes. Giving every state a stable name makes it citable in a review.
3. **A status per component.** Zed's four statuses (Work In Progress, Ready To Build, Live, Deprecated) fit an audit. kjk could mark each screen as Audited, Needs work, or Not started, which would also be the audit's progress report.
4. **A scope or category per component.** Zed groups by purpose (Forms and Input, Navigation, Overlays and Layering). kjk screens could be grouped as Menus, Character, Inventory and stores, Conversation, HUD, Options and Loading, which is how the owner would search.
5. **Descriptions travel with the component.** Zed uses the type's doc comment as the description (`Documented` derive). kjk can keep the description next to the registration and write it into the sidecar.
6. **A visible backdrop for translucency.** Zed draws previews on a diagonal-stripe pattern so transparent parts show (`pattern_slash` in `crates/component/src/component_layout.rs`). KOTOR's panels are translucent, so the gallery should draw each state over both a plain backdrop and a patterned one, or show the alpha as a second image.
7. **One scene description, many backends.** GPUI paints into a platform-neutral `Scene` of a few primitive kinds, and the same scene goes to Metal, DirectX, wgpu and a test renderer (`crates/gpui/src/scene.rs`, `crates/gpui_platform/src/gpui_platform.rs`). kjk already works this way with `DrawList`, `raster` and the Bevy material. The lesson is to keep it that way, and to test that the two backends agree.
8. **Deterministic time for screenshots.** Zed's visual tests control the clock (`advance_clock`) so that tooltips and animations are captured at a known moment (`crates/gpui/src/platform/visual_test.rs`). kjk's headless screens should take a fixed time and a fixed seed, so the same state always gives the same PNG.
9. **Baselines and an update switch.** Zed compares screenshots with stored baselines and rewrites them with `UPDATE_BASELINE=1` (`crates/zed/src/visual_test_runner.rs`). kjk could keep the gallery's PNGs and sidecar hashes in the repository, compare them in a check, and update them with a single command. A change then shows up as a visible diff in a pull request.
10. **Typed design tokens.** Zed components read colours from a theme (`cx.theme().colors().text`, `.text_muted`, `.border`) and not from literals (`crates/component/src/component_layout.rs`). For kjk, the tokens would be the values in `ui::paint::Palette` and the skin (`ui::skin`), written out in the sidecars so the audit can say which token a state uses.
11. **A low level escape hatch beside the high level.** GPUI offers the `canvas` element so a custom drawing can sit inside the normal tree (`crates/gpui/src/elements/canvas.rs`). kjk's equivalent is already the `DrawList`. New components should be built from named, reusable pieces (button, list row, bar) so that the gallery can show the piece once and each screen that uses it.

## 5. Suggested shape of the kjk storybook

This is a proposal for the owner to react to, not something I built.

- A **manifest** in Rust, next to each screen, that lists its states. Each state has a name, the game (K1 or TSL), the steps, the window size, and a status and category.
- `kjk gui storybook` (a new subcommand over the existing renderers) walks the manifest. For each state it writes `<screen>/<state>.png` and `<screen>/<state>.json`.
- The JSON sidecar holds the state's name, description, steps, size, layout file, the controls and their `ControlState`, the textures and fonts used, the palette values used, and a hash of the PNG. An agent can read the sidecar and does not need to look at the image to know what is on the screen.
- A single `index.json` lists every state, and a generated `index.html` shows them in a grid grouped by category, with the status and the diff against the baseline.
- A test checks that every screen has a manifest entry and that every sidecar is current.

## 6. Risks and what I could not check

- I read GPUI's source and not its behaviour. I did not build it, run it or measure it.
- I did not read `component_preview.rs` beyond its structure and header, nor the inspector or the accessibility module in detail.
- The history of the removed `storybook` crate is not available from a shallow clone, so I make no claim about it.
- The gpui-component figures (stars, version, component count) come from a page read through a summarising fetch tool, and may be off in detail. They are given only to show that GPUI has users outside Zed.
- Whether `render_to_image` works headlessly on Windows is stated by its presence in `crates/gpui_windows`, not by a test I ran.

## Sources

- https://www.gpui.rs/
- https://zed.dev/blog/videogame
- https://crates.io/api/v1/crates/gpui
- https://github.com/longbridge/gpui-component
- https://github.com/zed-industries/zed, commit 8aeb64f7444d25e91e3369bf7f4985d03bf336de: `crates/gpui/README.md`, `crates/gpui/Cargo.toml`, `crates/gpui/src/scene.rs`, `crates/gpui/src/window.rs`, `crates/gpui/src/elements/canvas.rs`, `crates/gpui/src/platform/visual_test.rs`, `crates/gpui/src/app/visual_test_context.rs`, `crates/gpui/examples/README.md`, `crates/gpui_platform/src/gpui_platform.rs`, `crates/gpui_windows/`, `crates/gpui_wgpu/`, `crates/component/src/component.rs`, `crates/component/src/component_layout.rs`, `crates/component_preview/src/component_preview.rs`, `crates/component_preview/examples/component_preview.rs`, `crates/ui/src/components/button/button.rs`, `crates/ui_macros/src/ui_macros.rs`, `crates/zed/src/visual_test_runner.rs`, `crates/zed/src/zed/visual_tests.rs`, `rust-toolchain.toml`
- kjk: `docs/UI.md`, `crates/ui/src/`, `rust-toolchain.toml`
