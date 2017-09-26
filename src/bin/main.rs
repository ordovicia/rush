extern crate rush;

use std::io::{self, Write};

use rush::errors::*;
use rush::job::Job;

fn main() {
    loop {
        print!("rush $ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if let Err(e) = spawn(input.as_bytes()) {
            eprintln!("Error: {:?}", e);
        }
    }
}

fn spawn(input: &[u8]) -> Result<()> {
    Job::from_input(input)?.spawn()
}
