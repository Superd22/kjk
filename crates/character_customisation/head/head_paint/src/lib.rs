//! Procedural paint for customized heads. Each layer (skin tone, lips, brows,
//! makeup, facial hair, details) is computed from the head's surface and
//! baked into the head's texture. The plan is in
//! `docs/research/2026-11-19-head-roadmap.md`, part C.

pub mod closest;
pub mod colour;
pub mod ffi;
pub mod mesh;
pub mod pixel_map;
pub mod delta_e;
pub mod paint;
pub mod pigment;
pub mod shading;
pub mod sheen;
pub mod surface;
pub mod noise;
pub mod layout_points;
pub mod wrinkles;
pub mod pores;
pub mod controls;
pub mod live;
#[cfg(target_arch = "wasm32")]
pub mod web;
pub mod folds;
pub mod lines;
pub mod eyes;
pub mod eyeball;
pub mod lips;
pub mod brows;
pub mod maps;
pub mod skin;
pub mod hair_grid;
pub mod face_grid;
pub mod hair_groups;
pub mod facial_hair;
