use std::boxed::Box;
use std::fmt;
use std::io;
use std::string::String;
use std::vec::Vec;

use super::command::Command;
use super::help;
use super::parsed_parameters::ParsedParameters;
use super::parsed_parameters::parse_command;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

/// This structure can be constructed from an io::Write, and it implements
/// fmt::Write. It is
/// a simple adapter for using the former as if it were the latter.
struct IoWriteAdapter {
    io_writer: Box<io::Write>,
}

impl IoWriteAdapter {
    pub fn new_stderr() -> IoWriteAdapter { IoWriteAdapter { io_writer: Box::new(io::stderr()) } }
}

impl fmt::Write for IoWriteAdapter {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        let mut buf = String::new();
        try!(buf.write_str(s));
        self.io_writer.write_all(&buf.into_bytes()[..]).unwrap();
        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result<(), fmt::Error> {
        let mut buf = String::new();
        try!(buf.write_char(c));
        self.io_writer.write_all(&buf.into_bytes()[..]).unwrap();
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments) -> Result<(), fmt::Error> {
        let mut buf = String::new();
        try!(buf.write_fmt(args));
        self.io_writer.write_all(&buf.into_bytes()[..]).unwrap();
        Ok(())
    }
}

fn parse_and_execute_impl<'a>(program: &'a str,
                              parameters: &'a Vec<String>,
                              commands: &'a Vec<Command>,
                              print_program_help: bool,
                              print_command_name: bool)
                              -> i32 {
    use std::fmt::Write;

    let cr = parse_command(&mut parameters.iter(), &mut commands.iter());
    if cr.is_err() {
        if print_program_help {
            help::print_program_help(&mut IoWriteAdapter::new_stderr(),
                                     program,
                                     &mut commands.iter())
                .unwrap();
        }
        return EXIT_FAILURE;
    }
    let command = cr.ok().unwrap();

    let ppr = ParsedParameters::new(command, &mut parameters.iter());
    if ppr.is_err() {
        let mut stderr = IoWriteAdapter::new_stderr();
        stderr.write_str(format!("ERROR: {}\n", ppr.err().unwrap()).as_ref()).unwrap();
        help::print_command_help(&mut stderr, program, command, print_command_name).unwrap();
        return EXIT_FAILURE;
    }
    ppr.unwrap().execute();

    EXIT_SUCCESS
}

pub fn parse_and_execute_command<'a>(program: &'a str,
                                     parameters: &'a Vec<String>,
                                     commands: &'a Vec<Command>)
                                     -> i32 {
    //! This function parses the given program parameters, and calls the
    //! appropriate command callback. It prints out usage information if the
    //! parameters are invalid, and returns a reasonable exit code for the process.
    //!
    //! This is the function which should be used for typical multi-command
    //! programs.

    parse_and_execute_impl(program, parameters, commands, true, true)
}

pub fn parse_and_execute<'a>(program: &'a str, parameters: Vec<String>, command: Command) -> i32 {
    //! This function parses the given program parameters and calls the given
    //! command's callback. It prints out usage information if the parameters are
    //! invalid, and returns a reasonable exit code for the process.
    //!
    //! This is the function which should be used for typical single-command
    //! programs.

    let commands: Vec<Command> = vec![command];
    let command_parameters: Vec<String> = vec![commands[0].get_name().clone()];
    let parameters: Vec<String> = command_parameters.into_iter().chain(parameters).collect();

    parse_and_execute_impl(program, &parameters, &commands, false, false)
}
