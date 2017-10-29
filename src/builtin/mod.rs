use errors::{Error, Result};

pub(super) mod cd;

pub(super) fn exec(argument_list: &[String]) -> Option<Result<()>> {
    match argument_list[0].as_ref() {
        "cd" => Some(cd::cd(argument_list)),
        _ => None,
    }
}
