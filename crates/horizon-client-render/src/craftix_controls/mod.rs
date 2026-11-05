//! New CRAFTIX controls, developed independently of the existing touch stack.
//!
//! Not wired into the game yet: keeping the current controls live until this
//! replacement has editor, engine integration and device verification avoids
//! losing input or a player's saved layouts during migration.
pub mod editor;
pub mod input;
pub mod layout;
pub mod paint;
pub mod bridge;
