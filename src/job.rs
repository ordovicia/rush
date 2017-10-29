//! Job and process structs.

use std::fmt;
use std::process as stdproc;

use errors::{Error, Result};

#[derive(Debug, PartialEq)]
pub(super) struct Job {
    process_list: process::Process,
    mode: JobMode,
}

#[derive(Debug, PartialEq)]
pub(super) enum JobMode {
    ForeGround,
    BackGround,
    #[allow(unused)] Suspended,
}

impl Job {
    pub(super) fn new(process_list: process::Process, mode: JobMode) -> Self {
        Self { process_list, mode }
    }

    pub(super) fn run(&self) -> Result<stdproc::ExitStatus> {
        let mut child = self.process_list.spawn()?;
        child.wait()
    }
}

impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.process_list,
            match self.mode {
                JobMode::BackGround => "&",
                _ => "",
            }
        )
    }
}

pub(super) mod process {
    use std::fs;
    use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
    use std::os::unix::process::ExitStatusExt;

    use builtin;
    use super::*;

    #[derive(Debug, PartialEq)]
    pub(crate) struct Process {
        argument_list: Vec<String>,
        input: Input,
        output: Output,
    }

    #[derive(Debug)]
    enum Child {
        External(stdproc::Child),
        Builtin,
    }

    #[derive(Debug)]
    pub(super) struct ChildList {
        head: Child,
        piped: Option<Box<ChildList>>,
    }

    impl ChildList {
        pub(super) fn wait(&mut self) -> Result<stdproc::ExitStatus> {
            if let Some(ref mut piped) = self.piped {
                piped.wait()?;
            }

            match self.head {
                Child::External(ref mut child) => child.wait().map_err(Error::from),
                Child::Builtin => Ok(stdproc::ExitStatus::from_raw(0)),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub(crate) enum Input {
        Inherit,
        Redirect(String),
        Pipe,
    }

    #[derive(Debug, PartialEq)]
    pub(crate) enum Output {
        Inherit,
        Redirect(OutputRedirect),
        Pipe(Box<Process>),
    }

    #[derive(Debug, PartialEq)]
    pub(crate) enum OutputRedirect {
        Truncate(String),
        Append(String),
    }

    impl Process {
        pub(crate) fn new(argument_list: Vec<String>, input: Input, output: Output) -> Self {
            Self {
                argument_list,
                input,
                output,
            }
        }

        pub(super) fn spawn(&self) -> Result<ChildList> {
            let stdin = match self.input {
                Input::Inherit => stdproc::Stdio::inherit(),
                Input::Redirect(ref file_name) => {
                    let file = fs::File::open(file_name)?;
                    let file_fd = file.into_raw_fd();
                    unsafe { stdproc::Stdio::from_raw_fd(file_fd) }
                }
                Input::Pipe => unreachable!(),
            };

            self.spawn_rec(stdin)
        }

        fn spawn_rec(&self, stdin: stdproc::Stdio) -> Result<ChildList> {
            use self::OutputRedirect::{Append, Truncate};

            assert!(!self.argument_list.is_empty());

            let (head, piped) = match self.output {
                Output::Inherit => {
                    let head = self.spawn_one(stdin, stdproc::Stdio::inherit())?;
                    (head, None)
                }
                Output::Redirect(ref redir_out) => {
                    let file = match *redir_out {
                        Truncate(ref file_name) => fs::File::create(file_name),
                        Append(ref file_name) => {
                            fs::OpenOptions::new().append(true).open(file_name)
                        }
                    }?;
                    let file = file.into_raw_fd();
                    let file = unsafe { stdproc::Stdio::from_raw_fd(file) };

                    let head = self.spawn_one(stdin, file)?;
                    (head, None)
                }
                Output::Pipe(ref piped) => {
                    let head = self.spawn_one(stdin, stdproc::Stdio::piped())?;

                    let stdin = match head {
                        Child::External(ref head) => head.stdout.as_ref().unwrap().as_raw_fd(),
                        Child::Builtin => {
                            return Err(Error::Builtin(
                                String::from("Could not make pipe to builtin commands"),
                            ));
                        }
                    };
                    let stdin = unsafe { stdproc::Stdio::from_raw_fd(stdin) };

                    let piped = piped.spawn_rec(stdin)?;
                    (head, Some(Box::new(piped)))
                }
            };

            Ok(ChildList { head, piped })
        }

        fn spawn_one(&self, stdin: stdproc::Stdio, stdout: stdproc::Stdio) -> Result<Child> {
            match builtin::exec(&self.argument_list) {
                Some(Ok(())) => Ok(Child::Builtin),
                Some(Err(e)) => Err(e),
                None => stdproc::Command::new(&self.argument_list[0])
                    .args(&self.argument_list[1..])
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn()
                    .map(Child::External)
                    .map_err(Error::from),
            }
        }
    }

    impl fmt::Display for Process {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            use self::OutputRedirect::{Append, Truncate};

            for arg in &self.argument_list {
                write!(f, "{} ", arg).unwrap();
            }
            if let Input::Redirect(ref file_name) = self.input {
                write!(f, "< {} ", file_name).unwrap()
            }
            match self.output {
                Output::Redirect(Truncate(ref file_name)) => write!(f, "> {} ", file_name).unwrap(),
                Output::Redirect(Append(ref file_name)) => write!(f, ">> {} ", file_name).unwrap(),
                Output::Pipe(ref p) => write!(f, "| {}", p).unwrap(),
                Output::Inherit => {}
            }

            Ok(())
        }
    }
}
