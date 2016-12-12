use std::collections::HashMap;
use std::ops::Deref;
use std::fmt;

use ast::*;

pub fn check_types(p: &Program) -> Result<(), TypeError> {
    p.check_types(&TypeContext::new()).map(|_| ())
}

#[derive(Debug)]
pub struct TypeError {
    err: String,
    stack: Vec<String>,
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "{}", self.err));
        for place in &self.stack {
            try!(writeln!(f, "in:"));
            try!(write!(f, "{}", place));
        }
        Ok(())
    }
}

impl TypeError {
    fn wrapped<T: fmt::Display>(mut self, inside: &T) -> TypeError {
        self.stack.push(format!("{}", inside));
        self
    }

    fn undefined(ident: &Ident) -> TypeError {
        Self::new(format!("Undefined identifier: {}", ident))
    }

    fn already_defined(ident: &Ident) -> TypeError {
        Self::new(format!("Identifier {} is already defined in the current scope",
                          ident))
    }

    fn not_a_function(ident: &Ident) -> TypeError {
        Self::new(format!("{} is not a function", ident))
    }

    fn invalid_type(expected: Type, actual: Type) -> TypeError {
        Self::new(format!("Incorrect type, expected: {}, actual: {}", expected, actual))
    }

    fn no_operator(op: Operator, lhs_t: Type, rhs_t: Type) -> TypeError {
        Self::new(format!("No {} operator for types: {} and {}", op, lhs_t, rhs_t))
    }

    fn invalid_call_arg_num(expected: usize, actual: usize) -> TypeError {
        Self::new(format!("Function expected {} arguments, but got {}",
                          expected,
                          actual))
    }

    fn invalid_call_arg_type(nth: usize, expected: &Type, actual: Type) -> TypeError {
        Self::new(format!("Incorrect type for {}th argument, expected: {}, actual: {}",
                          nth + 1,
                          expected,
                          actual))
    }

    fn new(err: String) -> TypeError {
        TypeError {
            err: err,
            stack: Vec::new(),
        }
    }
}

type TypeResult = Result<Type, TypeError>;

#[derive(Debug)]
struct TypeContext {
    // identifier -> (type, current_scope)
    idents: HashMap<Ident, (Type, bool)>,
    ret_type: Type,
}

impl TypeContext {
    fn new() -> TypeContext {
        TypeContext {
            idents: HashMap::new(),
            ret_type: Type::TVoid,
        }
    }

    fn new_function_scope(&self, ret_type: &Type) -> TypeContext {
        let mut ctx = self.new_scope();
        ctx.ret_type = ret_type.clone();
        ctx
    }

    fn new_scope(&self) -> TypeContext {
        let mut idents = self.idents.clone();
        idents.iter_mut().map(|(_, ref mut e)| e.1 = false).collect::<Vec<()>>();
        let ret_type = self.ret_type.clone();
        TypeContext {
            idents: idents,
            ret_type: ret_type,
        }
    }

    fn get(&self, ident: &Ident) -> Option<Type> {
        self.idents.get(ident).map(|e| e.0.clone())
    }

    fn is_local(&self, ident: &Ident) -> bool {
        self.idents.get(ident).map(|e| e.1).unwrap_or(false)
    }

    fn set(&mut self, ident: &Ident, t: &Type) {
        self.idents.insert(ident.clone(), (t.clone(), true));
    }
}

trait HasType<Context> {
    fn get_type(&self, ctx: Context) -> Type {
        self.check_types(ctx).unwrap()
    }

    fn check_types(&self, ctx: Context) -> TypeResult {
        self.do_check_types(ctx)
    }

    fn do_check_types(&self, ctx: Context) -> TypeResult;
}

impl<T: ?Sized, Context> HasType<Context> for Box<T>
    where T: HasType<Context>
{
    fn do_check_types(&self, ctx: Context) -> TypeResult {
        self.deref().check_types(ctx)
    }
}

trait IntroducesVar<Context> {
    fn introduce_var(&self, t: &Type, ctx: Context) -> TypeResult {
        self.do_introduce_var(t, ctx)
    }

    fn do_introduce_var(&self, t: &Type, ctx: Context) -> TypeResult;
}

impl<'a> HasType<&'a TypeContext> for Program {
    fn do_check_types(&self, ctx: &TypeContext) -> TypeResult {
        let mut new_ctx = ctx.new_scope();
        let Program(ref defs) = *self;

        for def in defs {
            let name = match *def {
                Def::DFunc(ref ident, _, _, _) => ident,
            };
            let t = def.get_type(&ctx);
            new_ctx.set(name, &t);
        }

        for def in defs {
            try!(def.check_types(&new_ctx));
        }
        Ok(Type::TVoid)
    }
}

impl<'a> HasType<&'a TypeContext> for Def {
    fn get_type(&self, _: &TypeContext) -> Type {
        match *self {
            Def::DFunc(_, ref args, ref ret_type, _) => {
                Type::TFunc(args.iter().map(|a| a.0.clone()).collect(),
                            Box::new(ret_type.clone()))
            }
        }
    }

    fn check_types(&self, ctx: &TypeContext) -> TypeResult {
        self.do_check_types(ctx).map_err(|e| e.wrapped(self))
    }

    fn do_check_types(&self, ctx: &TypeContext) -> TypeResult {
        match *self {
            Def::DFunc(_, ref args, ref ret_type, ref stmts) => {
                let mut new_ctx = ctx.new_function_scope(ret_type);
                for arg in args {
                    try!(introduce_name(&arg.1, &arg.0, &mut new_ctx));
                }
                try!(stmts.check_types(&mut new_ctx));
                Ok(ret_type.clone())
            }
        }
    }
}

impl<'a> HasType<&'a mut TypeContext> for Vec<Stmt> {
    fn do_check_types(&self, ctx: &mut TypeContext) -> TypeResult {
        for x in self {
            try!(x.check_types(ctx));
        }
        Ok(Type::TVoid)
    }
}

impl<'a> HasType<&'a mut TypeContext> for Stmt {
    fn check_types(&self, ctx: &mut TypeContext) -> TypeResult {
        self.do_check_types(ctx).map_err(|e| e.wrapped(self))
    }

    fn do_check_types(&self, ctx: &mut TypeContext) -> TypeResult {
        match *self {
            Stmt::SBlock(ref stmts) => {
                let mut new_ctx = ctx.new_scope();
                try!(stmts.check_types(&mut new_ctx));
            }
            Stmt::SDecl(ref t, ref decls) => {
                for decl in decls {
                    try!(decl.introduce_var(t, ctx));
                }
            }
            Stmt::SAssign(ref ident, ref expr) => {
                let itype = try!(get_type(ident, ctx));
                let etype = try!(expr.check_types(ctx));
                try!(expect_type(itype, etype));
            }
            Stmt::SInc(ref ident) |
            Stmt::SDec(ref ident) => {
                let itype = try!(get_type(ident, ctx));
                try!(expect_type(Type::TInt, itype));
            }
            Stmt::SReturnE(ref expr) => {
                let etype = try!(expr.check_types(ctx));
                try!(expect_type(ctx.ret_type.clone(), etype));
            }
            Stmt::SReturn => {
                try!(expect_type(ctx.ret_type.clone(), Type::TVoid));
            }
            Stmt::SExpr(ref expr) => {
                try!(expr.check_types(ctx));
            }
            Stmt::SIf(ref expr, ref stmts) |
            Stmt::SWhile(ref expr, ref stmts) => {
                let etype = try!(expr.check_types(ctx));
                try!(expect_type(Type::TBool, etype));
                let mut new_ctx = ctx.new_scope();
                try!(stmts.check_types(&mut new_ctx));
            }
            Stmt::SIfElse(ref expr, ref if_t, ref if_f) => {
                let etype = try!(expr.check_types(ctx));
                try!(expect_type(Type::TBool, etype));
                let mut new_ctx = ctx.new_scope();
                try!(if_t.check_types(&mut new_ctx));
                new_ctx = ctx.new_scope();
                try!(if_f.check_types(&mut new_ctx));
            }
        };
        Ok(Type::TVoid)
    }
}

impl<'a> IntroducesVar<&'a mut TypeContext> for VarDecl {
    fn introduce_var(&self, t: &Type, ctx: &mut TypeContext) -> TypeResult {
        self.do_introduce_var(t, ctx).map_err(|e| e.wrapped(&format!("{}\n", self)))
    }

    fn do_introduce_var(&self, t: &Type, ctx: &mut TypeContext) -> TypeResult {
        match *self {
            VarDecl::Init(ref ident, ref expr) => {
                let etype = try!(expr.check_types(ctx));
                try!(expect_type(t.clone(), etype));
                try!(introduce_name(ident, t, ctx));
            }
            VarDecl::NoInit(ref ident) => {
                try!(introduce_name(ident, t, ctx));
            }
        };
        Ok(Type::TVoid)
    }
}

impl<'a> HasType<&'a TypeContext> for Expr {
    fn check_types(&self, ctx: &TypeContext) -> TypeResult {
        self.do_check_types(ctx).map_err(|e| TypeError::wrapped(e, &format!("{}\n", self)))
    }

    fn do_check_types(&self, ctx: &TypeContext) -> TypeResult {
        match *self {
            Expr::EVar(ref ident) => get_type(ident, ctx),
            Expr::ELit(ref l) => l.check_types(ctx),
            Expr::ECall(ref f, ref args) => check_call_types(f, args, ctx),
            Expr::ENeg(ref e) => expect_type(Type::TInt, try!(e.check_types(ctx))),
            Expr::ENot(ref e) => expect_type(Type::TBool, try!(e.check_types(ctx))),
            Expr::EBinOp(ref lhs, ref op, ref rhs) => {
                let lhs_t = try!(lhs.check_types(ctx));
                let rhs_t = try!(rhs.check_types(ctx));
                match *op {
                    Operator::OpAdd => check_add_types(lhs_t, rhs_t),
                    Operator::OpSub | Operator::OpMul | Operator::OpDiv => {
                        try!(expect_type(Type::TInt, lhs_t));
                        try!(expect_type(Type::TInt, rhs_t));
                        Ok(Type::TInt)
                    }
                    Operator::OpLess | Operator::OpLessE | Operator::OpGreater |
                    Operator::OpGreaterE => {
                        try!(expect_type(Type::TInt, lhs_t));
                        try!(expect_type(Type::TInt, rhs_t));
                        Ok(Type::TBool)
                    }
                    Operator::OpEq | Operator::OpNEq => {
                        // TODO something better?
                        try!(expect_type(Type::TInt, lhs_t));
                        try!(expect_type(Type::TInt, rhs_t));
                        Ok(Type::TBool)
                    }
                    Operator::OpOr | Operator::OpAnd => {
                        try!(expect_type(Type::TBool, lhs_t));
                        try!(expect_type(Type::TBool, rhs_t));
                        Ok(Type::TBool)
                    }
                }
            }
        }
    }
}

fn check_call_types(fname: &Ident, args: &Vec<Expr>, ctx: &TypeContext) -> TypeResult {
    if let Type::TFunc(args_types, ret_type) = try!(get_type(fname, ctx)) {
        if args.len() != args_types.len() {
            return Err(TypeError::invalid_call_arg_num(args_types.len(), args.len()));
        }
        for (index, (expected_type, expr)) in args_types.iter().zip(args).enumerate() {
            let actual_type = try!(expr.check_types(ctx));
            if actual_type != *expected_type {
                return Err(TypeError::invalid_call_arg_type(index, expected_type, actual_type));
            }
        }
        Ok(*ret_type)
    } else {
        Err(TypeError::not_a_function(fname))
    }
}

fn check_add_types(lhs_t: Type, rhs_t: Type) -> TypeResult {
    if (lhs_t != Type::TInt && lhs_t != Type::TString) || lhs_t != rhs_t {
        Err(TypeError::no_operator(Operator::OpAdd, lhs_t, rhs_t))
    } else {
        Ok(lhs_t)
    }
}

impl<'a> HasType<&'a TypeContext> for Lit {
    fn do_check_types(&self, _: &TypeContext) -> TypeResult {
        Ok(match *self {
            Lit::LInt(_) => Type::TInt,
            Lit::LTrue | Lit::LFalse => Type::TBool,
            Lit::LString(_) => Type::TString,
        })
    }
}

fn expect_type(expected: Type, actual: Type) -> TypeResult {
    if expected == actual {
        Ok(expected)
    } else {
        Err(TypeError::invalid_type(expected, actual))
    }
}

fn get_type(name: &Ident, ctx: &TypeContext) -> TypeResult {
    match ctx.get(name) {
        Some(t) => Ok(t),
        None => Err(TypeError::undefined(name)),
    }
}

fn introduce_name(name: &Ident, t: &Type, ctx: &mut TypeContext) -> TypeResult {
    if ctx.is_local(name) {
        Err(TypeError::already_defined(name))
    } else {
        ctx.set(name, t);
        Ok(Type::TVoid)
    }
}
