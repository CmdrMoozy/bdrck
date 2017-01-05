use std::fmt;
use std::io;
use std::result;

/// This structure can be constructed from an io::Write, and it implements
/// fmt::Write. It is a simple adapter for using the former as if it were the
/// latter.
pub struct IoWriteAdapter {
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
