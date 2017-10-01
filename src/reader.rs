use rustyline;
use job::Job;

use errors::{Result, Error};

pub(crate) struct Reader {
    rl: rustyline::Editor<()>,
    prompt: &'static str,
}

impl Reader {
    pub(crate) fn new() -> Self {
        Reader {
            rl: rustyline::Editor::<()>::new(),
            prompt: "rush $ ",
        }
    }

    pub(crate) fn read_job(&mut self) -> Result<Job> {
        use parser;

        let line = self.rl.readline(self.prompt)?;
        parser::parse_job(line.as_bytes()).map_err(Error::from)
    }
}
