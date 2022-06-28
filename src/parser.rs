#![allow(dead_code)]

use crate::lexer;
use std::collections::HashMap;
use std::iter::{Iterator, Peekable};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Object(HashMap<String, Value>),
    List(Vec<Value>),
    Boolean(bool),
    Null,
    Number(f64),
    String(String),
}

pub struct Parser {}

#[derive(Debug)]
pub enum Error {
    LexError(lexer::Error),
    ParseError(String),
}

impl From<lexer::Error> for Error {
    fn from(le: lexer::Error) -> Self {
        Error::LexError(le)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::LexError(le) => write!(f,"{}",le),
            Error::ParseError(s) => write!(f, "\x1b[31;1merror\x1b[0m: {}\x1b[0m\n", s),
        } 
    }
}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn parse(&self, input: &str) -> Result<Option<Value>, Error> {
        let tokens = lexer::Lexer::new().lex(input)?;

        if tokens.is_empty() {
            return Ok(None);
        }

        let value = self.parse_value(&mut tokens.iter().peekable())?;
        Ok(Some(value))
    }

    fn parse_value<'a, I>(&self, tokens: &mut Peekable<I>) -> Result<Value, Error>
    where
        I: Iterator<Item = &'a lexer::Token>,
    {
        match tokens.next().as_ref() {
            Some(lexer::Token::Null) => Ok(Value::Null),
            Some(lexer::Token::Bool(b)) => Ok(Value::Boolean(*b)),
            Some(lexer::Token::Number(n)) => Ok(Value::Number(*n)),
            Some(lexer::Token::String(s)) => Ok(Value::String(s.clone())),
            Some(lexer::Token::Punct('[')) => {
                let mut list: Vec<Value> = vec![];
                loop {
                    if tokens.peek() == Some(&&lexer::Token::Punct(']')) {
                        tokens.next();
                        break;
                    }

                    let value = self.parse_value(tokens)?;
                    list.push(value);

                    match tokens.peek() {
                        Some(&lexer::Token::Punct(',')) => {
                            tokens.next();
                            continue;
                        }
                        Some(&lexer::Token::Punct(']')) => {
                            tokens.next();
                            break;
                        }
                        Some(_) => (),
                        None => {
                            return Err(Error::ParseError(
                                "unexpected end of input while parsing list".to_string(),
                            ))
                        }
                    }
                }
                return Ok(Value::List(list));
            }
            Some(&lexer::Token::Punct('{')) => {
                let mut hm = HashMap::<String, Value>::new();
                loop {
                    if tokens.peek() == Some(&&lexer::Token::Punct('}')) {
                        tokens.next();
                        break;
                    }

                    let key = match self.parse_value(tokens)? {
                        Value::String(s) => s,
                        _ => return Err(Error::ParseError("invalid key value".to_string())),
                    };

                    if tokens.peek() == Some(&&lexer::Token::Punct(':')) {
                        tokens.next();
                    } else {
                        return Err(Error::ParseError(format!("expected ':', got '{:?}'", tokens.next())));
                    }

                    let value = self.parse_value(tokens)?;
                    hm.insert(key, value);

                    match tokens.peek() {
                        Some(&lexer::Token::Punct(',')) => {
                            tokens.next();
                            continue;
                        }
                        Some(&lexer::Token::Punct('}')) => {
                            tokens.next();
                            break;
                        }
                        Some(_) => (),
                        None => {
                            return Err(Error::ParseError(
                                "unexpected end of input while parsing list".to_string(),
                            ))
                        }
                    }
                }

                return Ok(Value::Object(hm));
            }
            t => Err(Error::ParseError(format!("unexpected token '{:?}'", t))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null() {
        let input = "null";
        let value = Parser::new().parse(&input).unwrap().unwrap();
        assert_eq!(value, Value::Null);
    }

    #[test]
    fn bool_true() {
        let input = "true";
        let value = Parser::new().parse(&input).unwrap().unwrap();
        assert_eq!(value, Value::Boolean(true));
    }

    #[test]
    fn bool_false() {
        let input = "false";
        let value = Parser::new().parse(&input).unwrap().unwrap();
        assert_eq!(value, Value::Boolean(false));
    }

    #[test]
    fn positive_integer() {
        let input = "42";
        let value = Parser::new().parse(&input).unwrap().unwrap();
        assert_eq!(value, Value::Number(42.0));
    }

    #[test]
    fn negative_integer() {
        let input = "-42";
        let value = Parser::new().parse(&input).unwrap().unwrap();
        assert_eq!(value, Value::Number(-42.0));
    }

    #[test]
    fn positive_float() {
        let input = "42.0";
        let value = Parser::new().parse(&input).unwrap().unwrap();
        assert_eq!(value, Value::Number(42.0));
    }

    #[test]
    fn negative_float() {
        let input = "-42.0";
        let value = Parser::new().parse(&input).unwrap().unwrap();
        assert_eq!(value, Value::Number(-42.0));
    }

    #[test]
    fn empty_list() {
        let input = "[]";
        let value = Parser::new().parse(&input).unwrap().unwrap();
        assert_eq!(value, Value::List(vec![]));
    }

    #[test]
    fn basic_list() {
        let input = "[true, false, 1, \"hello\"]";
        let value = Parser::new().parse(&input).unwrap().unwrap();
        assert_eq!(
            value,
            Value::List(vec![
                Value::Boolean(true),
                Value::Boolean(false),
                Value::Number(1.0),
                Value::String("hello".to_string())
            ])
        );
    }

    #[test]
    fn flat_object() {
        let mut hm = HashMap::new();
        hm.insert("key1".to_string(), Value::String("value".to_string()));
        hm.insert("key2".to_string(), Value::Number(123.0));
        let input = r#"{"key1": "value", "key2": 123.0}"#;
        let value = Parser::new().parse(&input).unwrap().unwrap();
        assert_eq!(value, Value::Object(hm));
    }

    #[test]
    fn deep_object() {
        let mut hm = HashMap::new();
        hm.insert("key1".to_string(), Value::String("value".to_string()));
        hm.insert("key2".to_string(), Value::Number(123.0));
        hm.insert("key3".to_string(), Value::Object(hm.clone()));
        let input = r#"{"key1": "value", "key2": 123.0, "key3": {"key1": "value", "key2": 123.0}}"#;
        let value = Parser::new().parse(&input).unwrap().unwrap();
        assert_eq!(value, Value::Object(hm));
    }

}
