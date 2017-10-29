use std::env;
use std::path;

use super::*;

pub(super) fn cd(args: &[String]) -> Result<()> {
    let target = if args.len() == 1 {
        env::home_dir().expect("Could not get your home directory")
    } else {
        path::Path::new(&args[1]).to_path_buf()
    };

    env::set_current_dir(&target).map_err(|_| {
        Error::Builtin(format!("cd: No such file or directory: {:?}", target))
    })
}
