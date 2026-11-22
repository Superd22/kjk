//! A hairstyle file: a few curves and sliders, written by hand in JSON.
//!
//! Places on the head are given as angles around the head's centre (see `frame`). The azimuth `az` is 0 at the front of the face, 90 on
//! the character's left and -90 on the character's right, and 180 at the back. The elevation `el` is 90 at the top of the head and
//! negative below the centre. A point of a curve is `[az, el, lift]`, where the lift is the height in millimetres above the hair's
//! surface (the scalp pushed out by the volume). Lengths are in millimetres.
//!
//! When `mirror` is true, every curve, bump, zone and the hairline are copied to the other side of the face with the azimuth negated.
//! An element can opt out with its own `"mirror": false`.

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Style {
    pub name: String,
    /// The KOTOR head this style is drawn for, such as `p_bastilah`.
    pub head: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub mirror: bool,
    #[serde(default = "default_seed")]
    pub seed: u64,
    pub colour: Colours,
    /// The front and side edge of the hair, as `[az, el]` knots. The hair grows where the elevation is above the line.
    pub hairline: Vec<[f64; 2]>,
    pub volume: Volume,
    pub length: Length,
    #[serde(default)]
    pub grid: Grid,
    #[serde(default = "default_strands")]
    pub strands: usize,
    /// A strand that follows no lock lies on the cap: it is kept within this height above the shell, and it ends where it leaves the
    /// cap, `cap.fade_mm` past the hairline.
    #[serde(default = "default_hold")]
    pub hold_mm: f64,
    #[serde(default)]
    pub streams: Vec<Stream>,
    #[serde(default)]
    pub partings: Vec<Parting>,
    #[serde(default)]
    pub ponytails: Vec<Ponytail>,
    #[serde(default)]
    pub curl: Curl,
    #[serde(default)]
    pub cap: Cap,
    #[serde(default)]
    pub paint: Paint,
}

fn default_seed() -> u64 {
    1
}

fn default_strands() -> usize {
    3000
}

fn default_hold() -> f64 {
    3.0
}

/// Hair colours as `#rrggbb`. The roots are `root`, the tips `tip`, and `painted` is the colour of the hair painted on the skin.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Colours {
    pub root: String,
    pub tip: String,
    pub painted: String,
    /// How much single strands differ in brightness, from 0 (not at all) to about 0.5.
    #[serde(default = "default_variation")]
    pub variation: f64,
}

fn default_variation() -> f64 {
    0.25
}

/// The height of the hair above the scalp.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Volume {
    /// The height everywhere inside the hairline.
    pub base_mm: f64,
    /// The height falls to nothing over this distance toward the hairline.
    pub taper_mm: f64,
    #[serde(default)]
    pub bumps: Vec<Bump>,
}

/// A local change of a value around a direction. Its weight is 1 at the centre and falls smoothly to 0 at `radius_deg`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bump {
    pub az: f64,
    pub el: f64,
    pub radius_deg: f64,
    pub mm: f64,
    pub mirror: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Length {
    /// The length of a strand, unless a zone changes it.
    pub default_mm: f64,
    /// Places where strands have another length. Each zone pulls the length toward its `mm` by its weight.
    #[serde(default)]
    pub zones: Vec<Bump>,
}

/// The grid of the flow field around the head.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grid {
    /// How many cells the longest side of the box has.
    pub cells: usize,
    /// How far the box reaches below the head's centre, in millimetres.
    pub below_mm: f64,
}

impl Default for Grid {
    fn default() -> Self {
        Grid { cells: 24, below_mm: 160.0 }
    }
}

/// A stream curve shows the flow. Grid points near it point along it. When `lock` is set, the strands that follow the curve are loose
/// hair and become cards.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stream {
    pub points: Vec<[f64; 3]>,
    /// How far the curve reaches, in grid cells.
    #[serde(default = "default_reach")]
    pub reach_cells: f64,
    pub lock: Option<Lock>,
    pub mirror: Option<bool>,
}

fn default_reach() -> f64 {
    1.0
}

/// Loose hair made into cards.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lock {
    pub cards: usize,
    /// The narrowest and the widest a card may be, in millimetres.
    pub width_mm: [f64; 2],
    /// How many quads a card has along its length.
    #[serde(default = "default_segments")]
    pub segments: usize,
    /// When true, each card gets a second strip at a right angle to it, which gives tails and buns volume from every side.
    #[serde(default)]
    pub crossed: bool,
    /// The card's upper edge is lifted off the head by this angle, in degrees, so that bangs stand off the forehead.
    #[serde(default)]
    pub lean_deg: f64,
}

fn default_segments() -> usize {
    4
}

/// A dividing curve on the hair's surface: hair on its two sides flows away from it in opposite directions.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parting {
    /// `[az, el]` points on the hair's surface.
    pub points: Vec<[f64; 2]>,
    pub mirror: Option<bool>,
}

/// A ponytail: hair near the start of the spine is drawn toward it, then flows along the spine.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ponytail {
    pub spine: Vec<[f64; 3]>,
    /// The radius around the spine's start inside which hair is gathered.
    pub gather_mm: f64,
    /// How strongly hair is drawn toward the spine's start. Fu et al. use 0.6.
    #[serde(default = "default_tightness")]
    pub tightness: f64,
    pub lock: Lock,
    pub mirror: Option<bool>,
}

fn default_tightness() -> f64 {
    0.6
}

/// A twist along each strand. With `wave` true, the strand swings from side to side in one plane instead of turning in a helix.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Curl {
    pub amplitude_mm: f64,
    #[serde(default = "default_period")]
    pub period_mm: f64,
    #[serde(default)]
    pub wave: bool,
    /// The curl grows from nothing at the root to its full size over this length.
    #[serde(default)]
    pub ramp_mm: f64,
}

fn default_period() -> f64 {
    40.0
}

/// The hair cap.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cap {
    /// The size of the cap's quads, in millimetres on the head.
    pub cell_mm: f64,
    /// The cap reaches this far past the hairline, where its texture fades out.
    pub fade_mm: f64,
}

impl Default for Cap {
    fn default() -> Self {
        Cap { cell_mm: 18.0, fade_mm: 6.0 }
    }
}

/// The hair painted into the head's texture.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Paint {
    /// The painted hair thins out over this distance past the hairline.
    pub band_mm: f64,
    /// How opaque the painted hair is, from 0 to 1.
    pub opacity: f64,
    /// Short hair painted outside the hairline, such as sideburns and the hair at the temples. Here a zone's `mm` is its density, from
    /// 0 to 1.
    #[serde(default)]
    pub zones: Vec<Bump>,
    /// The edge of the painted hair, as `[az, el]` knots, when it lies lower than the cap's hairline, as for short hair on the sides.
    /// Without it the painted hair follows the cap's hairline.
    pub hairline: Option<Vec<[f64; 2]>>,
    /// When true (the default), the painted hair starts where the cap and the cards meet the skin, and frays past that outline. When
    /// false, it starts at the style's hairline.
    #[serde(default = "default_true")]
    pub follow_mesh: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Paint {
    fn default() -> Self {
        Paint { band_mm: 5.0, opacity: 0.9, zones: Vec::new(), hairline: None, follow_mesh: true }
    }
}

/// A line of `[az, el]` knots with, when `mirror` is true, a copy of each knot on the other side, sorted by azimuth.
fn mirrored_line(knots: &[[f64; 2]], mirror: bool) -> Vec<[f64; 2]> {
    let mut out = knots.to_vec();
    if mirror {
        for k in knots {
            if k[0].abs() > 1e-9 && k[0].abs() < 180.0 - 1e-9 {
                out.push([-k[0], k[1]]);
            }
        }
    }
    out.sort_by(|a, b| a[0].total_cmp(&b[0]));
    out
}

fn mirrored_point(p: [f64; 3]) -> [f64; 3] {
    [-p[0], p[1], p[2]]
}

impl Style {
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// The style with every mirrored element copied to the other side, so that the rest of the code never looks at the mirror flags.
    pub fn expanded(&self) -> Style {
        let mut s = self.clone();
        let on = |own: Option<bool>| own.unwrap_or(self.mirror);
        let bumps = |list: &[Bump]| -> Vec<Bump> {
            let mut out = list.to_vec();
            for b in list {
                if on(b.mirror) && b.az.abs() > 1e-9 && b.az.abs() < 180.0 - 1e-9 {
                    out.push(Bump { az: -b.az, mirror: Some(false), ..b.clone() });
                }
            }
            out
        };
        s.volume.bumps = bumps(&self.volume.bumps);
        s.length.zones = bumps(&self.length.zones);
        s.paint.zones = bumps(&self.paint.zones);
        for st in &self.streams {
            if on(st.mirror) {
                s.streams.push(Stream { points: st.points.iter().map(|&p| mirrored_point(p)).collect(), mirror: Some(false), ..st.clone() });
            }
        }
        for p in &self.partings {
            if on(p.mirror) {
                s.partings.push(Parting { points: p.points.iter().map(|q| [-q[0], q[1]]).collect(), mirror: Some(false) });
            }
        }
        for p in &self.ponytails {
            if on(p.mirror) {
                s.ponytails.push(Ponytail { spine: p.spine.iter().map(|&q| mirrored_point(q)).collect(), mirror: Some(false), ..p.clone() });
            }
        }
        s.hairline = mirrored_line(&self.hairline, self.mirror);
        s.paint.hairline = self.paint.hairline.as_deref().map(|k| mirrored_line(k, self.mirror));
        s.mirror = false;
        s
    }

    /// The elevation of the hairline at an azimuth, by straight lines between the knots, wrapping round the back of the head.
    pub fn hairline_el(&self, az: f64) -> f64 {
        hairline_el(&self.hairline, az)
    }
}

/// The elevation of a line of `[az, el]` knots, sorted by azimuth, at an azimuth.
pub fn hairline_el(knots: &[[f64; 2]], az: f64) -> f64 {
    match knots.len() {
        0 => -90.0,
        1 => knots[0][1],
        n => {
            let wrap = |a: f64| (a + 180.0).rem_euclid(360.0) - 180.0;
            let az = wrap(az);
            for k in 0..n {
                let (a, b) = (knots[k], knots[(k + 1) % n]);
                let span = if k + 1 < n { b[0] - a[0] } else { b[0] + 360.0 - a[0] };
                let mut x = az - a[0];
                if k + 1 == n && x < 0.0 {
                    x += 360.0;
                }
                if (0.0..=span).contains(&x) {
                    let t = if span > 1e-9 { x / span } else { 0.0 };
                    return a[1] + t * (b[1] - a[1]);
                }
            }
            // Before the first knot: on the segment that wraps from the last knot.
            let (a, b) = (knots[n - 1], knots[0]);
            let span = b[0] + 360.0 - a[0];
            let t = (az + 360.0 - a[0]) / span;
            a[1] + t * (b[1] - a[1])
        }
    }
}

/// The weight of a bump at an angle of `angle_deg` from its centre: 1 at the centre, falling smoothly to 0 at its radius.
pub fn bump_weight(angle_deg: f64, radius_deg: f64) -> f64 {
    let x = (angle_deg / radius_deg).clamp(0.0, 1.0);
    let w = 1.0 - x * x;
    w * w
}

/// Reads a colour written as `#rrggbb` into sRGB values from 0 to 1.
pub fn parse_colour(hex: &str) -> [f64; 3] {
    let h = hex.trim_start_matches('#');
    let channel = |i: usize| u8::from_str_radix(h.get(i..i + 2).unwrap_or("80"), 16).unwrap_or(128) as f64 / 255.0;
    [channel(0), channel(2), channel(4)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hairline_interpolates_and_wraps_round_the_back() {
        let knots = vec![[-90.0, 0.0], [0.0, 30.0], [90.0, 0.0], [180.0, -40.0]];
        assert!((hairline_el(&knots, 45.0) - 15.0).abs() < 1e-9);
        assert!((hairline_el(&knots, 135.0) + 20.0).abs() < 1e-9);
        // From 180 at -40 to -90 at 0: halfway is -135.
        assert!((hairline_el(&knots, -135.0) + 20.0).abs() < 1e-9);
        assert!((hairline_el(&knots, -180.0) + 40.0).abs() < 1e-9);
    }

    #[test]
    fn mirroring_copies_curves_with_the_azimuth_negated() {
        let text = r##"{"name":"t","head":"h","mirror":true,"colour":{"root":"#000000","tip":"#ffffff","painted":"#101010"},
            "hairline":[[0,20],[90,0],[180,-40]],"volume":{"base_mm":5,"taper_mm":5},"length":{"default_mm":100},
            "streams":[{"points":[[30,40,0],[60,0,0]]},{"points":[[0,60,0],[0,10,0]],"mirror":false}]}"##;
        let s = Style::from_json(text).expect("style").expanded();
        assert_eq!(s.streams.len(), 3);
        assert_eq!(s.streams[2].points[0], [-30.0, 40.0, 0.0]);
        assert_eq!(s.hairline.len(), 4);
        assert!((s.hairline_el(-90.0) - 0.0).abs() < 1e-9);
    }
}
