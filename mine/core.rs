use std::collections::HashMap;
use crate::types::MalType;

pub(crate) fn ns() -> HashMap<String, MalType> {
    let mut symbols = HashMap::new();

    // symbols.insert("+".to_string(), function!(a: Int, b: Int -> Int { a + b }));
    symbols.insert("+".to_string(), binary_operator!(Int + Int -> Int));
    symbols.insert("-".to_string(), binary_operator!(Int - Int -> Int));
    symbols.insert("*".to_string(), binary_operator!(Int * Int -> Int));
    symbols.insert("/".to_string(), binary_operator!(Int / Int -> Int));

    symbols.insert("<".to_string(), binary_operator!(Int < Int -> Bool));
    symbols.insert(">".to_string(), binary_operator!(Int > Int -> Bool));
    symbols.insert("<=".to_string(), binary_operator!(Int <= Int -> Bool));
    symbols.insert(">=".to_string(), binary_operator!(Int >= Int -> Bool));

    symbols.insert("inc".to_string(), function!(x: Int -> Int { x + 1 }));
    symbols.insert("dec".to_string(), function!(x: Int -> Int { x - 1 }));

    // TODO implement for all types
    symbols.insert("==".to_string(), binary_operator!(Int == Int -> Bool));
    symbols.insert("!=".to_string(), binary_operator!(Int != Int -> Bool));

    symbols
}
