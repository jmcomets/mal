use crate::types::MalType;

pub(crate) fn pr_str(t: &MalType, readably: bool) -> String {
    use MalType::*;
    match t {
        List(elements)          => pr_list(elements, "(", ")", readably),
        Vector(elements)        => pr_list(elements, "[", "]", readably),
        Symbol(s)               => s.clone(),
        Number(n)               => n.to_string(),
        Bool(b)                 => b.to_string(),
        Str(s)                  => if readably { "\"".to_string() + s + "\"" } else { s.clone() },
        Nil                     => "nil".to_string(),
        Function(_)             => "#<function>".to_string(),
    }
}

fn pr_list<'a, It: IntoIterator<Item=&'a MalType>>(elements: It, opening: &str, closing: &str, readably: bool) -> String {
    let mut s = String::new();
    s += opening;
    for (i, elem) in elements.into_iter().enumerate() {
        if i != 0 {
            s += " ";
        }
        s += &pr_str(elem, readably);
    }
    s += closing;
    s
}
