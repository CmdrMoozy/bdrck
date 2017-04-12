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
