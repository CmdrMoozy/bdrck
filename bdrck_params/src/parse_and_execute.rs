use ::command::{CommandResult, ExecutableCommand};
use ::error::Result;
use ::help;
use ::parsed_parameters::ParsedParameters;
use ::parsed_parameters::parse_command;
use std::boxed::Box;
use std::fmt;
use std::io;
use std::result;
use std::string::String;
use std::vec::Vec;

/// This structure can be constructed from an io::Write, and it implements
/// fmt::Write. It is a simple adapter for using the former as if it were the
/// latter.
struct IoWriteAdapter {
    io_writer: Box<io::Write>,
}

impl IoWriteAdapter {
    pub fn new_stderr() -> IoWriteAdapter { IoWriteAdapter { io_writer: Box::new(io::stderr()) } }
}

impl fmt::Write for IoWriteAdapter {
    fn write_str(&mut self, s: &str) -> result::Result<(), fmt::Error> {
        let mut buf = String::new();
        try!(buf.write_str(s));
        self.io_writer.write_all(&buf.into_bytes()[..]).unwrap();
        Ok(())
    }
}

fn execute_command<'cl, 'cbl, E>(parsed_parameters: ParsedParameters,
                                 commands: Vec<ExecutableCommand<'cl, 'cbl, E>>)
                                 -> CommandResult<E> {
    parsed_parameters.execute(commands.into_iter())
}

fn parse_and_execute_impl<E>(program: &str,
                             parameters: &Vec<String>,
                             commands: Vec<ExecutableCommand<E>>,
                             print_program_help: bool,
                             print_command_name: bool)
                             -> Result<CommandResult<E>> {
    let mut parameters_iterator = parameters.iter().peekable();

    let command = match parse_command(&mut parameters_iterator,
                                      &mut commands.iter().map(|ec| ec.command)) {
        Ok(c) => c,
        Err(e) => {
            if print_program_help {
                try!(help::print_program_help(&mut IoWriteAdapter::new_stderr(),
                                              program,
                                              &mut commands.iter().map(|ec| ec.command)));
            }
            return Err(e);
        },
    };

    let parsed_parameters = match ParsedParameters::new(command, &mut parameters_iterator) {
        Ok(p) => p,
        Err(e) => {
            try!(help::print_command_help(&mut IoWriteAdapter::new_stderr(),
                                          program,
                                          command,
                                          print_command_name));
            return Err(e);
        },
    };

    Ok(execute_command(parsed_parameters, commands))
}

pub fn parse_and_execute_command<E>(program: &str,
                                    parameters: &Vec<String>,
                                    commands: Vec<ExecutableCommand<E>>)
                                    -> Result<CommandResult<E>> {
    //! This function parses the given program parameters, and calls the
    //! appropriate command callback. It prints out usage information if the
    //! parameters are invalid, and returns a reasonable exit code for the process.
    //!
    //! This is the function which should be used for typical multi-command
    //! programs.

    parse_and_execute_impl(program, parameters, commands, true, true)
}

pub fn parse_and_execute<E>(program: &str,
                            parameters: &Vec<String>,
                            command: ExecutableCommand<E>)
                            -> Result<CommandResult<E>> {
    //! This function parses the given program parameters and calls the given
    //! command's callback. It prints out usage information if the parameters are
    //! invalid, and returns a reasonable exit code for the process.
    //!
    //! This is the function which should be used for typical single-command
    //! programs.

    parse_and_execute_impl(program,
                           &Some(command.command.name.clone())
                               .into_iter()
                               .chain(parameters.iter().cloned())
                               .collect(),
                           vec![command],
                           false,
                           false)
}
