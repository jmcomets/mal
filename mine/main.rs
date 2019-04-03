// #![deny(warnings)]

use std::io;
use std::iter::FromIterator;

#[macro_use] extern crate lazy_static;

use rustyline::{Editor, error::ReadlineError};

use itertools::Itertools;

#[macro_use] mod macros;
mod core;
mod env;
mod printer;
mod reader;
mod types;

use env::Env;

use types::{
    MalType as AST,
    MalError as ASTError,
    MalArgs as ASTArgs,
};

use ASTError::*;

fn eval_ast_list<T>(elements: T, env: &Env) -> Result<T, ASTError>
    where T: IntoIterator<Item=AST> + FromIterator<AST>,
{
    elements.into_iter()
        .map(|x| eval(x, env.pass()))
        .collect()
}

fn eval_ast(ast: AST, env: &Env) -> Result<AST, ASTError> {
    match ast {
        AST::Symbol(symbol)   => Ok(env.get(&symbol).unwrap_or(AST::Symbol(symbol))),
        AST::List(elements)   => eval_ast_list(elements, env).map(AST::List),
        AST::Vector(elements) => eval_ast_list(elements, env).map(AST::Vector),
        ast @ _               => Ok(ast),
    }
}

fn eval_def(symbol: AST, value: AST, env: &mut Env) -> Result<AST, ASTError> {
    if let AST::Symbol(symbol) = symbol {
        let ast = eval(value, env.pass())?;
        env.set(symbol.to_string(), ast.clone());
        Ok(ast)
    } else {
        return Err(CanOnlyDefineSymbols(symbol.clone()));
    }
}

fn eval_let<'a, It>(bindings: It, env: &mut Env) -> Result<(), ASTError>
    where It: IntoIterator<Item=AST>,
{
    for (symbol, value) in bindings.into_iter().tuples() {
        if let AST::Symbol(symbol) = symbol {
            let value = eval(value.clone(), env.pass())?;
            env.set(symbol.to_string(), value);
        } else {
            return Err(CanOnlyDefineSymbols(symbol.clone()));
        }
    }
    Ok(())
}

fn eval_cond(condition: AST, if_true_body: AST, if_false_body: AST, env: &Env) -> Result<AST, ASTError> {
    Ok({
        // A temporary env is used to prevent mutation when evaluating the condition
        match eval(condition, env.wrap())? {
            AST::Nil | AST::Bool(false) => if_false_body,
            _                           => if_true_body,
        }
    })
}

fn eval_fn<'a, It>(bindings: It, body: AST, env: &Env) -> Result<AST, ASTError>
    where It: IntoIterator<Item=AST>,
{
    let it = bindings.into_iter();
    let (min_capacity, _) = it.size_hint();
    let mut symbols = Vec::with_capacity(min_capacity);
    for value in it {
        if let AST::Symbol(symbol) = value {
            symbols.push(symbol);
        } else {
            return Err(CanOnlyDefineSymbols(value.clone()));
        }
    }

    let captured_env = env.wrap();

    Ok(make_function!(move |args: ASTArgs| -> Result<AST, ASTError> {
        expect_arity!(args, symbols.len());

        let call_env = captured_env.wrap();
        for (symbol, value) in symbols.iter().zip(args.into_iter()) {
            call_env.set(symbol.to_string(), value);
        }

        eval(body.clone(), call_env)
    }))
}

fn eval(mut ast: AST, mut env: Env) -> Result<AST, ASTError> {
    loop {
        if let AST::List(mut elements) = ast.clone() {
            if elements.is_empty() {
                return Ok(AST::List(elements));
            }

            if let AST::Symbol(symbol) = elements.pop_front().unwrap() {
                let mut args = elements;

                match symbol.as_str() {
                    "def!" => {
                        expect_arity!(args, 2);
                        let def_symbol = args.pop_front().unwrap();
                        let def_value = args.pop_front().unwrap();

                        ast = eval_def(def_symbol, def_value, &mut env)?;
                        continue; // don't run the `apply` phase just yet
                    }

                    "let*" => {
                        expect_arity!(args, 2);
                        let let_symbol = args.pop_front().unwrap();
                        let let_value = args.pop_front().unwrap();

                        let mut let_env = env.wrap();

                        match let_symbol {
                            AST::List(bindings)   => eval_let(bindings, &mut let_env)?,
                            AST::Vector(bindings) => eval_let(bindings, &mut let_env)?,
                            _                     => return Err(CannotBindArguments(let_symbol.clone()))
                        }

                        ast = let_value;
                        env = let_env;
                        continue; // don't run the `apply` phase just yet
                    }

                    "do" => {
                        ast = AST::Nil;
                        for arg in args {
                            ast = eval(arg, env.pass())?;
                        }
                        continue; // don't run the `apply` phase just yet
                    }

                    "if" => {
                        expect_arity!(args, 2, 3);
                        let if_predicate = args.pop_front().unwrap();
                        let if_true_body = args.pop_front().unwrap();
                        let if_false_body = args.pop_front().unwrap_or(AST::Nil);

                        ast = eval_cond(if_predicate, if_true_body, if_false_body, &env)?;
                        continue; // don't run the `apply` phase just yet
                    }

                    "fn*" => {
                        expect_arity!(args, 2);
                        let fn_symbol = args.pop_front().unwrap();
                        let fn_body = args.pop_front().unwrap();

                        match fn_symbol {
                            AST::List(bindings) => {
                                ast = eval_fn(bindings, fn_body, &env)?;
                                continue; // don't run the `apply` phase just yet
                            }

                            AST::Vector(bindings) => {
                                ast = eval_fn(bindings, fn_body, &env)?;
                                continue; // don't run the `apply` phase just yet
                            }

                            fn_symbol @ _ => return Err(CannotBindArguments(fn_symbol))
                        }
                    }

                    _ => {} // run the `apply` phase
                }
            } else {
                // run the `apply` phase
            }

            // `apply` phase
            if let AST::List(mut elements) = eval_ast(ast, &mut env)? {
                let symbol = elements.pop_front().unwrap();
                return {
                    match symbol {
                        AST::Function(func) => func(elements),
                        AST::Symbol(symbol) => Err(SymbolNotFound(symbol.to_string())),
                        ast @ _             => Err(NotEvaluable(ast)),
                    }
                };
            } else {
                unreachable!();
            }
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
        CanOnlyDefineSymbols(ast)                  => format!("can only define symbols (not '{}')", print(&ast)),
        CannotBindArguments(ast)                   => format!("cannot bind arguments using '{}', expected a list", print(&ast)),
        NotEvaluable(ast)                          => format!("cannot evaluate '{}'", print(&ast)),
        SymbolNotFound(symbol)                     => format!("symbol '{}' not found", symbol),
        TypeCheckFailed {}                         => format!("typecheck failed"),
        ArityError { expected, reached }           => format!("arity error, tried to call symbol expecting {} arguments with {}", expected, reached),
        UnbalancedString                           => "unbalanced string".to_string(),
        MismatchedDelimiters(open, close, reached) => format!("unclosed delimiter '{}', expected a '{}' but got '{}'", open, close, reached),
        UnmatchedDelimiter(open, close)            => format!("unclosed '{}', expected a '{}'", open, close),
        NotHashable(ast)                           => format!("{} is not hashable", print(&ast)),
        OddMapEntries                              => "odd number of entries in map".to_string(),
        DuplicateKey(ast)                          => format!("duplicate key {}", print(&ast)),
        LoneDeref                                  => "'@' must be followed by a value".to_string(),
        IOError(e)                                 => format!("I/O error: {:?}", e),
    }
}

fn eval_print(ast: AST, env: Env) -> String {
    match eval(ast, env) {
        Ok(ast) => print(&ast),
        Err(e)  => print_error(&e),
    }
}

fn read(reader: &mut reader::Reader, s: &str) -> Result<Option<AST>, ASTError> {
    reader.push(s)?;
    reader.pop()
}

fn rep(reader: &mut reader::Reader, s: &str, env: Env) -> String {
    match read(reader, s) {
        Ok(Some(ast)) => eval_print(ast, env),
        Ok(None)      => "EOF".to_string(),
        Err(e)        => print_error(&e),
    }
}

fn main() -> io::Result<()> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    let _ = rl.load_history(".mal-history");

    let env = core::ns();

    // add `eval` method to repl environment
    let captured_env = env.wrap();
    env.set("eval".to_string(), function!(ast {
        eval(ast.clone(), captured_env.pass())
    }));

    let mut reader = reader::Reader::new();

    rep(&mut reader, "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \")\")))))", env.pass());

    loop {
        match rl.readline("user> ") {
            Ok(line) => {
                rl.add_history_entry(line.to_string());
                rl.save_history(".mal-history").unwrap();
                if line.len() > 0 {
                    println!("{}", rep(&mut reader, &line, env.pass()));
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