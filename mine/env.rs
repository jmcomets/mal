use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use crate::types::MalType;

pub(crate) struct Env<'a> {
    outer: Option<Box<Env<'a>>>,
    data: HashMap<String, EnvValue<'a>>,
}

impl<'a> Env<'a> {
    pub fn new() -> Self {
        Env {
            outer: None,
            data: HashMap::new(),
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&EnvValue>
        where String: Borrow<Q>,
              Q: Hash + Eq,
    {
        self.data.get(key)
            .or_else(|| self.outer.as_ref().and_then(|outer| outer.get(key)))
    }

    pub fn set(&mut self, key: String, value: EnvValue<'a>) {
        self.data.insert(key, value);
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

pub(crate) enum EnvValue<'a> {
    #[allow(dead_code)]
    Value(MalType),
    Callable {
        delegate: Box<'a + Fn(&[MalType]) -> MalType>,
        arity: usize,
    }
}

#[derive(Debug)]
pub(crate) struct ArityError {
    pub expected: usize,
    pub reached: usize,
}

impl<'a> EnvValue<'a> {
    pub fn callable2<F>(f: F) -> Self
        where F: 'a + Fn(&MalType, &MalType) -> MalType,
    {
        EnvValue::Callable {
            delegate: Box::new(move |args| { f(&args[0], &args[1]) }),
            arity: 2,
        }
    }

    pub fn try_call(&self, args: &[MalType]) -> Result<MalType, ArityError> {
        match self {
            EnvValue::Value(value) => {
                if !args.is_empty() {
                    return Err(ArityError {
                        expected: 0,
                        reached: args.len(),
                    });
                }

                Ok(value.clone())
            }
            EnvValue::Callable { arity, delegate } => {
                if *arity != args.len() {
                    return Err(ArityError {
                        expected: *arity,
                        reached: args.len(),
                    });
                }

                Ok(delegate(args))
            }
        }
    }
}
