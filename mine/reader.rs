use im::HashMap as ImHashMap;
use std::str::{self, FromStr};

use regex::Regex;

use crate::types::{MalType, MalError, MalHashable, MalNumber};

struct Reader(Vec<Token>);

impl Reader {
    fn new(mut tokens: Vec<Token>) -> Self {
        tokens.reverse();
        Self(tokens)
    }

    fn next(&mut self) -> Option<Token> {
        self.0.pop()
    }

    fn peek(&self) -> Option<&Token> {
        self.0.last()
    }
}

struct ReaderFeed(Vec<Token>);

impl ReaderFeed {
    fn new() -> Self {
        Self(vec![])
    }

    fn feed_line(&mut self, s: &str) -> Result<(), MalError> {
        const TOKENS_REGEX: &str = r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#;
        lazy_static! { static ref RE: Regex = Regex::new(TOKENS_REGEX).unwrap(); }

        // tokenize
        let it = RE.captures_iter(s);

        // optimization: reserve the space for the expected number of tokens
        let (lower, _) = it.size_hint();
        self.0.reserve(lower);

        // append the tokens
        for captures in it {
            let s = str::from_utf8(captures[1].as_bytes()).unwrap();
            if s == ";" { break; } // ignore any following tokens
            self.0.push(Token::from_str(s)?);
        }

        Ok(())
    }

    fn finalize(self) -> Reader {
        Reader::new(self.0)
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Special(char),    // []{}()'`~^@
    SpliceUnquote,    // ~@
    Literal(Literal), // integers, floats, booleans, strings, nil, ...
    Symbol(String),   // identifiers
}

const SPECIAL_CHARS: &str = "[]{}()'`~^@";
const SPLICE_UNQUOTE: &str = "~@";

impl FromStr for Token {
    type Err = MalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // special characters / comments
        if s.len() == 1 {
            let c = s.chars().next().unwrap();
            if SPECIAL_CHARS.contains(c) {
                return Ok(Token::Special(c));
            }
        } else if s == SPLICE_UNQUOTE {
            return Ok(Token::SpliceUnquote);
        }

        // literals
        match s.parse::<Literal>() {
            Ok(lit) => Ok(Token::Literal(lit)),

            Err(LiteralParseError::UnbalancedString) => {
                Err(MalError::UnbalancedString)
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

pub(crate) fn read_str(s: &str) -> Result<Option<MalType>, MalError> {
    let mut feed = ReaderFeed::new();
    feed.feed_line(s)?;
    read_form(&mut feed.finalize())
}

fn read_form(reader: &mut Reader) -> Result<Option<MalType>, MalError> {
    reader.next()
        .map(|token| {
            match token {
                Token::Special('(') => read_list(reader, ')', |elements| Ok(MalType::List(elements.into_iter().collect()))),
                Token::Special('[') => read_list(reader, ']', |elements| Ok(MalType::Vector(elements))),
                Token::Special('{') => read_list(reader, '}', |elements| {
                    if elements.len() % 2 != 0 {
                        return Err(MalError::OddMapEntries);
                    }

                    let mut map = ImHashMap::new();

                    let mut it = elements.into_iter();
                    while let Some(key) = it.next() {
                        let key = MalHashable::try_from(key)
                            .map_err(MalError::NotHashable)?;

                        // this cannot fail because the length is even
                        let value = it.next().unwrap();

                        let previous = map.insert(key.clone(), value);
                        if previous.is_some() {
                            return Err(MalError::DuplicateKey(key.into()));
                        }
                    }

                    Ok(MalType::Dict(map))
                }),
                Token::Special('@') => {
                    if let Some(value) = read_form(reader)? {
                        let deref_symbol = MalType::Symbol("deref".to_string());
                        Ok(make_list!(deref_symbol, value))
                    } else {
                        Err(MalError::LoneDeref)
                    }
                }
                _                   => Ok(read_atom(token))
            }
        })
        .transpose()
}

fn read_list<T>(reader: &mut Reader, closing: char, consumer: fn(Vec<MalType>) -> Result<T, MalError>) -> Result<T, MalError> {
    let mut paren_matched = false;
    let mut elements = vec![];
    while let Some(token) = reader.peek() {
        if token == &Token::Special(closing) {
            reader.next();
            paren_matched = true;
            break;
        }

        if let Some(element) = read_form(reader)? {
            elements.push(element);
        }
    }

    if paren_matched {
        consumer(elements)
    } else {
        Err(MalError::UnbalancedList)
    }
}

fn read_atom(token: Token) -> MalType {
    match token {
        Token::Symbol(s)                  => MalType::Symbol(s),
        Token::Literal(Literal::Int(i))   => MalType::Number(MalNumber::Int(i)),
        Token::Literal(Literal::Float(f)) => MalType::Number(MalNumber::Float(f)),
        Token::Literal(Literal::Bool(b))  => MalType::Bool(b),
        Token::Literal(Literal::Str(s))   => MalType::Str(s),
        Token::Literal(Literal::Nil)      => MalType::Nil,
        _                                 => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_char_parsing() {
        for c in SPECIAL_CHARS.chars() {
            assert_eq!(Token::from_str(&c.to_string()).unwrap(), Token::Special(c));
        }

        assert_eq!(Token::from_str(SPLICE_UNQUOTE).unwrap(), Token::SpliceUnquote);

        assert_eq!(Token::from_str("true").unwrap(), Token::Literal(Literal::Bool(true)));
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
