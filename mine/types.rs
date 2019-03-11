use std::fmt;

#[derive(Clone)]
pub(crate) enum MalType {
    Unimplemented, // TODO remove
    List(Vec<MalType>),
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
            Unimplemented => write!(fmt, "Unimplemented"),
            List(x)       => write!(fmt, "List {{ {:?} }}", x),
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

// TODO generate this code via macro_rules!

#[allow(dead_code)]
impl MalType {
    pub(crate) fn is_list(&self) -> bool {
        match self {
            MalType::List(_) => true,
            _                => false,
        }
    }

    pub(crate) fn as_list(&self) -> Option<&[MalType]> {
        match self {
            MalType::List(l) => Some(&l[..]),
            _                => None,
        }
    }

    pub(crate) fn into_list(self) -> Option<Vec<MalType>> {
        match self {
            MalType::List(l) => Some(l),
            _                => None,
        }
    }

    pub(crate) fn is_symbol(&self) -> bool {
        match self {
            MalType::Symbol(_) => true,
            _                  => false,
        }
    }

    pub(crate) fn as_symbol(&self) -> Option<&str> {
        match self {
            MalType::Symbol(s) => Some(s),
            _                  => None,
        }
    }

    pub(crate) fn into_symbol(self) -> Option<String> {
        match self {
            MalType::Symbol(s) => Some(s),
            _                  => None,
        }
    }

    pub(crate) fn is_int(&self) -> bool {
        match self {
            MalType::Int(_) => true,
            _               => false,
        }
    }

    pub(crate) fn as_int(&self) -> Option<&i64> {
        match self {
            MalType::Int(i) => Some(i),
            _               => None,
        }
    }

    pub(crate) fn into_int(self) -> Option<i64> {
        match self {
            MalType::Int(i) => Some(i),
            _                => None,
        }
    }

    pub(crate) fn is_float(&self) -> bool {
        match self {
            MalType::Float(_) => true,
            _                 => false,
        }
    }

    pub(crate) fn as_float(&self) -> Option<&f64> {
        match self {
            MalType::Float(f) => Some(f),
            _                 => None,
        }
    }

    pub(crate) fn into_float(self) -> Option<f64> {
        match self {
            MalType::Float(f) => Some(f),
            _                => None,
        }
    }

    pub(crate) fn is_bool(&self) -> bool {
        match self {
            MalType::Bool(_) => true,
            _                => false,
        }
    }

    pub(crate) fn as_bool(&self) -> Option<&bool> {
        match self {
            MalType::Bool(b) => Some(b),
            _                => None,
        }
    }

    pub(crate) fn into_bool(self) -> Option<bool> {
        match self {
            MalType::Bool(b) => Some(b),
            _                => None,
        }
    }

    pub(crate) fn is_str(&self) -> bool {
        match self {
            MalType::Str(_) => true,
            _               => false,
        }
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            MalType::Str(s) => Some(s),
            _               => None,
        }
    }

    pub(crate) fn into_str(self) -> Option<String> {
        match self {
            MalType::Str(s) => Some(s),
            _                => None,
        }
    }

    pub(crate) fn is_nil(&self) -> bool {
        self == &MalType::Nil
    }
}
