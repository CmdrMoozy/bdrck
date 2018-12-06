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

use crate::error::*;
use crate::flags::command::{parse_command, Command, CommandResult};
use crate::flags::help;
use crate::flags::value::Values;
use std::io::Write;

fn parse_and_execute_impl<E, W: Write>(
    program: &str,
    args: &[String],
    commands: Vec<Command<E>>,
    mut output_writer: Option<W>,
    print_program_help: bool,
    print_command_name: bool,
) -> Result<CommandResult<E>> {
    let mut args_iterator = args.iter().peekable();

    let mut command = parse_command(
        program,
        &mut args_iterator,
        commands,
        output_writer.as_mut(),
        print_program_help,
    )?;

    let values = match Values::new(&command.flags, args_iterator) {
        Ok(vs) => vs,
        Err(e) => {
            help::print_command_help(
                output_writer.as_mut(),
                program,
                &command,
                print_command_name,
            )?;
            return Err(e);
        }
    };

    Ok(command.execute(values))
}

/// This function parses the given program parameters, and calls the appropriate
/// command callback. It prints out usage information if the parameters are
/// invalid, and returns a reasonable exit code for the process.
///
/// This is the function which should be used for typical multi-command
/// programs.
pub fn parse_and_execute_command<E, W: Write>(
    program: &str,
    args: &[String],
    commands: Vec<Command<E>>,
    output_writer: Option<W>,
) -> Result<CommandResult<E>> {
    parse_and_execute_impl(program, args, commands, output_writer, true, true)
}

/// This function parses the given program parameters and calls the given
/// command's callback. It prints out usage information if the parameters are
/// invalid, and returns a reasonable exit code for the process.
///
/// This is the function which should be used for typical single-command
/// programs.
pub fn parse_and_execute<E, W: Write>(
    program: &str,
    args: &[String],
    command: Command<E>,
    output_writer: Option<W>,
) -> Result<CommandResult<E>> {
    let args: Vec<String> = Some(command.name.clone())
        .into_iter()
        .chain(args.iter().cloned())
        .collect();
    parse_and_execute_impl(
        program,
        args.as_slice(),
        vec![command],
        output_writer,
        false,
        false,
    )
}
