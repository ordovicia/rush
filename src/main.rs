extern crate rush;
extern crate rustyline;

use std::process;
use std::os::unix::process::ExitStatusExt;

use rush::reader::Reader;
use rush::errors::{Result, Error};

fn main() {
    let mut reader = Reader::new();

    loop {
        match run(&mut reader) {
            Err(Error::Eof) => {
                println!("EOE");
                break;
            }
            Err(Error::Interrupted) => {
                println!("Interrupted");
                break;
            }
            Ok(status) => println!("Exit with {}", status),
            err => eprintln!("Error: {:?}", err),
        }
    }
}

fn run(reader: &mut Reader) -> Result<process::ExitStatus> {
    let job = reader.read_job()?;
    // job.run()
    println!("{}", job);
    Ok(process::ExitStatus::from_raw(0))
}
