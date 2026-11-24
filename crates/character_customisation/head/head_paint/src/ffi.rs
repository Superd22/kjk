//! A C interface to the closest point search, for the analysis tools written in Python. They load the library with `ctypes`, build a hierarchy
//! once per mesh with `closest_new`, and ask it for many points at once.

use crate::closest::Bvh;

/// Builds a hierarchy over a mesh. `vertices` holds `vertex_count` points of three f64, and `triangles` holds `triangle_count` triples of u32.
///
/// # Safety
/// Both pointers must be valid for the counts given. The result must be freed with `closest_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn closest_new(vertices: *const f64, vertex_count: usize, triangles: *const u32, triangle_count: usize) -> *mut Bvh {
    // SAFETY: the caller promises that both arrays hold as many values as the counts say.
    let (v, t) = unsafe { (std::slice::from_raw_parts(vertices, vertex_count * 3), std::slice::from_raw_parts(triangles, triangle_count * 3)) };
    let vertices = v.as_chunks::<3>().0.to_vec();
    let triangles = t.as_chunks::<3>().0.to_vec();
    Box::into_raw(Box::new(Bvh::new(vertices, triangles)))
}

/// Frees a hierarchy.
///
/// # Safety
/// `bvh` must come from `closest_new` and must not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn closest_free(bvh: *mut Bvh) {
    if !bvh.is_null() {
        // SAFETY: the caller promises that the pointer came from `closest_new` and is not used again.
        drop(unsafe { Box::from_raw(bvh) });
    }
}

/// Finds the closest point of the mesh to each of `count` query points. For each point it writes its distance to `distance`, the triangle
/// to `triangle`, the three barycentric weights to `barycentric` and the closest point to `point`. Any of the four outputs may be null.
/// A mesh with no triangle gives NaN distances.
///
/// # Safety
/// `bvh` must come from `closest_new`. `queries` holds `count` points of three f64. Each output that is not null must hold `count` values
/// (3 times as many for `barycentric` and `point`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn closest_query(
    bvh: *const Bvh,
    queries: *const f64,
    count: usize,
    distance: *mut f64,
    triangle: *mut u32,
    barycentric: *mut f64,
    point: *mut f64,
) {
    // SAFETY: the caller promises that the pointers are valid for the counts given.
    let (bvh, queries) = unsafe { (&*bvh, std::slice::from_raw_parts(queries, count * 3)) };
    let points: Vec<[f64; 3]> = queries.as_chunks::<3>().0.to_vec();
    for (i, c) in bvh.closest_all(&points).into_iter().enumerate() {
        // SAFETY: each output holds `count` values, or three times as many, as promised.
        unsafe {
            if !distance.is_null() {
                *distance.add(i) = c.map_or(f64::NAN, |c| c.distance);
            }
            if !triangle.is_null() {
                *triangle.add(i) = c.map_or(u32::MAX, |c| c.triangle);
            }
            for k in 0..3 {
                if !barycentric.is_null() {
                    *barycentric.add(i * 3 + k) = c.map_or(f64::NAN, |c| c.barycentric[k]);
                }
                if !point.is_null() {
                    *point.add(i * 3 + k) = c.map_or(f64::NAN, |c| c.point[k]);
                }
            }
        }
    }
}
