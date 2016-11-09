use std::{self, fmt};

#[derive(Debug, Clone)]
pub struct Program(pub std::vec::Vec<Stmt>);

#[derive(Debug, Clone)]
pub enum Stmt {
    Assign(String, Expr),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Const(i32),
    Ident(String),
    BinOp(Box<Expr>, Operator, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Program(ref stmts) = *self;
        for ref stmt in stmts {
            try!(write!(f, "{};\n", stmt));
        }
        Ok(())
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Stmt::Assign(ref iden, ref expr) => write!(f, "{} = {}", iden, expr),
            Stmt::Expr(ref expr) => write!(f, "{}", expr),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expr::Const(val) => write!(f, "{}", val),
            Expr::Ident(ref ide) => write!(f, "{}", ide),
            Expr::BinOp(ref lhs, ref op, ref rhs) => write!(f, "({} {} {})", lhs, op, rhs),
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let op = match *self {
            Operator::Add => '+',
            Operator::Sub => '-',
            Operator::Mul => '*',
            Operator::Div => '/',
        };
        write!(f, "{}", op)
    }
}
