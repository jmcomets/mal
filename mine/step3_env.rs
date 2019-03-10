#[macro_use] extern crate lazy_static;

// #![deny(warnings)]

use std::io::{
    self,
    Write
};

mod printer;
mod reader;
mod types;

#[macro_use] mod env;

use env::Env;

fn read(s: &str) -> Result<Option<types::MalType>, reader::Error> {
    reader::read_str(s)
}

#[derive(Debug)]
enum EvalError {
    NotEvaluable(types::MalType),
    SymbolNotFound(String),
    ArityError {
        symbol: String,
        expected: usize,
        reached: usize,
    }
}

fn eval(ast: types::MalType, env: &mut Env) -> Result<types::MalType, EvalError> {
    if let types::MalType::List(elements) = ast {
        let mut evaluated_elements = vec![];
        for element in elements {
            let evaluated = eval(element, env)?;
            evaluated_elements.push(evaluated);
        }

        if !evaluated_elements.is_empty() {
                use EvalError::*;
            match &evaluated_elements[0] {
                types::MalType::Symbol(symbol) => {
                    if symbol == "def!" {
                        unimplemented!()
                    } else if symbol == "let*" {
                        unimplemented!()
                    } else if let Some(env_value) = env.get(symbol) {
                        env_value.try_call(&evaluated_elements[1..])
                            .map_err(|e| {
                                ArityError {
                                    symbol: symbol.to_string(),
                                    expected: e.expected,
                                    reached: e.reached,
                                }
                            })
                    } else {
                        Err(SymbolNotFound(symbol.to_string()))
                    }

                }

                t @ _ => Err(NotEvaluable(t.clone())),

            }
        } else {
            Ok(types::MalType::List(vec![]))
        }
    } else {
        Ok(ast)
    }
}

fn print(t: types::MalType) -> String {
    printer::pr_str(&t)
}

fn rep(s: &str) -> String {
    let mut env = Env::new();

    env.set("+".to_string(), arithmetic_operations!(+));
    env.set("-".to_string(), arithmetic_operations!(-));
    env.set("*".to_string(), arithmetic_operations!(*));
    env.set("/".to_string(), arithmetic_operations!(/));

    match read(s) {
        Ok(Some(t)) => {
            match eval(t, &mut env) {
                Ok(t) => print(t),
                Err(e) => {
                    use EvalError::*;
                    match e {
                        NotEvaluable(_t) => "evaluation error".to_string(), // TODO add `print(t)`
                        SymbolNotFound(symbol) => format!("symbol {} not found", symbol),
                        ArityError { symbol, expected, reached } => format!("cannot call {} with {} args (expected {})", symbol, reached, expected),
                    }
                }
            }
        }
        Ok(None)                             => "EOF".to_string(),
        Err(reader::Error::UnbalancedString) => "unbalanced string".to_string(),
        Err(reader::Error::UnbalancedList)   => "unbalanced list".to_string(),
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
