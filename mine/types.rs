#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MalType {
    List(Vec<MalType>),
    Symbol(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Nil,
    Unimplemented,
}

// TODO generate this code via macro_rules!

#[allow(dead_code)]
impl MalType {
    pub(crate) fn is_list(&self) -> bool {
        match self {
            MalType::List(_) => true,
            _                => false,
        }
    }

    pub(crate) fn as_list(&self) -> Option<&[MalType]> {
        match self {
            MalType::List(l) => Some(&l[..]),
            _                => None,
        }
    }

    pub(crate) fn into_list(self) -> Option<Vec<MalType>> {
        match self {
            MalType::List(l) => Some(l),
            _                => None,
        }
    }

    pub(crate) fn is_symbol(&self) -> bool {
        match self {
            MalType::Symbol(_) => true,
            _                  => false,
        }
    }

    pub(crate) fn as_symbol(&self) -> Option<&str> {
        match self {
            MalType::Symbol(s) => Some(s),
            _                  => None,
        }
    }

    pub(crate) fn into_symbol(self) -> Option<String> {
        match self {
            MalType::Symbol(s) => Some(s),
            _                  => None,
        }
    }

    pub(crate) fn is_int(&self) -> bool {
        match self {
            MalType::Int(_) => true,
            _               => false,
        }
    }

    pub(crate) fn as_int(&self) -> Option<&i64> {
        match self {
            MalType::Int(i) => Some(i),
            _               => None,
        }
    }

    pub(crate) fn into_int(self) -> Option<i64> {
        match self {
            MalType::Int(i) => Some(i),
            _                => None,
        }
    }

    pub(crate) fn is_float(&self) -> bool {
        match self {
            MalType::Float(_) => true,
            _                 => false,
        }
    }

    pub(crate) fn as_float(&self) -> Option<&f64> {
        match self {
            MalType::Float(f) => Some(f),
            _                 => None,
        }
    }

    pub(crate) fn into_float(self) -> Option<f64> {
        match self {
            MalType::Float(f) => Some(f),
            _                => None,
        }
    }

    pub(crate) fn is_bool(&self) -> bool {
        match self {
            MalType::Bool(_) => true,
            _                => false,
        }
    }

    pub(crate) fn as_bool(&self) -> Option<&bool> {
        match self {
            MalType::Bool(b) => Some(b),
            _                => None,
        }
    }

    pub(crate) fn into_bool(self) -> Option<bool> {
        match self {
            MalType::Bool(b) => Some(b),
            _                => None,
        }
    }

    pub(crate) fn is_str(&self) -> bool {
        match self {
            MalType::Str(_) => true,
            _               => false,
        }
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            MalType::Str(s) => Some(s),
            _               => None,
        }
    }

    pub(crate) fn into_str(self) -> Option<String> {
        match self {
            MalType::Str(s) => Some(s),
            _                => None,
        }
    }

    pub(crate) fn is_nil(&self) -> bool {
        self == &MalType::Nil
    }
}
