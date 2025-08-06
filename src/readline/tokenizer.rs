use std::collections::VecDeque;

pub struct Tokenizer;

impl Tokenizer {
    pub fn new() -> Self {
        Self
    }

    pub fn tokenize(&self, line: &str) -> Result<TokenList, ()> {
        enum Status {
            Idle,
            Token,
            Quote(char),
        }

        let mut status = Status::Idle;
        let mut tokens = VecDeque::new();
        let mut token = String::new();
        let mut begin = 0;
        for (i, c) in line.char_indices() {
            match status {
                Status::Idle => {
                    if c == '"' || c == '\'' {
                        begin = i + 1;
                        status = Status::Quote(c);
                    } else if c != ' ' {
                        begin = i;
                        token.push(c);
                        status = Status::Token;
                    }
                }
                Status::Token => {
                    if c == '"' || c == '\'' {
                        return Err(());
                    } else if c == ' ' {
                        tokens.push_back(Token::new(&token, begin, i));
                        token.clear();
                        status = Status::Idle;
                    } else {
                        token.push(c);
                    }
                }
                Status::Quote(q) => {
                    if c == q {
                        tokens.push_back(Token::quoted(&token, begin, i));
                        token.clear();
                        status = Status::Idle;
                    } else {
                        token.push(c);
                    }
                }
            }
        }
        match status {
            Status::Token => tokens.push_back(Token::new(&token, begin, line.chars().count())),
            Status::Quote(_) => return Err(()),
            _ => {}
        }
        Ok(TokenList { tokens })
    }
}

#[derive(Debug, PartialEq)]
pub struct TokenList {
    tokens: VecDeque<Token>,
}

#[derive(Debug, PartialEq)]
struct Token {
    text: String,
    begin: usize,
    end: usize,
    quoted: bool,
}

impl Token {
    fn new(text: &str, begin: usize, end: usize) -> Self {
        Self {
            text: text.into(),
            begin,
            end,
            quoted: false,
        }
    }

    fn quoted(text: &str, begin: usize, end: usize) -> Self {
        Self {
            text: text.into(),
            begin,
            end,
            quoted: true,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenize() {
        let t = Tokenizer::new();
        assert_eq!(
            t.tokenize("a"),
            Ok(TokenList::from([Token::new("a", 0, 1)]))
        );
        assert_eq!(
            t.tokenize("    a    b    "),
            Ok(TokenList::from([
                Token::new("a", 4, 5),
                Token::new("b", 9, 10)
            ]))
        );
        assert_eq!(
            t.tokenize("hello"),
            Ok(TokenList::from([Token::new("hello", 0, 5)]))
        );
        assert_eq!(
            t.tokenize("  hello  "),
            Ok(TokenList::from([Token::new("hello", 2, 7)]))
        );
        assert_eq!(
            t.tokenize("hello world"),
            Ok(TokenList::from([
                Token::new("hello", 0, 5),
                Token::new("world", 6, 11)
            ]))
        );
        assert_eq!(
            t.tokenize("hello  world"),
            Ok(TokenList::from([
                Token::new("hello", 0, 5),
                Token::new("world", 7, 12)
            ]))
        );
        assert_eq!(
            t.tokenize(" hello world "),
            Ok(TokenList::from([
                Token::new("hello", 1, 6),
                Token::new("world", 7, 12)
            ]))
        );
        assert_eq!(
            t.tokenize("say hello world"),
            Ok(TokenList::from([
                Token::new("say", 0, 3),
                Token::new("hello", 4, 9),
                Token::new("world", 10, 15)
            ]))
        );
        assert_eq!(
            t.tokenize("say \"hello world\""),
            Ok(TokenList::from([
                Token::new("say", 0, 3),
                Token::quoted("hello world", 5, 16)
            ]))
        );
        assert_eq!(
            t.tokenize("  say \"hello world\"  "),
            Ok(TokenList::from([
                Token::new("say", 2, 5),
                Token::quoted("hello world", 7, 18)
            ]))
        );
        assert_eq!(
            t.tokenize("say \"hello world\" twice"),
            Ok(TokenList::from([
                Token::new("say", 0, 3),
                Token::quoted("hello world", 5, 16),
                Token::new("twice", 18, 23)
            ]))
        );
        assert_eq!(
            t.tokenize("say \"nothing\" twice"),
            Ok(TokenList::from([
                Token::new("say", 0, 3),
                Token::quoted("nothing", 5, 12),
                Token::new("twice", 14, 19)
            ]))
        );
        assert_eq!(
            t.tokenize("say \"\" twice"),
            Ok(TokenList::from([
                Token::new("say", 0, 3),
                Token::quoted("", 5, 5),
                Token::new("twice", 7, 12)
            ]))
        );
        assert_eq!(t.tokenize("hello 'world"), Err(()));
        assert_eq!(t.tokenize("hello'world  "), Err(()));
    }
}
