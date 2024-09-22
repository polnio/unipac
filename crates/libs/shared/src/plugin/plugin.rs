pub use super::error::Error;
use super::results::ListPackagesResult;
use libloading::{Library, Symbol};

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

        let result = ffi_list_packages();
        if let Some(err) = result.err {
            return Err(Error::LibraryError(err.into()).into());
        }
        let packages = result
            .data
            .unwrap()
            .into_iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>();
        Ok(packages)
    }
}
