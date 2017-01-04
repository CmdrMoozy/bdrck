use ::command::ExecutableCommand;
use ::error::Result;
use ::parse_and_execute::parse_and_execute;
use ::parse_and_execute::parse_and_execute_command;
use ::parsed_parameters::get_program_parameters;
use std::env;
use std::process;
use std::result;
use std::vec::Vec;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

fn handle_result<E>(r: Result<result::Result<(), E>>) -> i32 {
    match r {
        Ok(command_result) => {
            match command_result {
                Ok(_) => EXIT_SUCCESS,
                Err(_) => {
                    error!("Command returned error code");
                    EXIT_FAILURE
                },
            }
        },
        Err(e) => {
            error!("{}", e);
            EXIT_FAILURE
        },
    }
}

pub fn main_impl_multiple_commands<E>(mut commands: Vec<ExecutableCommand<E>>) -> ! {
    //! Parses command-line parameters and executes the specified command.
    //!
    //! This function exits this process with an appropriate exit code. Like
    //! std::process::exit, because this function never returns and it terminates
    //! the process, no destructors on the current stack or any other thread's
    //! stack will be run. The caller should ensure that this function is called
    //! from the only thread, and that any destructors which need to be run are in
    //! the stack of the command callback.

    let program = env::args().next().unwrap();
    let parameters = get_program_parameters();
    process::exit(handle_result(parse_and_execute_command(program.as_ref(),
                                                          &parameters,
                                                          &mut commands)));
}

pub fn main_impl_single_command<E>(command: ExecutableCommand<E>) -> ! {
    //! Parses command-line parameters and executes the given command.
    //!
    //! This function exits this process with an appropriate exit code. Like
    //! std::process::exit, because this function never returns and it terminates
    //! the process, no destructors on the current stack or any other thread's
    //! stack will be run. The caller should ensure that this function is called
    //! from the only thread, and that any destructors which need to be run are in
    //! the stack of the command callback.

    let program = env::args().next().unwrap();
    let parameters = get_program_parameters();
    process::exit(handle_result(parse_and_execute(program.as_ref(), parameters, command)));
}
