use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::types::MalType;

pub(crate) struct Env {
    outer: Option<Box<Env>>,
    data: RefCell<HashMap<String, EnvValue>>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            outer: None,
            data: RefCell::new(HashMap::new()),
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<EnvValue>
        where String: Borrow<Q>,
              Q: Hash + Eq,
    {
        self.data.borrow().get(key)
            .map(Clone::clone)
            .or_else(|| self.outer.as_ref().and_then(|outer| outer.get(key)))
    }

    pub fn set(&self, key: String, value: EnvValue) {
        self.data.borrow_mut().insert(key, value);
    }
}

macro_rules! match_binary_operation {
    ($($left:tt $op:tt $right:tt => $out:tt),*) => {
        |a: &$crate::types::MalType, b: &$crate::types::MalType| {
            use $crate::types::MalType::*;
            match (a, b) {
                $(($left(left), $right(right)) => Some($out(left $op right)),)*
                _                              => None,
            }
        }
    }
}

macro_rules! arithmetic_operations {
    ($op:tt) => {
        $crate::env::EnvValue::callable2(|a, b| {
            let matchers = match_binary_operation! {
                Int $op Int => Int,
                Float $op Float => Float
            };

            matchers(a, b).unwrap()
        })
    }
}

#[derive(Clone)]
pub(crate) enum EnvValue {
    #[allow(dead_code)]
    Value(MalType),
    Callable {
        delegate: Rc<Fn(&[EnvValue]) -> MalType>,
        arity: usize,
    }
}

impl EnvValue {
    pub fn callable2<F>(f: F) -> Self
        where F: 'static + Fn(&MalType, &MalType) -> MalType,
    {
        EnvValue::Callable {
            delegate: Rc::new(move |args| {
                match (&args[0], &args[1]) {
                    (EnvValue::Value(a), EnvValue::Value(b)) => f(a, b),
                    _ => unimplemented!(),
                }
            }),
            arity: 2,
        }
    }
}
