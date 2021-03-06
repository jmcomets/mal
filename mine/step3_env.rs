// #![deny(warnings)]

use std::io;

#[macro_use] extern crate lazy_static;

use rustyline::{Editor, error::ReadlineError};

mod env;
#[macro_use] mod macros;
mod printer;
mod reader;
mod types;

use env::{Env, EnvRef};

use types::{
    MalType as AST,
    MalError as ASTError,
};

fn read(s: &str) -> Result<Option<AST>, ASTError> {
    reader::read_str(s)
}

use ASTError::*;

fn eval_def(args: &[AST], env: &mut EnvRef) -> Result<AST, ASTError> {
    if args.len() == 2 {
        if let AST::Symbol(symbol) = &args[0] {
            let value = eval(&args[1], env)?;
            env.set(symbol.to_string(), value.clone());
            Ok(value)
        } else {
            Err(CanOnlyDefineSymbols(args[0].clone()))
        }
    } else {
        Err(ArityError {
            expected: 2,
            reached: args.len(),
        })
    }
}

fn eval_let(args: &[AST], env: &EnvRef) -> Result<AST, ASTError> {
    if args.len() == 2 {
        match &args[0] {
            AST::List(bindings) | AST::Vector(bindings) => {
                let mut new_env = EnvRef::refer_to(env.clone());
                for let_args in bindings.chunks(2) {
                    if let AST::Symbol(symbol) = &let_args[0] {
                        let value = eval(&let_args[1], &mut new_env)?;
                        new_env.set(symbol.to_string(), value);
                    } else {
                        return Err(CanOnlyDefineSymbols(let_args[0].clone()));
                    }
                }
                eval(&args[1], &mut new_env)
            }
            _ => unimplemented!()
        }
    } else {
        Err(ArityError {
            expected: 2,
            reached: args.len(),
        })
    }
}

fn eval_ast(ast: &AST, env: &mut EnvRef) -> Result<AST, ASTError> {
    match ast {
        AST::Symbol(symbol) => {
            Ok(env.get(&symbol[..])
                .unwrap_or(ast.clone()))
        }

        AST::List(elements) => {
            let mut evals = vec![];
            for element in elements {
                let evaluated = eval(element, env)?;
                evals.push(evaluated);
            }
            Ok(AST::List(evals))
        }

        AST::Vector(elements) => {
            let mut evals = vec![];
            for element in elements {
                let evaluated = eval(element, env)?;
                evals.push(evaluated);
            }
            Ok(AST::Vector(evals))
        }

        ast @ _ => Ok(ast.clone()),
    }
}

fn eval_apply(ast: &AST) -> Result<AST, ASTError> {
    if let AST::List(elems) = ast {
        match &elems[0] {
            AST::Symbol(symbol)          => Err(SymbolNotFound(symbol.to_string())),
            ast @ _                      => Err(NotEvaluable(ast.clone())),
        }
    } else {
        unreachable!()
    }
}

fn eval(ast: &AST, env: &mut EnvRef) -> Result<AST, ASTError> {
    if let AST::List(elems) = ast {
        if !elems.is_empty() {
            if let AST::Symbol(symbol) = &elems[0] {
                if symbol == "def!" {
                    return eval_def(&elems[1..], env);
                } else if symbol == "let*" {
                    return eval_let(&elems[1..], env);
                }
            }
            eval_apply(&eval_ast(ast, env)?)
        } else {
            Ok(AST::List(vec![]))
        }
    } else {
        eval_ast(ast, env)
    }
}

fn print(ast: &AST) -> String {
    printer::pr_str(&ast, true)
}

fn eval_print(ast: &AST, env: &mut EnvRef) -> String {
    match eval(ast, env) {
        Ok(ast) => print(&ast),
        Err(e)  => {
            match e {
                CanOnlyDefineSymbols(ast)        => format!("can only def! symbols (not '{}')", print(&ast)),
                NotEvaluable(ast)                => format!("cannot evaluate '{}'", print(&ast)),
                SymbolNotFound(symbol)           => format!("symbol '{}' not found", symbol),
                TypeCheckFailed {}               => format!("typecheck failed"),
                ArityError { expected, reached } =>
                    format!("arity error, tried to call symbol expecting {} arguments with {}", reached, expected),
                _                                => unimplemented!(),
            }
        }
    }
}

fn rep(s: &str, env: &mut EnvRef) -> String {
    match read(s) {
        Ok(Some(ast))         => eval_print(&ast, env),
        Ok(None)              => "EOF".to_string(),
        Err(UnbalancedString) => "unbalanced string".to_string(),
        Err(UnbalancedList)   => "unbalanced list".to_string(),
        _                     => unimplemented!(),
    }
}

fn default_env() -> Env {
    let mut env = Env::new();
    env.set("+".to_string(), binary_operator!(Number + Number -> Number));
    env.set("-".to_string(), binary_operator!(Number - Number -> Number));
    env.set("*".to_string(), binary_operator!(Number * Number -> Number));
    env.set("/".to_string(), binary_operator!(Number / Number -> Number));
    env
}

fn main() -> io::Result<()> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    let _ = rl.load_history(".mal-history");

    let mut env = EnvRef::new(default_env());

    // let mut line = String::new();
    loop {
        match rl.readline("user> ") {
            Ok(line) => {
                rl.add_history_entry(line.to_string());
                rl.save_history(".mal-history").unwrap();
                if line.len() > 0 {
                    println!("{}", rep(&line, &mut env));
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
