use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;

use crate::types::MalType;

pub(crate) struct Env {
    outer: Option<Box<Env>>,
    data: RefCell<HashMap<String, MalType>>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            outer: None,
            data: RefCell::new(HashMap::new()),
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<MalType>
        where String: Borrow<Q>,
              Q: Hash + Eq,
    {
        self.data.borrow().get(key)
            .map(Clone::clone)
            .or_else(|| self.outer.as_ref().and_then(|outer| outer.get(key)))
    }

    pub fn set(&self, key: String, value: MalType) {
        self.data.borrow_mut().insert(key, value);
    }
}
