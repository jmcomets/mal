// #![deny(warnings)]

#[macro_use] extern crate lazy_static;

use std::io::{
    self,
    Write
};

mod printer;
mod reader;
mod types;

#[macro_use] mod env;

use env::{Env, EnvValue};
use types::MalType as AST;
use reader::Error as ReadError;

fn read(s: &str) -> Result<Option<AST>, ReadError> {
    reader::read_str(s)
}

#[derive(Debug)]
enum EvalError {
    NotEvaluable(AST),
    CannotDefCallable,
    CanOnlyDefSymbol(AST),
    SymbolNotFound(String),
    ArityError {
        // symbol: String,
        expected: usize,
        reached: usize,
    }
}

fn eval_ast<'a>(ast: AST, env: &Env) -> Result<EnvValue, EvalError> {
    match ast.clone() {
        AST::Symbol(symbol) => {
            Ok(env.get(&symbol)
                .unwrap_or(EnvValue::Value(ast)))
        }

        AST::List(elements) => {
            let mut evals = vec![];
            for element in elements {
                let evaluated = eval(element, env)?;
                evals.push(evaluated);
            }
            Ok(EnvValue::Value(AST::List(evals)))
        }

        ast @ _ => Ok(EnvValue::Value(ast)),
    }
}

fn eval_def(args: &[EnvValue], env: &Env) -> Result<AST, EvalError> {
    use EvalError::*;
    if args.len() == 2 {
        let key = &args[0];
        let value = &args[1];
        match key {
            EnvValue::Value(AST::Symbol(symbol)) => {
                env.set(symbol.to_string(), value.clone());
                Ok(match value {
                    EnvValue::Value(ast) => ast.clone(),
                    _                    => AST::Nil,
                })
            }

            EnvValue::Callable { .. } => Err(CannotDefCallable),
            EnvValue::Value(ast @ _)  => Err(CanOnlyDefSymbol(ast.clone())),
        }
    } else {
        Err(ArityError {
            expected: 2,
            reached: args.len(),
        })
    }
}

fn eval_symbol(symbol: &EnvValue, args: &[EnvValue], env: &Env) -> Result<AST, EvalError> {
    use EvalError::*;
    match symbol {
        EnvValue::Value(AST::Symbol(symbol)) => {
            if symbol == "def!" {
                eval_def(args, env)
            } else if symbol == "let*" {
                unimplemented!()
            } else {
                Err(SymbolNotFound(symbol.to_string()))
            }
        }

        EnvValue::Value(ast @ _) => Err(EvalError::NotEvaluable(ast.clone())),

        EnvValue::Callable { arity, delegate } => {
            if *arity == args.len() {
                Ok(delegate(args))
            } else {
                Err(ArityError {
                    expected: *arity,
                    reached: args.len(),
                })
            }
        }
    }
}

fn eval(ast: AST, env: &Env) -> Result<AST, EvalError> {
    if let AST::List(elems) = ast {
        if !elems.is_empty() {
            let mut evals = vec![];
            for element in elems {
                let evaluated = eval_ast(element, env)?;
                evals.push(evaluated);
            }

            let symbol = &evals[0];
            let args = &evals[1..];
            eval_symbol(symbol, args, env)
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

fn eval_print(ast: types::MalType, env: &Env) -> String {
    match eval(ast, env) {
        Ok(ast) => print(ast),
        Err(e)  => {
            use EvalError::*;
            match e {
                CannotDefCallable      => "cannot def! a callable".to_string(),
                CanOnlyDefSymbol(ast)  => format!("can only def! symbols (not '{}')", print(ast)),
                NotEvaluable(ast)      => format!("cannot evaluate '{}'", print(ast)),
                SymbolNotFound(symbol) => format!("symbol '{}' not found", symbol),
                ArityError { expected, reached } =>
                    format!("arity error, tried to call symbol expecting {} arguments with {}", reached, expected),
            }
        }
    }
}

fn rep(s: &str, env: &Env) -> String {
    use ReadError::*;
    match read(s) {
        Ok(Some(ast))         => eval_print(ast, env),
        Ok(None)              => "EOF".to_string(),
        Err(UnbalancedString) => "unbalanced string".to_string(),
        Err(UnbalancedList)   => "unbalanced list".to_string(),
    }
}

fn default_env() -> Env {
    let env = Env::new();
    env.set("+".to_string(), arithmetic_operations!(+));
    env.set("-".to_string(), arithmetic_operations!(-));
    env.set("*".to_string(), arithmetic_operations!(*));
    env.set("/".to_string(), arithmetic_operations!(/));
    env.set("+".to_string(), arithmetic_operations!(+));
    env.set("-".to_string(), arithmetic_operations!(-));
    env.set("*".to_string(), arithmetic_operations!(*));
    env.set("/".to_string(), arithmetic_operations!(/));
    env
}

fn main() -> io::Result<()> {
    let input = io::stdin();
    let mut output = io::stdout();

    let env = default_env();

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
        writeln!(&mut output, "{}", rep(line, &env))?;
        output.flush()?;
    }

    Ok(())
}
