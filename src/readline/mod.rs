use log::{debug, error};
use rustyline::config::Builder;
use rustyline::error::ReadlineError;
use rustyline::history::MemHistory;
use rustyline::{CompletionType, Editor};
use tokenizer::{TokenList, Tokenizer};

mod tokenizer;

pub struct Reader {
    tokenizer: Tokenizer,
    rusty: Editor<(), MemHistory>,
}

impl Reader {
    pub fn new() -> Self {
        let rusty =
            Editor::with_config(Builder::new().completion_type(CompletionType::List).build())
                .unwrap();
        //rl.set_helper(Some(CommandHelper::new(commands)));
        Self {
            tokenizer: Tokenizer::new(),
            rusty,
        }
    }

    pub fn read_line(&mut self) -> Result<TokenList, ()> {
        let result = self.rusty.readline(">> ");
        match result {
            Ok(line) => match self.tokenizer.tokenize(&line) {
                Ok(list) => Ok(list),
                Err(_) => Err(()),
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
                Err(())
            }
            Err(ReadlineError::Eof) => {
                debug!("CTRL-D");
                Err(())
            }
            Err(err) => {
                error!("{err:?}");
                Err(())
            }
        }
    }
}
