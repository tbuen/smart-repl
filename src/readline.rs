use crate::Selection;
use crate::tokenizer;
use crate::tokenizer::TokenList;
use rustyline::completion::{Completer, Pair};
use rustyline::config::Builder;
use rustyline::error::ReadlineError;
use rustyline::history::MemHistory;
use rustyline::{CompletionType, Context, Editor, Helper, Highlighter, Hinter, Validator};
use std::rc::Rc;

pub enum ReadError {
    InvalidInput,
    Eof,
    Interrupted,
    Io(String),
}

pub struct Reader {
    rusty: Editor<MyHelper, MemHistory>,
    prompt: String,
}

impl Reader {
    pub fn new(prompt: &str, parse_tree: Rc<Selection>) -> Self {
        let mut rusty =
            Editor::with_config(Builder::new().completion_type(CompletionType::List).build())
                .unwrap();
        rusty.set_helper(Some(MyHelper::new(parse_tree)));
        Self {
            rusty,
            prompt: prompt.into(),
        }
    }

    pub fn read_line(&mut self) -> Result<TokenList, ReadError> {
        match self.rusty.readline(&self.prompt) {
            Ok(line) => match tokenizer::tokenize(&line) {
                Ok(list) => Ok(list),
                Err(()) => Err(ReadError::InvalidInput),
            },
            Err(ReadlineError::Interrupted) => Err(ReadError::Interrupted),
            Err(ReadlineError::Eof) => Err(ReadError::Eof),
            Err(err) => Err(ReadError::Io(err.to_string())),
        }
    }
}

#[derive(Helper, Highlighter, Hinter, Validator)]
struct MyHelper {
    parse_tree: Rc<Selection>,
}

impl MyHelper {
    fn new(parse_tree: Rc<Selection>) -> Self {
        Self { parse_tree }
    }
}

impl Completer for MyHelper {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let line = &line[..pos];
        let mut pairs = vec![];
        let mut rpos = 0;

        if let Ok(mut tokens) = tokenizer::tokenize(line) {
            let mut sel = self.parse_tree.as_ref();

            loop {
                match sel {
                    Selection::Fixed(map) => match tokens.pop_front() {
                        Some(token) if !token.quoted => {
                            if let Some(s) = map.get(&token.text)
                                && line.len() > token.end
                            {
                                sel = s;
                            } else {
                                for str in map.keys().filter(|k| k.starts_with(&token.text)) {
                                    pairs.push(Pair {
                                        display: str.clone(),
                                        replacement: str.clone() + " ",
                                    });
                                }
                                rpos = token.begin;
                                break;
                            }
                        }
                        Some(_) => break,
                        None => {
                            for str in map.keys() {
                                pairs.push(Pair {
                                    display: str.clone(),
                                    replacement: str.clone() + " ",
                                });
                            }
                            rpos = pos;
                            break;
                        }
                    },
                    Selection::String((str, s)) => match tokens.pop_front() {
                        Some(token) if line.len() > token.end => {
                            sel = s;
                        }
                        Some(_) => break,
                        None => {
                            pairs.push(Pair {
                                display: format!("<{str}>"),
                                replacement: format!("<{str}> "),
                            });
                            rpos = pos;
                            break;
                        }
                    },
                    Selection::Bool((map, s)) => match tokens.pop_front() {
                        Some(token) if !token.quoted => {
                            if map.values().any(|v| v == &token.text) && line.len() > token.end {
                                sel = s;
                            } else {
                                for str in map.values().filter(|v| v.starts_with(&token.text)) {
                                    pairs.push(Pair {
                                        display: str.clone(),
                                        replacement: str.clone() + " ",
                                    });
                                }
                                rpos = token.begin;
                                break;
                            }
                        }
                        Some(_) => break,
                        None => {
                            for str in map.values() {
                                pairs.push(Pair {
                                    display: str.clone(),
                                    replacement: str.clone() + " ",
                                });
                            }
                            rpos = pos;
                            break;
                        }
                    },
                    Selection::End => break,
                }
            }
        }
        Ok((rpos, pairs))
    }
}
