# Minimum GPU target for kjk

This document was researched on 2026-11-18.
It answers one question.
Which graphics card should kjk name as its minimum, and which one as its recommended card?

kjk renders through Bevy 0.19.1 and wgpu 29.
It ships for Windows x86-64, macOS on Apple silicon, and Linux x86-64.

Every claim carries the URL of the page it was read on, or the path of the kjk file it was read in.
The words "not verified" mark every claim that could not be confirmed on a primary source.
This was a quick pass of about 20 minutes.
No frame rate was measured on any of the cards named here.

## Summary

- The proposed minimum is an NVIDIA GeForce GTX 1050 Ti with 4 GB, an AMD Radeon RX 570 with 4 GB, or an Apple M1. The graphics driver must offer Vulkan 1.1 or Direct3D 12.
- Integrated graphics of the Intel Iris Xe and AMD Radeon Vega 8 class should be listed as supported at 720p with low settings, once someone has played the game on one.
- The proposed recommended card is an NVIDIA GeForce GTX 1660 Super, an RTX 2060 or an AMD Radeon RX 6600, with 6 GB or more.
- Shadows at Ultra need a card with hardware ray tracing. DLSS needs an NVIDIA RTX card. Both are optional, and the game falls back without them.
- The cheapest card on Vast.ai close to the minimum was a GTX 1650 at $0.049 an hour, or a GTX 1060 at $0.016 an hour. No GTX 1050 Ti was offered when this was checked.

## The Steam Hardware Survey for October 2026

The most recent survey Steam has published is the one for October 2026 (https://store.steampowered.com/hwsurvey/).
Both pages below were read directly.

### Operating systems

The main survey page combines all platforms (https://store.steampowered.com/hwsurvey/).

| Operating system | Share in October 2026 | Change from September |
|---|---|---|
| Windows | 93.70% | -1.33 points |
| Linux | 3.95% | +0.90 points |
| macOS | 2.35% | +0.43 points |

### Video memory

The main page lists the video memory of the players' cards (https://store.steampowered.com/hwsurvey/).

| Video memory | Share |
|---|---|
| 512 MB | 2.81% |
| 1 GB | 1.46% |
| 2 GB | 3.78% |
| 3 GB | 1.30% |
| 4 GB | 5.54% |
| 6 GB | 5.20% |
| 8 GB | 26.92% |
| 10 GB | 1.84% |
| 11 GB | 0.94% |
| 12 GB | 12.91% |
| 16 GB | 26.35% |
| 20 GB | 1.28% |
| 22 GB | 0.97% |
| 24 GB | 5.40% |
| 32 GB | 1.40% |
| 64 GB | 0.52% |
| Other | 1.38% |

Added up from that table, 89.27% of players have 4 GB or more, and 83.73% have 6 GB or more.
8.05% report 2 GB or less.
Integrated graphics share the computer's memory, so the 512 MB and 1 GB rows probably hold many integrated chips (not verified).

### The weakest cards that still have a meaningful share

The video card page lists each card's share of Windows players (https://store.steampowered.com/hwsurvey/videocard/).
These are the oldest or slowest dedicated cards with 0.5% or more in October 2026.

| Card | Share | Generation |
|---|---|---|
| NVIDIA GeForce GTX 1650 | 2.44% | Turing without ray tracing, 2019 |
| NVIDIA GeForce GTX 1060 | 1.45% | Pascal, 2016 |
| NVIDIA GeForce GTX 1050 Ti | 1.14% | Pascal, 2016 |
| NVIDIA GeForce GTX 1070 | 0.61% | Pascal, 2016 |
| AMD Radeon RX 580 2048SP | 0.58% | Polaris, 2017 |
| AMD Radeon RX 580 | 0.57% | Polaris, 2017 |
| NVIDIA GeForce GTX 1050 | 0.54% | Pascal, 2016 |

These older cards are below 0.5% on the same page (https://store.steampowered.com/hwsurvey/videocard/).

| Card | Share |
|---|---|
| Intel UHD Graphics 620 | 0.44% |
| AMD Radeon RX 570 | 0.35% |
| AMD Radeon Vega 8 Graphics | 0.30% |
| Intel UHD Graphics 630 | 0.27% |
| AMD Radeon RX 550 | 0.21% |
| NVIDIA GeForce GT 1030 | 0.20% |
| NVIDIA GeForce GTX 970 | 0.18% |
| NVIDIA GeForce GTX 750 Ti | 0.17% |
| NVIDIA GeForce GTX 960 | 0.16% |
| Intel HD Graphics 4600 | 0.16% |

The GTX 970, GTX 750 Ti, GTX 960 and HD Graphics 4600 together come to 0.67%.
Cards older than Pascal and Polaris are therefore no longer worth a promise in the minimum line.

### Integrated graphics

The video card page lists these integrated chips (https://store.steampowered.com/hwsurvey/videocard/).

| Integrated chip | Share |
|---|---|
| Intel Iris Xe Graphics | 1.78% |
| AMD Radeon Graphics | 1.75% |
| Intel(R) UHD Graphics | 1.27% |
| AMD Radeon(TM) Graphics | 0.97% |
| Intel UHD Graphics | 0.72% |
| Intel(R) Graphics | 0.60% |
| AMD Radeon (TM) Graphics | 0.51% |
| Intel UHD Graphics 620 | 0.44% |
| AMD Radeon 780M Graphics | 0.32% |
| AMD Radeon Vega 8 Graphics | 0.30% |
| Intel Arc Graphics | 0.29% |
| Intel UHD Graphics 630 | 0.27% |
| AMD Radeon Vega 3 Graphics | 0.19% |
| Intel HD Graphics 520 | 0.19% |
| Intel HD Graphics 620 | 0.19% |
| Intel HD Graphics 4600 | 0.16% |

These rows add up to 9.95% of Windows players.
The page also has an "Other" row of 8.58%, which holds more integrated chips that the page does not name.
So roughly one Windows player in ten plays on integrated graphics.
"AMD Radeon Graphics" is the name that recent Ryzen processors give their integrated graphics, but some of those rows could be other chips (not verified).
"Intel Arc Graphics" with no model number is the integrated graphics of Intel Core Ultra processors (not verified).

### Direct3D and Vulkan

The video card page sorts the cards by the Direct3D version they support (https://store.steampowered.com/hwsurvey/videocard/).

| Direct3D class | Share |
|---|---|
| Direct3D 12 cards | 90.63% |
| Direct3D 11 cards | 0.40% |
| Direct3D 10 cards | 0.20% |
| Direct3D 9 cards | 0.01% |
| "DirectX 8 GPUs and below" | 8.76% |

The 8.76% row is almost certainly cards the survey could not classify rather than real Direct3D 8 hardware, because no Direct3D 8 card can run Windows 10 (not verified).
The Direct3D 11 class is mostly Intel HD Graphics 4000, Ivy Bridge and Haswell.
The survey has no table for Vulkan.
The Steam Deck appears in the Direct3D 12 table with 0.46% of that table.

## What wgpu and Bevy require

### Backends compiled into kjk

Bevy's render crate turns on wgpu's `dx12`, `metal` and `vulkan` features (https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_render/Cargo.toml).
wgpu's OpenGL backend only comes with Bevy's `gles` feature, which maps to `wgpu/gles` in the same file.
kjk's `Cargo.toml` does not turn on `gles` or `webgl2`.
So the native builds have no OpenGL or OpenGL ES fallback.
A computer without Vulkan, Direct3D 12 or Metal cannot run kjk at all.

wgpu's README lists Vulkan, Direct3D 12 and Metal as first class backends (https://github.com/gfx-rs/wgpu/blob/trunk/README.md).
It lists OpenGL 3.3 and OpenGL ES 3.0 as "Downlevel/Best Effort Support" only.

On native builds, Bevy asks wgpu for every backend it was built with, through `Backends::all()` (https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_render/src/settings.rs).
Bevy's default priority is `Functionality`, and the `WGPU_SETTINGS_PRIO` environment variable can change it (same file).

### Vulkan

wgpu 29 accepts a Vulkan 1.0 device, but it hides the device unless it also has the `VK_KHR_storage_buffer_storage_class` and `VK_KHR_maintenance1` extensions (https://github.com/gfx-rs/wgpu/blob/v29.0.4/wgpu-hal/src/vulkan/adapter.rs#L2208-L2225).
Both extensions are part of Vulkan 1.1, so any Vulkan 1.1 driver passes this check.
wgpu also hides a device whose driver reports no Vulkan conformance, unless the device is MoltenVK or an instance flag allows it (https://github.com/gfx-rs/wgpu/blob/v29.0.4/wgpu-hal/src/vulkan/adapter.rs#L2189-L2205).
Features such as timeline semaphores and the Vulkan memory model are used when the device offers Vulkan 1.2 or 1.3, and are skipped otherwise (same file, around line 375).

### Direct3D 12

wgpu 29 creates its Direct3D 12 device at feature level 11_0 (https://github.com/gfx-rs/wgpu/blob/v29.0.4/wgpu-hal/src/dx12/adapter.rs#L93).
It then reads the highest level the card supports, from 11_0 up to 12_2 (same file, lines 109 to 134).
So a card with a Direct3D 12 driver at feature level 11_0 is enough for wgpu itself.

### Metal

wgpu's Metal backend runs on every Apple silicon Mac.
The adapter code checks Metal GPU families for single capabilities, such as multisample resolve, and does not reject a device for its family (https://github.com/gfx-rs/wgpu/blob/v29.0.4/wgpu-hal/src/metal/adapter.rs).
Every Apple silicon chip, starting with the M1, belongs to the Apple 7 family or later (not verified on Apple's own page during this pass).

### Limits

Outside WebGL2, Bevy starts from `wgpu::Limits::default()` (https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_render/src/settings.rs).
Those are the WebGPU default limits, which every desktop card with Vulkan 1.1 or Direct3D 12 meets (not verified card by card).

## What kjk asks of the card

kjk adds nothing that a Vulkan 1.1 or Direct3D 12 card lacks, as long as the optional features stay off.
These are the features kjk uses, each with the card it needs.

| Feature | Where in kjk | What it needs | Required? |
|---|---|---|---|
| Occlusion queries on the rooms | `crates/game/src/room_queries.rs:485` | `wgpu::QueryType::Occlusion`, which is core wgpu and needs no feature flag | Yes, but every Vulkan, Direct3D 12 and Metal card has it |
| Ambient occlusion | `crates/game/src/ambient_occlusion.rs:249` | Compute shaders and at least 5 storage textures per shader stage | No. The pass turns itself off below 5, as Bevy's own does |
| Shadow tiles | `crates/game/src/shadow_tiles.rs:1356` | Read-only storage buffers in the fragment shader | Yes, and every Vulkan 1.1 or Direct3D 12 card has them |
| Skin once | `crates/game/src/skin_once.rs:1` | A compute pass | No. It is off unless `KJK_SKIN_ONCE=on` or Shadows is at Ultra |
| Shadows at Ultra | `crates/game/src/ray_structures.rs:99` | `EXPERIMENTAL_RAY_QUERY`, ray hit vertex return, extended acceleration structure vertex formats, and acceleration structure binding arrays | No. Without them, Ultra draws as High does, and the feature is never used under Metal |
| TAA | `Cargo.toml`, the `bevy_anti_alias` feature | Motion vectors and a temporal pass, which any card can draw | No. MSAA 4x is the default (`crates/game/src/display.rs:107`) |
| DLSS | `crates/game/Cargo.toml`, the `dlss` feature, and `crates/game/src/temporal.rs:380` | An NVIDIA RTX card under Vulkan on Windows or Linux | No. The choice falls back to TAA |
| Timestamp queries | Not used anywhere in `crates/` | | |

The ray query features exist only on cards with hardware ray tracing.
Those are NVIDIA RTX 20 series and later, AMD RX 6000 series and later, and Intel Arc (not verified on vendor pages).
`KJK_RAYS=off` makes the render device without them, so a run behaves like a card that has none (`crates/game/src/runtime.rs:134`).

So nothing in kjk's required path shuts out an old or integrated card that wgpu accepts.
What will shut those cards out is speed and video memory.
The enhanced lighting, the soft shadows, ambient occlusion and TAA all cost frame time.
`docs/LIGHTING-PLAN.md:27` already names a GTX 1050 Ti class machine as the one that decides the lighting's speed.

## What comparable games list

These lines were read from Steam's store data for each game, through the store's `appdetails` endpoint.

| Game | Released | Minimum graphics line on Steam |
|---|---|---|
| STAR WARS: Dark Forces Remaster (Nightdive) | Feb 28, 2024 | "GPU with DirectX 11 or Vulkan 1.1 support" (https://store.steampowered.com/app/2292260/) |
| STAR WARS Episode I: Jedi Power Battles (Aspyr) | Jan 23, 2025 | "Nvidia GeForce GTX 650 / AMD Radeon R7 260X / Intel Arc A310" (https://store.steampowered.com/app/2443630/) |
| Tomb Raider IV-VI Remastered (Aspyr) | Feb 14, 2025 | "GeForce GT 620 1gb / Radeon HD 8670D" (https://store.steampowered.com/app/2525380/) |
| Heretic + Hexen (Nightdive and id Software) | Aug 7, 2025 | "NVIDIA Geforce GTX 650 TI (2GB) or AMD HD 7750 (1GB)", DirectX 11 (https://store.steampowered.com/app/3286930/) |
| R.E.P.O. (semiwork) | Feb 26, 2025 | "GTX 970", DirectX 10 (https://store.steampowered.com/app/3241660/) |
| PEAK (Aggro Crab and Landfall) | Jun 16, 2025 | "GTX 1060 or RX 6600 XT", DirectX 12 (https://store.steampowered.com/app/3527290/) |
| Tiny Glade (Pounce Light) | Sep 23, 2024 | "For 720p: A graphics card with Vulkan 1.2 support and 2+ GB of video memory: Radeon R9 270 (driver 21.3.1), GeForce GTX 670 (at least Kepler architecture, driver 472.12 or newer), Intel Iris Xe, or similar." Its Mac minimum is "Apple M1 or Apple A18 Pro (Apple silicon only)" (https://store.steampowered.com/app/2198150/) |

Tiny Glade is included because it is a recent Rust game built on Bevy's ECS, and it names a Vulkan version, a video memory size, and an integrated chip in its line.

For reference, the original games' store pages list much older hardware.
KOTOR 1 asks for "32 MB with Hardware T&L" and DirectX 9.0b (https://store.steampowered.com/app/32370/).
KOTOR 2 asks for a Radeon HD 5450, a GeForce 260 or an Intel HD 4400, with 256 MB of video memory (https://store.steampowered.com/app/208580/).
No store page for the KOTOR remake was found in Steam's store search on 2026-11-18.

The remasters of older games list cards from 2012 and 2013, such as the GTX 650.
The new 3D indie games list cards from 2014 to 2016, such as the GTX 970 and the GTX 1060.
The remasters that keep the old look are the closest to kjk's default look, but kjk's lighting is closer to the new indie games' cost.

## Driver support for the oldest NVIDIA cards

NVIDIA's 580 driver branch is the last one that supports Maxwell, Pascal and Volta cards, on both Windows and Linux.
This covers the GTX 750 Ti, the GTX 900 series and the GTX 10 series.
The sources are NVIDIA's support plan page (https://nvidia.custhelp.com/app/answers/detail/a_id/5706/) and its Unix legacy driver page (https://nvidia.custhelp.com/app/answers/detail/a_id/3142/).
These two pages were seen through a web search summary, not opened directly.
The summary says the 580 branch receives security fixes until 2028.
A GTX 1050 Ti minimum therefore stays on a maintained driver for about two more years.

## Recommendation

### Minimum

The minimum should be "NVIDIA GeForce GTX 1050 Ti (4 GB) or AMD Radeon RX 570 (4 GB), with a Vulkan 1.1 or Direct3D 12 driver. On a Mac, an Apple M1."

- The GTX 1050 Ti has 1.14% of Windows players on its own, and the GTX 1050, GTX 1060 and RX 580 sit just above it (https://store.steampowered.com/hwsurvey/videocard/).
- The project already treats the GTX 1050 Ti class as the machine that decides the lighting's speed (`docs/LIGHTING-PLAN.md:27`).
- 89.27% of players have 4 GB of video memory or more (https://store.steampowered.com/hwsurvey/).
- Nothing in kjk's required path needs a feature that a Vulkan 1.1 or Direct3D 12 card lacks.
- Older cards, such as the GTX 750 Ti and GTX 970, would probably start, but together they hold under 1% of players and only get security fixes now.

The minimum line should also say what the game turns off on such a card.
At the minimum, the player is expected to use 1080p with the original lighting, MSAA, and Shadows at Medium or lower (not measured).

Integrated graphics are about 10% of Windows players, so they should not be excluded by name.
Intel Iris Xe and AMD Radeon Vega 8 or newer should be added to the minimum line at 720p once someone has played kjk on one.
Intel HD Graphics 4600 and older have no Vulkan driver on Windows and only reach Direct3D feature level 11_1, so they should not be promised (not verified).

### Recommended

The recommended card should be "NVIDIA GeForce GTX 1660 Super or RTX 2060, or AMD Radeon RX 6600, with 6 GB or more."

- These cards should run the enhanced lighting, soft shadows, TAA and ambient occlusion at 1080p (not measured).
- 83.73% of players have 6 GB or more (https://store.steampowered.com/hwsurvey/).
- The RTX 2060 and RX 6600 also have hardware ray tracing, so they can run Shadows at Ultra. The RTX 2060 can also run DLSS.

### Card to rent for testing the minimum

On 2026-11-18 the Vast.ai public offers list had no GTX 1050 Ti and no GTX 750 Ti for rent (read through `https://console.vast.ai/api/v0/bundles/`).
These close cards were offered.

| Card | Offers | Cheapest price per GPU per hour |
|---|---|---|
| GTX 1060 | 3 | $0.016 |
| GTX 1650 | 2 | $0.049 |
| GTX 1660 Super | 6 | $0.045 |
| RTX 2060 | 6 | $0.054 |
| RTX 3060 | 24 | $0.041 |

The GTX 1650 is the best stand-in for the minimum, because it is the most played card in the survey and is only a little faster than a GTX 1050 Ti (not measured).
A GTX 1060 is the cheapest, but it is clearly faster than a GTX 1050 Ti, so a pass on it proves less.
An RTX 2060 or RX 6600 machine can test the recommended line.
Clore.ai's listings were not checked in this pass.

A Vast.ai machine is a Linux Docker container without a display.
kjk would have to run with its headless option, and the container would need the NVIDIA driver's graphics capability for Vulkan, not only compute (not verified).
The project's own GTX 1050 Ti tester remains the better check of the minimum, because it is the real card on Windows.

## Open questions

- No frame time has been measured on a GTX 1050 Ti, a GTX 1650, or an integrated chip with the current lighting.
- It is not known whether kjk starts on Intel UHD 620 under Windows, or on Mesa's Vulkan driver for older Intel chips under Linux.
- The survey's macOS and Linux pages were not read, so the share of Apple chips and of Linux cards is unknown here.
