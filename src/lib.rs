#[macro_use]
extern crate nom;

pub mod job;
mod parser;

#[derive(Debug, PartialEq)]
pub enum Error {
    ParseError,
}

pub type Result<T> = std::result::Result<T, Error>;
