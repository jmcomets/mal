use std::collections::HashMap;
use crate::types::MalType;

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

    symbols.insert("not".to_string(), function!(pred: Bool -> Bool { !pred }));

    symbols.insert("=".to_string(), function!(a, b -> Bool { Ok(a == b) }));
    symbols.insert("!=".to_string(), function!(a, b -> Bool { Ok(a != b) }));

    symbols.insert("nil".to_string(), function!(x -> Bool {
        Ok(if let Nil = x { true } else { false })
    }));

    symbols.insert("list?".to_string(), function!(ls -> Bool {
        Ok(if let List(_) = ls { true } else { false })
    }));

    symbols.insert("empty?".to_string(), function!(ls -> Bool {
        Ok(if let List(ls) = ls { ls.is_empty() } else { false })
    }));

    symbols.insert("list".to_string(), make_function!(
            |args: &[MalType]| -> MalResult {
                Ok(List(args.to_owned()))
            }
    ));

    symbols.insert("count".to_string(), function!(ls -> Int {
        if let List(ls) = ls { Ok(ls.len() as i64) } else { Err(TypeCheckFailed{}) }
    }));

    symbols
}
