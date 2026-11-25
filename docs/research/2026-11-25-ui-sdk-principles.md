# UI SDK (ui-sdk.dev) and what kjk can take from it

Researched on 2026-11-25. Every claim below was read from the page named next to it, fetched on that day.

## Summary and recommendation

UI SDK is a beta JavaScript library from Vercel. It gives AI agents a set of small interaction behaviors (a popup, a selection model, typeahead) that an agent assembles into web components such as a Select or a Combobox. It also gives agents a structured way to read and operate the finished interface without screenshots.

kjk cannot adopt it. The library is built for React, Vue, Svelte, Solid and Web Components, and kjk draws KOTOR's `.gui` layouts with a Rust runtime.

Recommendation: borrow ideas, do not adopt the code. Three ideas carry over well:

1. A structured snapshot of a screen that lists each control with its label, value and available actions.
2. A record of who caused each change, so that a test or an agent can tell a keyboard press from a scripted action.
3. A checker that verifies each screen's navigation (focus order, directional moves) against a stated pattern.

The web-specific parts (framework bindings, ARIA, DOM, server rendering, WebMCP) do not apply. The site's benchmark of agents operating UI is also web-only and says nothing about KOTOR layouts.

The site states no design principles of its own, and it has no design tokens, no spec and no visual style guidance. It is not a source for kjk's design tokens or look. It is useful only as a model for how to make a UI legible to agents.

## 1. What it is

- Name and purpose: "A library of behaviors for agents to build and drive the next generation of user interfaces." Source: https://ui-sdk.dev/ and https://github.com/vercel-labs/ui
- Maker: Vercel. The site names Vercel, and the source sits in the `vercel-labs` GitHub organization. Source: https://ui-sdk.dev/ and https://github.com/vercel-labs/ui
- Licence: Apache-2.0. Source: https://github.com/vercel-labs/ui
- Maturity: it is in beta. The docs say "UI SDK is in beta, so some APIs may still change." The README's beta terms say the framework, APIs, documentation and behaviour may change before general availability. Source: https://ui-sdk.dev/docs and https://github.com/vercel-labs/ui
- Age: the changelog has one entry, dated 12 November 2026, "Introducing UI SDK", with no version numbers. The library is therefore 13 days old on the day of this research. Source: https://ui-sdk.dev/docs/core/changelog
- Activity: the repository shows about 7.5k stars, 749 commits on `main`, 164 open issues and 29 open pull requests. Its Releases section was empty and no tags were listed. The repository uses a `.changeset` directory. Source: https://github.com/vercel-labs/ui (a figure read from a fetched page summary, not verified against the GitHub API).
- Problem it solves: component libraries ship finished components, such as a Select. UI SDK ships the parts instead (a popup, a selection model, typeahead), so that an agent can combine them into a Select, an autocomplete, a combobox, or a component nobody has built yet. Source: https://ui-sdk.dev/
- Frameworks: React 19 or newer, Vue, Svelte, Solid and Web Components. Source: https://ui-sdk.dev/docs

## 2. Principles and architecture

The site does not list design principles. The docs page says so in effect, and its emphasis is on agents assembling behaviour primitives into components. Source: https://ui-sdk.dev/docs

The principles that the site's pages do state or show are these.

### Behaviours, not finished components

- The library breaks complex interactions (keyboard navigation, focus, selection, popups, drag and drop, screen reader support) into reusable behaviours. It lists more than 50 of them, for example `createRovingFocus`, `createTypeahead`, `createPopup`, `createSelection`, `createListbox`, `createReorder` and `createVirtualizer`. Source: https://ui-sdk.dev/
- A prebuilt component library (Color Picker, Calendar, Tabs, Tree, Command and others) is built from the same behaviours. Source: https://ui-sdk.dev/

### Behaviour is separated from rendering

- `definePrimitive` splits a component into state (the values it tracks), an API (methods and shared logic) and slots (the parts that render). Each slot declares attributes, whether the part is present in a given state, and its event handlers. The same definition can power React parts, web components or other adapters. Source: https://ui-sdk.dev/docs/core/kernel/define-a-primitive
- The docs ask that slot attributes and state stay pure, because they also run on the server. Source: https://ui-sdk.dev/docs/core/kernel/define-a-primitive

### Legible to agents

- Agent bridge. After the app calls `createAgent()`, `agent.snapshot()` returns a structured list of components with their references, labels, values and available actions. An agent calls `agent.act(...)`, which routes the change through the component's own API and change callback. The app can approve or refuse an action with an `allow` callback. Change callbacks receive `source: "agent"`. Source: https://ui-sdk.dev/docs/agents
- Observation stream. `observe` reports each change as one `ObservedChange` object. It has the component kind and id, the accessible label, the key, the new and previous value, a `reason` (such as `item-press` or `drag`), a `source` (such as `keyboard`, `mouse`, `programmatic` or `agent`) and a `ref` that locates the control. Password, card and one-time-code fields report `"[redacted]"`. Source: https://ui-sdk.dev/docs/core/observe
- WebMCP. Tools can be exposed to an agent, or registered with a browser that supports WebMCP. Source: https://ui-sdk.dev/docs/agents
- Docs shipped with the package. Each package ships Markdown docs that match the installed version (`llms.txt`, `core.md`, `rendering.md`), and the setup asks the user to add a note to `AGENTS.md` that points to them. Source: https://ui-sdk.dev/docs
- A skill. The skill walks an agent through reading the installed docs, choosing behaviours, defining a primitive, rendering it in the user's framework and testing the result. It ships inside `@ui-sdk/core` at `skills/ui-sdk/SKILL.md`, and the docs say "That copy always matches the version you installed." Source: https://ui-sdk.dev/docs/agents/skills
- Testing kit. `checkPattern` verifies "roles, names, states, references between parts, and keyboard interactions". For example, it requires Home and End on listboxes, tabs and toolbars, and a name on every tab list. `checkDirection` compares focus and state after arrow keys. `announced` captures the text sent to the screen reader announcer. The docs still say to use a real browser to check a finished component with a keyboard and a screen reader. Source: https://ui-sdk.dev/docs/core/testing

### Benchmark claim

- The site compares a project settings screen built with UI SDK against the same screen built with Base UI, Ark UI, Reka UI and Bits UI. The task was to rename a project, change its visibility, update members and notifications, and save. The UI SDK agent used the bridge, and the others used browser tools. The figures are medians of three runs with Sonnet through Claude Code.
- In React the bridge took 2 tool calls against 17, 13 seconds against 56, and $0.021 against $0.138. Vue and Svelte were close to the same. Token use fell by roughly 91 to 92 percent. Every run completed the task.
- The page also says that a UI SDK screen driven with browser tools "took about as much as the other libraries," so the saving comes from the bridge and not from the components.
- The vendor ran this benchmark on a settings form, with three runs per row. It is a claim by the maker. Source: https://ui-sdk.dev/docs/agents

## 3. What carries over to kjk, and what does not

kjk draws KOTOR's own `.gui` layouts with a Rust runtime (`ui::paint` turns a layout plus each control's `ControlState` into a `DrawList`, `ui::screen::Stack` holds the open screens, and `ui::vm::ViewModel` is everything the screens read). The headless tools are `kjk gui render`, `kjk gui screen ... --step`, `kjk gui sweep` and `kjk gui sheet`. Source: docs/UI.md in this repository.

| UI SDK idea | Carries over? | kjk piece it would map to |
|---|---|---|
| A structured snapshot of every control with label, value and actions (`agent.snapshot()`) | Yes, as an idea | Each screen could emit a text or JSON dump next to its PNG. `kjk gui screen` already takes scripted steps, and the repository already has a "dump" (docs/UI.md mentions `hud.busy` in it). A per-screen control list with the control's tag, label, state and the `Intent` it would raise would let an agent audit a screen without reading pixels. |
| One action path for people and agents (`agent.act`) | Mostly already done | The `--step "press TAG"` and `--step "type TEXT"` steps in `kjk gui screen` already drive screens through the same focus and `Intent` path as a controller. No change is needed other than keeping that property. |
| A change stream with a `source` and a `reason` (`observe`) | Yes, as an idea | `Intent`s raised by `screen::Stack`. Tagging each `Intent` with its origin (pointer, direction key, scripted step) would let a test report what caused a change. This is cheap and helps debugging of controller and pointer paths. |
| Behaviour separated from rendering (`definePrimitive`) | Partly, and kjk already does it | kjk already separates `paint` (layout to `DrawList`) from `raster` (CPU) and the GPU material. The behaviour of a list, a slider or an edit box lives in `screen::Stack` and `screens/`. The principle supports keeping behaviours such as "list with a scroll position" shared across screens. It does not require a new design. |
| Reusable behaviours as a catalogue (roving focus, listbox, typeahead, reorder) | Yes, as a reference list | The design reference the owner wants should list kjk's own shared behaviours (`MOVETO` focus, lists, edit boxes, sliders, the veil) with their rules. The catalogue idea is useful. The UI SDK behaviours themselves are not. |
| A testing kit that checks a pattern's keys and focus (`checkPattern`, `checkDirection`) | Yes, as an idea | `kjk gui probes` and the approved test `gui_layouts` already check layouts. A navigation check that proves every control can be reached by `MOVETO` and the four directions, and that Back always leaves a screen, would give the controller and Steam Deck targets a test. This is the most useful idea for kjk's audit. |
| Docs and a skill shipped with the version, plus an `AGENTS.md` note | Yes | The design reference belongs in the repository as plain Markdown, with a short pointer from the agents' instructions. This matches how UI SDK ships `llms.txt` and `SKILL.md` next to the code, and it keeps the reference in step with the code. |
| Accessible names for every control | Partly | KOTOR layouts name controls by tag (`BTN_SEL3`) and take their words from the talk table. A name per control in the dump is useful for agents. Screen reader support is not a stated kjk target. |
| Redacting sensitive values | No | kjk screens hold no passwords or card numbers. |
| Framework adapters, server rendering, DOM `bind`, ARIA, WebMCP | No | These solve web problems. kjk has no DOM, no framework and no browser. |
| Benchmark (agent through the bridge against agent through browser tools) | Not transferable | It measured a web form. It says nothing about reading a game screen. It does support the general point that structured data is cheaper for an agent than screenshots, but kjk should measure that on its own screens before relying on it. |
| Design tokens, colour, type scale, spacing, motion | Not present | The site documents no tokens or visual style for its own components in the pages read. It cannot supply kjk's tokens. Those have to come from KOTOR's layouts and art, and from game-UI guidance such as controller and TV-distance rules. |

Points the research could not establish:

- The site's list of "behaviours" covers focus, selection, drag and drop, direct manipulation, lists, trees, motion, popups, preferences and canvas (https://ui-sdk.dev/docs). The pages for each were not read in detail, so the exact behaviours' rules are not described here.
- The docs on `agent.snapshot()` were read only through the agents page, which gives no example of the snapshot's output. Its exact shape is therefore unknown.
- No specification or schema document was found. The site's own text is the only description of the formats.

## 4. Recommendation

Borrow specific ideas. Do not adopt the library and do not spend more time on it.

What to do with the ideas, in the order of value:

1. Add a per-screen control dump (control tag, label, state, reachable neighbours, and the `Intent` raised) to the headless render path, so that an agent auditing a screen reads text and a PNG together.
2. Write a navigation check in the style of `checkPattern` for every screen: all controls reachable, Back leaves, and no focus trap. Run it in `kjk gui sweep`. This serves the controller and Steam Deck targets directly.
3. Tag each `Intent` with its origin, so that scripted runs and real input can be told apart in logs and tests.
4. Keep the design reference as repository Markdown that ships with the code, with a short pointer from the agents' instructions, as UI SDK does with `llms.txt`, `SKILL.md` and `AGENTS.md`.

Do not look to UI SDK for design tokens, visual style or screen inventory. For those, a different source is needed.

## Sources

- https://ui-sdk.dev/
- https://ui-sdk.dev/docs
- https://ui-sdk.dev/docs/agents
- https://ui-sdk.dev/docs/agents/skills
- https://ui-sdk.dev/docs/core/changelog
- https://ui-sdk.dev/docs/core/observe
- https://ui-sdk.dev/docs/core/testing
- https://ui-sdk.dev/docs/core/kernel/define-a-primitive
- https://github.com/vercel-labs/ui
