use libc::*;

use ast::{Expr, Ident, Lit, Operator};

use parser::many::*;
use parser::to_ast::*;

#[link(name = "parse", kind = "static")]
extern "C" {
    static EXPR_TYPE_BINOP: c_int;
    static EXPR_TYPE_UNARY: c_int;
    static EXPR_TYPE_CALL: c_int;
    static EXPR_TYPE_IDENT: c_int;
    static EXPR_TYPE_LIT: c_int;
    static EXPR_TYPE_LIT_INT: c_int;
    static EXPR_TYPE_LIT_STR: c_int;
    static EXPR_TYPE_LIT_BOOL: c_int;
}

#[repr(C)]
pub struct expr_t {
    t: i32,
    ptr: *mut c_void,
}

impl ToAst<Expr> for expr_t {
    fn to_ast(&self) -> TAResult<Expr> {
        unsafe {
            if self.t == EXPR_TYPE_IDENT {
                let var = try!((self.ptr as *mut c_char).to_ast());
                return Ok(Expr::EVar(var));
            }
            if self.t == EXPR_TYPE_LIT {
                return (self.ptr as *mut expr_lit_t).to_ast();
            }
            if self.t == EXPR_TYPE_CALL {
                return (self.ptr as *mut expr_call_t).to_ast();
            }
            if self.t == EXPR_TYPE_UNARY {
                return (self.ptr as *mut expr_unary_t).to_ast();
            }
            if self.t == EXPR_TYPE_BINOP {
                return (self.ptr as *mut expr_binop_t).to_ast();
            }
        }
        return Err(format!("Unknown expression type: {}", self.t));
    }
}

impl ToAst<Operator> for *mut c_char {
    fn to_ast(&self) -> TAResult<Operator> {
        let op_str: String = try!(self.to_ast());
        match op_str.as_ref() {
            "+" => Ok(Operator::OpAdd),
            "-" => Ok(Operator::OpSub),
            "*" => Ok(Operator::OpMul),
            "/" => Ok(Operator::OpDiv),
            "<" => Ok(Operator::OpLess),
            ">" => Ok(Operator::OpGreater),
            "<=" => Ok(Operator::OpLessE),
            ">=" => Ok(Operator::OpGreaterE),
            "==" => Ok(Operator::OpEq),
            "!=" => Ok(Operator::OpNEq),
            "&&" => Ok(Operator::OpAnd),
            "||" => Ok(Operator::OpOr),
            _ => Err(format!("Unknown operator: {}", op_str)),
        }
    }
}

#[repr(C)]
struct expr_binop_t {
    lhs: *mut expr_t,
    rhs: *mut expr_t,
    op: *mut c_char,
}

impl ToAst<Expr> for expr_binop_t {
    fn to_ast(&self) -> TAResult<Expr> {
        let lhs = try!(self.lhs.to_ast());
        let rhs = try!(self.rhs.to_ast());
        let op: Operator = try!(self.op.to_ast());
        Ok(Expr::EBinOp(Box::new(lhs), op, Box::new(rhs)))
    }
}

#[repr(C)]
struct expr_unary_t {
    expr: *mut expr_t,
    op: c_char,
}

impl ToAst<Expr> for expr_unary_t {
    fn to_ast(&self) -> TAResult<Expr> {
        let op = (self.op as u8) as char;
        let e = try!(self.expr.to_ast());
        match op {
            '-' => Ok(Expr::ENeg(Box::new(e))),
            '!' => Ok(Expr::ENot(Box::new(e))),
            _ => Err(format!("Unknown unary operator: {}", op)),
        }
    }
}

#[repr(C)]
struct expr_call_t {
    fname: *mut c_char,
    args: *mut many_t,
}

impl ToAst<Expr> for expr_call_t {
    fn to_ast(&self) -> TAResult<Expr> {
        let fname: Ident = try!(self.fname.to_ast());
        let args = try!(many_t::to_vec(self.args, expr_t::to_ast));
        Ok(Expr::ECall(fname, args))
    }
}

#[repr(C)]
struct expr_lit_t {
    t: i32,
    lit: *mut c_char,
}

impl ToAst<Expr> for expr_lit_t {
    fn to_ast(&self) -> TAResult<Expr> {
        let lit_str: String = try!(self.lit.to_ast());
        let lit = unsafe {
            if self.t == EXPR_TYPE_LIT_INT {
                let x = try!(match lit_str.parse::<i32>() {
                    Ok(x) => Ok(x),
                    Err(x) => Err(format!("Cannot convert {} to i32: {}", lit_str, x)),
                });
                Lit::LInt(x)
            } else if self.t == EXPR_TYPE_LIT_STR {
                Lit::LString(lit_str)
            } else if self.t == EXPR_TYPE_LIT_BOOL {
                match lit_str.as_ref() {
                    "true" => Lit::LTrue,
                    "false" => Lit::LFalse,
                    _ => return Err(format!("Unknown boolean literal: {}", lit_str)),
                }
            } else {
                return Err(format!("Unknown literal type: {}", self.t));
            }
        };
        Ok(Expr::ELit(lit))
    }
}
