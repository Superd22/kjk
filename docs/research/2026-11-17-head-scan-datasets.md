# Head scan datasets for the face editor benchmark

This document was researched on 2026-11-17.
It answers two questions.
The first question is which sets of 3D human head scans a private individual can obtain to measure how closely the face editor reproduces a real head.
The second question is whether shapes learned from a dataset could ship inside this MIT-licensed repository.

A learned shape is any shape computed from a dataset, such as an average head, a set of PCA shapes, or slider values fitted against the scans.
PCA means principal component analysis, which is a method that finds the main directions in which a set of shapes varies.
A registered mesh is a scan that was refitted so that every person has the same vertices in the same order.
A raw scan is the unprocessed output of the scanner, with a different vertex count for each person.

Every claim carries the URL of the page it was read on.
The words "not verified" mark every claim that could not be confirmed on a primary source.
Nothing in this document is legal advice.

## Summary table

| Dataset | People | How to get it | Can a private individual get it | Licence in a few words |
|---|---|---|---|---|
| Headspace and LYHM (York) | 1519 | Signed PDF agreement sent by email | No | Non-commercial research, university staff only |
| FaceScape (Nanjing) | 847 released, 938 captured | Signed PDF agreement sent by email | No | Non-commercial research, no distribution |
| FLAME 2023 Open (MPI) | A model trained on 3800 heads, no individual people | Free account on the FLAME site | Yes | CC BY 4.0 with two extra restrictions |
| FLAME 2017 to 2023, other versions | Same model | Free account | Yes | Non-commercial scientific research, no redistribution |
| CoMA (MPI) | 12 | Free account | Yes | Non-commercial, no redistribution |
| D3DFACS registrations (MPI) | 10 | Free FLAME account | Yes | Non-commercial scientific research, no redistribution |
| FaMoS (MPI) | 95 | Free account | Yes | Non-commercial, no redistribution |
| NoW validation scans (MPI) | 20 with scans, 100 in total | Free account | Yes (sign-up form not verified) | Non-commercial scientific research, no redistribution |
| VOCASET (MPI) | 12 | Free account | Yes | Non-commercial, no redistribution |
| NPHM (TU Munich) | 255 in the paper, 124 on the project page | Google form | Not verified | Research only |
| Multiface (Meta) | 13 | Open download script | Yes | CC BY-NC 4.0 |
| Ava-256 (Meta) | 256 | Open download script | Yes | CC BY-NC 4.0 |
| ICT FaceKit (USC) | A model with 100 PCA shapes, no individual people | Open GitHub repository | Yes | MIT |
| BU-3DFE and BU-4DFE | 100 and 101 | Agreement signed by an institution | No | Non-profit research |
| BP4D and BP4D+ | 41 and 140 | Agreement signed by an institution | No | Non-profit research |
| FaceWarehouse | 150 | Email stating title and affiliation | Unlikely | Academic use, derived shapes may not go on the Internet |
| Basel Face Model 2017 and 2019 | A model built from 200 heads | Web form asking for an institution | Probably (not verified) | Non-commercial, no derivatives |
| Basel Face Model 2009 example scans | 10 | Open download | Yes | Academic education and research |
| H3DS 2.0 (Crisalix) | 59 | Google form, reviewed | Possibly | Non-commercial research |
| 3DFAW-Video | 66 (not verified) | Signed agreement | No | Non-profit research |
| Stirling/ESRC | 99 | Signed agreement sent by email | Possibly | Research only |
| Florence 2D/3D | 53 (not verified) | Google form | Not verified | Academic, non-commercial (not verified for the scans) |
| Bosphorus | 105 (not verified) | Site is offline | Not verified | Not verified |
| FRGC v2 | 466 | Agreement signed by the institution's legal signatory | No | Internal research, no redistribution |
| HSRD-100 | 10 | Open download on Hugging Face | Yes | CC BY 4.0 |
| MakeHuman targets | Artist-made, no real people | Open download | Yes | CC0 |

## Short answer

No truly open dataset with 50 or more real heads was found.
"Truly open" here means CC0, CC BY, public domain, MIT, Apache or a similar permissive licence.

The dataset that fits the benchmark best is Headspace, which has 1519 people wearing caps.
A private individual cannot obtain Headspace, because the agreement must be signed by a university employee.

A private individual can download four useful sets today.
Ava-256 has 256 people under CC BY-NC 4.0.
FaMoS has 95 people under a Max Planck non-commercial licence.
The NoW benchmark gives 20 raw neutral scans under a Max Planck non-commercial licence.
Multiface has 13 people under CC BY-NC 4.0.

Only three sources clearly allow shipping derived shapes in this repository.
They are ICT FaceKit (MIT), FLAME 2023 Open (CC BY 4.0) and the MakeHuman targets (CC0).
None of the three contains scans of individual real people.

## Headspace and the Liverpool-York Head Model

### People

Headspace has 1519 subjects.
The page says "1519 subjects wearing tight fitting latex caps to reduce the effect of hairstyles" (https://www-users.york.ac.uk/~np7/research/Headspace/).
The Liverpool-York Head Model (LYHM) is a statistical head model built from 1212 of these subjects (https://www-users.york.ac.uk/~np7/research/Headspace/).
The LYHM page says the model has a roughly even split of male and female subjects (https://www-users.york.ac.uk/~np7/research/LYHM/).
The pages give no age range, no ethnic breakdown and no body weight figures.
The per-subject metadata records "gender, declared ethnic group, age".
A figure of 606 men and 606 women in the journal paper is not verified.

### Data

The data is raw scans from a 3dMD scanner.
The page says "The data is in OBJ format with BMP textures."
Texture is "available for over 80% of subjects".
Registered meshes also exist.
The download folder contains "MICA-FLAME registrations (OBJ files) & FLAME model parameters (NPZ files)" (https://www-users.york.ac.uk/~np7/research/LYHM/LYHMagreement08.pdf).
Subjects wear latex caps, so scalp hair is hidden.
The metadata records beard and moustache descriptors, so some subjects have facial hair.
Whether the eyes are open is not verified.

### How to obtain it

The applicant signs a PDF agreement and emails it to the University of York.
The data is then shared through a Google Drive folder.
The Headspace page says "It can only be requested by academics that are verifiable university employees."
The agreement says "this form must be completed by an academic that is a university employee engaged in non-commercial research - if a student completes this form, it will not be accepted."
A private individual without a university cannot get Headspace.

### Licence

The agreement says "The LYHM models and Headspace dataset are available for non-commercial research and education purposes only."
It also says the data "will not be copied, sold, traded or exploited for any commercial purposes."
The agreement has no clause about derived models, statistics or benchmarking.
Both quotes come from https://www-users.york.ac.uk/~np7/research/LYHM/LYHMagreement08.pdf.

### Size

The OBJ dataset is 38 GB.
The raw 3dMD images are 30 GB.
The FLAME registrations are 137 MB.
The LYHM models are 205 MB.
These sizes come from the agreement PDF.

## FaceScape

The old site https://facescape.nju.edu.cn/ no longer resolves.
The data moved to https://nju-3dv.github.io/projects/FaceScape/.

### People

The authors captured 938 people.
The paper says "We invited 938 people between the ages of 16 and 70 as subjects" and describes them as "mostly from Asia" (https://ar5iv.labs.arxiv.org/html/2003.13989).
The public release has 847 people with 20 expressions each (https://nju-3dv.github.io/projects/FaceScape/).
The male and female counts are not given in text.
Body weight is not stated.

### Data

The release contains registered meshes, which the authors call topologically uniform meshes.
Each person has 20 meshes, 20 displacement maps and a texture.
The page says "The texture maps and displacement maps reach 4K resolution" (https://github.com/zhuhao-nju/facescape/blob/master/doc/doc_tu_model.md).
The textures of subjects 360 to 847 have a mosaic over the eyes for privacy.
One of the 20 expressions is neutral and one has the eyes closed.
The NPHM paper says FaceScape participants wore a bathing cap (https://ar5iv.labs.arxiv.org/html/2212.02761).
Facial hair is not verified.

### How to obtain it

The applicant signs a PDF licence agreement and emails it.
The licence says "The permission for application is only open to researchers or faculty of universities or research institutes. It is prohibited for students or business entities to apply."
A private individual cannot get the full dataset.
The FaceScape bilinear model version 1.6 can be downloaded without a licence key (https://github.com/zhuhao-nju/facescape/blob/master/doc/external_link_fsbm.md).
A bilinear model is a statistical model with one set of shapes for identity and one for expression.
That page says users must still obey the agreement, "including no-commercial-use and no-distribution."

### Licence

The licence says "Licenses free of charge are limited to non-commercial research use only."
It says "You are not allowed to sublicense or distribute this dataset in whole or in part to any third party."
Uses that the agreement does not name need "prior written permission from CITE LAB".
The agreement has no clause on derived models.
The quotes come from https://nju-3dv.github.io/projects/FaceScape/static/license/LicenseAgreement_FaceScape.pdf.

### Size

The registered meshes are 120 GB.
The multi-view photographs are 689 GB.
The parametric model is 15.4 GB on Google Drive.
These sizes come from https://github.com/zhuhao-nju/facescape/blob/master/doc/facescape_googledrive.md.

## FLAME and the datasets registered to it

FLAME is a statistical head model from the Max Planck Institute for Intelligent Systems (MPI).
All the MPI sites below use the same kind of sign-up form.
The FLAME form asks for an email address, a password and three consent boxes (https://flame.is.tue.mpg.de/register.php).
The form has no field for an institution.
A private individual can register.

### The FLAME model

The site says "FLAME uses a linear shape space trained from 3800 scans of human heads" (https://flame.is.tue.mpg.de/).
The paper says the 3800 heads come from the CAESAR body scan database, with 2100 women and 1700 men (https://download.is.tue.mpg.de/flame/flame_paper.pdf).
CAESAR is a survey of full-body scans of American and European civilians.
The paper says the scans span "a wide range of ages, ethnicities, and both genders".
The FLAME site does not offer the CAESAR head scans.
The claim that CAESAR is sold separately is not verified.
FLAME is a model and contains no individual people.
The model has no hair and covers the full head and neck.

FLAME 2023 Open has its own licence.
The site news says "Released FLAME 2023 Open Model with the Open Source License (CC-BY-4.0)" (https://flame.is.tue.mpg.de/).
The licence page says "Max-Planck grants you the right to share and adapt the model for any purpose, including commercial use" (https://flame.is.tue.mpg.de/modellicense.html).
The user must "give appropriate credit, provide a link to the license, and indicate if changes were made".
Two extra restrictions apply.
The licence says "The model may not be used for pornographic purposes".
The licence also forbids using the model "to create fake, libelous, misleading, or defamatory content of any kind".
The exact files in the FLAME 2023 Open download and their size are not verified, because the download page needs a login.

All other FLAME versions are under an academic licence.
That licence allows use "for the sole purpose of performing non-commercial scientific research".
It says "The Data & Software may not be reproduced, modified and/or made available in any form to any third party".
The FLAME texture model is separate and is under CC BY-NC-SA 4.0 (https://flame.is.tue.mpg.de/texturelicense.html).

### CoMA

CoMA has 12 subjects.
The page says "20,466 meshes of extreme expressions captured over 12 different subjects" (https://coma.is.tue.mpg.de/).
The meshes are registered in the FLAME topology.
The age, sex and ethnicity of the 12 subjects are not verified.
Whether raw scans are included is not verified.
The download size is not verified.
A private individual can register at https://coma.is.tue.mpg.de/register.php.
The licence allows "non-commercial scientific research", non-commercial education and non-commercial artistic projects (https://coma.is.tue.mpg.de/license.html).
The licence says "The Software may not be reproduced, modified and/or made available in any form to any third party".

### D3DFACS registrations

The D3DFACS registrations cover 10 subjects and 519 expression sequences (https://github.com/TimoBolkart/FLAME-Universe).
They are meshes in the FLAME topology.
They are downloaded with the same FLAME account.
The licence allows use "for the sole purpose of performing non-commercial scientific research" (https://flame.is.tue.mpg.de/datalicense.html).
It says "The data may not be reproduced, modified and/or made available in any form to any third party".
The licence of the original D3DFACS dataset is not verified, because its site did not load.

### LYHM registrations in the FLAME topology

The FLAME site mentions "FLAME registrations for 1216 subjects" of the Headspace dataset (https://flame.is.tue.mpg.de/).
The MICA repository lists 1211 subjects (https://github.com/Zielon/MICA/tree/master/datasets).
These registrations are not on the MPI site.
They are released by York after the signed Headspace agreement.
A private individual cannot get them.

### NoW benchmark

The NoW benchmark has 100 subjects, with 55 women and 45 men (https://now.is.tue.mpg.de/dataset.html).
The subjects were chosen to vary in age, body mass index and sex.
Each subject has one neutral raw head scan of about 120,000 vertices.
The scans are released only for the 20 validation subjects.
The page says "we do not provide the ground truth scans for the test data" (https://now.is.tue.mpg.de/).
The licence allows use "for the sole purpose of performing non-commercial scientific research" (https://now.is.tue.mpg.de/license.html).
The sign-up form of this site was not checked, so open registration is not verified.
Whether the subjects wear caps is not verified.

### FaMoS

FaMoS has 95 subjects with 28 motion sequences each (https://tempeh.is.tue.mpg.de/).
The data is about 600,000 registered meshes in the FLAME topology.
The licence allows non-commercial scientific research, education and artistic projects (https://tempeh.is.tue.mpg.de/license.html).
The licence forbids redistribution.
The demographics of the subjects, the download size and the presence of raw scans are not verified.

### VOCASET

VOCASET has 12 speakers (https://voca.is.tue.mpg.de/).
The licence allows "non-commercial scientific research, non-commercial education, or non-commercial artistic projects" (https://voca.is.tue.mpg.de/license.html).
The licence forbids making the data available to third parties.

## NPHM

NPHM stands for Neural Parametric Head Models, from the Technical University of Munich.

### People

The current paper says "Our dataset comprises 255 subjects, 29% female, and contains over 5200 3D scans" (https://ar5iv.labs.arxiv.org/html/2212.02761).
The paper's table lists 188 men and 67 women.
The project page still gives the older figure of 124 identities (https://simongiebenhain.github.io/NPHM/).
Age, ethnicity and body weight are not stated.

### Data

The raw scans come from two Artec Eva scanners and average 3.5 million faces.
Each person has 20 to 23 expressions.
The paper says "The neutral expression has an open mouth to avoid topological issues."
Registered meshes in a fixed template topology are also provided (https://github.com/SimonGiebenhain/NPHM).
Subjects wear no cap.
The scans capture natural hair, but the paper says "we do not capture loose hair".
Whether the download includes colour is not verified.
Open eyes and facial hair are not verified.

### How to obtain it

Access goes through a Google form (https://forms.gle/66xWfAxzCvsoqcNZ8).
The form needs a Google sign-in, so its terms could not be read.
Whether the form requires an academic affiliation is not verified.
Three GitHub issues from 2025 and 2026 report that requests received no answer (https://github.com/SimonGiebenhain/NPHM/issues/27, https://github.com/SimonGiebenhain/NPHM/issues/28, https://github.com/SimonGiebenhain/NPHM/issues/29).

### Licence

The README says "Due to its sensitivity our data is restricted for research purposes only!" (https://github.com/SimonGiebenhain/NPHM/blob/main/dataset/README.md).
The code is under MIT.
The LICENSE file says the MIT licence "DOES NOT concern the NPHM dataset" (https://github.com/SimonGiebenhain/NPHM/blob/main/LICENSE).
The clauses on commercial use and derived models are not verified.
The download size is not stated.

## Multiface

Multiface contains "the faces of 13 identities" (https://github.com/facebookresearch/multiface).
The data includes tracked meshes, textures at 1024 by 1024 pixels, and photographs from 40 to 160 cameras.
The full dataset is 65 TB.
A small version of 16.2 GB is offered.
The download is open through a script in the repository, with no registration.
The licence is Creative Commons Attribution-NonCommercial 4.0 (https://raw.githubusercontent.com/facebookresearch/multiface/main/LICENSE).
The licence allows the user to "produce, reproduce, and Share Adapted Material for NonCommercial purposes only."
Benchmarking in this project is allowed.
Shapes adapted from Multiface cannot be relicensed under MIT, because the non-commercial condition follows them.
The age, sex and ethnicity of the 13 people are not verified.

## Ava-256

Ava-256 is a Meta dataset of 256 people (https://github.com/facebookresearch/ava-256).
The repository says "ava-256 is CC BY-NC 4.0 licensed".
The download is open through a script, with no form.
The default download is 4 TB.
The dataset includes a registered mesh for each subject.
Whether the meshes can be downloaded without the images is not verified.
The demographics, the presence of caps and the presence of hair are not verified.
Ava-256 is the largest set of real heads that a private individual can download without a form.

## USC ICT FaceKit

The licence is MIT (https://raw.githubusercontent.com/USC-ICT/ICT-FaceKit/master/LICENSE).
The repository ships a model called ICT Face Model Light (https://github.com/USC-ICT/ICT-FaceKit).
The README says the model "includes a set of 100 PCA modes of linear morph targets."
The model also has 53 expression shapes named after Apple ARKit.
The whole model has 26,719 vertices.
The face part has 9,409 vertices.
The model ships no texture.
The 100 identity files are PCA shapes and not individual people.
The repository contains no scan of any individual person.
The related paper says the work is "Based on a combined data set of 4000 high resolution facial scans" (https://arxiv.org/abs/2004.03458).
The link between that figure and the 100 shapes of the Light model is not verified.
A fuller model with more than 200 shapes is not in the repository.
The README says "The full model will be released under a different USC specific license."

## BU-3DFE, BU-4DFE, BP4D and BP4D+

All four come from Binghamton University (https://www.cs.binghamton.edu/~lijun/Research/3DFE/3DFE_Analysis.html).

BU-3DFE has 100 subjects.
The page says "100 subjects (56% female, 44% male), ranging age from 18 years to 70 years old, with a variety of ethnic/racial ancestries".
Each subject has 25 raw scans with texture, and one of the 25 is neutral.
The scans have 20,000 to 35,000 polygons (https://www.cs.binghamton.edu/~lijun/Research/3DFE/Yin_FGR06_a.pdf).
The raw scans show hair.
Caps, open eyes and facial hair are not verified.

BU-4DFE has 101 subjects, with 58 women and 43 men.
It contains 3D video of expressions.

BP4D has 41 participants aged 18 to 29.
BP4D+ has 140 subjects aged 18 to 66.
Both contain 3D video of spontaneous expressions and no separate neutral scans.
BP4D is about 2.6 TB.

Access needs a written agreement.
The page says the agreement "must first be signed by the recipient and the research administration office director of your institution".
The page says "Students are not eligible to be a recipient."
A private individual cannot meet this condition.
The page says the data goes "to parties who are pursuing research for non-profit use."
The text of the agreement is not published, so its clauses on derived models are not verified.

## FaceWarehouse

FaceWarehouse comes from Zhejiang University (http://kunzhou.net/zjugaps/facewarehouse/).
The page says "we captured 150 individuals aged 7-80 from various ethnic backgrounds."
The capture device is a Kinect depth camera, which has a low resolution.
Each person has a neutral expression and 19 other expressions.
The data includes the raw depth captures and fitted meshes in a shared topology.
The applicant sends an email with "your name, title, affiliation" and the intended use.
The page says "we only send the data to approved researchers."
A private individual is not excluded in words, but approval is unlikely and is not verified.
The licence forbids commercial use.
The licence says "the data may not be included in the electronic version of a publication, nor placed on the Internet."
The licence says "These restrictions apply to any representations (other than images or video) derived from the data, including but not limited to simplifications, remeshing, and the fitting of smooth surfaces."
Shapes derived from FaceWarehouse therefore cannot ship in this repository.
The sex balance and the download size are not stated.

## Basel Face Model

The Basel Face Model (BFM) is a statistical face model from the University of Basel.

### People

The 2009 model was built from 200 people.
The paper says "face scans of 100 female and 100 male persons, most of them Europeans. The age of the persons is between 8 and 62 years with an average of 24.97 years and the weight is between 40 and 123 kilogram with an average of 66.48 kilogram" (https://shapemodelling.cs.unibas.ch/gravis-site-archive/publications/2009/BFModel09.pdf).
The 2017 and 2019 models are each "derived from 200 Neutral Expression Heads and 160 Expression Deformations" (https://faces.dmi.unibas.ch/bfm/BFM2019_NonCommercial_License_Agreement.pdf).
The 2017 paper says the authors included "more people over 40 years" (https://arxiv.org/abs/1709.08398).

### Data

The downloads are statistical models and not scans.
The 200 training scans are not distributed.
The 2009 model has 53,490 vertices and one colour value per vertex (https://faces.dmi.unibas.ch/bfm/index.php?nav=1-1-0&id=details).
The scanner captured the face "from ear to ear", so the model has no back of the head and no hair.
The 2017 paper says the scans show "No make-up, beards or hair in the facial area."

One exception exists.
The 2009 release offers ten registered example scans of people who are not in the training set (https://faces.dmi.unibas.ch/bfm/index.php?nav=1-1-1&id=scans).
The page says "Ten registered example scans are provided for the usage in academic education and research."
These ten scans are 94 MB and need no login (https://faces.dmi.unibas.ch/bfm/index.php?nav=1-2&id=downloads).

### How to obtain it

Each model has a web form that asks for "your real full name, institution, and valid e-mail address" (https://faces.dmi.unibas.ch/bfm/bfm2019.html).
The form does not state that a university is required.
Whether a request without an institution is accepted is not verified.

### Licence

The 2017 and 2019 licences use the same words.
They grant use "for internal, non-commercial research, evaluation or testing purposes."
Clause 3.3 says "LICENSEE shall not reverse engineer, disassemble or modify the DATA, or create derivatives based on the DATA".
The same clause forbids distributing the data to any third party.
Shapes derived from the Basel Face Model therefore cannot ship in this repository.
Private evaluation fits the permitted purpose.
The full text of the 2009 licence was not found, so its clause on derivatives is not verified.

## H3DS

H3DS comes from the company Crisalix (https://github.com/CrisalixSA/h3ds).
Version 1.0 has 23 subjects and version 2.0 has 59 subjects.
The scans are raw textured scans of the full head, made with an Artec Eva scanner.
The paper describes "high-resolution textured 3D meshes capturing the full head, including hair and upper body clothing" (https://ar5iv.labs.arxiv.org/html/2310.08784).
Subjects wear no cap, so hair covers the head.
Age, ethnicity and body weight are not stated.
Access goes through a Google form, and Crisalix reviews each application.
The form could not be opened without a Google sign-in.
The licence defines the licensee as "your academic institution, non-profit organization, company or yourself" (https://drive.google.com/file/d/1_Uy5jklFEQMGvw0W-wTJvdwRqOtSAf7l/view).
A private individual is therefore inside the wording of the licence.
The signature block asks for a Principal Investigator and an affiliation.
Whether Crisalix accepts an applicant without them is not verified.
The licence says the licensee must use the data "solely for non-commercial research purposes".
The licence says "The Licensee may publish research results derived from the use of the Licensed Intellectual Property, provided that appropriate attribution is given".
The licence forbids using the files "to improve a commercial product or a service".
The licence does not mention derived shapes.
The download size is not stated.

## 3DFAW and 3DFAW-Video

The 2016 3DFAW data is photographs annotated with 66 landmarks (https://ar5iv.labs.arxiv.org/html/1609.09545).
It is not a set of head scans.
No dataset page or licence was found for it.

3DFAW-Video pairs videos with "high-resolution 3D ground truth meshes" from a Di4D scanner (https://3dfaw.github.io).
A count of 66 subjects is not verified.
The agreement says "The recipient of the Database must be a full-time faculty, researcher or employee of an organization (not a student)" (https://3dfaw.github.io/3DFAW-Video_EULA-ICCV2019.pdf).
A private individual does not meet this wording.
The agreement says "The Database will be used for non-for-profit research only."

## Stirling/ESRC 3D Face Database

### People

The site says "Currently we have 45 male and 54 female sets available" (http://pics.stir.ac.uk/ESRC/).
That is 99 people.
Age, ethnicity and body weight are not stated.

### Data

Both raw scans and registered meshes exist (http://pics.stir.ac.uk/ESRC/3d_images.htm).
The page says "The raw, unconformed files are presented at half the available resolution, about 85000 vertices. The conformed files are reduced a standard face mesh with 3746 vertices."
The files are OBJ meshes with a texture image.
The site says "Subjects were photographed wearing a white cap, to contain their hair".
The eyes are open, and the scanner distorts them.
Some men have beards.
The scanner has two stereo camera pairs, so the scans probably cover only the front of the head.
That coverage is an inference and is not verified.

### How to obtain it

The site says "please download the license agreement, sign and scan it and return to 3dfacedb@gmail.com."
The signature block asks for a name, an institution and an email.
The agreement has no written rule about faculty status or institutional email.
A private individual is not excluded by the text.
Whether Stirling accepts a signer without an institution is not verified.

### Licence

The agreement says "The images may be used for research purposes only." (http://pics.stir.ac.uk/ESRC/license%20agreement.pdf).
It says "Images may not be passed to a third party, beyond the immediate research group of the person signing the agreement."
It also says that a user who adds value to the images "must offer them back to the database."
The agreement does not mention derived models.

### Size

The raw neutral scans are 314 MB for men and 354 MB for women.
The registered neutral meshes are 12 MB for men and 9 MB for women.

## Florence 2D/3D

The Florence dataset comes from the University of Florence (https://www.micc.unifi.it/resources/datasets/florence-3d-faces/).
The MICA repository lists it with 53 subjects (https://github.com/Zielon/MICA/tree/master/datasets).
The count is not verified on the Florence page itself.
The scans were made with a 3dMD face scanner (https://3dmd.com/2012/05/02/florence-faces/).
Access goes through a Google form (https://forms.gle/AvgSzckLZUvMPX9X8).
The form could not be opened without a Google sign-in.
The access conditions and the licence of the scans are not verified.

## Bosphorus

The Bosphorus site http://bosphorus.ee.boun.edu.tr/ no longer resolves.
Every fact about this dataset is therefore not verified.
A library record gives 105 subjects, with 60 men and 45 women (https://service.tib.eu/ldmservice/dataset/bosphorus-database-for-3d-face-analysis).
An indexed copy of the access page said that students are not eligible and that a department director must co-sign.
It is unclear whether the dataset is still distributed.

## FRGC v2

FRGC stands for Face Recognition Grand Challenge, and the University of Notre Dame distributes it.
The validation set has 466 subjects in 4,007 sessions (https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=150264).
The data is range scans with texture from a Minolta Vivid sensor.
A range scan records only the surface that faces the sensor, so these scans cover the front of the face.
The licence must be signed "by an individual authorized to make legal commitments in the name of your organization" (https://cvrl.nd.edu/projects/data/).
The same page says the signed licence must come from an "INSTITUTIONAL e-mail address".
A private individual cannot get FRGC v2.
The licence says the data "will not be further distributed, published, copied, or disseminated in any way or form whatsoever, whether for profit or not" (https://cvrl.nd.edu/media/django-summernote/2018-09-19/c7654649-5277-4d8c-b069-483d8ffa3039.pdf).
The download is about 72 GB.

## Truly open datasets

### Real heads

No dataset under CC0, CC BY, public domain or a permissive licence with 50 or more real heads was found.
The open sets of real people that were found are all small.

HSRD-100 has 10 people aged 21 to 64, with 5 men and 5 women (https://huggingface.co/datasets/digitalrealitylab/HSRD-100).
Each person has 10 full-body clothed scans with 16K textures.
The card says "This dataset is licensed under Creative Commons Attribution 4.0 International (CC BY 4.0)".
The repository is about 246 GB, and single files can be downloaded separately.
The head detail is that of a full-body scan.

The Infinite 3D Head Scan by Lee Perry-Smith is 1 head under CC BY 3.0 (http://web.archive.org/web/20221028232246/http://ir-ltd.net/infinite-3d-head-scan-released/).
The original page now returns an error.

The Smithsonian 3D collection has fewer than 10 human heads under CC0 (https://3d.si.edu/cc0).
Only the Abraham Lincoln life masks record a real face.
The other items are sculpted busts.
The exact count is not verified.

The NIOSH headforms are 5 averaged head shapes (https://archive.cdc.gov/www_cdc_gov/niosh/npptl/topics/respirators/headforms/default.html).
NIOSH is the United States National Institute for Occupational Safety and Health.
Each headform is the average of 5 people.
The public domain status of the files is not verified.

Sketchfab has a licence filter for CC0 and CC BY (https://api.sketchfab.com/v3/licenses).
No curated collection of 50 or more real people under those licences was found.
One user collection holds about 45 to 55 head scans marked CC BY (https://api.sketchfab.com/v3/collections/f0d3a2291fa4416d820219d5b59eb5b5).
That count is an estimate and is not verified.
The scans vary a lot in quality.
Nothing confirms that the scanned people gave consent.

### Commercial scan vendors

3D Scan Store offers about 2 or 3 free heads for personal use (https://www.3dscanstore.com/blog/Free-3D-Head-Model).
Its licence forbids the user to "Release the 3D models or scans or derivative products under Open Source Licences" (https://www.3dscanstore.com/index.php?route=information/information&information_id=5).
Renderpeople offers free full-body models (https://renderpeople.com/free-3d-people/).
Its terms forbid "the creation of datasets, the development and training of neural networks and machine learning models" without written consent (https://renderpeople.com/general-terms-and-conditions/).
The Triplegangers licence page could not be opened, so its terms are not verified.

### Synthetic and artist-made sets

Microsoft FaceSynthetics is the dataset of the paper "Fake It Till You Make It" (https://github.com/microsoft/FaceSynthetics).
It contains 100,000 rendered images with 2D landmarks and no 3D meshes.
Its licence requires the user to "use the Data solely for Computational Use for non-commercial research" (https://raw.githubusercontent.com/microsoft/FaceSynthetics/main/LICENSE.txt).
It is neither open nor useful as a 3D benchmark.
Microsoft SynthMoCap has the same licence and ships no head meshes (https://github.com/microsoft/SynthMoCap).

MakeHuman is an open-source character creator.
Its base mesh and its targets "have been released under CC0 1.0 Universal" (https://raw.githubusercontent.com/makehumancommunity/makehuman/master/LICENSE.md).
A target is an artist-made shape change, such as a wider jaw.
The targets are not scans of real people.

The Blender Studio Human Base Meshes are artist-made and are reported as CC0 (https://www.blender.org/download/demo-files/).
The CC0 statement was seen only in a search result and is not verified.

### Other research sets

FaceVerse has 128 identities and needs a form signed by "a full-time academic staff member" (https://github.com/LizhenWangT/FaceVerse-Dataset).
RenderMe-360 has 500 people and needs an institutional email address (https://renderme-360.github.io/inner-download.html).
CustomHumans has 81 people and rejects "Requests from non-institutional e-mail accounts" (https://custom-humans.ait.ethz.ch/).
The 3D Facial Norms database has about 2,447 participants, and its 3D surfaces are under controlled access (https://www.facebase.org/facial_norms/).
CAESAR is sold and is not open, and its price is not verified.

## Ranked recommendation

### Datasets a private individual can realistically obtain

1. Ava-256 has 256 people and an open download. It is the largest option. The 4 TB default size is the obstacle, and a mesh-only download is not verified.
2. FaMoS has 95 people in the FLAME topology. It needs only a free account.
3. The NoW validation set has 20 raw neutral scans. Its subjects were chosen to vary in age, body mass index and sex.
4. Multiface has 13 people, and the project already uses it.
5. CoMA, VOCASET and the D3DFACS registrations add 12, 12 and 10 people. They need only a free account.
6. HSRD-100 has 10 people under CC BY 4.0. It is the only set here with no non-commercial condition.
7. The Basel 2009 example scans add 10 faces with no login.
8. Stirling/ESRC has 99 people and H3DS has 59 people. Both need a request, and the answer for a private individual is not verified.

Headspace, FaceScape, BU-3DFE, BP4D, 3DFAW-Video, FRGC v2, FaceVerse, RenderMe-360 and CustomHumans are closed to a private individual.
FaceWarehouse, NPHM, Florence and Bosphorus are uncertain or unlikely.

The Max Planck licences allow "non-commercial scientific research".
None of them mentions benchmarking for an open-source game.
Whether a hobby project counts as scientific research is a judgement that the licences do not settle.
The Max Planck Institute gives ps-license@tue.mpg.de as its contact for licence questions.
The CC BY-NC 4.0 licence of Ava-256 and Multiface has no research condition, so private benchmarking with them is clearly allowed.

### Best benchmark for all ages, weights and sexes without hair

1. Headspace would be the best fit. It has 1519 people of recorded age and sex, and every subject wears a cap. It is closed to a private individual. A written request to the University of York for an exception is the only route, and its outcome is not verified.
2. Stirling/ESRC is the best fit that might be obtainable. It has 99 people who wear a white cap. Its age range and weight range are not stated.
3. The NoW validation scans are the best fit that is obtainable today. They vary in age, body mass index and sex. They cover only 20 people, and caps are not verified.
4. Ava-256 gives the most people. Its age range, its weight range and the presence of hair are not verified.
5. FLAME 2023 Open can stand in for missing variety. Random heads sampled from the model are hairless and reflect 3800 scanned people of many ages and weights. These heads are model output and not real people, so they test coverage of the FLAME shape space only.

FaceScape covers ages 16 to 70 with caps, but it is closed and its subjects are mostly from Asia.
The Basel Face Model 2009 is the only source that states a weight range, which is 40 to 123 kilograms, but its scans are not distributed.

### Sources that allow shipping derived shapes

1. ICT FaceKit is under MIT. Its 100 PCA shapes can ship in this repository with no added condition.
2. The MakeHuman base mesh and targets are under CC0. They can ship with no condition. They are artist-made.
3. FLAME 2023 Open is under CC BY 4.0. Shapes derived from it can ship if the repository credits the Max Planck Institute, links the licence, states the changes, and carries the two use restrictions. Those shapes stay under CC BY 4.0 and do not become MIT.
4. HSRD-100 and the Infinite head scan are under CC BY. Shapes derived from them can ship with credit. They cover only 11 people.

No other dataset in this document allows it.
The Max Planck data licences forbid making modified data available to third parties.
CC BY-NC 4.0 keeps its non-commercial condition on adapted material, which conflicts with MIT.
FaceWarehouse and the Basel Face Model forbid derived shapes in words.
The research-only agreements of Headspace, FaceScape, Stirling, H3DS and FRGC do not mention derived shapes, so shipping them would need written permission from each owner.

Hand-tuned slider values are a grey area.
A person who adjusts sliders while reading benchmark error numbers copies far less from the data than an average or a PCA shape does.
No licence read for this document addresses that case.

## Pages that could not be read

- The Google forms for NPHM, H3DS and Florence need a Google sign-in.
- The Bosphorus site and the old FaceScape site no longer resolve.
- The download pages of FLAME and CoMA need a login, so file lists and sizes are not verified.
- The Triplegangers licence page, the Smithsonian CC0 page and the Blender demo files page refused automated reading.
- The Binghamton agreement for BU-3DFE and BP4D is not published online.
