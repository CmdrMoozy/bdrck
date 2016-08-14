use std::option::Option as Optional;
use std::string::String;

/// An option is a non-positional parameter to a command. Options can
/// either be normal options or flags. Normal options must be passed by
/// name along with a value. Flags are options whose value is either true
/// or false, and is false by default. Passing a flag by name means
/// flipping its value to true.
#[derive(Debug)]
pub struct Option {
    pub name: String,
    pub help: String,
    pub short_name: Optional<char>,
    pub default_value: Optional<String>,
    pub is_optional: bool,
    pub is_flag: bool,
}

impl Option {
    pub fn required(name: &str,
                    help: &str,
                    short_name: Optional<char>,
                    default_value: Optional<&str>)
                    -> Option {
        //! Constructs a required option. This option may have a default value.
        //! But, importantly, it will always have some value inside the command
        //! function.

        return Option {
            name: name.to_string(),
            help: help.to_string(),
            short_name: short_name,
            default_value: default_value.map(|dv| dv.to_string()),
            is_optional: false,
            is_flag: false,
        };
    }

    pub fn optional(name: &str, help: &str, short_name: Optional<char>) -> Option {
        //! Construct an optional value. This option does not have a default
        //! value, and it may have no value to access inside the command function.

        return Option {
            name: name.to_string(),
            help: help.to_string(),
            short_name: short_name,
            default_value: None,
            is_optional: true,
            is_flag: false,
        };
    }

    pub fn flag(name: &str, help: &str, short_name: Optional<char>) -> Option {
        //! Construct a flag option. This option's value is either true or false,
        //! and is false unless it is explicitly passed to the command as an
        //! argument.

        return Option {
            name: name.to_string(),
            help: help.to_string(),
            short_name: short_name,
            default_value: None,
            is_optional: false,
            is_flag: true,
        };
    }
}

pub fn find_option<'a, I>(options: I, name: &str) -> Optional<&'a Option>
    where I: Iterator<Item = &'a Option>
{
    //! Given an iterator over a collection of Options, locate the option with the
    //! given name (which can be either a short name or a long name). If no such
    //! Option is found, returns None instead.

    let mut result: Optional<&'a Option> = None;
    for o in options {
        if o.name == name {
            result = Some(o);
            break;
        } else if let Some(sn) = o.short_name {
            if result.is_none() && name.starts_with(sn) {
                result = Some(o);
            }
        }
    }
    return result;
}

#[cfg(test)]
mod test {
    use super::find_option;
    use super::Option;

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
}
