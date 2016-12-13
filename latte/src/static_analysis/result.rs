use std::{self, fmt};

use static_analysis::type_error::TypeError;
use static_analysis::return_error::ReturnError;

pub type Result = std::result::Result<(), Error>;

#[derive(Debug)]
pub enum Error {
    Type(TypeError),
    Return(ReturnError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Type(ref e) => writeln!(f, "Error (typechecker):\n{}", e),
            Error::Return(ref e) => writeln!(f, "Error (returns):\n{}", e),
        }
    }
}
