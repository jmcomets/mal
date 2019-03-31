// #![deny(warnings)]

use std::io;
use std::iter::FromIterator;

#[macro_use] extern crate lazy_static;

use rustyline::{Editor, error::ReadlineError};

#[macro_use] mod macros;

mod core;
mod env;
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

fn eval_ast_list<T>(elements: T, env: &mut EnvRef) -> Result<Vec<AST>, ASTError>
    where T: IntoIterator<Item=AST> + FromIterator<AST>
{
    elements.into_iter()
        .map(|x| eval(x, env.clone()))
        .collect()
}

fn eval_ast(ast: AST, env: &mut EnvRef) -> Result<AST, ASTError> {
    match ast.clone() {
        AST::Symbol(symbol)   => Ok(env.get(&symbol[..]).unwrap_or(ast)),
        AST::List(elements)   => eval_ast_list(elements, env).map(AST::List),
        AST::Vector(elements) => eval_ast_list(elements, env).map(AST::Vector),
        ast @ _               => Ok(ast),
    }
}

fn eval_apply(ast: &AST) -> Result<AST, ASTError> {
    if let AST::List(elements) = ast {
        match &elements[0] {
            AST::Function(func)          => func(&elements[1..]),
            AST::Symbol(symbol)          => Err(SymbolNotFound(symbol.to_string())),
            ast @ _                      => Err(NotEvaluable(ast.clone())),
        }
    } else {
        unreachable!()
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
                return Err(ArityError {
                    expected: expected,
                    reached: $args.len(),
                });
            }
        }
    }
}

fn eval(mut ast: AST, mut env: EnvRef) -> Result<AST, ASTError> {
    loop {
        if let AST::List(elements) = &ast {
            if elements.is_empty() {
                return Ok(AST::List(vec![]));
            }

            if let AST::Symbol(symbol) = &elements[0] {
                let args = (&elements[1..]).to_owned();

                match symbol.as_str() {
                    "def!" => {
                        expect_arity!(args, 2);
                        let def_symbol = &args[0];
                        let def_value = args[1].clone();

                        if let AST::Symbol(symbol) = def_symbol {
                            let new_ast = eval(def_value, env.clone())?;
                            env.set(symbol.to_string(), new_ast.clone());

                            ast = new_ast;
                            continue; // don't run the `apply` phase just yet
                        } else {
                            return Err(CanOnlyDefineSymbols(def_symbol.clone()));
                        }
                    }

                    "let*" => {
                        expect_arity!(args, 2);
                        let let_symbol = &args[0];
                        let let_value = args[1].clone();

                        let mut new_env = EnvRef::refer_to(env);
                        match let_symbol {
                            AST::List(bindings) | AST::Vector(bindings) => {
                                for let_args in bindings.chunks(2) {
                                    let binding_symbol = let_args[0].clone();
                                    let binding_value = let_args[1].clone();

                                    if let AST::Symbol(symbol) = &binding_symbol {
                                        let value = eval(binding_value.clone(), new_env.clone())?;
                                        new_env.set(symbol.to_string(), value);
                                    } else {
                                        return Err(CanOnlyDefineSymbols(binding_symbol.clone()));
                                    }
                                }
                            }
                            _ => return Err(CannotBindArguments(let_symbol.clone()))
                        }

                        ast = let_value;
                        env = new_env;
                        continue; // don't run the `apply` phase just yet
                    }

                    "do" => {
                        let mut new_ast = AST::Nil;
                        for arg in args {
                            new_ast = eval(arg, env.clone())?;
                        }

                        ast = new_ast;
                        continue; // don't run the `apply` phase just yet
                    }

                    "if" => {
                        expect_arity!(args, 2, 3);

                        let if_predicate = args[0].clone();
                        let if_true_branch = args[1].clone();
                        let if_false_branch = if args.len() > 2 { args[2].clone() } else { AST::Nil };

                        // A temporary env is used to prevent mutation when evaluating the condition
                        let condition_env = EnvRef::refer_to(env.clone());

                        let new_ast = {
                            match eval(if_predicate, condition_env)? {
                                AST::Nil | AST::Bool(false) => if_false_branch,
                                _                           => if_true_branch,
                            }
                        };

                        ast = new_ast;
                        continue; // don't run the `apply` phase just yet
                    }

                    "fn*" => {
                        expect_arity!(args, 2);

                        let fn_symbol = args[0].clone();
                        let fn_body = args[1].clone();

                        match fn_symbol.clone() {
                            AST::List(bindings) | AST::Vector(bindings) => {
                                let mut symbols = vec![];
                                for value in bindings {
                                    if let AST::Symbol(symbol) = value {
                                        symbols.push(symbol.clone());
                                    } else {
                                        return Err(CanOnlyDefineSymbols(fn_symbol));
                                    }
                                }

                                let captured_env = env.clone();
                                let new_ast = make_function!(move |args: &[AST]| -> Result<AST, ASTError> {
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

                                    eval(fn_body.clone(), call_env)
                                });

                                ast = new_ast;
                                continue; // don't run the `apply` phase just yet
                            }

                            _ => return Err(CannotBindArguments(fn_symbol))
                        }
                    }

                    _ => {} // run the `apply` phase
                }
            } else {
                // run the `apply` phase
            }

            // `apply` phase
            return eval_apply(&eval_ast(ast, &mut env)?);
        } else {
            return eval_ast(ast, &mut env);
        }
    }
}

fn print(ast: &AST) -> String {
    printer::pr_str(&ast, true)
}

fn print_error(ast_error: &ASTError) -> String {
    match ast_error {
        CanOnlyDefineSymbols(ast)        => format!("can only define symbols (not '{}')", print(&ast)),
        CannotBindArguments(ast)         => format!("cannot bind arguments using '{}', expected a list", print(&ast)),
        NotEvaluable(ast)                => format!("cannot evaluate '{}'", print(&ast)),
        SymbolNotFound(symbol)           => format!("symbol '{}' not found", symbol),
        TypeCheckFailed {}               => format!("typecheck failed"),
        ArityError { expected, reached } => format!("arity error, tried to call symbol expecting {} arguments with {}", expected, reached),
        UnbalancedString                 => "unbalanced string".to_string(),
        UnbalancedList                   => "unbalanced list".to_string(),
        NotHashable(ast)                 => format!("{} is not hashable", print(&ast)),
        OddMapEntries                    => "odd number of entries in map".to_string(),
        DuplicateKey(ast)                => format!("duplicate key {}", print(&ast)),
        LoneDeref                        => "'@' must be followed by a value".to_string(),
        IOError(e)                       => format!("I/O error: {:?}", e),
    }
}

fn eval_print(ast: AST, env: EnvRef) -> String {
    match eval(ast, env) {
        Ok(ast) => print(&ast),
        Err(e)  => print_error(&e),
    }
}

fn rep(s: &str, env: EnvRef) -> String {
    match read(s) {
        Ok(Some(ast)) => eval_print(ast, env),
        Ok(None)      => "EOF".to_string(),
        Err(e)        => print_error(&e),
    }
}

fn main() -> io::Result<()> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    let _ = rl.load_history(".mal-history");

    // initialize repl environment
    let mut env = Env::new();
    for (symbol, value) in core::ns() {
        env.set(symbol, value);
    }

    // add `eval` method to repl environment
    let mut env = EnvRef::new(env);
    let captured_env = env.clone();
    env.set("eval".to_string(), function!(ast {
        eval(ast.clone(), captured_env.clone())
    }));

    rep("(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \")\")))))", env.clone());

    loop {
        match rl.readline("user> ") {
            Ok(line) => {
                rl.add_history_entry(line.to_string());
                rl.save_history(".mal-history").unwrap();
                if line.len() > 0 {
                    println!("{}", rep(&line, env.clone()));
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
