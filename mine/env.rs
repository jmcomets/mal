// use std::ops::{Deref, DerefMut};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::types::MalType;

#[derive(Clone, Debug)]
pub(crate) struct Env {
    outer: Option<EnvRef>,
    data: HashMap<String, MalType>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            outer: None,
            data: HashMap::new(),
        }
    }

    pub fn wrap(outer: EnvRef) -> Self {
        Env {
            outer: Some(outer),
            data: HashMap::new(),
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<MalType>
        where String: Borrow<Q>,
              Q: Hash + Eq,
    {
        self.data.get(key)
            .map(Clone::clone)
            .or_else(|| {
                self.outer.as_ref()
                    .and_then(|outer| outer.get(key))
            })
    }

    pub fn set(&mut self, key: String, value: MalType) {
        self.data.insert(key, value);
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EnvRef(Rc<RefCell<Env>>);

impl EnvRef {
    pub fn new(env: Env) -> Self {
        EnvRef(Rc::new(RefCell::new(env)))
    }

    pub fn refer_to(outer: EnvRef) -> Self {
        Self::new(Env::wrap(outer))
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<MalType>
        where String: Borrow<Q>,
              Q: Hash + Eq,
    {
        (*self.0).borrow().get(key)
    }

    pub fn set(&mut self, key: String, value: MalType) {
        self.0
            .borrow_mut()
            .set(key, value)
    }
}
