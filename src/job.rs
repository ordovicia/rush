use std::fmt;
use super::Result;
use parser;

#[derive(Debug, PartialEq)]
pub struct Job<'a> {
    pub(crate) process_list: Vec<Process<'a>>,
    pub(crate) mode: JobMode,
}

impl<'a> Job<'a> {
    pub fn from_input(input: &'a [u8]) -> Result<Self> {
        parser::parse_job(input)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum JobMode {
    ForeGround,
    BackGround,
    Suspended,
}

#[derive(PartialEq)]
pub(crate) struct Process<'a> {
    pub argument_list: Vec<&'a [u8]>,
    pub redirect_in: Option<&'a [u8]>,
    pub redirect_out: Option<OutputRedirection<'a>>,
}

impl<'a> fmt::Debug for Process<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Process {{ argument_list: [").unwrap();

        assert!(self.argument_list.len() > 0);
        write!(f, "\"{}\"", String::from_utf8_lossy(self.argument_list[0])).unwrap();
        for arg in &self.argument_list[1..] {
            write!(f, ", \"{}\"", String::from_utf8_lossy(arg)).unwrap();
        }

        write!(
            f,
            "] redirect_in: {:?}, redirect_out: {:?}",
            self.redirect_in.map(String::from_utf8_lossy),
            self.redirect_out
        )
    }
}

#[derive(PartialEq)]
pub(crate) struct OutputRedirection<'a> {
    pub filename: &'a [u8],
    pub mode: WriteMode,
}

impl<'a> fmt::Debug for OutputRedirection<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "OutputRedirection {{ filename: \"{}\", mode: {:?} }}",
            String::from_utf8_lossy(self.filename),
            self.mode
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WriteMode {
    Truncate,
    Append,
}
