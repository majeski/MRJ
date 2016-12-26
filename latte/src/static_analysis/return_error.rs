use std::fmt;

use ast::Ident;

#[derive(Debug)]
pub struct ReturnError {
    class: Option<Ident>,
    func: Ident,
}

impl fmt::Display for ReturnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f,
                 "Not all control paths return a value in {}",
                 self.get_place())
    }
}

impl ReturnError {
    pub fn function(ident: &Ident) -> ReturnError {
        ReturnError {
            class: None,
            func: ident.clone(),
        }
    }

    pub fn method(class_name: &Ident, ident: &Ident) -> ReturnError {
        ReturnError {
            class: Some(class_name.clone()),
            func: ident.clone(),
        }
    }

    fn get_place(&self) -> String {
        match self.class {
            Some(ref c) => format!("class {}, method {}", c, self.func),
            None => format!("function {}", self.func),
        }
    }
}
