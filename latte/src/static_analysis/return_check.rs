use ast::*;

use static_analysis::has_return::*;
use static_analysis::return_error::ReturnError;

pub fn run(p: &Program) -> Result<(), ReturnError> {
    for def in &p.0 {
        match *def {
            Def::DFunc(ref f) => {
                if !f.has_return() {
                    return Err(ReturnError::function(&f.ident));
                }
            }
            Def::DClass(ref c) => {
                for m in &c.methods {
                    if !m.has_return() {
                        return Err(ReturnError::method(&c.name, &m.ident));
                    }
                }
            }
        };
    }
    Ok(())
}
