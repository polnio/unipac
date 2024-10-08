use super::error::Error;
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
        let data = match (result.data, result.err) {
            (Some(data), None) => data,
            (None, Some(err)) => return Err(Error::LibraryError(err.into_string()).into()),
            _ => return Err(Error::BadResponse),
        };

        println!("{:?}", data);

        let packages = data
            .to_vec()
            .into_iter()
            .map(|p| p.into_string())
            .collect::<Vec<_>>();

        Ok(packages)
    }
}
