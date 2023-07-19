use std::fs::{OpenOptions, self};
use std::io::Write;
use std::path::Path;

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
        let filename = path.to_str().unwrap();
        self.log(&format!("Starting shred of file: {}", filename));
        let mut buffer = OpenOptions::new().write(true).open(path).unwrap();
        let mut pos: usize = 0;
        let zeros: [u8;BLOCK_SIZE] = [0;BLOCK_SIZE];
        let original_len: usize = buffer.metadata().unwrap().len() as usize;

        // Overwrite file data
        while pos < original_len {
            self.log(&format!("pos: {}", pos));
            let bytes_written = buffer.write(&zeros[0..(original_len - pos).min(BLOCK_SIZE)]).unwrap();
            pos += bytes_written;
        }
        buffer.flush().unwrap();

        // Deallocate
        if self.options.deallocate {
            match fs::remove_file(path) {
                Ok(_) => self.log(&format!("Removed {}", filename)),
                Err(e) => self.write(&format!("Failed to remove {}: {}", filename, e.kind())) // TOOD: stderr
            }
        }
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
        if path.is_dir() {
            // Traverse directory structure and shred_file all
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::io::{BufWriter, Read};
    use rand::Rng;
    use std::path::Path;
    use std::fs::File;
    use serial_test::serial;

    fn make_random_data_file(bytes: usize) -> Result<String, ()> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Error occurred")
            .as_secs();
        let path = &format!("/tmp/schred-{}", since_the_epoch);
        let file = File::create(path).unwrap();
        let mut writer = BufWriter::new(file);
        
        let mut rng = rand::thread_rng();
        let mut buffer = [0; 1024];
        let mut remaining_size = bytes;
        
        while remaining_size > 0 {
            let to_write = std::cmp::min(remaining_size, buffer.len());
            let buffer=  &mut buffer[..to_write];
            rng.fill(buffer);
            writer.write(buffer).unwrap();
            
            remaining_size -= to_write;
        }
        Ok(path.to_owned())
    }

    fn is_zeroed(path: &Path) -> bool {
        let mut buffer = File::open(path).unwrap();
        let mut vec: Vec<u8> = Vec::new();
        let _ = buffer.read_to_end(&mut vec);
        return vec.iter().all(|&x| x == 0);
    }

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
    #[test]
    #[serial]
    fn shred_32kib_file() {
        let s = Shredder::new(ShredOptions {
            verbose: true,
            ..Default::default()
        });
        let path = make_random_data_file(32768).unwrap();
        let path = Path::new(&path);
        s.shred(path).unwrap();
        assert!(is_zeroed(path));
    }
    #[test]
    #[serial]
    fn shred_43001_byte_file() {
        let s = Shredder::new(ShredOptions {
            verbose: true,
            ..Default::default()
        });
        let path = make_random_data_file(43001).unwrap();
        let path = Path::new(&path);
        s.shred(path).unwrap();
        assert!(is_zeroed(path));
    }
    #[test]
    #[serial]
    fn deallocate_after_shred() {
        let s = Shredder::new(ShredOptions {
            verbose: true,
            deallocate: true,
            ..Default::default()
        });
        let path = make_random_data_file(32768).unwrap();
        let path = Path::new(&path);
        s.shred(path).unwrap();
        assert!(!path.exists());
    }
}
