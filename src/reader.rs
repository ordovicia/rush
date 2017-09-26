use rustyline;

use job::Job;
use errors::{Result, Error};

pub struct Reader {
    rl: rustyline::Editor<()>,
    prompt: &'static str,
}

impl Reader {
    pub fn new() -> Self {
        Reader {
            rl: rustyline::Editor::<()>::new(),
            prompt: "rush $ ",
        }
    }

    pub fn set_prompt(&mut self, prompt: &'static str) {
        self.prompt = prompt;
    }

    pub fn read_job(&mut self) -> Result<Job> {
        use parser;

        let line = self.readline()?;
        parser::parse_job(line.as_bytes()).map_err(|e| Error::from(e))
    }

    fn readline(&mut self) -> Result<String> {
        self.rl.readline(self.prompt).map_err(|e| Error::from(e))
    }
}
