use std::{process, fs};
use std::os::unix::io::{FromRawFd, IntoRawFd};

use errors::{Result, Error};

#[derive(Debug, PartialEq)]
pub struct Job {
    pub(crate) process_list: Vec<Process>,
    pub(crate) mode: JobMode,
}

impl Job {
    pub fn run(&self) -> Result<process::ExitStatus> {
        assert!(self.process_list.len() > 0);

        let mut child = Job::spawn_process(&self.process_list[0])?;
        child.wait().map_err(|e| Error::from(e))
    }

    fn spawn_process(process: &Process) -> Result<process::Child> {
        assert!(process.argument_list.len() > 0);

        let stdin = match process.redirect_in {
            Some(ref file_name) => {
                let file = fs::File::open(file_name)?;
                let file_fd = file.into_raw_fd();
                unsafe { process::Stdio::from_raw_fd(file_fd) }
            }
            None => process::Stdio::inherit(),
        };

        let stdout = match process.redirect_out {
            Some(OutputRedirection {
                     ref file_name,
                     ref mode,
                 }) => {
                let file = match *mode {
                    WriteMode::Truncate => fs::File::create(file_name)?,
                    WriteMode::Append => fs::OpenOptions::new().append(true).open(file_name)?,
                };
                let file_fd = file.into_raw_fd();
                unsafe { process::Stdio::from_raw_fd(file_fd) }
            }
            None => process::Stdio::inherit(),
        };

        process::Command::new(&process.argument_list[0])
            .args(&process.argument_list[1..])
            .stdin(stdin)
            .stdout(stdout)
            .spawn()
            .map_err(|e| Error::from(e))
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
    pub file_name: String,
    pub mode: WriteMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WriteMode {
    Truncate,
    Append,
}
