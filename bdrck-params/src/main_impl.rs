use std::env;
use std::process;
use std::vec::Vec;

use super::command::ExecutableCommand;
use super::parse_and_execute::parse_and_execute;
use super::parse_and_execute::parse_and_execute_command;
use super::parsed_parameters::get_program_parameters;

pub fn main_impl_multiple_commands(commands: Vec<ExecutableCommand>) -> ! {
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
    process::exit(parse_and_execute_command(program.as_ref(), &parameters, &commands));
}

pub fn main_impl_single_command(command: ExecutableCommand) -> ! {
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
    process::exit(parse_and_execute(program.as_ref(), parameters, command));
}
