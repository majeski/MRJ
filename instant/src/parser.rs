use std::ops::Index;
use std::ops::Range;
use std::ops::RangeFrom;
use std;
use nom::*;

use ast::*;

pub fn parse(raw: String) -> Result<Program, String> {
    match program(raw.as_bytes()) {
        IResult::Done(rest, ref t) if rest.is_empty() => Ok(t.clone()),
        IResult::Error(err) => Err(format!("{:?}", err)),
        _ => panic!("impossible happened"),
    }
}

named!(program (&[u8]) -> Program, complete!(
    chain!(
        multispace? ~
        stmts: separated_list!(semi, statement) ~
        eof,
        || Program(stmts)
    )
));

named!(statement (&[u8]) -> Stmt, alt!(assignment | stmt_expr));
named!(assignment (&[u8]) -> Stmt, complete!(
    chain!(
        ident: identifier ~
        ass_op ~
        expr: expr1,
        || Stmt::Assign(ident, expr)
    )
));

named!(stmt_expr (&[u8]) -> Stmt, map!(expr1, Stmt::Expr));

named!(expr1 (&[u8]) -> Expr, alt!(do_expr1 | expr2));
named!(expr2 (&[u8]) -> Expr, alt!(do_expr2 | expr3));
named!(expr3 (&[u8]) -> Expr, alt!(do_expr3 | expr4));

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
named!(do_expr2 (&[u8]) -> Expr, complete!(
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
        )),
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
        ident: map_res!(
            map_res!(
                _identifier,
                std::str::from_utf8),
            std::str::FromStr::from_str) ~
        multispace?,
        || ident
    )
);


fn _identifier<'a, T: ?Sized>(input: &'a T) -> IResult<&'a T, &'a T>
    where T: Index<Range<usize>, Output = T> + Index<RangeFrom<usize>, Output = T>,
          &'a T: IterIndices + InputLength
{
    let input_length = input.input_len();
    if input_length == 0 {
        return IResult::Error(Err::Position(ErrorKind::Custom(6), input));
    }

    for (idx, item) in input.iter_indices() {
        let c = item.as_char() as u8;
        if !(is_alphanumeric(c) || c == '_' as u8) || (is_digit(c) && idx == 0) {
            if idx == 0 {
                return IResult::Error(Err::Position(ErrorKind::Custom(6), input));
            } else {
                return IResult::Done(&input[idx..], &input[0..idx]);
            }
        }
    }
    IResult::Done(&input[input_length..], input)
}

named!(number (&[u8]) -> i32,
    chain!(
        int: map_res!(
            map_res!(
                digit,
                std::str::from_utf8),
            std::str::FromStr::from_str) ~
        multispace?,
        || int
    )
);

macro_rules! symbol (
    ($name:ident, $s: expr) =>
    (named!($name (&[u8]) -> char,
    complete!(chain!(s: char!($s) ~ multispace?, || s)));)
);

symbol!(semi, ';');
symbol!(ass_op, '=');
symbol!(add_op, '+');
symbol!(sub_op, '-');
symbol!(mul_op, '*');
symbol!(div_op, '/');
symbol!(paren_b, '(');
symbol!(paren_e, ')');
