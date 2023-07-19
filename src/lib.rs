use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::{process, io}; 

const BLOCK_SIZE: usize = 16384;

#[derive(Default)]
pub struct ShredOptions {
    pub verbose: bool,
    pub deallocate: bool,
    pub recursive: bool,
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

    fn write(&self, message: &str) {
        println!("schred: {}", message);
    }

    fn log(&self, message: &str) {
        if self.options.verbose {
            self.write(message);
        }
    }

    fn shred_file(&self, path: &Path) {
        self.log(&format!("Starting shred of file: {}", path.to_str().unwrap()));
        let mut buffer = OpenOptions::new().write(true).open(path).unwrap();
        let mut pos: usize = 0;
        let zeros: [u8;BLOCK_SIZE] = [0;BLOCK_SIZE];
        let original_len: usize = buffer.metadata().unwrap().len() as usize;
        while pos < original_len {
            self.log(&format!("pos: {}", pos));
            let bytes_written = buffer.write(&zeros[0..(original_len - pos).min(BLOCK_SIZE)]).unwrap();
            pos += bytes_written;
        }
        buffer.flush().unwrap();
    }

    pub fn shred(&self, path: &Path) -> Result<(), ShredError> {
        if path.is_dir() && !self.options.recursive {
            return Err(ShredError::DirectoryWithoutRecursive);
        }
        if !path.exists() {
            return Err(ShredError::PathDoesntExist);
        }
        if path.is_file() {
            self.shred_file(path);
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
