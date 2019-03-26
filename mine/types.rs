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

#[allow(unused_macros)]
macro_rules! make_function {
    ($f:expr) => {
        {
            #[allow(unused_imports)]
            use $crate::types::{
                MalType::{self, *},
                MalError::*,
                MalResult,
            };

            Function(std::rc::Rc::new($f))
        }
    }
}

#[allow(unused_macros)]
macro_rules! bind_args {
    ($args:expr, $($binding:tt),*) => {
        let nb_args = 0 $(+ {stringify!($binding); 1})*;
        if $args.len() != nb_args {
            return Err(ArityError {
                expected: nb_args,
                reached: $args.len(),
            });
        }

        let mut arg_index = 0;
        $(
            let $binding = &$args[arg_index];

            #[allow(unused)] {
                arg_index += 1;
            }
        )*
    };

    ($args:expr, $($arg:tt : $argtype:tt),*) => {
        let nb_args = 0 $(+ {stringify!($arg); 1})*;
        if $args.len() != nb_args {
            return Err(ArityError {
                expected: nb_args,
                reached: $args.len(),
            });
        }

        let mut arg_index = 0;
        $(
            let $arg = {
                if let $argtype($arg) = $args[arg_index] {
                    $arg
                } else {
                    return Err(TypeCheckFailed{});
                }
            };

            #[allow(unused)] {
                arg_index += 1;
            }
        )*
    }
}

#[allow(unused_macros)]
macro_rules! function {
    ($($arg:tt),* -> Nil $body:block) => {
        make_function!({
            |args: &[MalType]| -> MalResult {
                bind_args!(args, $($arg),*);
                $body;
                Ok(Nil)
            }
        })
    };

    ($($arg:tt),* -> $rettype:tt $body:block) => {
        make_function!({
            |args: &[MalType]| -> MalResult {
                bind_args!(args, $($arg),*);
                $body.map($rettype)
            }
        })
    };

    ($($arg:tt : $argtype:tt),* -> $rettype:tt $body:block) => {
        make_function!({
            |args: &[MalType]| -> MalResult {
                bind_args!(args, $($arg : $argtype),*);
                Ok($rettype($body))
            }
        })
    }
}

#[allow(unused_macros)]
macro_rules! function_chain {
    ($($f:expr),*) => {
        make_function!({
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
