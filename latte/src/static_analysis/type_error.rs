use std::fmt;

use ast::{Type, Operator, Ident};

#[derive(Debug)]
pub struct TypeError {
    err: String,
    stack: Vec<String>,
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.err)?;
        for place in &self.stack {
            writeln!(f, "in:")?;
            write!(f, "{}", place)?;
        }
        Ok(())
    }
}

impl TypeError {
    pub fn wrapped<T: fmt::Display>(mut self, inside: &T) -> TypeError {
        self.stack.push(format!("{}", inside));
        self
    }

    pub fn invalid_type(expected: Type, actual: Type) -> TypeError {
        Self::new(format!("Incorrect type, expected: {}, actual: {}", expected, actual))
    }

    pub fn no_operator(op: Operator, lhs_t: Type, rhs_t: Type) -> TypeError {
        Self::new(format!("No {} operator for types: {} and {}", op, lhs_t, rhs_t))
    }

    // int main()
    pub fn no_main() -> TypeError {
        Self::new(format!("No main function"))
    }

    pub fn invalid_main_type() -> TypeError {
        Self::new(format!("Invalid type of main function"))
    }

    // Identifier
    pub fn undefined(ident: &Ident) -> TypeError {
        Self::new(format!("Undefined identifier: {}", ident))
    }

    pub fn already_defined(ident: &Ident) -> TypeError {
        Self::new(format!("Identifier {} is already defined in the current scope",
                          ident))
    }

    // Declaration
    pub fn void_decl() -> TypeError {
        Self::new(format!("Cannot declare variable with void type"))
    }

    // Function
    pub fn not_a_function(ident: &Ident) -> TypeError {
        Self::new(format!("{} is not a function", ident))
    }

    pub fn invalid_call_arg_num(expected: usize, actual: usize) -> TypeError {
        Self::new(format!("Function expected {} arguments, but got {}",
                          expected,
                          actual))
    }

    pub fn invalid_call_arg_type(nth: usize, expected: &Type, actual: Type) -> TypeError {
        Self::new(format!("Incorrect type for {}th argument, expected: {}, actual: {}",
                          nth + 1,
                          expected,
                          actual))
    }

    // ctor
    fn new(msg: String) -> TypeError {
        TypeError {
            err: msg,
            stack: Vec::new(),
        }
    }
}
