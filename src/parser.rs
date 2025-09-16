use crate::tokenizer::TokenList;
use crate::{Args, Callback, Repl};

pub type Tree<Ctx> = Vec<Item<Ctx>>;

pub struct Item<Ctx> {
    pub name: String,
    pub typ: ItemType<Ctx>,
    pub children: Tree<Ctx>,
}

pub enum ItemType<Ctx> {
    Group,
    Command(Callback<Ctx>),
    //Param(ParamType),
}

/*enum ParamType {
    Bool,
    String,
}*/

pub fn parse<Ctx>(repl: &Repl<Ctx>, ctx: Option<&Ctx>, tree: &Tree<Ctx>, tokens: &TokenList) {
    //let mut cmd: Option<&Command<Ctx>> = None;
    //let mut grp: Option<&Group<Ctx>> = None;
    //let mut item: Option<&ParseTreeItem<Ctx>> = None;
    let mut tree = tree;
    let mut cb = None;
    for token in tokens {
        if let Some(i) = tree.iter().find(|i| i.name == token.text) {
            match &i.typ {
                ItemType::Group => tree = &i.children,
                ItemType::Command(c) => {
                    cb = Some(c);
                    break;
                } //ItemType::Param(_) => todo!(),
            }
        } else {
            eprintln!("*** not found: {}", token.text);
            break;
        }
    }
    if let Some(c) = cb {
        (c)(repl, ctx, Args {});
    }
}
