use ast::*;

use optimization::optimize::*;

impl Optimize for Expr {
    fn optimize(self) -> Expr {
        match self {
            Expr::ENeg(e) => {
                let e = e.optimize();
                match is_int_lit(&*e) {
                    true => Expr::ELit(Lit::LInt(-to_int(*e))),
                    false => Expr::ENeg(e),
                }
            }
            Expr::ENot(e) => {
                let e = e.optimize();
                match is_bool_lit(&*e) {
                    true => Expr::ELit(to_lit(!to_bool(*e))),
                    false => Expr::ENot(e),
                }
            }
            Expr::EBinOp(lhs, op, rhs) => {
                let lhs = *lhs.optimize();
                let rhs = *rhs.optimize();
                if is_int_lit(&lhs) && is_int_lit(&rhs) && is_safe_op(op, &rhs) {
                    let (l, r) = (to_int(lhs), to_int(rhs));
                    Expr::ELit(match op {
                        Operator::OpAdd => Lit::LInt(l + r),
                        Operator::OpSub => Lit::LInt(l - r),
                        Operator::OpMul => Lit::LInt(l * r),
                        Operator::OpDiv => Lit::LInt(l / r),
                        Operator::OpMod => Lit::LInt(l % r),
                        Operator::OpLess => to_lit(l < r),
                        Operator::OpLessE => to_lit(l <= r),
                        Operator::OpGreater => to_lit(l > r),
                        Operator::OpGreaterE => to_lit(l >= r),
                        Operator::OpEq => to_lit(l == r),
                        Operator::OpNEq => to_lit(l != r),
                        _ => unreachable!(),
                    })
                } else if is_bool_lit(&lhs) && is_bool_lit(&rhs) {
                    let (l, r) = (to_bool(lhs), to_bool(rhs));
                    Expr::ELit(to_lit(match op {
                        Operator::OpEq => l == r,
                        Operator::OpNEq => l != r,
                        Operator::OpAnd => l && r,
                        Operator::OpOr => l || r,
                        _ => unreachable!(),
                    }))
                } else if is_bool_lit(&lhs) {
                    let l = to_bool(lhs);
                    match op {
                        Operator::OpEq => {
                            match l {
                                true => rhs,
                                false => Expr::ENot(Box::new(rhs)),
                            }
                        },
                        Operator::OpNEq => {
                            match l {
                                true => Expr::ENot(Box::new(rhs)),
                                false => rhs,
                            }
                        }
                        Operator::OpAnd => {
                            match l {
                                true => rhs,
                                false => Expr::ELit(Lit::LFalse),
                            }
                        }
                        Operator::OpOr => {
                            match l {
                                true => Expr::ELit(Lit::LTrue),
                                false => rhs,
                            }
                        }
                        _ => unreachable!(),
                    }
                } else if is_str_lit(&lhs) && is_str_lit(&rhs) && is_safe_str_op(op) {
                    let (l, r) = (to_str(lhs), to_str(rhs));
                    match op {
                        Operator::OpEq => Expr::ELit(to_lit(l == r)),
                        Operator::OpNEq => Expr::ELit(to_lit(l != r)),
                        _ => unreachable!(),
                    }
                } else {
                    Expr::EBinOp(Box::new(lhs), op, Box::new(rhs))
                }
            }
            Expr::ECall(ident, es) => {
                Expr::ECall(ident, es.into_iter().map(Expr::optimize).collect())
            }
            _ => self,
        }
    }
}

fn to_lit(b: bool) -> Lit {
    match b {
        true => Lit::LTrue,
        false => Lit::LFalse,
    }
}

fn is_bool_lit(e: &Expr) -> bool {
    match *e {
        Expr::ELit(Lit::LTrue) |
        Expr::ELit(Lit::LFalse) => true,
        _ => false,
    }
}

fn to_bool(e: Expr) -> bool {
    match e {
        Expr::ELit(Lit::LTrue) => true,
        Expr::ELit(Lit::LFalse) => false,
        _ => unreachable!(),
    }
}

fn is_int_lit(e: &Expr) -> bool {
    match *e {
        Expr::ELit(Lit::LInt(_)) => true,
        _ => false,
    }
}

fn to_int(e: Expr) -> i32 {
    match e {
        Expr::ELit(Lit::LInt(x)) => x,
        _ => unreachable!(),
    }
}

fn is_str_lit(e: &Expr) -> bool {
    match *e {
        Expr::ELit(Lit::LString(_)) => true,
        _ => false,
    }
}

fn to_str(e: Expr) -> String {
    match e {
        Expr::ELit(Lit::LString(s)) => s,
        _ => unreachable!(),
    }
}

fn is_safe_op(op: Operator, rhs: &Expr) -> bool {
    match *rhs {
        Expr::ELit(Lit::LInt(0)) => op != Operator::OpDiv && op != Operator::OpMod,
        _ => true,
    }
}

fn is_safe_str_op(op: Operator) -> bool {
    op == Operator::OpEq || op == Operator::OpNEq
}
