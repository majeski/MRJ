use std::collections::HashSet;
use std::ops::Deref;

use ast::*;

type VarSet = HashSet<String>;

pub fn check_vars(program: &Program) -> Result<(), String> {
    let mut vars = VarSet::new();
    program.check_vars(&mut vars)
}

impl Program {
    fn check_vars(&self, vars: &mut VarSet) -> Result<(), String> {
        for stmt in &self.0 {
            try!(stmt.check_vars(vars));
        }
        Ok(())
    }
}

impl Stmt {
    fn check_vars(&self, vars: &mut VarSet) -> Result<(), String> {
        match *self {
            Stmt::Assign(ref name, ref e) => {
                vars.insert(name.clone());
                e.check_vars(vars)
            },
            Stmt::Expr(ref e) => e.check_vars(vars)
        }
    }
}

impl Expr {
    fn check_vars(&self, vars: &mut VarSet) -> Result<(), String> {
        match *self {
            Expr::BinOp(ref lhs, _, ref rhs) => {
                try!(lhs.deref().check_vars(vars));            
                try!(rhs.deref().check_vars(vars));
                Ok(())
            },
            Expr::Ident(ref name) => {
                if !vars.contains(name) {
                    Err(name.clone())
                } else {
                    Ok(())
                }
            }
            _ => Ok(())
        }
    }
}
