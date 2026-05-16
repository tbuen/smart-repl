use crate::Args;
use crate::Selection;
use crate::tokenizer::TokenList;

#[derive(Debug)]
pub(crate) enum ParseResult {
    Success,
    MissingCommand,
    InvalidCommand,
    MissingParameter,
    InvalidParameter,
    ExtraToken,
}

pub(crate) fn parse(sel: &Selection, mut tokens: TokenList) -> (ParseResult, Vec<String>, Args) {
    let mut sel = sel;
    let mut result = ParseResult::Success;
    let mut commands = Vec::new();
    let mut args = Args::new();

    while matches!(result, ParseResult::Success) {
        match sel {
            Selection::Fixed(fixed) => match tokens.pop_front() {
                Some(token) if !token.quoted => {
                    if let Some((_, s)) = fixed.iter().find(|(n, _)| n == &token.text) {
                        commands.push(token.text);
                        sel = s;
                    } else {
                        result = ParseResult::InvalidCommand;
                    }
                }
                Some(_) => result = ParseResult::InvalidCommand,
                None => result = ParseResult::MissingCommand,
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
                None => result = ParseResult::MissingParameter,
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
                        result = ParseResult::InvalidParameter;
                    }
                }
                Some(_) => result = ParseResult::InvalidParameter,
                None if *optional => {
                    args.add_alt(name.to_owned(), None);
                    sel = next;
                }
                None => result = ParseResult::MissingParameter,
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
                        result = ParseResult::InvalidParameter;
                    }
                }
                Some(_) => result = ParseResult::InvalidParameter,
                None if *optional => {
                    args.add_bool(name.to_owned(), None);
                    sel = next;
                }
                None => result = ParseResult::MissingParameter,
            },
            Selection::End => {
                if tokens.is_empty() {
                    break;
                }
                result = ParseResult::ExtraToken;
            }
        }
    }
    (result, commands, args)
}
