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
    CanOnlyLetSymbol(AST),
    SymbolNotFound(String),
    ASTError(ASTError),
}
use EvalError::*;
use ReadError::*;
use ASTError::*;

fn eval_def(args: &[AST], env: &Env) -> Result<AST, EvalError> {
    if args.len() == 2 {
        if let AST::Symbol(symbol) = &args[0] {
            let value = eval(args[1].clone(), env)?;
            env.set(symbol.to_string(), value.clone());
            Ok(value)
        } else {
            Err(CanOnlyDefSymbol(args[0].clone()))
        }
    } else {
        Err(ASTError(ArityError {
            expected: 2,
            reached: args.len(),
        }))
    }
}

fn eval_let(args: &[AST], env: &Env) -> Result<AST, EvalError> {
    let new_env = Env::wrap(env);
    if args.len() == 2 {
        if let AST::List(bindings) = &args[0] {
            for let_args in bindings.chunks(2) {
                if let AST::Symbol(symbol) = &let_args[0] {
                    let value = eval(let_args[1].clone(), &new_env)?;
                    new_env.set(symbol.to_string(), value);
                } else {
                    return Err(CanOnlyLetSymbol(let_args[0].clone()));
                }
            }
            eval(args[1].clone(), &new_env)
        } else {
            unimplemented!()
        }
    } else {
        Err(ASTError(ArityError {
            expected: 2,
            reached: args.len(),
        }))
    }
}

fn eval_ast(ast: AST, env: &Env) -> Result<AST, EvalError> {
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

fn eval_apply(ast: AST) -> Result<AST, EvalError> {
    if let AST::List(elems) = ast {
        match &elems[0] {
            AST::NativeFunc { func, .. } => func(&elems[1..]).map_err(ASTError),
            AST::Symbol(symbol)          => Err(SymbolNotFound(symbol.to_string())),
            ast @ _                      => Err(NotEvaluable(ast.clone())),
        }
    } else {
        unreachable!()
    }
}

fn eval(ast: AST, env: &Env) -> Result<AST, EvalError> {
    if let AST::List(elems) = ast.clone() {
        if !elems.is_empty() {
            if let AST::Symbol(symbol) = &elems[0] {
                if symbol == "def!" {
                    return eval_def(&elems[1..], env);
                } else if symbol == "let*" {
                    return eval_let(&elems[1..], env);
                }
            }
            eval_apply(eval_ast(ast, env)?)
        } else {
            Ok(AST::List(vec![]))
        }
    } else {
        eval_ast(ast, &env)
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
                CanOnlyLetSymbol(ast)  => format!("can only let! symbols (not '{}')", print(ast)),
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

fn main() -> io::Result<()> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    let _ = rl.load_history(".mal-history");

    let env = Env::new();
    env.set("+".to_string(), binary_operator!(Int + Int -> Int));
    env.set("-".to_string(), binary_operator!(Int - Int -> Int));
    env.set("*".to_string(), binary_operator!(Int * Int -> Int));
    env.set("/".to_string(), binary_operator!(Int / Int -> Int));

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


