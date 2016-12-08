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
    fn to_ast(&self) -> Expr {
        unsafe {
            if self.t == EXPR_TYPE_IDENT {
                return Expr::EVar((self.ptr as *mut c_char).to_ast());
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
        panic!("Unknown expression type");
    }
}

impl ToAst<Operator> for *mut c_char {
    fn to_ast(&self) -> Operator {
        let op_str: String = self.to_ast();
        match op_str.as_ref() {
            "+" => Operator::OpAdd,
            "-" => Operator::OpSub,
            "*" => Operator::OpMul,
            "/" => Operator::OpDiv,
            "<" => Operator::OpLess,
            ">" => Operator::OpGreater,
            "<=" => Operator::OpLessE,
            ">=" => Operator::OpGreaterE,
            "==" => Operator::OpEq,
            "!=" => Operator::OpNEq,
            "&&" => Operator::OpAnd,
            "||" => Operator::OpOr,
            _ => panic!("unknown operator"),
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
    fn to_ast(&self) -> Expr {
        let lhs = self.lhs.to_ast();
        let rhs = self.rhs.to_ast();
        let op: Operator = self.op.to_ast();
        Expr::EBinOp(Box::new(lhs), op, Box::new(rhs))
    }
}

#[repr(C)]
struct expr_unary_t {
    expr: *mut expr_t,
    op: c_char,
}

impl ToAst<Expr> for expr_unary_t {
    fn to_ast(&self) -> Expr {
        let op = (self.op as u8) as char;
        let e = self.expr.to_ast();
        match op {
            '-' => Expr::ENeg(Box::new(e)),
            '!' => Expr::ENot(Box::new(e)),
            _ => panic!("Unknown unary operator"),
        }
    }
}

#[repr(C)]
struct expr_call_t {
    fname: *mut c_char,
    args: *mut many_t,
}

impl ToAst<Expr> for expr_call_t {
    fn to_ast(&self) -> Expr {
        let fname: Ident = self.fname.to_ast();
        let args = many_t::to_vec(self.args, expr_t::to_ast);
        Expr::ECall(fname, args)
    }
}

#[repr(C)]
struct expr_lit_t {
    t: i32,
    lit: *mut c_char,
}

impl ToAst<Expr> for expr_lit_t {
    fn to_ast(&self) -> Expr {
        let lit_str: String = self.lit.to_ast();
        let lit = unsafe {
            if self.t == EXPR_TYPE_LIT_INT {
                Lit::LInt(lit_str.parse().expect("int literal err"))
            } else if self.t == EXPR_TYPE_LIT_STR {
                Lit::LString(lit_str)
            } else if self.t == EXPR_TYPE_LIT_BOOL {
                match lit_str.as_ref() {
                    "true" => Lit::LTrue,
                    "false" => Lit::LFalse,
                    _ => panic!("unknown boolean literal"),
                }
            } else {
                panic!("uknown literal type")
            }
        };
        Expr::ELit(lit)
    }
}
