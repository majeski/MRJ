use std::ops::Deref;

use ast::*;
use static_analysis::type_context::TypeContext;
use static_analysis::type_error::TypeError;

type TypeResult<T> = Result<T, TypeError>;

pub fn run(p: &Program) -> Result<(), TypeError> {
    let ctx = TypeContext::new();
    let res = ctx.in_new_scope(|mut ctx| p.check_types(&mut ctx));
    res.map(|_| ())
}

trait HasType<T, Context> {
    fn check_types(&self, ctx: Context) -> TypeResult<T> {
        self.do_check_types(ctx)
    }

    fn do_check_types(&self, ctx: Context) -> TypeResult<T>;
}

trait IntroducesVar<Context> {
    fn introduce_var(&self, t: &Type, ctx: Context) -> TypeResult<()> {
        self.do_introduce_var(t, ctx)
    }

    fn do_introduce_var(&self, t: &Type, ctx: Context) -> TypeResult<()>;
}

impl<T: ?Sized, E, Context> HasType<E, Context> for Box<T>
    where T: HasType<E, Context>
{
    fn do_check_types(&self, ctx: Context) -> TypeResult<E> {
        self.deref().check_types(ctx)
    }
}

impl<'a> HasType<(), &'a mut TypeContext> for Program {
    fn do_check_types(&self, ctx: &mut TypeContext) -> TypeResult<()> {
        let Program(ref defs) = *self;

        for def in defs {
            ctx.set(def.get_ident(), &def.get_type());
        }
        check_main_function(ctx)?;

        for def in defs {
            def.check_types(&ctx)?;
        }
        Ok(())
    }
}

fn check_main_function(ctx: &TypeContext) -> TypeResult<()> {
    if let Some(x) = ctx.get(&Ident(format!("main"))) {
        if x != Type::TFunc(vec![], Box::new(Type::TInt)) {
            Err(TypeError::invalid_main_type())
        } else {
            Ok(())
        }
    } else {
        Err(TypeError::no_main())
    }
}

impl<'a> HasType<(), &'a TypeContext> for Def {
    fn check_types(&self, ctx: &TypeContext) -> TypeResult<()> {
        self.do_check_types(ctx).map_err(|e| e.wrapped(self))
    }

    fn do_check_types(&self, ctx: &TypeContext) -> TypeResult<()> {
        match *self {
            Def::DFunc(_, ref args, ref ret_type, ref stmts) => {
                ctx.in_function_scope(ret_type, |mut ctx| {
                    for arg in args {
                        introduce_name(&arg.1, &arg.0, &mut ctx)?;
                    }
                    stmts.check_types(&mut ctx)
                })
            }
        }
    }
}

impl<'a> HasType<(), &'a mut TypeContext> for Vec<Stmt> {
    fn do_check_types(&self, ctx: &mut TypeContext) -> TypeResult<()> {
        for x in self {
            x.check_types(ctx)?;
        }
        Ok(())
    }
}

impl<'a> HasType<(), &'a mut TypeContext> for Stmt {
    fn check_types(&self, ctx: &mut TypeContext) -> TypeResult<()> {
        self.do_check_types(ctx).map_err(|e| e.wrapped(self))
    }

    fn do_check_types(&self, ctx: &mut TypeContext) -> TypeResult<()> {
        match *self {
            Stmt::SBlock(ref stmts) => {
                ctx.in_new_scope(|mut ctx| stmts.check_types(&mut ctx))?;
            }
            Stmt::SDecl(ref t, ref decls) => {
                if *t == Type::TVoid {
                    return Err(TypeError::void_decl());
                }
                for decl in decls {
                    decl.introduce_var(t, ctx)?;
                }
            }
            Stmt::SAssign(ref ident, ref expr) => {
                let itype = get_type(ident, ctx)?;
                let etype = expr.check_types(ctx)?;
                expect_type(itype, etype)?;
            }
            Stmt::SInc(ref ident) |
            Stmt::SDec(ref ident) => {
                let itype = get_type(ident, ctx)?;
                expect_type(Type::TInt, itype)?;
            }
            Stmt::SReturnE(ref expr) => {
                let etype = expr.check_types(ctx)?;
                expect_type(ctx.get_ret_type(), etype)?;
            }
            Stmt::SReturn => {
                expect_type(ctx.get_ret_type(), Type::TVoid)?;
            }
            Stmt::SExpr(ref expr) => {
                expr.check_types(ctx)?;
            }
            Stmt::SIf(ref expr, ref stmts) |
            Stmt::SWhile(ref expr, ref stmts) => {
                let etype = expr.check_types(ctx)?;
                expect_type(Type::TBool, etype)?;
                ctx.in_new_scope(|mut ctx| stmts.check_types(&mut ctx))?;
            }
            Stmt::SIfElse(ref expr, ref if_t, ref if_f) => {
                let etype = expr.check_types(ctx)?;
                expect_type(Type::TBool, etype)?;
                ctx.in_new_scope(|mut ctx| if_t.check_types(&mut ctx))?;
                ctx.in_new_scope(|mut ctx| if_f.check_types(&mut ctx))?;
            }
        };
        Ok(())
    }
}

impl<'a> IntroducesVar<&'a mut TypeContext> for VarDecl {
    fn introduce_var(&self, t: &Type, ctx: &mut TypeContext) -> TypeResult<()> {
        self.do_introduce_var(t, ctx).map_err(|e| e.wrapped(&format!("{}\n", self)))
    }

    fn do_introduce_var(&self, t: &Type, ctx: &mut TypeContext) -> TypeResult<()> {
        match *self {
            VarDecl::Init(ref ident, ref expr) => {
                let etype = expr.check_types(ctx)?;
                expect_type(t.clone(), etype)?;
                introduce_name(ident, t, ctx)?;
            }
            VarDecl::NoInit(ref ident) => {
                introduce_name(ident, t, ctx)?;
            }
        };
        Ok(())
    }
}

impl<'a> HasType<Type, &'a TypeContext> for Expr {
    fn check_types(&self, ctx: &TypeContext) -> TypeResult<Type> {
        self.do_check_types(ctx).map_err(|e| e.wrapped(&format!("{}\n", self)))
    }

    fn do_check_types(&self, ctx: &TypeContext) -> TypeResult<Type> {
        match *self {
            Expr::EVar(ref ident) => get_type(ident, ctx),
            Expr::ELit(ref l) => l.check_types(ctx),
            Expr::ECall(ref f, ref args) => check_call_types(f, args, ctx),
            Expr::ENeg(ref e) => expect_type(Type::TInt, e.check_types(ctx)?),
            Expr::ENot(ref e) => expect_type(Type::TBool, e.check_types(ctx)?),
            Expr::EBinOp(ref lhs, ref op, ref rhs) => {
                let lhs_t = lhs.check_types(ctx)?;
                let rhs_t = rhs.check_types(ctx)?;
                match *op {
                    Operator::OpAdd => check_add_types(lhs_t, rhs_t),
                    Operator::OpSub | Operator::OpMul | Operator::OpDiv => {
                        expect_type(Type::TInt, lhs_t)?;
                        expect_type(Type::TInt, rhs_t)?;
                        Ok(Type::TInt)
                    }
                    Operator::OpLess | Operator::OpLessE | Operator::OpGreater |
                    Operator::OpGreaterE => {
                        expect_type(Type::TInt, lhs_t)?;
                        expect_type(Type::TInt, rhs_t)?;
                        Ok(Type::TBool)
                    }
                    Operator::OpEq | Operator::OpNEq => {
                        // TODO something better?
                        expect_type(Type::TInt, lhs_t)?;
                        expect_type(Type::TInt, rhs_t)?;
                        Ok(Type::TBool)
                    }
                    Operator::OpOr | Operator::OpAnd => {
                        expect_type(Type::TBool, lhs_t)?;
                        expect_type(Type::TBool, rhs_t)?;
                        Ok(Type::TBool)
                    }
                }
            }
        }
    }
}

fn check_call_types(fname: &Ident, args: &Vec<Expr>, ctx: &TypeContext) -> TypeResult<Type> {
    if let Type::TFunc(args_types, ret_type) = get_type(fname, ctx)? {
        if args.len() != args_types.len() {
            return Err(TypeError::invalid_call_arg_num(args_types.len(), args.len()));
        }
        for (index, (expected_type, expr)) in args_types.iter().zip(args).enumerate() {
            let actual_type = expr.check_types(ctx)?;
            if actual_type != *expected_type {
                return Err(TypeError::invalid_call_arg_type(index, expected_type, actual_type));
            }
        }
        Ok(*ret_type)
    } else {
        Err(TypeError::not_a_function(fname))
    }
}

fn check_add_types(lhs_t: Type, rhs_t: Type) -> TypeResult<Type> {
    if (lhs_t != Type::TInt && lhs_t != Type::TString) || lhs_t != rhs_t {
        Err(TypeError::no_operator(Operator::OpAdd, lhs_t, rhs_t))
    } else {
        Ok(lhs_t)
    }
}

impl<'a> HasType<Type, &'a TypeContext> for Lit {
    fn do_check_types(&self, _: &TypeContext) -> TypeResult<Type> {
        Ok(match *self {
            Lit::LInt(_) => Type::TInt,
            Lit::LTrue | Lit::LFalse => Type::TBool,
            Lit::LString(_) => Type::TString,
        })
    }
}

fn expect_type(expected: Type, actual: Type) -> TypeResult<Type> {
    if expected == actual {
        Ok(expected)
    } else {
        Err(TypeError::invalid_type(expected, actual))
    }
}

fn get_type(name: &Ident, ctx: &TypeContext) -> TypeResult<Type> {
    match ctx.get(name) {
        Some(t) => Ok(t),
        None => Err(TypeError::undefined(name)),
    }
}

fn introduce_name(name: &Ident, t: &Type, ctx: &mut TypeContext) -> TypeResult<()> {
    if ctx.is_local(name) {
        Err(TypeError::already_defined(name))
    } else {
        ctx.set(name, t);
        Ok(())
    }
}
