use std::fmt::Display;
use std::str::FromStr;

const WHITESPACE: [char; 4] = ['\t', '\r', ' ', '\n'];
const PUNCTUATION: [char; 8] = ['(', ')', '[', ']', '{', '}', ':', ','];
const NUMBER_CHAR: [char; 5] = ['-', '+', '.', 'e', 'E'];

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    Punct(char),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::String(s) => write!(f, "{}", s),
            Token::Number(n) => write!(f, "{}", n),
            Token::Bool(b) => write!(f, "{}", b),
            Token::Null => write!(f, "Null"),
            Token::Punct(c) => write!(f, "{}", c),
        }
    }
}

pub(crate) struct Lexer {
    line: usize,
    col: usize,
    index: usize,
}

#[derive(Debug)]
pub struct Error {
    text: String,
    line: usize,
    col: usize,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let margin = (self.col + 1).to_string().len() + 1;
        let col = self.col + 1;
        write!(f,
                "\x1b[31;1merror\x1b[0m: unexpected character\n{:>margin$}\n{} |{}\n{:>margin$}\x1b[31;1m{:>col$}\x1b[0m\n",
                "|",
                self.line,
                self.text,
                "|",
                "^")
    }
}

impl Lexer {
    pub(crate) fn new() -> Self {
        Lexer {
            line: 0,
            col: 0,
            index: 0,
        }
    }

    fn lex_string(&mut self, input: &str) -> Option<String> {
        let mut chars = input[self.index..].chars().peekable();

        if chars.peek() == Some(&'"') {
            chars.next();
            self.index += 1;
            self.col += 1;
            let mut s = String::new();

            let mut escape = false;

            for ch in chars {
                self.index += 1;
                self.col += 1;
                if escape {
                    s.push(ch);
                    escape = false;
                } else if ch == '\\' {
                    s.push(ch);
                    escape = true;
                } else if ch == '"' {
                    break;
                } else {
                    s.push(ch);
                }
            }
            Some(s)
        } else {
            None
        }
    }

    fn lex_number(&mut self, input: &str) -> Option<f64> {
        let mut chars = input[self.index..].chars().peekable();
        let mut s = String::new();
        while chars.peek().unwrap_or(&'X').is_digit(10)
            | NUMBER_CHAR.contains(&chars.peek().unwrap_or(&' '))
        {
            s.push(chars.next().unwrap());
            self.index += 1;
            self.col += 1;
        }
        if !s.is_empty() {
            Some(f64::from_str(&s).unwrap())
        } else {
            None
        }
    }

    fn lex_boolean(&mut self, input: &str) -> Option<bool> {
        let true_len = "true".len();
        let false_len = "false".len();

        if input[self.index..].len() >= true_len
            && &input[self.index..self.index + true_len] == "true"
        {
            self.index += true_len;
            self.col += true_len;
            Some(true)
        } else if input[self.index..].len() >= false_len
            && &input[self.index..self.index + false_len] == "false"
        {
            self.index += false_len;
            self.col += false_len;
            Some(false)
        } else {
            None
        }
    }

    fn lex_null(&mut self, input: &str) -> bool {
        let null_len = "null".len();

        if input[self.index..].len() >= null_len
            && &input[self.index..self.index + null_len] == "null"
        {
            self.index += null_len;
            self.col += null_len;
            true
        } else {
            false
        }
    }

    fn eat_whitespace(&mut self, input: &str) -> bool {
        let mut is_whitespace = false;
        let mut chars = input[self.index..].chars().peekable();
        while WHITESPACE.contains(&chars.peek().unwrap_or(&'X')) {
            if chars.peek() == Some(&'\n') {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
            self.index += 1;
            chars.next();
            is_whitespace = true;
        }
        is_whitespace
    }

    pub(crate) fn lex(&mut self, input: &str) -> Result<Vec<Token>, Error> {
        let input_len = input.len();
        let mut tokens = vec![];

        while self.index < input_len {
            // whitespace
            if self.eat_whitespace(input) {
                continue;
            }

            // string
            if let Some(s) = self.lex_string(input) {
                tokens.push(Token::String(s));
                continue;
            }

            // number
            if let Some(i) = self.lex_number(input) {
                tokens.push(Token::Number(i));
                continue;
            }

            // boolean
            if let Some(b) = self.lex_boolean(input) {
                tokens.push(Token::Bool(b));
                continue;
            }

            // null
            if self.lex_null(input) {
                tokens.push(Token::Null);
                continue;
            }

            // punctuation
            let mut chars = input[self.index..].chars().peekable();
            if PUNCTUATION.contains(&chars.peek().unwrap_or(&' ')) {
                tokens.push(Token::Punct(chars.next().unwrap()));
                self.index += 1;
                self.col += 1;
                continue;
            }

            // otherwise return error
            return Err(Error {
                text: input.lines().nth(self.line).unwrap().to_string(),
                line: (self.col + 1).to_string().len() + 1,
                col: self.col,
            });
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex("").unwrap();
        assert_eq!(&tokens[..], []);
    }

    #[test]
    fn just_whitespace() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex("        ").unwrap();
        assert_eq!(&tokens[..], []);
    }

    #[test]
    fn just_null() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex("null").unwrap();
        assert_eq!(&tokens[..], [Token::Null]);
    }

    #[test]
    fn simple_string_and_number() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex(r#"{"key": 123}"#).unwrap();
        assert_eq!(
            &tokens[..],
            [
                Token::Punct('{'),
                Token::String("key".to_string()),
                Token::Punct(':'),
                Token::Number(123.0),
                Token::Punct('}')
            ]
        );
    }

    #[test]
    fn simple_float_number() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex(r#"{"key": 123.0}"#).unwrap();
        assert_eq!(
            &tokens[..],
            [
                Token::Punct('{'),
                Token::String("key".to_string()),
                Token::Punct(':'),
                Token::Number(123.0),
                Token::Punct('}')
            ]
        );
    }

    #[test]
    fn negative_number() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex(r#"{"key": -2}"#).unwrap();
        assert_eq!(
            &tokens[..],
            [
                Token::Punct('{'),
                Token::String("key".to_string()),
                Token::Punct(':'),
                Token::Number(-2.0),
                Token::Punct('}')
            ]
        );
    }

    #[test]
    fn negative_float_number() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex(r#"{"key": -2.0}"#).unwrap();
        assert_eq!(
            &tokens[..],
            [
                Token::Punct('{'),
                Token::String("key".to_string()),
                Token::Punct(':'),
                Token::Number(-2.0),
                Token::Punct('}')
            ]
        );
    }

    #[test]
    fn exponential_float_number() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex(r#"{"key": 1.0E+2}"#).unwrap();
        assert_eq!(
            &tokens[..],
            [
                Token::Punct('{'),
                Token::String("key".to_string()),
                Token::Punct(':'),
                Token::Number(100.0),
                Token::Punct('}')
            ]
        );
    }

    #[test]
    fn string_and_bool() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex(r#"{"key": true}"#).unwrap();
        assert_eq!(
            &tokens[..],
            [
                Token::Punct('{'),
                Token::String("key".to_string()),
                Token::Punct(':'),
                Token::Bool(true),
                Token::Punct('}')
            ]
        );
    }

    #[test]
    fn string_and_null() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex(r#"{"key": null}"#).unwrap();
        assert_eq!(
            &tokens[..],
            [
                Token::Punct('{'),
                Token::String("key".to_string()),
                Token::Punct(':'),
                Token::Null,
                Token::Punct('}')
            ]
        );
    }

    #[test]
    fn complex_object() {
        let input = r#"
            [
              true,
              false,
              null,
              123,
              -123,
              194037878.6297381,
              -194037878.6297381,
              {
                "key": "value"
              }
            ]
"#;
        let mut lexer = Lexer::new();
        let tokens = lexer.lex(input).unwrap();
        assert_eq!(
            &tokens[..],
            [
                Token::Punct('['),
                Token::Bool(true),
                Token::Punct(','),
                Token::Bool(false),
                Token::Punct(','),
                Token::Null,
                Token::Punct(','),
                Token::Number(123_f64),
                Token::Punct(','),
                Token::Number(-123_f64),
                Token::Punct(','),
                Token::Number(194037878.6297381),
                Token::Punct(','),
                Token::Number(-194037878.6297381),
                Token::Punct(','),
                Token::Punct('{'),
                Token::String("key".to_string()),
                Token::Punct(':'),
                Token::String("value".to_string()),
                Token::Punct('}'),
                Token::Punct(']'),
            ]
        );
    }
}
