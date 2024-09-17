use abi_stable::library::LibraryError;

#[derive(Debug)]
pub enum Error {
    OpenLibrary(LibraryError),
    SymbolNotFound(&'static str),
}
