use std::collections::HashMap;
use std::ops::Deref;

use ast::*;
use builtins::*;
use static_analysis::type_context::TypeContext;
use static_analysis::type_error::TypeError;

type TypeResult<T> = Result<T, TypeError>;

pub fn run(p: &Program) -> Result<(), TypeError> {
    let ctx: TypeContext = TypeContext::new();
    let res = ctx.in_new_scope(|mut ctx| p.check_types(&mut ctx));
    res.map(|_| ())
}

trait HasType<Ret, Context> {
    fn check_types(&self, ctx: Context) -> TypeResult<Ret> {
        self.do_check_types(ctx)
    }

    fn do_check_types(&self, ctx: Context) -> TypeResult<Ret>;
}

impl<T: ?Sized, E, Context> HasType<E, Context> for Box<T>
    where T: HasType<E, Context>
{
    fn do_check_types(&self, ctx: Context) -> TypeResult<E> {
        self.deref().check_types(ctx)
    }
}

impl<'a> HasType<(), &'a mut TypeContext> for Program {
    fn do_check_types(&self, mut ctx: &mut TypeContext) -> TypeResult<()> {
        for builtin in get_builtin_functions() {
            add_ident(&builtin.ident, &builtin.get_type(), &mut ctx)?;
        }

        let (classes, functions) = divide_definitions(&self.0);
        for c in &classes {
            add_class(c, &mut ctx)?;
        }
        for c in &classes {
            c.check_fields(ctx)?;
        }

        for f in &functions {
            f.check_signature(ctx)?;
            add_ident(&f.ident, &f.get_type(), &mut ctx)?;
        }

        for def in &self.0 {
            def.check_types(ctx)?;
        }
        check_main_function(ctx)?;
        Ok(())
    }
}

fn divide_definitions<'a>(defs: &'a Vec<Def>) -> (Vec<&'a Class>, Vec<&'a Func>) {
    let mut classes: Vec<&'a Class> = Vec::new();
    let mut functions: Vec<&'a Func> = Vec::new();
    for def in defs {
        match *def {
            Def::DClass(ref c) => classes.push(c),
            Def::DFunc(ref f) => functions.push(f),
        }
    }
    (classes, functions)
}

fn add_class(c: &Class, ctx: &mut TypeContext) -> TypeResult<()> {
    if ctx.get_type(&c.name).is_some() {
        return Err(TypeError::name_already_defined(&c.name));
    }

    let mut fields: HashMap<Ident, Type> = HashMap::new();
    for v in &c.vars {
        if fields.contains_key(&v.ident) {
            return Err(TypeError::field_already_defined(&c.name, &v.ident));
        }
        fields.insert(v.ident.clone(), v.get_type());
    }
    for f in &c.methods {
        if fields.contains_key(&f.ident) {
            return Err(TypeError::field_already_defined(&c.name, &f.ident));
        }
        fields.insert(f.ident.clone(), f.get_type());
    }
    ctx.add_class(&c.name, &c.superclass, fields);
    Ok(())
}

fn check_main_function(ctx: &TypeContext) -> TypeResult<()> {
    if let Some(func_type) = ctx.get_type(&Ident(format!("main"))) {
        if *func_type != Type::TFunc(vec![], Box::new(Type::TInt)) {
            Err(TypeError::invalid_main_type())
        } else {
            Ok(())
        }
    } else {
        Err(TypeError::no_main())
    }
}

impl<'a> HasType<(), &'a TypeContext> for Def {
    fn do_check_types(&self, ctx: &TypeContext) -> TypeResult<()> {
        match *self {
            Def::DClass(ref c) => c.check_types(ctx),
            Def::DFunc(ref f) => f.check_types(ctx),
        }
    }
}

impl Class {
    fn check_fields(&self, ctx: &TypeContext) -> TypeResult<()> {
        self.do_check_fields(ctx).map_err(|e| e.wrapped(&format!("class {}\n", self.name)))
    }

    fn do_check_fields(&self, ctx: &TypeContext) -> TypeResult<()> {
        ctx.in_new_scope(|mut ctx| {
                for v in &self.vars {
                    v.check_types(&mut ctx)?;
                }
                Ok(())
            })?;
        for f in &self.methods {
            f.check_signature(ctx)?;
        }

        if let Some(ref superclass) = self.superclass {
            for v in &self.vars {
                if ctx.get_field_type(superclass, &v.ident).is_some() {
                    return Err(TypeError::var_override(&v.ident));
                }
            }

            for f in &self.methods {
                // TODO: covariance and contravariance
                if let Some(actual) = ctx.get_field_type(superclass, &f.ident) {
                    let expected = f.get_type();
                    if actual != &expected {
                        return Err(TypeError::invalid_override(&f.ident, &expected, &actual));
                    }
                }
            }
        }
        Ok(())
    }
}

impl<'a> HasType<(), &'a TypeContext> for Class {
    fn check_types(&self, ctx: &TypeContext) -> TypeResult<()> {
        self.do_check_types(ctx).map_err(|e| e.wrapped(&format!("class {}", self.name)))
    }

    fn do_check_types(&self, ctx: &TypeContext) -> TypeResult<()> {
        ctx.in_class_scope(&self.name, true, |ctx| {
            for f in &self.methods {
                ctx.in_function_scope(&f.ret_type, |mut ctx| f.check_types(&mut ctx))?;
            }
            Ok(())
        })
    }
}

impl Func {
    fn check_signature(&self, ctx: &TypeContext) -> TypeResult<()> {
        self.do_check_signature(ctx)
            .map_err(|e| e.wrapped(&format!("function signature {}\n", self.ident)))
    }

    fn do_check_signature(&self, ctx: &TypeContext) -> TypeResult<()> {
        expect_valid_type(&self.ret_type, ctx)?;
        ctx.in_new_scope(|mut ctx| {
                for arg in &self.args {
                    arg.check_types(&mut ctx)?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

impl<'a> HasType<(), &'a TypeContext> for Func {
    fn check_types(&self, ctx: &TypeContext) -> TypeResult<()> {
        self.do_check_types(ctx).map_err(|e| e.wrapped(&format!("function {}\n", self.ident)))
    }

    fn do_check_types(&self, ctx: &TypeContext) -> TypeResult<()> {
        ctx.in_function_scope(&self.ret_type, |mut ctx| {
            for arg in &self.args {
                arg.check_types(&mut ctx)?;
            }
            self.body.check_types(&mut ctx)?;
            Ok(())
        })
    }
}

impl<'a> HasType<(), &'a mut TypeContext> for Var {
    fn check_types(&self, mut ctx: &mut TypeContext) -> TypeResult<()> {
        self.do_check_types(&mut ctx)
            .map_err(|e| e.wrapped(&format!("{} {}\n", self.t, self.ident)))
    }

    fn do_check_types(&self, mut ctx: &mut TypeContext) -> TypeResult<()> {
        expect_declarable_type(&self.t, &ctx)?;
        add_ident(&self.ident, &self.t, &mut ctx)?;
        Ok(())
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
            Stmt::SEmpty => {}
            Stmt::SBlock(ref stmts) => {
                ctx.in_new_scope(|mut ctx| stmts.check_types(&mut ctx))?;
            }
            Stmt::SDecl(ref t, ref decls) => {
                expect_declarable_type(t, ctx)?;
                for decl in decls {
                    decl.check_types(ctx)?;
                }
            }
            Stmt::SAssign(ref ident, ref expr) => {
                let itype = ident.check_types(ctx)?;
                let etype = expr.check_types(ctx)?;
                expect_type(&itype, &etype, ctx)?;
            }
            Stmt::SInc(ref ident) |
            Stmt::SDec(ref ident) => {
                let itype = ident.check_types(ctx)?;
                expect_type(&Type::TInt, &itype, ctx)?;
            }
            Stmt::SReturnE(ref expr) => {
                let etype = expr.check_types(ctx)?;
                expect_type(ctx.get_ret_type(), &etype, ctx)?;
            }
            Stmt::SReturn => {
                expect_type(ctx.get_ret_type(), &Type::TVoid, ctx)?;
            }
            Stmt::SExpr(ref expr) => {
                expr.check_types(ctx)?;
            }
            Stmt::SIf(ref expr, ref stmts) |
            Stmt::SWhile(ref expr, ref stmts) => {
                let etype = expr.check_types(ctx)?;
                expect_type(&Type::TBool, &etype, ctx)?;
                ctx.in_new_scope(|mut ctx| stmts.check_types(&mut ctx))?;
            }
            Stmt::SIfElse(ref expr, ref if_t, ref if_f) => {
                let etype = expr.check_types(ctx)?;
                expect_type(&Type::TBool, &etype, ctx)?;
                ctx.in_new_scope(|mut ctx| if_t.check_types(&mut ctx))?;
                ctx.in_new_scope(|mut ctx| if_f.check_types(&mut ctx))?;
            }
        };
        Ok(())
    }
}

impl<'a> HasType<(), &'a mut TypeContext> for VarDecl {
    fn check_types(&self, ctx: &mut TypeContext) -> TypeResult<()> {
        self.do_check_types(ctx).map_err(|e| e.wrapped(&format!("{}\n", self)))
    }

    fn do_check_types(&self, ctx: &mut TypeContext) -> TypeResult<()> {
        match *self {
            VarDecl::Init(ref t, ref ident, ref expr) => {
                let etype = expr.check_types(ctx)?;
                expect_type(t, &etype, ctx)?;
                add_ident(ident, t, ctx)?;
            }
            VarDecl::NoInit(ref t, ref ident) => {
                add_ident(ident, t, ctx)?;
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
            Expr::EVar(ref ident) => ident.check_types(ctx),
            Expr::ELit(ref l) => l.check_types(ctx),
            Expr::ECall(ref f, ref args) => check_call_types(f, args, ctx),
            Expr::ENeg(ref e) => expect_type(&Type::TInt, &e.check_types(ctx)?, ctx),
            Expr::ENot(ref e) => expect_type(&Type::TBool, &e.check_types(ctx)?, ctx),
            Expr::EBinOp(ref lhs, ref op, ref rhs) => {
                let lhs_t = lhs.check_types(ctx)?;
                let rhs_t = rhs.check_types(ctx)?;
                match *op {
                    Operator::OpAdd => check_add_types(lhs_t, rhs_t),
                    Operator::OpSub | Operator::OpMul | Operator::OpDiv | Operator::OpMod => {
                        expect_type(&Type::TInt, &lhs_t, ctx)?;
                        expect_type(&Type::TInt, &rhs_t, ctx)?;
                        Ok(Type::TInt)
                    }
                    Operator::OpLess | Operator::OpLessE | Operator::OpGreater |
                    Operator::OpGreaterE => {
                        expect_type(&Type::TInt, &lhs_t, ctx)?;
                        expect_type(&Type::TInt, &rhs_t, ctx)?;
                        Ok(Type::TBool)
                    }
                    Operator::OpEq | Operator::OpNEq => {
                        // TODO something better?
                        expect_type(&Type::TInt, &lhs_t, ctx)?;
                        expect_type(&Type::TInt, &rhs_t, ctx)?;
                        Ok(Type::TBool)
                    }
                    Operator::OpOr | Operator::OpAnd => {
                        expect_type(&Type::TBool, &lhs_t, ctx)?;
                        expect_type(&Type::TBool, &rhs_t, ctx)?;
                        Ok(Type::TBool)
                    }
                }
            }
        }
    }
}

fn check_call_types(ident: &FieldGet, args: &Vec<Expr>, ctx: &TypeContext) -> TypeResult<Type> {
    if let Type::TFunc(ref arg_types, ref ret_type) = ident.check_types(ctx)? {
        if args.len() != arg_types.len() {
            return Err(TypeError::invalid_call_arg_num(arg_types.len(), args.len()));
        }
        for (index, (expected, expr)) in arg_types.iter().zip(args).enumerate() {
            let actual = expr.check_types(ctx)?;
            expect_type(expected, &actual, ctx).map_err(|_| {
                TypeError::invalid_call_arg_type(index, expected, actual)
            })?;
        }
        Ok(ret_type.deref().clone())
    } else {
        Err(TypeError::not_a_function(ident))
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
            Lit::LNull => Type::TNull,
        })
    }
}

impl<'a> HasType<Type, &'a TypeContext> for FieldGet {
    fn check_types(&self, ctx: &TypeContext) -> TypeResult<Type> {
        self.do_check_types(ctx).map_err(|e| e.wrapped(&format!("{}\n", self)))
    }

    fn do_check_types(&self, ctx: &TypeContext) -> TypeResult<Type> {
        match self.field {
            None => self.ident.check_types(ctx),
            Some(ref field) => {
                if let Type::TObject(ref cname) = self.ident.check_types(ctx)? {
                    ctx.in_class_scope(cname, false, |ctx| field.check_types(&ctx))
                } else {
                    Err(TypeError::not_an_object(&self.ident))
                }
            }
        }
    }
}

impl<'a> HasType<Type, &'a TypeContext> for Ident {
    fn do_check_types(&self, ctx: &TypeContext) -> TypeResult<Type> {
        match ctx.get_type(self) {
            Some(t) => Ok(t.clone()),
            None => Err(TypeError::undefined(self)),
        }
    }
}

fn add_ident(ident: &Ident, t: &Type, ctx: &mut TypeContext) -> TypeResult<()> {
    expect_valid_type(t, &ctx)?;
    if ctx.is_local(ident) {
        Err(TypeError::already_defined(ident))
    } else {
        ctx.set_type(ident, t);
        Ok(())
    }
}

fn expect_type(expected: &Type, actual: &Type, ctx: &TypeContext) -> TypeResult<Type> {
    match expected == actual || conforms_lsp(expected, actual, ctx) {
        true => Ok(expected.clone()),
        false => Err(TypeError::invalid_type(expected, actual)),
    }
}

fn expect_declarable_type(t: &Type, ctx: &TypeContext) -> TypeResult<()> {
    expect_valid_type(t, ctx)?;
    match *t {
        Type::TInt |
        Type::TString |
        Type::TBool |
        Type::TObject(..) => Ok(()),
        _ => Err(TypeError::non_declarable(t)),
    }
}

fn expect_valid_type(t: &Type, ctx: &TypeContext) -> TypeResult<()> {
    match *t {
        Type::TObject(ref cname) => {
            match ctx.class_exists(cname) {
                true => Ok(()),
                false => Err(TypeError::inexistent_type(t)),
            }
        }
        _ => Ok(()),
    }
}

fn conforms_lsp(expected: &Type, actual: &Type, ctx: &TypeContext) -> bool {
    match (expected, actual) {
        (&Type::TObject(ref sup), &Type::TObject(ref sub)) => ctx.is_subclass_of(sub, sup),
        (&Type::TObject(..), &Type::TNull) => true,
        (_, _) => false,
    }
}
