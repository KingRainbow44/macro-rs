//! # macro-rs
//!
//! A lightweight macro library for recording & playing back keyboard and mouse events.
mod macros;
pub(crate) mod utils;

pub use macros::Macro;

pub use device_query::Keycode;