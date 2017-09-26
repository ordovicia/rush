use std::{result, io};

use rustyline;
use nom;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ParseError(nom::IError<u32>),
    ExecuteError(io::Error),
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
