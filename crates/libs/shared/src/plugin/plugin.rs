pub use super::error::Error;
use super::results::ListPackagesResult;
use libloading::{Library, Symbol};
use std::ffi::CStr;

pub struct Plugin {
    lib: Library,
}

impl Plugin {
    pub fn load(path: &str) -> Result<Self, Error> {
        if !path.contains(".") {
            Self::load(format!("libunipac_{}_plugin.so", path).as_str())
        } else {
            let lib = unsafe { Library::new(path) }.map_err(Error::OpenLibrary)?;
            Ok(Self { lib })
        }
    }

    pub fn list_packages(&self) -> Result<Vec<String>, Error> {
        let ffi_list_packages: Symbol<extern "C" fn() -> ListPackagesResult> =
            unsafe { self.lib.get(b"ffi_list_packages") }
                .map_err(|_| Error::SymbolNotFound("ffi_list_packages"))?;

        let result = (ffi_list_packages)();
        if !result.err.is_null() {
            let err = unsafe { CStr::from_ptr(result.err) }
                .to_string_lossy()
                .into_owned();
            return Err(Error::LibraryError(err).into());
        }
        let packages = unsafe {
            std::slice::from_raw_parts(result.data, result.len)
                .to_vec()
                .into_iter()
                .map(|p| CStr::from_ptr(p).to_string_lossy().into_owned())
                .collect()
        };
        Ok(packages)
    }
}
