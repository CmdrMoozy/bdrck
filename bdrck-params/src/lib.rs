use std::error::Error;
use std::fmt;
use std::string::String;

#[derive(Debug)]
pub enum ErrorKind {
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
            ErrorKind::MissingOptionValue { name: ref _n } => {
                "No default or specified value for option"
            }
        }
    }
}

impl fmt::Display for ParamsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::MissingOptionValue { name: ref n } => {
                f.write_str(format!("No default or specified value for option '--{}'", n).as_str())
            }
        }
    }
}

pub mod argument;
pub mod command;
pub mod option;

#[cfg(test)]
mod option_test;

mod detail;

#[cfg(test)]
mod detail_test;
