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

use error::*;
use libc::{self, c_int};
use std::io::{self, Write};

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

/// Return whether or not the given stream is a TTY (as opposed to, for example,
/// a pipe).
pub fn isatty(stream: Stream) -> bool {
    ::atty::is(match stream {
        Stream::Stdout => ::atty::Stream::Stdout,
        Stream::Stderr => ::atty::Stream::Stderr,
        Stream::Stdin => ::atty::Stream::Stdin,
    })
}

fn to_io_result(ret: c_int) -> ::std::io::Result<()> {
    match ret {
        0 => Ok(()),
        _ => Err(::std::io::Error::last_os_error()),
    }
}

// This struct handles a) disabling the echoing of characters typed to stdin,
// and b) remembering to reset the terminal attributes afterwards (via Drop).
struct DisableEcho {
    initial_attributes: libc::termios,
}

impl DisableEcho {
    fn new() -> Result<Self> {
        let mut initial_attributes = unsafe { ::std::mem::uninitialized() };
        let mut attributes = unsafe { ::std::mem::uninitialized() };
        to_io_result(unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut initial_attributes) })?;
        to_io_result(unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut attributes) })?;

        // Don't echo characters typed to stdin.
        attributes.c_lflag &= !libc::ECHO;
        // But, *do* echo the newline when the user hits ENTER.
        attributes.c_lflag |= libc::ECHONL;
        to_io_result(unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &attributes) })?;

        Ok(DisableEcho {
            initial_attributes: initial_attributes,
        })
    }
}

impl Drop for DisableEcho {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &self.initial_attributes);
        }
    }
}

fn remove_newline(mut s: String) -> Result<String> {
    // Remove the trailing newline (if any - not finding one is an error).
    if !s.ends_with('\n') {
        return Err(::std::io::Error::new(
            ::std::io::ErrorKind::UnexpectedEof,
            "unexpected end of input",
        ).into());
    }
    s.pop();

    // If this is windows and so there's also a \r, remove that too.
    if s.ends_with('\r') {
        s.pop();
    }

    Ok(s)
}

/// Prompt the user for a string (read from Stdin) using the given output stream
/// (Stdout or Stderr) to display the given prompt message.
///
/// If `is_sensitive` is true, then the users characters will not be echoed back
/// (e.g. this will behave like a password prompt).
///
/// Note that it is an error for output_stream to be Stdin, or for this function
/// to be called when the given output stream or Stdin are not TTYs.
pub fn prompt_for_string(
    output_stream: Stream,
    prompt: &str,
    is_sensitive: bool,
) -> Result<String> {
    if !isatty(output_stream) || !isatty(Stream::Stdin) {
        return Err(Error::Precondition(format_err!(
            "Cannot prompt for interactive user input when {:?} and Stdin are not TTYs",
            output_stream
        )));
    }

    let mut output_stream: Box<dyn Write> = match output_stream {
        Stream::Stdout => Box::new(io::stdout()),
        Stream::Stderr => Box::new(io::stderr()),
        Stream::Stdin => {
            return Err(Error::InvalidArgument(format_err!(
                "Cannot output interactive prompts on Stdin"
            )))
        }
    };
    write!(output_stream, "{}", prompt)?;
    // We have to flush so the user sees the prompt immediately.
    output_stream.flush()?;

    Ok({
        let _disable_echo = match is_sensitive {
            false => None,
            true => Some(DisableEcho::new()?),
        };
        let mut ret = String::new();
        io::stdin().read_line(&mut ret)?;
        remove_newline(ret)?
    })
}
