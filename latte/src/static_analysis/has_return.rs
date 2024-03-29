use ast::*;

pub trait HasReturn {
    fn has_return(&self) -> bool;
}

impl HasReturn for Func {
    fn has_return(&self) -> bool {
        self.ret_type == Type::TVoid || self.body.has_return()
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
