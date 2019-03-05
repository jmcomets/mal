#[derive(Debug, PartialEq)]
pub(crate) enum MalType {
    List(Vec<Box<MalType>>),
    Symbol(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Nil,
    Unimplemented,
}
