use std::fmt::Error;
use std::fmt::Write;
use std::result::Result;

use super::command::Command;

pub fn print_program_help<'a, CI>(f: &mut Write,
                                  program: &str,
                                  mut commands: CI)
                                  -> Result<(), Error>
    where CI: Iterator<Item = &'a Command>
{
    try!(f.write_str(format!(
        "Usage: {} command [options ...] [arguments ...]\n", program).as_ref()));
    try!(f.write_str("Available commands:\n"));
    while let Some(command) = commands.next() {
        try!(f.write_str(format!("\t{} - {}\n", command.get_name(), command.get_help()).as_ref()));
    }
    Ok(())
}

pub fn print_command_help(f: &mut Write,
                          program: &str,
                          command: &Command,
                          print_command_name: bool)
                          -> Result<(), Error> {
    try!(f.write_str(format!("Usage: {} ", program).as_ref()));
    if print_command_name {
        try!(f.write_str(format!("{} ", command.get_name()).as_ref()));
    }
    try!(f.write_str("[options ...] [arguments ...]\n"));

    if !command.get_options().is_empty() {
        try!(f.write_str("\nOptions:\n"));
        for option in command.get_options() {
            try!(f.write_str(format!("\t--{}", option.name).as_ref()));
            if option.short_name.is_some() {
                try!(f.write_str(format!(", -{}", option.short_name.as_ref().unwrap()).as_ref()));
            }
            try!(f.write_str(format!(" - {}", option.help).as_ref()));

            if option.is_flag {
                try!(f.write_str(" [Flag, default: off]"));
            } else if option.default_value.is_some() {
                try!(f.write_str(format!(" [Default: {}]",
                                         option.default_value.as_ref().unwrap())
                    .as_ref()));
            }

            try!(f.write_str("\n"));
        }
    }

    if !command.get_arguments().is_empty() {
        try!(f.write_str("\nPositional arguments:"));
        for argument in command.get_arguments() {
            try!(f.write_str(format!("\n\t{} - {}", argument.name, argument.help).as_ref()));
            if argument.default_value.is_some() {
                try!(f.write_str(format!(" [Default: {}]",
                                         &argument.default_value.as_ref().unwrap()[..]
                                             .join(", "))
                    .as_ref()));
            }
        }
        if command.last_argument_is_variadic() {
            try!(f.write_str(" [One or more]"));
        }
        try!(f.write_str("\n"));
    }

    Ok(())
}