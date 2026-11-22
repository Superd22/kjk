# How real lips look, for a procedural lip paint layer

Date of the research is 2026-11-20. Every claim has a link. Where I only saw an abstract, a search summary or a trade article, the text says so. Where something is my own calculation or guess, the text starts with "Inference".

## Summary of the numbers most useful for painting lips

These are the numbers I would start from. Each one is explained, with its source, in the sections below.

- The lower lip of a Caucasian woman measures about L* 46, C* 26.6 and hue 32 degrees. Converted to a* and b* this is roughly a* 22 and b* 14 (Inference, see section 1.2).
- The lower lip of an African American woman measures about L* 33, C* 16 and hue 37 degrees. This is roughly a* 13 and b* 10 (Inference).
- Hispanic American and Asian Chinese women sit between those two. Both groups average about L* 42.4, C* 25 and hue 35.5 degrees, which is roughly a* 20 and b* 14.5 (Inference).
- Across 410 women the lower lip ranged from L* 24.6 to 54.4. The darkest of 11 lip tones was L* 28.5 with C* 10.1. The lightest was L* 48.0.
- Within one person the upper lip is darker and less red than the lower lip. In a study of 19 French Caucasian women the upper lip was about 2 to 4 L* units darker and about 3 to 4.5 a* units less red than the lower lip (my arithmetic on the published group means).
- The lip loses redness with age. In 1000 Korean women the average lip a* fell from 17.8 in the twenties to 14.6 in the sixties. The average L* did not change with age (37.8 to 37.9).
- The white roll, the pale ridge along the vermilion border, is 2 to 3 mm wide according to a standard anatomy reference.
- The upper vermilion is about 6.5 mm high at the centre and the lower vermilion is about 11.6 mm high in attractive young Caucasian women. The lower is therefore about 1.8 times the upper. The mouth is about 48 mm wide.
- In Chinese women the upper vermilion height falls from 8.1 mm in the twenties to 6.6 mm in the fifties. The mouth widens from 45.4 mm to 49.5 mm over the same ages.
- Vertical lip lines start to show at about age 32 to 33 and are clearly visible by about age 45. I found no measured spacing or width in millimetres.

## 1. Lip colour against the skin beside it

### 1.1 What the lip colour depends on

- The lip is red mainly because of blood. A 2025 paper from the LVMH research group says lip colour is mainly set by blood flow and the amount of oxygenated haemoglobin, and that redder lips are seen when capillaries run parallel to the lip surface. It says melanin has a smaller effect. Source: [Charton et al. 2026, Int J Cosmet Sci](https://pmc.ncbi.nlm.nih.gov/articles/PMC12877972/).
- The same group measured the lip spectrum. The lip reflects little light between 400 and 600 nm and reflects more between 600 and 700 nm. This is why it looks red. Source: [Vergnaud et al. 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC10410235/).
- The vermilion has a thick surface layer of living cells and no hair follicles. An atlas says the many tall dermal papillae let blood vessels come close to the surface and so give the zone its red colour. Source: [Digital Histology, lip](https://digitalhistology.org/organs-systems/digestive/oral-cavity/lip/lip-1/).
- A cadaver study of the commissure in 10 men measured the following layers. The vermilion epidermis was 135 micrometres thick, against 81 micrometres for the skin beside it and 199 micrometres for the inner mucosa. The stratum corneum was 20 micrometres on the vermilion and 25 micrometres on the skin, and it was absent on the mucosa. Source: [PMC8191552](https://pmc.ncbi.nlm.nih.gov/articles/PMC8191552).
- A 2025 review says the vermilion has 10 to 15 keratinocytes per melanocyte, against 4 to 1 in facial skin. I only saw this through the review and did not check its original source. Source: [Sun, Liu and Zhang 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12308779/).
- The Korean lip study says the lack of a link between lip lightness and age is probably due to the low melanin content of the vermilion. It also says one recent study found that lower lip lightness correlates with melanin. I did not read that study. Source: [Image-based lip aging, Korean women](https://pmc.ncbi.nlm.nih.gov/articles/PMC10776892).
- A 2023 paper (abstract only) built separate models for the vermilion, the mucosa and the facial skin from lip photographs. It found that melanin and blood volume differ between the three regions. It does not give the numbers in the abstract. Source: [Imai and Tsumura, CIC 2023](https://library.imaging.org/cic/articles/31/1/27).

### 1.2 Measured lip colour by ethnicity

The best data is the lip colour chart study by Charton and colleagues. It measured the natural lower lip of 514 women aged 19 to 68 with a hyperspectral camera under D65 light and a 10 degree observer. The camera uses crossed polarisers, so there is no specular shine in the numbers. The measurement spot was on "the darker part of the lower lip". The numbers are therefore darker than an average over the whole lip. Source: [Charton et al. 2026](https://pmc.ncbi.nlm.nih.gov/articles/PMC12877972/).

| Group (women) | L* | C* | Hue in degrees | Standard deviation of L* |
|---|---|---|---|---|
| African American (103) | 33.14 | 16.16 | 36.75 | 4.99 |
| Asian Chinese (104) | 42.36 | 24.99 | 35.46 | 3.16 |
| Hispanic American (100) | 42.48 | 25.06 | 35.52 | 4.03 |
| Caucasian, French and American (207) | 45.96 | 26.56 | 31.93 | 3.31 |

- The whole panel spanned hue 20 to 52 degrees and L* 23 to 56. Source: [Charton et al. 2026](https://pmc.ncbi.nlm.nih.gov/articles/PMC12877972/).
- A smaller earlier study of 410 women (the same data minus the Chinese group) reported an overall mean L* of 41.9, C* of 23.6 and hue of 34.0 degrees. The lowest and highest L* were 24.6 and 54.4. Source: [Vergnaud et al. 2024, Lip color diversity](https://pmc.ncbi.nlm.nih.gov/articles/PMC10823443/).
- That study found 11 lip tones. The darkest tone had L* 28.5, C* 10.1 and hue 37.1 degrees. The lightest, most saturated tone had L* 47.9, C* 29.6 and hue 30.7 degrees. Source: [Vergnaud et al. 2024](https://pmc.ncbi.nlm.nih.gov/articles/PMC10823443/).
- Inference. I converted the group means to a* and b* with a* = C* times cos(hue) and b* = C* times sin(hue). This gives roughly a* 22.5 and b* 14.0 for Caucasian women, a* 20.4 and b* 14.5 for Hispanic American and Asian Chinese women, and a* 13.0 and b* 9.7 for African American women. The paper does not report a* and b* per group, and the mean of a converted value is not exactly the converted mean, so treat these as approximate.
- Darker skin gives a darker, less saturated, browner lip, not a more purple one, in these numbers. African American lips had the lowest L* and C* and the highest hue angle, which means they were more yellow-brown than the Caucasian lips. Source: [Charton et al. 2026](https://pmc.ncbi.nlm.nih.gov/articles/PMC12877972/).
- African American women also had the widest spread in lightness and saturation. Their hue spread was the smallest of the four groups (standard deviation 3.28 degrees). Source: [Charton et al. 2026](https://pmc.ncbi.nlm.nih.gov/articles/PMC12877972/).
- The groups overlap strongly. Hispanic and Asian women shared 21 of the chart's 53 tones. The overlap between African American and Caucasian women was only 4 tones. Source: [Charton et al. 2026](https://pmc.ncbi.nlm.nih.gov/articles/PMC12877972/).
- A Japanese study of 85 women found four lip types, which were pink, pale, purple and brown. The share of pink fell with age and the share of purple rose with age. Brown was present at all ages. The page gives no percentages. Source: [Kao, lip color and shape with aging](https://www.kao.com/global/en/innovation/research-development/product-development/make-up/uncovering-lip-reality/).

### 1.3 Lip against perioral skin

- I did not find a paper with measured L*, a* and b* of the vermilion and of the perioral skin side by side. This is the biggest gap in the note.
- A conference report on 914 women (238 American Black, 238 American White, 225 French White and 213 Japanese, aged 18 to 65) measured lips and perioral skin. Lips were strongly redder in hue than the perioral skin. The vermilion was darker than the perioral skin in every group except African American women. There was no significant difference in chroma between lips and perioral skin. I could only read this through a search summary of the news article, so the exact numbers and the measuring method are unconfirmed. Source: [MDedge news report](https://mdedge.com/dermatology/article/7089/pigmentation-disorders/lips-color-characteristics-appear-vary-ethnicity-age).
- A large skin colour study found that facial redness (a*) was similar across Caucasian, Chinese, Kurdish and Thai skin, and that the biggest differences between groups were in yellowness (b*). The abstract gives no numbers. Source: [Xiao et al. 2017](https://eprints.whiterose.ac.uk/id/eprint/100718/).
- Inference. If facial skin a* is about the same in all groups, then the redness contrast between lip and skin is much larger for Caucasian lips (a* about 22) than for African American lips (a* about 13). The lip in darker skin would then differ from the skin mostly in lightness and hue, not in redness. I did not find a source that states this directly.
- Inference. The lip being darker than the skin for most people, and not for African American women, fits the lower lip L* of 33 for that group, which is close to a dark skin L*. I did not find the skin L* values to check this.

### 1.4 Upper lip against lower lip

- In 19 French Caucasian women, three different instruments agreed that the lower lip was lighter, redder, less yellow and more saturated than the upper lip. Source: [Vergnaud et al. 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC10410235/).
- From the cluster tables of that study (hyperspectral camera), the lower lip against the upper lip in the three groups was as follows. The numbers in the brackets are the lower value minus the upper value, which I calculated.

| Group | Lower lip L*, a*, b* | Upper lip L*, a*, b* | Lower minus upper |
|---|---|---|---|
| 1 (9 women) | 47.4, 23.3, 13.2 | 43.6, 18.9, 12.1 | L* +3.8, a* +4.5 |
| 2 (7 women) | 50.8, 19.9, 13.5 | 46.6, 17.1, 12.9 | L* +4.2, a* +2.8 |
| 3 (3 women) | 50.9, 21.7, 11.7 | 49.1, 18.8, 9.7 | L* +1.7, a* +2.9 |

- Only the French Caucasian sample was measured for this. I found no upper versus lower numbers for darker skin tones.
- I found no measured source for a darker upper lip, or a two-tone lower lip, in darker skins. A pigmentation review states in general that vermilion colour runs from reddish-pink to brown by ethnicity and that physiologic pigmentation is common in darker-skinned people. I saw this only in a search summary of the StatPearls page. Source: [StatPearls, lip anatomy](https://www.ncbi.nlm.nih.gov/books/NBK507900/).
- The Japanese dataset of 765 people also said the lower lip colour changes more with age than the upper lip. I only saw this in the abstract. Source: [J-STAGE abstract, vol. 46 no. 3](https://www.jstage.jst.go.jp/browse/koshohin/46/3/_contents/-char/en).

### 1.5 Men and women

- I found no measured lip colour comparison between men and women. A 2010 study had viewers adjust lip colour in photographs. They raised the redness contrast of lips to make female faces look more feminine and lowered it to make male faces look more masculine. Source: [Stephen and McKeegan 2010 summary](https://researchers.mq.edu.au/en/publications/lip-colour-affects-perceived-sex-typicality-and-attractiveness-of/).
- Inference. A male lip should therefore be painted closer to the skin colour than a female lip, with less lightness and redness contrast.

### 1.6 Age

- In 1000 Korean women (20 per year of age, aged 20 to 69, one camera and fixed lighting) the average lip colour by decade was as follows. These come from a normal photograph converted to CIELAB, not a calibrated colour device, and the dark mouth-line pixels were removed. Source: [Korean lip aging study](https://pmc.ncbi.nlm.nih.gov/articles/PMC10776892).

| Age | L* | a* | b* |
|---|---|---|---|
| 20s | 37.9 | 17.8 | 16.5 |
| 30s | 37.8 | 16.8 | 16.5 |
| 40s | 37.5 | 16.1 | 16.3 |
| 50s | 37.2 | 15.5 | 16.2 |
| 60s | 37.8 | 14.6 | 15.5 |

- Redness (a*) fell with age and had the strongest correlation of the colour numbers (r = minus 0.279). Yellowness (b*) fell slightly. Lightness showed no correlation with age. Source: [Korean lip aging study](https://pmc.ncbi.nlm.nih.gov/articles/PMC10776892).
- Inference. The a* fall is about 3.2 units over 40 years, or about 0.08 units per year, which is about 18 percent of the starting value.
- A Japanese study of 138 women aged 16 to 78 found that lip L* and a* decreased with age. Source: [Age-related differences in the functional properties of lips, 2018](https://pubmed.ncbi.nlm.nih.gov/29405429/).
- A Korean study of 114 women found that redness of both lips and blood flow decreased significantly with age. Source: [Age-related changes in lip characteristics in Korean women, 2019](https://pubmed.ncbi.nlm.nih.gov/30358914/).
- The Japanese dataset of 765 people said lip colour loses brightness, redness and yellowness after the thirties. Smoking affected it strongly, sun and season hardly at all. Source: [J-STAGE abstract](https://www.jstage.jst.go.jp/browse/koshohin/46/3/_contents/-char/en).
- The photograph study and the Japanese studies disagree on lightness. I would paint the age change as less red with lightness roughly unchanged, and treat any darkening as optional.

## 2. The vermilion border and the shape of the lips

### 2.1 The white roll

- The vermilion border is the rim of paler skin that separates the red lip from the skin around it. Source: [NIH Elements of Morphology, lips and mouth](https://elementsofmorphology.nih.gov/anatomy-oral.shtml).
- A standard anatomy reference gives the white roll as a pale convexity 2 to 3 mm wide along the vermilion border, formed by the bulge of the orbicularis oris muscle underneath. I saw this in a search summary of the StatPearls page. Source: [StatPearls](https://www.ncbi.nlm.nih.gov/books/NBK507900/).
- The white roll is described as a thin pale line that highlights the colour difference between the vermilion and the skin. It forms a V at the centre of the upper lip that together with the vermilion peaks forms the Cupid's bow. Source: [Vermilion border, Wikipedia](https://en.wikipedia.org/wiki/Vermilion_border).
- The dermis at the vermilion border is only about 0.5 mm thick because the muscle lies close beneath it (a cited figure, not mine). Source: [PMC8191552](https://pmc.ncbi.nlm.nih.gov/articles/PMC8191552).
- The authors of the lip colour study report that the Cupid's bow appears lighter because of the muscle configuration underneath. Source: [Vergnaud et al. 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC10410235/).
- I found no measured edge profile, such as the width in millimetres over which the colour goes from lip red to skin colour. Inference. The border is a pale line about 2 to 3 mm wide with a fairly sharp red edge on its lip side and a soft fade on its skin side. This is a painting suggestion from the anatomy, not a measurement.

### 2.2 Age and the border

- The Kao lip study says problems with vaguely defined contours increase with age, without giving numbers. Source: [Kao](https://www.kao.com/global/en/innovation/research-development/product-development/make-up/uncovering-lip-reality/).
- A 3D study of 169 Han Chinese women found that the upper vermilion height fell from 8.10 mm (twenties) to 6.57 mm (fifties). The lower vermilion height fell from 9.79 mm to 8.57 mm. The skin part of the upper lip grew from 13.75 mm to 16.74 mm. Source: [Chong et al. 2021](https://pmc.ncbi.nlm.nih.gov/articles/PMC7984336).
- The same study lists two angles at the bow that grew from about 130 degrees to about 138 degrees between the twenties and the fifties. A summary of the paper says these are bow angles, but I could not confirm the landmark definition. If they are bow angles, the bow becomes flatter with age. Source: [Chong et al. 2021](https://pmc.ncbi.nlm.nih.gov/articles/PMC7984336).
- Changes were small until 40 and larger after 40. Source: [Chong et al. 2021](https://pmc.ncbi.nlm.nih.gov/articles/PMC7984336).

### 2.3 Cupid's bow and proportions

- The Cupid's bow is the contour of the upper vermilion border. It curves from the commissures up to two peaks at the base of the philtral ridges, with a dip between the peaks. Source: [NIH Elements of Morphology](https://elementsofmorphology.nih.gov/anatomy-oral.shtml).
- In one year old children the vermilion height at the two peaks is always greater than at the middle dip. The paper gives no millimetres in its abstract, and the ratio did not differ by sex. Source: [Guide to bilateral cleft lip markings, 2021](https://pubmed.ncbi.nlm.nih.gov/34402316/).
- I found no adult value in millimetres for the peak height above the dip. A Chinese adult study gave an angle range of 109 to 160 degrees for the bow. I saw this only in a search summary. Source: [Cupid's peak study, ResearchGate listing](https://www.researchgate.net/publication/384986999_A_new_perspective_Quantitative_description_of_the_Cupid's_peak_aesthetics_essence).
- Attractive young Caucasian women (400 photographs) had an upper vermilion height of 6.47 mm (standard deviation 1.36) and a lower vermilion height of 11.64 mm (1.46). The mouth width was 48.06 mm (3.34). Source: [Ideal lips of Caucasian female](https://pubmed.ncbi.nlm.nih.gov/39198279/).
- Attractive young Caucasian men (300 photographs) had an upper vermilion height of 5.90 mm (1.62), a lower vermilion height of 11.15 mm (1.88) and a mouth width of 50.98 mm (3.74). Source: [Ideal Caucasian male lips](https://pubmed.ncbi.nlm.nih.gov/35290428/).
- Inference. The lower vermilion is about 1.80 times the upper in women and about 1.89 times in men (my division of the means above).
- In 169 Han Chinese women the lower vermilion was only about 1.2 to 1.3 times the upper (9.79 against 8.10 mm in the twenties, 8.57 against 6.57 mm in the fifties, my division). The mouth width was 45.38 mm in the twenties and 49.51 mm in the fifties. The philtrum width was about 11.7 mm and did not change with age. Source: [Chong et al. 2021](https://pmc.ncbi.nlm.nih.gov/articles/PMC7984336).
- A review says the ideal upper to lower red lip ratio is about 1 to 1 up to 1 to 1.6, and that a young upper vermilion is 7 to 9 mm high. Source: [Sun, Liu and Zhang 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12308779/).
- A study of 200 young adults (abstract summary only) gave a philtrum width of about 11.4 mm in men and 10.2 mm in women. Source: [Surgical and Radiologic Anatomy 2017](https://link.springer.com/doi/10.1007/s00276-017-1840-0).
- The Korean study defines lip ratio as mouth width divided by the sum of the upper and lower lip heights. It rose from 2.91 in the twenties to 3.42 in the sixties. The lip heights were in image units, not millimetres. Source: [Korean lip aging study](https://pmc.ncbi.nlm.nih.gov/articles/PMC10776892).

## 3. The mouth line and the corners

- The stomion is the point where the upper and lower lips meet. I found no measured colour or width for the closed mouth line.
- Indirect evidence is that the Korean study had to remove the dark pixels at the boundary between the lips (red value below 0.3 on a 0 to 1 scale) so that they would not affect the lip colour. This shows that the mouth line is much darker than the lips themselves. Source: [Korean lip aging study](https://pmc.ncbi.nlm.nih.gov/articles/PMC10776892).
- The commissure is where the side edges of the upper and lower vermilion join. Source: [NIH Elements of Morphology](https://elementsofmorphology.nih.gov/anatomy-oral.shtml).
- The French pilot study says some Asian women complain that the lips look dull at the corners compared with the centre. It links this to low blood flow and reduced haemoglobin oxygen saturation, citing another paper that I did not read. Source: [Vergnaud et al. 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC10410235/).
- The mouth widens by about 4 mm between the twenties and the fifties in Chinese women, so the corners move outward with age. Source: [Chong et al. 2021](https://pmc.ncbi.nlm.nih.gov/articles/PMC7984336).
- The length of the lower oral commissure fell significantly with age in 114 Korean women. Source: [Korean women lip study, 2019](https://pubmed.ncbi.nlm.nih.gov/30358914/).
- Inference. Paint the mouth line as a thin dark line, darker than the darkest lip tone, and let the corners fade toward the skin colour with less red than the lip centre.

## 4. Vertical lip lines

- I found no measured spacing, width or depth of the vertical furrows in millimetres. This is a gap.
- In 100 women, vertical wrinkle number and visibility rose linearly with age and became visible in the fifth decade. The surface pattern of the lip, recorded with a small skin sensor, fell into 3 main groups unrelated to age. Source: [Lévêque and Goubanova 2004](https://pubmed.ncbi.nlm.nih.gov/15178912/).
- A review of that work projects the first wrinkle at about age 32 to 33 and clear visibility by about age 45. In a clinical study 33.3 percent of women were rated severe against 6.6 percent of men. Source: [Sun, Liu and Zhang 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12308779/).
- In 1000 Korean women the upper lip vertical wrinkle index rose from 0.70 to 1.29 between the twenties and the sixties. The lower lip vertical index rose from 1.70 to 2.06. Horizontal wrinkles of the lower lip rose from 1.03 to 1.72. These indices have no units. The upper lip has the stronger vertical increase with age (r = 0.433). Source: [Korean lip aging study](https://pmc.ncbi.nlm.nih.gov/articles/PMC10776892).
- The lower lip already has more vertical lines than the upper lip at age 20 (1.70 against 0.70 on the same scale). Inference. A young lip should carry a few faint lines, mostly on the lower lip, and an older lip should have more lines, strongest on the upper lip.
- In 114 Indian women with Fitzpatrick types IV to VI, fine lines on the lower lip rose 23.2 percent between the twenties and the forties. Source: [Clin Cosmet Investig Dermatol 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC10387256/).
- The upper lip is more hydrated than the lower lip. Source: [Lévêque and Goubanova 2004](https://pubmed.ncbi.nlm.nih.gov/15178912/).

## 5. Highlights, wet and dry

- I found no measurement of where the specular highlight falls on lips. The colour studies used crossed polarisers to remove shine on purpose. Source: [Charton et al. 2026](https://pmc.ncbi.nlm.nih.gov/articles/PMC12877972/).
- The lips lose water about three times as fast as the cheek, which is why they dry and chap. Source: [Sun, Liu and Zhang 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12308779/).
- In Japanese women the water loss of the lip fell until about age 30, and the authors link this to lip dryness problems. Source: [Age-related differences in the functional properties of lips, 2018](https://pubmed.ncbi.nlm.nih.gov/29405429/).
- The vermilion has no sweat glands and almost no oil glands, so its shine comes from saliva and products. Source: [Basel histology atlas](https://tales.nmc.unibas.ch/en/histology-12/haut-und-anhangsorgane-anatomische-mikroskopie-49/lip-1858).
- Inference. The lower vermilion is fuller and tilts upward toward a light from above, so it should carry the main highlight, as a wide soft band near the centre of the lower lip. The upper vermilion tilts downward, which fits the measured darker upper lip. A dry lip should show a weaker, broader highlight and finer surface lines. A wet lip should show a sharper, brighter highlight that follows the lip curve and a slightly redder, deeper base. None of this is from a measured study.
- A 3D artists' forum (not a primary source) says the gloss on lips reads like a smooth coat on top of the lip surface that ignores the lip's small bumps. Source: [DAZ forum thread](https://www.daz3d.com/forums/discussion/comment/2435296).

## 6. The inner lip

- The vermilion ends at the wet–dry line, and the labial mucosa starts there. The mucosa lies inside the mouth. Source: [Digital Histology, lip](https://digitalhistology.org/organs-systems/digestive/oral-cavity/lip/lip-1/).
- The mucosa epithelium is thicker than the vermilion (199 against 135 micrometres) and has no stratum corneum. Its dermis is thinner (557 against 673 micrometres). Source: [PMC8191552](https://pmc.ncbi.nlm.nih.gov/articles/PMC8191552).
- The inner section of the lip mucosa is less hydrated than the outer section. I saw this only as a citation in the French pilot paper. Source: [Vergnaud et al. 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC10410235/).
- I found no L*, a* and b* values for the labial mucosa. I found a 2018 paper whose title says that melanin amounts cause the colour difference between the vermilion and the lip mucosa, but I could not open its abstract. Source: [PubMed 30224165](https://pubmed.ncbi.nlm.nih.gov/30224165/).
- Inference. The mucosa should be painted as a more saturated, slightly darker pink-red than the vermilion in light skins, with a soft edge at the wet line. In darker skins the vermilion carries more melanin than the mucosa, so the inner lip should look redder and pinker than the outside. This is my reading of the histology and the 2018 title, not a measurement.

## 7. Lipstick and lip liner

These sources are trade articles and a make-up artist interview. They describe technique and give no measurements.

- A common lip-lift method overlines only the Cupid's bow on the upper lip and the centre of the lower lip, leaves the outer sections on the natural line, and blends the liner inward. Source: [Yahoo Style, lip liner technique](https://www.yahoo.com/now/lip-liner-technique-makes-look-141640543.html).
- Other guides say to draw just outside the natural edge and blend inward, and warn that a gap that is too big makes the lips look dipped in colour. Source: [Woman and Home, overlining](https://www.womanandhome.com/beauty/makeup/overlining-lip-liner-technique/).
- Several guides suggest a liner close to the natural lip tone and a lipstick two to three shades lighter than the liner. Source: [Maybelline, how to overline lips](https://www.maybelline.com.au/makeup-tips/lips/how-to-overline-lips).
- No source I found gives the overdraw distance in millimetres. Inference. Because the white roll is 2 to 3 mm wide, a believable overdraw cannot be much more than about 1 to 2 mm without covering the white roll.
- I found no source on underdrawing the lip.
- For dark lipstick, one make-up artist advises against a shade that is too brown, because it blends into the skin. For one actress he lined the edge with a black-cherry liner, filled the lip with a burgundy and put a bright berry stripe in the centre for depth. He asked for clean edges but "not severity". Source: [Yahoo, Nick Barose on dark lipstick](https://www.yahoo.com/lifestyle/lupita-nyongos-makeup-artist-nick-171340362.html).
- Another guide starts a vamp lip with a deep wine pencil and swipes dark lipstick over it. Source: [MAC, Diva lips](https://www.maccosmetics.co.uk/mac-trend-makeup-looks/diva-lips).
- A 2005 paper measured lipstick spreading past the vermilion border over time on 20 women (the "spread phenomenon") and showed that lip wrinkles worsen it. Source: [Improving lip wrinkles, lipstick image analysis](https://pubmed.ncbi.nlm.nih.gov/15998326/).

## Things I could not find

- Measured L*, a* and b* of the vermilion and the perioral skin of the same people.
- Any lip colour difference between men and women, and any lip colour numbers for men.
- Upper versus lower lip colour for darker skin tones, and any measured two-tone lower lip.
- Measured stomion and commissure colour, and the width of the closed mouth line.
- Measured spacing, width and depth in millimetres of vertical lip lines.
- A measured edge profile of the vermilion border, and any number for how the border softens with age.
- Adult Cupid's bow peak height above the dip in millimetres. I found only toddler ratios without numbers and an angle range seen in a search summary.
- Measured highlight position on lips, and numbers for wet against dry lips.
- Measured colour of the labial mucosa against the vermilion.
- Any measured overdraw distance for lip liner.
- The full text of three papers I could only see in part were the Japanese lip study of 765 people ([J-STAGE](https://www.jstage.jst.go.jp/browse/koshohin/46/3/_contents/-char/en)), the L'Oréal perioral study of 914 women ([MDedge](https://mdedge.com/dermatology/article/7089/pigmentation-disorders/lips-color-characteristics-appear-vary-ethnicity-age)) and the 2018 vermilion against mucosa melanin paper ([PubMed 30224165](https://pubmed.ncbi.nlm.nih.gov/30224165/)).

## Caveats on the data

- The two big colour studies use different devices and different spots. The hyperspectral camera measured a small spot on the darker part of the lower lip. The Korean study averaged the whole visible lip from an ordinary photograph. This is why the Korean L* (about 38) is lower than the French pilot L* (about 47) and the Caucasian lower lip L* (about 46). A paint layer should pick one source and stay with it. In the French pilot the three devices also disagreed by up to about 6 L* units and 9 a* units on the same lips, so the absolute values are only as good as the device. Source: [Vergnaud et al. 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC10410235/).
- Most of the numbers are for women who use lipstick. The chart study required lipstick use at least twice a week. Source: [Charton et al. 2026](https://pmc.ncbi.nlm.nih.gov/articles/PMC12877972/).
