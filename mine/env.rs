use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use crate::types::MalType;

#[derive(Debug)]
pub(crate) struct Env<'a> {
    outer: Option<&'a Env<'a>>,
    data: HashMap<String, MalType>,
}

impl<'a> Env<'a> {
    pub fn new() -> Self {
        Env {
            outer: None,
            data: HashMap::new(),
        }
    }

    pub fn wrap(outer: &'a Env) -> Self {
        Env {
            outer: Some(outer),
            data: HashMap::new(),
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&MalType>
        where String: Borrow<Q>,
              Q: Hash + Eq,
    {
        self.data.get(key)
            .or_else(|| {
                self.outer.as_ref()
                    .and_then(|outer| outer.get(key))
            })
    }

    pub fn set(&mut self, key: String, value: MalType) {
        self.data.insert(key, value);
    }
}
