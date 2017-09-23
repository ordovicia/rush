use std::result;

#[derive(Debug, PartialEq)]
pub enum Error {
    ParseJob,
    SpawnChild,
    WaitChild,
}

pub type Result<T> = result::Result<T, Error>;
