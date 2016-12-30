use backtrace::Backtrace;
use msgpack;
use std::cmp::{Eq, PartialEq};
use std::convert::From;
use std::env;
use std::error;
use std::fmt;
use std::io;
use std::result;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Decode { cause: String },
    Encode { cause: String },
    Environment { cause: String },
    IdentifierTypeMismatch,
    Io { cause: String },
    UnrecognizedIdentifier,
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

impl From<env::VarError> for Error {
    fn from(e: env::VarError) -> Error {
        Error::new(ErrorKind::Environment { cause: e.to_string() })
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error { Error::new(ErrorKind::Io { cause: e.to_string() }) }
}

impl From<msgpack::decode::Error> for Error {
    fn from(e: msgpack::decode::Error) -> Error {
        Error::new(ErrorKind::Decode { cause: e.to_string() })
    }
}

impl From<msgpack::encode::Error> for Error {
    fn from(e: msgpack::encode::Error) -> Error {
        Error::new(ErrorKind::Encode { cause: e.to_string() })
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::Decode { cause: _ } => "Decoding error",
            ErrorKind::Encode { cause: _ } => "Encoding error",
            ErrorKind::Environment { cause: _ } => "Environment error",
            ErrorKind::IdentifierTypeMismatch => "Identifier / type mismatch",
            ErrorKind::Io { cause: _ } => "Input/output error",
            ErrorKind::UnrecognizedIdentifier => "Unrecognized configuration identifier",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        match self.kind {
            ErrorKind::Decode { cause: ref c } => {
                f.write_str(format!("{}: {}", self.description(), c).as_str())
            },
            ErrorKind::Encode { cause: ref c } => {
                f.write_str(format!("{}: {}", self.description(), c).as_str())
            },
            ErrorKind::Environment { cause: ref c } => {
                f.write_str(format!("{}: {}", self.description(), c).as_str())
            },
            ErrorKind::Io { cause: ref c } => {
                f.write_str(format!("{}: {}", self.description(), c).as_str())
            },
            _ => f.write_str(self.description()),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
