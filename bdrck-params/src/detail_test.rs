use std::string::String;
use std::vec::Vec;

use super::command::Command;
use super::detail::parse_command;

fn build_command_for_test(name: &str) -> Command {
    return Command {
        name: name.to_string(),
        help: name.to_string(),
        options: Vec::new(),
        arguments: Vec::new(),
        last_argument_is_variadic: false,
    };
}

fn parse_command_works(program_parameters: &Vec<String>,
                       commands: &Vec<Command>,
                       expected_name: &str)
                       -> bool {
    return parse_command(program_parameters.iter(), commands.iter())
        .map_or(false, |c| c.name == expected_name);
}

#[test]
fn test_parse_invalid_command() {
    let program_parameters: Vec<String> = vec![
		"biff".to_string(),
		"foo".to_string(),
		"bar".to_string(),
		"baz".to_string(),
	];

    let commands = vec![
		build_command_for_test("foo"),
		build_command_for_test("bar"),
		build_command_for_test("baz"),
	];

    assert!(parse_command(program_parameters.iter(), commands.iter()).is_none());
}

#[test]
fn test_parse_command_no_arguments() {
    let program_parameters: Vec<String> = vec![
		"bar".to_string(),
	];

    let commands = vec![
		build_command_for_test("foo"),
		build_command_for_test("bar"),
		build_command_for_test("baz"),
	];

    assert!(parse_command_works(&program_parameters, &commands, "bar"));
}

#[test]
fn test_parse_command_arguments() {
    let program_parameters: Vec<String> = vec![
		"baz".to_string(),
		"foo".to_string(),
		"bar".to_string(),
		"baz".to_string(),
	];

    let commands = vec![
		build_command_for_test("foo"),
		build_command_for_test("bar"),
		build_command_for_test("baz"),
	];

    assert!(parse_command_works(&program_parameters, &commands, "baz"));
}
