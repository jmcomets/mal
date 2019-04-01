use std::borrow::Borrow;
use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

use fnv::FnvHashMap;

use crate::types::MalType;

#[derive(Debug)]
struct EnvStruct {
    data: RefCell<FnvHashMap<String, MalType>>,
    outer: Option<Env>,
}

impl EnvStruct {
    pub fn new() -> Self {
        EnvStruct {
            outer: None,
            data: RefCell::new(FnvHashMap::default()),
        }
    }

    fn wrap(outer: Env) -> Self {
        EnvStruct {
            outer: Some(outer),
            data: RefCell::new(FnvHashMap::default()),
        }
    }

    fn get<Q: ?Sized>(&self, key: &Q) -> Option<MalType>
        where String: Borrow<Q>,
              Q: Hash + Eq,
    {
        self.data
            .borrow()
            .get(key)
            .map(Clone::clone)
            .or_else(|| {
                self.outer.as_ref()
                    .and_then(|outer| outer.get(key))
            })
    }

    fn set(&self, key: String, value: MalType) {
        self.data
            .borrow_mut()
            .insert(key, value);
    }
}

#[derive(Debug)]
pub(crate) struct Env(Rc<EnvStruct>);

impl Env {
    pub fn pass(&self) -> Self {
        self.clone()
    }

    pub fn wrap(&self) -> Self {
        Self(Rc::new(EnvStruct::wrap(self.clone())))
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<MalType>
        where String: Borrow<Q>,
              Q: Hash + Eq,
    {
        self.0.get(key)
    }

    pub fn set(&self, key: String, value: MalType) {
        self.0.set(key, value)
    }

    // private clone to ensure clear outside use
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for Env {
    fn default() -> Self {
        Self(Rc::new(EnvStruct::new()))
    }
}
