#![feature(vec_into_raw_parts)]
pub mod plugin;

#[cfg(feature = "plugin-loader")]
pub use plugin::Plugin;
