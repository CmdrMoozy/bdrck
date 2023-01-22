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

use crate::error::{Error, Result};
use once_cell::sync::Lazy;
use rmp_serde::{Deserializer, Serializer};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::boxed::Box;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

/// An Identifier uniquely identifies a configuration file.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identifier {
    /// The unique name of the application this configuration file belongs to.
    pub application: String,
    /// The application-specific unique name for this particular configuration
    /// file.
    pub name: String,
}

#[cfg(target_os = "windows")]
fn get_configuration_directory(application: &str) -> Result<PathBuf> {
    let mut path = PathBuf::from(env::var("APPDATA")?);
    path.push(application);

    fs::create_dir_all(path.as_path())?;
    if !path.is_dir() {
        return Err(Error::InvalidArgument(format!(
            "configuration path '{}' is not a directory",
            path.as_path().display()
        )));
    }

    Ok(path)
}

#[cfg(not(target_os = "windows"))]
fn get_configuration_directory(application: &str) -> Result<PathBuf> {
    let mut path = PathBuf::new();
    path.push(
        env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or(env::var("HOME").map(|home| {
                let mut home = PathBuf::from(home);
                home.push(".config");
                home
            }))?,
    );
    path.push(application);

    fs::create_dir_all(path.as_path())?;
    if !path.is_dir() {
        return Err(Error::InvalidArgument(format!(
            "configuration path '{}' is not a directory",
            path.as_path().display()
        )));
    }

    Ok(path)
}

fn get_configuration_path(id: &Identifier, custom_path: Option<&Path>) -> Result<PathBuf> {
    custom_path.map_or(
        {
            let mut path = PathBuf::new();
            path.push(get_configuration_directory(id.application.as_str())?.as_path());
            path.push(id.name.clone() + ".mp");
            Ok(path)
        },
        |custom_path| Ok(PathBuf::from(custom_path)),
    )
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
        }
        Err(error) => match error.kind() {
            io::ErrorKind::NotFound => Ok(default.clone()),
            _ => Err(Error::from(error)),
        },
    }
}

/// A Configuration represents a set of configuration values, initially loaded
/// from disk, and which can be persisted back to disk e.g. just before the
/// application exits. Generally it is expected that only one instance per
/// Identifier is needed globally, and the other functions in this module are
/// intended to provide an easy singleton interface for this class.
pub struct Configuration<T> {
    path: PathBuf,
    default: T,
    current: T,
}

impl<T: Clone + Serialize + DeserializeOwned> Configuration<T> {
    /// Initialize a new Configuration with the given identifier, default set of
    /// configuration values, and custom disk persistence path (optional). An
    /// error might occur if determining the persistence path to use fails, or
    /// if deserializing the previously persisted configuration (if any) fails.
    pub fn new(id: Identifier, default: T, custom_path: Option<&Path>) -> Result<Configuration<T>> {
        let path: PathBuf = get_configuration_path(&id, custom_path)?;
        let current: T = deserialize(&path, &default)?;

        Ok(Configuration {
            path: path,
            default: default,
            current: current,
        })
    }

    /// Return this instance's current set of configuration values.
    pub fn get(&self) -> &T {
        &self.current
    }

    /// Replace all existing configuration values with the given entirely new
    /// set of configuration values.
    pub fn set(&mut self, config: T) {
        self.current = config
    }

    /// Reset all of this instance's configuration values back to their default
    /// values (specified previously on construction).
    pub fn reset(&mut self) {
        self.current = self.default.clone()
    }

    /// Persist this instance's current configuration values to disk, so they
    /// can be re-loaded on the next construction.
    pub fn persist(&self) -> Result<()> {
        use std::io::Write;

        self.path.parent().map_or(
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid configuration path",
            )),
            fs::create_dir_all,
        )?;
        let data = serialize(&self.current)?;
        let mut file = fs::File::create(self.path.as_path())?;
        file.write_all(data.as_slice())?;
        file.flush()?;
        Ok(())
    }
}

static SINGLETONS: Lazy<Mutex<HashMap<Identifier, Box<dyn Any + Send>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// new initializes a new configuration singleton with the given identifer,
/// default set of configuration values, and custom disk persistence path
/// (optional). An error might occur if determining the persistence path to use
/// fails, or if deserializing the previously persisted configuration (if any)
/// fails.
pub fn new<T: Clone + Serialize + DeserializeOwned + Send + 'static>(
    id: Identifier,
    default: T,
    custom_path: Option<&Path>,
) -> Result<()> {
    use std::ops::DerefMut;
    let config: Configuration<T> = Configuration::new(id.clone(), default, custom_path)?;
    let mut guard = lock(&SINGLETONS);
    guard.deref_mut().insert(id, Box::new(config));
    Ok(())
}

/// remove persists and then removes the configuration singleton matching the
/// given identifier. After calling this function, the configuration in question
/// will be unavailable.
pub fn remove<T: Clone + Serialize + DeserializeOwned + 'static>(id: &Identifier) -> Result<()> {
    let mut guard = lock(&SINGLETONS);

    if let Some(instance) = guard.get(id) {
        if let Some(config) = instance.downcast_ref::<Configuration<T>>() {
            config.persist()?;
        } else {
            return Err(Error::InvalidArgument(format!(
                "wrong type specified for configuration {:?}",
                id
            )));
        }
    }

    match guard.remove(id) {
        Some(_) => Ok(()),
        None => {
            return Err(Error::InvalidArgument(format!(
                "unrecognized configuration identifier: {:?}",
                id
            )));
        }
    }
}

/// instance_apply is a very generic function which applies the given function
/// to the configuration singleton matching the given identifier. It is an error
/// if the identifier is unrecognized, or if the given callback operates on a
/// Configuration of the wrong type.
pub fn instance_apply<T: 'static, R, F: FnOnce(&Configuration<T>) -> R>(
    id: &Identifier,
    f: F,
) -> Result<R> {
    match lock(&SINGLETONS).get(id) {
        Some(instance) => match instance.downcast_ref() {
            Some(config) => Ok(f(config)),
            None => {
                return Err(Error::InvalidArgument(format!(
                    "wrong type specified for configuration {:?}",
                    id
                )));
            }
        },
        None => {
            return Err(Error::InvalidArgument(format!(
                "unrecognized configuration identifier: {:?}",
                id
            )));
        }
    }
}

/// instance_apply_mut is a very generic function which applies the given
/// mutation function once to the configuration singleton matching the given
/// identifier. It is an error if the identifier is unrecognized, or if the
/// given callback operates on a Configuration of the wrong type.
pub fn instance_apply_mut<T: 'static, R, F: FnOnce(&mut Configuration<T>) -> R>(
    id: &Identifier,
    f: F,
) -> Result<R> {
    match lock(&SINGLETONS).get_mut(id) {
        Some(instance) => match instance.downcast_mut() {
            Some(config) => Ok(f(config)),
            None => {
                return Err(Error::InvalidArgument(format!(
                    "wrong type specified for configuration {:?}",
                    id
                )));
            }
        },
        None => {
            return Err(Error::InvalidArgument(format!(
                "unrecognized configuration identifier: {:?}",
                id
            )));
        }
    }
}

/// get returns the entire current set of configuration values in the
/// configuration singleton matching the given identifier.
pub fn get<T: Clone + Serialize + DeserializeOwned + 'static>(id: &Identifier) -> Result<T> {
    instance_apply::<T, T, _>(id, |instance| instance.get().clone())
}

/// set replaces all existing configuration values with the given entirely new
/// set of configuration values in the configuration singleton matching the
/// given identifier..
pub fn set<T: Clone + Serialize + DeserializeOwned + 'static>(
    id: &Identifier,
    config: T,
) -> Result<()> {
    instance_apply_mut(id, move |instance| instance.set(config))
}

/// reset modifies the configuration singleton matching the given identifier to
/// its default values.
pub fn reset<T: Clone + Serialize + DeserializeOwned + 'static>(id: &Identifier) -> Result<()> {
    instance_apply_mut::<T, _, _>(id, |instance| instance.reset())
}

/// persist writes the configuration singleton matching the given identifier to
/// disk.
pub fn persist<T: Clone + Serialize + DeserializeOwned + 'static>(id: &Identifier) -> Result<()> {
    instance_apply::<T, _, _>(id, |instance| instance.persist())?
}
