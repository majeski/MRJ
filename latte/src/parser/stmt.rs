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
    fn to_ast(&self) -> TAResult<Stmt> {
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
                    Ok(Stmt::SReturn)
                } else {
                    let e = try!((self.ptr as *mut expr_t).to_ast());
                    Ok(Stmt::SReturnE(e))
                };
            }
            if self.t == STMT_TYPE_BLOCK {
                let stmts = try!(many_t::to_vec(self.ptr as *mut many_t, stmt_t::to_ast));
                return Ok(Stmt::SBlock(stmts));
            }
            if self.t == STMT_TYPE_EXPR {
                let e = try!((self.ptr as *mut expr_t).to_ast());
                return Ok(Stmt::SExpr(e));
            }
            if self.t == STMT_TYPE_IF {
                return (self.ptr as *mut stmt_if_t).to_ast();
            }
            if self.t == STMT_TYPE_WHILE {
                return (self.ptr as *mut stmt_while_t).to_ast();
            }
        }
        Err(format!("Unknown statement type: {}", self.t))
    }
}


#[repr(C)]
struct stmt_var_decls_t {
    var_type: *mut c_char,
    inits: *mut many_t,
}

impl ToAst<Stmt> for stmt_var_decls_t {
    fn to_ast(&self) -> TAResult<Stmt> {
        let t: Type = try!(self.var_type.to_ast());
        let inits = try!(many_t::to_vec(self.inits, var_decl_t::to_ast));
        Ok(Stmt::SDecl(t, inits))
    }
}

#[repr(C)]
struct var_decl_t {
    ident: *mut c_char,
    expr: *mut expr_t,
}

impl ToAst<VarDecl> for var_decl_t {
    fn to_ast(&self) -> TAResult<VarDecl> {
        let ident: Ident = try!(self.ident.to_ast());
        if self.expr.is_null() {
            return Ok(VarDecl::NoInit(ident));
        } else {
            let e = try!(self.expr.to_ast());
            return Ok(VarDecl::Init(ident, e));
        }
    }
}

#[repr(C)]
struct stmt_assign_t {
    ident: *mut c_char,
    expr: *mut expr_t,
}

impl ToAst<Stmt> for stmt_assign_t {
    fn to_ast(&self) -> TAResult<Stmt> {
        let ident: Ident = try!(self.ident.to_ast());
        let expr = try!(self.expr.to_ast());
        Ok(Stmt::SAssign(ident, expr))
    }
}

#[repr(C)]
struct stmt_postfix_t {
    ident: *mut c_char,
    is_decr: i32,
}

impl ToAst<Stmt> for stmt_postfix_t {
    fn to_ast(&self) -> TAResult<Stmt> {
        let ident: Ident = try!(self.ident.to_ast());
        match self.is_decr {
            0 => Ok(Stmt::SDec(ident)),
            1 => Ok(Stmt::SInc(ident)),
            _ => Err(format!("Unknown postfix operator flag: {}", self.is_decr)),
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
    fn to_ast(&self) -> TAResult<Stmt> {
        let cond = try!(self.cond.to_ast());
        let if_s = try!(self.if_s.to_ast());
        if self.else_s.is_null() {
            Ok(Stmt::SIf(cond, Box::new(if_s)))
        } else {
            let else_s = try!(self.else_s.to_ast());
            Ok(Stmt::SIfElse(cond, Box::new(if_s), Box::new(else_s)))
        }
    }
}

#[repr(C)]
struct stmt_while_t {
    cond: *mut expr_t,
    stmt: *mut stmt_t,
}

impl ToAst<Stmt> for stmt_while_t {
    fn to_ast(&self) -> TAResult<Stmt> {
        let cond = try!(self.cond.to_ast());
        let stmt = try!(self.stmt.to_ast());
        Ok(Stmt::SWhile(cond, Box::new(stmt)))
    }
}
