#[allow(unused_macros)]
macro_rules! expect_arity {
    ($args:expr, $($expected:expr),*) => {
        #[allow(unused_assignments, unused_variables)] {
            $(
                let mut expected = Some($expected);
                if $args.len() == $expected {
                    expected = None;
                }
            )*

            if let Some(expected) = expected {
                return Err($crate::types::MalError::ArityError {
                    expected: expected,
                    reached: $args.len(),
                });
            }
        }
    }
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
                if let $argtype($arg) = &$args[arg_index] {
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
    ($($arg:tt),* $body:block) => {
        make_function!({
            |args: &[MalType]| -> MalResult {
                bind_args!(args, $($arg),*);
                $body
            }
        })
    };

    ($($arg:tt : $argtype:tt),* $body:block) => {
        make_function!({
            |args: &[MalType]| -> MalResult {
                bind_args!(args, $($arg : $argtype),*);
                $body
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
                $body.map($rettype)
            }
        })
    }
}

#[allow(unused_macros)]
macro_rules! variadic_function {
    ($args:tt $body:block) => {
        make_function!({
            |$args: &[MalType]| -> MalResult {
                $body
            }
        })
    };
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
        function!(a: $left, b: $right -> $out { Ok((*a) $op (*b)) })
    }
}

#[allow(unused_macros)]
macro_rules! number_operator {
    ($op:tt) => {
        function_chain!(
            function!(a: Int,   b: Int   -> Int   { Ok((*a) $op (*b)) }),
            function!(a: Float, b: Float -> Float { Ok((*a) $op (*b)) }),
            function!(a: Int,   b: Float -> Float { Ok((*a as f64) $op (*b)) }),
            function!(a: Float, b: Int   -> Float { Ok((*a) $op (*b as f64))  })
        )
    }
}

#[allow(unused_macros)]
macro_rules! number_predicate {
    ($op:tt) => {
        function_chain!(
            function!(a: Int,   b: Int   -> Bool { Ok((*a) $op (*b)) }),
            function!(a: Float, b: Float -> Bool { Ok((*a) $op (*b)) }),
            function!(a: Int,   b: Float -> Bool { Ok((*a as f64) $op (*b)) }),
            function!(a: Float, b: Int   -> Bool { Ok((*a) $op (*b as f64))  })
        )
    }
}
