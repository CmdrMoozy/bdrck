extern crate chrono;
#[macro_use]
extern crate log;

mod cli;
mod debug;

pub use cli::init_cli_logger;
pub use debug::init_debug_logger;
