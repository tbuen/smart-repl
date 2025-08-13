use super::readline::TokenList;
use super::{Args, Command, Group, Repl};

pub fn parse<Ctx>(
    repl: &Repl<Ctx>,
    ctx: Option<&Ctx>,
    grps: &[Group<Ctx>],
    cmds: &[Command<Ctx>],
    tokens: &TokenList,
) {
    let mut cmd: Option<&Command<Ctx>> = None;
    let mut grp: Option<&Group<Ctx>> = None;
    for token in tokens {
        if let Some(c) = cmd {
            println!("command found: {}", c.name);
            break;
        } else if let Some(g) = grp {
            println!("group found: {}", g.name);
            if let Some(c) = g.cmds.iter().find(|c| c.name == token.text) {
                cmd = Some(c);
            } else {
                eprintln!("No such command: {}", token.text);
                break;
            }
        } else if let Some(g) = grps.iter().find(|g| g.name == token.text) {
            grp = Some(g);
        } else if let Some(c) = cmds.iter().find(|c| c.name == token.text) {
            cmd = Some(c);
        } else {
            eprintln!("No such command or group: {}", token.text);
            break;
        }
    }
    if let Some(c) = cmd {
        (c.cb)(repl, ctx, Args {});
    }
}
