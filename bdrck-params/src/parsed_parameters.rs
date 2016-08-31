use std::collections::HashMap;
use std::env;
use std::iter::Peekable;
use std::option::Option as Optional;
use std::result::Result;
use std::string::String;
use std::vec::Vec;

use super::ErrorKind;
use super::ParamsError;
use super::argument::Argument;
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

fn parse_command<'a, PI, CI>(parameters: &mut PI,
                             commands: &mut CI)
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

    let (name, mut value) = extract_option_name_and_value(parameter);

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
    if next_parameter_is_value && value.is_none() {
        value = Some(parameters.next().unwrap().as_ref());
    }
    return Ok(Some(OptionParameters {
        name: option_obj.name.as_ref(),
        value: value,
        option_obj: option_obj,
    }));
}

#[derive(Eq, PartialEq)]
struct ParsedOption<'a> {
    name: &'a str,
    value: Optional<&'a str>,
    bool_value: Optional<bool>,
}

fn parse_bool<'a>(value: &'a str) -> Result<bool, ParamsError> {
    //! Return the boolean interpretation of a string, or an error if the string
    //! isn't recognized as a valid boolean value.

    return match value.trim().to_lowercase().as_ref() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ParamsError { kind: ErrorKind::InvalidBooleanValue { value: value.to_owned() } }),
    };
}

fn parse_option<'a, PI, OI>(parameters: &mut Peekable<PI>,
                            options: OI)
                            -> Result<Optional<ParsedOption<'a>>, ParamsError>
    where PI: Iterator<Item = &'a String>,
          OI: Iterator<Item = &'a Option>
{
    //! Parse the next option from the given iterator over program parameters. If
    //! there are no more option parameters, returns None. If an option argument is
    //! found but some error occurs, then an error is returned instead.

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

    if !option_parameters.option_obj.is_flag && option_parameters.value.is_none() {
        return Err(ParamsError {
            kind: ErrorKind::MissingOptionValue { name: option_parameters.name.to_owned() },
        });
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

fn parse_all_options<'a, PI>(parameters: &mut Peekable<PI>,
                             parsed_parameters: &ParsedParameters<'a>)
                             -> Result<Vec<ParsedOption<'a>>, ParamsError>
    where PI: Iterator<Item = &'a String>
{
    //! Call parse_option repeatedly on the given iterator until an error is
    //! encountered or there are no more options to parse. Returns a possibly empty
    //! vector of parsed options, or an error if one was encountered.

    let mut parsed: Vec<ParsedOption<'a>> = Vec::new();
    loop {
        let pr = parse_option(parameters, parsed_parameters.command.options.iter());
        if pr.is_err() {
            return Err(pr.err().unwrap());
        }
        let po = pr.ok().unwrap();
        if po.is_none() {
            return Ok(parsed);
        }
        parsed.push(po.unwrap());
    }
}

fn emplace_all_options<'a, PI>(parameters: &mut Peekable<PI>,
                               parsed_parameters: &mut ParsedParameters<'a>)
                               -> Optional<ParamsError>
    where PI: Iterator<Item = &'a String>
{
    //! Calls parse_all_options, and adds the result to the given parsed parameters
    //! structure. An error is returned if one is encountered, and the parsed
    //! parameters structure is not modified.

    let po = parse_all_options(parameters, parsed_parameters);
    if po.is_err() {
        return Some(po.err().unwrap());
    }
    let parsed_options = po.ok().unwrap();

    for parsed_option in &parsed_options {
        if parsed_option.bool_value.is_none() {
            parsed_parameters.options.insert(parsed_option.name, parsed_option.value.unwrap());
        } else {
            parsed_parameters.flags.insert(parsed_option.name, parsed_option.bool_value.unwrap());
        }
    }

    return None;
}

fn parse_all_arguments<'a, PI>(parameters: &mut Peekable<PI>,
                               arguments: &'a Vec<Argument>,
                               last_argument_is_variadic: bool)
                               -> Result<HashMap<&'a str, Vec<&'a str>>, ParamsError>
    where PI: Iterator<Item = &'a String>
{
    //! Parses all of the positional arguments from the given iterator over program
    //! parameters, and returns either a possibly empty HashMap of parsed arguments
    //! or an error, if one is encountered.

    let mut parsed: HashMap<&'a str, Vec<&'a str>> = HashMap::new();
    if arguments.is_empty() {
        return Ok(parsed);
    }

    if arguments.len() >= 2 {
        for argument in &arguments[..arguments.len() - 1] {
            let v = parameters.next();
            if v.is_none() {
                return Err(ParamsError {
                    kind: ErrorKind::MissingArgumentValue { name: argument.name.clone() },
                });
            }
            parsed.insert(argument.name.as_str(), vec![v.unwrap().as_str()]);
        }
    }

    let last_argument: &Argument = &arguments[arguments.len() - 1];
    let last_argument_values: Vec<&'a str> = parameters.map(|v| v.as_str()).collect();
    if last_argument_is_variadic {
        parsed.insert(last_argument.name.as_str(), last_argument_values);
    } else {
        if last_argument_values.len() != 1 {
            return Err(ParamsError {
                kind: ErrorKind::WrongNumberOfArgumentValues { count: last_argument_values.len() },
            });
        }
        parsed.insert(last_argument.name.as_str(), last_argument_values);
    }

    return Ok(parsed);
}

fn emplace_all_arguments<'a, PI>(parameters: &mut Peekable<PI>,
                                 parsed_parameters: &mut ParsedParameters<'a>)
                                 -> Optional<ParamsError>
    where PI: Iterator<Item = &'a String>
{
    //! Parses all of the positional arguments from the given iterator over program
    //! parameters, and adds the result to the given parsed parameters structure.
    //! An error is returned if one is encountered, and the parsed parameters
    //! structure is not modified.

    let parsed_arguments = parse_all_arguments(parameters,
                                               &parsed_parameters.command.arguments,
                                               parsed_parameters.command.last_argument_is_variadic);
    if parsed_arguments.is_err() {
        return Some(parsed_arguments.err().unwrap());
    }
    parsed_parameters.arguments = parsed_arguments.ok().unwrap();
    return None;
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

/// This structure encapsulates the output from parsing the program's parameters
/// according to a Command. It provides accessor functions to retrieve the
/// values conveniently.
pub struct ParsedParameters<'a> {
    command: &'a Command,
    options: HashMap<&'a str, &'a str>,
    flags: HashMap<&'a str, bool>,
    arguments: HashMap<&'a str, Vec<&'a str>>,
}

impl<'a> ParsedParameters<'a> {
    pub fn new<PI, CI>(parameters: &mut PI,
                       commands: &mut CI)
                       -> Result<ParsedParameters<'a>, ParamsError>
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

        let mut peekable_parameters = parameters.peekable();

        build_default_options(&mut parsed);
        if let Some(e) = emplace_all_options(&mut peekable_parameters, &mut parsed) {
            return Err(e);
        }
        if let Some(e) = emplace_all_arguments(&mut peekable_parameters, &mut parsed) {
            return Err(e);
        }
        return all_options_are_present(&parsed).map_or(Ok(parsed), |e| Err(e));
    }

    pub fn get_option<'b>(&'b self, name: &'b str) -> Optional<&&str> {
        return self.options.get(&name.as_ref());
    }

    pub fn get_flag<'b>(&'b self, name: &'b str) -> Optional<&bool> {
        return self.flags.get(&name.as_ref());
    }

    pub fn get_argument<'b>(&'b self, name: &'b str) -> Optional<&Vec<&str>> {
        return self.arguments.get(&name.as_ref());
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
    use super::super::argument::Argument;
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
        return parse_command(&mut program_parameters.iter(), &mut commands.iter())
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

        assert!(parse_command(&mut program_parameters.iter(), &mut commands.iter()).ok().is_none());
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
        assert!(parsed.options.get("b").map_or(false, |v| *v == "b"));
        assert!(parsed.options.get("d").map_or(false, |v| *v == "d"));

        assert!(parsed.flags.len() == 2);
        assert!(parsed.flags.get("g").map_or(false, |v| *v == false));
        assert!(parsed.flags.get("h").map_or(false, |v| *v == false));
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
            // Option with no value.
            TestCase(
                vec!["--foobar".to_owned()],
                vec![Option::required("foobar", "foobar", Some('f'), None)],
                Err(ParamsError {
                    kind: ErrorKind::MissingOptionValue { name: "foobar".to_owned() },
                })),
            TestCase(
                vec!["--foobar".to_owned(), "--barbaz".to_owned()],
                vec![Option::required("foobar", "foobar", Some('f'), None)],
                Err(ParamsError {
                    kind: ErrorKind::MissingOptionValue { name: "foobar".to_owned() },
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

    #[test]
    fn test_parsed_parameters_construction_options() {
        let parameters: Vec<String> = vec![
            "foobar".to_owned(),
            "--opta".to_owned(),
            "foo".to_owned(),
            "--optc".to_owned(),
            "bar".to_owned(),
            "-f".to_owned(),
            "--g=false".to_owned(),
        ];

        let commands = vec![
            Command {
                name: "foobar".to_owned(),
                help: "foobar".to_owned(),
                options: vec![
                    Option::required("opta", "opta", Some('a'), None),
                    Option::required("optb", "optb", Some('b'), Some("baz")),
                    Option::optional("optc", "optc", Some('c')),
                    Option::optional("optd", "optd", Some('d')),
                    Option::flag("opte", "opte", Some('e')),
                    Option::flag("optf", "optf", Some('f')),
                    Option::flag("optg", "optg", Some('g')),
                ],
                arguments: Vec::new(),
                last_argument_is_variadic: false,
            },
        ];

        let mut expected_options = HashMap::new();
        expected_options.insert("opta", "foo");
        expected_options.insert("optb", "baz");
        expected_options.insert("optc", "bar");

        let mut expected_flags = HashMap::new();
        expected_flags.insert("opte", false);
        expected_flags.insert("optf", true);
        expected_flags.insert("optg", false);

        let pr = ParsedParameters::new(&mut parameters.iter(), &mut commands.iter());
        assert!(pr.is_ok());
        let parsed = pr.ok().unwrap();

        assert!(parsed.command.name == commands[0].name);
        assert!(parsed.options == expected_options);
        assert!(parsed.flags == expected_flags);
    }

    #[test]
    fn test_parsed_parameters_construction_arguments() {
        let parameters: Vec<String> = vec![
            "foobar".to_owned(),
            "--opta=oof".to_owned(),
            "foo".to_owned(),
            "bar".to_owned(),
            "baz".to_owned(),
        ];

        let commands = vec![
            Command {
                name: "foobar".to_owned(),
                help: "foobar".to_owned(),
                options: vec![
                    Option::required("opta", "opta", Some('a'), None),
                ],
                arguments: vec![
                    Argument {
                        name: "arga".to_owned(),
                        help: "arga".to_owned(),
                    },
                    Argument {
                        name: "argb".to_owned(),
                        help: "argb".to_owned(),
                    },
                    Argument {
                        name: "argc".to_owned(),
                        help: "argc".to_owned(),
                    },
                ],
                last_argument_is_variadic: false,
            },
        ];

        let mut expected_arguments = HashMap::new();
        expected_arguments.insert("arga", vec!["foo"]);
        expected_arguments.insert("argb", vec!["bar"]);
        expected_arguments.insert("argc", vec!["baz"]);

        let pr = ParsedParameters::new(&mut parameters.iter(), &mut commands.iter());
        assert!(pr.is_ok());
        let parsed = pr.ok().unwrap();

        assert!(parsed.command.name == commands[0].name);
        assert!(parsed.arguments.len() == expected_arguments.len());
        assert!(parsed.arguments == expected_arguments);
    }

    #[test]
    fn test_parsed_parameters_construction_variadic_last_argument_empty() {
        let parameters: Vec<String> = vec![
            "foobar".to_owned(),
            "--opta=oof".to_owned(),
            "foo".to_owned(),
            "bar".to_owned(),
        ];

        let commands = vec![
            Command {
                name: "foobar".to_owned(),
                help: "foobar".to_owned(),
                options: vec![
                    Option::required("opta", "opta", Some('a'), None),
                ],
                arguments: vec![
                    Argument {
                        name: "arga".to_owned(),
                        help: "arga".to_owned(),
                    },
                    Argument {
                        name: "argb".to_owned(),
                        help: "argb".to_owned(),
                    },
                    Argument {
                        name: "argc".to_owned(),
                        help: "argc".to_owned(),
                    },
                ],
                last_argument_is_variadic: true,
            },
        ];

        let mut expected_arguments = HashMap::new();
        expected_arguments.insert("arga", vec!["foo"]);
        expected_arguments.insert("argb", vec!["bar"]);
        expected_arguments.insert("argc", Vec::new());

        let pr = ParsedParameters::new(&mut parameters.iter(), &mut commands.iter());
        assert!(pr.is_ok());
        let parsed = pr.ok().unwrap();

        assert!(parsed.command.name == commands[0].name);
        assert!(parsed.arguments.len() == expected_arguments.len());
        assert!(parsed.arguments == expected_arguments);
    }

    #[test]
    fn test_parsed_parameters_construction_variadic_last_argument_many() {
        let parameters: Vec<String> = vec![
            "foobar".to_owned(),
            "--opta=oof".to_owned(),
            "foo".to_owned(),
            "bar".to_owned(),
            "baz".to_owned(),
            "quux".to_owned(),
        ];

        let commands = vec![
            Command {
                name: "foobar".to_owned(),
                help: "foobar".to_owned(),
                options: vec![
                    Option::required("opta", "opta", Some('a'), None),
                ],
                arguments: vec![
                    Argument {
                        name: "arga".to_owned(),
                        help: "arga".to_owned(),
                    },
                    Argument {
                        name: "argb".to_owned(),
                        help: "argb".to_owned(),
                    },
                    Argument {
                        name: "argc".to_owned(),
                        help: "argc".to_owned(),
                    },
                ],
                last_argument_is_variadic: true,
            },
        ];

        let mut expected_arguments = HashMap::new();
        expected_arguments.insert("arga", vec!["foo"]);
        expected_arguments.insert("argb", vec!["bar"]);
        expected_arguments.insert("argc", vec!["baz", "quux"]);

        let pr = ParsedParameters::new(&mut parameters.iter(), &mut commands.iter());
        assert!(pr.is_ok());
        let parsed = pr.ok().unwrap();

        assert!(parsed.command.name == commands[0].name);
        assert!(parsed.arguments.len() == expected_arguments.len());
        assert!(parsed.arguments == expected_arguments);
    }
}
