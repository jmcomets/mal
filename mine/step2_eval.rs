#[macro_use] extern crate lazy_static;

// #![deny(warnings)]

use std::collections::HashMap;
use std::io::{
    self,
    Write
};

mod printer;
mod reader;
mod types;

fn read(s: &str) -> Result<Option<types::MalType>, types::MalError> {
    reader::read_str(s)
}

struct Callable<'a>(Box<'a + Fn(&[types::MalType]) -> types::MalType>);

impl<'a> Callable<'a> {
    fn new2<F>(f: F) -> Self
        where F: 'a + Fn(&types::MalType, &types::MalType) -> types::MalType,
    {
        Callable(Box::new(move |args| {
            if args.len() != 2 { panic!("function expected 2 arguments, got {}", args.len()); }
            f(&args[0], &args[1])
        }))
    }
}

type ReplEnv = HashMap<String, Callable<'static>>;

#[derive(Debug)]
struct EvalError;

fn eval(ast: types::MalType, repl_env: &ReplEnv) -> Result<types::MalType, EvalError> {
    if let types::MalType::List(elements) = ast {
        let mut evaluated_elements = vec![];
        for element in elements {
            let evaluated = eval(element, repl_env)?;
            evaluated_elements.push(evaluated);
        }

        if !evaluated_elements.is_empty() {
            match &evaluated_elements[0] {
                types::MalType::Symbol(symbol) => {
                    repl_env.get(symbol)
                        .map(|callable| callable.0(&evaluated_elements[1..]))
                        .ok_or(EvalError)
                }

                _ => Err(EvalError),

            }
        } else {
            Ok(types::MalType::List(vec![]))
        }
    } else {
        Ok(ast)
    }
}

fn print(t: types::MalType) -> String {
    printer::pr_str(&t, true)
}

fn rep(s: &str) -> String {
    let mut repl_env = HashMap::new();

    macro_rules! arithmetic_operation {
        ($op:tt) => {
            Callable::new2(|a, b| {
                use types::MalType::*;
                let ret = if let (Number(a), Number(b)) = (a, b) {
                    Some(Number((*a) $op (*b)))
                } else {
                    None
                };

                ret.unwrap()
            })
        }
    }

    repl_env.insert("+".to_string(), arithmetic_operation!(+));
    repl_env.insert("-".to_string(), arithmetic_operation!(-));
    repl_env.insert("*".to_string(), arithmetic_operation!(*));
    repl_env.insert("/".to_string(), arithmetic_operation!(/));

    match read(s) {
        Ok(Some(t))                            => eval(t, &repl_env).map(print).unwrap_or("evaluation error".to_string()),
        Ok(None)                               => "EOF".to_string(),
        Err(types::MalError::UnbalancedString) => "unbalanced string".to_string(),
        Err(types::MalError::UnbalancedList)   => "unbalanced list".to_string(),
        _                                      => unimplemented!(),
    }
}

fn main() -> io::Result<()> {
    let input = io::stdin();
    let mut output = io::stdout();

    let mut line = String::new();
    loop {
        write!(&mut output, "user> ")?;
        output.flush()?;

        line.clear();
        let nb_bytes_read = input.read_line(&mut line)?;
        if nb_bytes_read == 0 {
            break;
        }
        let line = line.trim_end();
        writeln!(&mut output, "{}", rep(line))?;
        output.flush()?;
    }

    Ok(())
}
