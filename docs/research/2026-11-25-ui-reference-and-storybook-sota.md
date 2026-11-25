# UI design reference and a "storybook" for kjk, state of the art in November 2026

Research done on 2026-11-25 and 2026-11-26. Every claim below names the page it came from. Where I could not fetch a primary page, the text says so and does not present the claim as checked.

## Summary and recommendation

kjk is not a web UI, so Storybook itself does not fit. Its ideas fit well, though, and kjk already has most of the machinery. Its `kjk gui screen <game> <screen> --step "press TAG"` command is already a story runner, and its `kjk gui sweep` and `kjk gui sheet` commands are already a catalogue builder.

What the field does in 2026 can be reduced to five points.

1. Storybook models a screen as a named, replayable state. A story is a component plus its inputs (args), an optional `play` function that drives it, and a rendered result that is compared with an approved baseline. Storybook now also generates JSON manifests of its components and serves them to coding agents through an MCP server. The agent asks "what components exist and what are their props" and does not scrape pictures.
2. Design tokens have a stable format. The W3C Design Tokens Community Group (DTCG) published its first stable version, 2025.10, on 28 October 2025. The format has no type for textures, 9-slice borders, bitmap fonts or sounds, but it allows vendor data in `$extensions`, so a game skin can extend it.
3. Agent-facing references are plain files and thin CLIs. `AGENTS.md` is the common convention for instructions. Google's open-source `DESIGN.md` format (version `alpha`) puts design tokens in YAML front matter and the reasoning in prose, with a linter. Android now ships a CLI and "skills" so that an agent can render a Compose preview to an image without a device. Xcode 26.3 lets an agent capture a SwiftUI preview over MCP. The pattern is the same everywhere. An agent gets a command that returns an image plus structured data.
4. In non-web UI, the borrowable principles are these. Engine themes hold styles apart from widgets (Unreal Common UI style assets, Godot themes, Unity USS variables). Previews are annotated, named, parameterised states (Flutter, SwiftUI, Compose). Screenshot tests are "record, compare, review" with a diff image and a JSON report (Paparazzi, Roborazzi, `egui_kittest`). Roborazzi also writes a JSON dump of every element's rectangle, which it presents as a way for agents to verify layout.
5. Game studios mostly design in Figma or Photoshop and then rebuild in the engine, and engine-side tooling (UI Toolkit, Common UI) carries the iteration. I found no first-party source that argues for HTML prototypes as the target of record.

**Recommendation.** Build a kjk "UI catalogue" in the repo with four parts.

- Hand-written story files that list each screen, its states (the `--step` lists), and the viewports to render (Steam Deck 1280 x 800, a 1080p TV, the original 4:3 frame, an ultrawide window).
- A DTCG token file (`ui/tokens/*.tokens.json`, with kjk types in `$extensions`) as the single place for colours, sizes and motion. An `xtask` command generates the Rust constants from it, and a generated Markdown page documents it.
- A generated catalogue. For each story and viewport, the existing renderer writes a PNG and a JSON sidecar that lists every control (tag, kind, rectangle, font height in pixels, focus links). A manifest `catalog.json` indexes them, and a static HTML viewer shows them for the owner.
- A hand-written `docs/ui/README.md` that follows the `llms.txt` shape (a short summary, then link lists), which is the entry point for agents, together with a short `DESIGN.md`-style page for the principles.

Target designs should be authored in-engine, as kjk-side overrides applied over KOTOR's layouts, and previewed with before and after sheets. Figma and HTML are not needed, because the art is KOTOR's own textures and the layouts are already data.

CI can only run what needs no game data. Pictures cannot be committed, because they contain Bioware and Aspyr art. The geometry sidecars and the token and story checks can be committed, and they need no art. The owner's rule that only approved tests are permanent applies here, so the checks below are proposals for approval.

## 1. Storybook's model and its agent features

### Stories, args, states, docs

- A story is "the rendered state of a UI component". The default export (the meta) names the component, and every named export is one story, such as `Primary` or `Secondary`. Source: https://storybook.js.org/docs/writing-stories
- Args are the component's inputs and can be edited live in the Controls panel. Stories can build on other stories' args. Source: https://storybook.js.org/docs/writing-stories
- A `play` function runs after the story renders. It simulates user actions and checks results with `expect`. Source: https://storybook.js.org/docs/writing-stories
- Parameters are static metadata for addons, and decorators wrap a story in providers or layout. Source: https://storybook.js.org/docs/writing-stories
- The page I fetched does not describe tags or autodocs, so I do not make claims about them.

### Tests

- Interaction tests are `play` functions, and the testing page calls them the foundation of testing in Storybook. The Vitest addon runs stories as real Vitest tests in browser mode. The Accessibility addon checks contrast, keyboard use and screen reader support. The older test-runner is the fallback for projects that cannot use the Vitest addon. Source: https://storybook.js.org/docs/writing-tests
- Visual regression is done by Chromatic, a cloud service from the Storybook team. It turns each story into a snapshot, renders it in cloud browsers, and pixel-diffs it against an approved baseline. Reviewers sign off in the service. Modes let one story be tested across themes, viewports and locales. SteadySnap freezes dynamic content to reduce flaky results, and TurboSnap re-snapshots only stories that depend on changed files. Source: https://www.chromatic.com/docs/

### Agent-facing features

The Storybook documentation lists these as a preview whose API may change. Source: https://storybook.js.org/docs/ai

- Agentic setup. `npm create storybook@latest` run through an agent produces step-by-step instructions that the agent follows to add Storybook, write stories and check them.
- The MCP addon, installed with `npx storybook add @storybook/addon-mcp`, serves `http://localhost:6006/mcp` while the dev server runs. Its named tools are `docs-list` (the documented components), `docs-show` (one component's properties and example stories), `get-storybook-story-instructions` (how to write or update stories), and `test-run` (run tests on the live Storybook).
- The page gives a sample `AGENTS.md` section that tells the agent to call these tools before acting and to verify props against the docs and not assume them.
- Manifests are JSON objects that map the Storybook's content. There is a components manifest built from story files and a docs manifest built from MDX. They are off by default and are switched on with `features: { componentsManifest: true }`. Source: https://storybook.js.org/docs/ai and https://storybook.js.org/docs/api/main-config/main-config-features
- An experimental `experimentalDocgenServer` option makes manifests use "a ref-based format with per-component JSON snapshots", which the page says loads faster for agents. Source: https://storybook.js.org/docs/api/main-config/main-config-features

The MCP code was first in its own repository. That repository is now archived, and its README says the project "has moved to storybookjs/storybook as of Storybook v10.6.0". It lists a Claude Code plugin and a Codex plugin with setup skills. Source: https://github.com/storybookjs/mcp

**Principle to borrow.** The agent should not look at pictures to learn what exists. It should read an index (`docs-list`), then read one entry (`docs-show`), then run a check (`test-run`). kjk can offer the same three verbs through `kjk gui` subcommands with `--json`, and does not need an MCP server, because agents already have a shell.

## 2. Design tokens

### The DTCG format

- Version 2025.10 is the first stable version. The home page says "First stable version 2025.10 now available" and links an announcement dated 28 October 2025. Source: https://www.designtokens.org/
- The specification page calls itself a W3C Community Group Final Report dated 28 October 2025, says "This specification is considered stable", and is not on the W3C Standards Track. Source: https://www.designtokens.org/tr/2025.10/format/
- A newer draft is dated 8 September 2026, is described as not to be implemented or cited, and still links 2025.10 as the latest published version. Source: https://www.designtokens.org/tr/drafts/format/
- Simple types are `color`, `dimension`, `fontFamily`, `fontWeight`, `duration`, `cubicBezier` and `number`. Composite types are `strokeStyle`, `border`, `transition` and `shadow`, plus gradient and typography. Source: https://www.designtokens.org/tr/2025.10/format/
- A `file` type is not defined. Section 8.8 lists "File" among types "still to be documented", next to font style and percentage. Source: https://www.designtokens.org/tr/2025.10/format/
- Aliases use `{group.token}`, and a `$ref` JSON pointer can point at a single property. Groups can inherit with `$extends`. Source: https://www.designtokens.org/tr/drafts/format/
- `$extensions` may hold vendor data on tokens and groups. Keys should use reverse domain notation, and tools "must preserve extension data they don't understand". Source: https://www.designtokens.org/tr/2025.10/format/
- A separate Resolver Module (also 2025.10) handles themes. It has sets of tokens, and modifiers (such as a `theme` modifier with `light` and `dark` contexts) that choose which sets apply. `resolutionOrder` says which wins. Source: https://www.designtokens.org/tr/2025.10/resolver/
- Adopters include Figma, Penpot, Sketch, Tokens Studio, Style Dictionary, Terrazzo and Supernova. Source: https://www.designtokens.org/

### Style Dictionary

Style Dictionary 4 has "first-class support for the DTCG format". Full support for 2025.10 is "a work in progress for v5". A converter turns older `value`/`type` JSON into `$value`/`$type`. Source: https://styledictionary.com/info/dtcg/. The page I fetched does not cover custom types, transforms or formats, so I did not check how a game-specific type would be added.

### Does DTCG fit a game skin

It fits the part that is plain values, and it needs extending for the rest.

| kjk need | In DTCG 2025.10 | What kjk adds |
|---|---|---|
| Colours (brass, blue steel, text colours, KOTOR 1 and 2 palettes) | `color`, with the Resolver for the two games | A `game` modifier in a resolver file, next to `light` and `dark` if those are ever needed |
| Font heights, spacing, margins in layout pixels | `dimension` | A note that units are layout pixels of the 640 x 480 or 800 x 600 frame, not CSS pixels |
| Motion (menu fades, focus pulse) | `duration`, `cubicBezier`, `transition` | None |
| Textures (frames, buttons, portraits) | No `file` type | A `com.kjk.texture` extension that names the resource and the game it comes from |
| 9-slice borders | `border` is a stroke, not an image slice | A `com.kjk.nineSlice` extension with four insets in texture pixels |
| Bitmap fonts | `fontFamily` names a family only | A `com.kjk.bitmapFont` extension with the TXI resource name and cell height |
| Sounds (focus move, accept, back) | None | A `com.kjk.sound` extension with a resource name |
| Control states (normal, focus, pressed, disabled) | None | A group per state, or a `com.kjk.states` map |
| Gamma-space blending | None | Document it as a rule. kjk blends in gamma space, as KOTOR does (`docs/UI.md`) |

Because tools must preserve `$extensions`, a standard DTCG tool will still carry these through. The real consumer is kjk's own `xtask`, so no third-party tool needs to understand the game types.

One caution applies. The texture and sound tokens would name KOTOR resources, which kjk does not ship. They must stay as names, never as files.

## 3. Agent-facing design references

- **Figma MCP server.** It gives agents "the design context they need to generate code from your Figma files", pulls variables, components and layout data into the IDE, and can write native content back to Figma Design, FigJam and Slides. The remote server is `https://mcp.figma.com/mcp`. Skills tell an agent which tools to call and in what order. Source: https://developers.figma.com/docs/figma-mcp-server/
- **Code Connect.** It links components in a code repository to the matching components in Figma. Connections are template files such as `Button.figma.ts` that map design properties to code props. The connections give agents "direct references to your actual code". Source: https://developers.figma.com/docs/code-connect/
- **`llms.txt`.** A proposal by Jeremy Howard, first published on 3 September 2024 and last modified on 10 August 2026. It is a Markdown file with an H1 name, a blockquote summary, and H2 sections of `[name](url)` links, with an "Optional" section for links an agent may skip. It is an informal proposal and not a standard, although the page says thousands of sites publish one. Source: https://llmstxt.org/
- **`AGENTS.md`.** An open plain-Markdown format for instructions to coding agents, now maintained by the Agentic AI Foundation under the Linux Foundation. Source: https://agents.md/
- **`DESIGN.md`.** Google Labs' open-source format. It is a Markdown file with YAML front matter for tokens (`colors`, `typography`, `rounded`, `spacing`, `motion`, `components`) and prose sections in a fixed order (Overview, Colors, Typography, Layout, Elevation and Depth, Shapes, Motion, Voice and Content, Components, Do's and Don'ts). Token references are written `{colors.primary}`. A linter reports duplicate tokens and checks WCAG contrast, and the CLI can export to DTCG. The spec's version is `alpha` and it says the components section is still evolving. Source: https://raw.githubusercontent.com/google-labs-code/design.md/main/docs/spec.md
- **Android CLI and skills.** Android ships a command-line tool designed for "agent-first workflows". It can "render Jetpack Compose previews to images on your machine, with no device or Android Studio required", inspect "live device screens and UI hierarchies", and install an agent skill with `android init`. Source: https://developer.android.com/tools/agents/android-cli
- **Xcode 26.3.** It exposes its capabilities over MCP, and Apple says agents can "verify their work visually by capturing Xcode Previews". Source: https://www.apple.com/newsroom/2026/02/xcode-26-point-3-unlocks-the-power-of-agentic-coding/. I could not confirm the name or schema of the preview tool from an Apple page.

**How design systems expose themselves to agents in 2026**, taken together. They publish an instructions file (`AGENTS.md` or a skill), a machine-readable index of components and tokens (Storybook manifests, `DESIGN.md` front matter, DTCG), and a command that returns an image for a named state (Android CLI, Xcode previews, Storybook `preview-stories`). A reference for kjk needs the same three things.

## 4. Non-web equivalents

For each one, the principle worth borrowing.

| Tool | What it does | Principle for kjk | Source |
|---|---|---|---|
| Unreal Common UI | A plugin whose style data assets "separate styling from UI elements, so you can share styles across multiple UIs". It also has input routing and gamepad navigation. A button style holds a brush per state and a text style for normal, hovered, selected and disabled. | Widgets never hold their own look. A style asset per control kind and per state is the single place to change it. Gamepad navigation is part of the UI framework, not a later addition. | https://dev.epicgames.com/documentation/en-us/unreal-engine/common-ui-plugin-for-advanced-user-interfaces-in-unreal-engine and https://dev.epicgames.com/documentation/unreal-engine/API/Plugins/CommonUI/UCommonButtonStyle |
| Unity UI Toolkit | USS style sheets "inspired by CSS", with custom properties, selectors, and Theme Style Sheets, plus UI Builder. Timberborn's team reports prototypes that took one to three days in the old system now taking hours, and near-zero merge conflicts. | Keep the look in a text file that people and agents can diff, and keep the UI logic apart so a component can be tested alone. | https://docs.unity.com/en-us/engine/current/manual/uitoolkits/uielements/uie-uss and https://unity.com/resources/timberborn |
| Godot themes | A Theme resource holds colours, constants, fonts, font sizes, icons and styleboxes, a `StyleBoxTexture` is a nine-patch, and `set_type_variation` derives one type from another. A theme editor has a live preview. | One resource carries every item type, including 9-slice textures, and variations avoid copies. This is the closest match to a KOTOR skin. | https://docs.godotengine.org/en/latest/classes/class_theme.html (class reference, seen only as a search result, since the tutorial pages returned errors) |
| Slint | A live preview in VS Code, a Live-Preview Data Editor to change properties without recompiling, and GUI test automation for CI. | Let the viewer change inputs without a rebuild. | https://slint.dev/ |
| egui | `egui_kittest` renders a harness and compares it with a saved image. | See section 5. | https://docs.rs/egui_kittest |
| iced | `iced_test` runs an app headlessly through a `Simulator` that can take snapshots. | A headless driver is the base of every state test. | https://docs.rs/iced_test/0.14.0 |
| Bevy | Bevy 0.20 was announced on 8 October 2026. A CI-testing feature can take screenshots on chosen frames, and Bevy's own UI testbed has a screenshot check. I found no dedicated headless `bevy_ui` snapshot API. | kjk's own CPU rasteriser is a better fit than a GPU screenshot, because it runs without a display. | https://bevy.org/news/ and https://docs.rs/bevy/0.17.3/src/testbed_ui/helpers.rs.html |
| Flutter widget previewer | `@Preview` marks a function as a named preview, with `group`, `size`, `brightness` and `textScaleFactor`. Several annotations can stack on one function. A search bar filters by name, group and file. | A preview is a named function with parameters, and a group is a first-class idea. | https://docs.flutter.dev/tools/widget-previewer |
| SwiftUI previews | `#Preview` blocks with names and traits, and a Variants view that shows many settings at once. I could not fetch Apple's page, so this is from general knowledge. | One screen, many variants in one picture. | https://developer.apple.com/documentation/xcode/previewing-your-apps-interface-in-xcode (page returned only its title) |
| Jetpack Compose | `@PreviewTest` turns a preview into a screenshot test. Reference images are recorded with `updateScreenshotTest`, checked with `validateScreenshotTest`, and a failure writes an HTML report with Reference, Actual and Diff side by side. Reference file names are the test function plus a hash of the preview parameters. | Name each reference by story and parameters. Review diffs side by side. The standalone plugin is no longer maintained, and Google recommends AGP test suites. | https://developer.android.com/studio/preview/compose-screenshot-testing |
| Paparazzi | Renders Android screens without a device. `recordPaparazziDebug` saves golden PNGs, `verifyPaparazziDebug` compares and writes diffs. Goldens live in the repository, and the README recommends Git LFS. | Goldens are plain PNGs in the repository, and a size policy is needed. | https://github.com/cashapp/paparazzi |
| Roborazzi | Record, compare and verify tasks, a `_compare.png` diff, a JSON report, a tunable change threshold, and optional AI assertions that run only when images differ. It also writes a "deterministic JSON sidecar" of element coordinates and an annotated image with numbered elements, which its page presents as a way for agents to verify layout changes. | The JSON sidecar of element rectangles is the thing kjk should copy most directly. | https://github.com/takahirom/roborazzi |

## 5. Snapshot and visual regression testing in Rust

- **`insta`** is the standard Rust snapshot library. It stores snapshots as files, reviews them with the companion `cargo-insta`, and diffs with the `similar` crate. Source: https://github.com/mitsuhiko/insta. The pages I fetched did not confirm binary snapshots, redactions or the CI settings, so I do not describe them here. The `egui_kittest` documentation itself advises preferring regular Rust tests or `insta` snapshots where possible, because image tests are slower and more brittle. Source: https://docs.rs/egui_kittest
- **`egui_kittest`** gives the clearest model for image snapshots. Reference images go in `tests/snapshots`. A per-pixel `threshold` (default 0.6, a weighted squared YIQ distance) and `max_failed_pixels` (default 0) decide a pass. Values can be set per operating system in `kittest.toml`. `UPDATE_SNAPSHOTS=true` rewrites only failing images, and `UPDATE_SNAPSHOTS=force` rewrites all. A failure writes `*.diff.png` and `*.new.png`. The documentation warns that raising `max_failed_pixels` above about 10 can hide real changes, and says egui keeps its images in Git LFS. Source: https://docs.rs/egui_kittest
- **`image-compare`** compares two images with RMS, MSSIM or hybrid luma and colour scores, returns a score from 0.0 to 1.0, and can return a per-pixel diff map that converts to a colour image. Images of different sizes return an error. Source: https://docs.rs/image-compare/latest/image_compare/
- **`iced_test`** has a headless `Simulator` with snapshot support. Source: https://docs.rs/iced_test/0.14.0

**What fits kjk.** kjk's rasteriser is on the CPU and deterministic, so most of the cross-machine noise that these tools tolerate does not exist. That allows an exact comparison (zero failed pixels) on the same machine, or on any machine if the rasteriser is bit-exact across operating systems. The more useful check is the JSON sidecar. It can be compared structurally with ordinary JSON snapshot assertions, and a pixel diff is only the second line of defence. A small helper in `xtask` that writes `*.new.png` and `*.diff.png` next to the golden, with an `UPDATE_SNAPSHOTS` switch, copies the `egui_kittest` workflow. I would not add a heavy dependency for it.

## 6. Where target designs should be authored

Findings from game studios, all indirect.

- Figma is the usual place for exploration. A widely repeated pattern is that designers test menus, dialogue and prompts in Figma and then rebuild the approved designs in Unity or Unreal. The source I found for this is a general article and not a studio postmortem. Source: https://www.toptal.com/designers/ui/game-ui
- King moved its design teams to Figma in 2022, builds component libraries there, and runs a proprietary C++ engine. The page does not describe the handoff mechanics and does not mention tokens. Source: https://www.figma.com/customers/how-king-brings-game-design-together/
- Engine-side tools carry the iteration. Timberborn's team rebuilt its UI in Unity UI Toolkit and reports prototypes that went from days to hours. Source: https://unity.com/resources/timberborn
- Epic's Common UI documents styles as engine data assets, which is the in-engine route. Source: https://dev.epicgames.com/documentation/en-us/unreal-engine/common-ui-plugin-for-advanced-user-interfaces-in-unreal-engine
- A Unreal Fest 2025 talk, "How to Build UI at Scale: Lessons from Marvel's Midnight Suns", covers building many screens quickly with shared functionality. I could read only its summary. Source: https://forums.unrealengine.com/t/talks-and-demos-how-to-build-ui-at-scale-lessons-from-marvel-s-midnight-suns-unreal-fest-orlando-2025/2674024
- The Diablo II Resurrected talk at GDC 2022 mentions controller support as a focus, and the page does not describe UI method. Source: https://gdcvault.com/play/1028091/Remastering-a-Classic-Diablo-II
- I found no first-party source about HTML prototypes as the reference for a game's UI, and none about how the Aspyr KOTOR releases or other remasters (Mass Effect Legendary Edition, Age of Empires) authored their UI. Treat any claim of "what remasters do" as unproven. The evidence supports "Figma for exploration, engine for the real thing" and little more.

**Recommendation for kjk.** Author in-engine.

- kjk's screens already are KOTOR's `.gui` layouts painted by a Rust runtime (`docs/UI.md`). The art is KOTOR's own textures. A Figma file or an HTML page would have to redraw all of it, and the redrawn version would then drift from the engine's output.
- A target design is a set of changes to a layout. The cleanest form is a kjk-side override file (TOML or RON, hand-written) applied after the `.gui` is read. It sets, per control tag, a rectangle, a font, a focus link, or a visibility. Agents can write and diff it, and `kjk gui screen` shows it at once.
- The owner then looks at a generated before and after sheet, not at a prototype of something else.
- HTML prototypes stay a throwaway tool for questions that have no KOTOR layout to start from, such as a new radial menu for a gamepad. The existing `prototype` skill covers that.
- Figma is optional. If the owner wants to sketch there, the Figma MCP server can read the sketch, but nothing in this plan depends on it.

## 7. Concrete recommendation for kjk

### Facts about the repository that shape this

- `docs/UI.md` says `fmt_odyssey::gui` reads a layout, `ui::paint` makes a `DrawList`, and `ui::raster` draws it on the CPU, "so headless pictures match" the GPU path.
- `crates/ui/src/sweep.rs` already paints every layout of an install at rest and reports failures, refusals, unknown control kinds and unread fields.
- `crates/ui/src/screens/` has 41 screen modules, and `crates/ui/src/probes/` has probes for many of them.
- `kjk gui screen`, `kjk gui sweep` and `kjk gui sheet` exist (`docs/UI.md`).
- `docs/TESTS.md` says the only permanent tests are the ones the owner approved.
- `.github/workflows/ci.yml` runs on `ubuntu-latest`. Nothing there installs KOTOR, and the repository is public (`https://github.com/Superd22/kjk.git`).

### File layout

```
docs/ui/
  README.md            hand-written, llms.txt shape, the entry point for agents
  DESIGN.md            hand-written principles, DESIGN.md section order
  tokens.md            generated from the tokens, never edited
  inventory.md         generated list of screens and states with check results
ui/
  tokens/
    kjk.tokens.json    hand-written, DTCG 2025.10, kjk types in $extensions
    kjk.resolver.json  hand-written, the game modifier (k1, k2) over the tokens
  stories/
    <screen>.toml      hand-written, one file per screen
  overrides/
    <screen>.toml      hand-written, target design as changes to a layout
  baselines/
    <screen>/<story>.<viewport>.json   committed sidecars, geometry only
crates/ui/src/tokens.rs   generated Rust constants
target/ui/                generated pictures, manifest and viewer, not committed
  catalog.json
  index.html
  <screen>/<story>.<viewport>.png
  <screen>/<story>.<viewport>.json
```

### What is hand-written and what is generated

| Item | Written by | Notes |
|---|---|---|
| Tokens, resolver | Hand | The single source for colours, sizes and motion. |
| `tokens.rs` and `docs/ui/tokens.md` | `cargo xtask ui-tokens` | A CI check fails when they are stale. |
| Story files | Hand | A story names a screen, a game, a list of `--step` strings, a viewport list and an input device (mouse or gamepad). |
| Overrides | Hand | Target designs. |
| PNG, sidecar, `catalog.json`, `index.html` | `cargo xtask ui-review` | Uses the existing `kjk gui screen` code path. |
| `docs/ui/inventory.md` | `cargo xtask ui-review` | Counts, check results and links. |

### The story file

A story file is short and has no logic.

```toml
screen = "inventory"
games = ["k1", "tsl"]

[[story]]
name = "empty"
steps = []

[[story]]
name = "item_selected"
steps = ["press LB_ITEMS", "press BTN_DOWN"]
input = "gamepad"

[viewports]
list = ["deck_1280x800", "tv_1920x1080", "frame_4x3"]
```

The story names follow the `Storybook` idea of one name per state. The `input` field matters for kjk, because the gamepad and mouse states differ in which glyphs show and which control has focus.

### The sidecar

Each rendered picture gets a JSON file next to it, modelled on Roborazzi's element dump. For every control it lists the tag, kind, rectangle in output pixels, state (focused, disabled), font name, rendered text height in pixels, and the `MOVETO` focus links. It also lists the story, the viewport, the game, and the list of refusals from the painter. It must not contain KOTOR strings from the talk table, only tags and numbers, so that it can be committed.

### Checks the generator runs

All of these read the sidecar and need no picture comparison.

1. Text height. At 1280 x 800, no essential text is below 9 pixels, and the target is 12 pixels. The sources are Valve's rule ("9px is the absolute minimum text size for approval, and 12px is recommended", from the search snippet of https://partner.steamgames.com/doc/steamdeck/compat, which I did not fetch directly) and Microsoft's handheld guideline (9 pixels minimum and 12 recommended at 720p, 14 and 18 at 1080p, readable from 12 inches). Source: https://learn.microsoft.com/en-us/gaming/gdk/docs/gdk-dev/pc-dev/handheld/handheld-guidelines-and-testcases
2. Focus reachability. Starting from the default focus, every enabled control can be reached through the `MOVETO` links, and the way back works.
3. Glyph correctness. A story with `input = "gamepad"` shows no keyboard or mouse glyph. Valve and Microsoft both require this. Sources: https://partner.steamgames.com/doc/steamdeck/compat and https://learn.microsoft.com/en-us/gaming/gdk/docs/gdk-dev/pc-dev/handheld/handheld-guidelines-and-testcases
4. Safe area. Controls that matter on a TV stay inside the central 90 percent of the frame. This is my proposed rule, and I did not find a first-party source for 90 percent.
5. Overlap and overflow. No two controls overlap unless the layout allows it, and no text overflows its control.
6. Contrast. Text against its background meets a WCAG ratio, computed in the same gamma space the engine blends in. The `DESIGN.md` linter does a similar check on tokens.
7. The existing sweep. Everything that `kjk gui sweep` already reports.

### What agents read

1. `docs/ui/README.md`, which lists the other pages in the `llms.txt` shape.
2. `docs/ui/DESIGN.md`, the principles and the do and don't list.
3. `ui/tokens/kjk.tokens.json`, or its generated `docs/ui/tokens.md`.
4. `kjk gui list --json`, `kjk gui show <screen> --json` and `kjk gui story <screen> <story> --viewport <v>`, which return the index, one screen's stories and one render with its sidecar path. These are the equivalents of `docs-list`, `docs-show` and a preview call.

The project `CLAUDE.md` or `AGENTS.md` gets a three-line pointer to `docs/ui/README.md` and a rule that a UI change must run `cargo xtask ui-review` for the affected screens and look at the picture.

An MCP server is not needed. Storybook built one because its content sits behind a browser and a dev server. kjk's content is behind a CLI the agent can already run.

### What the owner looks at

`target/ui/index.html`, one static page generated from `catalog.json`. It has a screen list, tabs for each story, a viewport switch, a before and after view (the baseline against the current render, or the original layout against the override), and the check results with failures first. This matches the Chromatic review step, but local. The page can also be published as an artifact when the owner wants to share it.

### CI

- On every pull request, with no game data, run these.
  - Parse the tokens and the story files, and fail on a broken alias or a screen name that does not exist.
  - Regenerate `tokens.rs` and `tokens.md` and fail if they changed.
  - Check that every override names a control tag that exists in the committed baseline for that screen.
  - Compare the committed sidecars in `ui/baselines/` against a fresh render only where game data exists.
- The pixel pictures are never committed. They contain Bioware and Aspyr art, and the repository is public. Paparazzi and `egui_kittest` commit their goldens to Git LFS, but that choice does not apply here.
- A render and a comparison need KOTOR installed. The owner's machine and the Windows PC (`docs` and memory notes) have it. CI has none, so the full `ui-review` is a local step before a pull request, and its result is summarised in the pull request description. If the owner later adds a self-hosted runner with the games, `ui-review --check` can run there.
- Because only owner-approved tests are permanent, I propose one approved test, `ui_catalogue`, that runs the checks above on a machine with game data. The generators and the checks themselves are tools, not tests.

### Suggested order

1. Add the sidecar to `kjk gui screen` and `kjk gui sweep`. This is a small change and it unlocks every check.
2. Write the first twenty stories from the screens that already have probes (`menus`, `chargen`, `hud`, `pazaak`, `dialog`, `party`).
3. Add `ui-review`, the viewer and the text-height and focus checks, and run them over every screen once to produce the audit list.
4. Write the tokens from what `Palette::of(game)` and `skin.rs` already hold, so that the first tokens describe the engine as it is today.
5. Add the override layer when the first target design needs it.

## What I could not verify

- Godot's tutorial pages and Unreal's Common UI tutorial pages returned errors or empty text. The Godot and Unreal rows rely on the class reference listings that the search tool returned and on the one Common UI overview page.
- Unity's USS variables and 9-slice properties were not described on the page I fetched. I did not state them as facts above.
- Apple's SwiftUI preview page returned only its title. The SwiftUI description is general knowledge, and the Xcode MCP claim is limited to Apple's newsroom wording.
- Valve's Steamworks page was seen only through a search summary. The 9 and 12 pixel figures should be confirmed on that page before they become a written rule.
- `insta`'s binary snapshot and CI settings were not on the pages I fetched.
- Style Dictionary's extension points for custom token types were not covered by its DTCG page.
- I did not run any of the tools. All findings come from documentation.
