use std::string::String;
use std::vec::Vec;

use super::command::Command;
use super::detail::build_default_options;
use super::detail::parse_command;
use super::option::Option;

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
fn test_parse_command_with_arguments() {
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

#[test]
fn test_build_default_options() {
    let command = Command {
        name: "test command".to_string(),
        help: "test command".to_string(),
        options: vec![
            Option::required("a", "a", None, None),
            Option::required("b", "b", None, Some("b")),
            Option::required("c", "c", Some('c'), None),
            Option::required("d", "d", Some('d'), Some("d")),
            Option::optional("e", "e", None),
            Option::optional("f", "f", Some('f')),
            Option::flag("g", "g", None),
            Option::flag("h", "h", Some('h')),
        ],
        arguments: Vec::new(),
        last_argument_is_variadic: false,
    };

    let (options, flags) = build_default_options(&command);

    assert!(options.len() == 2);
    assert!(options.get(&"b".to_string()).map_or(false, |v| *v == "b"));
    assert!(options.get(&"d".to_string()).map_or(false, |v| *v == "d"));

    assert!(flags.len() == 2);
    assert!(flags.get(&"g".to_string()).map_or(false, |v| *v == false));
    assert!(flags.get(&"h".to_string()).map_or(false, |v| *v == false));
}
