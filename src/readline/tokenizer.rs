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
                        tokens.push_back(Token::plain(&token, begin, i));
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
            Status::Token => tokens.push_back(Token::plain(&token, begin, line.chars().count())),
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
    fn plain(text: &str, begin: usize, end: usize) -> Self {
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

    impl<const N: usize> From<[Token; N]> for TokenList {
        fn from(s: [Token; N]) -> Self {
            TokenList { tokens: s.into() }
        }
    }

    #[test]
    fn test_simple() {
        let t = Tokenizer::new();
        assert_eq!(t.tokenize("a"), Ok([Token::plain("a", 0, 1)].into()));
        assert_eq!(
            t.tokenize("abcde"),
            Ok([Token::plain("abcde", 0, 5)].into())
        );
        assert_eq!(
            t.tokenize("abcde fghij"),
            Ok([Token::plain("abcde", 0, 5), Token::plain("fghij", 6, 11)].into())
        );
        assert_eq!(
            t.tokenize("123 abcde fghij"),
            Ok([
                Token::plain("123", 0, 3),
                Token::plain("abcde", 4, 9),
                Token::plain("fghij", 10, 15)
            ]
            .into())
        );
    }

    #[test]
    fn test_whitespace() {
        let t = Tokenizer::new();
        assert_eq!(
            t.tokenize("    a    b    "),
            Ok([Token::plain("a", 4, 5), Token::plain("b", 9, 10)].into())
        );
        assert_eq!(
            t.tokenize("  abcde  "),
            Ok([Token::plain("abcde", 2, 7)].into())
        );
        assert_eq!(
            t.tokenize("abcde  fghij"),
            Ok([Token::plain("abcde", 0, 5), Token::plain("fghij", 7, 12)].into())
        );
        assert_eq!(
            t.tokenize(" abcde fghij "),
            Ok([Token::plain("abcde", 1, 6), Token::plain("fghij", 7, 12)].into())
        );
    }

    #[test]
    fn test_quoting() {
        let t = Tokenizer::new();
        assert_eq!(
            t.tokenize("say \"abcde fghij\""),
            Ok([
                Token::plain("say", 0, 3),
                Token::quoted("abcde fghij", 5, 16)
            ]
            .into())
        );
        assert_eq!(
            t.tokenize("  say 'abcde  fghij'  "),
            Ok([
                Token::plain("say", 2, 5),
                Token::quoted("abcde  fghij", 7, 19)
            ]
            .into())
        );
        assert_eq!(
            t.tokenize("say 'abcde fghij ' twice"),
            Ok([
                Token::plain("say", 0, 3),
                Token::quoted("abcde fghij ", 5, 17),
                Token::plain("twice", 19, 24)
            ]
            .into())
        );
        assert_eq!(
            t.tokenize("say \"nothing\" twice"),
            Ok([
                Token::plain("say", 0, 3),
                Token::quoted("nothing", 5, 12),
                Token::plain("twice", 14, 19)
            ]
            .into())
        );
        assert_eq!(
            t.tokenize("say '' twice"),
            Ok([
                Token::plain("say", 0, 3),
                Token::quoted("", 5, 5),
                Token::plain("twice", 7, 12)
            ]
            .into())
        );
    }

    #[test]
    fn test_invalid() {
        let t = Tokenizer::new();
        assert_eq!(t.tokenize("abcde 'fghij"), Err(()));
        assert_eq!(t.tokenize("abcde' fghij"), Err(()));
        assert_eq!(t.tokenize("abcde'fghij "), Err(()));
        assert_eq!(t.tokenize("abcde'fghij\""), Err(()));
    }
}
