use schred::*;
use std::path::Path;

fn main() {
    println!("Hello, main");
    let s = Shredder::new(ShredOptions::default());
    s.shred(Path::new("target")).unwrap();
}