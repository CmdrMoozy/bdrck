use std::collections::HashMap;
use std::env;
use std::option::Option as Optional;
use std::result::Result;
use std::string::String;
use std::vec::Vec;

use super::ErrorKind;
use super::ParamsError;
use super::command::Command;

// Returns the current program's parameters (accessed essentialy via
// std::env::args()) collected into a Vec. The 0'th parameter (the executable)
// is omitted.
pub fn get_program_parameters() -> Vec<String> {
    return env::args()
        .skip(1) // Skip the first argument, which is our executable.
        .collect::<Vec<String>>();
}

// Look up by name the command indicated by the first element of the given
// range of program parameters. If a matching command could not be found,
// return None instead.
pub fn parse_command<'a, 'b, PI, CI>(mut parameters: PI, mut commands: CI) -> Optional<&'b Command>
    where PI: Iterator<Item = &'a String>,
          CI: Iterator<Item = &'b Command>
{
    if let Some(command_parameter) = parameters.next() {
        return commands.find(|&command| command.name == **command_parameter);
    } else {
        return None;
    }
}

// Constructs maps for options and flags which contain the default values (if
// any) for each of the given command's options. Note that all flags have a
// default value of false.
fn build_default_options(command: &Command) -> (HashMap<&String, &String>, HashMap<&String, bool>) {
    let mut options: HashMap<&String, &String> = HashMap::new();
    let mut flags: HashMap<&String, bool> = HashMap::new();

    for o in &command.options {
        if let Some(ref dv) = o.default_value {
            options.insert(&o.name, dv);
        } else if o.is_flag {
            flags.insert(&o.name, false);
        }
    }

    return (options, flags);
}

// Checks if all of the given command's options are present in the given map
// of option names to values. If an option is missing, returns an error with
// more detailed information. Otherwise, returns None.
fn all_options_are_present(command: &Command,
                           options: &HashMap<&String, &String>)
                           -> Optional<ParamsError> {
    for o in &command.options {
        if o.is_optional || o.is_flag {
            continue;
        }

        if !options.contains_key(&o.name) {
            return Some(ParamsError {
                kind: ErrorKind::MissingOptionValue { name: o.name.clone() },
            });
        }
    }

    return None;
}

// Parse all of the command-line options present in the given range and collect
// them into maps from option name to value and from flag name to value,
// respectively. If options parsing fails (options are missing, or an invalid
// option is encountered), return an error instead with relevant details.
pub fn parse_options<'a, PI>
    (mut parameters: PI,
     command: &Command)
     -> Result<(HashMap<&String, &String>, HashMap<&String, bool>), ParamsError>
    where PI: Iterator<Item = &'a String>
{
    let (options, flags) = build_default_options(command);



    return all_options_are_present(command, &options).map_or(Ok((options, flags)), |e| Err(e));
}
