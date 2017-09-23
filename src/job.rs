use std::process;

use errors::*;
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

    pub fn spawn(&self) -> Result<()> {
        assert!(self.process_list.len() > 0);
        let proc0 = &self.process_list[0];

        assert!(proc0.argument_list.len() > 0);
        let mut child = match process::Command::new(&proc0.argument_list[0])
            .args(&proc0.argument_list[1..])
            .spawn() {
            Ok(child) => child,
            Err(_) => {
                return Err(Error::SpawnChild);
            }
        };

        match child.wait() {
            Ok(status) => println!("child exited with status {}", status),
            Err(_) => {
                return Err(Error::WaitChild);
            }
        }

        Ok(())
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
