//! The processor's preparation of one head: the per-vertex fields the paint pass interpolates, and the head's fine detail (its wrinkle
//! lines and pore landmarks), which the layers turn into their own buffers.

use head_paint::live::Layout;
use head_paint::paint::HeadDetail;
use head_paint::pores::PoreLandmarks;
use head_paint::shading::ShapeMaps;
use head_paint::sheen::{SheenLandmarks, sheen_weights};

/// How many 32-bit floats the vertex buffer holds per layout vertex: the layout point (2), the sky light, the bump in millimetres, the
/// three sheen weights, and the point of the head in millimetres (3).
pub const VERTEX_FLOATS: usize = 10;

/// One head, ready to paint.
pub struct PreparedHead {
    /// The head's mesh positions in metres.
    pub positions: Vec<[f64; 3]>,
    /// The head's wrinkle lines, pore landmarks and fold anchors, as `HeadDetail::new` builds them. `None` on a mesh without
    /// the landmarks, where the fine detail layers paint nothing.
    pub detail: Option<HeadDetail>,
    /// `VERTEX_FLOATS` numbers per layout vertex. The point of the head is stored relative to `origin_mm`.
    pub vertices: Vec<f32>,
    /// The middle of the head's bounding box in millimetres, rounded to a whole millimetre. A head's points lie up to 700 mm from the
    /// mesh's origin, and the GPU interpolates them in 32-bit floats, so they are interpolated relative to this point and it is added
    /// back per pixel. That makes the interpolation error about 7 times smaller.
    pub origin_mm: [f64; 3],
    /// Which vertices the eye paint reads the lids' edges from. The painter sets it (`GpuPainter::set_eye_edge`).
    pub eye_edge: head_paint::eyes::LidEdge,
}

impl PreparedHead {
    /// Computes the per-vertex fields (the sky light, the bump, the sheen's region weights and the position), before the GPU interpolates them
    /// to pixels.
    pub fn new(layout: &Layout, positions: Vec<[f64; 3]>) -> Self {
        let maps = ShapeMaps::compute(&positions, &layout.mesh_triangles);
        let sheen = sheen_weights(&positions, &SheenLandmarks::OUR_HEAD);
        let mut low = [f64::MAX; 3];
        let mut high = [f64::MIN; 3];
        for p in &positions {
            for a in 0..3 {
                low[a] = low[a].min(1000.0 * p[a]);
                high[a] = high[a].max(1000.0 * p[a]);
            }
        }
        let origin_mm = [0, 1, 2].map(|a| (0.5 * (low[a] + high[a])).round());
        let mut vertices = Vec::with_capacity(VERTEX_FLOATS * layout.uv.len());
        for (uv, &m) in layout.uv.iter().zip(&layout.layout_to_mesh) {
            let p = positions[m];
            let fields = [uv[0], uv[1], maps.sky[m], maps.bump_mm[m], sheen[m][0], sheen[m][1], sheen[m][2], 1000.0 * p[0] - origin_mm[0], 1000.0 * p[1] - origin_mm[1], 1000.0 * p[2] - origin_mm[2]];
            vertices.extend(fields.map(|v| v as f32));
        }
        let detail = HeadDetail::new(&layout.uv, &layout.triangles, &layout.layout_to_mesh, &positions);
        PreparedHead { positions, detail, vertices, origin_mm, eye_edge: Default::default() }
    }

    /// The words of the shared `Marks` uniform of `common.wgsl`: the pore landmarks, or zeros with `present` at 0 on a mesh without them.
    pub fn marks_words(&self) -> Vec<u32> {
        marks_words(self.detail.as_ref().map(|d| &d.landmarks))
    }
}

/// The words of the `Marks` uniform for some landmarks.
pub fn marks_words(marks: Option<&PoreLandmarks>) -> Vec<u32> {
    let Some(m) = marks else { return vec![0; 44] };
    let points = [m.nose[0], m.nose[1], m.forehead, m.chin, m.mouth[0], m.mouth[1], m.eyes[0][0], m.eyes[0][1], m.eyes[1][0], m.eyes[1][1]];
    let mut words = Vec::with_capacity(44);
    for p in points {
        words.extend([p[0], p[1], p[2], 0.0].map(|v| (v as f32).to_bits()));
    }
    words.extend([1, 0, 0, 0]);
    words
}
