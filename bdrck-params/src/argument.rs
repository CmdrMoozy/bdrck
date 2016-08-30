use std::string::String;

/// An argument is a positional parameter. It must come after any Options
/// the command supports, and can have a default value if it is not
/// specified by the user explicitly.
///
/// The final Argument for a Command can be variadic (that is, it can
/// accept more than one value), but whether or not this is the case is a
/// property of the Command, not of the Argument (because the Argument only
/// stores a description of the argument, not its final value).
#[derive(Debug)]
pub struct Argument {
    pub name: String,
    pub help: String,
}
