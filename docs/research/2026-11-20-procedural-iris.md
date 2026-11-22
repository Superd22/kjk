# Procedural iris research

Date: 2026-11-20.

This note collects what could be confirmed about real eyes and shipped eye creators, so that kjk can paint a procedural eyeball.
The eyeball is a sphere of radius 12.3 mm with a texture of about 512 pixels per eye.

Every claim has a link.
A claim marked "search summary" was only seen in a web search result summary and not on the page itself.
A claim marked "our choice" is a number we pick for the shader and is not a measurement.
Section "What could not be confirmed" lists the gaps.

## What this means for our eyeball

The recipe has 10 steps.
Every step is evaluated per pixel from the direction of the point on the sphere.

1. Convert the point to an angle theta from the eye's forward axis. The iris radius is 5.9 mm, taken from a visible iris diameter of about 11.8 mm. On a sphere of radius 12.3 mm this gives a half angle of asin(5.9 / 12.3), which is about 28.7 degrees. Treat the cornea bulge as ignored (our choice).
2. Compute the normalised iris radius r as theta divided by that half angle. A pixel with r above 1 is eye white. A pixel with r below the pupil ratio is pupil.
3. Pupil. The pupil radius is a slider value between 0.26 and 0.57 of the iris radius. These two ends are the mean ratios at maximum constriction and at rest in a study of 42 people. The default is 0.40 (our choice). Fade the edge over about 0.02 of r so that it does not alias.
4. Fibres. Sample noise in polar space, stretched strongly along r and fine around the angle, so that it draws radial streaks. Use two octaves, one coarse (about 24 streaks around) and one fine (about 90 streaks). These counts are our choice.
5. Base colour. The iris has two stacked colours. The back layer is nearly black, because the pigment epithelium is black. The stroma in front holds the eye colour. Blue and grey irises show the black back layer through a nearly clear stroma. Brown irises have a stroma that is opaque with melanin.
6. Collarette. Put a raised, brighter, slightly yellower ring where r is about 0.33 of the way from the pupil edge to the limbus (the third from the task brief, not from a source). The inner zone is the pupillary zone and the outer zone is the ciliary zone. Give the inner zone a lighter, smoother fibre pattern.
7. Crypts. Darken a few soft almond blobs on both sides of the collarette. Use cellular noise thresholded to about 5 to 8 blobs per eye (our choice), elongated along r.
8. Furrows. Darken thin concentric arcs in the outer third of the ciliary zone, broken by the fibre noise. Make them stronger on brown eyes (our choice).
9. Outer ring. Multiply the colour by a dark band that starts at about r = 0.85 and reaches its darkest at r = 1.0. This reproduces the darker outer ring of the iris. MetaHuman exposes it as a limbal ring with a size of 0.6 to 0.85 and a softness of 0.02 to 0.15, which we reuse as slider ranges.
10. Eye white. Beyond r = 1 paint a warm off-white, a faint reddish tint that grows toward the left and right corners, a few thin vein curves, and a darker edge near the lids. Use the same 512 pixel texture, so the veins must be painted from the angle around the iris and not from the iris radius.

The sliders are these.

| Slider | Range | Source of the idea |
|---|---|---|
| Iris colour | Hue, saturation and value, or a melanin amount from 0 to 1 | Every game below has one |
| Second colour | A colour for the inner ring around the pupil, with a width from 0 to 0.5 of the radius | Hazel and central heterochromia, see section "How the eye colours arise" |
| Eye difference | Separate colour for each eye | BG3 and Veilguard offer heterochromia |
| Pupil size | 0.26 to 0.57 of the iris radius | The measured range in the section "Real sizes" |
| Limbal ring size | 0.6 to 0.85 | MetaHuman |
| Limbal ring softness | 0.02 to 0.15 | MetaHuman |
| Sclera tint | One colour, default a warm off-white | MetaHuman |
| Redness | 0 to 1 | MetaHuman vascularity, Veilguard bloodshot slider |

Do not add more sliders than this.
Fibre scale, crypt count and furrow strength stay as fixed values (our choice), and can be derived from the colour so that blue eyes look smoother and brown eyes look denser.

## Real sizes

Several measurements apply.

- The typical limbus diameter is 12 mm in a study of pupil motion, which models the iris edge with a radius of 6 mm. Source: [PMC9257589](https://pmc.ncbi.nlm.nih.gov/articles/PMC9257589).
- The horizontal visible iris diameter averaged 11.81 mm with a standard deviation of 0.24 mm in one group, and 11.98 mm with 0.33 in the other. The page was seen only in a search summary, and which group is the healthy group was not checked. Source: [search result list for HVID](https://pmc.ncbi.nlm.nih.gov/articles/PMC8284630).
- Contact lens references use 11.8 mm as the average, with a typical range of 11.6 to 12.0 mm. These are secondary sources, seen in a search summary. Source: [CL Spectrum](https://clspectrum.com/issues/1999/november/hvids-when-lens-size-matters).
- The measured value depends on the instrument. On 35 healthy eyes the Orbscan gave 11.77 mm and the IOL Master 700 gave 12.40 mm. Source: [Scientific Reports 2023 (search summary)](https://doaj.org/article/4a9f9451773248d99baff7717e5dfc93).
- So the figure of about 11.7 to 11.8 mm for the visible iris is right, and the 12 mm figure is the limbus including its edge.

The pupil follows these numbers.

- The pupil can be less than 2 mm to as large as 9 mm wide. The maximum varies a lot between people and falls with age. Source: [Wikipedia, Iris](https://en.wikipedia.org/wiki/Iris_(anatomy)).
- At rest in the dark, the pupil was 56.6 percent of the iris diameter, with a standard deviation of 7.0 points. At maximum constriction in light it was 26.2 percent, with a standard deviation of 4.3 points. Source: [PMC9257589](https://pmc.ncbi.nlm.nih.gov/articles/PMC9257589).
- With a 12 mm limbus, those ratios give about 6.8 mm at rest and about 3.1 mm when constricted. This is the paper's own derived estimate in the fetched summary.
- The iris is thin. Its average thickness was 0.34 mm in the same study, which matters little for a flat texture. Source: [PMC9257589](https://pmc.ncbi.nlm.nih.gov/articles/PMC9257589).

The limbal ring is an optical effect and not a pigment.

- It appears as a dark ring around the iris in some people and comes from the optical properties of the region between cornea and sclera. It is not present in everyone. Source: [Wikipedia, Eye color](https://en.wikipedia.org/wiki/Eye_color).
- No source gave a width in millimetres. One study gave the difference between the limbus diameter and the corneal white to white diameter as 0.33 mm with a standard deviation of 0.24 mm, which is a different measurement. Source: [search summary](https://pubmed.ncbi.nlm.nih.gov/26815917/).
- For a texture, use the MetaHuman values as the tested range, see the section on shipped creators.

## How the iris is layered

The layer names come from one anatomy page.

- The stroma is the front pigmented layer with fibres and blood vessels. It connects to the sphincter and dilator muscles. Source: [Wikipedia, Iris](https://en.wikipedia.org/wiki/Iris_(anatomy)).
- The collarette is the thickest region. It separates the pupillary zone from the ciliary zone and is a remnant of the embryonic pupil's coating. Source: [Wikipedia, Iris](https://en.wikipedia.org/wiki/Iris_(anatomy)).
- The crypts of Fuchs lie on either side of the collarette, and more crypts sit on the base of the iris. Source: [Wikipedia, Iris](https://en.wikipedia.org/wiki/Iris_(anatomy)).
- The page names folds, not furrows. These are radial contraction folds, the structural folds of Schwalbe, and circular contraction folds. The folds change when the iris dilates. Source: [Wikipedia, Iris](https://en.wikipedia.org/wiki/Iris_(anatomy)).
- The back layer, the pigment epithelium, is black. Source: [Wikipedia, Iris](https://en.wikipedia.org/wiki/Iris_(anatomy)).

How an artist builds this is described by Lefohn and colleagues.

- Ocularists paint artificial eyes with 30 to 70 layers of paint, with clear coat between layers. The authors' demonstration irises use 12 to 50 semi-transparent layered textures, depending on the colour. Source: [Lefohn et al. 2003, summary of the abstract and demos](https://stars.library.ucf.edu/facultybib2000/3883). Seen in a search summary. The PDF could not be read here.
- A student reimplementation notes that the paper allows layers in nearly any order. This is from a student page. Source: [Stanford CMPT 761](https://cs.stanford.edu/~zhossain/cmpt_761.html).
- The lesson for us is that a few stacked semi-transparent layers, each with its own noise, beat one complicated formula. Our recipe uses five, which are the black back, the stroma colour, the fibres, the collarette and the outer ring.

## How the eye colours arise

The main source is the Wikipedia page on eye colour, which summarises the literature.

- Colour depends on pigment in the iris and on how the stroma scatters light. Source: [Wikipedia, Eye color](https://en.wikipedia.org/wiki/Eye_color).
- Brown comes from a high concentration of melanin in the stroma, which absorbs both short and long wavelengths. Source: [Wikipedia, Eye color](https://en.wikipedia.org/wiki/Eye_color).
- Blue has little stroma melanin and no blue pigment. The dark back layer absorbs long wavelengths, and the stroma scatters short wavelengths by Rayleigh scattering. This is a structural colour that shifts with light. Source: [Wikipedia, Eye color](https://en.wikipedia.org/wiki/Eye_color).
- Green combines a yellowish pigment called lipochrome with the blue scattering. Source: [Wikipedia, Eye color](https://en.wikipedia.org/wiki/Eye_color).
- Hazel is usually golden or light brown from lipochrome, and the word is also used for a mixed brown and green eye. Source: [Wikipedia, Eye color](https://en.wikipedia.org/wiki/Eye_color).
- Grey eyes may have larger collagen deposits in the stroma, which give Mie scattering that depends little on wavelength. Source: [Wikipedia, Eye color](https://en.wikipedia.org/wiki/Eye_color).
- Melanin is the only pigment that matters much for normal iris colour. Source: [Wikipedia, Iris](https://en.wikipedia.org/wiki/Iris_(anatomy)).
- Newborns with little stromal pigment are usually blue eyed. If melanin is deposited later, the eyes become brown. Source: [Wikipedia, Iris](https://en.wikipedia.org/wiki/Iris_(anatomy)).
- Caution. The iris page also says brown eyes have more eumelanin and blue and green eyes have more pheomelanin. This did not match the eye colour page and was not checked against a primary source, so do not use it.

Shares of the world population are about 79 percent brown, 8 to 10 percent blue, 5 percent hazel, 3 percent grey and 2 percent green. Source: [Wikipedia, Eye color](https://en.wikipedia.org/wiki/Eye_color).

How to turn this into one slider is our choice.
Use a melanin amount m between 0 and 1.
At m near 0 the stroma is clear and the colour is a blue made from the black back layer plus a blue scatter tint.
At middle m, add a yellow lipochrome tint to get green and hazel.
At m near 1, the stroma is an opaque brown.
The user can also pick a plain colour, which then replaces the blue scatter tint.

## The eye white

The sclera is not pure white.

- The sclera has no melanin, which makes it a good place to judge yellowing in jaundice. The blue channel carried the most yellowness contrast in a study of newborns. Source: [Leung et al. 2019, Jaundice Eye Color Index](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6420273/). Seen in a search summary.
- A smartphone screening paper scores yellowness with the CIELAB b* value, and masks out the iris, pupil, lid skin and limbal vessels first. Source: [arXiv 2603.00161](https://arxiv.org/pdf/2603.00161). Seen in a search summary.
- One 2017 paper used the sclera as a near white calibration target for skin colour. Source: [search summary](https://pmc.ncbi.nlm.nih.gov/articles/PMC5782436). The summary gave no RGB values.
- No source found gave an RGB triple for a healthy sclera. Use a warm off-white of about (0.93, 0.91, 0.87) in linear sRGB display space as our choice, and let the slider move it toward yellow.
- No source found measured redness toward the corners or darkening near the lids. Both are our choice. Start the redness at the left and right corners, make the vein curves thin and branching, and darken by about 15 percent at the lid edges. This also hides the seam where the texture meets the lids.
- MetaHuman's sclera section has Tint, Transmission Spread and Transmission Color. When custom tint is off, the tint is derived from the skin tone. It also has vascularity intensity and coverage and a vein rotation. Source: [MetaHuman eye material controls](https://dev.epicgames.com/documentation/metahuman/eye-material-controls), seen in a search summary.

## What shipped creators give

MetaHuman is the most detailed.

- Iris section. Eye colour, iris size, global saturation (range 0.0 to 4.0), global tint, and a set of iris pattern presets. Source: [MetaHuman eye material controls](https://dev.epicgames.com/documentation/metahuman/eye-material-controls), seen in a search summary.
- Limbal ring colour, size (range 0.6 to 0.85) and softness (range 0.02 to 0.15). Source: [MetaHuman Python API, iris properties](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/MetaHumanCharacterEyeIrisProperties), seen in a search summary because the page itself redirected to an error.
- Cornea section. Size, which is how to change the iris region, plus limbus softness and limbus colour. Source: [MetaHuman eye material controls](https://dev.epicgames.com/documentation/metahuman/eye-material-controls), seen in a search summary.
- Pupil section. A dilation control, found only in the Korean page. Source: [MetaHuman eye material tools (ko)](https://dev.epicgames.com/documentation/metahuman/eye-material-tools?lang=ko), seen in a search summary.

Other games give fewer controls.

- Baldur's Gate 3. The character creator has an eye colour and a heterochromia choice that sets a second colour for the other eye from the same palette. Source: [BG3 wiki, Character Creation](https://bg3.wiki/wiki/Character_Creation), seen in a search summary.
- Cyberpunk 2077. The base game has one eye colour choice, and separate eye colours were never a feature, according to a Steam thread. Heterochromia comes from mods. The list of base colours was not found. Sources: [Steam thread](https://steamcommunity.com/app/1091500/discussions/0/3370404731622086415), [Heterochromia mod](https://www.nexusmods.com/cyberpunk2077/mods/10619), both seen in a search summary.
- Dragon Age The Veilguard. The previews list eye colour, a heterochromia option, a hooded eye slider, a bloodshot eye slider and a cataract slider. No limbal ring or sclera colour option was found. Source: [PC Gamer](https://www.pcgamer.com/games/dragon-age/dragon-age-the-veilguards-character-creator-fixes-my-old-nemesis-from-inquisition-the-lighting), seen in a search summary.
- Black Desert Online. The eye is split into pupil, iris and lens, and each part can be moved, rotated and resized, with colour for the iris. Female characters have extra pupil styles such as hearts. Sources: [Adventurer's Guide](https://www.naeu.playblackdesert.com/en-US/Wiki?wikiNo=5), [GamePressure](https://www.gamepressure.com/blackdesert/character-creation/z18711), both seen in a search summary.

What this shows is that every creator has iris colour, and most have a second eye colour.
Only MetaHuman has a limbal ring, a sclera tint and vein controls.
Veilguard is the one that offers bloodshot and cataract as sliders.
That supports the slider list at the top of this note.

## What could not be confirmed

- The Wecker, Samavati and Gavrilova iris paper has no abstract in any index found. Its method is unknown to us. Reference: [DOI 10.1145/1101389.1101411](https://api.crossref.org/works/10.1145%2F1101389.1101411).
- The Lefohn paper PDF could not be read here, so its colour blending equations are missing. The page was [escholarship.org/uc/item/8d88d3qc](https://escholarship.org/uc/item/8d88d3qc).
- No Jimenez paper on eye rendering was found. A search returned only his skin papers, for example [Separable Subsurface Scattering](https://iryoku.com/separable-sss).
- The 0.33 collarette position, the crypt counts, the fibre counts and the sclera colour are our choices and no source was found for them.
- The MetaHuman docs were seen only through search summaries, because the fetch tool returned only page titles. Recheck the exact slider ranges on the pages before copying them.
- The Cyberpunk list of colours, the Veilguard sclera options and the Black Desert pupil limits were not found.
