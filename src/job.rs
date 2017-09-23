use super::Result;
use parser;

#[derive(Debug, PartialEq)]
pub struct Job {
    pub(crate) process_list: Vec<Process>,
    pub(crate) mode: JobMode,
}

impl Job {
    pub fn from_input(input: &[u8]) -> Result<Self> {
        parser::parse_job(input)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum JobMode {
    ForeGround,
    BackGround,
    Suspended,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Process {
    pub argument_list: Vec<String>,
    pub redirect_in: Option<String>,
    pub redirect_out: Option<OutputRedirection>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct OutputRedirection {
    pub filename: String,
    pub mode: WriteMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WriteMode {
    Truncate,
    Append,
}
