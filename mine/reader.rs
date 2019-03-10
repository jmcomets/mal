use std::str::{self, FromStr};

use regex::Regex;

use crate::types::MalType;

struct Reader(Vec<Token>);

impl Reader {
    fn new(mut tokens: Vec<Token>) -> Self {
        tokens.reverse();
        Reader(tokens)
    }

    fn next(&mut self) -> Option<Token> {
        self.0.pop()
    }

    fn peek(&self) -> Option<&Token> {
        self.0.last()
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum Error {
    UnbalancedString,
    UnbalancedList,
}

const TOKENS_REGEX: &str = r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#;

#[derive(Debug, PartialEq)]
pub(crate) struct ReaderParseError;

impl FromStr for Reader {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! { static ref RE: Regex = Regex::new(TOKENS_REGEX).unwrap(); }

        // tokenize
        let tokens: Result<Vec<_>, _> = RE.captures_iter(s)
            .map(|cap| Token::from_str(str::from_utf8(cap[1].as_bytes()).unwrap()))
            .collect();
        tokens.map(Reader::new)
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Special(char),        // []{}()'`~^@
    SpecialTwoCharacters, // @~
    Comment,              // The ";" token
    Literal(Literal),     // integers, floats, booleans, strings, nil, ...
    Symbol(String),       // identifiers
}

const SPECIAL_CHARS: &str = "[]{}()'`~^@";
const SPECIAL_TWO_CHARS: &str = "@~";
const COMMENT_CHAR: char = ';';

impl FromStr for Token {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // special characters / comments
        if s.len() == 1 {
            let c = s.chars().next().unwrap();
            if SPECIAL_CHARS.contains(c) {
                return Ok(Token::Special(c));
            } else if c == COMMENT_CHAR {
                return Ok(Token::Comment);
            }
        } else if s == SPECIAL_TWO_CHARS {
            return Ok(Token::SpecialTwoCharacters);
        }

        // literals
        match s.parse::<Literal>() {
            Ok(lit) => Ok(Token::Literal(lit)),

            Err(LiteralParseError::UnbalancedString) => {
                Err(Error::UnbalancedString)
            }

            _ => {
                Ok(Token::Symbol(s.to_owned()))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Nil,
}

const NIL_STR: &str = "nil";
const STRING_QUOTE_CHAR: char = '"';

#[derive(Debug, PartialEq)]
enum LiteralParseError {
    UnbalancedString,
    Unspecified,
}

impl FromStr for Literal {
    type Err = LiteralParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // nil
        if s == NIL_STR {
            return Ok(Literal::Nil);
        }

        // booleans
        if let Ok(b) = s.parse::<bool>() {
            return Ok(Literal::Bool(b));
        }

        // integers
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Literal::Int(i));
        }

        // floats
        if let Ok(f) = s.parse::<f64>() {
            return Ok(Literal::Float(f));
        }

        // strings
        if s.len() >= 2 {
            let first = s.chars().next().unwrap();

            if first == STRING_QUOTE_CHAR {
                let last = s.chars().last().unwrap();
                if last != STRING_QUOTE_CHAR {
                    return Err(LiteralParseError::UnbalancedString);
                }

                let s = s.chars()
                    .skip(1)            // after the opening quote
                    .take(s.len() - 2)  // before the closing quote
                    .collect();
                return Ok(Literal::Str(s));
            }
        }

        Err(LiteralParseError::Unspecified)
    }
}

pub(crate) fn read_str(s: &str) -> Result<Option<MalType>, Error> {
    let mut reader = Reader::from_str(s)?;
    read_form(&mut reader)
}

fn read_form(reader: &mut Reader) -> Result<Option<MalType>, Error> {
    reader.next()
        .map(|token| {
            if token == Token::Special('(') {
                read_list(reader)
            } else {
                Ok(read_atom(token))
            }
        })
        .transpose()
}

fn read_list(reader: &mut Reader) -> Result<MalType, Error> {
    let mut paren_matched = false;
    let mut elements = vec![];
    while let Some(token) = reader.peek() {
        if token == &Token::Special(')') {
            reader.next();
            paren_matched = true;
            break;
        }

        if let Some(element) = read_form(reader)? {
            elements.push(element);
        }
    }

    if paren_matched {
        Ok(MalType::List(elements))
    } else {
        Err(Error::UnbalancedList)
    }
}


fn read_atom(token: Token) -> MalType {
    match token {
        Token::Symbol(s)                  => MalType::Symbol(s),
        Token::Literal(Literal::Int(i))   => MalType::Int(i),
        Token::Literal(Literal::Float(f)) => MalType::Float(f),
        Token::Literal(Literal::Bool(b))  => MalType::Bool(b),
        Token::Literal(Literal::Str(s))   => MalType::Str(s),
        Token::Literal(Literal::Nil)      => MalType::Nil,
        _                                 => MalType::Unimplemented,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_read_str() {
    //     assert_eq!(read_str("123"), Some(Ok(MalType::Int(123))));
    //     assert_eq!(read_str("(123 456)"), Some(Ok(MalType::List(vec![
    //                                                      Box::new(MalType::Int(123)),
    //                                                      Box::new(MalType::Int(456)),
    //     ]))));
    // }


    #[test]
    fn test_special_char_parsing() {
        for c in SPECIAL_CHARS.chars() {
            assert_eq!(Token::from_str(&c.to_string()), Ok(Token::Special(c)));
        }

        assert_eq!(Token::from_str(SPECIAL_TWO_CHARS), Ok(Token::SpecialTwoCharacters));

        assert_eq!(Token::from_str("true"), Ok(Token::Literal(Literal::Bool(true))));
    }

    #[test]
    fn test_literal_parsing() {
        // booleans
        assert_eq!(Literal::from_str("true"), Ok(Literal::Bool(true)));
        assert_eq!(Literal::from_str("false"), Ok(Literal::Bool(false)));

        // integers
        assert_eq!(Literal::from_str("123"), Ok(Literal::Int(123)));

        // floats
        assert_eq!(Literal::from_str("1.2"), Ok(Literal::Float(1.2)));
        assert_eq!(Literal::from_str("0.2"), Ok(Literal::Float(0.2)));
        assert_eq!(Literal::from_str("0.0"), Ok(Literal::Float(0.0)));

        // nil
        assert_eq!(Literal::from_str("nil"), Ok(Literal::Nil));

        // strings
        assert_eq!(Literal::from_str("\"foobar\""), Ok(Literal::Str("foobar".to_string())));
        assert_eq!(Literal::from_str("\"foobar"), Err(LiteralParseError::UnbalancedString));
    }
}
