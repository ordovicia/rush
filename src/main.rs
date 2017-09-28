extern crate rush;
extern crate rustyline;

use rush::Rush;

fn main() {
    let mut rush = Rush::new();
    rush.repl();
}
