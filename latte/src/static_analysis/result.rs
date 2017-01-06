use std::{self, fmt};

use static_analysis::type_error::TypeError;
use static_analysis::return_error::ReturnError;

pub type Result = std::result::Result<(), Error>;

#[derive(Debug)]
pub enum Error {
    Class(String),
    Type(TypeError),
    Return(ReturnError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Class(ref e) => write!(f, "Error (class hierarchy):\n{}", e),
            Error::Type(ref e) => write!(f, "Error (typechecker):\n{}", e),
            Error::Return(ref e) => write!(f, "Error (returns):\n{}", e),
        }
    }
}
