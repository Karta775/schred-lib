use std::path::Path;
use std::{process, io}; 

#[derive(Default)]
pub struct ShredOptions {
    verbose: bool,
    deallocate: bool,
    recursive: bool,
}

#[derive(Debug)]
pub enum ShredError {
    DirectoryWithoutRecursive,
    PathDoesntExist,
}

pub struct Shredder {
    options: ShredOptions,
}

impl Shredder {
    pub fn new(options: ShredOptions) -> Self {
        return Shredder { options }
    }

    pub fn shred(&self, path: &Path) -> Result<(), ShredError> {
        if path.is_dir() && !self.options.recursive {
            return Err(ShredError::DirectoryWithoutRecursive);
        }
        if !path.exists() {
            return Err(ShredError::PathDoesntExist);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_without_recursive() {
        let s = Shredder::new(ShredOptions::default());
        assert!(s.shred(Path::new("./test/")).is_err());
    }
    #[test]
    fn path_doesnt_exist() {
        let s = Shredder::new(ShredOptions::default());
        assert!(s.shred(Path::new("./fake_path/")).is_err());
    }
}
