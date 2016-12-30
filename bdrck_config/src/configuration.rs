use ::error::{Error, ErrorKind, Result};
use msgpack::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::boxed::Box;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identifier {
    pub application: String,
    pub name: String,
}

#[cfg(target_os = "windows")]
fn get_configuration_directory(application: &str) -> Result<PathBuf> {
    let mut path = PathBuf::from(try!(env::var("APPDATA")));
    path.push(application);

    try!(fs::create_dir_all(path.as_path()));
    if !path.is_dir() {
        return Err(Error::new(ErrorKind::Io {
            cause: "Configuration directory is not a directory".to_owned(),
        }));
    }

    Ok(path)
}

#[cfg(not(target_os = "windows"))]
fn get_configuration_directory(application: &str) -> Result<PathBuf> {
    let mut path = PathBuf::new();
    path.push(try!(env::var("XDG_CONFIG_HOME")
        .map(|config_home| PathBuf::from(config_home))
        .or(env::var("HOME").map(|home| {
            let mut home = PathBuf::from(home);
            home.push(".config");
            home
        }))));
    path.push(application);

    try!(fs::create_dir_all(path.as_path()));
    if !path.is_dir() {
        return Err(Error::new(ErrorKind::Io {
            cause: "Configuration directory is not a directory".to_owned(),
        }));
    }

    Ok(path)
}

fn get_configuration_path(id: &Identifier, custom_path: Option<&str>) -> Result<PathBuf> {
    custom_path.map_or({
                           let mut path = PathBuf::new();
                           path.push(try!(get_configuration_directory(id.application.as_str()))
                               .as_path());
                           path.push(id.name.clone() + ".mp");
                           Ok(path)
                       },
                       |custom_path| Ok(PathBuf::from(custom_path.to_owned())))
}

fn serialize<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    try!(v.serialize(&mut Serializer::new(&mut buf)));
    Ok(buf)
}

fn deserialize<T: Clone + Deserialize>(path: &PathBuf, default: &T) -> Result<T> {
    match fs::File::open(path) {
        Ok(file) => {
            let mut deserializer = Deserializer::new(file);
            Ok(try!(Deserialize::deserialize(&mut deserializer)))
        },
        Err(error) => {
            match error.kind() {
                io::ErrorKind::NotFound => Ok(default.clone()),
                _ => Err(Error::from(error)),
            }
        },
    }
}

pub struct Configuration<T> {
    path: PathBuf,
    default: T,
    current: T,
}

impl<T: Clone + Serialize + Deserialize> Configuration<T> {
    pub fn new(id: Identifier, default: T, custom_path: Option<&str>) -> Result<Configuration<T>> {
        let path: PathBuf = try!(get_configuration_path(&id, custom_path));
        let current: T = try!(deserialize(&path, &default));

        Ok(Configuration {
            path: path,
            default: default,
            current: current,
        })
    }

    pub fn get(&self) -> &T { &self.current }

    pub fn set(&mut self, config: T) { self.current = config }

    pub fn reset(&mut self) { self.current = self.default.clone() }

    pub fn persist(&self) -> Result<()> {
        use std::io::Write;

        try!(self.path.parent().map_or(Err(io::Error::new(io::ErrorKind::InvalidInput,
                                                          "Invalid configuration path")),
                                       |dir| fs::create_dir_all(dir)));
        let data = try!(serialize(&self.current));
        let mut file = try!(fs::File::create(self.path.as_path()));
        try!(file.write_all(data.as_slice()));
        Ok(())
    }
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

pub fn instance_apply<T: 'static, R, F: FnOnce(&Configuration<T>) -> R>(id: &Identifier,
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

pub fn instance_apply_mut<T: 'static, R, F: FnOnce(&mut Configuration<T>) -> R>(id: &Identifier,
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
