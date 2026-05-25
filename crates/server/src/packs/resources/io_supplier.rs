use std::fs::File;
use std::io;
use std::path::PathBuf;

pub trait IoSupplier {
    type Output;
    fn get(&self) -> io::Result<Self::Output>;
}

impl<F, T> IoSupplier for F
where
    F: Fn() -> io::Result<T>,
{
    type Output = T;
    fn get(&self) -> io::Result<T> {
        self()
    }
}

pub fn create(path: PathBuf) -> impl IoSupplier<Output = File> {
    move || File::open(&path)
}
