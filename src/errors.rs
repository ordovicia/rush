use std::{result, io};

use rustyline;
use nom;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ReadError(rustyline::error::ReadlineError),
    Eof, // Ctrl-D
    Interrupted, // Ctrl-C
    ParseError(nom::IError<u32>),
    ExecuteError(io::Error),
}

impl From<rustyline::error::ReadlineError> for Error {
    fn from(e: rustyline::error::ReadlineError) -> Self {
        use rustyline::error::ReadlineError::{Eof, Interrupted};

        match e {
            Eof => Error::Eof,
            Interrupted => Error::Interrupted,
            e => Error::ReadError(e),
        }
    }
}

impl From<nom::IError> for Error {
    fn from(e: nom::IError) -> Self {
        Error::ParseError(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::ExecuteError(e)
    }
}
