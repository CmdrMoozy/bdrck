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
use flags::command::{Command, ExecutableCommand};
use std::io::Write;

pub fn print_program_help<'cbl, W: Write, E>(
    f: Option<&mut W>,
    program: &str,
    commands: &[ExecutableCommand<'cbl, E>],
) -> Result<()> {
    if f.is_none() {
        return Ok(());
    }
    let f = f.unwrap();

    f.write_fmt(format_args!("Usage: {} command [flags ...]\n", program))?;
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
    f: Option<&mut W>,
    program: &str,
    command: &Command,
    print_command_name: bool,
) -> Result<()> {
    if f.is_none() {
        return Ok(());
    }
    let f = f.unwrap();

    f.write_fmt(format_args!("Usage: {}", program))?;
    if print_command_name {
        f.write_fmt(format_args!("{} ", command.name))?;
    }
    f.write_fmt(format_args!("[flags ...]\n"))?;

    if command
        .flags
        .iter()
        .filter(|s| s.is_named())
        .next()
        .is_some()
    {
        f.write_fmt(format_args!("\nNamed flags:\n"))?;
        for spec in command.flags.iter().filter(|s| s.is_named()) {
            f.write_fmt(format_args!("\t--{}", spec.get_name()))?;
            if let Some(short_name) = spec.get_short_name() {
                f.write_fmt(format_args!(", -{}", short_name))?;
            }
            f.write_fmt(format_args!(" - {}", spec.get_help()))?;

            if spec.is_boolean() {
                f.write_fmt(format_args!(" [Boolean, default: false]"))?;
            } else if let Some(default_value) = spec.get_required_default_value() {
                f.write_fmt(format_args!(" [Default: {}]", default_value))?;
            }

            f.write_fmt(format_args!("\n"))?;
        }
    }

    if command
        .flags
        .iter()
        .filter(|s| s.is_positional())
        .next()
        .is_some()
    {
        f.write_fmt(format_args!("\nPositional arguments:"))?;
        for spec in command.flags.iter().filter(|s| s.is_positional()) {
            f.write_fmt(format_args!("\n\t{}", spec.get_name()))?;
            if spec.is_variadic() {
                f.write_fmt(format_args!(" [One or more]"))?;
            }
        }
        f.write_fmt(format_args!("\n"))?;
    }

    Ok(())
}
