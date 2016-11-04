use std;
use nom::*;

use ast::*;

named!(pub program (&[u8]) -> Program, chain!(
        stmts: separated_list!(semi, statement) ~
        multispace? ~
        eof,
        || Program(stmts)
    )
);

named!(pub statement (&[u8]) -> Stmt, alt!(assignment | stmt_expr));
named!(assignment (&[u8]) -> Stmt, complete!(
    chain!(
        ident: identifier ~
        ass_op ~
        expr: expr1,
        || Stmt::Assign(ident, expr)
    )
));

named!(stmt_expr (&[u8]) -> Stmt, map!(expr1, Stmt::Expr));

named!(pub expr1 (&[u8]) -> Expr, alt!(do_expr1 | expr2));
named!(pub expr2 (&[u8]) -> Expr, alt!(do_expr2 | expr3));
named!(pub expr3 (&[u8]) -> Expr, alt!(do_expr3 | expr4));

// +
named!(do_expr1 (&[u8]) -> Expr, complete!(
    chain!(
        lhs: expr2 ~
        add_op ~
        rhs: expr1,
        || Expr::BinOp(Box::new(lhs), Operator::Add, Box::new(rhs))
    )
));

// -
named!(pub do_expr2 (&[u8]) -> Expr, complete!(
    chain!(
        mut expr: expr3 ~
        many1!(tap!(rhs: preceded!(sub_op, expr3) =>
            expr = Expr::BinOp(
                Box::new(expr),
                Operator::Sub,
                Box::new(rhs.clone())
            )
        )),
        || expr
    )
));

// * /
named!(do_expr3 (&[u8]) -> Expr, complete!(
    chain!(
        mut expr: expr4 ~
        many1!(alt!(
            tap!(rhs: preceded!(mul_op, expr4) => expr =
                Expr::BinOp(
                    Box::new(expr),
                    Operator::Mul,
                    Box::new(rhs.clone())
                )
            ) |
            tap!(rhs: preceded!(div_op, expr4) => expr =
                Expr::BinOp(
                    Box::new(expr),
                    Operator::Div,
                    Box::new(rhs.clone())
                )
            )
        )) ,
        || expr
    )
));


named!(expr4 (&[u8]) -> Expr, alt_complete!(
    delimited!(paren_b, expr1, paren_e) |
    const_expr |
    ident_expr
));

named!(const_expr (&[u8]) -> Expr, map!(number, Expr::Const));
named!(ident_expr (&[u8]) -> Expr, map!(identifier, Expr::Ident));

named!(identifier (&[u8]) -> String,
    chain!(
        multispace? ~
        ident: map_res!(
            map_res!(alpha, std::str::from_utf8),
            std::str::FromStr::from_str) ,
        || ident
    )
);

named!(number( &[u8] ) -> i32,
    chain!(
        multispace? ~
        int: map_res!(
            map_res!(
                digit,
                std::str::from_utf8),
            std::str::FromStr::from_str) ,
        || int
    )
);

macro_rules! symbol (
    ($name:ident, $s: expr) =>
    (named!($name (&[u8]) -> char,
    complete!(chain!(multispace? ~ s: char!($s), || s)));)
);

symbol!(semi, ';');
symbol!(ass_op, '=');
symbol!(add_op, '+');
symbol!(sub_op, '-');
symbol!(mul_op, '*');
symbol!(div_op, '/');
symbol!(paren_b, '(');
symbol!(paren_e, ')');
