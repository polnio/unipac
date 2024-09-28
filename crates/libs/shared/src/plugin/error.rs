#[derive(Debug)]
pub enum Error {
    OpenLibrary(libloading::Error),
    SymbolNotFound(&'static str),
    BadResponse,
    LibraryError(String),
}
