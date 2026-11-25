# Game UI standards for 2026, and how remasters keep a classic's feel

Research for the kjk UI audit. Written on 2026-11-25.

## Summary

This note answers one question. Which UI and UX standards should an audit of kjk's screens use, and how do good remasters modernise a classic RPG's UI without losing its feel?

The short answers are below.

- **Text size.** The Xbox Accessibility Guidelines (XAG) are the most precise public source. They ask for text with a body height of at least 26 px at 1080p on a console and at least 18 px at 1080p on a PC. Body height is measured from the top of the tallest letter to the bottom of the lowest tail, with a 200% scaling option. Valve's Steam Deck Verified rule is looser. It sets 9 px as a hard floor at 1280 × 800 and recommends 12 px. Valve measures "character height", and it does not say whether that means the same thing as XAG's body height. The audit must apply XAG's measure to the actual rendered pixels.
- **Contrast.** Normal text needs 4.5 to 1 against its background, large text and inactive text need 3 to 1, and a high contrast mode needs 7 to 1.
- **TV distance.** Keep essential UI at least 5% of the screen away from every edge (96 px at the sides and 54 px at the top and bottom, at 1080p). Keep the screen simple, so that crossing it takes at most six presses. Avoid hover tooltips.
- **Input.** Every screen must be usable with the mouse, the keyboard and a controller, each on its own. Button glyphs must match the device the player used last, and Steam Deck Verified makes this a pass or fail rule. Focus order must be predictable. Mouse clicks must fire when the button is released, not when it is pressed. Every control must be remappable. A hold must have a tap alternative.
- **Modern RPG patterns.** The sources confirm several patterns that a 2003 RPG lacks. These are item comparison, sorting and filtering, search or a second way to find things in long lists, HUD toggles, subtitle options, and first-launch accessibility options. The first-party evidence for new-item markers, quest tracking and loadouts is thin, so the rubric marks those as recommendations and not as requirements.
- **Remasters.** The teams that kept the feel did four things. They changed as little as possible except what makes the game understandable and playable. They tried the modern version of a signature element, and kept the old one when the modern one stopped feeling like the game (Diablo II's inventory grid). They shipped the classic and the modern look side by side when fans disagreed (Age of Empires III's HUD layouts, Final Fantasy Pixel Remaster's fonts). They made every added helper optional (Diablo II's miss indicator).
- **The rubric.** Section 5 proposes 89 numbered checks. About 45 of them can be measured automatically from a screen's control tree, plus a rendered image for contrast. The rest need a person or a judge.

### What this research could not confirm

Several sources named in the brief were blocked or gave no UI detail. This is stated plainly so that nobody builds on a claim the research did not establish.

- No public interview or talk on the Knights of the Old Republic remake's UI exists. Only development-history news came up.
- No first-party Larian post about Baldur's Gate 3's inventory or controller UI was reachable. The Baldur's Gate 3 facts below come from search summaries of patch notes, and they are marked as secondary.
- The Mass Effect Legendary Edition, System Shock remake, and Diablo II Resurrected press articles from PC Gamer were blocked. Where a quote comes only from a search summary, the text says so.
- The Pillars of Eternity II sources found say nothing specific about its UI.
- PlayStation's public accessibility page gives no developer rules and no text sizes. Sony's technical requirements are not public.
- The Apple page gives sizes in points. It does not say how points map to pixels on a TV.

## 1. Accessibility and readability guidelines

### 1.1 Which documents count

| Source | What it is | Use in the audit |
|---|---|---|
| [Xbox Accessibility Guidelines, overview](https://learn.microsoft.com/en-us/gaming/accessibility/guidelines) | Version 3.2, published 8 June 2023, last updated by Microsoft on 14 November 2026. Microsoft says it is a set of best practices and not a compliance checklist. | Main source for numbers. |
| [Game Accessibility Guidelines, full list](https://gameaccessibilityguidelines.com/full-list/) | A community list sorted into basic, intermediate and advanced levels. | Source for rules without numbers, and for priority levels. |
| [AbleGamers, Accessible Player Experiences](https://accessible.games/accessible-player-experiences/) | 22 patterns, 12 for access and 10 for challenge. The page lists names and gives little detail. | Source for the names of patterns. |
| [Apple, Designing for games](https://developer.apple.com/design/human-interface-guidelines/designing-for-games) and [Game controls](https://developer.apple.com/design/human-interface-guidelines/game-controls) | Apple's human interface guidelines for games. | Source for TV-sized text and button sizes, and for controller button conventions. |
| [Valve, Steam Deck Verified requirements](https://partner.steamgames.com/doc/steamdeck/compat) and [recommendations](https://partner.steamgames.com/doc/steamdeck/recommendations) | The conditions for the Verified badge. | Source for the handheld target. |
| [Microsoft, Designing for Xbox and TV](https://learn.microsoft.com/en-us/previous-versions/windows/uwp/xbox-apps/designing-for-tv) | An archived page on the 10-foot experience. | Source for TV safe areas and TV distance rules. |
| [W3C, WCAG 2.2](https://www.w3.org/TR/WCAG22/) | The web standard. It is not written for games, but XAG copies several numbers from it. | Source for target size and focus appearance. |
| [Sony, PlayStation accessibility](https://www.playstation.com/en-us/accessibility/) | Player-facing page only. | It shows what the console already offers (text size, screen reader, controller remapping), so a game must not fight those. |

### 1.2 Text size

XAG 101 measures text by its **body height**. That is the number of pixels from the top of the highest ascender (the stem of an "h") to the bottom of the lowest descender (the tail of a "g"). A pixel only counts if its contrast against the background is above 2 to 1, so a dark outline does not add to the height. The source is [XAG 101](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/101).

The default minimum sizes in XAG 101 are these.

- Console: at least 26 px at 1080p and 52 px at 4K.
- PC and VR: at least 18 px at 1080p and 36 px at 4K.
- Mobile and game streaming: 18 px at 100 DPI, 36 px at 200 DPI and 72 px at 400 DPI.

XAG 101 says these are minimums for the default size at first launch. It also says that players must be able to scale text up to 200% of the minimum, which means 52 px on a console at 1080p. It says the text must scale without losing content, without needing to scroll in two directions, and with headers still visibly larger than body text.

Because the figures scale linearly with resolution, the table below derives the sizes for the resolutions kjk targets. The 720p, 800p and 1440p columns are my arithmetic from the 1080p figure. Microsoft does not publish them.

| Target | 720p | 800p (Steam Deck) | 1080p | 1440p | 4K |
|---|---|---|---|---|---|
| XAG console minimum body height (TV distance) | 17 px | 19 px | 26 px | 35 px | 52 px |
| XAG PC minimum body height (desk distance) | 12 px | 13 px | 18 px | 24 px | 36 px |
| XAG console text at the 200% setting | 35 px | 39 px | 52 px | 69 px | 104 px |

The other public numbers are these. They use different measures, so they must not be compared directly with the table above.

- **Steam Deck.** [Valve](https://partner.steamgames.com/doc/steamdeck/compat) says interface text must be easily readable at 12 inches (30 cm). It says "the smallest on-screen font character should never fall below 9 pixels in height at 1280x800", and recommends aiming for 12 px. Valve recommends 1280 × 800 as the native resolution and 1280 × 720 as acceptable.
- **Apple.** [Apple](https://developer.apple.com/design/human-interface-guidelines/designing-for-games) lists the default and minimum text sizes in points. On tvOS the default is 29 pt and the minimum is 23 pt. On macOS they are 13 pt and 10 pt. On iOS they are 17 pt and 11 pt. Apple does not say how many pixels one point is on a TV.
- **Microsoft's 10-foot page.** [Microsoft](https://learn.microsoft.com/en-us/previous-versions/windows/uwp/xbox-apps/designing-for-tv) says main text and reading content should be at least 15 effective pixels, and non-critical text at least 12 effective pixels. On Xbox the default scale is 200%, so that is 30 px and 24 px at 1080p. That page counts font size and not body height.
- **Game Accessibility Guidelines.** [The guideline on font size](https://gameaccessibilityguidelines.com/allow-the-font-size-to-be-adjusted/) gives no number. It says no single size suits everyone, and so the player must be able to change it.

My reading for the audit is as follows. TV distance should use the XAG console figure of 26 px body height at 1080p as the target, which is 19 px at 800p. The Steam Deck figure of 12 px is the lowest value Valve recommends, and XAG's PC figure at 800p (13 px) is close to it. Any text under 9 px at 800p fails Steam Deck Verified. The audit needs one agreed way to convert kjk's bitmap font pixel heights into body height, which Section 6 covers.

### 1.3 Spacing, line length and case

XAG 101 gives these defaults for any block of text longer than two lines. The player may be offered controls for them instead.

- A line is at most 80 characters, or 40 for Chinese, Japanese and Korean. Spaces are not counted.
- Line spacing is at least 1.5 times the font size.
- Paragraph spacing is at least twice the line spacing.
- Letter spacing is at least 0.12 times the font size, and word spacing at least 0.16 times.
- A sentence must be available in sentence case and not only in capitals.
- Text is left or right aligned and not centred or justified.
- At least one sans serif font option exists. If the game uses a decorative font, a plain font option must exist as well.

The same page says the glyph inside a button icon must meet the same minimum size as text, and that icons must scale with text up to 200%.

[XAG 104](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/104) adds rules for subtitles. Subtitles must meet the text minimum by default. They must scale to at least 200% of it. Lines must be at most 40 characters, with at most two lines on screen (three in exceptional cases). Text must be in mixed case. The player must be able to set a solid background behind the subtitle and an opacity from 0% to 100%. The speaker must be named when the speaker changes. Subtitles must be on by default or settable before any speech plays. The player must be able to preview the result.

### 1.4 Contrast

[XAG 102](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/102) gives these contrast ratios.

| Element | Minimum ratio against its background |
|---|---|
| Standard-size text and important visual elements | 4.5 to 1 |
| Large text and large visual elements | 3 to 1 |
| Text on an inactive element (locked, disabled) | 3 to 1 |
| Placeholder text and typed text in an input field | 4.5 to 1 (3 to 1 if large) |
| Every element in a high contrast mode | 7 to 1 |

Large text means 52 px at 1080p on a console and 36 px at 1080p on a PC. When text sits on a non-solid background, the ratio is measured against the lowest contrasting part of that background. Text that is part of a picture does not count as text, so XAG 102 says images must not contain text, apart from logos. XAG 102 also says the game should read the platform's high contrast setting when it exists, and that colour alone must not carry information. [WCAG 2.2](https://www.w3.org/TR/WCAG22/) uses the same 4.5, 3 and 3 to 1 figures for text, large text and user interface parts, and asks that focused and unfocused states differ by at least 3 to 1 (criterion 2.4.13, level AAA).

### 1.5 Targets, buttons and the TV safe area

- **Apple** sets a default of 66 × 66 pt and a minimum of 56 × 56 pt for buttons on tvOS, and 28 × 28 pt default and 20 × 20 pt minimum on macOS. The source is [Designing for games](https://developer.apple.com/design/human-interface-guidelines/designing-for-games).
- **Microsoft's 10-foot page** says interactive elements must be at least 32 effective pixels high, which is 64 px at 1080p on Xbox. The same page says the screen should be as simple as a phone screen, and that crossing the screen edge to edge should take at most six presses.
- **WCAG 2.2** asks for pointer targets of at least 24 × 24 CSS pixels, or enough spacing so that a 24 px circle around each target does not touch another. The source is [the Understanding page for 2.5.8](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html).
- **TV safe area.** The same Microsoft page says to draw only non-essential visuals within 5% of the screen edges. At Xbox's 960 × 540 effective size, that is 48 effective pixels at the sides and 27 at the top and bottom. At 1080p that is 96 px and 54 px. Panel backgrounds may reach the edge, but text and interactive elements must stay inside.
- **Tooltips.** The same page says to avoid hover-style tooltips on TV, because on a controller a tooltip appears when an item gets focus and then disappears. This does not mean description text is forbidden. A description panel that stays on screen, as KOTOR already has, is the safer choice.

The TV rules do not apply on a handheld, because the Deck's screen has no overscan. They matter for a 1080p or 4K TV. The audit should therefore test the safe area only when the output is a TV.

### 1.6 What the guidelines say about settings and menus

[XAG 112](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/112) and [XAG 114](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/114) cover menus. Both are testable by hand, and some by machine.

- The path to the accessibility settings must be reachable from the first launch. Microsoft's best example asks about accessibility on the first screen.
- Focus order must be logical and consistent across the whole game, and it must follow the visual layout. If a layout changes after scaling, the focus order must update.
- A linear menu (one column or one row) must loop from the last item to the first. A grid must not loop. Microsoft encourages an option to switch looping off.
- The interactions "select", "back" and "change tab" must use the same buttons on every screen, and the prompt bar must stay in the same place with the same relative order.
- Long sets of content, such as quest logs and large inventories, need more than one way to find something, such as filter tabs plus a search field.
- A map must have a text list as an alternative to scrolling the map.
- Every submenu needs a persistent way back to the main menu or the previous screen.
- If focus can move onto an element with the mouse or keyboard, focus must be able to move off it with the same input.
- UI context must not change unless the player started the change, or else a notification must be shown.
- A preview of an option's effect should be shown, ideally in a real game scene. Microsoft's example is Immortals Fenyx Rising, where each HUD preset shows a screenshot.
- Interface text should need no more than a lower secondary school reading level. Story text is exempt.
- Labels must sit next to the control they describe, or inside the same focus frame.

## 2. Input

### 2.1 Controller and mouse parity

[XAG 107](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/107) and [XAG 112](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/112) set the following rules, which apply to kjk's three input types.

- The whole UI must work with digital input alone (keyboard keys or D-pad). The mouse and analog sticks are allowed as extras but must not be required.
- The player must be able to start the game, change settings, play, and quit using the keyboard alone, and also using the controller alone.
- A single pointer action must fire on the up event (the release of the button), not the down event. Moving the pointer away before release must cancel it. If it cannot be cancelled, an undo must exist.
- No control may require two buttons at once, a drag, or a multi-finger gesture, unless a simpler alternative exists.
- A long press must last less than 3 seconds. Microsoft also says to avoid holds above 2 to 3 seconds in general.
- Analog sensitivity must be adjustable, by at least 50% of the default in either direction.

[Valve's Steam Deck Verified rules](https://partner.steamgames.com/doc/steamdeck/compat) add the following. The Deck's default controller layout must reach all content without the player changing any setting. If the game needs text entry, it must open the Steam on-screen keyboard or provide its own controller-only entry. If the game uses Steam's own input layer, text input works only when the game is launched through Big Picture mode, according to [Valve's best practices](https://partner.steamgames.com/doc/features/steam_controller/steam_input_gamepad_emulation_bestpractices). A launcher, if any, must also be controller navigable.

### 2.2 Glyphs that follow the device

Three sources agree on this, and two give exact wording.

- **Valve.** "On-screen glyphs must match the inputs being used, whether it's Deck, Steam Controller, or Xbox glyphs." Mouse and keyboard glyphs "should not be shown if they are not the active input." Interacting with a Deck input must not show a non-controller glyph. The source is [the Steam Deck Verified page](https://partner.steamgames.com/doc/steamdeck/compat). The recommendations page also warns against prompts flipping between controller and keyboard icons during mixed input, but it gives no rule for how to avoid it.
- **Apple.** When several controllers are connected, "use labels and glyphs that match the one that the player is actively using." Apple also says to prefer symbols to text for controller buttons, and to use the connected controller's own labelling scheme. The source is [Game controls](https://developer.apple.com/design/human-interface-guidelines/game-controls).
- **Microsoft.** When a player remaps a control, the new mapping must show correctly in every hint, tip and tutorial. The source is [XAG 107](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/107).

Valve adds that unknown controllers should fall back to a generic glyph set, and that a game should prefer Steam's glyph art because it updates without a game patch. This is useful for kjk only if kjk uses Steam Input. [Valve's best practices](https://partner.steamgames.com/doc/features/steam_controller/steam_input_gamepad_emulation_bestpractices) describe this.

A game developer's article on multi-platform UI says that button prompts should be specific to the current platform, and that prompts for the primary action should sit where players can cross-reference them, such as the foot of the screen. The source is [Secretly console first](https://www.gamedeveloper.com/design/secretly-console-first-a-better-approach-to-multi-platform-game-ui-design). That article also says a UI that is legible at 10 feet is legible up close, "every time". This is one author's opinion, and it is consistent with the XAG figures.

### 2.3 Focus navigation

- Apple's UI conventions for a controller, from [Game controls](https://developer.apple.com/design/human-interface-guidelines/game-controls), are these. A activates. B cancels or goes back. The left and right shoulder buttons move to the previous and next screen or section. The thumbsticks and D-pad move the selection. Menu opens settings or pauses. Home is reserved for the system.
- Microsoft's own examples use A to select, B to go back, the bumpers (LB and RB) to change tabs, and the triggers (LT and RT) to change pages. See [XAG 112](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/112).
- Microsoft's 10-foot page says the focus visual must be a border that makes the focused element unmistakable. It says nested interactive items inside a list row do not work well with up, down, left and right navigation, and that the usual fix is to move them into a context menu. It also says to open a navigation pane from the controller's View button or by pushing left from the leftmost item. The source is [Designing for Xbox and TV](https://learn.microsoft.com/en-us/previous-versions/windows/uwp/xbox-apps/designing-for-tv).
- The Titan Quest remaster team said that adding gamepad input was easy next to the work of "changing all the bits and pieces of the navigation system". They also had to add a target cone for aiming without a cursor, a loot window that lists everything under the cursor, and radial menus in place of keyboard shortcuts. The source is [the Xbox Wire article](https://news.xbox.com/en-us/2018/03/21/titan-quest-remastering-pc-classic-xbox-one/). It matters for kjk, because the original KOTOR menus were also built for a mouse and keyboard.

### 2.4 Hold, toggle and remapping

- [XAG 107](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/107) asks that players can remap all controls inside the game, including analog sticks and the Escape key, and that the game ideally lets any action go to any input and not only swap two buttons. It prefers remapping actions over remapping individual inputs, so that a two-button chord can become one button. It asks for inversion on each axis separately.
- Microsoft's example of a game that converts holds to presses is The Long Dark's "accessible interactions" setting.
- Dragon Age: The Veilguard's options are the clearest recent first-party example. They include "Disable UI hold inputs" (hold or tap), "Ability wheel activation" (hold or tap), full input remapping on controller and keyboard, separate stick and trigger deadzone sliders, and separate sensitivity sliders. The source is [EA's accessibility spotlight](https://www.ea.com/games/dragon-age/dragon-age-the-veilguard/news/accessibility-spotlight) and [its resource page](https://www.ea.com/able/resources/dragon-age-the-veilguard). The resource page also notes that movement can be remapped only between the two analog sticks, so even a good implementation has gaps.
- [Game Accessibility Guidelines](https://gameaccessibilityguidelines.com/full-list/) lists "make all UI areas accessible with the same input method as gameplay" and "support more than one input device" among its motor guidelines. It also lists "include a 0.5-second cool-down between inputs" as an advanced item.

## 3. Modern RPG UX patterns

This section lists patterns that a 2003 RPG lacks. Each says how well the sources back it.

| Pattern | What a first-party or standards source says | How strong the evidence is |
|---|---|---|
| Item comparison | Diablo II Resurrected has an item compare that shows the stat difference between an equipped piece and a new one. This is reported in press coverage of the remaster, found through search but not opened. Diablo IV has an "Advanced Tooltip Compare" setting. | Press only. No Blizzard primary page was reachable. |
| Sorting | Baldur's Gate 3's patch 8 notes, as summarised by news sites, fixed a "sort by latest" option that showed the oldest items first. The Mass Effect Legendary Edition coverage lists sorting for inventory and stores. | Search summaries only. |
| Filtering and a second way to find things | XAG 112 asks for more than one way to find content in large sets, and gives Minecraft's filter tabs plus a search field as its example. Baldur's Gate 3 patch 2 added rarity filters, according to news summaries. | Strong from Microsoft. The Larian claim is secondary. |
| Junk marking and bulk selling | Mass Effect Legendary Edition lets the player flag items as junk, then convert or sell them together, per the PC Gamer coverage seen in search. | Press summary only. |
| Item descriptions and glossary | XAG 114 asks for definitions of game terms, and gives Minecraft's "how to play" index as the example. Veilguard has a Codex and Glossary. | Strong. |
| Tooltips and context help | XAG 114 asks for context-sensitive help for each element, such as the "Edit Name" help line in Forza. On TV, hover tooltips should be avoided. | Strong, with the TV caveat. |
| Quest and objective tracking | Veilguard lets the player set objective marker visibility to Off, Pulse (Short), Pulse (Long) or Always, and hide the objective tracker. Game Accessibility Guidelines asks that players can be reminded of current objectives and controls during play. | Medium. |
| Confirmation and undo | XAG 107 asks that a single-pointer action can be cancelled before completion, or that a simple undo exists. The Accessible Player Experiences list includes the patterns "Undo Redo" and "Save Early, Save Often". | Strong for the rule. No source gives a standard for a RPG inventory undo. |
| Loadouts | No standards source covers loadouts. This is a common genre pattern, but the research found no first-party document that states it as a requirement. | Not established. |
| Accessibility options menu | See section 1.6. Microsoft recommends offering these at first launch, and the Game Accessibility Guidelines basic level asks for "ensure all settings are saved and remembered". | Strong. |
| HUD toggles | Veilguard allows hiding Abilities, Damage Numbers, Hints, Mini Map, Objective Tracker, Player Health and Tutorials. Diablo II Resurrected allows hiding some HUD elements, item names and the minimap. | Medium. |
| Settings preview | XAG 114 and XAG 104 ask for a preview of text, subtitle and HUD options, ideally in a real scene. | Strong. |
| Minimal and full subtitles | The Outer Worlds offers three conversation subtitle modes (on, minimal, off), with a separate toggle for barks and cinematics. See [XAG 104](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/104). | Strong. |

A note on Baldur's Gate 3. A "Controller Style" option, which lets the player override which controller's icons show, is in its patch 1 notes according to a news summary found through search. It is a useful model for kjk's glyph rule, because it lets the player correct a wrong guess.

The search also found no clear first-party material on new-item markers. That does not mean the pattern is wrong. It means this note cannot cite a standard for it, and the audit should treat it as a design recommendation.

## 4. Remaster case studies

Each case states what the source says, then what the team kept and what it changed. I mark my own inferences as such.

### 4.1 Diablo II Resurrected

- **Rule used.** The GDC 2022 talk by Drew McCrory, as reported by Game Developer, says the team settled on "making as little modernizations as possible except those that make the game more understandable and playable". A proposed separate charm bag was rejected because it did not fit the legacy or improve accessibility. The GDC session page, by Robert Gallerani of Blizzard, says the talk covers what the team chose not to add. The sources are [the Game Developer article](https://www.gamedeveloper.com/marketing/exploring-modernity-legacy-and-accessibility-in-diablo-ii-resurrected) and [the GDC Vault page](https://gdcvault.com/play/1028091/Remastering-a-Classic-Diablo-II).
- **Added and optional.** The attack-miss text indicator is on by default and can be turned off, because "the purists don't need it". Other accessibility options named are colourblind toggles, larger text, UI scaling, automatic gold pickup, hold-to-toggle, camera shake off, subtitle toggles and audio sliders per sound type. The same article says that on a controller potions can be used straight from a quick action, skipping the inventory.
- **A kept signature.** A search summary of press coverage quotes a Vicarious Visions developer. The team first turned the inventory into a list for the controller, found it "totally usable", and then went back to the grid, because "it wasn't Diablo 2 anymore". I did not open the article. The candidate sources are [PCGamesN](https://www.pcgamesn.com/diablo-2-resurrected/fix-diablo-2) and [Windows Central](https://windowscentral.com/diablo-2-resurrected-developer-interview). Treat the quote as unverified until one of them is read.
- **Inference for kjk.** The grid and the item art stayed. What changed was how the controller moves the cursor over them and which helpers were added around them.

### 4.2 Age of Empires III Definitive Edition

The team's [HUD and UI post](https://www.ageofempires.com/news/aoe3de-hud-and-ui) says three layouts are offered. The "Classic" layout keeps the original Age of Empires III arrangement, with the mini-map at the bottom left. The "Definitive" layout resembles earlier Definitive Editions, with the mini-map at the bottom right. The default layout combines the two. The HUD is less obtrusive, appearing and disappearing as units are selected. Unit and building stat blocks were rebuilt, so that stats are on one panel, and HUD scaling was added to the settings.

The inference is that when the team could not decide between the old and the new arrangement, it gave the player both. It still shipped one better default. The research found no developer post on Age of Empires II Definitive Edition's UI, only a PC Gamer piece that was blocked.

### 4.3 Baldur's Gate Enhanced Edition

Trent Oster's [postmortem](https://www.gamedeveloper.com/programming/postmortem-overhaul-games-i-baldur-s-gate-enhanced-edition-i-) is the best primary source for a classic RPG UI. The team rebuilt the UI at 1024 × 768 and made party portraits as large as possible. It followed Apple's 44 px minimum button height and failed it only in a few places, such as class selection. It reused the original structures and asset formats to "keep the original feel". It rewrote the UI code. For touch, it used a drag for scrolling, a single tap for moving and interacting, and a "smart radius" that makes taps forgiving. The team described itself as curators: "We didn't want to change Baldur's Gate just to show off our development skills."

A search summary also quotes lead designer Phillip Daigle on the expansion Siege of Dragonspear: every new thing was held against the question "Does this feel like Baldur's Gate?" I did not open that article, so this is secondary. It is still a usable review test for kjk (rubric item K-8).

### 4.4 Mass Effect Legendary Edition

- **Stated aim.** [RPG Site](https://www.rpgsite.net/feature/10719-mass-effect-legendary-edition-whats-changed-in-mass-effect-1) quotes the team's aim as "to retain the feeling and the magic of the existing game, but to make small tweaks and changes for quality of life".
- **What changed in the first game.** The HUD was brought closer to the second and third games, a dedicated melee button was added, controller support became native, key bindings improved, 21:9 support was added, and customisation options were matched across the three games. Press coverage also lists junk marking and sorting, but only through search summaries.
- **How the team decided.** Mac Walters said that modders' work gave "an initial great data set" of what players care about. The source is [Game Developer](https://www.gamedeveloper.com/game-platforms/a-decade-of-mods-helped-devs-zero-in-on-improvements-for-i-mass-effect-legendary-edition-i-).
- **Inference.** The UI of the oldest game was moved towards the newest game of the same series, so that the series stayed consistent. This is a strong model for kjk, because kjk already shows KOTOR 1 with KOTOR 2's layouts (the "blue steel" skin).

### 4.5 System Shock remake

A search summary of an Epic Games Store interview quotes Stephen Kick. The team "took the UI and control scheme from System Shock 2" for the remake, and said it needed "a ton of tweaks to make it feel familiar to modern audiences and even more work to get it working on a controller". The same search found a PC Gamer review that says the inventory is easier to use but unpleasant to move through with a cursor on the right stick. I did not open either. The source to read first is [Epic's interview](https://store.epicgames.com/news/system-shock-2-25th-anniversary-remaster-interview-stephen-kick).

The inference is that the team used a later UI from the same series as the base, and that a cursor driven by a stick is the weak point to test.

### 4.6 Final Fantasy Pixel Remaster

Producer Naofumi Takuma [said](https://www.siliconera.com/interview-preparing-final-fantasy-pixel-remaster-for-the-switch-and-ps4/) that "there is inevitably a conflict" between preserving the original work and giving an enjoyable experience to a first-time player. The font was the visible example. Fans disliked the first fonts, so the Switch and PlayStation 4 versions added pixel fonts for Western and Japanese text that the player can swap in the menu, as [GoNintendo's report](https://www.gonintendo.com/contents/21170-square-enix-details-the-considerable-amount-of-work-that-went-into-the-fonts-of) says. Takuma described the options on difficulty in the same way: they keep growth, and are "not an invincibility type of function".

The inference is that a contested signature element can be offered in two forms with a switch, and that the player's choice is saved.

### 4.7 Other cases

- **Pillars of Eternity II.** [Game Developer](https://www.gamedeveloper.com/design/building-i-pillars-of-eternity-ii-i-without-losing-that-infinity-engine-charm) quotes Josh Sawyer on keeping "the overall feeling and atmosphere really true to the spirit of the original games". The article says nothing about UI details, so it supports only the general stance.
- **Titan Quest.** See section 2.3. The team treated the move from a monitor to a "37 to 43 inch" TV as a UI problem and not only as a porting problem.
- **Knights of the Old Republic remake.** No public UI material exists.

### 4.8 Principles drawn from the cases

These are my synthesis. Each one has at least one source above.

1. Keep the identity-carrying surfaces. These are the layout shape (the grid, the HUD arrangement), the visual skin, the sound and the signature font. Sources: Diablo II Resurrected, Age of Empires III, Baldur's Gate Enhanced Edition.
2. Change how the player gets around, not how the screen looks. Navigation, controller focus, glyphs, scaling, sorting and filtering are where all cases made their changes. Sources: Titan Quest, Diablo II Resurrected, Mass Effect Legendary Edition.
3. Make every new helper optional or removable, so that purists can turn it off. Source: the Diablo II Resurrected miss indicator.
4. When fans split on a signature element, ship both forms with a switch. Sources: Age of Empires III's three layouts, and Final Fantasy Pixel Remaster's fonts.
5. Prototype the modern form, look at it, and be ready to go back. Source: the Diablo II Resurrected inventory (unverified quote).
6. Borrow from a later UI of the same family, when one exists. Sources: Mass Effect and System Shock.
7. Give each change a test question. "Does this feel like Baldur's Gate?" is the clearest. Source: Daigle (secondary).
8. Use the community's own improvements as input. Sources: Mass Effect's use of mods, and Beamdog's attention to the Infinity UI mod (secondary).

## 5. Proposed audit rubric for kjk

### How to read this rubric

Each item has an ID, a rule, a source and a check type. The check type is one of these.

- **Auto** means a script can decide pass or fail from a screen's control tree (positions, sizes, text, colours, focus links). Where it also needs a rendered image, the table says "Auto, from render".
- **Human** means a person must look at the screen or use it.
- **Judge** means a model or a reviewer judges a picture against a stated question. A judge is not a measure, and its verdict is advice.

Each item should be checked at three outputs. These are 1280 × 800 handheld, 1920 × 1080 on a TV, and 3840 × 2160 on a TV. The number thresholds below use the 1080p figure unless a value for 800p is given. A threshold may be scaled linearly for the other resolutions, as in the table in section 1.2.

Source links carry full addresses. Where the same source repeats, the first row shows the link and the later rows name it by its XAG number.

### 5.1 Text (T)

| ID | Rule | Source | Check |
|---|---|---|---|
| T-1 | On a TV output, every piece of body text has a body height of at least 26 px at 1080p (19 px at 800p, 52 px at 4K). | [XAG 101](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/101) | Auto, once the font body height is converted (see section 6). |
| T-2 | On a handheld output at 1280 × 800, no text has a character height under 9 px, and the target for body text is 12 px. | [Steam Deck Verified](https://partner.steamgames.com/doc/steamdeck/compat) | Auto. |
| T-3 | The player can scale text to 200% of the default, and no content is lost, cut off, or needs scrolling in both directions. | XAG 101 | Auto for overflow and clipping at 200%. Human for loss of meaning. |
| T-4 | A large heading stays visibly larger than body text after scaling. | XAG 101 | Auto, by comparing text sizes. |
| T-5 | Icons and button glyphs have the same minimum size as text, and the glyph fits inside its text line. | XAG 101 | Auto. |
| T-6 | A block of more than two lines has at most 80 characters per line, not counting spaces. | XAG 101 | Auto. |
| T-7 | A block of more than two lines has line spacing of at least 1.5 times the font size, or the player can change it. | XAG 101 | Auto, with an exception list for the original KOTOR fonts. |
| T-8 | A text option exists that is not all capitals, and at least one plain sans serif font is selectable. | XAG 101 | Auto for the font list. Human for the case. |
| T-9 | Lines of text are left aligned in a left-to-right language, and not centred or justified, unless the line is a one- or two-word label. | XAG 101 | Auto. |
| T-10 | Interface text reads at a lower secondary school level. Story text is exempt. | [XAG 114](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/114) | Judge. |
| T-11 | No picture that carries information contains text, apart from logos. | [XAG 102](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/102) | Auto for baked art listed in the asset table. Human for the rest. |
| T-12 | Subtitles are mixed case, at most 40 characters per line, at most two lines, scale to 200%, have a background with 0% to 100% opacity, and name the speaker when it changes. | [XAG 104](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/104) | Auto for the first five. Human for the speaker name. |

### 5.2 Contrast and colour (C)

| ID | Rule | Source | Check |
|---|---|---|---|
| C-1 | Standard text has a contrast of at least 4.5 to 1 against the lowest contrasting part of its background. | [XAG 102](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/102) | Auto, from render. |
| C-2 | Large text (52 px at 1080p), and text on an inactive element, has at least 3 to 1. | XAG 102 | Auto, from render. |
| C-3 | Placeholder and typed text in an input field has at least 4.5 to 1. | XAG 102 | Auto, from render. |
| C-4 | Icons, borders, sliders and other parts that identify a control have at least 3 to 1 against what is next to them. | [WCAG 2.2, 1.4.11](https://www.w3.org/TR/WCAG22/) | Auto, from render. |
| C-5 | A high contrast mode exists, and in it every UI element has at least 7 to 1. | XAG 102 | Auto, from render, once the mode exists. |
| C-6 | No information is carried by colour alone. A selected, disabled, locked or good-versus-bad state has a second signal such as a shape, a label or an icon. | XAG 102 | Judge, with Auto support by simulating colour blindness and comparing states. |
| C-7 | The player can add a solid background behind on-screen text and set its opacity. | XAG 102 | Auto for the setting. |

### 5.3 Layout, safe area and size (L)

| ID | Rule | Source | Check |
|---|---|---|---|
| L-1 | On a TV output, no text or control lies within 5% of any screen edge (96 px at the sides and 54 px at the top and bottom at 1080p). Panel backgrounds may reach the edge. | [Microsoft, Designing for Xbox and TV](https://learn.microsoft.com/en-us/previous-versions/windows/uwp/xbox-apps/designing-for-tv) | Auto. |
| L-2 | Every control that can be activated has a height of at least 64 px at 1080p on a TV output. | Same page (32 effective pixels at 200%). Apple's [56 pt minimum on tvOS](https://developer.apple.com/design/human-interface-guidelines/designing-for-games) agrees in order of size. | Auto. Expect conflicts with KOTOR's small buttons. |
| L-3 | On a mouse output, every pointer target is at least 24 × 24 px, or has 24 px of clear space around it. | [WCAG 2.2, 2.5.8](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html) | Auto. |
| L-4 | Neighbouring controls do not overlap, and a list or grid does not run off a panel. | [Apple](https://developer.apple.com/design/human-interface-guidelines/designing-for-games) | Auto. |
| L-5 | The screen has no more elements than a phone screen would, and the farthest focus path across it takes at most six presses. | Microsoft, Designing for Xbox and TV | Auto for the six-press path. Judge for density. |
| L-6 | At aspect ratios of 16:10, 16:9, 21:9 and 4:3, nothing is clipped, overlapped or left unreachable. | Apple, Designing for games | Auto at each ratio. |
| L-7 | The same kind of element sits in the same place on every screen. Examples are the title, the prompt bar, and the back control. | [XAG 112](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/112) | Auto for prompt-bar position. Human for the rest. |
| L-8 | A label sits next to its control, or is inside the same focus frame. | [XAG 114](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/114) | Auto, by distance. |
| L-9 | Large blocks of text have headings. | XAG 114 | Judge. |

### 5.4 Focus and navigation (N)

| ID | Rule | Source | Check |
|---|---|---|---|
| N-1 | Every control that can be activated can be reached by moving focus from the first focused control, with digital input alone. | [XAG 112](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/112) | Auto, as a graph search over focus links. |
| N-2 | There is no focus trap. From every control, focus can move away using the same input that moved it there. | XAG 112 | Auto. |
| N-3 | A direction press moves focus to the control that lies in that direction on screen. If the link goes elsewhere, a test flags it for a human to confirm it was intended. | XAG 112 | Auto for the geometric check. Human for exceptions. |
| N-4 | If control A links right to control B, control B links left to control A, unless the screen documents an exception. | XAG 112 | Auto. |
| N-5 | A single-column or single-row list loops from last to first. A grid does not. | XAG 112 | Auto. |
| N-6 | The focused control is visibly different from every unfocused control. The difference is at least 3 to 1 in contrast, and covers at least a 2 px border around the control. | [WCAG 2.2, 2.4.13](https://www.w3.org/TR/WCAG22/) and the Microsoft TV page | Auto, from render. |
| N-7 | The focused control is not hidden behind another element, and a list scrolls it into view. | [WCAG 2.2, 2.4.11](https://www.w3.org/TR/WCAG22/) | Auto. |
| N-8 | After a resize or a text scale change, the focus order matches the new layout. | XAG 112 | Auto. |
| N-9 | Every screen has a way back (the back button or Escape) that leads to the same kind of place every time. | XAG 112 | Auto. |
| N-10 | A tab bar can be switched with the left and right shoulder buttons on every screen that has one. | [Apple, Game controls](https://developer.apple.com/design/human-interface-guidelines/game-controls) and XAG 112 | Auto. |
| N-11 | Nested controls inside a list row do not need a separate focus step. They are moved to a context menu. | Microsoft, Designing for Xbox and TV | Auto for nested focusable controls. |
| N-12 | A long list or inventory has at least one second way to find an item, such as a filter, a sort or a search. | XAG 112 | Human, with a Judge on fit. |
| N-13 | A map or a galaxy map has a list alternative that does not need cursor scrolling. | XAG 112 | Human. |
| N-14 | Every screen is reachable from the main menu or the in-game menu, and no screen opens by surprise without a notification. | XAG 112 and XAG 114 | Human. |

### 5.5 Input and glyphs (I)

| ID | Rule | Source | Check |
|---|---|---|---|
| I-1 | Every screen is fully usable with a controller alone. | [XAG 107](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/107) and [Steam Deck Verified](https://partner.steamgames.com/doc/steamdeck/compat) | Auto for reachability (N-1). Human for the rest, including text entry. |
| I-2 | Every screen is fully usable with the keyboard alone. | XAG 107 | Auto for reachability. Human for the rest. |
| I-3 | Every screen is fully usable with the mouse alone, including a visible way back. | [Apple, Game controls](https://developer.apple.com/design/human-interface-guidelines/game-controls) | Human. |
| I-4 | Every prompt that shows a button shows the glyph of the device the player used last. A keyboard or mouse glyph is never shown after a controller press. | Steam Deck Verified, Apple Game controls | Auto, by switching the device in a test and scanning the prompt text. |
| I-5 | The glyph set matches the controller type (Xbox, PlayStation, Steam Deck, Switch), or the player can choose it. | Apple Game controls, plus the Baldur's Gate 3 "Controller Style" option (secondary) | Auto for the setting. Human for correctness. |
| I-6 | After a remap, every prompt, hint and tutorial shows the new binding. | XAG 107 | Auto. |
| I-7 | The prompt bar lists the actions that exist on this screen with this device, and only those. | XAG 112 | Auto. |
| I-8 | Text entry opens an on-screen keyboard or a controller-only entry method. | Steam Deck Verified | Human. |
| I-9 | A button or click fires on release, and moving the pointer away before release cancels it. | XAG 107 | Auto for event handling. Human for feel. |
| I-10 | No action needs two simultaneous buttons, a drag, or a hold above 3 seconds without an alternative. | XAG 107 | Auto from the action table. |
| I-11 | Every hold input has a tap alternative setting. | XAG 107, [EA Veilguard](https://www.ea.com/able/resources/dragon-age-the-veilguard) | Auto for the setting. |
| I-12 | All controls can be remapped inside the game on keyboard and controller, including Escape and stick clicks. | XAG 107 | Auto from the binding table. |
| I-13 | Stick and trigger deadzones and sensitivity can be adjusted, by at least 50% either way. | XAG 107 | Auto. |
| I-14 | Controller buttons in UI follow Apple's conventions. A activates, B goes back, shoulders change tab or section, and Menu opens settings. | Apple Game controls | Auto from the input map. |
| I-15 | Switching between devices mid-session works at once, without a restart, and does not lose focus. | Steam Deck recommendations | Human. |

### 5.6 Feedback, confirmation and safety (F)

| ID | Rule | Source | Check |
|---|---|---|---|
| F-1 | Every destructive action (sell, destroy, overwrite a save, delete a save) asks for confirmation and names what is affected. | [XAG 107](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/107), [APX "Undo Redo"](https://accessible.games/accessible-player-experiences/) | Auto for the presence of a dialog. Human for wording. |
| F-2 | Where a confirmation would be too slow, an undo exists. | Same sources. | Human. |
| F-3 | The default button in a confirmation dialog is the safe one. | Not in the sources. My recommendation. | Auto. |
| F-4 | A control that is disabled or locked says why. | [XAG 114](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/114) | Auto for a reason string. |
| F-5 | Every action that changes state has visible feedback within one frame or a short, named delay. | [Game Accessibility Guidelines](https://gameaccessibilityguidelines.com/full-list/) (make clear that interactive elements are interactive) | Human. |
| F-6 | Settings are saved and remembered. | Game Accessibility Guidelines (basic level) | Auto. |
| F-7 | A setting with a visual effect has a preview. | XAG 114 | Auto for the presence of a preview. |

### 5.7 Modern RPG patterns (R)

| ID | Rule | Source | Check |
|---|---|---|---|
| R-1 | Selecting an item in a store or inventory shows the stat change against what is equipped in the same slot, for the selected character. | Diablo II Resurrected coverage (secondary). No primary source reached. | Human. Auto for the presence of a comparison row. |
| R-2 | An item list can be sorted. At least by name, type and recency. | Baldur's Gate 3 and Mass Effect coverage (secondary). | Auto for the presence of a sort control. |
| R-3 | An item list can be filtered by type. | XAG 112 | Auto for the presence of a filter control. |
| R-4 | A long list has a search field or quick jump. | XAG 112 | Auto. |
| R-5 | Items that are new since the player last looked are marked. | Design recommendation, no standard cited. | Human. |
| R-6 | The current objective can be seen, reminded and hidden. | Game Accessibility Guidelines, [EA Veilguard](https://www.ea.com/games/dragon-age/dragon-age-the-veilguard/news/accessibility-spotlight) | Human. |
| R-7 | A game term has a definition within reach. | XAG 114 | Human. |
| R-8 | Item or power descriptions show requirements the character does not meet in a way that is not colour only. | XAG 102 | Judge. |
| R-9 | Marking junk and bulk selling exist in stores. | Mass Effect coverage (secondary). | Human. |
| R-10 | A loadout (saved equipment sets) exists. | No source. My recommendation only. | Human. |

### 5.8 Options and onboarding (O)

| ID | Rule | Source | Check |
|---|---|---|---|
| O-1 | Accessibility options can be reached from the first screen at first launch, and are on a screen of their own. | [XAG 112](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/112) | Human. |
| O-2 | The game's defaults meet the numbers in T-1, T-2 and C-1 without the player changing anything. | XAG 112 | Auto, by running the checks above on the default settings. |
| O-3 | The game reads the platform's text size, high contrast and caption settings where they exist. | XAG 102, XAG 104 | Human. |
| O-4 | HUD parts can be hidden one by one. | EA Veilguard, Diablo II Resurrected coverage | Auto for the setting. |
| O-5 | Each option has a one-line description in plain words, and values such as "minimal" are explained. | XAG 104, XAG 114 | Auto for a description string. Judge for wording. |
| O-6 | Camera shake, flashing effects and motion can be reduced. | [Game Accessibility Guidelines](https://gameaccessibilityguidelines.com/full-list/) | Auto for the setting. |
| O-7 | A short way to restore each option page to its defaults exists. | Not in the sources. My recommendation. | Auto. |

### 5.9 Feel of KOTOR (K)

The sources do not give numbers for feel. These items turn the remaster principles in section 4.8 into questions that a person or a judge can answer.

| ID | Rule | Source | Check |
|---|---|---|---|
| K-1 | The screen keeps its KOTOR layout shape, where each region sits and how big it is relative to the others, unless a rubric item above forces a change. | Principle 1 (section 4.8) | Judge, shown the original and the new screen next to each other. |
| K-2 | The screen keeps KOTOR's frame art, its colours and its sounds. | Principle 1 | Judge. |
| K-3 | The signature font stays the default, or a choice is offered with a plain font next to it. | [Final Fantasy Pixel Remaster](https://www.siliconera.com/interview-preparing-final-fantasy-pixel-remaster-for-the-switch-and-ps4/) | Human. |
| K-4 | Where a rule above needs a change that the owner may dislike, both forms are offered with a switch. | [Age of Empires III Definitive Edition](https://www.ageofempires.com/news/aoe3de-hud-and-ui) | Human. |
| K-5 | Every new helper can be switched off. | [Diablo II Resurrected](https://www.gamedeveloper.com/marketing/exploring-modernity-legacy-and-accessibility-in-diablo-ii-resurrected) | Auto for the setting. |
| K-6 | Every change on the screen has a written reason that names a rubric item. | Principle 2 | Human. |
| K-7 | KOTOR 1's screens in kjk look like KOTOR 1's and not like KOTOR 2's, and the same for KOTOR 2. | Existing kjk skin rule in docs/UI.md | Judge. |
| K-8 | For each change, a reviewer can answer yes to "Does this still feel like KOTOR?" | Phillip Daigle's question for Baldur's Gate (secondary) | Human. |

The count is T 12, C 7, L 9, N 14, I 15, F 7, R 10, O 7 and K 8, which makes 89 items. Roughly half of them (about 45) have an Auto check, and the rest need a person or a judge.

## 6. Open questions the audit must settle first

1. **How to measure body height for KOTOR's bitmap fonts.** XAG measures from the top of the tallest letter to the bottom of the lowest tail. KOTOR's fonts are bitmap fonts described in a TXI file. The audit needs one tool that renders each font at its drawn size and measures body height from pixels, so that T-1 and T-2 compare the same quantity. docs/UI.md says the menus' words stay at the font's own size on a window that draws a frame larger, so the measure must be taken after kjk's own scaling.
2. **Which output the TV rules apply to.** L-1, L-2 and T-1 are TV rules. The audit needs a switch in the test harness for "TV at 1080p" and "handheld at 800p". The Steam Deck needs no safe area.
3. **A line of 1.5 spacing and 80 characters.** These are XAG's fallback defaults for players who cannot adjust the text. If kjk offers adjustment, these two rules may be dropped. The owner should decide which.
4. **Primary sources still missing.** A primary source for item comparison, new-item markers and loadouts would strengthen R-1, R-5 and R-10. The Diablo III blog post on item comparison was found but its page redirected away. The original Diablo IV and Baldur's Gate 3 documentation is worth another try.
5. **The "feel" check.** K-1 to K-8 rely on a human or a judge. The memory notes say sub-agent judges are only for judging, and that a measure should come before a judge. A measure of "layout shape kept" could be built from the screen's control rectangles against KOTOR's original, and would turn K-1 into an Auto check.

## Sources

Every link above is a full address. These are the primary sources read in full or in part, grouped by kind.

- Xbox Accessibility Guidelines: [overview](https://learn.microsoft.com/en-us/gaming/accessibility/guidelines), [101 text display](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/101), [102 contrast](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/102), [104 subtitles and captions](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/104), [107 input](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/107), [112 UI navigation](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/112), [114 UI context and help](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/114).
- Other guidelines: [Game Accessibility Guidelines](https://gameaccessibilityguidelines.com/full-list/), [Accessible Player Experiences](https://accessible.games/accessible-player-experiences/), [WCAG 2.2](https://www.w3.org/TR/WCAG22/), [WCAG 2.5.8 explained](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html).
- Platforms: [Apple, Designing for games](https://developer.apple.com/design/human-interface-guidelines/designing-for-games), [Apple, Game controls](https://developer.apple.com/design/human-interface-guidelines/game-controls), [Valve, Deck Verified](https://partner.steamgames.com/doc/steamdeck/compat), [Valve, Deck recommendations](https://partner.steamgames.com/doc/steamdeck/recommendations), [Valve, Steam Input best practices](https://partner.steamgames.com/doc/features/steam_controller/steam_input_gamepad_emulation_bestpractices), [Microsoft, Designing for Xbox and TV](https://learn.microsoft.com/en-us/previous-versions/windows/uwp/xbox-apps/designing-for-tv), [PlayStation accessibility](https://www.playstation.com/en-us/accessibility/).
- Developer posts and talks: [Game Developer on the Diablo II Resurrected GDC talk](https://www.gamedeveloper.com/marketing/exploring-modernity-legacy-and-accessibility-in-diablo-ii-resurrected), [GDC Vault, Remastering a Classic: Diablo II Resurrected](https://gdcvault.com/play/1028091/Remastering-a-Classic-Diablo-II), [Age of Empires III Definitive Edition HUD and UI](https://www.ageofempires.com/news/aoe3de-hud-and-ui), [Baldur's Gate Enhanced Edition postmortem](https://www.gamedeveloper.com/programming/postmortem-overhaul-games-i-baldur-s-gate-enhanced-edition-i-), [Dragon Age Veilguard accessibility spotlight](https://www.ea.com/games/dragon-age/dragon-age-the-veilguard/news/accessibility-spotlight), [Veilguard resource page](https://www.ea.com/able/resources/dragon-age-the-veilguard), [Titan Quest remaster on Xbox Wire](https://news.xbox.com/en-us/2018/03/21/titan-quest-remastering-pc-classic-xbox-one/), [Secretly console first](https://www.gamedeveloper.com/design/secretly-console-first-a-better-approach-to-multi-platform-game-ui-design).
- Interviews and press, read in part or through search summaries only: [RPG Site on Mass Effect Legendary Edition](https://www.rpgsite.net/feature/10719-mass-effect-legendary-edition-whats-changed-in-mass-effect-1), [Game Developer on the Legendary Edition mods](https://www.gamedeveloper.com/game-platforms/a-decade-of-mods-helped-devs-zero-in-on-improvements-for-i-mass-effect-legendary-edition-i-), [Siliconera on Final Fantasy Pixel Remaster](https://www.siliconera.com/interview-preparing-final-fantasy-pixel-remaster-for-the-switch-and-ps4/), [GoNintendo on the font work](https://www.gonintendo.com/contents/21170-square-enix-details-the-considerable-amount-of-work-that-went-into-the-fonts-of), [Epic Games Store, Stephen Kick interview](https://store.epicgames.com/news/system-shock-2-25th-anniversary-remaster-interview-stephen-kick), [Game Developer on Pillars of Eternity II](https://www.gamedeveloper.com/design/building-i-pillars-of-eternity-ii-i-without-losing-that-infinity-engine-charm), [PCGamesN on Diablo II Resurrected](https://www.pcgamesn.com/diablo-2-resurrected/fix-diablo-2), [Windows Central on Diablo II Resurrected](https://windowscentral.com/diablo-2-resurrected-developer-interview).
