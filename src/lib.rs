use readline::Reader;

mod readline;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("VERSION");

pub struct Repl<Ctx> {
    reader: Reader,
    _grps: Vec<Group<Ctx>>,
    _cmds: Vec<Command<Ctx>>,
}

impl<Ctx> Repl<Ctx> {
    pub fn builder() -> ReplBuilder<Ctx> {
        ReplBuilder::<Ctx> {
            grps: Vec::new(),
            cmds: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            while let Ok(_list) = self.reader.read_line() {
                //match self.reader.read_line() {
                //    Ok(_list) => {
                println!("Line read");
                //    }
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
                //Err(_) => break,
            }
        }
    }
}

pub struct ReplBuilder<Ctx> {
    grps: Vec<Group<Ctx>>,
    cmds: Vec<Command<Ctx>>,
}

impl<Ctx> ReplBuilder<Ctx> {
    pub fn with_group(self, _grp: Group<Ctx>) -> ReplBuilder<Ctx> {
        self
    }

    pub fn with_command(self, _cmd: Command<Ctx>) -> ReplBuilder<Ctx> {
        self
    }

    pub fn build(self) -> Repl<Ctx> {
        Repl::<Ctx> {
            reader: Reader::new(),
            _grps: self.grps,
            _cmds: self.cmds,
        }
    }
}

pub struct Group<Ctx> {
    _name: String,
    cmds: Vec<Command<Ctx>>,
}

impl<Ctx> Group<Ctx> {
    pub fn new(name: &str) -> Self {
        Self {
            _name: name.into(),
            cmds: Vec::new(),
        }
    }

    pub fn with_command(mut self, cmd: Command<Ctx>) -> Self {
        self.cmds.push(cmd);
        self
    }
}

pub struct Command<Ctx> {
    _name: String,
    params: Vec<Parameter>,
    _cb: Box<dyn FnOnce(Ctx, Args)>,
}

impl<Ctx> Command<Ctx> {
    pub fn new<Cb>(name: &str, cb: Cb) -> Self
    where
        Cb: FnOnce(Ctx, Args) + 'static,
    {
        Self {
            _name: name.into(),
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
