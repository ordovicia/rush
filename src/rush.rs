//! The Rush shell.

use std::{fmt, process};

use reader::Reader;
use errors::{Error, Result};

pub struct Rush {
    reader: Reader,
}

impl Rush {
    pub fn new() -> Self {
        Self {
            reader: Reader::new(),
        }
    }

    /// Run read-eval-print loop.
    /// The loop is broken when reaches to EOF, or when interrupted.
    pub fn repl(&mut self) {
        loop {
            match self.run() {
                Ok(status) => println!("Exit with {}", status),
                Err(Error::Eof) | Err(Error::Interrupted) => break,
                Err(Error::IO(err)) => Self::display_error(err),
                Err(Error::Builtin(err)) => Self::display_error(err),
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
