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

use crate::flags::error::{ValueError, ValueResult};
use crate::flags::spec::Specs;
use crate::flags::value::Values;
use std::fmt;
use std::iter::Peekable;

/// An alias for Result, which has an arbitrary Error type. This is used to
/// denote the actual Result returned by a caller-provided Command
/// implementation.
///
/// Note that the Ok value accepted is just (); this library has no logic to
/// deal with application-specific success return values.
pub type CommandResult<E> = ::std::result::Result<(), E>;

/// The caller-provided callback trait object which will be called for a
/// particular Command.
pub type CommandCallback<'a, E> = Box<dyn FnMut(Values) -> CommandResult<E> + 'a>;

/// A command is a single sub-command for a given program. Each command has
/// its own description as well as sets of options and arguments that it
/// accepts.
pub struct Command<'a, E> {
    /// The name of the command. For the common case, the user must specify this
    /// explicitly as the first argument, e.g.
    /// "$BINARY command [ ... flags ... ]".
    pub name: String,
    /// The help string which explains this command's purpose. This is displayed
    /// to the user when appropriate.
    pub help: String,
    /// The full set of flags this Command supports.
    pub flags: Specs,
    /// The callback which is the actual Command's implementation. After parsing
    /// command-line arguments, this callback is called with the flag values.
    pub callback: CommandCallback<'a, E>,
}

impl<'a, E> Command<'a, E> {
    /// A convenience function to construct a new Command with the given
    /// properties.
    pub fn new(name: &str, help: &str, flags: Specs, callback: CommandCallback<'a, E>) -> Self {
        Command {
            name: name.to_owned(),
            help: help.to_owned(),
            flags: flags,
            callback: callback,
        }
    }

    /// A convenience function to call into this Command's implementation with
    /// the given set of parsed command-line flag values.
    pub fn execute(&mut self, values: Values) -> CommandResult<E> {
        self.callback.as_mut()(values)
    }
}

impl<'a, E> fmt::Debug for Command<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(
            format!(
                "Command {{ {:#?}, {:#?}, {:#?} }}",
                self.name, self.help, self.flags
            )
            .as_str(),
        )
    }
}

impl<'a, E> PartialEq for Command<'a, E> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// Look up by name the command indicated by the first element of the given
/// range of program parameters. If locating a matching command fails, an error
/// will be returned instead. Otherwise, the index of the command in the given
/// list of commands is returned.
pub(crate) fn parse_command<'a, 'b, I: Iterator<Item = &'a String>, E>(
    args: &mut Peekable<I>,
    commands: &Vec<Command<'b, E>>,
) -> ValueResult<usize> {
    let idx = match args.next() {
        None => return Err(ValueError::MissingCommand),
        Some(command_arg) => match commands
            .iter()
            .position(|command| command.name == *command_arg)
        {
            None => return Err(ValueError::UnknownCommand(command_arg.to_owned())),
            Some(idx) => idx,
        },
    };
    Ok(idx)
}
