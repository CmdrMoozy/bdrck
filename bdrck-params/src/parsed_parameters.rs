use std::collections::HashMap;
use std::env;
use std::option::Option as Optional;
use std::result::Result;
use std::string::String;
use std::vec::Vec;

use super::ErrorKind;
use super::ParamsError;
use super::command::Command;

fn parse_command<'a, PI, CI>(mut parameters: PI,
                             mut commands: CI)
                             -> Result<&'a Command, ParamsError>
    where PI: Iterator<Item = &'a String>,
          CI: Iterator<Item = &'a Command>
{
    //! Look up by name the command indicated by the first element of the given
    //! range of program parameters. If a matching command could not be found,
    //! return None instead.

    if let Some(command_parameter) = parameters.next() {
        return commands.find(|&command| command.name == **command_parameter)
            .map_or(Err(ParamsError {
                        kind: ErrorKind::UnrecognizedCommand { name: (*command_parameter).clone() },
                    }),
                    |command| Ok(command));
    } else {
        return Err(ParamsError { kind: ErrorKind::NoCommandSpecified });
    }
}

fn build_default_options<'a>(parsed: &mut ParsedParameters<'a>) {
    //! Constructs maps for options and flags which contain the default values (if
    //! any) for each of the given command's options. Note that all flags have a
    //! default value of false.

    for o in &parsed.command.options {
        if let Some(ref dv) = o.default_value {
            parsed.options.insert(&o.name, dv);
        } else if o.is_flag {
            parsed.flags.insert(&o.name, false);
        }
    }
}

fn all_options_are_present<'a>(parsed: &ParsedParameters<'a>) -> Optional<ParamsError> {
    //! Checks if all of the given command's options are present in the given map
    //! of option names to values. If an option is missing, returns an error with
    //! more detailed information. Otherwise, returns None.

    for o in &parsed.command.options {
        if o.is_optional || o.is_flag {
            continue;
        }

        if parsed.get_option(&o.name).is_none() {
            return Some(ParamsError {
                kind: ErrorKind::MissingOptionValue { name: o.name.clone() },
            });
        }
    }

    return None;
}

pub fn get_program_parameters() -> Vec<String> {
    //! Returns the current program's parameters (accessed essentialy via
    //! std::env::args()) collected into a Vec. The 0'th parameter (the executable)
    //! is omitted.

    return env::args()
        .skip(1) // Skip the first argument, which is our executable.
        .collect::<Vec<String>>();
}

/// This structure encapsulates the output from parsing the program's parameters
/// according to a Command. It provides accessor functions to retrieve the
/// values
/// conveniently.
pub struct ParsedParameters<'a> {
    command: &'a Command,
    options: HashMap<&'a String, &'a String>,
    flags: HashMap<&'a String, bool>,
    arguments: HashMap<&'a String, Vec<&'a String>>,
}

impl<'a> ParsedParameters<'a> {
    pub fn new<PI, CI>(parameters: PI, commands: CI) -> Result<ParsedParameters<'a>, ParamsError>
        where PI: Iterator<Item = &'a String>,
              CI: Iterator<Item = &'a Command>
    {
        //! Construct a new ParsedParameters instance by parsing the command,
        //! options, flags, and arguments from the given iterator over the set
        //! of program parameters.

        let command_result = parse_command(parameters, commands);
        if command_result.is_err() {
            return Err(command_result.unwrap_err());
        }

        let mut parsed = ParsedParameters {
            command: command_result.unwrap(),
            options: HashMap::new(),
            flags: HashMap::new(),
            arguments: HashMap::new(),
        };

        build_default_options(&mut parsed);

        return all_options_are_present(&parsed).map_or(Ok(parsed), |e| Err(e));
    }

    pub fn get_option<S>(&self, name: S) -> Option<&&String>
        where S: AsRef<str>
    {
        return self.options.get(&name.as_ref().to_string());
    }

    pub fn get_flag<S>(&self, name: S) -> Option<&bool>
        where S: AsRef<str>
    {
        return self.flags.get(&name.as_ref().to_string());
    }

    pub fn get_argument<S>(&self, name: S) -> Option<&Vec<&String>>
        where S: AsRef<str>
    {
        return self.arguments.get(&name.as_ref().to_string());
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::string::String;
    use std::vec::Vec;
    use super::ParsedParameters;

    use super::build_default_options;
    use super::parse_command;
    use super::super::command::Command;
    use super::super::option::Option;

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
            .ok()
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

        assert!(parse_command(program_parameters.iter(), commands.iter()).ok().is_none());
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
            name: "test".to_string(),
            help: "test".to_string(),
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

        let mut parsed = ParsedParameters {
            command: &command,
            options: HashMap::new(),
            flags: HashMap::new(),
            arguments: HashMap::new(),
        };

        build_default_options(&mut parsed);

        assert!(parsed.options.len() == 2);
        assert!(parsed.options.get(&"b".to_string()).map_or(false, |v| *v == "b"));
        assert!(parsed.options.get(&"d".to_string()).map_or(false, |v| *v == "d"));

        assert!(parsed.flags.len() == 2);
        assert!(parsed.flags.get(&"g".to_string()).map_or(false, |v| *v == false));
        assert!(parsed.flags.get(&"h".to_string()).map_or(false, |v| *v == false));
    }

}
