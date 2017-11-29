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

use error::*;
use flags::help;
use flags::spec::Specs;
use flags::value::Values;
use std::fmt;
use std::io::Write;
use std::iter::Peekable;

pub type CommandResult<E> = ::std::result::Result<(), E>;
pub type CommandCallback<'a, E> = Box<FnMut(Values) -> CommandResult<E> + 'a>;

/// A command is a single sub-command for a given program. Each command has
/// its own description as well as sets of options and arguments that it
/// accepts.
pub struct Command<'a, E> {
    pub name: String,
    pub help: String,
    pub flags: Specs,
    pub callback: CommandCallback<'a, E>,
}

impl<'a, E> Command<'a, E> {
    pub fn new(name: &str, help: &str, flags: Specs, callback: CommandCallback<'a, E>) -> Self {
        Command {
            name: name.to_owned(),
            help: help.to_owned(),
            flags: flags,
            callback: callback,
        }
    }

    pub fn execute(&mut self, values: Values) -> CommandResult<E> { self.callback.as_mut()(values) }
}

impl<'a, E> fmt::Debug for Command<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(
            format!(
                "Command {{ {:#?}, {:#?}, {:#?} }}",
                self.name,
                self.help,
                self.flags
            ).as_str(),
        )
    }
}

impl<'a, E> PartialEq for Command<'a, E> {
    fn eq(&self, other: &Self) -> bool { self.name == other.name }
}

/// Look up by name the command indicated by the first element of the given
/// range of program parameters. If a matching command could not be found,
/// return None instead.
pub fn parse_command<'a, 'b, I: Iterator<Item = &'a String>, E, W: Write>(
    program: &str,
    args: &mut Peekable<I>,
    mut commands: Vec<Command<'b, E>>,
    output_writer: Option<&mut W>,
    print_program_help: bool,
) -> Result<Command<'b, E>> {
    let idx: Result<usize> = match args.next() {
        Some(command_arg) => match commands
            .iter()
            .position(|command| command.name == *command_arg)
        {
            Some(command) => Ok(command),
            None => Err(format!("Unrecognized command '{}'", command_arg).into()),
        },
        None => Err("No command specified".into()),
    };

    if let Err(e) = idx {
        if print_program_help {
            help::print_program_help(output_writer, program, &commands)?;
        }
        return Err(e);
    }

    Ok(commands.remove(idx.unwrap()))
}
