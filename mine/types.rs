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

#[allow(unused_macros)]
macro_rules! make_function {
    ($f:expr) => {
        $crate::types::MalType::Function(std::rc::Rc::new($f))
    }
}

#[allow(unused_macros)]
macro_rules! function {
    ($($arg:tt : $argtype:tt),* -> $rettype:tt $body:block) => {
        make_function!({
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
        })
    }
}

#[allow(unused_macros)]
macro_rules! function_chain {
    ($($f:expr),*) => {
        make_function!({
            use $crate::types::{
                MalType::{self, Function},
                MalError::*,
                MalResult,
            };

            let functions = vec![$($f, )*];

            move |args: &[MalType]| -> MalResult {
                for f in functions.iter() {
                    if let Function(f) = f {
                        match f(args) {
                            Err(TypeCheckFailed{}) => {},
                            ret @ Ok(_) | ret @ Err(_)   => return ret,
                        }
                    } else {
                        panic!("expected function, got {:?}", f);
                    }
                }

                Err(TypeCheckFailed{})
            }
        })
    }
}

#[allow(unused_macros)]
macro_rules! binary_operator {
    ($left:tt $op:tt $right:tt -> $out:tt) => {
        function!(a: $left, b: $right -> $out { a $op b })
    }
}

#[allow(unused_macros)]
macro_rules! number_operator {
    ($op:tt) => {
        function_chain!(
            function!(a: Int, b: Int -> Int { a $op b }),
            function!(a: Float, b: Float -> Float { a $op b }),
            function!(a: Int, b: Float -> Float { a as f64 $op b }),
            function!(a: Float, b: Int -> Float { a $op b as f64  })
        )
    }
}

#[allow(unused_macros)]
macro_rules! number_predicate {
    ($op:tt) => {
        function_chain!(
            function!(a: Int, b: Int -> Bool { a $op b }),
            function!(a: Float, b: Float -> Bool { a $op b }),
            function!(a: Int, b: Float -> Bool { (a as f64) $op b }),
            function!(a: Float, b: Int -> Bool { a $op (b as f64)  })
        )
    }
}
