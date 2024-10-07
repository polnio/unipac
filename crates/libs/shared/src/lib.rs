#![feature(vec_into_raw_parts)]
pub mod plugin;

#[cfg(feature = "plugin-loader")]
pub use plugin::Plugin;

#[cfg(feature = "headers")]
pub fn generate_headers() -> ::std::io::Result<()> {
    ::safer_ffi::headers::builder()
        .to_file("unipac-shared.h")?
        .generate()
}
