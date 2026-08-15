#![allow(clippy::print_stdout)]
use std::cmp;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use colored::Colorize as _;
use log::trace;

use crate::args::Args;
use crate::parser;
use crate::parser::ParseConfig;
use crate::readline;
use crate::readline::Reader;

const HELP_CMD: &str = "help";

pub struct Repl<'a, Ctx> {
    ctx: Option<&'a Ctx>,
    reader: Reader,
    help: Option<HelpList>,
    parse_tree: Rc<Selection>,
    cb_map: CbMap<Ctx>,
}

pub struct ReplBuilder<'a, Ctx> {
    ctx: Option<&'a Ctx>,
    prompt: String,
    help: bool,
    grps: Vec<Group<Ctx>>,
    cmds: Vec<Command<Ctx>>,
}

pub struct Group<Ctx> {
    name: String,
    help: Option<String>,
    grps: Vec<Group<Ctx>>,
    cmds: Vec<Command<Ctx>>,
}

pub struct Command<Ctx> {
    name: String,
    help: Option<String>,
    params: Vec<Parameter>,
    cb: Callback<Ctx>,
}

pub struct Parameter {
    ptype: ParamType,
    name: String,
    optional: bool,
}

pub(crate) enum Selection {
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
        values: (String, String),
        next: Box<Selection>,
    },
    End,
}

enum ParamType {
    String,
    Alt(Vec<String>),
    Bool(String, String),
}

enum HelpList {
    GrpCmd(Vec<HelpItem>),
    Param(Vec<HelpItem>),
}

struct HelpItem {
    name: String,
    help: Option<String>,
    next: Option<HelpList>,
}

type Callback<Ctx> = Box<dyn Fn(Option<&Ctx>, Args)>;
type CbMap<Ctx> = HashMap<Vec<String>, Callback<Ctx>>;

impl<'a, Ctx> Repl<'a, Ctx> {
    pub fn builder() -> ReplBuilder<'a, Ctx> {
        ReplBuilder::new()
    }

    pub fn run(&mut self) {
        loop {
            match self.reader.read_line() {
                Ok(tokens) if tokens.is_empty() => (),
                Ok(tokens) => {
                    trace!("{tokens:?}");

                    if self.help.is_some()
                        && let Some(t) = tokens.front()
                        && t.text == HELP_CMD
                    {
                        match parser::parse(&ParseConfig::for_help(), &self.parse_tree, tokens) {
                            Ok((cmds, _)) => {
                                self.display_help(&cmds[1..]);
                            }
                            Err(e) => println!("{}", e.to_string().bold()),
                        }
                    } else {
                        match parser::parse(&ParseConfig::default(), &self.parse_tree, tokens) {
                            Ok((cmds, args)) => {
                                if let Some(cb) = self.cb_map.get(&cmds) {
                                    (cb)(self.ctx, args);
                                }
                            }
                            Err(e) => println!("{}", e.to_string().bold()),
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
                e @ Err(readline::Error::Readline(_)) => {
                    println!("{}", e.unwrap_err().to_string().bold());
                    break;
                }
                Err(readline::Error::Eof | readline::Error::Interrupted) => {
                    println!("Bye");
                    break;
                }
                Err(e) => println!("{}", e.to_string().bold()),
            }
        }
    }

    fn display_help(&self, tokens: &[String]) {
        let mut help = self.help.as_ref();
        let mut hierarchy = Vec::new();
        for t in tokens {
            if let Some(HelpList::GrpCmd(l)) = help {
                for h in l {
                    if t == &h.name {
                        hierarchy.push(h);
                        help = h.next.as_ref();
                    }
                }
            }
        }

        let hierarchy_str = hierarchy
            .iter()
            .fold(String::new(), |s, h| s + &h.name + " ");

        match help {
            Some(HelpList::GrpCmd(l)) => {
                let max = l.iter().fold(0, |m, x| cmp::max(m, x.name.len()));
                for i in l {
                    let mut x = "   ";
                    if i.next.is_some() {
                        x = "[…]";
                    }
                    if let Some(h) = &i.help {
                        println!(
                            "{}{:<w$} {x}    {}",
                            hierarchy_str.bold(),
                            i.name.bold(),
                            h,
                            w = max
                        );
                    } else {
                        println!("{}{} {x}", hierarchy_str.bold(), i.name.bold());
                    }
                }
            }
            Some(HelpList::Param(l)) => {
                if let Some(p) = hierarchy.last() {
                    let param_str = l.iter().fold(String::new(), |s, h| s + &h.name + " ");
                    if let Some(h) = &p.help {
                        println!("{}{}   {h}", hierarchy_str.bold(), param_str.bold());
                    } else {
                        println!("{}{}", hierarchy_str.bold(), param_str.bold());
                    }
                }
            }
            None => {
                if let Some(p) = hierarchy.last() {
                    if let Some(h) = &p.help {
                        println!("{}   {h}", hierarchy_str.bold());
                    } else {
                        println!("{}", hierarchy_str.bold());
                    }
                }
            }
        }
    }
}

impl<'a, Ctx> ReplBuilder<'a, Ctx> {
    const DEFAULT_PROMPT: &'static str = ">";

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

    pub fn build(self) -> Repl<'a, Ctx> {
        let mut parse_tree;
        let mut help = None;
        let cb_map;
        if self.help {
            let (help_tree, help_items) = Self::build_help_tree(&self.grps, &self.cmds);
            (parse_tree, cb_map) = Self::build_parse_tree(self.grps, self.cmds);
            if let Selection::Fixed(ref mut fixed) = parse_tree {
                fixed.push_front((HELP_CMD.into(), help_tree));
            }
            help = Some(help_items);
        } else {
            (parse_tree, cb_map) = Self::build_parse_tree(self.grps, self.cmds);
        }

        let rc_parse_tree = Rc::new(parse_tree);
        Repl::<Ctx> {
            ctx: self.ctx,
            reader: Reader::new(&self.prompt, Rc::clone(&rc_parse_tree)),
            help,
            parse_tree: rc_parse_tree,
            cb_map,
        }
    }

    fn new() -> Self {
        Self {
            ctx: None,
            prompt: Self::DEFAULT_PROMPT.into(),
            help: false,
            grps: Vec::new(),
            cmds: Vec::new(),
        }
    }

    fn build_help_tree(grps: &Vec<Group<Ctx>>, cmds: &Vec<Command<Ctx>>) -> (Selection, HelpList) {
        let mut fixed = VecDeque::new();
        let mut items = Vec::new();
        for c in cmds {
            let mut params = None;
            let mut param_vec = Vec::new();
            for p in &c.params {
                param_vec.push({
                    let name = match &p.ptype {
                        ParamType::String => {
                            if p.optional {
                                format!("['{}']", p.name)
                            } else {
                                format!("'{}'", p.name)
                            }
                        }
                        ParamType::Alt(_) => p.name.clone(),
                        ParamType::Bool(t, f) => {
                            if p.optional {
                                format!("[{t}|{f}]")
                            } else {
                                format!("{t}|{f}")
                            }
                        }
                    };
                    HelpItem {
                        name,
                        help: None,
                        next: None,
                    }
                });
            }
            if !param_vec.is_empty() {
                params = Some(HelpList::Param(param_vec));
            }
            items.push(HelpItem {
                name: c.name.clone(),
                help: c.help.clone(),
                next: params,
            });
            fixed.push_back((c.name.clone(), Selection::End));
        }
        for g in grps {
            let (s, h) = Self::build_help_tree(&g.grps, &g.cmds);
            items.push(HelpItem {
                name: g.name.clone(),
                help: g.help.clone(),
                next: Some(h),
            });
            fixed.push_back((g.name.clone(), s));
        }
        if fixed.is_empty() {
            (Selection::End, HelpList::GrpCmd(items))
        } else {
            (Selection::Fixed(fixed), HelpList::GrpCmd(items))
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
                    } => Selection::Bool {
                        name,
                        optional,
                        values: (t, f),
                        next: Box::new(s),
                    },
                };
            }
            cbs.insert(vec![c.name.clone()], c.cb);
            fixed.push_back((c.name, s));
        }
        for g in grps {
            let (s, c) = Self::build_parse_tree(g.grps, g.cmds);
            #[expect(clippy::iter_over_hash_type)]
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
}

impl<Ctx> Group<Ctx> {
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

impl Parameter {
    pub fn string(name: &str) -> Self {
        Self {
            ptype: ParamType::String,
            name: name.into(),
            optional: false,
        }
    }

    pub fn alt(name: &str, values: Vec<String>) -> Self {
        Self {
            ptype: ParamType::Alt(values),
            name: name.into(),
            optional: false,
        }
    }

    pub fn bool(name: &str, true_name: &str, false_name: &str) -> Self {
        Self {
            ptype: ParamType::Bool(true_name.into(), false_name.into()),
            name: name.into(),
            optional: false,
        }
    }
}
