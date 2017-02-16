use argument::Argument;
use command::{Command, CommandResult, ExecutableCommand};
use ::error::*;
use help;
use io::get_writer_impl;
use option::Option;
use option::find_option;
use std::collections::HashMap;
use std::env;
use std::iter::Peekable;
use std::option::Option as Optional;
use std::string::String;
use std::vec::Vec;

pub fn get_program_parameters() -> Vec<String> {
    //! Returns the current program's parameters (accessed essentialy via
    //! std::env::args()) collected into a Vec. The 0'th parameter (the executable)
    //! is omitted.

    env::args()
        .skip(1) // Skip the first argument, which is our executable.
        .collect()
}

fn build_default_options(command: &Command, parsed: &mut ParsedParameters) {
    //! Constructs maps for options and flags which contain the default values (if
    //! any) for each of the given command's options. Note that all flags have a
    //! default value of false.

    for o in &command.options {
        if let Some(ref dv) = o.default_value {
            parsed.options.insert(o.name.clone(), dv.clone());
        } else if o.is_flag {
            parsed.flags.insert(o.name.clone(), false);
        }
    }
}

/// An option parameter is the string representation of an option, extracted
/// from an iterator over program parameters.
struct OptionParameters<'cl> {
    value: Optional<String>,
    option_obj: &'cl Option,
}

fn find_option_name_and_value(option_parameter: &str) -> (&str, Optional<&str>) {
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

fn next_option_parameters<'pl, 'cl, PI, OI>(parameters: &mut Peekable<PI>,
                                            options: OI)
                                            -> Result<Optional<OptionParameters<'cl>>>
    where PI: Iterator<Item = &'pl String>,
          OI: Iterator<Item = &'cl Option>
{
    //! Parse an option name and value from the given iterator's current position.
    //! If the given iterator is already at the end or the string at its current
    //! position is not an option, return None. Otherwise, return either a valid
    //! option or an error.

    let parameter: &'pl str = match parameters.peek() {
        Some(p) => {
            match p.starts_with("-") {
                false => return Ok(None),
                true => p,
            }
        },
        None => return Ok(None),
    };
    parameters.next();

    let (name, mut value) = find_option_name_and_value(parameter);

    // Lookup the option by name.
    let option_obj: &Option = match find_option(options, name) {
        Some(oo) => oo,
        None => bail!("Unrecognized option '{}'", name),
    };

    // Search for the value in the next parameter, if this option is not a flag.
    // For flags, because explicit values are optional, it is ambiguous whether or
    // not the next parameter is a flag value or an argument.
    if !option_obj.is_flag {
        let next_parameter_is_value: bool = parameters.peek()
            .map_or(false, |v| !v.starts_with("-"));
        if next_parameter_is_value && value.is_none() {
            value = Some(parameters.next().unwrap().as_str());
        }
    }

    Ok(Some(OptionParameters {
        value: value.map(|v| v.to_owned()),
        option_obj: option_obj,
    }))
}

#[derive(Eq, PartialEq)]
struct ParsedOption {
    name: String,
    value: Optional<String>,
    bool_value: Optional<bool>,
}

fn parse_bool(value: &str) -> Result<bool> {
    //! Return the boolean interpretation of a string, or an error if the string
    //! isn't recognized as a valid boolean value.

    match value.trim().to_lowercase().as_ref() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => bail!("Invalid boolean value '{}'", value),
    }
}

fn parse_option<'pl, 'cl, PI, OI>(parameters: &mut Peekable<PI>,
                                  options: OI)
                                  -> Result<Optional<ParsedOption>>
    where PI: Iterator<Item = &'pl String>,
          OI: Iterator<Item = &'cl Option>
{
    //! Parse the next option from the given iterator over program parameters. If
    //! there are no more option parameters, returns None. If an option argument is
    //! found but some error occurs, then an error is returned instead.

    let option_parameters: OptionParameters<'cl> = match try!(next_option_parameters(parameters,
                                                                                     options)) {
        Some(op) => op,
        None => return Ok(None),
    };

    if !option_parameters.option_obj.is_flag && option_parameters.value.is_none() {
        bail!("No default or specified value for option '{}'",
              option_parameters.option_obj.name);
    }

    let bool_value: Optional<bool> = match option_parameters.option_obj.is_flag {
        false => None,
        true => {
            Some(match option_parameters.value.as_ref() {
                Some(v) => try!(parse_bool(v.as_str())),
                None => true,
            })
        },
    };

    Ok(Some(ParsedOption {
        name: option_parameters.option_obj.name.clone(),
        value: option_parameters.value,
        bool_value: bool_value,
    }))
}

fn parse_all_options<'pl, PI>(command: &Command,
                              parameters: &mut Peekable<PI>)
                              -> Result<Vec<ParsedOption>>
    where PI: Iterator<Item = &'pl String>
{
    //! Call parse_option repeatedly on the given iterator until an error is
    //! encountered or there are no more options to parse. Returns a possibly empty
    //! vector of parsed options, or an error if one was encountered.

    let mut parsed: Vec<ParsedOption> = Vec::new();
    while let Some(parsed_option) = try!(parse_option(parameters, command.options.iter())) {
        parsed.push(parsed_option);
    }
    Ok(parsed)
}

fn emplace_all_options<'pl, PI>(command: &Command,
                                parameters: &mut Peekable<PI>,
                                parsed_parameters: &mut ParsedParameters)
                                -> Result<()>
    where PI: Iterator<Item = &'pl String>
{
    //! Calls parse_all_options, and adds the result to the given parsed parameters
    //! structure. An error is returned if one is encountered, and the parsed
    //! parameters structure is not modified.

    for parsed_option in try!(parse_all_options(command, parameters)) {
        if let Some(v) = parsed_option.bool_value {
            parsed_parameters.flags.insert(parsed_option.name, v);
        } else {
            parsed_parameters.options.insert(parsed_option.name, parsed_option.value.unwrap());
        }
    }

    Ok(())
}

fn parse_all_arguments<'pl, 'cl, PI>(parameters: &mut Peekable<PI>,
                                     arguments: &'cl Vec<Argument>,
                                     last_argument_is_variadic: bool)
                                     -> Result<HashMap<String, Vec<String>>>
    where PI: Iterator<Item = &'pl String>
{
    //! Parses all of the positional arguments from the given iterator over program
    //! parameters, and returns either a possibly empty HashMap of parsed arguments
    //! or an error, if one is encountered.

    let mut parsed: HashMap<String, Vec<String>> = HashMap::new();
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
                bail!("Missing value for argument '{}'", argument.name);
            }
            parsed.insert(argument.name.clone(), vec![v.unwrap().clone()]);
        }
    }

    let last_argument: &Argument = &arguments[arguments.len() - 1];
    let mut last_argument_values: Vec<String> = parameters.map(|v| v.clone()).collect();
    if last_argument_values.is_empty() && last_argument.default_value.is_some() {
        last_argument_values =
            last_argument.default_value.as_ref().unwrap().iter().map(|v| v.clone()).collect();
    }
    if last_argument_is_variadic {
        parsed.insert(last_argument.name.clone(), last_argument_values);
    } else {
        if last_argument_values.len() != 1 {
            bail!("Wrong number of values ({}) for argument '{}'",
                  last_argument_values.len(),
                  last_argument.name);
        }
        parsed.insert(last_argument.name.clone(), last_argument_values);
    }

    Ok(parsed)
}

fn emplace_all_arguments<'pl, PI>(command: &Command,
                                  parameters: &mut Peekable<PI>,
                                  parsed_parameters: &mut ParsedParameters)
                                  -> Result<()>
    where PI: Iterator<Item = &'pl String>
{
    //! Parses all of the positional arguments from the given iterator over program
    //! parameters, and adds the result to the given parsed parameters structure.
    //! An error is returned if one is encountered, and the parsed parameters
    //! structure is not modified.

    parsed_parameters.arguments = try!(parse_all_arguments(parameters,
                                                           &command.arguments,
                                                           command.last_argument_is_variadic));

    Ok(())
}

fn all_options_are_present(command: &Command, options: &HashMap<String, String>) -> Result<()> {
    //! Checks if all of the given command's options are present in the given map
    //! of option names to values. If an option is missing, returns an error with
    //! more detailed information. Otherwise, returns None.

    for o in &command.options {
        if o.is_optional || o.is_flag {
            continue;
        }

        if options.get(o.name.as_str()).is_none() {
            bail!("No default or specified value for option '{}'", o.name);
        }
    }

    Ok(())
}

pub fn parse_command<'pl, 'cbl, PI, E>(program: &str,
                                       parameters: &mut Peekable<PI>,
                                       mut commands: Vec<ExecutableCommand<'cbl, E>>,
                                       print_program_help: bool)
                                       -> Result<ExecutableCommand<'cbl, E>>
    where PI: Iterator<Item = &'pl String>
{
    //! Look up by name the command indicated by the first element of the given
    //! range of program parameters. If a matching command could not be found,
    //! return None instead.

    let idx: Result<usize> = match parameters.next() {
        Some(command_parameter) => {
            match commands.iter().position(|command| command.command.name == *command_parameter) {
                Some(command) => Ok(command),
                None => Err(format!("Unrecognized command '{}'", command_parameter).into()),
            }
        },
        None => Err("No command specified".into()),
    };

    if let Err(e) = idx {
        if print_program_help {
            try!(help::print_program_help(&mut get_writer_impl(), program, &commands));
        }
        return Err(e);
    }

    Ok(commands.remove(idx.unwrap()))
}

/// This structure encapsulates the output from parsing the program's parameters
/// according to a Command. It provides accessor functions to retrieve the
/// values conveniently.
pub struct ParsedParameters {
    options: HashMap<String, String>,
    flags: HashMap<String, bool>,
    arguments: HashMap<String, Vec<String>>,
}

impl ParsedParameters {
    pub fn new<'pl, PI>(command: &Command,
                        parameters: &mut Peekable<PI>)
                        -> Result<ParsedParameters>
        where PI: Iterator<Item = &'pl String>
    {
        //! Construct a new ParsedParameters instance by parsing the command,
        //! options, flags, and arguments from the given iterator over the set
        //! of program parameters.

        let mut parsed = ParsedParameters {
            options: HashMap::new(),
            flags: HashMap::new(),
            arguments: HashMap::new(),
        };

        build_default_options(command, &mut parsed);
        try!(emplace_all_options(command, parameters, &mut parsed));
        try!(emplace_all_arguments(command, parameters, &mut parsed));
        try!(all_options_are_present(command, &parsed.options));

        Ok(parsed)
    }

    pub fn execute<'cbl, E>(self, mut command: ExecutableCommand<'cbl, E>) -> CommandResult<E> {
        command.execute(self.options, self.flags, self.arguments)
    }
}
