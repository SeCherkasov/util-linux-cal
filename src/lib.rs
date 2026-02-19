//! Calendar display utility with support for multiple calendar systems.
//!
//! Features:
//! - Gregorian and Julian calendar support
//! - Customizable week start (Monday/Sunday)
//! - Week numbers and Julian day display
//! - Plugin system for holiday highlighting

pub mod args;
pub mod calendar;
pub mod formatter;
pub mod types;

#[cfg(feature = "plugins")]
pub mod plugin_api;
