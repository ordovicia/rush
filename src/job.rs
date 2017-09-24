use std::process;

use errors::*;
use parser;

#[derive(Debug, PartialEq)]
pub struct Job {
    pub(crate) process_list: Vec<Process>,
    pub(crate) mode: JobMode,
}

impl Job {
    pub fn from_input(input: &[u8]) -> Result<(&[u8], Self)> {
        parser::parse_job(input)
    }

    pub fn spawn(&self) -> Result<()> {
        assert!(self.process_list.len() > 0);

        let mut child = Job::spawn_process(&self.process_list[0])?;
        let status = child.wait().map_err(|e| Error::ExecuteError(e))?;
        println!("child exited with status {}", status);

        Ok(())
    }

    fn spawn_process(process: &Process) -> Result<process::Child> {
        assert!(process.argument_list.len() > 0);

        process::Command::new(&process.argument_list[0])
            .args(&process.argument_list[1..])
            .spawn()
            .map_err(|e| Error::ExecuteError(e))
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
