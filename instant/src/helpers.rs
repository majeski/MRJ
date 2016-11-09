use std::collections::HashSet;
use std::ops::Deref;

use ast::*;

type VarSet = HashSet<String>;

pub fn check_vars(program: &Program) -> Result<(), String> {
    let mut vars = VarSet::new();
    program.check_vars(&mut vars)
}

trait CheckVars {
    fn check_vars(&self, vars: &mut VarSet) -> Result<(), String>;
}

impl CheckVars for Program {
    fn check_vars(&self, vars: &mut VarSet) -> Result<(), String> {
        for stmt in &self.0 {
            try!(stmt.check_vars(vars));
        }
        Ok(())
    }
}

impl CheckVars for Stmt {
    fn check_vars(&self, vars: &mut VarSet) -> Result<(), String> {
        match *self {
            Stmt::Assign(ref name, ref e) => {
                try!(e.check_vars(vars));
                vars.insert(name.clone());
                Ok(())
            }
            Stmt::Expr(ref e) => e.check_vars(vars),
        }
    }
}

impl CheckVars for Expr {
    fn check_vars(&self, vars: &mut VarSet) -> Result<(), String> {
        match *self {
            Expr::BinOp(ref lhs, _, ref rhs) => {
                try!(lhs.deref().check_vars(vars));
                try!(rhs.deref().check_vars(vars));
                Ok(())
            }
            Expr::Ident(ref name) => {
                if !vars.contains(name) {
                    Err(name.clone())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}
