use crate::types::MalType;

pub(crate) fn pr_str(t: &MalType) -> String {
    use MalType::*;
    match t {
        List(elements) => pr_list(elements, "(", ")"),
        Vector(elements) => pr_list(elements, "[", "]"),
        Symbol(s) => s.clone(),
        Int(i)    => i.to_string(),
        Float(f)  => f.to_string(),
        Bool(b)   => b.to_string(),
        Str(s)    => "\"".to_string() + s + "\"",
        Nil       => "nil".to_string(),

        NativeFunc { name, .. } => name.to_string(),
    }
}

fn pr_list(elements: &[MalType], opening: &str, closing: &str) -> String {
    let mut s = String::new();
    s += opening;
    for (i, elem) in elements.iter().enumerate() {
        if i != 0 {
            s += " ";
        }
        s += &pr_str(elem);
    }
    s += closing;
    s
}
