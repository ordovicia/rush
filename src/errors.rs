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
        use self::Error::ExecuteError;

        match (self, other) {
            (&ExecuteError(_), &ExecuteError(_)) => true,
            (s, o) => s == o,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Error(nom::Err<u32>),
    Incomplete(nom::Needed),
}
