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
                MalNumber::{self as Number, *},
                MalResult,
            };

            Function(std::rc::Rc::new($f))
        }
    }
}

#[allow(unused_macros)]
macro_rules! bind_args {
    ($args:expr, $($binding:tt $( : $binding_type:tt )? ),*) => {
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

            $(
                let $binding = {
                    if let $binding_type($binding) = $binding {
                        $binding
                    } else {
                        return Err(TypeCheckFailed{});
                    }
                };
            )?

            #[allow(unused)] {
                arg_index += 1;
            }
        )*
    };
}

#[allow(unused_macros)]
macro_rules! function {
    ($($arg:tt $( : $arg_type:tt )? ),* $body:block) => {
        make_function!({
            move |args: &[MalType]| -> MalResult {
                bind_args!(args, $($arg $( : $arg_type )? ),*);
                $body
            }
        })
    };

    ($($arg:tt $( : $arg_type:tt )? ),* -> $rettype:tt $body:block) => {
        make_function!({
            move |args: &[MalType]| -> MalResult {
                bind_args!(args, $($arg $( : $arg_type )? ),*);
                $body.map($rettype)
            }
        })
    }
}

#[allow(unused_macros)]
macro_rules! variadic_function {
    ($args:tt $body:block) => {
        make_function!({
            move |$args: &[MalType]| -> MalResult {
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
