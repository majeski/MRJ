use libc::*;

use ast::{Def, FuncArg, Ident, Type};

use parser::many::*;
use parser::stmt::*;
use parser::to_ast::*;

#[link(name = "parse", kind = "static")]
extern "C" {
    static DEF_TYPE_FUNC: c_int;
}

#[repr(C)]
pub struct def_t {
    t: i32,
    ptr: *mut c_void,
}

impl ToAst<Def> for def_t {
    fn to_ast(&self) -> Def {
        unsafe {
            if self.t == DEF_TYPE_FUNC {
                return (self.ptr as *mut def_func_t).to_ast();
            }
            panic!("unknown definition type");
        }
    }
}

#[repr(C)]
struct def_func_t {
    ret_type: *mut c_char,
    ident: *mut c_char,
    args: *mut many_t,
    stmt: *mut stmt_t,
}

impl ToAst<Def> for def_func_t {
    fn to_ast(&self) -> Def {
        let ret_type: Type = self.ret_type.to_ast();
        let ident: Ident = self.ident.to_ast();
        let args = many_t::to_vec(self.args, func_arg_t::to_ast);
        let stmt = self.stmt.to_ast();
        Def::DFunc(ident, args, ret_type, stmt)
    }
}

#[repr(C)]
struct func_arg_t {
    var_type: *mut c_char,
    ident: *mut c_char,
}

impl ToAst<FuncArg> for func_arg_t {
    fn to_ast(&self) -> FuncArg {
        let var_type: Type = self.var_type.to_ast();
        let ident: Ident = self.ident.to_ast();
        FuncArg(var_type, ident)
    }
}
