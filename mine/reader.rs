use std::collections::VecDeque;
use std::str::{self, FromStr};

use regex::Regex;

use crate::types::{MalType, MalError, MalNumber};

pub(crate) struct Reader {
    tokens: VecDeque<Token>,
    forms: VecDeque<MalType>,
    unclosed_lists: Vec<(char, char, Vec<MalType>)>,
}

pub(crate) fn read_str(s: &str) -> Result<Option<MalType>, MalError> {
    let mut reader = Reader::new();
    reader.push(s)?;
    Ok(reader.pop()) // TODO raise an error if the reader isn't empty
}

impl Reader {
    pub fn new() -> Self {
        Self{
            tokens: VecDeque::new(),
            forms: VecDeque::new(),
            unclosed_lists: vec![],
        }
    }

    pub fn push(&mut self, s: &str) -> Result<(), MalError> {
        const TOKENS_REGEX: &str = r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#;
        lazy_static! { static ref RE: Regex = Regex::new(TOKENS_REGEX).unwrap(); }

        // tokenize
        let it = RE.captures_iter(s);

        // optimization: reserve the space for the expected number of tokens
        let (lower, _) = it.size_hint();
        self.tokens.reserve(lower);

        // append the tokens
        for captures in it {
            let s = str::from_utf8(captures[1].as_bytes()).unwrap();
            if s == ";" { break; } // ignore any following tokens
            self.tokens.push_back(Token::from_str(s)?);
        }

        // read each form in the stream, yielding each "full" form
        while let Some(form) = self.read_form()? {
            if let Some((_, _, ref mut elements)) = self.unclosed_lists.last_mut() {
                elements.push(form);
            } else {
                self.forms.push_back(form);
            }
        }

        Ok(())
    }

    pub fn pop(&mut self) -> Option<MalType> {
        self.forms.pop_front()
    }

    pub fn has_unclosed_lists(&self) -> bool {
        !self.unclosed_lists.is_empty()
    }

    fn read_list_opening(&mut self, opening: char, closing: char) -> Result<Option<MalType>, MalError> {
        self.unclosed_lists.push((opening, closing, vec![])); // opens the list
        let list_position = self.unclosed_lists.len();
        loop {
            if let Some(element) = self.read_form()? {
                if self.unclosed_lists.len() < list_position {
                    return Ok(Some(element));
                } else {
                    let ref mut elements = self.unclosed_lists.last_mut().unwrap().2;
                    elements.push(element);
                }
            } else {
                break;
            }
        }

        Ok(None)
    }

    fn read_list_closing(&mut self, opening: char, closing: char) -> Result<Option<Vec<MalType>>, MalError> {
        if let Some((open, close, _)) = self.unclosed_lists.last() {
            if &closing == close {
                let (_, _, elements) = self.unclosed_lists.pop().unwrap(); // closes the list
                Ok(Some(elements))
            } else {
                return Err(MalError::MismatchedDelimiters(*open, *close, closing));
            }
        } else {
            return Err(MalError::UnmatchedDelimiter(opening, closing));
        }
    }

    fn read_form(&mut self) -> Result<Option<MalType>, MalError> {
        if let Some(token) = self.tokens.pop_front() {
            match token {
                Token::Special('(') => {
                    self.read_list_opening('(', ')')
                }

                Token::Special(')') => {
                    Ok(self.read_list_closing('(', ')')?
                        .map(|elements| MalType::List(elements.into_iter().collect())))
                }

                Token::Special('[') => {
                    self.read_list_opening('[', ']')
                }

                Token::Special(']') => {
                    Ok(self.read_list_closing('[', ']')?
                        .map(|elements| MalType::Vector(elements)))
                }

                Token::Special('{') => {
                    self.read_list_opening('{', '}')
                }

                Token::Special('}') => {
                    self.read_list_closing('{', '}')?
                        .map(|elements| MalType::dict_from_elements(elements))
                        .transpose()
                }

                token @ _ => self.read_singleton(token).map(Some),
            }
        } else {
            Ok(None)
        }
    }

    fn read_singleton(&mut self, token: Token) -> Result<MalType, MalError> {
        if let Token::Special(c) = token {
            if let Some(aliased) = self.read_aliased(c.to_string()) {
                return aliased;
            }
        }

        if let Token::SpliceUnquote = token {
            if let Some(aliased) = self.read_aliased("~@".to_string()) {
                return aliased;
            }
        }

        Ok(read_atom(token))
    }

    fn read_aliased(&mut self, s: String) -> Option<Result<MalType, MalError>> {
        let aliased = {
            match s.as_str() {
                "@"  => Some("deref".to_string()),
                "'"  => Some("quote".to_string()),
                "`"  => Some("quasiquote".to_string()),
                "~"  => Some("unquote".to_string()),
                "~@" => Some("splice-unquote".to_string()),
                _    => None,
            }
        };

        aliased.map(|aliased| {
            if let Some(value) = self.read_form()? {
                let symbol = MalType::Symbol(aliased);
                Ok(make_list!(symbol, value))
            } else {
                Err(MalError::MissingFormForAlias(s, aliased))
            }
        })
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
        token @ _                         => unimplemented!("{:?}", token),
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

#[cfg(test)]
mod tests {
    use super::*;
    use MalType::*;
    use MalNumber::*;

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

    #[test]
    fn test_form_read_empty_collections() {
        assert_eq!(read_str("()").unwrap(), Some(make_list!()));
        assert_eq!(read_str("[]").unwrap(), Some(make_vector!()));
        assert_eq!(read_str("{}").unwrap(), Some(make_dict!()));
    }

    #[test]
    fn test_form_push_pop() {
        let mut reader = Reader::new();
        assert_eq!(reader.push("(").unwrap(), ());
        assert_eq!(reader.push(")").unwrap(), ());
        assert_eq!(reader.pop(), Some(make_list!()));
        assert_eq!(reader.pop(), None);
    }

    #[test]
    fn test_pop_when_incomplete_returns_none() {
        let mut reader = Reader::new();
        assert_eq!(reader.push("(").unwrap(), ());
        assert_eq!(reader.pop(), None);
        assert_eq!(reader.push("1").unwrap(), ());
        assert_eq!(reader.pop(), None);
        assert_eq!(reader.push("2)").unwrap(), ());
        assert_eq!(reader.pop(), Some(make_list!(Number(Int(1)), Number(Int(2)))));
        assert_eq!(reader.pop(), None);
    }

    #[test]
    fn test_multiple_pops() {
        let mut reader = Reader::new();
        assert_eq!(reader.push("(1 2) (3 4)").unwrap(), ());
        assert_eq!(reader.pop(), Some(make_list!(Number(Int(1)), Number(Int(2)))));
        assert_eq!(reader.pop(), Some(make_list!(Number(Int(3)), Number(Int(4)))));
        assert_eq!(reader.pop(), None);
    }
}
