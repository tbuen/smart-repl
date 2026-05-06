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

    #[allow(clippy::too_many_lines)]
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
                    /*if let Some((_, s)) = fixed.iter().find(|(n, _)| n == &token.text) {
                        commands.push(token.text);
                        sel = s;
                    } else {*/
                    Selection::Fixed(fixed) => match tokens.pop_front() {
                        Some(token) if !token.quoted => {
                            if let Some((_, s)) = fixed.iter().find(|(n, _)| n == &token.text)
                                && line.len() > token.end
                            {
                                sel = s;
                            } else {
                                for (n, _) in
                                    fixed.iter().filter(|(n, _)| n.starts_with(&token.text))
                                {
                                    pairs.push(Pair {
                                        display: n.clone(),
                                        replacement: n.clone() + " ",
                                    });
                                }
                                rpos = token.begin;
                                break;
                            }
                        }
                        Some(_) => break,
                        None => {
                            for (n, _) in fixed {
                                pairs.push(Pair {
                                    display: n.clone(),
                                    replacement: n.clone() + " ",
                                });
                            }
                            rpos = pos;
                            break;
                        }
                    },
                    Selection::String {
                        name,
                        optional,
                        next,
                    } => match tokens.pop_front() {
                        Some(token) if line.len() > token.end => {
                            sel = next;
                        }
                        Some(_) => break,
                        None => {
                            if *optional {
                                pairs.push(Pair {
                                    display: format!("[<{name}>]"),
                                    replacement: format!("[<{name}>] "),
                                });
                            } else {
                                pairs.push(Pair {
                                    display: format!("<{name}>"),
                                    replacement: format!("<{name}> "),
                                });
                            }
                            rpos = pos;
                            break;
                        }
                    },
                    Selection::Alt {
                        name: _,
                        optional,
                        values,
                        next,
                    } => match tokens.pop_front() {
                        Some(token) if !token.quoted => {
                            if values.contains(&token.text) && line.len() > token.end {
                                sel = next;
                            } else {
                                for str in values.iter().filter(|v| v.starts_with(&token.text)) {
                                    if *optional {
                                        pairs.push(Pair {
                                            display: format!("[{}]", str.clone()),
                                            replacement: str.clone() + " ",
                                        });
                                    } else {
                                        pairs.push(Pair {
                                            display: str.clone(),
                                            replacement: str.clone() + " ",
                                        });
                                    }
                                }
                                rpos = token.begin;
                                break;
                            }
                        }
                        Some(_) => break,
                        None => {
                            for str in values {
                                if *optional {
                                    pairs.push(Pair {
                                        display: format!("[{}]", str.clone()),
                                        replacement: str.clone() + " ",
                                    });
                                } else {
                                    pairs.push(Pair {
                                        display: str.clone(),
                                        replacement: str.clone() + " ",
                                    });
                                }
                            }
                            rpos = pos;
                            break;
                        }
                    },
                    Selection::Bool {
                        name: _,
                        optional,
                        map,
                        next,
                    } => match tokens.pop_front() {
                        Some(token) if !token.quoted => {
                            if map.values().any(|v| v == &token.text) && line.len() > token.end {
                                sel = next;
                            } else {
                                for str in map.values().filter(|v| v.starts_with(&token.text)) {
                                    if *optional {
                                        pairs.push(Pair {
                                            display: format!("[{}]", str.clone()),
                                            replacement: str.clone() + " ",
                                        });
                                    } else {
                                        pairs.push(Pair {
                                            display: str.clone(),
                                            replacement: str.clone() + " ",
                                        });
                                    }
                                }
                                rpos = token.begin;
                                break;
                            }
                        }
                        Some(_) => break,
                        None => {
                            for str in map.values() {
                                if *optional {
                                    pairs.push(Pair {
                                        display: format!("[{}]", str.clone()),
                                        replacement: str.clone() + " ",
                                    });
                                } else {
                                    pairs.push(Pair {
                                        display: str.clone(),
                                        replacement: str.clone() + " ",
                                    });
                                }
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
