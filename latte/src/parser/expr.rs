use libc::*;

use ast::{Expr, Lit, Operator};

use parser::field_get::*;
use parser::many::*;
use parser::to_ast::*;

#[link(name = "parse", kind = "static")]
extern "C" {
    static EXPR_TYPE_BINOP: c_int;
    static EXPR_TYPE_CALL: c_int;
    static EXPR_TYPE_FIELD: c_int;
    static EXPR_TYPE_LIT: c_int;
    static EXPR_TYPE_LIT_BOOL: c_int;
    static EXPR_TYPE_LIT_INT: c_int;
    static EXPR_TYPE_LIT_NULL: c_int;
    static EXPR_TYPE_LIT_STR: c_int;
    static EXPR_TYPE_NEW_ARR: c_int;
    static EXPR_TYPE_UNARY: c_int;
}

#[repr(C)]
pub struct expr_t {
    t: i32,
    ptr: *mut c_void,
}

impl ToAst<Expr> for expr_t {
    fn to_ast(&self) -> TAResult<Expr> {
        unsafe {
            if self.t == EXPR_TYPE_FIELD {
                let var = (self.ptr as *mut field_get_t).to_ast()?;
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
            if self.t == EXPR_TYPE_NEW_ARR {
                return (self.ptr as *mut expr_new_array_t).to_ast();
            }
        }
        return Err(format!("Unknown expression type: {}", self.t));
    }
}

impl ToAst<Operator> for *mut c_char {
    fn to_ast(&self) -> TAResult<Operator> {
        let op_str: String = self.to_ast()?;
        match op_str.as_ref() {
            "+" => Ok(Operator::OpAdd),
            "-" => Ok(Operator::OpSub),
            "*" => Ok(Operator::OpMul),
            "/" => Ok(Operator::OpDiv),
            "%" => Ok(Operator::OpMod),
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
        let lhs = self.lhs.to_ast()?;
        let rhs = self.rhs.to_ast()?;
        let op: Operator = self.op.to_ast()?;
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
        let e = self.expr.to_ast()?;
        match op {
            '-' => Ok(Expr::ENeg(Box::new(e))),
            '!' => Ok(Expr::ENot(Box::new(e))),
            _ => Err(format!("Unknown unary operator: {}", op)),
        }
    }
}

#[repr(C)]
struct expr_call_t {
    field: *mut field_get_t,
    args: *mut many_t,
}

impl ToAst<Expr> for expr_call_t {
    fn to_ast(&self) -> TAResult<Expr> {
        let field = self.field.to_ast()?;
        let args = many_t::to_vec(self.args, expr_t::to_ast)?;
        Ok(Expr::ECall(field, args))
    }
}

#[repr(C)]
struct expr_lit_t {
    t: i32,
    lit: *mut c_char,
}

impl ToAst<Expr> for expr_lit_t {
    fn to_ast(&self) -> TAResult<Expr> {
        let lit = unsafe {
            if self.t == EXPR_TYPE_LIT_NULL {
                Lit::LNull
            } else {
                let lit_str: String = self.lit.to_ast()?;
                if self.t == EXPR_TYPE_LIT_INT {
                    let x = match lit_str.parse::<i32>() {
                        Ok(x) => Ok(x),
                        Err(x) => Err(format!("Cannot convert {} to i32: {}", lit_str, x)),
                    }?;
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
            }
        };
        Ok(Expr::ELit(lit))
    }
}

#[repr(C)]
struct expr_new_array_t {
    t: *mut c_char,
    size: *mut expr_t,
}

impl ToAst<Expr> for expr_new_array_t {
    fn to_ast(&self) -> TAResult<Expr> {
        Ok(Expr::ENewArray(self.t.to_ast()?, Box::new(self.size.to_ast()?)))
    }
}
