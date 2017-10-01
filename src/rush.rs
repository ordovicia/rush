use std::process;

use reader::Reader;
use errors::{Result, Error};

pub struct Rush {
    reader: Reader,
}

impl Default for Rush {
    fn default() -> Self {
        Self::new()
    }
}

impl Rush {
    pub fn new() -> Self {
        Self { reader: Reader::new() }
    }

    pub fn repl(&mut self) {
        loop {
            match self.run() {
                Err(Error::Eof) => {
                    println!("EOF");
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

    fn run(&mut self) -> Result<process::ExitStatus> {
        let job = self.reader.read_job()?;
        job.run().map_err(Error::from)
    }
}
