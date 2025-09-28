use log::trace;
use readline::{ReadError, Reader};
use std::collections::HashMap;
use std::rc::Rc;

mod parser;
mod readline;
mod tokenizer;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("VERSION");

type CbMap<Ctx> = HashMap<Vec<String>, Callback<Ctx>>;

#[derive(Debug)]
enum Selection {
    Fixed(HashMap<String, Selection>),
    String((String, Box<Selection>)),
    Bool((HashMap<bool, String>, Box<Selection>)),
    End,
}

pub struct Repl<'a, Ctx> {
    ctx: Option<&'a Ctx>,
    reader: Reader,
    parse_tree: Rc<Selection>,
    cb_map: CbMap<Ctx>,
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
                    let (cmd, args) = parser::parse(&self.parse_tree, &tokens);
                    if let Some(cb) = self.cb_map.get(&cmd) {
                        (cb)(self, self.ctx, args);
                    }
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
                Err(ReadError::InvalidInput) => eprintln!("Invalid input"),
                Err(ReadError::Io(e)) => {
                    eprintln!("{e}");
                    break;
                }
                Err(ReadError::Eof | ReadError::Interrupted) => {
                    println!("Bye");
                    break;
                }
            }
        }
    }

    pub fn help(&self) {
        /*
        println!("COMMANDS");
        for c in self
            .tree
            .iter()
            .filter(|i| matches!(i.typ, ItemType::Command))
        {
            println!("    {}", c.name);
        }
        println!("GROUPS");
        for g in self
            .tree
            .iter()
            .filter(|i| matches!(i.typ, ItemType::Group))
        {
            for c in &g.children {
                println!("    {} {}", g.name, c.name);
            }
        }
        */
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

    fn build_parse_tree(grps: Vec<Group<Ctx>>, cmds: Vec<Command<Ctx>>) -> (Selection, CbMap<Ctx>) {
        let mut map = HashMap::new();
        let mut cbs = HashMap::new();
        for g in grps {
            let (s, c) = Self::build_parse_tree(g.grps, g.cmds);
            for (path, cb) in c {
                let mut p = vec![g.name.clone()];
                p.extend(path);
                cbs.insert(p, cb);
            }
            map.insert(g.name, s);
        }
        for mut c in cmds {
            let mut s = Selection::End;
            while let Some(p) = c.params.pop() {
                s = match p {
                    Parameter::String(n) => Selection::String((n, Box::new(s))),
                    Parameter::Bool(t, f) => {
                        let mut map = HashMap::new();
                        map.insert(true, t);
                        map.insert(false, f);
                        Selection::Bool((map, Box::new(s)))
                    }
                };
            }
            cbs.insert(vec![c.name.clone()], c.cb);
            map.insert(c.name, s);
        }
        if map.is_empty() {
            (Selection::End, cbs)
        } else {
            (Selection::Fixed(map), cbs)
        }
    }

    #[must_use]
    pub fn build(self) -> Repl<'a, Ctx> {
        let (parse_tree, cb_map) = Self::build_parse_tree(self.grps, self.cmds);

        trace!("{parse_tree:?}");
        let parse_tree = Rc::new(parse_tree);
        Repl::<Ctx> {
            ctx: self.ctx,
            reader: Reader::new(&self.prompt, parse_tree.clone()),
            parse_tree,
            cb_map,
        }
    }
}

pub struct Group<Ctx> {
    name: String,
    grps: Vec<Group<Ctx>>,
    cmds: Vec<Command<Ctx>>,
}

impl<Ctx> Group<Ctx> {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            grps: Vec::new(),
            cmds: Vec::new(),
        }
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
    Bool(String, String),
}

impl Parameter {
    #[must_use]
    pub fn string(name: &str) -> Self {
        Self::String(name.into())
    }

    #[must_use]
    pub fn bool(true_name: &str, false_name: &str) -> Self {
        Self::Bool(true_name.into(), false_name.into())
    }
}

pub struct Args;
