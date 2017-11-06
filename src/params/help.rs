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
use params::command::{Command, ExecutableCommand};
use std::fmt::Write;

pub fn print_program_help<'cbl, E>(
    f: &mut Write,
    program: &str,
    commands: &[ExecutableCommand<'cbl, E>],
) -> Result<()> {
    let mut s = String::new();

    write!(
        s,
        "Usage: {} command [options ...] [arguments ...]\n",
        program
    )?;
    write!(s, "Available commands:\n")?;
    for command in commands.iter() {
        write!(s, "\t{} - {}\n", command.command.name, command.command.help)?;
    }

    write!(f, "{}", s)?;
    Ok(())
}

pub fn print_command_help(
    f: &mut Write,
    program: &str,
    command: &Command,
    print_command_name: bool,
) -> Result<()> {
    let mut s = String::new();

    write!(s, "Usage: {} ", program)?;
    if print_command_name {
        write!(s, "{} ", command.name)?;
    }
    write!(s, "[options ...] [arguments ...]\n")?;

    if !command.options.is_empty() {
        write!(s, "\nOptions:\n")?;
        for option in &command.options {
            write!(s, "\t--{}", option.name)?;
            if let Some(short_name) = option.short_name.as_ref() {
                write!(s, ", -{}", short_name)?;
            }
            write!(s, " - {}", option.help)?;

            if option.is_flag {
                write!(s, " [Flag, default: off]")?;
            } else if let Some(default_value) = option.default_value.as_ref() {
                write!(s, " [Default: {}]", default_value)?;
            }

            write!(s, "\n")?;
        }
    }

    if !command.arguments.is_empty() {
        write!(s, "\nPositional arguments:")?;
        for argument in &command.arguments {
            write!(s, "\n\t{}", argument)?;
        }
        if command.last_argument_is_variadic {
            write!(s, " [One or more]")?;
        }
        write!(s, "\n")?;
    }

    write!(f, "{}", s)?;
    Ok(())
}
