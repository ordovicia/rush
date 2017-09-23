extern crate rush;

use std::io::{self, Write};
use rush::job::Job;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        println!("{:?}", Job::from_input(buf.as_bytes()));
    }
}
