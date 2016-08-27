use std::collections::HashMap;
use std::env;
use std::iter::Peekable;
use std::option::Option as Optional;
use std::result::Result;
use std::string::String;
use std::vec::Vec;

use super::ErrorKind;
use super::ParamsError;
use super::command::Command;
use super::option::Option;
use super::option::find_option;

pub fn get_program_parameters() -> Vec<String> {
    //! Returns the current program's parameters (accessed essentialy via
    //! std::env::args()) collected into a Vec. The 0'th parameter (the executable)
    //! is omitted.

    return env::args()
        .skip(1) // Skip the first argument, which is our executable.
        .collect::<Vec<String>>();
}

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

/// An option parameter is the string representation of an option, extracted
/// from an iterator
/// over program parameters.
struct OptionParameters<'a> {
    name: &'a str,
    value: Optional<&'a str>,
    option_obj: &'a Option,
}

fn extract_option_name_and_value<'a>(option_parameter: &'a str) -> (&'a str, Optional<&'a str>) {
    //! Extracts the option name (and, if one is present, the option value) from
    //! the given
    //! single option parameter. This involves stripping off the leading "-" or
    //! "--" prefix,
    //! and splitting the given parameter on its "=" character (if any).

    let trimmed: &str = match option_parameter.starts_with("--") {
        false => &option_parameter[1..],
        true => &option_parameter[2..],
    };
    let equals_idx = trimmed.rfind('=');
    let name: &str = equals_idx.map_or(trimmed, |ei| &trimmed[0..ei]);
    let value: Optional<&str> = equals_idx.map_or(None, |ei| Some(&trimmed[ei + 1..]));

    return (name, value);
}

fn next_option_parameters<'a, PI, OI>(parameters: &mut Peekable<PI>,
                                      options: OI)
                                      -> Result<Optional<OptionParameters<'a>>, ParamsError>
    where PI: Iterator<Item = &'a String>,
          OI: Iterator<Item = &'a Option>
{
    //! Parse an option name and value from the given iterator's current position.
    //! If the given
    //! iterator is already at the end or the string at its current position is not
    //! an option,
    //! return None. Otherwise, return either a valid option or an error.

    let parameter: &'a str;
    {
        let op: Optional<&&'a String> = parameters.peek();
        if op.is_none() {
            return Ok(None);
        }
        parameter = op.unwrap();

        // If the given option doesn't start with "-", assume it's an argument instead.
        if !parameter.starts_with("-") {
            return Ok(None);
        }
    }
    // Since we got a valid option, advance the iterator.
    parameters.next();

    let (name, value) = extract_option_name_and_value(parameter);

    // Lookup the option by name.
    let option_obj: &Option;
    {
        let oo: Optional<&Option> = find_option(options, name);
        if oo.is_none() {
            return Err(ParamsError {
                kind: ErrorKind::UnrecognizedOption { name: name.to_owned() },
            });
        }
        option_obj = oo.unwrap();
    }

    let next_parameter_is_value: bool = parameters.peek().map_or(false, |v| !v.starts_with("-"));
    return Ok(Some(OptionParameters {
        name: option_obj.name.as_ref(),
        value: value.or(match next_parameter_is_value {
            false => None,
            true => Some(parameters.next().unwrap().as_ref()),
        }),
        option_obj: option_obj,
    }));
}

#[derive(Eq, PartialEq)]
struct ParsedOption<'a> {
    name: &'a str,
    value: Optional<&'a str>,
    bool_value: Optional<bool>,
}

fn parse_bool<S>(value: S) -> Result<bool, ParamsError>
    where S: AsRef<str>
{
    return match value.as_ref().trim().to_lowercase().as_ref() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => {
            Err(ParamsError {
                kind: ErrorKind::InvalidBooleanValue { value: value.as_ref().to_string() },
            })
        },
    };
}

fn parse_option<'a, PI, OI>(parameters: &mut Peekable<PI>,
                            options: OI)
                            -> Result<Optional<ParsedOption<'a>>, ParamsError>
    where PI: Iterator<Item = &'a String>,
          OI: Iterator<Item = &'a Option>
{
    let option_parameters: OptionParameters<'a>;
    {
        let opr = next_option_parameters(parameters, options);
        if opr.is_err() {
            return Err(opr.err().unwrap());
        }
        let opo = opr.ok().unwrap();
        if opo.is_none() {
            return Ok(None);
        }
        option_parameters = opo.unwrap();
    }

    let bool_value: Optional<bool>;
    {
        if option_parameters.option_obj.is_flag {
            if let Some(v) = option_parameters.value {
                let bv = parse_bool(v);
                if bv.is_err() {
                    return Err(bv.err().unwrap());
                }
                bool_value = Some(bv.ok().unwrap());
            } else {
                bool_value = Some(true);
            }
        } else {
            bool_value = None;
        }
    }

    return Ok(Some(ParsedOption {
        name: option_parameters.name,
        value: option_parameters.value,
        bool_value: bool_value,
    }));
}

/// This structure encapsulates the output from parsing the program's parameters
/// according to a Command. It provides accessor functions to retrieve the
/// values conveniently.
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

    pub fn get_option<S>(&self, name: S) -> Optional<&&String>
        where S: AsRef<str>
    {
        return self.options.get(&name.as_ref().to_owned());
    }

    pub fn get_flag<S>(&self, name: S) -> Optional<&bool>
        where S: AsRef<str>
    {
        return self.flags.get(&name.as_ref().to_owned());
    }

    pub fn get_argument<S>(&self, name: S) -> Optional<&Vec<&String>>
        where S: AsRef<str>
    {
        return self.arguments.get(&name.as_ref().to_string());
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::option::Option as Optional;
    use std::string::String;
    use std::vec::Vec;
    use super::ParsedOption;
    use super::ParsedParameters;

    use super::build_default_options;
    use super::parse_command;
    use super::parse_option;
    use super::super::ErrorKind;
    use super::super::ParamsError;
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

    #[test]
    fn test_parse_option() {
        struct TestCase<'a>(Vec<String>,
                            Vec<Option>,
                            Result<Optional<ParsedOption<'a>>, ParamsError>);
        let test_cases: Vec<TestCase> = vec![
            // Empty iterator.
            TestCase(Vec::new(), Vec::new(), Ok(None)),
            // Iterator pointing to an argument instead of an option.
            TestCase(vec!["foobar".to_owned()], Vec::new(), Ok(None)),
            // Option name not found.
            TestCase(vec!["--foobar".to_owned()], Vec::new(), Err(ParamsError {
                kind: ErrorKind::UnrecognizedOption { name: "foobar".to_owned() },
            })),
            // Option with value, using "-" or "--" and long or short name.
            TestCase(
                vec!["--foobar=baz".to_owned()],
                vec![Option::required("foobar", "foobar", Some('f'), None)],
                Ok(Some(ParsedOption {
                    name: "foobar",
                    value: Some("baz"),
                    bool_value: None,
                }))),
            TestCase(
                vec!["-f=baz".to_owned()],
                vec![Option::required("foobar", "foobar", Some('f'), None)],
                Ok(Some(ParsedOption {
                    name: "foobar",
                    value: Some("baz"),
                    bool_value: None,
                }))),
            TestCase(
                vec!["-foobar".to_owned(), "baz".to_owned()],
                vec![Option::required("foobar", "foobar", Some('f'), None)],
                Ok(Some(ParsedOption {
                    name: "foobar",
                    value: Some("baz"),
                    bool_value: None,
                }))),
            TestCase(
                vec!["--f".to_owned(), "baz".to_owned()],
                vec![Option::required("foobar", "foobar", Some('f'), None)],
                Ok(Some(ParsedOption {
                    name: "foobar",
                    value: Some("baz"),
                    bool_value: None,
                }))),
            // Flag with no explicit value, using "-" or "--" and long or short name.
            TestCase(
                vec!["--foobar".to_owned()],
                vec![Option::flag("foobar", "foobar", Some('f'))],
                Ok(Some(ParsedOption {
                    name: "foobar",
                    value: None,
                    bool_value: Some(true),
                }))),
            TestCase(
                vec!["-f".to_owned()],
                vec![Option::flag("foobar", "foobar", Some('f'))],
                Ok(Some(ParsedOption {
                    name: "foobar",
                    value: None,
                    bool_value: Some(true),
                }))),
            // Flag with explicit value, using "-" or "--" and long or short name.
            TestCase(
                vec!["--foobar=true".to_owned()],
                vec![Option::flag("foobar", "foobar", Some('f'))],
                Ok(Some(ParsedOption {
                    name: "foobar",
                    value: Some("true"),
                    bool_value: Some(true),
                }))),
            TestCase(
                vec!["-f=false".to_owned()],
                vec![Option::flag("foobar", "foobar", Some('f'))],
                Ok(Some(ParsedOption {
                    name: "foobar",
                    value: Some("false"),
                    bool_value: Some(false),
                }))),
            TestCase(
                vec!["-foobar".to_owned(), "false".to_owned()],
                vec![Option::flag("foobar", "foobar", Some('f'))],
                Ok(Some(ParsedOption {
                    name: "foobar",
                    value: Some("false"),
                    bool_value: Some(false),
                }))),
            TestCase(
                vec!["--f".to_owned(), "true".to_owned()],
                vec![Option::flag("foobar", "foobar", Some('f'))],
                Ok(Some(ParsedOption {
                    name: "foobar",
                    value: Some("true"),
                    bool_value: Some(true),
                }))),
        ];

        for test_case in &test_cases {
            assert!(parse_option(&mut test_case.0.iter().peekable(), test_case.1.iter()) ==
                    test_case.2);
        }
    }
}
