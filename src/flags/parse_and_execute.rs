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
use flags::command::{CommandResult, ExecutableCommand};
use flags::help;
use flags::parsed_parameters::ParsedParameters;
use flags::parsed_parameters::parse_command;
use std::io::Write;
use std::option::Option as Optional;

fn parse_and_execute_impl<E, W: Write>(
    program: &str,
    parameters: &[String],
    commands: Vec<ExecutableCommand<E>>,
    mut output_writer: Optional<W>,
    print_program_help: bool,
    print_command_name: bool,
) -> Result<CommandResult<E>> {
    let mut parameters_iterator = parameters.iter().peekable();

    let mut command = parse_command(
        program,
        &mut parameters_iterator,
        commands,
        output_writer.as_mut(),
        print_program_help,
    )?;
    let parsed_parameters = match ParsedParameters::new(&command.command, &mut parameters_iterator)
    {
        Ok(p) => p,
        Err(e) => {
            help::print_command_help(
                output_writer.as_mut(),
                program,
                &command.command,
                print_command_name,
            )?;
            return Err(e);
        },
    };

    Ok(command.execute(parsed_parameters))
}

/// This function parses the given program parameters, and calls the appropriate
/// command callback. It prints out usage information if the parameters are
/// invalid, and returns a reasonable exit code for the process.
///
/// This is the function which should be used for typical multi-command
/// programs.
pub fn parse_and_execute_command<E, W: Write>(
    program: &str,
    parameters: &[String],
    commands: Vec<ExecutableCommand<E>>,
    output_writer: Optional<W>,
) -> Result<CommandResult<E>> {
    parse_and_execute_impl(program, parameters, commands, output_writer, true, true)
}

/// This function parses the given program parameters and calls the given
/// command's callback. It prints out usage information if the parameters are
/// invalid, and returns a reasonable exit code for the process.
///
/// This is the function which should be used for typical single-command
/// programs.
pub fn parse_and_execute<E, W: Write>(
    program: &str,
    parameters: &[String],
    command: ExecutableCommand<E>,
    output_writer: Optional<W>,
) -> Result<CommandResult<E>> {
    let parameters: Vec<String> = Some(command.command.name.clone())
        .into_iter()
        .chain(parameters.iter().cloned())
        .collect();
    parse_and_execute_impl(
        program,
        parameters.as_slice(),
        vec![command],
        output_writer,
        false,
        false,
    )
}
