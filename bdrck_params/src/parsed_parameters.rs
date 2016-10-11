use std::collections::HashMap;
use std::env;
use std::iter::Peekable;
use std::option::Option as Optional;
use std::result::Result;
use std::string::String;
use std::vec::Vec;

use super::argument::Argument;
use super::command::Command;
use super::command::ExecutableCommand;
use super::error::*;
use super::option::Option;
use super::option::find_option;

pub fn get_program_parameters() -> Vec<String> {
    //! Returns the current program's parameters (accessed essentialy via
    //! std::env::args()) collected into a Vec. The 0'th parameter (the executable)
    //! is omitted.

    env::args()
        .skip(1) // Skip the first argument, which is our executable.
        .collect()
}

fn build_default_options(parsed: &mut ParsedParameters) {
    //! Constructs maps for options and flags which contain the default values (if
    //! any) for each of the given command's options. Note that all flags have a
    //! default value of false.

    for o in parsed.command.get_options() {
        if let Some(ref dv) = o.default_value {
            parsed.options.insert(&o.name, dv.clone());
        } else if o.is_flag {
            parsed.flags.insert(&o.name, false);
        }
    }
}

/// An option parameter is the string representation of an option, extracted
/// from an iterator over program parameters.
struct OptionParameters<'pl, 'cl> {
    value: Optional<&'pl str>,
    option_obj: &'cl Option,
}

fn extract_option_name_and_value(option_parameter: &str) -> (&str, Optional<&str>) {
    //! Extracts the option name (and, if one is present, the option value) from
    //! the given single option parameter. This involves stripping off the leading
    //! "-" or "--" prefix, and splitting the given parameter on its "=" character
    //! (if any).

    let trimmed: &str = match option_parameter.starts_with("--") {
        false => &option_parameter[1..],
        true => &option_parameter[2..],
    };
    let equals_idx = trimmed.rfind('=');
    let name = equals_idx.map_or(trimmed, |ei| &trimmed[0..ei]);
    let value = equals_idx.map_or(None, |ei| Some(&trimmed[ei + 1..]));

    (name, value)
}

fn next_option_parameters<'pl, 'cl, PI, OI>
    (parameters: &mut Peekable<PI>,
     options: OI)
     -> Result<Optional<OptionParameters<'pl, 'cl>>, ParamsError>
    where PI: Iterator<Item = &'pl String>,
          OI: Iterator<Item = &'cl Option>
{
    //! Parse an option name and value from the given iterator's current position.
    //! If the given iterator is already at the end or the string at its current
    //! position is not an option, return None. Otherwise, return either a valid
    //! option or an error.

    let parameter: &'pl str;
    {
        let op: Optional<&&'pl String> = parameters.peek();
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

    // Search for the value in the next parameter, if this option is not a flag.
    // For flags, because explicit values are optional, it is ambiguous whether or
    // not the next parameter is a flag value or an argument.
    if !option_obj.is_flag {
        let next_parameter_is_value: bool = parameters.peek()
            .map_or(false, |v| !v.starts_with("-"));
        if next_parameter_is_value && value.is_none() {
            value = Some(parameters.next().unwrap().as_ref());
        }
    }

    Ok(Some(OptionParameters {
        value: value,
        option_obj: option_obj,
    }))
}

#[derive(Eq, PartialEq)]
struct ParsedOption<'cl, 'pl> {
    name: &'cl str,
    value: Optional<&'pl str>,
    bool_value: Optional<bool>,
}

fn parse_bool(value: &str) -> Result<bool, ParamsError> {
    //! Return the boolean interpretation of a string, or an error if the string
    //! isn't recognized as a valid boolean value.

    match value.trim().to_lowercase().as_ref() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ParamsError { kind: ErrorKind::InvalidBooleanValue { value: value.to_owned() } }),
    }
}

fn parse_option<'pl, 'cl, PI, OI>(parameters: &mut Peekable<PI>,
                                  options: OI)
                                  -> Result<Optional<ParsedOption<'cl, 'pl>>, ParamsError>
    where PI: Iterator<Item = &'pl String>,
          OI: Iterator<Item = &'cl Option>
{
    //! Parse the next option from the given iterator over program parameters. If
    //! there are no more option parameters, returns None. If an option argument is
    //! found but some error occurs, then an error is returned instead.

    let option_parameters: OptionParameters<'pl, 'cl>;
    {
        let opo = try!(next_option_parameters(parameters, options));
        if opo.is_none() {
            return Ok(None);
        }
        option_parameters = opo.unwrap();
    }

    if !option_parameters.option_obj.is_flag && option_parameters.value.is_none() {
        return Err(ParamsError {
            kind: ErrorKind::MissingOptionValue { name: option_parameters.option_obj.name.clone() },
        });
    }

    let bool_value: Optional<bool>;
    {
        if option_parameters.option_obj.is_flag {
            if let Some(v) = option_parameters.value {
                bool_value = Some(try!(parse_bool(v)));
            } else {
                bool_value = Some(true);
            }
        } else {
            bool_value = None;
        }
    }

    Ok(Some(ParsedOption {
        name: option_parameters.option_obj.name.as_str(),
        value: option_parameters.value,
        bool_value: bool_value,
    }))
}

fn parse_all_options<'pl, 'cl, PI>(parameters: &mut Peekable<PI>,
                                   parsed_parameters: &ParsedParameters<'cl>)
                                   -> Result<Vec<ParsedOption<'cl, 'pl>>, ParamsError>
    where PI: Iterator<Item = &'pl String>
{
    //! Call parse_option repeatedly on the given iterator until an error is
    //! encountered or there are no more options to parse. Returns a possibly empty
    //! vector of parsed options, or an error if one was encountered.

    let mut parsed: Vec<ParsedOption<'cl, 'pl>> = Vec::new();
    while let Some(parsed_option) = try!(parse_option(parameters,
                                                      parsed_parameters.command
                                                          .get_options()
                                                          .iter())) {
        parsed.push(parsed_option);
    }
    Ok(parsed)
}

fn emplace_all_options<'pl, 'cl, PI>(parameters: &mut Peekable<PI>,
                                     parsed_parameters: &mut ParsedParameters<'cl>)
                                     -> Result<(), ParamsError>
    where PI: Iterator<Item = &'pl String>
{
    //! Calls parse_all_options, and adds the result to the given parsed parameters
    //! structure. An error is returned if one is encountered, and the parsed
    //! parameters structure is not modified.

    for parsed_option in &try!(parse_all_options(parameters, parsed_parameters)) {
        if parsed_option.bool_value.is_none() {
            parsed_parameters.options
                .insert(parsed_option.name, parsed_option.value.unwrap().to_owned());
        } else {
            parsed_parameters.flags.insert(parsed_option.name, parsed_option.bool_value.unwrap());
        }
    }

    Ok(())
}

fn parse_all_arguments<'pl, 'cl, PI>(parameters: &mut Peekable<PI>,
                                     arguments: &'cl Vec<Argument>,
                                     last_argument_is_variadic: bool)
                                     -> Result<HashMap<&'cl str, Vec<String>>, ParamsError>
    where PI: Iterator<Item = &'pl String>
{
    //! Parses all of the positional arguments from the given iterator over program
    //! parameters, and returns either a possibly empty HashMap of parsed arguments
    //! or an error, if one is encountered.

    let mut parsed: HashMap<&'cl str, Vec<String>> = HashMap::new();
    if arguments.is_empty() {
        return Ok(parsed);
    }

    if arguments.len() >= 2 {
        for argument in &arguments[..arguments.len() - 1] {
            let v = parameters.next().or(argument.default_value
                .as_ref()
                .map(|dv| dv.first())
                .map_or(None, |dv| Some(dv.unwrap())));
            if v.is_none() {
                return Err(ParamsError {
                    kind: ErrorKind::MissingArgumentValue { name: argument.name.clone() },
                });
            }
            parsed.insert(argument.name.as_str(), vec![v.unwrap().clone()]);
        }
    }

    let last_argument: &Argument = &arguments[arguments.len() - 1];
    let mut last_argument_values: Vec<String> = parameters.map(|v| v.clone()).collect();
    if last_argument_values.is_empty() && last_argument.default_value.is_some() {
        last_argument_values =
            last_argument.default_value.as_ref().unwrap().iter().map(|v| v.clone()).collect();
    }
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

    Ok(parsed)
}

fn emplace_all_arguments<'pl, 'cl, PI>(parameters: &mut Peekable<PI>,
                                       parsed_parameters: &mut ParsedParameters<'cl>)
                                       -> Result<(), ParamsError>
    where PI: Iterator<Item = &'pl String>
{
    //! Parses all of the positional arguments from the given iterator over program
    //! parameters, and adds the result to the given parsed parameters structure.
    //! An error is returned if one is encountered, and the parsed parameters
    //! structure is not modified.

    parsed_parameters.arguments =
        try!(parse_all_arguments(parameters,
                                 parsed_parameters.command.get_arguments(),
                                 parsed_parameters.command.last_argument_is_variadic()));

    Ok(())
}

fn all_options_are_present(parsed: &ParsedParameters) -> Result<(), ParamsError> {
    //! Checks if all of the given command's options are present in the given map
    //! of option names to values. If an option is missing, returns an error with
    //! more detailed information. Otherwise, returns None.

    for o in parsed.command.get_options() {
        if o.is_optional || o.is_flag {
            continue;
        }

        if parsed.get_options().get(o.name.as_str()).is_none() {
            return Err(ParamsError {
                kind: ErrorKind::MissingOptionValue { name: o.name.clone() },
            });
        }
    }

    Ok(())
}

pub fn parse_command<'pl, 'cl, PI, CI>(parameters: &mut Peekable<PI>,
                                       commands: &mut CI)
                                       -> Result<&'cl Command, ParamsError>
    where PI: Iterator<Item = &'pl String>,
          CI: Iterator<Item = &'cl Command>
{
    //! Look up by name the command indicated by the first element of the given
    //! range of program parameters. If a matching command could not be found,
    //! return None instead.

    if let Some(command_parameter) = parameters.next() {
        return commands.find(|&command| *command.get_name() == **command_parameter)
            .map_or(Err(ParamsError {
                        kind: ErrorKind::UnrecognizedCommand { name: (*command_parameter).clone() },
                    }),
                    |command| Ok(command));
    }

    Err(ParamsError { kind: ErrorKind::NoCommandSpecified })
}

/// This structure encapsulates the output from parsing the program's parameters
/// according to a Command. It provides accessor functions to retrieve the
/// values conveniently.
pub struct ParsedParameters<'cl> {
    command: &'cl Command,
    options: HashMap<&'cl str, String>,
    flags: HashMap<&'cl str, bool>,
    arguments: HashMap<&'cl str, Vec<String>>,
}

impl<'cl> ParsedParameters<'cl> {
    pub fn new<'pl, PI>(command: &'cl Command,
                        parameters: &mut Peekable<PI>)
                        -> Result<ParsedParameters<'cl>, ParamsError>
        where PI: Iterator<Item = &'pl String>
    {
        //! Construct a new ParsedParameters instance by parsing the command,
        //! options, flags, and arguments from the given iterator over the set
        //! of program parameters.

        let mut parsed = ParsedParameters {
            command: command,
            options: HashMap::new(),
            flags: HashMap::new(),
            arguments: HashMap::new(),
        };

        build_default_options(&mut parsed);
        try!(emplace_all_options(parameters, &mut parsed));
        try!(emplace_all_arguments(parameters, &mut parsed));
        try!(all_options_are_present(&parsed));

        Ok(parsed)
    }

    pub fn get_command(&self) -> &Command { self.command }
    pub fn get_options(&self) -> &HashMap<&'cl str, String> { &self.options }

    pub fn execute<'cbl>(&self, executable_command: &mut ExecutableCommand<'cl, 'cbl>) {
        executable_command.execute(&self.options, &self.flags, &self.arguments);
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::iter::Peekable;
    use std::option::Option as Optional;
    use std::string::String;
    use std::vec::Vec;
    use super::ParsedOption;
    use super::ParsedParameters;

    use super::build_default_options;
    use super::parse_command;
    use super::parse_option;
    use super::super::argument::Argument;
    use super::super::command::Command;
    use super::super::error::*;
    use super::super::option::Option;

    fn build_command_for_test(name: &str) -> Command {
        Command::new(name.to_owned(),
                     name.to_owned(),
                     Vec::new(),
                     Vec::new(),
                     false)
            .ok()
            .unwrap()
    }

    fn parse_command_works(program_parameters: &Vec<String>,
                           commands: &Vec<Command>,
                           expected_name: &str)
                           -> bool {
        return parse_command(&mut program_parameters.iter().peekable(),
                             &mut commands.iter())
            .ok()
            .map_or(false, |c| *c.get_name() == expected_name);
    }

    fn parse_command_and_parameters<'a, PI, CI>(parameters: &mut Peekable<PI>,
                                                commands: &mut CI)
                                                -> Result<ParsedParameters<'a>, ParamsError>
        where PI: Iterator<Item = &'a String>,
              CI: Iterator<Item = &'a Command>
    {
        let command = try!(parse_command(parameters, commands));
        ParsedParameters::new(command, parameters)
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

        assert!(parse_command(&mut program_parameters.iter().peekable(),
                              &mut commands.iter())
            .ok()
            .is_none());
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
        let command = Command::new("test".to_owned(),
                                   "test".to_owned(),
                                   vec![
                Option::required("a", "a", None, None),
                Option::required("b", "b", None, Some("b")),
                Option::required("c", "c", Some('c'), None),
                Option::required("d", "d", Some('d'), Some("d")),
                Option::optional("e", "e", None),
                Option::optional("f", "f", Some('f')),
                Option::flag("g", "g", None),
                Option::flag("h", "h", Some('h')),
            ],
                                   Vec::new(),
                                   false)
            .ok()
            .unwrap();

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
                            Result<Optional<ParsedOption<'a, 'a>>, ParamsError>);
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
            Command::new(
                "foobar".to_owned(),
                "foobar".to_owned(),
                vec![
                    Option::required("opta", "opta", Some('a'), None),
                    Option::required("optb", "optb", Some('b'), Some("baz")),
                    Option::optional("optc", "optc", Some('c')),
                    Option::optional("optd", "optd", Some('d')),
                    Option::flag("opte", "opte", Some('e')),
                    Option::flag("optf", "optf", Some('f')),
                    Option::flag("optg", "optg", Some('g')),
                ],
                Vec::new(),
                false
            ).ok().unwrap(),
        ];

        let mut expected_options = HashMap::new();
        expected_options.insert("opta", "foo".to_owned());
        expected_options.insert("optb", "baz".to_owned());
        expected_options.insert("optc", "bar".to_owned());

        let mut expected_flags = HashMap::new();
        expected_flags.insert("opte", false);
        expected_flags.insert("optf", true);
        expected_flags.insert("optg", false);

        let pr = parse_command_and_parameters(&mut parameters.iter().peekable(),
                                              &mut commands.iter());
        assert!(pr.is_ok());
        let parsed = pr.ok().unwrap();

        assert!(*parsed.command.get_name() == *commands[0].get_name());
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
            Command::new(
                "foobar".to_owned(),
                "foobar".to_owned(),
                vec![
                    Option::required("opta", "opta", Some('a'), None),
                ],
                vec![
                    Argument {
                        name: "arga".to_owned(),
                        help: "arga".to_owned(),
                        default_value: None,
                    },
                    Argument {
                        name: "argb".to_owned(),
                        help: "argb".to_owned(),
                        default_value: None,
                    },
                    Argument {
                        name: "argc".to_owned(),
                        help: "argc".to_owned(),
                        default_value: None,
                    },
                ],
                false
            ).ok().unwrap(),
        ];

        let mut expected_arguments = HashMap::new();
        expected_arguments.insert("arga", vec!["foo".to_owned()]);
        expected_arguments.insert("argb", vec!["bar".to_owned()]);
        expected_arguments.insert("argc", vec!["baz".to_owned()]);

        let pr = parse_command_and_parameters(&mut parameters.iter().peekable(),
                                              &mut commands.iter());
        assert!(pr.is_ok());
        let parsed = pr.ok().unwrap();

        assert!(*parsed.command.get_name() == *commands[0].get_name());
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
            Command::new(
                "foobar".to_owned(),
                "foobar".to_owned(),
                vec![
                    Option::required("opta", "opta", Some('a'), None),
                ],
                vec![
                    Argument {
                        name: "arga".to_owned(),
                        help: "arga".to_owned(),
                        default_value: None,
                    },
                    Argument {
                        name: "argb".to_owned(),
                        help: "argb".to_owned(),
                        default_value: None,
                    },
                    Argument {
                        name: "argc".to_owned(),
                        help: "argc".to_owned(),
                        default_value: None,
                    },
                ],
                true
            ).ok().unwrap(),
        ];

        let mut expected_arguments = HashMap::new();
        expected_arguments.insert("arga", vec!["foo".to_owned()]);
        expected_arguments.insert("argb", vec!["bar".to_owned()]);
        expected_arguments.insert("argc", Vec::new());

        let pr = parse_command_and_parameters(&mut parameters.iter().peekable(),
                                              &mut commands.iter());
        assert!(pr.is_ok());
        let parsed = pr.ok().unwrap();

        assert!(*parsed.command.get_name() == *commands[0].get_name());
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
            Command::new(
                "foobar".to_owned(),
                "foobar".to_owned(),
                vec![
                    Option::required("opta", "opta", Some('a'), None),
                ],
                vec![
                    Argument {
                        name: "arga".to_owned(),
                        help: "arga".to_owned(),
                        default_value: None,
                    },
                    Argument {
                        name: "argb".to_owned(),
                        help: "argb".to_owned(),
                        default_value: None,
                    },
                    Argument {
                        name: "argc".to_owned(),
                        help: "argc".to_owned(),
                        default_value: None,
                    },
                ],
                true
            ).ok().unwrap(),
        ];

        let mut expected_arguments = HashMap::new();
        expected_arguments.insert("arga", vec!["foo".to_owned()]);
        expected_arguments.insert("argb", vec!["bar".to_owned()]);
        expected_arguments.insert("argc", vec!["baz".to_owned(), "quux".to_owned()]);

        let pr = parse_command_and_parameters(&mut parameters.iter().peekable(),
                                              &mut commands.iter());
        assert!(pr.is_ok());
        let parsed = pr.ok().unwrap();

        assert!(*parsed.command.get_name() == *commands[0].get_name());
        assert!(parsed.arguments.len() == expected_arguments.len());
        assert!(parsed.arguments == expected_arguments);
    }

    #[test]
    fn test_parsed_parameters_default_arguments() {
        let parameters: Vec<String> = vec![
            "foobar".to_owned(),
            "--opta=oof".to_owned(),
            "foo".to_owned(),
        ];

        let commands = vec![
            Command::new(
                "foobar".to_owned(),
                "foobar".to_owned(),
                vec![
                    Option::required("opta", "opta", Some('a'), None),
                ],
                vec![
                    Argument {
                        name: "arga".to_owned(),
                        help: "arga".to_owned(),
                        default_value: Some(vec!["dva".to_owned()]),
                    },
                    Argument {
                        name: "argb".to_owned(),
                        help: "argb".to_owned(),
                        default_value: Some(vec!["dvb".to_owned()]),
                    },
                    Argument {
                        name: "argc".to_owned(),
                        help: "argc".to_owned(),
                        default_value: Some(vec!["dvc".to_owned()]),
                    },
                ],
                false
            ).ok().unwrap(),
        ];

        let mut expected_arguments = HashMap::new();
        expected_arguments.insert("arga", vec!["foo".to_owned()]);
        expected_arguments.insert("argb", vec!["dvb".to_owned()]);
        expected_arguments.insert("argc", vec!["dvc".to_owned()]);

        let pr = parse_command_and_parameters(&mut parameters.iter().peekable(),
                                              &mut commands.iter());
        assert!(pr.is_ok());
        let parsed = pr.ok().unwrap();

        assert!(*parsed.command.get_name() == *commands[0].get_name());
        assert!(parsed.arguments.len() == expected_arguments.len());
        assert!(parsed.arguments == expected_arguments);
    }

    #[test]
    fn test_parsed_parameters_default_arguments_variadic() {
        let parameters: Vec<String> = vec![
            "foobar".to_owned(),
            "--opta=oof".to_owned(),
            "foo".to_owned(),
        ];

        let commands = vec![
            Command::new(
                "foobar".to_owned(),
                "foobar".to_owned(),
                vec![
                    Option::required("opta", "opta", Some('a'), None),
                ],
                vec![
                    Argument {
                        name: "arga".to_owned(),
                        help: "arga".to_owned(),
                        default_value: Some(vec!["dva".to_owned()]),
                    },
                    Argument {
                        name: "argb".to_owned(),
                        help: "argb".to_owned(),
                        default_value: Some(vec!["dvb".to_owned()]),
                    },
                    Argument {
                        name: "argc".to_owned(),
                        help: "argc".to_owned(),
                        default_value: Some(vec!["dvc1".to_owned(), "dvc2".to_owned()]),
                    },
                ],
                true
            ).ok().unwrap(),
        ];

        let mut expected_arguments = HashMap::new();
        expected_arguments.insert("arga", vec!["foo".to_owned()]);
        expected_arguments.insert("argb", vec!["dvb".to_owned()]);
        expected_arguments.insert("argc", vec!["dvc1".to_owned(), "dvc2".to_owned()]);

        let pr = parse_command_and_parameters(&mut parameters.iter().peekable(),
                                              &mut commands.iter());
        assert!(pr.is_ok());
        let parsed = pr.ok().unwrap();

        assert!(*parsed.command.get_name() == *commands[0].get_name());
        assert!(parsed.arguments.len() == expected_arguments.len());
        assert!(parsed.arguments == expected_arguments);
    }

    #[test]
    fn test_parsed_parameters_flag_and_arguments() {
        let parameters: Vec<String> = vec![
            "foobar".to_owned(),
            "--flag".to_owned(),
            "foo".to_owned(),
        ];

        let commands = vec![
            Command::new(
                "foobar".to_owned(),
                "foobar".to_owned(),
                vec![
                    Option::flag("flag", "flag", Some('f')),
                ],
                vec![
                    Argument {
                        name: "arga".to_owned(),
                        help: "arga".to_owned(),
                        default_value: None,
                    },
                ],
                false
            ).ok().unwrap(),
        ];

        let mut expected_flags = HashMap::new();
        expected_flags.insert("flag", true);

        let mut expected_arguments = HashMap::new();
        expected_arguments.insert("arga", vec!["foo".to_owned()]);

        let pr = parse_command_and_parameters(&mut parameters.iter().peekable(),
                                              &mut commands.iter());
        assert!(pr.is_ok());
        let parsed = pr.ok().unwrap();

        assert!(*parsed.command.get_name() == *commands[0].get_name());
        assert!(parsed.options.len() == 0);
        assert!(parsed.flags.len() == expected_flags.len());
        assert!(parsed.flags == expected_flags);
        assert!(parsed.arguments.len() == expected_arguments.len());
        assert!(parsed.arguments == expected_arguments);
    }

    #[test]
    fn test_parsed_parameters_flag_value_and_arguments() {
        let parameters: Vec<String> = vec![
            "foobar".to_owned(),
            "--flag=true".to_owned(),
            "foo".to_owned(),
        ];

        let commands = vec![
            Command::new(
                "foobar".to_owned(),
                "foobar".to_owned(),
                vec![
                    Option::flag("flag", "flag", Some('f')),
                ],
                vec![
                    Argument {
                        name: "arga".to_owned(),
                        help: "arga".to_owned(),
                        default_value: None,
                    },
                ],
                false
            ).ok().unwrap(),
        ];

        let mut expected_flags = HashMap::new();
        expected_flags.insert("flag", true);

        let mut expected_arguments = HashMap::new();
        expected_arguments.insert("arga", vec!["foo".to_owned()]);

        let pr = parse_command_and_parameters(&mut parameters.iter().peekable(),
                                              &mut commands.iter());
        assert!(pr.is_ok());
        let parsed = pr.ok().unwrap();

        assert!(*parsed.command.get_name() == *commands[0].get_name());
        assert!(parsed.options.len() == 0);
        assert!(parsed.flags.len() == expected_flags.len());
        assert!(parsed.flags == expected_flags);
        assert!(parsed.arguments.len() == expected_arguments.len());
        assert!(parsed.arguments == expected_arguments);
    }
}
