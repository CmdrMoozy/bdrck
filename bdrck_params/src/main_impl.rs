use ::command::{CommandResult, ExecutableCommand};
use ::error::Result;
use ::parse_and_execute::parse_and_execute;
use ::parse_and_execute::parse_and_execute_command;
use ::parsed_parameters::get_program_parameters;
use std::env;
use std::error;
use std::process;
use std::vec::Vec;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

fn handle_result<E: error::Error>(r: Result<CommandResult<E>>) -> i32 {
    match r {
        Ok(command_result) => {
            match command_result {
                Ok(_) => EXIT_SUCCESS,
                Err(e) => {
                    error!("{}", e);
                    EXIT_FAILURE
                },
            }
        },
        Err(e) => {
            error!("Internal error: {}", e);
            EXIT_FAILURE
        },
    }
}

pub fn main_impl_multiple_commands<E: error::Error>(commands: Vec<ExecutableCommand<E>>) -> ! {
    //! Parses command-line parameters and executes the specified command.
    //!
    //! This function exits this process with an appropriate exit code. Like
    //! std::process::exit, because this function never returns and it terminates
    //! the process, no destructors on the current stack or any other thread's
    //! stack will be run. The caller should ensure that this function is called
    //! from the only thread, and that any destructors which need to be run are in
    //! the stack of the command callback.

    process::exit(handle_result(parse_and_execute_command(env::args().next().unwrap().as_ref(),
                                                          &get_program_parameters(),
                                                          commands)));
}

pub fn main_impl_single_command<E: error::Error>(command: ExecutableCommand<E>) -> ! {
    //! Parses command-line parameters and executes the given command.
    //!
    //! This function exits this process with an appropriate exit code. Like
    //! std::process::exit, because this function never returns and it terminates
    //! the process, no destructors on the current stack or any other thread's
    //! stack will be run. The caller should ensure that this function is called
    //! from the only thread, and that any destructors which need to be run are in
    //! the stack of the command callback.

    process::exit(handle_result(parse_and_execute(env::args().next().unwrap().as_ref(),
                                                  &get_program_parameters(),
                                                  command)));
}
