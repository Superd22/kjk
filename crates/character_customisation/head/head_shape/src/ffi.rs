//! A C interface to the smoothing and the neck's rim, for the analysis tools written in Python. They load the library with `ctypes`,
//! prepare the mesh once with `shape_smoother_new`, and smooth many heads with it.

use crate::neck::place_rim;
use crate::smooth::{Method, Smoother};

/// Prepares a mesh. `base` holds `vertex_count` points of three f64 in metres, `triangles` holds `triangle_count` triples of u32, and
/// `keep` holds one byte per vertex, not 0 for a vertex that must not move. `ramp` is in metres.
///
/// # Safety
/// The pointers must be valid for the counts given. The result must be freed with `shape_smoother_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shape_smoother_new(
    base: *const f64,
    vertex_count: usize,
    triangles: *const u32,
    triangle_count: usize,
    keep: *const u8,
    ramp: f64,
) -> *mut Smoother {
    // SAFETY: the caller promises that the arrays hold as many values as the counts say.
    let (b, t, k) = unsafe {
        (
            std::slice::from_raw_parts(base, vertex_count * 3),
            std::slice::from_raw_parts(triangles, triangle_count * 3),
            std::slice::from_raw_parts(keep, vertex_count),
        )
    };
    let keep: Vec<bool> = k.iter().map(|&x| x != 0).collect();
    Box::into_raw(Box::new(Smoother::new(b.as_chunks::<3>().0, t.as_chunks::<3>().0, &keep, ramp)))
}

/// Frees a prepared mesh.
///
/// # Safety
/// `smoother` must come from `shape_smoother_new` and must not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shape_smoother_free(smoother: *mut Smoother) {
    if !smoother.is_null() {
        // SAFETY: the caller promises that the pointer came from `shape_smoother_new` and is not used again.
        drop(unsafe { Box::from_raw(smoother) });
    }
}

/// The number of merged vertices.
///
/// # Safety
/// `smoother` must come from `shape_smoother_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shape_smoother_merged_count(smoother: *const Smoother) -> usize {
    // SAFETY: the caller promises a valid pointer.
    unsafe { &*smoother }.merged_count()
}

/// Writes each vertex's merged vertex to `groups` (one u32 per vertex) and each merged vertex's weight to `weights` (one f64 per merged
/// vertex). Either output may be null.
///
/// # Safety
/// `smoother` must come from `shape_smoother_new`, and each output that is not null must hold as many values as said.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shape_smoother_info(smoother: *const Smoother, groups: *mut u32, weights: *mut f64) {
    // SAFETY: the caller promises a valid pointer and outputs of the right size.
    unsafe {
        let s = &*smoother;
        if !groups.is_null() {
            std::ptr::copy_nonoverlapping(s.groups().as_ptr(), groups, s.groups().len());
        }
        if !weights.is_null() {
            std::ptr::copy_nonoverlapping(s.weights().as_ptr(), weights, s.weights().len());
        }
    }
}

/// Smooths `positions` (the vertex count given to `shape_smoother_new`, three f64 each) into `out`. `method` 0 is Taubin's method with
/// `rounds` rounds, and 1 is the implicit method with `strength`.
///
/// # Safety
/// `smoother` must come from `shape_smoother_new`, and `positions` and `out` must each hold three f64 per vertex.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shape_smooth(smoother: *const Smoother, positions: *const f64, out: *mut f64, method: u32, rounds: u32, strength: f64) {
    // SAFETY: the caller promises valid pointers of the right size.
    unsafe {
        let s = &*smoother;
        let n = s.groups().len();
        let p = std::slice::from_raw_parts(positions, n * 3).as_chunks::<3>().0;
        let m = if method == 0 { Method::Taubin { rounds } } else { Method::Implicit { strength } };
        let q = s.smooth(p, m);
        std::ptr::copy_nonoverlapping(q.as_ptr().cast::<f64>(), out, n * 3);
    }
}

/// Moves the neck's rim onto `ring` (`ring_count` points of three f64, a closed polyline) and carries the move to the rest of the head
/// (`neck::place_rim`). `base` and `positions` hold `vertex_count` points of three f64, `rim` holds `rim_count` u32 vertex indices in loop
/// order, `below` holds one byte per vertex, not 0 for a vertex under the rim, and `reach` is in metres. The result goes to `out`, three
/// f64 per vertex.
///
/// # Safety
/// The pointers must be valid for the counts given.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shape_place_rim(
    base: *const f64,
    positions: *const f64,
    vertex_count: usize,
    rim: *const u32,
    rim_count: usize,
    below: *const u8,
    ring: *const f64,
    ring_count: usize,
    reach: f64,
    out: *mut f64,
) {
    // SAFETY: the caller promises that the arrays hold as many values as the counts say.
    unsafe {
        let b = std::slice::from_raw_parts(base, vertex_count * 3).as_chunks::<3>().0;
        let p = std::slice::from_raw_parts(positions, vertex_count * 3).as_chunks::<3>().0;
        let r = std::slice::from_raw_parts(rim, rim_count);
        let under: Vec<bool> = std::slice::from_raw_parts(below, vertex_count).iter().map(|&x| x != 0).collect();
        let g = std::slice::from_raw_parts(ring, ring_count * 3).as_chunks::<3>().0;
        let q = place_rim(b, p, r, &under, g, reach);
        std::ptr::copy_nonoverlapping(q.as_ptr().cast::<f64>(), out, vertex_count * 3);
    }
}
