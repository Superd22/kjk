//! The GPU painter in the browser, through WebGPU, for the live paint page.
//!
//! A page makes one `GpuPainter` with `GpuPainter.create(...)`, gives it a head with `set_head`, prepares a size once with `prepare`, then
//! calls `paint` with one number per control and one switch per layer every time a slider moves, and `read` to get the texture's bytes.
//! The module also exports `head_paint`'s own browser functions, such as `controls_json` and `memory_bytes`.
//! The painter is shared between the page's calls and the futures they return, so no future holds it while it waits for the GPU.

use crate::eyeball::EyeballPainter;
use crate::gpu::Gpu;
use crate::painter;
use crate::probe;
use head_paint::controls;
use head_paint::hair_groups::{GroupKind, HairGroup, HairGroups, History};
use head_paint::live::Layout;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// Fills a `size` by `size` texture on the GPU and returns the RGBA bytes read back.
#[wasm_bindgen]
pub async fn probe_fill(size: u32) -> Result<Vec<u8>, JsError> {
    let gpu = Gpu::request().await.map_err(|e| JsError::new(&e))?;
    Ok(probe::fill(&gpu, size).await)
}

/// How many texels of `rgba` differ from the probe's pattern.
#[wasm_bindgen]
pub fn probe_mismatches(rgba: &[u8], size: u32) -> u32 {
    probe::mismatches(rgba, size) as u32
}

/// The ids of the controls that the GPU painter does not paint yet, as a JSON list.
#[wasm_bindgen]
pub fn unpainted_controls_json() -> String {
    serde_json::to_string(&crate::layers::unpainted_controls()).expect("json")
}

/// The detail tile's side in mm, its size in texels, and the skin a texel of the head's texture covers at 2,048 pixels, in mm. A viewer
/// repeats the tile `texture_mm / tile_mm` times over the texture, where `texture_mm` is 2,048 times the last.
#[wasm_bindgen]
pub fn tile_sizes() -> Vec<f64> {
    vec![crate::tile::TILE_MM, f64::from(crate::tile::TILE_PIXELS), crate::layers::maps::TILE_TEXEL_MM]
}

#[wasm_bindgen]
pub struct GpuPainter {
    inner: Rc<RefCell<painter::GpuPainter>>,
    /// The eyeballs' texture, which is painted apart from the head's layout (`eyeball.rs`).
    balls: Rc<EyeballPainter>,
    /// The positions of the head last set, for where each iris sits in its eye's opening.
    positions: RefCell<Vec<[f64; 3]>>,
    /// The steps of undo and redo of the hair groups.
    history: RefCell<History>,
    /// The flow line being shaped, as its group's number and its own, or `None` while a group's own points are shaped.
    flow: Cell<Option<(usize, usize)>>,
}

fn triples(flat: &[u32]) -> Vec<[u32; 3]> {
    flat.as_chunks::<3>().0.to_vec()
}

#[wasm_bindgen]
impl GpuPainter {
    /// Makes a painter on the browser's GPU. `uv` holds two numbers per layout vertex, `triangles` three layout vertex numbers per
    /// triangle, `layout_to_mesh` the mesh vertex of each layout vertex, and `mesh_triangles` three mesh vertex numbers per triangle.
    pub async fn create(uv: Vec<f64>, triangles: Vec<u32>, layout_to_mesh: Vec<u32>, mesh_triangles: Vec<u32>) -> Result<GpuPainter, JsError> {
        let gpu = Gpu::request().await.map_err(|e| JsError::new(&e))?;
        let layout = Layout {
            uv: uv.as_chunks::<2>().0.to_vec(),
            triangles: triples(&triangles),
            layout_to_mesh: layout_to_mesh.iter().map(|&i| i as usize).collect(),
            mesh_triangles: triples(&mesh_triangles),
        };
        let balls = Rc::new(EyeballPainter::new(&gpu));
        Ok(GpuPainter { inner: Rc::new(RefCell::new(painter::GpuPainter::new(gpu, layout))), balls, positions: RefCell::new(Vec::new()), history: RefCell::new(History::default()), flow: Cell::new(None) })
    }

    /// Puts a head under the paint. `positions` holds three numbers per mesh vertex, in metres.
    pub fn set_head(&self, positions: &[f64]) {
        let p = positions.as_chunks::<3>().0.to_vec();
        *self.positions.borrow_mut() = p.clone();
        self.inner.borrow_mut().set_head(p);
    }

    /// Makes the eye paint and the irises read each lid's edge from the rim that the eye curve lays on the visible edge of the opening
    /// (`head_paint::eyes::RIMS`), for a head built on the curve, or from the lid edge loop when `rim` is false, as before.
    pub fn set_eye_rim(&self, rim: bool) {
        self.inner.borrow_mut().set_eye_edge(if rim { head_paint::eyes::LidEdge::Rim } else { head_paint::eyes::LidEdge::Loop });
    }

    /// Paints the eyeballs' texture from `values`, one number per control, and submits the work. Each iris sits in the middle of its
    /// eye's opening on the head last set, moved by the gaze sliders. The texture is `eyeball_size` texels high and twice as wide.
    pub fn paint_eyeballs(&self, values: &[f64]) {
        let ball = controls::recipe_from(values).eyes.ball;
        let centres = head_paint::eyeball::iris_centres_on(&self.positions.borrow(), &ball, self.inner.borrow().eye_edge());
        let radii = head_paint::eyes::eyeball_radii(&self.positions.borrow());
        let gpu = self.inner.borrow().gpu().clone();
        self.balls.paint(&gpu, &ball, centres, radii);
    }

    /// The height of the eyeballs' texture in texels: each eye's square is this wide.
    pub fn eyeball_size(&self) -> u32 {
        self.balls.size()
    }

    /// Where each iris lies in its eye's square of the eyeballs' texture, for the viewer's light inside the iris. For the texture's left
    /// half and then its right half: the iris centre's x and y, the iris radius, and the pupil radius as a share of the iris radius. The
    /// square's half width is 1, as in `shaders/eyeball.wgsl`.
    pub fn eyeball_irises(&self, values: &[f64]) -> Vec<f32> {
        let ball = controls::recipe_from(values).eyes.ball;
        let positions = self.positions.borrow();
        let centres = head_paint::eyeball::iris_centres_on(&positions, &ball, self.inner.borrow().eye_edge());
        let radii = head_paint::eyes::eyeball_radii(&positions);
        [1, 0]
            .into_iter()
            .flat_map(|e| [centres[e][0] / radii[e], centres[e][1] / radii[e], ball.iris_share(radii[e]), ball.get(head_paint::eyeball::k::PUPIL)])
            .map(|v| v as f32)
            .collect()
    }

    /// Resolves to the last eyeballs' texture painted, as a `Uint8Array` of RGBA bytes, row by row from the top. The eye on the head's
    /// x > 0 side is the left half, as the viewer's eyeball mesh maps it.
    pub fn read_eyeballs(&self) -> js_sys::Promise {
        let gpu = self.inner.borrow().gpu().clone();
        let buffer = self.balls.texels().clone();
        let n = u64::from(self.balls.size());
        wasm_bindgen_futures::future_to_promise(async move {
            let bytes = gpu.read(&buffer, 8 * n * n).await;
            Ok(js_sys::Uint8Array::from(bytes.as_slice()).into())
        })
    }

    /// Sets the values written straight onto the facial hair's grid, from the JSON of `hair_grid::GridEdit::from_json`, or clears them for
    /// an empty text. Returns false when the text could not be read.
    pub fn set_hair_edit(&self, json: &str) -> bool {
        let edit = if json.trim().is_empty() { None } else { head_paint::hair_grid::GridEdit::from_json(json) };
        let ok = json.trim().is_empty() || edit.is_some();
        self.inner.borrow_mut().set_hair_edit(edit);
        ok
    }

    /// The paint grid on the head last set, for drawing it: for each point on the skin, eleven numbers, its place and its skin's normal
    /// in metres, its density in hairs per square millimetre, its direction of growth, and the number of its zone in `face_grid::ZONES`.
    /// The grid edit, if any, is laid over the hair groups.
    pub fn hair_grid_points(&self) -> Vec<f32> {
        let inner = self.inner.borrow();
        inner.head().and_then(|h| h.detail.as_ref()).and_then(|d| d.beard.as_ref()).map_or(Vec::new(), |b| b.view_points(inner.hair_groups(), inner.hair_edit()))
    }

    /// The sliders of a hair group and the presets, as JSON: `{"sliders": [{id, label, part, min, max, default, step, unit, help,
    /// stubble_only, pull_only, area_only}, ...], "presets": [[id, label], ...]}`.
    pub fn hair_group_spec_json(&self) -> String {
        use head_paint::hair_groups::{GROUP_SLIDERS, PRESETS};
        let sliders: Vec<serde_json::Value> = GROUP_SLIDERS
            .iter()
            .map(|s| serde_json::json!({ "id": s.id, "label": s.label, "part": s.part, "min": s.min, "max": s.max, "default": s.default, "step": s.step, "unit": s.unit, "help": s.help, "stubble_only": s.stubble_only, "pull_only": s.pull_only, "area_only": s.area_only }))
            .collect();
        serde_json::json!({ "sliders": sliders, "presets": PRESETS }).to_string()
    }

    /// The facial hair's hair groups as JSON (`hair_groups::HairGroups`), the top one first.
    pub fn hair_groups_json(&self) -> String {
        self.inner.borrow().hair_groups().to_json()
    }

    /// Sets the facial hair's hair groups from JSON, or clears them for an empty text, and forgets the steps of undo. Returns false when
    /// the text could not be read.
    pub fn set_hair_groups_json(&self, json: &str) -> bool {
        let groups = if json.trim().is_empty() { Some(Default::default()) } else { HairGroups::from_json(json) };
        let ok = groups.is_some();
        if let Some(g) = groups {
            *self.inner.borrow_mut().hair_groups_mut() = g;
            self.history.borrow_mut().clear();
        }
        ok
    }

    /// Keeps the hair groups as they are, as one step of undo. The page calls it before each edit.
    pub fn hair_groups_checkpoint(&self) {
        let inner = self.inner.borrow();
        self.history.borrow_mut().checkpoint(inner.hair_groups());
    }

    /// Goes back one step of the hair groups' edits. Returns false when there is none.
    pub fn hair_groups_undo(&self) -> bool {
        self.history.borrow_mut().undo(self.inner.borrow_mut().hair_groups_mut())
    }

    /// Goes forward one step that was undone. Returns false when there is none.
    pub fn hair_groups_redo(&self) -> bool {
        self.history.borrow_mut().redo(self.inner.borrow_mut().hair_groups_mut())
    }

    /// Whether there is a step to undo and one to redo, as 1 or 0.
    pub fn hair_groups_history(&self) -> Vec<u8> {
        let h = self.history.borrow();
        vec![u8::from(h.can_undo()), u8::from(h.can_redo())]
    }

    fn with_groups<T>(&self, f: impl FnOnce(&mut HairGroups, usize) -> T) -> Option<T> {
        let mut inner = self.inner.borrow_mut();
        let cols = inner.head().and_then(|h| h.detail.as_ref()).and_then(|d| d.beard.as_ref()).map(|b| b.grid.cols)?;
        Some(f(inner.hair_groups_mut(), cols))
    }

    /// Runs an edit on hair group `group`, or on its flow line being shaped as a stroke of its own.
    fn with_group<T>(&self, group: usize, f: impl FnOnce(&mut HairGroup, usize) -> T) -> Option<T> {
        let line = self.flow_of(group);
        self.with_groups(|gs, cols| {
            let g = gs.groups.get_mut(group)?;
            match line {
                Some(n) => g.edit_flow(n, |s| f(s, cols)),
                None => Some(f(g, cols)),
            }
        })
        .flatten()
    }

    /// The flow line of group `group` being shaped, if any.
    fn flow_of(&self, group: usize) -> Option<usize> {
        self.flow.get().filter(|f| f.0 == group).map(|f| f.1)
    }

    /// Shapes flow line `line` of hair group `group` with the handles from now on, instead of the group's own points. A `line` of -1
    /// goes back to the group's own points.
    pub fn hair_flow_focus(&self, group: usize, line: i32) {
        self.flow.set(usize::try_from(line).ok().map(|n| (group, n)));
    }

    /// Adds an empty flow line to hair group `group` and shapes it from now on (`hair_flow_focus`). Its points are added by clicking on
    /// the head. Returns its number, or -1.
    pub fn hair_flow_add(&self, group: usize) -> i32 {
        let n = self.inner.borrow_mut().hair_groups_mut().groups.get_mut(group).map(HairGroup::add_flow);
        self.flow.set(n.map(|n| (group, n)));
        n.map_or(-1, |n| n as i32)
    }

    /// Removes flow line `line` of hair group `group`, and goes back to shaping the group's own points.
    pub fn hair_flow_remove(&self, group: usize, line: usize) {
        if let Some(g) = self.inner.borrow_mut().hair_groups_mut().groups.get_mut(group) {
            g.remove_flow(line);
        }
        self.flow.set(None);
    }

    /// The flow lines of hair group `group` on the head last set, each ending in an arrow head, for drawing them
    /// (`HairGroup::flow_on_skin`).
    pub fn hair_flow_lines(&self, group: usize) -> Vec<f32> {
        let inner = self.inner.borrow();
        let (Some(b), Some(g)) = (inner.head().and_then(|h| h.detail.as_ref()).and_then(|d| d.beard.as_ref()), inner.hair_groups().groups.get(group)) else { return Vec::new() };
        g.flow_on_skin(&b.grid)
    }

    /// Adds a preset of `hair_groups::PRESETS` at the top of the list of hair groups, laid on the paint grid of the head last set. A preset
    /// of several groups adds them all, in its order.
    /// Returns false when there is no such preset or no head.
    pub fn hair_group_add(&self, preset: &str) -> bool {
        self.with_groups(|gs, _| gs.add_preset(preset)).unwrap_or(false)
    }

    /// Adds an empty stroke, or an empty filled area with `area`, at the top of the list. Its points are added by clicking on the head.
    /// Returns its number, or -1 without a head.
    pub fn hair_group_add_empty(&self, area: bool) -> i32 {
        self.with_groups(|gs, _| gs.add_empty(area) as i32).unwrap_or(-1)
    }

    /// Adds an empty pull at the top of the list. Its points are added by clicking on the head. Returns its number, or -1 without a head.
    pub fn hair_group_add_pull(&self) -> i32 {
        self.with_groups(|gs, _| gs.add_pull() as i32).unwrap_or(-1)
    }

    /// Removes hair group `group`.
    pub fn hair_group_remove(&self, group: usize) {
        self.inner.borrow_mut().hair_groups_mut().remove(group);
    }

    /// Copies hair group `group` just above itself. Returns the copy's number, or -1.
    pub fn hair_group_duplicate(&self, group: usize) -> i32 {
        self.inner.borrow_mut().hair_groups_mut().duplicate(group).map_or(-1, |n| n as i32)
    }

    /// Moves hair group `group` one place up the list, toward the top, or down. Returns its new number.
    pub fn hair_group_shift(&self, group: usize, up: bool) -> u32 {
        self.inner.borrow_mut().hair_groups_mut().shift(group, up) as u32
    }

    /// Renames hair group `group`.
    pub fn hair_group_rename(&self, group: usize, name: &str) {
        if let Some(g) = self.inner.borrow_mut().hair_groups_mut().groups.get_mut(group) {
            g.name = name.trim().to_string();
        }
    }

    /// Sets a slider of hair group `group`, by its id in `hair_groups::GROUP_SLIDERS`, or one of its switches: "mirror", "hidden", or
    /// "area", which closes a stroke into a filled area (1) or opens a filled area into a stroke (0).
    pub fn hair_group_set(&self, group: usize, id: &str, value: f64) {
        let mut inner = self.inner.borrow_mut();
        let Some(g) = inner.hair_groups_mut().groups.get_mut(group) else { return };
        match id {
            "mirror" => g.mirror = value != 0.0,
            "hidden" => g.hidden = value != 0.0,
            "area" if value != 0.0 => {
                g.close();
            }
            "area" => {
                g.open(None);
            }
            _ => {
                if let Some(v) = g.value_mut(id) {
                    *v = value;
                }
            }
        }
    }

    /// The control points of hair group `group` on the head last set, for drawing them as handles (`HairGroup::handles`, nine numbers
    /// each), then, after a single NaN, its curve on the skin (`HairGroup::curve_on_skin`). While one of its flow lines is shaped, the
    /// same for the flow line.
    pub fn hair_group_view(&self, group: usize) -> Vec<f32> {
        let inner = self.inner.borrow();
        let (Some(b), Some(g)) = (inner.head().and_then(|h| h.detail.as_ref()).and_then(|d| d.beard.as_ref()), inner.hair_groups().groups.get(group)) else { return Vec::new() };
        let line = self.flow_of(group).and_then(|n| g.flow_stroke(n));
        let g = line.as_ref().unwrap_or(g);
        let mut out = g.handles(&b.grid);
        out.push(f32::NAN);
        out.extend(g.curve_on_skin(&b.grid));
        out
    }

    /// The edges of hair group `group`'s width on the head last set, for drawing them (`HairGroup::widths_on_skin`). A flow line has
    /// no width.
    pub fn hair_group_widths(&self, group: usize) -> Vec<f32> {
        if self.flow_of(group).is_some() {
            return Vec::new();
        }
        let inner = self.inner.borrow();
        let (Some(b), Some(g)) = (inner.head().and_then(|h| h.detail.as_ref()).and_then(|d| d.beard.as_ref()), inner.hair_groups().groups.get(group)) else { return Vec::new() };
        g.widths_on_skin(&b.grid)
    }

    /// The other groups that paint, for drawing them faintly while group `except` is shaped: the end points of their strokes, nine numbers
    /// each (place and normal in metres, the group's number, 0 for its first point or 1 for its last, 1 on the mirror image or 0), then
    /// after a single NaN their curves on the skin (`HairGroup::curve_on_skin`) one after the other.
    pub fn hair_group_others(&self, except: usize) -> Vec<f32> {
        let inner = self.inner.borrow();
        let Some(b) = inner.head().and_then(|h| h.detail.as_ref()).and_then(|d| d.beard.as_ref()) else { return Vec::new() };
        let others = || inner.hair_groups().groups.iter().enumerate().filter(|(n, g)| *n != except && !g.hidden);
        let mut out = Vec::new();
        for (n, g) in others() {
            for e in g.ends_on_skin(&b.grid).chunks(8) {
                out.extend_from_slice(&e[..6]);
                out.extend([n as f32, e[6], e[7]]);
            }
        }
        out.push(f32::NAN);
        for (_, g) in others() {
            out.extend(g.curve_on_skin(&b.grid));
        }
        out
    }

    /// The paint grid's point under a ray from `origin` along `dir`, three numbers each, in the head's frame in metres, as its column
    /// and row, or nothing when the ray meets no point of the grid (`FaceGrid::snap`).
    pub fn hair_grid_snap(&self, origin: &[f64], dir: &[f64]) -> Vec<f64> {
        let inner = self.inner.borrow();
        let Some(b) = inner.head().and_then(|h| h.detail.as_ref()).and_then(|d| d.beard.as_ref()) else { return Vec::new() };
        if origin.len() < 3 || dir.len() < 3 {
            return Vec::new();
        }
        b.grid.snap([origin[0], origin[1], origin[2]].map(|x| 1000.0 * x), [dir[0], dir[1], dir[2]]).map_or(Vec::new(), |(i, j)| vec![i as f64, j as f64])
    }

    /// Moves control point `point` of hair group `group` to a column and row of the paint grid. `on_mirror` is true when the point was
    /// picked on the group's mirror image.
    pub fn hair_group_move(&self, group: usize, point: usize, i: f64, j: f64, on_mirror: bool) {
        self.with_group(group, |g, cols| g.move_point(point, i, j, on_mirror, cols));
    }

    /// Sets the width of control point `point` of hair group `group`, in millimetres. With a point of -1, it scales every point's width
    /// so that the widest is `mm` (`HairGroup::set_width`).
    pub fn hair_group_width(&self, group: usize, point: i32, mm: f64) {
        self.with_group(group, |g, _| g.set_width(usize::try_from(point).ok(), mm));
    }

    /// Bends piece `segment` of hair group `group` so that its middle lies at a column and row of the paint grid.
    pub fn hair_group_bend(&self, group: usize, segment: usize, i: f64, j: f64, on_mirror: bool) {
        self.with_group(group, |g, cols| g.bend_segment(segment, i, j, on_mirror, cols));
    }

    /// Takes the bend out of piece `segment` of hair group `group`.
    pub fn hair_group_straighten(&self, group: usize, segment: usize) {
        self.with_group(group, |g, _| g.straighten_segment(segment));
    }

    /// Makes control point `point` of hair group `group` a corner, or a smooth point.
    pub fn hair_group_corner(&self, group: usize, point: usize, corner: bool) {
        self.with_group(group, |g, _| g.set_corner(point, corner));
    }

    /// Adds a control point to hair group `group` at a column and row of the paint grid, in the curve's order. Returns its number.
    pub fn hair_group_insert(&self, group: usize, i: f64, j: f64) -> i32 {
        self.with_group(group, |g, cols| g.insert_point(i, j, cols) as i32).unwrap_or(-1)
    }

    /// Adds a control point to hair group `group` after its last point, or before its first with `at_start`. Returns its number.
    pub fn hair_group_extend(&self, group: usize, i: f64, j: f64, at_start: bool) -> i32 {
        self.with_group(group, |g, cols| g.extend(i, j, at_start, cols) as i32).unwrap_or(-1)
    }

    /// Removes control point `point` of hair group `group`, unless it is the group's last, or one of a filled area's last three.
    pub fn hair_group_remove_point(&self, group: usize, point: usize) {
        self.with_group(group, |g, _| g.remove_point(point));
    }

    /// Splits hair group `group` at its control point `at`, or at its piece `at` with `segment`. A stroke becomes two strokes, the second
    /// just under the first in the list. A filled area opens into a stroke. Returns false when nothing changed, as at a stroke's end.
    pub fn hair_group_split(&self, group: usize, at: usize, segment: bool) -> bool {
        if self.flow_of(group).is_some() {
            return false;
        }
        self.inner.borrow_mut().hair_groups_mut().split(group, at, segment)
    }

    /// Joins stroke `b` onto stroke `a`, from `a`'s last point (or its first, with `a_start`) to `b`'s first point (or its last, with
    /// `b_end`). Joining a stroke's two ends closes it into a filled area. Returns the joined group's number, or -1.
    pub fn hair_group_join(&self, a: usize, a_start: bool, b: usize, b_end: bool) -> i32 {
        if self.flow_of(a).is_some() {
            return -1;
        }
        self.inner.borrow_mut().hair_groups_mut().join(a, a_start, b, b_end).map_or(-1, |n| n as i32)
    }

    /// The kind of hair group `group`: "stroke", "area" or "stubble", or nothing.
    pub fn hair_group_kind(&self, group: usize) -> String {
        self.inner.borrow().hair_groups().groups.get(group).map_or(String::new(), |g| {
            match g.kind {
                GroupKind::Stroke => "stroke",
                GroupKind::Area => "area",
                GroupKind::Stubble => "stubble",
            }
            .to_string()
        })
    }

    /// Prepares a size, a multiple of 64. A head must be set. Resolves when the size is ready to paint.
    pub fn prepare(&self, resolution: u32) -> js_sys::Promise {
        let inner = self.inner.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let started = inner.borrow_mut().begin_prepare(resolution);
            if let Some((gpu, buffer, size)) = started {
                let covered = gpu.read(&buffer, size).await;
                inner.borrow_mut().finish_prepare(resolution, &covered);
            }
            Ok(JsValue::UNDEFINED)
        })
    }

    /// Paints at a prepared size and submits the work to the GPU. `values` holds one number per control and `switches` one byte per layer
    /// (0 is off). It returns before the GPU has finished.
    pub fn paint(&self, resolution: u32, values: &[f64], switches: &[u8]) {
        let recipe = controls::recipe_from(values);
        let layers = controls::layers_from(&switches.iter().map(|&s| s != 0).collect::<Vec<_>>());
        self.inner.borrow_mut().paint(resolution, &recipe, &layers);
    }

    /// Resolves once the GPU has finished every paint submitted so far.
    pub fn done(&self) -> js_sys::Promise {
        let gpu = self.inner.borrow().gpu().clone();
        wasm_bindgen_futures::future_to_promise(async move {
            gpu.done().await;
            Ok(JsValue::UNDEFINED)
        })
    }

    /// Resolves to the maps of the last paint made at a size while the light from maps was on, as a `Uint8Array` laid out as `read`'s:
    /// per texel the normal's x and y along the texture's u and v, the gloss and the detail tile's mask.
    pub fn read_maps(&self, resolution: u32) -> js_sys::Promise {
        let (gpu, buffer) = {
            let p = self.inner.borrow();
            (p.gpu().clone(), p.maps_texels(resolution).clone())
        };
        let n = u64::from(resolution);
        wasm_bindgen_futures::future_to_promise(async move {
            let bytes = gpu.read(&buffer, 4 * n * n).await;
            Ok(js_sys::Uint8Array::from(bytes.as_slice()).into())
        })
    }

    /// Resolves to the detail tile at `size` texels per side (a multiple of 16), as a `Uint8Array` of RGBA bytes whose first row is v = 0.
    pub fn detail_tile(&self, size: u32) -> js_sys::Promise {
        let gpu = self.inner.borrow().gpu().clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let bytes = crate::tile::detail_tile(&gpu, size).await;
            Ok(js_sys::Uint8Array::from(bytes.as_slice()).into())
        })
    }

    /// Resolves to the last texture painted at a size, as a `Uint8Array` of RGBA bytes, row by row from the top.
    pub fn read(&self, resolution: u32) -> js_sys::Promise {
        let (gpu, buffer) = {
            let p = self.inner.borrow();
            (p.gpu().clone(), p.texels(resolution).clone())
        };
        let n = u64::from(resolution);
        wasm_bindgen_futures::future_to_promise(async move {
            let bytes = gpu.read(&buffer, 4 * n * n).await;
            Ok(js_sys::Uint8Array::from(bytes.as_slice()).into())
        })
    }
}
