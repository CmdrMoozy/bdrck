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
use params::command::{CommandResult, ExecutableCommand};
use params::help;
use params::io::get_writer_impl;
use params::parsed_parameters::ParsedParameters;
use params::parsed_parameters::parse_command;
use std::string::String;
use std::vec::Vec;

fn parse_and_execute_impl<E>(
    program: &str,
    parameters: &[String],
    commands: Vec<ExecutableCommand<E>>,
    print_program_help: bool,
    print_command_name: bool,
) -> Result<CommandResult<E>> {
    let mut parameters_iterator = parameters.iter().peekable();

    let command = parse_command(
        program,
        &mut parameters_iterator,
        commands,
        print_program_help,
    )?;
    let parsed_parameters = match ParsedParameters::new(&command.command, &mut parameters_iterator)
    {
        Ok(p) => p,
        Err(e) => {
            help::print_command_help(
                &mut get_writer_impl(),
                program,
                &command.command,
                print_command_name,
            )?;
            return Err(e);
        },
    };

    Ok(parsed_parameters.execute(command))
}

/// This function parses the given program parameters, and calls the appropriate
/// command callback. It prints out usage information if the parameters are
/// invalid, and returns a reasonable exit code for the process.
///
/// This is the function which should be used for typical multi-command
/// programs.
pub fn parse_and_execute_command<E>(
    program: &str,
    parameters: &[String],
    commands: Vec<ExecutableCommand<E>>,
) -> Result<CommandResult<E>> {
    parse_and_execute_impl(program, parameters, commands, true, true)
}

/// This function parses the given program parameters and calls the given
/// command's callback. It prints out usage information if the parameters are
/// invalid, and returns a reasonable exit code for the process.
///
/// This is the function which should be used for typical single-command
/// programs.
pub fn parse_and_execute<E>(
    program: &str,
    parameters: &[String],
    command: ExecutableCommand<E>,
) -> Result<CommandResult<E>> {
    let parameters: Vec<String> = Some(command.command.name.clone())
        .into_iter()
        .chain(parameters.iter().cloned())
        .collect();
    parse_and_execute_impl(program, parameters.as_slice(), vec![command], false, false)
}
