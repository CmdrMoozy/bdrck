extern crate backtrace;
#[macro_use]
extern crate lazy_static;
extern crate rmp_serde as msgpack;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod configuration;
pub mod error;

#[cfg(test)]
mod tests;
