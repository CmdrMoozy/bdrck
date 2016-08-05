use std::string::String;
use std::vec::Vec;

use super::argument::Argument;
use super::option::Option;

pub struct Command {
    pub name: String,
    pub help: String,
    pub options: Vec<Option>,
    pub arguments: Vec<Argument>,
    pub last_argument_is_variadic: bool,
}
