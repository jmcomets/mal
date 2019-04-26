#![allow(unused)] // TODO remove this

use im::Vector as ImVec;
use im::HashMap as ImHashMap;

use std::cmp;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io;
use std::ops::{Add, Sub, Mul, Div};
use std::rc::Rc;
use std::cell::RefCell;

use crate::env::Env;

#[derive(Clone)]
pub(crate) enum MalType {
    Atom(Rc<RefCell<MalType>>),
    Bool(bool),
    Dict(ImHashMap<MalHashable, MalType>),
    List(ImVec<MalType>),
    Nil,
    Number(MalNumber),
    Str(String),
    Symbol(String),
    Vector(Vec<MalType>),
    Function(Rc<dyn Fn(MalArgs) -> MalResult>),
    UserFunction {
      body: Rc<MalType>,
      env: Env,
      symbols: Vec<String>,
    },
}

pub(crate) type MalArgs = ImVec<MalType>;

impl MalType {
    pub fn atom(inner: Self) -> Self {
        MalType::Atom(Rc::new(RefCell::new(inner)))
    }

    pub fn dict_from_elements(elements: Vec<MalType>) -> Result<Self, MalError> {
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
    }

    pub fn user_function<It>(symbols: It, body: Self, env: Env) -> Self
        where It: IntoIterator<Item=String>
    {
        MalType::UserFunction {
            body: Rc::new(body),
            env: env.wrap(),
            symbols: symbols.into_iter().collect(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum MalError {
    TypeCheckFailed {
        // expected: Vec<String>,
        // reached: String,
    },
    ArityError {
        // symbol: String,
        expected: usize,
        reached: usize,
    },
    NotEvaluable(MalType),
    CanOnlyDefineSymbols(MalType),
    CannotBindArguments(MalType),
    SymbolNotFound(String),
    UnbalancedString,
    MismatchedDelimiters(char, char, char),
    UnmatchedDelimiter(char, char),
    OddMapEntries,
    NotHashable(MalType),
    DuplicateKey(MalType),
    MissingFormForAlias(String, String),
    IOError(io::Error),
}

pub(crate) type MalResult = Result<MalType, MalError>;

impl fmt::Debug for MalType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use MalType::*;
        match self {
            Atom(x)     => write!(fmt, "Atom {{ {:?} }}", x),
            List(x)     => write!(fmt, "List {{ {:?} }}", x),
            Vector(x)   => write!(fmt, "Vector {{ {:?} }}", x),
            Dict(x)     => write!(fmt, "Dict {{ {:?} }}", x),
            Symbol(x)   => write!(fmt, "Symbol {{ {:?} }}", x),
            Number(x)   => write!(fmt, "{:?}", x),
            Bool(x)     => write!(fmt, "Bool {{ {:?} }}", x),
            Str(x)      => write!(fmt, "Str {{ {:?} }}", x),
            Nil         => write!(fmt, "Nil"),
            Function(_) | UserFunction { .. } => write!(fmt, "Function(...)"),
        }
    }
}

impl PartialEq for MalType {
    fn eq(&self, other: &Self) -> bool {
        use MalType::*;
        match (self, other) {
            (Number(a), Number(b)) => a == b,
            (Bool(a), Bool(b))     => a == b,
            (Str(a), Str(b))       => a == b,
            (Vector(a), Vector(b)) => a == b,
            (List(a), List(b))     => a == b,
            (Dict(a), Dict(b))     => a == b,
            (Symbol(a), Symbol(b)) => a == b,
            (Nil, Nil)             => true,
            _                      => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum MalNumber {
    Int(i64),
    Float(f64),
}

impl PartialOrd for MalNumber {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        use MalNumber::*;
        match (self, other) {
            (Float(a), Float(b)) => a.partial_cmp(b),
            (Int(a), Int(b))     => a.partial_cmp(b),
            (Float(a), Int(b))   => a.partial_cmp(&(*b as f64)),
            (Int(a), Float(b))   => (*a as f64).partial_cmp(b),
        }
    }
}

impl PartialEq for MalNumber {
    fn eq(&self, other: &Self) -> bool {
        use MalNumber::*;
        match (self, other) {
            (Float(a), Float(b)) => a == b,
            (Int(a), Int(b))     => a == b,
            (Float(a), Int(b))   => *a == (*b as f64),
            (Int(a), Float(b))   => (*a as f64) == *b,
        }
    }
}

macro_rules! impl_number_op {
    ($trait:tt, $method:tt) => {
        impl $trait for MalNumber {
            type Output = Self;

            fn $method(self, other: Self) -> Self::Output {
                use MalNumber::*;
                match (self, other) {
                    (Int(a), Int(b))     => Int(a.$method(b)),
                    (Float(a), Float(b)) => Float(a.$method(b)),
                    (Float(a), Int(b))   => Float(a.$method(b as f64)),
                    (Int(a), Float(b))   => Float((a as f64).$method(b)),
                }
            }
        }
    }
}

impl_number_op!(Add, add);
impl_number_op!(Sub, sub);
impl_number_op!(Mul, mul);
impl_number_op!(Div, div);

impl ToString for MalNumber {
    fn to_string(&self) -> String {
        use MalNumber::*;
        match self {
            Int(i)   => i.to_string(),
            Float(f) => f.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MalHashable {
    Bool(bool),
    Int(i64),
    List(Vec<MalHashable>),
    Nil,
    Str(String),
    Symbol(String),
}

impl MalHashable {
    pub fn try_from(value: MalType) -> Result<MalHashable, MalType> {
        use MalHashable::*;
        match value {
            MalType::List(elements)            => {
                Self::try_from_list(elements.clone())
                    .map(List)
                    .ok_or(MalType::List(elements))
            }
            MalType::Symbol(x)                 => Ok(Symbol(x)),
            MalType::Number(MalNumber::Int(x)) => Ok(Int(x)),
            MalType::Bool(x)                   => Ok(Bool(x)),
            MalType::Nil                       => Ok(Nil),
            MalType::Str(x)                    => Ok(Str(x)),
            value @ _                          => Err(value),
        }
    }

    fn try_from_list<It>(elements: It) -> Option<Vec<MalHashable>>
        where It: IntoIterator<Item=MalType>,
    {
        elements.into_iter()
            .map(Self::try_from)
            .map(Result::ok)
            .collect()
    }
}

impl Into<MalType> for MalHashable {
    fn into(self) -> MalType {
        use MalHashable::*;
        match self {
            List(x)   => MalType::List(x.into_iter().map(Self::into).collect()),
            Symbol(x) => MalType::Symbol(x),
            Int(x)    => MalType::Number(MalNumber::Int(x)),
            Bool(x)   => MalType::Bool(x),
            Nil       => MalType::Nil,
            Str(x)    => MalType::Str(x),
        }
    }
}

impl Eq for MalHashable {
}

impl Hash for MalHashable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use MalHashable::*;
        match self {
            List(x)   => Hash::hash(&(1,  x), state),
            Symbol(x) => Hash::hash(&(3,  x), state),
            Int(x)    => Hash::hash(&(5,  x), state),
            Bool(x)   => Hash::hash(&(7,  x), state),
            Nil       => Hash::hash(&(9, ()), state),
            Str(x)    => Hash::hash(&(13, x), state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn it_can_hash_some_types() {
        let hashables = vec![
            MalType::Bool(true),
            make_list!(),
            MalType::Nil,
            MalType::Number(MalNumber::Int(42)),
            MalType::Str("".to_string()),
            MalType::Symbol("".to_string()),
        ];

        for hashable in hashables {
            assert!(MalHashable::try_from(hashable.clone()).is_ok());
        }
    }

    #[test]
    fn it_cannot_hash_some_types() {
        let non_hashables = vec![
            MalType::Atom(Rc::new(RefCell::new(MalType::Symbol("".to_string())))),
            make_dict!(),
            MalType::Number(MalNumber::Float(0.)),
            make_vector!(),
            function!(x { Ok(MalType::Nil) }),
        ];

        for non_hashable in non_hashables {
            assert!(MalHashable::try_from(non_hashable.clone()).is_err());
        }
    }
}
