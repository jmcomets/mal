use crate::types::MalType;

pub(crate) fn pr_str(t: &MalType) -> String {
    match t {
        MalType::List(elements) => {
            let mut s = String::new();
            s += "[";
            for (i, elem) in elements.iter().enumerate() {
                if i != 0 {
                    s += ", ";
                }
                s += &pr_str(elem);
            }
            s += "]";
            s
        }
        MalType::Symbol(s) => s.clone(),
        MalType::Int(i)    => i.to_string(),
        MalType::Float(f)  => f.to_string(),
        MalType::Bool(b)   => b.to_string(),
        MalType::Str(s)    => "\"".to_string() + s + "\"",
        MalType::Nil       => "nil".to_string(),
    }
}
