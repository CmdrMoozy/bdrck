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
use std::io::Write;

pub fn print_program_help<'cbl, W: Write, E>(
    f: &mut W,
    program: &str,
    commands: &[ExecutableCommand<'cbl, E>],
) -> Result<()> {
    f.write_fmt(format_args!(
        "Usage: {} command [options ...] [arguments ...]\n",
        program
    ))?;
    f.write_fmt(format_args!("Available commands:\n"))?;
    for command in commands.iter() {
        f.write_fmt(format_args!(
            "\t{} - {}\n",
            command.command.name,
            command.command.help
        ))?;
    }

    Ok(())
}

pub fn print_command_help<W: Write>(
    f: &mut W,
    program: &str,
    command: &Command,
    print_command_name: bool,
) -> Result<()> {
    f.write_fmt(format_args!("Usage: {}", program))?;
    if print_command_name {
        f.write_fmt(format_args!("{} ", command.name))?;
    }
    f.write_fmt(format_args!("[options ...] [arguments ...]\n"))?;

    if !command.options.is_empty() {
        f.write_fmt(format_args!("\nOptions:\n"))?;
        for option in &command.options {
            f.write_fmt(format_args!("\t--{}", option.name))?;
            if let Some(short_name) = option.short_name.as_ref() {
                f.write_fmt(format_args!(", -{}", short_name))?;
            }
            f.write_fmt(format_args!(" - {}", option.help))?;

            if option.is_flag {
                f.write_fmt(format_args!(" [Flag, default: off]"))?;
            } else if let Some(default_value) = option.default_value.as_ref() {
                f.write_fmt(format_args!(" [Default: {}]", default_value))?;
            }

            f.write_fmt(format_args!("\n"))?;
        }
    }

    if !command.arguments.is_empty() {
        f.write_fmt(format_args!("\nPositional arguments:"))?;
        for argument in &command.arguments {
            f.write_fmt(format_args!("\n\t{}", argument))?;
        }
        if command.last_argument_is_variadic {
            f.write_fmt(format_args!(" [One or more]"))?;
        }
        f.write_fmt(format_args!("\n"))?;
    }

    Ok(())
}
