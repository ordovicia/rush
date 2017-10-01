use std::{result, io};

use rustyline;
use nom;

pub(crate) type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub(crate) enum Error {
    Read(rustyline::error::ReadlineError),
    Eof, // Ctrl-D
    Interrupted, // Ctrl-C
    Parse(nom::IError<u32>),
    Exectute(io::Error),
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
        Error::Exectute(e)
    }
}
