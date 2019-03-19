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
            NativeFunc { name, signature, .. } => {
                fmt.debug_struct("NativeFunc")
                    .field("name", name)
                    .field("signature", signature)
                    .finish()
            }
        }
    }
}

pub(crate) type MalResult = Result<MalType, MalError>;

pub(crate) enum MalError {
    CallError,
    TypeCheckFailed {
        // expected: Vec<String>,
        // reached: String,
    },
    ArityError {
        // symbol: String,
        expected: usize,
        reached: usize,
    }
}

macro_rules! function {
    ($($arg:tt : $argtype:tt),* -> $rettype:tt $body:block) => {
        $crate::types::MalType::Function(std::rc::Rc::new({
            use $crate::types::{
                MalType::{self, *},
                MalError::*,
                MalResult,
            };

            |args: &[MalType]| -> MalResult {
                let nb_args = 0 $(+ {stringify!($arg); 1})*;
                if args.len() != nb_args {
                    return Err(ArityError {
                        expected: nb_args,
                        reached: args.len(),
                    });
                }

                let mut arg_index = 0;
                $(
                    let $arg = {
                        if let $argtype($arg) = args[arg_index] {
                            $arg
                        } else {
                            return Err(TypeCheckFailed{});
                        }
                    };

                    #[allow(unused)] {
                        arg_index += 1;
                    }
                )*

                Ok($rettype($body))
            }
        }))
    }
}

macro_rules! function_chain {
    ($($f:expr),*) => {
        $crate::types::MalType::Function(std::rc::Rc::new({
            use $crate::types::{
                MalType,
                MalError::*,
                MalResult,
            };

            |args: &[MalType]| -> MalResult {
                $(
                    match $f(args) {
                        Err(TypeCheckFailed{}) => {},
                        ret @ Ok(_) | Err(_)   => return ret,
                    }
                )*

                Err(TypeCheckFailed{})
            }
        }))
    }
}

macro_rules! binary_operator {
    ($left:tt $op:tt $right:tt -> $out:tt) => {
        function!(a: $left, b: $right -> $out { a $op b })
    }
}
