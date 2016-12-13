use ast::*;
use static_analysis::return_error::ReturnError;

pub fn run(p: &Program) -> Result<(), ReturnError> {
    let Program(ref defs) = *p;
    for def in defs {
        if !def.has_return() {
            return Err(ReturnError::new(def.get_ident()));
        }
    }
    Ok(())
}

trait HasReturn {
    fn has_return(&self) -> bool;
}

impl HasReturn for Def {
    fn has_return(&self) -> bool {
        match *self {
            Def::DFunc(_, _, Type::TVoid, _) => true,
            Def::DFunc(_, _, _, ref stmts) => stmts.has_return(),
        }
    }
}

impl HasReturn for Vec<Stmt> {
    fn has_return(&self) -> bool {
        self.iter().any(Stmt::has_return)
    }
}

impl HasReturn for Stmt {
    fn has_return(&self) -> bool {
        match *self {
            Stmt::SReturn |
            Stmt::SReturnE(_) => true,
            Stmt::SBlock(ref stmts) => stmts.has_return(),
            Stmt::SIfElse(_, ref s1, ref s2) => s1.has_return() && s2.has_return(),
            _ => false,
        }
    }
}
