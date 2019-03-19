// #![deny(warnings)]

use std::io;

#[macro_use] extern crate lazy_static;

use rustyline::{Editor, error::ReadlineError};

mod printer;
mod reader;
#[macro_use] mod types;
mod env;
mod core;

use std::rc::Rc;
use env::{Env, EnvRef};

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

fn eval_def(args: &[AST], env: &mut EnvRef) -> Result<AST, EvalError> {
    if args.len() == 2 {
        if let AST::Symbol(symbol) = &args[0] {
            let value = eval(&args[1], env)?;
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

fn eval_let(args: &[AST], env: &EnvRef) -> Result<AST, EvalError> {
    if args.len() == 2 {
        match &args[0] {
            AST::List(bindings) | AST::Vector(bindings) => {
                let mut new_env = EnvRef::refer_to(env.clone());
                for let_args in bindings.chunks(2) {
                    if let AST::Symbol(symbol) = &let_args[0] {
                        let value = eval(&let_args[1], &mut new_env)?;
                        new_env.set(symbol.to_string(), value);
                    } else {
                        return Err(CanOnlyLetSymbol(let_args[0].clone()));
                    }
                }
                eval(&args[1], &mut new_env)
            }
            _ => unimplemented!()
        }
    } else {
        Err(ASTError(ArityError {
            expected: 2,
            reached: args.len(),
        }))
    }
}

fn eval_do(args: &[AST], env: &mut EnvRef) -> Result<AST, EvalError> {
    let mut ret = AST::Nil;
    for arg in args {
        ret = eval(arg, env)?;
    }
    Ok(ret)
}

fn eval_if(args: &[AST], env: &mut EnvRef) -> Result<AST, EvalError> {
    if args.len() < 2 {
        return Err(ASTError(ArityError {
            expected: 2,
            reached: args.len(),
        }));
    }

    if args.len() > 3 {
        return Err(ASTError(ArityError {
            expected: 3,
            reached: args.len(),
        }));
    }

    // A temporary env is used to prevent mutation when evaluating the condition
    let mut condition_env = EnvRef::refer_to(env.clone());

    match eval(&args[0], &mut condition_env)? {
        AST::Nil | AST::Bool(false) => {
            if args.len() > 2 {
                eval(&args[2], env)
            } else {
                Ok(AST::Nil)
            }
        }

        _ => eval(&args[1], env),
    }
}

fn eval_fn(args: &[AST], env: &EnvRef) -> Result<AST, EvalError> {
    if args.len() != 2 {
        return Err(ASTError(ArityError {
            expected: 2,
            reached: args.len(),
        }));
    }

    match &args[0] {
        AST::List(bindings) | AST::Vector(bindings) => {
            let mut symbols = vec![];
            for value in bindings {
                match value {
                    AST::Symbol(symbol) => {
                        symbols.push(symbol.clone());
                    }

                    _ => unimplemented!() // can only name args using symbols
                }
            }

            let body = args[1].clone();
            let captured_env = EnvRef::refer_to(env.clone());
            let f = move |args: &[AST]| -> Result<AST, ASTError> {
                if args.len() != symbols.len() {
                    return Err(ArityError {
                        expected: symbols.len(),
                        reached: args.len(),
                    });
                }

                let mut call_env = EnvRef::refer_to(captured_env.clone());
                for (symbol, value) in symbols.iter().zip(args.iter()) {
                    call_env.set(symbol.clone(), value.clone());
                }

                eval(&body, &mut call_env)
                    .map_err(|_| CallError)
            };

            Ok(AST::Function(Rc::new(f)))
        }

        _ => unimplemented!() // cannot read args from anything other than a list/vector
    }
}

fn eval_ast(ast: &AST, env: &mut EnvRef) -> Result<AST, EvalError> {
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

fn eval_apply(ast: &AST) -> Result<AST, EvalError> {
    if let AST::List(elems) = ast {
        match &elems[0] {
            AST::Function(func)          => func(&elems[1..]).map_err(ASTError),
            AST::Symbol(symbol)          => Err(SymbolNotFound(symbol.to_string())),
            ast @ _                      => Err(NotEvaluable(ast.clone())),
        }
    } else {
        unreachable!()
    }
}

fn eval(ast: &AST, env: &mut EnvRef) -> Result<AST, EvalError> {
    if let AST::List(elems) = ast {
        if !elems.is_empty() {
            if let AST::Symbol(symbol) = &elems[0] {
                let args = &elems[1..];
                if symbol == "def!" {
                    return eval_def(args, env);
                } else if symbol == "let*" {
                    return eval_let(args, env);
                } else if symbol == "do" {
                    return eval_do(args, env);
                } else if symbol == "if" {
                    return eval_if(args, env);
                } else if symbol == "fn*" {
                    return eval_fn(args, env);
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
                CanOnlyDefSymbol(ast)                      => format!("can only def! symbols (not '{}')", print(&ast)),
                CanOnlyLetSymbol(ast)                      => format!("can only let! symbols (not '{}')", print(&ast)),
                NotEvaluable(ast)                          => format!("cannot evaluate '{}'", print(&ast)),
                SymbolNotFound(symbol)                     => format!("symbol '{}' not found", symbol),
                ASTError(CallError)                        => format!("call error"),
                ASTError(TypeCheckFailed {})               => format!("typecheck failed"),
                ASTError(ArityError { expected, reached }) =>
                    format!("arity error, tried to call symbol expecting {} arguments with {}", expected, reached),
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
    }
}

fn main() -> io::Result<()> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    let _ = rl.load_history(".mal-history");

    let mut env = Env::new();
    for (symbol, value) in core::ns() {
        env.set(symbol, value);
    }

    let mut env = EnvRef::new(env);

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
