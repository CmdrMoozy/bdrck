use backtrace::Backtrace;
use msgpack;
use std::cmp::{Eq, PartialEq};
use std::convert::From;
use std::error;
use std::fmt;
use std::io;
use std::result;
use std::string::String;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Encoding { cause: String },
    Io { cause: String },
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub backtrace: Backtrace,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error {
            kind: kind,
            backtrace: Backtrace::new(),
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool { self.kind == other.kind }
}

impl Eq for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error { Error::new(ErrorKind::Io { cause: e.to_string() }) }
}

impl From<msgpack::encode::Error> for Error {
    fn from(e: msgpack::encode::Error) -> Error {
        Error::new(ErrorKind::Encoding { cause: e.to_string() })
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::Encoding { cause: _ } => "Encoding error",
            ErrorKind::Io { cause: _ } => "Input/output error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        match self.kind {
            ErrorKind::Encoding { cause: ref c } => {
                f.write_str(format!("{}: {}", self.description(), c).as_str())
            },
            ErrorKind::Io { cause: ref c } => {
                f.write_str(format!("{}: {}", self.description(), c).as_str())
            },
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
