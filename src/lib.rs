mod args;
mod parser;
mod readline;
mod repl;
mod tokenizer;

pub use crate::args::*;
pub use crate::repl::*;

use std::error;
use std::fmt;
use std::result;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("VERSION");

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    ArgNotAvailable,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ArgNotAvailable => write!(f, "argument not available"),
        }
    }
}
