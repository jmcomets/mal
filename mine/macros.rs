macro_rules! make_list {
    ($($item:expr),*) => {
        #[allow(unused_mut)] {
            let mut v = im::Vector::new();
            $(
                v.push_back($item);
            )*
            $crate::types::MalType::List(v)
        }
    }
}

macro_rules! make_function {
    ($f:expr) => {
        {
            #[allow(unused_imports)]
            use $crate::types::{
                MalArgs,
                MalError::*,
                MalNumber::{self as Number, *},
                MalResult,
                MalType::{self, *},
            };

            Function(std::rc::Rc::new($f))
        }
    }
}

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

macro_rules! bind_args {
    ($args:expr, $($binding:tt $( : $binding_type:tt )? ),*) => {
        let nb_args = 0 $(+ {stringify!($binding); 1})*;
        expect_arity!($args, nb_args);

        let mut it = $args.into_iter();
        $(
            let $binding = it.next().unwrap();

            $(
                let $binding = {
                    if let $binding_type($binding) = $binding {
                        $binding
                    } else {
                        return Err(TypeCheckFailed{});
                    }
                };
            )?
        )*
    };
}

macro_rules! function {
    ($($arg:tt $( : $arg_type:tt )? ),* $body:block) => {
        make_function!({
            move |args: MalArgs| -> MalResult {
                bind_args!(args, $($arg $( : $arg_type )? ),*);
                $body
            }
        })
    };

    ($($arg:tt $( : $arg_type:tt )? ),* -> $rettype:tt $body:block) => {
        make_function!({
            move |args: MalArgs| -> MalResult {
                bind_args!(args, $($arg $( : $arg_type )? ),*);
                $body.map($rettype)
            }
        })
    }
}

macro_rules! variadic_function {
    ($args:tt $body:block) => {
        make_function!({
            move |$args: MalArgs| -> MalResult {
                $body
            }
        })
    };
}

macro_rules! binary_operator {
    ($left:tt $op:tt $right:tt -> $out:tt) => {
        function!(a: $left, b: $right -> $out { Ok(a $op b) })
    }
}
