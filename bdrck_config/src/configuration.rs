// Copyright 2015 Axel Rasmussen
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use error::{Error, Result};
use msgpack::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::any::Any;
use std::boxed::Box;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identifier {
    pub application: String,
    pub name: String,
}

#[cfg(target_os = "windows")]
fn get_configuration_directory(application: &str) -> Result<PathBuf> {
    let mut path = PathBuf::from(env::var("APPDATA")?);
    path.push(application);

    fs::create_dir_all(path.as_path())?;
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
    path.push(env::var("XDG_CONFIG_HOME").map(PathBuf::from)
        .or(env::var("HOME").map(|home| {
            let mut home = PathBuf::from(home);
            home.push(".config");
            home
        }))?);
    path.push(application);

    fs::create_dir_all(path.as_path())?;
    if !path.is_dir() {
        bail!("Configuration directory is not a directory");
    }

    Ok(path)
}

fn get_configuration_path(id: &Identifier, custom_path: Option<&Path>) -> Result<PathBuf> {
    custom_path.map_or({
                           let mut path = PathBuf::new();
                           path.push(get_configuration_directory(id.application.as_str())
                               ?
                               .as_path());
                           path.push(id.name.clone() + ".mp");
                           Ok(path)
                       },
                       |custom_path| Ok(PathBuf::from(custom_path)))
}

fn serialize<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    v.serialize(&mut Serializer::new(&mut buf))?;
    Ok(buf)
}

fn deserialize<T: Clone + DeserializeOwned>(path: &PathBuf, default: &T) -> Result<T> {
    match fs::File::open(path) {
        Ok(file) => {
            let mut deserializer = Deserializer::new(file);
            Ok(Deserialize::deserialize(&mut deserializer)?)
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

impl<T: Clone + Serialize + DeserializeOwned> Configuration<T> {
    pub fn new(id: Identifier, default: T, custom_path: Option<&Path>) -> Result<Configuration<T>> {
        let path: PathBuf = get_configuration_path(&id, custom_path)?;
        let current: T = deserialize(&path, &default)?;

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

        self.path
            .parent()
            .map_or(Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid configuration path")),
                    fs::create_dir_all)?;
        let data = serialize(&self.current)?;
        let mut file = fs::File::create(self.path.as_path())?;
        file.write_all(data.as_slice())?;
        file.flush()?;
        Ok(())
    }
}

lazy_static! {
    static ref SINGLETONS: Mutex<HashMap<Identifier, Box<Any + Send>>> = Mutex::new(HashMap::new());
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub fn new<T: Clone + Serialize + DeserializeOwned + Send + 'static>(id: Identifier,
                                                                default: T,
                                                                custom_path: Option<&Path>)
                                                                -> Result<()> {
    use std::ops::DerefMut;
    let config: Configuration<T> = Configuration::new(id.clone(), default, custom_path)?;
    let mut guard = lock(&SINGLETONS);
    guard.deref_mut().insert(id, Box::new(config));
    Ok(())
}

pub fn remove<T: Clone + Serialize + DeserializeOwned + 'static>(id: &Identifier) -> Result<()> {
    let mut guard = lock(&SINGLETONS);

    if let Some(instance) = guard.get(id) {
        if let Some(config) = instance.downcast_ref::<Configuration<T>>() {
            config.persist()?;
        } else {
            bail!("Wrong type specified for configuration with the given identifier");
        }
    }

    match guard.remove(id) {
        Some(_) => Ok(()),
        None => bail!("Unrecognized configuration identifier"),
    }
}

pub fn instance_apply<T: 'static, R, F: FnOnce(&Configuration<T>) -> R>(id: &Identifier,
                                                                        f: F)
                                                                        -> Result<R> {
    match lock(&SINGLETONS).get(id) {
        Some(instance) => {
            match instance.downcast_ref() {
                Some(config) => Ok(f(config)),
                None => bail!("Wrong type specified for configuration with the given identifier"),
            }
        },
        None => bail!("Unrecognized configuration identifier"),
    }
}

pub fn instance_apply_mut<T: 'static, R, F: FnOnce(&mut Configuration<T>) -> R>(id: &Identifier,
                                                                                f: F)
                                                                                -> Result<R> {
    match lock(&SINGLETONS).get_mut(id) {
        Some(instance) => {
            match instance.downcast_mut() {
                Some(config) => Ok(f(config)),
                None => bail!("Wrong type specified for configuration with the given identifier"),
            }
        },
        None => bail!("Unrecognized configuration identifier"),
    }
}

pub fn get<T: Clone + Serialize + DeserializeOwned + 'static>(id: &Identifier) -> Result<T> {
    instance_apply::<T, T, _>(id, |instance| instance.get().clone())
}

pub fn set<T: Clone + Serialize + DeserializeOwned + 'static>(id: &Identifier, config: T) -> Result<()> {
    instance_apply_mut(id, move |instance| instance.set(config))
}

pub fn reset<T: Clone + Serialize + DeserializeOwned + 'static>(id: &Identifier) -> Result<()> {
    instance_apply_mut::<T, _, _>(id, |instance| instance.reset())
}

pub fn persist<T: Clone + Serialize + DeserializeOwned + 'static>(id: &Identifier) -> Result<()> {
    instance_apply::<T, _, _>(id, |instance| instance.persist())?
}
