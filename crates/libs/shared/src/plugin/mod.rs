pub mod results;

#[cfg(feature = "plugin-loader")]
mod error;
#[cfg(feature = "plugin-loader")]
mod plugin;

#[cfg(feature = "plugin-loader")]
pub use error::Error;
#[cfg(feature = "plugin-loader")]
pub use plugin::Plugin;

#[macro_export]
macro_rules! export_plugin {
    () => {
        #[no_mangle]
        extern "C" fn ffi_list_packages() -> unipac_shared::plugin::results::ListPackagesResult {
            unipac_shared::plugin::results::call_list_packages(list_packages)
        }
    };
}
