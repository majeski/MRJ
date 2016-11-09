use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::vec::Vec;
use std::{cmp, io, mem, vec};

use ast::{self, Operator};

type VariableMap = HashMap<String, usize>;
type VariableSet = HashSet<String>;

pub struct Program(vec::Vec<Stmt>);

enum Stmt {
    Assign(String, Expr),
    Expr(Expr),
}

enum Expr {
    Const(i32),
    Ident(String),
    BinOp(Box<Expr>, Operator, Box<Expr>, bool),
}

pub fn prepare_ast(p: &ast::Program) -> Program {
    let mut jvm_stmts = vec::Vec::new();
    for stmt in &p.0 {
        jvm_stmts.push(stmt_to_jvm(stmt));
    }

    Program(jvm_stmts)
}

fn stmt_to_jvm(stmt: &ast::Stmt) -> Stmt {
    match *stmt {
        ast::Stmt::Assign(ref iden, ref e) => Stmt::Assign(iden.clone(), expr_to_jvm(e)),
        ast::Stmt::Expr(ref e) => Stmt::Expr(expr_to_jvm(e)),
    }
}

fn expr_to_jvm(expr: &ast::Expr) -> Expr {
    match *expr {
        ast::Expr::Const(x) => Expr::Const(x),
        ast::Expr::Ident(ref x) => Expr::Ident(x.clone()),
        ast::Expr::BinOp(ref lhs, op, ref rhs) => {
            Expr::BinOp(Box::new(expr_to_jvm(lhs)),
                        op,
                        Box::new(expr_to_jvm(rhs)),
                        false)
        }
    }
}

pub fn optimize(program: &mut Program) {
    let Program(ref mut stmts) = *program;
    for stmt in stmts {
        match *stmt {
            Stmt::Expr(ref mut e) => optimize_stack_size(e),
            Stmt::Assign(_, ref mut e) => optimize_stack_size(e),
        };
    }
}

// returns stack size
fn optimize_stack_size(expr: &mut Expr) -> i32 {
    match *expr {
        Expr::BinOp(ref mut lhs, _, ref mut rhs, ref mut reversed) => {
            let mut lsize = optimize_stack_size(lhs);
            let mut rsize = optimize_stack_size(rhs);
            if rsize > lsize {
                mem::swap(lhs, rhs);
                mem::swap(&mut lsize, &mut rsize);
                *reversed = true;
            }
            cmp::max(lsize, rsize + 1)
        }
        _ => 1,
    }
}

pub struct JVMContext<'a> {
    vars: HashMap<String, usize>,
    out: &'a mut io::Write,
}

impl<'a> JVMContext<'a> {
    pub fn new(out: &'a mut io::Write) -> JVMContext<'a> {
        JVMContext {
            vars: HashMap::new(),
            out: out,
        }
    }

    fn add_var(&mut self, name: &String) {
        if !self.vars.contains_key(name) {
            let idx = self.vars.len() + 1;
            self.vars.insert(name.clone(), idx);
        }
    }
}

trait JVMCompile {
    fn compile(&self, ctx: &mut JVMContext) -> io::Result<()>;
}

pub fn compile(program: &Program, class_name: &str, ctx: &mut JVMContext) -> io::Result<()> {
    try!(create_prelude(program, class_name, ctx));
    for stmt in &program.0 {
        try!(stmt.compile(ctx))
    }
    try!(create_end(ctx));
    Ok(())
}

fn create_prelude(program: &Program, class_name: &str, ctx: &mut JVMContext) -> io::Result<()> {
    let stmts = &program.0;
    let stack_size = cmp::max(2, stmts.iter().map(Stmt::stack_size).max().unwrap_or(0));
    // + main argument
    let vars_count = variable_count(stmts) + 1;

    try!(ctx.out.write_fmt(format_args!(".class public {}\n", class_name)));
    try!(ctx.out.write_all(b"\
        .super java/lang/Object\n\
        \n\
        .method public <init>()V\n\
            aload_0\n\
            invokenonvirtual java/lang/Object/<init>()V\n\
            return\n\
        .end method\n\
        \n\
        .method public static main([Ljava/lang/String;)V\n\
        "));

    try!(ctx.out.write_fmt(format_args!(".limit stack  {}\n", stack_size)));
    try!(ctx.out.write_fmt(format_args!(".limit locals {}\n", vars_count)));
    Ok(())
}

fn variable_count(stmts: &Vec<Stmt>) -> usize {
    let mut vars = VariableSet::new();
    for stmt in stmts {
        match *stmt {
            Stmt::Assign(ref name, _) => {
                vars.insert(name.clone());
            }
            _ => {}
        }
    }
    vars.len()
}

trait StackSize {
    fn stack_size(&self) -> i32;
}

impl StackSize for Stmt {
    fn stack_size(&self) -> i32 {
        match *self {
            Stmt::Assign(_, ref e) => e.stack_size(),
            Stmt::Expr(ref e) => e.stack_size(),
        }
    }
}

impl StackSize for Expr {
    fn stack_size(&self) -> i32 {
        match *self {
            Expr::BinOp(ref lhs, _, ref rhs, _) => {
                cmp::max(lhs.deref().stack_size(), rhs.deref().stack_size() + 1)
            }
            _ => 1,
        }
    }
}

fn create_end(ctx: &mut JVMContext) -> io::Result<()> {
    try!(ctx.out.write_all(b"return\n"));
    try!(ctx.out.write_all(b".end method\n"));
    Ok(())
}

impl JVMCompile for Stmt {
    fn compile(&self, ctx: &mut JVMContext) -> io::Result<()> {
        match *self {
            Stmt::Assign(ref name, ref e) => {
                try!(e.compile(ctx));
                ctx.add_var(name);
                store_var(name, ctx)
            }
            Stmt::Expr(ref e) => {
                try!(e.compile(ctx));
                try!(ctx.out.write_all(b"\
                    getstatic java/lang/System/out Ljava/io/PrintStream;\n\
                    swap\n\
                    invokevirtual java/io/PrintStream/println(I)V\n"));
                Ok(())
            }
        }
    }
}

impl JVMCompile for Expr {
    fn compile(&self, ctx: &mut JVMContext) -> io::Result<()> {
        match *self {
            Expr::Const(x) => x.compile(ctx),
            Expr::Ident(ref x) => load_var(x, ctx),
            Expr::BinOp(ref lhs, op, ref rhs, reversed) => {
                try!(lhs.deref().compile(ctx));
                try!(rhs.deref().compile(ctx));
                if reversed && op != Operator::Add && op != Operator::Mul {
                    try!(ctx.out.write_all(b"swap\n"));
                }
                try!(op.compile(ctx));
                Ok(())
            }
        }
    }
}

impl JVMCompile for Operator {
    fn compile(&self, ctx: &mut JVMContext) -> io::Result<()> {
        let op = match *self {
            Operator::Add => b"iadd\n",
            Operator::Sub => b"isub\n",
            Operator::Mul => b"imul\n",
            Operator::Div => b"idiv\n",
        };
        ctx.out.write_all(op)
    }
}

impl JVMCompile for i32 {
    fn compile(&self, ctx: &mut JVMContext) -> io::Result<()> {
        let instr = match *self {
            0...5 => format!("iconst_{}\n", self),
            6...127 => format!("bipush {}\n", self),
            _ => format!("ldc {}\n", self),
        };
        ctx.out.write_all(instr.as_bytes())
    }
}

fn load_var(name: &String, ctx: &mut JVMContext) -> io::Result<()> {
    let short_idx = 3 as usize;
    let instr = match ctx.vars.get(name) {
        Some(idx) if idx <= &short_idx => format!("iload_{}\n", idx),
        Some(idx) => format!("iload {}\n", idx),
        None => panic!("Undefined variable"),
    };
    ctx.out.write_all(instr.as_bytes())
}

fn store_var(name: &String, ctx: &mut JVMContext) -> io::Result<()> {
    let short_idx = 3 as usize;
    let instr = match ctx.vars.get(name) {
        Some(idx) if idx <= &short_idx => format!("istore_{}\n", idx),
        Some(idx) => format!("istore {}\n", idx),
        None => panic!("Undefined variable"),
    };
    ctx.out.write_all(instr.as_bytes())
}
