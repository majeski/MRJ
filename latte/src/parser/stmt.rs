use libc::*;

use ast::{Ident, Stmt, Type, VarDecl};

use parser::expr::*;
use parser::many::*;
use parser::to_ast::*;

#[link(name = "parse", kind = "static")]
extern "C" {
    static STMT_TYPE_VAR_INIT: c_int;
    static STMT_TYPE_ASSIGN: c_int;
    static STMT_TYPE_POSTFIX: c_int;
    static STMT_TYPE_RETURN: c_int;
    static STMT_TYPE_BLOCK: c_int;
    static STMT_TYPE_EXPR: c_int;
    static STMT_TYPE_IF: c_int;
    static STMT_TYPE_WHILE: c_int;
}

#[repr(C)]
pub struct stmt_t {
    t: i32,
    ptr: *mut c_void,
}

impl ToAst<Stmt> for stmt_t {
    fn to_ast(&self) -> Stmt {
        unsafe {
            if self.t == STMT_TYPE_VAR_INIT {
                return (self.ptr as *mut stmt_var_decls_t).to_ast();
            }
            if self.t == STMT_TYPE_ASSIGN {
                return (self.ptr as *mut stmt_assign_t).to_ast();
            }
            if self.t == STMT_TYPE_POSTFIX {
                return (self.ptr as *mut stmt_postfix_t).to_ast();
            }
            if self.t == STMT_TYPE_RETURN {
                return if self.ptr.is_null() {
                    Stmt::SReturn
                } else {
                    Stmt::SReturnE((self.ptr as *mut expr_t).to_ast())
                };
            }
            if self.t == STMT_TYPE_BLOCK {
                return Stmt::SBlock(many_t::to_vec(self.ptr as *mut many_t, stmt_t::to_ast));
            }
            if self.t == STMT_TYPE_EXPR {
                return Stmt::SExpr((self.ptr as *mut expr_t).to_ast());
            }
            if self.t == STMT_TYPE_IF {
                return (self.ptr as *mut stmt_if_t).to_ast();
            }
            if self.t == STMT_TYPE_WHILE {
                return (self.ptr as *mut stmt_while_t).to_ast();
            }
        }
        panic!("Unknown statement type");
    }
}


#[repr(C)]
struct stmt_var_decls_t {
    var_type: *mut c_char,
    inits: *mut many_t,
}

impl ToAst<Stmt> for stmt_var_decls_t {
    fn to_ast(&self) -> Stmt {
        let t: Type = self.var_type.to_ast();
        let inits = many_t::to_vec(self.inits, var_decl_t::to_ast);
        Stmt::SDecl(t, inits)
    }
}

#[repr(C)]
struct var_decl_t {
    ident: *mut c_char,
    expr: *mut expr_t,
}

impl ToAst<VarDecl> for var_decl_t {
    fn to_ast(&self) -> VarDecl {
        let ident: Ident = self.ident.to_ast();
        unsafe {
            if self.expr.is_null() {
                return VarDecl::NoInit(ident);
            } else {
                return VarDecl::Init(ident, (*self.expr).to_ast());
            }
        }
    }
}

#[repr(C)]
struct stmt_assign_t {
    ident: *mut c_char,
    expr: *mut expr_t,
}

impl ToAst<Stmt> for stmt_assign_t {
    fn to_ast(&self) -> Stmt {
        let ident: Ident = self.ident.to_ast();
        let expr = unsafe { (*self.expr).to_ast() };
        Stmt::SAssign(ident, expr)
    }
}

#[repr(C)]
struct stmt_postfix_t {
    ident: *mut c_char,
    is_decr: i32,
}

impl ToAst<Stmt> for stmt_postfix_t {
    fn to_ast(&self) -> Stmt {
        let ident: Ident = self.ident.to_ast();
        match self.is_decr {
            0 => Stmt::SDec(ident),
            1 => Stmt::SInc(ident),
            _ => panic!("unknown postfix operator"),
        }
    }
}

#[repr(C)]
struct stmt_if_t {
    cond: *mut expr_t,
    if_s: *mut stmt_t,
    else_s: *mut stmt_t,
}

impl ToAst<Stmt> for stmt_if_t {
    fn to_ast(&self) -> Stmt {
        unsafe {
            let cond = (*self.cond).to_ast();
            let if_s = (*self.if_s).to_ast();
            if self.else_s.is_null() {
                Stmt::SIf(cond, Box::new(if_s))
            } else {
                let else_s = (*self.else_s).to_ast();
                Stmt::SIfElse(cond, Box::new(if_s), Box::new(else_s))
            }
        }
    }
}

#[repr(C)]
struct stmt_while_t {
    cond: *mut expr_t,
    stmt: *mut stmt_t,
}

impl ToAst<Stmt> for stmt_while_t {
    fn to_ast(&self) -> Stmt {
        unsafe {
            let cond = (*self.cond).to_ast();
            let stmt = (*self.stmt).to_ast();
            Stmt::SWhile(cond, Box::new(stmt))
        }
    }
}
