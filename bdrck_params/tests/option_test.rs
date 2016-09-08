extern crate bdrck_params;
use self::bdrck_params::option::Option;
use self::bdrck_params::option::find_option;

fn find_option_works(options: &Vec<Option>, query: &str, expected_name: &str) -> bool {
    return find_option(options.iter(), query).map_or(false, |o| o.name == expected_name);
}

#[test]
fn test_find_option() {
    let options = vec![
		Option::required("foo", "", Some('o'), None),
		Option::required("bar", "", Some('r'), None),
		Option::flag("baz", "", Some('z')),
		Option::flag("zab", "", Some('Z')),
		Option::required("rab", "", Some('R'), None),
		Option::required("oof", "", Some('O'), None),
		Option::required("foobar", "", Some('f'), None),
		Option::flag("barbaz", "", Some('b')),
		Option::flag("zabrab", "", Some('B')),
		Option::required("raboof", "", Some('F'), None),
	];

    assert!(find_option_works(&options, "foo", "foo"));
    assert!(find_option_works(&options, "o", "foo"));
    assert!(find_option_works(&options, "bar", "bar"));
    assert!(find_option_works(&options, "r", "bar"));
    assert!(find_option_works(&options, "baz", "baz"));
    assert!(find_option_works(&options, "z", "baz"));
    assert!(find_option_works(&options, "zab", "zab"));
    assert!(find_option_works(&options, "Z", "zab"));
    assert!(find_option_works(&options, "rab", "rab"));
    assert!(find_option_works(&options, "R", "rab"));
    assert!(find_option_works(&options, "oof", "oof"));
    assert!(find_option_works(&options, "O", "oof"));
    assert!(find_option_works(&options, "foobar", "foobar"));
    assert!(find_option_works(&options, "f", "foobar"));
    assert!(find_option_works(&options, "barbaz", "barbaz"));
    assert!(find_option_works(&options, "b", "barbaz"));
    assert!(find_option_works(&options, "zabrab", "zabrab"));
    assert!(find_option_works(&options, "B", "zabrab"));
    assert!(find_option_works(&options, "raboof", "raboof"));
    assert!(find_option_works(&options, "F", "raboof"));

    assert!(!find_option_works(&options, "foo", "bar"));
    assert!(!find_option_works(&options, "syn", "syn"));
    assert!(!find_option_works(&options, "s", "syn"));
    assert!(!find_option_works(&options, "ack", "ack"));
    assert!(!find_option_works(&options, "a", "ack"));
    assert!(!find_option_works(&options, "-", "foobar"));
}
