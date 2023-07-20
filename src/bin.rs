use schred::*;
use std::path::Path;

fn main() {
    let s = Shredder::new(ShredOptions {
        ..ShredOptions::default()
    });
    s.shred(Path::new("test.bin")).unwrap();
}