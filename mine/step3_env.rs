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
use types::MalType as AST;
use reader::Error as ReadError;

fn read(s: &str) -> Result<Option<AST>, ReadError> {
    reader::read_str(s)
}

#[derive(Debug)]
enum EvalError {
    NotEvaluable(AST),
    SymbolNotFound(String),
    ArityError {
        symbol: String,
        expected: usize,
        reached: usize,
    }
}

fn eval_symbol(symbol: String, args: &[AST], env: &mut Env) -> Result<AST, EvalError> {
    use EvalError::*;
    if symbol == "def!" {
        unimplemented!()
    } else if symbol == "let*" {
        unimplemented!()
    } else if let Some(env_value) = env.get(&symbol) {
        env_value.try_call(args)
            .map_err(|e| {
                ArityError {
                    symbol: symbol,
                    expected: e.expected,
                    reached: e.reached,
                }
            })
    } else {
        Err(SymbolNotFound(symbol))
    }
}

fn eval(ast: AST, env: &mut Env) -> Result<AST, EvalError> {
    if let AST::List(elements) = ast {
        let mut evaluated_elements = vec![];
        for element in elements {
            let evaluated = eval(element, env)?;
            evaluated_elements.push(evaluated);
        }

        if !evaluated_elements.is_empty() {
            match &evaluated_elements[0] {
                AST::Symbol(symbol) => eval_symbol(symbol.clone(), &evaluated_elements[1..], env),
                ast @ _             => Err(EvalError::NotEvaluable(ast.clone())),
            }
        } else {
            Ok(AST::List(vec![]))
        }
    } else {
        Ok(ast)
    }
}

fn print(ast: types::MalType) -> String {
    printer::pr_str(&ast)
}

fn eval_print(ast: types::MalType, env: &mut Env) -> String {
    match eval(ast, env) {
        Ok(ast) => print(ast),
        Err(e)  => {
            use EvalError::*;
            match e {
                NotEvaluable(ast)      => format!("cannot evaluate '{}'", print(ast)),
                SymbolNotFound(symbol) => format!("symbol '{}' not found", symbol),
                ArityError { symbol, expected, reached } =>
                    format!("cannot call '{}' with {} args (expected {})", symbol, reached, expected),
            }
        }
    }
}

fn rep(s: &str) -> String {
    let mut env = Env::new();

    env.set("+".to_string(), arithmetic_operations!(+));
    env.set("-".to_string(), arithmetic_operations!(-));
    env.set("*".to_string(), arithmetic_operations!(*));
    env.set("/".to_string(), arithmetic_operations!(/));

    use ReadError::*;
    match read(s) {
        Ok(Some(ast))         => eval_print(ast, &mut env),
        Ok(None)              => "EOF".to_string(),
        Err(UnbalancedString) => "unbalanced string".to_string(),
        Err(UnbalancedList)   => "unbalanced list".to_string(),
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
