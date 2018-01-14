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
use flags::command::{Command, CommandResult};
use flags::parse_and_execute::parse_and_execute;
use flags::parse_and_execute::parse_and_execute_command;
use std::env;
use std::process;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

/// Returns the current program's parameters (accessed essentialy via
/// `std::env::args`) collected into a Vec. The 0'th parameter (the executable)
/// is omitted.
pub fn get_program_parameters() -> Vec<String> {
    env::args()
        .skip(1) // Skip the first argument, which is our executable.
        .collect()
}

fn handle_result<E: ::std::error::Error>(r: Result<CommandResult<E>>) -> i32 {
    match r {
        Ok(command_result) => match command_result {
            Ok(_) => EXIT_SUCCESS,
            Err(e) => {
                error!("{}", e);
                EXIT_FAILURE
            },
        },
        Err(e) => {
            error!("Internal error: {}", e);
            EXIT_FAILURE
        },
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
pub fn main_impl_multiple_commands<E: ::std::error::Error>(commands: Vec<Command<E>>) -> ! {
    process::exit(handle_result(parse_and_execute_command(
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
pub fn main_impl_single_command<E: ::std::error::Error>(command: Command<E>) -> ! {
    process::exit(handle_result(parse_and_execute(
        env::args().next().unwrap().as_ref(),
        &get_program_parameters(),
        command,
        Some(::std::io::stderr()),
    )));
}
