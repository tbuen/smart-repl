pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("VERSION");

pub struct Repl;

impl Repl {
    pub fn builder() -> ReplBuilder {
        ReplBuilder
    }

    pub fn run(&self) {}
}

pub struct ReplBuilder;

impl ReplBuilder {
    pub fn with_command(self, _cmd: Command) -> ReplBuilder {
        self
    }

    pub fn with_group(self, _grp: Group) -> ReplBuilder {
        self
    }

    pub fn build(self) -> Repl {
        Repl
    }
}

pub struct Command {
    _name: String,
}

impl Command {
    pub fn new(name: &str) -> Self {
        Self { _name: name.into() }
    }
}

pub struct Group {
    _name: String,
}

impl Group {
    pub fn new(name: &str) -> Self {
        Self { _name: name.into() }
    }

    pub fn with_command(self, _cmd: Command) -> Self {
        self
    }
}
