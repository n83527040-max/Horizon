//! New CRAFTIX controls, developed independently of the existing touch stack.
//!
//! An experimental opt-in Android game path uses this core and its editor.
//! The default still uses the old controls until the replacement has been
//! verified in a real runtime and remaining compatibility work is complete.
pub mod editor;
pub mod input;
pub mod layout;
pub mod paint;
pub mod bridge;
pub mod migrate;
pub mod ui;

pub mod toggle_owners;
pub mod transfer;
pub mod png_icon;
pub mod text_guard;
