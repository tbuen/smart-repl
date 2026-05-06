use colored::Colorize;
use log::trace;
use parser::ParseResult;
use readline::{ReadError, Reader};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

mod parser;
mod readline;
mod tokenizer;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("VERSION");

type CbMap<Ctx> = HashMap<Vec<String>, Callback<Ctx>>;

struct HelpText {
    cmds: Vec<String>,
    params: Option<Vec<(String, Option<String>)>>,
    text: Option<String>,
}
type HelpTexts = Vec<HelpText>;

#[derive(Debug)]
enum Selection {
    Fixed(VecDeque<(String, Selection)>),
    String {
        name: String,
        optional: bool,
        next: Box<Selection>,
    },
    Alt {
        name: String,
        optional: bool,
        values: Vec<String>,
        next: Box<Selection>,
    },
    Bool {
        name: String,
        optional: bool,
        map: HashMap<bool, String>,
        next: Box<Selection>,
    },
    End,
}

pub struct Repl<'a, Ctx> {
    ctx: Option<&'a Ctx>,
    reader: Reader,
    help: Option<String>,
    parse_tree: Rc<Selection>,
    cb_map: CbMap<Ctx>,
    help_texts: HelpTexts,
}

impl<'a, Ctx> Repl<'a, Ctx> {
    #[must_use]
    pub fn builder() -> ReplBuilder<'a, Ctx> {
        ReplBuilder::new()
    }

    pub fn run(&mut self) {
        loop {
            match self.reader.read_line() {
                Ok(tokens) if tokens.is_empty() => {}
                Ok(tokens) => {
                    trace!("{tokens:?}");
                    let (result, cmds, args) = parser::parse(&self.parse_tree, tokens);
                    if let Some(h) = &self.help
                        && let Some(c) = cmds.first()
                        && c == h
                    {
                        match result {
                            ParseResult::Success | ParseResult::MissingCommand => {
                                self.help(&cmds[1..]);
                            }
                            _ => println!("{}", "Unknown command.".red().bold()),
                        }
                    } else {
                        match result {
                            ParseResult::Success => {
                                if let Some(cb) = self.cb_map.get(&cmds) {
                                    (cb)(self.ctx, args);
                                }
                            }
                            ParseResult::MissingCommand => {
                                println!("{}", "Missing command.".red().bold());
                            }
                            ParseResult::InvalidCommand => {
                                println!("{}", "Invalid command.".red().bold());
                            }
                            ParseResult::MissingParameter => {
                                println!("{}", "Missing parameter.".red().bold());
                            }
                            ParseResult::InvalidParameter => {
                                println!("{}", "Invalid parameter.".red().bold());
                            }
                            ParseResult::ExtraToken => {
                                println!("{}", "Too many parameters.".red().bold());
                            }
                        }
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
                Err(ReadError::InvalidInput) => println!("{}", "Invalid input.".red().bold()),
                Err(ReadError::Io(e)) => {
                    println!("{e}");
                    break;
                }
                Err(ReadError::Eof | ReadError::Interrupted) => {
                    println!("Bye");
                    break;
                }
            }
        }
    }

    fn help(&self, tokens: &[String]) {
        let mut last: Option<&Vec<String>> = None;
        for HelpText { cmds, params, text } in &self.help_texts {
            if cmds.starts_with(tokens) || tokens.starts_with(cmds) {
                let mut start = 0;
                if let Some(l) = last {
                    while let Some(a) = cmds.get(start)
                        && let Some(b) = l.get(start)
                        && a == b
                    {
                        start += 1;
                    }
                    if start == 0 {
                        println!();
                    }
                }
                for _ in 0..start {
                    print!("  ");
                }
                print!("{}\t\t", cmds[start].bold());
                match text {
                    Some(s) => println!("{s}"),
                    None => println!(),
                }
                if let Some(ps) = params {
                    for p in ps {
                        for _ in 0..=start {
                            print!("  ");
                        }
                        print!("{}\t\t", p.0);
                        match &p.1 {
                            Some(s) => println!("{s}"),
                            None => println!(),
                        }
                    }
                }
                last = Some(cmds);
            }
        }
    }
}

pub struct ReplBuilder<'a, Ctx> {
    ctx: Option<&'a Ctx>,
    prompt: String,
    help: bool,
    grps: Vec<Group<Ctx>>,
    cmds: Vec<Command<Ctx>>,
}

impl<'a, Ctx> ReplBuilder<'a, Ctx> {
    const DEFAULT_PROMPT: &'static str = ">";

    fn new() -> Self {
        Self {
            ctx: None,
            prompt: ReplBuilder::<Ctx>::DEFAULT_PROMPT.into(),
            help: false,
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
    pub fn with_help(mut self) -> Self {
        self.help = true;
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

    fn build_help_tree(grps: &Vec<Group<Ctx>>, cmds: &Vec<Command<Ctx>>) -> (Selection, HelpTexts) {
        let mut fixed = VecDeque::new();
        let mut texts = Vec::new();
        for c in cmds {
            let mut params = None;
            let mut param_vec = Vec::new();
            for p in &c.params {
                param_vec.push(match &p.ptype {
                    ParamType::String => {
                        if p.optional {
                            (format!("[<{}>]", p.name), None)
                        } else {
                            (format!("<{}>", p.name), None)
                        }
                    }
                    ParamType::Alt(_) => (p.name.clone(), None),
                    ParamType::Bool(t, f) => {
                        if p.optional {
                            (format!("[{t}|{f}]"), None)
                        } else {
                            (format!("{t}|{f}"), None)
                        }
                    }
                });
            }
            if !param_vec.is_empty() {
                params = Some(param_vec);
            }
            texts.push(HelpText {
                cmds: vec![c.name.clone()],
                params,
                text: c.help.clone(),
            });
            fixed.push_back((c.name.clone(), Selection::End));
        }
        for g in grps {
            let (s, h) = Self::build_help_tree(&g.grps, &g.cmds);
            texts.push(HelpText {
                cmds: vec![g.name.clone()],
                params: None,
                text: g.help.clone(),
            });
            for HelpText { cmds, params, text } in h {
                let mut p = vec![g.name.clone()];
                p.extend(cmds);
                texts.push(HelpText {
                    cmds: p,
                    params,
                    text,
                });
            }
            fixed.push_back((g.name.clone(), s));
        }
        if fixed.is_empty() {
            (Selection::End, texts)
        } else {
            (Selection::Fixed(fixed), texts)
        }
    }

    fn build_parse_tree(grps: Vec<Group<Ctx>>, cmds: Vec<Command<Ctx>>) -> (Selection, CbMap<Ctx>) {
        let mut fixed = VecDeque::new();
        let mut cbs = HashMap::new();
        for mut c in cmds {
            let mut s = Selection::End;
            while let Some(p) = c.params.pop() {
                s = match p {
                    Parameter {
                        ptype: ParamType::String,
                        name,
                        optional,
                    } => Selection::String {
                        name,
                        optional,
                        next: Box::new(s),
                    },
                    Parameter {
                        ptype: ParamType::Alt(v),
                        name,
                        optional,
                    } => Selection::Alt {
                        name,
                        optional,
                        values: v,
                        next: Box::new(s),
                    },
                    Parameter {
                        ptype: ParamType::Bool(t, f),
                        name,
                        optional,
                    } => {
                        let mut map = HashMap::new();
                        map.insert(true, t);
                        map.insert(false, f);
                        Selection::Bool {
                            name,
                            optional,
                            map,
                            next: Box::new(s),
                        }
                    }
                };
            }
            cbs.insert(vec![c.name.clone()], c.cb);
            fixed.push_back((c.name, s));
        }
        for g in grps {
            let (s, c) = Self::build_parse_tree(g.grps, g.cmds);
            for (path, cb) in c {
                let mut p = vec![g.name.clone()];
                p.extend(path);
                cbs.insert(p, cb);
            }
            fixed.push_back((g.name, s));
        }
        if fixed.is_empty() {
            (Selection::End, cbs)
        } else {
            (Selection::Fixed(fixed), cbs)
        }
    }

    #[must_use]
    pub fn build(self) -> Repl<'a, Ctx> {
        const HELP: &str = "help";
        let help;
        let mut parse_tree;
        let cb_map;
        let help_texts;
        if self.help {
            help = Some(HELP.into());
            let help_tree;
            (help_tree, help_texts) = Self::build_help_tree(&self.grps, &self.cmds);
            trace!("{help_tree:?}");
            (parse_tree, cb_map) = Self::build_parse_tree(self.grps, self.cmds);
            if let Selection::Fixed(ref mut fixed) = parse_tree {
                fixed.push_front((HELP.into(), help_tree));
            }
        } else {
            help = None;
            help_texts = Vec::new();
            (parse_tree, cb_map) = Self::build_parse_tree(self.grps, self.cmds);
        }

        trace!("{parse_tree:?}");
        let parse_tree = Rc::new(parse_tree);
        Repl::<Ctx> {
            ctx: self.ctx,
            reader: Reader::new(&self.prompt, parse_tree.clone()),
            help,
            parse_tree,
            cb_map,
            help_texts,
        }
    }
}

pub struct Group<Ctx> {
    name: String,
    help: Option<String>,
    grps: Vec<Group<Ctx>>,
    cmds: Vec<Command<Ctx>>,
}

impl<Ctx> Group<Ctx> {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            help: None,
            grps: Vec::new(),
            cmds: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_help(mut self, text: &str) -> Self {
        self.help = Some(text.into());
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
}

type Callback<Ctx> = Box<dyn Fn(Option<&Ctx>, Args)>;

pub struct Command<Ctx> {
    name: String,
    help: Option<String>,
    params: Vec<Parameter>,
    cb: Callback<Ctx>,
}

impl<Ctx> Command<Ctx> {
    pub fn new<Cb>(name: &str, cb: Cb) -> Self
    where
        Cb: Fn(Option<&Ctx>, Args) + 'static,
    {
        Self {
            name: name.into(),
            help: None,
            params: Vec::new(),
            cb: Box::new(cb),
        }
    }

    #[must_use]
    pub fn with_help(mut self, text: &str) -> Self {
        self.help = Some(text.into());
        self
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn with_parameter(mut self, param: Parameter) -> Self {
        if let Some(p) = self.params.last()
            && p.optional
        {
            panic!("A non-optional parameter is not allowed after an optional one.");
        }
        self.params.push(param);
        self
    }

    #[must_use]
    pub fn with_optional_parameter(mut self, param: Parameter) -> Self {
        let mut p = param;
        p.optional = true;
        self.params.push(p);
        self
    }
}

enum ParamType {
    String,
    Alt(Vec<String>),
    Bool(String, String),
}

pub struct Parameter {
    ptype: ParamType,
    name: String,
    optional: bool,
}

impl Parameter {
    #[must_use]
    pub fn string(name: &str) -> Self {
        Self {
            ptype: ParamType::String,
            name: name.into(),
            optional: false,
        }
    }

    #[must_use]
    pub fn alt(name: &str, values: Vec<String>) -> Self {
        Self {
            ptype: ParamType::Alt(values),
            name: name.into(),
            optional: false,
        }
    }

    #[must_use]
    pub fn bool(name: &str, true_name: &str, false_name: &str) -> Self {
        Self {
            ptype: ParamType::Bool(true_name.into(), false_name.into()),
            name: name.into(),
            optional: false,
        }
    }
}

#[derive(Debug)]
pub enum ArgError {
    NotAvailable,
}

pub struct Args {
    strings: HashMap<String, Option<String>>,
    alts: HashMap<String, Option<String>>,
    bools: HashMap<String, Option<bool>>,
}

impl Args {
    fn new() -> Self {
        Args {
            strings: HashMap::new(),
            alts: HashMap::new(),
            bools: HashMap::new(),
        }
    }

    fn add_string(&mut self, name: &str, val: Option<&str>) {
        self.strings
            .insert(name.into(), val.map(std::convert::Into::into));
    }

    fn add_alt(&mut self, name: &str, val: Option<&str>) {
        self.alts
            .insert(name.into(), val.map(std::convert::Into::into));
    }

    fn add_bool(&mut self, name: &str, val: Option<bool>) {
        self.bools.insert(name.into(), val);
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn get_string(&self, name: &str) -> Result<Option<String>, ArgError> {
        match self.strings.get(name) {
            Some(o) => Ok(o.clone()),
            None => Err(ArgError::NotAvailable),
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn get_alt(&self, name: &str) -> Result<Option<String>, ArgError> {
        match self.alts.get(name) {
            Some(o) => Ok(o.clone()),
            None => Err(ArgError::NotAvailable),
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn get_bool(&self, name: &str) -> Result<Option<bool>, ArgError> {
        match self.bools.get(name) {
            Some(o) => Ok(*o),
            None => Err(ArgError::NotAvailable),
        }
    }
}
