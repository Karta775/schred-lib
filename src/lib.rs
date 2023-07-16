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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_txt() {
        let s = Shredder::new(ShredOptions::default());
        s.shred(Path::new("test.txt"));
        assert!(true); // FIXME: Actually test logic
    }
}
