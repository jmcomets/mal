use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub(crate) enum MalType {
    List(Vec<MalType>),
    Vector(Vec<MalType>),
    Symbol(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Nil,
    #[allow(dead_code)]
    Function(Rc<dyn Fn(&[MalType]) -> MalResult>),
}

impl fmt::Debug for MalType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use MalType::*;
        match self {
            List(x)                            => write!(fmt, "List {{ {:?} }}", x),
            Vector(x)                          => write!(fmt, "Vector {{ {:?} }}", x),
            Symbol(x)                          => write!(fmt, "Symbol {{ {:?} }}", x),
            Int(x)                             => write!(fmt, "Int {{ {:?} }}", x),
            Float(x)                           => write!(fmt, "Float {{ {:?} }}", x),
            Bool(x)                            => write!(fmt, "Bool {{ {:?} }}", x),
            Str(x)                             => write!(fmt, "Str {{ {:?} }}", x),
            Nil                                => write!(fmt, "Nil"),
            Function(_)                        => write!(fmt, "Function(...)"),
        }
    }
}

impl PartialEq for MalType {
    fn eq(&self, other: &Self) -> bool {
        use MalType::*;
        match (self, other) {
            (Int(a), Int(b))       => a == b,
            (Float(a), Float(b))   => a == b,
            (Float(a), Int(b))     => *a == (*b as f64),
            (Int(a), Float(b))     => (*a as f64) == *b,
            (Bool(a), Bool(b))     => a == b,
            (Str(a), Str(b))       => a == b,
            (Vector(a), Vector(b)) => a == b,
            (List(a), List(b))     => a == b,
            (Symbol(a), Symbol(b)) => a == b,
            (Function(_), _)       => false,
            (_, Function(_))       => false,
            (Nil, Nil)             => true,
            _                      => false,
        }
    }
}

pub(crate) type MalResult = Result<MalType, MalError>;

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
}
