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

/// command defines structures for configuring a command-line binary's
/// command(s). A binary might have several (sub-)Commands (e.g. like Git, where
/// commit, log, etc. are all commands), or it might just have a single Command
/// if it's entirely a single-use binary.
pub mod command;
/// main_impl provides command-line-application-specific main() implementations.
pub mod main_impl;
/// parse_and_execute provides functions to parse command-line arguments, and
/// execute the relevant command based upon those arguments.
pub mod parse_and_execute;
/// spec defines the structures which are used to describe a single command-line
/// flag (Spec) or a set of flags as they relate to a command (Specs).
pub mod spec;
/// value defines structures which represent the values obtained by parsing
/// command-line flags, and utilities for accessing them in a type-safe way.
pub mod value;

mod help;

// Re-export the most commonly used symbols, so most users of this module can
// just do "use bdrck::flags::*;" and get the right thing.

pub use self::command::{Command, CommandCallback, CommandResult};
pub use self::main_impl::{main_impl_multiple_commands, main_impl_single_command};
pub use self::spec::{Spec, Specs};
pub use self::value::Values;
