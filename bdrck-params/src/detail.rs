use std::env;
use std::option::Option as Optional;
use std::string::String;
use std::vec::Vec;

use super::command::Command;

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

pub fn parse_command_from_env_args<'a, CI>(commands: CI) -> (Vec<String>, Optional<&'a Command>)
    where CI: Iterator<Item = &'a Command>
{
    let parameters = env::args()
        .skip(1) // Skip the first argument, which is our executable.
        .collect::<Vec<_>>();
    let command: Optional<&'a Command> = parse_command(parameters.iter(), commands);
    return (parameters, command);
}
