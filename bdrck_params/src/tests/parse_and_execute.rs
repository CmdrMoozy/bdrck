use argument::Argument;
use command::{Command, CommandCallback, CommandResult, ExecutableCommand};
use io::*;
use option::Option;
use parse_and_execute::{parse_and_execute, parse_and_execute_command};
use std::collections::HashMap;
use std::option::Option as Optional;
use tests::bdrck_test::fn_instrumentation::FnInstrumentation;

fn build_trivial_command_for_test(name: &str) -> Command {
    Command::new(name, name, vec![], vec![], false).unwrap()
}

type TestCallback = Box<FnMut(HashMap<String, String>,
                              HashMap<String, bool>,
                              HashMap<String, Vec<String>>)>;

fn noop_command_callback(_: HashMap<String, String>,
                         _: HashMap<String, bool>,
                         _: HashMap<String, Vec<String>>)
                         -> CommandResult<()> {
    Ok(())
}

fn parse_and_execute_test_impl(parameters: Vec<&str>,
                               mut commands: Vec<Command>,
                               expected_command_name: &str,
                               mut expected_callback: TestCallback) {
    let instrumentation = FnInstrumentation::new();
    let callback: CommandCallback<()> =
        Box::new(|options, flags, arguments| -> CommandResult<()> {
            instrumentation.record_call();
            expected_callback(options, flags, arguments);
            Ok(())
        });

    let mut expected_command: Optional<Command> = None;
    if let Some(expected_command_position) =
        commands.iter().position(|c| c.name == expected_command_name) {
        expected_command = Some(commands.remove(expected_command_position));
    }

    let mut executable_commands: Vec<ExecutableCommand<()>> = commands.into_iter()
        .map(|c| ExecutableCommand::new(c, Box::new(noop_command_callback)))
        .collect();
    if let Some(expected_command) = expected_command {
        executable_commands.push(ExecutableCommand::new(expected_command, callback));
    }

    assert!(instrumentation.get_call_count() == 0);
    let parameters: Vec<String> = parameters.into_iter().map(|p| p.to_owned()).collect();
    let res = parse_and_execute_command("program", parameters.as_slice(), executable_commands);
    assert!(res.is_ok(), "{}", res.err().unwrap());
    assert!(res.unwrap().is_ok());
    assert!(instrumentation.get_call_count() == 1);
}

#[test]
fn test_parse_and_execute() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    let instrumentation = FnInstrumentation::new();
    let callback: CommandCallback<()> = Box::new(|options, flags, arguments| {
        instrumentation.record_call();

        assert!(options.len() == 2);
        assert!(flags.len() == 2);
        assert!(arguments.len() == 1);

        Ok(())
    });

    let program = "program".to_owned();
    let parameters = vec!["--opta=quuz".to_owned(), "--flagb".to_owned(), "baz".to_owned()];
    let executable_command =
        ExecutableCommand::new(Command::new("foobar",
                                            "foobar",
                                            vec![Option::required("opta", "opta", None, None),
                                                 Option::required("optb",
                                                                  "optb",
                                                                  None,
                                                                  Some("oof")),
                                                 Option::flag("flaga", "flaga", None),
                                                 Option::flag("flagb", "flagb", None)],
                                            vec![Argument::new("arga", "arga", None)],
                                            false)
                                   .unwrap(),
                               callback);

    assert!(instrumentation.get_call_count() == 0);
    assert!(parse_and_execute(program.as_ref(), &parameters, executable_command).is_ok());
    assert!(instrumentation.get_call_count() == 1);
}

#[test]
#[should_panic(expected = "Unrecognized command 'biff'")]
fn test_parse_invalid_command() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    parse_and_execute_test_impl(vec!["biff", "foo", "bar", "baz"],
                                vec![build_trivial_command_for_test("foo"),
                                     build_trivial_command_for_test("bar"),
                                     build_trivial_command_for_test("baz")],
                                "biff",
                                Box::new(|_, _, _| {}));
}

#[test]
fn test_parse_command_no_arguments() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    parse_and_execute_test_impl(vec!["bar"],
                                vec![build_trivial_command_for_test("foo"),
                                     build_trivial_command_for_test("bar"),
                                     build_trivial_command_for_test("baz")],
                                "bar",
                                Box::new(|options, flags, arguments| {
                                    assert!(options.len() == 0);
                                    assert!(flags.len() == 0);
                                    assert!(arguments.len() == 0);
                                }));
}

#[test]
fn test_parse_command_with_unused_arguments() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    parse_and_execute_test_impl(vec!["baz", "foo", "bar", "baz"],
                                vec![build_trivial_command_for_test("foo"),
                                     build_trivial_command_for_test("bar"),
                                     build_trivial_command_for_test("baz")],
                                "baz",
                                Box::new(|options, flags, arguments| {
                                    assert!(options.len() == 0);
                                    assert!(flags.len() == 0);
                                    assert!(arguments.len() == 0);
                                }));
}

#[test]
fn test_default_options() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    parse_and_execute_test_impl(vec!["foo"],
                                vec![Command::new("foo",
                                                  "foo",
                                                  vec![Option::required("a",
                                                                        "a",
                                                                        None,
                                                                        Some("a")),
                                                       Option::required("b",
                                                                        "b",
                                                                        Some('b'),
                                                                        Some("b")),
                                                       Option::optional("c", "c", None),
                                                       Option::optional("d", "d", Some('d')),
                                                       Option::flag("e", "e", None),
                                                       Option::flag("f", "f", Some('f'))],
                                                  vec![],
                                                  false)
                                         .unwrap()],
                                "foo",
                                Box::new(|options, flags, arguments| {
        assert!(options.len() == 2);
        assert!(options.get("a").map_or(false, |v| *v == "a"));
        assert!(options.get("b").map_or(false, |v| *v == "b"));
        assert!(options.get("c").is_none());
        assert!(options.get("d").is_none());

        assert!(flags.len() == 2);
        assert!(flags.get("e").map_or(false, |v| *v == false));
        assert!(flags.get("f").map_or(false, |v| *v == false));

        assert!(arguments.len() == 0);
    }));
}

#[test]
#[should_panic(expected = "No default or specified value for option 'a'")]
fn test_missing_required_option() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    parse_and_execute_test_impl(vec!["foo"],
                                vec![Command::new("foo",
                                                  "foo",
                                                  vec![Option::required("a", "a", None, None),
                                                       Option::required("b",
                                                                        "b",
                                                                        Some('b'),
                                                                        Some("b")),
                                                       Option::optional("c", "c", None),
                                                       Option::optional("d", "d", Some('d')),
                                                       Option::flag("e", "e", None),
                                                       Option::flag("f", "f", Some('f'))],
                                                  vec![],
                                                  false)
                                         .unwrap()],
                                "foo",
                                Box::new(|_, _, _| {}));
}

#[test]
#[should_panic(expected = "Unrecognized option 'foo'")]
fn test_parse_invalid_option() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    parse_and_execute_test_impl(vec!["foo", "--foo=bar"],
                                vec![Command::new("foo", "foo", vec![], vec![], false).unwrap()],
                                "foo",
                                Box::new(|_, _, _| {}));
}

#[test]
#[should_panic(expected = "No default or specified value for option 'foobar'")]
fn test_parse_missing_option_value() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    parse_and_execute_test_impl(vec!["foo", "--foobar", "--barbaz"],
                                vec![Command::new("foo",
                                                  "foo",
                                                  vec![Option::required("foobar",
                                                                        "foobar",
                                                                        None,
                                                                        None),
                                                       Option::required("barbaz",
                                                                        "barbaz",
                                                                        None,
                                                                        None)],
                                                  vec![],
                                                  false)
                                         .unwrap()],
                                "foo",
                                Box::new(|_, _, _| {}));
}

#[test]
fn test_parse_option_format_variations() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    let mut expected_options = HashMap::new();
    expected_options.insert("opta".to_owned(), "a".to_owned());
    expected_options.insert("optb".to_owned(), "b".to_owned());
    expected_options.insert("optc".to_owned(), "c".to_owned());
    expected_options.insert("optd".to_owned(), "d".to_owned());

    parse_and_execute_test_impl(vec!["foo", "--opta=a", "--b=b", "-optc", "c", "-d", "d"],
                                vec![Command::new("foo",
                                                  "foo",
                                                  vec![Option::required("opta",
                                                                        "opta",
                                                                        Some('a'),
                                                                        None),
                                                       Option::required("optb",
                                                                        "optb",
                                                                        Some('b'),
                                                                        None),
                                                       Option::required("optc",
                                                                        "optc",
                                                                        Some('c'),
                                                                        None),
                                                       Option::required("optd",
                                                                        "optd",
                                                                        Some('d'),
                                                                        None)],
                                                  vec![],
                                                  false)
                                         .unwrap()],
                                "foo",
                                Box::new(move |options, _, _| {
                                    assert_eq!(expected_options, options);
                                }));
}

#[test]
fn test_parse_flag_format_variations() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    let mut expected_flags = HashMap::new();
    expected_flags.insert("flaga".to_owned(), true);
    expected_flags.insert("flagb".to_owned(), true);
    expected_flags.insert("flagc".to_owned(), true);
    expected_flags.insert("flagd".to_owned(), false);

    parse_and_execute_test_impl(vec!["foo", "--flaga", "-b", "--flagc=true", "-d=false"],
                                vec![Command::new("foo",
                                                  "foo",
                                                  vec![Option::flag("flaga",
                                                                    "flaga",
                                                                    Some('a')),
                                                       Option::flag("flagb",
                                                                    "flagb",
                                                                    Some('b')),
                                                       Option::flag("flagc",
                                                                    "flagc",
                                                                    Some('c')),
                                                       Option::flag("flagd",
                                                                    "flagd",
                                                                    Some('d'))],
                                                  vec![],
                                                  false)
                                         .unwrap()],
                                "foo",
                                Box::new(move |_, flags, _| {
                                    assert_eq!(expected_flags, flags);
                                }));
}

#[test]
fn test_parse_options() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    let mut expected_options = HashMap::new();
    expected_options.insert("opta".to_owned(), "foo".to_owned());
    expected_options.insert("optb".to_owned(), "defaultb".to_owned());
    expected_options.insert("optc".to_owned(), "bar".to_owned());
    expected_options.insert("optd".to_owned(), "baz".to_owned());

    let mut expected_flags = HashMap::new();
    expected_flags.insert("optf".to_owned(), false);
    expected_flags.insert("optg".to_owned(), true);
    expected_flags.insert("opth".to_owned(), false);

    parse_and_execute_test_impl(vec!["foobar",
                                     "--opta",
                                     "foo",
                                     "--optc=bar",
                                     "--optd",
                                     "baz",
                                     "-g",
                                     "--h=false"],
                                vec![Command::new("foobar",
                                                  "foobar",
                                                  vec![Option::required("opta",
                                                                        "opta",
                                                                        Some('a'),
                                                                        None),
                                                       Option::required("optb",
                                                                        "optb",
                                                                        Some('b'),
                                                                        Some("defaultb")),
                                                       Option::required("optc",
                                                                        "optc",
                                                                        Some('c'),
                                                                        None),
                                                       Option::optional("optd",
                                                                        "optd",
                                                                        Some('d')),
                                                       Option::optional("opte",
                                                                        "opte",
                                                                        Some('e')),
                                                       Option::flag("optf", "optf", Some('f')),
                                                       Option::flag("optg", "optg", Some('g')),
                                                       Option::flag("opth", "opth", Some('h'))],
                                                  vec![],
                                                  false)
                                         .unwrap()],
                                "foobar",
                                Box::new(move |options, flags, _| {
                                    assert_eq!(expected_options, options);
                                    assert_eq!(expected_flags, flags);
                                }));
}

#[test]
fn test_parse_arguments() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    let mut expected_options = HashMap::new();
    expected_options.insert("opta".to_owned(), "oof".to_owned());
    expected_options.insert("optb".to_owned(), "rab".to_owned());

    let mut expected_flags = HashMap::new();
    expected_flags.insert("optc".to_owned(), true);
    expected_flags.insert("optd".to_owned(), false);

    let mut expected_arguments = HashMap::new();
    expected_arguments.insert("arga".to_owned(), vec!["foo".to_owned()]);
    expected_arguments.insert("argb".to_owned(), vec!["bar".to_owned()]);
    expected_arguments.insert("argc".to_owned(), vec!["baz".to_owned()]);

    parse_and_execute_test_impl(vec!["foobar",
                                     "--opta=oof",
                                     "--optb",
                                     "rab",
                                     "--optc",
                                     "--optd=false",
                                     "foo",
                                     "bar",
                                     "baz"],
                                vec![Command::new("foobar",
                                                  "foobar",
                                                  vec![Option::required("opta",
                                                                        "opta",
                                                                        Some('a'),
                                                                        None),
                                                       Option::required("optb",
                                                                        "optb",
                                                                        Some('b'),
                                                                        None),
                                                       Option::flag("optc", "optc", Some('c')),
                                                       Option::flag("optd", "optd", Some('d'))],
                                                  vec![Argument::new("arga", "arga", None),
                                                       Argument::new("argb", "argb", None),
                                                       Argument::new("argc", "argc", None)],
                                                  false)
                                         .unwrap()],
                                "foobar",
                                Box::new(move |options, _, arguments| {
                                    assert_eq!(expected_options, options);
                                    assert_eq!(expected_arguments, arguments);
                                }));
}

#[test]
fn test_parse_variadic_last_argument_empty() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    let mut expected_options = HashMap::new();
    expected_options.insert("opta".to_owned(), "oof".to_owned());

    let mut expected_arguments = HashMap::new();
    expected_arguments.insert("arga".to_owned(), vec!["foo".to_owned()]);
    expected_arguments.insert("argb".to_owned(), vec!["bar".to_owned()]);
    expected_arguments.insert("argc".to_owned(), vec![]);

    parse_and_execute_test_impl(vec!["foobar", "--opta=oof", "foo", "bar"],
                                vec![Command::new("foobar",
                                                  "foobar",
                                                  vec![Option::required("opta",
                                                                        "opta",
                                                                        Some('a'),
                                                                        None)],
                                                  vec![Argument::new("arga", "arga", None),
                                                       Argument::new("argb", "argb", None),
                                                       Argument::new("argc", "argc", None)],
                                                  true)
                                         .unwrap()],
                                "foobar",
                                Box::new(move |options, _, arguments| {
                                    assert_eq!(expected_options, options);
                                    assert_eq!(expected_arguments, arguments);
                                }));
}

#[test]
fn test_parse_variadic_last_argument_many() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    let mut expected_options = HashMap::new();
    expected_options.insert("opta".to_owned(), "oof".to_owned());

    let mut expected_arguments = HashMap::new();
    expected_arguments.insert("arga".to_owned(), vec!["foo".to_owned()]);
    expected_arguments.insert("argb".to_owned(), vec!["bar".to_owned()]);
    expected_arguments.insert("argc".to_owned(), vec!["baz".to_owned(), "quux".to_owned()]);

    parse_and_execute_test_impl(vec!["foobar", "--opta=oof", "foo", "bar", "baz", "quux"],
                                vec![Command::new("foobar",
                                                  "foobar",
                                                  vec![Option::required("opta",
                                                                        "opta",
                                                                        Some('a'),
                                                                        None)],
                                                  vec![Argument::new("arga", "arga", None),
                                                       Argument::new("argb", "argb", None),
                                                       Argument::new("argc", "argc", None)],
                                                  true)
                                         .unwrap()],
                                "foobar",
                                Box::new(move |options, _, arguments| {
                                    assert_eq!(expected_options, options);
                                    assert_eq!(expected_arguments, arguments);
                                }));
}

#[test]
fn test_parse_default_arguments() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    let mut expected_options = HashMap::new();
    expected_options.insert("opta".to_owned(), "oof".to_owned());

    let mut expected_arguments = HashMap::new();
    expected_arguments.insert("arga".to_owned(), vec!["foo".to_owned()]);
    expected_arguments.insert("argb".to_owned(), vec!["dvb".to_owned()]);
    expected_arguments.insert("argc".to_owned(), vec!["dvc".to_owned()]);

    parse_and_execute_test_impl(vec!["foobar", "--opta=oof", "foo"],
                                vec![Command::new("foobar",
                                                  "foobar",
                                                  vec![Option::required("opta",
                                                                        "opta",
                                                                        Some('a'),
                                                                        None)],
                                                  vec![
                    Argument::new("arga", "arga", Some(vec!["dva".to_owned()])),
                    Argument::new("argb", "argb", Some(vec!["dvb".to_owned()])),
                    Argument::new("argc", "argc", Some(vec!["dvc".to_owned()])),
                ],
                                                  false)
                                         .unwrap()],
                                "foobar",
                                Box::new(move |options, _, arguments| {
                                    assert_eq!(expected_options, options);
                                    assert_eq!(expected_arguments, arguments);
                                }));
}

#[test]
fn test_parse_default_variadic_arguments() {
    // Do not write any output from unit tests.
    set_writer_impl(WriterImpl::Noop);

    let mut expected_options = HashMap::new();
    expected_options.insert("opta".to_owned(), "oof".to_owned());

    let mut expected_arguments = HashMap::new();
    expected_arguments.insert("arga".to_owned(), vec!["foo".to_owned()]);
    expected_arguments.insert("argb".to_owned(), vec!["dvb".to_owned()]);
    expected_arguments.insert("argc".to_owned(),
                              vec!["dvc1".to_owned(), "dvc2".to_owned()]);

    parse_and_execute_test_impl(vec!["foobar", "--opta=oof", "foo"],
                                vec![Command::new("foobar",
                                                  "foobar",
                                                  vec![Option::required("opta",
                                                                        "opta",
                                                                        Some('a'),
                                                                        None)],
                                                  vec![
                    Argument::new("arga", "arga", Some(vec!["dva".to_owned()])),
                    Argument::new("argb", "argb", Some(vec!["dvb".to_owned()])),
                    Argument::new("argc", "argc", Some(vec!["dvc1".to_owned(), "dvc2".to_owned()])),
                ],
                                                  true)
                                         .unwrap()],
                                "foobar",
                                Box::new(move |options, _, arguments| {
                                    assert_eq!(expected_options, options);
                                    assert_eq!(expected_arguments, arguments);
                                }));
}
