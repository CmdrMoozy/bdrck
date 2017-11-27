// Copyright 2015 Axel Rasmussen
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use flags::command::{Command, CommandCallback, CommandResult, ExecutableCommand};
use flags::parse_and_execute::{parse_and_execute, parse_and_execute_command};
use flags::spec::{Spec, Specs};
use flags::value::{Value, Values};
use testing::fn_instrumentation::FnInstrumentation;

fn build_trivial_command_for_test(name: &str) -> Command {
    Command::new(name, name, Specs::new(vec![]).unwrap())
}

type TestCallback = Box<FnMut(Values)>;

fn noop_command_callback(_: Values) -> CommandResult<()> { Ok(()) }

fn parse_and_execute_test_impl(
    parameters: Vec<&str>,
    mut commands: Vec<Command>,
    expected_command_name: &str,
    mut expected_callback: TestCallback,
) {
    let instrumentation = FnInstrumentation::new();
    let callback: CommandCallback<()> = Box::new(|p| -> CommandResult<()> {
        instrumentation.record_call();
        expected_callback(p);
        Ok(())
    });

    let mut expected_command: Option<Command> = None;
    if let Some(expected_command_position) = commands
        .iter()
        .position(|c| c.name == expected_command_name)
    {
        expected_command = Some(commands.remove(expected_command_position));
    }

    let mut executable_commands: Vec<ExecutableCommand<()>> = commands
        .into_iter()
        .map(|c| {
            ExecutableCommand::new(c, Box::new(noop_command_callback))
        })
        .collect();
    if let Some(expected_command) = expected_command {
        executable_commands.push(ExecutableCommand::new(expected_command, callback));
    }

    assert!(instrumentation.get_call_count() == 0);
    let parameters: Vec<String> = parameters.into_iter().map(|p| p.to_owned()).collect();
    let res = parse_and_execute_command::<(), ::std::io::Stderr>(
        "program",
        parameters.as_slice(),
        executable_commands,
        None,
    );
    assert!(res.is_ok(), "{}", res.err().unwrap());
    assert!(res.unwrap().is_ok());
    assert!(instrumentation.get_call_count() == 1);
}

fn into_expected_values(values: Vec<(&'static str, Value)>) -> Values {
    values
        .into_iter()
        .map(|tuple| (tuple.0.to_owned(), tuple.1))
        .collect()
}

#[test]
fn test_parse_and_execute() {
    let expected_vs = into_expected_values(vec![
        ("flaga", Value::Single("quuz".to_owned())),
        ("flagb", Value::Single("baz".to_owned())),
        ("boola", Value::Boolean(false)),
        ("boolb", Value::Boolean(false)),
    ]);

    let instrumentation = FnInstrumentation::new();
    let callback: CommandCallback<()> = Box::new(|vs| {
        instrumentation.record_call();
        assert_eq!(expected_vs, vs);
        Ok(())
    });

    let program = "program".to_owned();
    let args = vec![
        "--flaga=quuz".to_owned(),
        "--flagb".to_owned(),
        "baz".to_owned(),
    ];
    let executable_command = ExecutableCommand::new(
        Command::new(
            "foobar",
            "foobar",
            Specs::new(vec![
                Spec::required("flaga", "flaga", None, None),
                Spec::required("flagb", "flagb", None, Some("oof")),
                Spec::boolean("boola", "boola", None),
                Spec::boolean("boolb", "boolb", None),
                Spec::positional("posa", "posa", None, false).unwrap(),
            ]).unwrap(),
        ),
        callback,
    );

    assert!(instrumentation.get_call_count() == 0);
    assert!(
        parse_and_execute::<(), ::std::io::Stderr>(
            program.as_ref(),
            &args,
            executable_command,
            None
        ).is_ok()
    );
    assert!(instrumentation.get_call_count() == 1);
}

#[test]
#[should_panic(expected = "Unrecognized command 'biff'")]
fn test_parse_invalid_command() {
    parse_and_execute_test_impl(
        vec!["biff", "foo", "bar", "baz"],
        vec![
            build_trivial_command_for_test("foo"),
            build_trivial_command_for_test("bar"),
            build_trivial_command_for_test("baz"),
        ],
        "biff",
        Box::new(|_| {}),
    );
}

#[test]
fn test_parse_command_no_arguments() {
    parse_and_execute_test_impl(
        vec!["bar"],
        vec![
            build_trivial_command_for_test("foo"),
            build_trivial_command_for_test("bar"),
            build_trivial_command_for_test("baz"),
        ],
        "bar",
        Box::new(|vs| {
            assert_eq!(into_expected_values(vec![]), vs);
        }),
    );
}

#[test]
fn test_parse_command_with_unused_arguments() {
    parse_and_execute_test_impl(
        vec!["baz", "foo", "bar", "baz"],
        vec![
            build_trivial_command_for_test("foo"),
            build_trivial_command_for_test("bar"),
            build_trivial_command_for_test("baz"),
        ],
        "baz",
        Box::new(|vs| {
            assert_eq!(into_expected_values(vec![]), vs);
        }),
    );
}

#[test]
fn test_default_values() {
    let expected_vs = into_expected_values(vec![
        ("a", Value::Single("a".to_owned())),
        ("b", Value::Single("b".to_owned())),
        ("e", Value::Boolean(false)),
        ("f", Value::Boolean(false)),
    ]);

    parse_and_execute_test_impl(
        vec!["foo"],
        vec![
            Command::new(
                "foo",
                "foo",
                Specs::new(vec![
                    Spec::required("a", "a", None, Some("a")),
                    Spec::required("b", "b", Some('b'), Some("b")),
                    Spec::optional("c", "c", None),
                    Spec::optional("d", "d", Some('d')),
                    Spec::boolean("e", "e", None),
                    Spec::boolean("f", "f", Some('f')),
                ]).unwrap(),
            ),
        ],
        "foo",
        Box::new(move |vs| {
            assert_eq!(expected_vs, vs);
        }),
    );
}

#[test]
#[should_panic(expected = "Unexpected missing value for flag 'a'")]
fn test_missing_required_flag() {
    parse_and_execute_test_impl(
        vec!["foo"],
        vec![
            Command::new(
                "foo",
                "foo",
                Specs::new(vec![
                    Spec::required("a", "a", None, None),
                    Spec::required("b", "b", Some('b'), Some("b")),
                    Spec::optional("c", "c", None),
                    Spec::optional("d", "d", Some('d')),
                    Spec::boolean("e", "e", None),
                    Spec::boolean("f", "f", Some('f')),
                ]).unwrap(),
            ),
        ],
        "foo",
        Box::new(|_| {}),
    );
}

#[test]
#[should_panic(expected = "Unrecognized flag 'foo'")]
fn test_parse_invalid_flag() {
    parse_and_execute_test_impl(
        vec!["foo", "--foo=bar"],
        vec![Command::new("foo", "foo", Specs::new(vec![]).unwrap())],
        "foo",
        Box::new(|_| {}),
    );
}

#[test]
#[should_panic(expected = "Missing value for flag 'foobar'")]
fn test_parse_missing_flag_value() {
    parse_and_execute_test_impl(
        vec!["foo", "--foobar", "--barbaz"],
        vec![
            Command::new(
                "foo",
                "foo",
                Specs::new(vec![
                    Spec::required("foobar", "foobar", None, None),
                    Spec::required("barbaz", "barbaz", None, None),
                ]).unwrap(),
            ),
        ],
        "foo",
        Box::new(|_| {}),
    );
}

#[test]
fn test_parse_flag_format_variations() {
    let expected_vs = into_expected_values(vec![
        ("flaga", Value::Single("a".to_owned())),
        ("flagb", Value::Single("b".to_owned())),
        ("flagc", Value::Single("c".to_owned())),
        ("flagd", Value::Single("d".to_owned())),
    ]);

    parse_and_execute_test_impl(
        vec!["foo", "--flaga=a", "--b=b", "-flagc", "c", "-d", "d"],
        vec![
            Command::new(
                "foo",
                "foo",
                Specs::new(vec![
                    Spec::required("flaga", "flaga", Some('a'), None),
                    Spec::required("flagb", "flagb", Some('b'), None),
                    Spec::required("flagc", "flagc", Some('c'), None),
                    Spec::required("flagd", "flagd", Some('d'), None),
                ]).unwrap(),
            ),
        ],
        "foo",
        Box::new(move |vs| {
            assert_eq!(expected_vs, vs);
        }),
    );
}

#[test]
fn test_parse_boolean_flag_format_variations() {
    let expected_vs = into_expected_values(vec![
        ("boola", Value::Boolean(true)),
        ("boolb", Value::Boolean(true)),
        ("boolc", Value::Boolean(true)),
        ("boold", Value::Boolean(false)),
    ]);

    parse_and_execute_test_impl(
        vec!["foo", "--boola", "-b", "--boolc=true", "-d=false"],
        vec![
            Command::new(
                "foo",
                "foo",
                Specs::new(vec![
                    Spec::boolean("boola", "boola", Some('a')),
                    Spec::boolean("boolb", "boolb", Some('b')),
                    Spec::boolean("boolc", "boolc", Some('c')),
                    Spec::boolean("boold", "boold", Some('d')),
                ]).unwrap(),
            ),
        ],
        "foo",
        Box::new(move |vs| {
            assert_eq!(expected_vs, vs);
        }),
    );
}

#[test]
fn test_parse_named_flags() {
    let expected_vs = into_expected_values(vec![
        ("flaga", Value::Single("foo".to_owned())),
        ("flagb", Value::Single("defaultb".to_owned())),
        ("flagc", Value::Single("bar".to_owned())),
        ("flagd", Value::Single("baz".to_owned())),
        ("flagf", Value::Boolean(false)),
        ("flagg", Value::Boolean(true)),
        ("flagh", Value::Boolean(false)),
    ]);

    parse_and_execute_test_impl(
        vec![
            "foobar",
            "--flaga",
            "foo",
            "--flagc=bar",
            "--flagd",
            "baz",
            "-g",
            "--h=false",
        ],
        vec![
            Command::new(
                "foobar",
                "foobar",
                Specs::new(vec![
                    Spec::required("flaga", "flaga", Some('a'), None),
                    Spec::required("flagb", "flagb", Some('b'), Some("defaultb")),
                    Spec::required("flagc", "flagc", Some('c'), None),
                    Spec::optional("flagd", "flagd", Some('d')),
                    Spec::optional("flage", "flage", Some('e')),
                    Spec::boolean("flagf", "flagf", Some('f')),
                    Spec::boolean("flagg", "flagg", Some('g')),
                    Spec::boolean("flagh", "flagh", Some('h')),
                ]).unwrap(),
            ),
        ],
        "foobar",
        Box::new(move |vs| {
            assert_eq!(expected_vs, vs);
        }),
    );
}

#[test]
fn test_parse_positional_flags() {
    let expected_vs = into_expected_values(vec![
        ("flaga", Value::Single("oof".to_owned())),
        ("flagb", Value::Single("rab".to_owned())),
        ("flagc", Value::Boolean(true)),
        ("flagd", Value::Boolean(false)),
        ("posa", Value::Repeated(vec!["foo".to_owned()])),
        ("posb", Value::Repeated(vec!["bar".to_owned()])),
        ("posc", Value::Repeated(vec!["baz".to_owned()])),
    ]);

    parse_and_execute_test_impl(
        vec![
            "foobar",
            "--flaga=oof",
            "--flagb",
            "rab",
            "--flagc",
            "--flagd=false",
            "foo",
            "bar",
            "baz",
        ],
        vec![
            Command::new(
                "foobar",
                "foobar",
                Specs::new(vec![
                    Spec::required("flaga", "flaga", Some('a'), None),
                    Spec::required("flagb", "flagb", Some('b'), None),
                    Spec::boolean("flagc", "flagc", Some('c')),
                    Spec::boolean("flagd", "flagd", Some('d')),
                    Spec::positional("posa", "posa", None, false).unwrap(),
                    Spec::positional("posb", "posb", None, false).unwrap(),
                    Spec::positional("posc", "posc", None, false).unwrap(),
                ]).unwrap(),
            ),
        ],
        "foobar",
        Box::new(move |vs| {
            assert_eq!(expected_vs, vs);
        }),
    );
}

#[test]
fn test_parse_variadic_flag_empty() {
    let expected_vs = into_expected_values(vec![
        ("flaga", Value::Single("oof".to_owned())),
        ("posa", Value::Repeated(vec!["foo".to_owned()])),
        ("posb", Value::Repeated(vec!["bar".to_owned()])),
        ("posc", Value::Repeated(vec![])),
    ]);

    parse_and_execute_test_impl(
        vec!["foobar", "--flaga=oof", "foo", "bar"],
        vec![
            Command::new(
                "foobar",
                "foobar",
                Specs::new(vec![
                    Spec::required("flaga", "flaga", Some('a'), None),
                    Spec::positional("posa", "posa", None, false).unwrap(),
                    Spec::positional("posb", "posb", None, false).unwrap(),
                    Spec::positional("posc", "posc", None, true).unwrap(),
                ]).unwrap(),
            ),
        ],
        "foobar",
        Box::new(move |vs| {
            assert_eq!(expected_vs, vs);
        }),
    );
}

#[test]
fn test_parse_variadic_flag_many() {
    let expected_vs = into_expected_values(vec![
        ("flaga", Value::Single("oof".to_owned())),
        ("posa", Value::Repeated(vec!["foo".to_owned()])),
        ("posb", Value::Repeated(vec!["bar".to_owned()])),
        (
            "posc",
            Value::Repeated(vec!["baz".to_owned(), "quux".to_owned()]),
        ),
    ]);

    parse_and_execute_test_impl(
        vec!["foobar", "--flaga=oof", "foo", "bar", "baz", "quux"],
        vec![
            Command::new(
                "foobar",
                "foobar",
                Specs::new(vec![
                    Spec::required("flaga", "flaga", Some('a'), None),
                    Spec::positional("posa", "posa", None, false).unwrap(),
                    Spec::positional("posb", "posb", None, false).unwrap(),
                    Spec::positional("posc", "posc", None, true).unwrap(),
                ]).unwrap(),
            ),
        ],
        "foobar",
        Box::new(move |vs| {
            assert_eq!(expected_vs, vs);
        }),
    );
}

#[test]
fn test_parse_default_positional_values() {
    let expected_vs = into_expected_values(vec![
        ("flaga", Value::Single("oof".to_owned())),
        ("posa", Value::Repeated(vec!["foo".to_owned()])),
        ("posb", Value::Repeated(vec!["dvb".to_owned()])),
        ("posc", Value::Repeated(vec!["dvc".to_owned()])),
    ]);

    parse_and_execute_test_impl(
        vec!["foobar", "--flaga=oof", "foo"],
        vec![
            Command::new(
                "foobar",
                "foobar",
                Specs::new(vec![
                    Spec::required("flaga", "flaga", Some('a'), None),
                    Spec::positional("posa", "posa", Some(&["dva"]), false).unwrap(),
                    Spec::positional("posb", "posb", Some(&["dvb"]), false).unwrap(),
                    Spec::positional("posc", "posc", Some(&["dvc"]), false).unwrap(),
                ]).unwrap(),
            ),
        ],
        "foobar",
        Box::new(move |vs| {
            assert_eq!(expected_vs, vs);
        }),
    );
}

#[test]
fn test_parse_default_variadic_values() {
    let expected_vs = into_expected_values(vec![
        ("flaga", Value::Single("oof".to_owned())),
        ("posa", Value::Repeated(vec!["foo".to_owned()])),
        ("posb", Value::Repeated(vec!["dvb".to_owned()])),
        (
            "posc",
            Value::Repeated(vec!["dvc1".to_owned(), "dvc2".to_owned()]),
        ),
    ]);

    parse_and_execute_test_impl(
        vec!["foobar", "--flaga=oof", "foo"],
        vec![
            Command::new(
                "foobar",
                "foobar",
                Specs::new(vec![
                    Spec::required("flaga", "flaga", Some('a'), None),
                    Spec::positional("posa", "posa", Some(&["dva"]), false).unwrap(),
                    Spec::positional("posb", "posb", Some(&["dvb"]), false).unwrap(),
                    Spec::positional("posc", "posc", Some(&["dvc1", "dvc2"]), true).unwrap(),
                ]).unwrap(),
            ),
        ],
        "foobar",
        Box::new(move |vs| {
            assert_eq!(expected_vs, vs);
        }),
    );
}
