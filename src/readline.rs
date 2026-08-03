use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};

pub struct CommandHelper {
    pub commands: Vec<&'static str>,
}

impl Helper for CommandHelper {}

impl Completer for CommandHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<String>)> {
        let mut matches = vec![];
        let prefix = &line[..pos];

        // TODO: less naive approach; a Trie?
        for &cmd in &self.commands {
            if cmd.starts_with(prefix) {
                matches.push(format!("{} ", cmd));
            }
        }

        Ok((0, matches))
    }
}

impl Hinter for CommandHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for CommandHelper {}
impl Validator for CommandHelper {}
