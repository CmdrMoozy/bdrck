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

use crate::error::*;
use crate::flags::command::{Command, CommandCallback, CommandResult};
use crate::flags::parse_and_execute::{parse_and_execute, parse_and_execute_single_command};
use crate::flags::spec::{Spec, Specs};
use crate::testing::fn_instrumentation::FnInstrumentation;
use flags_values::value::{Value, Values};

fn into_expected_values(values: Vec<(&'static str, Value)>) -> Values {
    values
        .into_iter()
        .map(|tuple| (tuple.0.to_owned(), tuple.1))
        .collect()
}

fn build_trivial_test_command(name: &str) -> Command<()> {
    Command::new(
        name,
        name,
        Specs::new(vec![]).unwrap(),
        Box::new(|vs| {
            assert_eq!(into_expected_values(vec![]), vs);
            Ok(())
        }),
    )
}

fn build_test_command(name: &str, specs: Specs, expected_vs: Values) -> Command<()> {
    Command::new(
        name,
        name,
        specs,
        Box::new(move |vs| {
            assert_eq!(expected_vs, vs);
            Ok(())
        }),
    )
}

fn assert_results_match(
    expected: Result<Option<CommandResult<()>>>,
    actual: Result<Option<CommandResult<()>>>,
) {
    fn stringify(r: &::std::result::Result<Option<CommandResult<()>>, String>) -> String {
        match r {
            Ok(r) => match r {
                None => "internally-handled error".to_owned(),
                Some(r) => format!("{:?} from command execution", r),
            },
            Err(e) => format!("internal error {}", e),
        }
    }

    let expected = expected.map_err(|e| format!("{:?}", e));
    let actual = actual.map_err(|e| format!("{:?}", e));

    assert!(
        expected == actual,
        "Expected {}, got {}",
        stringify(&expected),
        stringify(&actual)
    );
}

fn parse_and_execute_result_test_impl(
    args: Vec<&'static str>,
    mut commands: Vec<Command<()>>,
    expected_command_name: &'static str,
    expected_result: Result<Option<CommandResult<()>>>,
) {
    // We need to take the command we expect to execute and modify its callback to
    // record that the function call happened. This has to be done in a certain
    // order to keep the borrow checker happy, while also dealing with the case
    // where the expected command does not exist.
    let instrumentation = FnInstrumentation::new();
    let expected_command: Option<Command<()>> = commands
        .iter()
        .position(|c| c.name == expected_command_name)
        .map(|idx| commands.remove(idx));
    let expected_command_metadata: Option<(String, String, Specs)> = expected_command
        .as_ref()
        .map(|c| (c.name.clone(), c.help.clone(), c.flags.clone()));
    let mut real_callback: Option<CommandCallback<()>> = expected_command.map(|c| c.callback);
    let callback: CommandCallback<()> = Box::new(|vs| {
        instrumentation.record_call();
        if let Some(real_callback) = real_callback.as_mut() {
            real_callback(vs)
        } else {
            Ok(())
        }
    });
    let mut modified_commands: Vec<Command<()>> = vec![];
    if let Some(m) = expected_command_metadata {
        modified_commands.push(Command {
            name: m.0,
            help: m.1,
            flags: m.2,
            callback: callback,
        });
    }
    let commands: Vec<Command<()>> = commands
        .into_iter()
        .chain(modified_commands.into_iter())
        .collect();

    // Actually parse the flags and execute the relevant command, and assert that
    // this worked as expected.
    assert!(instrumentation.get_call_count() == 0);
    let args: Vec<String> = args.into_iter().map(|arg| arg.to_owned()).collect();
    let res =
        parse_and_execute::<(), ::std::io::Stderr>("program", args.as_slice(), commands, None);
    let expected_call_count = match res {
        Err(_) => 0,
        Ok(r) => match r {
            None => 0,
            Some(_) => 1,
        },
    };
    assert_results_match(expected_result, res);
    assert_eq!(expected_call_count, instrumentation.get_call_count());
}

fn parse_and_execute_test_impl(
    args: Vec<&'static str>,
    commands: Vec<Command<()>>,
    expected_command_name: &'static str,
) {
    parse_and_execute_result_test_impl(args, commands, expected_command_name, Ok(Some(Ok(()))));
}

#[test]
fn test_parse_and_execute_single_command() {
    crate::init().unwrap();

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
    let command = Command::new(
        "foobar",
        "foobar",
        Specs::new(vec![
            Spec::required("flaga", "flaga", None, None),
            Spec::required("flagb", "flagb", None, Some("oof")),
            Spec::boolean("boola", "boola", None),
            Spec::boolean("boolb", "boolb", None),
            Spec::positional("posa", "posa", None, false).unwrap(),
        ])
        .unwrap(),
        callback,
    );

    assert!(instrumentation.get_call_count() == 0);
    assert!(parse_and_execute_single_command::<(), ::std::io::Stderr>(
        program.as_ref(),
        &args,
        command,
        None
    )
    .is_ok());
    assert!(instrumentation.get_call_count() == 1);
}

#[test]
fn test_parse_invalid_command() {
    crate::init().unwrap();

    parse_and_execute_result_test_impl(
        vec!["biff", "foo", "bar", "baz"],
        vec![
            build_trivial_test_command("foo"),
            build_trivial_test_command("bar"),
            build_trivial_test_command("baz"),
        ],
        "biff",
        Ok(None),
    );
}

#[test]
fn test_parse_command_no_arguments() {
    crate::init().unwrap();

    parse_and_execute_test_impl(
        vec!["bar"],
        vec![
            build_trivial_test_command("foo"),
            build_trivial_test_command("bar"),
            build_trivial_test_command("baz"),
        ],
        "bar",
    );
}

#[test]
fn test_parse_command_with_unused_arguments() {
    crate::init().unwrap();

    parse_and_execute_test_impl(
        vec!["baz", "foo", "bar", "baz"],
        vec![
            build_trivial_test_command("foo"),
            build_trivial_test_command("bar"),
            build_trivial_test_command("baz"),
        ],
        "baz",
    );
}

#[test]
fn test_default_values() {
    crate::init().unwrap();

    let expected_vs = into_expected_values(vec![
        ("a", Value::Single("a".to_owned())),
        ("b", Value::Single("b".to_owned())),
        ("e", Value::Boolean(false)),
        ("f", Value::Boolean(false)),
    ]);

    parse_and_execute_test_impl(
        vec!["foo"],
        vec![build_test_command(
            "foo",
            Specs::new(vec![
                Spec::required("a", "a", None, Some("a")),
                Spec::required("b", "b", Some('b'), Some("b")),
                Spec::optional("c", "c", None),
                Spec::optional("d", "d", Some('d')),
                Spec::boolean("e", "e", None),
                Spec::boolean("f", "f", Some('f')),
            ])
            .unwrap(),
            expected_vs,
        )],
        "foo",
    );
}

#[test]
fn test_missing_required_flag() {
    crate::init().unwrap();

    parse_and_execute_result_test_impl(
        vec!["foo"],
        vec![build_test_command(
            "foo",
            Specs::new(vec![
                Spec::required("a", "a", None, None),
                Spec::required("b", "b", Some('b'), Some("b")),
                Spec::optional("c", "c", None),
                Spec::optional("d", "d", Some('d')),
                Spec::boolean("e", "e", None),
                Spec::boolean("f", "f", Some('f')),
            ])
            .unwrap(),
            into_expected_values(vec![]),
        )],
        "foo",
        Ok(None),
    );
}

#[test]
fn test_parse_invalid_flag() {
    crate::init().unwrap();

    parse_and_execute_result_test_impl(
        vec!["foo", "--foo=bar"],
        vec![build_test_command(
            "foo",
            Specs::new(vec![]).unwrap(),
            into_expected_values(vec![]),
        )],
        "foo",
        Ok(None),
    );
}

#[test]
fn test_parse_missing_flag_value() {
    crate::init().unwrap();

    parse_and_execute_result_test_impl(
        vec!["foo", "--foobar", "--barbaz"],
        vec![build_test_command(
            "foo",
            Specs::new(vec![
                Spec::required("foobar", "foobar", None, None),
                Spec::required("barbaz", "barbaz", None, None),
            ])
            .unwrap(),
            into_expected_values(vec![]),
        )],
        "foo",
        Ok(None),
    );
}

#[test]
fn test_parse_flag_format_variations() {
    crate::init().unwrap();

    let expected_vs = into_expected_values(vec![
        ("flaga", Value::Single("a".to_owned())),
        ("flagb", Value::Single("b".to_owned())),
        ("flagc", Value::Single("c".to_owned())),
        ("flagd", Value::Single("d".to_owned())),
    ]);

    parse_and_execute_test_impl(
        vec!["foo", "--flaga=a", "--b=b", "-flagc", "c", "-d", "d"],
        vec![build_test_command(
            "foo",
            Specs::new(vec![
                Spec::required("flaga", "flaga", Some('a'), None),
                Spec::required("flagb", "flagb", Some('b'), None),
                Spec::required("flagc", "flagc", Some('c'), None),
                Spec::required("flagd", "flagd", Some('d'), None),
            ])
            .unwrap(),
            expected_vs,
        )],
        "foo",
    );
}

#[test]
fn test_parse_boolean_flag_format_variations() {
    crate::init().unwrap();

    let expected_vs = into_expected_values(vec![
        ("boola", Value::Boolean(true)),
        ("boolb", Value::Boolean(true)),
        ("boolc", Value::Boolean(true)),
        ("boold", Value::Boolean(false)),
    ]);

    parse_and_execute_test_impl(
        vec!["foo", "--boola", "-b", "--boolc=true", "-d=false"],
        vec![build_test_command(
            "foo",
            Specs::new(vec![
                Spec::boolean("boola", "boola", Some('a')),
                Spec::boolean("boolb", "boolb", Some('b')),
                Spec::boolean("boolc", "boolc", Some('c')),
                Spec::boolean("boold", "boold", Some('d')),
            ])
            .unwrap(),
            expected_vs,
        )],
        "foo",
    );
}

#[test]
fn test_parse_named_flags() {
    crate::init().unwrap();

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
        vec![build_test_command(
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
            ])
            .unwrap(),
            expected_vs,
        )],
        "foobar",
    );
}

#[test]
fn test_parse_positional_flags() {
    crate::init().unwrap();

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
        vec![build_test_command(
            "foobar",
            Specs::new(vec![
                Spec::required("flaga", "flaga", Some('a'), None),
                Spec::required("flagb", "flagb", Some('b'), None),
                Spec::boolean("flagc", "flagc", Some('c')),
                Spec::boolean("flagd", "flagd", Some('d')),
                Spec::positional("posa", "posa", None, false).unwrap(),
                Spec::positional("posb", "posb", None, false).unwrap(),
                Spec::positional("posc", "posc", None, false).unwrap(),
            ])
            .unwrap(),
            expected_vs,
        )],
        "foobar",
    );
}

#[test]
fn test_parse_variadic_flag_empty() {
    crate::init().unwrap();

    let expected_vs = into_expected_values(vec![
        ("flaga", Value::Single("oof".to_owned())),
        ("posa", Value::Repeated(vec!["foo".to_owned()])),
        ("posb", Value::Repeated(vec!["bar".to_owned()])),
        ("posc", Value::Repeated(vec![])),
    ]);

    parse_and_execute_test_impl(
        vec!["foobar", "--flaga=oof", "foo", "bar"],
        vec![build_test_command(
            "foobar",
            Specs::new(vec![
                Spec::required("flaga", "flaga", Some('a'), None),
                Spec::positional("posa", "posa", None, false).unwrap(),
                Spec::positional("posb", "posb", None, false).unwrap(),
                Spec::positional("posc", "posc", None, true).unwrap(),
            ])
            .unwrap(),
            expected_vs,
        )],
        "foobar",
    );
}

#[test]
fn test_parse_variadic_flag_many() {
    crate::init().unwrap();

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
        vec![build_test_command(
            "foobar",
            Specs::new(vec![
                Spec::required("flaga", "flaga", Some('a'), None),
                Spec::positional("posa", "posa", None, false).unwrap(),
                Spec::positional("posb", "posb", None, false).unwrap(),
                Spec::positional("posc", "posc", None, true).unwrap(),
            ])
            .unwrap(),
            expected_vs,
        )],
        "foobar",
    );
}

#[test]
fn test_parse_default_positional_values() {
    crate::init().unwrap();

    let expected_vs = into_expected_values(vec![
        ("flaga", Value::Single("oof".to_owned())),
        ("posa", Value::Repeated(vec!["foo".to_owned()])),
        ("posb", Value::Repeated(vec!["dvb".to_owned()])),
        ("posc", Value::Repeated(vec!["dvc".to_owned()])),
    ]);

    parse_and_execute_test_impl(
        vec!["foobar", "--flaga=oof", "foo"],
        vec![build_test_command(
            "foobar",
            Specs::new(vec![
                Spec::required("flaga", "flaga", Some('a'), None),
                Spec::positional("posa", "posa", Some(&["dva"]), false).unwrap(),
                Spec::positional("posb", "posb", Some(&["dvb"]), false).unwrap(),
                Spec::positional("posc", "posc", Some(&["dvc"]), false).unwrap(),
            ])
            .unwrap(),
            expected_vs,
        )],
        "foobar",
    );
}

#[test]
fn test_parse_default_variadic_values() {
    crate::init().unwrap();

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
        vec![build_test_command(
            "foobar",
            Specs::new(vec![
                Spec::required("flaga", "flaga", Some('a'), None),
                Spec::positional("posa", "posa", Some(&["dva"]), false).unwrap(),
                Spec::positional("posb", "posb", Some(&["dvb"]), false).unwrap(),
                Spec::positional("posc", "posc", Some(&["dvc1", "dvc2"]), true).unwrap(),
            ])
            .unwrap(),
            expected_vs,
        )],
        "foobar",
    );
}
