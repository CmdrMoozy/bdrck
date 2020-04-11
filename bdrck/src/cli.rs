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
use atty;
use failure::format_err;
use libc::{self, c_int};
use std::io::{self, Read, Write};
use std::mem::MaybeUninit;

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
    inner: MaybeUninit<libc::termios>,
}

impl TerminalAttributes {
    fn new(fd: c_int) -> IoResult<Self> {
        let mut attrs = MaybeUninit::uninit();
        to_io_result(unsafe { libc::tcgetattr(fd, attrs.as_mut_ptr()) })?;
        Ok(TerminalAttributes { inner: attrs })
    }

    fn apply(&self, fd: c_int) -> IoResult<()> {
        to_io_result(unsafe { libc::tcsetattr(fd, libc::TCSANOW, self.inner.as_ptr()) })
    }
}

impl AbstractTerminalAttributes for TerminalAttributes {
    fn enable(&mut self, flag: TerminalFlag) {
        unsafe { *self.inner.as_mut_ptr() }.c_lflag &= !flag.to_value();
    }

    fn disable(&mut self, flag: TerminalFlag) {
        unsafe { *self.inner.as_mut_ptr() }.c_lflag |= flag.to_value();
    }
}

/// This trait describes an abstract input or output stream.
///
/// This trait primarily exists for testing purposes. In almost all cases, users
/// will instead just use the concrete type `Stream` defined below.
pub trait AbstractStream {
    /// A type which describes the attributes of this stream / terminal.
    type Attributes: AbstractTerminalAttributes;

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
#[derive(Clone, Copy, Debug)]
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
        atty::is(match *self {
            Stream::Stdout => atty::Stream::Stdout,
            Stream::Stderr => atty::Stream::Stderr,
            Stream::Stdin => atty::Stream::Stdin,
        })
    }

    fn get_attributes(&self) -> IoResult<Self::Attributes> {
        TerminalAttributes::new(self.to_fd())
    }

    fn set_attributes(&mut self, attributes: &Self::Attributes) -> IoResult<()> {
        attributes.apply(self.to_fd())
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
struct DisableEcho<S: AbstractStream> {
    stream: S,
    initial_attributes: S::Attributes,
}

impl<S: AbstractStream> DisableEcho<S> {
    fn new(mut stream: S) -> Result<Self> {
        let initial_attributes = stream.get_attributes()?;

        let mut attributes = stream.get_attributes()?;
        // Don't echo characters typed to stdin.
        attributes.disable(TerminalFlag::Echo);
        // But, *do* echo the newline when the user hits ENTER.
        attributes.enable(TerminalFlag::EchoNewlines);
        stream.set_attributes(&attributes)?;

        Ok(DisableEcho {
            stream: stream,
            initial_attributes: initial_attributes,
        })
    }
}

impl<S: AbstractStream> Drop for DisableEcho<S> {
    fn drop(&mut self) {
        self.stream
            .set_attributes(&self.initial_attributes)
            .unwrap();
    }
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
    input_stream: IS,
    output_stream: OS,
    prompt: &str,
    is_sensitive: bool,
) -> Result<String> {
    use io::BufRead;

    if !input_stream.isatty() {
        return Err(Error::Precondition(format_err!(
            "Cannot prompt for interactive user input when the input stream is not a TTY"
        )));
    }
    if !output_stream.isatty() {
        return Err(Error::Precondition(format_err!(
            "Cannot prompt for interactive user input when the output straem is not a TTY"
        )));
    }

    let mut reader = io::BufReader::new(match input_stream.as_reader() {
        None => {
            return Err(Error::Precondition(format_err!(
                "The given input stream must support `Read`"
            )))
        }
        Some(r) => r,
    });
    let mut writer = match output_stream.as_writer() {
        None => {
            return Err(Error::Precondition(format_err!(
                "The given output stream must support `Write`"
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
        reader.read_line(&mut ret)?;
        remove_newline(ret)?
    })
}

/// Prompt for a string as per `prompt_for_string`, but additionally have the
/// user enter the value again to confirm we get the same answer twice. This is
/// useful for e.g. password entry.
pub fn prompt_for_string_confirm<IS: AbstractStream + Copy, OS: AbstractStream + Copy>(
    input_stream: IS,
    output_stream: OS,
    prompt: &str,
    is_sensitive: bool,
) -> Result<String> {
    loop {
        let string = prompt_for_string(input_stream, output_stream, prompt, is_sensitive)?;
        if string == prompt_for_string(input_stream, output_stream, "Confirm: ", is_sensitive)? {
            return Ok(string);
        }
    }
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
    pub fn new<IS: AbstractStream + Copy, OS: AbstractStream + Copy>(
        provided: Option<&str>,
        input_stream: IS,
        output_stream: OS,
        prompt: &str,
        is_sensitive: bool,
        confirm: bool,
    ) -> Result<Self> {
        let prompted: Option<String> = match provided {
            None => Some(match confirm {
                false => prompt_for_string(input_stream, output_stream, prompt, is_sensitive)?,
                true => {
                    prompt_for_string_confirm(input_stream, output_stream, prompt, is_sensitive)?
                }
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
pub fn continue_confirmation<IS: AbstractStream + Copy, OS: AbstractStream + Copy>(
    input_stream: IS,
    output_stream: OS,
    description: &str,
) -> Result<bool> {
    let prompt = format!("{}Continue? [Yes/No] ", description);
    loop {
        let original_response = prompt_for_string(
            input_stream,
            output_stream,
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
                    return Err(Error::Precondition(format_err!(
                        "The given output stream must support `Write`"
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
