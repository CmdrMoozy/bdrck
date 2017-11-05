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
use params::argument::Argument;
use params::option::Option;
use std::collections::HashMap;
use std::fmt;
use std::result;
use std::string::String;
use std::vec::Vec;

/// A command is a single sub-command for a given program. Each command has
/// its own description as well as sets of options and arguments that it
/// accepts.
#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub help: String,
    pub options: Vec<Option>,
    pub arguments: Vec<Argument>,
    pub last_argument_is_variadic: bool,
}

impl Command {
    pub fn new(name: &str,
               help: &str,
               options: Vec<Option>,
               arguments: Vec<Argument>,
               last_argument_is_variadic: bool)
               -> Result<Command> {
        //! Constructs a new Command structure. Performs some validation on the inputs,
        //! and returns either a valid Command or an appropriate error.

        // All arguments after the first one with a default value must also have
        // default values.
        if !arguments.iter()
            .skip_while(|a| a.default_value.is_none())
            .all(|a| a.default_value.is_some()) {
            bail!("Missing default argument value");
        }

        // All arguments other than the last one must have at most one default value.
        if !arguments.is_empty() &&
           !&arguments[..arguments.len() - 1]
            .iter()
            .all(|a| a.default_value.as_ref().map_or(0, |dv| dv.len()) <= 1) {
            bail!("Too many default argument values");
        }

        // The last argument can have more than one default value only if it is
        // variadic.
        let has_multiple_defaults = arguments.iter().last().map_or(false, |a| {
            a.default_value.as_ref().map_or(false, |dv| dv.len() > 1)
        });
        if !last_argument_is_variadic && has_multiple_defaults {
            bail!("Too many default argument values");
        }

        Ok(Command {
            name: name.to_owned(),
            help: help.to_owned(),
            options: options,
            arguments: arguments,
            last_argument_is_variadic: last_argument_is_variadic,
        })
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Command) -> bool { self.name == other.name }
}

pub type CommandResult<E> = result::Result<(), E>;

pub type CommandCallback<'a, E> = Box<FnMut(HashMap<String, String>,
                                            HashMap<String, bool>,
                                            HashMap<String, Vec<String>>)
                                            -> CommandResult<E> + 'a>;

/// An `ExecutableCommand` is a Command alongside a callback function which can
/// be called to execute the command in question.
pub struct ExecutableCommand<'a, E> {
    pub command: Command,
    callback: CommandCallback<'a, E>,
}

impl<'a, E> fmt::Debug for ExecutableCommand<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(format!("{:#?}", self.command).as_ref())?;
        Ok(())
    }
}

impl<'a, E> PartialEq<Command> for ExecutableCommand<'a, E> {
    fn eq(&self, other: &Command) -> bool { self.command == *other }
}

impl<'a, E> ExecutableCommand<'a, E> {
    pub fn new(command: Command, callback: CommandCallback<'a, E>) -> ExecutableCommand<'a, E> {
        ExecutableCommand {
            command: command,
            callback: callback,
        }
    }

    pub fn execute(&mut self,
                   options: HashMap<String, String>,
                   flags: HashMap<String, bool>,
                   arguments: HashMap<String, Vec<String>>)
                   -> CommandResult<E> {
        self.callback.as_mut()(options, flags, arguments)
    }
}
