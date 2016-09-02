use std::boxed::Box;
use std::fmt;
use std::io;
use std::string::String;

use super::command::Command;
use super::help;
use super::parsed_parameters::ParsedParameters;

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

fn parse_and_execute_impl<'a, PI, CI>(program: &'a str,
                                      parameters: &mut PI,
                                      commands: &mut CI,
                                      _: bool,
                                      _: bool)
                                      -> i32
    where PI: Iterator<Item = &'a String>,
          CI: Iterator<Item = &'a Command>
{
    let ppr = ParsedParameters::new(parameters, commands);
    if ppr.is_err() {
        help::print_program_help(&mut IoWriteAdapter::new_stderr(), program, commands).unwrap();
        return EXIT_FAILURE;
    }
    let parsed = ppr.unwrap();
    parsed.execute();

    EXIT_SUCCESS
}

pub fn parse_and_execute_command<'a, PI, CI>(program: &'a str,
                                             parameters: &mut PI,
                                             commands: &mut CI)
                                             -> i32
    where PI: Iterator<Item = &'a String>,
          CI: Iterator<Item = &'a Command>
{
    //! This function parses the given program parameters, and calls the
    //! appropriate command callback. It prints out usage information if the
    //! parameters are invalid, and returns a reasonable exit code for the process.
    //!
    //! This is the function which should be used for typical multi-command
    //! programs.

    parse_and_execute_impl(program, parameters, commands, true, true)
}
