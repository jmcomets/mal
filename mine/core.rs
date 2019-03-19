use std::collections::HashMap;
use crate::types::MalType;
use crate::printer::pr_str;

pub(crate) fn ns() -> HashMap<String, MalType> {
    let mut symbols = HashMap::new();

    symbols.insert("+".to_string(), number_operator!(+));
    symbols.insert("-".to_string(), number_operator!(-));
    symbols.insert("*".to_string(), number_operator!(*));
    symbols.insert("/".to_string(), number_operator!(/));

    symbols.insert("<".to_string(), number_predicate!(<));
    symbols.insert(">".to_string(), number_predicate!(>));
    symbols.insert("<=".to_string(), number_predicate!(<=));
    symbols.insert(">=".to_string(), number_predicate!(>=));

    symbols.insert("inc".to_string(), function!(x: Int -> Int { x + 1 }));
    symbols.insert("dec".to_string(), function!(x: Int -> Int { x - 1 }));

    symbols.insert("not".to_string(), function!(x -> Bool {
        Ok({
            match x {
                Bool(p)               => !p,
                Int(n)                => !(*n == 0),
                Float(n)              => !(*n == 0.),
                List(ls) | Vector(ls) => !ls.is_empty(),
                Nil                   => true,
                _                     => false,
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
            Nil                   => Ok(true),
            List(ls) | Vector(ls) => Ok(ls.is_empty()),
            _                     => Err(TypeCheckFailed{}),
        }
    }));

    symbols.insert("list".to_string(), make_function!(
            |args: &[MalType]| -> MalResult {
                Ok(List(args.to_owned()))
            }
    ));

    symbols.insert("count".to_string(), function!(ls -> Int {
        match ls {
            Nil                   => Ok(0),
            List(ls) | Vector(ls) => Ok(ls.len() as i64),
            _                     => Err(TypeCheckFailed{}),
        }
    }));

    symbols.insert("pr-str".to_string(), make_function!(
            |args: &[MalType]| -> MalResult {
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
            }
    ));

    symbols.insert("str".to_string(), make_function!(
            |args: &[MalType]| -> MalResult {
                Ok(Str({
                    let mut s = String::new();
                    for arg in args.iter() {
                        s += &pr_str(arg, false);
                    }
                    s
                }))
            }
    ));

    symbols.insert("prn".to_string(), make_function!(
            |args: &[MalType]| -> MalResult {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    print!("{}", pr_str(arg, true));
                }
                println!();
                Ok(Nil)
            }
    ));

    symbols.insert("println".to_string(), make_function!(
            |args: &[MalType]| -> MalResult {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    print!("{}", pr_str(arg, false));
                }
                println!();
                Ok(Nil)
            }
    ));

    symbols
}
