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
use crate::flags::command::{Command, CommandResult};
use crate::flags::parse_and_execute::{parse_and_execute, parse_and_execute_single_command};
use std::env;
use std::fmt::{Debug, Display};
use std::process;

/// The integer which is returned from main() if the program exits successfully.
pub(crate) const EXIT_SUCCESS: i32 = 0;
/// The integer which is returned from main() if the program exits with any
/// error.
pub(crate) const EXIT_FAILURE: i32 = 1;

/// Returns the current program's parameters (accessed essentialy via
/// `std::env::args`) collected into a Vec. The 0'th parameter (the executable)
/// is omitted.
pub(crate) fn get_program_parameters() -> Vec<String> {
    env::args()
        .skip(1) // Skip the first argument, which is our executable.
        .collect()
}

/// This is a utility function, which handles the given result returned by a
/// Command implementation. The *outer* Result being an Err means that something
/// went wrong internally in the command-line argument parsing library. The
/// *inner* Result, on the other hand, is the actual Result returned by the
/// caller-provided Command implementation itself.
///
/// Overall, if an error is encountered, it is printed to standard output. In
/// either case, the appropriate exit code (EXIT_SUCCESS or EXIT_FAILURE) is
/// returned.
pub(crate) fn handle_result<E: Display + Debug>(r: Result<Option<CommandResult<E>>>) -> i32 {
    match r {
        // No internal error.
        Ok(r) => match r {
            // The command was not executed, but the error was handled internally.
            None => EXIT_FAILURE,
            // The command was executed, and we got a result back from it.
            Some(r) => match r {
                // The command returned success.
                Ok(_) => EXIT_SUCCESS,
                // The command returned an error to us.
                Err(e) => {
                    eprintln!(
                        "{}",
                        match cfg!(debug_assertions) {
                            false => e.to_string(),
                            true => format!("{:?}", e),
                        }
                    );
                    EXIT_FAILURE
                }
            },
        },
        // An internal error which should be surfaced to the user.
        Err(e) => {
            eprintln!(
                "Error parsing command-line flags: {}",
                match cfg!(debug_assertions) {
                    false => e.to_string(),
                    true => format!("{:?}", e),
                },
            );
            EXIT_FAILURE
        }
    }
}

/// Parses command-line parameters and executes the specified command.
///
/// This function exits this process with an appropriate exit code. Like
/// `std::process::exit`, because this function never returns and it terminates
/// the process, no destructors on the current stack or any other thread's
/// stack will be run. The caller should ensure that this function is called
/// from the only thread, and that any destructors which need to be run are in
/// the stack of the command callback.
pub fn main_impl<E: Display + Debug>(commands: Vec<Command<E>>) -> ! {
    process::exit(handle_result(parse_and_execute(
        env::args().next().unwrap().as_ref(),
        &get_program_parameters(),
        commands,
        Some(::std::io::stderr()),
    )));
}

/// Parses command-line parameters and executes the given command.
///
/// This function exits this process with an appropriate exit code. Like
/// `std::process::exit`, because this function never returns and it terminates
/// the process, no destructors on the current stack or any other thread's
/// stack will be run. The caller should ensure that this function is called
/// from the only thread, and that any destructors which need to be run are in
/// the stack of the command callback.
pub fn main_impl_single_command<E: Display + Debug>(command: Command<E>) -> ! {
    process::exit(handle_result(parse_and_execute_single_command(
        env::args().next().unwrap().as_ref(),
        &get_program_parameters(),
        command,
        Some(::std::io::stderr()),
    )));
}
