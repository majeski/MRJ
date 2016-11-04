use std::cmp;
use std::mem;
use std::io;
use std::ops::Deref;
use std::vec::Vec;
use std::collections::HashMap;
use std::collections::HashSet;

use ast::*;

type VariableMap = HashMap<String, usize>;
type VariableSet = HashSet<String>;

pub fn optimize(program: &mut Program) {
    let Program(ref mut stmts) = *program;
    for stmt in stmts {
        match *stmt {
            Stmt::Expr(ref mut e) => {
                optimize_stack_size(e);
            }
            _ => {}
        };
    }
}

// returns stack size
fn optimize_stack_size(expr: &mut Expr) -> i32 {
    match *expr {
        Expr::Const(_) => 1,
        Expr::Ident(_) => 1,
        Expr::BinOp(ref mut lhs, Operator::Sub, ref mut rhs) |
        Expr::BinOp(ref mut lhs, Operator::Div, ref mut rhs) => {
            cmp::max(optimize_stack_size(lhs), optimize_stack_size(rhs) + 1)
        }
        Expr::BinOp(ref mut lhs, Operator::Add, ref mut rhs) |
        Expr::BinOp(ref mut lhs, Operator::Mul, ref mut rhs) => {
            let mut lsize = optimize_stack_size(lhs);
            let mut rsize = optimize_stack_size(rhs);
            if rsize > lsize {
                mem::swap(lhs, rhs);
                mem::swap(&mut lsize, &mut rsize);
            }
            cmp::max(lsize, rsize + 1)
        }
    }
}

pub fn compile(program: &Program, out: &mut io::Write) -> io::Result<()> {
    let &Program(ref stmts) = program;

    try!(create_prelude(program, out));
    let mut vars = VariableMap::new();
    for stmt in stmts {
        try!(compile_stmt(stmt, &mut vars, out));
    }
    try!(create_end(out));
    Ok(())
}

fn compile_stmt(stmt: &Stmt, vars: &mut VariableMap, out: &mut io::Write) -> io::Result<()> {
    match *stmt {
        Stmt::Assign(ref name, ref expr) => compile_assign(name, expr, vars, out),
        Stmt::Expr(ref e) => {
            try!(compile_expr(e, vars, out));
            try!(out.write_all(b"\
                getstatic java/lang/System/out Ljava/io/PrintStream;\n\
                swap\n\
                invokevirtual java/io/PrintStream/println(I)V\n"));
            Ok(())
        }
    }
}

fn compile_assign(name: &String,
                  expr: &Expr,
                  vars: &mut VariableMap,
                  out: &mut io::Write)
                  -> io::Result<()> {
    try!(compile_expr(expr, vars, out));
    if !vars.contains_key(name) {
        let idx = vars.len();
        vars.insert(name.clone(), idx);
    }
    pop_val(name, vars, out)
}

fn compile_expr(expr: &Expr, vars: &VariableMap, out: &mut io::Write) -> io::Result<()> {
    match *expr {
        Expr::Const(x) => push_int(x, out),
        Expr::Ident(ref x) => push_val(x, vars, out),
        Expr::BinOp(ref lhs, op, ref rhs) => {
            try!(compile_expr(lhs.deref(), vars, out));
            try!(compile_expr(rhs.deref(), vars, out));
            try!(compile_op(op, out));
            Ok(())
        }
    }
}

fn push_int(val: i32, out: &mut io::Write) -> io::Result<()> {
    match val {
        0...5 => out.write_fmt(format_args!("iconst_{}\n", val)),
        _ => out.write_fmt(format_args!("ldc {}\n", val)),
    }
}

fn push_val(name: &String, vars: &VariableMap, out: &mut io::Write) -> io::Result<()> {
    let short_idx = 3 as usize;
    match vars.get(name) {
        Some(idx) if idx <= &short_idx => out.write_fmt(format_args!("iload_{}\n", idx)),
        Some(idx) => out.write_fmt(format_args!("iload {}\n", idx)),
        None => panic!("Undefined variable"),
    }
}

fn pop_val(name: &String, vars: &VariableMap, out: &mut io::Write) -> io::Result<()> {
    let short_idx = 3 as usize;
    match vars.get(name) {
        Some(idx) if idx <= &short_idx => out.write_fmt(format_args!("istore_{}\n", idx)),
        Some(idx) => out.write_fmt(format_args!("istore {}\n", idx)),
        None => panic!("Undefined variable"),
    }
}

fn compile_op(op: Operator, out: &mut io::Write) -> io::Result<()> {
    match op {
        Operator::Add => out.write_all(b"iadd\n"),
        Operator::Sub => out.write_all(b"isub\n"),
        Operator::Mul => out.write_all(b"imul\n"),
        Operator::Div => out.write_all(b"idiv\n"),
    }
}

fn create_prelude(program: &Program, out: &mut io::Write) -> io::Result<()> {
    let Program(ref stmts) = *program;
    let stack_size = cmp::max(2,
                              match stmts.iter().map(stack_size_stmt).max() {
                                  Some(x) => x,
                                  None => 0,
                              });
    let vars_count = variable_count(stmts) + 1;

    try!(out.write_all(b"\
        .class public Instant\n\
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

    try!(out.write_fmt(format_args!(".limit stack  {}\n", stack_size)));
    try!(out.write_fmt(format_args!(".limit locals {}\n", vars_count)));
    Ok(())
}

fn create_end(out: &mut io::Write) -> io::Result<()> {
    try!(out.write_all(b"return\n"));
    try!(out.write_all(b".end method\n"));
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

fn stack_size_stmt(stmt: &Stmt) -> i32 {
    match *stmt {
        Stmt::Assign(_, ref e) => stack_size_expr(e),
        Stmt::Expr(ref e) => stack_size_expr(e),
    }
}

fn stack_size_expr(expr: &Expr) -> i32 {
    match *expr {
        Expr::BinOp(ref lhs, _, ref rhs) => {
            cmp::max(stack_size_expr(lhs.deref()),
                     stack_size_expr(rhs.deref()) + 1)
        }
        _ => 1,
    }
}
