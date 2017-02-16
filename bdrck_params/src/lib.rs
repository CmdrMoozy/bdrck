#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

pub mod argument;
pub mod command;
pub mod error;
pub mod io;
pub mod main_impl;
pub mod option;
pub mod parse_and_execute;

mod help;
mod parsed_parameters;

#[cfg(test)]
mod tests;

// Re-export most commonly used symbols, to allow using this library with just
// one "use".

pub use argument::Argument;
pub use command::{Command, CommandCallback, CommandResult, ExecutableCommand};
pub use main_impl::{main_impl_multiple_commands, main_impl_single_command};
pub use option::Option;
