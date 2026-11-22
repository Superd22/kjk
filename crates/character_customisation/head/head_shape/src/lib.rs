//! The last steps of building a customized head's shape. They run after
//! FLAME's shapes and our own controls have moved the vertices, every time a
//! head is built, so an edited head gets them too.
//!
//! `smooth` removes the fine ripples that a head fitted to KOTOR's flat
//! triangles carries, and the base head's own fine relief, while the lids,
//! the eyeballs, the mouth's inside, the nostrils and the ears stay as they
//! are. The study behind it is in
//! `docs/research/2026-11-19-head-roadmap.md`.
//!
//! `neck` puts the open edge at the bottom of the neck on the ring where
//! KOTOR's player heads end, so the head joins KOTOR's bodies. It runs
//! before `smooth`, which then keeps that edge and everything below it still.

pub mod ffi;
pub mod neck;
pub mod smooth;
