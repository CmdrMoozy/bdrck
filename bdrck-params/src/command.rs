use std::string::String;
use std::vec::Vec;

use super::argument::Argument;
use super::option::Option;

/// A command is a single sub-command for a given program. Each command has
/// its own description as well as sets of options and arguments that it
/// accepts.
pub struct Command {
    pub name: String,
    pub help: String,
    pub options: Vec<Option>,
    pub arguments: Vec<Argument>,
    pub last_argument_is_variadic: bool,
}
