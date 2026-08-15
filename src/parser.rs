use std::error;
use std::fmt;

use crate::args::Args;
use crate::repl::Selection;
use crate::tokenizer::TokenList;

#[derive(Debug, Clone)]
pub(crate) enum Error {
    MissingCommand,
    InvalidCommand,
    MissingParameter,
    InvalidParameter,
    ExtraToken,
}

#[derive(Debug, Default)]
pub(crate) struct ParseConfig {
    accept_missing_command: bool,
}

pub(crate) fn parse(
    cfg: &ParseConfig,
    sel: &Selection,
    mut tokens: TokenList,
) -> Result<(Vec<String>, Args), Error> {
    let mut sel = sel;
    let mut commands = Vec::new();
    let mut args = Args::new();

    loop {
        match sel {
            Selection::Fixed(fixed) => match tokens.pop_front() {
                Some(token) if !token.quoted => {
                    if let Some((_, s)) = fixed.iter().find(|(n, _)| n == &token.text) {
                        commands.push(token.text);
                        sel = s;
                    } else {
                        return Err(Error::InvalidCommand);
                    }
                }
                Some(_) => return Err(Error::InvalidCommand),
                None if cfg.accept_missing_command => break,
                None => return Err(Error::MissingCommand),
            },
            Selection::String {
                name,
                optional,
                next,
            } => match tokens.pop_front() {
                Some(token) => {
                    args.add_string(name.to_owned(), Some(token.text));
                    sel = next;
                }
                None if *optional => {
                    args.add_string(name.to_owned(), None);
                    sel = next;
                }
                None => return Err(Error::MissingParameter),
            },
            Selection::Alt {
                name,
                optional,
                values,
                next,
            } => match tokens.pop_front() {
                Some(token) if !token.quoted => {
                    if values.iter().find(|s| *s == &token.text).is_some() {
                        args.add_alt(name.to_owned(), Some(token.text));
                        sel = next;
                    } else {
                        return Err(Error::InvalidParameter);
                    }
                }
                Some(_) => return Err(Error::InvalidParameter),
                None if *optional => {
                    args.add_alt(name.to_owned(), None);
                    sel = next;
                }
                None => return Err(Error::MissingParameter),
            },
            Selection::Bool {
                name,
                optional,
                values,
                next,
            } => match tokens.pop_front() {
                Some(token) if !token.quoted => {
                    let (t, f) = values;
                    if t == &token.text {
                        args.add_bool(name.to_owned(), Some(true));
                        sel = next;
                    } else if f == &token.text {
                        args.add_bool(name.to_owned(), Some(false));
                        sel = next;
                    } else {
                        return Err(Error::InvalidParameter);
                    }
                }
                Some(_) => return Err(Error::InvalidParameter),
                None if *optional => {
                    args.add_bool(name.to_owned(), None);
                    sel = next;
                }
                None => return Err(Error::MissingParameter),
            },
            Selection::End => {
                if tokens.is_empty() {
                    break;
                }
                return Err(Error::ExtraToken);
            }
        }
    }
    Ok((commands, args))
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingCommand => write!(f, "missing command"),
            Error::InvalidCommand => write!(f, "invalid command"),
            Error::MissingParameter => write!(f, "missing parameter"),
            Error::InvalidParameter => write!(f, "invalid parameter"),
            Error::ExtraToken => write!(f, "too many parameters"),
        }
    }
}

impl ParseConfig {
    pub(crate) fn for_help() -> Self {
        Self {
            accept_missing_command: true,
        }
    }
}
