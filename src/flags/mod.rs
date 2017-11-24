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

pub mod argument;
pub mod command;
pub mod main_impl;
pub mod option;
pub mod parse_and_execute;
pub mod parsed_parameters;
pub mod spec;

mod help;
mod util;

// Re-export most commonly used symbols, to allow using this library with just
// one "use".

pub use self::argument::Argument;
pub use self::command::{Command, CommandCallback, CommandResult, ExecutableCommand};
pub use self::main_impl::{main_impl_multiple_commands, main_impl_single_command};
pub use self::option::Option;
