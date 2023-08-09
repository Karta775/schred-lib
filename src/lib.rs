use std::fs::{OpenOptions, self, File};
use std::io::{Write, Seek, SeekFrom};
use std::path::Path;
use rand_core::{RngCore, OsRng};

const BLOCK_SIZE: usize = 16384; // 16KiB

pub struct ShredOptions {
    pub verbose: bool,
    pub deallocate: bool,
    pub recursive: bool,
    pub zero_passes: u8,
    pub rand_passes: u8,
}

impl Default for ShredOptions {
    fn default() -> Self {
        ShredOptions { 
            verbose: cfg!(debug_assertions), // Enable verbose for debug builds
            deallocate: false,
            recursive: false,
            zero_passes: 1,
            rand_passes: 2,
        }
    }
}

#[derive(Debug)]
pub enum ShredError {
    DirectoryWithoutRecursive,
    PathDoesntExist,
}

pub struct Shredder {
    options: ShredOptions,
}

pub struct DataGenerator;
impl DataGenerator {
    fn zero() -> [u8; BLOCK_SIZE] {
        [0;BLOCK_SIZE]
    }
    
    fn random() -> [u8; BLOCK_SIZE] {
        let mut rand_data = [0u8; BLOCK_SIZE];
        OsRng.fill_bytes(&mut rand_data);
        rand_data
    }
}

impl Shredder {
    pub fn new(options: ShredOptions) -> Self {
        return Shredder { options }
    }

    /// Write to stdout.
    fn write(&self, message: &str) {
        println!("schred: {}", message);
    }

    /// Write to stderr.
    fn error(&self, message: &str) {
        eprintln!("schred: ERROR: {}", message);
    }

    /// Log some message to be shown in verbose mode.
    fn log(&self, message: &str) {
        if self.options.verbose {
            self.write(message);
        }
    }

    /// Perform n passes overwriting with data specified by a generator function.
    fn overwrite_file_with_data(&self, file: &mut File, data_fn: fn() -> [u8; BLOCK_SIZE]) {
        let original_len: usize = file.metadata().unwrap().len() as usize;

        file.seek(SeekFrom::Start(0)).expect("Failed to seek to start of file");
        let mut pos: usize = 0;
        while pos < original_len {
            let data = data_fn();
            let end = (original_len - pos).min(BLOCK_SIZE);
            let bytes_written = file.write(&data[0..end]).unwrap();
            pos += bytes_written;
        }
        file.sync_all().unwrap();
    }

    /// Shred a single file.
    fn shred_file(&self, path: &Path) {
        let filename = path.to_str().unwrap();
        self.log(&format!("Starting shred of file: {}", filename));

        // Overwrite file data
        let mut file = OpenOptions::new()
                                .write(true)
                                .open(path)
                                .unwrap();
        let mut passes = 0;
        let total_passes = self.options.rand_passes + self.options.zero_passes;

        // Overwrite with random data
        for _ in 0..self.options.rand_passes {
            passes += 1;
            self.log(&format!("Pass {}/{}: wiping with random data", passes, total_passes));
            self.overwrite_file_with_data(&mut file, DataGenerator::random);
        }

        // Overwrite with zeros 
        for _ in 0..self.options.zero_passes {
            passes += 1;
            self.log(&format!("Pass {}/{}: wiping with zeros", passes, total_passes));
            self.overwrite_file_with_data(&mut file, DataGenerator::zero);
        }

        // Deallocate
        if self.options.deallocate {
            match fs::remove_file(path) {
                Ok(_) => self.log(&format!("Removed {}", filename)),
                Err(e) => self.error(&format!("Failed to remove {}: {}", filename, e.kind()))
            }
        }
    }

    /// Shred the resources at `path` based on ShredOptions.
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
            // Recursively shred sub directories
            let sub_paths = fs::read_dir(path).expect("Error reading path");
            for sub_path in sub_paths {
                self.shred(&sub_path.expect("Error getting sub path").path()).unwrap();
            }
            // Deallocate(?) dir
            if self.options.deallocate {
                let filename = path.to_str().expect("Couldn't get filename");
                match fs::remove_dir(path) {
                    Ok(_) => self.log(&format!("Removed {}", filename)),
                    Err(e) => self.error(&format!("Failed to remove {}: {}", filename, e.kind()))
                }
            }
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

    /// Make a tmpfile with random data of size `bytes`.
    fn make_random_data_file(bytes: usize) -> Result<String, ()> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Error occurred")
            .as_secs();
        // TODO: Make this cross-platform
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

    /// Check if a file contains only zeros.
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
