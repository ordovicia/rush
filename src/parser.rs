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
//! job         := proc_list "&"? end_job
//! end_job     := eof | ";" | "\n" | "\r"
//! ```

use std::str::{self, FromStr};

use nom::{IResult, multispace};

use errors::*;
use job::*;

pub(crate) fn parse_job(input: &[u8]) -> Result<(&[u8], Job)> {
    match job(input) {
        IResult::Done(remained, job) => Ok((remained, job)),
        IResult::Error(e) => Err(Error::ParseError(ParseError::Error(e))),
        IResult::Incomplete(e) => Err(Error::ParseError(ParseError::Incomplete(e))),
    }
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
       alt!(process_2many | process_1only));

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
           redirect_in: opt!(complete!(redirect_in)) >>
           (Process {
               argument_list,
               redirect_in,
               redirect_out: None,
           })
       ));
named!(process_cmd_pipe<Process>, // .. | [proc |] ..
       do_parse!(
           argument_list: command >>
           pipe >>
           (Process {
               argument_list,
               redirect_in: None,
               redirect_out: None,
           })
       ));
named!(process_out<Process>, // .. | [proc (> file)]
       do_parse!(
           argument_list: command >>
           redirect_out: opt!(complete!(redirect_out)) >>
           (Process {
               argument_list,
               redirect_in: None,
               redirect_out,
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
           redirect_in: redirect_in >>
           redirect_out: opt!(complete!(redirect_out)) >>
           (Process {
                argument_list,
                redirect_in: Some(redirect_in),
                redirect_out,
           })
       ));
named!(process_out_in<Process>, // proc > file0 (< file1)
       do_parse!(
           argument_list: command >>
           redirect_out: redirect_out >>
           redirect_in: opt!(complete!(redirect_in)) >>
           (Process {
                argument_list,
                redirect_in,
                redirect_out: Some(redirect_out),
           })
       ));
named!(process_cmd<Process>,  // proc
       do_parse!(
           argument_list: command >>
           (Process {
               argument_list,
               redirect_in: None,
               redirect_out: None,
           })
       ));

named!(command<Vec<String> >, ws!(many1!(token)));

named!(redirect_in<String>,
       ws!(do_parse!(
               tag_s!("<") >>
               file_name: token >>
               (file_name)
          )
       ));

named!(redirect_out<OutputRedirection>,
       alt!(redirect_out_trunc | redirect_out_append));
named!(redirect_out_trunc<OutputRedirection>,
       ws!(do_parse!(
               tag_s!(">") >>
               file_name: token >>
               (OutputRedirection {
                   file_name,
                   mode: WriteMode::Truncate,
               })
          )
       ));
named!(redirect_out_append<OutputRedirection>,
       ws!(do_parse!(
               tag_s!(">>") >>
               file_name: token >>
               (OutputRedirection {
                   file_name,
                   mode: WriteMode::Append,
               })
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

    #[test]
    fn token_test() {
        assert_eq!(
            token(b"t"),
            Done(str_ref!(EMPTY), String::from("t")));
        assert_eq!(
            token(b"token"),
            Done(str_ref!(EMPTY), String::from("token")));
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

        if !token(b"").is_incomplete() {
            assert!(false);
        }
    }

    #[test]
    fn command_test() {
        assert_eq!(
            command(b"cmd"),
            Done(str_ref!(EMPTY), string_vec!["cmd"]));
        assert_eq!(
            command(b"cmd arg"),
            Done(str_ref!(EMPTY), string_vec!["cmd", "arg"]));
        assert_eq!(
            command(b" cmd  arg0\targ1 \t"),
            Done(str_ref!(EMPTY), string_vec!["cmd", "arg0", "arg1"]));
    }

    #[test]
    fn redirect_in_test() {
        assert_eq!(
            redirect_in(b"< file_name"),
            Done(str_ref!(EMPTY), String::from("file_name")));
        assert_eq!(
            redirect_in(b" <file_name "),
            Done(str_ref!(EMPTY), String::from("file_name")));
    }

    #[test]
    fn redirect_out_test() {
        assert_eq!(
            redirect_out(b"> file_name"),
            Done(str_ref!(EMPTY),
                 OutputRedirection {
                     file_name: String::from("file_name"),
                     mode: WriteMode::Truncate
                 }));
        assert_eq!(
            redirect_out(b" >file_name "),
            Done(str_ref!(EMPTY),
                 OutputRedirection {
                     file_name: String::from("file_name"),
                     mode: WriteMode::Truncate
                 }));
        assert_eq!(
            redirect_out(b">> file_name"),
            Done(str_ref!(EMPTY),
                 OutputRedirection {
                     file_name: String::from("file_name"),
                     mode: WriteMode::Append
                 }));
    }

    #[test]
    fn process_test() {
        assert_eq!(
            process_1only_elem(b"cmd"),
            Done(str_ref!(EMPTY),
                 Process {
                     argument_list: string_vec!["cmd"],
                     redirect_in: None,
                     redirect_out: None,
                 }));
        assert_eq!(
            process_1only_elem(b"cmd < file"),
            Done(str_ref!(EMPTY),
                 Process {
                     argument_list: string_vec!["cmd"],
                     redirect_in: Some(String::from("file")),
                     redirect_out: None,
                 }));
        assert_eq!(
            process_1only_elem(b"cmd > file"),
            Done(str_ref!(EMPTY),
                 Process {
                     argument_list: string_vec!["cmd"],
                     redirect_in: None,
                     redirect_out: Some(OutputRedirection {
                         file_name: String::from("file"),
                         mode: WriteMode::Truncate
                     }),
                 }));
        assert_eq!(
            process_1only_elem(b"cmd arg0 arg1 < file0 >> file1"),
            Done(str_ref!(EMPTY),
                 Process {
                     argument_list: string_vec!["cmd", "arg0", "arg1"],
                     redirect_in: Some(String::from("file0")),
                     redirect_out: Some(OutputRedirection {
                         file_name: String::from("file1"),
                         mode: WriteMode::Append
                     }),
                 }));

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
    fn parse_job_test() {
        assert_eq!(
            parse_job(b"cmd0 < file0 | cmd1 arg1 > file1"),
            Ok((str_ref!(EMPTY), Job {
                process_list: vec![
                    Process {
                        argument_list: string_vec!["cmd0"],
                        redirect_in: Some(String::from("file0")),
                        redirect_out: None,
                    },
                    Process {
                        argument_list: string_vec!["cmd1", "arg1"],
                        redirect_in: None,
                        redirect_out: Some(
                            OutputRedirection {
                                file_name: String::from("file1"),
                                mode: WriteMode::Truncate
                            }),
                    },
                ],
                mode: JobMode::ForeGround
            })));
        assert_eq!(
            parse_job(b"cmd0 < file0 | cmd1 arg1 | cmd2 arg2 arg3 >> file3 &"),
            Ok((str_ref!(EMPTY), Job {
                process_list: vec![
                    Process {
                        argument_list: string_vec!["cmd0"],
                        redirect_in: Some(String::from("file0")),
                        redirect_out: None,
                    },
                    Process {
                        argument_list: string_vec!["cmd1", "arg1"],
                        redirect_in: None,
                        redirect_out: None,
                    },
                    Process {
                        argument_list: string_vec!["cmd2", "arg2", "arg3"],
                        redirect_in: None,
                        redirect_out: Some(
                            OutputRedirection {
                                file_name: String::from("file3"),
                                mode: WriteMode::Append
                            }),
                    },
                ],
                mode: JobMode::BackGround
            })));

        assert_eq!(
            parse_job(b"cmd0 < file0 | cmd1 arg1 | cmd2 arg2 arg3 >> file3 &"),
            parse_job(b" cmd0 < file0 \t | cmd1 arg1 | cmd2 arg2 arg3 >> file3 & \n"));

        macro_rules! assert_err {
            ($s: expr) => { assert!(parse_job($s).is_err()) }
        }

        assert_err!(b"| cmd");
        assert_err!(b"cmd |");
        assert_err!(b"&");
        assert_err!(b"cmd < file |");

        assert_err!(b"> file");
        assert_err!(b"file >");

        assert_err!(b"& cmd");
        assert_err!(b"cmd0 & | cmd1");
    }
}
