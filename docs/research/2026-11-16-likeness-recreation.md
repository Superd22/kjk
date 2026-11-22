# Recreating a real person's likeness in a character creator

This note asks one question. How much control does a character creator need so that a player can rebuild a real person's face, such as their own, a friend's or a celebrity's, well enough that other people recognise it?

It was written on 2026-11-16 for kjk. Other notes in this folder cover the neighbouring subjects, and this note does not repeat them. The design survey is in `docs/research/2026-11-16-character-creation-design.md`. The technology note (`docs/research/2026-11-16-character-creation-tech.md`) explains morph targets, FaceGen's 130-number face coordinate, MetaHuman's DNA files, and the four options A to D for kjk. The sculpting note (`docs/research/2026-11-16-character-sculpting.md`) measured KOTOR's heads and proposed bone-offset sculpting (design A) and a baked soft deformation field (design B).

Every claim links to its source. Where only a press article or a fan page was available, the text says so with the words "press source" or "fan source". Section 5 reports a measurement that was taken for this note from the KOTOR game files. This note does not cover the legal side of importing photos, and it does not make recommendations for kjk. Both were left for later.

## 1. Terms used in this note

A **likeness** is a face that people who know the real person would name as that person.

**Featural processing** means judging a face by its parts, such as the shape of the eyes or the thickness of the lips.

**Configural processing** means judging a face by how its parts relate to each other. Maurer, Le Grand and Mondloch split it into three kinds. The first kind is seeing the basic layout of a face, which is two eyes above a nose above a mouth. The second kind is holistic processing, which glues the parts into one whole. The third kind is sensitivity to the spacing between the parts ([Maurer, Le Grand and Mondloch, Trends in Cognitive Sciences, 2002](https://mail.maurer.ca/dpubs/Maurer_ReviewFaceProcessing.pdf)).

**Internal features** are the eyes, the eyebrows, the nose and the mouth. **External features** are the hair, the face outline, the jaw line and the ears.

**Pigmentation** (also called **surface reflectance**) is everything about how the skin reflects light. It includes the skin colour, its hue, its shine, its translucency, and how these vary across the face, for example freckles or a darker upper lip ([Sinha et al., Proceedings of the IEEE, 2006, result 9](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)). In a game, pigmentation lives in the face texture and the skin shader. **Shape** is the 3D geometry, which lives in the mesh.

A **caricature** is a face whose differences from the average face have been exaggerated. An **anti-caricature** is a face whose differences have been reduced.

A **principal component** is one direction in which the faces of a scanned set differ from their average face. A morphable model describes a face as the average plus a weighted sum of these directions. The first component explains the most variation, the second the next most, and so on. **Variance retained** is the share of the scanned set's total variation that the first *k* components explain.

## 2. What people use to recognise a face

### Familiar faces and unfamiliar faces are different problems

Bruce and Young's 1986 model of face recognition is still the standard framework. In it, the brain first builds a structural description of the face in view. It then compares that description with stored "face recognition units", one for each face the person knows. A match then activates the person's identity and, after that, their name ([Bruce and Young, British Journal of Psychology, 1986](https://scilit.com/publications/4c2f77953ab24bf0b4255283ab4a4e24)). Andrew Young's 2017 Bartlett Lecture says that this model has been the dominant view for the three decades since ([Young, 2017](https://eprints.whiterose.ac.uk/126254/1/2017_10_09_45th_Bartlett_Lecture_Young_ACCEPTED_VERSION.pdf)).

For a character creator, the important consequence is that recognising a known person is far more forgiving than matching two pictures of a stranger. Burton, Wilson, Cowan and Bruce showed people poor-quality security camera video. People who knew the filmed people identified them well. People who did not know them did poorly, and police officers with forensic experience did as poorly as other strangers ([Burton et al., Psychological Science, 1999](https://research.bond.edu.au/en/publications/face-recognition-in-poor-quality-video-evidence-from-security-sur/)). When the researchers hid the body or the walk in the video, recognition dropped only a little. When they hid the head, recognition dropped sharply ([Sinha et al., 2006, result 2](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)).

A player who rebuilds their own face, a friend's face or a famous face is therefore in the easy case. The audience already knows the person. The audience also tolerates a lot of degradation, as the next section shows.

### Very little detail is enough

Harmon and Julesz turned photographs of familiar faces into blocks of averaged grey. People still recognised them with high accuracy at only 16 × 16 blocks ([Sinha et al., 2006, result 1](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)). Harmon's block portrait of Abraham Lincoln illustrated his November 1973 Scientific American article "The Recognition of Faces" ([Scientific American](https://www.scientificamerican.com/article/the-recognition-of-faces/)).

Yip and Sinha blurred famous faces and asked people to name them without hints. People named more than half of them at an effective resolution of 7 × 10 pixels. Recognition reached its ceiling at 19 × 27 pixels ([Sinha et al., 2006, result 1](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)). At such low resolutions the individual features are a blur, so the authors conclude that the useful information lies in the overall arrangement of the face ([same paper](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)).

Jarudi and Sinha then tested what happens to internal and external features as resolution falls. At high resolution, the internal features of familiar faces were more useful than the external ones. As resolution fell, the order reversed, and the hair and the face outline became the more useful set ([Jarudi and Sinha, MIT AI Memo 2003-004](https://dspace.mit.edu/handle/1721.1/7274)).

Colour helps most when shape is degraded. When images are blurred, people recognise faces better in colour than in grey. Pseudo-coloured faces, whose hues are wrong, help as much as natural colours. The authors read this as colour helping the eye find the boundaries of the eyes, the hair line and other parts, not as the exact hue carrying identity ([Sinha et al., 2006, result 10](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)).

Line drawings alone are weak. In a study by Graham Davies and colleagues, people recognised 47% of famous faces from line drawings that held only contours, against 90% from the original photographs ([Sinha et al., 2006, result 3](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)). Shading and the dark and light areas of a face carry much of its identity.

### Features and configuration both matter

People can sometimes name a famous face from one feature alone, such as the eyes or the eyebrows. But when the top half of one famous face is joined to the bottom half of another, both people become hard to name. When the halves are shifted sideways so they no longer line up, both become easy to name again ([Sinha et al., 2006, result 4](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)). The whole face changes how each part is seen.

Absolute distances matter less than one might expect. Celebrity faces squeezed to 25% of their original width were recognised as well as the original faces ([Sinha et al., 2006, result 6](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)). The squeeze destroys the distance between the eyes and every ratio between width and height. It keeps the ratios of distances in the same direction, for example the ratio of the forehead's height to the nose's length. The authors suggest that the brain encodes these same-direction ratios ([same paper](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)).

### Which features carry the most identity

**Eyebrows.** Sadr, Jarudi and Sinha erased the eyebrows from 50 photographs of celebrities. People named the faces without eyebrows much less often than the originals. They also named them less often than faces without eyes ([Sadr, Jarudi and Sinha, Perception, 2003](https://pubmed.ncbi.nlm.nih.gov/12729380/)). The authors give reasons why eyebrows survive degradation. Eyebrows are large and high in contrast, they sit on a ridge that is less affected by shadow, and their own shape changes little when the face moves ([Sinha et al., 2006, result 5](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)). Among the other inner features, studies usually find the eyes most important, then the mouth, then the nose ([same paper](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)).

**A ranked list of 20 features.** Abudarham and Yovel built a face space of 20 named features and measured how well people notice a change in each one. They then tested which features change a face's identity when they are altered ([Abudarham and Yovel, Journal of Vision, 2016](https://people.socsci.tau.ac.il/mu/galityovel/files/2024/11/Reverseengineetingthefacespace2016.pdf)). The table below lists their measure of perceptual sensitivity for each feature, from the feature-tagging experiment and the feature-comparison experiment. A higher number means that people agree more about the feature and notice changes in it more easily.

| Feature | Tagging | Comparison |
|---|---|---|
| Lip thickness | 0.80 | 0.75 |
| Hair colour | 0.77 | 0.71 |
| Eye colour | 0.75 | 0.73 |
| Eye shape | 0.72 | 0.61 |
| Eyebrow thickness | 0.71 | 0.63 |
| Ear protrusion | 0.71 | 0.60 |
| Forehead height | 0.65 | 0.69 |
| Hair length | 0.71 | 0.57 |
| Eye size | 0.65 | 0.68 |
| Skin texture (marks, freckles, wrinkles) | 0.68 | 0.42 |
| Jaw width | 0.58 | 0.54 |
| Eyebrow shape | 0.58 | 0.47 |
| Nose size | 0.49 | 0.60 |
| Nose shape | 0.56 | 0.37 |
| Skin colour | 0.45 | 0.57 |
| Face proportion | 0.53 | 0.38 |
| Cheek shape | 0.41 | 0.57 |
| Chin shape | 0.54 | 0.12 |
| Distance between the eyes | 0.37 | 0.27 |
| Mouth size | 0.34 | 0.21 |

Four to six of the top features were enough to explain most of how similar people judged two faces to be. Reaching the same match took 16 of the lowest features. Face distances based only on the top five features (lip thickness, hair colour, eye colour, eye shape and eyebrow thickness) correlated with judged similarity at r = 0.62, about as well as all 20 features together ([Abudarham and Yovel, 2016](https://people.socsci.tau.ac.il/mu/galityovel/files/2024/11/Reverseengineetingthefacespace2016.pdf)). In a further experiment, the researchers swapped features of 15 faces with features copied from other faces. When they replaced the top features as far as possible, people judged the result to be a different person. When they replaced the five lowest features (mouth size, eye distance, face proportion, skin colour and nose size) just as far, people still saw the same person ([Abudarham and Yovel, 2016](https://people.socsci.tau.ac.il/mu/galityovel/files/2024/11/Reverseengineetingthefacespace2016.pdf)).

The authors warn that the ranking was measured on white faces. They give hair colour as an example of a feature that varies little among Asian or African faces, and that may therefore rank lower for them ([Abudarham and Yovel, 2016](https://people.socsci.tau.ac.il/mu/galityovel/files/2024/11/Reverseengineetingthefacespace2016.pdf)).

Several of the top features are surface features, not 3D shape. Hair colour, eye colour, eyebrow thickness and skin texture are all things a game paints in a texture or sets in a shader. Lip thickness, eye shape and forehead height are shape features, but in a low-polygon game head the painted texture also draws much of the lip edge, the eye outline and the hair line.

**Hair.** Ellis, Shepherd and Davies found that famous faces were named better from their internal features than from their external features. For faces seen only once, the two sets did equally well ([Ellis, Shepherd and Davies, Perception, 1979](https://moodle2.units.it/pluginfile.php/336418/mod_resource/content/0/Ellis79.pdf)). Hair still matters a great deal in practice. Toseeb, Keeble and Bryant showed faces with and without hair. When the hair stayed the same between learning and test, it made no difference. When the hair was added or removed between learning and test, accuracy fell sharply ([Toseeb, Keeble and Bryant, PLoS ONE, 2012](https://pmc.ncbi.nlm.nih.gov/articles/PMC3312903)). Sinha and Poggio's "presidential illusion" pasted Bill Clinton's inner features into Al Gore's head, and the picture still read as Gore. In the picture, only the head outline and the hair are Gore's. Sinha and Poggio argue that the brain makes strong use of this overall head shape ([Sinha and Poggio, Nature, 1996](https://www.nature.com/articles/384404c0.pdf)). Hair is also the easiest feature for a real person to change, which is one reason why it is less reliable for the brain.

### Shape against pigmentation

O'Toole, Vetter and Blanz split laser scans of heads into their 3D shape and their 2D colour map. They then made two sets of faces. In one set every face had its own colour map on the average shape. In the other set every face had its own shape with the average colour map. Sixty observers learned faces from one set and were tested at 0, 30 and 60 degrees of rotation ([O'Toole, Vetter and Blanz, Vision Research, 1999](https://shapemodelling.cs.unibas.ch/gravis-site-archive/publications/VisRes99.pdf)). Both kinds of information "contributed substantially", and neither alone matched the performance on the unaltered faces. For female faces the two were about equally useful at every angle. For male faces, shape was more useful, and the authors suggest that stubble in the scans hid part of the skin ([same paper](https://shapemodelling.cs.unibas.ch/gravis-site-archive/publications/VisRes99.pdf)). The same paper cites earlier work in which colour mattered more in front views and shape mattered more in angled views ([same paper](https://shapemodelling.cs.unibas.ch/gravis-site-archive/publications/VisRes99.pdf)).

Russell and Sinha repeated the question with familiar faces. People were asked to recognise colour photographs of their own friends, manipulated so that only the shape or only the pigmentation was useful. People recognised their friends better from pigmentation than from shape ([Russell and Sinha, Perception, 2007](https://cupola.gettysburg.edu/psyfac/1)). Sinha's review sums up the evidence across laser scans, artificial faces and morphed photographs. People use shape and pigmentation about equally ([Sinha et al., 2006, result 9](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)).

For a game, this means that a creator with perfect shape controls and a generic skin texture can at best reach about half of a likeness. The texture is not decoration. It carries about as much identity as the mesh.

### Caricatures and simplified faces

Rhodes, Brennan and Carey drew familiar faces as line drawings of 37 lines through 169 fixed points. They then exaggerated each face's differences from an average face (a caricature) or reduced them (an anti-caricature). People named the caricatures fastest, the true drawings next and the anti-caricatures slowest. Accuracy was the same for all three, and people rated the caricature as the best likeness ([Rhodes, Brennan and Carey, Cognitive Psychology, 1987](https://www.harvardlds.org/wp-content/uploads/2018/05/Rhodes-Identification-and-ratings-of-caricatures-implications-for-mental-representations-of-faces..pdf)). Lee, Byatt and Rhodes later found that caricaturing improves identification and anti-caricaturing worsens it ([Lee, Byatt and Rhodes, Psychological Science, 2000](https://acnpsearch.tweb-dev.unibo.it/singlejournalindex/76260235)). Sinha's review adds that the caricature effect is strongest in degraded images, such as line drawings and pictures flashed briefly ([Sinha et al., 2006, result 7](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)).

A low-polygon game head is a degraded image in exactly this sense. A creator for such heads should therefore let the player push a feature somewhat past what is realistic. A likeness built from a few strong, exaggerated cues will read better than a careful average.

### The other-race effect

People recognise faces of their own ethnic group better than faces of other groups. Meissner and Brigham's meta-analysis of 30 years of studies found that own-group faces produced more correct recognitions and fewer false alarms than other-group faces ([Meissner and Brigham, Psychology, Public Policy, and Law, 2001](https://works.bepress.com/christian_meissner/12)). The amount of contact with the other group reduced the effect only slightly. The correlation averaged over 29 studies was r = −0.13 ([Stelter et al., 2023, reporting Meissner and Brigham](https://rssclab.ucr.edu/wp-content/uploads/2026/06/Stelter-et-al-2023.pdf)).

This has two consequences for a creator. First, the features that make a face recognisable differ between groups, as Abudarham and Yovel warn for hair colour. Second, a creator built and tested by people of one group will tend to give fine control where that group looks, and coarse control elsewhere. Statistical face models share the bias. The 9,663 scans behind the LSFM model, for example, were 82% white ([Booth et al., CVPR 2016](https://www.doc.ic.ac.uk/~aroussos/Booth_etal_3DMMlearntFrom10000Faces_CVPR16.pdf)).

## 3. How many numbers a face needs

### What the published models use

Statistical face models give an upper bound on how many independent controls a face can usefully have. Each model below was built from scanned faces. The technology note already describes FaceGen's split of 50 symmetric shape numbers, 30 asymmetric shape numbers and 50 colour numbers, so it is listed here only for comparison.

| Model | Faces scanned | Shape numbers | Colour numbers | What the paper says about accuracy |
|---|---|---|---|---|
| Blanz and Vetter, 1999 | 200 young adults, about 70,000 vertices each | Up to 199 | Up to 199 | Fitting is done on the whole face, then again on 4 regions (eyes, nose, mouth and the rest) ([SIGGRAPH 1999](https://www.cs.princeton.edu/courses/archive/spring11/cos598A/pdfs/Blanz99.pdf)) |
| Blanz and Vetter, 2003 (face recognition) | Same 200 faces | 99 for the whole face, plus 99 for each of the 4 regions | Same as shape | 990 numbers in total. Identification was 95.0% correct on CMU-PIE side views and 95.9% on FERET ([IEEE TPAMI 2003](https://mi.informatik.uni-siegen.de/publications/blanz_tpami03.pdf)) |
| Basel Face Model, 2009 | 200 (100 women, 100 men), 53,490 vertices | 199 | 199 | Shape and colour are two independent models ([Paysan et al., AVSS 2009](https://shapemodelling.cs.unibas.ch/gravis-site-archive/publications/2009/BFModel09.pdf)) |
| FLAME, 2017 | Heads of about 3,800 body scans for identity | 90 or 300 | None in the base model | 90 identity components hold 98% of the variance, and 300 hold effectively 100%. 100 expression components hold 98% of the expression variance ([Li et al., ACM TOG 2017](https://download.is.tue.mpg.de/flame/flame_paper.pdf)) |
| LSFM, 2016 | 9,663 people | As many as needed for 99.5% of the variance | Not part of the shape model | The authors pick the number of components that explains 99.5% of the training variance ([Booth et al., CVPR 2016](https://www.doc.ic.ac.uk/~aroussos/Booth_etal_3DMMlearntFrom10000Faces_CVPR16.pdf)) |

The FLAME paper explains why the higher counts exist. A model with 49 identity components fits the overall shape of a scanned head well but cannot capture "localized person specific details". With 300 components, the fit error is close to zero millimetres over most of the face ([Li et al., 2017](https://download.is.tue.mpg.de/flame/flame_paper.pdf)).

I did not find a model paper that prints the number of components needed for exactly 90% or 95% of the variance. The papers publish these figures only as curves. The two numbers that are printed are the ones in the table. FLAME reaches 98% with 90 shape components ([Li et al., 2017](https://download.is.tue.mpg.de/flame/flame_paper.pdf)), and LSFM truncates at 99.5% ([Booth et al., 2016](https://www.doc.ic.ac.uk/~aroussos/Booth_etal_3DMMlearntFrom10000Faces_CVPR16.pdf)). Because the variance per component falls off quickly, 90% is reached with far fewer components than 98%, but the exact count depends on the model.

### Variance is not likeness

These numbers measure geometric error in millimetres on 50,000-vertex scans. They do not measure whether a viewer recognises the person. Two results from section 2 show that the gap is large.

- People recognise familiar faces at 7 × 10 pixels ([Sinha et al., 2006](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)). A model that only gets the broad shape right already passes this test, if the colour is right too.
- Four to six well-chosen features explain as much of judged similarity as all 20 of Abudarham and Yovel's features ([Abudarham and Yovel, 2016](https://people.socsci.tau.ac.il/mu/galityovel/files/2024/11/Reverseengineetingthefacespace2016.pdf)).

Principal components are also a poor choice of player control. Each one changes the whole face at once, and none has a plain name. FaceGen hides its 130 numbers behind named "linear controls" for this reason, as the technology note describes. Blanz and Vetter write that "the expressiveness of the model can be increased by dividing faces into independent subregions", and they used 4 regions (the eyes, the nose, the mouth and the rest of the face) from the same 200 scans ([Blanz and Vetter, 1999](https://www.cs.princeton.edu/courses/archive/spring11/cos598A/pdfs/Blanz99.pdf)). Regional controls with names are the right shape for a player, even when the numbers under them come from statistics.

### Shape against texture in morphable models

The morphable model papers keep shape and colour as two separate models of equal size. Blanz and Vetter used 99 shape numbers and 99 colour numbers per region ([Blanz and Vetter, 2003](https://mi.informatik.uni-siegen.de/publications/blanz_tpami03.pdf)). The Basel Face Model has 199 of each ([Paysan et al., 2009](https://shapemodelling.cs.unibas.ch/gravis-site-archive/publications/2009/BFModel09.pdf)). FaceGen has 80 shape numbers and 50 colour numbers, as the technology note records. The perception experiments that swap one for the other, described in section 2, find that people rely on both about equally ([O'Toole, Vetter and Blanz, 1999](https://shapemodelling.cs.unibas.ch/gravis-site-archive/publications/VisRes99.pdf); [Russell and Sinha, 2007](https://cupola.gettysburg.edu/psyfac/1)).

None of these colour models include hair. The Basel team removed the hair from the scans by hand and filled the gap ([Paysan et al., 2009](https://shapemodelling.cs.unibas.ch/gravis-site-archive/publications/2009/BFModel09.pdf)). Yet hair colour and hair length are among the most noticed features in Abudarham and Yovel's list. A game must give hair its own controls, outside any face model.

## 4. Games and tools that recreate real people

This section groups the products by how the player gets the face in. Many official pages from 2K, Apple and Epic refused automated fetches while this note was written. Where that happened, a press source stands in and is marked as such.

### Photo and camera import in sports and shooter games

**Tiger Woods PGA Tour 08 (EA Sports, 2007).** The "Photo Game Face" feature built a 3D face from photographs. EA recommended two photos, one from the front and one in profile. Players took them with the PlayStation 3 camera or the Xbox Live Vision camera, or uploaded a JPEG to EA's GamerNet website. The face was then placed on a golfer whose body came from the normal creator (press source, [Worth Playing](https://worthplaying.com/article/2007/9/17/reviews/45122-ps3-review-tiger-woods-pga-tour-08/)). A Globe and Mail reviewer waited nearly 20 minutes for his two photos to be processed. He called the result "freakily accurate" and said it reproduced small moles and faint wrinkles (press source, [The Globe and Mail](https://www.theglobeandmail.com/technology/if-only-tiger-woods-pga-tour-08-could-take-a-mulligan/article20401579/)). The Worth Playing reviewer used one phone photo, and his three-year-old son recognised him in the game at once (press source, [Worth Playing](https://worthplaying.com/article/2007/9/17/reviews/45122-ps3-review-tiger-woods-pga-tour-08/)). GameRevolution found that many faces looked right and many did not (press source, [GameRevolution](https://www.gamerevolution.com/review/38823-tiger-woods-08-review-2)).

**Rainbow Six Vegas (Ubisoft, 2006).** On Xbox 360, the player took two photos with the Xbox Live Vision camera, one from the front and one from the side. The game then built the multiplayer character's head. Microsoft's Xbox Wire said the process gave "some surprising results", and that its main benefit was recognising co-op teammates by their faces ([Xbox Wire](https://news.xbox.com/en-us/2006/11/25/rainbow-six-vegas-face-mapping/)). The camera face-mapping technology was licensed from a company called Digimask. It also appeared in Rainbow Six Vegas 2, Tiger Woods 08 to 12, Fight Night Round 4, FIFA 10 and Pro Evolution Soccer 2008 to 2011, among others (fan source, [Wikipedia, Xbox Live Vision](https://en.wikipedia.org/wiki/Xbox_Live_Vision)).

**EverQuest II (2012) did not build faces from photos.** Its SOEmote feature used a webcam to track 64 points on the player's face and drive the expressions of their existing avatar (press source, [Tom's Hardware](https://www.tomshardware.com/news/SOEmote-EverQuest-SOE-MMORPG-Live-Driver,16701.html); [Engadget](https://www.engadget.com/2012/08/07/sony-takes-soemote-live-for-everquest-ii/)). I found no evidence of a photo-to-face feature in EverQuest II.

**EA Sports Game Face (about 2011 to 2013).** The player uploaded a photo to EA's website, which generated a 3D face for use in Madden, FIFA and other EA games. An iOS app took a front and a side photo, and the player then edited the face in a browser-based Game Face Creator (press sources, [Bleacher Report](https://bleacherreport.com/articles/1303279-madden-13-ea-sports-improved-game-face-app-creates-awesome-gamer-likenesses); [TouchArcade](https://toucharcade.com/games/ea-sports-game-face-3d-avatar)). Press reports say FIFA dropped photo upload after FIFA 16 (press source, [Dexerto](https://www.dexerto.com/ea-sports-fc/ea-fc-27-leak-reveals-you-can-scan-yourself-into-the-game-for-first-time-in-a-decade-3394992/)).

**NBA 2K face scan (from NBA 2K15, 2014).** NBA 2K15 scanned the player's face with the PlayStation Camera or Kinect. The player held still about 15 to 30 cm from the camera and turned their head slowly, then fine-tuned the result in the game (press source, [Engadget](https://www.engadget.com/2014/09/17/nba-2k15-scans-your-face-thinks-youre-beautiful/)). Results varied widely. CinemaBlend wrote that "when it doesn't work, the results are terrifying" (press source, [CinemaBlend](https://www.cinemablend.com/games/NBA-2K15-Face-Scans-Can-Horrifying-67716.html)). The scan has since moved to the MyNBA2K phone app. The app asks for a plain background, good light, and no hat or glasses, and it shows lights that must all turn green before upload (press source, [Operation Sports](https://www.operationsports.com/how-to-face-scan-your-myplayer-in-nba-2k26)). Players still report warped scans (press source, [Windows Report](https://windowsreport.com/2k26-face-scan-not-working/)). 2K's own support page is at [support.nba2k.com](https://support.nba2k.com/hc/en-us/articles/42220890638227-NBA-2K-How-To-Scan-Your-Face), but it refused automated access.

**WWE 2K Image Uploader (WWE 2K19 onward).** The player uploads an image on 2K's website and downloads it in the game through the Image Manager ([2K support, refused automated access](https://support.wwe2k.com/hc/en-us/articles/42408597808019-WWE-2K-Image-Uploader-Tool-and-Community-Creations)). In the creator, the player picks Face, then Face Photo, and lines the photo up on the head by hand (press source, [Prima Games](https://primagames.com/gaming/wwe-2k22-caw-create-a-superstar-guide)). The photo becomes the face texture. The player still shapes the head with sliders. Guides say the photo must show a neutral expression, or the texture distorts (press source, [Sportskeeda](https://www.sportskeeda.com/esports/wwe-2k23-guide-how-upload-face-create-superstar)). Community creators also share edited "face textures" through the same uploader (press source, [Operation Sports](https://www.operationsports.com/wwe-2k25-face-texture-template-guide)). WWE 2K is the clearest commercial case of texture-only likeness on top of slider-shaped geometry.

### Photo import in tools

**FaceGen Modeller PhotoFit.** PhotoFit needs one front photo, and profile photos are optional. The user drags crosshairs onto the pupils, the outer cheekbones, the nose wings, the mouth corners, the jaw corners and the bottom of the chin ([FaceGen, Create from photo](https://facegen.com/man_create_photo.htm)). The photo must show a closed mouth, a neutral expression, eyes looking ahead, flash lighting, no hair over the lower forehead, no glasses and no long facial hair. Areas painted pure green are ignored and filled in by the model ([same page](https://facegen.com/man_create_photo.htm)). Modeller has more than 150 controls for editing the result ([FaceGen Modeller](https://facegen.com/modeller.htm)). FaceGen makes no hair ([Hilpert et al., 2024, arXiv](https://arxiv.org/pdf/2408.13084)).

**Epic MetaHuman.** Mesh to MetaHuman builds a MetaHuman "Identity" from a 3D mesh or from video ([Epic, Mesh to MetaHuman](https://dev.epicgames.com/documentation/metahuman/mesh-to-metahuman)). The mesh can have any topology. Its eyes must be open with the whites visible, and it should carry the skin colour texture ([Epic, From mesh](https://dev.epicgames.com/documentation/metahuman/from-mesh)). Video must come from an iPhone or iPad with a TrueDepth depth camera, or from a pair of stereo cameras. Epic warns that the solve "will produce good results only when the markers are active and well-tracked" ([Epic, From video footage](https://dev.epicgames.com/documentation/metahuman/from-video-footage)). MetaHuman Animator animates a face from video. Since Unreal Engine 5.6 it works in real time from webcams and many phones, but its offline depth solve still needs a TrueDepth or stereo camera ([Epic, MetaHuman Animator](https://dev.epicgames.com/documentation/metahuman/metahuman-animator)). I found no official MetaHuman path from an ordinary photo to a face.

**Ready Player Me.** Ready Player Me built a 3D avatar from one selfie. The user then edited hair, eyebrows, eyes, colours, glasses and clothes (press source, [80.lv, 2021](https://80.lv/articles/creating-3d-avatars-from-a-single-selfie)). Netflix acquired the company in December 2025, and all Ready Player Me services shut down on 31 January 2026 (press source, [PocketGamer.biz](http://www.pocketgamer.biz/netflix-acquires-avatar-creation-platform-ready-player-me/)).

### Stylised avatars with few parts

**Nintendo Mii.** Nintendo's Mii Maker can start a Mii from a photo. The player first picks gender, skin tone, eye colour, hair colour and hairstyle by hand. The console then takes a photo and offers preset faces built from Nintendo's set of face parts. The player can edit every feature afterwards ([Nintendo Support, Wii U](https://en-americas-support.nintendo.com/app/answers/detail/a_id/1719/~/how-to-create-a-mii-from-a-photo)). Nintendo calls the camera "the easiest way to make a Mii" ([Nintendo UK](https://www.nintendo.com/en-gb/Hardware/Nintendo-3DS-Family/Instant-Software/Mii-Maker/Mii-Maker-115540.html)). The photo never sets the hair or the colours. A family games reviewer said that a photo start still needed about half an hour of adjustment (press source, [GamePeople](https://www.gamepeople.co.uk/family_3ds_miimaker.htm)).

**Apple Memoji.** Apple says the player can "choose skin color and freckles, hairstyle and color, facial features, headwear, glasses, and more" ([Apple Support](https://support.apple.com/guide/watch/memoji-apdfb5de8996/watchos)). The editor has 11 tabs, which are skin, hairstyle, brows, eyes, head, nose, mouth, ears, facial hair, eyewear and headwear. There is no photo import. A Popular Science guide says that skin tone, hair colour and eye colour are where the editor gives the most accuracy, and that facial hair is the most distinctive feature when the person has it (press source, [Popular Science](https://www.popsci.com/story/diy/make-great-accurate-memoji/)).

Mii and Memoji show the perception results of section 2 in practice. Both reach a recognisable likeness with a hairstyle, a hair colour, a skin tone, an eyebrow choice and a few exaggerated parts. I found no peer-reviewed study that measures how often people recognise a Mii.

### Slider and sculpting creators

The sculpting note already describes how Black Desert, Fallout 4, The Sims 4 and MetaHuman edit faces. This part covers only how well players recreate real people with them.

- **Black Desert Online.** Players recreated Anne Hathaway, Emilia Clarke, Viggo Mortensen and others before the Western launch (press source, [Kotaku](https://kotaku.com/famous-celebrities-recreated-in-stunning-mmo-1671136546)). The official Customization Album, where players share and download appearances, has its own "Celebrity" category ([Black Desert, Beauty Album](https://blackdesert.pearlabyss.com/TR/en-us/Game/BeautyAlbum)).
- **Dragon's Dogma 2.** Capcom released the character creator as a free download before launch in March 2024 (press source, [80.lv](https://80.lv/articles/capcom-released-free-character-creator-storage-for-dragon-s-dogma-2)). Players posted recreations of Matt Damon, Zendaya and Clint Eastwood (press source, [VideoGamer](https://www.videogamer.com/news/dragons-dogma-2-character-creator-celebrity-faces/)).
- **Elden Ring.** Players recreated Keanu Reeves, Ringo Starr and David Schwimmer (press source, [PC Gamer](https://pcgamer.com/elden-ring-players-are-recreating-everyone-from-keanu-reeves-to-kratos-as-their-tarnished)).
- **Fallout 4.** Players made Arnold Schwarzenegger, Liam Neeson and Walter White, with mixed results (press source, [PCGamesN](https://pcgamesn.com/fallout-4/fallout-4-character-creator-does-famous-folk-the-good-the-bad-and-the-uncanny)). A launch guide said that changing the hairstyle and hair colour "can drastically change the appearance", and advised tuning the size, angle and position of the eyes and brows (press source, [SiliconANGLE](https://siliconangle.com/blog/2015/11/17/how-to-make-a-fallout-4-character-that-isnt-butt-ugly/)).
- **Saints Row (2022).** The free Boss Factory creator let players share and upvote characters across platforms. Players made Morbius, Shaggy and David Tennant (press source, [ComicBook.com](https://comicbook.com/gaming/news/saints-row-boss-factory-morbius-shaggy-character-creator)).
- **Mass Effect.** Mass Effect 2, Mass Effect 3 and the Legendary Edition share faces as 36-character codes. The Legendary Edition does not always reproduce an imported face exactly (press source, [GameWatcher](https://www.gamewatcher.com/news/mass-effect-legendary-edition-character-creation-face-codes-import)).
- **The Sims 4.** Players on EA's forum say that Create a Sim cannot reproduce a crooked nose, a dimple in a set place or a particular eyebrow shape. They say that Sims of real people "still look like Sims", and they use custom eyebrows, skin overlays and eyes for realism (fan source, [EA Forums](https://forums.ea.com/discussions/the-sims-4-general-discussion-en/help-with-creating-a-sim-that-looks-like-someone/570885)).
- **Baldur's Gate 3.** The creator offers preset faces only. Larian explained that characters tuned with many sliders "often end up looking alike" (press source, [80.lv](https://80.lv/articles/larian-explained-why-there-are-no-face-sliders-in-baldur-s-gate-3)). I found no lookalike culture around it comparable to the games above.

Most players do not build lookalikes themselves. Research on Chinese MMO creators found that players who are not artists "don't know where to start", and fall back on presets or imported share codes (press source, [Game Developer](https://www.gamedeveloper.com/design/how-to-help-non-professional-players-better-customize-their-character-faces-)). Share codes and share hubs are how most lookalikes spread.

### Studies that measure avatar likeness

- **Photo to creator sliders.** Shi and colleagues at NetEase trained a network to turn a photo into a game's face creator parameters. They report that it ran in a live game and was used more than a million times ([Shi et al., ICCV 2019](https://openaccess.thecvf.com/content_ICCV_2019/html/Shi_Face-to-Parameter_Translation_for_Game_Character_Auto-Creation_ICCV_2019_paper.html)).
- **Scanned avatars.** Fysh and colleagues built 120 avatars from handheld 3D scans. People who knew the scanned people recognised about 88% of them from the avatars and 92% from photographs ([Fysh et al., Behavior Research Methods, 2021](https://eprints.whiterose.ac.uk/178070/1/Fysh2021_Article_AvatarsWithFacesOfRealPeopleAC.pdf)).
- **Hand-made avatars in MetaHuman.** Hilpert and colleagues had each of 33 participants' most similar avatar built by hand in MetaHuman over a photo. The avatars scored only 3.34 on a physical similarity scale. Moving five features (chin shape, jaw width, eye shape, lip thickness and nose shape) halfway away from the participant's own values caused no significant drop in similarity. Moving them all the way did ([Hilpert et al., 2024, arXiv preprint](https://arxiv.org/pdf/2408.13084)).
- **Look-alike avatars and the brain.** Gonzalez-Franco and colleagues built hairless look-alike avatars from three photos. Participants rated them less realistic than photos, but the brain's response to one's own face still appeared, and it was stronger for avatars that participants rated as more alike ([Gonzalez-Franco et al., Frontiers in Human Neuroscience, 2016](https://pmc.ncbi.nlm.nih.gov/articles/PMC4971066/)).

### What the products have in common

- Every photo import asks for the same conditions. These are a front photo (often plus a profile), a neutral expression, even or flash light, no glasses and no hair over the forehead. NBA 2K's failures from 2014 to today mostly come from breaking these conditions.
- Every photo import is a starting point. Tiger Woods, NBA 2K, Mii Maker and FaceGen all expect the player to adjust the face afterwards.
- The photo rarely sets the hair. Mii Maker asks for hair by hand, FaceGen makes none, and Gonzalez-Franco's avatars were hairless.
- The praised results came from the texture. The Tiger Woods reviewer praised the moles and wrinkles, which are pigmentation, not geometry.

## 5. Texture against geometry on low-polygon heads

### What a KOTOR head can show

The sculpting note measured KOTOR's player heads at 733 to 893 triangles in the skinned face mesh. For this note I also read the size of the head textures from the games' high-quality texture packs (`swpc_tex_tpa.erf`), with a throwaway script that was not added to the repository.

- In KOTOR 1, every player head texture checked is 256 × 256 pixels. This covers all 30 heads named `PMHA01` to `PMHC05` and `PFHA01` to `PFHC05`, plus Bastila's and Carth's heads.
- In KOTOR 2, 16 of the 23 player head textures checked are 512 × 512 pixels, and the other 7 are 256 × 256 pixels.

The face itself takes up only part of each texture, because the same image also holds the scalp, the ears, the neck and often the hair. Even so, a 256 × 256 head texture gives the face far more pixels than the 19 × 27 pixels at which Yip and Sinha's viewers reached their best recognition of famous faces ([Sinha et al., 2006, result 1](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)). Texture resolution is therefore not what limits a likeness on KOTOR's heads. What limits it is which features the player can control.

### Where each top feature lives on a KOTOR head

The table takes the most noticed features from Abudarham and Yovel's list in section 2 and says where each one lives on a KOTOR-era head. The last column says whether the sculpting note's designs can change it.

| Feature | Where it lives | Can the sculpting designs change it? |
|---|---|---|
| Lip thickness | Mostly the painted lip edge in the texture, and a little the lip vertices | Partly. Design A moves the lip bones `f_um_g`, `f_llm_g` and `f_rlm_g`. The painted lip edge stays where it is. |
| Hair colour | The hair texture, or the hair part of the head texture | No. It needs a tint mask, as the technology note's option A proposes. |
| Eye colour | The eye mesh's texture | No. It needs an iris tint. |
| Eye shape | The eyelid meshes and the painted eye outline | Partly. Design A can move the bones under the eyes. |
| Eyebrow thickness and shape | Painted in the head texture | No. The bones under the brows move the skin, not the painted hairs' thickness. |
| Ear protrusion | Ear vertices | Design B only, if a region is placed on the ears. |
| Forehead height | The hair line, which is painted or modelled in the hair | Design B can move the forehead skin, but the hair line comes with the hairstyle. |
| Hair length | The hair mesh | No. It needs a choice of hairstyles. |
| Eye size | The eyelid meshes | Partly, with design A's eye region. |
| Skin texture (freckles, marks, wrinkles) | The texture | No. It needs texture layers. |
| Jaw width | Jaw vertices | Yes. Design A scales `f_jaw_g` sideways. |

Six of these eleven features cannot be changed by any amount of geometry work. They need texture layers, tints or alternative meshes for hair. This matches the products in section 4. WWE 2K lets players put a photo on the face as a texture, and its community trades face textures. Mii and Memoji make hair, skin and eyebrows separate choices. FaceGen and MetaHuman do not make hair from photos. In two studies of avatars built from photos, one of which used MetaHuman, people rated the resulting avatars as only moderately similar to themselves or as less realistic than photographs ([Hilpert et al., 2024](https://arxiv.org/pdf/2408.13084); [Gonzalez-Franco et al., 2016](https://pmc.ncbi.nlm.nih.gov/articles/PMC4971066/)).

### Painted shading on old heads

KOTOR's head textures have light and shadow painted into them, as the sculpting note records. This has two effects on likeness.

- **It helps texture-only likeness.** Painted shading carries the dark and light pattern that people use to recognise a face. Line drawings without that pattern are recognised far less often, at 47% against 90% for photographs in Davies's study ([Sinha et al., 2006, result 3](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)). A new face texture with a nose shadow, eye sockets and cheek shading in the right places can change the perceived shape of a low-polygon head more than a small mesh change.
- **It fights geometry edits.** When design A or B moves a feature, the painted shadow moves with the vertices only as far as the texture stretches. A large edit leaves the shadow in a place that contradicts the new shape. This is another reason for the small limits that the sculpting note proposes.

### Photo projection

A photo-projection feature takes a photograph of a real face and turns it into the head's texture, and sometimes also its shape. There are three levels.

1. **Texture only, placed by hand.** This is the WWE 2K approach. The player lines a photo up on the existing head (press source, [Prima Games](https://primagames.com/gaming/wwe-2k22-caw-create-a-superstar-guide)). It is simple, and it brings in the pigmentation that carries much of identity. It also brings in the photo's lighting, its shadows and its expression, which is why guides insist on a neutral face and even light (press source, [Sportskeeda](https://www.sportskeeda.com/esports/wwe-2k23-guide-how-upload-face-create-superstar)).
2. **Shape and texture fitted from landmarks.** This is the FaceGen PhotoFit approach. The user marks about a dozen points, such as the pupils and the mouth corners, and the software fits a statistical model to them ([FaceGen](https://facegen.com/man_create_photo.htm)). The Tiger Woods and Rainbow Six Vegas features worked from one front and one profile photo in the same spirit ([Xbox Wire](https://news.xbox.com/en-us/2006/11/25/rainbow-six-vegas-face-mapping/)).
3. **Full analysis by synthesis.** This is the Blanz and Vetter approach. The software renders the morphable model with an estimated light and pose, and changes the model's numbers until the render matches the photo. It recovers the shape, the colour without the photo's lighting, and the lighting itself ([Blanz and Vetter, 1999](https://www.cs.princeton.edu/courses/archive/spring11/cos598A/pdfs/Blanz99.pdf)). In 2003 the fit took 4.5 minutes on a 2 GHz Pentium 4 ([Blanz and Vetter, 2003](https://mi.informatik.uni-siegen.de/publications/blanz_tpami03.pdf)).

Levels 2 and 3 need a statistical face model with a shared topology. kjk has no such model for KOTOR's heads, as the technology note explains. Level 1 needs only a mapping from the photo to each head's texture layout. KOTOR's heads do not share one texture layout, so kjk would need a mapping for each head, or one new texture layout shared by refitted heads.

## 6. Summary of the findings

- People recognise faces they know from very little detail. More than half of famous faces were named at 7 × 10 pixels, and recognition reached its ceiling at 19 × 27 pixels ([Sinha et al., 2006](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)). A low-polygon head with a 256 × 256 texture has enough detail for a likeness.
- Pigmentation carries about as much identity as 3D shape. With their friends' faces, people did better from pigmentation than from shape ([O'Toole, Vetter and Blanz, 1999](https://shapemodelling.cs.unibas.ch/gravis-site-archive/publications/VisRes99.pdf); [Russell and Sinha, 2007](https://cupola.gettysburg.edu/psyfac/1)).
- A few features carry most of the identity. In Abudarham and Yovel's study, lip thickness, hair colour, eye colour, eye shape and eyebrow thickness explained judged similarity as well as all 20 features together. Changing mouth size, eye distance, face proportion, skin colour and nose size did not change who people saw ([Abudarham and Yovel, 2016](https://people.socsci.tau.ac.il/mu/galityovel/files/2024/11/Reverseengineetingthefacespace2016.pdf)).
- Eyebrows matter more than intuition suggests. Removing them hurt recognition of celebrities more than removing the eyes ([Sadr, Jarudi and Sinha, 2003](https://pubmed.ncbi.nlm.nih.gov/12729380/)).
- Hair matters more at low detail and whenever it changes. External features overtake internal ones as resolution falls ([Jarudi and Sinha, 2003](https://dspace.mit.edu/handle/1721.1/7274)), and changing the hair between learning and test sharply lowers accuracy ([Toseeb, Keeble and Bryant, 2012](https://pmc.ncbi.nlm.nih.gov/articles/PMC3312903)).
- Exaggeration helps. Caricatures are recognised at least as well as true faces, and faster, especially in degraded images ([Rhodes, Brennan and Carey, 1987](https://www.harvardlds.org/wp-content/uploads/2018/05/Rhodes-Identification-and-ratings-of-caricatures-implications-for-mental-representations-of-faces..pdf); [Sinha et al., 2006](https://mplab.ucsd.edu/users/marni/Igert/19results_sinha_06.pdf)).
- Moderate shape errors go unnoticed. Moving five shape features halfway away from a person's real values did not lower rated similarity ([Hilpert et al., 2024](https://arxiv.org/pdf/2408.13084)).
- Statistical models use far more numbers than likeness needs. FLAME holds 98% of shape variance with 90 components ([Li et al., 2017](https://download.is.tue.mpg.de/flame/flame_paper.pdf)). These counts measure millimetres of error on dense scans, not recognition, and they leave out hair.
- Photo import in shipped games is a starting point, not a result. Every product asks for a neutral front photo in even light, often with a profile, and every one expects the player to correct the face by hand afterwards. The best-reviewed results came from the photo texture, and hair was always chosen separately.
- The surface features that carry the most identity on an old head are paint, not geometry. These are the hair colour, eye colour, eyebrows, facial hair and skin marks. Six of the eleven most noticed features in Abudarham and Yovel's list cannot be changed by moving vertices on a KOTOR-era head.
