use ::command::Command;
use ::error::Result;
use std::fmt::Write;

pub fn print_program_help<'a, CI>(f: &mut Write, program: &str, mut commands: CI) -> Result<()>
    where CI: Iterator<Item = &'a Command>
{
    try!(f.write_str(format!(
        "Usage: {} command [options ...] [arguments ...]\n", program).as_ref()));
    try!(f.write_str("Available commands:\n"));
    while let Some(command) = commands.next() {
        try!(f.write_str(format!("\t{} - {}\n", command.name, command.help).as_ref()));
    }
    Ok(())
}

pub fn print_command_help(f: &mut Write,
                          program: &str,
                          command: &Command,
                          print_command_name: bool)
                          -> Result<()> {
    try!(f.write_str(format!("Usage: {} ", program).as_ref()));
    if print_command_name {
        try!(f.write_str(format!("{} ", command.name).as_ref()));
    }
    try!(f.write_str("[options ...] [arguments ...]\n"));

    if !command.options.is_empty() {
        try!(f.write_str("\nOptions:\n"));
        for option in &command.options {
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

    if !command.arguments.is_empty() {
        try!(f.write_str("\nPositional arguments:"));
        for argument in &command.arguments {
            try!(f.write_fmt(format_args!("\n\t{}", argument)));
        }
        if command.last_argument_is_variadic {
            try!(f.write_str(" [One or more]"));
        }
        try!(f.write_str("\n"));
    }

    Ok(())
}
