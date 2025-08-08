use log::trace;
use readline::{ReadError, Reader};

mod parser;
mod readline;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("VERSION");

pub struct Repl<Ctx> {
    reader: Reader,
    cmds: Vec<Command<Ctx>>,
    grps: Vec<Group<Ctx>>,
}

impl<Ctx> Repl<Ctx> {
    pub fn builder() -> ReplBuilder<Ctx> {
        ReplBuilder::new()
    }

    pub fn run(&mut self) {
        loop {
            match self.reader.read_line() {
                Ok(tokens) => {
                    trace!("{tokens:?}");
                    parser::parse(&self.cmds, &self.grps, &tokens);
                }
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
                Err(ReadError::InvalidInput) => println!("Invalid input"),
                Err(ReadError::Io(e)) => {
                    eprintln!("{e}");
                    break;
                }
                Err(ReadError::Eof) | Err(ReadError::Interrupted) => break,
            }
        }
    }
}

pub struct ReplBuilder<Ctx> {
    prompt: String,
    cmds: Vec<Command<Ctx>>,
    grps: Vec<Group<Ctx>>,
}

impl<Ctx> ReplBuilder<Ctx> {
    const DEFAULT_PROMPT: &str = ">";

    fn new() -> Self {
        Self {
            prompt: ReplBuilder::<Ctx>::DEFAULT_PROMPT.into(),
            cmds: Vec::new(),
            grps: Vec::new(),
        }
    }

    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.prompt = prompt.into();
        self
    }

    pub fn with_group(mut self, grp: Group<Ctx>) -> Self {
        self.grps.push(grp);
        self
    }

    pub fn with_command(mut self, cmd: Command<Ctx>) -> Self {
        self.cmds.push(cmd);
        self
    }

    pub fn build(self) -> Repl<Ctx> {
        Repl::<Ctx> {
            reader: Reader::new(&self.prompt),
            grps: self.grps,
            cmds: self.cmds,
        }
    }
}

pub struct Group<Ctx> {
    name: String,
    cmds: Vec<Command<Ctx>>,
}

impl<Ctx> Group<Ctx> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            cmds: Vec::new(),
        }
    }

    pub fn with_command(mut self, cmd: Command<Ctx>) -> Self {
        self.cmds.push(cmd);
        self
    }
}

pub struct Command<Ctx> {
    name: String,
    params: Vec<Parameter>,
    _cb: Box<dyn FnOnce(Ctx, Args)>,
}

impl<Ctx> Command<Ctx> {
    pub fn new<Cb>(name: &str, cb: Cb) -> Self
    where
        Cb: FnOnce(Ctx, Args) + 'static,
    {
        Self {
            name: name.into(),
            params: Vec::new(),
            _cb: Box::new(cb),
        }
    }

    pub fn with_parameter(mut self, param: Parameter) -> Self {
        self.params.push(param);
        self
    }
}

pub enum Parameter {
    String(String),
}

impl Parameter {
    pub fn string(name: &str) -> Self {
        Self::String(name.into())
    }
}

pub struct Args;
