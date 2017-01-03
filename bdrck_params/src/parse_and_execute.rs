use ::command::ExecutableCommand;
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

fn execute_command<'cl, 'cbl>(parsed_parameters: &ParsedParameters<'cl>,
                              commands: &mut Vec<ExecutableCommand<'cl, 'cbl>>) {
    let executable_command =
        commands.iter_mut().skip_while(|ec| *ec != parsed_parameters.get_command()).next().unwrap();
    parsed_parameters.execute(executable_command);
}

fn parse_and_execute_impl(program: &str,
                          parameters: &Vec<String>,
                          commands: &mut Vec<ExecutableCommand>,
                          print_program_help: bool,
                          print_command_name: bool)
                          -> Result<()> {
    let mut parameters_iterator = parameters.iter().peekable();

    let cr = parse_command(&mut parameters_iterator,
                           &mut commands.iter().map(|ec| ec.get_command()));
    if cr.is_err() {
        if print_program_help {
            try!(help::print_program_help(&mut IoWriteAdapter::new_stderr(),
                                          program,
                                          &mut commands.iter().map(|ec| ec.get_command())));
        }
        return Err(cr.err().unwrap());
    }
    let command = cr.ok().unwrap();

    let ppr = ParsedParameters::new(command, &mut parameters_iterator);
    if ppr.is_err() {
        let mut stderr = IoWriteAdapter::new_stderr();
        help::print_command_help(&mut stderr, program, command, print_command_name).unwrap();
        return Err(ppr.err().unwrap());
    }
    let parsed_parameters = ppr.unwrap();

    execute_command(&parsed_parameters, commands);

    Ok(())
}

pub fn parse_and_execute_command(program: &str,
                                 parameters: &Vec<String>,
                                 commands: &mut Vec<ExecutableCommand>)
                                 -> Result<()> {
    //! This function parses the given program parameters, and calls the
    //! appropriate command callback. It prints out usage information if the
    //! parameters are invalid, and returns a reasonable exit code for the process.
    //!
    //! This is the function which should be used for typical multi-command
    //! programs.

    parse_and_execute_impl(program, parameters, commands, true, true)
}

pub fn parse_and_execute(program: &str,
                         parameters: Vec<String>,
                         command: ExecutableCommand)
                         -> Result<()> {
    //! This function parses the given program parameters and calls the given
    //! command's callback. It prints out usage information if the parameters are
    //! invalid, and returns a reasonable exit code for the process.
    //!
    //! This is the function which should be used for typical single-command
    //! programs.

    let mut commands = vec![command];
    let command_parameters: Vec<String> = vec![commands[0].get_command().get_name().clone()];
    let parameters: Vec<String> = command_parameters.into_iter().chain(parameters).collect();

    parse_and_execute_impl(program, &parameters, &mut commands, false, false)
}
