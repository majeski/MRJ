use ast::*;
use static_analysis::has_return::*;

use optimization::optimize::*;

impl Optimize for Vec<Stmt> {
    fn optimize(self) -> Vec<Stmt> {
        let mut res: Vec<Stmt> = Vec::new();
        for stmt in self {
            let stmt = stmt.optimize();
            let has_return = stmt.has_return();
            match stmt {
                Stmt::SEmpty => {}
                _ => res.push(stmt),
            }
            if has_return {
                break;
            }
        }
        res
    }
}

impl Optimize for Stmt {
    fn optimize(self) -> Stmt {
        match self {
            Stmt::SBlock(stmts) => {
                let stmts = stmts.optimize();
                if stmts.len() == 1 {
                    match stmts[0].clone() {
                        Stmt::SBlock(inner_stmts) => Stmt::SBlock(inner_stmts),
                        _ => Stmt::SBlock(stmts),
                    }
                } else {
                    match stmts.is_empty() {
                        true => Stmt::SEmpty,
                        false => Stmt::SBlock(stmts),
                    }
                }
            }
            Stmt::SDecl(t, decls) => {
                Stmt::SDecl(t, decls.into_iter().map(VarDecl::optimize).collect())
            }
            Stmt::SAssign(field, e) => Stmt::SAssign(field, e.optimize()),
            Stmt::SReturnE(e) => Stmt::SReturnE(e.optimize()),
            Stmt::SExpr(e) => Stmt::SExpr(e.optimize()),
            Stmt::SIf(e, iftrue) => {
                let cond = e.optimize();
                match cond {
                    Expr::ELit(Lit::LTrue) => Stmt::SBlock(vec![*iftrue]).optimize(),
                    Expr::ELit(Lit::LFalse) => Stmt::SEmpty,
                    _ => Stmt::SIf(cond, iftrue.optimize()),
                }
            }
            Stmt::SIfElse(e, iftrue, iffalse) => {
                let cond = e.optimize();
                match cond {
                    Expr::ELit(Lit::LTrue) => Stmt::SBlock(vec![*iftrue]).optimize(),
                    Expr::ELit(Lit::LFalse) => Stmt::SBlock(vec![*iffalse]).optimize(),
                    _ => Stmt::SIfElse(cond, iftrue.optimize(), iffalse.optimize()),
                }
            }
            Stmt::SWhile(e, s) => {
                let cond = e.optimize();
                match cond {
                    Expr::ELit(Lit::LFalse) => Stmt::SEmpty,
                    _ => Stmt::SWhile(cond, s.optimize()),
                }
            }
            Stmt::SFor(t, i, e, s) => Stmt::SFor(t, i, e.optimize(), s.optimize()),
            _ => self,
        }
    }
}

impl Optimize for VarDecl {
    fn optimize(self) -> VarDecl {
        match self {
            VarDecl::Init(t, i, e) => VarDecl::Init(t, i, e.optimize()),
            _ => self,
        }
    }
}
