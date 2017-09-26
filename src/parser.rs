//! # Syntax of job (white space skipped)
//!
//! ```ignore
//! proc     := token+
//!
//! redir_in     := "<" token
//! redir_trunc  := ">" token
//! redir_append := ">>" token
//! redir_out    := redir_trunc
//!               | redir_append
//!
//! proc_list_1 := proc
//!              | proc redir_in redir_out?
//!              | proc redir_out redir_in?
//!
//! proc_in     := proc redir_in?
//! proc_out    := proc redir_out?
//! proc_pipe   := proc "|"
//! proc_list_2 := proc_in "|" proc_pipe* proc_out
//!
//! proc_list   := proc_list_1
//!              | proc_list_2
//!
//! end_job     := eof | ";" | "\n" | "\r"
//! job         := proc_list "&"? end_job
//! ```

use std::result;
use std::str::{self, FromStr};

use nom::{self, multispace};

use job::{Job, JobMode};
use job::process::{self, Process};

pub(crate) fn parse_job(input: &[u8]) -> result::Result<Job, nom::IError<u32>> {
    job(input).to_full_result()
}

named!(job<Job>,
       do_parse!(
           process_list: process_list >>
           bg: opt!(complete!(background)) >>
           opt!(complete!(multispace)) >>
           end_of_job >>
           (Job {
               process_list,
               mode: if bg.is_some() {
                        JobMode::BackGround
                     } else {
                         JobMode::ForeGround
                     }
            })
       ));

named!(process_list<Vec<Process> >,
       alt!(complete!(process_2many) | process_1only));

named!(process_2many<Vec<Process> >, // proc (< file) | proc | ... | proc (> file)
       do_parse!(
           proc_first: process_in >>
           pipe >>
           proc_pipe: many0!(process_cmd_pipe) >>
           proc_last: process_out >>
           (concat_processes(proc_first, proc_pipe, proc_last))
       ));

named!(process_in<Process>, // [proc (< file)] | ..
       do_parse!(
           argument_list: command >>
           input: opt!(complete!(redirect_in)) >>
           (Process {
               argument_list,
               input,
               output: Some(process::Output::Pipe),
           })
       ));
named!(process_cmd_pipe<Process>, // .. | [proc |] ..
       do_parse!(
           argument_list: command >>
           pipe >>
           (Process {
               argument_list,
               input: Some(process::Input::Pipe),
               output: Some(process::Output::Pipe),
           })
       ));
named!(process_out<Process>, // .. | [proc (> file)]
       do_parse!(
           argument_list: command >>
           output: opt!(complete!(redirect_out)) >>
           (Process {
               argument_list,
               input: Some(process::Input::Pipe),
               output,
           })
       ));

fn concat_processes(
    proc_first: Process,
    proc_pipe: Vec<Process>,
    proc_last: Process,
) -> Vec<Process> {
    let mut concated = vec![proc_first];
    concated.extend(proc_pipe.into_iter());
    concated.push(proc_last);
    concated
}

named!(process_1only<Vec<Process> >,
       map!(
           process_1only_elem,
           |p| vec![p]
       ));
named!(process_1only_elem<Process>,
       alt!(complete!(process_in_out) | complete!(process_out_in) | process_cmd));
named!(process_in_out<Process>, // proc < file0 (> file1)
       do_parse!(
           argument_list: command >>
           input: redirect_in >>
           output: opt!(complete!(redirect_out)) >>
           (Process {
                argument_list,
                input: Some(input),
                output,
           })
       ));
named!(process_out_in<Process>, // proc > file0 (< file1)
       do_parse!(
           argument_list: command >>
           output: redirect_out >>
           input: opt!(complete!(redirect_in)) >>
           (Process {
                argument_list,
                input,
                output: Some(output),
           })
       ));
named!(process_cmd<Process>,  // proc
       do_parse!(
           argument_list: command >>
           (Process {
               argument_list,
               input: None,
               output: None,
           })
       ));

named!(command<Vec<String> >, ws!(many1!(token)));

named!(redirect_in<process::Input>,
       ws!(do_parse!(
               tag_s!("<") >>
               file_name: token >>
               (process::Input::Redirect(file_name))
          )
       ));

named!(redirect_out<process::Output>,
       alt!(redirect_trunc | redirect_append));
named!(redirect_trunc<process::Output>,
       ws!(do_parse!(
               tag_s!(">") >>
               file_name: token >>
               (process::Output::Redirect(process::OutputRedirect::Truncate(file_name)))
          )
       ));
named!(redirect_append<process::Output>,
       ws!(do_parse!(
               tag_s!(">>") >>
               file_name: token >>
               (process::Output::Redirect(process::OutputRedirect::Append(file_name)))
          )
       ));

named!(pipe, tag_s!("|"));
named!(background, tag_s!("&"));

named!(token<String>,
       map_res!(
           map_res!(
               recognize!(tk),
               str::from_utf8
            ),
            FromStr::from_str
       ));
named!(tk<()>,
       do_parse!(
           none_of!("<>|& \t;\r\n") >>
           opt!(complete!(is_not!("<>|& \t;\r\n"))) >>
           ()
       ));

named!(end_of_job, alt!(eof | eol));
named!(eof, eof!());
named!(eol, is_a!(";\r\n"));

#[cfg(test)]
mod tests {
    use nom::IResult::Done;

    use super::*;

    macro_rules! str_ref {
        ($s: expr) => { & $s [..] }
    }
    macro_rules! string_vec {
        ($($s: expr), *) => { vec![$(String::from($s)), *] }
    }

    const EMPTY: &'static [u8] = b"";
    macro_rules! empty {
        () => { str_ref!(EMPTY) }
    }

    #[test]
    fn token_test() {
        assert_eq!(
            token(b"t"),
            Done(empty!(), String::from("t")));
        assert_eq!(
            token(b"token"),
            Done(empty!(), String::from("token")));
        assert_eq!(
            token(b"token<"),
            Done(str_ref!(b"<"), String::from("token")));
        assert_eq!(
            token(b"token>|&"),
            Done(str_ref!(b">|&"), String::from("token")));
        assert_eq!(
            token(b"token "),
            Done(str_ref!(b" "), String::from("token")));
        assert_eq!(
            token(b"token token"),
            Done(str_ref!(b" token"), String::from("token")));
        assert_eq!(
            token(b"token\ttoken  "),
            Done(str_ref!(b"\ttoken  "), String::from("token")));

        assert!(token(b"").is_incomplete());
    }

    #[test]
    fn command_test() {
        assert_eq!(
            command(b"cmd"),
            Done(empty!(), string_vec!["cmd"]));
        assert_eq!(
            command(b"cmd arg"),
            Done(empty!(), string_vec!["cmd", "arg"]));
        assert_eq!(
            command(b" cmd  arg0\targ1 \t"),
            Done(empty!(), string_vec!["cmd", "arg0", "arg1"]));
    }

    #[test]
    fn redirect_in_test() {
        use self::process::Input::Redirect;

        assert_eq!(
            redirect_in(b"< file_name"),
            Done(
                empty!(),
                Redirect(String::from("file_name"))
            ));
        assert_eq!(
            redirect_in(b" <file_name "),
            Done(
                empty!(),
                Redirect(String::from("file_name"))
            ));
    }

    #[test]
    fn redirect_out_test() {
        use self::process::Output::Redirect;
        use self::process::OutputRedirect::{Truncate, Append};

        assert_eq!(
            redirect_out(b"> file_name"),
            Done(
                empty!(),
                Redirect(Truncate(String::from("file_name")))
            ));
        assert_eq!(
            redirect_out(b" >file_name "),
            Done(
                empty!(),
                Redirect(Truncate(String::from("file_name")))
            ));
        assert_eq!(
            redirect_out(b">> file_name"),
            Done(
                empty!(),
                Redirect(Append(String::from("file_name")))
            ));
    }

    #[test]
    fn process_test() {
        use self::process::{Input, Output};
        use self::process::OutputRedirect::{Truncate, Append};

        assert_eq!(
            process_1only_elem(b"cmd"),
            Done(
                empty!(),
                Process {
                    argument_list: string_vec!["cmd"],
                    input: None,
                    output: None,
                 }
            ));
        assert_eq!(
            process_1only_elem(b"cmd < file"),
            Done(
                empty!(),
                Process {
                    argument_list: string_vec!["cmd"],
                    input: Some(Input::Redirect(String::from("file"))),
                    output: None,
                }
            ));
        assert_eq!(
            process_1only_elem(b"cmd > file"),
            Done(
                empty!(),
                Process {
                    argument_list: string_vec!["cmd"],
                    input: None,
                    output: Some(Output::Redirect(Truncate(String::from("file"))))
                },
            ));
        assert_eq!(
            process_1only_elem(b"cmd arg0 arg1 < file0 >> file1"),
            Done(
                empty!(),
                Process {
                    argument_list: string_vec!["cmd", "arg0", "arg1"],
                    input: Some(Input::Redirect(String::from("file0"))),
                    output: Some(Output::Redirect(Append(String::from("file1"))))
                }
            ));

        assert_eq!(
            process_1only_elem(b"cmd arg0 arg1 < file0 >> file1"),
            process_1only_elem(b"cmd arg0 arg1 >> file1 < file0"));
        assert_eq!(
            process_1only_elem(b"cmd arg0 arg1 < file0 >> file1"),
            process_1only_elem(b" cmd \t arg0 arg1 >> file1 < file0 \n"));

        assert!(!process_1only_elem(b"< file0 cmd").is_done());
        assert!(!process_1only_elem(b"> file0 cmd").is_done());
    }

    #[test]
    fn job_test() {
        use self::process::{Input, Output};
        use self::process::OutputRedirect::{Truncate, Append};

        assert_eq!(
            job(b"cmd"),
            Done(
                empty!(),
                Job {
                    process_list: vec![
                        Process {
                            argument_list: string_vec!["cmd"],
                            input: None,
                            output: None,
                        },
                    ],
                    mode: JobMode::ForeGround,
                }
            ));
        assert_eq!(
            job(b"cmd < file0 > file1"),
            Done(
                empty!(),
                Job {
                    process_list: vec![
                        Process {
                            argument_list: string_vec!["cmd"],
                            input: Some(Input::Redirect(String::from("file0"))),
                            output: Some(Output::Redirect(Truncate(String::from("file1")))),
                        },
                    ],
                    mode: JobMode::ForeGround,
                }
            ));
        assert_eq!(
            job(b"cmd0 | cmd1"),
            Done(
                empty!(),
                Job {
                    process_list: vec![
                        Process {
                            argument_list: string_vec!["cmd0"],
                            input: None,
                            output: Some(Output::Pipe),
                        },
                        Process {
                            argument_list: string_vec!["cmd1"],
                            input: Some(Input::Pipe),
                            output: None,
                        },
                    ],
                    mode: JobMode::ForeGround,
                }
            ));
        assert_eq!(
            job(b"cmd0 < file0 | cmd1 arg1 > file1"),
            Done(
                empty!(),
                Job {
                    process_list: vec![
                        Process {
                            argument_list: string_vec!["cmd0"],
                            input: Some(Input::Redirect(String::from("file0"))),
                            output: Some(Output::Pipe),
                        },
                        Process {
                            argument_list: string_vec!["cmd1", "arg1"],
                            input: Some(Input::Pipe),
                            output: Some(Output::Redirect(Truncate(String::from("file1")))),
                        },
                    ],
                    mode: JobMode::ForeGround,
                }
            ));
        assert_eq!(
            job(b"cmd0 < file0 | cmd1 arg1 | cmd2 arg2 arg3 >> file3 &"),
            Done(
                empty!(),
                Job {
                    process_list: vec![
                        Process {
                            argument_list: string_vec!["cmd0"],
                            input: Some(Input::Redirect(String::from("file0"))),
                            output: Some(Output::Pipe),
                        },
                        Process {
                            argument_list: string_vec!["cmd1", "arg1"],
                            input: Some(Input::Pipe),
                            output: Some(Output::Pipe),
                        },
                        Process {
                            argument_list: string_vec!["cmd2", "arg2", "arg3"],
                            input: Some(Input::Pipe),
                            output: Some(Output::Redirect(Append(String::from("file3")))),
                        },
                    ],
                    mode: JobMode::BackGround,
                }));

        assert_eq!(
            job(b"cmd0 < file0 | cmd1 arg1 | cmd2 arg2 arg3 >> file3 &"),
            job(b" cmd0 < file0 \t | cmd1 arg1 | cmd2 arg2 arg3 >> file3 & \n"));

        macro_rules! assert_err {
            ($s: expr) => { assert!(job($s).is_err()) }
        }

        assert_err!(b"| cmd");
        assert_err!(b"cmd |");
        assert_err!(b"&");
        assert_err!(b"cmd < file |");

        assert_err!(b"> file");
        assert_err!(b"file >");

        assert_err!(b"& cmd");
        assert_err!(b"cmd0 & | cmd1");

        assert_err!(b"cmd0 > file | cmd1");
        assert_err!(b"cmd0 | cmd1 < file");
        assert_err!(b"cmd0 | cmd1 > file | cmd2");
    }
}
