use std::io;
use std::path::PathBuf;

/// A lazy supplier of a value, typically bytes read from a resource.
///
/// Corresponds to Java's `IoSupplier<T>` — but as a concrete enum rather
/// than a `@FunctionalInterface`, because all real-world sources of pack
/// resources are either file-backed or in-memory.
///
/// # Design note
/// The set of supplier kinds is intentionally closed (enum, not trait).
/// Any code that needs to supply values from an unpredictable source
/// must materialise those values before wrapping them in `InMemory`.
/// This eliminates all `dyn`/`Box`/`Arc` overhead and keeps the type
/// `Clone`-able and `const`-constructible.
#[derive(Clone, Debug)]
pub enum IoSupplier<T> {
    File(PathBuf),
    InMemory(T),
}

impl IoSupplier<Vec<u8>> {
    /// Creates a supplier that reads the entire file at `path` into
    /// memory on each call to [`get`](Self::get).
    pub fn from_path(path: PathBuf) -> Self {
        Self::File(path)
    }

    /// Lazily reads the value (disk I/O deferred until this call).
    pub fn get(&self) -> io::Result<Vec<u8>> {
        match self {
            Self::File(path) => std::fs::read(path),
            Self::InMemory(data) => Ok(data.clone()),
        }
    }
}
