use std::{fmt, process};

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
                Ok(status) => println!("Exit with {}", status),
                Err(Error::Eof) => {
                    println!("EOF");
                    break;
                }
                Err(Error::Interrupted) => {
                    println!("Interrupted");
                    break;
                }
                Err(Error::IO(err)) => Self::display_error(err),
                Err(Error::BuiltinExec(err)) => Self::display_error(err),
                Err(err) => eprintln!("rush: {:?}", err),
            }
        }
    }

    fn display_error<E: fmt::Display>(err: E) {
        eprintln!("rush: {}", err);
    }

    fn run(&mut self) -> Result<process::ExitStatus> {
        let job = self.reader.read_job()?;
        job.run().map_err(Error::from)
    }
}
