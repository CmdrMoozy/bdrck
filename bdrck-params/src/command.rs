use std::collections::HashMap;
use std::fmt;
use std::string::String;
use std::vec::Vec;

use super::ErrorKind;
use super::ParamsError;
use super::argument::Argument;
use super::option::Option;

/// A command is a single sub-command for a given program. Each command has
/// its own description as well as sets of options and arguments that it
/// accepts.
pub struct Command {
    name: String,
    help: String,
    options: Vec<Option>,
    arguments: Vec<Argument>,
    last_argument_is_variadic: bool,
    callback: Box<Fn(&HashMap<&str, &str>,
                     &HashMap<&str, bool>,
                     &HashMap<&str, Vec<&str>>)>,
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(f.write_str(format!("Command {{ name: {:#?}, help: {:#?}, options: {:#?}, \
                                  arguments: {:#?}, last_argument_is_variadic: {:#?} }}",
                                 self.name,
                                 self.help,
                                 self.options,
                                 self.arguments,
                                 self.last_argument_is_variadic)
            .as_ref()));
        Ok(())
    }
}

impl Command {
    pub fn new(name: String,
               help: String,
               options: Vec<Option>,
               arguments: Vec<Argument>,
               last_argument_is_variadic: bool,
               callback: Box<Fn(&HashMap<&str, &str>,
                                &HashMap<&str, bool>,
                                &HashMap<&str, Vec<&str>>)>)
               -> Result<Command, ParamsError> {
        //! Constructs a new Command structure. Performs some validation on the inputs,
        //! and returns either a valid Command or an appropriate error.

        // All arguments after the first one with a default value must also have
        // default values.
        if !arguments.iter()
            .skip_while(|a| a.default_value.is_none())
            .all(|a| a.default_value.is_some()) {
            return Err(ParamsError { kind: ErrorKind::MissingDefaultArgumentValue });
        }

        // All arguments other than the last one must have at most one default value.
        if arguments.len() > 0 &&
           !&arguments[..arguments.len() - 1]
            .iter()
            .all(|a| a.default_value.as_ref().map_or(0, |dv| dv.len()) <= 1) {
            return Err(ParamsError { kind: ErrorKind::TooManyDefaultArgumentValues });
        }

        // The last argument can have more than one default value only if it is
        // variadic.
        if !last_argument_is_variadic &&
           arguments.iter().last().map_or(false, |a| {
            a.default_value.as_ref().map_or(false, |dv| dv.len() > 1)
        }) {
            return Err(ParamsError { kind: ErrorKind::TooManyDefaultArgumentValues });
        }

        Ok(Command {
            name: name,
            help: help,
            options: options,
            arguments: arguments,
            last_argument_is_variadic: last_argument_is_variadic,
            callback: callback,
        })
    }

    pub fn get_name(&self) -> &String { &self.name }
    pub fn get_help(&self) -> &String { &self.help }
    pub fn get_options(&self) -> &Vec<Option> { &self.options }
    pub fn get_arguments(&self) -> &Vec<Argument> { &self.arguments }
    pub fn last_argument_is_variadic(&self) -> bool { self.last_argument_is_variadic }

    pub fn execute(&self,
                   options: &HashMap<&str, &str>,
                   flags: &HashMap<&str, bool>,
                   arguments: &HashMap<&str, Vec<&str>>) {
        self.callback.as_ref()(options, flags, arguments);
    }
}
