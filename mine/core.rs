use std::fs::File;
use std::io::{Read, BufReader};

use crate::env::Env;
use crate::printer::pr_str;
use crate::reader::read_str;

pub(crate) fn ns() -> Env {
    let env = Env::default();

    env.set("+".to_string(), binary_operator!(Number + Number -> Number));
    env.set("-".to_string(), binary_operator!(Number - Number -> Number));
    env.set("*".to_string(), binary_operator!(Number * Number -> Number));
    env.set("/".to_string(), binary_operator!(Number / Number -> Number));

    env.set("<".to_string(), binary_operator!(Number < Number -> Bool));
    env.set(">".to_string(), binary_operator!(Number > Number -> Bool));
    env.set("<".to_string(), binary_operator!(Number <= Number -> Bool));
    env.set(">".to_string(), binary_operator!(Number >= Number -> Bool));

    env.set("inc".to_string(), function!(x: Number -> Number { Ok(x + Int(1)) }));
    env.set("dec".to_string(), function!(x: Number -> Number { Ok(x - Int(1)) }));

    env.set("not".to_string(), function!(x -> Bool {
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

    env.set("=".to_string(), function!(a, b -> Bool { Ok(a == b) }));
    env.set("!=".to_string(), function!(a, b -> Bool { Ok(a != b) }));

    env.set("nil".to_string(), function!(x -> Bool {
        Ok(if let Nil = x { true } else { false })
    }));

    env.set("list?".to_string(), function!(ls -> Bool {
        Ok(if let List(_) = ls { true } else { false })
    }));

    env.set("empty?".to_string(), function!(ls -> Bool {
        match ls {
            Nil        => Ok(true),
            List(ls)   => Ok(ls.is_empty()),
            Vector(ls) => Ok(ls.is_empty()),
            _          => Err(TypeCheckFailed{}),
        }
    }));

    env.set("list".to_string(), variadic_function!(args {
        Ok(List(args.to_owned()))
    }));

    env.set("count".to_string(), function!(ls -> Number {
        match ls {
            Nil        => Ok(Int(0)),
            List(ls)   => Ok(Int(ls.len() as i64)),
            Vector(ls) => Ok(Int(ls.len() as i64)),
            _          => Err(TypeCheckFailed{}),
        }
    }));

    env.set("pr-str".to_string(), variadic_function!(args {
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

    env.set("str".to_string(), variadic_function!(args {
        Ok(Str({
            let mut s = String::new();
            for arg in args.iter() {
                s += &pr_str(arg, false);
            }
            s
        }))
    }));

    env.set("prn".to_string(), variadic_function!(args {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{}", pr_str(arg, true));
        }
        println!();
        Ok(Nil)
    }));

    env.set("println".to_string(), variadic_function!(args {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{}", pr_str(arg, false));
        }
        println!();
        Ok(Nil)
    }));

    env.set("read-string".to_string(), function!(s: Str {
        read_str(&s).transpose().unwrap_or(Ok(Nil))
    }));

    env.set("slurp".to_string(), function!(filename: Str {
        let file = File::open(&filename).map_err(IOError)?;

        let mut contents = String::new();
        let mut buffered_reader = BufReader::new(file);
        buffered_reader.read_to_string(&mut contents).map_err(IOError)?;

        read_str(&contents).transpose().unwrap_or(Ok(Nil))
    }));

    env.set("atom".to_string(), function!(x {
        Ok(MalType::atom(x.clone()))
    }));

    env.set("atom?".to_string(), function!(x -> Bool {
        Ok(if let Atom(_) = x { true } else { false })
    }));

    env.set("deref".to_string(), function!(x: Atom {
        Ok({
            let x = x.borrow();
            x.clone()
        })
    }));

    env.set("reset!".to_string(), function!(x: Atom, value {
        Ok(x.replace(value.clone()))
    }));

    //env.set("swap!".to_string(), function!(x: Atom, f: Function {
    //    Ok(x.replace_with(f)) // TODO have `f` take extra arguments
    //}));

    env.set("cons".to_string(), function!(head, tail {
        let mut tail = {
            match tail {
                List(list)   => list,
                Vector(list) => list.into_iter().collect(),
                _            => return Err(TypeCheckFailed {}),
            }
        };
        tail.push_front(head);
        Ok(List(tail))
    }));

    env.set("concat".to_string(), variadic_function!(args {
        let mut concatenated = im::Vector::new();
        for arg in args {
            match arg {
                List(list)   => concatenated.extend(list),
                Vector(list) => concatenated.extend(list),
                value @ _    => concatenated.push_back(value),
            }
        }
        Ok(List(concatenated))
    }));


    env
}
