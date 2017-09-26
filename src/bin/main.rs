extern crate rush;
extern crate rustyline;

use std::process;

use rush::reader::Reader;
use rush::errors::{Result, Error};

fn main() {
    let mut reader = Reader::new();

    loop {
        match run(&mut reader) {
            Err(Error::Eof) |
            Err(Error::Interrupted) => break,
            Ok(status) => println!("Exit with {}", status),
            e => eprintln!("Error: {:?}", e),
        }
    }
}

fn run(reader: &mut Reader) -> Result<process::ExitStatus> {
    let job = reader.read_job()?;
    job.run()
}
