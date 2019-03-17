use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::types::MalType;

#[derive(Clone, Debug)]
pub(crate) struct Env {
    outer: Option<Rc<Env>>,
    data: HashMap<String, MalType>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            outer: None,
            data: HashMap::new(),
        }
    }

    pub fn wrap(outer: Rc<Env>) -> Self {
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
