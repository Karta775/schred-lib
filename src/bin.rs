use schred::*;
use std::path::Path;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("You must provide a path to shred!");
    }

    let s = Shredder::new(ShredOptions {
        recursive: true,
        deallocate: true,
        ..ShredOptions::default()
    });


    s.shred(Path::new(&args[1])).unwrap();
}
