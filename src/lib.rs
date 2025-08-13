use log::trace;
use readline::{ReadError, Reader};

mod parser;
mod readline;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("VERSION");

pub struct Repl<'a, Ctx> {
    ctx: Option<&'a Ctx>,
    reader: Reader,
    grps: Vec<Group<Ctx>>,
    cmds: Vec<Command<Ctx>>,
}

impl<'a, Ctx> Repl<'a, Ctx> {
    #[must_use]
    pub fn builder() -> ReplBuilder<'a, Ctx> {
        ReplBuilder::new()
    }

    pub fn run(&mut self) {
        loop {
            match self.reader.read_line() {
                Ok(tokens) => {
                    trace!("{tokens:?}");
                    parser::parse(self, self.ctx, &self.grps, &self.cmds, &tokens);
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
                Err(ReadError::Eof | ReadError::Interrupted) => break,
            }
        }
    }

    pub fn help(&self) {
        println!("COMMANDS");
        for c in &self.cmds {
            println!("    {}", c.name);
        }
        println!("GROUPS");
        for g in &self.grps {
            for c in &g.cmds {
                println!("    {} {}", g.name, c.name);
            }
        }
    }
}

pub struct ReplBuilder<'a, Ctx> {
    ctx: Option<&'a Ctx>,
    prompt: String,
    grps: Vec<Group<Ctx>>,
    cmds: Vec<Command<Ctx>>,
}

impl<'a, Ctx> ReplBuilder<'a, Ctx> {
    const DEFAULT_PROMPT: &'static str = ">";

    fn new() -> Self {
        Self {
            ctx: None,
            prompt: ReplBuilder::<Ctx>::DEFAULT_PROMPT.into(),
            grps: Vec::new(),
            cmds: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_context(mut self, ctx: &'a Ctx) -> Self {
        self.ctx = Some(ctx);
        self
    }

    #[must_use]
    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.prompt = prompt.into();
        self
    }

    #[must_use]
    pub fn with_group(mut self, grp: Group<Ctx>) -> Self {
        self.grps.push(grp);
        self
    }

    #[must_use]
    pub fn with_command(mut self, cmd: Command<Ctx>) -> Self {
        self.cmds.push(cmd);
        self
    }

    #[must_use]
    pub fn build(self) -> Repl<'a, Ctx> {
        Repl::<Ctx> {
            ctx: self.ctx,
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
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            cmds: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_command(mut self, cmd: Command<Ctx>) -> Self {
        self.cmds.push(cmd);
        self
    }
}

type Callback<Ctx> = Box<dyn Fn(&Repl<Ctx>, Option<&Ctx>, Args)>;

pub struct Command<Ctx> {
    name: String,
    params: Vec<Parameter>,
    cb: Callback<Ctx>,
}

impl<Ctx> Command<Ctx> {
    pub fn new<Cb>(name: &str, cb: Cb) -> Self
    where
        Cb: Fn(&Repl<Ctx>, Option<&Ctx>, Args) + 'static,
    {
        Self {
            name: name.into(),
            params: Vec::new(),
            cb: Box::new(cb),
        }
    }

    #[must_use]
    pub fn with_parameter(mut self, param: Parameter) -> Self {
        self.params.push(param);
        self
    }
}

pub enum Parameter {
    String(String),
}

impl Parameter {
    #[must_use]
    pub fn string(name: &str) -> Self {
        Self::String(name.into())
    }
}

pub struct Args;
