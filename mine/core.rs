use std::fs::File;
use std::io::{Read, BufReader};
use std::collections::HashMap;

use crate::types::MalType;
use crate::printer::pr_str;
use crate::reader::read_str;

pub(crate) fn ns() -> HashMap<String, MalType> {
    let mut symbols = HashMap::new();

    symbols.insert("+".to_string(), binary_operator!(Number + Number -> Number));
    symbols.insert("-".to_string(), binary_operator!(Number - Number -> Number));
    symbols.insert("*".to_string(), binary_operator!(Number * Number -> Number));
    symbols.insert("/".to_string(), binary_operator!(Number / Number -> Number));

    symbols.insert("<".to_string(), binary_operator!(Number < Number -> Bool));
    symbols.insert(">".to_string(), binary_operator!(Number > Number -> Bool));
    symbols.insert("<".to_string(), binary_operator!(Number <= Number -> Bool));
    symbols.insert(">".to_string(), binary_operator!(Number >= Number -> Bool));

    symbols.insert("inc".to_string(), function!(x: Number -> Number { Ok(x + Int(1)) }));
    symbols.insert("dec".to_string(), function!(x: Number -> Number { Ok(x - Int(1)) }));

    symbols.insert("not".to_string(), function!(x -> Bool {
        Ok({
            match x {
                Bool(p)    => !p,
                Number(n)  => !(n == Int(0)),
                List(ls)   => !ls.is_empty(),
                Vector(ls) => !ls.is_empty(),
                Nil        => true,
                _          => false,
            }
        })
    }));

    symbols.insert("=".to_string(), function!(a, b -> Bool { Ok(a == b) }));
    symbols.insert("!=".to_string(), function!(a, b -> Bool { Ok(a != b) }));

    symbols.insert("nil".to_string(), function!(x -> Bool {
        Ok(if let Nil = x { true } else { false })
    }));

    symbols.insert("list?".to_string(), function!(ls -> Bool {
        Ok(if let List(_) = ls { true } else { false })
    }));

    symbols.insert("empty?".to_string(), function!(ls -> Bool {
        match ls {
            Nil        => Ok(true),
            List(ls)   => Ok(ls.is_empty()),
            Vector(ls) => Ok(ls.is_empty()),
            _          => Err(TypeCheckFailed{}),
        }
    }));

    symbols.insert("list".to_string(), variadic_function!(args {
        Ok(List(args.to_owned()))
    }));

    symbols.insert("count".to_string(), function!(ls -> Number {
        match ls {
            Nil        => Ok(Int(0)),
            List(ls)   => Ok(Int(ls.len() as i64)),
            Vector(ls) => Ok(Int(ls.len() as i64)),
            _          => Err(TypeCheckFailed{}),
        }
    }));

    symbols.insert("pr-str".to_string(), variadic_function!(args {
        Ok(Str({
            let mut s = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    s += " ";
                }
                s += &pr_str(arg, true);
            }
            s
        }))
    }));

    symbols.insert("str".to_string(), variadic_function!(args {
        Ok(Str({
            let mut s = String::new();
            for arg in args.iter() {
                s += &pr_str(arg, false);
            }
            s
        }))
    }));

    symbols.insert("prn".to_string(), variadic_function!(args {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{}", pr_str(arg, true));
        }
        println!();
        Ok(Nil)
    }));

    symbols.insert("println".to_string(), variadic_function!(args {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{}", pr_str(arg, false));
        }
        println!();
        Ok(Nil)
    }));

    symbols.insert("read-string".to_string(), function!(s: Str {
        read_str(&s).transpose().unwrap_or(Ok(Nil))
    }));

    symbols.insert("slurp".to_string(), function!(filename: Str {
        let file = File::open(&filename).map_err(IOError)?;

        let mut contents = String::new();
        let mut buffered_reader = BufReader::new(file);
        buffered_reader.read_to_string(&mut contents).map_err(IOError)?;

        read_str(&contents).transpose().unwrap_or(Ok(Nil))
    }));

    symbols.insert("atom".to_string(), function!(x {
        Ok(MalType::atom(x.clone()))
    }));

    symbols.insert("atom?".to_string(), function!(x -> Bool {
        Ok(if let Atom(_) = x { true } else { false })
    }));

    symbols.insert("deref".to_string(), function!(x: Atom {
        Ok({
            let x = x.borrow();
            x.clone()
        })
    }));

    symbols.insert("reset!".to_string(), function!(x: Atom, value {
        Ok(x.replace(value.clone()))
    }));

    //symbols.insert("swap!".to_string(), function!(x: Atom, f: Function {
    //    Ok(x.replace_with(f)) // TODO have `f` take extra arguments
    //}));

    symbols
}
