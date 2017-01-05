extern crate bdrck_test;
use self::bdrck_test::fn_instrumentation::FnInstrumentation;

extern crate bdrck_params;
use self::bdrck_params::parse_and_execute::parse_and_execute;
use self::bdrck_params::parse_and_execute::parse_and_execute_command;
use self::bdrck_params::argument::Argument;
use self::bdrck_params::command::{Command, CommandCallback};
use self::bdrck_params::command::ExecutableCommand;
use self::bdrck_params::option::Option;

#[test]
fn test_parse_and_execute_command() {
    let instrumentation = FnInstrumentation::new();
    let callback: CommandCallback<()> = Box::new(|options, flags, arguments| {
        instrumentation.record_call();

        assert!(options.len() == 2);
        assert!(flags.len() == 2);
        assert!(arguments.len() == 1);

        Ok(())
    });

    let program = "program".to_owned();
    let parameters = vec![
        "foobar".to_owned(),
        "--opta=quuz".to_owned(),
        "--flagb".to_owned(),
        "baz".to_owned(),
    ];
    let executable_commands = vec![
        ExecutableCommand::new(Command::new("foobar",
                "foobar",
                vec![
                    Option::required("opta", "opta", None, None),
                    Option::required("optb", "optb", None, Some("oof")),
                    Option::flag("flaga", "flaga", None),
                    Option::flag("flagb", "flagb", None),
                ],
                vec![
                    Argument::new("arga", "arga", None),
                ],
                false)
                .unwrap(), callback),
    ];

    assert!(instrumentation.get_call_count() == 0);
    assert!(parse_and_execute_command(program.as_ref(), &parameters, executable_commands).is_ok());
    assert!(instrumentation.get_call_count() == 1);
}

#[test]
fn test_parse_and_execute() {
    let instrumentation = FnInstrumentation::new();
    let callback: CommandCallback<()> = Box::new(|options, flags, arguments| {
        instrumentation.record_call();

        assert!(options.len() == 2);
        assert!(flags.len() == 2);
        assert!(arguments.len() == 1);

        Ok(())
    });

    let program = "program".to_owned();
    let parameters = vec![
        "--opta=quuz".to_owned(),
        "--flagb".to_owned(),
        "baz".to_owned(),
    ];
    let executable_command = ExecutableCommand::new(Command::new("foobar",
                               "foobar",
                               vec![
            Option::required("opta", "opta", None, None),
            Option::required("optb", "optb", None, Some("oof")),
            Option::flag("flaga", "flaga", None),
            Option::flag("flagb", "flagb", None),
        ],
                               vec![
                                   Argument::new("arga", "arga", None),
                            ],
                               false)
        .unwrap(), callback);

    assert!(instrumentation.get_call_count() == 0);
    assert!(parse_and_execute(program.as_ref(), &parameters, executable_command).is_ok());
    assert!(instrumentation.get_call_count() == 1);
}
