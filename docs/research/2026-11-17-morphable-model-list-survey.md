# Survey of the curated list of 3D morphable model software and data

This document was researched on 2026-11-17.
It goes through every entry of the curated list of 3D morphable model software and data (https://github.com/3d-morphable-models/curated-list-of-awesome-3D-Morphable-Model-software-and-data).
It follows the earlier survey in `docs/research/2026-11-17-head-scan-datasets.md`.
Entries that the earlier survey already covered are only mentioned here when the list or a primary source shows something new.

The document answers two questions.
The first question is which sets of real 3D heads a private individual can obtain for the face editor benchmark.
The second question is which shape models have a licence that lets derived shapes ship inside this MIT-licensed repository.

A few terms are used throughout.
A 3D morphable model, or 3DMM, is a statistical model of a shape: an average shape plus a set of directions in which the shape can vary.
PCA means principal component analysis, which is the usual method to find those directions.
A registered mesh is a scan that was refitted so that every person has the same vertices in the same order.
Non-rigid registration is the process that bends a template mesh onto a scan to make a registered mesh.
NICP means non-rigid iterative closest point, which is the most common non-rigid registration method.
A local or part-based model is a model whose shape directions each move only one region of the face, such as the nose or the lips.

Every claim carries the URL of the page it was read on.
The words "not verified" mark every claim that could not be confirmed on a primary source.
Nothing in this document is legal advice.

## Summary table of the new relevant finds

| Name | People | How to get it | Can a private individual get it | Licence in a few words | Useful for |
|---|---|---|---|---|---|
| FaceVerse model v0 to v3 | Learned from East Asian subjects, 2,000 head scans and 60,000 RGB-D images | Open Google Drive links in the GitHub repository | Yes | BSD-2-Clause on the repository, model files not named separately | B, with a question to the authors |
| FaceVerse model v4 | Same source | OneDrive link in the GitHub repository | Yes (link not verified) | MIT on the repository | B, with a question to the authors |
| MeshMonk | No data | Open GitHub repository and PyPI | Yes | Apache-2.0 | Registration tool |
| menpo3d NICP | No data | Open GitHub repository | Yes | BSD-3-Clause | Registration tool |
| scalismo and scalismo-faces | No data | Open GitHub repository | Yes | Apache-2.0 | Registration and model building tool |
| FUSS (Bernard group) | No data | Open GitHub repository | Yes | MIT | Model building tool (not inspected) |
| 3D Growth Curves | Learned from 5,443 people aged 0.05 to 88, all of European ancestry | Open GitHub repository | Yes | GPL-3.0 on the repository | B only with permission, children and elderly |
| S3DFM | 77 people aged 16 to 73 | Direct download | Yes | CC BY-NC-ND, academic research only | A, weak (front face depth maps) |
| Universal Head Model (UHM) | Learned from about 10,000 faces and 1,212 heads | Agreement signed by academic staff | No | CC BY-NC 4.0 and non-commercial agreement | Neither |
| LSFM | 8,402 people, with models for children under 7 and adults over 50 | Application by academic staff for a medical use | No, and the link is dead | Non-commercial academic | Neither |
| Bolkart's BU-3DFE models (global, local wavelet, multilinear) | Learned from 100 people aged 18 to 70 | Open GitHub repositories | Yes | Non-commercial research only | Local benchmark aid only |
| Surrey Face Model | Learned from 169 people | Direct download (3,448 vertices) or registration | Yes for the small model | Non-commercial | Local benchmark aid only |
| York Ear Model | Learned from 10 pairs of ear scans and 500 ear images | Agreement signed by academic staff | No | Non-commercial research | Neither |
| INFACE | Infants, 2,394 scans | Email to the authors, academics only | No | Research only | Neither |
| UoY 3D face dataset | About 350 people | Agreement signed by academic staff | No | Non-commercial research | Neither |

## Short answer

The list adds no new truly open set of real head scans.
"Truly open" here means CC0, CC BY, public domain, MIT, BSD, Apache or a similar permissive licence.
The only new set of real faces that a private individual can download today is S3DFM.
S3DFM has 77 people, but its data is front-facing depth maps under a no-derivatives, academic-only licence.

The list adds one new candidate for shippable shapes.
That candidate is the FaceVerse face model, whose repositories are under BSD-2-Clause and MIT.
The FaceVerse model was learned from a dataset whose own licence is strict, so a written confirmation from the authors is advisable before shipping anything derived from it.

The list adds several permissive registration tools.
MeshMonk (Apache-2.0), menpo3d NICP (BSD-3-Clause) and scalismo (Apache-2.0) are the useful ones.

The list adds one important source of children's and elderly face shapes.
3D Growth Curves gives an average face and a PCA model for every age from 0.5 years to about 69 years, for each sex.
Its repository is under GPL-3.0, so shipping it in an MIT repository needs the authors' permission.

The only local or part-based face models on the list are research-only or AGPL.
They are Bolkart's local wavelet models and Bernard's local support factors.

The FLAME 2023 Open licence was re-checked and is confirmed as CC BY 4.0 with two use restrictions.
It is described in its own section below.

## FLAME 2023 Open licence, re-verified

The licence is on https://flame.is.tue.mpg.de/modellicense.html.
That page holds two licences.
The first is the Academic Model License.
It says "The following license applies to these models only: FLAME 2017, FLAME 2019, FLAME 2020, FLAME 2023, FLAME Blender Add-on (FLAME 2020)".
The second is the Open Model License.
It says "The following license applies ONLY to the FLAME 2023 Open model; all other models are covered by the non-commercial license above."

The Open Model License links the words "Creative Commons Attribution license" to https://creativecommons.org/licenses/by/4.0/.
Its grant reads: "The FLAME model is under a Creative Commons Attribution license. Hence, Max-Planck grants you the right to share and adapt the model for any purpose, including commercial use under following terms".
The terms are these, quoted in full.

- "You must give appropriate credit, provide a link to the license, and indicate if changes were made;"
- "You may not apply legal terms of technological measures that legally restrict others from doing anything the license permits."
- "The model may not be used for pornographic purposes or to generate pornographic material whether commercial or not. The Model may further not be used to create fake, libelous, misleading, or defamatory content of any kind excluding analyses in peer-reviewed scientific research."

The licence also asks the user to "cite the FLAME paper when reporting results with the Model."
The ban on pornographic and on fake or defamatory content goes beyond the standard CC BY 4.0 text.
A repository that ships shapes derived from FLAME 2023 Open should therefore carry these two restrictions next to the credit line.
Derived shapes stay under these terms and do not become MIT.

The licence defines what it covers as "the FLAME 2023 Open model, and software (the "Model")".
The site news gives the date and the file name.
It says "11/25: Released FLAME 2023 Open Model with the Open Source License (CC-BY-4.0); previous versions are only available for non-commercial research purposes" (https://flame.is.tue.mpg.de/).
It also says "11/25: Released conversion code to translate the expression parameters from the FLAME 2023 (flame2023.pkl) model to the FLAME 2023 Open (flame2023_Open.pkl)" (https://flame.is.tue.mpg.de/).

What the Open model contains was checked as far as possible without logging in.
The download is a single archive at https://download.is.tue.mpg.de/download.php?domain=flame&sfile=FLAME2023Open.zip, and it returns a login page to an anonymous request.
The converter repository says the Open model has its own expression basis of 100 components, which differs from the FLAME 2023 expression basis (https://github.com/Suraj-Bhor/flame_to_flame_open_converter).
The converter's parameter files carry 300 shape components and 100 expression components for both models.
So the Open model covers the identity shape space and an expression space.
Whether its identity shape space is identical to that of FLAME 2023 is not verified.
A third-party note reports that the archive holds `flame2023_Open.pkl` and a file named `FLAME2023_Open Readme.pdf`, and that the mesh has 5,023 vertices with two separate eyeballs (https://github.com/Edderic/breathesafe/blob/main/python/dimension_pipeline/FLAME_OPEN.md).
That note is not a primary source, so these contents are not verified.

The texture is not covered by the Open licence.
The texture licence page says "The Texture is under a Creative Commons BY-NC-SA 4.0 license" (https://flame.is.tue.mpg.de/texturelicense.html).
That licence allows only non-commercial use and requires derived work to carry the same licence.
FLAME textures and anything derived from them therefore cannot ship in an MIT repository.

The earlier survey's claim is correct.
It should be read with one precision: CC BY 4.0 applies to the FLAME 2023 Open shape and expression model only, and not to the texture or to the other FLAME versions.

## Face models on the list

### FaceVerse model

The earlier survey covered the FaceVerse dataset, which needs a signature from academic staff.
The FaceVerse model is a separate download with a different licence.

The model was built "from hybrid East Asian face datasets containing 60K fused RGB-D images and 2K high-fidelity 3D head scan models" (https://github.com/LizhenWangT/FaceVerse).
All subjects are East Asian.
The README says the base model can "predict accurate rough 3D face models in different genders, ages, etc."
The age range and the number of distinct people are not verified.

There are five versions (https://github.com/LizhenWangT/FaceVerse).
Version 0 is the paper version.
Version 1 is the recommended face PCA model with detail generators.
Version 2 is "only the PCA base model", with expression components fitted "to the 52 blendshapes defined by Apple".
Version 3 is a full head with two eyeballs, according to the research agent's reading of the README (not verified).
Version 4 is in a separate repository and adds a network that predicts parameters from photos (https://github.com/LizhenWangT/FaceVerse_v4).

The files of versions 0 to 3 are on open Google Drive and Baidu links in the README.
A private individual can download them without a form.
The version 4 file `faceverse_v4_2.npy` is on OneDrive, and its link refused an automated request, so whether it works is not verified.

The repository of versions 0 to 3 is under the BSD-2-Clause licence, according to the GitHub licence detection (https://github.com/LizhenWangT/FaceVerse/blob/main/LICENSE).
The README says "Our License can be found in LICENSE."
The repository of version 4 is under the MIT licence (https://github.com/LizhenWangT/FaceVerse_v4/blob/main/LICENSE).
Neither README gives the model files a separate licence.

One doubt remains.
The FaceVerse dataset licence is non-commercial, names video games, and forbids sharing "any portion of derived data" (https://github.com/LizhenWangT/FaceVerse-Dataset).
The model was learned from that kind of data.
Nothing says whether the dataset clause reaches the published model.
A written confirmation from the authors (Lizhen Wang and Yebin Liu at Tsinghua University) would settle it.

FaceVerse could serve (B) if the authors confirm.
It covers only East Asian faces, so it would add variety to a model like ICT FaceKit rather than replace it.

### Large Scale Facial Model (LSFM)

LSFM is a family of PCA face shape models built from the MeIn3D dataset of about 10,000 people (https://github.com/menpo/lsfm/blob/master/MODELS.md).
The general model uses 8,402 subjects.
The family includes models by age group.
The under-7 model has 493 subjects, the 7 to 18 model has 2,029, the 18 to 50 model has 5,106, and the over-50 model has 446.
It also includes models by sex and by ethnicity, with 7,138 White, 178 Black and 137 Chinese subjects.
The models are shape only, face only, with 53,215 vertices in the full version and 28,431 in the cropped version.

The old application page set three conditions (http://web.archive.org/web/20210512205421/https://xip.uclb.com/i/healthcare_tools/LSFM.html).
The requester had to be "a SENIOR RESEARCHER (i.e. a Faculty member … NOT a PhD student or a postdoctoral researcher)".
The use had to be "a MEDICALLY ORIENTED APPLICATION".
The request had to come from an academic email address.
The licence was the "LSFM Non-Commercial Licence 1.00", "For non-commercial academic use only", for 12 months.

The link on the list is dead.
https://xip.uclb.com/i/healthcare_tools/LSFM.html redirects to a 404 page.
The iBUG page says "applications should be made by an academic member of staff or manager" (https://ibug.doc.ic.ac.uk/resources/lsfm/).
A private individual cannot obtain LSFM.

### Universal Head Model (UHM)

UHM is a full head model with the face, the cranium, the ears, the eyes, the teeth and the tongue (https://ar5iv.labs.arxiv.org/html/1911.08008).
It combines LSFM, with about 10,000 faces, and LYHM, with 1,212 heads.
The ear part was learned from 254 ear meshes, including "113 children's ears acquired via CT scans".
The eye region comes from 72 meshes sculpted by an artist.
The teeth and the tongue are "not fitted to any training data", so they do not vary.

The README says "This agreement should be completed by a full-time academic staff member (not a student) or a craniofacial clinician" (https://github.com/steliosploumpis/Universal_Head_3DMM).
The agreement says "The UHM models are available for non-commercial research and education purposes only."
The repository LICENSE is CC BY-NC 4.0.
The Google Drive file `UHM_models_v2.zip` (2.2 GB) still downloads without a login, but the agreement still governs its use.
A private individual is not eligible, and derived shapes cannot ship.

### Statistical 3D Shape Models of Human Faces (Brunton, Bolkart, Wuhrer)

The gforge page on the list now redirects to GitHub.
The models are in three repositories by Timo Bolkart.

- https://github.com/TimoBolkart/GlobalLocalFaceModels holds a global PCA model and a local wavelet PCA model of neutral faces.
- https://github.com/TimoBolkart/MultilinearModelFitting holds a multilinear model of identity and expression.
- https://github.com/TimoBolkart/MultilinearWaveletModel holds a localized multilinear wavelet model of 43 MB.

A wavelet model splits the face into overlapping regions at several scales and learns a small model for each region.
These are the only local face models on the list that come with trained data.

All of them were learned from BU-3DFE, which has 100 adults aged 18 to 70 of several ethnicities (http://www.cs.binghamton.edu/~lijun/Research/3DFE/3DFE_Analysis.html).
They cover the face only.
The old page said "All face models were computed using the BU-3DFE face database" and that the BU-3DFE owners gave "permission to make the statistical models available for non-commercial research purposes" (http://web.archive.org/web/20210726042206/http://facepage.gforge.inria.fr/).
Each README says the models are "provided for NON-COMMERCIAL RESEARCH PURPOSES only" and that "The redistribution of the code is not permitted."
A private individual can download them directly.
Derived shapes cannot ship.
The method itself is published, so the project could build its own local wavelet model from shippable data.

### Multilinear Autoencoder (MAE)

The gforge page now redirects to https://github.com/vabrevaya/mae.
The repository holds Lua code and a template mesh, with no README text and no licence file.
The trained models were learned from BU-3DFE, Bosphorus and D3DFACS.
They were only on the old page, and the archived copy of that page does not keep the model files (http://web.archive.org/web/20200917112610/http://mae.gforge.inria.fr/).
The trained models are effectively lost.

### Surrey Face Model

The link on the list, https://cvssp.org/facemodel, returns 404.
The current page is https://cvssp.org/faceweb/3dmm/facemodels/.
The model was built from "169 high-quality 3D face images" (https://www.cvssp.org/faceweb/3dmm).
The age, sex and ethnicity of those people are not verified.
It covers the face only, at four resolutions from about 1,000 to 29,000 vertices.

The 3,448-vertex shape model is a direct download and is "free for non-commercial use".
The other resolutions need a registration form that asks for an organisation and "requires manual verification" (https://cvssp.org/faceweb/3dmm/facemodels/register.html).
A commercial licence with texture is negotiated with the University of Surrey.
The eos library ships the same 3,448-vertex model as `share/sfm_shape_3448.bin`, with 6 expression blendshapes (https://github.com/patrikhuber/eos/tree/master/share).
The eos README says the model "is free for use for non-commercial purposes" (https://github.com/patrikhuber/eos/blob/master/README.md).
Derived shapes cannot ship.

### Albedo Morphable Model (AlbedoMM)

AlbedoMM is a model of skin colour and shine, not of shape (https://github.com/waps101/AlbedoMM).
It was learned from 73 scans.
The README says it gives "permission to use the model and code only for academic research purposes."
It is not useful for (A) or (B).

### INFACE, INCRAN and INCLEFT

These are models of infants from ETH Zurich (https://github.com/tillns/INFACE).
INFACE is a face model, INCRAN is a model of the outer head and the skull, and INCLEFT covers cleft lip and palate.
The MICCAI 2024 page says INFACE was built from "2394 various 3D scans" (https://papers.miccai.org/miccai-2024/443-Paper2260.html).
The number of distinct infants is not stated.
The code is MIT, but the model files are not in the repository.
The README says "Access is only provided to academic researchers. Usage of the models is restricted solely to research purposes."
A private individual cannot obtain the models.

### ICT FaceKit at the new link

The list now points to https://github.com/VGL-Group/ICT-FaceKit.
That address redirects to https://github.com/USC-ICT/ICT-FaceKit.
Nothing has changed since the earlier survey.
The last commit is from 2020-12-01, the licence is still MIT, and the full model is still not released.

One fact is new compared with the earlier survey.
The paper behind the model describes its training data (https://ar5iv.labs.arxiv.org/html/2004.03458).
It used 79 light-stage subjects, "34 female, and 45 male, ranging from age 18 to 67", each in 26 expressions.
It also used 99 full-head scans from Triplegangers, each in 20 expressions.
That is about 178 people.
Whether the 100 shapes of the released Light model come from all of these people is not verified.

### York Ear Model and York 3D Ear dataset

The current page is https://www-users.york.ac.uk/~np7/research/YEM/.
The York Ear Model is a 3D model of the ear only.
It was started from 10 pairs of ear scans of the SYMARE database and extended with 500 annotated ear photographs.
The York 3D Ear dataset is 500 ear meshes reconstructed from those photographs, not real scans.
The page says the agreement "should be completed by a full-time academic staff member (not a student)".
The licence is "non-commercial research and education purposes only".
A private individual cannot obtain it.

## Registration and model-building tools on the list

These tools matter because the project does its own registration.

### MeshMonk

MeshMonk is not on the list, but it was found as the registration engine of 3D Growth Curves.
It describes itself as an "Open mesh-to-mesh registration framework" (https://github.com/TheWebMonks/meshmonk).
It is a C++20 library with Python bindings, installed with `pip install 'meshmonk[io]'`.
It offers rigid, non-rigid and multi-resolution registration.
The licence is Apache-2.0.
The last push was on 2026-10-22, so it is maintained.
It is the strongest permissive registration tool found in this survey.

### menpo3d NICP and the LSFM pipeline

The LSFM pipeline builds a 3DMM from a folder of face meshes (https://github.com/menpo/lsfm/blob/master/CODE.md).
It places 68 landmarks, registers each mesh with NICP, builds a PCA model, removes failed registrations and builds the model again.
The code is under BSD-3-Clause (https://github.com/menpo/lsfm/blob/master/LICENSE).
The last commit is from 2017, and it pins old Python packages.
It builds its template from the Basel Face Model, which is non-commercial.
CODE.md says "LSFM requires the Basel Face Model in order to prepare the facial template".

The NICP code itself is in menpo3d, in `menpo3d/correspond/nicp.py` (https://github.com/menpo/menpo3d/blob/master/menpo3d/correspond/nicp.py).
menpo3d is under BSD-3-Clause (https://github.com/menpo/menpo3d/blob/master/LICENSE.txt).
Its last commit is from 2024-05-06.
menpo3d ships a file `data/template.ply` whose origin is not verified.
It may be derived from the Basel Face Model, so the project should use its own template.

### Basel Face Registration pipeline, scalismo and scalismo-faces

The Basel pipeline reproduces the paper "Morphable Face Models – An Open Framework" (https://github.com/unibas-gravis/basel-face-pipeline).
It registers the BU-3DFE scans with a Gaussian process model and then builds a morphable model.
A Gaussian process model here is a statistical model of smooth deformations that the registration uses as a prior.
The pipeline is written in Scala and is under Apache-2.0.
It depends on scalismo-ui, which is under GPL-3.0 (https://github.com/unibas-gravis/scalismo-ui).
Its build uses Bintray, which shut down in 2021, so whether it still builds is not verified.
The reference mesh and the region masks come from the Basel Face Model 2017 page and fall under its non-commercial licence.

scalismo is under Apache-2.0, with a last commit on 2024-01-23 (https://github.com/unibas-gravis/scalismo).
scalismo-faces is under Apache-2.0, with a last commit on 2023-06-02 (https://github.com/unibas-gravis/scalismo-faces).
scalismo-faces ships no real face model, only small random test models.
Both are good references for Gaussian process registration.

### Bernard's tools

Florian Bernard's tools on the list are all under AGPL-3.0.

- sparsePdmFitting fits a shape model to a sparse point cloud, in MATLAB (https://github.com/fbernardpi/sparsePdmFitting).
- NmfSync makes matchings across many shapes agree with each other (https://github.com/fbernardpi/NmfSync).
- LocalSupportFactors learns shape deformations that each act on one region (https://github.com/fbernardpi/LocalSupportFactors).
- TransSync2 aligns many scans together (https://github.com/fbernardpi/TransSync2).

The download buttons on the author's site (https://sites.google.com/site/fbernardpi/code) have no link behind them, so the GitHub repositories are the working source.
LocalSupportFactors is the closest match to a part-based model method.
Its paper tests it on brain structures of 17 subjects and on 1,531 human bodies, not on faces (https://arxiv.org/abs/1510.08291).
The AGPL licence would require any program that uses it to be released under the AGPL as well, so it cannot be linked into an MIT project.

The licence of the auction algorithm on MATLAB File Exchange (https://de.mathworks.com/matlabcentral/fileexchange/48448-fast-linear-assignment-problem-using-auction-algorithm-mex) is not verified.

The same author has a newer MIT-licensed repository named FUSS, for unsupervised statistical shape model learning (https://github.com/fbernardpi/FUSS).
It is not on the list and was not inspected further.

### 3D Face Modeling from Diverse Raw Scan Data (3DFC)

3DFC learns a face model and dense correspondence from raw scans with a neural network (https://github.com/liuf1990/3DFC).
The repository has no licence file, so all rights are reserved.
The trained model is on Google Drive with no stated licence.
It is not usable here.

### Shape My Face (Registration by Surface-to-Surface Translation)

Shape My Face registers raw face scans with a network that combines mesh decoders with "a specialized PCA model of the mouth" (https://arxiv.org/abs/2012.09235).
The repository holds only a README that says "Code coming soon", and has no licence file (https://github.com/mbahri/smf).
Its last commit is from 2021-04-12.
It is not usable, but the idea of a separate mouth model fits the project's lip sub-system.

### 3D Growth Curves

3D Growth Curves gives an expected face shape for each age and sex, with a PCA model of normal variation around each one (https://github.com/harrymatthews50/3DGrowthCurves).
The paper says the curves were "trained from 5443 males and females aged between 0.05 and 88 years" (https://europepmc.org/article/PMC/PMC8190313).
The four source databases are AHEAD with 853 subjects, 3D Facial Norms with 1,886, IUPUI with 730 and PSU with 1,973.
All subjects are of European ancestry.
The paper says the source data was "collected without consent for broad data sharing and are not publicly available".
Only the derived curves are published.

The repository holds 229 female models from age 0.5 to 68.9 and 172 male models from age 0.5 to 51.8, in steps of 0.3 years (https://github.com/harrymatthews50/3DGrowthCurves/tree/master/FacialAssessmentToolbox/GrowthCurves).
Each file is 13 to 24 MB.
The mesh is a template of 7,160 points that covers the face only.
The files are MATLAB class objects, so reading them needs MATLAB or a conversion done in MATLAB.
The file names contain a body-weight value, for example `F_Age=10.10BW=1.00.mat`, which suggests a body weight factor; its meaning is not verified.

The repository is under GPL-3.0 (https://github.com/harrymatthews50/3DGrowthCurves/blob/master/LICENSE).
The model files have no separate licence.
Whether GPL-3.0 covers the model files is not verified.
GPL-3.0 does not fit an MIT repository, so shipping derived shapes needs the authors' written permission.
The research agent found the contacts harry.matthews@kuleuven.be and peter.claes@kuleuven.be (not verified on a primary page).
For (A), these models are useful locally to generate average faces of children and elderly people, to test whether the editor can reach those shapes.

### eos and face3d

eos is a C++ library that fits a 3DMM to 2D landmarks, under Apache-2.0 (https://github.com/patrikhuber/eos/blob/master/LICENSE).
Its code is a clean reference, but the models it ships are non-commercial, as described in the Surrey section.
face3d is a Python teaching toolkit for 3D faces (https://github.com/YadiraF/face3d).
It has no licence file, so all rights are reserved.

## Face data on the list

### S3DFM

S3DFM is the Speech-driven 3D Facial Motion database of the University of Edinburgh (https://groups.inf.ed.ac.uk/trimbot2020/DYNAMICFACES/index.html).
It has 77 participants, 27 women and 50 men, aged 16 to 73.
The page lists nationality codes from North American to South and Southeast Asian, without a count per group.
Each sequence is a depth map of 600 by 600 points over 500 frames, with an infrared image and audio.
The capture is a front-facing face only, with no colour texture and no registered mesh.
The files download directly with no form, and a sample zip of 2.4 GB returned HTTP 200.
The licence is CC BY-NC-ND, and the page says the data "can only be used for academic research".
The ND part means no derivatives may be shared, so nothing derived from it can ship.
It could serve (A) only for the front of the face, and only if a hobby benchmark counts as academic research.

### UoY 3D face dataset

The current page is https://www-users.york.ac.uk/~np7/research/UoYfaces/.
The dataset has about 5,000 3D images of about 350 people, with 15 captures each.
The subjects were University of York Computer Science staff and students around 2003.
Age, sex and ethnicity figures are not given.
The data is OBJ face meshes with BMP textures, in several poses and expressions, including one with the eyes closed.
The agreement says "The full-time academic staff member, named below, agrees to the following restrictions" (https://www-users.york.ac.uk/~np7/research/UoYfaces/UoYfaces-agreement.pdf).
The licence is "non-commercial research and education purposes only".
A private individual cannot obtain it.

### D3DFACS raw scans

The link on the list now redirects to https://darrencosker.github.io/.
That page says D3DFACS has "over 500 FACS-coded dynamic 3D sequences from 10 individuals".
The dataset site https://vision.cs.bath.ac.uk/facedata does not respond today.
An archived page says "To obtain the data, please send an email to dpc@cs.bath.ac.uk. After signing an agreement form you will then be sent instructions for download" (http://web.archive.org/web/20221009193841/http://www.cs.bath.ac.uk/~dpc/D3DFACS/).
The agreement text is not verified.
The registered meshes are already covered by the earlier survey.

### FaceCap and MonFaceCap

FaceCap is one stereo sequence of 200 frames with textured 3D face reconstructions (https://vcai.mpi-inf.mpg.de/projects/FaceCap/).
The page says only "senior project managers or senior researchers" may request it, and "Any commercial use is hereby excluded".
MonFaceCap is 4 video sequences with tracked face meshes (https://vcai.mpi-inf.mpg.de/projects/MonFaceCap/).
Its page says "Only institutional email addresses are accepted".
Neither is open to a private individual, and both have too few people.

### Parametric Face Image Generator and Mooney Face Generator

Both render 2D images from the Basel Face Model.
The Parametric Face Image Generator is under Apache-2.0 and outputs images, depth maps and landmarks (https://github.com/unibas-gravis/parametric-face-image-generator).
The Mooney Face Generator makes two-tone 2D images and is under Apache-2.0 (https://github.com/updown2/MooneyFaceGenerator).
Neither gives meshes.

## Body models that include a head

### SMPL and SMPL-X

Both are Max Planck body models (https://smpl.is.tue.mpg.de/, https://smpl-x.is.tue.mpg.de/).
The model licences allow only "non-commercial scientific research, non-commercial education, or non-commercial artistic projects" and forbid distribution (https://smpl.is.tue.mpg.de/modellicense.html, https://smpl-x.is.tue.mpg.de/modellicense.html).
A separate "Body" licence is CC BY 4.0, but it "excludes the shape blendshapes or the tools to create 3D bodies using the shape blendshapes" (https://smpl.is.tue.mpg.de/bodylicense.html).
That means one fixed mesh is open, but the shape space is not.
The face of SMPL-X is FLAME, which is covered above.

### STAR

STAR is a body model trained with 10,000 more scans than SMPL (https://star.is.tue.mpg.de/).
Its licence is non-commercial (https://star.is.tue.mpg.de/license.html).
Its head is the same low-detail head as SMPL.

### GHUM and GHUML

GHUM is a Google body model with a 16-dimensional body shape space "trained based on 4,329 male and female body shape scans" and a 20-dimensional facial expression space (https://github.com/google-research/google-research/tree/master/ghum).
No detailed face identity space was found (not verified).
The model files are not in the repository.
They are requested through a Google form that asks for a "University or Academic Institution affiliated with" and an academic email address.
The licence of the model files is not published, so it is not verified.
A private individual cannot request it.

### MPII Human Shape and CAESAR

MPII Human Shape offers body shape spaces and fitted meshes learned from CAESAR, and its two 640 MB downloads work (https://humanshape.mpi-inf.mpg.de/).
The page says both "Licensed under the Simplified BSD License" and "freely available for non-commercial and educational purposes only".
The linked licence file now redirects to the page itself, so the real terms are not verified.
The meshes are low-resolution bodies, and redistributing shapes derived from CAESAR scans is doubtful.

CAESAR is the source of the FLAME identity space.
The SAE store page (https://store.sae.org/caesar/) is behind an anti-robot page and could not be read.
The non-profit WEAR (https://bodysizeshape.com/) lists "The North American and European CAESAR 3D Body Scan Collection, representing 4,459 individuals".
The price and the terms of the scans are not verified.

## Image-fitting software on the list

All these projects fit a face model to photographs.
None of them is a source of shippable shapes, and only two of them handle 3D scans.

The two FLAME tools are the useful ones.
flame-fitting has a script `fit_scan.py` that fits FLAME to a raw 3D scan with a scan-to-mesh distance and 51 landmarks (https://github.com/Rubikplayer/flame-fitting).
It has no licence file, so all rights are reserved.
TF_FLAME fits FLAME to 3D landmarks and to meshes already in FLAME topology (https://github.com/TimoBolkart/TF_FLAME).
Its README says arbitrary scans would need a scan-to-mesh loss.
It has no licence file either.

Several repositories ship data derived from the Basel Face Model 2009 even though that model is non-commercial.
3DDFA ships BFM shape and expression bases (https://github.com/cleardusk/3DDFA).
3DDFA_V2 ships a modified BFM in `configs/bfm_noneck_v3.pkl`, and its own `bfm/readme.md` says it is "only for academic use" (https://github.com/cleardusk/3DDFA_V2).
3dmm_cnn ships `lib/BaselFace.dat` under a USC academic licence (https://github.com/anhttran/3dmm_cnn).
The project must not take shapes from any of these.

The remaining entries are listed with their code licence and model in one line each.

- 3DMM_edges uses BFM and is under Apache-2.0 (https://github.com/waps101/3DMM_edges).
- 3dmm_basic uses BFM and has no licence file (https://github.com/anhttran/3dmm_basic).
- scalismo-faces uses BFM and is under Apache-2.0 (https://github.com/unibas-gravis/scalismo-faces).
- The CVPR 2017 regression network uses BFM and states no licence (https://talhassner.github.io/home/publication/2017_CVPR).
- Expression-Net uses BFM and has no licence file (https://github.com/fengju514/Expression-Net).
- Face-Pose-Net uses BFM, links BFM files on Dropbox and has no licence file (https://github.com/fengju514/Face-Pose-Net).
- extreme_3d_faces uses BFM and is under a USC academic licence (https://github.com/anhttran/extreme_3d_faces).
- RingNet uses FLAME and is under MIT for the code, although its README still says non-commercial (https://github.com/soubhiksanyal/RingNet).
- PRNet uses BFM-derived data and is under MIT for the code (https://github.com/YadiraF/PRNet).
- pix2vertex uses its own depth model and has no licence file (https://github.com/matansel/pix2vertex).
- 3DMMasSTN uses BFM and is under Apache-2.0 (https://github.com/anilbas/3DMMasSTN).
- VRN regresses a voxel volume and is under MIT (https://github.com/AaronJackson/vrn).
- Deep3DFaceReconstruction uses BFM and is under MIT for the code (https://github.com/microsoft/Deep3DFaceReconstruction).
- Facial_Details_Synthesis uses BFM 2017 and is under MIT for the code (https://github.com/apchenstu/Facial_Details_Synthesis).
- photometric_optimization uses FLAME, and its README says non-commercial research (https://github.com/HavenFeng/photometric_optimization).
- WM3DR uses BFM and has no licence file (https://github.com/kalyo-zjl/WM3DR).
- Rotate-and-Render uses BFM, links a copy of BFM files and is under CC BY 4.0 for its own work (https://github.com/Hangz-nju-cuhk/Rotate-and-Render).
- DECA uses FLAME and is under a Max Planck non-commercial licence (https://github.com/YadiraF/DECA).
- Nonlinear_Face_3DMM uses BFM topology and is under Apache-2.0 (https://github.com/tranluan/Nonlinear_Face_3DMM).
- PIRender uses BFM and is under CC BY-NC 4.0 (https://github.com/RenYurui/PIRender).
- SynergyNet uses BFM data from 3DDFA and is under MIT (https://github.com/choyingw/SynergyNet).
- VariTex uses BFM 2017, and its README says CC BY-NC-SA 4.0 (https://github.com/mcbuehler/VariTex).
- PBIDR adds detail to textured meshes, and its models are non-commercial (https://github.com/deepinsight/insightface/tree/master/reconstruction/PBIDR).
- Occlusion-Robust-MoFA uses BFM 2017 and is under MIT (https://github.com/unibas-gravis/Occlusion-Robust-MoFA).

## Entries skipped as irrelevant

Each entry below was checked, and one reason is given for skipping it.

- The Google group (https://groups.google.com/g/3d-morphable-models) is a mailing list and holds no data.
- SMAL and CAFM are animal models.
- CAPE is a model of clothed bodies.
- Three-D Safari (smalst) fits an animal model to images.
- Dyna, 3DPW, Human3.6M, BUFF, Unite the People and CMU Panoptic are body or pose data without usable head scans.
- HUMBI has 772 people, but its faces are fits of the Surrey model with 10 shape values, not scans, and its data licence is not published (https://github.com/zhixuany/HUMBI).
- MuCo-3DHP, MuPoTS-3D, MonoPerfCap, IntrinsicMoCap, MPI-INF-3DHP, MARCOnI, EgoCap, BinoCap, the Inertial Depth Tracker data, Performance Capture from Sparse Multi-view Video and the Kinects mocap data are body pose or performance capture. Several of their old links now return 404.
- People Snapshot and the Inria dressed human data are body shape data.
- YouTube 3D Hands is hand data.
- Neural Body Fitter, SMPLify-X and ExPose fit body models to images.
- The Basel Illumination Prior 2017 holds lighting parameters, not shapes. It is under CC BY 4.0 at https://shapemodelling.cs.unibas.ch/gravis-site-archive/PMM/data/bip/.
- The Basel tutorials are courses and offer no face model data (https://shapemodelling.cs.unibas.ch/ssm-course).
- The Basel face morpher (https://face-morpher.scalismo.org/) and the Nottingham VRN demo (https://cvl-demos.cs.nott.ac.uk/vrn/) are dead web services. The pix2vertex Binder page still loads.
- The NoW benchmark, Headspace, LYHM, FaceScape, CoMA, VOCASET, BU-3DFE, the Florence dataset, H3D-Net, Multiface, the Basel Face Model 2009 and 2017, and FLAME are covered by the earlier survey. Nothing new was found about them on the list, apart from the FLAME re-check above.

## Ranked recommendation

### Shapes that can ship in this repository

1. ICT FaceKit Light stays first. It is under MIT, covers the full head with eyes and mouth parts, and was learned from about 178 people aged 18 to 67.
2. FLAME 2023 Open is second. It is under CC BY 4.0 with two use restrictions, and it was learned from 3,800 heads. The repository must credit the Max Planck Institute, link the licence, state the changes, and repeat the two restrictions. The FLAME texture must not be used.
3. FaceVerse is a new third candidate. Its repositories are under BSD-2-Clause and MIT, and it adds East Asian faces. The project should first ask the authors in writing whether the model files fall under those licences despite the dataset's terms.
4. 3D Growth Curves is the only source of children's and elderly faces that a private individual can download. It is under GPL-3.0, so the project should ask the KU Leuven authors for permission to ship derived shapes under MIT or CC BY. Until then it should be used only locally.

No part-based or local face model on the list can ship.
The project can build its own local models with the published methods (wavelet regions as in Bolkart's work, or local support factors as in Bernard's work) on top of ICT FaceKit or FLAME 2023 Open shapes.

### Benchmark data a private individual can get

1. The list adds no strong new benchmark. The recommendations of the earlier survey stand: Ava-256, FaMoS, the NoW validation scans and Multiface.
2. S3DFM adds 77 real people aged 16 to 73 with a direct download. Its depth maps show only the front of the face and its licence is academic and no-derivatives, so it is a weak addition.
3. 3D Growth Curves can generate average faces for any age and sex as local test targets. These targets cover children and elderly people, which no obtainable scan set does.
4. Bolkart's BU-3DFE models and the small Surrey model can generate local test faces under a non-commercial licence. They add little over FLAME 2023 Open.

### Registration tools to use

1. MeshMonk is the first choice. It is under Apache-2.0, maintained in 2026, and installable from PyPI.
2. menpo3d NICP is a good BSD-3-Clause reference implementation of NICP.
3. scalismo is an Apache-2.0 reference for Gaussian process registration, but it is written in Scala.
4. flame-fitting shows how to fit FLAME to raw scans, but it has no licence, so it should only be read, not copied.

## Pages that could not be read

- The FLAME 2023 Open archive needs a login, so its exact files are not verified.
- The FaceVerse version 4 OneDrive link refused an automated request.
- The licence pop-up of the MATLAB File Exchange auction solver could not be read.
- The SAE CAESAR store page is behind an anti-robot page.
- The MPII Human Shape licence file now redirects to the project page.
- The D3DFACS dataset site at vision.cs.bath.ac.uk does not respond.
- The LSFM application page at xip.uclb.com returns 404.
