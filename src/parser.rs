use std::str;
use std::str::FromStr;

use nom::{self, multispace};

use errors::*;
use job::*;

pub(crate) fn parse_job(input: &[u8]) -> Result<Job> {
    if let nom::IResult::Done(_, job) = job(input) {
        Ok(job)
    } else {
        Err(Error::ParseJob)
    }
}

named!(job<&[u8], Job>,
       do_parse!(
           process_list: separated_nonempty_list_complete!(pipe, process) >>
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

named!(process<&[u8], Process>,
       alt!(process0 | process1 | process2));
named!(process0<&[u8], Process>,
       do_parse!(
           argument_list: command >>
           redirect_in: complete!(redirect_in) >>
           redirect_out: opt!(complete!(redirect_out)) >>
           (Process {
               argument_list,
               redirect_in: Some(redirect_in),
               redirect_out
           })
       ));
named!(process1<&[u8], Process>,
       do_parse!(
           argument_list: command >>
           redirect_out: complete!(redirect_out) >>
           redirect_in: opt!(complete!(redirect_in)) >>
           (Process {
               argument_list,
               redirect_in,
               redirect_out: Some(redirect_out)
           })
       ));
named!(process2<&[u8], Process>,
       do_parse!(
           argument_list: command >>
           (Process {
               argument_list,
               redirect_in: None,
               redirect_out: None
           })
       ));

named!(command<Vec<String> >, ws!(many1!(token)));

named!(redirect_in<String>,
       ws!(do_parse!(
               tag_s!("<") >>
               filename: token >>
               (filename)
          )
       ));

named!(redirect_out<OutputRedirection>,
       alt!(redirect_out_trunc | redirect_out_append));
named!(redirect_out_trunc<OutputRedirection>,
       ws!(do_parse!(
               tag_s!(">") >>
               filename: token >>
               (OutputRedirection {
                   filename,
                   mode: WriteMode::Truncate
               } )
          )
       ));
named!(redirect_out_append<OutputRedirection>,
       ws!(do_parse!(
               tag_s!(">>") >>
               filename: token >>
               (OutputRedirection {
                   filename,
                   mode: WriteMode::Append
               } )
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
           none_of!("<>|& \t\r\n") >>
           opt!(complete!(is_not!("<>|& \t\r\n"))) >>
           ()
       ));

named!(end_of_job, alt!(eof | eol));
named!(eof, eof!());
named!(eol, is_a!(";\r\n"));

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult::Done;

    macro_rules! str_ref {
        ($s: expr) => { & $s [..] }
    }
    macro_rules! string_vec {
        ($($s: expr), *) => { vec![$(String::from($s)), *] }
    }

    #[test]
    fn token_test() {
        let empty = str_ref!(b"");

        assert_eq!(
            token(b"t"),
            Done(empty, String::from("t")));
        assert_eq!(
            token(b"token"),
            Done(empty, String::from("token")));
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
        let empty = &b""[..];

        assert_eq!(
            command(b"cmd"),
            Done(empty, string_vec!["cmd"]));
        assert_eq!(
            command(b"cmd arg"),
            Done(empty, string_vec!["cmd", "arg"]));
        assert_eq!(
            command(b" cmd  arg0\targ1 \t"),
            Done(empty, string_vec!["cmd", "arg0", "arg1"]));
    }

    #[test]
    fn redirect_in_test() {
        let empty = str_ref!(b"");

        assert_eq!(
            redirect_in(b"< filename"),
            Done(empty, String::from("filename")));
        assert_eq!(
            redirect_in(b" <filename "),
            Done(empty, String::from("filename")));
    }

    #[test]
    fn redirect_out_test() {
        let empty = str_ref!(b"");

        assert_eq!(
            redirect_out(b"> filename"),
            Done(empty,
                 OutputRedirection {
                     filename: String::from("filename"),
                     mode: WriteMode::Truncate
                 }));
        assert_eq!(
            redirect_out(b" >filename "),
            Done(empty,
                 OutputRedirection {
                     filename: String::from("filename"),
                     mode: WriteMode::Truncate
                 }));
        assert_eq!(
            redirect_out(b">> filename"),
            Done(empty,
                 OutputRedirection {
                     filename: String::from("filename"),
                     mode: WriteMode::Append
                 }));
    }

    #[test]
    fn process_test() {
        let empty = str_ref!(b"");

        assert_eq!(
            process(b"cmd"),
            Done(empty,
                 Process {
                     argument_list: string_vec!["cmd"],
                     redirect_in: None,
                     redirect_out: None,
                 }));
        assert_eq!(
            process(b"cmd < file"),
            Done(empty,
                 Process {
                     argument_list: string_vec!["cmd"],
                     redirect_in: Some(String::from("file")),
                     redirect_out: None,
                 }));
        assert_eq!(
            process(b"cmd > file"),
            Done(empty,
                 Process {
                     argument_list: string_vec!["cmd"],
                     redirect_in: None,
                     redirect_out: Some(OutputRedirection {
                         filename: String::from("file"),
                         mode: WriteMode::Truncate
                     }),
                 }));
        assert_eq!(
            process(b"cmd arg0 arg1 < file0 >> file1"),
            Done(empty,
                 Process {
                     argument_list: string_vec!["cmd", "arg0", "arg1"],
                     redirect_in: Some(String::from("file0")),
                     redirect_out: Some(OutputRedirection {
                         filename: String::from("file1"),
                         mode: WriteMode::Append
                     }),
                 }));

        assert_eq!(
            process(b"cmd arg0 arg1 < file0 >> file1"),
            process(b"cmd arg0 arg1 >> file1 < file0"));
        assert_eq!(
            process(b"cmd arg0 arg1 < file0 >> file1"),
            process(b" cmd \t arg0 arg1 >> file1 < file0 \n"));

        assert!(!process(b"< file0 cmd").is_done());
        assert!(!process(b"> file0 cmd").is_done());
    }

    #[test]
    fn parse_job_test() {
        assert_eq!(
            parse_job(b"cmd0 < file0 | cmd1 arg1 > file1"),
            Ok(Job {
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
                                filename: String::from("file1"),
                                mode: WriteMode::Truncate
                            }),
                    },
                ],
                mode: JobMode::ForeGround
            }));
        assert_eq!(
            parse_job(b"cmd0 < file0 | cmd1 arg1 < file1 > file2 | cmd2 arg2 arg3 >> file3 &"),
            Ok(Job {
                process_list: vec![
                    Process {
                        argument_list: string_vec!["cmd0"],
                        redirect_in: Some(String::from("file0")),
                        redirect_out: None,
                    },
                    Process {
                        argument_list: string_vec!["cmd1", "arg1"],
                        redirect_in: Some(String::from("file1")),
                        redirect_out: Some(
                            OutputRedirection {
                                filename: String::from("file2"),
                                mode: WriteMode::Truncate
                            }),
                    },
                    Process {
                        argument_list: string_vec!["cmd2", "arg2", "arg3"],
                        redirect_in: None,
                        redirect_out: Some(
                            OutputRedirection {
                                filename: String::from("file3"),
                                mode: WriteMode::Append
                            }),
                    },
                ],
                mode: JobMode::BackGround
            }));

        assert_eq!(
            parse_job(b"cmd0 < file0 | cmd1 arg1 < file1 > file2 | cmd2 arg2 arg3 >> file3 &"),
            parse_job(b" cmd0 < file0 \t | cmd1 arg1 < file1 > file2 | cmd2 arg2 arg3 >> file3 & \n"));

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
