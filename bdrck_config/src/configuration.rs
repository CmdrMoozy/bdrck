use ::error::{Error, ErrorKind, Result};
use ::generic::GenericConfiguration;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identifier {
    pub application: String,
    pub name: String,
}

lazy_static! {
    static ref SINGLETONS: HashMap<Identifier, Mutex<Box<Any + Send>>> = HashMap::new();
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub fn instance_apply<T: 'static, R, F: FnOnce(&GenericConfiguration<T>) -> R>(id: &Identifier,
                                                                               f: F)
                                                                               -> Result<R> {
    match SINGLETONS.get(id) {
        Some(mutex) => {
            match (*lock(mutex)).downcast_ref() {
                Some(config) => Ok(f(config)),
                None => Err(Error::new(ErrorKind::IdentifierTypeMismatch)),
            }
        },
        None => Err(Error::new(ErrorKind::UnrecognizedIdentifier)),
    }
}

pub fn instance_apply_mut<T: 'static, R, F: FnOnce(&mut GenericConfiguration<T>) -> R>
    (id: &Identifier,
     f: F)
     -> Result<R> {
    use std::ops::DerefMut;
    match SINGLETONS.get(id) {
        Some(mutex) => {
            let mut guard = lock(mutex);
            match guard.deref_mut().downcast_mut() {
                Some(config) => Ok(f(config)),
                None => Err(Error::new(ErrorKind::IdentifierTypeMismatch)),
            }
        },
        None => Err(Error::new(ErrorKind::UnrecognizedIdentifier)),
    }
}

pub fn get<T: Clone + Serialize + Deserialize + 'static>(id: &Identifier) -> Result<T> {
    instance_apply::<T, T, _>(id, |instance| instance.get().clone())
}

pub fn set<T: Clone + Serialize + Deserialize + 'static>(id: &Identifier, config: T) -> Result<()> {
    instance_apply_mut(id, move |instance| instance.set(config))
}

pub fn reset<T: Clone + Serialize + Deserialize + 'static>(id: &Identifier) -> Result<()> {
    instance_apply_mut::<T, _, _>(id, |instance| instance.reset())
}

pub fn persist<T: Clone + Serialize + Deserialize + 'static>(id: &Identifier) -> Result<()> {
    try!(instance_apply::<T, _, _>(id, |instance| instance.persist()))
}
