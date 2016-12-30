use ::error::Result;
use msgpack::Serializer;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use std::vec::Vec;

fn serialize<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    try!(v.serialize(&mut Serializer::new(&mut buf)));
    Ok(buf)
}

pub struct GenericConfiguration<T> {
    path: String,
    defaults: T,
    current: T,
}

impl<T: Clone + Serialize + Deserialize> GenericConfiguration<T> {
    pub fn new(path: &str, defaults: T, current: T) -> GenericConfiguration<T> {
        GenericConfiguration {
            path: path.to_owned(),
            defaults: defaults,
            current: current,
        }
    }

    pub fn get(&self) -> &T { &self.current }

    pub fn set(&mut self, config: T) { self.current = config }

    pub fn reset(&mut self) { self.current = self.defaults.clone() }

    pub fn persist(&self) -> Result<()> {
        use std::io::Write;

        let path = Path::new(self.path.as_str());
        try!(path.parent().map_or(Err(io::Error::new(io::ErrorKind::InvalidInput,
                                                     "Invalid configuration path")),
                                  |dir| fs::create_dir_all(dir)));
        let data = try!(serialize(&self.current));
        let mut file = try!(fs::File::create(path));
        try!(file.write_all(data.as_slice()));
        Ok(())
    }
}
