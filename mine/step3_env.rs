#[macro_use] extern crate lazy_static;

// #![deny(warnings)]

use std::io::{
    self,
    Write
};

mod printer;
mod reader;
mod types;
mod env;

use env::Env;

fn read(s: &str) -> Result<Option<types::MalType>, reader::Error> {
    reader::read_str(s)
}

#[derive(Debug)]
struct EvalError;

fn eval(ast: types::MalType, env: &Env) -> Result<types::MalType, EvalError> {
    if let types::MalType::List(elements) = ast {
        let mut evaluated_elements = vec![];
        for element in elements {
            let evaluated = eval(element, env)?;
            evaluated_elements.push(evaluated);
        }

        if !evaluated_elements.is_empty() {
            let symbol = evaluated_elements[0].as_symbol().unwrap();
            env.get(symbol)
                .map(|callable| callable.call(&evaluated_elements[1..]))
                .ok_or(EvalError)
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

    // env.set("+", arithmetic_operation!(+));
    // env.set("-", arithmetic_operation!(-));
    // env.set("*", arithmetic_operation!(*));
    // env.set("/", arithmetic_operation!(/));

    match read(s) {
        Ok(Some(t))                          => eval(t, &env).map(print).unwrap_or("evaluation error".to_string()),
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
