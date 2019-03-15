use std::fmt;

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
    NativeFunc {
        name: &'static str,
        signature: (Vec<&'static str>, &'static str),
        func: fn(&[MalType]) -> MalResult,
    }
}

impl fmt::Debug for MalType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use MalType::*;
        match self {
            List(x)       => write!(fmt, "List {{ {:?} }}", x),
            Vector(x)     => write!(fmt, "Vector {{ {:?} }}", x),
            Symbol(x)     => write!(fmt, "Symbol {{ {:?} }}", x),
            Int(x)        => write!(fmt, "Int {{ {:?} }}", x),
            Float(x)      => write!(fmt, "Float {{ {:?} }}", x),
            Bool(x)       => write!(fmt, "Bool {{ {:?} }}", x),
            Str(x)        => write!(fmt, "Str {{ {:?} }}", x),
            Nil           => write!(fmt, "Nil"),
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

macro_rules! binary_operator {
    ($left:tt $op:tt $right:tt -> $out:tt) => {
            $crate::types::MalType::NativeFunc {
                name: stringify!($op),
                signature: (vec![stringify!($left), stringify!($right)], stringify!($out)),
                func: {
                    use $crate::types::{
                        MalType::{self, *},
                        MalError::*,
                        MalResult,
                    };

                    |args: &[MalType]| -> MalResult {
                        if args.len() != 2 {
                            return Err(ArityError {
                                expected: 2,
                                reached: args.len(),
                            });
                        }

                        if let ($left(lhs), $right(rhs)) = (&args[0], &args[1]) {
                            Ok($out(lhs $op rhs))
                        } else {
                            Err(TypeCheckFailed{})
                        }
                    }
                },
            }
    }
}
