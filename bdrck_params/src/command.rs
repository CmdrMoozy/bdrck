use std::collections::HashMap;
use std::fmt;
use std::string::String;
use std::vec::Vec;

use super::argument::Argument;
use super::error::*;
use super::option::Option;

/// A command is a single sub-command for a given program. Each command has
/// its own description as well as sets of options and arguments that it
/// accepts.
#[derive(Debug)]
pub struct Command {
    name: String,
    help: String,
    options: Vec<Option>,
    arguments: Vec<Argument>,
    last_argument_is_variadic: bool,
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
            .skip_while(|a| a.default_value().is_none())
            .all(|a| a.default_value().is_some()) {
            return Err(Error::new(ErrorKind::MissingDefaultArgumentValue));
        }

        // All arguments other than the last one must have at most one default value.
        if arguments.len() > 0 &&
           !&arguments[..arguments.len() - 1]
            .iter()
            .all(|a| a.default_value().map_or(0, |dv| dv.len()) <= 1) {
            return Err(Error::new(ErrorKind::TooManyDefaultArgumentValues));
        }

        // The last argument can have more than one default value only if it is
        // variadic.
        if !last_argument_is_variadic &&
           arguments.iter().last().map_or(false,
                                          |a| a.default_value().map_or(false, |dv| dv.len() > 1)) {
            return Err(Error::new(ErrorKind::TooManyDefaultArgumentValues));
        }

        Ok(Command {
            name: name.to_owned(),
            help: help.to_owned(),
            options: options,
            arguments: arguments,
            last_argument_is_variadic: last_argument_is_variadic,
        })
    }

    pub fn get_name(&self) -> &String { &self.name }
    pub fn get_help(&self) -> &String { &self.help }
    pub fn get_options(&self) -> &Vec<Option> { &self.options }
    pub fn get_arguments(&self) -> &Vec<Argument> { &self.arguments }
    pub fn last_argument_is_variadic(&self) -> bool { self.last_argument_is_variadic }
}

impl PartialEq for Command {
    fn eq(&self, other: &Command) -> bool { self.name == other.name }
}

type CommandCallback<'a> = Box<FnMut(&HashMap<&str, String>,
                                     &HashMap<&str, bool>,
                                     &HashMap<&str, Vec<String>>) + 'a>;

/// An ExecutableCommand is a Command alongside a callback function which can
/// be called to execute the command in question.
pub struct ExecutableCommand<'a, 'b> {
    command: &'a Command,
    callback: CommandCallback<'b>,
}

impl<'a, 'b> fmt::Debug for ExecutableCommand<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(f.write_str(format!("{:#?}", self.command).as_ref()));
        Ok(())
    }
}

impl<'a, 'b> PartialEq<Command> for ExecutableCommand<'a, 'b> {
    fn eq(&self, other: &Command) -> bool { self.command == other }
}

impl<'a, 'b> ExecutableCommand<'a, 'b> {
    pub fn new(command: &'a Command, callback: CommandCallback<'b>) -> ExecutableCommand<'a, 'b> {
        ExecutableCommand {
            command: command,
            callback: callback,
        }
    }

    pub fn get_command(&self) -> &'a Command { self.command }

    pub fn execute(&mut self,
                   options: &HashMap<&str, String>,
                   flags: &HashMap<&str, bool>,
                   arguments: &HashMap<&str, Vec<String>>) {
        self.callback.as_mut()(options, flags, arguments);
    }
}
