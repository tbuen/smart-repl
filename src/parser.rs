use crate::Args;
use crate::Selection;
use crate::tokenizer::TokenList;

#[derive(Debug)]
pub enum ParseResult {
    Success,
    MissingCommand,
    InvalidCommand,
    MissingParameter,
    InvalidParameter,
    ExtraToken,
}

pub fn parse(sel: &Selection, mut tokens: TokenList) -> (ParseResult, Vec<String>, Args) {
    let mut sel = sel;
    let mut result = ParseResult::Success;
    let mut commands = Vec::new();

    while matches!(result, ParseResult::Success) {
        match sel {
            Selection::Fixed(map) => match tokens.pop_front() {
                Some(token) if !token.quoted => {
                    if let Some(s) = map.get(&token.text) {
                        commands.push(token.text);
                        sel = s;
                    } else {
                        result = ParseResult::InvalidCommand;
                    }
                }
                Some(_) => result = ParseResult::InvalidCommand,
                None => result = ParseResult::MissingCommand,
            },
            Selection::String((_str, s)) => match tokens.pop_front() {
                Some(_token) => {
                    // push token.text to args
                    sel = s;
                }
                None => result = ParseResult::MissingParameter,
            },
            Selection::Bool((map, s)) => match tokens.pop_front() {
                Some(token) if !token.quoted => {
                    if map.values().any(|v| v == &token.text) {
                        // push bool to args
                        sel = s;
                    } else {
                        result = ParseResult::InvalidParameter;
                    }
                }
                Some(_) => result = ParseResult::InvalidParameter,
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
    (result, commands, Args {})
}
