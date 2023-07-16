
#[derive(Default)]
pub struct ShredOptions {
    verbose: bool,
    deallocate: bool,
}

pub struct Shredder {
    options: ShredOptions,
}

impl Shredder {
    pub fn new(options: ShredOptions) -> Self {
        return Shredder { options }
    }

    pub fn shred(&self, file: &str) {
        println!("Hi!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_txt() {
        let s = Shredder::new(ShredOptions::default());
        s.shred("test.txt");
        assert!(true); // FIXME: Actually test logic
    }
}
