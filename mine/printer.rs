use crate::types::{MalType, MalHashable};

pub(crate) fn pr_str(t: &MalType, readably: bool) -> String {
    use MalType::*;
    match t {
        Atom(x)     => pr_str(&x.borrow(), readably),
        List(x)     => pr_list(x, "(", ")", readably),
        Vector(x)   => pr_list(x, "[", "]", readably),
        Dict(x)     => pr_dict(x, "{", "}", readably),
        Symbol(x)   => x.clone(),
        Number(x)   => x.to_string(),
        Bool(x)     => x.to_string(),
        Str(x)      => if readably { "\"".to_string() + x + "\"" } else { x.clone() },
        Nil         => "nil".to_string(),
        Function(_) => "#<function>".to_string(),
    }
}

fn pr_list<'a, It>(elements: It, opening: &str, closing: &str, readably: bool) -> String
    where It: IntoIterator<Item=&'a MalType>,
{
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

fn pr_dict<'a, It>(elements: It, opening: &str, closing: &str, readably: bool) -> String
    where It: IntoIterator<Item=(&'a MalHashable, &'a MalType)>,
{
    let mut s = String::new();
    s += opening;
    for (i, (key, value)) in elements.into_iter().enumerate() {
        if i != 0 {
            s += " ";
        }
        s += &pr_str(&key.clone().into(), readably);
        s += " ";
        s += &pr_str(value, readably);
    }
    s += closing;
    s
}
