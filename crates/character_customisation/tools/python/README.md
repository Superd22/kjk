# Likeness fit

This script takes a front photo of a person and prints a share code for the Likeness Lab prototype. The share code sets the face proportions and the face averages (gender, age and region). The paint and the hair keep the page's defaults, and a person finishes them by hand in the page.

## Running it

```
.venv/bin/python -I fit.py photo.jpg --gender 1 --age 0.4 --regions 0,0,1 --overlay check.png | pbcopy
```

The share code goes to standard output. The report goes to standard error. The report lists the photo checks, the ten largest shape values and the residual of each landmark before and after the fit.

- `--gender` goes from −1 (female) to 1 (male). `--age` goes from 0 (about 25 years) to 1 (about 90 years), as in MakeHuman. `--regions` gives the African, Asian and Caucasian weights. All three default to the page's average face.
- `--fit-averages` searches gender, age and region instead of taking them from the options. It works on renders of the head. On real photos it drifts to the extremes, so the person running the script should give these three values.
- `--caricature 1.4` multiplies every shape value before writing the code. The page's caricature slider does the same thing live.
- `--asym` fits the left and right sides separately and uses the asymmetry shapes. By default both sides are tied together.
- `--page likeness-lab.html` takes the share code version and the paint defaults from a copy of the page. Without it, the script writes version 2 with the defaults of 2026-11-16.
- `--force` writes a code even when the photo checks refuse the photo.

The script refuses a photo when the face is turned more than 12 degrees, tilted more than 20 degrees, the mouth is open, or one side of the face is more than 2.2 times brighter than the other. It warns about smaller turns and tilts, smiles, closed eyes, small faces and several faces. Level studio portraits read 5 to 10 degrees of tilt, which is why the tilt limits are wider.

## How it works

1. MediaPipe's face landmarker finds 478 points on the photo. When the face is small in the frame, the script retries on smaller crops.
2. `correspondence.json` pairs 38 of those points with vertices of the head. The pupils use the eyeball centres. `pick_correspondence.py` chose the vertices once by rendering the average head and taking the visible vertex nearest to each detected point. Left and right picks are mirrored.
3. Both sides are put in the same frame. The midpoint between the pupils is the origin, the line between the pupils is level, and the distance between the pupils is 1. The head is projected straight onto the front plane.
4. The solver finds the shape values with bounded least squares. Each shape is split into its two one-sided targets, each between 0 and 1. A ridge penalty keeps values small when the landmarks do not need them. The solve repeats three times, because the eye frame moves as the head changes.
5. The solver models the detector. `calibrate.py` rendered 60 random faces and measured, for each landmark and axis, how far the detected point moves when the true point moves (the gain) and how much noise it has. A noisy landmark counts for less. The gain is kept at 0.5 or more, so a small difference in a photo is never multiplied more than twice.

The ears and the neck are not fitted, because no front landmark sees them.

## Test results

`test_roundtrip.py` builds random faces, renders them with painted brows and lips, fits the render and compares the fitted head with the true one. The error is measured over the front of the face, with each head scaled to an eye distance of 1.

| Setting, 20 faces | Recovered in the photo plane | Recovered in depth |
|---|---|---|
| Gender and age given, flat projection | 51% | 22% |
| Gender and age searched | 50% | 18% |
| Gender and age given, the page's camera distance | 49% | 21% |
| Exact landmarks instead of the detector | 60% | 9% |

The last row is the ceiling for these 38 landmarks. With about 380 exact landmarks the ceiling rises to 74%. But MediaPipe's extra points do not add anything, because they follow its own face model more than the photo.

A run takes about 1.5 seconds. The script was also run on public-domain NASA portraits. Most of them were refused for open smiles, which is the expected result.

The share code was checked against the page's own decode and deform code, run in Node. Both build the same head to within 0.0002 mm.

## Limits

- The fit sets proportions. Proportions rank low for recognition. Eyebrows, hair, eye colour and the lip edge rank higher, and those are paint.
- The calibration was measured on renders. A real photo may respond differently, mostly at the eye corners, where the measured gain was low.
- A front photo shows almost nothing about depth. The nose length and the chin projection stay close to the average.
- MakeHuman's averages were built mostly from white faces. The fit will be coarser on other faces.
- A profile photo is not supported yet.

## Files

- `fit.py` is the script.
- `head.py` loads the head and builds share codes. `render.py` draws test faces. `detect.py` runs MediaPipe and the photo checks.
- `pick_correspondence.py` and `calibrate.py` rebuild `correspondence.json`. Run the picker first, then the calibration.
- `export_mh.py` rebuilds the head data from the MakeHuman assets.
- The head data and the MediaPipe model are in `../likeness-fit-data`.
