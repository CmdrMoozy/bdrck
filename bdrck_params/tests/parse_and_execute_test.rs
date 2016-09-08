use std::collections::HashMap;

extern crate bdrck_test;
use self::bdrck_test::fn_instrumentation::FnInstrumentation;

extern crate bdrck_params;
use self::bdrck_params::parse_and_execute::EXIT_SUCCESS;
use self::bdrck_params::parse_and_execute::parse_and_execute;
use self::bdrck_params::parse_and_execute::parse_and_execute_command;
use self::bdrck_params::argument::Argument;
use self::bdrck_params::command::Command;
use self::bdrck_params::command::ExecutableCommand;
use self::bdrck_params::option::Option;

#[test]
fn test_parse_and_execute_command() {
    let instrumentation = FnInstrumentation::new();
    let callback: Box<FnMut(&HashMap<&str, String>,
                            &HashMap<&str, bool>,
                            &HashMap<&str, Vec<String>>)> =
        Box::new(|options, flags, arguments| {
            instrumentation.record_call();

            assert!(options.len() == 2);
            assert!(flags.len() == 2);
            assert!(arguments.len() == 1);
        });

    let program = "program".to_owned();
    let parameters = vec![
        "foobar".to_owned(),
        "--opta=quuz".to_owned(),
        "--flagb".to_owned(),
        "baz".to_owned(),
    ];
    let commands = vec![
        Command::new("foobar".to_owned(),
                "foobar".to_owned(),
                vec![
                    Option::required("opta", "opta", None, None),
                    Option::required("optb", "optb", None, Some("oof")),
                    Option::flag("flaga", "flaga", None),
                    Option::flag("flagb", "flagb", None),
                ],
                vec![Argument {
                    name: "arga".to_owned(),
                    help: "arga".to_owned(),
                    default_value: None,
                }],
                false)
                .unwrap(),
    ];
    let mut executable_commands = vec![
        ExecutableCommand::new(&commands[0], callback),
    ];

    assert!(instrumentation.get_call_count() == 0);
    assert!(parse_and_execute_command(program.as_ref(),
                                      &parameters,
                                      &mut executable_commands) ==
            EXIT_SUCCESS);
    assert!(instrumentation.get_call_count() == 1);
}

#[test]
fn test_parse_and_execute() {
    let instrumentation = FnInstrumentation::new();
    let callback: Box<FnMut(&HashMap<&str, String>,
                            &HashMap<&str, bool>,
                            &HashMap<&str, Vec<String>>)> =
        Box::new(|options, flags, arguments| {
            instrumentation.record_call();

            assert!(options.len() == 2);
            assert!(flags.len() == 2);
            assert!(arguments.len() == 1);
        });

    let program = "program".to_owned();
    let parameters = vec![
        "--opta=quuz".to_owned(),
        "--flagb".to_owned(),
        "baz".to_owned(),
    ];
    let command = Command::new("foobar".to_owned(),
                               "foobar".to_owned(),
                               vec![
            Option::required("opta", "opta", None, None),
            Option::required("optb", "optb", None, Some("oof")),
            Option::flag("flaga", "flaga", None),
            Option::flag("flagb", "flagb", None),
        ],
                               vec![Argument {
                                        name: "arga".to_owned(),
                                        help: "arga".to_owned(),
                                        default_value: None,
                                    }],
                               false)
        .unwrap();
    let executable_command = ExecutableCommand::new(&command, callback);

    assert!(instrumentation.get_call_count() == 0);
    assert!(parse_and_execute(program.as_ref(), parameters, executable_command) ==
            EXIT_SUCCESS);
    assert!(instrumentation.get_call_count() == 1);
}
