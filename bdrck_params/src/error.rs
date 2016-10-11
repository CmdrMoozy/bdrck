use std::error::Error;
use std::fmt;
use std::result::Result;
use std::string::String;

#[derive(Debug, Eq, PartialEq)]
pub enum ErrorKind {
    MissingDefaultArgumentValue,
    TooManyDefaultArgumentValues,
    NoCommandSpecified,
    UnrecognizedCommand { name: String },
    NotAnOption,
    UnrecognizedOption { name: String },
    MissingOptionValue { name: String },
    InvalidBooleanValue { value: String },
    MissingArgumentValue { name: String },
    WrongNumberOfArgumentValues { count: usize },
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParamsError {
    pub kind: ErrorKind,
}

impl Error for ParamsError {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::MissingDefaultArgumentValue => {
                "Missing default argument value: all arguments after the first one with a default \
                 value must also have default values."
            },
            ErrorKind::TooManyDefaultArgumentValues => {
                "Too many default argument values: arguments must have at most one default value, \
                 except the last argument which may have more than one only if it is variadic."
            },
            ErrorKind::NoCommandSpecified => "No command specified",
            ErrorKind::UnrecognizedCommand { name: _ } => "Unrecognized command",
            ErrorKind::NotAnOption => "Parameter is not an option",
            ErrorKind::UnrecognizedOption { name: _ } => "Unrecognized option",
            ErrorKind::MissingOptionValue { name: _ } => "No default or specified value for option",
            ErrorKind::InvalidBooleanValue { value: _ } => "Invalid boolean value",
            ErrorKind::MissingArgumentValue { name: _ } => "Missing argument value",
            ErrorKind::WrongNumberOfArgumentValues { count: _ } => {
                "Wrong number of argument values; expected 1"
            },
        }
    }
}

impl fmt::Display for ParamsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::UnrecognizedCommand { name: ref n } => {
                f.write_str(format!("{} '{}'", self.description(), n).as_str())
            },
            ErrorKind::UnrecognizedOption { name: ref n } => {
                f.write_str(format!("{} '{}'", self.description(), n).as_str())
            },
            ErrorKind::MissingOptionValue { name: ref n } => {
                f.write_str(format!("{} '--{}'", self.description(), n).as_str())
            },
            ErrorKind::InvalidBooleanValue { value: ref v } => {
                f.write_str(format!("{} '{}'", self.description(), v).as_str())
            },
            ErrorKind::MissingArgumentValue { name: ref n } => {
                f.write_str(format!("{} '{}'", self.description(), n).as_str())
            },
            ErrorKind::WrongNumberOfArgumentValues { count: ref c } => {
                f.write_str(format!("{} got {}", self.description(), c).as_str())
            },
            _ => f.write_str(self.description()),
        }
    }
}

pub type ParamsResult<T> = Result<T, ParamsError>;
