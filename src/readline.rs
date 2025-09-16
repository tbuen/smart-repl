use crate::tokenizer::{self, TokenList};
use log::{debug, error};
use rustyline::config::Builder;
use rustyline::error::ReadlineError;
use rustyline::history::MemHistory;
use rustyline::{CompletionType, Editor};

pub enum ReadError {
    InvalidInput,
    Eof,
    Interrupted,
    Io(String),
}

pub struct Reader {
    rusty: Editor<(), MemHistory>,
    prompt: String,
}

impl Reader {
    pub fn new(prompt: &str) -> Self {
        let rusty =
            Editor::with_config(Builder::new().completion_type(CompletionType::List).build())
                .unwrap();
        //rl.set_helper(Some(CommandHelper::new(commands)));
        Self {
            rusty,
            prompt: prompt.into(),
        }
    }

    pub fn read_line(&mut self) -> Result<TokenList, ReadError> {
        match self.rusty.readline(&self.prompt) {
            Ok(line) => match tokenizer::tokenize(&line) {
                Ok(list) => Ok(list),
                Err(()) => Err(ReadError::InvalidInput),
            },
            /*
            match self.rl.helper().unwrap().parse(&line) {
                Ok(res) => {
                    if self.interpret(res) {
                        match self.rl.add_history_entry(line) {
                            Ok(b) => println!("history: {b}"),
                            Err(e) => println!("{:?}", e),
                        }
                    }
                }
                Err(_) => println!("## invalid input"),
            },*/
            Err(ReadlineError::Interrupted) => {
                debug!("CTRL-C");
                Err(ReadError::Interrupted)
            }
            Err(ReadlineError::Eof) => {
                debug!("CTRL-D");
                Err(ReadError::Eof)
            }
            Err(err) => {
                error!("{err:?}");
                Err(ReadError::Io(err.to_string()))
            }
        }
    }
}
