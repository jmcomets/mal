// #![deny(warnings)]

#[macro_use] extern crate lazy_static;

use rustyline::{Editor, error::ReadlineError};

use std::io;

mod printer;
mod reader;
#[macro_use] mod types;
mod env;

use env::Env;

use types::{
    MalType as AST,
    MalError as ASTError,
};
use reader::Error as ReadError;

fn read(s: &str) -> Result<Option<AST>, ReadError> {
    reader::read_str(s)
}

enum EvalError {
    NotEvaluable(AST),
    CanOnlyDefSymbol(AST),
    SymbolNotFound(String),
    ASTError(ASTError),
}
use EvalError::*;
use ReadError::*;
use ASTError::*;

fn eval_ast<'a>(ast: AST, env: &Env) -> Result<AST, EvalError> {
    match ast.clone() {
        AST::Symbol(symbol) => {
            Ok(env.get(&symbol)
                .unwrap_or(ast))
        }

        AST::List(elements) => {
            let mut evals = vec![];
            for element in elements {
                let evaluated = eval(element, env)?;
                evals.push(evaluated);
            }
            Ok(AST::List(evals))
        }

        ast @ _ => Ok(ast),
    }
}

fn eval_def(args: &[AST], env: &Env) -> Result<AST, EvalError> {
    if args.len() == 2 {
        let key = &args[0];
        let value = &args[1];
        match key {
            AST::Symbol(symbol) => {
                env.set(symbol.to_string(), value.clone());
                Ok(value.clone())
            }

            ast @ _  => Err(CanOnlyDefSymbol(ast.clone())),
        }
    } else {
        Err(ASTError(ArityError {
            expected: 2,
            reached: args.len(),
        }))
    }
}

fn eval_symbol(symbol: &AST, args: &[AST], env: &Env) -> Result<AST, EvalError> {
    match symbol {
        AST::Symbol(symbol) => {
            if symbol == "def!" {
                eval_def(args, env)
            } else if symbol == "let*" {
                unimplemented!()
            } else {
                Err(SymbolNotFound(symbol.to_string()))
            }
        }

        AST::NativeFunc { func, .. } => func(args).map_err(ASTError),

        ast @ _ => Err(EvalError::NotEvaluable(ast.clone())),
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

fn print(ast: AST) -> String {
    printer::pr_str(&ast)
}

fn eval_print(ast: AST, env: &Env) -> String {
    match eval(ast, env) {
        Ok(ast) => print(ast),
        Err(e)  => {
            match e {
                CanOnlyDefSymbol(ast)  => format!("can only def! symbols (not '{}')", print(ast)),
                NotEvaluable(ast)      => format!("cannot evaluate '{}'", print(ast)),
                SymbolNotFound(symbol) => format!("symbol '{}' not found", symbol),
                ASTError(TypeCheckFailed {}) => format!("typecheck failed"),
                ASTError(ArityError { expected, reached }) =>
                    format!("arity error, tried to call symbol expecting {} arguments with {}", reached, expected),
            }
        }
    }
}

fn rep(s: &str, env: &Env) -> String {
    match read(s) {
        Ok(Some(ast))         => eval_print(ast, env),
        Ok(None)              => "EOF".to_string(),
        Err(UnbalancedString) => "unbalanced string".to_string(),
        Err(UnbalancedList)   => "unbalanced list".to_string(),
    }
}

fn default_env() -> Env {
    let env = Env::new();
    env.set("+".to_string(), binary_operator!(Int + Int -> Int));
    env.set("-".to_string(), binary_operator!(Int - Int -> Int));
    env.set("*".to_string(), binary_operator!(Int * Int -> Int));
    env.set("/".to_string(), binary_operator!(Int / Int -> Int));
    env
}

fn main() -> io::Result<()> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    let _ = rl.load_history(".mal-history");

    let env = default_env();

    // let mut line = String::new();
    loop {
        match rl.readline("user> ") {
            Ok(line) => {
                rl.add_history_entry(line.to_string());
                rl.save_history(".mal-history").unwrap();
                if line.len() > 0 {
                    println!("{}", rep(&line, &env));
                }
            },
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof)         => break,
            Err(err) => {
                eprintln!("readline error: {:?}", err);
                break
            }
        }
    }

    Ok(())
}


