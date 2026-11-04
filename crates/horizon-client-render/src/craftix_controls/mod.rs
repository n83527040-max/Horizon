//! CRAFTIX controls: the replacement touch/HUD system.
//!
//! `layout` is the versioned data model, `input` the per-pointer router,
//! `paint` the round glass/neon HUD, `editor` the transactional named layouts
//! and `ui` the on-screen editor. `crate::touch` hosts them: it owns the live
//! instance, applies engine semantics and persists `craftix-controls.json`.
//! The old sprite-based touch stack was removed from the active path.
pub mod editor;
pub mod input;
pub mod layout;
pub mod paint;
pub mod ui;
