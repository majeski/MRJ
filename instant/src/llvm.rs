use std::io;
use std::collections::HashMap;
use std::ops::Deref;
use std::fmt;
use std::fmt::Display;

use ast::*;

pub struct LLVMContext<'a> {
    vars: HashMap<String, Val>,
    out: &'a mut io::Write,
    next_id: i32,
}

enum Val {
    Const(i32),
    Register(i32),
}

impl Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Val::Const(x) => write!(f, "{}", x),
            Val::Register(x) => write!(f, "%{}", x),
        }
    }
}

impl<'a> LLVMContext<'a> {
    pub fn new(out: &'a mut io::Write) -> LLVMContext<'a> {
        LLVMContext {
            vars: HashMap::new(),
            out: out,
            next_id: 1,
        }
    }

    fn get_next_id(&mut self) -> Val {
        let id = self.next_id;
        self.next_id += 1;
        Val::Register(id)
    }
}

pub fn compile(program: &Program, context: &mut LLVMContext) -> io::Result<()> {
    program.compile(context)
}

impl Program {
    fn compile(&self, context: &mut LLVMContext) -> io::Result<()> {
        let Program(ref stmts) = *self;
        try!(context.out.write_all(b"\
            declare void @printInt(i32)\n\
            define i32 @main() {\n"));
        for stmt in stmts {
            try!(stmt.compile(context));
        }
        try!(context.out.write_all(b"ret i32 0\n}\n"));
        Ok(())
    }
}

impl Stmt {
    fn compile(&self, context: &mut LLVMContext) -> io::Result<()> {
        match *self {
            Stmt::Assign(ref name, ref e) => {
                let res_id = try!(e.compile(context));
                let var_id = context.get_next_id();
                try!(context.out
                    .write_fmt(format_args!("{} = alloca i32\n", var_id)));
                try!(context.out
                    .write_fmt(format_args!("store i32 {}, i32* {}\n", res_id, var_id)));
                context.vars.insert(name.clone(), var_id);
                Ok(())
            }
            Stmt::Expr(ref e) => {
                let id = try!(e.compile(context));
                try!(context.out.write_fmt(format_args!("call void @printInt(i32 {})\n", id)));
                Ok(())
            }
        }
    }
}

impl Expr {
    fn compile(&self, context: &mut LLVMContext) -> io::Result<Val> {
        match *self {
            Expr::Const(x) => Ok(Val::Const(x)),
            Expr::Ident(ref s) => {
                let id = context.get_next_id();
                let var_id = context.vars.get(s).expect("Undefined variable");
                try!(context.out.write_fmt(format_args!("{} = load i32, i32* {}\n", id, var_id)));
                Ok(id)
            }
            Expr::BinOp(ref lhs, op, ref rhs) => {
                let l = try!(lhs.deref().compile(context));
                let r = try!(rhs.deref().compile(context));
                op.compile(l, r, context)
            }
        }
    }
}

impl Operator {
    fn compile(&self, lval: Val, rval: Val, context: &mut LLVMContext) -> io::Result<Val> {
        let op = match *self {
            Operator::Add => "add",
            Operator::Sub => "sub",
            Operator::Mul => "mul",
            Operator::Div => "sdiv",
        };
        let res_id = context.get_next_id();
        try!(context.out
            .write_fmt(format_args!("{} = {} i32 {}, {}\n", res_id, op, lval, rval)));
        Ok(res_id)
    }
}
