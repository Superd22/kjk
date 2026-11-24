//! The eyes of a customized head.
//!
//! The upper lid (`piece`) is one rigid generated piece per eye. The skin round the eye stays as the default head has it, KOTOR's lid
//! bone turns the piece for the blink, and a lash strip runs along its edge. `piece_check` checks the piece over the whole blink.
//! `caruncle` makes the pink wedge that fills the inner corner between the lids and the eyeball. `web` is the browser build.
//!
//! The earlier eye shape versions (Eyes v3 to Eyes v6, with their corner, iris, socket, lid strip and area code) were removed on
//! 2026-11-20. They are in the backup `likeness-fit-code-before-cleanup-2026-11-20-1823.tgz`.

pub mod caruncle;
pub mod piece;
pub mod piece_check;
#[cfg(target_arch = "wasm32")]
pub mod web;
