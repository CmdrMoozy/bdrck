use std::fmt;
use std::io;
use std::result;
use std::sync::Mutex;

/// The various fmt::Write implementations bdrck_params can use to e.g. print
/// out program help information. By default, bdrck_params uses Info. This can
/// be changed globally via set_writer_impl.
#[derive(Clone)]
pub enum WriterImpl {
    /// Print information to stdout.
    Stdout,
    /// Print information to stderr.
    Stderr,
    /// Log information using info!().
    Info,
    /// Silently discard any output information.
    Noop,
}

fn write_to_io_writer(writer: &mut io::Write, s: &str) -> result::Result<(), fmt::Error> {
    use std::fmt::Write;
    let mut buf = String::new();
    try!(buf.write_str(s));
    writer.write_all(&buf.into_bytes()[..]).unwrap();
    Ok(())
}

lazy_static! {
    static ref WRITER_IMPL: Mutex<WriterImpl> = Mutex::new(WriterImpl::Info);
}

/// Change the writer implementation used by all bdrck_params functions.
pub fn set_writer_impl(i: WriterImpl) {
    let mut guard = match WRITER_IMPL.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = i;
}

/// This structure implements fmt::Write using one of various implementations
/// described by WriterImpl. This structure should generally not be
/// instantiated directly. Instead, callers should use get_writer_impl().
pub struct Writer {
    writer_impl: WriterImpl,
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> result::Result<(), fmt::Error> {
        match self.writer_impl {
            WriterImpl::Stdout => write_to_io_writer(&mut io::stdout(), s),
            WriterImpl::Stderr => write_to_io_writer(&mut io::stderr(), s),
            WriterImpl::Info => {
                info!("{}", s);
                Ok(())
            },
            WriterImpl::Noop => Ok(()),
        }
    }
}

/// Return a structure which uses the current global WriterImpl to implement
/// fmt::Write.
pub fn get_writer_impl() -> Writer {
    let guard = match WRITER_IMPL.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    Writer { writer_impl: (*guard).clone() }
}
