use std::collections::HashMap;
use crate::types::MalType;

pub(crate) fn ns() -> HashMap<String, MalType> {
    let mut symbols = HashMap::new();

    symbols.insert("+".to_string(), binary_operator!(Int + Int -> Int));
    symbols.insert("-".to_string(), binary_operator!(Int - Int -> Int));
    symbols.insert("*".to_string(), binary_operator!(Int * Int -> Int));
    symbols.insert("/".to_string(), binary_operator!(Int / Int -> Int));

    symbols.insert("<".to_string(), binary_operator!(Int < Int -> Bool));
    symbols.insert(">".to_string(), binary_operator!(Int > Int -> Bool));
    symbols.insert("<=".to_string(), binary_operator!(Int <= Int -> Bool));
    symbols.insert(">=".to_string(), binary_operator!(Int >= Int -> Bool));

    // TODO implement for all types
    symbols.insert("==".to_string(), binary_operator!(Int == Int -> Bool));
    symbols.insert("!=".to_string(), binary_operator!(Int != Int -> Bool));

    symbols
}
