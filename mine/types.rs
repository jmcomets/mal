use std::fmt;
use std::io;
use std::ops;
use std::rc::Rc;
use std::cmp;

#[derive(Clone)]
pub(crate) enum MalType {
    List(Vec<MalType>),
    Vector(Vec<MalType>),
    Symbol(String),
    Number(MalNumber),
    Bool(bool),
    Str(String),
    Nil,
    #[allow(dead_code)]
    Function(Rc<dyn Fn(&[MalType]) -> MalResult>),
}

#[allow(unused)]
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
    UnbalancedList,
    IOError(io::Error),
}

pub(crate) type MalResult = Result<MalType, MalError>;

impl fmt::Debug for MalType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use MalType::*;
        match self {
            List(x)     => write!(fmt, "List {{ {:?} }}", x),
            Vector(x)   => write!(fmt, "Vector {{ {:?} }}", x),
            Symbol(x)   => write!(fmt, "Symbol {{ {:?} }}", x),
            Number(x)   => write!(fmt, "{:?}", x),
            Bool(x)     => write!(fmt, "Bool {{ {:?} }}", x),
            Str(x)      => write!(fmt, "Str {{ {:?} }}", x),
            Nil         => write!(fmt, "Nil"),
            Function(_) => write!(fmt, "Function(...)"),
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
        impl ops::$trait for MalNumber {
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
