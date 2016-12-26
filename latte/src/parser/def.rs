use libc::*;
use either::Either;

use ast::{Class, Def, Func, Ident, Var};

use parser::many::*;
use parser::stmt::*;
use parser::to_ast::*;

#[link(name = "parse", kind = "static")]
extern "C" {
    static DEF_TYPE_FUNC: c_int;
    static DEF_TYPE_CLASS: c_int;

    static CLASS_MEMBER_TYPE_FUNC: c_int;
    static CLASS_MEMBER_TYPE_VAR: c_int;
}

#[repr(C)]
pub struct def_t {
    t: i32,
    ptr: *mut c_void,
}

impl ToAst<Def> for def_t {
    fn to_ast(&self) -> TAResult<Def> {
        unsafe {
            if self.t == DEF_TYPE_FUNC {
                return Ok(Def::DFunc((self.ptr as *mut func_t).to_ast()?));
            }
            if self.t == DEF_TYPE_CLASS {
                return Ok(Def::DClass((self.ptr as *mut class_t).to_ast()?));
            }
            Err(format!("Unknown definition type: {}", self.t))
        }
    }
}

#[repr(C)]
struct class_t {
    name: *mut c_char,
    superclass: *mut c_char,
    members: *mut many_t,
}

impl ToAst<Class> for class_t {
    fn to_ast(&self) -> TAResult<Class> {
        let superclass: Option<Ident> = match self.superclass.is_null() {
            true => None,
            false => Some(self.superclass.to_ast()?)
        };
        let members = many_t::to_vec(self.members, class_member_t::to_ast)?;
        let vars = members.iter()
            .filter(|v| v.is_right())
            .map(|v| v.clone().right().unwrap())
            .collect();
        let funcs =
            members.iter().filter(|f| f.is_left()).map(|f| f.clone().left().unwrap()).collect();
        Ok(Class {
            name: self.name.to_ast()?,
            superclass: superclass,
            vars: vars,
            methods: funcs,
        })
    }
}

#[repr(C)]
struct class_member_t {
    t: i32,
    ptr: *mut c_void,
}

impl ToAst<Either<Func, Var>> for class_member_t {
    fn to_ast(&self) -> TAResult<Either<Func, Var>> {
        unsafe {
            if self.t == CLASS_MEMBER_TYPE_FUNC {
                return Ok(Either::Left((self.ptr as *mut func_t).to_ast()?));
            }
            if self.t == CLASS_MEMBER_TYPE_VAR {
                return Ok(Either::Right((self.ptr as *mut var_t).to_ast()?));
            }
            Err(format!("Unknwon definition type: {}", self.t))

        }
    }
}

#[repr(C)]
struct func_t {
    ret_type: *mut c_char,
    ident: *mut c_char,
    args: *mut many_t,
    body: *mut many_t,
}

impl ToAst<Func> for func_t {
    fn to_ast(&self) -> TAResult<Func> {
        Ok(Func {
            ret_type: self.ret_type.to_ast()?,
            ident: self.ident.to_ast()?,
            args: many_t::to_vec(self.args, var_t::to_ast)?,
            body: many_t::to_vec(self.body, stmt_t::to_ast)?,
        })
    }
}

#[repr(C)]
struct var_t {
    t: *mut c_char,
    ident: *mut c_char,
}

impl ToAst<Var> for var_t {
    fn to_ast(&self) -> TAResult<Var> {
        Ok(Var {
            t: self.t.to_ast()?,
            ident: self.ident.to_ast()?,
        })
    }
}
