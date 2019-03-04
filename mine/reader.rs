// #[macro_use]
// extern crate lazy_static;

use regex::Regex;

use crate::types::MalType;

pub struct Reader {
    tokens: Vec<Token>,
    position: usize,
}

impl Reader {
    fn new(tokens: Vec<Token>) -> Self {
        Reader {
            tokens: tokens,
            position: 0,
        }
    }

    pub fn next(&mut self) -> Option<&Token> {
        if self.position < self.tokens.len() {
            let ref token = self.tokens[self.position];
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        if self.position < self.tokens.len() {
            let ref token = self.tokens[self.position];
            // don't advance
            Some(token)
        } else {
            None
        }
    }
}

#[derive(PartialEq)]
pub struct Token(String);

fn read_str(s: &str) {
    let tokens = tokenize(s);
    let mut reader = Reader::new(tokens);
    read_form(&mut reader);
}

pub fn tokenize(s: &str) -> Vec<Token> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#).unwrap();
    }
    RE.captures_iter(s)
        .map(|group| {
            let bytes = group[0].as_bytes().to_owned();
            String::from_utf8(bytes).unwrap()
        })
        .map(Token)
        .collect()
}

fn read_form(reader: &mut Reader) -> MalType {
    if reader.peek() == Some(&Token("(".to_string())) {
        read_list(reader)
    } else {
        read_atom(reader)
    }
}

fn read_list(reader: &mut Reader) -> MalType {
    let mut eof_reached = false;
    let mut elements = vec![];
    while let Some(token) = reader.peek() {
        if token == &Token(")".to_string()) {
            eof_reached = true;
            break;
        }
        elements.push(read_form(reader));
    }

    if !eof_reached {
        // TODO error
    }

    unimplemented!()
}


fn read_atom(reader: &mut Reader) -> MalType {
    // TODO

    unimplemented!()
}
