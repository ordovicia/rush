use std::{result, io};

use nom;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ParseError(ParseError),
    ExecuteError(io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Error::ExecuteError(_), &Error::ExecuteError(_)) => true,
            (s, o) => s == o,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::ExecuteError(e)
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Error(nom::Err<u32>),
    Incomplete(nom::Needed),
}
