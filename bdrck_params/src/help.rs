use command::{Command, ExecutableCommand};
use error::Result;
use std::fmt::Write;

pub fn print_program_help<'cbl, E>(f: &mut Write,
                                   program: &str,
                                   commands: &Vec<ExecutableCommand<'cbl, E>>)
                                   -> Result<()> {
    write!(f,
           "Usage: {} command [options ...] [arguments ...]\n",
           program)?;
    write!(f, "Available commands:\n")?;
    for command in commands.iter() {
        write!(f, "\t{} - {}\n", command.command.name, command.command.help)?;
    }
    Ok(())
}

pub fn print_command_help(f: &mut Write,
                          program: &str,
                          command: &Command,
                          print_command_name: bool)
                          -> Result<()> {
    write!(f, "Usage: {} ", program)?;
    if print_command_name {
        write!(f, "{} ", command.name)?;
    }
    write!(f, "[options ...] [arguments ...]\n")?;

    if !command.options.is_empty() {
        write!(f, "\nOptions:\n")?;
        for option in &command.options {
            write!(f, "\t--{}", option.name)?;
            if let Some(short_name) = option.short_name.as_ref() {
                write!(f, ", -{}", short_name)?;
            }
            write!(f, " - {}", option.help)?;

            if option.is_flag {
                write!(f, " [Flag, default: off]")?;
            } else if let Some(default_value) = option.default_value.as_ref() {
                write!(f, " [Default: {}]", default_value)?;
            }

            write!(f, "\n")?;
        }
    }

    if !command.arguments.is_empty() {
        write!(f, "\nPositional arguments:")?;
        for argument in &command.arguments {
            write!(f, "\n\t{}", argument)?;
        }
        if command.last_argument_is_variadic {
            write!(f, " [One or more]")?;
        }
        write!(f, "\n")?;
    }

    Ok(())
}
