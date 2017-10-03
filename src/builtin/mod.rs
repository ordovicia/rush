use errors::{Result, Error};

pub(super) mod cd;

pub(super) fn exec(argument_list: &[String]) -> Result<()> {
    match argument_list[0].as_ref() {
        "cd" => cd::cd(argument_list),
        _ => Err(Error::NotBuiltin),
    }
}
