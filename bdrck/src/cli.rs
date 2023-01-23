// Copyright 2015 Axel Rasmussen
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::error::*;
use errno;
use libc::{self, c_int};
use std::fmt;
use std::io::{self, Read, Write};
use std::mem::MaybeUninit;
use tracing::debug;

/// An alias for std::io::Result.
pub type IoResult<T> = io::Result<T>;

fn to_io_result(ret: c_int) -> IoResult<()> {
    match ret {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}

/// This enum describes high-level terminal flags, in an OS-agnostic way.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TerminalFlag {
    /// A flag indicating that typed characters should be echoed.
    Echo,
    /// A flag indicating that newlines, specifically, should be echoed.
    EchoNewlines,
}

impl TerminalFlag {
    fn to_value(&self) -> libc::tcflag_t {
        match *self {
            TerminalFlag::Echo => libc::ECHO,
            TerminalFlag::EchoNewlines => libc::ECHONL,
        }
    }
}

/// This trait describes an abstract type which describes the attributes of a
/// terminal.
///
/// This trait primarily exists for testing purposes. In almost all cases, users
/// will instead just use the concrete type `Stream` defined below.
pub trait AbstractTerminalAttributes {
    /// Enable a flag in this set of attributes.
    fn enable(&mut self, flag: TerminalFlag);

    /// Disable a flag in this set of attributes.
    fn disable(&mut self, flag: TerminalFlag);
}

/// This is an opaque structure which encapsulates the state / attributes of an
/// interactive terminal. The contents of this structure are OS-specific.
pub struct TerminalAttributes {
    inner: libc::termios,
}

impl TerminalAttributes {
    fn new(fd: c_int) -> IoResult<Self> {
        let mut attrs = MaybeUninit::uninit();
        to_io_result(unsafe { libc::tcgetattr(fd, attrs.as_mut_ptr()) })?;
        Ok(TerminalAttributes {
            inner: unsafe { attrs.assume_init() },
        })
    }

    /// Create a new TerminalAttributes, with an "empty" state (no flags
    /// enabled).
    pub fn new_empty() -> Self {
        TerminalAttributes {
            inner: unsafe { MaybeUninit::zeroed().assume_init() },
        }
    }

    fn apply(&self, fd: c_int) -> IoResult<()> {
        to_io_result(unsafe { libc::tcsetattr(fd, libc::TCSANOW, &self.inner) })
    }

    /// Test whether or not the given `TerminalFlag` is currently enabled.
    pub fn is_enabled(&self, flag: TerminalFlag) -> bool {
        self.inner.c_lflag & flag.to_value() != 0
    }
}

impl PartialEq for TerminalAttributes {
    fn eq(&self, other: &Self) -> bool {
        self.inner.c_iflag == other.inner.c_iflag
            && self.inner.c_oflag == other.inner.c_oflag
            && self.inner.c_cflag == other.inner.c_cflag
            && self.inner.c_lflag == other.inner.c_lflag
            && self.inner.c_line == other.inner.c_line
            && self.inner.c_cc == other.inner.c_cc
            && self.inner.c_ispeed == other.inner.c_ispeed
            && self.inner.c_ospeed == other.inner.c_ospeed
    }
}

impl Eq for TerminalAttributes {}

fn debug_format_flag_field(
    v: libc::tcflag_t,
    fs: &'static [(&'static str, libc::tcflag_t)],
) -> std::result::Result<String, fmt::Error> {
    use fmt::Write;

    let mut remaining_v: libc::tcflag_t = v;
    let mut s = String::new();
    for &(fname, fvalue) in fs {
        if (v & fvalue) != 0 {
            let was_empty = s.is_empty();
            write!(
                &mut s,
                "{}{}",
                match was_empty {
                    true => "",
                    false => " | ",
                },
                fname
            )?;
            remaining_v &= !v;
        }
    }
    if remaining_v != 0 {
        let was_empty = s.is_empty();
        write!(
            &mut s,
            "{}(extra: {:x})",
            match was_empty {
                true => "",
                false => " ",
            },
            remaining_v
        )?;
    }
    Ok(s)
}

fn debug_format_c_cc_field(c_cc: &[libc::cc_t; 32]) -> std::result::Result<String, fmt::Error> {
    use fmt::Write;

    const INDICES: &'static [(&'static str, usize)] = &[
        ("VDISCARD", libc::VDISCARD),
        ("VEOF", libc::VEOF),
        ("VEOL", libc::VEOL),
        ("VEOL2", libc::VEOL2),
        ("VERASE", libc::VERASE),
        ("VINTR", libc::VINTR),
        ("VKILL", libc::VKILL),
        ("VLNEXT", libc::VLNEXT),
        ("VMIN", libc::VMIN),
        ("VQUIT", libc::VQUIT),
        ("VREPRINT", libc::VREPRINT),
        ("VSTART", libc::VSTART),
        ("VSTOP", libc::VSTOP),
        ("VSUSP", libc::VSUSP),
        ("VSWTC", libc::VSWTC),
        ("VTIME", libc::VTIME),
        ("VWERASE", libc::VWERASE),
    ];

    let mut s = String::new();
    for &(name, idx) in INDICES {
        let was_empty = s.is_empty();
        write!(
            &mut s,
            "{}{}:{}",
            match was_empty {
                true => "",
                false => ", ",
            },
            name,
            c_cc[idx]
        )?;
    }
    Ok(s)
}

impl fmt::Debug for TerminalAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TerminalAttributes")
            .field(
                "c_iflag",
                &debug_format_flag_field(
                    self.inner.c_iflag,
                    &[
                        ("IGNBRK", libc::IGNBRK),
                        ("BRKINT", libc::BRKINT),
                        ("IGNPAR", libc::IGNPAR),
                        ("PARMRK", libc::PARMRK),
                        ("INPCK", libc::INPCK),
                        ("ISTRIP", libc::ISTRIP),
                        ("INLCR", libc::INLCR),
                        ("IGNCR", libc::IGNCR),
                        ("ICRNL", libc::ICRNL),
                        ("IXON", libc::IXON),
                        ("IXANY", libc::IXANY),
                        ("IXOFF", libc::IXOFF),
                        ("IMAXBEL", libc::IMAXBEL),
                        ("IUTF8", libc::IUTF8),
                    ],
                )?,
            )
            .field(
                "c_oflag",
                &debug_format_flag_field(
                    self.inner.c_oflag,
                    &[
                        ("OPOST", libc::OPOST),
                        ("OLCUC", libc::OLCUC),
                        ("ONLCR", libc::ONLCR),
                        ("ONOCR", libc::ONOCR),
                        ("ONLRET", libc::ONLRET),
                        ("OFILL", libc::OFILL),
                        ("OFDEL", libc::OFDEL),
                        ("NLDLY", libc::NLDLY),
                        ("CRDLY", libc::CRDLY),
                        ("TABDLY", libc::TABDLY),
                        ("BSDLY", libc::BSDLY),
                        ("VTDLY", libc::VTDLY),
                        ("FFDLY", libc::FFDLY),
                    ],
                )?,
            )
            .field(
                "c_cflag",
                &debug_format_flag_field(
                    self.inner.c_cflag,
                    &[
                        ("CBAUD", libc::CBAUD),
                        ("CBAUDEX", libc::CBAUDEX),
                        ("CSIZE", libc::CSIZE),
                        ("CSTOPB", libc::CSTOPB),
                        ("CREAD", libc::CREAD),
                        ("PARENB", libc::PARENB),
                        ("PARODD", libc::PARODD),
                        ("HUPCL", libc::HUPCL),
                        ("CLOCAL", libc::CLOCAL),
                        ("CIBAUD", libc::CIBAUD),
                        ("CMSPAR", libc::CMSPAR),
                        ("CRTSCTS", libc::CRTSCTS),
                    ],
                )?,
            )
            .field(
                "c_lflag",
                &debug_format_flag_field(
                    self.inner.c_lflag,
                    &[
                        ("ISIG", libc::ISIG),
                        ("ICANON", libc::ICANON),
                        ("ECHO", libc::ECHO),
                        ("ECHOE", libc::ECHOE),
                        ("ECHOK", libc::ECHOK),
                        ("ECHONL", libc::ECHONL),
                        ("ECHOCTL", libc::ECHOCTL),
                        ("ECHOPRT", libc::ECHOPRT),
                        ("ECHOKE", libc::ECHOKE),
                        ("FLUSHO", libc::FLUSHO),
                        ("NOFLSH", libc::NOFLSH),
                        ("TOSTOP", libc::TOSTOP),
                        ("PENDIN", libc::PENDIN),
                        ("IEXTEN", libc::IEXTEN),
                    ],
                )?,
            )
            .field("c_cc", &debug_format_c_cc_field(&self.inner.c_cc)?)
            .field("c_ispeed", unsafe { &libc::cfgetispeed(&self.inner) })
            .field("c_ospeed", unsafe { &libc::cfgetospeed(&self.inner) })
            .finish()
    }
}

impl AbstractTerminalAttributes for TerminalAttributes {
    fn enable(&mut self, flag: TerminalFlag) {
        self.inner.c_lflag |= flag.to_value();
    }

    fn disable(&mut self, flag: TerminalFlag) {
        self.inner.c_lflag &= !flag.to_value();
    }
}

/// This trait describes an abstract input or output stream.
///
/// This trait primarily exists for testing purposes. In almost all cases, users
/// will instead just use the concrete type `Stream` defined below.
pub trait AbstractStream {
    /// A type which describes the attributes of this stream / terminal.
    type Attributes: AbstractTerminalAttributes + fmt::Debug;

    /// Returns whether or not this stream refers to an interactive terminal (a
    /// TTY), as opposed to, for example, a pipe.
    fn isatty(&self) -> bool;

    /// Retrieve the current attributes of this stream / terminal.
    fn get_attributes(&self) -> IoResult<Self::Attributes>;

    /// Modify this stream's / terminal's attributes to match the given state.
    fn set_attributes(&mut self, attributes: &Self::Attributes) -> IoResult<()>;

    /// Return a `Read` for this stream, if reading is supported.
    fn as_reader(&self) -> Option<Box<dyn Read>>;

    /// Return a `Write` for this stream, if writing is supported.
    fn as_writer(&self) -> Option<Box<dyn Write>>;
}

/// Standard input / output streams.
#[derive(Debug)]
pub enum Stream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
    /// Standard input.
    Stdin,
}

impl Stream {
    fn to_fd(&self) -> c_int {
        match *self {
            Stream::Stdout => libc::STDOUT_FILENO,
            Stream::Stderr => libc::STDERR_FILENO,
            Stream::Stdin => libc::STDIN_FILENO,
        }
    }
}

impl AbstractStream for Stream {
    type Attributes = TerminalAttributes;

    fn isatty(&self) -> bool {
        let ret = unsafe { libc::isatty(self.to_fd()) };
        let error: i32 = errno::errno().into();
        match ret {
            1 => true,
            0 => match error {
                libc::EBADF => false,
                libc::ENOTTY => false,
                _ => {
                    debug!(
                        "Unrecognized isatty errno: {}; assuming {:?} is not a TTY",
                        error, *self
                    );
                    false
                }
            },
            _ => {
                debug!(
                    "Unrecognized isatty return code: {}; assuming {:?} is not a TTY",
                    ret, *self
                );
                false
            }
        }
    }

    fn get_attributes(&self) -> IoResult<Self::Attributes> {
        TerminalAttributes::new(self.to_fd())
    }

    fn set_attributes(&mut self, attributes: &Self::Attributes) -> IoResult<()> {
        let ret = attributes.apply(self.to_fd());
        debug_assert!(ret.is_err() || *attributes == Self::Attributes::new(self.to_fd()).unwrap());
        ret
    }

    fn as_reader(&self) -> Option<Box<dyn Read>> {
        match *self {
            Stream::Stdin => Some(Box::new(io::stdin())),
            _ => None,
        }
    }

    fn as_writer(&self) -> Option<Box<dyn Write>> {
        match *self {
            Stream::Stdout => Some(Box::new(io::stdout())),
            Stream::Stderr => Some(Box::new(io::stderr())),
            _ => None,
        }
    }
}

/// This structure handles a) disabling the echoing of characters typed to
/// `Stdin`, and b) remembering to reset the terminal attributes afterwards
/// (via `Drop`).
struct DisableEcho<'s, S: AbstractStream> {
    stream: &'s mut S,
    initial_attributes: S::Attributes,
}

impl<'s, S: AbstractStream> DisableEcho<'s, S> {
    fn new(stream: &'s mut S) -> Result<Self> {
        let initial_attributes = stream.get_attributes()?;
        debug!("Initial stream attributes: {:#?}", initial_attributes);

        let mut attributes = stream.get_attributes()?;
        // Don't echo characters typed to stdin.
        attributes.disable(TerminalFlag::Echo);
        // But, *do* echo the newline when the user hits ENTER.
        attributes.enable(TerminalFlag::EchoNewlines);
        debug!("Setting attributes to: {:#?}", attributes);
        stream.set_attributes(&attributes)?;

        Ok(DisableEcho {
            stream: stream,
            initial_attributes: initial_attributes,
        })
    }
}

impl<'s, S: AbstractStream> Drop for DisableEcho<'s, S> {
    fn drop(&mut self) {
        self.stream
            .set_attributes(&self.initial_attributes)
            .unwrap();
    }
}

fn require_isatty<S: AbstractStream>(s: &mut S) -> Result<()> {
    if !s.isatty() {
        Err(Error::Precondition(format!(
            "cannot prompt interactively when the I/O streams are not TTYs"
        )))
    } else {
        Ok(())
    }
}

fn build_input_reader<IS: AbstractStream>(
    input_stream: &mut IS,
) -> Result<io::BufReader<Box<dyn Read>>> {
    require_isatty(input_stream)?;
    Ok(io::BufReader::new(match input_stream.as_reader() {
        None => {
            return Err(Error::Precondition(format!(
                "the given input stream must support `Read`"
            )))
        }
        Some(r) => r,
    }))
}

fn remove_newline(mut s: String) -> Result<String> {
    // Remove the trailing newline (if any - not finding one is an error).
    if !s.ends_with('\n') {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "unexpected end of input").into());
    }
    s.pop();

    // If this is windows and so there's also a \r, remove that too.
    if s.ends_with('\r') {
        s.pop();
    }

    Ok(s)
}

fn prompt_for_string_impl<IS: AbstractStream, OS: AbstractStream>(
    input_stream: &mut IS,
    // We have to take the reader as a parameter, since it must be "global",
    // even if this function is e.g. called in a loop. Otherwise, because it's
    // buffered, we might buffer some input and then discard it.
    input_reader: &mut io::BufReader<Box<dyn Read>>,
    output_stream: &mut OS,
    prompt: &str,
    is_sensitive: bool,
) -> Result<String> {
    use io::BufRead;

    require_isatty(output_stream)?;
    // It's fine to construct a separate writer, potentially on each loop
    // iteration or whatever, because we flush immediately, and don't do any
    // buffering.
    let mut writer = match output_stream.as_writer() {
        None => {
            return Err(Error::Precondition(format!(
                "the given output stream must support `Write`"
            )))
        }
        Some(w) => w,
    };

    write!(writer, "{}", prompt)?;
    // We have to flush so the user sees the prompt immediately.
    writer.flush()?;

    Ok({
        let _disable_echo = match is_sensitive {
            false => None,
            true => Some(DisableEcho::new(input_stream)?),
        };
        let mut ret = String::new();
        input_reader.read_line(&mut ret)?;
        remove_newline(ret)?
    })
}

/// Prompt the user for a string (read from the given input stream) using the
/// given output stream (typically standard output or standard error) to display
/// the given prompt message.
///
/// If `is_sensitive` is true, then the users characters will not be echoed back
/// (e.g. this will behave like a password prompt).
///
/// Note that there are various requirements for the given streams, and this
/// function will return an error if any of them are not met:
///
/// - Both `input_stream` and `output_stream` must be TTYs.
/// - `input_stream` must return a valid `Read` instance.
/// - `output_stream` must return a valid `Write` instance.
pub fn prompt_for_string<IS: AbstractStream, OS: AbstractStream>(
    mut input_stream: IS,
    mut output_stream: OS,
    prompt: &str,
    is_sensitive: bool,
) -> Result<String> {
    let mut input_reader = build_input_reader(&mut input_stream)?;
    prompt_for_string_impl(
        &mut input_stream,
        &mut input_reader,
        &mut output_stream,
        prompt,
        is_sensitive,
    )
}

fn prompt_for_string_confirm_impl<IS: AbstractStream, OS: AbstractStream>(
    input_stream: &mut IS,
    input_reader: &mut io::BufReader<Box<dyn Read>>,
    output_stream: &mut OS,
    prompt: &str,
    is_sensitive: bool,
) -> Result<String> {
    loop {
        let string = prompt_for_string_impl(
            input_stream,
            input_reader,
            output_stream,
            prompt,
            is_sensitive,
        )?;
        if string
            == prompt_for_string_impl(
                input_stream,
                input_reader,
                output_stream,
                "Confirm: ",
                is_sensitive,
            )?
        {
            return Ok(string);
        }
    }
}

/// Prompt for a string as per `prompt_for_string`, but additionally have the
/// user enter the value again to confirm we get the same answer twice. This is
/// useful for e.g. password entry.
pub fn prompt_for_string_confirm<IS: AbstractStream, OS: AbstractStream>(
    mut input_stream: IS,
    mut output_stream: OS,
    prompt: &str,
    is_sensitive: bool,
) -> Result<String> {
    let mut input_reader = build_input_reader(&mut input_stream)?;
    prompt_for_string_confirm_impl(
        &mut input_stream,
        &mut input_reader,
        &mut output_stream,
        prompt,
        is_sensitive,
    )
}

/// MaybePromptedString is a wrapper for getting user input interactively, while
/// also allowing the value to be specified at call time. This is useful e.g.
/// when we want to prompt users interactively, but want to predefine the values
/// in unit tests, or when users can specify a value either interactively or via
/// flags.
pub struct MaybePromptedString {
    value: String,
    was_provided: bool,
}

impl MaybePromptedString {
    /// Construct a new MaybePromptedString, either using the given value or
    /// prompting the user interactively with the given options.
    pub fn new<IS: AbstractStream, OS: AbstractStream>(
        provided: Option<&str>,
        mut input_stream: IS,
        mut output_stream: OS,
        prompt: &str,
        is_sensitive: bool,
        confirm: bool,
    ) -> Result<Self> {
        let mut input_reader = build_input_reader(&mut input_stream)?;
        let prompted: Option<String> = match provided {
            None => Some(match confirm {
                false => prompt_for_string_impl(
                    &mut input_stream,
                    &mut input_reader,
                    &mut output_stream,
                    prompt,
                    is_sensitive,
                )?,
                true => prompt_for_string_confirm_impl(
                    &mut input_stream,
                    &mut input_reader,
                    &mut output_stream,
                    prompt,
                    is_sensitive,
                )?,
            }),
            Some(_) => None,
        };

        let was_provided = provided.is_some();
        let value = provided.map_or_else(|| prompted.unwrap(), |s| s.to_owned());

        Ok(MaybePromptedString {
            value: value,
            was_provided: was_provided,
        })
    }

    /// Returns true if this string was provided, or false if it is the result
    /// of an interactive prompt.
    pub fn was_provided(&self) -> bool {
        self.was_provided
    }

    /// "Unwraps" this structure into its underlying string.
    pub fn into_inner(self) -> String {
        self.value
    }
}

/// Display a "<description> Continue?" confirmation. Returns true if the user
/// replies "yes" (or similar), or false otherwise.
pub fn continue_confirmation<IS: AbstractStream, OS: AbstractStream>(
    mut input_stream: IS,
    mut output_stream: OS,
    description: &str,
) -> Result<bool> {
    let mut input_reader = build_input_reader(&mut input_stream)?;
    let prompt = format!("{}Continue? [Yes/No] ", description);

    loop {
        let original_response = prompt_for_string_impl(
            &mut input_stream,
            &mut input_reader,
            &mut output_stream,
            prompt.as_str(),
            /*is_sensitive=*/ false,
        )?;
        let response = original_response.trim().to_lowercase();
        if response == "y" || response == "yes" {
            return Ok(true);
        } else if response == "n" || response == "no" {
            return Ok(false);
        } else {
            let mut writer = match output_stream.as_writer() {
                None => {
                    return Err(Error::Precondition(format!(
                        "the given output stream must support `Write`"
                    )))
                }
                Some(w) => w,
            };
            write!(writer, "Invalid response '{}'.\n", original_response)?;
            // We have to flush so the user sees the prompt immediately.
            writer.flush()?;
        }
    }
}
