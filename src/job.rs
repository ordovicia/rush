use std::process as stdproc;
use std::fs;
use std::os::unix::io::{FromRawFd, IntoRawFd};

use errors::{Result, Error};

#[derive(Debug, PartialEq)]
pub struct Job {
    pub(crate) process_list: Vec<process::Process>,
    pub(crate) mode: JobMode,
}

impl Job {
    pub fn run(&self) -> Result<stdproc::ExitStatus> {
        assert!(self.process_list.len() > 0);

        let mut child = Job::spawn_process(&self.process_list[0])?;
        child.wait().map_err(|e| Error::from(e))
    }

    fn spawn_process(process: &process::Process) -> Result<stdproc::Child> {
        use self::process::*;
        use self::process::OutputRedirect::{Truncate, Append};

        assert!(process.argument_list.len() > 0);

        let stdin = match process.input {
            Some(Input::Redirect(ref file_name)) => {
                let file = fs::File::open(file_name)?;
                let file_fd = file.into_raw_fd();
                unsafe { stdproc::Stdio::from_raw_fd(file_fd) }
            }
            None => stdproc::Stdio::inherit(),
            _ => unimplemented!(),
        };

        let stdout = match process.output {
            Some(Output::Redirect(ref redir_out)) => {
                let file = match *redir_out {
                    Truncate(ref file_name) => fs::File::create(file_name),
                    Append(ref file_name) => fs::OpenOptions::new().append(true).open(file_name),
                }?;
                let file_fd = file.into_raw_fd();
                unsafe { stdproc::Stdio::from_raw_fd(file_fd) }
            }
            None => stdproc::Stdio::inherit(),
            _ => unimplemented!(),
        };

        stdproc::Command::new(&process.argument_list[0])
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
    #[allow(unused)]
    Suspended,
}

pub(crate) mod process {
    #[derive(Debug, PartialEq)]
    pub(crate) struct Process {
        pub argument_list: Vec<String>,
        pub input: Option<Input>,
        pub output: Option<Output>,
    }

    #[derive(Debug, PartialEq)]
    pub(crate) enum Input {
        Redirect(String),
        Pipe,
    }

    #[derive(Debug, PartialEq)]
    pub(crate) enum Output {
        Redirect(OutputRedirect),
        Pipe,
    }

    #[derive(Debug, PartialEq)]
    pub(crate) enum OutputRedirect {
        Truncate(String),
        Append(String),
    }
}
