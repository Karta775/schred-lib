use schred::*;
use std::path::Path;

fn main() {
    let s = Shredder::new(ShredOptions {
        verbose: true,
        ..ShredOptions::default()
    });
    s.shred(Path::new("test.bin")).unwrap();
}