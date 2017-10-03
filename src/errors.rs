use std::{result, io};

use rustyline;
use nom;

pub(super) type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub(super) enum Error {
    // Read
    Read(rustyline::error::ReadlineError),
    Eof, // Ctrl-D
    Interrupted, // Ctrl-C

    // Parse
    Parse(nom::IError<u32>),

    // Execute
    NotBuiltin,
    BuiltinExec(String),
    IO(io::Error),
}

impl From<rustyline::error::ReadlineError> for Error {
    fn from(e: rustyline::error::ReadlineError) -> Self {
        use rustyline::error::ReadlineError::{Eof, Interrupted};

        match e {
            Eof => Error::Eof,
            Interrupted => Error::Interrupted,
            e => Error::Read(e),
        }
    }
}

impl From<nom::IError> for Error {
    fn from(e: nom::IError) -> Self {
        Error::Parse(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e)
    }
}
