use std::error::Error;
use std::fmt;
use std::string::String;

#[derive(Debug)]
pub enum ErrorKind {
    NoCommandSpecified,
    UnrecognizedCommand {
        name: String,
    },
    MissingOptionValue {
        name: String,
    },
}

#[derive(Debug)]
pub struct ParamsError {
    pub kind: ErrorKind,
}

impl Error for ParamsError {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::NoCommandSpecified => "No command specified",
            ErrorKind::UnrecognizedCommand { name: ref _n } => "Unrecognized command",
            ErrorKind::MissingOptionValue { name: ref _n } => {
                "No default or specified value for option"
            }
        }
    }
}

impl fmt::Display for ParamsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::NoCommandSpecified => f.write_str(self.description()),
            ErrorKind::UnrecognizedCommand { name: ref n } => {
                f.write_str(format!("Unrecognized command '{}'", n).as_str())
            }
            ErrorKind::MissingOptionValue { name: ref n } => {
                f.write_str(format!("No default or specified value for option '--{}'", n).as_str())
            }
        }
    }
}

pub mod argument;
pub mod command;
pub mod option;
pub mod parsed_parameters;
