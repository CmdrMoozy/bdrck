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

/// Standard input / output streams.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
