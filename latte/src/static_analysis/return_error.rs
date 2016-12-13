use std::fmt;

use ast::Ident;

#[derive(Debug)]
pub struct ReturnError {
    func: Ident,
}

impl fmt::Display for ReturnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f,
                 "Not all control paths return a value in function {}",
                 self.func)
    }
}

impl ReturnError {
    pub fn new(ident: &Ident) -> ReturnError {
        ReturnError { func: ident.clone() }
    }
}
