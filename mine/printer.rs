use crate::types::MalType;

pub(crate) fn pr_str(t: &MalType) -> String {
    use MalType::*;
    match t {
        Unimplemented => "unimplemented".to_string(),

        List(elements) => {
            let mut s = String::new();
            s += "(";
            for (i, elem) in elements.iter().enumerate() {
                if i != 0 {
                    s += " ";
                }
                s += &pr_str(elem);
            }
            s += ")";
            s
        }

        Symbol(s) => s.clone(),
        Int(i)    => i.to_string(),
        Float(f)  => f.to_string(),
        Bool(b)   => b.to_string(),
        Str(s)    => "\"".to_string() + s + "\"",
        Nil       => "nil".to_string(),

        NativeFunc { name, .. } => name.to_string(),
    }
}
