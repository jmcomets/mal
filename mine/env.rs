use std::borrow::Borrow;
use std::collections::HashMap;

use crate::types::MalType;

pub(crate) struct Env {
    outer: Option<Box<Env>>,
    data: HashMap<String, Callable<'static>>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            outer: None,
            data: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Callable<'static>> {
        self.data.get(key)
            .or_else(|| self.outer.as_ref().and_then(|outer| outer.get(key)))
    }

    pub fn set(&mut self, key: String, value: Value) {
        unimplemented!()
    }
}

/*

macro_rules! match_binary_operation {
    ($($left:tt $op:tt $right:tt => $out:tt),*) => {
        |a: &MalType, b: &MalType| {
            use MalType::*;
            match (a, b) {
                $(($left(left), $right(right)) => Some($out(left $op right)),)*
                    _                          => None,
            }
        }
    }
}

macro_rules! arithmetic_operation {
    ($op:tt) => {
        Callable::new2(|a, b| {
            let matchers = match_binary_operation! {
                Int $op Int => Int,
                Float $op Float => Float
            };

            matchers(a, b).unwrap()
        })
    }
}

*/

pub(crate) enum Value {
}

pub(crate) struct Callable<'a>(Box<'a + Fn(&[MalType]) -> MalType>);

impl<'a> Callable<'a> {
    pub fn new2<F>(f: F) -> Self
        where F: 'a + Fn(&MalType, &MalType) -> MalType,
    {
        Callable(Box::new(move |args| {
            if args.len() != 2 { panic!("function expected 2 arguments, got {}", args.len()); }
            f(&args[0], &args[1])
        }))
    }

    pub fn call(&self, args: &[MalType]) -> MalType {
        self.0(args)
    }
}

