//! # Syntax of job (white space skipped)
//!
//! ```ignore
//! arg_list     := token+
//!
//! redir_in     := "<" token
//! redir_trunc  := ">" token
//! redir_append := ">>" token
//! redir_out    := redir_trunc
//!               | redir_append
//!
//! proc_cdr     := arg_list proc_out?
//! pipe_proc    := "|" proc_cdr
//! proc_out     := pipe_proc
//!               | redir_out
//!
//! proc_car     := arg_list
//!               | arg_list proc_out
//!               | arg_list redir_in proc_out?
//!
//! end_job     := eof | ";" | "\n" | "\r"
//! job         := proc_car "&"? end_job
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
           process_list: process_car >>
           bg: opt!(complete!(background)) >>
           opt!(complete!(multispace)) >>
           end_of_job >>
           (Job::new(
                   process_list,
                   if bg.is_some() {
                       JobMode::BackGround
                   } else {
                       JobMode::ForeGround
                   }
           ))
       ));

named!(process_car<Process>,
       alt!(complete!(process_car_inout) | complete!(process_car_out) | process_car_cmd));
named!(process_car_cmd<Process>,
       do_parse!(
           argument_list: argument_list >>
           (Process::new(
                   argument_list,
                   process::Input::Inherit,
                   process::Output::Inherit,
           ))
       ));
named!(process_car_out<Process>,
       do_parse!(
           argument_list: argument_list >>
           output: process_output >>
           (Process::new(
                   argument_list,
                   process::Input::Inherit,
                   output,
           ))
       ));
named!(process_car_inout<Process>,
       do_parse!(
           argument_list: argument_list >>
           input: redirect_in >>
           output: opt!(complete!(process_output)) >>
           (Process::new(
                   argument_list,
                   input,
                   output.unwrap_or(process::Output::Inherit),
           ))
       ));
named!(process_output<process::Output>,
       alt!(pipe_process | redirect_out));
named!(pipe_process<process::Output>,
       do_parse!(
           pipe >>
           process: process_cdr >>
           (process::Output::Pipe(Box::new(process)))
       ));
named!(process_cdr<Process>,
       do_parse!(
           argument_list: argument_list >>
           output: opt!(complete!(process_output)) >>
           (Process::new(
                   argument_list,
                   process::Input::Pipe,
                   output.unwrap_or(process::Output::Inherit)
           ))
       ));

named!(argument_list<Vec<String> >, ws!(many1!(token)));

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
    fn argument_list_test() {
        assert_eq!(
            argument_list(b"cmd"),
            Done(empty!(), string_vec!["cmd"]));
        assert_eq!(
            argument_list(b"cmd arg"),
            Done(empty!(), string_vec!["cmd", "arg"]));
        assert_eq!(
            argument_list(b" cmd  arg0\targ1 \t"),
            Done(empty!(), string_vec!["cmd", "arg0", "arg1"]));
    }

    #[test]
    fn redirect_in_test() {
        use self::process::Input::Redirect;

        assert_eq!(
            redirect_in(b"< file_name"),
            Done(empty!(), Redirect(String::from("file_name"))));
        assert_eq!(
            redirect_in(b" <file_name "),
            Done(empty!(), Redirect(String::from("file_name"))));
    }

    #[test]
    fn redirect_out_test() {
        use self::process::Output::Redirect;
        use self::process::OutputRedirect::{Truncate, Append};

        assert_eq!(
            redirect_out(b"> file_name"),
            Done(empty!(), Redirect(Truncate(String::from("file_name")))));
        assert_eq!(
            redirect_out(b" >file_name "),
            Done(empty!(), Redirect(Truncate(String::from("file_name")))));
        assert_eq!(
            redirect_out(b">> file_name"),
            Done(empty!(), Redirect(Append(String::from("file_name")))));
    }

    #[test]
    fn process_test() {
        use self::process::{Input, Output};
        use self::process::OutputRedirect::{Truncate, Append};

        assert_eq!(
            process_car(b"cmd"),
            Done(
                empty!(),
                Process::new(
                    string_vec!["cmd"],
                    Input::Inherit,
                    Output::Inherit,
                ),
            ));
        assert_eq!(
            process_car(b"cmd < file"),
            Done(
                empty!(),
                Process::new(
                    string_vec!["cmd"],
                    Input::Redirect(String::from("file")),
                    Output::Inherit,
                ),
            ));
        assert_eq!(
            process_car(b"cmd > file"),
            Done(
                empty!(),
                Process::new(
                    string_vec!["cmd"],
                    Input::Inherit,
                    Output::Redirect(Truncate(String::from("file")))
                ),
            ));
        assert_eq!(
            process_car(b"cmd arg0 arg1 < file0 >> file1"),
            Done(
                empty!(),
                Process::new(
                    string_vec!["cmd", "arg0", "arg1"],
                    Input::Redirect(String::from("file0")),
                    Output::Redirect(Append(String::from("file1")))
                ),
            ));

        assert_eq!(
            process_car(b"cmd arg0 arg1 < file0 >> file1"),
            process_car(b" cmd \t arg0 arg1 < file0 >> file1\n"));

        assert!(process_car(b"< file cmd").is_err());
        assert!(process_car(b"> file cmd").is_err());
        assert!(
            if let Done(remained, _) = process_car(b"cmd > file0 < file1") {
                let remained = String::from_utf8_lossy(remained);
                remained == String::from("< file1")
            } else {
                false
            });
    }

    #[test]
    fn job_test() {
        use self::process::{Input, Output};
        use self::process::OutputRedirect::{Truncate, Append};

        assert_eq!(
            job(b"cmd"),
            Done(
                empty!(),
                Job::new(
                    Process::new(
                        string_vec!["cmd"],
                        Input::Inherit,
                        Output::Inherit,
                    ),
                    JobMode::ForeGround,
                ),
            ));
        assert_eq!(
            job(b"cmd < file0 > file1"),
            Done(
                empty!(),
                Job::new(
                    Process::new(
                        string_vec!["cmd"],
                        Input::Redirect(String::from("file0")),
                        Output::Redirect(Truncate(String::from("file1"))),
                    ),
                    JobMode::ForeGround,
                ),
            ));
        assert_eq!(
            job(b"cmd0 | cmd1"),
            Done(
                empty!(),
                Job::new(
                    {
                        let proc1 = Process::new(
                            string_vec!["cmd1"],
                            Input::Pipe,
                            Output::Inherit,
                        );
                        Process::new(
                            string_vec!["cmd0"],
                            Input::Inherit,
                            Output::Pipe(Box::new(proc1)),
                        )
                    },
                    JobMode::ForeGround,
                ),
            ));
        assert_eq!(
            job(b"cmd0 < file0 | cmd1 arg1 > file1"),
            Done(
                empty!(),
                Job::new(
                    {
                        let proc1 = Process::new(
                            string_vec!["cmd1", "arg1"],
                            Input::Pipe,
                            Output::Redirect(Truncate(String::from("file1"))),
                        );
                        Process::new(
                            string_vec!["cmd0"],
                            Input::Redirect(String::from("file0")),
                            Output::Pipe(Box::new(proc1)),
                        )
                    },
                    JobMode::ForeGround,
                ),
            ));
        assert_eq!(
            job(b"cmd0 < file0 | cmd1 arg1 | cmd2 arg2 arg3 >> file3 &"),
            Done(
                empty!(),
                Job::new(
                    {
                        let proc2 = Process::new(
                            string_vec!["cmd2", "arg2", "arg3"],
                            Input::Pipe,
                            Output::Redirect(Append(String::from("file3"))),
                        );
                        let proc1 = Process::new(
                            string_vec!["cmd1", "arg1"],
                            Input::Pipe,
                            Output::Pipe(Box::new(proc2)),
                        );
                        Process::new(
                            string_vec!["cmd0"],
                            Input::Redirect(String::from("file0")),
                            Output::Pipe(Box::new(proc1)),
                        )
                    },
                    JobMode::BackGround,
                ),
            ));

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
