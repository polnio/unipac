#[derive(Debug)]
pub enum Error {
    OpenLibrary(libloading::Error),
    SymbolNotFound(&'static str),
    LibraryError(String),
}
