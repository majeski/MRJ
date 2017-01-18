use std::collections::HashSet;
use std::ops::Deref;

use ast::*;

pub fn collect_string_lit(p: &Program) -> HashSet<String> {
    let mut res: HashSet<String> = HashSet::new();
    p.collect(&mut res);
    res
}

trait CollectStringLit {
    fn collect(&self, res: &mut HashSet<String>);
}

impl<T> CollectStringLit for Vec<T>
    where T: CollectStringLit
{
    fn collect(&self, res: &mut HashSet<String>) {
        for e in self {
            e.collect(res);
        }
    }
}

impl<T: Sized> CollectStringLit for Box<T>
    where T: CollectStringLit
{
    fn collect(&self, res: &mut HashSet<String>) {
        self.deref().collect(res);
    }
}

impl CollectStringLit for Program {
    fn collect(&self, res: &mut HashSet<String>) {
        for d in &self.0 {
            d.collect(res);
        }
    }
}

impl CollectStringLit for Def {
    fn collect(&self, res: &mut HashSet<String>) {
        match *self {
            Def::DClass(ref c) => c.collect(res),
            Def::DFunc(ref f) => f.collect(res),
        }
    }
}

impl CollectStringLit for Class {
    fn collect(&self, res: &mut HashSet<String>) {
        for m in &self.methods {
            m.collect(res);
        }
    }
}

impl CollectStringLit for Func {
    fn collect(&self, res: &mut HashSet<String>) {
        self.body.collect(res);
    }
}

impl CollectStringLit for Stmt {
    fn collect(&self, res: &mut HashSet<String>) {
        match *self {
            Stmt::SIf(ref e, ref s) |
            Stmt::SWhile(ref e, ref s) => {
                e.collect(res);
                s.collect(res);
            }
            Stmt::SBlock(ref stmts) => stmts.collect(res),
            Stmt::SDecl(_, ref var_decls) => var_decls.collect(res),
            Stmt::SAssign(_, ref e) |
            Stmt::SReturnE(ref e) |
            Stmt::SExpr(ref e) => e.collect(res),
            Stmt::SIfElse(ref e, ref s1, ref s2) => {
                e.collect(res);
                s1.collect(res);
                s2.collect(res);
            }
            Stmt::SFor(_, _, ref e, ref stmt) => {
                e.collect(res);
                stmt.collect(res);
            }
            _ => {}
        }
    }
}

impl CollectStringLit for VarDecl {
    fn collect(&self, res: &mut HashSet<String>) {
        if let VarDecl::Init(_, _, ref e) = *self {
            e.collect(res);
        }
        if let VarDecl::NoInit(Type::TString, _) = *self {
            res.insert(String::new());
        }
    }
}

impl CollectStringLit for Expr {
    fn collect(&self, res: &mut HashSet<String>) {
        match *self {
            Expr::ELit(ref l) => l.collect(res),
            Expr::ECall(_, ref exprs) => exprs.collect(res),
            Expr::ENeg(ref e) |
            Expr::ENot(ref e) => e.collect(res),
            Expr::EBinOp(ref e1, _, ref e2) => {
                e1.collect(res);
                e2.collect(res);
            }
            _ => {}
        }
    }
}

impl CollectStringLit for Lit {
    fn collect(&self, res: &mut HashSet<String>) {
        if let Lit::LString(ref s) = *self {
            res.insert(s.clone());
        }
    }
}
