extern crate backtrace;
#[macro_use]
extern crate log;

pub mod argument;
pub mod command;
pub mod error;
pub mod help;
pub mod main_impl;
pub mod option;
pub mod parse_and_execute;

mod parsed_parameters;

// Re-export most commonly used symbols, to allow using this library with just
// one "use".

pub use argument::Argument;
pub use command::{Command, CommandCallback, CommandResult, ExecutableCommand};
pub use main_impl::{main_impl_multiple_commands, main_impl_single_command};
pub use option::Option;
